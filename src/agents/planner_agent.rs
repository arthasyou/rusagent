use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};
use model_gateway_rs::sdk::openai::OpenAiSdk;

use crate::agent::{
    core::base_agent::{AgentBehavior, BaseAgent},
    planning::{AgentPlan, Planner},
    types::{AgentCapability, AgentType},
};
use crate::multi_agent::communication::{Message, MessageType};
use crate::shared::GlobalContext;
use crate::error::Result;
use crate::input::model::UserTaskInput;

/// 规划Agent，负责生成执行计划
pub struct PlannerAgent {
    base: BaseAgent,
    planner: Planner<OpenAiSdk>,
}

impl PlannerAgent {
    pub fn new(id: Option<String>) -> Self {
        let id = id.unwrap_or_else(|| BaseAgent::generate_id(&AgentType::Planner));
        let capabilities = vec![AgentCapability::TaskPlanning];

        Self {
            base: BaseAgent::new(id, AgentType::Planner, capabilities),
            planner: Planner::default(),
        }
    }

    /// 处理规划请求
    async fn handle_planning_request(&mut self, payload: serde_json::Value) -> Result<Message> {
        info!("PlannerAgent {} handling planning request", self.base.id);

        // 解析用户输入
        let user_input = if let Ok(input) = serde_json::from_value::<UserTaskInput>(payload.clone()) {
            input
        } else {
            // 尝试从payload构建UserTaskInput
            let goal = payload.get("task")
                .or_else(|| payload.get("goal"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown task");

            let content = payload.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let description = payload.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            UserTaskInput {
                goal: goal.to_string(),
                content: content.to_string(),
                description,
                constraints: None,
                references: None,
            }
        };

        // 生成计划
        match self.planner.generate_plan(&user_input).await {
            Ok(llm_output) => {
                // 将整个LLM输出作为计划返回
                // TODO: 实现更智能的计划解析
                let plan_json = serde_json::json!({
                    "plan": format!("{:?}", llm_output), // 临时方案：将输出转为字符串
                    "user_input": user_input,
                });

                Ok(Message::new(
                    self.base.id.clone(),
                    payload.get("requester_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    MessageType::ResultNotification,
                    serde_json::json!({
                        "status": "success",
                        "plan": plan_json,
                    }),
                ))
            }
            Err(e) => {
                error!("Planning failed: {:?}", e);
                Ok(Message::new(
                    self.base.id.clone(),
                    payload.get("requester_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    MessageType::Error,
                    serde_json::json!({
                        "error": format!("Planning failed: {}", e),
                    }),
                ))
            }
        }
    }

    /// 优化现有计划
    #[allow(dead_code)]
    async fn optimize_plan(&self, plan: &AgentPlan) -> Result<AgentPlan> {
        // TODO: 实现计划优化逻辑
        // 例如：并行化独立步骤、合并相似步骤等
        Ok(plan.clone())
    }
}

#[async_trait]
impl AgentBehavior for PlannerAgent {
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
        info!("PlannerAgent {} initialized", self.base.id);
        Ok(())
    }

    async fn process_message(&mut self, message: Message) -> Result<Option<Message>> {
        debug!("PlannerAgent {} processing message: {:?}", 
               self.base.id, message.message_type);

        match &message.message_type {
            MessageType::TaskAssignment => {
                // 如果任务是生成计划，则处理
                if let Some(task_type) = message.payload.get("task_type").and_then(|v| v.as_str()) {
                    if task_type == "planning" {
                        let response = self.handle_planning_request(message.payload).await?;
                        return Ok(Some(response));
                    }
                }
                Ok(None)
            }
            MessageType::Custom(msg_type) if msg_type == "PlanningRequest" => {
                let response = self.handle_planning_request(message.payload).await?;
                Ok(Some(response))
            }
            _ => {
                debug!("PlannerAgent ignoring message type: {:?}", message.message_type);
                Ok(None)
            }
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("PlannerAgent {} shutting down", self.base.id);
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
            "capabilities": self.base.capabilities,
        })
    }
}