use std::path::PathBuf;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::Frame;
use ratatui_plt::prelude::*;
use ratatui_plt::series::GridData;

use sugarscape_viz::{ReaderError, SimulationDataReader, Step, StepData};

#[derive(Parser, Debug)]
#[command(
    name = "sugarscape-viz",
    about = "Read simulation Parquet stats (visualization hook)"
)]
struct Cli {
    /// Parquet file produced by sugarscape-sim
    #[arg(long, value_name = "FILE")]
    input: PathBuf,

    /// Update interval in milliseconds
    #[arg(long, value_name = "MS", default_value = "1000")]
    update_interval: u64,

    /// Number of rows to render
    #[arg(long, value_name = "N", default_value = "50")]
    rows: usize,

    /// Number of columns to render
    #[arg(long, value_name = "N", default_value = "50")]
    cols: usize,

    /// Maximum wealth to render in histogram
    #[arg(long, value_name = "W", default_value = "35.0")]
    max_wealth: f64,

    /// Number of bins to render in histogram
    #[arg(long, value_name = "N", default_value = "20")]
    bins: usize,

    /// Maximun sugar capacity
    #[arg(long, value_name = "C", default_value = "4.0")]
    max_capacity: f64,

    /// Number of agents to render in histogram
    #[arg(long, value_name = "N", default_value = "20")]
    num_agents: usize,
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    let mut reader = SimulationDataReader::new(&cli.input)?;
    let (tx, rx) = mpsc::channel::<Result<(Step, StepData), ReaderError>>();
    thread::spawn(move || loop {
        match reader.next() {
            None => break,
            Some(Ok(step_data)) => {
                if tx.send(Ok(step_data)).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(cli.update_interval));
            }
            Some(Err(e)) => {
                let _ = tx.send(Err(e));
                break;
            }
        }
    });

    ratatui::run(move |terminal| -> Result<()> {
        loop {
            match rx.try_recv() {
                Ok(Ok((step, data))) => {
                    terminal.draw(|frame| {
                        render(
                            frame,
                            step,
                            &data,
                            cli.rows,
                            cli.cols,
                            cli.max_wealth,
                            cli.bins,
                            cli.max_capacity,
                            cli.num_agents,
                        )
                    })?;
                }
                Ok(Err(e)) => return Err(e.into()),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => return Ok(()),
            }
            if event::poll(Duration::from_millis(50))? && event::read()?.is_key_press() {
                return Ok(());
            }
        }
    })?;

    Ok(())
}

fn render(
    frame: &mut Frame,
    step: Step,
    data: &StepData,
    grid_rows: usize,
    grid_cols: usize,
    max_wealth: f64,
    bins: usize,
    max_capacity: f64,
    num_agents: usize,
) {
    let [header, distributions, histogram, footer] = frame.area().layout(
        &Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .spacing(1),
    );
    let [levels, ages] =
        distributions.layout(&Layout::horizontal([Constraint::Percentage(50); 2]).spacing(1));

    render_title(frame, header);
    render_heatmap_of_levels(frame, levels, data, grid_rows, grid_cols, max_capacity);
    render_scatter_plot_of_agents(frame, ages, data);
    render_histogram_of_wealth(frame, histogram, data, max_wealth, num_agents, bins);
    render_step_number(frame, footer, step);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Line::from_iter([
        Span::from("Sugarscape Visualization").bold(),
        Span::from(" (Press any q to quit, press p to pause)"),
    ]);
    frame.render_widget(title.centered(), area);
}

