use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskPlan {
    pub steps: Vec<TaskStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStep {
    pub tool: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TaskResult {
    pub steps: Vec<TaskStepResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStepResult {
    pub tool: String,
    pub tool_success: bool,
    pub eval_success: Option<bool>,
    pub reason: Option<String>,
    pub output: serde_json::Value,
}
