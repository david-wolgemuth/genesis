# Genesis Viewer

A Three.js viewer that loads frame data produced by the Rust engine and animates it as a 3D heightmap terrain.

## Usage

Open `viewer.html?run=run-NNNN` in a browser (requires a local HTTP server due to fetch() cross-origin restrictions).

Quick local server:
```
cd genesis
python3 -m http.server 8080
# then open: http://localhost:8080/viewer/viewer.html?run=run-0001
```

## Controls

| Input | Action |
|-------|--------|
| Drag | Orbit camera |
| Scroll | Zoom |
| Space / Play button | Play/pause animation |
| ← → arrow keys | Step frame-by-frame |
| Scrubber | Jump to any frame |
| G / Guided button | Toggle guided camera |

## What you're looking at

**Terrain** — The grid rendered as a 3D heightmap. Deep ocean trenches are geometrically low. Coastal shelves are middle. Land rises up. Depth is literal: you can see the terrain shape at a glance.

**Cell color** — Blended from the dominant element type in each cell. Element colors come from `elements.toml` (embedded in `manifest.json`). Empty cells are colored by terrain type (dark blue = deep, lighter = shallow, green = land).

**Brightness** — Activity intensity. High bonding activity = bright. Inert cells = dim.

**White glow** — Presence of bonded agents. Cells with composites have a slight white overlay.

## Camera script

The curator produces `camera.json` alongside the frame data. The viewer follows this script by default — it's a guided tour of the interesting moments (first bond, catalysis events, milestones). Annotations appear at keyframes.

Grab the camera (drag) or press G at any time to switch to free orbit mode.

## Data format

The viewer loads:
- `manifest.json` — run metadata, element colors, frame file list
- `frames/frame-NNNN.json` — one per snapshot interval (grid state at that tick)
- `camera.json` — keyframe list for guided tour

The engine produces these files. The viewer renders them. They never mix.
