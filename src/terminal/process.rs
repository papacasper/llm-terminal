#![allow(dead_code)]
use super::pty::PseudoTerminal;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ProcessManager {
    terminals: HashMap<Uuid, PseudoTerminal>,
    active_terminal: Option<Uuid>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            terminals: HashMap::new(),
            active_terminal: None,
        }
    }

    pub fn create_terminal(&mut self) -> Result<Uuid> {
        let terminal_id = Uuid::new_v4();
        let pty = PseudoTerminal::new()?;

        self.terminals.insert(terminal_id, pty);

        // If this is the first terminal, make it active
        if self.active_terminal.is_none() {
            self.active_terminal = Some(terminal_id);
        }

        Ok(terminal_id)
    }

    pub fn get_terminal(&self, id: &Uuid) -> Option<&PseudoTerminal> {
        self.terminals.get(id)
    }

    pub fn get_terminal_mut(&mut self, id: &Uuid) -> Option<&mut PseudoTerminal> {
        self.terminals.get_mut(id)
    }

    pub fn get_active_terminal(&self) -> Option<&PseudoTerminal> {
        self.active_terminal
            .as_ref()
            .and_then(|id| self.terminals.get(id))
    }

    pub fn get_active_terminal_mut(&mut self) -> Option<&mut PseudoTerminal> {
        self.active_terminal
            .as_ref()
            .and_then(|id| self.terminals.get_mut(id))
    }

    pub fn set_active_terminal(&mut self, id: Uuid) -> Result<()> {
        if self.terminals.contains_key(&id) {
            self.active_terminal = Some(id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Terminal with id {} not found", id))
        }
    }

    pub fn remove_terminal(&mut self, id: &Uuid) -> Option<PseudoTerminal> {
        // If we're removing the active terminal, switch to another one
        if self.active_terminal == Some(*id) {
            self.active_terminal = self.terminals.keys().find(|&k| k != id).cloned();
        }

        self.terminals.remove(id)
    }

    pub fn get_terminal_ids(&self) -> Vec<Uuid> {
        self.terminals.keys().cloned().collect()
    }

    pub fn get_active_terminal_id(&self) -> Option<Uuid> {
        self.active_terminal
    }

    pub fn terminal_count(&self) -> usize {
        self.terminals.len()
    }

    pub async fn send_input_to_active(&self, input: &str) -> Result<()> {
        if let Some(terminal) = self.get_active_terminal() {
            terminal.send_input(input).await
        } else {
            Err(anyhow::anyhow!("No active terminal"))
        }
    }

    pub async fn read_output_from_active(&mut self) -> Option<String> {
        if let Some(terminal) = self.get_active_terminal_mut() {
            terminal.read_output().await
        } else {
            None
        }
    }

    pub fn cleanup_dead_terminals(&mut self) {
        let mut dead_terminals = Vec::new();

        for (id, terminal) in self.terminals.iter_mut() {
            if !terminal.is_running() {
                dead_terminals.push(*id);
            }
        }

        for id in dead_terminals {
            self.remove_terminal(&id);
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
