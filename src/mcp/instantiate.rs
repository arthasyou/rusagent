use mcp_client::{
    core::protocol::result::InitializeResult,
    registry::{get_mcp_registry, register_mcp_clients},
};

pub async fn init_mcp() {
    // Initialize the Model-Controller-Presenter (MCP) architecture
    // let registry = get_mcp_registry();
    let config = mcp_config();
    register_mcp_clients(config).await.unwrap();
    register_mcp_info().await;
}

fn mcp_config() -> Vec<(&'static str, &'static str)> {
    // Create and return the MCP configuration
    vec![
        ("chart", "http://localhost:18000/sse?service=corpus"),
        // ("counter", "http://localhost:18000/sse?service=counter"),
    ]
}

async fn register_mcp_info() {
    let registry = get_mcp_registry();
    let keys = registry.list_keys();
    for key in keys {
        let client = registry.get(&key).unwrap();
        let tools = client.initialize().await;
        let a: InitializeResult = tools.unwrap();
        println!("Tools for {}: {:?}", key, a);
    }
}
