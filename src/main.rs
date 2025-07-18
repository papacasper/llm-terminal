mod app;
mod code_executor;
mod config;
mod llm;
mod models;
mod ui;

use app::AppState;
use models::{AppMode, MessageRole};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app_state = AppState::new();

    // Create channel for async LLM responses
    let (response_tx, mut response_rx) = mpsc::channel(100);

    // Run the main loop
    let res = run_app(&mut terminal, &mut app_state, response_tx, &mut response_rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app_state: &mut AppState,
    response_tx: mpsc::Sender<Result<String>>,
    response_rx: &mut mpsc::Receiver<Result<String>>,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, app_state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let Err(err) = handle_key_event(app_state, key, response_tx.clone()).await {
                    eprintln!("Error handling key event: {}", err);
                }
            }
        }

        // Handle LLM responses
        if let Ok(response) = response_rx.try_recv() {
            if let Err(err) = app_state.handle_llm_response(response).await {
                eprintln!("Error handling LLM response: {}", err);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app_state.app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_key_event(
    app_state: &mut AppState,
    key: KeyEvent,
    response_tx: mpsc::Sender<Result<String>>,
) -> Result<()> {
    // Handle Enter key specially for sending messages
    if key.code == KeyCode::Enter && matches!(app_state.app.mode, AppMode::Chat) {
        if !app_state.app.input_buffer.trim().is_empty() {
            let message = app_state.app.input_buffer.clone();
            app_state.app.input_buffer.clear();
            
            // Add user message to current tab and get provider info
            let (provider, model, messages) = {
                if let Some(current_tab) = app_state.app.current_tab_mut() {
                    current_tab.add_message(models::Message::user(message.clone()));
                    current_tab.set_waiting(true);
                    
                    (current_tab.provider.clone(), current_tab.model.clone(), current_tab.messages.clone())
                } else {
                    return Ok(());
                }
            };
            
            // Find client for the current tab's provider
            if let Ok(client) = app_state.find_client_for_provider(&provider) {
                let client_clone = client.clone();
                let response_tx_clone = response_tx.clone();
                
                // Send message asynchronously
                tokio::spawn(async move {
                    let result = client_clone.send_message(&messages, &model).await;
                    let _ = response_tx_clone.send(result).await;
                });
            } else {
                // No client available, add error message
                let error_msg = format!("No API key configured for {}", provider.as_str());
                if let Some(current_tab) = app_state.app.current_tab_mut() {
                    current_tab.add_message(models::Message::assistant(error_msg));
                    current_tab.set_waiting(false);
                }
            }
        }
    } else {
        // Handle other key events
        app_state.handle_key_event(key)?;
    }

    Ok(())
}

fn ui(f: &mut Frame, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Content area
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    match app_state.app.mode {
        AppMode::Chat => {
            render_tabs(f, &app_state.app, chunks[0]);
            render_chat_area(f, &app_state.app, chunks[1]);
        }
        AppMode::Settings => {
            render_tabs(f, &app_state.app, chunks[0]);
            ui::render_settings(f, &app_state.app, chunks[1]);
        }
    }

    render_status_bar(f, &app_state.app, chunks[2]);
}

fn render_tabs(f: &mut Frame, app: &models::App, area: Rect) {
    let tab_titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let title = if tab.is_waiting {
                format!("{} ⏳", tab.title)
            } else {
                tab.title.clone()
            };
            
            let style = if i == app.current_tab {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            
            Line::from(vec![Span::styled(title, style)])
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));

    f.render_widget(tabs, area);
}

fn render_chat_area(f: &mut Frame, app: &models::App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Messages area
            Constraint::Length(3), // Input area
        ])
        .split(area);

    render_messages(f, app, chunks[0]);
    render_input_area(f, app, chunks[1]);
}

fn render_messages(f: &mut Frame, app: &models::App, area: Rect) {
    if let Some(current_tab) = app.current_tab() {
        let messages: Vec<Line> = current_tab
            .messages
            .iter()
            .flat_map(|msg| {
                let role_style = match msg.role {
                    MessageRole::User => Style::default().fg(Color::Cyan),
                    MessageRole::Assistant => Style::default().fg(Color::Green),
                };
                
                let role_name = match msg.role {
                    MessageRole::User => "You",
                    MessageRole::Assistant => current_tab.provider.as_str(),
                };
                
                // Split long messages into multiple lines
                let content_lines: Vec<String> = msg.content
                    .lines()
                    .flat_map(|line| {
                        if line.len() <= 80 {
                            vec![line.to_string()]
                        } else {
                            line.chars()
                                .collect::<Vec<_>>()
                                .chunks(80)
                                .map(|chunk| chunk.iter().collect::<String>())
                                .collect()
                        }
                    })
                    .collect();

                let mut result = Vec::new();
                
                // Add role header
                result.push(Line::from(vec![
                    Span::styled(format!("{}:", role_name), role_style),
                ]));
                
                // Add content lines
                for line in content_lines {
                    result.push(Line::from(vec![
                        Span::raw(format!("  {}", line)),
                    ]));
                }
                
                // Add empty line for spacing
                result.push(Line::from(vec![Span::raw("")]));
                
                result
            })
            .collect();

        let messages_widget = Paragraph::new(messages)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Chat - {}", current_tab.provider.as_str()))
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        f.render_widget(messages_widget, area);
    }
}

fn render_input_area(f: &mut Frame, app: &models::App, area: Rect) {
    let input_text = if app.input_buffer.is_empty() {
        "Type your message here..."
    } else {
        &app.input_buffer
    };

    let style = if app.input_buffer.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let input_widget = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input (Press Enter to send)")
        )
        .style(style)
        .wrap(Wrap { trim: true });

    f.render_widget(input_widget, area);
}

fn render_status_bar(f: &mut Frame, app: &models::App, area: Rect) {
    let current_tab = app.current_tab();
    let status_text = if let Some(tab) = current_tab {
        // Extract model name for better display
        let model_display = if tab.model.contains("claude-3-5-sonnet") {
            "Claude 3.5 Sonnet"
        } else if tab.model.contains("claude-3-opus") {
            "Claude 3 Opus"
        } else if tab.model.contains("claude-3-sonnet") {
            "Claude 3 Sonnet"
        } else if tab.model.contains("claude-3-haiku") {
            "Claude 3 Haiku"
        } else if tab.model.contains("gpt-4o-mini") {
            "GPT-4o Mini"
        } else if tab.model.contains("gpt-4o") {
            "GPT-4o"
        } else if tab.model.contains("gpt-4-turbo") {
            "GPT-4 Turbo"
        } else if tab.model.contains("gpt-3.5-turbo") {
            "GPT-3.5 Turbo"
        } else {
            &tab.model
        };
        
        let code_exec_status = if tab.code_execution_enabled { "🔧" } else { "🚫" };
        
        if tab.is_waiting {
            format!("⏳ Waiting for response... | {} {} | Tab: {} | Ctrl+, settings", 
                   model_display, code_exec_status, app.current_tab + 1)
        } else {
            format!("✅ Ready | {} {} | Tab: {} | Ctrl+, settings", 
                   model_display, code_exec_status, app.current_tab + 1)
        }
    } else {
        "No active tab".to_string()
    };

    let status_style = if current_tab.map(|t| t.is_waiting).unwrap_or(false) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let status_widget = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status_widget, area);
}

