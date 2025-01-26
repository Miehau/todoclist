use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph},
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
            .split(frame.area());

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

    // Create layout
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),  // Today list
            Constraint::Percentage(30),  // Inbox list
        ])
        .split(frame.area());

    // Create Today list
    let today_items: Vec<ListItem> = if app.today_tasks().is_empty() {
        vec![ListItem::new("No tasks for Today")]
    } else {
        app.today_tasks()
            .iter()
            .filter(|task| !task.is_completed)
            .map(|task| {
                let status_symbol = if task.is_completed { "✓" } else { "☐" };
                let content = if let Some(due) = &task.due {
                    format!("{} {} ({})", status_symbol, task.content, due.date)
                } else {
                    format!("{} {}", status_symbol, task.content)
                };
                ListItem::new(content)
            })
            .collect()
    };

    let today_list = List::new(today_items)
        .block(Block::bordered().title("Today"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    // Create Inbox list
    let inbox_items: Vec<ListItem> = if app.tasks.is_empty() {
        vec![ListItem::new("No tasks in Inbox")]
    } else {
        app.tasks
            .iter()
            .filter(|task| !task.is_completed)
            .map(|task| {
                let status_symbol = if task.is_completed { "✓" } else { "☐" };
                let content = if let Some(due) = &task.due {
                    format!("{} {} ({})", status_symbol, task.content, due.date)
                } else {
                    format!("{} {}", status_symbol, task.content)
                };
                ListItem::new(content)
            })
            .collect()
    };

    let inbox_list = List::new(inbox_items)
        .block(Block::bordered().title("Inbox"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    // Render both lists
    frame.render_stateful_widget(today_list, layout[0], &mut app.today_list_state);
    frame.render_stateful_widget(inbox_list, layout[1], &mut app.list_state);
}
