use rand::seq::IteratorRandom;
use std::collections::{HashMap, HashSet};

use bimap::BiMap;

use crate::agents::Agents;
use crate::config::{AgentParams, CellPosition, WorldParams};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct AgentId(usize);

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
struct CellId(usize);

#[derive(Debug)]
pub struct World {
    capacities: Vec<f32>,
    levels: Vec<f32>,
    growth_rate: f32,

    width: u8,
    height: u8,

    // Cells not in the map are empty.
    // Using a BiMap instead of a HashMap to ensure that the
    // insertion and removal of agents is O(1).
    locations: BiMap<CellId, AgentId>,
    agents: Agents,

    step_count: usize,
}

impl World {
    pub fn new(world: &WorldParams, agents: &AgentParams) -> Self {
        let max_capacity = world.capacity_distribution.max_capacity;
        let reduction_factor = world.capacity_distribution.reduction_factor;
        let num_cells = world.width as usize * world.height as usize;

        let mut capacities = vec![0.0; num_cells];
        let mut peaks = HashSet::new();
        for peak in world.capacity_distribution.peaks.iter() {
            let idx = coord_to_idx(peak.row, peak.col, world.width);
            capacities[idx] = max_capacity;
            peaks.insert(peak);
        }

        for col in 0..world.width {
            for row in 0..world.height {
                if !peaks.contains(&CellPosition { row, col }) {
                    let mut distance: f32 = f32::INFINITY;
                    for peak in peaks.iter() {
                        let current = euclidean_distance(&CellPosition { row, col }, peak);
                        distance = f32::min(distance, current);
                    }
                    let idx = coord_to_idx(row, col, world.width);
                    let capacity = f32::max(0.0, max_capacity - distance * reduction_factor);
                    capacities[idx] = capacity;
                }
            }
        }

        let agents = Agents::new(agents);
        let locations = World::populate(&agents, num_cells);
        World {
            capacities,
            levels: vec![0.0; num_cells],
            growth_rate: world.growth_rate as f32,
            width: world.width,
            height: world.height,
            locations,
            agents,
            step_count: 0,
        }
    }

    fn populate(agents: &Agents, num_cells: usize) -> BiMap<CellId, AgentId> {
        let mut locations = BiMap::new();
        for (agent, cell) in (0..num_cells)
            .sample(&mut rand::rng(), agents.count)
            .iter()
            .enumerate()
        {
            locations.insert(CellId(*cell), AgentId(agent));
        }
        locations
    }

    pub fn step(&mut self) {
        self.growback_rule();
        let to_remove = self.movement_rule();
        // Increment the step count after the movement rule to ensure that the replacement rule
        // works for agents that have a lifetime of 1 step.
        self.step_count += 1;
        self.replacement_rule(to_remove);
    }

    #[inline]
    fn growback_rule(&mut self) {
        for (cell_idx, level) in self.levels.iter_mut().enumerate() {
            let capacity = self.capacities[cell_idx];
            *level = f32::min(capacity, *level + self.growth_rate);
        }
    }

    #[inline]
    fn movement_rule(&mut self) -> HashSet<AgentId> {
        {
            let mut rng = rand::rng();
            let mut updates = HashMap::new();
            let mut cells_to_remove = Vec::with_capacity(self.locations.len());
            for (cell, agent) in self.locations.iter() {
                let vision = self.agents.visions[agent.0];
                let mut nearby: HashSet<CellId> = HashSet::with_capacity(4 * vision as usize);
                nearby.insert(*cell);
                let mut current_max_level = self.levels[cell.0];
                let cell_coords = idx_to_coord(cell.0, self.width);
                for direction in [north_to, south_to, west_to, east_to] {
                    for i in 1..=vision {
                        let alternative = direction(&cell_coords, i as u8, self.width, self.height);
                        match alternative {
                            Some(alternative) => {
                                let idx =
                                    coord_to_idx(alternative.row, alternative.col, self.width);
                                if !self.locations.contains_left(&CellId(idx)) {
                                    current_max_level =
                                        f32::max(current_max_level, self.levels[idx]);
                                    nearby.insert(CellId(idx));
                                }
                            }
                            None => break,
                        }
                    }
                }
                cells_to_remove.push(*cell);
                let selected = nearby
                    .iter()
                    .filter(|cell| self.levels[cell.0] >= current_max_level)
                    .choose(&mut rng)
                    .unwrap();
                updates.insert(*selected, *agent);
            }
            self.locations
                .retain(|cell, _| !cells_to_remove.contains(cell));
            self.locations.extend(updates);

            let to_remove = self.consume_sugar();
            to_remove
        }
    }

