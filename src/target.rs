use crate::RedOxideResult;
use async_openai::{
    config::OpenAIConfig,
    // Ensure ChatCompletionRequestMessage (the Enum) is imported
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

#[async_trait]
pub trait Target: Send + Sync {
    /// Sends a prompt to the target and returns the raw string response
    async fn send_prompt(&self, prompt: &str) -> RedOxideResult<String>;
}

pub struct OpenAITarget {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAITarget {
    pub fn new(api_key: String, model: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client, model }
    }
}

#[async_trait]
impl Target for OpenAITarget {
    async fn send_prompt(&self, prompt: &str) -> RedOxideResult<String> {
        // Build the specific User Message Struct
        let user_msg_struct = ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?;

        // Wrap it in the Generic Enum (This fixes the error)
        let message = ChatCompletionRequestMessage::User(user_msg_struct);

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![message]) // Pass as a Vec
            .build()?;

        let response = self.client.chat().create(request).await?;

        Ok(response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}
