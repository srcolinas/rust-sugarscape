use rand::seq::IndexedRandom;

use crate::config::{AgentParams, RandomDistribution};

#[derive(Debug)]
pub struct Agents {
    pub wealths: Vec<f32>,
    pub metabolic_rates: Vec<f32>,
    pub visions: Vec<u32>,
    pub ages: Vec<u32>,

    pub count: usize,

    params: AgentParams,
}

impl Agents {
    pub fn new(agents: &AgentParams) -> Agents {
        let (visions, ages) = Agents::initialize_u32_attributes(agents);
        let (wealths, metabolic_rates) = Agents::initialize_f32_attributes(agents);
        Agents {
            wealths,
            visions,
            ages,
            metabolic_rates,
            count: agents.count,
            params: agents.clone(),
        }
    }

    fn initialize_u32_attributes(agents: &AgentParams) -> (Vec<u32>, Vec<u32>) {
        let mut visions: Vec<u32> = Vec::with_capacity(agents.count);
        let mut ages: Vec<u32> = Vec::with_capacity(agents.count);

        for (attribute, distribution) in [
            (&mut visions, &agents.vision_distribution),
            (&mut ages, &agents.max_age_distribution),
        ] {
            match distribution {
                RandomDistribution::Uniform { min, max } => {
                    let possible_values: Vec<u32> = (*min..=*max).collect();
                    let mut rng = rand::rng();
                    for _ in 0..agents.count {
                        attribute.push(*possible_values.choose(&mut rng).unwrap());
                    }
                }
            }
        }
        (visions, ages)
    }

    fn initialize_f32_attributes(agents: &AgentParams) -> (Vec<f32>, Vec<f32>) {
        let mut wealths: Vec<f32> = Vec::with_capacity(agents.count);
        let mut metabolic_rates: Vec<f32> = Vec::with_capacity(agents.count);
        for (attribute, distribution) in [
            (&mut wealths, &agents.wealth_distribution),
            (&mut metabolic_rates, &agents.metabolic_rate_distribution),
        ] {
            match distribution {
                RandomDistribution::Uniform { min, max } => {
                    let possible_values: Vec<f32> = (*min..=*max).map(|x| x as f32).collect();
                    let mut rng = rand::rng();
                    for _ in 0..agents.count {
                        attribute.push(*possible_values.choose(&mut rng).unwrap());
                    }
                }
            }
        }
        (wealths, metabolic_rates)
    }

    pub fn add_new_agent_at(&mut self, idx: usize) {
        self.add_new_agent_at_u32_attributes(idx);
        self.add_new_agent_at_f32_attributes(idx);
    }

    fn add_new_agent_at_u32_attributes(&mut self, idx: usize) {
        for (attribute, distribution) in [
            (&mut self.visions, &self.params.vision_distribution),
            (&mut self.ages, &self.params.max_age_distribution),
        ] {
            match distribution {
                RandomDistribution::Uniform { min, max } => {
                    let possible_values: Vec<u32> = (*min..=*max).collect();
                    let value = possible_values.choose(&mut rand::rng()).unwrap();
                    attribute[idx] = *value;
                }
            }
        }
    }

    fn add_new_agent_at_f32_attributes(&mut self, idx: usize) {
        for (attribute, distribution) in [
            (
                &mut self.metabolic_rates,
                &self.params.metabolic_rate_distribution,
            ),
            (&mut self.wealths, &self.params.wealth_distribution),
        ] {
            match distribution {
                RandomDistribution::Uniform { min, max } => {
                    let possible_values: Vec<f32> = (*min..=*max).map(|x| x as f32).collect();
                    let value = possible_values.choose(&mut rand::rng()).unwrap();
                    attribute[idx] = *value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use p_test::p_test;

    use crate::config::AgentParams;

    fn from_defaults(customize: impl FnOnce(AgentParams) -> AgentParams) -> Agents {
        let params = customize(AgentParams::default());
        Agents::new(&params)
    }
    #[p_test((1, ), (10, ))]
    fn size_of_vectors(num_agents: usize) {
        let agents = from_defaults(|p| AgentParams {
            count: num_agents,
            ..p
        });
        assert_eq!(agents.ages.len(), num_agents);
        assert_eq!(agents.visions.len(), num_agents);
        assert_eq!(agents.wealths.len(), num_agents);
        assert_eq!(agents.count, num_agents);
    }
    // NOTE: I am not testing the distribution of the different
    // attributes of the agent, because the complexity seems too
    // high given its value at this point.

    #[test]
    fn new_agent_is_added_at_cell_with_same_metabolic_rate_distribution() {
        let metabolic_rate = 50;
        let mut agents = from_defaults(|p| AgentParams {
            count: 100,
            metabolic_rate_distribution: RandomDistribution::Uniform {
                min: metabolic_rate,
                max: metabolic_rate,
            },
            ..p
        });
        agents.metabolic_rates[0] = metabolic_rate as f32;
        agents.add_new_agent_at(0);
        assert_eq!(agents.metabolic_rates[0], metabolic_rate as f32);
    }

    #[test]
    fn new_agent_is_added_at_cell_with_same_vision_distribution() {
        let vision = 5;
        let mut agents = from_defaults(|p| AgentParams {
            count: 100,
            vision_distribution: RandomDistribution::Uniform {
                min: vision,
                max: vision,
            },
            ..p
        });
        agents.visions[0] = 0;
        agents.add_new_agent_at(0);
        assert_eq!(agents.visions[0], vision);
    }

    #[test]
    fn new_agent_is_added_at_cell_with_same_max_age_distribution() {
        let max_age = 100;
        let mut agents = from_defaults(|p| AgentParams {
            count: 100,
            max_age_distribution: RandomDistribution::Uniform {
                min: max_age,
                max: max_age,
            },
            ..p
        });
        agents.ages[0] = 0;
        agents.add_new_agent_at(0);
        assert_eq!(agents.ages[0], max_age);
    }

    #[test]
    fn new_agent_is_added_at_cell_with_same_wealth_distribution() {
        let wealth = 100;
        let mut agents = from_defaults(|p| AgentParams {
            count: 100,
            wealth_distribution: RandomDistribution::Uniform {
                min: wealth,
                max: wealth,
            },
            ..p
        });
        agents.wealths[0] = 0.0;
        agents.add_new_agent_at(0);
        assert_eq!(agents.wealths[0], wealth as f32);
    }
}
