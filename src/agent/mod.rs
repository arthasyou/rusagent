pub mod context;
pub mod core;
pub mod execution;
pub mod memory;
pub mod planning;
pub mod state;
pub mod types;
pub mod verification;

// 多Agent支持
pub mod multi;
pub mod agents;
pub mod shared;

// 单Agent导出（保持向后兼容）
pub use core::Agent;
pub use context::*;
pub use execution::Executor;
pub use memory::*;
pub use planning::{AgentPlan, AgentStep, Planner};
pub use state::*;
pub use verification::*;

// 多Agent导出
pub use multi::{AgentManager, AgentManagerConfig, Message, MessageBus};
pub use shared::{GlobalContext, MemoryPool, SharedMemory};
