use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Message type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Task assignment
    TaskAssignment,
    /// Status update
    StatusUpdate,
    /// Result notification
    ResultNotification,
    /// Resource request
    ResourceRequest,
    /// Resource response
    ResourceResponse,
    /// Heartbeat
    Heartbeat,
    /// Error report
    Error,
    /// Control command
    Control(ControlCommand),
    /// Custom message
    Custom(String),
}

/// Control command type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlCommand {
    Start,
    Stop,
    Pause,
    Resume,
    Shutdown,
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum MessagePriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Urgent = 3,
}


/// Message struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: String,
    /// Sender ID
    pub sender_id: String,
    /// Receiver ID (None means broadcast)
    pub receiver_id: Option<String>,
    /// Message type
    pub message_type: MessageType,
    /// Message priority
    pub priority: MessagePriority,
    /// Message content
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Correlation ID (for associating requests and responses)
    pub correlation_id: Option<String>,
    /// Message expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

impl Message {
    /// Create new message
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

    /// Create broadcast message
    pub fn broadcast(
        sender_id: String,
        message_type: MessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self::new(sender_id, None, message_type, payload)
    }

    /// Create response message
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

    /// Set priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set expiration time
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if broadcast message
    pub fn is_broadcast(&self) -> bool {
        self.receiver_id.is_none()
    }
}

/// Message filter
#[derive(Debug, Clone, Default)]
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
        // Check sender
        if let Some(ref sender) = self.sender_id
            && &message.sender_id != sender {
                return false;
            }

        // Check message type
        if let Some(ref types) = self.message_types
            && !types.contains(&message.message_type) {
                return false;
            }

        // Check priority
        if let Some(min_priority) = self.min_priority
            && message.priority < min_priority {
                return false;
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