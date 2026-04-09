use crate::config::AgentParams;

use super::Model;

impl Model {
    pub(super) fn init_wealths(_agents: &AgentParams, cells: usize) -> Vec<i32> {
        vec![-1; cells]
    }
}
#[cfg(test)]
mod tests {
    use crate::config::{AgentParams, RandomDistribution};

    use super::super::Model;

    #[test]
    fn distribution_is_within_bounds() {
        let model = Model::from_default(|w, a| {
            (
                w,
                AgentParams {
                    wealth_distribution: RandomDistribution::Uniform { min: 0, max: 100 },
                    ..a
                },
            )
        });
        let wealths = model.wealths;
        assert!(wealths.iter().all(|w| (&0..=&100).contains(&w)));
    }
}