fn render_heatmap_of_levels(
    frame: &mut Frame,
    area: Rect,
    data: &StepData,
    grid_rows: usize,
    grid_cols: usize,
    max_capacity: f64,
) {
    // Heatmap::render always builds the colorbar from data.value_bounds(), not from `.norm()`,
    // so we draw the colorbar ourselves with the same range as LinearNorm.
    let [plot_area, cbar_area] =
        area.layout(&Layout::horizontal([Constraint::Min(1), Constraint::Length(12)]).spacing(1));

    let xs: Vec<f64> = (0..grid_cols).map(|c| c as f64).collect();
    let ys: Vec<f64> = (0..grid_rows).map(|r| r as f64).collect();
    let mut values = vec![vec![0.0_f64; grid_cols]; grid_rows];
    for i in 0..data.rows.len() {
        let r = data.rows[i] as usize;
        let c = data.cols[i] as usize;
        if r < grid_rows && c < grid_cols {
            values[r][c] = data.levels[i] as f64;
        }
    }
    let grid = GridData::new(xs, ys, values);
    let no_ticks = Axis::new().locator(NullLocator);
    let heatmap = Heatmap::new(grid)
        .colormap(Viridis)
        .norm(LinearNorm::new(0.0, max_capacity))
        .title("Sugar levels")
        .aspect_ratio(AspectRatio::Equal)
        .x_axis(no_ticks.clone())
        .y_axis(no_ticks)
        .show_colorbar(false);
    frame.render_widget(&heatmap, plot_area);

    let colorbar = Colorbar::new(&Viridis, 0.0, max_capacity);
    frame.render_widget(&colorbar, cbar_area);
}

fn render_scatter_plot_of_agents(frame: &mut Frame, area: Rect, data: &StepData) {
    let [plot_area, cbar_area] =
        area.layout(&Layout::horizontal([Constraint::Min(1), Constraint::Length(12)]).spacing(1));

    let points: Vec<(f64, f64)> = (0..data.rows.len())
        .map(|i| (data.rows[i] as f64, data.cols[i] as f64))
        .collect();
    let color_vals: Vec<f64> = data.ages.iter().map(|&a| a as f64).collect();

    let no_ticks = Axis::new().locator(NullLocator);
    let scatter = {
        let base = ScatterPlot::new()
            .series(
                Series::new("agents")
                    .data(points)
                    .marker(MarkerShape::Square),
            )
            .colormap(Viridis)
            .title("Agents (color = age)")
            .aspect_ratio(AspectRatio::Equal)
            .x_axis(no_ticks.clone())
            .y_axis(no_ticks)
            .show_legend(false);
        if color_vals.is_empty() {
            base
        } else {
            base.color_values(color_vals)
        }
    };
    frame.render_widget(&scatter, plot_area);

    let (cbar_lo, cbar_hi) = if data.ages.is_empty() {
        (0.0, 1.0)
    } else {
        let mn = *data.ages.iter().min().expect("ages non-empty") as f64;
        let mx = *data.ages.iter().max().expect("ages non-empty") as f64;
        (mn, mx)
    };
    let colorbar = Colorbar::new(&Viridis, cbar_lo, cbar_hi);
    frame.render_widget(&colorbar, cbar_area);
}

fn render_histogram_of_wealth(
    frame: &mut Frame,
    area: Rect,
    data: &StepData,
    max_wealth: f64,
    num_agents: usize,
    bins: usize,
) {
    // Histogram::render_single overwrites manual y bounds with max(count)*1.1,
    // so the axis always tracks the data. StairsPlot + baseline respects
    // Manual bounds and clips bar fill to the plot via PlotArea::contains.
    let n_bins = bins.max(1);
    let lo = 0.0;
    let hi = max_wealth;
    let bin_width = (hi - lo) / n_bins as f64;
    let edges: Vec<f64> = (0..=n_bins).map(|i| lo + i as f64 * bin_width).collect();
    let mut counts = vec![0.0_f64; n_bins];
    for &w in &data.wealths {
        let v = w as f64;
        if v.is_finite() && v >= lo && v <= hi {
            let idx = ((v - lo) / bin_width).floor() as usize;
            let idx = idx.min(n_bins - 1);
            counts[idx] += 1.0;
        }
    }
    let y_max = num_agents as f64;
    let stairs = StairsPlot::new()
        .dataset(StairsDataset::new(
            "count",
            edges,
            counts,
            ratatui::style::Color::Cyan,
        ))
        .title("Wealth distribution")
        .baseline(0.0)
        .x_axis(Axis::new().bounds(Bounds::Manual(lo, hi)))
        .y_axis(Axis::new().bounds(Bounds::Manual(0.0, y_max)))
        .show_legend(false);
    frame.render_widget(&stairs, area);
}

fn render_step_number(frame: &mut Frame, area: Rect, step: Step) {
    let step = Line::from_iter([Span::from(format!("Step: {step}"))]);
    frame.render_widget(step.left_aligned(), area);
}
