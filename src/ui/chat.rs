use crate::models::{App, MessageRole};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_chat(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [Constraint::Percentage(90), Constraint::Percentage(10)].as_ref(),
        )
        .split(f.area());

    let chat_view = render_chat_view(app);
    let input_view = render_input_view(app);

    f.render_widget(chat_view, chunks[0]);
    f.render_widget(input_view, chunks[1]);
}

fn render_chat_view(app: &App) -> Paragraph<'_> {
    let current_tab = &app.tabs[app.current_tab];

    let messages: Vec<Line> = current_tab
        .messages
        .iter()
        .map(|msg| {
            let prefix = match msg.role {
                MessageRole::User => "You",
                MessageRole::Assistant => "Assistant",
            };
            Line::from(vec![Span::raw(format!("{}: {}", prefix, msg.content))])
        })
        .collect();

    Paragraph::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Chat"))
        .style(Style::default().fg(Color::White))
}

fn render_input_view(app: &App) -> Paragraph<'_> {
    Paragraph::new(app.input_buffer.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .style(Style::default().fg(Color::Yellow))
}
