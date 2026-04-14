use crate::config::SeedConfig;
use crate::world::grid::Grid;
use rand::Rng;

/// Initialize grid terrain from seed configuration.
pub fn init_terrain(grid: &mut Grid, seed: &SeedConfig, rng: &mut impl Rng) {
    let width = grid.width as f64;
    for y in 0..grid.height {
        for x in 0..grid.width {
            let t = if grid.width <= 1 { 0.0 } else { x as f64 / (width - 1.0) };
            // Linear gradient from left_depth to right_depth
            let base = seed.terrain.left_depth * (1.0 - t) + seed.terrain.right_depth * t;
            // Add roughness noise
            let noise = (rng.gen::<f64>() - 0.5) * 2.0 * seed.terrain.roughness * 50.0;
            grid.cell_mut(x, y).elevation = base + noise;
        }
    }
}

/// Derive pressure from depth. Pressure increases linearly with depth underwater.
/// At surface, pressure = 1.0 atm.
///
/// Uses the hydrostatic pressure equation: P = P_surface + (rho * g * h) / P_atm.
/// The constant 10.0 represents the depth (in simulation units) per additional
/// atmosphere of pressure — analogous to Earth's ~10.3 meters of seawater per atm.
/// This is a universal physical relationship, not a tunable parameter.
const DEPTH_PER_ATM: f64 = 10.0;

pub fn pressure_from_depth(depth: f64) -> f64 {
    1.0 + (depth / DEPTH_PER_ATM).max(0.0)
}

/// Derive base temperature from depth and surface temperature.
/// Deeper water is colder (gradient from config).
///
/// Temperature floors at the cosmic microwave background temperature (~2.7K)
/// as an absolute minimum, but practically floors at a cold-ocean baseline.
const MIN_OCEAN_TEMP: f64 = 200.0; // kelvin — deep ocean floor minimum

pub fn temperature_from_depth(depth: f64, surface_temp: f64, gradient: f64) -> f64 {
    if depth > 0.0 {
        // Underwater: temperature decreases with depth
        (surface_temp - depth * gradient).max(MIN_OCEAN_TEMP)
    } else {
        surface_temp
    }
}
