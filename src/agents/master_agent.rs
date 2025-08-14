use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use crate::agent::{
    core::base_agent::{AgentBehavior, BaseAgent},
    types::{AgentCapability, AgentType, TaskStatus},
};
use crate::multi_agent::{
    communication::{Message, MessageType},
    coordination::task_queue::{Task, TaskQueue},
};
use crate::shared::GlobalContext;
use crate::error::Result;

/// Master Agent responsible for task distribution and global coordination
pub struct MasterAgent {
    base: BaseAgent,
    task_queue: Arc<TaskQueue>,
    active_tasks: std::collections::HashMap<String, Task>,
}

impl MasterAgent {
    pub fn new(id: Option<String>) -> Self {
        let id = id.unwrap_or_else(|| BaseAgent::generate_id(&AgentType::Master));
        let capabilities = vec![
            AgentCapability::Coordination,
            AgentCapability::TaskPlanning,
        ];

        Self {
            base: BaseAgent::new(id, AgentType::Master, capabilities),
            task_queue: Arc::new(TaskQueue::new()),
            active_tasks: std::collections::HashMap::new(),
        }
    }

    /// Decompose tasks
    async fn decompose_task(&self, task: &Task) -> Result<Vec<Task>> {
        // TODO: Implement intelligent task decomposition logic
        // Simply return the original task here
        Ok(vec![task.clone()])
    }

    /// Assign task to suitable Agent
    #[allow(dead_code)]
    async fn assign_task(&mut self, task: &mut Task, agent_id: String) -> Result<()> {
        task.assigned_to = Some(agent_id.clone());
        task.status = TaskStatus::Assigned(agent_id);
        
        info!("Task {} assigned to agent {}", task.id, task.assigned_to.as_ref().unwrap());
        Ok(())
    }

    /// Handle task assignment request
    async fn handle_task_assignment(&mut self, payload: serde_json::Value) -> Result<Message> {
        // Parse task from payload
        let task: Task = serde_json::from_value(payload.clone())
            .map_err(|e| crate::error::agent_error::AgentError::ParseError(e.to_string()))?;

        // Add to task queue
        self.task_queue.enqueue(task.clone()).await?;
        
        // Decompose task
        let subtasks = self.decompose_task(&task).await?;
        
        // TODO: Find suitable executor and assign task
        
        // Return confirmation message
        Ok(Message::new(
            self.base.id.clone(),
            Some(task.created_by.clone()),
            MessageType::ResultNotification,
            serde_json::json!({
                "task_id": task.id,
                "status": "accepted",
                "subtasks": subtasks.len()
            }),
        ))
    }

    /// Handle status update
    async fn handle_status_update(&mut self, payload: serde_json::Value) -> Result<Option<Message>> {
        if let Some(task_id) = payload.get("task_id").and_then(|v| v.as_str())
            && let Some(status) = payload.get("status") {
                // Update task status
                if let Ok(task_status) = serde_json::from_value::<TaskStatus>(status.clone()) {
                    info!("Task {} status updated to {:?}", task_id, task_status);
                    
                    // If task completed, remove from active tasks
                    if matches!(task_status, TaskStatus::Completed | TaskStatus::Failed(_)) {
                        self.active_tasks.remove(task_id);
                    }
                }
            }
        Ok(None)
    }
}

#[async_trait]
impl AgentBehavior for MasterAgent {
    fn get_id(&self) -> &str {
        &self.base.id
    }

    fn get_type(&self) -> AgentType {
        self.base.agent_type
    }

    fn get_capabilities(&self) -> &[AgentCapability] {
        &self.base.capabilities
    }

    async fn initialize(&mut self, context: Arc<GlobalContext>) -> Result<()> {
        self.base.context = Some(context);
        info!("MasterAgent {} initialized", self.base.id);
        Ok(())
    }

    async fn process_message(&mut self, message: Message) -> Result<Option<Message>> {
        debug!("MasterAgent {} processing message: {:?}", self.base.id, message.message_type);

        match &message.message_type {
            MessageType::TaskAssignment => {
                let response = self.handle_task_assignment(message.payload).await?;
                Ok(Some(response))
            }
            MessageType::StatusUpdate => {
                self.handle_status_update(message.payload).await
            }
            MessageType::ResourceRequest => {
                // TODO: Handle resource request
                Ok(None)
            }
            _ => {
                debug!("MasterAgent ignoring message type: {:?}", message.message_type);
                Ok(None)
            }
        }
    }

    async fn run(&mut self) -> Result<()> {
        info!("MasterAgent {} starting main loop", self.base.id);
        
        // Main loop: process task queue
        loop {
            // Check if there are tasks pending assignment
            if let Some(task) = self.task_queue.dequeue().await {
                info!("Processing task: {}", task.id);
                
                // TODO: Implement intelligent task assignment logic
                // 1. Find suitable Agent
                // 2. Send task assignment message
                // 3. Track task status
                
                self.active_tasks.insert(task.id.clone(), task);
            }
            
            // Avoid busy waiting
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("MasterAgent {} shutting down", self.base.id);
        
        // Save pending tasks
        let pending_tasks = self.task_queue.get_all_pending().await;
        if !pending_tasks.is_empty() {
            info!("Saving {} pending tasks", pending_tasks.len());
            // TODO: Persist unfinished tasks
        }
        
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn get_status(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.base.id,
            "type": self.base.agent_type,
            "healthy": self.is_healthy(),
            "active_tasks": self.active_tasks.len(),
            "queued_tasks": self.task_queue.size()
        })
    }
}