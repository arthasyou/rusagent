use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTaskInput {
    /// 简短的任务目标，例如 "生成柱状图"
    pub goal: String,

    /// 任务数据，例如 "苹果 5 个，香蕉 7 个，橙子 3 个"
    pub content: String,

    /// 任务背景说明，例如 "用于月度水果采购报告"
    pub description: Option<String>,

    /// 特殊要求，例如 "适合投影，简洁"
    pub constraints: Option<String>,

    /// 附加引用（预留字段）
    pub references: Option<Vec<String>>,
}

impl UserTaskInput {
    pub fn new(
        goal: String,
        content: String,
        description: Option<String>,
        constraints: Option<String>,
        references: Option<Vec<String>>,
    ) -> Self {
        Self {
            goal,
            content,
            description,
            constraints,
            references,
        }
    }
}

impl Default for UserTaskInput {
    fn default() -> Self {
        Self {
            goal: "语料扩充".to_owned(),
            content: "先生成扩充大纲，再根据大纲的逐条进行扩充；大纲中每1条涉及1个知识块"
                .to_owned(),
            description: Some("适应于中医领域".to_owned()),
            constraints: Some("只能在中医领域使用".to_owned()),
            references: None,
        }
    }
}
