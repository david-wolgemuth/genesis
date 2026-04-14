use crate::config::StarConfig;
use crate::world::grid::Grid;

/// Update energy input for all cells based on star position.
/// The star sweeps across the grid as a cosine function of the current tick.
pub fn update_star_energy(grid: &mut Grid, star: &StarConfig, tick: u64) {
    let period = star.orbit.period as f64;
    let phase = (tick as f64 / period) * 2.0 * std::f64::consts::PI;

    let base_energy = star.star.energy_output / (star.star.distance * star.star.distance);

    for cell in &mut grid.cells {
        // Star angle relative to cell x position — creates day/night sweep
        let cell_phase = phase + (cell.x as f64 / grid.width as f64) * std::f64::consts::PI;
        let cos_angle = cell_phase.cos();

        // Day side gets energy, night side gets none
        let surface_energy = if cos_angle > 0.0 {
            base_energy * cos_angle
        } else {
            0.0
        };

        // UV attenuation underwater — exponential decay with depth
        let depth = cell.depth();
        let uv_atten = (-star.energy_model.uv_water_attenuation * depth).exp();
        cell.uv_intensity = surface_energy * uv_atten;

        // Energy budget starts from star energy; geothermal is added separately
        cell.energy_budget = cell.uv_intensity;
    }
}

/// Reset energy budgets to zero before recomputing sources.
pub fn reset_energy(grid: &mut Grid) {
    for cell in &mut grid.cells {
        cell.energy_budget = 0.0;
        cell.uv_intensity = 0.0;
    }
}

/// Place geothermal vents and add their energy output to cells.
pub fn apply_geothermal(grid: &mut Grid, vent_positions: &[(usize, usize)], energy_output: f64) {
    for &(vx, vy) in vent_positions {
        // Vent heats its cell and neighbors
        let neighbors = grid.neighbors(vx, vy);
        let idx = grid.idx(vx, vy);
        grid.cells[idx].energy_budget += energy_output;
        grid.cells[idx].temperature += energy_output * 0.5;
        for (nx, ny) in neighbors {
            let nidx = grid.idx(nx, ny);
            grid.cells[nidx].energy_budget += energy_output * 0.3;
            grid.cells[nidx].temperature += energy_output * 0.1;
        }
    }
}
