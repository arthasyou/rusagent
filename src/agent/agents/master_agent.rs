use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use crate::agent::{
    core::base_agent::{AgentBehavior, BaseAgent},
    multi::{
        communication::{Message, MessageType},
        coordination::task_queue::{Task, TaskQueue},
    },
    shared::GlobalContext,
    types::{AgentCapability, AgentType, TaskStatus},
};
use crate::error::Result;

/// 主控Agent，负责任务分配和全局协调
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

    /// 分解任务
    async fn decompose_task(&self, task: &Task) -> Result<Vec<Task>> {
        // TODO: 实现智能任务分解逻辑
        // 这里简单地返回原任务
        Ok(vec![task.clone()])
    }

    /// 分配任务给合适的Agent
    #[allow(dead_code)]
    async fn assign_task(&mut self, task: &mut Task, agent_id: String) -> Result<()> {
        task.assigned_to = Some(agent_id.clone());
        task.status = TaskStatus::Assigned(agent_id);
        
        info!("Task {} assigned to agent {}", task.id, task.assigned_to.as_ref().unwrap());
        Ok(())
    }

    /// 处理任务分配请求
    async fn handle_task_assignment(&mut self, payload: serde_json::Value) -> Result<Message> {
        // 从payload解析任务
        let task: Task = serde_json::from_value(payload.clone())
            .map_err(|e| crate::error::agent_error::AgentError::ParseError(e.to_string()))?;

        // 添加到任务队列
        self.task_queue.enqueue(task.clone()).await?;
        
        // 分解任务
        let subtasks = self.decompose_task(&task).await?;
        
        // TODO: 查找合适的执行者并分配任务
        
        // 返回确认消息
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

    /// 处理状态更新
    async fn handle_status_update(&mut self, payload: serde_json::Value) -> Result<Option<Message>> {
        if let Some(task_id) = payload.get("task_id").and_then(|v| v.as_str()) {
            if let Some(status) = payload.get("status") {
                // 更新任务状态
                if let Ok(task_status) = serde_json::from_value::<TaskStatus>(status.clone()) {
                    info!("Task {} status updated to {:?}", task_id, task_status);
                    
                    // 如果任务完成，从活动任务中移除
                    if matches!(task_status, TaskStatus::Completed | TaskStatus::Failed(_)) {
                        self.active_tasks.remove(task_id);
                    }
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
                // TODO: 处理资源请求
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
        
        // 主循环：处理任务队列
        loop {
            // 检查是否有待分配的任务
            if let Some(task) = self.task_queue.dequeue().await {
                info!("Processing task: {}", task.id);
                
                // TODO: 实现智能任务分配逻辑
                // 1. 查找合适的Agent
                // 2. 发送任务分配消息
                // 3. 跟踪任务状态
                
                self.active_tasks.insert(task.id.clone(), task);
            }
            
            // 避免忙等待
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("MasterAgent {} shutting down", self.base.id);
        
        // 保存未完成的任务
        let pending_tasks = self.task_queue.get_all_pending().await;
        if !pending_tasks.is_empty() {
            info!("Saving {} pending tasks", pending_tasks.len());
            // TODO: 持久化未完成的任务
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