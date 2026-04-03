#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SimulationConfig {
    pub world: WorldParams,
    pub agents: AgentParams,
    pub run: RunParams,
}

#[derive(Debug, Deserialize)]
pub struct WorldParams {
    pub width: u32,
    pub height: u32,
    pub growth_rate: u8,
    pub capacity_distribution: Distribution,
    pub initial_level_distribution: Distribution,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Distribution {
    Uniform { min: u32, max: u32 },
}

#[derive(Debug, Deserialize)]
pub struct AgentParams {
    pub count: usize,
    pub wealth_distribution: Distribution,
    pub metabolic_rate_distribution: Distribution,
    pub vision_distribution: Distribution,
    pub max_age_distribution: Distribution,
}

#[derive(Debug, Deserialize)]
pub struct RunParams {
    pub iterations: u64,
}
