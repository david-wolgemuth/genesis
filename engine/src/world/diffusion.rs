use crate::agents::entity::Agent;
use crate::world::grid::Grid;
use rand::Rng;
use std::collections::HashMap;

/// Fick's first law discretized on the grid.
/// Free (unbonded) agents drift toward cells with lower agent density.
/// Diffusion rate inversely proportional to agent mass.
pub fn diffuse_agents(
    grid: &mut Grid,
    agents: &mut HashMap<u64, Agent>,
    rng: &mut impl Rng,
) {
    // Collect agents that can diffuse (free, unbonded agents)
    let diffusable: Vec<u64> = agents
        .iter()
        .filter(|(_, a)| a.bonds.is_empty() && !a.is_composite())
        .map(|(id, _)| *id)
        .collect();

    for agent_id in diffusable {
        let agent = match agents.get(&agent_id) {
            Some(a) => a,
            None => continue,
        };
        let (ax, ay) = (agent.x, agent.y);
        let mass = agent.mass();

        // Diffusion probability inversely proportional to mass
        let diff_prob = (1.0 / mass).min(1.0) * 0.3;
        if rng.gen::<f64>() > diff_prob {
            continue;
        }

        let neighbors = grid.neighbors(ax, ay);
        if neighbors.is_empty() {
            continue;
        }

        // Find the neighbor with lowest agent density (Fick's law: flux toward lower concentration)
        let current_density = grid.cell(ax, ay).agent_ids.len();
        let mut best = None;
        let mut best_density = current_density;

        for &(nx, ny) in &neighbors {
            let nd = grid.cell(nx, ny).agent_ids.len();
            if nd < best_density {
                best_density = nd;
                best = Some((nx, ny));
            }
        }

        // If all neighbors are equally or more dense, pick a random one with some probability
        let target = if let Some(pos) = best {
            pos
        } else if rng.gen::<f64>() < 0.1 {
            neighbors[rng.gen_range(0..neighbors.len())]
        } else {
            continue;
        };

        // Move the agent
        let (tx, ty) = target;
        grid.cell_mut(ax, ay).agent_ids.remove(&agent_id);
        grid.cell_mut(tx, ty).agent_ids.insert(agent_id);
        if let Some(a) = agents.get_mut(&agent_id) {
            a.x = tx;
            a.y = ty;
        }
    }
}
