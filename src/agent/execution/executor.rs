use std::io::{self, Write};

use serde_json::json;

use crate::{
    agent::{context::AgentContext, memory::Memory, planning::AgentStep, types::StepResult},
    error::agent_error::AgentError,
};

#[derive(Debug, Default, Clone)]
pub struct Executor;

impl Executor {
    pub async fn execute(
        &self,
        step: &AgentStep,
        // TODO: 添加 AgentContext 和 Memory 参数
        _context: &AgentContext,
        _memory: &Memory,
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

                // 提示问题
                if let Some(input) = &step.input {
                    if let Some(question) = input.get("question").and_then(|v| v.as_str()) {
                        println!("❓ {}", question);
                    }
                }

                print!("👉 请输入你的回答：");
                io::stdout().flush().unwrap(); // 确保立即输出提示

                let mut user_input = String::new();
                io::stdin()
                    .read_line(&mut user_input)
                    .map_err(|e| AgentError::ExecutionError(format!("读取用户输入失败: {}", e)))?;
                let user_input = user_input.trim(); // 去除换行符

                // 构造返回结果
                let answer_json = json!({ "answer": user_input });

                Ok(StepResult {
                    output: answer_json.to_string(),
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
