// agent/task.rs

use uuid::Uuid;

use crate::agent::{
    context::AgentContext, executor::Executor, momory::Memory, plan::AgentPlan, state::AgentState,
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
            memory: Memory::default(), // Assuming Memory has a default implementation
            context: AgentContext::default(),
            executor: Executor::default(),
        }
    }

    pub async fn run_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(step) = self.plan.next_pending_step(&self.state) {
            println!("🚀 执行 Step {}: {}", step.step_id, step.description);

            self.state
                .set_step_status(step.step_id, StepStatus::Executing);

            let result = self
                .executor
                .execute(&step, &self.context, &self.memory)
                .await;

            match result {
                Ok(output) => {
                    println!("✅ 成功: {}", output.output);
                    self.state.set_step_status(step.step_id, StepStatus::Done);
                    self.state.append_result(step.step_id, output);
                }
                Err(err) => {
                    println!("❌ 失败: {:?}", err);
                    self.state.set_step_status(step.step_id, StepStatus::Failed);
                    break;
                }
            }
        }

        println!("🎯 Agent 任务执行完毕");
        Ok(())
    }
}
