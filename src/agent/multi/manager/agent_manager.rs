use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::agent::{
    core::base_agent::{AgentBehavior, AgentLifecycleState},
    multi::{
        communication::{MessageBus, MessageBusConfig, MessageReceiver, Message, MessageType},
        registry::{AgentInfo, AgentRegistry, RegistryConfig},
    },
    shared::GlobalContext,
    types::{AgentCapability, AgentStatus, AgentType},
};
use crate::error::{Result, Error};
use crate::error::agent_error::AgentError;

/// Agent管理器配置
#[derive(Debug, Clone)]
pub struct AgentManagerConfig {
    pub message_bus_config: MessageBusConfig,
    pub registry_config: RegistryConfig,
    pub max_agents: usize,
    pub enable_auto_scaling: bool,
}

impl Default for AgentManagerConfig {
    fn default() -> Self {
        Self {
            message_bus_config: MessageBusConfig::default(),
            registry_config: RegistryConfig::default(),
            max_agents: 100,
            enable_auto_scaling: false,
        }
    }
}

/// Agent运行时信息
struct AgentRuntime {
    task_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    state: AgentLifecycleState,
    agent_id: String,
    agent_type: AgentType,
}

/// Agent管理器，负责Agent的生命周期管理
pub struct AgentManager {
    /// Agent运行时映射
    agents: Arc<RwLock<HashMap<String, AgentRuntime>>>,
    /// 消息总线
    message_bus: Arc<MessageBus>,
    /// Agent注册表
    registry: Arc<AgentRegistry>,
    /// 全局上下文
    context: Arc<GlobalContext>,
    /// 配置
    config: AgentManagerConfig,
    /// 管理器状态
    running: Arc<RwLock<bool>>,
}

