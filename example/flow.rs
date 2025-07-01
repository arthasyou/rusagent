use rusagent::{agent::planner::Planner, input::model::UserTaskInput};

#[tokio::main]
async fn main() {
    // 模拟用户输入
    let user_input = UserTaskInput::default();

    let p = Planner::default();

    // 1️⃣ 调用 planner 生成任务计划
    let plan = p.generate_plan(user_input).await.unwrap();
    println!("🎯 生成的任务计划: {:?}", plan);
    // match plan {
    //     Ok(task_plan) => {
    //         println!("🎯 生成的任务计划: {:?}", task_plan);

    //         // 2️⃣ 调用 executor 执行任务计划
    //         let result = Executor::execute_plan(task_plan).await;
    //         match result {
    //             Ok(task_result) => {
    //                 println!("✅ 执行结果: {:?}", task_result);
    //             }
    //             Err(e) => {
    //                 eprintln!("❌ 执行任务失败: {}", e);
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("❌ 生成任务计划失败: {}", e);
    //     }
    // }
}
