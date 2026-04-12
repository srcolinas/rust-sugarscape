use rand::seq::IteratorRandom;
use std::collections::HashSet;

use crate::config::{AgentParams, WorldParams};

#[derive(Debug)]
pub struct World {
    capacities: Vec<f32>,
    levels: Vec<i32>,

    width: u8,
    height: u8,

    empty_cells: HashSet<usize>,
}

impl World {
    pub fn new(world: &WorldParams) -> Self {
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
        World {
            capacities,
            levels: vec![0; num_cells],
            width: world.width,
            height: world.height,
            empty_cells: HashSet::new(),
        }
    }

    pub fn populate(&mut self, agents: &AgentParams) {
        let num_cells = self.width as usize * self.height as usize;
        let range = 0..(num_cells - 1);
        for i in range.sample(&mut rand::rng(), num_cells - agents.count) {
            self.empty_cells.insert(i);
        }
    }

    pub fn step(&self) {
        println!("Runing an step");
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
    use approx::assert_relative_eq;
    use p_test::p_test;

    use crate::config::{CellCapacityDistribution, CellPosition, WorldParams};

    fn from_defaults(customize: impl FnOnce(WorldParams) -> WorldParams) -> World {
        let params = customize(WorldParams::default());
        World::new(&params)
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
        let peaks = [CellPosition { x: 1, y: 1 }, CellPosition { x: 3, y: 3 }];
        let world = from_defaults(|w| WorldParams {
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
        });
        let capacities = world.capacities;
        let idx = coord_to_idx(x, y, width);
        assert_relative_eq!(capacities[idx], expected_value, epsilon = 1e-5);
    }

    #[p_test((5, 5, 10, 15), (2, 3, 5, 1), (2, 2, 4, 0))]
    fn number_of_empty_cells_is_correct(
        width: u8,
        height: u8,
        num_agents: usize,
        num_empty_cells: usize,
    ) {
        let mut world = from_defaults(|w| WorldParams { width, height, ..w });
        world.populate(&AgentParams {
            count: num_agents,
            ..AgentParams::default()
        });
        assert_eq!(world.empty_cells.len(), num_empty_cells);
    }
}
