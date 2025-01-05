use crossterm::event::{KeyCode, KeyEvent};
use crate::app::App;

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) {
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
