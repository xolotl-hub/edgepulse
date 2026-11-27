use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::engine::EdgePulseState;
use crate::scale;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedPlotPoint {
    pub x: f64,
    pub y: f64,
    pub gap: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedPlot {
    pub id: String,
    pub title: String,
    pub color: String,
    #[serde(rename = "lineWidth")]
    pub line_width: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane: Option<String>,
    pub points: Vec<ProjectedPlotPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedHLine {
    pub id: String,
    pub y: f64,
    pub title: String,
    pub color: String,
    pub style: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedBox {
    pub id: String,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub color: String,
    #[serde(rename = "fillColor")]
    pub fill_color: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedMarker {
    pub id: String,
    pub x: f64,
    pub y: f64,
    #[serde(rename = "type")]
    pub kind: String,
    pub label: String,
    pub color: String,
    #[serde(rename = "showPointer")]
    pub show_pointer: bool,
    #[serde(rename = "pillY")]
    pub pill_y: f64,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedOverlays {
    pub plots: Vec<ProjectedPlot>,
    pub hlines: Vec<ProjectedHLine>,
    pub boxes: Vec<ProjectedBox>,
    pub markers: Vec<ProjectedMarker>,
}

/// Proyecta, recorta al viewport (culling) y resuelve anti-colisión vertical de marcadores en Rust
pub fn project_overlays(state: &EdgePulseState) -> ProjectedOverlays {
    let Some(overlays) = &state.overlays else {
        return ProjectedOverlays::default();
    };
    let vp = &state.viewport;
    let plot_w = vp.plot_width();
    let plot_h = vp.plot_height();
    if plot_w <= 0.0 || plot_h <= 0.0 || state.candles.is_empty() {
        return ProjectedOverlays::default();
    }

    let mut out = ProjectedOverlays::default();

    // 1. Plots con culling horizontal + decimación LOD en zoom lejano (bar_spacing < 2.0)
    let use_lod = vp.bar_spacing < 2.0;
    for plot in &overlays.plots {
        let is_bottom_pane = plot.pane.as_deref() == Some("bottom");
        let mut points = Vec::new();
        let mut pending_gap = true;
        let mut last_kept_x: Option<f64> = None;

        for (idx, pt) in plot.data.iter().enumerate() {
            let Some(val) = pt.value else {
                pending_gap = true;
                last_kept_x = None;
                continue;
            };
            let Some(x) = state.time_to_coordinate(pt.time) else {
                pending_gap = true;
                last_kept_x = None;
                continue;
            };
            let y = if is_bottom_pane { val } else { scale::price_to_y(vp, val) };
            if !x.is_finite() || !y.is_finite() {
                pending_gap = true;
                last_kept_x = None;
                continue;
            }
            if x < -vp.bar_spacing * 2.0 || x > plot_w + vp.bar_spacing * 2.0 {
                pending_gap = true;
                last_kept_x = None;
                continue;
            }

            let is_last = idx + 1 == plot.data.len();
            if use_lod && !pending_gap && !is_last {
                if let Some(prev_x) = last_kept_x {
                    if (x - prev_x).abs() < 0.75 {
                        continue;
                    }
                }
            }

            points.push(ProjectedPlotPoint {
                x,
                y,
                gap: pending_gap,
                value: if is_bottom_pane { Some(val) } else { None },
            });
            last_kept_x = Some(x);
            pending_gap = false;
        }
        if !points.is_empty() {
            out.plots.push(ProjectedPlot {
                id: plot.id.clone(),
                title: plot.title.clone(),
                color: plot.color.clone(),
                line_width: plot.line_width.unwrap_or(1.8),
                style: plot.style.clone(),
                pane: plot.pane.clone(),
                points,
            });
        }
    }

    // 2. HLines con culling vertical
    for hl in &overlays.hlines {
        let y = scale::price_to_y(vp, hl.price).round() + 0.5;
        if !y.is_finite() || y < 0.0 || y > plot_h {
            continue;
        }
        out.hlines.push(ProjectedHLine {
            id: hl.id.clone(),
            y,
            title: hl.title.clone(),
            color: hl.color.clone(),
            style: hl.style.clone().unwrap_or_else(|| "dashed".to_string()),
        });
    }

    // 3. Boxes con culling 2D
    for b in &overlays.boxes {
        let (Some(x1), Some(x2)) = (
            state.time_to_coordinate(b.time_start),
            state.time_to_coordinate(b.time_end),
        ) else {
            continue;
        };
        let y_top = scale::price_to_y(vp, b.price_top);
        let y_bot = scale::price_to_y(vp, b.price_bottom);
        if !x1.is_finite() || !x2.is_finite() || !y_top.is_finite() || !y_bot.is_finite() {
            continue;
        }
        let left = x1.min(x2);
        let width = (x2 - x1).abs();
        let top = y_top.min(y_bot);
        let height = (y_bot - y_top).abs();

        if left + width < 0.0 || left > plot_w || top + height < 0.0 || top > plot_h {
            continue;
        }

        out.boxes.push(ProjectedBox {
            id: b.id.clone(),
            left,
            top,
            width,
            height,
            color: b.color.clone(),
            fill_color: b.fill_color.clone(),
            title: b.title.clone(),
        });
    }

    // 4. Markers con culling y anti-colisión vertical por (time, lado)
    let mut stack_slots: HashMap<(i64, bool), usize> = HashMap::new();
    let pill_h = 15.0;
    let stack_step = 18.0;

    for m in &overlays.markers {
        let Some(x) = state.time_to_coordinate(m.time) else {
            continue;
        };
        let y = scale::price_to_y(vp, m.price);
        if !x.is_finite() || !y.is_finite() || x < 0.0 || x > plot_w {
            continue;
        }
        let is_buy = m.kind == "buy";
        let key = (m.time.round() as i64, is_buy);
        let slot = *stack_slots.get(&key).unwrap_or(&0);
        stack_slots.insert(key, slot + 1);

        let raw_pill_y = if is_buy {
            y + 16.0 + (slot as f64) * stack_step
        } else {
            y - 31.0 - (slot as f64) * stack_step
        };
        let pill_y = raw_pill_y.clamp(2.0, (plot_h - pill_h - 2.0).max(2.0));

        out.markers.push(ProjectedMarker {
            id: m.id.clone(),
            x,
            y,
            kind: m.kind.clone(),
            label: m.label.clone(),
            color: m.color.clone(),
            show_pointer: slot == 0,
            pill_y,
        });
    }

    out
}
