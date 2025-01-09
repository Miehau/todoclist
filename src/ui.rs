use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    if !app.onboarding_complete {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(frame.size());

        // Title
        frame.render_widget(
            Paragraph::new("Welcome! Please enter your name:")
                .block(
                    Block::bordered()
                        .title(" Onboarding ")
                        .title_alignment(Alignment::Center)
                        .style(Style::default().fg(Color::LightBlue))
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White)),
            layout[0],
        );

        // Input field
        let input = Paragraph::new(app.input_buffer.as_str())
            .block(
                Block::bordered()
                    .title(" Your Name ")
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::LightYellow))
            )
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        frame.render_widget(input, layout[1]);

        // Instructions
        frame.render_widget(
            Paragraph::new("Press Enter to continue")
                .block(
                    Block::bordered()
                        .style(Style::default().fg(Color::LightMagenta))
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White)),
            layout[2],
        );
        return;
    }

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
