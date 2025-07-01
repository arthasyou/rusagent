use super::model::UserTaskInput;

// TODO: 实现输入数据的校验与标准化逻辑
pub fn validate_input(input: &UserTaskInput) -> bool {
    // 简单示例：至少要有 goal
    !input.goal.trim().is_empty()
}
