use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    agent::{
        context::AgentContext,
        types::{AgentCapability, AgentType},
    },
    error::Result,
    multi_agent::communication::message::Message,
    shared::GlobalContext,
};

/// Base Agent behavior trait that all Agent types must implement
#[async_trait]
pub trait AgentBehavior: Send + Sync {
    /// Get the unique identifier of the Agent
    fn get_id(&self) -> &str;

    /// Get the type of the Agent
    fn get_type(&self) -> AgentType;

    /// Get the list of Agent capabilities
    fn get_capabilities(&self) -> &[AgentCapability];

    /// Initialize the Agent
    async fn initialize(&mut self, context: Arc<GlobalContext>) -> Result<()>;

    /// Process received messages
    async fn process_message(&mut self, message: Message) -> Result<Option<Message>>;

    /// Execute the Agent's main loop (if any)
    async fn run(&mut self) -> Result<()> {
        Ok(())
    }

    /// Gracefully shut down the Agent
    async fn shutdown(&mut self) -> Result<()>;

    /// Get the health status of the Agent
    fn is_healthy(&self) -> bool {
        true
    }

    /// Get a status snapshot of the Agent (for monitoring and debugging)
    fn get_status(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.get_id(),
            "type": self.get_type(),
            "healthy": self.is_healthy()
        })
    }
}

/// Base Agent struct providing common functionality
#[derive(Debug, Clone)]
pub struct BaseAgent {
    pub id: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<AgentCapability>,
    pub context: Option<Arc<GlobalContext>>,
    pub local_context: AgentContext,
}

impl BaseAgent {
    pub fn new(id: String, agent_type: AgentType, capabilities: Vec<AgentCapability>) -> Self {
        Self {
            id,
            agent_type,
            capabilities,
            context: None,
            local_context: AgentContext::default(),
        }
    }

    /// Generate a new Agent ID
    pub fn generate_id(agent_type: &AgentType) -> String {
        format!("{}-{}", agent_type, uuid::Uuid::new_v4().simple())
    }
}
