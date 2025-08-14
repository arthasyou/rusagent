use mcp_client::registry::{get_mcp_registry, register_mcp_clients};

use crate::tools::model::{TOOL_REGISTRY, ToolInfo};

pub async fn init_mcp() {
    // Initialize the Model-Controller-Presenter (MCP) architecture
    let config = mcp_config();
    register_mcp_clients(config).await.unwrap();
    register_mcp_info().await;
}

fn mcp_config() -> Vec<(&'static str, &'static str)> {
    // Create and return the MCP configuration
    vec![
        ("corpus", "http://localhost:18000/sse?service=corpus"),
        // ("counter", "http://localhost:18000/sse?service=counter"),
    ]
}

async fn register_mcp_info() {
    let registry = get_mcp_registry();
    let keys = registry.list_keys();

    for key in keys {
        let client = registry.get(&key).unwrap();
        let tools_result = client.initialize().await;

        match tools_result {
            Ok(init_result) => {
                // Extract tools from InitializeResult and register them
                if let Some(_tools_capability) = init_result.capabilities.tools {
                    // Get the actual tools list from the server
                    match client.get_tools().await {
                        Ok(tools_list) => {
                            let mut tool_registry = TOOL_REGISTRY.write().unwrap();

                            for tool in tools_list {
                                let tool_name = tool.name.clone();
                                let tool_info = ToolInfo::new_with_server(
                                    tool.name,
                                    tool.description,
                                    tool.input_schema,
                                    key.clone(),
                                );

                                tool_registry.insert(tool_name.clone(), tool_info);
                                println!(
                                    "Registered tool from MCP server '{key}': {tool_name}"
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to list tools from MCP server '{key}': {e:?}");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize MCP server '{key}': {e:?}");
            }
        }
    }
}
