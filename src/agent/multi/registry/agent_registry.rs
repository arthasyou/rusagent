use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::agent::types::{AgentCapability, AgentStatus, AgentType};
use crate::error::{Result, Error};
use crate::error::agent_error::AgentError;

/// Agent信息
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub agent_type: AgentType,
    pub capabilities: Vec<AgentCapability>,
    pub status: AgentStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl AgentInfo {
    pub fn new(id: String, agent_type: AgentType, capabilities: Vec<AgentCapability>) -> Self {
        Self {
            id,
            agent_type,
            capabilities,
            status: AgentStatus::Active,
            last_heartbeat: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    /// 更新心跳时间
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
    }

    /// 检查是否存活（基于心跳）
    pub fn is_alive(&self, timeout: Duration) -> bool {
        Utc::now() - self.last_heartbeat < timeout
    }
}

/// Agent注册表配置
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// 心跳超时时间（秒）
    pub heartbeat_timeout_secs: i64,
    /// 清理间隔（秒）
    pub cleanup_interval_secs: u64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            heartbeat_timeout_secs: 30,
            cleanup_interval_secs: 60,
        }
    }
}

/// Agent注册表，管理所有Agent的注册信息
pub struct AgentRegistry {
    /// Agent信息映射（agent_id -> AgentInfo）
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// 能力索引（capability -> agent_ids）
    capability_index: Arc<RwLock<HashMap<AgentCapability, Vec<String>>>>,
    /// 类型索引（agent_type -> agent_ids）
    type_index: Arc<RwLock<HashMap<AgentType, Vec<String>>>>,
    /// 配置
    config: RegistryConfig,
}

impl AgentRegistry {
    /// 创建新的Agent注册表
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 注册Agent
    pub async fn register(&self, info: AgentInfo) -> Result<()> {
        let agent_id = info.id.clone();
        let agent_type = info.agent_type;
        let capabilities = info.capabilities.clone();

        // 注册基本信息
        self.agents.write().await.insert(agent_id.clone(), info);

        // 更新能力索引
        let mut cap_index = self.capability_index.write().await;
        for capability in capabilities {
            cap_index
                .entry(capability)
                .or_insert_with(Vec::new)
                .push(agent_id.clone());
        }

        // 更新类型索引
        self.type_index
            .write()
            .await
            .entry(agent_type)
            .or_insert_with(Vec::new)
            .push(agent_id.clone());

        info!("Agent {} registered successfully", agent_id);
        Ok(())
    }

    /// 注销Agent
    pub async fn unregister(&self, agent_id: &str) -> Result<()> {
        // 获取Agent信息
        let info = self
            .agents
            .write()
            .await
            .remove(agent_id)
            .ok_or_else(|| Error::AgentError(AgentError::AgentNotFound(agent_id.to_string())))?;

        // 从能力索引中移除
        let mut cap_index = self.capability_index.write().await;
        for capability in &info.capabilities {
            if let Some(agents) = cap_index.get_mut(capability) {
                agents.retain(|id| id != agent_id);
                if agents.is_empty() {
                    cap_index.remove(capability);
                }
            }
        }

        // 从类型索引中移除
        let mut type_index = self.type_index.write().await;
        if let Some(agents) = type_index.get_mut(&info.agent_type) {
            agents.retain(|id| id != agent_id);
            if agents.is_empty() {
                type_index.remove(&info.agent_type);
            }
        }

        info!("Agent {} unregistered successfully", agent_id);
        Ok(())
    }

