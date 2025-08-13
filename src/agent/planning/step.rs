use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::types::StepStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub step_id: usize,

    pub description: String,

    #[serde(default)]
    pub status: StepStatus,

    pub action: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,

    #[serde(default)]
    pub is_succeeded: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
}

impl Default for AgentStep {
    fn default() -> Self {
        Self {
            step_id: 0,
            description: String::new(),
            status: StepStatus::default(),
            action: String::new(),
            tool: None,
            parameters: None,
            input: None,
            output: None,
            is_succeeded: false,
            error_code: None,
            error_reason: None,
        }
    }
}
