use std::io::{self, Write};

use mcp_client::registry::get_mcp_registry;
use serde_json::json;

use crate::{
    agent::{context::AgentContext, memory::Memory, planning::AgentStep, types::StepResult},
    error::agent_error::AgentError,
    tools::model::TOOL_REGISTRY,
};

#[derive(Debug, Default, Clone)]
pub struct Executor;

impl Executor {
    async fn call_mcp_tool(
        &self,
        tool_name: &str,
        parameters: &Option<serde_json::Value>,
    ) -> Result<serde_json::Value, AgentError> {
        println!("🛠️ 调用 MCP 工具: {}", tool_name);
        
        // 1. 从 TOOL_REGISTRY 获取工具信息和对应的 MCP 服务器
        let mcp_server_name = {
            let tool_registry = TOOL_REGISTRY.read().unwrap();
            match tool_registry.get(tool_name) {
                Some(tool_info) => {
                    if tool_info.mcp_server.is_empty() {
                        return Err(AgentError::ExecutionError(
                            format!("工具 '{}' 不是 MCP 工具", tool_name)
                        ));
                    }
                    tool_info.mcp_server.clone()
                },
                None => {
                    return Err(AgentError::ExecutionError(
                        format!("未找到工具: {}", tool_name)
                    ));
                }
            }
        };

        // 2. 通过 MCP 服务器名称获取对应的客户端
        let registry = get_mcp_registry();
        let client = registry.get(&mcp_server_name).map_err(|e| {
            AgentError::ExecutionError(format!(
                "无法获取 MCP 服务器 '{}' 的客户端: {}", 
                mcp_server_name, e
            ))
        })?;

        // 3. 构造工具调用参数，按照你提供的格式
        let tool_call_params = match parameters {
            Some(params) => {
                // 如果 parameters 已经包含了 name 和 arguments，直接使用
                if params.get("name").is_some() && params.get("arguments").is_some() {
                    params.clone()
                } else {
                    // 否则包装成标准格式
                    json!({
                        "name": tool_name,
                        "arguments": params
                    })
                }
            }
            None => {
                json!({
                    "name": tool_name,
                    "arguments": {}
                })
            }
        };

        println!("🔧 调用 MCP 服务器 '{}' 的工具 '{}', 参数: {:?}", 
                 mcp_server_name, tool_name, tool_call_params);

        // 4. 调用 MCP 工具
        match client.call_tool(tool_call_params).await {
            Ok(response) => {
                println!("✅ MCP工具调用成功: {:?}", response);
                // 尝试解析响应为JSON
                match response {
                    mcp_client::core::protocol::message::JsonRpcMessage::Response(resp) => {
                        Ok(resp.result.unwrap_or(json!({"success": true})))
                    }
                    _ => Ok(json!({"result": "工具执行完成"})),
                }
            }
            Err(e) => Err(AgentError::ExecutionError(format!(
                "MCP工具 '{}' 调用失败: {}",
                tool_name, e
            ))),
        }
    }

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

                    // 实际调用 MCP 工具
                    match self.call_mcp_tool(tool_name, &step.parameters).await {
                        Ok(result) => Ok(StepResult {
                            output: result.to_string(),
                            success: true,
                        }),
                        Err(e) => {
                            println!("❌ MCP工具调用失败: {:?}", e);
                            // 降级为模拟输出
                            let simulated_output = json!({
                                "result": format!("{} 工具执行失败，使用模拟结果", tool_name),
                                "error": e.to_string(),
                            });
                            Ok(StepResult {
                                output: simulated_output.to_string(),
                                success: false,
                            })
                        }
                    }
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