    /// 更新Agent状态
    pub async fn update_status(&self, agent_id: &str, status: AgentStatus) -> Result<()> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| Error::AgentError(AgentError::AgentNotFound(agent_id.to_string())))?;
        
        agent.status = status;
        Ok(())
    }

    /// 更新心跳
    pub async fn heartbeat(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| Error::AgentError(AgentError::AgentNotFound(agent_id.to_string())))?;
        
        agent.update_heartbeat();
        Ok(())
    }

    /// 获取Agent信息
    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentInfo> {
        self.agents.read().await.get(agent_id).cloned()
    }

    /// 获取所有Agent
    pub async fn get_all_agents(&self) -> Vec<AgentInfo> {
        self.agents.read().await.values().cloned().collect()
    }

    /// 根据类型查找Agent
    pub async fn find_by_type(&self, agent_type: AgentType) -> Vec<AgentInfo> {
        let type_index = self.type_index.read().await;
        let agents = self.agents.read().await;

        if let Some(agent_ids) = type_index.get(&agent_type) {
            agent_ids
                .iter()
                .filter_map(|id| agents.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 根据能力查找Agent
    pub async fn find_by_capability(&self, capability: &AgentCapability) -> Vec<AgentInfo> {
        let cap_index = self.capability_index.read().await;
        let agents = self.agents.read().await;

        if let Some(agent_ids) = cap_index.get(capability) {
            agent_ids
                .iter()
                .filter_map(|id| agents.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 查找存活的Agent
    pub async fn find_alive_agents(&self) -> Vec<AgentInfo> {
        let timeout = Duration::seconds(self.config.heartbeat_timeout_secs);
        self.agents
            .read()
            .await
            .values()
            .filter(|agent| agent.is_alive(timeout))
            .cloned()
            .collect()
    }

    /// 查找空闲的Agent
    pub async fn find_idle_agents(&self) -> Vec<AgentInfo> {
        self.agents
            .read()
            .await
            .values()
            .filter(|agent| agent.status == AgentStatus::Idle)
            .cloned()
            .collect()
    }

    /// 清理死亡的Agent
    pub async fn cleanup_dead_agents(&self) -> Vec<String> {
        let timeout = Duration::seconds(self.config.heartbeat_timeout_secs);
        let agents = self.agents.read().await;
        
        let dead_agents: Vec<String> = agents
            .iter()
            .filter(|(_, agent)| !agent.is_alive(timeout))
            .map(|(id, _)| id.clone())
            .collect();

        drop(agents);

        for agent_id in &dead_agents {
            warn!("Cleaning up dead agent: {}", agent_id);
            let _ = self.unregister(agent_id).await;
        }

        dead_agents
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> RegistryStats {
        let agents = self.agents.read().await;
        let timeout = Duration::seconds(self.config.heartbeat_timeout_secs);

        let total = agents.len();
        let alive = agents.values().filter(|a| a.is_alive(timeout)).count();
        let idle = agents.values().filter(|a| a.status == AgentStatus::Idle).count();
        let busy = agents.values().filter(|a| a.status == AgentStatus::Busy).count();

        let mut by_type = HashMap::new();
        for agent in agents.values() {
            *by_type.entry(agent.agent_type).or_insert(0) += 1;
        }

        RegistryStats {
            total_agents: total,
            alive_agents: alive,
            idle_agents: idle,
            busy_agents: busy,
            agents_by_type: by_type,
        }
    }

    /// 启动清理任务
    pub fn start_cleanup_task(self: Arc<Self>) {
        let interval = self.config.cleanup_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval));
            
            loop {
                interval.tick().await;
                let dead_agents = self.cleanup_dead_agents().await;
                if !dead_agents.is_empty() {
                    info!("Cleaned up {} dead agents", dead_agents.len());
                }
            }
        });
    }
}

/// 注册表统计信息
#[derive(Debug)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub alive_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub agents_by_type: HashMap<AgentType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_registration() {
        let registry = AgentRegistry::new(RegistryConfig::default());
        
        let agent_info = AgentInfo::new(
            "test-agent".to_string(),
            AgentType::Executor,
            vec![AgentCapability::TaskExecution],
        );

        registry.register(agent_info).await.unwrap();

        let agent = registry.get_agent("test-agent").await.unwrap();
        assert_eq!(agent.id, "test-agent");
        assert_eq!(agent.agent_type, AgentType::Executor);
    }

    #[tokio::test]
    async fn test_find_by_capability() {
        let registry = AgentRegistry::new(RegistryConfig::default());
        
        let agent1 = AgentInfo::new(
            "agent1".to_string(),
            AgentType::Executor,
            vec![AgentCapability::TaskExecution],
        );
        
        let agent2 = AgentInfo::new(
            "agent2".to_string(),
            AgentType::Planner,
            vec![AgentCapability::TaskPlanning],
        );

        registry.register(agent1).await.unwrap();
        registry.register(agent2).await.unwrap();

        let executors = registry
            .find_by_capability(&AgentCapability::TaskExecution)
            .await;
        assert_eq!(executors.len(), 1);
        assert_eq!(executors[0].id, "agent1");
    }
}