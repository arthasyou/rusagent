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
Your only output should be valid JSON matching this structure:

{
  "steps": [
    {
      "need_tool": true,
      "tool_name": "string",
      "description": "string",
      "params": [
        {
          "name": "string",
          "type": "string",
          "value": "string"
        }
      ]
    },
    {
      "need_tool": false,
      "description": "string",
      "result": "string"
    }
  ]
}

Never include any notes, explanations, or natural language. 
Only output the JSON.
"#;
    ChatMessage::system(content)
}

fn generate_user_message(input: &UserTaskInput) -> ChatMessage {
    let content = build_task_prompt(&input);
    ChatMessage::user(content.as_str())
}