    #[inline]
    fn consume_sugar(&mut self) -> HashSet<AgentId> {
        let mut to_remove = HashSet::new();
        for (cell, agent) in self.locations.iter() {
            let update = self.levels[cell.0] - self.agents.metabolic_rates[agent.0];
            self.agents.wealths[agent.0] += update;
            self.levels[cell.0] = 0.0;
            if self.agents.wealths[agent.0] <= 0.0 {
                to_remove.insert(*agent);
            }
        }
        to_remove
    }

    #[inline]
    fn replacement_rule(&mut self, to_remove: HashSet<AgentId>) {
        let step_count = self.step_count as u32;
        let to_replace = Vec::from_iter(
            self.locations
                .iter()
                .filter(|(_, agent)| {
                    self.agents.ages[agent.0] >= step_count || to_remove.contains(agent)
                })
                .map(|(cell, _)| *cell),
        );
        let mut rng = rand::rng();
        let num_cells = self.width as usize * self.height as usize;
        for cell in to_replace {
            self.locations.remove_by_left(&cell).unwrap();
            let new_cell = (0..num_cells)
                .filter(|&cell| !self.locations.contains_left(&CellId(cell)))
                .choose(&mut rng)
                .unwrap();
            let new_agent_idx = (0..self.agents.count)
                .filter(|&idx| !self.locations.contains_right(&AgentId(idx)))
                .choose(&mut rng)
                .unwrap();
            self.agents.add_new_agent_at(new_agent_idx);
            self.locations
                .insert(CellId(new_cell), AgentId(new_agent_idx));
        }
    }
}

#[inline]
fn coord_to_idx(row: u8, col: u8, width: u8) -> usize {
    row as usize * width as usize + col as usize
}

#[inline]
fn idx_to_coord(idx: usize, width: u8) -> CellPosition {
    CellPosition {
        row: idx as u8 / width,
        col: idx as u8 % width,
    }
}

#[inline]
fn north_to(coords: &CellPosition, by: u8, _w: u8, _h: u8) -> Option<CellPosition> {
    coords.row.checked_sub(by).map(|x| CellPosition {
        row: x,
        col: coords.col,
    })
}

#[inline]
fn south_to(coords: &CellPosition, by: u8, _w: u8, height: u8) -> Option<CellPosition> {
    coords.row.checked_add(by).and_then(|x| {
        if x >= height {
            None
        } else {
            Some(CellPosition {
                row: x,
                col: coords.col,
            })
        }
    })
}

#[inline]
fn west_to(coords: &CellPosition, by: u8, _w: u8, _h: u8) -> Option<CellPosition> {
    coords.col.checked_sub(by).map(|x| CellPosition {
        row: coords.row,
        col: x,
    })
}

#[inline]
fn east_to(coords: &CellPosition, by: u8, width: u8, _h: u8) -> Option<CellPosition> {
    coords.col.checked_add(by).and_then(|x| {
        if x >= width {
            None
        } else {
            Some(CellPosition {
                row: coords.row,
                col: x,
            })
        }
    })
}

