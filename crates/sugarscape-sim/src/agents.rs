use crate::config::AgentParams;

#[derive(Debug)]
pub struct Agents {
    wealths: Vec<i32>,
    visions: Vec<i32>,
    ages: Vec<i32>,
}

impl Agents {
    pub fn new(agents: &AgentParams) -> Agents {
        let wealths = vec![-1; agents.count];
        let visions = vec![-1; agents.count];
        let ages = vec![-1; agents.count];

        Agents {
            wealths,
            visions,
            ages,
        }
    }
}

#[cfg(test)]
mod tests {
    use p_test::p_test;

    use crate::{config::AgentParams, Agents};

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
    }

    // #[test]
    // fn wealth_distribution_is_within_bounds() {
    //     let model = Model::from_default(|w, a| {
    //         (
    //             w,
    //             AgentParams {
    //                 wealth_distribution: RandomDistribution::Uniform { min: 0, max: 100 },
    //                 ..a
    //             },
    //         )
    //     });
    //     let wealths = model.wealths;
    //     assert!(wealths.iter().all(|w| (&0..=&100).contains(&w)));
    // }
}
