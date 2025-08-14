use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agent::types::{Priority, TaskStatus, TaskType};
use crate::error::{Result, Error};
use crate::error::agent_error::AgentError;

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub status: TaskStatus,
    pub payload: serde_json::Value,
    pub dependencies: Vec<String>,
    pub assigned_to: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl Task {
    pub fn new(
        task_type: TaskType,
        priority: Priority,
        payload: serde_json::Value,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_type,
            priority,
            status: TaskStatus::Pending,
            payload,
            dependencies: Vec::new(),
            assigned_to: None,
            created_by,
            created_at: now,
            updated_at: now,
            deadline: None,
            metadata: serde_json::json!({}),
        }
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, task_id: String) {
        if !self.dependencies.contains(&task_id) {
            self.dependencies.push(task_id);
        }
    }

    /// 设置截止时间
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(deadline) = self.deadline {
            Utc::now() > deadline
        } else {
            false
        }
    }

    /// 更新状态
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

/// 任务队列
pub struct TaskQueue {
    /// 待处理任务（按优先级分组）
    pending: Arc<RwLock<HashMap<Priority, VecDeque<Task>>>>,
    /// 进行中的任务
    in_progress: Arc<RwLock<HashMap<String, Task>>>,
    /// 已完成的任务
    completed: Arc<RwLock<Vec<Task>>>,
    /// 失败的任务
    failed: Arc<RwLock<Vec<Task>>>,
    /// 任务依赖图
    dependencies: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        let mut pending = HashMap::new();
        pending.insert(Priority::Critical, VecDeque::new());
        pending.insert(Priority::High, VecDeque::new());
        pending.insert(Priority::Normal, VecDeque::new());
        pending.insert(Priority::Low, VecDeque::new());

        Self {
            pending: Arc::new(RwLock::new(pending)),
            in_progress: Arc::new(RwLock::new(HashMap::new())),
            completed: Arc::new(RwLock::new(Vec::new())),
            failed: Arc::new(RwLock::new(Vec::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 入队任务
    pub async fn enqueue(&self, task: Task) -> Result<()> {
        // 检查依赖
        if !task.dependencies.is_empty() {
            self.dependencies
                .write()
                .await
                .insert(task.id.clone(), task.dependencies.clone());
        }

        // 根据优先级入队
        self.pending
            .write()
            .await
            .get_mut(&task.priority)
            .ok_or_else(|| Error::AgentError(AgentError::InternalError("Invalid priority".into())))?
            .push_back(task);

        Ok(())
    }

    /// 出队任务（考虑优先级和依赖）
    pub async fn dequeue(&self) -> Option<Task> {
        let mut pending = self.pending.write().await;
        let in_progress = self.in_progress.read().await;
        let completed = self.completed.read().await;

        // 按优先级顺序检查
        for priority in [Priority::Critical, Priority::High, Priority::Normal, Priority::Low] {
            if let Some(queue) = pending.get_mut(&priority) {
                // 查找可执行的任务（依赖已满足）
                let mut index = None;
                for (i, task) in queue.iter().enumerate() {
                    if self.are_dependencies_satisfied(task, &in_progress, &completed).await {
                        index = Some(i);
                        break;
                    }
                }

                if let Some(i) = index {
                    return queue.remove(i);
                }
            }
        }

        None
    }

    /// 检查任务依赖是否满足
    async fn are_dependencies_satisfied(
        &self,
        task: &Task,
        in_progress: &HashMap<String, Task>,
        completed: &[Task],
    ) -> bool {
        if task.dependencies.is_empty() {
            return true;
        }

        for dep_id in &task.dependencies {
            // 检查是否在进行中
            if in_progress.contains_key(dep_id) {
                return false;
            }

            // 检查是否已完成
            let is_completed = completed.iter().any(|t| &t.id == dep_id);
            if !is_completed {
                return false;
            }
        }

        true
    }

    /// 标记任务开始
    pub async fn mark_in_progress(&self, mut task: Task) -> Result<()> {
        task.update_status(TaskStatus::InProgress);
        self.in_progress
            .write()
            .await
            .insert(task.id.clone(), task);
        Ok(())
    }

    /// 标记任务完成
    pub async fn mark_completed(&self, task_id: &str) -> Result<()> {
        if let Some(mut task) = self.in_progress.write().await.remove(task_id) {
            task.update_status(TaskStatus::Completed);
            self.completed.write().await.push(task);
            
            // 清理依赖信息
            self.dependencies.write().await.remove(task_id);
            Ok(())
        } else {
            Err(Error::AgentError(AgentError::TaskNotFound(task_id.to_string())))
        }
    }

    /// 标记任务失败
    pub async fn mark_failed(&self, task_id: &str, reason: String) -> Result<()> {
        if let Some(mut task) = self.in_progress.write().await.remove(task_id) {
            task.update_status(TaskStatus::Failed(reason));
            self.failed.write().await.push(task);
            Ok(())
        } else {
            Err(Error::AgentError(AgentError::TaskNotFound(task_id.to_string())))
        }
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        // 检查进行中
        if let Some(task) = self.in_progress.read().await.get(task_id) {
            return Some(task.status.clone());
        }

        // 检查已完成
        if let Some(task) = self.completed.read().await.iter().find(|t| t.id == task_id) {
            return Some(task.status.clone());
        }

        // 检查失败
        if let Some(task) = self.failed.read().await.iter().find(|t| t.id == task_id) {
            return Some(task.status.clone());
        }

        // 检查待处理
        for queue in self.pending.read().await.values() {
            if let Some(task) = queue.iter().find(|t| t.id == task_id) {
                return Some(task.status.clone());
            }
        }

        None
    }

    /// 获取所有待处理任务
    pub async fn get_all_pending(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        for queue in self.pending.read().await.values() {
            tasks.extend(queue.iter().cloned());
        }
        tasks
    }

    /// 获取队列大小
    pub fn size(&self) -> usize {
        // 这是一个简化版本，实际使用时应该使用异步版本
        0
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> TaskQueueStats {
        let pending_count: usize = self
            .pending
            .read()
            .await
            .values()
            .map(|q| q.len())
            .sum();

        TaskQueueStats {
            pending: pending_count,
            in_progress: self.in_progress.read().await.len(),
            completed: self.completed.read().await.len(),
            failed: self.failed.read().await.len(),
        }
    }

    /// 清理过期任务
    pub async fn cleanup_expired(&self) -> usize {
        let mut count = 0;
        
        for queue in self.pending.write().await.values_mut() {
            let expired: Vec<usize> = queue
                .iter()
                .enumerate()
                .filter(|(_, task)| task.is_expired())
                .map(|(i, _)| i)
                .collect();

            for i in expired.into_iter().rev() {
                queue.remove(i);
                count += 1;
            }
        }

        count
    }
}

/// 任务队列统计信息
#[derive(Debug)]
pub struct TaskQueueStats {
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub failed: usize,
}