# Genesis

An emergent simulation of alien chemistry and life. Elements combine into molecules, molecules into complex structures, structures into self-replicating organisms — governed by real physics on an alien planet.

The planet's elements are invented. The equations are real.

## What It Produces

A growing archive of visual artifacts — isometric terrain views, activity heat maps, population charts, and narrated accounts of what the simulation discovered each cycle. Browse them in `archive/`.

## How It Works

The simulation is a Rust program that reads element definitions and environment config from `universe/`, runs agents on a grid, and outputs artifacts to `archive/`. Each cycle adds one real scientific mechanism (diffusion, reaction kinetics, osmotic pressure, natural selection...) sourced from actual research.

## Quick Start

```bash
cd engine && cargo run -- --seed ../universe/seeds/tidal-pool.toml
```

## Project Structure

```
universe/       # config you edit — elements, environment, star, seeds
engine/         # Rust simulation — world, agents, render
archive/        # visual output — browse this
notebook/       # process journal — one entry per cycle
```

## Docs

- **`SPEC.md`** — architecture, mental models, design principles
- **`RUNBOOK.md`** — the autonomous PR workflow
- **`AGENTS.md`** — quick-start guide for AI agents working on this project
- **`tree.org`** — concept tree tracking what's explored and 
what's next
