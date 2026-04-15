use crate::serialize::snapshot::Event;

/// Write a plain-language narrative of the simulation run.
///
/// This is a field researcher's log: what happened, what was expected,
/// what was surprising. Written to narrator.md in the run directory.
pub fn narrate(
    events: &[Event],
    seed_name: &str,
    seed_description: &str,
    grid_width: usize,
    grid_height: usize,
    total_frames: usize,
) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!("# Run Narrative — {}\n\n", seed_name));
    md.push_str(&format!("*{}*\n\n", seed_description));
    md.push_str(&format!("Grid: {}×{} cells. {} frames captured.\n\n", grid_width, grid_height, total_frames));
    md.push_str("---\n\n");

    // Pull out key events
    let first_bond = events.iter().find_map(|e| {
        if let Event::FirstBond { tick, x, y, elements } = e {
            Some((*tick, *x, *y, elements.clone()))
        } else {
            None
        }
    });

    let first_catalysis = events.iter().find_map(|e| {
        if let Event::FirstCatalysis { tick, x, y, catalyst, reaction } = e {
            Some((*tick, *x, *y, catalyst.clone(), reaction.clone()))
        } else {
            None
        }
    });

    let milestones: Vec<(u64, u64)> = events
        .iter()
        .filter_map(|e| {
            if let Event::BondCountMilestone { tick, count } = e {
                Some((*tick, *count))
            } else {
                None
            }
        })
        .collect();

    let final_end = events.iter().find_map(|e| {
        if let Event::SimulationEnd { tick, total_bonds_formed, total_bonds_broken, conservation_ok } = e {
            Some((*tick, *total_bonds_formed, *total_bonds_broken, *conservation_ok))
        } else {
            None
        }
    });

    let pop_snapshots: Vec<(u64, u64, u64, u64)> = events
        .iter()
        .filter_map(|e| {
            if let Event::PopulationSnapshot { tick, free_agents, bonded_agents, total_bonds, .. } = e {
                Some((*tick, *free_agents, *bonded_agents, *total_bonds))
            } else {
                None
            }
        })
        .collect();

    // The chemistry ignition
    md.push_str("## The Birth of Chemistry\n\n");
    if let Some((tick, x, y, elements)) = &first_bond {
        md.push_str(&format!(
            "The first bond in this universe formed at tick **{}**, at grid position ({}, {}). \
             A **{}** and a **{}** found each other and locked together — the first composite \
             entity in the simulation.\n\n",
            tick, x, y, elements[0], elements[1]
        ));
    } else {
        md.push_str(
            "No bonds formed during this run. The elements drifted freely without making contact, \
             or conditions were never right for bond formation. This can happen when element \
             density is too low or energy conditions don't match any bond rule.\n\n",
        );
    }

    // Catalysis
    if let Some((tick, x, y, catalyst, reaction)) = &first_catalysis {
        md.push_str("## Catalysis Emerges\n\n");
        md.push_str(&format!(
            "At tick **{}**, the first catalytic event occurred at ({}, {}). A **{}** element \
             was present when **{}** and **{}** bonded — its proximity multiplied the bond \
             formation rate. Chemistry began to bootstrap itself: the presence of one composite \
             made it easier for more composites to form nearby.\n\n",
            tick, x, y, catalyst, reaction[0], reaction[1]
        ));
    }

    // Bond milestones
    if !milestones.is_empty() {
        md.push_str("## Bonding Activity\n\n");
        for (tick, count) in &milestones {
            md.push_str(&format!(
                "- Tick {}: **{}** total bonds formed\n",
                tick, count
            ));
        }
        md.push('\n');

        if milestones.len() >= 3 {
            md.push_str(
                "Bond formation accelerated over time, suggesting catalytic feedback \
                 loops or concentration effects in active regions.\n\n",
            );
        }
    }

    // Population trajectory
    if pop_snapshots.len() >= 2 {
        md.push_str("## Population Trajectory\n\n");
        md.push_str("| Tick | Free | Bonded | Bonds |\n");
        md.push_str("|------|------|--------|-------|\n");
        for (tick, free, bonded, bonds) in &pop_snapshots {
            md.push_str(&format!("| {} | {} | {} | {} |\n", tick, free, bonded, bonds));
        }
        md.push('\n');

        // Narrative on trajectory
        let first_snap = pop_snapshots.first().unwrap();
        let last_snap = pop_snapshots.last().unwrap();
        let bonded_fraction_start = first_snap.2 as f64 / (first_snap.1 + first_snap.2).max(1) as f64;
        let bonded_fraction_end = last_snap.2 as f64 / (last_snap.1 + last_snap.2).max(1) as f64;

        if bonded_fraction_end > bonded_fraction_start + 0.05 {
            md.push_str(&format!(
                "The bonded fraction grew from {:.1}% to {:.1}% over the run — \
                 chemistry was actively building structure.\n\n",
                bonded_fraction_start * 100.0,
                bonded_fraction_end * 100.0
            ));
        } else if bonded_fraction_end > 0.0 {
            md.push_str(&format!(
                "The bonded fraction remained stable around {:.1}% — \
                 bond formation and breaking were roughly in equilibrium.\n\n",
                bonded_fraction_end * 100.0
            ));
        }
    }

    // Final state
    if let Some((tick, formed, broken, conservation_ok)) = final_end {
        md.push_str("## Final State\n\n");
        md.push_str(&format!(
            "The simulation ran to tick **{}**. \
             **{}** bonds formed, **{}** broke. \
             Net bonds: **{}**. Conservation law: **{}**.\n\n",
            tick,
            formed,
            broken,
            formed.saturating_sub(broken),
            if conservation_ok { "HELD — no elements were created or destroyed" } else { "VIOLATED — bug detected" }
        ));

        if !conservation_ok {
            md.push_str(
                "> **Conservation violation detected.** This indicates a bug in the simulation. \
                 The element count changed during the run. This run's data should not be trusted.\n\n",
            );
        }
    }

    // What's next
    md.push_str("## Recommendations for Next Cycle\n\n");

    if first_bond.is_none() {
        md.push_str(
            "- **Increase element density** or reduce grid size. With current parameters, \
             elements rarely share a cell long enough to bond.\n",
        );
        md.push_str("- **Reduce bond energy requirements** — some endothermic bonds may never form if energy is too sparse.\n");
    } else if first_catalysis.is_none() {
        md.push_str(
            "- **Add more catalytic elements** (gamma, epsilon) to the initial distribution. \
             Catalysis emerged only if concentrations were high enough for catalysts to be in the same cell as reacting pairs.\n",
        );
    } else {
        md.push_str(
            "- **Arrhenius kinetics** — temperature-dependent reaction rates are not yet implemented. \
             Adding them would make the volcanic ridge significantly more active than the deep ocean.\n",
        );
        md.push_str(
            "- **Composite tracking** — larger composites (3+ elements) are not yet forming. \
             Composite-to-composite bonding would be the next step toward complex chemistry.\n",
        );
    }

    md.push_str(
        "- **Viewer** — load this run in `viewer/viewer.html?run=run-XXXX` to see the 3D terrain \
         animation and camera script playback.\n",
    );

    md
}
