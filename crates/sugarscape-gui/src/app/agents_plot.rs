use eframe::egui::{self, Color32, Vec2, Vec2b};
use egui_plot::{MarkerShape, Plot, PlotPoints, Points};

use sugarscape_gui::StepData;

use super::plot_common::{grid_map_bounds, plot_data_aspect, MAP_PLOT_HEIGHT};

/// Older agents use deeper blue (`t` in \[0, 1\] from youngest to oldest).
fn agent_blue_for_age_normalized(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = (195.0 - 175.0 * t) as u8;
    let g = (225.0 - 165.0 * t) as u8;
    let b = (255.0 - 115.0 * t) as u8;
    Color32::from_rgb(r, g, b)
}

pub fn show_agents(ui: &mut egui::Ui, data: &StepData, grid_width: usize, grid_height: usize) {
    if data.rows.is_empty() {
        ui.label("No agent data for this step.");
        return;
    }

    let age_max = data.ages.iter().copied().max().unwrap_or(1).max(1) as f32;

    const BINS: usize = 40;
    let mut bins: Vec<Vec<[f64; 2]>> = vec![Vec::new(); BINS];
    for i in 0..data.rows.len() {
        let x = data.cols[i] as f64 + 0.5;
        let y = data.rows[i] as f64 + 0.5;
        let t = (data.ages[i] as f32 / age_max).clamp(0.0, 1.0);
        let bi = ((BINS - 1) as f32 * t).round() as usize;
        let bi = bi.min(BINS - 1);
        bins[bi].push([x, y]);
    }

    let mut grid_centers = Vec::with_capacity(grid_width.saturating_mul(grid_height));
    for row in 0..grid_height {
        for col in 0..grid_width {
            grid_centers.push([col as f64 + 0.5, row as f64 + 0.5]);
        }
    }

    let plot_w = ui.available_width().max(160.0);
    let aspect = plot_data_aspect(grid_width, grid_height);

    Plot::new("agents_plot")
        .height(MAP_PLOT_HEIGHT)
        .width(plot_w)
        .auto_bounds(Vec2b::FALSE)
        .default_x_bounds(0.0, grid_width as f64)
        .default_y_bounds(0.0, grid_height as f64)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_drag(false)
        .allow_axis_zoom_drag(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .data_aspect(aspect)
        .set_margin_fraction(Vec2::splat(0.02))
        .show_axes(Vec2b::FALSE)
        .show_grid(Vec2b::FALSE)
        .show(ui, |plot_ui| {
            plot_ui.set_auto_bounds(Vec2b::FALSE);
            plot_ui.set_plot_bounds(grid_map_bounds(grid_width, grid_height));

            let dark = plot_ui.ctx().global_style().visuals.dark_mode;
            let grid_dot = if dark {
                Color32::from_rgba_unmultiplied(120, 160, 220, 55)
            } else {
                Color32::from_rgba_unmultiplied(100, 140, 200, 45)
            };
            plot_ui.points(
                Points::new("cell_centers", PlotPoints::from(grid_centers))
                    .shape(MarkerShape::Circle)
                    .radius(1.2)
                    .color(grid_dot),
            );

            for (bi, pts) in bins.into_iter().enumerate() {
                if pts.is_empty() {
                    continue;
                }
                let t = (bi as f32 + 0.5) / BINS as f32;
                let color = agent_blue_for_age_normalized(t);
                plot_ui.points(
                    Points::new(format!("agents_{bi}"), PlotPoints::from(pts))
                        .shape(MarkerShape::Circle)
                        .radius(4.5)
                        .color(color),
                );
            }
        });
}
