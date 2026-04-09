use crate::config::AgentParams;

use super::Model;

impl Model {
    pub(super) fn init_visions(_agents: &AgentParams, cells: usize) -> Vec<i32> {
        vec![-1; cells]
    }
}
