use std::collections::HashMap;

use crate::agent::types::{StepResult, StepStatus};

#[derive(Debug, Default, Clone)]
pub struct AgentState {
    pub step_status: HashMap<String, StepStatus>,
    pub step_results: HashMap<String, StepResult>,
}

impl AgentState {
    pub fn set_step_status(&mut self, step_id: usize, status: StepStatus) {
        self.step_status.insert(step_id.to_string(), status);
    }

    pub fn append_result(&mut self, step_id: usize, result: StepResult) {
        self.step_results.insert(step_id.to_string(), result);
    }

    pub fn get_step_status(&self, step_id: usize) -> Option<&StepStatus> {
        self.step_status.get(&step_id.to_string())
    }
}
