use crate::{
    agent::{context::AgentContext, memory::Memory, planning::AgentStep, types::StepResult},
    error::agent_error::AgentError,
};

#[derive(Debug, Default, Clone)]
pub struct Verifier;

impl Verifier {
    pub fn verify(
        &self,
        step: &AgentStep,
        result: &StepResult,
        _context: &AgentContext,
        _memory: &Memory,
    ) -> Result<(), AgentError> {
        match step.action.as_str() {
            "call_tool" => {
                let value: serde_json::Value = serde_json::from_str(&result.output)?;
                if !value.get("result").is_some_and(|v| v.is_string()) {
                    return Err(AgentError::VerificationError(
                        "call_tool 的输出中缺少 result 字段或类型不正确".into(),
                    ));
                }
            }

            "ask_user" => {
                let value: serde_json::Value = serde_json::from_str(&result.output)?;
                if let Some(answer) = value.get("answer").and_then(|v| v.as_str()) {
                    if answer.trim().is_empty() {
                        return Err(AgentError::VerificationError(
                            "ask_user 的回答内容为空".into(),
                        ));
                    }
                } else {
                    return Err(AgentError::VerificationError(
                        "ask_user 的输出中缺少 answer 字段".into(),
                    ));
                }
            }

            unknown => {
                return Err(AgentError::VerificationError(format!(
                    "无法验证未知 action 类型: {unknown}"
                )));
            }
        }

        Ok(())
    }
}
