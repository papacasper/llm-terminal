use crate::config::Config;
use crate::llm::{ClaudeClient, LLMClient, OpenAIClient};
use crate::models::{App, AppMode, LLMProvider, Message};
use crate::terminal::TerminalEmulator;
use anyhow::{anyhow, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AppState {
    pub app: App,
    pub llm_clients: Vec<Arc<dyn LLMClient>>,
}

impl AppState {
    pub fn new() -> Self {
        let app = App::new();
        let settings = Config::load_settings();
        
        let mut app_with_settings = app;
        app_with_settings.settings = settings;

        let llm_clients = Self::create_llm_clients(&app_with_settings);

        Self {
            app: app_with_settings,
            llm_clients,
        }
    }

    fn create_llm_clients(app: &App) -> Vec<Arc<dyn LLMClient>> {
        let mut clients: Vec<Arc<dyn LLMClient>> = Vec::new();

        if let Some(ref claude_key) = app.settings.claude_api_key {
            clients.push(Arc::new(ClaudeClient::new(claude_key.clone())));
        }

        if let Some(ref openai_key) = app.settings.openai_api_key {
            clients.push(Arc::new(OpenAIClient::new(openai_key.clone())));
        }

        clients
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.app.mode {
            AppMode::Chat => self.handle_chat_key_event(key),
            AppMode::Terminal => self.handle_terminal_key_event(key),
            AppMode::Settings => self.handle_settings_key_event(key),
        }
    }

    fn handle_chat_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.quit();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.quit();
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.add_new_tab();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.close_current_tab();
            }
            KeyCode::Char(',') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.toggle_mode();
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.app.previous_tab();
                } else {
                    self.app.next_tab();
                }
            }
            KeyCode::Enter => {
                if !self.app.input_buffer.trim().is_empty() {
                    let message = self.app.input_buffer.clone();
                    self.app.input_buffer.clear();
                    return self.send_message(message);
                }
            }
            KeyCode::Backspace => {
                self.app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.app.input_buffer.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_terminal_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.quit();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.quit();
            }
            KeyCode::Char(',') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.toggle_mode();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.quit();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.quit();
            }
            KeyCode::Char(',') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.app.toggle_mode();
            }
            KeyCode::Esc => {
                self.app.toggle_mode();
            }
            _ => {}
        }
        Ok(())
    }

    fn send_message(&mut self, content: String) -> Result<()> {
        // Get provider, model, and add user message
        let (provider, model, messages) = {
            let current_tab = self.app.current_tab_mut()
                .ok_or_else(|| anyhow!("No current tab"))?;

            // Add user message
            let user_message = Message::user(content);
            current_tab.add_message(user_message);
            current_tab.set_waiting(true);

            (current_tab.provider.clone(), current_tab.model.clone(), current_tab.messages.clone())
        };

        // Find the appropriate client for this tab's provider
        let client = self.find_client_for_provider(&provider)?;
        let client_clone = client.clone();

        // Send message in background
        let (_tx, _rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let result = client_clone.send_message(&messages, &model).await;
            let _ = _tx.send(result).await;
        });

        // For now, we'll handle the response synchronously
        // In a real implementation, you'd want to handle this asynchronously
        // and update the UI when the response arrives
        Ok(())
    }


    pub async fn handle_llm_response(&mut self, response: Result<String>) -> Result<()> {
        let current_tab = self.app.current_tab_mut()
            .ok_or_else(|| anyhow!("No current tab"))?;

        current_tab.set_waiting(false);

        match response {
            Ok(content) => {
                let assistant_message = Message::assistant(content);
                current_tab.add_message(assistant_message);
            }
            Err(e) => {
                let error_message = Message::assistant(format!("Error: {}", e));
                current_tab.add_message(error_message);
            }
        }

        Ok(())
    }


    pub fn find_client_for_provider(&self, provider: &LLMProvider) -> Result<Arc<dyn LLMClient>> {
        self.llm_clients
            .iter()
            .find(|client| client.provider() == *provider)
            .cloned()
            .ok_or_else(|| anyhow!("No client available for provider: {:?}", provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let app_state = AppState::new();
        assert!(!app_state.app.tabs.is_empty());
        assert_eq!(app_state.app.current_tab, 0);
    }

    #[test]
    fn test_input_handling() {
        let mut app_state = AppState::new();
        
        // Test character input
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        app_state.handle_key_event(key).unwrap();
        assert_eq!(app_state.app.input_buffer, "h");
        
        // Test backspace
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        app_state.handle_key_event(key).unwrap();
        assert_eq!(app_state.app.input_buffer, "");
    }
}
