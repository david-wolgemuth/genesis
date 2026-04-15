use crate::serialize::snapshot::Event;
use serde::Serialize;

/// A single camera keyframe: at this frame index, position the camera here.
#[derive(Debug, Serialize)]
pub struct CameraKeyframe {
    /// Frame index (index into the frames array, not tick number).
    pub frame: usize,
    /// Camera world position [x, y, z].
    pub camera: [f64; 3],
    /// Look-at target [x, y, z].
    pub target: [f64; 3],
    /// Human-readable note displayed as annotation.
    pub note: String,
}

/// The camera script: an ordered list of keyframes for the viewer to follow.
#[derive(Debug, Serialize)]
pub struct CameraScript {
    pub keyframes: Vec<CameraKeyframe>,
}

impl CameraScript {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Run manifest written alongside frames so the viewer knows what to load.
#[derive(Debug, Serialize)]
pub struct RunManifest {
    pub run: String,
    pub seed: String,
    pub total_ticks: u64,
    pub total_agents: usize,
    /// Colors for each element by name (from elements.toml).
    pub element_colors: std::collections::HashMap<String, String>,
    /// Relative paths to frame files, in order.
    pub frames: Vec<String>,
}

impl RunManifest {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Examine events and produce a camera script highlighting the most interesting moments.
///
/// The curator selects up to 5 keyframes:
/// - Always: opening overview (frame 0)
/// - If present: first bond location
/// - If present: first catalysis location
/// - If present: a bond milestone (50 or 100 bonds)
/// - Always: closing overview (last frame)
pub fn curate(
    events: &[Event],
    total_frames: usize,
    grid_width: usize,
    grid_height: usize,
) -> CameraScript {
    let center_x = grid_width as f64 / 2.0;
    let center_z = grid_height as f64 / 2.0;

    // Overview camera: positioned above and back from the terrain center
    let overview_cam = [center_x, center_x.max(center_z) * 1.5, center_z + center_x];
    let overview_target = [center_x, 0.0, center_z];

    let last_frame = total_frames.saturating_sub(1);

    let mut keyframes = vec![CameraKeyframe {
        frame: 0,
        camera: overview_cam,
        target: overview_target,
        note: "Opening — full terrain overview".to_string(),
    }];

    // Track which events have been encoded as keyframes
    let mut bond_keyframe_added = false;
    let mut catalysis_keyframe_added = false;
    let mut milestone_keyframe_added = false;

    // Compute total ticks for frame→index mapping
    // We'll use a simple heuristic: events have a tick, frames are evenly spaced
    // We need to find which frame index corresponds to an event tick.
    // Since we don't have the exact tick→frame mapping here, we approximate:
    // frame_index ≈ (event_tick / total_ticks) * total_frames

    // Find the simulation end event to know total ticks
    let total_ticks = events
        .iter()
        .find_map(|e| {
            if let Event::SimulationEnd { tick, .. } = e {
                Some(*tick)
            } else {
                None
            }
        })
        .unwrap_or(1);

    let tick_to_frame = |tick: u64| -> usize {
        let frac = tick as f64 / total_ticks as f64;
        ((frac * total_frames as f64) as usize).min(last_frame)
    };

    for event in events {
        match event {
            Event::FirstBond { tick, x, y, elements } if !bond_keyframe_added => {
                bond_keyframe_added = true;
                let fx = *x as f64;
                let fz = *y as f64;
                keyframes.push(CameraKeyframe {
                    frame: tick_to_frame(*tick),
                    // Zoom in from above the bond site
                    camera: [fx + 10.0, 20.0, fz + 15.0],
                    target: [fx, 0.0, fz],
                    note: format!(
                        "First bond at ({}, {}) — {} + {} join for the first time",
                        x, y, elements[0], elements[1]
                    ),
                });
            }
            Event::FirstCatalysis { tick, x, y, catalyst, reaction } if !catalysis_keyframe_added => {
                catalysis_keyframe_added = true;
                let fx = *x as f64;
                let fz = *y as f64;
                keyframes.push(CameraKeyframe {
                    frame: tick_to_frame(*tick),
                    camera: [fx + 12.0, 25.0, fz + 12.0],
                    target: [fx, 0.0, fz],
                    note: format!(
                        "First catalysis at ({}, {}) — {} accelerates {} + {} bonding",
                        x, y, catalyst, reaction[0], reaction[1]
                    ),
                });
            }
            Event::BondCountMilestone { tick, count }
                if (*count == 50 || *count == 100) && !milestone_keyframe_added =>
            {
                milestone_keyframe_added = true;
                keyframes.push(CameraKeyframe {
                    frame: tick_to_frame(*tick),
                    camera: overview_cam,
                    target: overview_target,
                    note: format!(
                        "Bond milestone: {} total bonds — chemistry is spreading",
                        count
                    ),
                });
            }
            _ => {}
        }
    }

    // Closing keyframe: pull back to overview
    if last_frame > 0 {
        keyframes.push(CameraKeyframe {
            frame: last_frame,
            camera: overview_cam,
            target: overview_target,
            note: "Final state — full terrain overview".to_string(),
        });
    }

    // Sort keyframes by frame index so they're in temporal order
    keyframes.sort_by_key(|kf| kf.frame);

    CameraScript { keyframes }
}
