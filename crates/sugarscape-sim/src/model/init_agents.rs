use rand::seq::IteratorRandom;
use std::collections::HashSet;

use crate::config::AgentParams;

use super::Model;

impl Model {
    pub(super) fn init_agents(
        agents: &AgentParams,
        num_cells: usize,
    ) -> (Vec<i32>, Vec<i32>, Vec<i32>, HashSet<usize>) {
    
        let mut empty_cells = HashSet::with_capacity(num_cells - agents.count as usize);
        let range = 0..(num_cells - 1);
        for i in range.sample(&mut rand::rng(), num_cells - agents.count as usize) {
            empty_cells.insert(i);
        }

        let mut wealths = vec![-1; num_cells];
        let mut visions = vec![-1; num_cells];
        let mut ages = vec![-1; num_cells];

        for i in range {
            if empty_cells.contains(&i) {
                continue;
            }
            wealths[i] = ...
        }
        (wealths, visions, ages, empty_cells)
    }
}

#[cfg(test)]
mod tests {
    use p_test::p_test;

    use crate::config::{AgentParams, RandomDistribution, WorldParams};

    use super::super::Model;

    // #[p_test((5, 5, 10, 15), (2, 3, 5, 1), (2, 2, 4, 0))]
    #[p_test((5, 5, 10, 15))]
    fn number_of_empty_cells_is_correct(
        width: u8,
        height: u8,
        num_agents: u32,
        num_empty_cells: usize,
    ) {
        let model = Model::from_default(|w, a| {
            (
                WorldParams { width, height, ..w },
                AgentParams {
                    count: num_agents,
                    ..a
                },
            )
        });
        let empty_cells = model.empty_cells;
        assert_eq!(empty_cells.len(), num_empty_cells);
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
