/// Three-tier clock hierarchy.
/// Stellar clock: 1 tick per cycle (updates star angle, day/night).
/// World clock: ~10 ticks per cycle (temperature, diffusion, pressure, UV).
/// Agent clock: ~1000 ticks per cycle (bonding, movement, interaction).
pub struct ClockSystem {
    pub stellar_period: u64,
    pub world_period: u64,
    pub agent_period: u64,
    pub current_tick: u64,
}

impl ClockSystem {
    pub fn new(stellar_period: u64, world_period: u64, agent_period: u64) -> Self {
        Self {
            stellar_period,
            world_period,
            agent_period,
            current_tick: 0,
        }
    }

    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    pub fn is_stellar_tick(&self) -> bool {
        self.current_tick > 0 && self.current_tick % self.stellar_period == 0
    }

    pub fn is_world_tick(&self) -> bool {
        self.current_tick > 0 && self.current_tick % self.world_period == 0
    }

    /// Agent ticks happen every tick (the innermost loop).
    pub fn is_agent_tick(&self) -> bool {
        true
    }

    pub fn stellar_ticks_elapsed(&self) -> u64 {
        self.current_tick / self.stellar_period
    }

    pub fn world_ticks_elapsed(&self) -> u64 {
        self.current_tick / self.world_period
    }
}
