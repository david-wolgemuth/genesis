# Run 0001: The Birth of Chemistry

## What Was Added

This is the first run of Genesis. Everything was built from scratch: a 2D grid with terrain, star energy input with day/night cycles, five alien elements with bonding rules, diffusion, conservation enforcement, and a minimal renderer.

The tidal pool seed creates a gradient terrain — deep ocean on the left, shallow coast in the middle, land on the right. 4,500 agents (2,000 alpha, 1,500 beta, 500 gamma, 300 delta, 200 epsilon) were scattered with a density bias toward the shallows.

## What Happened

Chemistry ignited almost immediately. The first bond formed at tick 100 — an alpha-gamma pair near the coast at position (67, 2). From there, bonding spread rapidly.

By tick 1,000, 942 bonds had formed and 1,878 agents (42%) were bonded. The rate of new bond formation decreased over time as free elements became scarcer — the classic signature of a bimolecular reaction approaching equilibrium.

By the end of the run (tick 5,000):
- **1,255 bonds** had formed
- **2,502 agents** (55.6%) were bonded
- **0 bonds** had broken
- **Conservation held perfectly** throughout

## What Was Expected vs. What Happened

**Expected:** Bonds forming preferentially near geothermal vents and on the day-lit side of the grid, with deep ocean remaining relatively inert.

**Observed:** Bonding happened everywhere agents were present. The lack of bond breaking (0 broken) suggests that once bonds formed, conditions never exceeded stability thresholds. This makes sense — the tidal pool terrain is shallow, pressures are moderate, and temperatures stay within the stability ranges of most bond rules.

**Expected:** Catalytic events where gamma accelerates alpha-beta bonding.

**Observed:** No catalytic events were detected. This is likely because the catalysis check requires the catalyst, the first reactant, and the second reactant to all be in the same cell simultaneously — a three-body coincidence that requires higher local density than the current placement provides.

## Comparison

No previous run exists. This is the baseline.

## What to Explore Next

1. **Bond breaking** — Currently no bonds break because temperatures stay below stability limits. Adding Arrhenius-style temperature-dependent reaction rates would make bonding more dynamic, with bonds forming and breaking based on local thermal fluctuations.

2. **Catalysis activation** — The three-body coincidence required for catalysis is too rare at current densities. Either increase agent density, reduce grid size, or implement a "neighborhood catalysis" model where catalysts in adjacent cells also count.

3. **Diffusion tuning** — Agents diffuse but slowly. Increasing diffusion rates or adding convection currents near vents would improve mixing and create more interesting spatial dynamics.

## Highlights

- [First Bonds](./highlights/first-bonds.html) — The first chemical bond in this universe: alpha and gamma joined at the coast.
