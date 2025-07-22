use std::collections::HashMap;

use crate::agent::planning::AgentStep;

#[derive(Debug, Default, Clone)]
pub struct Memory {
    pub history: Vec<String>,
    pub step_cache: HashMap<String, AgentStep>,
}

impl Memory {
    pub fn record_step(&mut self, step_id: &str, step: &AgentStep) {
        self.step_cache.insert(step_id.to_string(), step.clone());
    }

    pub fn log(&mut self, message: String) {
        self.history.push(message);
    }
}
