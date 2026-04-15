# Genesis Runbook: Autonomous PR Loop

This document describes how an autonomous agent operates on the Genesis simulation. Each cycle produces one pull request. The human's only verbs are **merge** and **close**. Everything inside a PR is unsupervised.

---

## The Golden Rule

**Research before code. Opinion before orientation.**

You will form your understanding of the topic from the real world *before* you look at the codebase. This is intentional. You should arrive at the code with a point of view about what matters, how the mechanism works, and what it should produce — not reverse-engineer your opinion from what's already implemented.

---

## The PR Loop

A single pull request follows these steps in this exact order:

### 1. Pick

Read `tree.org` in the repo root. This is the concept tree — an org-mode file tracking what's been explored, what's implemented, what's shallow, what's blocked, and what's undiscovered.

Choose what to work on using this distribution:

- **70%** — the highest-priority gap. What does the simulation most obviously need next?
- **20%** — a random `:shallow:` node. Something that's implemented but thin. Deepen it.
- **10%** — pure discovery. Don't implement anything. Just research, add nodes to the tree, expand the map. The PR contains only research and tree updates. This is a valid, complete PR.

Before picking, zoom out. Re-read the root-level nodes and ask: "what is the overall arc, and where are the biggest structural gaps?" Do not rat-hole into one branch for many consecutive cycles.

Write a one-sentence summary of what you chose and why. This becomes the PR title.

### 2. Research

**Use web search.** This is not optional. Do not rely on your training data alone.

Search for the topic you picked. Read Wikipedia articles, academic summaries, educational resources. You are looking for:

- **The mechanism** — what actually happens, physically or chemically or biologically
- **The equation** — the mathematical relationship that governs it. Arrhenius, Fick's law, Van't Hoff, Wright-Fisher, whatever applies.
- **The parameters** — what inputs the equation needs, what it produces, what units, what ranges are realistic
- **The boundary conditions** — when does this mechanism matter? Under what temperatures, pressures, concentrations? When is it negligible?
- **The observable effects** — if this mechanism is working, what would you see? What would change in the simulation's behavior?

Produce a research document: `archive/runs/run-NNNN/research.md`. This is the first artifact of the PR. It should be written in plain language, include the equation(s), cite sources with links, and end with a section titled **"What This Means for the Simulation"** that translates the science into concrete expectations for what should change when implemented.

**Do not look at the codebase yet.** Do not read the existing Rust code. Do not check what's already implemented. Your research should be independent of the current state of the engine.

### 3. Orient

Now read the codebase. Specifically:

- `tree.org` — the concept tree (you read the top-level in Pick, now read the relevant branch in detail)
- The prime at the top of `tree.org` — a few lines summarizing the last cycle
- The previous run's output in `archive/runs/run-NNNN-1/` — what did the simulation produce last time?
- The relevant source files in `engine/src/` — what exists today?
- `universe/elements.toml` and other config files — what are the current rules?

The gap between what your research says should happen and what the simulation currently does is the work. Name that gap explicitly: "The research says reaction rates should be temperature-dependent. Currently, bonding probability is flat regardless of temperature. I need to add Arrhenius scaling to the bond evaluation in `bonding.rs`."

### 4. Implement

Now write code. This is the tight inner loop:

1. Edit the relevant source files
2. Compile (`cargo build`)
3. If compiler errors, read them, fix them, go to 2
4. Run quick smoke test (`cargo test` or a short simulation run)
5. If tests fail, fix and go to 2
6. If it compiles and passes smoke tests, proceed

**Rules:**

- Follow the architecture in SPEC.md. World, agents, and render are separate. Render never modifies state. Conservation law is enforced.
- New equations are **parameterized**. Hardcode universal constants. Everything else comes from config or simulation state.
- If the mechanism needs a new config field, add it to the appropriate TOML file with a sensible default and a comment explaining what it is.
- If you add a new entity component, it should be a trait that any agent can have, not a type-specific field.
- If you cannot compile after 10 attempts, stop. Revert all code changes. Mark the node as `:blocked:` in `tree.org` with a note explaining what went wrong. The PR is still valid — it contains the research, the attempt, and the failure record. Push it.

### 5. Run

Execute the simulation at full scale:

- Use the default seed (or the most appropriate seed for the mechanism being tested)
- Run for the full time budget
- Run with at least 2 different random seeds to verify behavior is consistent
- Enforce the time budget — kill the process if it exceeds the limit

