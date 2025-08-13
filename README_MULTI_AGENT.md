# Multi-Agent System Documentation

## 概述

本项目已经从单agent架构成功转换为多agent协作架构。新架构支持多个专门化的agent并行工作，通过消息总线进行通信和协作。

## 核心组件

### 1. Agent类型

- **MasterAgent** - 主控Agent，负责任务分配和全局协调
- **PlannerAgent** - 规划Agent，负责任务规划和分解
- **ExecutorAgent** - 执行Agent，负责具体任务执行
- **VerifierAgent** - 验证Agent，负责结果验证
- **MonitorAgent** - 监控Agent，负责系统监控和健康检查

### 2. 通信机制

- **MessageBus** - 消息总线，支持点对点和广播消息
- **Message** - 统一的消息格式，包含发送者、接收者、消息类型和负载
- **MessageType** - 预定义的消息类型，如TaskAssignment、StatusUpdate等

### 3. 管理组件

- **AgentManager** - Agent生命周期管理器
- **AgentRegistry** - Agent注册表，支持按类型和能力查找
- **TaskQueue** - 任务队列，支持优先级和依赖管理
- **SharedMemory** - 共享内存池，用于agent间数据共享

## 运行示例

所有示例都已经修复并可以正常运行：

### 1. 简单多Agent示例
```bash
cargo run --example simple_multi_agent
```
展示基本的agent创建、消息发送和状态管理。这是最简单的入门示例。

### 2. 多Agent交互示例
```bash
cargo run --example agent_interaction
```
展示agent之间的实际交互，包括：
- 规划任务发送给PlannerAgent
- 执行任务发送给ExecutorAgent
- 验证请求发送给VerifierAgent
- 系统状态展示

### 3. 多Agent演示
```bash
cargo run --example multi_agent_demo
```
展示完整的多agent系统，包括所有5种agent类型的创建和基本交互。

### 4. 完整协作示例
```bash
cargo run --example multi_agent_collaboration
```
展示复杂的多agent协作流程，包括：
- 规划任务处理
- 执行任务分配
- 验证请求处理
- 状态更新广播
- 健康检查
- 任务协调

## 使用指南

### 创建Agent
```rust
// 创建主控Agent
let master = Box::new(MasterAgent::new(Some("master-001".to_string())));
let master_id = manager.spawn_agent(master).await?;

// 创建执行Agent，带有特定能力
let executor = Box::new(ExecutorAgent::new(
    Some("executor-001".to_string()),
    vec![AgentCapability::ToolCalling("web_search".to_string())],
));
let executor_id = manager.spawn_agent(executor).await?;
```

### 发送消息
```rust
// 点对点消息
let message = Message::new(
    "sender_id".to_string(),
    Some("receiver_id".to_string()),
    MessageType::TaskAssignment,
    serde_json::json!({"task": "example"}),
);
manager.send_message(message).await?;

// 广播消息
let broadcast = Message::broadcast(
    "sender_id".to_string(),
    MessageType::StatusUpdate,
    serde_json::json!({"status": "ready"}),
);
manager.broadcast_message(broadcast).await?;
```

### 查找Agent
```rust
// 按能力查找
let agents = manager.find_agents_by_capability(&AgentCapability::ToolCalling("web_search".to_string())).await;

// 按类型查找
let planners = manager.find_agents_by_type(AgentType::Planner).await;

// 查找空闲Agent
let idle_agents = manager.find_idle_agents().await;
```

## 架构特点

1. **松耦合设计** - Agent之间通过消息通信，无直接依赖
2. **可扩展性** - 易于添加新的Agent类型和消息类型
3. **容错性** - 支持Agent故障恢复和消息重试
4. **并行处理** - 多个Agent可以并行执行任务
5. **动态发现** - 通过注册表动态发现和选择合适的Agent

## 注意事项

1. **消息处理限制** - 当前实现中，agent的消息处理能力有限。Agent被移动到独立任务后，消息处理主要依赖于日志记录。未来版本将改进这一点。

2. **任务协调** - MasterAgent的任务分配逻辑目前是基础实现，需要根据具体业务需求进行扩展。

3. **性能考虑** - 在高并发场景下，消息总线可能成为瓶颈，需要进行性能优化。

## 后续改进建议

1. **增强消息处理** - 实现更完善的agent消息处理机制
2. **任务编排** - 添加更复杂的任务编排和工作流支持
3. **持久化** - 添加任务和消息的持久化支持
4. **监控增强** - 完善监控指标和告警机制
5. **动态扩缩容** - 支持根据负载动态创建和销毁Agent