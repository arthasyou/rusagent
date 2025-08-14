use std::{collections::HashMap, sync::Arc};

use tokio::{
    sync::{RwLock, oneshot},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

use crate::{
    agent::{
        core::base_agent::AgentBehavior,
        types::{AgentCapability, AgentLifecycleState, AgentStatus, AgentType},
    },
    error::{Error, Result, agent_error::AgentError},
    multi_agent::{
        communication::{Message, MessageBus, MessageBusConfig, MessageReceiver, MessageType},
        registry::{AgentInfo, AgentRegistry, RegistryConfig},
    },
    shared::GlobalContext,
};

/// Agent manager configuration
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

/// Agent runtime information
struct AgentRuntime {
    task_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    state: AgentLifecycleState,
    agent_id: String,
    agent_type: AgentType,
}

/// Agent manager responsible for Agent lifecycle management
pub struct AgentManager {
    /// Agent runtime mapping
    agents: Arc<RwLock<HashMap<String, AgentRuntime>>>,
    /// Message bus
    message_bus: Arc<MessageBus>,
    /// Agent registry
    registry: Arc<AgentRegistry>,
    /// Global context
    context: Arc<GlobalContext>,
    /// Configuration
    config: AgentManagerConfig,
    /// Manager state
    running: Arc<RwLock<bool>>,
}

impl AgentManager {
    /// Create new Agent manager
    pub fn new(context: Arc<GlobalContext>, config: AgentManagerConfig) -> Self {
        let message_bus = Arc::new(MessageBus::new(config.message_bus_config.clone()));
        let registry = Arc::new(AgentRegistry::new(config.registry_config.clone()));

        // Start registry cleanup task
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

    /// Start Agent
    pub async fn spawn_agent(&self, mut agent: Box<dyn AgentBehavior>) -> Result<String> {
        let agent_id = agent.get_id().to_string();
        let agent_type = agent.get_type();
        let capabilities = agent.get_capabilities().to_vec();

        // Check Agent quantity limit
        if self.agents.read().await.len() >= self.config.max_agents {
            return Err(Error::AgentError(AgentError::ResourceExhausted(
                "Maximum agent limit reached".into(),
            )));
        }

        // Initialize Agent
        agent.initialize(self.context.clone()).await?;

        // Register to message bus
        let message_receiver = self.message_bus.register_agent(agent_id.clone()).await?;

        // Register to registry
        let agent_info = AgentInfo::new(agent_id.clone(), agent_type, capabilities);
        self.registry.register(agent_info).await?;

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Start Agent task
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

        // Save runtime information
        // Note: agent has been moved to task, we create a placeholder here
        // Actual agent logic runs in the task
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

    /// Agent main loop
    async fn agent_loop(
        mut agent: Box<dyn AgentBehavior>,
        agent_id: String,
        mut message_receiver: MessageReceiver,
        _message_bus: Arc<MessageBus>,
        registry: Arc<AgentRegistry>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) {
        info!("Agent {} started", agent_id);

        // Start heartbeat task
        let heartbeat_handle = Self::start_heartbeat_task(agent_id.clone(), registry.clone());

        // Split Agent into two parts: one for running, one for message processing
        let agent_id_for_run = agent_id.clone();

        // Create a channel to coordinate agent.run() execution
        let (run_tx, run_rx) = tokio::sync::oneshot::channel::<()>();

        // Start a task to run agent.run()
        tokio::spawn(async move {
            tokio::select! {
                result = agent.run() => {
                    if let Err(e) = result {
                        error!("Agent {} run error: {:?}", agent_id_for_run, e);
                    }
                }
                _ = run_rx => {
                    // Received stop signal, execute shutdown logic
                    if let Err(e) = agent.shutdown().await {
                        error!("Agent {} shutdown error: {:?}", agent_id_for_run, e);
                    }
                }
            }
        });

        // Message processing loop
        loop {
            tokio::select! {
                // Receive message
                Some(msg) = message_receiver.recv() => {
                    // Since agent has been moved, we can only log
                    debug!("Agent {} received message: {:?}", agent_id, msg.message_type);
                    // TODO: Consider using message forwarding mechanism instead of direct processing
                }
                // Shutdown signal
                _ = &mut shutdown_rx => {
                    info!("Agent {} received shutdown signal", agent_id);
                    break;
                }
            }
        }

        // Cleanup work
        heartbeat_handle.abort();
        let _ = run_tx.send(()); // Notify agent task to stop

        info!("Agent {} stopped", agent_id);
    }

    /// Process Agent messages
    #[allow(dead_code)]
    async fn handle_agent_message(
        agent: &mut Box<dyn AgentBehavior>,
        message: Message,
        message_bus: &Arc<MessageBus>,
        registry: &Arc<AgentRegistry>,
    ) -> Result<()> {
        match &message.message_type {
            MessageType::Control(cmd) => {
                // Process control command
                match cmd {
                    crate::multi_agent::communication::message::ControlCommand::Stop => {
                        // Handled externally
                    }
                    _ => {
                        warn!("Unhandled control command: {:?}", cmd);
                    }
                }
            }
            MessageType::StatusUpdate => {
                // Update Agent status
                if let Some(status) = message.payload.get("status")
                    && let Ok(status) = serde_json::from_value::<AgentStatus>(status.clone()) {
                        registry.update_status(agent.get_id(), status).await?;
                    }
            }
            _ => {
                // Let Agent process other messages
                if let Some(response) = agent.process_message(message).await? {
                    message_bus.send(response).await?;
                }
            }
        }

        Ok(())
    }

    /// Start heartbeat task
    fn start_heartbeat_task(agent_id: String, registry: Arc<AgentRegistry>) -> JoinHandle<()> {
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

    /// Terminate Agent
    pub async fn terminate_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;

        if let Some(mut runtime) = agents.remove(agent_id) {
            // Send shutdown signal
            if let Some(tx) = runtime.shutdown_tx.take() {
                let _ = tx.send(());
            }

            // Wait for task to finish
            if let Some(handle) = runtime.task_handle.take() {
                let _ = tokio::time::timeout(std::time::Duration::from_secs(10), handle).await;
            }

            // Remove from registry
            self.registry.unregister(agent_id).await?;

            // Remove from message bus
            self.message_bus.unregister_agent(agent_id).await?;

            info!("Agent {} terminated", agent_id);
            Ok(())
        } else {
            Err(Error::AgentError(AgentError::AgentNotFound(
                agent_id.to_string(),
            )))
        }
    }

    /// Get Agent status
    pub async fn get_agent_status(&self, agent_id: &str) -> Result<serde_json::Value> {
        let agents = self.agents.read().await;

        if let Some(runtime) = agents.get(agent_id) {
            Ok(serde_json::json!({
                "id": runtime.agent_id,
                "type": runtime.agent_type,
                "state": runtime.state,
            }))
        } else {
            Err(Error::AgentError(AgentError::AgentNotFound(
                agent_id.to_string(),
            )))
        }
    }

    /// Get status of all Agents
    pub async fn get_all_agent_status(&self) -> Vec<serde_json::Value> {
        self.agents
            .read()
            .await
            .values()
            .map(|runtime| {
                serde_json::json!({
                    "id": runtime.agent_id,
                    "type": runtime.agent_type,
                    "state": runtime.state,
                })
            })
            .collect()
    }

    /// Send message to specific Agent
    pub async fn send_message(&self, message: Message) -> Result<()> {
        self.message_bus.send(message).await
    }

    /// Broadcast message to all Agents
    pub async fn broadcast_message(&self, message: Message) -> Result<()> {
        self.message_bus.send(message).await
    }

    /// Find Agent by capability
    pub async fn find_agents_by_capability(&self, capability: &AgentCapability) -> Vec<AgentInfo> {
        self.registry.find_by_capability(capability).await
    }

    /// Find Agent by type
    pub async fn find_agents_by_type(&self, agent_type: AgentType) -> Vec<AgentInfo> {
        self.registry.find_by_type(agent_type).await
    }

    /// Get idle Agents
    pub async fn find_idle_agents(&self) -> Vec<AgentInfo> {
        self.registry.find_idle_agents().await
    }

    /// Shutdown all Agents
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

    /// Get manager statistics
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

/// Manager statistics
#[derive(Debug)]
pub struct ManagerStats {
    pub total_agents: usize,
    pub alive_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub total_messages: u64,
    pub failed_messages: u64,
}
