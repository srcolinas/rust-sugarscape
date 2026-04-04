use std::collections::HashSet;

use crate::config::{AgentParams, WorldParams};

#[derive(Debug)]
pub struct Model {
    // NOTE: This struct relies on the fact that two agents cannot occupy the same cell.
    pub capacities: Vec<f32>,
    pub levels: Vec<u8>,
    pub wealths: Vec<u32>,
    pub visions: Vec<u32>,
    pub ages: Vec<u32>,

    width: u8,
    height: u8,
}

impl Model {
    pub fn new(world: WorldParams, agents: AgentParams) -> Model {
        let capacities = Model::init_capacities(&world);
        let num_cells = capacities.len();
        Model {
            capacities,
            levels: vec![0; num_cells],
            wealths: Model::init_wealths(&agents, num_cells),
            visions: Model::init_visions(&agents, num_cells),
            ages: Model::init_ages(&agents, num_cells),
            width: world.width,
            height: world.height,
        }
    }
    fn init_capacities(world: &WorldParams) -> Vec<f32> {
        let max_capacity = world.capacity_distribution.max_capacity as f32;
        let reduction_factor = world.capacity_distribution.reduction_factor;
        let num_cells = world.width as usize * world.height as usize;

        let mut capacities = vec![0.0; num_cells];
        let mut peaks = HashSet::new();
        for peak in world.capacity_distribution.peaks.iter() {
            let idx = coord_to_idx(peak.x, peak.y, world.width);
            capacities[idx] = max_capacity;
            peaks.insert((peak.x, peak.y));
        }

        for x in 0..world.width {
            for y in 0..world.height {
                if !peaks.contains(&(x, y)) {
                    let mut distance: f32 = f32::INFINITY;
                    for (px, py) in peaks.iter() {
                        let current = euclidean_distance(x, y, *px, *py);
                        distance = f32::min(distance, current);
                    }
                    let idx = coord_to_idx(x, y, world.width);
                    let capacity = f32::max(0.0, max_capacity - distance * reduction_factor);
                    capacities[idx] = capacity;
                }
            }
        }
        capacities
    }
    fn init_wealths(agents: &AgentParams, cells: usize) -> Vec<u32> {
        vec![0; cells]
    }
    fn init_visions(agents: &AgentParams, cells: usize) -> Vec<u32> {
        vec![0; cells]
    }
    fn init_ages(agents: &AgentParams, cells: usize) -> Vec<u32> {
        vec![0; cells]
    }
}

#[inline]
fn coord_to_idx(x: u8, y: u8, width: u8) -> usize {
    x as usize + y as usize * width as usize
}

#[inline]
fn euclidean_distance(x1: u8, y1: u8, x2: u8, y2: u8) -> f32 {
    ((x1 as f32 - x2 as f32).powi(2) + (y1 as f32 - y2 as f32).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentParams, CellCapacityDistribution, CellPosition, WorldParams};
    use approx::assert_relative_eq;
    use p_test::p_test;

    /// [`Model::new`] using default [`WorldParams`] / [`AgentParams`]; override fields with `..w` / `..a`.
    fn create_model(
        customize: impl FnOnce(WorldParams, AgentParams) -> (WorldParams, AgentParams),
    ) -> Model {
        let (world, agents) = customize(WorldParams::default(), AgentParams::default());
        Model::new(world, agents)
    }

    #[p_test(
        (1, 1, 1),
        (5, 5, 25),
        (10, 10, 100),
    )]
    fn test_vectors_have_correct_size(width: u8, height: u8, expected_size: usize) {
        let model = create_model(|w, a| (WorldParams { width, height, ..w }, a));
        assert_eq!(model.capacities.len(), expected_size);
        assert_eq!(model.levels.len(), expected_size);
        assert_eq!(model.wealths.len(), expected_size);
        assert_eq!(model.visions.len(), expected_size);
        assert_eq!(model.ages.len(), expected_size);
    }

    #[p_test(
        (0, 0, 10.0),
        (2, 2, 10.0),
    )]
    fn test_peaks_have_max_capacity(x: u8, y: u8, max_capacity: f32) {
        let width = 10;
        let model = create_model(|w, a| {
            (
                WorldParams {
                    width,
                    height: 10,
                    capacity_distribution: CellCapacityDistribution {
                        peaks: vec![CellPosition { x, y }],
                        max_capacity,
                        reduction_factor: 1.0,
                    },
                    ..w
                },
                a,
            )
        });

        let idx = coord_to_idx(x, y, width);
        assert_eq!(model.capacities[idx], max_capacity);
    }

    #[p_test(
        (10.0, 0.5, 9.5),
    )]
    fn test_reduction_factor_orthogonal_distance(
        max_capacity: f32,
        reduction_factor: f32,
        distance: f32,
    ) {
        let x: u8 = 1;
        let y: u8 = 1;
        let capacities = compute_reduced_capacities(max_capacity, reduction_factor, 3, 3, x, y);
        let epsilon = 1e-5;

        let deltas: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dx, dy) in deltas.iter() {
            let cx = u8::try_from(x as i8 + dx).unwrap();
            let cy = u8::try_from(y as i8 + dy).unwrap();
            let idx = coord_to_idx(cx, cy, 3);
            assert_relative_eq!(capacities[idx], distance, epsilon = epsilon);
        }
    }
    fn compute_reduced_capacities(
        max_capacity: f32,
        reduction_factor: f32,
        width: u8,
        height: u8,
        px: u8,
        py: u8,
    ) -> Vec<f32> {
        let model = create_model(|w, a| {
            (
                WorldParams {
                    width,
                    height,
                    capacity_distribution: CellCapacityDistribution {
                        reduction_factor,
                        max_capacity,
                        peaks: vec![CellPosition { x: px, y: py }],
                    },
                    ..w
                },
                a,
            )
        });
        model.capacities
    }
}
