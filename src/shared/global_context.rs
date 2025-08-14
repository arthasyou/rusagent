use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::agent::types::RuntimeMode;

/// Global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub runtime_mode: RuntimeMode,
    pub max_concurrent_tasks: usize,
    pub task_timeout_secs: u64,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            runtime_mode: RuntimeMode::MultiAgent,
            max_concurrent_tasks: 10,
            task_timeout_secs: 300,
            enable_logging: true,
            log_level: "info".to_string(),
        }
    }
}

/// Runtime information
#[derive(Debug, Clone)]
pub struct RuntimeInfo {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub environment: String,
}

impl Default for RuntimeInfo {
    fn default() -> Self {
        Self {
            start_time: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
        }
    }
}

/// Global context shared by all Agents
#[derive(Clone, Debug)]
pub struct GlobalContext {
    /// Global configuration
    pub config: Arc<RwLock<GlobalConfig>>,
    /// Runtime information
    pub runtime_info: Arc<RuntimeInfo>,
    /// Shared data storage
    pub shared_data: Arc<RwLock<serde_json::Value>>,
    /// MCP tool registry reference (if needed)
    pub tool_registry: Option<Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>>,
}

impl GlobalContext {
    /// Create new global context
    pub fn new(config: GlobalConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            runtime_info: Arc::new(RuntimeInfo::default()),
            shared_data: Arc::new(RwLock::new(serde_json::json!({}))),
            tool_registry: None,
        }
    }

    /// Get read-only reference to configuration
    pub async fn get_config(&self) -> GlobalConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config<F>(&self, updater: F)
    where
        F: FnOnce(&mut GlobalConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut config);
    }

    /// Get shared data
    pub async fn get_shared_data(&self, key: &str) -> Option<serde_json::Value> {
        self.shared_data.read().await.get(key).cloned()
    }

    /// Set shared data
    pub async fn set_shared_data(&self, key: String, value: serde_json::Value) {
        if let Some(obj) = self.shared_data.write().await.as_object_mut() {
            obj.insert(key, value);
        }
    }

    /// Get runtime mode
    pub async fn get_runtime_mode(&self) -> RuntimeMode {
        self.config.read().await.runtime_mode
    }

    /// Check if multi-agent mode
    pub async fn is_multi_agent_mode(&self) -> bool {
        matches!(self.get_runtime_mode().await, RuntimeMode::MultiAgent)
    }
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self::new(GlobalConfig::default())
    }
}