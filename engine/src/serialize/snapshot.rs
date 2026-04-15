use crate::agents::entity::Agent;
use crate::world::grid::{Activity, Grid};
use serde::Serialize;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Event log — the fossil record
// ---------------------------------------------------------------------------

/// Events worth recording. We snapshot transitions, not continuous state.
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

// ---------------------------------------------------------------------------
// Frame data — full grid state at a point in time
// ---------------------------------------------------------------------------

/// Per-cell snapshot data. This is the bulk of the frame file.
#[derive(Debug, Serialize)]
pub struct CellFrame {
    pub x: usize,
    pub y: usize,
    pub elevation: f64,
    pub temperature: f64,
    pub pressure: f64,
    pub energy: f64,
    pub agent_count: usize,
    pub bonded_count: usize,
    /// Max bond count of any single agent in this cell (proxy for complexity).
    pub max_complexity: usize,
    /// The element type with the highest count in this cell, if any.
    pub dominant_element: Option<String>,
    /// Activity level: 0.0 = cold, 0.5 = warm, 1.0 = hot.
    pub activity: f64,
}

/// Grid geometry and all cell states for one frame.
#[derive(Debug, Serialize)]
pub struct GridFrame {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<CellFrame>,
}

/// Per-frame aggregate statistics.
#[derive(Debug, Serialize)]
pub struct FrameStats {
    pub total_agents: usize,
    pub free: usize,
    pub bonded: usize,
    pub bonds_formed_this_tick: u64,
    pub bonds_broken_this_tick: u64,
}

/// Complete frame data: grid state + stats at a given tick.
#[derive(Debug, Serialize)]
pub struct FrameData {
    pub tick: u64,
    pub grid: GridFrame,
    pub stats: FrameStats,
}

impl FrameData {
    /// Capture current simulation state as a frame.
    pub fn capture(
        tick: u64,
        grid: &Grid,
        agents: &HashMap<u64, Agent>,
        bonds_formed_this_tick: u64,
        bonds_broken_this_tick: u64,
    ) -> Self {
        let total_agents = agents.len();
        let bonded = agents.values().filter(|a| !a.bonds.is_empty()).count();
        let free = total_agents - bonded;

        let cells = grid
            .cells
            .iter()
            .map(|cell| {
                let cell_agents: Vec<&Agent> = cell
                    .agent_ids
                    .iter()
                    .filter_map(|id| agents.get(id))
                    .collect();

                let bonded_count = cell_agents.iter().filter(|a| !a.bonds.is_empty()).count();

                let max_complexity = cell_agents
                    .iter()
                    .map(|a| a.bonds.len())
                    .max()
                    .unwrap_or(0);

                // Count element types to find dominant
                let mut elem_counts: HashMap<&str, usize> = HashMap::new();
                for agent in &cell_agents {
                    if let Some(name) = agent.element_name() {
                        *elem_counts.entry(name).or_insert(0) += 1;
                    }
                }
                let dominant_element = elem_counts
                    .into_iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(name, _)| name.to_string());

                let activity = match cell.activity {
                    Activity::Cold => 0.0,
                    Activity::Warm => 0.5,
                    Activity::Hot => 1.0,
                };

                CellFrame {
                    x: cell.x,
                    y: cell.y,
                    elevation: cell.elevation,
                    temperature: cell.temperature,
                    pressure: cell.pressure,
                    energy: cell.energy_budget,
                    agent_count: cell.agent_ids.len(),
                    bonded_count,
                    max_complexity,
                    dominant_element,
                    activity,
                }
            })
            .collect();

        FrameData {
            tick,
            grid: GridFrame {
                width: grid.width,
                height: grid.height,
                cells,
            },
            stats: FrameStats {
                total_agents,
                free,
                bonded,
                bonds_formed_this_tick,
                bonds_broken_this_tick,
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}
