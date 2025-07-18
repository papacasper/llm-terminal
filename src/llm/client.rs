use crate::models::{LLMProvider, Message, MessageRole};
use anyhow::{Context, Result};
use reqwest::Client;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait LLMClient: Send + Sync {
    async fn send_message(&self, messages: &[Message]) -> Result<String>;
    fn provider(&self) -> LLMProvider;
}

pub struct HttpLLMClient {
    client: Arc<Client>,
}

impl HttpLLMClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client: Arc::new(client),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to convert our Message format to API format
pub fn messages_to_api_format(messages: &[Message]) -> Vec<serde_json::Value> {
    messages
        .iter()
        .map(|msg| {
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };
            
            serde_json::json!({
                "role": role,
                "content": msg.content
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Message;

    #[test]
    fn test_messages_to_api_format() {
        let messages = vec![
            Message::user("Hello".to_string()),
            Message::assistant("Hi there!".to_string()),
        ];

        let api_messages = messages_to_api_format(&messages);
        
        assert_eq!(api_messages.len(), 2);
        assert_eq!(api_messages[0]["role"], "user");
        assert_eq!(api_messages[0]["content"], "Hello");
        assert_eq!(api_messages[1]["role"], "assistant");
        assert_eq!(api_messages[1]["content"], "Hi there!");
    }
}
