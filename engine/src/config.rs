//! Configuration types and loaders for Genesis.
//!
//! All simulation parameters are loaded from TOML files in the `universe/` directory.
//! The engine never hardcodes element properties or environment conditions — those
//! are the alien part of the simulation and belong in config. Only universal physical
//! constants and equations are hardcoded.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// --- Elements config (universe/elements.toml) ---

/// A single element type in this universe.
/// Defines the intrinsic properties that affect bonding, diffusion, and rendering.
#[derive(Debug, Deserialize, Clone)]
pub struct ElementDef {
    /// Unique name used as the key for bond rules and agent references.
    pub name: String,
    /// Display symbol (e.g., "α", "β") for rendering.
    pub symbol: String,
    /// Mass in simulation units. Affects diffusion rate (heavier = slower).
    pub mass: f64,
    /// Electric charge. Affects bonding compatibility.
    pub charge: i32,
    /// Four bonding slots [N, E, S, W]. 1 = open, 0 = closed.
    /// Two elements bond when one has an open slot opposite the other's open slot.
    pub shape: [u8; 4],
    /// Hex color for rendering (e.g., "#4a90d9").
    pub color: String,
}

/// A rule governing whether two element types can bond.
/// Loaded from `[[bond_rule]]` entries in elements.toml.
#[derive(Debug, Deserialize, Clone)]
pub struct BondRule {
    /// The two element names that this rule applies to (order-independent).
    pub pair: [String; 2],
    /// Energy released when the bond forms (exothermic). Joules in simulation units.
    #[serde(default)]
    pub energy_released: f64,
    /// Energy required to form the bond (endothermic). Must be available in the cell.
    #[serde(default)]
    pub energy_required: f64,
    /// Bond breaks if cell temperature exceeds this (kelvin).
    pub stability_max_temp: f64,
    /// Bond breaks if cell pressure drops below this (atm).
    pub stability_min_pressure: f64,
}

/// A catalysis rule: the presence of a catalyst element accelerates a specific reaction.
/// Loaded from `[[catalysis_rule]]` entries in elements.toml.
#[derive(Debug, Deserialize, Clone)]
pub struct CatalysisRule {
    /// Element name that acts as the catalyst.
    pub catalyst: String,
    /// The two-element reaction that is accelerated (order-independent).
    pub reaction: [String; 2],
    /// Multiplier applied to bond formation probability when catalyst is present.
    pub rate_multiplier: f64,
}

/// Top-level elements config, deserialized from universe/elements.toml.
#[derive(Debug, Deserialize, Clone)]
pub struct ElementsConfig {
    /// All element type definitions.
    #[serde(rename = "element")]
    pub elements: Vec<ElementDef>,
    /// Bond formation/breaking rules.
    #[serde(rename = "bond_rule")]
    pub bond_rules: Vec<BondRule>,
    /// Catalysis rules (optional).
    #[serde(default, rename = "catalysis_rule")]
    pub catalysis_rules: Vec<CatalysisRule>,
}

// --- Environment config (universe/environment.toml) ---

/// Grid dimensions and depth range.
#[derive(Debug, Deserialize, Clone)]
pub struct GridConfig {
    /// Number of cells along the x axis.
    pub width: usize,
    /// Number of cells along the y axis.
    pub height: usize,
    /// [min_elevation, max_elevation] for terrain generation.
    pub depth_range: [f64; 2],
}

/// Atmospheric conditions at the planet's surface.
#[derive(Debug, Deserialize, Clone)]
pub struct AtmosphereConfig {
    /// UV intensity at the surface (arbitrary energy units).
    pub uv_surface_intensity: f64,
}

/// Ocean properties affecting temperature distribution.
#[derive(Debug, Deserialize, Clone)]
pub struct OceanConfig {
    /// Temperature decrease per unit of depth (kelvin per depth unit).
    pub depth_temperature_gradient: f64,
    /// Temperature at sea level (kelvin).
    pub surface_temperature: f64,
}

/// Geothermal vent configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct GeothermalConfig {
    /// Number of geothermal vents to place.
    pub vent_count: usize,
    /// Energy output per vent per world tick.
    pub vent_energy_output: f64,
    /// Placement strategy: "random" or specific coordinates.
    pub vent_placement: String,
}

/// Top-level environment config, deserialized from universe/environment.toml.
#[derive(Debug, Deserialize, Clone)]
pub struct EnvironmentConfig {
    /// Grid size and depth range.
    pub grid: GridConfig,
    /// Atmospheric conditions.
    pub atmosphere: AtmosphereConfig,
    /// Ocean temperature model.
    pub ocean: OceanConfig,
    /// Geothermal vent parameters.
    pub geothermal: GeothermalConfig,
}

