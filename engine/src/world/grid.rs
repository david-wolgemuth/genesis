use std::collections::HashSet;

/// A single cell on the 2D grid.
#[derive(Debug, Clone)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub elevation: f64,
    pub temperature: f64,
    pub pressure: f64,
    pub uv_intensity: f64,
    pub energy_budget: f64,
    pub agent_ids: HashSet<u64>,
    pub activity: Activity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Hot,
    Warm,
    Cold,
}

impl Cell {
    /// Create a new cell. Temperature and pressure are initialized to defaults
    /// and recomputed from terrain during simulation setup.
    pub fn new(x: usize, y: usize, elevation: f64) -> Self {
        Self {
            x,
            y,
            elevation,
            temperature: 0.0,  // recomputed from depth + config during init
            pressure: 0.0,     // recomputed from depth during init
            uv_intensity: 0.0,
            energy_budget: 0.0,
            agent_ids: HashSet::new(),
            activity: Activity::Cold,
        }
    }

    pub fn is_underwater(&self) -> bool {
        self.elevation < 0.0
    }

    pub fn depth(&self) -> f64 {
        if self.elevation < 0.0 {
            -self.elevation
        } else {
            0.0
        }
    }
}

/// The 2D simulation grid.
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                cells.push(Cell::new(x, y, 0.0));
            }
        }
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[self.idx(x, y)]
    }

    pub fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let idx = self.idx(x, y);
        &mut self.cells[idx]
    }

    /// Returns the (x, y) coordinates of cells adjacent to (x, y),
    /// including diagonals (8-neighborhood, clamped to grid bounds).
    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(8);
        let x_min = x.saturating_sub(1);
        let y_min = y.saturating_sub(1);
        let x_max = (x + 1).min(self.width - 1);
        let y_max = (y + 1).min(self.height - 1);
        for ny in y_min..=y_max {
            for nx in x_min..=x_max {
                if nx != x || ny != y {
                    result.push((nx, ny));
                }
            }
        }
        result
    }

    /// Tag cells by activity level for compute optimization.
    /// Hot cells get full agent tick resolution. Warm cells tick at reduced
    /// frequency. Cold cells are skipped entirely.
    const HOT_AGENT_THRESHOLD: usize = 3;
    const HOT_ENERGY_THRESHOLD: f64 = 10.0;

    pub fn update_activity(&mut self) {
        for cell in &mut self.cells {
            if cell.agent_ids.is_empty() {
                cell.activity = Activity::Cold;
            } else if cell.agent_ids.len() >= Self::HOT_AGENT_THRESHOLD
                || cell.energy_budget > Self::HOT_ENERGY_THRESHOLD
            {
                cell.activity = Activity::Hot;
            } else {
                cell.activity = Activity::Warm;
            }
        }
    }
}
