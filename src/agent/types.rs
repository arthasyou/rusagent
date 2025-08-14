use std::fmt;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub output: String,
    pub success: bool,
}

// ===== 多Agent相关类型定义 =====

/// Agent生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentLifecycleState {
    Created,
    Initializing,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
}

/// Agent类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Master,
    Planner,
    Executor,
    Verifier,
    Monitor,
    Custom(&'static str),
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentType::Master => write!(f, "master"),
            AgentType::Planner => write!(f, "planner"),
            AgentType::Executor => write!(f, "executor"),
            AgentType::Verifier => write!(f, "verifier"),
            AgentType::Monitor => write!(f, "monitor"),
            AgentType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Agent能力定义
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    TaskPlanning,
    TaskExecution,
    TaskVerification,
    Monitoring,
    Coordination,
    ToolCalling(String), // 具体的工具名称
    Custom(String),
}

/// 任务类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Planning,
    Execution,
    Verification,
    Analysis,
    Monitoring,
    Composite,
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned(String), // Agent ID
    InProgress,
    Completed,
    Failed(String), // 失败原因
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// Agent状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Busy,
    Idle,
    Offline,
    Failed,
}

/// 访问级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,
    Private,
    Shared,
}

/// 运行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeMode {
    SingleAgent,
    MultiAgent,
}
