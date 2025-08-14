use std::{collections::HashMap, path::PathBuf};

use serde_json::Value;

/// Represents the context for an agent task, containing all relevant information and configuration.
///
/// # Fields
/// - `user_prompt`: The original intent or prompt for the task, provided by the user (e.g., "Create
///   a customer profile analysis").
/// - `available_tools`: A list of tool names currently available in the system (e.g., ["llm-query",
///   "ocr-extract"]).
/// - `input_files`: Paths to input data files, file contents, or file name lists required for the
///   task.
/// - `env_vars`: General environment parameters such as language, model name, user role, timezone,
///   etc.
/// - `config`: Optional structured task configuration, typically loaded from JSON or YAML.
/// - `session_id`: Optional session tracking identifier for logging, auditing, or context
///   continuation.
/// - `trace_id`: Optional trace identifier for tracking the execution flow.
/// - `timestamp`: Optional start time in UNIX milliseconds, used for TTL or timeout control.
/// - `metadata`: Custom key-value data for use by special tools or plugins.
#[derive(Debug, Default, Clone)]
pub struct AgentContext {
    pub user_prompt: Option<String>,
    pub available_tools: Vec<String>,
    pub input_files: Vec<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub config: Option<Value>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub timestamp: Option<u64>,
    pub metadata: HashMap<String, Value>,
}

// impl Default for AgentContext {
//     fn default() -> Self {
//         Self {
//             user_prompt: None,
//             available_tools: Vec::new(),
//             input_files: Vec::new(),
//             env_vars: HashMap::new(),
//             config: None,
//             session_id: None,
//             trace_id: None,
//             timestamp: None,
//             metadata: HashMap::new(),
//         }
//     }
// }
impl AgentContext {
    /// Adds a tool to the list of available tools.
    pub fn add_tool(&mut self, tool_name: String) {
        self.available_tools.push(tool_name);
    }

    /// Sets the user prompt for the context.
    pub fn set_user_prompt(&mut self, prompt: String) {
        self.user_prompt = Some(prompt);
    }
}