impl AgentManager {
    /// 创建新的Agent管理器
    pub fn new(context: Arc<GlobalContext>, config: AgentManagerConfig) -> Self {
        let message_bus = Arc::new(MessageBus::new(config.message_bus_config.clone()));
        let registry = Arc::new(AgentRegistry::new(config.registry_config.clone()));

        // 启动注册表清理任务
        registry.clone().start_cleanup_task();

        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            message_bus,
            registry,
            context,
            config,
            running: Arc::new(RwLock::new(true)),
        }
    }

    /// 启动Agent
    pub async fn spawn_agent(
        &self,
        mut agent: Box<dyn AgentBehavior>,
    ) -> Result<String> {
        let agent_id = agent.get_id().to_string();
        let agent_type = agent.get_type();
        let capabilities = agent.get_capabilities().to_vec();

        // 检查Agent数量限制
        if self.agents.read().await.len() >= self.config.max_agents {
            return Err(Error::AgentError(AgentError::ResourceExhausted(
                "Maximum agent limit reached".into(),
            )));
        }

        // 初始化Agent
        agent.initialize(self.context.clone()).await?;

        // 注册到消息总线
        let message_receiver = self.message_bus.register_agent(agent_id.clone()).await?;

        // 注册到注册表
        let agent_info = AgentInfo::new(agent_id.clone(), agent_type, capabilities);
        self.registry.register(agent_info).await?;

        // 创建关闭通道
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // 启动Agent任务
        let agent_id_clone = agent_id.clone();
        let message_bus = self.message_bus.clone();
        let registry = self.registry.clone();
        
        let task_handle = tokio::spawn(async move {
            Self::agent_loop(
                agent,
                agent_id_clone,
                message_receiver,
                message_bus,
                registry,
                shutdown_rx,
            )
            .await;
        });

        // 保存运行时信息
        // 注意：agent已经被移动到task中，这里我们创建一个占位符
        // 实际的agent逻辑在task中运行
        let runtime = AgentRuntime {
            task_handle: Some(task_handle),
            shutdown_tx: Some(shutdown_tx),
            state: AgentLifecycleState::Running,
            agent_id: agent_id.clone(),
            agent_type,
        };

        self.agents.write().await.insert(agent_id.clone(), runtime);

        info!("Agent {} spawned successfully", agent_id);
        Ok(agent_id)
    }

    /// Agent主循环
    async fn agent_loop(
        mut agent: Box<dyn AgentBehavior>,
        agent_id: String,
        mut message_receiver: MessageReceiver,
        _message_bus: Arc<MessageBus>,
        registry: Arc<AgentRegistry>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) {
        info!("Agent {} started", agent_id);

        // 启动心跳任务
        let heartbeat_handle = Self::start_heartbeat_task(
            agent_id.clone(),
            registry.clone(),
        );

        // 将Agent分成两部分：一个用于运行，一个用于消息处理
        let agent_id_for_run = agent_id.clone();
        
        // 创建一个channel来协调agent.run()的执行
        let (run_tx, run_rx) = tokio::sync::oneshot::channel::<()>();
        
        // 启动一个任务来运行agent.run()
        tokio::spawn(async move {
            tokio::select! {
                result = agent.run() => {
                    if let Err(e) = result {
                        error!("Agent {} run error: {:?}", agent_id_for_run, e);
                    }
                }
                _ = run_rx => {
                    // 收到停止信号，执行关闭逻辑
                    if let Err(e) = agent.shutdown().await {
                        error!("Agent {} shutdown error: {:?}", agent_id_for_run, e);
                    }
                }
            }
        });

        // 消息处理循环
        loop {
            tokio::select! {
                // 接收消息
                Some(msg) = message_receiver.recv() => {
                    // 由于agent已经被移动，我们只能记录日志
                    debug!("Agent {} received message: {:?}", agent_id, msg.message_type);
                    // TODO: 考虑使用消息转发机制而不是直接处理
                }
                // 关闭信号
                _ = &mut shutdown_rx => {
                    info!("Agent {} received shutdown signal", agent_id);
                    break;
                }
            }
        }

        // 清理工作
        heartbeat_handle.abort();
        let _ = run_tx.send(()); // 通知agent任务停止

        info!("Agent {} stopped", agent_id);
    }

    /// 处理Agent消息
    #[allow(dead_code)]
    async fn handle_agent_message(
        agent: &mut Box<dyn AgentBehavior>,
        message: Message,
        message_bus: &Arc<MessageBus>,
        registry: &Arc<AgentRegistry>,
    ) -> Result<()> {
        match &message.message_type {
            MessageType::Control(cmd) => {
                // 处理控制命令
                match cmd {
                    crate::agent::multi::communication::message::ControlCommand::Stop => {
                        // 由外部处理
                    }
                    _ => {
                        warn!("Unhandled control command: {:?}", cmd);
                    }
                }
            }
            MessageType::StatusUpdate => {
                // 更新Agent状态
                if let Some(status) = message.payload.get("status") {
                    if let Ok(status) = serde_json::from_value::<AgentStatus>(status.clone()) {
                        registry.update_status(agent.get_id(), status).await?;
                    }
                }
            }
            _ => {
                // 让Agent处理其他消息
                if let Some(response) = agent.process_message(message).await? {
                    message_bus.send(response).await?;
                }
            }
        }

        Ok(())
    }

    /// 启动心跳任务
    fn start_heartbeat_task(
        agent_id: String,
        registry: Arc<AgentRegistry>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                if let Err(e) = registry.heartbeat(&agent_id).await {
                    error!("Agent {} heartbeat error: {:?}", agent_id, e);
                    break;
                }
            }
        })
    }

    /// 终止Agent
    pub async fn terminate_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;
        
        if let Some(mut runtime) = agents.remove(agent_id) {
            // 发送关闭信号
            if let Some(tx) = runtime.shutdown_tx.take() {
                let _ = tx.send(());
            }

            // 等待任务结束
            if let Some(handle) = runtime.task_handle.take() {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    handle,
                ).await;
            }

            // 从注册表中移除
            self.registry.unregister(agent_id).await?;
            
            // 从消息总线中移除
            self.message_bus.unregister_agent(agent_id).await?;

            info!("Agent {} terminated", agent_id);
            Ok(())
        } else {
            Err(Error::AgentError(AgentError::AgentNotFound(agent_id.to_string())))
        }
    }

    /// 获取Agent状态
    pub async fn get_agent_status(&self, agent_id: &str) -> Result<serde_json::Value> {
        let agents = self.agents.read().await;
        
        if let Some(runtime) = agents.get(agent_id) {
            Ok(serde_json::json!({
                "id": runtime.agent_id,
                "type": runtime.agent_type,
                "state": runtime.state,
            }))
        } else {
            Err(Error::AgentError(AgentError::AgentNotFound(agent_id.to_string())))
        }
    }

    /// 获取所有Agent的状态
    pub async fn get_all_agent_status(&self) -> Vec<serde_json::Value> {
        self.agents
            .read()
            .await
            .values()
            .map(|runtime| serde_json::json!({
                "id": runtime.agent_id,
                "type": runtime.agent_type,
                "state": runtime.state,
            }))
            .collect()
    }

    /// 发送消息给特定Agent
    pub async fn send_message(&self, message: Message) -> Result<()> {
        self.message_bus.send(message).await
    }

    /// 广播消息给所有Agent
    pub async fn broadcast_message(&self, message: Message) -> Result<()> {
        self.message_bus.send(message).await
    }

    /// 根据能力查找Agent
    pub async fn find_agents_by_capability(
        &self,
        capability: &AgentCapability,
    ) -> Vec<AgentInfo> {
        self.registry.find_by_capability(capability).await
    }

    /// 根据类型查找Agent
    pub async fn find_agents_by_type(&self, agent_type: AgentType) -> Vec<AgentInfo> {
        self.registry.find_by_type(agent_type).await
    }

    /// 获取空闲的Agent
    pub async fn find_idle_agents(&self) -> Vec<AgentInfo> {
        self.registry.find_idle_agents().await
    }

    /// 关闭所有Agent
    pub async fn shutdown_all(&self) -> Result<()> {
        *self.running.write().await = false;

        let agent_ids: Vec<String> = self.agents.read().await.keys().cloned().collect();
        
        for agent_id in agent_ids {
            if let Err(e) = self.terminate_agent(&agent_id).await {
                error!("Failed to terminate agent {}: {:?}", agent_id, e);
            }
        }

        info!("All agents shut down");
        Ok(())
    }

    /// 获取管理器统计信息
    pub async fn get_stats(&self) -> ManagerStats {
        let registry_stats = self.registry.get_stats().await;
        let message_stats = self.message_bus.get_stats().await;

        ManagerStats {
            total_agents: registry_stats.total_agents,
            alive_agents: registry_stats.alive_agents,
            idle_agents: registry_stats.idle_agents,
            busy_agents: registry_stats.busy_agents,
            total_messages: message_stats.total_messages,
            failed_messages: message_stats.failed_deliveries,
        }
    }
}

/// 管理器统计信息
#[derive(Debug)]
pub struct ManagerStats {
    pub total_agents: usize,
    pub alive_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub total_messages: u64,
    pub failed_messages: u64,
}