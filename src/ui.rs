use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    if !app.onboarding_complete && app.api_key.is_none() {
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
            Paragraph::new("Welcome! Please enter your API key:")
                .block(
                    Block::bordered()
                        .title(" API Key Setup ")
                        .title_alignment(Alignment::Center)
                        .style(Style::default().fg(Color::LightBlue))
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White)),
            layout[0],
        );

        // Input field with placeholder
        let display_text = if app.input_buffer.is_empty() {
            "7x9y2z8w4v5q1r3t6u0o2jhbjhb2jh12nvc1h2".to_string()
        } else {
            app.input_buffer.as_str().to_string()
        };
        
        let input = Paragraph::new(display_text)
            .block(
                Block::bordered()
                    .title(" API Key ")
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(
                        if app.input_buffer.is_empty() {
                            Color::DarkGray
                        } else if app.is_valid_api_key() {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        }
                    ))
            )
            .style(Style::default().fg(
                if app.input_buffer.is_empty() {
                    Color::DarkGray
                } else {
                    Color::White
                }
            ))
            .alignment(Alignment::Center);
        frame.render_widget(input, layout[1]);

        // Instructions
        let instructions = if app.input_buffer.is_empty() {
            "Enter your API key"
        } else if !app.is_valid_api_key() {
            "API key cannot be empty"
        } else {
            "Press Enter to continue"
        };
        
        frame.render_widget(
            Paragraph::new(instructions)
                .block(
                    Block::bordered()
                        .style(Style::default().fg(
                            if app.input_buffer.is_empty() {
                                Color::LightMagenta
                            } else if !app.is_valid_api_key() {
                                Color::Red
                            } else {
                                Color::LightGreen
                            }
                        ))
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White)),
            layout[2],
        );
        return;
    }

    // If no tasks, show a placeholder
    let items: Vec<ListItem> = if app.tasks.is_empty() {
        vec![ListItem::new("No tasks found")]
    } else {
        app.tasks
            .iter()
            .map(|task| {
                let content = if task.is_completed {
                    format!("✓ {}", task.content)
                } else {
                    format!("☐ {}", task.content)
                };
                ListItem::new(content)
            })
            .collect()
    };

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
