use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LLMProvider {
    Claude,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaudeModel {
    Sonnet35, // Latest Claude 3.5 Sonnet (best for coding)
    Haiku35,  // Claude 3.5 Haiku (faster, still capable)
    Opus3,    // Claude 3 Opus (most capable for complex tasks)
    Sonnet3,  // Claude 3 Sonnet (good balance)
    Haiku3,   // Claude 3 Haiku (fastest, lighter tasks)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpenAIModel {
    GPT4o,
    GPT4oMini,
    GPT4Turbo,
    GPT35Turbo,
}

impl LLMProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            LLMProvider::Claude => "Claude",
            LLMProvider::OpenAI => "OpenAI",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Claude" => Some(LLMProvider::Claude),
            "OpenAI" => Some(LLMProvider::OpenAI),
            _ => None,
        }
    }

    pub fn default_model(&self) -> String {
        match self {
            // Claude 3.5 Sonnet is currently the best for coding tasks
            LLMProvider::Claude => ClaudeModel::Sonnet35.model_id(),
            // GPT-4o is OpenAI's most capable model for coding
            LLMProvider::OpenAI => OpenAIModel::GPT4o.model_id(),
        }
    }

    pub fn available_models(&self) -> Vec<String> {
        match self {
            // Ordered by coding capability and recency (best first)
            LLMProvider::Claude => vec![
                ClaudeModel::Sonnet35.model_id(), // Best for coding (latest)
                ClaudeModel::Haiku35.model_id(),  // Fast and capable (latest)
                ClaudeModel::Opus3.model_id(),    // Most capable for complex tasks
                ClaudeModel::Sonnet3.model_id(),  // Good balance
                ClaudeModel::Haiku3.model_id(),   // Legacy fast model
            ],
            // Ordered by coding capability (best first)
            LLMProvider::OpenAI => vec![
                OpenAIModel::GPT4o.model_id(),      // Best overall
                OpenAIModel::GPT4Turbo.model_id(),  // Good for complex tasks
                OpenAIModel::GPT4oMini.model_id(),  // Cost-effective
                OpenAIModel::GPT35Turbo.model_id(), // Legacy, still capable
            ],
        }
    }
}

impl ClaudeModel {
    pub fn model_id(&self) -> String {
        match self {
            // Latest models (as of 2024-2025)
            ClaudeModel::Sonnet35 => "claude-3-5-sonnet-20241022".to_string(),
            ClaudeModel::Haiku35 => "claude-3-5-haiku-20241022".to_string(),

            // Claude 3 series (stable)
            ClaudeModel::Opus3 => "claude-3-opus-20240229".to_string(),
            ClaudeModel::Sonnet3 => "claude-3-sonnet-20240229".to_string(),
            ClaudeModel::Haiku3 => "claude-3-haiku-20240307".to_string(),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ClaudeModel::Sonnet35 => "Claude 3.5 Sonnet (Latest)",
            ClaudeModel::Haiku35 => "Claude 3.5 Haiku (Latest)",
            ClaudeModel::Opus3 => "Claude 3 Opus",
            ClaudeModel::Sonnet3 => "Claude 3 Sonnet",
            ClaudeModel::Haiku3 => "Claude 3 Haiku",
        }
    }
}

impl OpenAIModel {
    pub fn model_id(&self) -> String {
        match self {
            OpenAIModel::GPT4o => "gpt-4o".to_string(),
            OpenAIModel::GPT4oMini => "gpt-4o-mini".to_string(),
            OpenAIModel::GPT4Turbo => "gpt-4-turbo".to_string(),
            OpenAIModel::GPT35Turbo => "gpt-3.5-turbo".to_string(),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            OpenAIModel::GPT4o => "GPT-4o",
            OpenAIModel::GPT4oMini => "GPT-4o Mini",
            OpenAIModel::GPT4Turbo => "GPT-4 Turbo",
            OpenAIModel::GPT35Turbo => "GPT-3.5 Turbo",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            role,
            content,
            timestamp: Utc::now(),
        }
    }

    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
}

#[derive(Debug, Clone)]
pub struct ChatTab {
    pub title: String,
    pub provider: LLMProvider,
    pub model: String,
    pub messages: Vec<Message>,
    pub is_waiting: bool,
    pub code_execution_enabled: bool,
}

impl ChatTab {
    pub fn new(title: String, provider: LLMProvider) -> Self {
        let model = provider.default_model();
        Self {
            title,
            provider: provider.clone(),
            model,
            messages: Vec::new(),
            is_waiting: false,
            code_execution_enabled: true,
        }
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn toggle_code_execution(&mut self) {
        self.code_execution_enabled = !self.code_execution_enabled;
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn set_waiting(&mut self, waiting: bool) {
        self.is_waiting = waiting;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub claude_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub default_provider: LLMProvider,
    pub telemetry_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            claude_api_key: None,
            openai_api_key: None,
            default_provider: LLMProvider::Claude,
            telemetry_enabled: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Chat,
    Terminal,
    Settings,
}

#[derive(Debug)]
pub struct App {
    pub tabs: Vec<ChatTab>,
    pub current_tab: usize,
    pub input_buffer: String,
    pub settings: Settings,
    pub mode: AppMode,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            tabs: Vec::new(),
            current_tab: 0,
            input_buffer: String::new(),
            settings: Settings::default(),
            mode: AppMode::Chat,
            should_quit: false,
        };

        // Create initial tab
        app.add_new_tab();
        app
    }

    pub fn add_new_tab(&mut self) {
        let tab_number = self.tabs.len() + 1;
        let title = format!("Chat {}", tab_number);
        let tab = ChatTab::new(title, self.settings.default_provider.clone());
        self.tabs.push(tab);
        self.current_tab = self.tabs.len() - 1;
    }

    pub fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.current_tab);
            if self.current_tab > 0 {
                self.current_tab -= 1;
            } else if self.current_tab >= self.tabs.len() {
                self.current_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.current_tab = (self.current_tab + 1) % self.tabs.len();
        }
    }

    pub fn previous_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.current_tab = if self.current_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.current_tab - 1
            };
        }
    }

    pub fn current_tab_mut(&mut self) -> Option<&mut ChatTab> {
        self.tabs.get_mut(self.current_tab)
    }

    pub fn current_tab(&self) -> Option<&ChatTab> {
        self.tabs.get(self.current_tab)
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::Chat => AppMode::Terminal,
            AppMode::Terminal => AppMode::Settings,
            AppMode::Settings => AppMode::Chat,
        };
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
