mod app;
mod config;
mod llm;
mod models;
mod terminal;
mod ui;

use anyhow::Result;
use app::AppState;
use models::{AppMode, MessageRole};
use std::process::Command;

// Simple terminal session for GUI (no async processes)
#[derive(Debug, Clone)]
struct SimpleTerminalSession {
    pub history: Vec<SimpleTerminalLine>,
    pub current_input: String,
}

#[derive(Debug, Clone)]
struct SimpleTerminalLine {
    pub content: String,
    pub line_type: SimpleTerminalLineType,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum SimpleTerminalLineType {
    Output,
    Error,
    System,
}

impl SimpleTerminalSession {
    fn new() -> Self {
        let mut session = Self {
            history: Vec::new(),
            current_input: String::new(),
        };

        // Add welcome message
        session.add_system_message("Terminal session started".to_string());
        session.add_system_message(format!(
            "Working directory: {}",
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .display()
        ));

        session
    }

    fn add_command(&mut self, command: String) {
        self.history.push(SimpleTerminalLine {
            content: format!("$ {}", command),
            line_type: SimpleTerminalLineType::System,
        });
    }

    fn add_output(&mut self, output: String) {
        for line in output.lines() {
            self.history.push(SimpleTerminalLine {
                content: line.to_string(),
                line_type: SimpleTerminalLineType::Output,
            });
        }
    }

    fn add_system_message(&mut self, message: String) {
        self.history.push(SimpleTerminalLine {
            content: message,
            line_type: SimpleTerminalLineType::System,
        });
    }
}

// GUI Application using egui
struct LLMTerminalApp {
    app_state: AppState,
    simple_terminal: SimpleTerminalSession,
}

impl LLMTerminalApp {
    fn new() -> Self {
        Self {
            app_state: AppState::new(),
            simple_terminal: SimpleTerminalSession::new(),
        }
    }
}

impl eframe::App for LLMTerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LLM Terminal Emulator");

            // Tab bar
            ui.horizontal(|ui| {
                let mut clicked_tab = None;
                let mut close_tab = None;

                for (i, tab) in self.app_state.app.tabs.iter().enumerate() {
                    let tab_name = if tab.is_waiting {
                        format!("{} ⏳", tab.title)
                    } else {
                        tab.title.clone()
                    };

                    // Create a horizontal group for each tab with close button
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Tab button
                            if ui
                                .selectable_label(i == self.app_state.app.current_tab, tab_name)
                                .clicked()
                            {
                                clicked_tab = Some(i);
                            }

                            // Close button (only show if more than one tab exists)
                            if self.app_state.app.tabs.len() > 1
                                && ui.small_button("×").on_hover_text("Close tab").clicked()
                            {
                                close_tab = Some(i);
                            }
                        });
                    });
                }

                // Handle tab selection
                if let Some(tab) = clicked_tab {
                    self.app_state.app.current_tab = tab;
                }

                // Handle tab closing
                if let Some(tab_index) = close_tab {
                    self.close_tab(tab_index);
                }

                // New tab button
                if ui.button("+ New Tab").clicked() {
                    self.app_state.app.add_new_tab();
                }
            });

            ui.separator();

            // Mode selector
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.app_state.app.mode, AppMode::Chat, "Chat");
                ui.selectable_value(&mut self.app_state.app.mode, AppMode::Terminal, "Terminal");
                ui.selectable_value(&mut self.app_state.app.mode, AppMode::Settings, "Settings");
            });

            ui.separator();

            match self.app_state.app.mode {
                AppMode::Chat => {
                    self.render_chat_mode(ui);
                }
                AppMode::Terminal => {
                    self.render_terminal_mode(ui);
                }
                AppMode::Settings => {
                    self.render_settings_mode(ui);
                }
            }
        });

        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}

