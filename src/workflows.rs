use crate::terminal::emulator::TerminalEmulator;
use anyhow::Result;

pub struct Workflow {
    pub name: String,
    pub steps: Vec<String>,
}

impl Workflow {
    pub fn new(name: impl Into<String>, steps: Vec<String>) -> Self {
        Self {
            name: name.into(),
            steps,
        }
    }

    pub async fn run(&self, terminal: &mut TerminalEmulator) -> Result<()> {
        for step in &self.steps {
            terminal.execute_command(step).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::emulator::TerminalEmulator;

    #[tokio::test]
    async fn test_workflow_run_empty() {
        let mut term = TerminalEmulator::new();
        let wf = Workflow::new("test", vec![]);
        assert!(wf.run(&mut term).await.is_ok());
    }
}
