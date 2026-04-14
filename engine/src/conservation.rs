use crate::agents::entity::Agent;
use std::collections::HashMap;

/// Element conservation checksum.
/// Counts every element in the simulation (free or bonded) and verifies
/// the totals match the expected counts established at initialization.
pub fn element_census(agents: &HashMap<u64, Agent>) -> HashMap<String, u64> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for agent in agents.values() {
        if let Some(name) = agent.element_name() {
            *counts.entry(name.to_string()).or_insert(0) += 1;
        }
    }
    counts
}

/// Verify conservation: current counts must match expected counts exactly.
/// Returns Ok(()) if conserved, Err with details if violated.
pub fn verify_conservation(
    agents: &HashMap<u64, Agent>,
    expected: &HashMap<String, u64>,
) -> Result<(), String> {
    let current = element_census(agents);

    for (element, &expected_count) in expected {
        let current_count = current.get(element).copied().unwrap_or(0);
        if current_count != expected_count {
            return Err(format!(
                "Conservation violation: {} expected {} but found {}",
                element, expected_count, current_count
            ));
        }
    }

    // Check for unexpected elements
    for (element, &count) in &current {
        if !expected.contains_key(element) {
            return Err(format!(
                "Conservation violation: unexpected element {} with count {}",
                element, count
            ));
        }
    }

    Ok(())
}
