# Genesis: An Emergent Chemistry-to-Life Simulation

## What This Is

A simulation of emergent complexity — from elements to molecules to self-replicators to organisms — running on an alien planet with real physics but novel chemistry. The planet orbits a star, has terrain with depth and elevation, and receives energy that drives all chemistry. The simulation runs in CI (GitHub Actions), produces visual artifacts, and accumulates into a browsable archive over time.

The human steers. The agent builds. The compiler validates. The archive is the product.

---

## Repository Structure

```
genesis/
  README.md

  universe/                     # config the human owns — editable on a phone
    elements.toml               # element types, properties, bonding rules
    environment.toml            # grid size, depth range, pressure curve
    star.toml                   # energy output, orbital period, axis
    seeds/                      # initial condition presets
      tidal-pool.toml
      deep-ocean.toml
      volcanic-ridge.toml

  engine/
    src/
      world/                    # environment, physics, energy
        grid.rs                 # 2D grid with depth/elevation per cell
        energy.rs               # star energy input, absorption, radiation
        diffusion.rs            # element/heat transport between cells
        terrain.rs              # topology, pressure from depth, temperature
        clocks.rs               # tick hierarchy — star/environment/agent
      agents/                   # entities, bonding, interaction
        entity.rs               # entity-component model, traits not types
        bonding.rs              # rule engine, reads elements.toml
        neighborhood.rs         # local-only interaction
        tick.rs                 # agent update loop
      render/                   # observation layer — NEVER modifies state
        snapshot.rs             # captures transitions, not every tick
        curator.rs              # decides what's interesting
        renderer.rs             # isometric visual output
        narrator.rs             # reads event log, writes a story
      conservation.rs           # the jar — nothing created or destroyed
      main.rs
    Cargo.toml

  archive/                      # browsable output
    index.html                  # auto-generated, links everything
    runs/
      run-NNNN/
        dashboard.html          # fixed views every run
        events.json             # transition log
        highlights/             # curator-selected moments
        narrator.md             # the story of this run

  notebook/                     # one entry per session
    YYYY-MM-DD.md
```

---

## The Three Layers

### world/ — The Stage

The environment. A 2D grid on a rectangular slab of terrain. Each cell has:

- **Position** (x, y)
- **Elevation/depth** — a scalar. Negative = underwater, positive = above sea level. This value determines pressure, UV exposure, temperature baseline.
- **Temperature** — derived from star energy input, depth, and geothermal activity.
- **Pressure** — derived from depth. Deeper = higher pressure. Affects what chemistry is stable.
- **UV intensity** — derived from whether the cell is above water, atmosphere thickness above it, and angle to star.
- **Contents** — references to agents present in this cell.
- **Chemical concentrations** — bulk amounts of free elements and simple compounds dissolved/present in the cell.

The star is modeled simply: energy output, distance, orbital period (for day/night). The slab rotates relative to the star, creating a day/night cycle where energy input varies per cell over time. No need for full orbital mechanics — just a light source sweeping across the grid.

Terrain is static at simulation start (loaded from seed config) but can evolve slowly — volcanic events, erosion, sedimentation. These happen on the slowest clock.

### agents/ — The Actors

Everything that is not terrain is an agent. A lone element drifting in a cell is an agent. A composite of 50 bonded elements is an agent. An organism with metabolism, motility, and reproduction is an agent. They all use the same entity-component architecture.

An entity is a bag of components (traits). Possible components include:

- **Position** — where on the grid
- **Element** — for base-level agents, what element type (references elements.toml)
- **Charge** — affects bonding
- **Shape** — affects what it can bond with (bonding slots)
- **Mass** — derived from components, affects diffusion speed
- **Bonds** — references to other agents this entity is bonded to
- **Internal structure** — for composites: the list of sub-agents and their bond topology
- **Catalytic properties** — does this agent speed up specific reactions nearby
- **Motility** — can it move on its own, and how fast
- **Metabolism** — does it consume some elements/energy and produce others
- **Membrane** — does it have a boundary that contains other agents
- **Replication template** — can it copy its structure

No agent is hard-typed as "molecule" or "cell" or "organism." These are just names for different configurations of traits. A molecule is an agent with bonds and internal structure. A cell is an agent with a membrane, metabolism, and internal agents. An organism is an agent with all of the above plus motility. The labels are human interpretation, not code categories.

#### Bonding

Bonding is the core mechanic. Two adjacent agents can bond if:

