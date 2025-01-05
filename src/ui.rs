use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let items = vec![
        "First Item",
        "Second Item",
        "Third Item",
        "Fourth Item",
        "Fifth Item",
    ]
    .iter()
    .map(|&i| ListItem::new(i))
    .collect::<Vec<ListItem>>();

    let list = List::new(items)
        .block(Block::bordered().title("Navigation List"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // For instructions
            Constraint::Min(3),     // For list
        ])
        .split(frame.size());

    // Render instructions
    frame.render_widget(
        Paragraph::new("Use ↑/↓ to navigate\nPress q to quit")
            .block(Block::bordered())
            .alignment(Alignment::Center),
        layout[0],
    );

    // Render list with state
    frame.render_stateful_widget(list, layout[1], &mut app.list_state);
}
