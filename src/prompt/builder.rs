use crate::{input::UserTaskInput, tools::ToolInfo};

pub fn build_task_prompt(input: &UserTaskInput) -> String {
    let references = input
        .references
        .as_ref()
        .map(|refs| refs.join(", "))
        .unwrap_or_else(|| "None".to_string());

    let description = input.description.as_deref().unwrap_or("None");
    let constraints = input.constraints.as_deref().unwrap_or("None");

    format!(
        r#"
Please generate a task plan.

Task Goal: {}
Data Content: {}
Background Information: {}
Constraints: {}
References: {}
"#,
        input.goal, input.content, description, constraints, references
    )
}

pub fn build_tools_prompt(tools: &[ToolInfo]) -> String {
    let tools_text = tools
        .iter()
        .map(|tool| {
            format!(
                "\n - name: {}\n - description: {}\n - params_schema: {}",
                tool.name, tool.description, tool.params_schema
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("Available tools:\n{}", tools_text)
}