impl LLMTerminalApp {
    fn render_chat_mode(&mut self, ui: &mut egui::Ui) {
        if let Some(current_tab) = self.app_state.app.current_tab() {
            ui.label(format!(
                "Chat with {} ({}) - Terminal Integration Enabled",
                current_tab.provider.as_str(),
                current_tab.model
            ));

            // Messages area
            egui::ScrollArea::vertical().show(ui, |ui| {
                for message in &current_tab.messages {
                    ui.horizontal(|ui| {
                        let (role_text, color) = match message.role {
                            MessageRole::User => ("You:", egui::Color32::LIGHT_BLUE),
                            MessageRole::Assistant => {
                                (current_tab.provider.as_str(), egui::Color32::LIGHT_GREEN)
                            }
                        };

                        ui.colored_label(color, role_text);
                    });

                    ui.label(&message.content);
                    ui.add_space(10.0);
                }
            });

            ui.separator();

            // Show recent terminal output in chat if available
            if !self.simple_terminal.history.is_empty() {
                ui.collapsing("Recent Terminal Activity", |ui| {
                    let recent_lines = self
                        .simple_terminal
                        .history
                        .iter()
                        .rev()
                        .take(5)
                        .collect::<Vec<_>>();
                    for line in recent_lines.iter().rev() {
                        let color = match line.line_type {
                            SimpleTerminalLineType::Output => egui::Color32::WHITE,
                            SimpleTerminalLineType::Error => egui::Color32::RED,
                            SimpleTerminalLineType::System => egui::Color32::GRAY,
                        };
                        ui.colored_label(color, &line.content);
                    }
                });
                ui.separator();
            }

            // Input area
            ui.horizontal(|ui| {
                let _response = ui.text_edit_multiline(&mut self.app_state.app.input_buffer);

                // Check if Enter was pressed
                let enter_pressed =
                    ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

                if (ui.button("Send").clicked() || enter_pressed)
                    && !self.app_state.app.input_buffer.trim().is_empty()
                {
                    let message = self.app_state.app.input_buffer.clone();
                    self.app_state.app.input_buffer.clear();

                    self.process_llm_message(message);
                }

                // Show hint for Enter key
                ui.label("💡 Press Enter to send (Shift+Enter for new line)");
            });
        }
    }

    fn render_terminal_mode(&mut self, ui: &mut egui::Ui) {
        ui.label("Terminal Emulator");

        // Terminal output area
        egui::ScrollArea::vertical().show(ui, |ui| {
            for line in &self.simple_terminal.history {
                let color = match line.line_type {
                    SimpleTerminalLineType::Output => egui::Color32::WHITE,
                    SimpleTerminalLineType::Error => egui::Color32::RED,
                    SimpleTerminalLineType::System => egui::Color32::GRAY,
                };

                ui.colored_label(color, &line.content);
            }
        });

        ui.separator();

        // Terminal input
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut self.simple_terminal.current_input);

