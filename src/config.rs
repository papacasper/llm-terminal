use crate::models::Settings;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct Config;

impl Config {
    pub fn load_settings() -> Settings {
        let mut settings = Settings::default();

        // Load from environment variables first
        if let Ok(claude_key) = std::env::var("ANTHROPIC_API_KEY") {
            settings.claude_api_key = Some(claude_key);
        } else if let Ok(claude_key) = std::env::var("CLAUDE_API_KEY") {
            settings.claude_api_key = Some(claude_key);
        }

        if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            settings.openai_api_key = Some(openai_key);
        }

        if let Ok(t) = std::env::var("LLM_TERMINAL_TELEMETRY") {
            let v = t.to_lowercase();
            settings.telemetry_enabled = v != "0" && v != "false";
        }

        // Try to load from config file
        if let Ok(config_settings) = Self::load_from_file() {
            if settings.claude_api_key.is_none() {
                settings.claude_api_key = config_settings.claude_api_key;
            }
            if settings.openai_api_key.is_none() {
                settings.openai_api_key = config_settings.openai_api_key;
            }
            settings.default_provider = config_settings.default_provider;
            settings.telemetry_enabled = config_settings.telemetry_enabled;
        }

        settings
    }

    fn load_from_file() -> Result<Settings> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(Settings::default());
        }

        let content =
            std::fs::read_to_string(&config_path).context("Failed to read config file")?;

        let settings: Settings = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(settings)
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get config directory")?;

        Ok(config_dir.join("llm-terminal").join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LLMProvider;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.claude_api_key.is_none());
        assert!(settings.openai_api_key.is_none());
        assert_eq!(settings.default_provider, LLMProvider::Claude);
    }
}
