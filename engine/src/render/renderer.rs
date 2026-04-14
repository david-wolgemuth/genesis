use crate::agents::entity::Agent;
use crate::agents::tick::TickStats;
use crate::render::snapshot::EventLog;
use crate::world::grid::Grid;
use std::collections::HashMap;

/// Escape special HTML characters in dynamic text to prevent injection.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Generate an isometric terrain view with activity heat map as an SVG string.
pub fn render_isometric(grid: &Grid) -> String {
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
pub fn render_heatmap(grid: &Grid) -> String {
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

/// Load a template file from the templates/ directory (compile-time embed).
const DASHBOARD_TEMPLATE: &str = include_str!("../../templates/dashboard.html");
const HIGHLIGHT_TEMPLATE: &str = include_str!("../../templates/highlight.html");

/// Format an event as an HTML snippet for the event log.
fn format_event(event: &crate::render::snapshot::Event) -> String {
    match event {
        crate::render::snapshot::Event::FirstBond { tick, elements, .. } => {
            format!("<span class=\"tick\">t={}</span> First bond: {} + {}", tick, html_escape(&elements[0]), html_escape(&elements[1]))
        }
        crate::render::snapshot::Event::FirstComposite3Plus { tick, size, elements, .. } => {
            format!("<span class=\"tick\">t={}</span> First {}-element composite: {:?}", tick, size, elements)
        }
        crate::render::snapshot::Event::FirstCatalysis { tick, catalyst, reaction, .. } => {
            format!("<span class=\"tick\">t={}</span> First catalysis: {} catalyzed {} + {}", tick, html_escape(catalyst), html_escape(&reaction[0]), html_escape(&reaction[1]))
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
    }
}

/// Generate dashboard HTML with population charts and overview.
/// Uses the dashboard.html template from engine/templates/.
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

    // Build element list HTML
    let max_count = elem_list.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let colors = ["#4a90d9", "#d94a4a", "#4ad94a", "#d9d94a", "#9a4ad9"];
    let mut elements_html = String::new();
    for (i, (name, count)) in elem_list.iter().enumerate() {
        let bar_width = (*count as f64 / max_count as f64 * 200.0) as usize;
        let color = colors[i % colors.len()];
        elements_html.push_str(&format!(
            r#"<li><span class="bar" style="width: {}px; background: {};"></span>{}: {}</li>"#,
            bar_width, color, html_escape(name), count
        ));
    }

    // Build events HTML
    let mut events_html = String::new();
    for event in &event_log.events {
        events_html.push_str(&format!(
            "<div class=\"event\">{}</div>\n",
            format_event(event)
        ));
    }

    DASHBOARD_TEMPLATE
        .replace("{{run_name}}", &html_escape(run_name))
        .replace("{{total_agents}}", &total_agents.to_string())
        .replace("{{free}}", &free.to_string())
        .replace("{{bonded}}", &bonded.to_string())
        .replace("{{bonds_formed}}", &stats.bonds_formed.to_string())
        .replace("{{bonds_broken}}", &stats.bonds_broken.to_string())
        .replace("{{elements}}", &elements_html)
        .replace("{{heatmap}}", &render_heatmap(grid))
        .replace("{{isometric}}", &render_isometric(grid))
        .replace("{{events}}", &events_html)
}

/// Generate a highlight page for a specific event.
/// Uses the highlight.html template from engine/templates/.
pub fn render_highlight(
    grid: &Grid,
    title: &str,
    description: &str,
    event_tick: u64,
) -> String {
    HIGHLIGHT_TEMPLATE
        .replace("{{title}}", &html_escape(title))
        .replace("{{event_tick}}", &event_tick.to_string())
        .replace("{{description}}", &html_escape(description))
        .replace("{{isometric}}", &render_isometric(grid))
}
