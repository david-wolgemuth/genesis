# Research: Emergent Chemistry and the Serialize/Viewer Architecture

**Run 0002 · Tidal Pool · Seed 42 · 2026-04-15**

---

## Topic

This cycle's work was structural rather than scientific: aligning the engine with the spec's
data/visualization separation. The `render/` module produced SVGs and HTML dashboards directly
from Rust, violating the "Data, not pixels" design principle. The refactoring replaces it with
a `serialize/` module that produces JSON frame data, and introduces a Three.js viewer that
handles all rendering at browse-time.

The scientific question driving the *content* of this run: **what are the baseline bonding
dynamics across different environmental conditions?** Three seeds were run and compared.

---

## The Mechanism: Thermodynamic Bond Stability

Bonding in this simulation follows real thermodynamic constraints, simplified:

**Exothermic bonds** (energy_released > 0): release energy on formation. Spontaneous if
shape-compatible and within stability temperature/pressure bounds. These form freely wherever
two compatible elements share a cell.

**Endothermic bonds** (energy_required > 0): require energy input from the environment to
form. Only possible in cells with sufficient energy budget (star energy + geothermal).

**Stability bounds**: every bond has a `stability_max_temp` (breaks above this temperature)
and `stability_min_pressure` (breaks below this pressure). These are read from `elements.toml`.

Sources:
- Atkins, P. & de Paula, J., *Physical Chemistry* — thermodynamic spontaneity (ΔG < 0)
- Mizushima, S. (1954) — bond stability as function of temperature and pressure
- NIST WebBook — bond dissociation energies and temperature dependence

---

## Equations Governing This System

**Bond formation probability** (current implementation):
```
P(bond) = min(0.1 × catalysis_multiplier, 1.0)
```
This is a flat probability — not yet temperature-dependent. The Arrhenius equation should
govern this in a future cycle:
```
k = A × exp(−Eₐ / RT)
```
where Eₐ is activation energy, R is the gas constant, T is local temperature.

**UV attenuation underwater** (Beer-Lambert law):
```
E_cell = E_surface × exp(−α × depth)
```
where α = `uv_water_attenuation` (0.05 per depth unit).

---

## What This Means for the Simulation

### Tidal Pool (Run 0002)

4,500 agents, gradient terrain from −200m (left) to +50m (right), 3 geothermal vents.
Agents biased toward the right (shallows), where star energy is highest.

Expected: high bonding activity near the coast, where alpha and beta elements concentrate
and UV energy is strongest. Endothermic bonds possible near surface.

Observed:
- **1,213 bonds formed, 0 broken** over 5,000 ticks
- First bond at tick 100. Hit 10 bonds in the same tick — early burst suggests many
  elements were already co-located when conditions became favorable.
- First catalysis at tick 103 (gamma catalyzing alpha+beta)
- Final state: **53.7% of agents bonded** (2,417 of 4,500)
- Bond formation slowed significantly after tick 2,000 — most accessible pairs already bonded

### Deep Ocean (Run 0003)

3,500 agents, uniform deep terrain −300m to −400m, uniform placement.

Expected: fewer bonds overall. High pressure helps pressure-dependent bonds (alpha-alpha at
min 2.0 atm, beta-beta at 1.0 atm), but low UV suppresses endothermic bond formation.
Geothermal vents are the only significant energy source.

Observed:
- **307 bonds formed, 0 broken** — 4× fewer bonds than Tidal Pool
- First bond and first catalysis both at tick 100 (same world tick)
- Final state: only **17.5% of agents bonded** (613 of 3,500)
- Plateau reached much earlier — system chemistry exhausted quickly

The deep ocean bond suppression is striking. The attenuation formula
`exp(−0.05 × 350) ≈ 0.000002` — at 350m depth, UV energy is essentially zero.
Only geothermal vents drove endothermic bonds, and those vent cells are few.
Most of the 307 bonds were exothermic (no energy required).

### Volcanic Ridge (Run 0004)

6,000 agents, high-roughness gradient creating varied elevations, uniform placement.

Expected: intermediate results. High roughness means some cells are shallow (high UV,
high temperature) and some are very deep (high pressure, low UV). Should show spatial
heterogeneity in bonding — active zones near surface-level cells.

Observed:
- **1,088 bonds formed, 0 broken** despite having 33% more agents than Tidal Pool
- Bond rate *per agent* was only 18% vs 27% in Tidal Pool
- Hit 10 and 50 bond milestones in the same tick (burst of 50+ bonds in a single tick!)
- The burst suggests many gamma+delta pairs (energy_released=4.0, stability_max_temp=600)
  were co-located at simulation start

---

## Key Finding: Zero Bond Breaking

Across all three runs, **no bonds were ever broken**. This is not a bug — it reflects the
current temperature/pressure regime:

- Bonds form only when conditions are within stability bounds
- Temperature is reset from baseline each world tick (depth-derived), keeping it stable
- Geothermal heating adds at most ~25K above surface temperature
- The stability_max_temp for most bonds is 300–700K, and surface temperature is 300K
- Deep cells run 200–295K, comfortably below all stability thresholds
- Pressure never drops below the initial depth-derived value

**Consequence**: the bond pool is monotonically growing. Every bond formed persists forever.
This makes the system unrealistically stable — real chemistry has bond-breaking and
reformation. The next cycle should add temperature dynamics that actually exceed stability
thresholds: either through higher geothermal output, or through Arrhenius-based feedback
where exothermic bonds locally heat cells, making those cells hostile to further bonding.

---

## What This Means for the Simulation (Summary)

| Seed | Agents | Bonds | % Bonded | Bond/Agent | Catalysis? |
|------|--------|-------|----------|------------|------------|
| Tidal Pool | 4,500 | 1,213 | 53.7% | 27% | Yes (tick 103) |
| Deep Ocean | 3,500 | 307 | 17.5% | 8.8% | Yes (tick 100) |
| Volcanic Ridge | 6,000 | 1,088 | 36.1% | 18% | Yes (tick 101) |

The environment shapes chemistry strongly. Shallow energy-rich zones (Tidal Pool) produce
3× the bond density of deep low-energy zones (Deep Ocean). This is the expected behavior
from the star energy model and is a good sign the physics is working correctly.

**Recommended next**: Arrhenius temperature-dependent reaction rates. Currently bond
probability is a flat 10%. Replacing this with `k = A × exp(−Eₐ / RT)` would make the
volcanic ridge dramatically more active and create real spatial differentiation.
