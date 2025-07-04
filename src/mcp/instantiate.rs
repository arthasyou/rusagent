use mcp_client::registry::{get_mcp_registry, register_mcp_clients};

pub async fn init_mcp() {
    // Initialize the Model-Controller-Presenter (MCP) architecture
    // let registry = get_mcp_registry();
    let config = mcp_config();
    register_mcp_clients(config).await.unwrap();
    let chart_client = get_mcp_registry().get("chart").unwrap();
    let r = chart_client.get_tools(Some(123)).await.unwrap();
    println!("Chart tools: {:?}", r);
}

fn mcp_config() -> Vec<(&'static str, &'static str)> {
    // Create and return the MCP configuration
    vec![
        ("chart", "http://localhost:18000/sse?service=chart"),
        ("counter", "http://localhost:18000/sse?service=counter"),
    ]
}
