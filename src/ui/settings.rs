use crate::models::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Settings")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Claude API Key
    let claude_key_status = if app.settings.claude_api_key.is_some() {
        "✓ Configured"
    } else {
        "✗ Not configured"
    };
    let claude_color = if app.settings.claude_api_key.is_some() {
        Color::Green
    } else {
        Color::Red
    };
    
    let claude_info = Paragraph::new(format!("Claude API Key: {}", claude_key_status))
        .style(Style::default().fg(claude_color))
        .block(Block::default().borders(Borders::ALL).title("Anthropic Claude"));
    f.render_widget(claude_info, chunks[1]);

    // OpenAI API Key
    let openai_key_status = if app.settings.openai_api_key.is_some() {
        "✓ Configured"
    } else {
        "✗ Not configured"
    };
    let openai_color = if app.settings.openai_api_key.is_some() {
        Color::Green
    } else {
        Color::Red
    };
    
    let openai_info = Paragraph::new(format!("OpenAI API Key: {}", openai_key_status))
        .style(Style::default().fg(openai_color))
        .block(Block::default().borders(Borders::ALL).title("OpenAI"));
    f.render_widget(openai_info, chunks[2]);

    // Default Provider
    let default_provider = Paragraph::new(format!("Default Provider: {}", app.settings.default_provider.as_str()))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Default Provider"));
    f.render_widget(default_provider, chunks[3]);

    // Instructions
    let instructions = vec![
        "API Keys are loaded from environment variables:",
        "  • ANTHROPIC_API_KEY or CLAUDE_API_KEY for Claude",
        "  • OPENAI_API_KEY for OpenAI",
        "",
        "Keyboard shortcuts:",
        "  • Ctrl+, : Toggle settings panel",
        "  • Ctrl+T : New tab",
        "  • Ctrl+W : Close tab",
        "  • Tab/Shift+Tab : Switch tabs",
        "  • Ctrl+Q : Quit",
    ];

    let instructions_text: Vec<Line> = instructions
        .iter()
        .map(|line| Line::from(vec![Span::raw(*line)]))
        .collect();

    let instructions_paragraph = Paragraph::new(instructions_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(instructions_paragraph, chunks[4]);
}
