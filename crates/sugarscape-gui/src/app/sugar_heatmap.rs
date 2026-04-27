use eframe::egui::{self, Color32, Pos2, Rect, Sense, StrokeKind, Vec2, Vec2b};
use egui_plot::{Heatmap, Plot};

use sugarscape_gui::StepData;

use super::plot_common::{grid_map_bounds, plot_data_aspect, MAP_PLOT_HEIGHT};

/// Blue-only heatmap stops: low values → pale blue, high values → deep blue.
const HEATMAP_BASE: [Color32; 10] = [
    Color32::from_rgb(235, 245, 255),
    Color32::from_rgb(210, 230, 252),
    Color32::from_rgb(180, 215, 248),
    Color32::from_rgb(145, 195, 240),
    Color32::from_rgb(110, 175, 230),
    Color32::from_rgb(80, 150, 220),
    Color32::from_rgb(55, 125, 205),
    Color32::from_rgb(40, 100, 185),
    Color32::from_rgb(28, 75, 155),
    Color32::from_rgb(18, 50, 120),
];

/// Fixed upper end of the sugar heatmap color scale (cell level is capped by capacity in the sim).
const HEATMAP_LEVEL_MAX: f64 = 10.0;

fn interpolate_palette(base_colors: &[Color32], resolution: usize) -> Vec<Color32> {
    let mut interpolated = vec![Color32::TRANSPARENT; resolution];
    if base_colors.is_empty() || resolution == 0 {
        return interpolated;
    }
    if base_colors.len() == 1 || resolution == 1 {
        return vec![base_colors[0]; resolution];
    }
    for (i, color) in interpolated.iter_mut().enumerate() {
        let i_rel = i as f64 / (resolution - 1) as f64;
        if i_rel >= 1.0 {
            *color = base_colors[base_colors.len() - 1];
        } else {
            let base_index_float = i_rel * (base_colors.len() - 1) as f64;
            let base_index = base_index_float as usize;
            let start_color = base_colors[base_index];
            let end_color = base_colors[base_index + 1];
            let gradient_level = base_index_float - base_index as f64;

            let delta_r = (end_color.r() as f64 - start_color.r() as f64) * gradient_level;
            let delta_g = (end_color.g() as f64 - start_color.g() as f64) * gradient_level;
            let delta_b = (end_color.b() as f64 - start_color.b() as f64) * gradient_level;

            let r = (start_color.r() as f64 + delta_r).round() as u8;
            let g = (start_color.g() as f64 + delta_g).round() as u8;
            let b = (start_color.b() as f64 + delta_b).round() as u8;
            *color = Color32::from_rgb(r, g, b);
        }
    }
    interpolated
}

fn color_for_normalized_value(v_rel: f64, palette: &[Color32]) -> Color32 {
    if palette.is_empty() {
        return Color32::GRAY;
    }
    let v_rel = v_rel.clamp(0.0, 1.0);
    let idx = (v_rel * (palette.len() - 1) as f64).round() as usize;
    palette[idx.min(palette.len() - 1)]
}

pub fn show_sugar_levels(ui: &mut egui::Ui, data: &StepData, grid_width: usize, grid_height: usize) {
    if data.levels.is_empty() {
        ui.label("No sugar data for this step.");
        return;
    }

    let cells = grid_width.saturating_mul(grid_height);
    let mut values = vec![0.0_f64; cells];
    for i in 0..data.rows.len() {
        let r = data.rows[i] as usize;
        let c = data.cols[i] as usize;
        if r < grid_height && c < grid_width {
            values[r * grid_width + c] = data.levels[i] as f64;
        }
    }

    let z_max = HEATMAP_LEVEL_MAX.max(1e-6);

    let palette = interpolate_palette(&HEATMAP_BASE, 128);

    let heatmap = Heatmap::new(values, grid_width)
        .palette(&HEATMAP_BASE)
        .range(0.0, z_max)
        .show_labels(false)
        .name("Sugar");

    let cbar_total_w = 52.0;
    let gap = 8.0;
    let plot_w = (ui.available_width() - cbar_total_w - gap).max(160.0);
    let aspect = plot_data_aspect(grid_width, grid_height);

    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(plot_w, MAP_PLOT_HEIGHT),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                Plot::new("sugar_heatmap")
                    .height(MAP_PLOT_HEIGHT)
                    .width(plot_w)
                    .data_aspect(aspect)
                    .auto_bounds(Vec2b::FALSE)
                    .default_x_bounds(0.0, grid_width as f64)
                    .default_y_bounds(0.0, grid_height as f64)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .allow_drag(false)
                    .allow_axis_zoom_drag(false)
                    .allow_boxed_zoom(false)
                    .allow_double_click_reset(false)
                    .set_margin_fraction(Vec2::splat(0.02))
                    .show_axes(Vec2b::FALSE)
                    .show_grid(Vec2b::FALSE)
                    .show(ui, |plot_ui| {
                        plot_ui.set_auto_bounds(Vec2b::FALSE);
                        plot_ui.set_plot_bounds(grid_map_bounds(grid_width, grid_height));
                        plot_ui.heatmap(heatmap);
                    });
            },
        );

        ui.allocate_ui_with_layout(
            Vec2::new(cbar_total_w, MAP_PLOT_HEIGHT),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.label(egui::RichText::new(format!("{z_max:.2}")).monospace());
                let bar_h = MAP_PLOT_HEIGHT - 36.0;
                let (rect, _) = ui.allocate_exact_size(Vec2::new(22.0, bar_h), Sense::hover());
                let painter = ui.painter_at(rect);
                const STRIPS: usize = 64;
                for i in 0..STRIPS {
                    let t = 1.0 - (i as f64 + 0.5) / STRIPS as f64;
                    let color = color_for_normalized_value(t, &palette);
                    let y0 = rect.top() + rect.height() * i as f32 / STRIPS as f32;
                    let y1 = rect.top() + rect.height() * (i + 1) as f32 / STRIPS as f32;
                    painter.rect_filled(
                        Rect::from_min_max(Pos2::new(rect.left(), y0), Pos2::new(rect.right(), y1)),
                        0.0,
                        color,
                    );
                }
                painter.rect_stroke(
                    rect,
                    0.0,
                    ui.visuals().widgets.noninteractive.bg_stroke,
                    StrokeKind::Inside,
                );
                ui.label(egui::RichText::new("0.00").monospace());
            },
        );
    });
}