1. Their shapes have compatible open bonding slots
2. The local energy conditions allow it (some bonds need energy input, some release energy)
3. The local temperature/pressure makes the bond stable

Bond evaluation reads from `elements.toml` for compatibility rules, then applies real thermodynamic equations to determine if conditions are met. The bonding rules are the most important config file in the project.

When agents bond, they form a composite agent. The composite's properties (mass, charge, catalytic behavior) are derived from its components — not assigned. If a composite happens to catalyze its own formation, that's autocatalysis. If it can template-copy itself, that's replication. Neither is coded as a special case.

#### Agent Tick

Each agent tick:

1. Check neighborhood for interaction candidates
2. Evaluate possible bonds (formation or breaking)
3. If motile, attempt movement
4. If metabolic, consume inputs and produce outputs
5. If replicating, attempt copy (imperfect — variation is guaranteed)
6. Check stability — does this agent still hold together at current temperature/pressure

### render/ — The Glass

Render observes simulation state and produces output. It never modifies state. It has no feedback path into the simulation. It could be removed entirely and the simulation would run identically.

Four sub-concerns:

- **Snapshot** — captures state at transitions, not every tick. The fossil record model: when something new appears, when something goes extinct, when a threshold is crossed. Boring stretches are skipped.
- **Curator** — examines what happened since last snapshot and decides what's interesting. Looks for: new agent types, spatial patterns, population shifts, chemistry firsts, boundary events. Selects the 2-3 most notable moments per run for highlight rendering.
- **Renderer** — produces isometric visual output. Does NOT render individual agents as labeled objects. Renders *activity and density*. Heat maps of bonding frequency. Clusters glowing by complexity. Flows of elements. Trails of movement. The visualization should look like watching bees in a hive — you see swarm behavior and patterns, not individual labeled dots.
- **Narrator** — reads the event log and writes a short narrative of what happened this run. "Bonding activity spiked near the volcanic ridge. A stable three-element composite appeared for the first time, then spread to neighboring cells. The deep trench remained inert."

---

## Configuration Architecture

### The Rule: Numbers Are Never Hardcoded in Equations

The engine has three categories of values:

1. **Universal constants** — hardcoded. Boltzmann constant, Avogadro's number, relationships between temperature and kinetic energy. These are the same in any universe.

2. **Equations** — hardcoded, sourced from real science, but fully parameterized. Diffusion rate is a function of temperature, molecular weight, and medium viscosity. Osmotic pressure follows the Van't Hoff equation. Reaction rates follow Arrhenius. The equations are real. The numbers that go into them come from config and simulation state.

3. **Element properties** — config. The alien part. These define the nouns of the simulation: what kinds of stuff exist, what they can bond with, their masses, charges, shapes.

4. **Environment conditions** — config. Grid dimensions, terrain shape, star properties, initial element distribution.

5. **Emergent values** — computed at runtime. Nobody decides these. They fall out of the equations acting on the config acting on the current state.

The test: **would this number be different on an alien planet?** If yes → config. If no → constant or equation.

### elements.toml

```toml
# The nouns of this universe. 
# Everything below is config. The bonding engine reads this.
# Shapes are abstract — think puzzle-piece slots, not molecular geometry.

[[element]]
name = "alpha"
symbol = "α"
mass = 1.0
charge = 1
shape = [1, 0, 1, 0]     # four bonding slots: open, closed, open, closed
color = "#4a90d9"

[[element]]
name = "beta"
symbol = "β"
mass = 2.5
charge = -1
shape = [0, 1, 0, 1]     # complementary to alpha
color = "#d94a4a"

# ... more elements

[[bond_rule]]
pair = ["alpha", "beta"]
energy_released = 2.0       # exothermic — happens spontaneously if adjacent
stability_max_temp = 500.0  # breaks apart above this temperature
stability_min_pressure = 0.0

[[bond_rule]]
pair = ["alpha", "alpha"]
energy_required = 5.0       # endothermic — needs energy input to form
stability_max_temp = 300.0
stability_min_pressure = 2.0  # only stable under pressure (deep water)

# Catalysis: presence of gamma near an alpha-beta pair doubles bond rate
[[catalysis_rule]]
catalyst = "gamma"
reaction = ["alpha", "beta"]
rate_multiplier = 2.0
```

### environment.toml

