use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde_json::Value;

pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub params_schema: Value,
    pub mcp_server: String,
}

impl ToolInfo {
    pub fn new(
        name: String,
        description: String,
        params_schema: Value,
        mcp_server: String,
    ) -> Self {
        Self {
            name,
            description,
            params_schema,
            mcp_server,
        }
    }

    pub fn new_with_server(
        name: String,
        description: String,
        params_schema: Value,
        mcp_server: String,
    ) -> Self {
        Self {
            name,
            description,
            params_schema,
            mcp_server,
        }
    }
}

pub static TOOL_REGISTRY: Lazy<std::sync::RwLock<HashMap<String, ToolInfo>>> =
    Lazy::new(|| std::sync::RwLock::new(HashMap::new()));
