#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("执行失败: {0}")]
    ExecutionError(String),

    #[error("验证失败: {0}")]
    VerificationError(String),

    #[error("计划已耗尽")]
    PlanExhausted,

    #[error("Agent未找到: {0}")]
    AgentNotFound(String),

    #[error("任务未找到: {0}")]
    TaskNotFound(String),

    #[error("消息传递失败: {0}")]
    MessageDeliveryError(String),

    #[error("资源耗尽: {0}")]
    ResourceExhausted(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("内部错误: {0}")]
    InternalError(String),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ModelError(#[from] model_gateway_rs::error::Error),
}
