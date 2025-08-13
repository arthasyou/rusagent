use rusagent::{
    agents::{
        executor_agent::ExecutorAgent,
        monitor_agent::MonitorAgent,
    },
    multi_agent::{
        manager::{AgentManager, AgentManagerConfig},
        communication::{Message, MessageType},
    },
    shared::{GlobalContext, global_context::GlobalConfig},
    agent::types::AgentCapability,
};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting simple multi-agent demo...");

    // 创建全局上下文
    let config = GlobalConfig::default();
    let context = Arc::new(GlobalContext::new(config));

    // 创建Agent管理器
    let manager_config = AgentManagerConfig::default();
    let manager = Arc::new(AgentManager::new(context.clone(), manager_config));

    // 创建一个执行Agent
    info!("Creating executor agent...");
    let executor = Box::new(ExecutorAgent::new(
        Some("executor-001".to_string()),
        vec![AgentCapability::ToolCalling("calculator".to_string())],
    ));
    let executor_id = manager.spawn_agent(executor).await?;
    info!("Executor agent spawned: {}", executor_id);

    // 创建一个监控Agent
    info!("Creating monitor agent...");
    let monitor = Box::new(MonitorAgent::new(Some("monitor-001".to_string())));
    let monitor_id = manager.spawn_agent(monitor).await?;
    info!("Monitor agent spawned: {}", monitor_id);

    // 等待Agent初始化
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 获取系统状态
    info!("\nChecking system status...");
    let all_status = manager.get_all_agent_status().await;
    for status in &all_status {
        info!("Agent: {}", serde_json::to_string_pretty(status)?);
    }

    // 发送一个简单任务给执行Agent
    info!("\nSending task to executor...");
    let task = serde_json::json!({
        "id": "task-001",
        "description": "Calculate 2 + 2",
        "priority": "high"
    });

    let task_message = Message::new(
        "user".to_string(),
        Some(executor_id.clone()),
        MessageType::TaskAssignment,
        task,
    );
    
    manager.send_message(task_message).await?;
    info!("Task sent to executor");

    // 发送状态更新（模拟任务完成）
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    info!("\nSending status update...");
    let status_update = Message::broadcast(
        executor_id.clone(),
        MessageType::StatusUpdate,
        serde_json::json!({
            "task_status": "completed",
            "task_id": "task-001",
            "agent_id": executor_id.clone(),
            "healthy": true,
            "result": "4"
        }),
    );
    manager.broadcast_message(status_update).await?;
    info!("Status update sent");

    // 请求健康检查
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    info!("\nRequesting health check...");
    let health_check = Message::new(
        "user".to_string(),
        Some(monitor_id.clone()),
        MessageType::Custom("HealthCheck".to_string()),
        serde_json::json!({}),
    );
    manager.send_message(health_check).await?;
    info!("Health check requested");

    // 等待处理完成
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 显示最终统计
    info!("\nFinal statistics:");
    let stats = manager.get_stats().await;
    info!("Total agents: {}", stats.total_agents);
    info!("Messages sent: {}", stats.total_messages);
    info!("Failed messages: {}", stats.failed_messages);

    // 关闭所有Agent
    info!("\nShutting down...");
    manager.shutdown_all().await?;

    info!("Demo completed successfully!");
    Ok(())
}