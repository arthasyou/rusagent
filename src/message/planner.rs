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

IMPORTANT CONSTRAINTS:
- You can ONLY use tools explicitly listed in the assistant message
- DO NOT create, invent, or assume any tools that are not listed
- If a required tool is not available, use ask_user action to inform the user

Available actions:
- call_tool: Call one of the available tools (ONLY tools listed in assistant message)
- ask_user: Ask the user for input or clarification

For call_tool actions:
- Set "tool" to the exact tool name from the assistant message tool list
- Set "parameters" to match the tool's params_schema exactly
- Verify the tool exists in the assistant message before using it

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

CRITICAL: Only use tools that are explicitly listed in the assistant message.
Never include any notes, explanations, or natural language.
Only output the JSON in the exact structure above.
"#;
    ChatMessage::system(content)
}

fn generate_user_message(input: &UserTaskInput) -> ChatMessage {
    let content = build_task_prompt(input);
    ChatMessage::user(content.as_str())
}
