use rusagent::mcp::instantiate::init_mcp;

#[tokio::main]
async fn main() {
    // 调用 build_tools_prompt
    init_mcp().await;
}
