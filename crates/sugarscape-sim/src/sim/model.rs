use crate::config::SimulationConfig;

#[derive(Debug)]
pub struct Model {
    capacities: Vec<u8>,
    levels: Vec<u8>,
    wealths: Vec<u32>,
    visions: Vec<u32>,
    ages: Vec<u32>,
}

impl Model {
    pub fn new(config: SimulationConfig) -> Model {
        let n = (config.world.width * config.world.height) as usize;
        Model {
            capacities: vec![0; n],
            levels: vec![0; n],
            wealths: vec![0; n],
            visions: vec![0; n],
            ages: vec![0; n],
        }
    }
}
