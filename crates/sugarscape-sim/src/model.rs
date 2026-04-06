use std::collections::HashSet;

use crate::config::{AgentParams, WorldParams};

#[derive(Debug)]
pub struct Model {
    // NOTE: Given that two agents cannot occupy the same cell, we can
    // have information about all agents in a single vector. We will
    // keep keep track of empty cells in a hash set for fast lookup.
    capacities: Vec<f32>,
    levels: Vec<i32>,
    wealths: Vec<i32>,
    visions: Vec<i32>,
    ages: Vec<i32>,

    width: u8,
    height: u8,

    empty_cells: HashSet<(usize, usize)>,
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

            empty_cells: HashSet::new(),
        }
    }

    pub fn step(&mut self) {
        println!("Stepping model, {:?}", self);
    }

    fn init_capacities(world: &WorldParams) -> Vec<f32> {
        let max_capacity = world.capacity_distribution.max_capacity;
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
    fn init_wealths(agents: &AgentParams, cells: usize) -> Vec<i32> {
        vec![-1; cells]
    }
    fn init_visions(agents: &AgentParams, cells: usize) -> Vec<i32> {
        vec![-1; cells]
    }
    fn init_ages(agents: &AgentParams, cells: usize) -> Vec<i32> {
        vec![-1; cells]
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
        "peak(1,1)", (2.0, 0.5, 1, 1, 2.0),
        "peak(3,3)", (2.0, 0.5, 3, 3, 2.0),
        "orthogonal(0,1)", (2.0, 0.5, 0, 1, 1.5),
        "orthogonal(1,0)", (2.0, 0.5, 1, 0, 1.5),
        "orthogonal(1,2)", (2.0, 0.5, 1, 2, 1.5),
        "orthogonal(2,1)", (2.0, 0.5, 2, 1, 1.5),
        "orthogonal(2,3)", (2.0, 0.5, 2, 3, 1.5),
        "orthogonal(3,2)", (2.0, 0.5, 3, 2, 1.5),
        "orthogonal(3,4)", (2.0, 0.5, 3, 4, 1.5),
        "orthogonal(4,3)", (2.0, 0.5, 4, 3, 1.5),
        "diagonal(0,0)", (2.0, 0.5, 0, 0, 1.292893),
        "diagonal(2,0)", (2.0, 0.5, 2, 0, 1.292893),
        "diagonal(0,2)", (2.0, 0.5, 0, 2, 1.292893),
        "diagonal(2,2)", (2.0, 0.5, 2, 2, 1.292893),
        "diagonal(4,2)", (2.0, 0.5, 4, 2, 1.292893),
        "diagonal(2,4)", (2.0, 0.5, 2, 4, 1.292893),
        "diagonal(4,4)", (2.0, 0.5, 4, 4, 1.292893),
        // Remaining non-peak cells (nearest-peak distance 2, √5, or √10)
        "dist2(3,1)", (2.0, 0.5, 3, 1, 1.0),
        "dist2(1,3)", (2.0, 0.5, 1, 3, 1.0),
        "dist_sqrt5(3,0)", (2.0, 0.5, 3, 0, 0.881966),
        "dist_sqrt5(4,1)", (2.0, 0.5, 4, 1, 0.881966),
        "dist_sqrt5(0,3)", (2.0, 0.5, 0, 3, 0.881966),
        "dist_sqrt5(1,4)", (2.0, 0.5, 1, 4, 0.881966),
        "dist_sqrt10(4,0)", (2.0, 0.5, 4, 0, 0.418861),
        "dist_sqrt10(0,4)", (2.0, 0.5, 0, 4, 0.418861),
    )]
    fn test_capacities_for_two_peaks_on_a_5x5_grid(
        max_capacity: f32,
        reduction_factor: f32,
        x: u8,
        y: u8,
        expected_value: f32,
    ) {
        // 5x5: P at (1,1) and (3,3); x→ columns, y↓ rows
        //       x: 0 1 2 3 4
        //     y0: . . . . .
        //     y1: . P . . .
        //     y2: . . . . .
        //     y3: . . . P .
        //     y4: . . . . .

        let width = 5;
        let height = 5;
        let peaks = vec![CellPosition { x: 1, y: 1 }, CellPosition { x: 3, y: 3 }];
        let model = create_model(|w, a| {
            (
                WorldParams {
                    width,
                    height,
                    capacity_distribution: CellCapacityDistribution {
                        reduction_factor,
                        max_capacity,
                        peaks: peaks
                            .iter()
                            .map(|p| CellPosition { x: p.x, y: p.y })
                            .collect(),
                    },
                    ..w
                },
                a,
            )
        });
        let capacities = model.capacities;
        let idx = coord_to_idx(x, y, width);
        assert_relative_eq!(capacities[idx], expected_value, epsilon = 1e-5);
    }
}
