use super::client::{messages_to_api_format, HttpLLMClient, LLMClient};
use crate::models::{LLMProvider, Message};
use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

pub struct OpenAIClient {
    http_client: HttpLLMClient,
    api_key: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http_client: HttpLLMClient::new(),
            api_key,
        }
    }

    fn create_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        );
        Ok(headers)
    }

    async fn make_request(&self, messages: &[Message], model: &str) -> Result<String> {
        let headers = self.create_headers()?;
        let api_messages = messages_to_api_format(messages);

        let request_body = json!({
            "model": model,
            "messages": api_messages,
            "max_tokens": 4096,
            "temperature": 0.7
        });

        let response = self
            .http_client
            .client()
            .post("https://api.openai.com/v1/chat/completions")
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenAI API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenAI API response")?;

        // Extract the content from OpenAI's response format
        let content = response_json["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj["message"]["content"].as_str())
            .ok_or_else(|| anyhow!("Invalid response format from OpenAI API"))?;

        Ok(content.to_string())
    }
}

#[async_trait::async_trait]
impl LLMClient for OpenAIClient {
    async fn send_message(&self, messages: &[Message], model: &str) -> Result<String> {
        if messages.is_empty() {
            return Err(anyhow!("No messages to send"));
        }

        self.make_request(messages, model).await
    }

    fn provider(&self) -> LLMProvider {
        LLMProvider::OpenAI
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new("test-key".to_string());
        assert_eq!(client.provider(), LLMProvider::OpenAI);
    }

    #[test]
    fn test_create_headers() {
        let client = OpenAIClient::new("test-key".to_string());
        let headers = client.create_headers().unwrap();

        assert!(headers.contains_key(AUTHORIZATION));
        assert!(headers.contains_key(CONTENT_TYPE));

        let auth_header = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert!(auth_header.starts_with("Bearer "));
    }
}
