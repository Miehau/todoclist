use crossterm::event::{KeyCode, KeyEvent};
use crate::app::App;

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
    if !app.onboarding_complete {
        match key_event.code {
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Enter => {
                if app.is_valid_api_key() {
                    if let Err(e) = app.api_key_manager.save_api_key("todoist", &app.input_buffer) {
                        eprintln!("Failed to save API key: {}", e);
                    } else {
                        app.api_key = Some(app.input_buffer.clone());
                        app.onboarding_complete = true;
                    }
                }
            }
            _ => {}
        }
        return;
    }

    match key_event.code {
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Up => {
            app.previous();
        }
        KeyCode::Down => {
            app.next();
        }
        KeyCode::Char('r') => {
            // Example: Press 'r' then a number to set refresh interval
            if let Some(interval) = key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                if let Ok(interval) = key_event.code.to_string().parse::<u64>() {
                    if let Err(e) = app.api_key_manager.save_refresh_interval(interval) {
                        eprintln!("Failed to save refresh interval: {}", e);
                    } else {
                        app.refresh_interval = interval;
                    }
                }
            }
        }
        _ => {}
    }
}
