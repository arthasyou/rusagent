use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 消息类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// 任务分配
    TaskAssignment,
    /// 状态更新
    StatusUpdate,
    /// 结果通知
    ResultNotification,
    /// 资源请求
    ResourceRequest,
    /// 资源响应
    ResourceResponse,
    /// 心跳
    Heartbeat,
    /// 错误报告
    Error,
    /// 控制命令
    Control(ControlCommand),
    /// 自定义消息
    Custom(String),
}

/// 控制命令类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlCommand {
    Start,
    Stop,
    Pause,
    Resume,
    Shutdown,
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// 消息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息ID
    pub id: String,
    /// 发送者ID
    pub sender_id: String,
    /// 接收者ID（None表示广播）
    pub receiver_id: Option<String>,
    /// 消息类型
    pub message_type: MessageType,
    /// 消息优先级
    pub priority: MessagePriority,
    /// 消息内容
    pub payload: serde_json::Value,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 相关ID（用于关联请求和响应）
    pub correlation_id: Option<String>,
    /// 消息过期时间
    pub expires_at: Option<DateTime<Utc>>,
}

impl Message {
    /// 创建新消息
    pub fn new(
        sender_id: String,
        receiver_id: Option<String>,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender_id,
            receiver_id,
            message_type,
            priority: MessagePriority::default(),
            payload,
            timestamp: Utc::now(),
            correlation_id: None,
            expires_at: None,
        }
    }

    /// 创建广播消息
    pub fn broadcast(
        sender_id: String,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self::new(sender_id, None, message_type, payload)
    }

    /// 创建响应消息
    pub fn response(
        sender_id: String,
        receiver_id: String,
        message_type: MessageType,
        payload: serde_json::Value,
        correlation_id: String,
    ) -> Self {
        let mut msg = Self::new(sender_id, Some(receiver_id), message_type, payload);
        msg.correlation_id = Some(correlation_id);
        msg
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置过期时间
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// 检查消息是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 是否是广播消息
    pub fn is_broadcast(&self) -> bool {
        self.receiver_id.is_none()
    }
}

/// 消息过滤器
#[derive(Debug, Clone)]
pub struct MessageFilter {
    pub sender_id: Option<String>,
    pub message_types: Option<Vec<MessageType>>,
    pub min_priority: Option<MessagePriority>,
}

impl MessageFilter {
    pub fn new() -> Self {
        Self {
            sender_id: None,
            message_types: None,
            min_priority: None,
        }
    }

    pub fn from_sender(mut self, sender_id: String) -> Self {
        self.sender_id = Some(sender_id);
        self
    }

    pub fn with_types(mut self, types: Vec<MessageType>) -> Self {
        self.message_types = Some(types);
        self
    }

    pub fn with_min_priority(mut self, priority: MessagePriority) -> Self {
        self.min_priority = Some(priority);
        self
    }

    pub fn matches(&self, message: &Message) -> bool {
        // 检查发送者
        if let Some(ref sender) = self.sender_id {
            if &message.sender_id != sender {
                return false;
            }
        }

        // 检查消息类型
        if let Some(ref types) = self.message_types {
            if !types.contains(&message.message_type) {
                return false;
            }
        }

        // 检查优先级
        if let Some(min_priority) = self.min_priority {
            if message.priority < min_priority {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            "agent1".to_string(),
            Some("agent2".to_string()),
            MessageType::TaskAssignment,
            serde_json::json!({"task": "test"}),
        );

        assert_eq!(msg.sender_id, "agent1");
        assert_eq!(msg.receiver_id, Some("agent2".to_string()));
        assert!(!msg.is_broadcast());
    }

    #[test]
    fn test_broadcast_message() {
        let msg = Message::broadcast(
            "master".to_string(),
            MessageType::StatusUpdate,
            serde_json::json!({"status": "ready"}),
        );

        assert!(msg.is_broadcast());
        assert_eq!(msg.receiver_id, None);
    }

    #[test]
    fn test_message_filter() {
        let msg = Message::new(
            "agent1".to_string(),
            None,
            MessageType::TaskAssignment,
            serde_json::json!({}),
        )
        .with_priority(MessagePriority::High);

        let filter = MessageFilter::new()
            .from_sender("agent1".to_string())
            .with_min_priority(MessagePriority::Normal);

        assert!(filter.matches(&msg));
    }
}