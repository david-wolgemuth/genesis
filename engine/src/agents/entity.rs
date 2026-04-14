use std::collections::HashSet;

/// An agent is an entity on the grid. It can be a free element or a composite
/// of bonded elements. Entity-component architecture: traits, not types.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: u64,
    pub x: usize,
    pub y: usize,
    /// The element type name (references elements.toml). None for composites.
    pub element: Option<String>,
    /// Element mass from config. Affects diffusion rate.
    pub element_mass: f64,
    /// IDs of agents this entity is bonded to.
    pub bonds: HashSet<u64>,
    /// For composites: the list of sub-agent IDs.
    pub components: Vec<u64>,
}

impl Agent {
    pub fn new_element(id: u64, element: String, mass: f64, x: usize, y: usize) -> Self {
        Self {
            id,
            x,
            y,
            element: Some(element),
            element_mass: mass,
            bonds: HashSet::new(),
            components: Vec::new(),
        }
    }

    pub fn is_composite(&self) -> bool {
        !self.components.is_empty()
    }

    /// Mass from the element config, used for diffusion and other mass-dependent logic.
    pub fn mass(&self) -> f64 {
        self.element_mass
    }

    pub fn element_name(&self) -> Option<&str> {
        self.element.as_deref()
    }
}
