use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::multi_agent::communication::message::{Message, MessageFilter};
use crate::error::{Result, Error};
use crate::error::agent_error::AgentError;

/// Message bus configuration
#[derive(Debug, Clone)]
pub struct MessageBusConfig {
    /// Broadcast channel capacity
    pub broadcast_capacity: usize,
    /// Point-to-point channel capacity
    pub p2p_capacity: usize,
    /// Message history size
    pub history_size: usize,
    /// Whether to enable message persistence
    pub enable_persistence: bool,
}

impl Default for MessageBusConfig {
    fn default() -> Self {
        Self {
            broadcast_capacity: 1000,
            p2p_capacity: 100,
            history_size: 1000,
            enable_persistence: false,
        }
    }
}

/// Message bus responsible for message passing between Agents
pub struct MessageBus {
    /// Broadcast sender
    broadcast_tx: broadcast::Sender<Arc<Message>>,
    /// Point-to-point channel mapping (agent_id -> sender)
    p2p_channels: Arc<RwLock<HashMap<String, mpsc::Sender<Arc<Message>>>>>,
    /// Message history
    message_history: Arc<RwLock<Vec<Arc<Message>>>>,
    /// Configuration
    config: MessageBusConfig,
    /// Statistics
    stats: Arc<RwLock<MessageBusStats>>,
}

/// Message bus statistics
#[derive(Debug, Default)]
pub struct MessageBusStats {
    pub total_messages: u64,
    pub broadcast_messages: u64,
    pub p2p_messages: u64,
    pub failed_deliveries: u64,
    pub expired_messages: u64,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new(config: MessageBusConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(config.broadcast_capacity);

        Self {
            broadcast_tx,
            p2p_channels: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::with_capacity(config.history_size))),
            config,
            stats: Arc::new(RwLock::new(MessageBusStats::default())),
        }
    }

    /// Register Agent and create its dedicated receiving channel
    pub async fn register_agent(&self, agent_id: String) -> Result<MessageReceiver> {
        let (tx, rx) = mpsc::channel(self.config.p2p_capacity);
        
        // Register point-to-point channel
        self.p2p_channels.write().await.insert(agent_id.clone(), tx);
        
        // Subscribe to broadcast channel
        let broadcast_rx = self.broadcast_tx.subscribe();
        
        info!("Agent {} registered to message bus", agent_id);

        Ok(MessageReceiver {
            agent_id,
            p2p_rx: rx,
            broadcast_rx,
        })
    }

    /// Unregister Agent
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        self.p2p_channels.write().await.remove(agent_id);
        info!("Agent {} unregistered from message bus", agent_id);
        Ok(())
    }

    /// Send message
    pub async fn send(&self, message: Message) -> Result<()> {
        // Check if message has expired
        if message.is_expired() {
            warn!("Attempting to send expired message: {}", message.id);
            self.stats.write().await.expired_messages += 1;
            return Err(Error::AgentError(AgentError::MessageDeliveryError("Message expired".into())));
        }

        let message = Arc::new(message);

        // Update statistics
        self.stats.write().await.total_messages += 1;

        // Save to history
        self.save_to_history(message.clone()).await;

        if message.is_broadcast() {
            // Broadcast message
            self.broadcast(message).await
        } else {
            // Point-to-point message
            self.send_p2p(message).await
        }
    }

    /// Send broadcast message
    async fn broadcast(&self, message: Arc<Message>) -> Result<()> {
        debug!("Broadcasting message: {:?}", message.id);
        self.stats.write().await.broadcast_messages += 1;

        match self.broadcast_tx.send(message) {
            Ok(_) => Ok(()),
            Err(_) => {
                error!("Failed to broadcast message - no receivers");
                self.stats.write().await.failed_deliveries += 1;
                Err(Error::AgentError(AgentError::MessageDeliveryError(
                    "No receivers for broadcast".into(),
                )))
            }
        }
    }

    /// Send point-to-point message
    async fn send_p2p(&self, message: Arc<Message>) -> Result<()> {
        let receiver_id = message
            .receiver_id
            .as_ref()
            .ok_or_else(|| Error::AgentError(AgentError::MessageDeliveryError("No receiver specified".into())))?;

        debug!("Sending P2P message to {}: {:?}", receiver_id, message.id);
        self.stats.write().await.p2p_messages += 1;

        let receiver_id_string = receiver_id.to_string();
        let channels = self.p2p_channels.read().await;
        if let Some(tx) = channels.get(receiver_id) {
            match tx.send(message).await {
                Ok(_) => Ok(()),
                Err(_) => {
                    error!("Failed to send message to {}", receiver_id_string);
                    self.stats.write().await.failed_deliveries += 1;
                    Err(Error::AgentError(AgentError::MessageDeliveryError(format!(
                        "Failed to send to {receiver_id_string}"
                    ))))
                }
            }
        } else {
            warn!("Agent {} not found", receiver_id_string);
            self.stats.write().await.failed_deliveries += 1;
            Err(Error::AgentError(AgentError::MessageDeliveryError(format!(
                "Agent {receiver_id_string} not found"
            ))))
        }
    }

    /// Save message to history
    async fn save_to_history(&self, message: Arc<Message>) {
        let mut history = self.message_history.write().await;
        
        // Limit history size
        if history.len() >= self.config.history_size {
            history.remove(0);
        }
        
        history.push(message);
    }

    /// Get message history
    pub async fn get_history(&self, filter: Option<MessageFilter>) -> Vec<Arc<Message>> {
        let history = self.message_history.read().await;
        
        if let Some(filter) = filter {
            history
                .iter()
                .filter(|msg| filter.matches(msg))
                .cloned()
                .collect()
        } else {
            history.clone()
        }
    }

    /// Get statistics
    pub async fn get_stats(&self) -> MessageBusStats {
        self.stats.read().await.clone()
    }

    /// Get list of registered Agents
    pub async fn get_registered_agents(&self) -> Vec<String> {
        self.p2p_channels.read().await.keys().cloned().collect()
    }
}

