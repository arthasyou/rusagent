use model_gateway_rs::{
    clients::llm::LlmClient,
    model::llm::ChatMessage,
    sdk::{ModelSDK, openai::OpenAIClient},
    traits::ModelClient,
};

use crate::{
    error::Result, input::model::UserTaskInput, message::planner::generate_planner_message,
};

pub struct Planner<T> {
    llm_client: LlmClient<T>,
}

impl<T, I, O> Planner<T>
where
    T: ModelSDK<Input = I, Output = O> + Sync + Send,
{
    pub fn new(llm_client: LlmClient<T>) -> Self {
        Self { llm_client }
    }

    pub async fn generate_plan(&self, input: &UserTaskInput) -> Result<O>
    where
        I: From<Vec<ChatMessage>> + Sync + Send + 'static,
    {
        let i = generate_planner_message(input);

        let r: O = self.llm_client.infer(i.into()).await?;
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
