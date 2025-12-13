//! Defines the interface for interacting with large language models (LLMs).
//!
//! The [`Target`] trait abstracts away the differences between various API providers
//! (OpenAI, Anthropic, Google, Meta), allowing strategies to be run against any supported backend.

use crate::RedOxideResult;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde_json::json;

/// A trait representing a target system (LLM) to be tested.
///
/// Implement this trait to add support for new APIs or local models.
#[async_trait]
pub trait Target: Send + Sync {
    /// Sends a prompt to the target and returns the raw string response.
    ///
    /// # Arguments
    /// * `prompt` - The text prompt to send to the model.
    ///
    /// # Returns
    /// A `Result` containing the model's text response or an error if the network request failed.
    async fn send_prompt(&self, prompt: &str) -> RedOxideResult<String>;
}

/// Implementation for OpenAI's Chat Completion API (e.g., GPT-3.5, GPT-4).
///
/// Uses the `async-openai` crate.
pub struct OpenAITarget {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAITarget {
    /// Creates a new OpenAI target.
    ///
    /// # Arguments
    /// * `api_key` - Your OpenAI API key (starts with `sk-`).
    /// * `model` - The model identifier (e.g., `gpt-4`, `gpt-3.5-turbo`).
    pub fn new(api_key: String, model: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client, model }
    }
}

#[async_trait]
impl Target for OpenAITarget {
    async fn send_prompt(&self, prompt: &str) -> RedOxideResult<String> {
        let user_msg_struct = ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?;

        let message = ChatCompletionRequestMessage::User(user_msg_struct);

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![message])
            .build()?;

        let response = self.client.chat().create(request).await?;

        Ok(response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

/// Implementation for Anthropic's Messages API (e.g., Claude 3).
pub struct AnthropicTarget {
    client: HttpClient,
    api_key: String,
    model: String,
}

impl AnthropicTarget {
    /// Creates a new Anthropic target.
    ///
    /// # Arguments
    /// * `api_key` - Your Anthropic API key (starts with `sk-ant-`).
    /// * `model` - The model identifier (e.g., `claude-3-opus-20240229`).
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: HttpClient::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl Target for AnthropicTarget {
    async fn send_prompt(&self, prompt: &str) -> RedOxideResult<String> {
        let url = "https://api.anthropic.com/v1/messages";

        let body = json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": [{ "role": "user", "content": prompt }]
        });

        let res = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let json_resp: serde_json::Value = res.json().await?;

        let text = json_resp["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(text)
    }
}

/// Implementation for Local Models running via Ollama.
///
/// This target does not require an API key by default.
pub struct OllamaTarget {
    client: HttpClient,
    endpoint: String,
    model: String,
}

impl OllamaTarget {
    /// Creates a new Ollama target.
    ///
    /// # Arguments
    /// * `endpoint` - The base URL (usually `http://localhost:11434`).
    /// * `model` - The model tag (e.g., `llama3`, `mistral`).
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            client: HttpClient::new(),
            endpoint,
            model,
        }
    }
}

#[async_trait]
impl Target for OllamaTarget {
    async fn send_prompt(&self, prompt: &str) -> RedOxideResult<String> {
        let url = format!("{}/api/chat", self.endpoint);

        let body = json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": prompt }],
            "stream": false
        });

        let res = self.client.post(&url).json(&body).send().await?;

        let json_resp: serde_json::Value = res.json().await?;

        let text = json_resp["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(text)
    }
}

/// Implementation for Google's Gemini API (Generative Language).
pub struct GeminiTarget {
    client: HttpClient,
    api_key: String,
    model: String,
}

impl GeminiTarget {
    /// Creates a new Gemini target.
    ///
    /// # Arguments
    /// * `api_key` - Your Google AI Studio API key.
    /// * `model` - The model identifier (e.g., `gemini-pro`).
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: HttpClient::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl Target for GeminiTarget {
    async fn send_prompt(&self, prompt: &str) -> RedOxideResult<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }]
        });

        let res = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Gemini API Error: {}", error_text));
        }

        let json_resp: serde_json::Value = res.json().await?;

        let text = json_resp["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(text)
    }
}
