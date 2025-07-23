use rusagent::{
    agent::{
        Agent,
        planning::{AgentPlan, Planner},
    },
    input::model::UserTaskInput,
    mcp::instantiate::init_mcp,
    utils::string_util::StripCodeBlock,
};

#[tokio::main]
async fn main() {
    println!("🚀 开始测试 rusagent 完整流程...");
    init_mcp().await;

    // 构造一个中医语料扩充任务，使用指定的文件 URL
    let user_input = UserTaskInput {
        goal: "使用指定文件进行中医语料扩充".to_string(),
        content: format!(
            "请使用以下文件进行中医语料扩充任务：\n内容文件: {}\n提示文件: {}",
            "https://minio.cyydm.shop/testbucket/upload/zy/ya.txt",
            "https://minio.cyydm.shop/testbucket/test/prompt.md"
        ),
        description: Some("中医语料扩充任务 - 使用前端上传的文件URL".to_string()),
        constraints: Some(format!(
            "必须使用这两个文件URL进行处理：\n- content_path: {}\n- prompt_path: {}",
            "https://minio.cyydm.shop/testbucket/upload/zy/ya.txt",
            "https://minio.cyydm.shop/testbucket/test/prompt.md"
        )),
        references: Some(vec![
            "https://minio.cyydm.shop/testbucket/upload/zy/ya.txt".to_string(),
            "https://minio.cyydm.shop/testbucket/test/prompt.md".to_string()
        ]),
    };

    println!("📋 用户任务: {}", user_input.goal);

    // 使用 Planner 生成计划
    let planner = Planner::default();
    println!("🧠 正在生成任务计划...");

    match planner.generate_plan(&user_input).await {
        Ok(plan_output) => {
            let content = plan_output.get_content();
            println!("📄 LLM原始输出:\n{}", content);

            let cleaned_content = content.strip_code_block();
            println!("🧹 清理后的JSON:\n{}", cleaned_content);

            // 尝试解析计划
            match serde_json::from_str::<AgentPlan>(cleaned_content) {
                Ok(plan) => {
                    println!("✅ 计划解析成功: {} 个步骤", plan.steps.len());

                    // 创建 Agent 并执行
                    let mut agent = Agent::new(plan);
                    println!("🤖 Agent 已创建并初始化 MCP");

                    // 执行任务
                    match agent.run_loop().await {
                        Ok(_) => println!("🎉 任务执行完成！"),
                        Err(e) => println!("❌ 任务执行失败: {}", e),
                    }
                }
                Err(e) => {
                    println!("❌ 计划JSON解析失败: {}", e);
                    println!("🔍 尝试手动检查JSON格式...");
                }
            }
        }
        Err(e) => {
            println!("❌ 计划生成失败: {}", e);
        }
    }
}
