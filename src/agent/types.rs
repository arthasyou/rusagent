use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StepStatus {
    #[default]
    Pending,
    Executing,
    Done,
    Failed,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub output: String,
    pub success: bool,
}

// ===== Multi-Agent related type definitions =====

/// Agent lifecycle state
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

/// Agent type
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
            AgentType::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Agent capability definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    TaskPlanning,
    TaskExecution,
    TaskVerification,
    Monitoring,
    Coordination,
    ToolCalling(String), // Specific tool name
    Custom(String),
}

/// Task type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Planning,
    Execution,
    Verification,
    Analysis,
    Monitoring,
    Composite,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum Priority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}


/// Task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskStatus {
    #[default]
    Pending,
    Assigned(String), // Agent ID
    InProgress,
    Completed,
    Failed(String), // Failure reason
    Cancelled,
}


/// Agent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Busy,
    Idle,
    Offline,
    Failed,
}

/// Access level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,
    Private,
    Shared,
}

/// Runtime mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeMode {
    SingleAgent,
    MultiAgent,
}