```toml
[grid]
width = 200
height = 100
depth_range = [-1000.0, 500.0]   # min elevation (deep trench) to max (highland)

[atmosphere]
initial_composition = { free_alpha = 0.3, free_beta = 0.5, free_gamma = 0.2 }
uv_surface_intensity = 100.0

[ocean]
depth_temperature_gradient = 0.05   # degrees per unit depth
surface_temperature = 300.0

[geothermal]
vent_count = 5
vent_energy_output = 50.0
vent_placement = "random"   # or specific coordinates
```

### star.toml

```toml
[star]
energy_output = 1000.0
distance = 1.0             # AU equivalent — scales energy received

[orbit]
period = 1000              # ticks per full day/night cycle
axial_tilt = 0.1           # affects seasonal variation if desired

[energy_model]
# Energy hitting a cell = (energy_output / distance²) * cos(angle_to_star)
# Angle changes each tick based on orbital period
# UV component scales with altitude — underwater cells get exponential dropoff
uv_water_attenuation = 0.05   # per unit depth
```

### Seed Files

A seed file defines initial conditions for a run:

```toml
# seeds/tidal-pool.toml
name = "Tidal Pool"
description = "Shallow coastal zone with strong energy input and tidal mixing"

[terrain]
# Heightmap or procedural generation params
type = "gradient"
left_depth = -200.0        # deeper ocean on left
right_depth = 50.0         # land on right
roughness = 0.3

[initial_agents]
# How many free elements to scatter
alpha = 5000
beta = 3000
gamma = 1000
delta = 500

[placement]
strategy = "density_by_depth"   # more stuff in shallows
```

---

## Clock Hierarchy

Not everything ticks at the same rate. Three clock layers:

| Clock | Ticks per cycle | What updates |
|-------|----------------|--------------|
| Stellar | 1 | Star angle, day/night, seasonal shift |
| World | ~10 | Temperature propagation, diffusion, pressure, UV |
| Agent | ~1000 | Bonding, movement, metabolism, replication |

Agent ticks are the innermost loop. A thousand agents check their neighbors and react. Then the world updates — heat diffuses, elements spread, temperature adjusts. Then (rarely) the star moves and energy input pattern shifts.

This prevents wasting compute on recalculating orbital mechanics every agent tick.

---

## Conservation Law

**Nothing is created or destroyed.** The simulation is a closed system with one input: energy from the star. The total count of each element type is constant across the entire grid for the entire run. When agents bond, the elements aren't consumed — they're part of the composite. When composites break, the elements return to the cell.

