use crate::terminal::emulator::TerminalEmulator;
use crate::terminal::emulator::{TerminalLine, TerminalLineType};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_terminal_interface(
    f: &mut Frame,
    terminal_emulator: &TerminalEmulator,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Terminal tab bar
            Constraint::Min(0),    // Terminal content
            Constraint::Length(3), // Terminal input
        ])
        .split(area);

    render_terminal_tabs(f, terminal_emulator, chunks[0]);
    render_terminal_content(f, terminal_emulator, chunks[1]);
    render_terminal_input(f, terminal_emulator, chunks[2]);
}

fn render_terminal_tabs(
    f: &mut Frame,
    terminal_emulator: &TerminalEmulator,
    area: Rect,
) {
    let tab_titles: Vec<Line> = terminal_emulator
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let style = if i == terminal_emulator.active_session {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            Line::from(vec![Span::styled(session.title.clone(), style)])
        })
        .collect();

    if tab_titles.is_empty() {
        let empty_tabs = ratatui::widgets::Tabs::new(vec![Line::from("No Terminal Sessions")])
            .block(Block::default().borders(Borders::ALL).title("Terminal"))
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty_tabs, area);
    } else {
        let tabs = ratatui::widgets::Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title("Terminal"))
            .select(terminal_emulator.active_session)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));
        f.render_widget(tabs, area);
    }
}

fn render_terminal_content(
    f: &mut Frame,
    terminal_emulator: &TerminalEmulator,
    area: Rect,
) {
    if let Some(session) = terminal_emulator.get_active_session() {
        // Convert terminal history to list items
        let items: Vec<ListItem> = session
            .history
            .iter()
            .map(|line| terminal_line_to_list_item(line))
            .collect();

        let terminal_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Terminal - {}", session.title))
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(terminal_list, area);
    } else {
        // No active session - show placeholder
        let placeholder = Paragraph::new("No active terminal session\n\nPress Ctrl+Shift+T to create a new terminal session")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Terminal")
            )
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true });

        f.render_widget(placeholder, area);
    }
}

fn render_terminal_input(
    f: &mut Frame,
    terminal_emulator: &TerminalEmulator,
    area: Rect,
) {
    if let Some(session) = terminal_emulator.get_active_session() {
        let input_text = if session.current_input.is_empty() {
            "$ "
        } else {
            &format!("$ {}", session.current_input)
        };

        let style = if session.current_input.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Green)
        };

        let input_widget = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command (Press Enter to execute)")
            )
            .style(style)
            .wrap(Wrap { trim: false });

        f.render_widget(input_widget, area);
    } else {
        // No active session
        let placeholder = Paragraph::new("No active terminal session")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command")
            )
            .style(Style::default().fg(Color::DarkGray));

        f.render_widget(placeholder, area);
    }
}

fn terminal_line_to_list_item(line: &TerminalLine) -> ListItem {
    let style = match line.line_type {
        TerminalLineType::Command => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        TerminalLineType::Output => Style::default().fg(Color::White),
        TerminalLineType::Error => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
        TerminalLineType::System => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC),
    };

    // Format timestamp for display (just time, not full date)
    let time_str = line.timestamp.format("%H:%M:%S").to_string();
    
    let content = Line::from(vec![
        Span::styled(format!("[{}] ", time_str), Style::default().fg(Color::DarkGray)),
        Span::styled(&line.content, style),
    ]);

    ListItem::new(content)
}

