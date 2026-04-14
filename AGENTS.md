# Agent Guide

You are working on **Genesis** — an emergent simulation that models elements combining into molecules, molecules into complex structures, and eventually into self-replicating organisms on an alien planet.

## Read First

- **`SPEC.md`** — the architecture, the mental models, the config format, the file tree. Read this before writing any code.
- **`RUNBOOK.md`** — the autonomous PR workflow. Read this if you are operating unsupervised.
- **`tree.org`** — the concept tree. This is the living map of what's been explored, what's implemented, and what's next.
- **`universe/`** — the config files. These define the physics of this particular planet. The human owns these.

## Core Principles

1. **Research before code.** Understand the real science before looking at the codebase. Use web search. Cite sources.
2. **Equations, not magic numbers.** Every mechanism is a real, parameterized equation. Hardcode only universal constants.
3. **Traits, not types.** Agents are bags of components. There is no molecule class or organism class.
4. **Conservation is sacred.** Elements are never created or destroyed. If the checksum fails, stop and fix it.
5. **Render never modifies state.** The `render/` layer observes. It has no feedback path into the simulation.
6. **The human steers.** You build, the human decides direction. Don't reorganize priorities without being asked.

## When In Doubt

- If you're unsure what to work on → read `tree.org`, pick the obvious gap.
- If you're unsure how something should work → search the web for the real science.
- If you can't get it compiling → revert and document what went wrong. That's still progress.
- If the output looks wrong → the debugger is the visualizer. Fix the rendering before assuming the simulation is b
roken.