            if (ui.button("Execute").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                && !self.simple_terminal.current_input.trim().is_empty()
            {
                let command = self.simple_terminal.current_input.clone();

                // Execute actual terminal commands
                self.simple_terminal.add_command(command.clone());

                // Execute the command and capture output
                match self.execute_shell_command(&command) {
                    Ok(output) => {
                        if !output.is_empty() {
                            self.simple_terminal.add_output(output);
                        } else {
                            self.simple_terminal
                                .add_output("Command completed successfully.".to_string());
                        }
                    }
                    Err(error) => {
                        self.simple_terminal.add_output(format!("Error: {}", error));
                    }
                }

                self.simple_terminal.current_input.clear();
            }
        });
    }

    fn render_settings_mode(&mut self, ui: &mut egui::Ui) {
        ui.label("Settings");

        ui.label("API Keys Configuration:");
        ui.label("• Set ANTHROPIC_API_KEY environment variable for Claude");
        ui.label("• Set OPENAI_API_KEY environment variable for OpenAI");

        ui.separator();

        ui.label("Available Providers:");
        for provider in ["Claude", "OpenAI"] {
            ui.label(format!(
                "• {} - {}",
                provider,
                if self
                    .app_state
                    .find_client_for_provider(
                        &provider
                            .parse::<models::LLMProvider>()
                            .unwrap_or(models::LLMProvider::Claude)
                    )
                    .is_ok()
                {
                    "✅ Configured"
                } else {
                    "❌ Not configured"
                }
            ));
        }
    }

    // Close a specific tab by index
    fn close_tab(&mut self, tab_index: usize) {
        if self.app_state.app.tabs.len() > 1 && tab_index < self.app_state.app.tabs.len() {
            self.app_state.app.tabs.remove(tab_index);

            // Adjust current_tab if necessary
            if tab_index <= self.app_state.app.current_tab && self.app_state.app.current_tab > 0 {
                self.app_state.app.current_tab -= 1;
            }

            // Ensure current_tab is within bounds
            if self.app_state.app.current_tab >= self.app_state.app.tabs.len() {
                self.app_state.app.current_tab = self.app_state.app.tabs.len().saturating_sub(1);
            }
        }
    }

    // Process LLM messages and detect/execute terminal commands
    fn process_llm_message(&mut self, message: String) {
        // Add user message to chat
        if let Some(current_tab) = self.app_state.app.current_tab_mut() {
            current_tab.add_message(models::Message::user(message.clone()));
        }

        // Parse and execute any terminal commands in the message
        let (response, executed_commands) = self.process_message_for_commands(&message);

        // Add LLM response to chat
        if let Some(current_tab) = self.app_state.app.current_tab_mut() {
            current_tab.add_message(models::Message::assistant(response));

            // If commands were executed, also show results
            if !executed_commands.is_empty() {
                let command_results = format!("\nExecuted {} command(s). Check terminal or recent activity above for results.", executed_commands.len());
                current_tab.add_message(models::Message::assistant(command_results));
            }
        }
    }

    // Parse message for terminal commands and execute them
    fn process_message_for_commands(&mut self, message: &str) -> (String, Vec<String>) {
        let mut executed_commands = Vec::new();

        // First check for explicit code blocks or command prefixes
        let explicit_commands = self.extract_explicit_commands(message);

        // Then intelligently determine what commands to run based on natural language
        let intelligent_commands = self.determine_commands_from_intent(message);

        // Combine both sets of commands
        let mut all_commands = explicit_commands;
        all_commands.extend(intelligent_commands);

        let response = if all_commands.is_empty() {
            // No commands to execute, provide a conversational response
            self.generate_conversational_response(message)
        } else {
            self.execute_commands_and_respond(all_commands, &mut executed_commands)
        };

        (response, executed_commands)
    }

    // Generate a helpful conversational response
    fn generate_conversational_response(&self, message: &str) -> String {
        format!(
            "I understand you said: \"{}\"\n\n{}",
            message.trim(),
            "I can help you with various tasks involving the terminal, such as:\n\
            • File and directory operations (create, list, move, copy, delete)\n\
            • System information (current directory, disk usage, running processes)\n\
            • Development tasks (git operations, building projects, running scripts)\n\
            • Text processing (searching, editing, viewing files)\n\n\
            Just ask me naturally, like:\n\
            • \"What files are in this directory?\"\n\
            • \"Create a new folder called 'projects'\"\n\
            • \"Show me the current directory\"\n\
            • \"Check if Python is installed\""
        )
    }

    // Execute commands and generate response
    fn execute_commands_and_respond(
        &mut self,
        commands: Vec<String>,
        executed_commands: &mut Vec<String>,
    ) -> String {
        let mut response = String::new();

        if commands.len() == 1 {
            response.push_str("I'll help you with that.\n\n");
        } else {
            response.push_str(&format!(
                "I'll execute {} commands to help you:\n\n",
                commands.len()
            ));
        }

        for command in commands {
            response.push_str(&format!("Running: `{}`\n", command));

            // Add command to terminal history
            self.simple_terminal.add_command(command.clone());

            // Execute the command
            match self.execute_shell_command(&command) {
                Ok(output) => {
                    if !output.is_empty() {
                        self.simple_terminal.add_output(output.clone());
                        response.push_str(&format!("{}\n\n", output));
                    } else {
                        self.simple_terminal
                            .add_output("Command completed successfully.".to_string());
                        response.push_str("✅ Done!\n\n");
                    }
                }
                Err(error) => {
                    let error_msg = format!("Error: {}", error);
                    self.simple_terminal.add_output(error_msg.clone());
                    response.push_str(&format!("❌ Error: {}\n\n", error));
                }
            }

            executed_commands.push(command);
        }

        response
    }

    // Extract explicit commands from code blocks and prefixes
    fn extract_explicit_commands(&self, message: &str) -> Vec<String> {
        let mut commands = Vec::new();

        // Pattern 1: Code blocks (```bash, ```shell, ```cmd, or just ```)
        let code_block_patterns = [
            ("```bash\n", "\n```"),
            ("```shell\n", "\n```"),
            ("```cmd\n", "\n```"),
            ("```powershell\n", "\n```"),
            ("```\n", "\n```"), // Generic code block
        ];

        for (start, end) in code_block_patterns {
            if let Some(start_pos) = message.find(start) {
                let content_start = start_pos + start.len();
                if let Some(end_pos) = message[content_start..].find(end) {
                    let command = message[content_start..content_start + end_pos].trim();
                    if !command.is_empty() {
                        for line in command.lines() {
                            let line = line.trim();
                            if !line.is_empty() && !line.starts_with('#') {
                                commands.push(line.to_string());
                            }
                        }
                    }
                }
            }
        }

        commands
    }

    // Intelligently determine commands based on natural language intent
    fn determine_commands_from_intent(&self, message: &str) -> Vec<String> {
        let message_lower = message.to_lowercase();
        let mut commands = Vec::new();

        // File and directory listing
        if message_lower.contains("list")
            && (message_lower.contains("file") || message_lower.contains("director"))
            || message_lower.contains("what")
                && (message_lower.contains("file") || message_lower.contains("folder"))
            || message_lower.contains("show")
                && (message_lower.contains("file") || message_lower.contains("content"))
        {
            if cfg!(target_os = "windows") {
                commands.push("dir".to_string());
            } else {
                commands.push("ls -la".to_string());
            }
        }
        // Current directory
        else if message_lower.contains("current") && message_lower.contains("director")
            || message_lower.contains("where am i")
            || message_lower.contains("working director")
        {
            if cfg!(target_os = "windows") {
                commands.push("cd".to_string());
            } else {
                commands.push("pwd".to_string());
            }
        }
        // Create directory/folder
        else if (message_lower.contains("create") || message_lower.contains("make"))
            && (message_lower.contains("folder") || message_lower.contains("director"))
        {
            if let Some(name) =
                self.extract_name_from_message(&message_lower, &["folder", "directory"])
            {
                commands.push(format!("mkdir {}", name));
            }
        }
        // Create file
        else if (message_lower.contains("create") || message_lower.contains("make"))
            && message_lower.contains("file")
        {
            if let Some(name) = self.extract_name_from_message(&message_lower, &["file"]) {
                if cfg!(target_os = "windows") {
                    commands.push(format!("New-Item -ItemType File -Name {}", name));
                } else {
                    commands.push(format!("touch {}", name));
                }
            }
        }
        // Check system information
        else if message_lower.contains("system") && message_lower.contains("info")
            || message_lower.contains("computer") && message_lower.contains("info")
        {
            if cfg!(target_os = "windows") {
                commands.push(
                    "systeminfo | Select-String 'OS Name', 'OS Version', 'System Type'".to_string(),
                );
            } else {
                commands.push("uname -a".to_string());
            }
        }
        // Check if software is installed
        else if message_lower.contains("check")
            && (message_lower.contains("installed") || message_lower.contains("available"))
        {
            if message_lower.contains("python") {
                commands.push("python --version".to_string());
            } else if message_lower.contains("node") || message_lower.contains("nodejs") {
                commands.push("node --version".to_string());
            } else if message_lower.contains("git") {
                commands.push("git --version".to_string());
            } else if message_lower.contains("cargo") || message_lower.contains("rust") {
                commands.push("cargo --version".to_string());
            }
        }
        // Git operations
        else if message_lower.contains("git") {
            if message_lower.contains("status") {
                commands.push("git status".to_string());
            } else if message_lower.contains("log") {
                commands.push("git log --oneline -10".to_string());
            } else if message_lower.contains("branch") {
                commands.push("git branch -a".to_string());
            }
        }
        // Disk usage
        else if message_lower.contains("disk")
            && (message_lower.contains("space") || message_lower.contains("usage"))
        {
            if cfg!(target_os = "windows") {
                commands.push("Get-WmiObject -Class Win32_LogicalDisk | Select-Object DeviceID,Size,FreeSpace".to_string());
            } else {
                commands.push("df -h".to_string());
            }
        }
        // Process list
        else if message_lower.contains("process")
            && (message_lower.contains("list") || message_lower.contains("running"))
        {
            if cfg!(target_os = "windows") {
                commands.push("Get-Process | Select-Object ProcessName, Id, CPU | Sort-Object CPU -Descending | Select-Object -First 10".to_string());
            } else {
                commands.push("ps aux | head -10".to_string());
            }
        }

        commands
    }

    // Extract name/identifier from natural language message
    fn extract_name_from_message(&self, message: &str, keywords: &[&str]) -> Option<String> {
        for keyword in keywords {
            if let Some(pos) = message.find(keyword) {
                let after_keyword = &message[pos + keyword.len()..];

                // Look for common patterns like "called 'name'" or "named 'name'"
                if let Some(start) = after_keyword
                    .find("called")
                    .or_else(|| after_keyword.find("named"))
                {
                    let name_part = &after_keyword[start + 6..].trim(); // Skip "called" or "named"

                    // Extract quoted names
                    if let Some(quote_start) = name_part.find("'").or_else(|| name_part.find("\""))
                    {
                        let quote_char = name_part.chars().nth(quote_start).unwrap();
                        let name_start = quote_start + 1;
                        if let Some(quote_end) = name_part[name_start..].find(quote_char) {
                            let name = &name_part[name_start..name_start + quote_end];
                            if !name.is_empty() {
                                return Some(name.to_string());
                            }
                        }
                    }

                    // Extract unquoted single word names
                    let words: Vec<&str> = name_part.split_whitespace().collect();
                    if !words.is_empty() && !words[0].is_empty() {
                        return Some(words[0].to_string());
                    }
                }
            }
        }
        None
    }

    // Legacy method - keeping for compatibility
    #[allow(dead_code)]
    fn extract_commands(&self, message: &str) -> Vec<String> {
        let mut commands = Vec::new();

        // Pattern 1: Code blocks (```bash, ```shell, ```cmd, or just ```)
        let code_block_patterns = [
            ("```bash\n", "\n```"),
            ("```shell\n", "\n```"),
            ("```cmd\n", "\n```"),
            ("```powershell\n", "\n```"),
            ("```\n", "\n```"), // Generic code block
        ];

        for (start, end) in code_block_patterns {
            if let Some(start_pos) = message.find(start) {
                let content_start = start_pos + start.len();
                if let Some(end_pos) = message[content_start..].find(end) {
                    let command = message[content_start..content_start + end_pos].trim();
                    if !command.is_empty() {
                        for line in command.lines() {
                            let line = line.trim();
                            if !line.is_empty() && !line.starts_with('#') {
                                commands.push(line.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Pattern 2: Explicit command prefixes
        let command_prefixes = ["run:", "execute:", "cmd:", "command:"];

        for line in message.lines() {
            let line = line.trim();
            for prefix in &command_prefixes {
                if line.to_lowercase().starts_with(prefix) {
                    let command = line[prefix.len()..].trim();
                    if !command.is_empty() {
                        commands.push(command.to_string());
                    }
                }
            }
        }

        // Pattern 3: Inline backticks for single commands
        let backtick_regex = "`([^`]+)`";
        if let Ok(re) = regex::Regex::new(backtick_regex) {
            for cap in re.captures_iter(message) {
                if let Some(command) = cap.get(1) {
                    let cmd = command.as_str().trim();
                    // Only consider as command if it looks like a shell command
                    if self.looks_like_command(cmd) {
                        commands.push(cmd.to_string());
                    }
                }
            }
        }

        commands
    }

    // Heuristic to determine if a string looks like a shell command
    #[allow(dead_code)]
    fn looks_like_command(&self, text: &str) -> bool {
        let common_commands = [
            "ls", "dir", "cd", "pwd", "mkdir", "rmdir", "rm", "cp", "mv", "cat", "type", "echo",
            "grep", "find", "touch", "chmod", "chown", "ps", "kill", "top", "df", "du", "tar",
            "zip", "unzip", "curl", "wget", "git", "npm", "pip", "python", "node", "java", "gcc",
            "make", "cargo", "rustc", "dotnet", "go",
        ];

        let first_word = text.split_whitespace().next().unwrap_or("");
        common_commands.contains(&first_word)
            || text.contains("./")
            || text.contains(".exe")
            || text.starts_with('/')
    }

    // Execute shell commands (PowerShell on Windows, bash-like on Unix)
    fn execute_shell_command(&self, command: &str) -> Result<String> {
        let output = if cfg!(target_os = "windows") {
            // On Windows, use PowerShell for better command support
            Command::new("powershell")
                .arg("-Command")
                .arg(command)
                .output()
        } else {
            // On Unix-like systems, use sh
            Command::new("sh").arg("-c").arg(command).output()
        }?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            if !stderr.is_empty() {
                return Err(anyhow::anyhow!("{}", stderr.trim()));
            } else {
                return Err(anyhow::anyhow!(
                    "Command failed with exit code {}",
                    output.status
                ));
            }
        }

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&stderr);
        }

        Ok(result.trim().to_string())
    }
}

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("LLM Terminal Emulator"),
        ..Default::default()
    };

    eframe::run_native(
        "LLM Terminal",
        options,
        Box::new(|_cc| Box::new(LLMTerminalApp::new())),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}

// Remove all the old TUI functions - they're no longer needed