Collect all output:

- Visual renders (isometric views, heat maps)
- Event log (`events.json`)
- Population/agent statistics
- Conservation check results

If the simulation panics or hangs, go back to step 4. If it crashes on all seeds, treat it as a failed implementation — revert, mark `:blocked:`, document why.

### 6. Compare

Load the previous run's output from `archive/runs/run-NNNN-1/`. Diff against the current run:

- **What changed?** — new agent types, different spatial patterns, different activity levels, different population curves
- **Did the new mechanism activate?** — if you added temperature-dependent bonding, did bonding rates actually change in hot vs cold regions? If nothing changed, the mechanism may need different conditions or parameter ranges. Note this.
- **Did anything unexpected happen?** — emergent behavior you didn't predict. This is the most interesting category.
- **Did anything break?** — behavior that was working before that stopped. Regressions.
- **Conservation check** — did element counts stay constant? If not, there's a bug. Go back to step 4.

### 7. Curate

Produce a **camera script** (`archive/runs/run-NNNN/camera.json`). This is a list of keyframes that tell the Three.js viewer where to point the camera and when. Each keyframe targets an interesting moment in the run.

For each notable moment, create a keyframe with: the frame number, the camera position, the look-at target, and a short note explaining what's happening. The viewer will interpolate smoothly between keyframes and display the notes as annotations.

Select criteria for keyframes — look for:

- **Firsts** — first stable bond, first composite of N+ elements, first autocatalytic loop, first replicator, first motile agent
- **Spatial patterns** — clustering, boundary formation, migration, territory
- **Phase transitions** — sudden shifts in behavior, population explosions or crashes
- **Symmetry breaking** — something that was uniform becoming non-uniform
- **The unexpected** — anything you didn't predict in your research step

If nothing interesting happened, produce a minimal camera script (just an overview). Not every run has dramatic moments. But note in the narrative *why* nothing interesting happened — that's useful data for the next cycle.

### 8. Narrate

Write `archive/runs/run-NNNN/narrator.md`. This is not a code changelog. It's the story of what happened in the simulation this cycle, written like a field researcher's log:

- What mechanism was added and what the science says it should do
- What actually happened when the simulation ran
- What was expected vs surprising
- What the comparison to the previous run revealed
- What the agent recommends exploring next and why
- Any connections noticed — "this mechanism might interact with X which isn't implemented yet"

This document should be readable by someone who has never seen the codebase. It references the camera script keyframes by frame number so the reader can scrub the viewer to the relevant moments.

### 9. Record

Update the project records:

**`tree.org`** — update tags on the node you worked on. `:todo:` → `:implemented:`, or `:shallow:` → `:implemented:`, or add `:blocked:` with a note. If your research discovered related topics, add 1-3 new nodes as `:discovered:`. Do not add more than 3 nodes per cycle. Do not rewrite the tree structure. Update the prime (the lines at the top) to summarize this cycle for the next agent's Orient step.

