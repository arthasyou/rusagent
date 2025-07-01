use std::sync::Arc;

use mcp_client::client::McpClient;
use mcp_transport::client::impls::sse::SseTransport;
use serde_json::Value;

use crate::{
    agent::task::{TaskPlan, TaskResult, TaskStepResult},
    error::Result,
};

pub struct Executor {
    mcp_client: Arc<McpClient<SseTransport>>,
}

impl Executor {
    pub fn new(mcp_client: Arc<McpClient<SseTransport>>) -> Self {
        Self { mcp_client }
    }

    pub async fn run_plan(&self, plan: TaskPlan) -> Result<TaskResult> {
        let mut result = TaskResult::default();

        for step in plan.steps {
            // let output = self.mcp_client.execute(&step.tool, &step.params).await?;
            result.steps.push(TaskStepResult {
                tool: step.tool,
                output: Value::Null,
                tool_success: todo!(),
                eval_success: todo!(),
                reason: todo!(), // Placeholder for actual output
            });
        }

        Ok(result)
    }
}
