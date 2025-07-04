use rusagent::{
    agent::planner::Planner, input::model::UserTaskInput, models::plan::Plan,
    utils::string_util::StripCodeBlock,
};
use serde_json::Value;

#[tokio::main]
async fn main() {
    // 模拟用户输入
    let user_input = UserTaskInput::default();

    let p = Planner::default();

    // 1️⃣ 调用 planner 生成任务计划
    let plan = p.generate_plan(&user_input).await.unwrap();
    let content = plan.first_message().unwrap();
    println!("{:?}", content);
    let c1 = content.strip_code_block();

    let value: Plan = serde_json::from_str(c1).unwrap();
    println!("{:#?}", value);
}
