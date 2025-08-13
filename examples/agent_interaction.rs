use std::sync::Arc;

use rusagent::{
    agent::{
        agents::{
            executor_agent::ExecutorAgent, master_agent::MasterAgent, planner_agent::PlannerAgent,
            verifier_agent::VerifierAgent,
        },
        multi::{
            communication::{Message, MessageType},
            manager::{AgentManager, AgentManagerConfig},
        },
        planning::AgentStep,
        shared::{GlobalContext, global_context::GlobalConfig},
        types::AgentCapability,
    },
    input::UserTaskInput,
};
use tracing::{Level, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting agent interaction demo...");

    // 创建全局上下文
    let config = GlobalConfig::default();
    let context = Arc::new(GlobalContext::new(config));

    // 创建Agent管理器
    let manager_config = AgentManagerConfig::default();
    let manager = Arc::new(AgentManager::new(context.clone(), manager_config));

    // 创建agents
    info!("Creating agents...");

    // 创建主控Agent
    let master = Box::new(MasterAgent::new(Some("master-001".to_string())));
    let master_id = manager.spawn_agent(master).await?;
    info!("Master agent spawned: {}", master_id);

    // 创建规划Agent
    let planner = Box::new(PlannerAgent::new(Some("planner-001".to_string())));
    let planner_id = manager.spawn_agent(planner).await?;
    info!("Planner agent spawned: {}", planner_id);

    // 创建执行Agent
    let executor = Box::new(ExecutorAgent::new(
        Some("executor-001".to_string()),
        vec![AgentCapability::ToolCalling("search".to_string())],
    ));
    let executor_id = manager.spawn_agent(executor).await?;
    info!("Executor agent spawned: {}", executor_id);

    // 创建验证Agent
    let verifier = Box::new(VerifierAgent::new(Some("verifier-001".to_string())));
    let verifier_id = manager.spawn_agent(verifier).await?;
    info!("Verifier agent spawned: {}", verifier_id);

    // 等待Agent初始化
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 测试1: 向Planner发送规划任务
    info!("\n=== Test 1: Send planning task to Planner ===");
    let user_input = UserTaskInput {
        goal: "搜索Rust教程".to_string(),
        content: "找一些关于Rust语言的入门教程".to_string(),
        description: Some("包括基础语法和实战项目".to_string()),
        constraints: None,
        references: None,
    };

    let planning_message = Message::new(
        "user".to_string(),
        Some(planner_id.clone()),
        MessageType::TaskAssignment,
        serde_json::json!({
            "task_type": "planning",
            "user_input": user_input,
        }),
    );

    info!("Sending planning task to planner...");
    manager.send_message(planning_message).await?;

    // 等待处理
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // 测试2: 向Executor发送执行任务
    info!("\n=== Test 2: Send execution task to Executor ===");

    // 创建一个执行步骤
    let step = AgentStep {
        step_id: 1,
        action: "search".to_string(),
        tool: Some("search".to_string()),
        input: Some(serde_json::json!({
            "query": "Rust programming tutorial"
        })),
        ..Default::default()
    };

    let execution_message = Message::new(
        "planner-001".to_string(), // 假设是planner发送的
        Some(executor_id.clone()),
        MessageType::TaskAssignment,
        serde_json::json!({
            "id": "task-001",
            "step": step,
        }),
    );

    info!("Sending execution task to executor...");
    manager.send_message(execution_message).await?;

    // 等待处理
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // 测试3: 向Verifier发送验证请求
    info!("\n=== Test 3: Send verification request to Verifier ===");

    let verification_message = Message::new(
        "executor-001".to_string(), // 假设是executor发送的
        Some(verifier_id.clone()),
        MessageType::Custom("VerificationRequest".to_string()),
        serde_json::json!({
            "task_id": "task-001",
            "task_type": "verification",
            "result": {
                "output": "Found 10 Rust tutorials",
                "success": true
            },
            "step": step,
        }),
    );

    info!("Sending verification request to verifier...");
    manager.send_message(verification_message).await?;

    // 等待处理
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // 显示系统状态
    info!("\n=== System Status ===");
    let stats = manager.get_stats().await;
    info!("Total agents: {}", stats.total_agents);
    info!("Total messages: {}", stats.total_messages);
    info!("Failed messages: {}", stats.failed_messages);

    // 显示所有Agent状态
    let all_status = manager.get_all_agent_status().await;
    info!("\nAgent statuses:");
    for status in all_status {
        info!("  - {}", serde_json::to_string(&status)?);
    }

    // 等待一段时间观察
    info!("\nWaiting for agents to process...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 关闭所有Agent
    info!("\nShutting down all agents...");
    manager.shutdown_all().await?;

    info!("Agent interaction demo completed!");
    Ok(())
}
