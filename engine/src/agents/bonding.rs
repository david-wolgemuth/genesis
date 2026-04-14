use crate::config::{BondRule, CatalysisRule, ElementDef};
use crate::world::grid::Grid;
use crate::agents::entity::Agent;
use rand::Rng;
use std::collections::HashMap;

/// Evaluate whether two agents can bond, given local conditions.
pub fn can_bond(
    a: &Agent,
    b: &Agent,
    rule: &BondRule,
    cell_temp: f64,
    cell_pressure: f64,
    cell_energy: f64,
    elements: &[ElementDef],
) -> bool {
    // Both must be free elements (not already composites with full bonds)
    let a_elem = match &a.element {
        Some(e) => e,
        None => return false,
    };
    let b_elem = match &b.element {
        Some(e) => e,
        None => return false,
    };

    // Check the rule matches this pair
    let pair_matches = (rule.pair[0] == *a_elem && rule.pair[1] == *b_elem)
        || (rule.pair[0] == *b_elem && rule.pair[1] == *a_elem);
    if !pair_matches {
        return false;
    }

    // Check shape compatibility: at least one matching open slot
    let a_def = elements.iter().find(|e| e.name == *a_elem);
    let b_def = elements.iter().find(|e| e.name == *b_elem);
    if let (Some(a_def), Some(b_def)) = (a_def, b_def) {
        // Compatible if any slot in a is open (1) where corresponding slot in b is open (1)
        // Using complementary check: a's slot i matches b's slot (i+2)%4 (opposite face)
        let mut compatible = false;
        for i in 0..4 {
            let opposite = (i + 2) % 4;
            if a_def.shape[i] == 1 && b_def.shape[opposite] == 1 {
                compatible = true;
                break;
            }
        }
        if !compatible {
            return false;
        }
    }

    // Check stability conditions
    if cell_temp > rule.stability_max_temp {
        return false;
    }
    if cell_pressure < rule.stability_min_pressure {
        return false;
    }

    // Endothermic bonds require energy input
    if rule.energy_required > 0.0 && cell_energy < rule.energy_required {
        return false;
    }

    true
}

/// Find matching bond rule for a pair of element names.
pub fn find_bond_rule<'a>(
    a_name: &str,
    b_name: &str,
    rules: &'a [BondRule],
) -> Option<&'a BondRule> {
    rules.iter().find(|r| {
        (r.pair[0] == a_name && r.pair[1] == b_name)
            || (r.pair[0] == b_name && r.pair[1] == a_name)
    })
}

/// Result of checking for catalysis in a cell.
pub struct CatalysisResult {
    pub multiplier: f64,
    /// Name of the catalyst that triggered the boost, if any.
    pub catalyst: Option<String>,
}

/// Check if a catalyst is present in the cell for a given reaction.
/// Returns the combined multiplier and the name of the first matching catalyst.
pub fn catalysis_check(
    cell_agents: &[&Agent],
    a_name: &str,
    b_name: &str,
    catalysis_rules: &[CatalysisRule],
) -> CatalysisResult {
    let mut multiplier = 1.0;
    let mut catalyst_name = None;
    for rule in catalysis_rules {
        let reaction_matches = (rule.reaction[0] == a_name && rule.reaction[1] == b_name)
            || (rule.reaction[0] == b_name && rule.reaction[1] == a_name);
        if !reaction_matches {
            continue;
        }
        // Check if catalyst element is present in the cell
        let catalyst_present = cell_agents.iter().any(|agent| {
            agent.element_name() == Some(&rule.catalyst)
        });
        if catalyst_present {
            multiplier *= rule.rate_multiplier;
            if catalyst_name.is_none() {
                catalyst_name = Some(rule.catalyst.clone());
            }
        }
    }
    CatalysisResult { multiplier, catalyst: catalyst_name }
}

/// Check whether an existing bond should break under current conditions.
pub fn should_break_bond(
    a: &Agent,
    b: &Agent,
    rules: &[BondRule],
    cell_temp: f64,
    cell_pressure: f64,
) -> bool {
    let a_name = match a.element_name() {
        Some(n) => n,
        None => return false,
    };
    let b_name = match b.element_name() {
        Some(n) => n,
        None => return false,
    };

    if let Some(rule) = find_bond_rule(a_name, b_name, rules) {
        // Bond breaks if temperature exceeds max or pressure drops below min
        cell_temp > rule.stability_max_temp || cell_pressure < rule.stability_min_pressure
    } else {
        // No rule found — bond shouldn't exist, break it
        true
    }
}

/// Attempt bonding between agents in a cell. Returns list of (agent_a_id, agent_b_id) pairs that bonded.
pub fn attempt_bonds(
    grid: &Grid,
    agents: &HashMap<u64, Agent>,
    x: usize,
    y: usize,
    elements: &[ElementDef],
    bond_rules: &[BondRule],
    catalysis_rules: &[CatalysisRule],
    rng: &mut impl Rng,
) -> Vec<(u64, u64)> {
    let cell = grid.cell(x, y);
    let cell_agents: Vec<&Agent> = cell
        .agent_ids
        .iter()
        .filter_map(|id| agents.get(id))
        .collect();

    let mut new_bonds = Vec::new();

    // Try all pairs of unbonded free elements in this cell
    for i in 0..cell_agents.len() {
        for j in (i + 1)..cell_agents.len() {
            let a = cell_agents[i];
            let b = cell_agents[j];

            // Skip if either is already bonded
            if !a.bonds.is_empty() || !b.bonds.is_empty() {
                continue;
            }

            let a_name = match a.element_name() {
                Some(n) => n,
                None => continue,
            };
            let b_name = match b.element_name() {
                Some(n) => n,
                None => continue,
            };

            if let Some(rule) = find_bond_rule(a_name, b_name, bond_rules) {
                if can_bond(a, b, rule, cell.temperature, cell.pressure, cell.energy_budget, elements) {
                    // Apply catalysis multiplier to bond probability
                    let cat_result = catalysis_check(&cell_agents, a_name, b_name, catalysis_rules);
                    let bond_prob = (0.1 * cat_result.multiplier).min(1.0);

                    if rng.gen::<f64>() < bond_prob {
                        new_bonds.push((a.id, b.id));
                    }
                }
            }
        }
    }

    new_bonds
}
