use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct ContextLoader;

impl ContextLoader {
    pub fn load_default_context() -> Result<Option<String>> {
        let mut files = vec![PathBuf::from("README.md")];
        if let Ok(entries) = std::fs::read_dir("src") {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                    files.push(entry.path());
                }
            }
        }
        let mut content = String::new();
        for file in files {
            if let Ok(text) = std::fs::read_to_string(&file) {
                content.push_str(&format!("\n--- {} ---\n", file.display()));
                content.push_str(&text);
            }
        }
        if content.is_empty() {
            Ok(None)
        } else {
            Ok(Some(content))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_context() {
        let ctx = ContextLoader::load_default_context().unwrap();
        assert!(ctx.is_some());
    }
}
