use rand::seq::IteratorRandom;
use std::collections::{HashMap, HashSet};

use crate::agents::Agents;
use crate::config::{AgentParams, WorldParams};

#[derive(Debug)]
struct AgentId(usize);

#[derive(Debug, Hash, Eq, PartialEq)]
struct CellId(usize);

#[derive(Debug)]
pub struct World {
    capacities: Vec<f32>,
    levels: Vec<f32>,
    growth_rate: f32,

    width: u8,
    height: u8,

    // Cells not in the map are empty.
    locations: HashMap<CellId, AgentId>,
    agents: Agents,
}

impl World {
    pub fn new(world: &WorldParams, agents: &AgentParams) -> Self {
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

        let agents = Agents::new(&agents);
        World {
            capacities,
            levels: vec![0.0; num_cells],
            growth_rate: world.growth_rate as f32,
            width: world.width,
            height: world.height,
            locations: World::populate(&agents, num_cells),
            agents,
        }
    }

    fn populate(agents: &Agents, num_cells: usize) -> HashMap<CellId, AgentId> {
        let mut locations = HashMap::new();
        for (cell_idx, agent_idx) in (0..num_cells)
            .sample(&mut rand::rng(), agents.count)
            .iter()
            .enumerate()
        {
            locations.insert(CellId(cell_idx), AgentId(*agent_idx));
        }
        locations
    }

    pub fn step(&mut self) {
        self.growback_rule();
        self.movement_rule();
        self.replacement_rule();
    }

    #[inline]
    fn growback_rule(&mut self) {
        for (cell_idx, level) in self.levels.iter_mut().enumerate() {
            let capacity = self.capacities[cell_idx];
            *level = f32::max(capacity, *level + self.growth_rate);
        }
    }

    #[inline]
    fn movement_rule(&mut self) {
        todo!()
    }

    #[inline]
    fn select_nearby_cells(&self) {
        todo!()
    }

    #[inline]
    fn replacement_rule(&mut self) {
        todo!()
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

    fn from_defaults(
        customize: impl FnOnce((WorldParams, AgentParams)) -> (WorldParams, AgentParams),
    ) -> World {
        let params = customize((WorldParams::default(), AgentParams::default()));
        World::new(&params.0, &params.1)
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
        let world = from_defaults(|(w, a)| {
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
        let capacities = world.capacities;
        let idx = coord_to_idx(x, y, width);
        assert_relative_eq!(capacities[idx], expected_value, epsilon = 1e-5);
    }

    #[p_test((5, 5, 10), (2, 3, 5), (2, 2, 4))]
    fn number_of_occupied_cells_is_correct(width: u8, height: u8, num_agents: usize) {
        let world = from_defaults(|(w, a)| {
            (
                WorldParams { width, height, ..w },
                AgentParams {
                    count: num_agents,
                    ..a
                },
            )
        });
        assert_eq!(world.locations.len(), num_agents);
    }

    fn test_growback_rule_doesnot_go_beyond_max_capacity() {
        let mut world = from_defaults(|(w, a)| {
            (
                WorldParams {
                    width: 2,
                    height: 2,
                    // Use a growth rate that ensure next step will go beyond max capacity if
                    // not capped properly by the implementation of the growback rule.
                    growth_rate: 3,
                    capacity_distribution: CellCapacityDistribution {
                        peaks: vec![CellPosition { x: 0, y: 0 }],
                        // Using a reduction factor that is greater than the
                        // minimun distance between a peak and a non-peak cell,
                        // ensures that capacity is cero except for the peak cells.
                        max_capacity: 1.0,
                        reduction_factor: 3.0,
                    },
                },
                a,
            )
        });
        world.step();
        assert_eq!(world.levels, vec![1.0, 0.0, 0.0, 0.0]);
    }
}
