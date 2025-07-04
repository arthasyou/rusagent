use serde_json::Value;

pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub params_schema: Value,
}

impl ToolInfo {
    pub fn new(name: String, description: String, params_schema: Value) -> Self {
        Self {
            name,
            description,
            params_schema,
        }
    }
}
