use crate::agents::bonding;
use crate::agents::entity::Agent;
use crate::config::ElementsConfig;
use crate::serialize::snapshot::Event;
use crate::world::grid::{Activity, Grid};
use rand::Rng;
use std::collections::HashMap;

/// Run one agent tick across the entire grid.
/// Returns events that occurred during this tick.
pub fn agent_tick(
    grid: &mut Grid,
    agents: &mut HashMap<u64, Agent>,
    config: &ElementsConfig,
    tick: u64,
    rng: &mut impl Rng,
    stats: &mut TickStats,
) -> Vec<Event> {
    let mut events = Vec::new();

    // Collect cells to process (skip cold cells)
    let cells_to_process: Vec<(usize, usize)> = (0..grid.height)
        .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let cell = grid.cell(x, y);
            match cell.activity {
                Activity::Hot => true,
                Activity::Warm => tick % 10 == 0, // Reduced frequency for warm cells
                Activity::Cold => false,
            }
        })
        .collect();

    // Phase 1: Attempt new bonds
    let mut all_new_bonds = Vec::new();
    for &(x, y) in &cells_to_process {
        let new_bonds = bonding::attempt_bonds(
            grid, agents, x, y, &config.elements, &config.bond_rules, &config.catalysis_rules, rng,
        );
        all_new_bonds.extend(new_bonds);
    }

    // Apply new bonds
    for (a_id, b_id) in &all_new_bonds {
        if let Some(a) = agents.get_mut(a_id) {
            a.bonds.insert(*b_id);
        }
        if let Some(b) = agents.get_mut(b_id) {
            b.bonds.insert(*a_id);
        }
        stats.bonds_formed += 1;

        // Check for interesting events
        let a_name = agents.get(a_id).and_then(|a| a.element_name()).unwrap_or("?").to_string();
        let b_name = agents.get(b_id).and_then(|b| b.element_name()).unwrap_or("?").to_string();
        let (ax, ay) = agents.get(a_id).map(|a| (a.x, a.y)).unwrap_or((0, 0));

        if stats.total_bonds_ever == 0 {
            events.push(Event::FirstBond {
                tick,
                x: ax,
                y: ay,
                elements: vec![a_name.clone(), b_name.clone()],
            });
        }
        stats.total_bonds_ever += 1;

        // Check for catalytic events
        let cell = grid.cell(ax, ay);
        let cell_agent_ids: Vec<u64> = cell.agent_ids.iter().copied().collect();
        let cell_agents: Vec<&Agent> = cell_agent_ids.iter().filter_map(|id| agents.get(id)).collect();
        let cat_result = bonding::catalysis_check(&cell_agents, &a_name, &b_name, &config.catalysis_rules);
        if cat_result.multiplier > 1.0 && !stats.catalysis_seen {
            stats.catalysis_seen = true;
            events.push(Event::FirstCatalysis {
                tick,
                x: ax,
                y: ay,
                catalyst: cat_result.catalyst.unwrap_or_else(|| "unknown".to_string()),
                reaction: vec![a_name.clone(), b_name.clone()],
            });
        }
    }

    // Phase 2: Check for bond breaking
    let mut bonds_to_break = Vec::new();
    for (id, agent) in agents.iter() {
        if agent.bonds.is_empty() {
            continue;
        }
        let cell = grid.cell(agent.x, agent.y);
        for &bonded_id in &agent.bonds {
            if *id < bonded_id {
                // Process each pair only once
                if let Some(bonded) = agents.get(&bonded_id) {
                    if bonding::should_break_bond(
                        agent,
                        bonded,
                        &config.bond_rules,
                        cell.temperature,
                        cell.pressure,
                    ) {
                        bonds_to_break.push((*id, bonded_id));
                    }
                }
            }
        }
    }

    for (a_id, b_id) in &bonds_to_break {
        if let Some(a) = agents.get_mut(a_id) {
            a.bonds.remove(b_id);
        }
        if let Some(b) = agents.get_mut(b_id) {
            b.bonds.remove(a_id);
        }
        stats.bonds_broken += 1;
    }

    events
}

/// Track statistics across ticks for event detection.
pub struct TickStats {
    pub bonds_formed: u64,
    pub bonds_broken: u64,
    pub total_bonds_ever: u64,
    pub catalysis_seen: bool,
    pub max_composite_size: usize,
    /// Milestone thresholds that have already been emitted.
    pub milestones_emitted: Vec<u64>,
}

impl TickStats {
    pub fn new() -> Self {
        Self {
            bonds_formed: 0,
            bonds_broken: 0,
            total_bonds_ever: 0,
            catalysis_seen: false,
            max_composite_size: 0,
            milestones_emitted: Vec::new(),
        }
    }

    /// Check if a milestone threshold has been crossed and not yet emitted.
    pub fn check_milestone(&mut self, threshold: u64) -> bool {
        if self.total_bonds_ever >= threshold && !self.milestones_emitted.contains(&threshold) {
            self.milestones_emitted.push(threshold);
            true
        } else {
            false
        }
    }
}
