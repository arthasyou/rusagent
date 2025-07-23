use rusagent::agent::{
    Agent,
    planning::{AgentPlan, AgentStep},
    types::StepStatus,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("🚀 测试 rusagent 基础执行流程（跳过LLM）...");

    // 手动构造一个测试计划
    let plan = AgentPlan {
        plan_id: "test-plan-001".to_string(),
        description: Some("中医气虚资料收集测试".to_string()),
        version: Some("1.0".to_string()),
        steps: vec![
            AgentStep {
                step_id: 1,
                description: "询问用户需要哪些具体的气虚信息".to_string(),
                status: StepStatus::Pending,
                action: "ask_user".to_string(),
                tool: None,
                parameters: None,
                input: Some(json!({
                    "question": "请问您希望了解气虚的哪些方面？症状、治疗方法还是饮食调理？"
                })),
                output: None,
                is_succeeded: false,
                error_code: None,
                error_reason: None,
            },
            AgentStep {
                step_id: 2,
                description: "使用网络搜索工具搜索相关资料".to_string(),
                status: StepStatus::Pending,
                action: "call_tool".to_string(),
                tool: Some("fetch_url".to_string()),
                parameters: Some(json!({
                    "url": "https://example.com/chinese-medicine/qi-deficiency"
                })),
                input: None,
                output: None,
                is_succeeded: false,
                error_code: None,
                error_reason: None,
            },
            AgentStep {
                step_id: 3,
                description: "总结收集到的资料".to_string(),
                status: StepStatus::Pending,
                action: "call_tool".to_string(),
                tool: Some("summarize_text".to_string()),
                parameters: Some(json!({
                    "text": "将在步骤2中获取的内容进行摘要"
                })),
                input: None,
                output: None,
                is_succeeded: false,
                error_code: None,
                error_reason: None,
            },
        ],
        is_succeeded: false,
        error_step_id: None,
    };

    println!("📋 测试计划: {}", plan.description.as_ref().unwrap());
    println!("🔢 总共 {} 个步骤", plan.steps.len());

    // 创建 Agent (使用MCP)
    println!("🤖 正在创建 Agent 并初始化 MCP...");
    let mut agent = Agent::new(plan);
    println!("✅ Agent 创建成功，ID: {}", agent.id);

    // 执行任务
    println!("🏃 开始执行任务...");
    match agent.run_loop().await {
        Ok(_) => println!("🎉 所有步骤执行完成！"),
        Err(e) => println!("❌ 任务执行中断: {}", e),
    }

    // 打印最终状态
    println!("\n📊 任务执行总结:");
    for step in &agent.plan.steps {
        let status_icon = match step.status {
            StepStatus::Done => "✅",
            StepStatus::Failed => "❌",
            StepStatus::Executing => "🔄",
            StepStatus::Pending => "⏳",
        };
        println!(
            "  {} Step {}: {}",
            status_icon, step.step_id, step.description
        );
    }
}
