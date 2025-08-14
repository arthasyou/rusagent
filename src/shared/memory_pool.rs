use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    agent::types::AccessLevel,
    error::{Error, Result, agent_error::AgentError},
};

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_level: AccessLevel,
    pub ttl: Option<i64>, // Time-to-live (seconds)
    pub metadata: serde_json::Value,
}

impl MemoryEntry {
    pub fn new(
        key: String,
        value: serde_json::Value,
        created_by: String,
        access_level: AccessLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            key,
            value,
            created_by,
            created_at: now,
            updated_at: now,
            access_level,
            ttl: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl_secs: i64) -> Self {
        self.ttl = Some(ttl_secs);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let elapsed = Utc::now() - self.created_at;
            elapsed.num_seconds() > ttl
        } else {
            false
        }
    }

    /// Update value
    pub fn update(&mut self, value: serde_json::Value) {
        self.value = value;
        self.updated_at = Utc::now();
    }
}

/// Shared memory interface
#[async_trait::async_trait]
pub trait SharedMemory: Send + Sync {
    /// Read memory
    async fn get(&self, key: &str) -> Option<MemoryEntry>;

    /// Write memory
    async fn set(&self, entry: MemoryEntry) -> Result<()>;

    /// Delete memory
    async fn delete(&self, key: &str) -> Result<()>;

    /// List all keys
    async fn list_keys(&self) -> Vec<String>;

    /// Clean up expired entries
    async fn cleanup_expired(&self) -> usize;
}

/// Memory pool, manages shared memory between agents
pub struct MemoryPool {
    /// Global memory (accessible by all agents)
    global_memory: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    /// Agent-specific memory (agent_id -> memories)
    agent_memory: Arc<RwLock<HashMap<String, HashMap<String, MemoryEntry>>>>,
    /// Memory pool configuration
    config: MemoryPoolConfig,
}

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    pub max_global_entries: usize,
    pub max_agent_entries: usize,
    pub enable_ttl: bool,
    pub cleanup_interval_secs: u64,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            max_global_entries: 10000,
            max_agent_entries: 1000,
            enable_ttl: true,
            cleanup_interval_secs: 60,
        }
    }
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(config: MemoryPoolConfig) -> Self {
        let pool = Self {
            global_memory: Arc::new(RwLock::new(HashMap::new())),
            agent_memory: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // Start cleanup task
        if pool.config.enable_ttl {
            pool.start_cleanup_task();
        }

        pool
    }

    /// Get global memory
    pub async fn get_global(&self, key: &str) -> Option<MemoryEntry> {
        let memory = self.global_memory.read().await;
        memory.get(key).cloned().filter(|entry| !entry.is_expired())
    }

    /// Set global memory
    pub async fn set_global(&self, entry: MemoryEntry) -> Result<()> {
        let mut memory = self.global_memory.write().await;

        // Check capacity limit
        if memory.len() >= self.config.max_global_entries && !memory.contains_key(&entry.key) {
            return Err(Error::AgentError(AgentError::ResourceExhausted(
                "Global memory pool is full".into(),
            )));
        }

        memory.insert(entry.key.clone(), entry);
        Ok(())
    }

    /// Get agent-specific memory
    pub async fn get_agent(&self, agent_id: &str, key: &str) -> Option<MemoryEntry> {
        let agent_memories = self.agent_memory.read().await;
        agent_memories
            .get(agent_id)
            .and_then(|memories| memories.get(key).cloned())
            .filter(|entry| !entry.is_expired())
    }

    /// Set agent-specific memory
    pub async fn set_agent(&self, agent_id: &str, entry: MemoryEntry) -> Result<()> {
        let mut agent_memories = self.agent_memory.write().await;
        let memories = agent_memories
            .entry(agent_id.to_string())
            .or_insert_with(HashMap::new);

        // Check capacity limit
        if memories.len() >= self.config.max_agent_entries && !memories.contains_key(&entry.key) {
            return Err(Error::AgentError(AgentError::ResourceExhausted(format!(
                "Agent {agent_id} memory is full"
            ))));
        }

        memories.insert(entry.key.clone(), entry);
        Ok(())
    }

    /// Delete global memory
    pub async fn delete_global(&self, key: &str) -> Result<()> {
        self.global_memory.write().await.remove(key);
        Ok(())
    }

    /// Delete agent-specific memory
    pub async fn delete_agent(&self, agent_id: &str, key: &str) -> Result<()> {
        if let Some(memories) = self.agent_memory.write().await.get_mut(agent_id) {
            memories.remove(key);
        }
        Ok(())
    }

    /// Clear all memory for an agent
    pub async fn clear_agent(&self, agent_id: &str) -> Result<()> {
        self.agent_memory.write().await.remove(agent_id);
        Ok(())
    }

    /// List global memory keys
    pub async fn list_global_keys(&self) -> Vec<String> {
        self.global_memory.read().await.keys().cloned().collect()
    }

    /// List agent memory keys
    pub async fn list_agent_keys(&self, agent_id: &str) -> Vec<String> {
        self.agent_memory
            .read()
            .await
            .get(agent_id)
            .map(|memories| memories.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Clean up expired memory entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut count = 0;

        // Clean up global memory
        let mut global_memory = self.global_memory.write().await;
        let expired_keys: Vec<String> = global_memory
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            global_memory.remove(&key);
            count += 1;
        }

        // Clean up agent memory
        let mut agent_memory = self.agent_memory.write().await;
        for (_, memories) in agent_memory.iter_mut() {
            let expired_keys: Vec<String> = memories
                .iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_keys {
                memories.remove(&key);
                count += 1;
            }
        }

        count
    }

    /// Start cleanup task
    fn start_cleanup_task(&self) {
        let global_memory = self.global_memory.clone();
        let agent_memory = self.agent_memory.clone();
        let interval = self.config.cleanup_interval_secs;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                interval.tick().await;

                let pool = MemoryPool {
                    global_memory: global_memory.clone(),
                    agent_memory: agent_memory.clone(),
                    config: MemoryPoolConfig::default(),
                };

                let cleaned = pool.cleanup_expired().await;
                if cleaned > 0 {
                    tracing::debug!("Cleaned up {} expired memory entries", cleaned);
                }
            }
        });
    }

    /// Get statistics
    pub async fn get_stats(&self) -> MemoryPoolStats {
        let global_count = self.global_memory.read().await.len();
        let agent_count: usize = self
            .agent_memory
            .read()
            .await
            .values()
            .map(|m| m.len())
            .sum();
        let agent_pools = self.agent_memory.read().await.len();

        MemoryPoolStats {
            global_entries: global_count,
            agent_entries: agent_count,
            agent_pools,
            max_global_entries: self.config.max_global_entries,
            max_agent_entries: self.config.max_agent_entries,
        }
    }
}

/// 内存池统计信息
#[derive(Debug)]
pub struct MemoryPoolStats {
    pub global_entries: usize,
    pub agent_entries: usize,
    pub agent_pools: usize,
    pub max_global_entries: usize,
    pub max_agent_entries: usize,
}

#[async_trait::async_trait]
impl SharedMemory for MemoryPool {
    async fn get(&self, key: &str) -> Option<MemoryEntry> {
        self.get_global(key).await
    }

    async fn set(&self, entry: MemoryEntry) -> Result<()> {
        self.set_global(entry).await
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.delete_global(key).await
    }

    async fn list_keys(&self) -> Vec<String> {
        self.list_global_keys().await
    }

    async fn cleanup_expired(&self) -> usize {
        self.cleanup_expired().await
    }
}