Energy enters from the star and exits as waste heat (or doesn't — it can accumulate, driving temperature up). But elements are conserved absolutely.

This constraint should be enforced programmatically — a checksum after each world tick that verifies element counts haven't changed. If they have, something is buggy. This is the single most important invariant in the codebase.

---

## Hot Spots and Dead Zones

Not all grid cells deserve equal compute. Each world tick, tag cells by activity level:

- **Hot** — agents present, bonds forming/breaking, energy flux high. Full agent tick resolution.
- **Warm** — some agents, low activity. Reduced tick frequency (every 10th agent tick).
- **Cold** — no agents, no activity. Skip entirely until diffusion brings something in.

This is critical for performance. A 200x100 grid is 20,000 cells. Most will be empty ocean. Don't waste cycles on them.

---

## The Isometric Renderer

The visual output is an isometric view of the terrain slab. The renderer should NOT draw individual agents as distinct objects (at chemistry scale, that's meaningless noise). Instead, it renders *aggregate properties per cell*:

- **Activity intensity** — how many bonding events this tick. High activity = bright/warm colors.
- **Complexity density** — average composite size in this cell. Simple free elements = dim. Large composites = bright.
- **Flow lines** — direction and magnitude of net agent movement between cells.
- **Population heat map** — where agents are concentrated.
- **Composition coloring** — dominant element type in the cell, blended.

The terrain itself renders as the isometric slab — ocean depth is visible as lower elevation, land rises up. The visualization overlays activity data onto the terrain.

As the simulation progresses into biological eras (organisms with motility, distinct morphology), the renderer can begin showing individual high-complexity agents as distinct sprites — but only when they're complex and few enough to be individually meaningful. This transition should happen naturally based on agent complexity threshold, not a hardcoded era switch.

The curator selects which region to render in detail. The dashboard always includes a global overview (flat top-down heat map of the full grid) plus the curator's chosen isometric detail view of the most interesting region.

---

## The Era Model (Emergent, Not Hardcoded)

The simulation does not switch modes. There are no coded eras. But the *character* of the simulation changes as complexity increases:

**Early** — lots of free elements, bonding events are the main activity. Visualizations show fizzy chemical activity. Hot spots near energy sources.

**Middle** — stable composites persist. Some catalyze reactions. Autocatalytic networks may appear. Visualizations show pulsing rhythmic patterns where chemical loops sustain themselves.

**Late** — self-replicating agents spread exponentially, consume free elements, compete. Variation and selection begin. Visualizations show spreading waves, territorial boundaries, population dynamics.

**Mature** — complex agents with membranes, internal structure, metabolism, motility. Visualizations show distinct creatures moving across terrain. Ecosystem dynamics emerge.

The renderer, curator, and narrator all adapt to what's actually happening — they don't check an era flag. The curator looks for transitions (first stable bond, first autocatalyst, first replicator, first motile agent) and highlights them.

---

## Design Principles

These are the mental models the codebase should embody:

1. **Traits not types.** No class hierarchy for molecules vs cells vs organisms. Entity-component only.
2. **Neighborhood is everything.** Agents interact with adjacent cells only. No action at distance.
3. **The jar.** Closed system, energy in only, elements conserved absolutely.
4. **Seeds not scripts.** Same initial conditions + same seed = same outcome. No predetermined event sequence.
5. **The fossil record.** Store transitions, not continuous state. Snapshot when something qualitatively changes.
6. **Garbage in, compost out.** Broken composites release their elements. Decay is fuel.
7. **The ratchet.** Detect irreversible phase transitions. Once free oxygen saturates the atmosphere, simplify assumptions accordingly.
8. **Pressure cooker.** Dense environments with high energy produce faster emergence than vast empty ones. Default seeds should be concentrated.
9. **The diff.** Every run's value is what changed. The curator compares against prior runs.
10. **Symmetry breaking.** The most important moments are when uniformity becomes non-uniform. Watch for and flag these.
11. **Assembly not construction.** Nothing is built top-down. Everything forms bottom-up from collision and bonding.
12. **The debugger is the visualizer.** If you can't tell what's happening from the rendered output, fix the rendering.

---

## Adding Complexity Over Time

The engine is designed to grow. New mechanisms are added by sourcing real science and coding parameterized equations. Examples of future additions, each sourced from a real topic:

- **Reaction kinetics** (Arrhenius equation) — temperature-dependent reaction rates
- **Osmotic pressure** (Van't Hoff equation) — transport across membranes
- **Diffusion** (Fick's laws) — how elements spread through medium
- **Genetic drift** (Wright-Fisher model) — population sampling effects
- **Natural selection** — differential reproduction based on fitness in environment
- **Predation** — agents consuming other agents for their component elements
- **Photosynthesis analog** — agents that convert star energy + elements into stored chemical energy
- **Atmospheric chemistry** — free elements in atmosphere reacting, producing greenhouse effects or UV shielding

Each addition follows the same pattern: real equation, parameterized, fed by element config and environment state. The equation is hardcoded. The values it operates on are not.

---

## What a Run Looks Like

1. Load config from `universe/`
2. Load seed from `universe/seeds/`
3. Initialize grid, terrain, place initial agents
4. Run the tick loop:
   - Agent ticks (inner loop, ~1000 per cycle)
   - World ticks (middle loop, ~10 per cycle)
   - Stellar ticks (outer loop, 1 per cycle)
   - Snapshot on transitions
5. After N cycles or time budget exhausted:
   - Curator selects highlights
   - Renderer produces dashboard + highlight views
   - Narrator writes the run story
   - Everything written to `archive/runs/run-NNNN/`
   - Index regenerated

---

## Initial Implementation Scope

For the first working version, implement:

- [ ] Grid with elevation/depth, temperature derived from depth
- [ ] Star energy input (simple cosine sweep for day/night)
- [ ] Elements loaded from TOML config
- [ ] Free element agents placed on grid from seed
- [ ] Bonding: adjacent agents evaluate bond rules, form composites
- [ ] Bond breaking: temperature/pressure exceeds stability
- [ ] Diffusion: free agents drift to neighboring cells (biased by concentration gradient)
- [ ] Conservation check after each world tick
- [ ] Hot spot tagging (skip cold cells)
- [ ] Basic isometric renderer: terrain + activity heat map
- [ ] Snapshot on: first bond, first 3+ composite, first catalytic event
- [ ] Dashboard: global overview + one detail view
- [ ] Event log as JSON

Do NOT implement in v1: metabolism, replication, membranes, motility, narrator, curator intelligence beyond simple thresholds. These come later as the system grows.
