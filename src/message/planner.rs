use model_gateway_rs::model::llm::ChatMessage;

use crate::{
    input::UserTaskInput, message::llm::generate_assistant_tools,
    prompt::builder::build_task_prompt,
};

pub fn generate_planner_message(input: &UserTaskInput) -> Vec<ChatMessage> {
    let system_message: ChatMessage = generate_system_message();
    let tools_message: ChatMessage = generate_assistant_tools();
    let user_message: ChatMessage = generate_user_message(input);
    vec![system_message, tools_message, user_message]
}

fn generate_system_message() -> ChatMessage {
    let content = r#"
You are a task planning assistant.
Your only output should be valid JSON matching this structure.

Available actions:
- call_tool: Call one of the available tools (see tool list in assistant message)
- ask_user: Ask the user for input or clarification

For call_tool actions:
- Set "tool" to the tool name (e.g., "fetch_url", "summarize_text", "generate_chart")
- Set "parameters" to the tool parameters as JSON object

For ask_user actions:
- Set "tool" to null
- Set "input" to contain the question: {"question": "Your question here"}

Output JSON structure:
{
  "plan_id": "string",
  "description": "string", 
  "steps": [
    {
      "step_id": 1,
      "description": "string",
      "action": "call_tool or ask_user",
      "tool": "tool_name or null",
      "parameters": {"param": "value"} or null,
      "input": {"question": "text"} or null
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
