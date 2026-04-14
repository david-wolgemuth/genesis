use crate::agents::entity::Agent;
use crate::world::grid::Grid;
use std::collections::HashMap;

/// Collect all agents in cells adjacent to (x, y), including the cell itself.
pub fn local_agents<'a>(
    grid: &Grid,
    agents: &'a HashMap<u64, Agent>,
    x: usize,
    y: usize,
) -> Vec<&'a Agent> {
    let mut result = Vec::new();

    // Include agents in the cell itself
    for &id in &grid.cell(x, y).agent_ids {
        if let Some(a) = agents.get(&id) {
            result.push(a);
        }
    }

    // Include agents in neighboring cells
    for (nx, ny) in grid.neighbors(x, y) {
        for &id in &grid.cell(nx, ny).agent_ids {
            if let Some(a) = agents.get(&id) {
                result.push(a);
            }
        }
    }

    result
}

/// Count agents of each element type in a cell.
pub fn element_census(
    grid: &Grid,
    agents: &HashMap<u64, Agent>,
    x: usize,
    y: usize,
) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for &id in &grid.cell(x, y).agent_ids {
        if let Some(agent) = agents.get(&id) {
            if let Some(name) = agent.element_name() {
                *counts.entry(name.to_string()).or_insert(0) += 1;
            }
        }
    }
    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}
