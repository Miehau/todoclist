use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use crate::todoist::PendingChange;
use crate::todoist::PendingChange::TaskCompletion;

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
                    if let Err(e) = app
                        .api_key_manager
                        .save_api_key("todoist", &app.input_buffer)
                    {
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
            let changes: Vec<PendingChange> = app.pending_changes.iter().cloned().collect();
            app.pending_changes = vec![];
            for change in &changes {
                match change {
                    TaskCompletion { task_id, completed } => {
                        if let Some(client) = &app.todoist_client {
                            let _ = client.update_task_completion(&task_id, *completed).await;
                        }
                    }
                }
            }
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
            if app.today_list_state.selected().is_none() && !app.today_tasks().is_empty() {
                app.today_list_state.select(Some(0));
                let selected_task_id = app.today_tasks().get(0).unwrap().id.clone();
                app.selected_task = app
                    .tasks
                    .iter()
                    .filter(|task| task.id == selected_task_id)
                    .map(|task| task.id.clone())
                    .next()
            }
            app.list_state.select(None);
        }
        KeyCode::Right => {
            // Move to Inbox list
            if app.list_state.selected().is_none() && !app.tasks.is_empty() {
                app.list_state.select(Some(0));
                app.selected_task = app
                    .tasks
                    .iter()
                    .filter(|task| !task.is_completed)
                    .filter(|task| {
                        if let Some(due) = &task.due {
                            let today = chrono::Local::now().date_naive();
                            let task_date =
                                chrono::NaiveDate::parse_from_str(&due.date, "%Y-%m-%d").ok();
                            task_date == Some(today)
                        } else {
                            false
                        }
                    })
                    .map(|task| task.id.clone())
                    .next();
            }
            app.today_list_state.select(None);
        }
        KeyCode::Char(' ') => {
            if let Some(selected_task_id) = &app.selected_task {
                let _ = app.toggle_task_completion(selected_task_id.clone()).await;
            }
        }
        _ => {}
    }
}
