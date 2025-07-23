use model_gateway_rs::{
    clients::llm::LlmClient,
    model::llm::{LlmInput, LlmOutput},
    sdk::{ModelSDK, openai::OpenAIClient},
    traits::ModelClient,
};

use crate::{
    error::Result, input::model::UserTaskInput, message::planner::generate_planner_message,
};

pub struct Planner<T>
where
    T: ModelSDK<Input = LlmInput, Output = LlmOutput> + Sync + Send,
{
    llm_client: LlmClient<T>,
}

impl<T> Planner<T>
where
    T: ModelSDK<Input = LlmInput, Output = LlmOutput> + Sync + Send,
{
    pub fn new(llm_client: LlmClient<T>) -> Self {
        Self { llm_client }
    }

    pub async fn generate_plan(&self, input: &UserTaskInput) -> Result<LlmOutput> {
        let i = generate_planner_message(input);
        println!("📜 生成计划消息: {:?}", i);
        let input = LlmInput {
            messages: i,
            max_tokens: Some(4096),
        };

        let r: LlmOutput = self.llm_client.infer(input).await?;
        Ok(r)
    }
}

impl Default for Planner<OpenAIClient> {
    fn default() -> Self {
        Self {
            llm_client: LlmClient::new(
                OpenAIClient::new("", "http://192.168.1.64:11434/v1", "llama4:scout").unwrap(),
            ),
        }
    }
}
