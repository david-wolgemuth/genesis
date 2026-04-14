use crate::agents::entity::Agent;
use crate::agents::tick::TickStats;
use crate::render::snapshot::EventLog;
use crate::world::grid::Grid;
use std::collections::HashMap;

/// Generate an isometric terrain view with activity heat map as an SVG string.
pub fn render_isometric(grid: &Grid, agents: &HashMap<u64, Agent>) -> String {
    let cell_w = 8.0_f64;
    let cell_h = 4.0_f64;
    let offset_x = (grid.height as f64) * cell_w / 2.0 + 20.0;
    let offset_y = 20.0;
    let svg_width = ((grid.width + grid.height) as f64 * cell_w / 2.0 + 40.0) as usize;
    let svg_height = ((grid.width + grid.height) as f64 * cell_h / 2.0 + 100.0) as usize;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        svg_width, svg_height, svg_width, svg_height
    );
    svg.push_str(r##"<rect width="100%" height="100%" fill="#0a0a1a"/>"##);

    // Draw cells in isometric order (back to front)
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = grid.cell(x, y);
            let iso_x = offset_x + (x as f64 - y as f64) * cell_w / 2.0;
            let iso_y = offset_y + (x as f64 + y as f64) * cell_h / 2.0;

            // Elevation offset (higher cells drawn higher)
            let elev_offset = -(cell.elevation / 50.0).clamp(-3.0, 3.0);

            // Color based on activity and contents
            let agent_count = cell.agent_ids.len();
            let color = if agent_count == 0 {
                cell_base_color(cell.elevation)
            } else {
                activity_color(agent_count, cell.energy_budget)
            };

            // Draw isometric diamond
            let cx = iso_x;
            let cy = iso_y + elev_offset;
            svg.push_str(&format!(
                r#"<polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" fill="{}" opacity="0.85"/>"#,
                cx, cy - cell_h / 2.0,
                cx + cell_w / 2.0, cy,
                cx, cy + cell_h / 2.0,
                cx - cell_w / 2.0, cy,
                color
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

fn cell_base_color(elevation: f64) -> &'static str {
    if elevation < -100.0 {
        "#0a1628" // deep ocean
    } else if elevation < -30.0 {
        "#0f2040" // ocean
    } else if elevation < 0.0 {
        "#1a3050" // shallows
    } else if elevation < 30.0 {
        "#2a3020" // coast
    } else {
        "#3a4030" // highland
    }
}

fn activity_color(agent_count: usize, energy: f64) -> &'static str {
    let intensity = agent_count as f64 + energy / 10.0;
    if intensity > 20.0 {
        "#ff4444" // very hot
    } else if intensity > 10.0 {
        "#ff8844"
    } else if intensity > 5.0 {
        "#ffcc44"
    } else if intensity > 2.0 {
        "#44aaff"
    } else {
        "#2266aa"
    }
}

/// Generate the global overview heat map (flat top-down, not isometric).
pub fn render_heatmap(grid: &Grid, agents: &HashMap<u64, Agent>) -> String {
    let scale = 6;
    let w = grid.width * scale;
    let h = grid.height * scale;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
        w, h
    );
    svg.push_str(&format!(r##"<rect width="{}" height="{}" fill="#0a0a1a"/>"##, w, h));

    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = grid.cell(x, y);
            let count = cell.agent_ids.len();
            if count == 0 && cell.energy_budget < 1.0 {
                continue; // skip empty cells for smaller SVG
            }
            let color = activity_color(count, cell.energy_budget);
            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.8"/>"#,
                x * scale, y * scale, scale, scale, color
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

/// Generate dashboard HTML with population charts and overview.
pub fn render_dashboard(
    grid: &Grid,
    agents: &HashMap<u64, Agent>,
    stats: &TickStats,
    event_log: &EventLog,
    run_name: &str,
) -> String {
    let total_agents = agents.len();
    let bonded = agents.values().filter(|a| !a.bonds.is_empty()).count();
    let free = total_agents - bonded;

    // Element counts
    let mut elem_counts: HashMap<String, usize> = HashMap::new();
    for agent in agents.values() {
        if let Some(name) = agent.element_name() {
            *elem_counts.entry(name.to_string()).or_insert(0) += 1;
        }
    }
    let mut elem_list: Vec<_> = elem_counts.into_iter().collect();
    elem_list.sort_by(|a, b| b.1.cmp(&a.1));

    let heatmap = render_heatmap(grid, agents);
    let isometric = render_isometric(grid, agents);

    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Genesis — {run_name}</title>
<style>
body {{ background: #0a0a1a; color: #c0c0c0; font-family: 'Courier New', monospace; margin: 2em; }}
h1 {{ color: #4a90d9; }}
h2 {{ color: #888; border-bottom: 1px solid #333; padding-bottom: 0.3em; }}
.stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1em; margin: 1em 0; }}
.stat {{ background: #111; padding: 1em; border: 1px solid #222; }}
.stat .label {{ color: #666; font-size: 0.8em; text-transform: uppercase; }}
.stat .value {{ color: #4a90d9; font-size: 1.5em; margin-top: 0.3em; }}
.views {{ display: grid; grid-template-columns: 1fr 1fr; gap: 2em; margin: 2em 0; }}
.view {{ background: #111; padding: 1em; border: 1px solid #222; overflow: auto; }}
.view h3 {{ color: #888; margin-top: 0; }}
.elements {{ list-style: none; padding: 0; }}
.elements li {{ padding: 0.3em 0; border-bottom: 1px solid #1a1a1a; }}
.bar {{ display: inline-block; height: 12px; margin-right: 8px; vertical-align: middle; }}
.events {{ max-height: 400px; overflow-y: auto; font-size: 0.85em; }}
.event {{ padding: 0.4em; border-bottom: 1px solid #1a1a1a; }}
.event .tick {{ color: #4a90d9; }}
</style>
</head>
<body>
<h1>Genesis — {run_name}</h1>

<div class="stats">
<div class="stat"><div class="label">Total Agents</div><div class="value">{total_agents}</div></div>
<div class="stat"><div class="label">Free</div><div class="value">{free}</div></div>
<div class="stat"><div class="label">Bonded</div><div class="value">{bonded}</div></div>
<div class="stat"><div class="label">Bonds Formed</div><div class="value">{}</div></div>
<div class="stat"><div class="label">Bonds Broken</div><div class="value">{}</div></div>
</div>

<h2>Element Distribution</h2>
<ul class="elements">
"#,
        stats.bonds_formed,
        stats.bonds_broken
    );

    let max_count = elem_list.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let colors = ["#4a90d9", "#d94a4a", "#4ad94a", "#d9d94a", "#9a4ad9"];
    for (i, (name, count)) in elem_list.iter().enumerate() {
        let bar_width = (*count as f64 / max_count as f64 * 200.0) as usize;
        let color = colors[i % colors.len()];
        html.push_str(&format!(
            r#"<li><span class="bar" style="width: {}px; background: {};"></span>{}: {}</li>"#,
            bar_width, color, name, count
        ));
    }

    html.push_str(r#"</ul>

<h2>Views</h2>
<div class="views">
<div class="view">
<h3>Global Heat Map</h3>
"#);
    html.push_str(&heatmap);
    html.push_str(r#"
</div>
<div class="view">
<h3>Isometric Terrain</h3>
"#);
    html.push_str(&isometric);
    html.push_str(r#"
</div>
</div>

<h2>Event Log</h2>
<div class="events">
"#);

    for event in &event_log.events {
        let desc = match event {
            crate::render::snapshot::Event::FirstBond { tick, elements, .. } => {
                format!("<span class=\"tick\">t={}</span> First bond: {} + {}", tick, elements[0], elements[1])
            }
            crate::render::snapshot::Event::FirstComposite3Plus { tick, size, elements, .. } => {
                format!("<span class=\"tick\">t={}</span> First {}-element composite: {:?}", tick, size, elements)
            }
            crate::render::snapshot::Event::FirstCatalysis { tick, catalyst, reaction, .. } => {
                format!("<span class=\"tick\">t={}</span> First catalysis: {} catalyzed {} + {}", tick, catalyst, reaction[0], reaction[1])
            }
            crate::render::snapshot::Event::BondCountMilestone { tick, count } => {
                format!("<span class=\"tick\">t={}</span> Bond milestone: {} total bonds", tick, count)
            }
            crate::render::snapshot::Event::PopulationSnapshot { tick, free_agents, bonded_agents, total_bonds, .. } => {
                format!("<span class=\"tick\">t={}</span> Population: {} free, {} bonded, {} bonds", tick, free_agents, bonded_agents, total_bonds)
            }
            crate::render::snapshot::Event::SimulationEnd { tick, total_bonds_formed, total_bonds_broken, conservation_ok } => {
                format!("<span class=\"tick\">t={}</span> End: {} formed, {} broken, conservation: {}", tick, total_bonds_formed, total_bonds_broken, if *conservation_ok { "OK" } else { "FAILED" })
            }
        };
        html.push_str(&format!(r#"<div class="event">{}</div>"#, desc));
    }

    html.push_str(r#"
</div>
</body>
</html>"#);

    html
}

/// Generate a highlight page for a specific event.
pub fn render_highlight(
    grid: &Grid,
    agents: &HashMap<u64, Agent>,
    title: &str,
    description: &str,
    event_tick: u64,
) -> String {
    let isometric = render_isometric(grid, agents);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Genesis — {title}</title>
<style>
body {{ background: #0a0a1a; color: #c0c0c0; font-family: 'Courier New', monospace; margin: 2em; }}
h1 {{ color: #4a90d9; }}
.meta {{ color: #666; margin-bottom: 2em; }}
.description {{ font-size: 1.1em; line-height: 1.6; margin: 1em 0; }}
.view {{ background: #111; padding: 1em; border: 1px solid #222; margin: 2em 0; display: inline-block; }}
</style>
</head>
<body>
<h1>{title}</h1>
<div class="meta">Tick {event_tick}</div>
<div class="description">{description}</div>
<div class="view">
{isometric}
</div>
</body>
</html>"#
    )
}
