use super::client::{HttpLLMClient, LLMClient, messages_to_api_format};
use crate::models::{LLMProvider, Message};
use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

pub struct ClaudeClient {
    http_client: HttpLLMClient,
    api_key: String,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http_client: HttpLLMClient::new(),
            api_key,
        }
    }

    fn create_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        Ok(headers)
    }

    async fn make_request(&self, messages: &[Message]) -> Result<String> {
        let headers = self.create_headers()?;
        let api_messages = messages_to_api_format(messages);

        let request_body = json!({
            "model": LLMProvider::Claude.model(),
            "max_tokens": 4096,
            "messages": api_messages
        });

        let response = self.http_client
            .client()
            .post("https://api.anthropic.com/v1/messages")
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Claude API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        // Extract the content from Claude's response format
        let content = response_json["content"]
            .as_array()
            .and_then(|arr| arr.get(0))
            .and_then(|obj| obj["text"].as_str())
            .ok_or_else(|| anyhow!("Invalid response format from Claude API"))?;

        Ok(content.to_string())
    }
}

#[async_trait::async_trait]
impl LLMClient for ClaudeClient {
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        if messages.is_empty() {
            return Err(anyhow!("No messages to send"));
        }

        self.make_request(messages).await
    }

    fn provider(&self) -> LLMProvider {
        LLMProvider::Claude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_client_creation() {
        let client = ClaudeClient::new("test-key".to_string());
        assert_eq!(client.provider(), LLMProvider::Claude);
    }

    #[test]
    fn test_create_headers() {
        let client = ClaudeClient::new("test-key".to_string());
        let headers = client.create_headers().unwrap();
        
        assert!(headers.contains_key("x-api-key"));
        assert!(headers.contains_key("anthropic-version"));
        assert!(headers.contains_key(CONTENT_TYPE));
    }
}
