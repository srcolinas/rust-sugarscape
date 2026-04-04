use crate::config::SimulationConfig;

mod model;

use model::Model;

pub fn run_simulation(config: SimulationConfig) {
    let model = Model::new(config.world, config.agents);
    println!("{model:?}");
}
