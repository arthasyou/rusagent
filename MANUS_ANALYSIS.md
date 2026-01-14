# rusagent 项目与“Manus”定位对比总结

本文基于仓库 `manus/` 目录中的市场调研图片内容，对“Manus”的定义进行提炼，并对当前 Rust 项目 `rusagent`（多 Agent 框架 + MCP 接入）做对比分析，判断它是否可以视为“一个非常简单的 Manus”。

## 1) 调研图里 Manus 是什么

从图片描述中，Manus 被定位为“通用智能体/全能型智能打工人”，核心区别不在于“更会聊天”，而在于“更能交付端到端任务结果（做事）”。

### 1.1 Manus 的关键特征

- **自主拆解 + 执行任务**：根据用户需求自主拆解目标，选择合适的方法并执行，而不是只做问答。
- **多任务并行处理**：能同时推进多个任务（例如写报告、分析数据、处理邮件等）以提升效率。
- **环境感知与自适应**：根据外部环境变化调整策略，不是固定响应式的对话 AI。
- **从对话到交付**：强调完成“端到端任务”，输出更接近可用的交付物（报告/表格/代码/方案等）。

### 1.2 “Manus 与 LLM / MCP 的关系”提炼

图片中明确提到：

- Manus 可以理解为 **LLM 的通用外壳（wrapper / orchestration layer）**，通过 **MCP 协议**来调用能力，提供更好的用户界面与工作流。
- MCP 被描述为标准化接口（“AI 与外部世界的 USB-C 接口”），并给出三大核心能力：
  - **Resources**：访问实时数据源（网页内容/数据库/文件）
  - **Tools**：调用外部功能与 API（搜索/计算/代码执行等）
  - **Prompts**：预设工作流程模板（任务流/角色定义）

## 2) rusagent 项目现状：像 Manus 的地方

从代码结构与示例来看，`rusagent` 确实具备“Manus-like”系统的一部分基础设施（偏框架/骨架）。

### 2.1 多 Agent 系统骨架

仓库包含多 Agent 的常见组件：

- `src/agents/`：`master_agent.rs`、`executor_agent.rs`、`monitor_agent.rs`、`planner_agent.rs`、`verifier_agent.rs` 等角色
- `src/multi_agent/`：
  - 通信：message bus / message types
  - 注册：agent registry
  - 管理：agent manager（spawn/shutdown）
  - 协调：task queue

示例 `examples/simple_multi_agent.rs` 展示了：

- spawn agent
- 发送 `TaskAssignment`
- 发送/广播 `StatusUpdate`
- 基本的“多 Agent + 消息”使用方式

### 2.2 MCP 接入与工具注册机制

`src/mcp/instantiate.rs` 做了 MCP client 注册与工具拉取，并将工具信息写入 `TOOL_REGISTRY`（`src/tools/model.rs`）。

这点与调研图中“Manus 通过 MCP 把 Tools/Resources/Prompts 接进来”的定位高度一致：项目已经具备 **“工具协议接入层 + 工具注册表”** 的雏形。

## 3) rusagent 距离“可用的 Manus”还差什么

当前项目更像“框架骨架/演示级实现”，距离调研图里的“通用智能体产品”还有明显缺口，主要集中在“规划-执行闭环”和“端到端交付”。

### 3.1 自主拆解/规划尚未落地

- `src/agents/master_agent.rs` 中 `decompose_task` 仍是占位：直接返回原任务（未做智能拆解）。
- `src/prompt/plan.rs` 目前为空文件，说明“计划/工作流提示词与结构”还未实现。

### 3.2 任务执行闭环在框架层面未完全接通

在 `src/multi_agent/manager/agent_manager.rs` 的 `agent_loop`：

- 收到消息时目前只做日志 `debug!`，并未将消息交给 `agent.process_message()` 执行。
- 真正的处理逻辑存在于 `handle_agent_message`，但主 loop 里并未使用它（代码里也留了 TODO）。

这会导致“发任务 → Agent 处理 → 回结果”的闭环在框架层面不完整，更像 demo 形态。

### 3.3 缺少 Computer Use / 环境操作能力

调研图里强调的能力之一是类似“Computer Use”（像人一样操作网页/应用/环境）。

当前项目主要是：

- 多 Agent 编排基础设施
- MCP 工具列表注册

但未见到：

- 浏览器/桌面自动化控制链路
- 文件/网页操作的端到端编排与验证

### 3.4 缺少 Artifacts（交付物）体系

调研图提到 Artifacts 输出（多模态/多样式的交付物）。

当前项目偏消息与状态更新，缺少：

- 产物存储/版本/引用（例如报告、表格、图表、代码文件）
- 任务产物与步骤的结构化关联

### 3.5 并行多任务与智能调度仍多为 TODO

项目具备 tokio、task queue 等基础，但：

- Master 的智能分配、并行计划、选择合适 executor 的策略仍未实现（多处 TODO）。

## 4) 结论：它是不是一个“非常简单的 Manus”

可以分两种理解来回答：

### 4.1 如果把 Manus 理解为“LLM 外壳 + 多 Agent 编排 + MCP 工具接入”

那么 `rusagent`：

- **方向很像**，且已经具备多 Agent 基础设施与 MCP 工具注册机制
- 更准确的表述是：**Manus-like 的雏形/骨架（框架级原型）**

### 4.2 如果把 Manus 理解为“能端到端自主完成复杂任务并交付产物的通用智能体产品”

那么 `rusagent` 目前：

- **还不是**可用的 Manus
- 更像是早期 demo/框架层实现，缺口集中在：
  - 规划与任务拆解
  - 调度与执行闭环
  - Computer Use / 环境操作
  - Artifacts 交付物体系

## 5) 下一步（可选）

若要把它推进到“最小可用 Manus（MVP）”，优先级通常是：

1. 先打通消息处理闭环（`AgentManager` 将消息交给 agent 执行并回传结果）
2. 实现最小 Planner → Executor 的计划结构与执行协议
3. 将 MCP tools 的调用串进 Executor，形成“计划步骤 → 工具调用 → 结果汇总”
4. 增加基础的 artifact 输出与持久化（哪怕先是文件输出/内存记录）

