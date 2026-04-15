mod config;
mod conservation;
mod world;
mod agents;
mod render;
mod serialize;

use crate::agents::entity::Agent;
use crate::agents::tick::{agent_tick, TickStats};
use crate::config::*;
use crate::conservation::{element_census, verify_conservation};
use crate::serialize::curator::{curate, RunManifest};
use crate::serialize::narrator::narrate;
use crate::serialize::snapshot::{Event, EventLog, FrameData};
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
    let snapshot_interval = (total_ticks / 10).max(1); // up to 10 frames per run

    println!("Running {} cycles ({} ticks)...", cli.cycles, total_ticks);

    // Collected frames for writing at the end
    let mut frames: Vec<FrameData> = Vec::new();

    // Track bonds formed/broken per snapshot interval for frame stats.
    // stats.bonds_formed is cumulative; we track the previous value and diff.
    let mut bonds_formed_at_last_snapshot: u64 = 0;
    let mut bonds_broken_at_last_snapshot: u64 = 0;

    // Capture frame 0 (initial state)
    let initial_frame = FrameData::capture(0, &grid, &agents, 0, 0);
    frames.push(initial_frame);

    // Main simulation loop
    for _ in 0..total_ticks {
        clocks.tick();

        // World tick: recompute energy, diffusion, temperature, pressure
        if clocks.is_world_tick() {
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
            update_star_energy(&mut grid, &star_config, clocks.current_tick);
            apply_geothermal(
                &mut grid,
                &vent_positions,
                env_config.geothermal.vent_energy_output,
            );
            diffuse_agents(&mut grid, &mut agents, &mut rng);
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

                // Capture a full frame
                let formed_delta = stats.bonds_formed - bonds_formed_at_last_snapshot;
                let broken_delta = stats.bonds_broken - bonds_broken_at_last_snapshot;
                let frame = FrameData::capture(
                    clocks.current_tick,
                    &grid,
                    &agents,
                    formed_delta,
                    broken_delta,
                );
                frames.push(frame);
                bonds_formed_at_last_snapshot = stats.bonds_formed;
                bonds_broken_at_last_snapshot = stats.bonds_broken;
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

        // (delta tracking is done at snapshot time from cumulative stats)

        // Check for bond milestones
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

    // Capture final frame (only if it wasn't already captured by periodic snapshot)
    let already_captured = frames.last().map(|f| f.tick == clocks.current_tick).unwrap_or(false);
    if !already_captured {
        let final_frame = FrameData::capture(
            clocks.current_tick,
            &grid,
            &agents,
            stats.bonds_formed - bonds_formed_at_last_snapshot,
            stats.bonds_broken - bonds_broken_at_last_snapshot,
        );
        frames.push(final_frame);
    }

    println!("\nSimulation complete.");
    println!("  Bonds formed:  {}", stats.bonds_formed);
    println!("  Bonds broken:  {}", stats.bonds_broken);
    println!("  Conservation:  {}", if conservation_ok { "OK" } else { "FAILED" });
    println!("  Frames:        {}", frames.len());

    // -----------------------------------------------------------------------
    // Write output artifacts — data only, no HTML, no SVGs
    // -----------------------------------------------------------------------
    let output_dir = &cli.output;
    let frames_dir = output_dir.join("frames");
    std::fs::create_dir_all(&frames_dir).expect("Failed to create frames directory");

    // events.json
    let events_json = event_log.to_json();
    std::fs::write(output_dir.join("events.json"), &events_json)
        .expect("Failed to write events.json");
    println!("  Wrote events.json ({} events)", event_log.events.len());

    // frames/frame-NNNN.json — one per snapshot
    let mut frame_paths: Vec<String> = Vec::new();
    for frame in &frames {
        let filename = format!("frame-{:04}.json", frame.tick);
        let path = frames_dir.join(&filename);
        std::fs::write(&path, frame.to_json()).expect("Failed to write frame");
        frame_paths.push(format!("frames/{}", filename));
    }
    println!("  Wrote {} frame files to frames/", frames.len());

    // camera.json — curator's camera script
    let camera_script = curate(
        &event_log.events,
        frames.len(),
        grid.width,
        grid.height,
    );
    let camera_json = camera_script.to_json();
    std::fs::write(output_dir.join("camera.json"), &camera_json)
        .expect("Failed to write camera.json");
    println!(
        "  Wrote camera.json ({} keyframes)",
        camera_script.keyframes.len()
    );

    // narrator.md
    let narrative = narrate(
        &event_log.events,
        &seed_config.name,
        &seed_config.description,
        grid.width,
        grid.height,
        frames.len(),
    );
    std::fs::write(output_dir.join("narrator.md"), &narrative)
        .expect("Failed to write narrator.md");
    println!("  Wrote narrator.md");

    // manifest.json — viewer entry point
    let mut element_colors: HashMap<String, String> = HashMap::new();
    for elem in &elements_config.elements {
        element_colors.insert(elem.name.clone(), elem.color.clone());
    }
    let run_name = output_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("run-unknown")
        .to_string();
    let manifest = RunManifest {
        run: run_name,
        seed: seed_config.name.clone(),
        total_ticks: clocks.current_tick,
        total_agents: agents.len(),
        element_colors,
        frames: frame_paths,
    };
    std::fs::write(output_dir.join("manifest.json"), manifest.to_json())
        .expect("Failed to write manifest.json");
    println!("  Wrote manifest.json");

    println!("\nAll artifacts written to {}", output_dir.display());
}

/// Place an agent on the grid according to the seed's placement strategy.
fn place_agent(grid: &Grid, seed: &SeedConfig, rng: &mut impl Rng) -> (usize, usize) {
    match seed.placement.strategy.as_str() {
        "density_by_depth" => {
            let x = {
                let biased = rng.gen::<f64>().powf(0.7);
                (biased * grid.width as f64) as usize
            }
            .min(grid.width - 1);
            let y = rng.gen_range(0..grid.height);
            (x, y)
        }
        _ => {
            let x = rng.gen_range(0..grid.width);
            let y = rng.gen_range(0..grid.height);
            (x, y)
        }
    }
}
