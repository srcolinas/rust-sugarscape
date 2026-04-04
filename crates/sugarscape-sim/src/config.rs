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
    pub capacity_distribution: CellCapacityDistribution,
}

#[derive(Debug, Deserialize)]
pub struct CellCapacityDistribution {
    peaks: Vec<CellPosition>,
    max_capacity: u8,
    reduction_factor: f32,
}

#[derive(Debug, Deserialize)]
pub struct CellPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RandomDistribution {
    Uniform { min: u32, max: u32 },
}

#[derive(Debug, Deserialize)]
pub struct AgentParams {
    pub count: usize,
    pub wealth_distribution: RandomDistribution,
    pub metabolic_rate_distribution: RandomDistribution,
    pub vision_distribution: RandomDistribution,
    pub max_age_distribution: RandomDistribution,
}

#[derive(Debug, Deserialize)]
pub struct RunParams {
    pub iterations: u64,
}
