use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Executing,
    Done,
    Failed,
}

impl Default for StepStatus {
    fn default() -> Self {
        StepStatus::Pending
    }
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub output: String,
    pub success: bool,
}

#[derive(Debug)]
pub enum AgentError {
    ExecutionError(String),
    VerificationError(String),
    PlanExhausted,
}
