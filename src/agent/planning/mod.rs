pub mod planner;
mod step;

pub use planner::Planner;
use serde::{Deserialize, Serialize};
pub use step::AgentStep;

use crate::agent::{state::AgentState, types::StepStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlan {
    pub plan_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    pub steps: Vec<AgentStep>,

    #[serde(default)]
    pub is_succeeded: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_step_id: Option<usize>,
}

impl AgentPlan {
    // pub fn new(plan_id: String, steps: Vec<AgentStep>) -> Self {
    //     Self {
    //         plan_id,
    //         description: None,
    //         version: None,
    //         steps,
    //         is_succeeded: false,
    //         error_step_id: None,
    //     }
    // }

    pub fn next_pending_step(&self, state: &AgentState) -> Option<&AgentStep> {
        self.steps.iter().find(|step| {
            match state.get_step_status(step.step_id) {
                Some(StepStatus::Pending) | None => true, // 默认视为 Pending
                _ => false,
            }
        })
    }
}
