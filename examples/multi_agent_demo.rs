use rusagent::{
    agents::{
        master_agent::MasterAgent,
        planner_agent::PlannerAgent,
        executor_agent::ExecutorAgent,
        verifier_agent::VerifierAgent,
        monitor_agent::{MonitorAgent, AlertRule, AlertCondition, AlertSeverity},
    },
    multi_agent::{
        manager::{AgentManager, AgentManagerConfig},
        communication::{Message, MessageType},
        coordination::task_queue::Task,
    },
    shared::{GlobalContext, global_context::GlobalConfig},
    agent::types::{AgentCapability, Priority, TaskType},
};
use std::sync::Arc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting multi-agent demo...");

    // 创建全局上下文
    let config = GlobalConfig::default();
    let context = Arc::new(GlobalContext::new(config));

    // 创建Agent管理器
    let config = AgentManagerConfig::default();
    let manager = Arc::new(AgentManager::new(context.clone(), config));

    // 创建各种类型的Agent
    info!("Creating agents...");

    // 创建主控Agent
    let master = Box::new(MasterAgent::new(Some("master-001".to_string())));
    let master_id = manager.spawn_agent(master).await?;
    info!("Master agent spawned: {}", master_id);

    // 创建规划Agent
    let planner = Box::new(PlannerAgent::new(Some("planner-001".to_string())));
    let planner_id = manager.spawn_agent(planner).await?;
    info!("Planner agent spawned: {}", planner_id);

    // 创建执行Agent（具有不同的能力）
    let executor1 = Box::new(ExecutorAgent::new(
        Some("executor-001".to_string()),
        vec![AgentCapability::ToolCalling("web_search".to_string())],
    ));
    let executor1_id = manager.spawn_agent(executor1).await?;
    info!("Executor agent 1 spawned: {}", executor1_id);

    let executor2 = Box::new(ExecutorAgent::new(
        Some("executor-002".to_string()),
        vec![AgentCapability::ToolCalling("file_io".to_string())],
    ));
    let executor2_id = manager.spawn_agent(executor2).await?;
    info!("Executor agent 2 spawned: {}", executor2_id);

    // 创建验证Agent
    let verifier = Box::new(VerifierAgent::new(Some("verifier-001".to_string())));
    let verifier_id = manager.spawn_agent(verifier).await?;
    info!("Verifier agent spawned: {}", verifier_id);

    // 创建监控Agent
    let monitor = Box::new(
        MonitorAgent::new(Some("monitor-001".to_string()))
            .add_alert_rule(AlertRule {
                name: "High Error Rate".to_string(),
                condition: AlertCondition::ErrorRateHigh(0.1),
                severity: AlertSeverity::Warning,
            })
            .add_alert_rule(AlertRule {
                name: "Task Failure Rate".to_string(),
                condition: AlertCondition::TaskFailureRate(0.2),
                severity: AlertSeverity::Critical,
            })
    );
    let monitor_id = manager.spawn_agent(monitor).await?;
    info!("Monitor agent spawned: {}", monitor_id);

    // 等待所有Agent初始化
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 显示所有Agent状态
    info!("All agents status:");
    let all_status = manager.get_all_agent_status().await;
    for status in all_status {
        info!("Agent status: {}", serde_json::to_string_pretty(&status)?);
    }

    // 创建一个示例任务
    let task = Task::new(
        TaskType::Planning,
        Priority::High,
        serde_json::json!({
            "description": "Research and summarize Rust async programming",
            "requirements": ["web search", "documentation", "code examples"]
        }),
        "user-001".to_string(),
    );

    info!("Sending task to master agent...");
    
    // 发送任务给主控Agent
    let task_message = Message::new(
        "user".to_string(),
        Some(master_id.clone()),
        MessageType::TaskAssignment,
        serde_json::to_value(&task)?,
    );
    
    manager.send_message(task_message).await?;

    // 等待任务处理
    info!("Waiting for task processing...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // 发送健康检查给监控Agent
    let health_check = Message::new(
        "user".to_string(),
        Some(monitor_id.clone()),
        MessageType::Custom("HealthCheck".to_string()),
        serde_json::json!({}),
    );
    
    manager.send_message(health_check).await?;

    // 等待更多处理
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 显示管理器统计信息
    let stats = manager.get_stats().await;
    info!("Manager statistics: {:?}", stats);

    // 查找具有特定能力的Agent
    let tool_agents = manager.find_agents_by_capability(&AgentCapability::ToolCalling("web_search".to_string())).await;
    info!("Agents with web_search capability: {}", tool_agents.len());

    // 关闭所有Agent
    info!("Shutting down all agents...");
    manager.shutdown_all().await?;

    info!("Multi-agent demo completed!");
    Ok(())
}