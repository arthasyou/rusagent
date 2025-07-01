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
            goal: "生成柱状图".to_owned(),
            content: "苹果 5 个，香蕉 7 个，橙子 3 个".to_owned(),
            description: Some("用于月度水果采购报告".to_owned()),
            constraints: Some("适合投影，简洁".to_owned()),
            references: None,
        }
    }
}
