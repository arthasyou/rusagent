use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use crate::agent::{
    core::base_agent::{AgentBehavior, BaseAgent},
    types::{AgentCapability, AgentType},
    verification::Verifier,
};
use crate::multi_agent::communication::{Message, MessageType};
use crate::shared::GlobalContext;
use crate::error::Result;

/// 验证Agent，负责验证任务执行结果
pub struct VerifierAgent {
    base: BaseAgent,
    verifier: Verifier,
    verification_rules: serde_json::Value,
}

impl VerifierAgent {
    pub fn new(id: Option<String>) -> Self {
        let id = id.unwrap_or_else(|| BaseAgent::generate_id(&AgentType::Verifier));
        let capabilities = vec![AgentCapability::TaskVerification];

        Self {
            base: BaseAgent::new(id, AgentType::Verifier, capabilities),
            verifier: Verifier::default(),
            verification_rules: serde_json::json!({}),
        }
    }

    /// 设置验证规则
    pub fn with_rules(mut self, rules: serde_json::Value) -> Self {
        self.verification_rules = rules;
        self
    }

    /// 处理验证请求
    async fn handle_verification_request(&mut self, message: Message) -> Result<Message> {
        info!("VerifierAgent {} handling verification request", self.base.id);

        let payload = &message.payload;
        
        // 提取需要验证的内容
        let task_id = payload.get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let result = payload.get("result")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        
        let step = payload.get("step");

        // 执行验证
        let verification_result = if let Some(step_value) = step {
            // 如果有步骤信息，使用现有的验证器
            if let Ok(agent_step) = serde_json::from_value::<crate::agent::planning::AgentStep>(step_value.clone()) {
                if let Ok(step_result) = serde_json::from_value::<crate::agent::types::StepResult>(result.clone()) {
                    match self.verifier.verify(
                        &agent_step,
                        &step_result,
                        &crate::agent::context::AgentContext::default(),
                        &crate::agent::memory::Memory::default(),
                    ) {
                        Ok(_) => serde_json::json!({
                            "valid": true,
                            "message": "Verification passed",
                        }),
                        Err(e) => serde_json::json!({
                            "valid": false,
                            "message": format!("Verification failed: {}", e),
                        }),
                    }
                } else {
                    serde_json::json!({
                        "valid": false,
                        "message": "Invalid result format",
                    })
                }
            } else {
                // 通用验证逻辑
                self.verify_with_rules(&result).await
            }
        } else {
            // 通用验证逻辑
            self.verify_with_rules(&result).await
        };

        // 返回验证结果
        Ok(Message::new(
            self.base.id.clone(),
            message.sender_id.clone().into(),
            MessageType::ResultNotification,
            serde_json::json!({
                "task_id": task_id,
                "verification_result": verification_result,
                "verifier_id": self.base.id,
            }),
        ))
    }

    /// 使用自定义规则验证
    async fn verify_with_rules(&self, result: &serde_json::Value) -> serde_json::Value {
        // TODO: 实现基于规则的验证逻辑
        // 这里可以根据verification_rules进行验证
        
        // 简单的验证示例
        if result.is_null() || (result.is_object() && result.as_object().unwrap().is_empty()) {
            serde_json::json!({
                "valid": false,
                "message": "Result is empty",
            })
        } else {
            serde_json::json!({
                "valid": true,
                "message": "Basic validation passed",
            })
        }
    }
}

#[async_trait]
impl AgentBehavior for VerifierAgent {
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
        info!("VerifierAgent {} initialized", self.base.id);
        Ok(())
    }

    async fn process_message(&mut self, message: Message) -> Result<Option<Message>> {
        debug!("VerifierAgent {} processing message: {:?}", 
               self.base.id, message.message_type);

        match &message.message_type {
            MessageType::TaskAssignment => {
                // 如果任务是验证，则处理
                if let Some(task_type) = message.payload.get("task_type").and_then(|v| v.as_str()) {
                    if task_type == "verification" {
                        let response = self.handle_verification_request(message).await?;
                        return Ok(Some(response));
                    }
                }
                Ok(None)
            }
            MessageType::Custom(msg_type) if msg_type == "VerificationRequest" => {
                let response = self.handle_verification_request(message).await?;
                Ok(Some(response))
            }
            _ => {
                debug!("VerifierAgent ignoring message type: {:?}", message.message_type);
                Ok(None)
            }
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("VerifierAgent {} shutting down", self.base.id);
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
            "has_custom_rules": !self.verification_rules.is_null(),
        })
    }
}