use crate::models::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_input(f: &mut Frame, app: &App, area: Rect) {
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

    let input_paragraph = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input (Press Enter to send)")
        )
        .style(style)
        .wrap(Wrap { trim: true });

    f.render_widget(input_paragraph, area);
}

pub fn render_input_status(f: &mut Frame, app: &App, area: Rect) {
    let current_tab = app.current_tab().unwrap();
    let provider_name = current_tab.provider.as_str();
    
    let status_text = if current_tab.is_waiting {
        format!("Waiting for {} response...", provider_name)
    } else {
        format!("Ready | Provider: {}", provider_name)
    };

    let status_style = if current_tab.is_waiting {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let status_paragraph = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(status_paragraph, area);
}
