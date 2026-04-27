mod agents_plot;
mod plot_common;
mod sugar_heatmap;
mod wealth_histogram;

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rand::random;

use eframe::egui::{self, Context};

use sugarscape_gui::{SimulationDataReader, Step, StepData};
use sugarscape_sim::{parse_config_yaml, run_simulation};

use agents_plot::show_agents;
use sugar_heatmap::show_sugar_levels;
use wealth_histogram::show_wealth_distribution;

/// Example config from the `sugarscape-sim` crate (`config.example.yaml`), baked in for the editor.
const CONFIG_EXAMPLE_YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../sugarscape-sim/config.example.yaml"
));

const PARQUET_BUFFER_LIMIT: usize = 1000;

pub struct SugarscapeApp {
    reader: Option<SimulationDataReader>,
    grid_width: usize,
    grid_height: usize,
    paused: bool,
    step: Step,
    data: Option<StepData>,
    side_file_content: String,
    show_config_panel: bool,
    run_save_feedback: String,
    run_rx: Option<mpsc::Receiver<Result<(PathBuf, usize, usize), String>>>,
}

impl SugarscapeApp {
    pub fn new() -> Self {
        let (grid_width, grid_height) = parse_config_yaml(CONFIG_EXAMPLE_YAML)
            .map(|c| (c.world.width as usize, c.world.height as usize))
            .unwrap_or((50, 50));
        Self {
            reader: None,
            grid_width,
            grid_height,
            paused: false,
            step: 0,
            data: None,
            side_file_content: CONFIG_EXAMPLE_YAML.to_owned(),
            show_config_panel: true,
            run_save_feedback: "Run the simulation to generate Parquet and visualize it."
                .to_owned(),
            run_rx: None,
        }
    }

    fn save_config_run(&mut self, ctx: &Context) {
        if self.run_rx.is_some() {
            self.run_save_feedback = "A run is already in progress.".to_owned();
            return;
        }

        let config = match parse_config_yaml(&self.side_file_content) {
            Ok(c) => c,
            Err(e) => {
                self.run_save_feedback = format!("Invalid config: {e}");
                return;
            }
        };

        let dir = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
        let name = format!("sugarscape-run-{:016x}.yaml", random::<u64>());
        let yaml_path = dir.join(name);
        if let Err(e) = std::fs::write(&yaml_path, self.side_file_content.as_bytes()) {
            self.run_save_feedback = format!("Save failed: {e}");
            return;
        }

        let parquet_path = yaml_path.with_extension("parquet");
        let parquet_display = parquet_path.display().to_string();
        let grid_w = config.world.width as usize;
        let grid_h = config.world.height as usize;

        let (tx, rx) = mpsc::channel();
        self.run_rx = Some(rx);
        thread::spawn(move || {
            let path = parquet_path;
            let result = run_simulation(config, &path, PARQUET_BUFFER_LIMIT)
                .map(|()| (path, grid_w, grid_h));
            let _ = tx.send(result.map_err(|e| e.to_string()));
        });

        self.run_save_feedback = format!(
            "Running… config saved to {} (output: {parquet_display})",
            yaml_path.display(),
        );
        ctx.request_repaint();
    }

    fn apply_run_success(&mut self, parquet_path: PathBuf, grid_w: usize, grid_h: usize) {
        self.grid_width = grid_w;
        self.grid_height = grid_h;
        match SimulationDataReader::new(&parquet_path) {
            Ok(mut reader) => match reader.next() {
                Some(Ok((step, data))) => {
                    self.reader = Some(reader);
                    self.step = step;
                    self.data = Some(data);
                    self.run_save_feedback = format!("Loaded {}", parquet_path.display());
                }
                Some(Err(e)) => {
                    self.reader = None;
                    self.data = None;
                    self.run_save_feedback = format!("Read error: {e}");
                }
                None => {
                    self.reader = None;
                    self.data = None;
                    self.run_save_feedback = "Parquet contained no steps.".to_owned();
                }
            },
            Err(e) => {
                self.reader = None;
                self.data = None;
                self.run_save_feedback = format!("Failed to open Parquet: {e}");
            }
        }
    }
}

impl eframe::App for SugarscapeApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = self.run_rx.as_ref() {
            match rx.try_recv() {
                Ok(Ok((path, gw, gh))) => {
                    self.run_rx = None;
                    self.apply_run_success(path, gw, gh);
                    ctx.request_repaint();
                }
                Ok(Err(msg)) => {
                    self.run_rx = None;
                    self.run_save_feedback = msg;
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.run_rx = None;
                    self.run_save_feedback = "Simulation thread ended unexpectedly.".to_owned();
                    ctx.request_repaint();
                }
            }
        }

        if !self.paused {
            if let Some(reader) = self.reader.as_mut() {
                match reader.next() {
                    Some(Ok((step, data))) => {
                        self.step = step;
                        self.data = Some(data);
                    }
                    Some(Err(e)) => {
                        eprintln!("Error reading step data: {}", e);
                    }
                    None => {}
                }
            }
        }

        if self.run_rx.is_some() || (self.reader.is_some() && !self.paused) {
            ctx.request_repaint();
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.show_config_panel {
            egui::Panel::left("file_viewer_panel")
                .resizable(true)
                .default_size(280.0)
                .min_size(160.0)
                .show_inside(ui, |ui| {
                    ui.heading("config.example.yaml");
                    ui.horizontal(|ui| {
                        if ui.button("Run").clicked() {
                            self.save_config_run(ui.ctx());
                        }
                    });
                    if !self.run_save_feedback.is_empty() {
                        ui.label(egui::RichText::new(&self.run_save_feedback).small().weak());
                    }
                    ui.separator();
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.side_file_content)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(24)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Sugarscape GUI");
                ui.checkbox(&mut self.show_config_panel, "Show config.example.yaml");
            });
            ui.label(format!("Step: {}", self.step));
            ui.add_space(4.0);
            if self.data.is_none() {
                ui.label("No data loaded.");
            } else {
                let gw = self.grid_width;
                let gh = self.grid_height;
                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label(format!("Sugar levels ({gw}×{gh})"));
                        if let Some(data) = self.data.as_ref() {
                            show_sugar_levels(ui, data, gw, gh);
                        }
                    });
                    cols[1].vertical(|ui| {
                        ui.label(format!("Agents ({gw}×{gh})"));
                        if let Some(data) = self.data.as_ref() {
                            show_agents(ui, data, gw, gh);
                        }
                    });
                });
            }
            ui.add_space(8.0);
            ui.label("Wealth distribution");
            if let Some(data) = self.data.as_ref() {
                show_wealth_distribution(ui, &data.wealths);
            }
        });
    }
}
