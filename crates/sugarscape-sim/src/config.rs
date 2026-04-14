use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct SimulationConfig {
    pub world: WorldParams,
    pub agents: AgentParams,
    pub run: RunParams,
}

#[derive(Debug, Deserialize, Validate)]
pub struct WorldParams {
    #[validate(range(min = 1))]
    pub width: u8,

    #[validate(range(min = 1))]
    pub height: u8,

    pub growth_rate: u8,

    pub capacity_distribution: CellCapacityDistribution,
}

impl Default for WorldParams {
    fn default() -> Self {
        WorldParams {
            width: 10,
            height: 10,
            growth_rate: 1,
            capacity_distribution: CellCapacityDistribution::default(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CellCapacityDistribution {
    pub peaks: Vec<CellPosition>,
    pub max_capacity: f32,

    #[validate(range(exclusive_min = 0.0))]
    pub reduction_factor: f32,
}

impl Default for CellCapacityDistribution {
    fn default() -> Self {
        CellCapacityDistribution {
            peaks: Vec::new(),
            max_capacity: 10.0,
            reduction_factor: 0.5,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CellPosition {
    pub row: u8,
    pub col: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RandomDistribution {
    Uniform { min: u32, max: u32 },
}

impl Default for RandomDistribution {
    fn default() -> Self {
        RandomDistribution::Uniform { min: 0, max: 0 }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct AgentParams {
    pub count: usize,
    pub wealth_distribution: RandomDistribution,
    pub metabolic_rate_distribution: RandomDistribution,
    pub vision_distribution: RandomDistribution,
    pub max_age_distribution: RandomDistribution,
}

#[derive(Debug, Deserialize)]
pub struct RunParams {
    pub iterations: u32,
}
