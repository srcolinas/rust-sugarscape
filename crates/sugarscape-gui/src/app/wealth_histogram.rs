use eframe::egui;
use egui_plot::{Bar, BarChart, Plot};

/// Bars and chart accents (blue family).
const HISTOGRAM_BLUE: egui::Color32 = egui::Color32::from_rgb(45, 110, 200);

fn bin_width_for_plot(wmin: f32, wmax: f32, bins: usize) -> f64 {
    if wmin >= wmax {
        (wmax.abs() * 0.05 + 0.5) as f64
    } else {
        ((wmax - wmin) / bins as f32) as f64 * 0.9
    }
}

pub fn show_wealth_distribution(ui: &mut egui::Ui, wealths: &[f32]) {
    const BINS: usize = 32;

    if wealths.is_empty() {
        ui.label("No wealth data for this step.");
        return;
    }

    let wmin = wealths.iter().copied().fold(f32::INFINITY, f32::min);
    let wmax = wealths.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    let bars: Vec<Bar> = if wmin >= wmax {
        vec![Bar::new(wmin as f64, wealths.len() as f64)]
    } else {
        let span = wmax - wmin;
        let mut counts = vec![0u32; BINS];
        for w in wealths {
            let t = ((*w - wmin) / span).clamp(0.0, 1.0);
            let idx = (t * BINS as f32).floor() as usize;
            let idx = idx.min(BINS - 1);
            counts[idx] += 1;
        }
        let bin_w = span / BINS as f32;
        counts
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let center = wmin + bin_w * (i as f32 + 0.5);
                Bar::new(center as f64, c as f64)
            })
            .collect()
    };

    Plot::new("wealth_distribution")
        .width(ui.available_width())
        .height(220.0)
        .x_axis_label("Wealth")
        .y_axis_label("Agents")
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(
                BarChart::new("count", bars)
                    .width(bin_width_for_plot(wmin, wmax, BINS))
                    .color(HISTOGRAM_BLUE),
            );
        });
}
