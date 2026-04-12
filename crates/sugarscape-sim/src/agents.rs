use rand::seq::IndexedRandom;

use crate::config::{AgentParams, RandomDistribution};

#[derive(Debug)]
pub struct Agents {
    wealths: Vec<u32>,
    visions: Vec<u32>,
    ages: Vec<u32>,
    metabolic_rates: Vec<u32>,

    pub count: usize,
}

impl Agents {
    pub fn new(agents: &AgentParams) -> Agents {
        let mut wealths: Vec<u32> = Vec::with_capacity(agents.count);
        let mut visions: Vec<u32> = Vec::with_capacity(agents.count);
        let mut ages: Vec<u32> = Vec::with_capacity(agents.count);
        let mut metabolic_rates: Vec<u32> = Vec::with_capacity(agents.count);

        for (attribute, distribution) in [
            (&mut wealths, &agents.wealth_distribution),
            (&mut visions, &agents.vision_distribution),
            (&mut ages, &agents.max_age_distribution),
            (&mut metabolic_rates, &agents.metabolic_rate_distribution),
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

        Agents {
            wealths,
            visions,
            ages,
            metabolic_rates,
            count: agents.count,
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
}
