use model_gateway_rs::{
    clients::llm::LlmClient,
    sdk::{ModelSDK, openai::OpenAIClient},
    traits::ModelClient,
};

use crate::{error::Result, input::model::UserTaskInput, prompt::builder::build_task_prompt};

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

    pub async fn generate_plan(&self, input: UserTaskInput) -> Result<O>
    where
        I: From<String> + Sync + Send + 'static,
    {
        let i = build_task_prompt(&input);
        let r: O = self.llm_client.infer(i.into()).await?;
        Ok(r)
    }
}

impl Default for Planner<OpenAIClient> {
    fn default() -> Self {
        Self {
            llm_client: LlmClient::new(
                OpenAIClient::new("", "http://localhost:11434/v1", "llama3.2").unwrap(),
            ),
        }
    }
}
