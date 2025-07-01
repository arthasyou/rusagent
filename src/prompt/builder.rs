use crate::{input::UserTaskInput, tools::ToolInfo};

pub fn build_task_prompt(input: &UserTaskInput) -> String {
    let references = input
        .references
        .as_ref()
        .map(|refs| refs.join(", "))
        .unwrap_or_else(|| "无".to_string());

    format!(
        "你是一个任务规划助手，请基于以下信息生成任务计划。\n输出必须是严格符合以下结构的 \
         JSON，不要输出除 JSON 外的其他内容：\n{{\n  \"steps\": [\n    {{ \"tool\": \"工具名称\", \
         \"params\": {{...}} }}\n  ]\n}}\n\n任务目标: {}\n数据内容: {}\n背景信息: {}\n限制条件: \
         {}\n参考资料: {}\n",
        input.goal,
        input.content,
        input.description.as_deref().unwrap_or("无"),
        input.constraints.as_deref().unwrap_or("无"),
        references
    )
}

pub fn build_tools_prompt(tools: &[ToolInfo]) -> String {
    tools
        .iter()
        .map(|tool| {
            format!(
                "- 工具名称: {}\n  功能: {}\n  参数格式: {}",
                tool.name, tool.description, tool.params_schema
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
