mod config;
mod conservation;
mod world;
mod agents;
mod render;

use crate::agents::entity::Agent;
use crate::agents::tick::{agent_tick, TickStats};
use crate::config::*;
use crate::conservation::{element_census, verify_conservation};
use crate::render::renderer::{render_dashboard, render_highlight};
use crate::render::snapshot::{Event, EventLog};
use crate::world::clocks::ClockSystem;
use crate::world::diffusion::diffuse_agents;
use crate::world::energy::{apply_geothermal, reset_energy, update_star_energy};
use crate::world::grid::Grid;
use crate::world::terrain::{init_terrain, pressure_from_depth, temperature_from_depth};

use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "genesis", about = "An emergent simulation of alien chemistry and life")]
struct Cli {
    /// Path to seed TOML file
    #[arg(long, short)]
    seed: PathBuf,

    /// Random seed for reproducibility
    #[arg(long, default_value = "42")]
    random_seed: u64,

    /// Number of simulation cycles to run
    #[arg(long, default_value = "50")]
    cycles: u64,

    /// Path to universe/ directory
    #[arg(long, default_value = "../universe")]
    universe: PathBuf,

    /// Output directory for archive artifacts
    #[arg(long, default_value = "../archive/runs/run-0001")]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    println!("Genesis — loading configuration...");

    // Load config
    let elements_config = load_elements(&cli.universe).expect("Failed to load elements");
    let env_config = load_environment(&cli.universe).expect("Failed to load environment");
    let star_config = load_star(&cli.universe).expect("Failed to load star");
    let seed_config = load_seed(&cli.seed).expect("Failed to load seed");

    println!("Seed: {} — {}", seed_config.name, seed_config.description);

    let mut rng = StdRng::seed_from_u64(cli.random_seed);

    // Initialize grid
    let mut grid = Grid::new(env_config.grid.width, env_config.grid.height);
    init_terrain(&mut grid, &seed_config, &mut rng);

    // Derive temperature and pressure from terrain
    for cell in &mut grid.cells {
        let depth = cell.depth();
        cell.temperature = temperature_from_depth(
            depth,
            env_config.ocean.surface_temperature,
            env_config.ocean.depth_temperature_gradient,
        );
        cell.pressure = pressure_from_depth(depth);
    }

    // Place geothermal vents
    let vent_positions: Vec<(usize, usize)> = (0..env_config.geothermal.vent_count)
        .map(|_| {
            // Place vents in the deeper (left) side of the grid, underwater
            let x = rng.gen_range(0..grid.width / 3);
            let y = rng.gen_range(0..grid.height);
            (x, y)
        })
        .collect();

    // Place initial agents
    let mut agents: HashMap<u64, Agent> = HashMap::new();
    let mut next_id: u64 = 1;

    let agent_counts = seed_config.initial_agents.as_map();
    for (elem_name, &count) in &agent_counts {
        // Look up element mass from config
        let mass = elements_config
            .elements
            .iter()
            .find(|e| e.name == *elem_name)
            .map(|e| e.mass)
            .unwrap_or(1.0);
        for _ in 0..count {
            let (x, y) = place_agent(&grid, &seed_config, &mut rng);
            let agent = Agent::new_element(next_id, elem_name.clone(), mass, x, y);
            grid.cell_mut(x, y).agent_ids.insert(next_id);
            agents.insert(next_id, agent);
            next_id += 1;
        }
    }

    println!(
        "Placed {} agents on {}x{} grid",
        agents.len(),
        grid.width,
        grid.height
    );

    // Establish conservation baseline
    let conservation_baseline = element_census(&agents);
    println!("Conservation baseline: {:?}", conservation_baseline);

    // Initialize simulation systems
    let mut clocks = ClockSystem::new(
        1000, // stellar: 1 per 1000 ticks
        100,  // world: 1 per 100 ticks
        1,    // agent: every tick
    );

    let mut event_log = EventLog::new();
    let mut stats = TickStats::new();

    let total_ticks = cli.cycles * 100; // 100 agent ticks per cycle
    let snapshot_interval = (total_ticks / 5).max(1); // ~5 snapshots per run, min 1

    println!("Running {} cycles ({} ticks)...", cli.cycles, total_ticks);