**`notebook/YYYY-MM-DD.md`** — a notebook entry. Unlike the narrator (which tells the simulation's story), the notebook tells the *process* story. What went well, what was hard, what the borrow checker caught, what you'd do differently. This is the meta-record.

**Regenerate `archive/index.html`** — add links to the new run's artifacts.

### 10. Package

The pull request should contain:

**Changed files:**
- Engine source code (if implementation succeeded)
- Config files (if new parameters were added)
- `tree.org` (always — even discovery-only PRs update the tree)

**New files:**
- `archive/runs/run-NNNN/research.md`
- `archive/runs/run-NNNN/frames/*.json` (frame snapshots)
- `archive/runs/run-NNNN/camera.json` (curator's camera script)
- `archive/runs/run-NNNN/narrator.md`
- `archive/runs/run-NNNN/events.json`
- `notebook/YYYY-MM-DD.md`
- Updated `archive/index.html`

**PR description:**

Short. A few sentences with relative links. The human reads this on their phone and decides whether to merge. It should tell them:

1. What you worked on (one sentence)
2. What happened (one sentence)
3. The most interesting thing (one sentence with a link to the viewer for this run)
4. What you'd explore next (one sentence)

Example:

> **Added temperature-dependent reaction rates (Arrhenius equation)**
>
> Bonding rates now scale with temperature. The volcanic ridge became the most active region — the camera script zooms in at frame 340. The deep trench went nearly inert, consistent with low thermal energy. [Watch the run](../../viewer/viewer.html?run=run-0012) · [Full narrative](./archive/runs/run-0012/narrator.md) · [Research notes](./archive/runs/run-0012/research.md)
>
> Recommends exploring diffusion (Fick's laws) next — currently elements don't spread between cells, so the ridge activity is isolated.

---

## Failure Modes and What To Do

### Can't compile after 10 attempts
Revert all code changes. PR contains: research, tree update with `:blocked:` tag and failure notes, notebook entry explaining what went wrong. This is a valid PR. Merge it — the research and the failure record have value.

### Simulation panics or hangs
Go back to Implement. If it still fails after 3 full Build→Run cycles, treat as compilation failure above. Revert and document.

### Nothing interesting happened
Ship it anyway. Frame data, narrator explaining why nothing changed, notebook entry. Sometimes the mechanism needs a different environment to activate. Note that in the narrative: "Diffusion was implemented but with current element density, concentration gradients are too shallow to produce visible effects. A denser seed or smaller grid might show more activity."

### Conservation check fails
This is a bug. Do not proceed. Go back to Implement and fix it. Conservation is the one invariant that can never be broken. If you can't fix it, revert.

### The mechanism doesn't match the research
This is fine. The simulation runs alien chemistry — real equations with alien elements. If the research says "small populations lose diversity faster" and the simulation shows the opposite, document the discrepancy. It might be a bug, or it might be that the alien element properties create different dynamics. Note it, don't force the outcome.

---

## What You Are Not Doing

- **You are not writing all the code from scratch each cycle.** You are incrementally adding to an existing, growing codebase.
- **You are not deciding what the simulation produces.** You implement mechanisms. The simulation produces whatever emerges.
- **You are not rendering visuals.** The engine produces JSON frame data. The Three.js viewer handles all rendering. Do not generate SVGs, HTML dashboards, or images in Rust.
- **You are not hardcoding numbers.** Every mechanism is an equation with parameters from config. The only hardcoded numbers are universal physical constants.
- **You are not optimizing for speed.** The simulation should be compute-rich, not artificially fast. If a run takes 10 minutes of genuine computation, that's good.
- **You are not skipping the research step.** Even if you already know about genetic drift or osmotic pressure from training data, search the web, read current sources, cite them. Your training data may be wrong or outdated. The research document is an artifact with value.

---

## Cycle Numbering and Run Numbering

- **Cycle** = one PR. Numbered sequentially. Cycle 1 is the first PR ever opened.
- **Run** = one simulation execution. A cycle may contain multiple runs (comparison runs, different seeds). Run numbers are global and sequential.
- The run directory is `archive/runs/run-NNNN/` where NNNN is zero-padded.
- The notebook entry uses the date, not the cycle number: `notebook/2026-04-13.md`. Multiple cycles on the same day append a suffix: `notebook/2026-04-13-b.md`.

---

## Meta Parameters

These are set by the human, not the agent. Respect them:

| Parameter | Where Set | What It Controls |
|-----------|-----------|-----------------|
| Time budget per run | CI config / env var | Max wall-clock seconds before kill |
| Max compile attempts | This runbook (10) | When to give up and revert |
| Max revision loops | This runbook (3) | Build→Run cycles before declaring failure |
| Crawl depth | tree.org convention | How many links deep to follow from seed sources |
| Min highlight threshold | Agent judgment | Curator's bar for what's interesting — develops over time |

---

## The First PR

The very first cycle is special. There is no previous run to compare against. There is no existing codebase to orient on. The first PR should:

1. **Research** the seed topic (whatever the initial concept tree's top-priority node is)
2. **Scaffold** the engine from SPEC.md — the grid, the element loader, basic bonding, the conservation check, the frame serializer
3. **Build the viewer** — a Three.js viewer.html that loads frame data and renders the terrain with activity overlays, with play/pause/scrub and orbit controls
4. **Run** with the default seed — probably just elements bouncing around and occasionally bonding
5. **Curate** whatever happens — produce a camera script that zooms to the first bond site
6. **Narrate** the birth of the simulation

The first PR's description: "Initial engine and viewer. Elements bond near energy sources. [Watch the run](../../viewer/viewer.html?run=run-0001)."

From there, every subsequent PR adds one mechanism, one layer of complexity, one step up the emergence ladder.
