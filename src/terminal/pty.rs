use anyhow::{anyhow, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct PseudoTerminal {
    child: Option<TokioChild>,
    output_receiver: mpsc::Receiver<String>,
    input_sender: mpsc::Sender<String>,
}

impl PseudoTerminal {
    pub fn new() -> Result<Self> {
        let working_directory = std::env::current_dir()?;
        let shell_command = Self::get_default_shell();
        
        let (output_sender, output_receiver) = mpsc::channel::<String>(1000);
        let (input_sender, mut input_receiver) = mpsc::channel::<String>(100);
        
        // Start the shell process
        let mut cmd = TokioCommand::new(&shell_command);
        cmd.current_dir(&working_directory)
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .kill_on_drop(true);
        
        // On Windows, we need to set up the environment properly
        #[cfg(windows)]
        {
            cmd.env("TERM", "xterm-256color");
        }
        
        let mut child = cmd.spawn()?;
        
        // Get handles to stdin, stdout, and stderr
        let mut stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to get stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to get stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow!("Failed to get stderr"))?;
        
        // Spawn task to handle input
        tokio::spawn(async move {
            while let Some(input) = input_receiver.recv().await {
                if let Err(e) = stdin.write_all(input.as_bytes()).await {
                    eprintln!("Failed to write to stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    eprintln!("Failed to flush stdin: {}", e);
                    break;
                }
            }
        });
        
        // Spawn task to handle stdout
        let output_sender_stdout = output_sender.clone();
        tokio::spawn(async move {
            let mut reader = TokioBufReader::new(stdout);
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if let Err(_) = output_sender_stdout.send(line.clone()).await {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
        });
        
        // Spawn task to handle stderr
        tokio::spawn(async move {
            let mut reader = TokioBufReader::new(stderr);
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        if let Err(_) = output_sender.send(format!("ERROR: {}", line)).await {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading stderr: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(Self {
            child: Some(child),
            output_receiver,
            input_sender,
        })
    }
    
    #[cfg(windows)]
    fn get_default_shell() -> String {
        // Check if PowerShell Core (pwsh) is available, otherwise use PowerShell 5.1
        if which::which("pwsh").is_ok() {
            "pwsh".to_string()
        } else {
            "powershell".to_string()
        }
    }
    
    #[cfg(not(windows))]
    fn get_default_shell() -> String {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
    
    pub async fn send_input(&self, input: &str) -> Result<()> {
        let input_with_newline = if input.ends_with('\n') {
            input.to_string()
        } else {
            format!("{}\n", input)
        };
        
        self.input_sender.send(input_with_newline).await
            .map_err(|_| anyhow!("Failed to send input to terminal"))?;
        Ok(())
    }
    
    pub async fn read_output(&mut self) -> Option<String> {
        self.output_receiver.recv().await
    }
    
    
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,     // Process is still running
                Err(_) => false,      // Error checking status, assume not running
            }
        } else {
            false
        }
    }
}

impl Drop for PseudoTerminal {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Try to kill the child process gracefully
            let _ = child.start_kill();
        }
    }
}
