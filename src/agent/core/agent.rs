// agent/task.rs

use uuid::Uuid;

use crate::agent::{
    context::AgentContext,
    execution::Executor,
    memory::Memory,
    planning::{AgentPlan, AgentStep},
    state::AgentState,
    types::StepStatus,
};

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub plan: AgentPlan,
    pub state: AgentState,
    pub memory: Memory, // Assuming Memory is defined in momory.rs
    pub context: AgentContext,
    pub executor: Executor,
}

impl Agent {
    pub fn new(plan: AgentPlan) -> Self {
        Self {
            id: Uuid::new_v4().simple().to_string(),
            plan,
            state: AgentState::default(),
            memory: Memory::default(),
            context: AgentContext::default(),
            executor: Executor::default(),
        }
    }

    pub async fn run_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // 查找下一个待执行的步骤
            let step_info = {
                self.plan
                    .steps
                    .iter()
                    .find(|step| match self.state.get_step_status(step.step_id) {
                        Some(StepStatus::Pending) | None => true,
                        _ => false,
                    })
                    .map(|step| {
                        (
                            step.step_id,
                            step.description.clone(),
                            step.action.clone(),
                            step.tool.clone(),
                            step.parameters.clone(),
                            step.input.clone(),
                        )
                    })
            };

            if let Some((step_id, description, action, tool, parameters, input)) = step_info {
                println!("🚀 执行 Step {}: {}", step_id, description);

                self.state.set_step_status(step_id, StepStatus::Executing);

                // 构造临时步骤对象
                let temp_step = AgentStep {
                    step_id,
                    description,
                    status: StepStatus::Executing,
                    action,
                    tool,
                    parameters,
                    input,
                    output: None,
                    is_succeeded: false,
                    error_code: None,
                    error_reason: None,
                };

                let result = self
                    .executor
                    .execute(&temp_step, &self.context, &self.memory)
                    .await;

                match result {
                    Ok(output) => {
                        println!("✅ 成功: {}", output.output);
                        self.state.set_step_status(step_id, StepStatus::Done);
                        self.state.append_result(step_id, output);

                        // 更新plan中的步骤状态
                        if let Some(plan_step) =
                            self.plan.steps.iter_mut().find(|s| s.step_id == step_id)
                        {
                            plan_step.status = StepStatus::Done;
                            plan_step.is_succeeded = true;
                        }
                    }
                    Err(err) => {
                        println!("❌ 失败: {:?}", err);
                        self.state.set_step_status(step_id, StepStatus::Failed);

                        // 更新plan中的步骤状态
                        if let Some(plan_step) =
                            self.plan.steps.iter_mut().find(|s| s.step_id == step_id)
                        {
                            plan_step.status = StepStatus::Failed;
                            plan_step.error_reason = Some(err.to_string());
                        }
                        break;
                    }
                }
            } else {
                // 没有更多待执行的步骤
                break;
            }
        }

        println!("🎯 Agent 任务执行完毕");
        Ok(())
    }
}
