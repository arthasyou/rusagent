use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::agent::types::RuntimeMode;

/// 全局配置
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

/// 运行时信息
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

/// 全局上下文，所有Agent共享的信息
#[derive(Clone, Debug)]
pub struct GlobalContext {
    /// 全局配置
    pub config: Arc<RwLock<GlobalConfig>>,
    /// 运行时信息
    pub runtime_info: Arc<RuntimeInfo>,
    /// 共享数据存储
    pub shared_data: Arc<RwLock<serde_json::Value>>,
    /// MCP工具注册表引用（如果需要）
    pub tool_registry: Option<Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>>,
}

impl GlobalContext {
    /// 创建新的全局上下文
    pub fn new(config: GlobalConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            runtime_info: Arc::new(RuntimeInfo::default()),
            shared_data: Arc::new(RwLock::new(serde_json::json!({}))),
            tool_registry: None,
        }
    }

    /// 获取配置的只读引用
    pub async fn get_config(&self) -> GlobalConfig {
        self.config.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config<F>(&self, updater: F)
    where
        F: FnOnce(&mut GlobalConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut *config);
    }

    /// 获取共享数据
    pub async fn get_shared_data(&self, key: &str) -> Option<serde_json::Value> {
        self.shared_data.read().await.get(key).cloned()
    }

    /// 设置共享数据
    pub async fn set_shared_data(&self, key: String, value: serde_json::Value) {
        if let Some(obj) = self.shared_data.write().await.as_object_mut() {
            obj.insert(key, value);
        }
    }

    /// 获取运行模式
    pub async fn get_runtime_mode(&self) -> RuntimeMode {
        self.config.read().await.runtime_mode
    }

    /// 是否是多Agent模式
    pub async fn is_multi_agent_mode(&self) -> bool {
        matches!(self.get_runtime_mode().await, RuntimeMode::MultiAgent)
    }
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self::new(GlobalConfig::default())
    }
}