use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// --- Elements config ---

#[derive(Debug, Deserialize, Clone)]
pub struct ElementDef {
    pub name: String,
    pub symbol: String,
    pub mass: f64,
    pub charge: i32,
    pub shape: [u8; 4],
    pub color: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BondRule {
    pub pair: [String; 2],
    #[serde(default)]
    pub energy_released: f64,
    #[serde(default)]
    pub energy_required: f64,
    pub stability_max_temp: f64,
    pub stability_min_pressure: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CatalysisRule {
    pub catalyst: String,
    pub reaction: [String; 2],
    pub rate_multiplier: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ElementsConfig {
    #[serde(rename = "element")]
    pub elements: Vec<ElementDef>,
    #[serde(rename = "bond_rule")]
    pub bond_rules: Vec<BondRule>,
    #[serde(default, rename = "catalysis_rule")]
    pub catalysis_rules: Vec<CatalysisRule>,
}

// --- Environment config ---

#[derive(Debug, Deserialize, Clone)]
pub struct GridConfig {
    pub width: usize,
    pub height: usize,
    pub depth_range: [f64; 2],
}

#[derive(Debug, Deserialize, Clone)]
pub struct AtmosphereConfig {
    pub uv_surface_intensity: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OceanConfig {
    pub depth_temperature_gradient: f64,
    pub surface_temperature: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeothermalConfig {
    pub vent_count: usize,
    pub vent_energy_output: f64,
    pub vent_placement: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnvironmentConfig {
    pub grid: GridConfig,
    pub atmosphere: AtmosphereConfig,
    pub ocean: OceanConfig,
    pub geothermal: GeothermalConfig,
}

// --- Star config ---

#[derive(Debug, Deserialize, Clone)]
pub struct StarProps {
    pub energy_output: f64,
    pub distance: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrbitConfig {
    pub period: u64,
    pub axial_tilt: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnergyModelConfig {
    pub uv_water_attenuation: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StarConfig {
    pub star: StarProps,
    pub orbit: OrbitConfig,
    pub energy_model: EnergyModelConfig,
}

// --- Seed config ---

#[derive(Debug, Deserialize, Clone)]
pub struct TerrainSeed {
    #[serde(rename = "type")]
    pub terrain_type: String,
    pub left_depth: f64,
    pub right_depth: f64,
    pub roughness: f64,
}

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

#[derive(Debug, Deserialize, Clone)]
pub struct PlacementConfig {
    pub strategy: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SeedConfig {
    pub name: String,
    pub description: String,
    pub terrain: TerrainSeed,
    pub initial_agents: InitialAgents,
    pub placement: PlacementConfig,
}

// --- Loading ---

pub fn load_elements(universe_dir: &Path) -> Result<ElementsConfig, String> {
    let path = universe_dir.join("elements.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_environment(universe_dir: &Path) -> Result<EnvironmentConfig, String> {
    let path = universe_dir.join("environment.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_star(universe_dir: &Path) -> Result<StarConfig, String> {
    let path = universe_dir.join("star.toml");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

pub fn load_seed(seed_path: &Path) -> Result<SeedConfig, String> {
    let text = std::fs::read_to_string(seed_path)
        .map_err(|e| format!("Failed to read {}: {}", seed_path.display(), e))?;
    toml::from_str(&text)
        .map_err(|e| format!("Failed to parse {}: {}", seed_path.display(), e))
}