/// Message receiver, each Agent holds one
pub struct MessageReceiver {
    /// Agent ID
    pub agent_id: String,
    /// Point-to-point message receiver
    p2p_rx: mpsc::Receiver<Arc<Message>>,
    /// Broadcast message receiver
    broadcast_rx: broadcast::Receiver<Arc<Message>>,
}

impl MessageReceiver {
    /// Receive next message (prioritize point-to-point messages)
    pub async fn recv(&mut self) -> Option<Arc<Message>> {
        tokio::select! {
            // Prioritize point-to-point messages
            Some(msg) = self.p2p_rx.recv() => {
                debug!("Agent {} received P2P message: {:?}", self.agent_id, msg.id);
                Some(msg)
            }
            // Then receive broadcast messages
            Ok(msg) = self.broadcast_rx.recv() => {
                // Filter out broadcast messages sent by self
                if msg.sender_id != self.agent_id {
                    debug!("Agent {} received broadcast message: {:?}", self.agent_id, msg.id);
                    Some(msg)
                } else {
                    // Recursively call to continue receiving next message
                    Box::pin(self.recv()).await
                }
            }
            else => None
        }
    }

    /// Try to receive message (non-blocking)
    pub fn try_recv(&mut self) -> Option<Arc<Message>> {
        // First try to receive point-to-point message
        if let Ok(msg) = self.p2p_rx.try_recv() {
            return Some(msg);
        }

        // Then try to receive broadcast message
        if let Ok(msg) = self.broadcast_rx.try_recv()
            && msg.sender_id != self.agent_id {
                return Some(msg);
            }

        None
    }

    /// Receive message (with filter)
    pub async fn recv_filtered(&mut self, filter: MessageFilter) -> Option<Arc<Message>> {
        loop {
            if let Some(msg) = self.recv().await {
                if filter.matches(&msg) {
                    return Some(msg);
                }
                // Continue receiving if not matched
            } else {
                return None;
            }
        }
    }
}

impl Clone for MessageBusStats {
    fn clone(&self) -> Self {
        Self {
            total_messages: self.total_messages,
            broadcast_messages: self.broadcast_messages,
            p2p_messages: self.p2p_messages,
            failed_deliveries: self.failed_deliveries,
            expired_messages: self.expired_messages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_agent::communication::message::MessageType;

    #[tokio::test]
    async fn test_message_bus_registration() {
        let bus = MessageBus::new(MessageBusConfig::default());
        
        let receiver = bus.register_agent("agent1".to_string()).await.unwrap();
        assert_eq!(receiver.agent_id, "agent1");

        let agents = bus.get_registered_agents().await;
        assert_eq!(agents.len(), 1);
        assert!(agents.contains(&"agent1".to_string()));
    }

    #[tokio::test]
    async fn test_p2p_messaging() {
        let bus = MessageBus::new(MessageBusConfig::default());
        
        let mut receiver = bus.register_agent("agent1".to_string()).await.unwrap();
        
        let msg = Message::new(
            "agent2".to_string(),
            Some("agent1".to_string()),
            MessageType::TaskAssignment,
            serde_json::json!({"task": "test"}),
        );

        bus.send(msg).await.unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.sender_id, "agent2");
        assert_eq!(received.receiver_id, Some("agent1".to_string()));
    }

    #[tokio::test]
    async fn test_broadcast_messaging() {
        let bus = MessageBus::new(MessageBusConfig::default());
        
        let mut receiver1 = bus.register_agent("agent1".to_string()).await.unwrap();
        let mut receiver2 = bus.register_agent("agent2".to_string()).await.unwrap();
        
        let msg = Message::broadcast(
            "master".to_string(),
            MessageType::StatusUpdate,
            serde_json::json!({"status": "ready"}),
        );

        bus.send(msg).await.unwrap();

        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();
        
        assert_eq!(received1.id, received2.id);
        assert_eq!(received1.sender_id, "master");
        assert!(received1.is_broadcast());
    }
}