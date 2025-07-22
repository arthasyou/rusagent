// src/agent/types.rs
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("执行失败: {0}")]
    ExecutionError(String),

    #[error("验证失败: {0}")]
    VerificationError(String),

    #[error("计划已耗尽")]
    PlanExhausted,

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ModelError(#[from] model_gateway_rs::error::Error),
}
