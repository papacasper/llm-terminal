#![allow(dead_code)]
use super::process::ProcessManager;
use anyhow::Result;
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub line_type: TerminalLineType,
}

#[derive(Debug, Clone)]
pub enum TerminalLineType {
    Output,
    Error,
    System,
}

impl TerminalLine {
    pub fn output(content: String) -> Self {
        Self {
            content,
            timestamp: chrono::Utc::now(),
            line_type: TerminalLineType::Output,
        }
    }

    pub fn error(content: String) -> Self {
        Self {
            content,
            timestamp: chrono::Utc::now(),
            line_type: TerminalLineType::Error,
        }
    }

    pub fn system(content: String) -> Self {
        Self {
            content,
            timestamp: chrono::Utc::now(),
            line_type: TerminalLineType::System,
        }
    }
}

pub struct TerminalSession {
    pub id: Uuid,
    pub title: String,
    pub history: VecDeque<TerminalLine>,
    pub current_input: String,
    pub working_directory: std::path::PathBuf,
    pub is_active: bool,
    max_history: usize,
}

impl TerminalSession {
    pub fn new(id: Uuid, title: String) -> Self {
        Self {
            id,
            title,
            history: VecDeque::new(),
            current_input: String::new(),
            working_directory: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from(".")),
            is_active: false,
            max_history: 1000, // Keep last 1000 lines
        }
    }

    pub fn add_line(&mut self, line: TerminalLine) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(line);
    }

    pub fn add_command(&mut self, command: String) {
        self.add_line(TerminalLine::system(format!("$ {}", command)));
    }

    pub fn add_output(&mut self, output: String) {
        // Split multi-line output into separate lines
        for line in output.lines() {
            self.add_line(TerminalLine::output(line.to_string()));
        }
    }

    pub fn add_error(&mut self, error: String) {
        for line in error.lines() {
            self.add_line(TerminalLine::error(line.to_string()));
        }
    }

    pub fn add_system_message(&mut self, message: String) {
        self.add_line(TerminalLine::system(message));
    }

    pub fn get_recent_lines(&self, count: usize) -> Vec<&TerminalLine> {
        self.history
            .iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

pub struct TerminalEmulator {
    pub process_manager: ProcessManager,
    pub sessions: Vec<TerminalSession>,
    pub active_session: usize,
}

impl TerminalEmulator {
    pub fn new() -> Self {
        let mut emulator = Self {
            process_manager: ProcessManager::new(),
            sessions: Vec::new(),
            active_session: 0,
        };

        // Create a default terminal session
        if let Err(e) = emulator.create_session() {
            eprintln!("Failed to create default terminal session: {}", e);
        }

        emulator
    }

    pub fn create_session(&mut self) -> Result<Uuid> {
        let terminal_id = self.process_manager.create_terminal()?;

        let session_title = format!("Terminal {}", self.sessions.len() + 1);
        let mut session = TerminalSession::new(terminal_id, session_title);

        // Add welcome message
        session.add_system_message("Terminal session started".to_string());
        session.add_system_message(format!(
            "Working directory: {}",
            session.working_directory.display()
        ));

        self.sessions.push(session);

        // If this is the first session, make it active
        if self.sessions.len() == 1 {
            self.active_session = 0;
            self.sessions[0].is_active = true;
        }

        Ok(terminal_id)
    }

    pub fn get_active_session(&self) -> Option<&TerminalSession> {
        self.sessions.get(self.active_session)
    }

    pub fn get_active_session_mut(&mut self) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(self.active_session)
    }

    pub fn set_active_session(&mut self, index: usize) -> Result<()> {
        if index < self.sessions.len() {
            // Deactivate current session
            if let Some(current) = self.sessions.get_mut(self.active_session) {
                current.is_active = false;
            }

            // Activate new session
            self.active_session = index;
            if let Some(new_active) = self.sessions.get_mut(self.active_session) {
                new_active.is_active = true;
                self.process_manager.set_active_terminal(new_active.id)?;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Session index {} out of bounds", index))
        }
    }

    pub fn close_session(&mut self, index: usize) -> Result<()> {
        if index < self.sessions.len() {
            let session = self.sessions.remove(index);
            self.process_manager.remove_terminal(&session.id);

            // Adjust active session index
            if index <= self.active_session && self.active_session > 0 {
                self.active_session -= 1;
            } else if self.sessions.is_empty() {
                self.active_session = 0;
            } else if self.active_session >= self.sessions.len() {
                self.active_session = self.sessions.len() - 1;
            }

            // Update active session
            if let Some(active) = self.sessions.get_mut(self.active_session) {
                active.is_active = true;
                self.process_manager.set_active_terminal(active.id)?;
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Session index {} out of bounds", index))
        }
    }

    pub async fn execute_command(&mut self, command: &str) -> Result<()> {
        if let Some(session) = self.get_active_session_mut() {
            // Add command to history
            session.add_command(command.to_string());

            // Clear current input
            session.current_input.clear();

            // Execute command
            self.process_manager.send_input_to_active(command).await?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("No active terminal session"))
        }
    }

    pub async fn update(&mut self) {
        // Read output from active terminal
        while let Some(output) = self.process_manager.read_output_from_active().await {
            if let Some(session) = self.get_active_session_mut() {
                if let Some(stripped) = output.strip_prefix("ERROR:") {
                    session.add_error(stripped.trim().to_string());
                } else {
                    session.add_output(output.trim().to_string());
                }
            }
        }

        // Cleanup dead terminals
        self.process_manager.cleanup_dead_terminals();
    }

    pub fn input_char(&mut self, c: char) {
        if let Some(session) = self.get_active_session_mut() {
            session.current_input.push(c);
        }
    }

    pub fn input_backspace(&mut self) {
        if let Some(session) = self.get_active_session_mut() {
            session.current_input.pop();
        }
    }

    pub async fn input_enter(&mut self) -> Result<()> {
        if let Some(session) = self.get_active_session() {
            let command = session.current_input.clone();
            if !command.trim().is_empty() {
                self.execute_command(&command).await?;
            }
        }
        Ok(())
    }

    pub fn get_session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn get_session_titles(&self) -> Vec<String> {
        self.sessions.iter().map(|s| s.title.clone()).collect()
    }

    pub fn next_session(&mut self) -> Result<()> {
        if !self.sessions.is_empty() {
            let next_index = (self.active_session + 1) % self.sessions.len();
            self.set_active_session(next_index)
        } else {
            Ok(())
        }
    }

    pub fn previous_session(&mut self) -> Result<()> {
        if !self.sessions.is_empty() {
            let prev_index = if self.active_session == 0 {
                self.sessions.len() - 1
            } else {
                self.active_session - 1
            };
            self.set_active_session(prev_index)
        } else {
            Ok(())
        }
    }
}

impl Default for TerminalEmulator {
    fn default() -> Self {
        Self::new()
    }
}
