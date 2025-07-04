use serde_json::json;

use crate::tools::ToolInfo;

pub fn instantiate_tool() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            name: "fetch_url".to_string(),
            description: "Fetches content from a given URL.".to_string(),
            params_schema: json!({
                "url": "string"
            }),
        },
        ToolInfo {
            name: "summarize_text".to_string(),
            description: "Summarizes provided text content.".to_string(),
            params_schema: json!({
                "text": "string"
            }),
        },
        ToolInfo {
            name: "generate_chart".to_string(),
            description: "Generates a chart from structured data.".to_string(),
            params_schema: json!({
                "type": "string",
                "data": "object"
            }),
        },
    ]
}
