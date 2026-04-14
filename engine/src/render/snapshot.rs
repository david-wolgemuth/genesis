use serde::Serialize;

/// Events worth recording — the fossil record.
/// We snapshot transitions, not continuous state.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "first_bond")]
    FirstBond {
        tick: u64,
        x: usize,
        y: usize,
        elements: Vec<String>,
    },
    #[serde(rename = "first_composite_3plus")]
    FirstComposite3Plus {
        tick: u64,
        x: usize,
        y: usize,
        size: usize,
        elements: Vec<String>,
    },
    #[serde(rename = "first_catalysis")]
    FirstCatalysis {
        tick: u64,
        x: usize,
        y: usize,
        catalyst: String,
        reaction: Vec<String>,
    },
    #[serde(rename = "bond_count_milestone")]
    BondCountMilestone {
        tick: u64,
        count: u64,
    },
    #[serde(rename = "population_snapshot")]
    PopulationSnapshot {
        tick: u64,
        free_agents: u64,
        bonded_agents: u64,
        total_bonds: u64,
        element_counts: Vec<(String, u64)>,
    },
    #[serde(rename = "simulation_end")]
    SimulationEnd {
        tick: u64,
        total_bonds_formed: u64,
        total_bonds_broken: u64,
        conservation_ok: bool,
    },
}

/// Accumulated event log for the run.
pub struct EventLog {
    pub events: Vec<Event>,
}

impl EventLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn record(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.events).unwrap_or_else(|_| "[]".to_string())
    }
}
