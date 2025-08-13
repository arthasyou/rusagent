use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::multi_agent::communication::message::{Message, MessageFilter};
use crate::error::{Result, Error};
use crate::error::agent_error::AgentError;

/// 消息总线配置
#[derive(Debug, Clone)]
pub struct MessageBusConfig {
    /// 广播通道容量
    pub broadcast_capacity: usize,
    /// 点对点通道容量
    pub p2p_capacity: usize,
    /// 消息历史记录大小
    pub history_size: usize,
    /// 是否启用消息持久化
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

/// 消息总线，负责Agent间的消息传递
pub struct MessageBus {
    /// 广播发送器
    broadcast_tx: broadcast::Sender<Arc<Message>>,
    /// 点对点通道映射（agent_id -> sender）
    p2p_channels: Arc<RwLock<HashMap<String, mpsc::Sender<Arc<Message>>>>>,
    /// 消息历史记录
    message_history: Arc<RwLock<Vec<Arc<Message>>>>,
    /// 配置
    config: MessageBusConfig,
    /// 统计信息
    stats: Arc<RwLock<MessageBusStats>>,
}

/// 消息总线统计信息
#[derive(Debug, Default)]
pub struct MessageBusStats {
    pub total_messages: u64,
    pub broadcast_messages: u64,
    pub p2p_messages: u64,
    pub failed_deliveries: u64,
    pub expired_messages: u64,
}

impl MessageBus {
    /// 创建新的消息总线
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

    /// 注册Agent，创建其专用的接收通道
    pub async fn register_agent(&self, agent_id: String) -> Result<MessageReceiver> {
        let (tx, rx) = mpsc::channel(self.config.p2p_capacity);
        
        // 注册点对点通道
        self.p2p_channels.write().await.insert(agent_id.clone(), tx);
        
        // 订阅广播通道
        let broadcast_rx = self.broadcast_tx.subscribe();
        
        info!("Agent {} registered to message bus", agent_id);

        Ok(MessageReceiver {
            agent_id,
            p2p_rx: rx,
            broadcast_rx,
        })
    }

    /// 注销Agent
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        self.p2p_channels.write().await.remove(agent_id);
        info!("Agent {} unregistered from message bus", agent_id);
        Ok(())
    }

    /// 发送消息
    pub async fn send(&self, message: Message) -> Result<()> {
        // 检查消息是否已过期
        if message.is_expired() {
            warn!("Attempting to send expired message: {}", message.id);
            self.stats.write().await.expired_messages += 1;
            return Err(Error::AgentError(AgentError::MessageDeliveryError("Message expired".into())));
        }

        let message = Arc::new(message);

        // 更新统计
        self.stats.write().await.total_messages += 1;

        // 保存到历史记录
        self.save_to_history(message.clone()).await;

        if message.is_broadcast() {
            // 广播消息
            self.broadcast(message).await
        } else {
            // 点对点消息
            self.send_p2p(message).await
        }
    }

    /// 发送广播消息
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

    /// 发送点对点消息
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
                        "Failed to send to {}",
                        receiver_id_string
                    ))))
                }
            }
        } else {
            warn!("Agent {} not found", receiver_id_string);
            self.stats.write().await.failed_deliveries += 1;
            Err(Error::AgentError(AgentError::MessageDeliveryError(format!(
                "Agent {} not found",
                receiver_id_string
            ))))
        }
    }

    /// 保存消息到历史记录
    async fn save_to_history(&self, message: Arc<Message>) {
        let mut history = self.message_history.write().await;
        
        // 限制历史记录大小
        if history.len() >= self.config.history_size {
            history.remove(0);
        }
        
        history.push(message);
    }

    /// 获取消息历史
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

    /// 获取统计信息
    pub async fn get_stats(&self) -> MessageBusStats {
        self.stats.read().await.clone()
    }

    /// 获取已注册的Agent列表
    pub async fn get_registered_agents(&self) -> Vec<String> {
        self.p2p_channels.read().await.keys().cloned().collect()
    }
}

/// 消息接收器，每个Agent持有一个
pub struct MessageReceiver {
    /// Agent ID
    pub agent_id: String,
    /// 点对点消息接收器
    p2p_rx: mpsc::Receiver<Arc<Message>>,
    /// 广播消息接收器
    broadcast_rx: broadcast::Receiver<Arc<Message>>,
}

impl MessageReceiver {
    /// 接收下一条消息（优先处理点对点消息）
    pub async fn recv(&mut self) -> Option<Arc<Message>> {
        tokio::select! {
            // 优先接收点对点消息
            Some(msg) = self.p2p_rx.recv() => {
                debug!("Agent {} received P2P message: {:?}", self.agent_id, msg.id);
                Some(msg)
            }
            // 然后接收广播消息
            Ok(msg) = self.broadcast_rx.recv() => {
                // 过滤掉自己发送的广播消息
                if msg.sender_id != self.agent_id {
                    debug!("Agent {} received broadcast message: {:?}", self.agent_id, msg.id);
                    Some(msg)
                } else {
                    // 递归调用继续接收下一条消息
                    Box::pin(self.recv()).await
                }
            }
            else => None
        }
    }

    /// 尝试接收消息（非阻塞）
    pub fn try_recv(&mut self) -> Option<Arc<Message>> {
        // 首先尝试接收点对点消息
        if let Ok(msg) = self.p2p_rx.try_recv() {
            return Some(msg);
        }

        // 然后尝试接收广播消息
        if let Ok(msg) = self.broadcast_rx.try_recv() {
            if msg.sender_id != self.agent_id {
                return Some(msg);
            }
        }

        None
    }

    /// 接收消息（带过滤器）
    pub async fn recv_filtered(&mut self, filter: MessageFilter) -> Option<Arc<Message>> {
        loop {
            if let Some(msg) = self.recv().await {
                if filter.matches(&msg) {
                    return Some(msg);
                }
                // 不匹配则继续接收
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