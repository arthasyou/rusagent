use crate::tools::{ToolInfo, model::TOOL_REGISTRY};

pub fn instantiate_tool() -> Vec<ToolInfo> {
    let registry = TOOL_REGISTRY.read().unwrap();

    registry
        .values()
        .map(|tool_info| ToolInfo {
            name: tool_info.name.clone(),
            description: tool_info.description.clone(),
            params_schema: tool_info.params_schema.clone(),
            mcp_server: tool_info.mcp_server.clone(),
        })
        .collect()
}

pub fn get_tool_info(tool_name: &str) -> Option<ToolInfo> {
    let registry = TOOL_REGISTRY.read().unwrap();
    registry.get(tool_name).map(|tool_info| ToolInfo {
        name: tool_info.name.clone(),
        description: tool_info.description.clone(),
        params_schema: tool_info.params_schema.clone(),
        mcp_server: tool_info.mcp_server.clone(),
    })
}

pub fn list_available_tools() -> Vec<String> {
    let registry = TOOL_REGISTRY.read().unwrap();
    registry.keys().cloned().collect()
}