// --- Star config (universe/star.toml) ---

/// Properties of the host star.
#[derive(Debug, Deserialize, Clone)]
pub struct StarProps {
    /// Total energy output (arbitrary units).
    pub energy_output: f64,
    /// Distance from star in AU-equivalents. Scales received energy by 1/d^2.
    pub distance: f64,
}

/// Orbital mechanics for day/night cycle.
#[derive(Debug, Deserialize, Clone)]
pub struct OrbitConfig {
    /// Ticks per full day/night cycle.
    pub period: u64,
    /// Axial tilt affecting seasonal variation.
    pub axial_tilt: f64,
}

/// Parameters for how star energy interacts with the environment.
#[derive(Debug, Deserialize, Clone)]
pub struct EnergyModelConfig {
    /// UV exponential decay rate per unit of water depth.
    pub uv_water_attenuation: f64,
}

/// Top-level star config, deserialized from universe/star.toml.
#[derive(Debug, Deserialize, Clone)]
pub struct StarConfig {
    /// Star intrinsic properties.
    pub star: StarProps,
    /// Orbital parameters for day/night cycle.
    pub orbit: OrbitConfig,
    /// Energy model parameters.
    pub energy_model: EnergyModelConfig,
}

// --- Seed config (universe/seeds/*.toml) ---

/// Terrain generation parameters for a seed.
#[derive(Debug, Deserialize, Clone)]
pub struct TerrainSeed {
    /// Generation algorithm: "gradient", "flat", etc.
    #[serde(rename = "type")]
    pub terrain_type: String,
    /// Elevation at the left edge of the grid.
    pub left_depth: f64,
    /// Elevation at the right edge of the grid.
    pub right_depth: f64,
    /// Noise amplitude as a fraction of depth range (0.0 = smooth, 1.0 = rough).
    pub roughness: f64,
}

/// Initial agent counts per element type.
#[derive(Debug, Deserialize, Clone)]
pub struct InitialAgents {
    #[serde(default)]
    pub alpha: u64,
    #[serde(default)]
    pub beta: u64,
    #[serde(default)]
    pub gamma: u64,
    #[serde(default)]
    pub delta: u64,
    #[serde(default)]
    pub epsilon: u64,
}

impl InitialAgents {
    /// Convert to a map of element name -> count for iteration.
    pub fn as_map(&self) -> HashMap<String, u64> {
        let mut m = HashMap::new();
        if self.alpha > 0 { m.insert("alpha".to_string(), self.alpha); }
        if self.beta > 0 { m.insert("beta".to_string(), self.beta); }
        if self.gamma > 0 { m.insert("gamma".to_string(), self.gamma); }
        if self.delta > 0 { m.insert("delta".to_string(), self.delta); }
        if self.epsilon > 0 { m.insert("epsilon".to_string(), self.epsilon); }
        m
    }
}

/// Agent placement strategy.
#[derive(Debug, Deserialize, Clone)]
pub struct PlacementConfig {
    /// How to distribute initial agents: "density_by_depth", "uniform", etc.
    pub strategy: String,
}

/// Top-level seed config, deserialized from a universe/seeds/*.toml file.
/// Defines the initial conditions for a simulation run.
#[derive(Debug, Deserialize, Clone)]
pub struct SeedConfig {
    /// Human-readable name for this seed.
    pub name: String,
    /// Description of what this seed represents.
    pub description: String,
    /// Terrain generation parameters.
    pub terrain: TerrainSeed,
    /// How many of each element to place initially.
    pub initial_agents: InitialAgents,
    /// Placement strategy for initial agents.
    pub placement: PlacementConfig,
}

// --- Loading ---

/// Load element definitions, bond rules, and catalysis rules from universe/elements.toml.
pub fn load_elements(universe_dir: &Path) -> Result<ElementsConfig, String> {
    let path = universe_dir.join("elements.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Load environment config (grid, atmosphere, ocean, geothermal) from universe/environment.toml.
pub fn load_environment(universe_dir: &Path) -> Result<EnvironmentConfig, String> {
    let path = universe_dir.join("environment.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Load star config (energy output, orbit, energy model) from universe/star.toml.
pub fn load_star(universe_dir: &Path) -> Result<StarConfig, String> {
    let path = universe_dir.join("star.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Load a seed config (terrain, initial agents, placement) from a specific .toml file.
pub fn load_seed(seed_path: &Path) -> Result<SeedConfig, String> {
    let text = std::fs::read_to_string(seed_path)
        .map_err(|e| format!("Failed to read {}: {}", seed_path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", seed_path.display(), e))
}
