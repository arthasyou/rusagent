use model_gateway_rs::model::llm::ChatMessage;

use crate::{
    input::UserTaskInput, message::llm::generate_assistant_tools,
    prompt::builder::build_task_prompt,
};

pub fn generate_planner_message(input: &UserTaskInput) -> Vec<ChatMessage> {
    let system_message: ChatMessage = generate_system_message();
    let assistant_tools: ChatMessage = generate_assistant_tools();
    let user_message: ChatMessage = generate_user_message(input);
    vec![system_message, assistant_tools, user_message]
}

fn generate_system_message() -> ChatMessage {
    let content = r#"
You are a task planning assistant.
Your only output should be valid JSON matching this structure.

Each field meaning:
- plan_id: Unique ID for this plan
- description: Plan description
- steps: A list of steps
- step_id: Step index (1-based)
- description: Step description
- action: What the step does (e.g., call_tool, ask_user)
- tool: Name of tool to call, or null if no tool needed
- parameters: Parameters for tool as JSON, or null
- input: Input for the step as JSON, or null

Output JSON example:
{
  "plan_id": "string",
  "description": "string",
  "steps": [
    {
      "step_id": 1,
      "description": "string",
      "action": "string",
      "tool": "string or null",
      "parameters": {} or null,
      "input": {} or null,
    }
  ]
}

Never include any notes, explanations, or natural language.
Only output the JSON in the exact structure above.
"#;
    ChatMessage::system(content)
}

fn generate_user_message(input: &UserTaskInput) -> ChatMessage {
    let content = build_task_prompt(&input);
    ChatMessage::user(content.as_str())
}