    // Main simulation loop
    for _ in 0..total_ticks {
        clocks.tick();

        // World tick: recompute energy, diffusion, temperature, pressure
        if clocks.is_world_tick() {
            // Reset temperatures and energy to baseline
            for cell in &mut grid.cells {
                let depth = cell.depth();
                cell.temperature = temperature_from_depth(
                    depth,
                    env_config.ocean.surface_temperature,
                    env_config.ocean.depth_temperature_gradient,
                );
                cell.pressure = pressure_from_depth(depth);
            }
            reset_energy(&mut grid);

            // Recompute energy: star first, then geothermal adds on top
            update_star_energy(&mut grid, &star_config, clocks.current_tick);
            apply_geothermal(
                &mut grid,
                &vent_positions,
                env_config.geothermal.vent_energy_output,
            );

            // Diffuse free agents
            diffuse_agents(&mut grid, &mut agents, &mut rng);

            // Update activity tagging
            grid.update_activity();

            // Conservation check — the sacred invariant
            if let Err(msg) = verify_conservation(&agents, &conservation_baseline) {
                panic!("CONSERVATION VIOLATION at tick {}: {}", clocks.current_tick, msg);
            }

            // Periodic population snapshot
            if clocks.current_tick % snapshot_interval == 0 {
                let bonded = agents.values().filter(|a| !a.bonds.is_empty()).count() as u64;
                let free = agents.len() as u64 - bonded;
                let total_bonds = agents
                    .values()
                    .map(|a| a.bonds.len() as u64)
                    .sum::<u64>()
                    / 2;

                let mut elem_counts: Vec<(String, u64)> = conservation_baseline
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
                elem_counts.sort_by(|a, b| a.0.cmp(&b.0));

                event_log.record(Event::PopulationSnapshot {
                    tick: clocks.current_tick,
                    free_agents: free,
                    bonded_agents: bonded,
                    total_bonds,
                    element_counts: elem_counts,
                });
            }
        }

        // Agent tick: bonding, bond breaking
        let events = agent_tick(
            &mut grid,
            &mut agents,
            &elements_config,
            clocks.current_tick,
            &mut rng,
            &mut stats,
        );

        for event in events {
            event_log.record(event);
        }

        // Check for bond milestones (handles jumps past threshold)
        let milestone_thresholds = [10, 50, 100, 500, 1000];
        for &threshold in &milestone_thresholds {
            if stats.check_milestone(threshold) {
                event_log.record(Event::BondCountMilestone {
                    tick: clocks.current_tick,
                    count: threshold,
                });
            }
        }
    }

    // Final conservation check
    let conservation_ok = verify_conservation(&agents, &conservation_baseline).is_ok();

    event_log.record(Event::SimulationEnd {
        tick: clocks.current_tick,
        total_bonds_formed: stats.bonds_formed,
        total_bonds_broken: stats.bonds_broken,
        conservation_ok,
    });

    println!("\nSimulation complete.");
    println!("  Bonds formed:  {}", stats.bonds_formed);
    println!("  Bonds broken:  {}", stats.bonds_broken);
    println!("  Conservation:  {}", if conservation_ok { "OK" } else { "FAILED" });

    // Write output artifacts
    let output_dir = &cli.output;
    std::fs::create_dir_all(output_dir).expect("Failed to create output directory");
    std::fs::create_dir_all(output_dir.join("highlights")).expect("Failed to create highlights directory");

    // Event log
    let events_json = event_log.to_json();
    std::fs::write(output_dir.join("events.json"), &events_json)
        .expect("Failed to write events.json");
    println!("  Wrote events.json ({} events)", event_log.events.len());

    // Dashboard
    let dashboard = render_dashboard(&grid, &agents, &stats, &event_log, &seed_config.name);
    std::fs::write(output_dir.join("dashboard.html"), &dashboard)
        .expect("Failed to write dashboard.html");
    println!("  Wrote dashboard.html");

    // Highlights — generate for notable events
    let mut highlight_count = 0;
    for event in &event_log.events {
        match event {
            Event::FirstBond { tick, x, y, elements } => {
                let html = render_highlight(
                    &grid,
                    "First Bond",
                    &format!(
                        "The first chemical bond in this universe: {} and {} joined at ({}, {}). \
                         A new composite emerged from the primordial mix.",
                        elements[0], elements[1], x, y
                    ),
                    *tick,
                );
                std::fs::write(
                    output_dir.join("highlights/first-bonds.html"),
                    &html,
                ).expect("Failed to write highlight");
                highlight_count += 1;
            }
            Event::FirstCatalysis { tick, catalyst, reaction, .. } => {
                let html = render_highlight(
                    &grid,
                    "First Catalysis",
                    &format!(
                        "The first catalytic event: {} accelerated the bonding of {} and {}. \
                         Chemistry begins to bootstrap itself.",
                        catalyst, reaction[0], reaction[1]
                    ),
                    *tick,
                );
                std::fs::write(
                    output_dir.join("highlights/first-catalysis.html"),
                    &html,
                ).expect("Failed to write highlight");
                highlight_count += 1;
            }
            _ => {}
        }
        if highlight_count >= 3 {
            break;
        }
    }
    println!("  Wrote {} highlights", highlight_count);

    println!("\nAll artifacts written to {}", output_dir.display());
}

/// Place an agent on the grid according to the seed's placement strategy.
fn place_agent(grid: &Grid, seed: &SeedConfig, rng: &mut impl Rng) -> (usize, usize) {
    match seed.placement.strategy.as_str() {
        "density_by_depth" => {
            // More agents in shallows (right side where elevation is higher but still underwater)
            // Use a bias toward the middle-right where the tidal zone is
            let x = {
                let biased = rng.gen::<f64>().powf(0.7); // bias toward higher values
                (biased * grid.width as f64) as usize
            }
            .min(grid.width - 1);
            let y = rng.gen_range(0..grid.height);
            (x, y)
        }
        _ => {
            // Uniform random
            let x = rng.gen_range(0..grid.width);
            let y = rng.gen_range(0..grid.height);
            (x, y)
        }
    }
}
