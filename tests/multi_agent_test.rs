#[cfg(test)]
mod multi_agent_tests {
    use std::sync::Arc;

    use rusagent::{
        agent::{
            core::base_agent::AgentBehavior,
            types::{AccessLevel, AgentCapability},
        },
        agents::{ExecutorAgent, MasterAgent},
        multi_agent::{Message, MessageBus, MessageBusConfig, MessageType},
        shared::{GlobalContext, MemoryEntry, MemoryPool},
    };

    #[tokio::test]
    async fn test_message_bus_communication() {
        // 创建消息总线
        let bus = MessageBus::new(MessageBusConfig::default());

        // 注册两个Agent
        let _receiver1 = bus.register_agent("agent1".to_string()).await.unwrap();
        let mut receiver2 = bus.register_agent("agent2".to_string()).await.unwrap();

        // Agent1发送消息给Agent2
        let msg = Message::new(
            "agent1".to_string(),
            Some("agent2".to_string()),
            MessageType::TaskAssignment,
            serde_json::json!({"task": "test"}),
        );

        bus.send(msg.clone()).await.unwrap();

        // Agent2应该收到消息
        let received = receiver2.recv().await.unwrap();
        assert_eq!(received.sender_id, "agent1");
        assert_eq!(received.receiver_id, Some("agent2".to_string()));
    }

    #[tokio::test]
    async fn test_broadcast_messaging() {
        let bus = MessageBus::new(MessageBusConfig::default());

        let mut receiver1 = bus.register_agent("agent1".to_string()).await.unwrap();
        let mut receiver2 = bus.register_agent("agent2".to_string()).await.unwrap();

        // 广播消息
        let msg = Message::broadcast(
            "master".to_string(),
            MessageType::StatusUpdate,
            serde_json::json!({"status": "ready"}),
        );

        bus.send(msg).await.unwrap();

        // 两个Agent都应该收到消息
        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();

        assert_eq!(received1.sender_id, "master");
        assert_eq!(received2.sender_id, "master");
        assert!(received1.is_broadcast());
        assert!(received2.is_broadcast());
    }

    #[tokio::test]
    async fn test_memory_pool() {
        let pool = MemoryPool::new(Default::default());

        // 设置全局内存
        let entry = MemoryEntry::new(
            "test_key".to_string(),
            serde_json::json!({"data": "test_value"}),
            "test_agent".to_string(),
            AccessLevel::Public,
        );

        pool.set_global(entry).await.unwrap();

        // 读取全局内存
        let retrieved = pool.get_global("test_key").await.unwrap();
        assert_eq!(retrieved.key, "test_key");
        assert_eq!(retrieved.created_by, "test_agent");

        // 设置Agent专属内存
        let agent_entry = MemoryEntry::new(
            "agent_key".to_string(),
            serde_json::json!({"private": "data"}),
            "agent1".to_string(),
            AccessLevel::Private,
        );

        pool.set_agent("agent1", agent_entry).await.unwrap();

        // 读取Agent内存
        let agent_data = pool.get_agent("agent1", "agent_key").await.unwrap();
        assert_eq!(agent_data.key, "agent_key");
    }

    #[tokio::test]
    async fn test_agent_initialization() {
        let context = Arc::new(GlobalContext::default());

        // 创建MasterAgent
        let mut master = MasterAgent::new(Some("test-master".to_string()));
        assert_eq!(master.get_id(), "test-master");
        assert_eq!(master.get_type(), rusagent::agent::types::AgentType::Master);

        // 初始化
        master.initialize(context.clone()).await.unwrap();
        assert!(master.is_healthy());

        // 创建ExecutorAgent
        let mut executor = ExecutorAgent::new(
            Some("test-executor".to_string()),
            vec![AgentCapability::ToolCalling("test_tool".to_string())],
        );

        executor.initialize(context).await.unwrap();

        let capabilities = executor.get_capabilities();
        assert!(capabilities.contains(&AgentCapability::TaskExecution));
        assert!(capabilities.contains(&AgentCapability::ToolCalling("test_tool".to_string())));
    }

    #[tokio::test]
    async fn test_message_priority() {
        use rusagent::multi_agent::communication::MessagePriority;

        let msg1 = Message::new(
            "sender".to_string(),
            Some("receiver".to_string()),
            MessageType::TaskAssignment,
            serde_json::json!({}),
        )
        .with_priority(MessagePriority::High);

        let msg2 = Message::new(
            "sender".to_string(),
            Some("receiver".to_string()),
            MessageType::StatusUpdate,
            serde_json::json!({}),
        )
        .with_priority(MessagePriority::Low);

        assert!(msg1.priority > msg2.priority);
    }

    #[tokio::test]
    async fn test_task_queue() {
        use rusagent::{
            agent::types::{Priority, TaskType},
            multi_agent::coordination::{Task, TaskQueue},
        };

        let queue = TaskQueue::new();

        // 创建高优先级任务
        let high_priority_task = Task::new(
            TaskType::Execution,
            Priority::High,
            serde_json::json!({"action": "important"}),
            "test_agent".to_string(),
        );

        // 创建普通优先级任务
        let normal_priority_task = Task::new(
            TaskType::Planning,
            Priority::Normal,
            serde_json::json!({"action": "normal"}),
            "test_agent".to_string(),
        );

        // 入队
        queue.enqueue(high_priority_task.clone()).await.unwrap();
        queue.enqueue(normal_priority_task.clone()).await.unwrap();

        // 出队应该先返回高优先级任务
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.priority, Priority::High);
    }
}
