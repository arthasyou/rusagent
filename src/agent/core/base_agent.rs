use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    agent::{
        context::AgentContext,
        types::{AgentCapability, AgentType},
    },
    error::Result,
    multi_agent::communication::message::Message,
    shared::GlobalContext,
};

/// 基础Agent行为trait，所有Agent类型都必须实现这个trait
#[async_trait]
pub trait AgentBehavior: Send + Sync {
    /// 获取Agent的唯一标识符
    fn get_id(&self) -> &str;

    /// 获取Agent的类型
    fn get_type(&self) -> AgentType;

    /// 获取Agent的能力列表
    fn get_capabilities(&self) -> &[AgentCapability];

    /// 初始化Agent
    async fn initialize(&mut self, context: Arc<GlobalContext>) -> Result<()>;

    /// 处理接收到的消息
    async fn process_message(&mut self, message: Message) -> Result<Option<Message>>;

    /// 执行Agent的主循环（如果有的话）
    async fn run(&mut self) -> Result<()> {
        Ok(())
    }

    /// 优雅地关闭Agent
    async fn shutdown(&mut self) -> Result<()>;

    /// 获取Agent的健康状态
    fn is_healthy(&self) -> bool {
        true
    }

    /// 获取Agent的状态快照（用于监控和调试）
    fn get_status(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.get_id(),
            "type": self.get_type(),
            "healthy": self.is_healthy()
        })
    }
}

/// 基础Agent结构体，提供通用功能
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

    /// 生成新的Agent ID
    pub fn generate_id(agent_type: &AgentType) -> String {
        format!("{}-{}", agent_type, uuid::Uuid::new_v4().simple())
    }
}
