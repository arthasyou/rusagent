use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use crate::agent::{
    core::base_agent::{AgentBehavior, BaseAgent},
    execution::Executor,
    types::{AgentCapability, AgentStatus, AgentType},
};
use crate::multi_agent::communication::{Message, MessageType};
use crate::shared::GlobalContext;
use crate::error::Result;

/// Executor Agent responsible for executing specific tasks
pub struct ExecutorAgent {
    base: BaseAgent,
    executor: Executor,
    current_task: Option<serde_json::Value>,
    capabilities: Vec<AgentCapability>,
}

impl ExecutorAgent {
    pub fn new(id: Option<String>, capabilities: Vec<AgentCapability>) -> Self {
        let id = id.unwrap_or_else(|| BaseAgent::generate_id(&AgentType::Executor));
        
        // Add basic execution capabilities
        let mut all_capabilities = vec![AgentCapability::TaskExecution];
        all_capabilities.extend(capabilities);

        Self {
            base: BaseAgent::new(id, AgentType::Executor, all_capabilities.clone()),
            executor: Executor,
            current_task: None,
            capabilities: all_capabilities,
        }
    }

    /// Create an ExecutorAgent that can call specific tools
    pub fn with_tool_capability(id: Option<String>, tool_name: String) -> Self {
        Self::new(id, vec![AgentCapability::ToolCalling(tool_name)])
    }

    /// Execute task
    async fn execute_task(&mut self, task: serde_json::Value) -> Result<serde_json::Value> {
        info!("ExecutorAgent {} executing task", self.base.id);
        
        // Extract step information from task
        if let Some(step) = task.get("step") {
            // Construct AgentStep object
            if let Ok(agent_step) = serde_json::from_value::<crate::agent::planning::AgentStep>(step.clone()) {
                // Execute using existing Executor
                let result = self.executor.execute(
                    &agent_step,
                    &self.base.local_context,
                    &crate::agent::memory::Memory::default(),
                ).await?;

                Ok(serde_json::json!({
                    "step_id": agent_step.step_id,
                    "output": result.output,
                    "success": result.success,
                }))
            } else {
                Err(crate::error::Error::AgentError(crate::error::agent_error::AgentError::ParseError(
                    "Invalid step format".into()
                )))
            }
        } else {
            // Simple task execution logic
            Ok(serde_json::json!({
                "result": "Task executed successfully",
                "executor_id": self.base.id,
            }))
        }
    }

    /// Handle task assignment
    async fn handle_task_assignment(&mut self, message: Message) -> Result<Message> {
        let task = message.payload;
        
        // Update status to busy
        self.current_task = Some(task.clone());
        
        // Send status update
        let _status_update = Message::new(
            self.base.id.clone(),
            message.sender_id.clone().into(),
            MessageType::StatusUpdate,
            serde_json::json!({
                "agent_id": self.base.id,
                "status": AgentStatus::Busy,
                "task_id": task.get("id").and_then(|v| v.as_str()).unwrap_or("unknown"),
            }),
        );

        // Execute task
        match self.execute_task(task.clone()).await {
            Ok(result) => {
                self.current_task = None;
                
                // Return execution result
                Ok(Message::new(
                    self.base.id.clone(),
                    message.sender_id.clone().into(),
                    MessageType::ResultNotification,
                    serde_json::json!({
                        "task_id": task.get("id").and_then(|v| v.as_str()).unwrap_or("unknown"),
                        "status": "completed",
                        "result": result,
                    }),
                ))
            }
            Err(e) => {
                self.current_task = None;
                
                // Return error
                Ok(Message::new(
                    self.base.id.clone(),
                    message.sender_id.clone().into(),
                    MessageType::Error,
                    serde_json::json!({
                        "task_id": task.get("id").and_then(|v| v.as_str()).unwrap_or("unknown"),
                        "error": e.to_string(),
                    }),
                ))
            }
        }
    }
}

#[async_trait]
impl AgentBehavior for ExecutorAgent {
    fn get_id(&self) -> &str {
        &self.base.id
    }

    fn get_type(&self) -> AgentType {
        self.base.agent_type
    }

    fn get_capabilities(&self) -> &[AgentCapability] {
        &self.capabilities
    }

    async fn initialize(&mut self, context: Arc<GlobalContext>) -> Result<()> {
        self.base.context = Some(context);
        info!("ExecutorAgent {} initialized with capabilities: {:?}", 
              self.base.id, self.capabilities);
        Ok(())
    }

    async fn process_message(&mut self, message: Message) -> Result<Option<Message>> {
        debug!("ExecutorAgent {} processing message: {:?}", 
               self.base.id, message.message_type);

        match &message.message_type {
            MessageType::TaskAssignment => {
                let response = self.handle_task_assignment(message).await?;
                Ok(Some(response))
            }
            MessageType::Control(cmd) => {
                // Handle control command
                debug!("ExecutorAgent received control command: {:?}", cmd);
                Ok(None)
            }
            _ => {
                debug!("ExecutorAgent ignoring message type: {:?}", message.message_type);
                Ok(None)
            }
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("ExecutorAgent {} shutting down", self.base.id);
        
        // If there's a task being executed, record it
        if let Some(task) = &self.current_task {
            info!("ExecutorAgent {} was executing task: {:?}", self.base.id, task);
        }
        
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        // Can add more complex health check logic
        true
    }

    fn get_status(&self) -> serde_json::Value {
        let status = if self.current_task.is_some() {
            AgentStatus::Busy
        } else {
            AgentStatus::Idle
        };

        serde_json::json!({
            "id": self.base.id,
            "type": self.base.agent_type,
            "status": status,
            "healthy": self.is_healthy(),
            "capabilities": self.capabilities,
            "current_task": self.current_task.is_some(),
        })
    }
}