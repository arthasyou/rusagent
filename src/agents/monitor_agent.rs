use async_trait::async_trait;
use chrono::{Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::agent::{
    core::base_agent::{AgentBehavior, BaseAgent},
    types::{AgentCapability, AgentType},
};
use crate::multi_agent::communication::{Message, MessageType};
use crate::shared::GlobalContext;
use crate::error::Result;

/// Monitoring metrics
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_task_duration: std::time::Duration,
    pub agent_health: HashMap<String, bool>,
    pub message_count: u64,
    pub error_count: u64,
}

/// Monitor Agent responsible for system monitoring and health checks
pub struct MonitorAgent {
    base: BaseAgent,
    metrics: Arc<RwLock<Metrics>>,
    alert_rules: Vec<AlertRule>,
    monitoring_interval: std::time::Duration,
}

/// Alert rule
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
}

/// Alert condition
#[derive(Debug, Clone)]
pub enum AlertCondition {
    ErrorRateHigh(f32),      // Error rate above threshold
    TaskFailureRate(f32),    // Task failure rate above threshold
    AgentUnhealthy(String),  // Specific Agent unhealthy
    MessageBacklog(usize),   // Message backlog
}

/// Alert severity
#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl MonitorAgent {
    pub fn new(id: Option<String>) -> Self {
        let id = id.unwrap_or_else(|| BaseAgent::generate_id(&AgentType::Monitor));
        let capabilities = vec![AgentCapability::Monitoring];

        Self {
            base: BaseAgent::new(id, AgentType::Monitor, capabilities),
            metrics: Arc::new(RwLock::new(Metrics::default())),
            alert_rules: Vec::new(),
            monitoring_interval: std::time::Duration::from_secs(10),
        }
    }

    /// Add alert rule
    pub fn add_alert_rule(mut self, rule: AlertRule) -> Self {
        self.alert_rules.push(rule);
        self
    }

    /// Set monitoring interval
    pub fn with_interval(mut self, interval: std::time::Duration) -> Self {
        self.monitoring_interval = interval;
        self
    }

    /// Handle status update
    async fn handle_status_update(&self, payload: serde_json::Value) -> Result<()> {
        let mut metrics = self.metrics.write().await;

        // Update task statistics
        if let Some(task_status) = payload.get("task_status").and_then(|v| v.as_str()) {
            match task_status {
                "completed" => metrics.completed_tasks += 1,
                "failed" => metrics.failed_tasks += 1,
                _ => {}
            }
            metrics.total_tasks += 1;
        }

        // Update Agent health status
        if let Some(agent_id) = payload.get("agent_id").and_then(|v| v.as_str())
            && let Some(healthy) = payload.get("healthy").and_then(|v| v.as_bool()) {
                metrics.agent_health.insert(agent_id.to_string(), healthy);
            }

        // Update message count
        metrics.message_count += 1;

        Ok(())
    }

    /// Check alert conditions
    async fn check_alerts(&self) -> Vec<(AlertRule, String)> {
        let metrics = self.metrics.read().await;
        let mut triggered_alerts = Vec::new();

        for rule in &self.alert_rules {
            let triggered = match &rule.condition {
                AlertCondition::ErrorRateHigh(threshold) => {
                    let error_rate = if metrics.message_count > 0 {
                        metrics.error_count as f32 / metrics.message_count as f32
                    } else {
                        0.0
                    };
                    if error_rate > *threshold {
                        Some(format!("Error rate {error_rate} exceeds threshold {threshold}"))
                    } else {
                        None
                    }
                }
                AlertCondition::TaskFailureRate(threshold) => {
                    let failure_rate = if metrics.total_tasks > 0 {
                        metrics.failed_tasks as f32 / metrics.total_tasks as f32
                    } else {
                        0.0
                    };
                    if failure_rate > *threshold {
                        Some(format!("Task failure rate {failure_rate} exceeds threshold {threshold}"))
                    } else {
                        None
                    }
                }
                AlertCondition::AgentUnhealthy(agent_id) => {
                    if let Some(&healthy) = metrics.agent_health.get(agent_id) {
                        if !healthy {
                            Some(format!("Agent {agent_id} is unhealthy"))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                AlertCondition::MessageBacklog(_threshold) => {
                    // TODO: Implement message backlog check
                    None
                }
            };

            if let Some(message) = triggered {
                triggered_alerts.push((rule.clone(), message));
            }
        }

        triggered_alerts
    }

    /// Send alert
    async fn send_alert(&self, rule: &AlertRule, message: String) -> Message {
        let severity_str = match rule.severity {
            AlertSeverity::Info => "INFO",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Critical => "CRITICAL",
        };

        warn!("[{}] Alert '{}': {}", severity_str, rule.name, message);

        Message::broadcast(
            self.base.id.clone(),
            MessageType::Custom("Alert".to_string()),
            serde_json::json!({
                "alert_name": rule.name,
                "severity": severity_str,
                "message": message,
                "timestamp": Utc::now(),
            }),
        )
    }

    /// Generate health report
    async fn generate_health_report(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        
        let error_rate = if metrics.message_count > 0 {
            metrics.error_count as f32 / metrics.message_count as f32
        } else {
            0.0
        };

        let success_rate = if metrics.total_tasks > 0 {
            metrics.completed_tasks as f32 / metrics.total_tasks as f32
        } else {
            0.0
        };

        serde_json::json!({
            "timestamp": Utc::now(),
            "overall_health": error_rate < 0.1 && success_rate > 0.8,
            "metrics": {
                "total_tasks": metrics.total_tasks,
                "completed_tasks": metrics.completed_tasks,
                "failed_tasks": metrics.failed_tasks,
                "error_rate": error_rate,
                "success_rate": success_rate,
                "message_count": metrics.message_count,
                "error_count": metrics.error_count,
            },
            "agent_health": metrics.agent_health,
        })
    }
}

#[async_trait]
impl AgentBehavior for MonitorAgent {
    fn get_id(&self) -> &str {
        &self.base.id
    }

    fn get_type(&self) -> AgentType {
        self.base.agent_type
    }

    fn get_capabilities(&self) -> &[AgentCapability] {
        &self.base.capabilities
    }

    async fn initialize(&mut self, context: Arc<GlobalContext>) -> Result<()> {
        self.base.context = Some(context);
        info!("MonitorAgent {} initialized with {} alert rules", 
              self.base.id, self.alert_rules.len());
        Ok(())
    }

    async fn process_message(&mut self, message: Message) -> Result<Option<Message>> {
        debug!("MonitorAgent {} processing message: {:?}", 
               self.base.id, message.message_type);

        match &message.message_type {
            MessageType::StatusUpdate => {
                self.handle_status_update(message.payload).await?;
                Ok(None)
            }
            MessageType::Error => {
                self.metrics.write().await.error_count += 1;
                Ok(None)
            }
            MessageType::Custom(msg_type) if msg_type == "HealthCheck" => {
                let report = self.generate_health_report().await;
                Ok(Some(Message::response(
                    self.base.id.clone(),
                    message.sender_id.clone(),
                    MessageType::ResultNotification,
                    report,
                    message.id.clone(),
                )))
            }
            _ => Ok(None)
        }
    }

    async fn run(&mut self) -> Result<()> {
        info!("MonitorAgent {} starting monitoring loop", self.base.id);
        
        let mut interval = tokio::time::interval(self.monitoring_interval);
        
        loop {
            interval.tick().await;
            
            // Check alerts
            let alerts = self.check_alerts().await;
            for (rule, message) in alerts {
                let _alert_msg = self.send_alert(&rule, message).await;
                // TODO: Send alert message
            }
            
            // Generate periodic health report
            if self.metrics.read().await.message_count % 100 == 0 {
                let report = self.generate_health_report().await;
                info!("Health report: {}", serde_json::to_string_pretty(&report)?);
            }
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("MonitorAgent {} shutting down", self.base.id);
        
        // Generate final report
        let final_report = self.generate_health_report().await;
        info!("Final health report: {}", serde_json::to_string_pretty(&final_report)?);
        
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn get_status(&self) -> serde_json::Value {
        let metrics = futures::executor::block_on(self.metrics.read());
        
        serde_json::json!({
            "id": self.base.id,
            "type": self.base.agent_type,
            "healthy": self.is_healthy(),
            "metrics_summary": {
                "total_tasks": metrics.total_tasks,
                "message_count": metrics.message_count,
                "error_count": metrics.error_count,
            },
            "alert_rules_count": self.alert_rules.len(),
        })
    }
}