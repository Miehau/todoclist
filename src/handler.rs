use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
        // Insert empty block for whitespace AI!
        _ => {}
    }
}
