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
                if !app.input_buffer.is_empty() {
                    app.onboarding_complete = true;
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
        _ => {}
    }
}
