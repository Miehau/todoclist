use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) {
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
        KeyCode::Left => {
            // Move to Today list
            if app.today_list_state.selected().is_none() && !app.today_tasks.is_empty() {
                app.today_list_state.select(Some(0));
            }
            app.list_state.select(None);
        }
        KeyCode::Right => {
            // Move to Inbox list
            if app.list_state.selected().is_none() && !app.tasks.is_empty() {
                app.list_state.select(Some(0));
            }
            app.today_list_state.select(None);
        }
        KeyCode::Char(' ') => {
            // correct the syntax AI?
            let client = app.todoist_client.as_ref().unwrap();
            if let Some(task) = app.list_state.selected() {
                let task = app.tasks.get_mut(task).unwrap();
                task.is_completed = !task.is_completed;
            }
        }
        _ => {}
    }
}
