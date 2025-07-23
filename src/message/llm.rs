use model_gateway_rs::model::llm::ChatMessage;

use crate::{
    input::UserTaskInput,
    prompt::builder::{build_task_prompt, build_tools_prompt},
    tools::instantiate::instantiate_tool,
};

impl From<UserTaskInput> for ChatMessage {
    fn from(input: UserTaskInput) -> Self {
        let content = build_task_prompt(&input);
        ChatMessage::user(content.as_str())
    }
}

pub fn generate_assistant_tools() -> ChatMessage {
    let tools = instantiate_tool();
    let content = build_tools_prompt(&tools);
    println!("🤖 Assistant Tools:\n{}", content);
    ChatMessage::assistant(content.as_str())
}
