use std::sync::Arc;

use rusagent::{
    agents::{
        executor_agent::ExecutorAgent,
        master_agent::MasterAgent,
        monitor_agent::{AlertCondition, AlertRule, AlertSeverity, MonitorAgent},
        planner_agent::PlannerAgent,
        verifier_agent::VerifierAgent,
    },
    multi_agent::{
        communication::{Message, MessageType},
        coordination::task_queue::Task,
        manager::{AgentManager, AgentManagerConfig},
    },
    shared::{GlobalContext, global_context::GlobalConfig},
    agent::types::{AgentCapability, Priority, TaskType},
    input::UserTaskInput,
};
use tracing::{Level, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting multi-agent collaboration demo...");

    // 创建全局上下文
    let config = GlobalConfig::default();
    let context = Arc::new(GlobalContext::new(config));

    // 创建Agent管理器
    let manager_config = AgentManagerConfig::default();
    let manager = Arc::new(AgentManager::new(context.clone(), manager_config));

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

    // 创建执行Agent
    let executor = Box::new(ExecutorAgent::new(
        Some("executor-001".to_string()),
        vec![AgentCapability::ToolCalling("web_search".to_string())],
    ));
    let executor_id = manager.spawn_agent(executor).await?;
    info!("Executor agent spawned: {}", executor_id);

    // 创建验证Agent
    let verifier = Box::new(VerifierAgent::new(Some("verifier-001".to_string())));
    let verifier_id = manager.spawn_agent(verifier).await?;
    info!("Verifier agent spawned: {}", verifier_id);

    // 创建监控Agent
    let monitor = Box::new(
        MonitorAgent::new(Some("monitor-001".to_string())).add_alert_rule(AlertRule {
            name: "Task Failure".to_string(),
            condition: AlertCondition::TaskFailureRate(0.5),
            severity: AlertSeverity::Critical,
        }),
    );
    let monitor_id = manager.spawn_agent(monitor).await?;
    info!("Monitor agent spawned: {}", monitor_id);

    // 等待所有Agent初始化
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 场景1: 发送规划任务给规划Agent
    info!("\n=== Scenario 1: Planning Task ===");
    let user_input = UserTaskInput {
        goal: "创建一个待办事项管理系统".to_string(),
        content: "我需要一个能够添加、删除、标记完成任务的待办事项管理系统".to_string(),
        description: Some("系统应该支持任务分类、优先级设置和截止日期".to_string()),
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

    manager.send_message(planning_message).await?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 场景2: 发送执行任务给执行Agent
    info!("\n=== Scenario 2: Execution Task ===");
    let execution_task = serde_json::json!({
        "id": "exec-001",
        "step": {
            "step_id": "1",
            "action": "search",
            "tool": "web_search",
            "tool_input": {
                "query": "Rust todo app tutorial"
            }
        }
    });

    let execution_message = Message::new(
        "user".to_string(),
        Some(executor_id.clone()),
        MessageType::TaskAssignment,
        execution_task,
    );

    manager.send_message(execution_message).await?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 场景3: 发送验证请求给验证Agent
    info!("\n=== Scenario 3: Verification Request ===");
    let verification_request = Message::new(
        "user".to_string(),
        Some(verifier_id.clone()),
        MessageType::Custom("VerificationRequest".to_string()),
        serde_json::json!({
            "task_id": "exec-001",
            "result": {
                "output": "搜索结果找到了10个相关的Rust待办事项教程",
                "success": true
            }
        }),
    );

    manager.send_message(verification_request).await?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 场景4: 发送状态更新给监控Agent
    info!("\n=== Scenario 4: Status Updates ===");

    // 成功的任务
    let success_update = Message::broadcast(
        "system".to_string(),
        MessageType::StatusUpdate,
        serde_json::json!({
            "task_status": "completed",
            "task_id": "task-001",
            "agent_id": executor_id.clone(),
            "healthy": true
        }),
    );
    manager.broadcast_message(success_update).await?;

    // 失败的任务
    let failure_update = Message::broadcast(
        "system".to_string(),
        MessageType::StatusUpdate,
        serde_json::json!({
            "task_status": "failed",
            "task_id": "task-002",
            "agent_id": executor_id.clone(),
            "healthy": true
        }),
    );
    manager.broadcast_message(failure_update).await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 场景5: 主控Agent协调任务
    info!("\n=== Scenario 5: Task Coordination ===");
    let coordination_task = Task::new(
        TaskType::Composite,
        Priority::High,
        serde_json::json!({
            "description": "协调多个Agent完成待办事项系统开发",
            "subtasks": [
                {"agent": "planner", "task": "设计系统架构"},
                {"agent": "executor", "task": "实现核心功能"},
                {"agent": "verifier", "task": "验证实现结果"}
            ]
        }),
        "user".to_string(),
    );

    let coordination_message = Message::new(
        "user".to_string(),
        Some(master_id.clone()),
        MessageType::TaskAssignment,
        serde_json::to_value(&coordination_task)?,
    );

    manager.send_message(coordination_message).await?;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 获取系统状态
    info!("\n=== System Status ===");

    // 获取健康报告
    let health_check = Message::new(
        "user".to_string(),
        Some(monitor_id.clone()),
        MessageType::Custom("HealthCheck".to_string()),
        serde_json::json!({}),
    );
    manager.send_message(health_check).await?;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 显示管理器统计信息
    let stats = manager.get_stats().await;
    info!("Manager statistics: {:?}", stats);

    // 查找具有特定能力的Agent
    let tool_agents = manager
        .find_agents_by_capability(&AgentCapability::ToolCalling("web_search".to_string()))
        .await;
    info!("Agents with web_search capability: {}", tool_agents.len());

    // 获取所有Agent状态
    let all_status = manager.get_all_agent_status().await;
    info!("Total active agents: {}", all_status.len());

    // 等待更多时间以便观察Agent行为
    info!("\nLetting agents work for a few seconds...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 关闭所有Agent
    info!("\nShutting down all agents...");
    manager.shutdown_all().await?;

    info!("Multi-agent collaboration demo completed!");
    Ok(())
}
