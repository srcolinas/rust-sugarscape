use egui_plot::PlotBounds;

pub const MAP_PLOT_HEIGHT: f32 = 320.0;

pub fn grid_map_bounds(grid_width: usize, grid_height: usize) -> PlotBounds {
    PlotBounds::from_min_max([0.0, 0.0], [grid_width as f64, grid_height as f64])
}

pub fn plot_data_aspect(grid_width: usize, grid_height: usize) -> f32 {
    let gh = grid_height.max(1) as f32;
    grid_width as f32 / gh
}
