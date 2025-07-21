use rusagent::{
    agent::{
        Agent,
        plan::{AgentPlan, Planner},
    },
    input::model::UserTaskInput,
    utils::string_util::StripCodeBlock,
};

#[tokio::main]
async fn main() {
    // 模拟用户输入
    let user_input = UserTaskInput::default();

    let p = Planner::default();

    // 调用 planner 生成任务计划
    let plan = p.generate_plan(&user_input).await.unwrap();
    let content = plan.get_content();
    // println!("{:?}", content);
    let c1 = content.strip_code_block();

    // println!("{}", c1);

    let plan: AgentPlan = serde_json::from_str(c1).unwrap();
    let mut agent = Agent::new(plan);
    // println!("Agent ID: {:#?}", agent.plan);
    agent.run_loop().await.unwrap();
}
