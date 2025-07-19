use serde_json::json;

use crate::agent::{
    context::AgentContext,
    momory::Memory,
    plan::AgentStep,
    types::{AgentError, StepResult},
};

#[derive(Debug, Default, Clone)]
pub struct Executor;

impl Executor {
    pub async fn execute(
        &self,
        step: &AgentStep,
        context: &AgentContext,
        memory: &Memory,
    ) -> Result<StepResult, AgentError> {
        match step.action.as_str() {
            "call_tool" => {
                if let Some(tool_name) = &step.tool {
                    println!("🛠️ 调用工具 [{}]，参数: {:?}", tool_name, step.parameters);

                    // TODO: 实际调用 MCP 工具（暂时模拟）
                    let simulated_output = json!({
                        "result": format!("{} 工具执行成功", tool_name),
                    });

                    Ok(StepResult {
                        output: simulated_output.to_string(),
                        success: true,
                    })
                } else {
                    Err(AgentError::ExecutionError("缺少 tool 字段".to_string()))
                }
            }

            "ask_user" => {
                println!("🧑 等待用户回答: {:?}", step.input);

                // 模拟用户交互输入
                let fake_answer = json!({
                    "answer": "模拟用户回答：中医基础理论",
                });

                Ok(StepResult {
                    output: fake_answer.to_string(),
                    success: true,
                })
            }

            other => Err(AgentError::ExecutionError(format!(
                "未知动作类型: {}",
                other
            ))),
        }
    }
}
