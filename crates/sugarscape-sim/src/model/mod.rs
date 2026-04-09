use std::collections::HashSet;

use crate::config::{AgentParams, WorldParams};

mod ages;
mod capacities;
mod geometry;
mod step;
mod visions;
mod wealths;

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
}

#[cfg(test)]
mod tests {
    use p_test::p_test;

    use crate::config::{AgentParams, WorldParams};

    use super::Model;

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
}
