pub mod communication;
pub mod coordination;
pub mod manager;
pub mod registry;

// Re-export commonly used types
pub use communication::{Message, MessageBus, MessageBusConfig, MessageReceiver, MessageType};
pub use manager::{AgentManager, AgentManagerConfig};
pub use registry::{AgentInfo, AgentRegistry, RegistryConfig};