#[inline]
fn euclidean_distance(p1: &CellPosition, p2: &CellPosition) -> f32 {
    ((p1.row as f32 - p2.row as f32).powi(2) + (p1.col as f32 - p2.col as f32).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use p_test::p_test;

    use crate::config::{CellCapacityDistribution, CellPosition, RandomDistribution, WorldParams};

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
        let peaks = [
            CellPosition { row: 1, col: 1 },
            CellPosition { row: 3, col: 3 },
        ];
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
                            .map(|p| CellPosition {
                                row: p.row,
                                col: p.col,
                            })
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

    #[test]
    fn growback_rule_doesnot_go_beyond_max_capacity() {
        let mut world = from_defaults(|(_w, a)| {
            (
                WorldParams {
                    width: 2,
                    height: 2,
                    // Use a growth rate that ensure next step will go beyond max capacity if
                    // not capped properly by the implementation of the growback rule.
                    growth_rate: 4,
                    capacity_distribution: CellCapacityDistribution {
                        peaks: vec![CellPosition { row: 0, col: 0 }],
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

    #[test]
    fn agents_move_to_nearby_cells() {
        let mut world = from_defaults(|(w, a)| {
            (
                WorldParams {
                    width: 1,
                    height: 2,
                    ..w
                },
                AgentParams {
                    count: 1,
                    vision_distribution: RandomDistribution::Uniform { min: 1, max: 1 },
                    ..a
                },
            )
        });
        // Since the selection is at random, there should be an
        // approximately even split between the two cells.
        // We need to run the step a number of times to ensure accurate results.
        let total: usize = 100;
        let num_selected = (0..total)
            .reduce(|acc, _| {
                world.step();
                if world.locations.contains_left(&CellId(0)) {
                    return acc + 1;
                }
                acc
            })
            .unwrap();
        assert_relative_eq!(num_selected as f32 / total as f32, 0.5, epsilon = 0.1);
    }

    #[p_test((1,), (2,), (3,), (4,))]
    fn agents_move_to_nearby_cells_with_max_level_as_tiebreaker(steps: usize) {
        let peak_row = 0;
        let peak_col = 0;
        let mut world = from_defaults(|(_w, a)| {
            (
                WorldParams {
                    width: 1,
                    height: 2,
                    growth_rate: 1,
                    capacity_distribution: CellCapacityDistribution {
                        // Using a reduction factor that is greater than the
                        // minimun distance between a peak and a non-peak cell,
                        // ensures that capacity is cero except for the peak cells.
                        peaks: vec![CellPosition {
                            row: peak_row,
                            col: peak_col,
                        }],
                        max_capacity: 1.0,
                        reduction_factor: 3.0,
                    },
                },
                AgentParams {
                    count: 1,
                    vision_distribution: RandomDistribution::Uniform { min: 2, max: 2 },
                    ..a
                },
            )
        });
        for _ in 0..steps {
            world.step();
        }
        assert_eq!(world.locations.len(), 1);
        assert_eq!(
            world
                .locations
                .contains_left(&CellId(coord_to_idx(peak_row, peak_col, world.width))),
            true
        );
    }

    #[p_test((10.0, 10))]
    fn agent_consumes_sugar_after_one_step(capacity: f32, metabolic_rate: u32) {
        let mut world = from_defaults(|(_w, a)| {
            (
                WorldParams {
                    width: 1,
                    height: 1,
                    growth_rate: 1,
                    capacity_distribution: CellCapacityDistribution {
                        peaks: vec![CellPosition { row: 0, col: 0 }],
                        max_capacity: capacity,
                        reduction_factor: 0.0,
                    },
                },
                AgentParams {
                    count: 1,
                    metabolic_rate_distribution: RandomDistribution::Uniform {
                        min: metabolic_rate,
                        max: metabolic_rate,
                    },
                    ..a
                },
            )
        });
        world.step();
        assert_eq!(world.levels[0], 0.0);
        assert_eq!(world.agents.wealths[0], capacity - metabolic_rate as f32);
    }

    #[test]
    fn agent_dies_when_it_has_no_sugar_after_one_step() {
        let mut world = from_defaults(|(_w, a)| {
            (
                WorldParams {
                    width: 1,
                    height: 1,
                    // No sugar grows back in this world, so the agent will die.
                    growth_rate: 0,
                    capacity_distribution: CellCapacityDistribution {
                        peaks: vec![CellPosition { row: 0, col: 0 }],
                        // This value doesn't matter because the growth rate is zero.
                        max_capacity: 100.0,
                        reduction_factor: 0.0,
                    },
                },
                AgentParams {
                    count: 1,
                    // The agent will have an initial wealth of 1, which, given that the metabolic rate
                    // is 10, and that there will be no sugar in the cell, will be consumed in one step.
                    wealth_distribution: RandomDistribution::Uniform { min: 1, max: 1 },
                    metabolic_rate_distribution: RandomDistribution::Uniform { min: 10, max: 10 },
                    vision_distribution: RandomDistribution::Uniform { min: 3, max: 3 },
                    ..a
                },
            )
        });
        // Inject a vision outside of the vision distribution to check that the agent in the
        // given index is a new one.
        world.agents.visions[0] = 15;
        world.step();
        assert_eq!(world.agents.visions[0], 3);
    }

    #[p_test((0, 0, 5, 0), (1, 0, 5, 5), (0, 1, 5, 1), (1, 1, 5, 6))]
    fn translation_from_coords_to_idx(row: u8, col: u8, width: u8, expected_idx: usize) {
        let idx = coord_to_idx(row, col, width);
        assert_eq!(idx, expected_idx);
    }

    #[p_test((0, 5, (0, 0)), (5, 5, (1, 0)), (1, 5, (0, 1)), (6, 5, (1, 1)))]
    fn translation_from_idx_to_coords(idx: usize, width: u8, expected_coords: (u8, u8)) {
        let coords = idx_to_coord(idx, width);
        assert_eq!(
            coords,
            CellPosition {
                row: expected_coords.0,
                col: expected_coords.1
            }
        );
    }

    #[p_test((1, 0, 1, 0, 0), (1, 1, 1, 0, 1))]
    fn movement_to_the_north(row: u8, col: u8, by: u8, expected_row: u8, expected_col: u8) {
        let result = north_to(&CellPosition { row, col }, by, 5, 5);
        assert_eq!(
            result,
            Some(CellPosition {
                row: expected_row,
                col: expected_col
            })
        );
    }

    #[p_test((0, 0, 1), (0, 1, 1))]
    fn movement_to_the_north_out_of_bounds(row: u8, col: u8, by: u8) {
        let result = north_to(&CellPosition { row, col }, by, 5, 5);
        assert_eq!(result, None);
    }

    #[p_test((0, 0, 1, 1, 0), (0, 1, 1, 1, 1))]
    fn movement_to_the_south(row: u8, col: u8, by: u8, expected_row: u8, expected_col: u8) {
        let result = south_to(&CellPosition { row, col }, by, 5, 5);
        assert_eq!(
            result,
            Some(CellPosition {
                row: expected_row,
                col: expected_col
            })
        );
    }

    #[p_test((1, 0, 1, 2), (1, 1, 1, 2))]
    fn movement_to_the_south_out_of_bounds(row: u8, col: u8, by: u8, height: u8) {
        let result = south_to(&CellPosition { row, col }, by, 5, height);
        assert_eq!(result, None);
    }

    #[p_test((0, 1, 1, 0, 0), (1, 1, 1, 1, 0))]
    fn movement_to_the_west(row: u8, col: u8, by: u8, expected_row: u8, expected_col: u8) {
        let result = west_to(&CellPosition { row, col }, by, 5, 5);
        assert_eq!(
            result,
            Some(CellPosition {
                row: expected_row,
                col: expected_col
            })
        );
    }

    #[p_test((0, 0, 1), (1, 0, 1))]
    fn movement_to_the_west_out_of_bounds(row: u8, col: u8, by: u8) {
        let result = west_to(&CellPosition { row, col }, by, 5, 5);
        assert_eq!(result, None);
    }

    #[p_test((0, 0, 1, 0, 1), (1, 0, 1, 1, 1))]
    fn movement_to_the_east(row: u8, col: u8, by: u8, expected_row: u8, expected_col: u8) {
        let result = east_to(&CellPosition { row, col }, by, 5, 5);
        assert_eq!(
            result,
            Some(CellPosition {
                row: expected_row,
                col: expected_col
            })
        );
    }

    #[p_test((0, 1, 1, 2), (1, 1, 1, 2))]
    fn movement_to_the_east_out_of_bounds(row: u8, col: u8, by: u8, width: u8) {
        let result = east_to(&CellPosition { row, col }, by, width, 5);
        assert_eq!(result, None);
    }

    #[p_test((1,), (2,), (3,), (4,), (5,))]
    fn replacement_rule_ensures_number_of_agents_is_constant(iterations: usize) {
        let mut world = from_defaults(|(w, a)| {
            (
                WorldParams {
                    width: 3,
                    height: 3,
                    ..w
                },
                AgentParams {
                    count: 5,
                    // I want agents die after a single step to quickly trigger the replacement rule.
                    max_age_distribution: RandomDistribution::Uniform { min: 1, max: 1 },
                    ..a
                },
            )
        });

        for _ in 0..iterations {
            world.step();
        }
        assert_eq!(world.locations.len(), world.agents.count);
    }
}
