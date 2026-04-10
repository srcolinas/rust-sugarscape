use std::collections::HashSet;

use crate::config::{AgentParams, WorldParams};

mod geometry;
mod init_agents;
mod init_world;
mod step;

#[derive(Debug, Default)]
pub struct Model {
    // NOTE: Given that two agents cannot occupy the same cell, we can
    // have information about all agents in a single vector. We will
    // keep keep track of empty cells in a hash set for fast lookup.

    // world-specific data
    capacities: Vec<f32>,
    levels: Vec<i32>,

    // agent-specific data
    wealths: Vec<i32>,
    visions: Vec<i32>,
    ages: Vec<i32>,

    width: u8,
    height: u8,

    empty_cells: HashSet<usize>,
}

impl Model {
    pub fn new(world: WorldParams, agents: AgentParams) -> Self {
        let (capacities, levels) = Model::init_world(&world);
        let num_cells = capacities.len();
        let (wealths, visions, ages, empty_cells) = Model::init_agents(&agents, num_cells);

        Model {
            capacities,
            levels,
            wealths,
            visions,
            ages,
            width: world.width,
            height: world.height,
            empty_cells,
        }
    }

    fn default() -> Self {
        Model::new(WorldParams::default(), AgentParams::default())
    }

    pub fn from_default(
        customize: impl FnOnce(WorldParams, AgentParams) -> (WorldParams, AgentParams),
    ) -> Model {
        let (world, agents) = customize(WorldParams::default(), AgentParams::default());
        Model::new(world, agents)
    }
}

#[cfg(test)]
mod tests {
    use p_test::p_test;

    use crate::config::WorldParams;

    use super::Model;

    #[p_test(
        (1, 1, 1),
        (5, 5, 25),
        (10, 10, 100),
    )]
    fn test_vectors_have_correct_size(width: u8, height: u8, expected_size: usize) {
        let model = Model::from_default(|w, a| (WorldParams { width, height, ..w }, a));
        assert_eq!(model.capacities.len(), expected_size);
        assert_eq!(model.levels.len(), expected_size);
        assert_eq!(model.wealths.len(), expected_size);
        assert_eq!(model.visions.len(), expected_size);
        assert_eq!(model.ages.len(), expected_size);
    }
}
