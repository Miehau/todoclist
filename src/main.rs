use crate::todoist::PendingChange::TaskCompletion;
use crate::todoist::{PendingChange, TodoistClient};
use crate::{
    app::{App, AppResult},
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

pub mod app;
pub mod config;
pub mod event;
pub mod handler;
pub mod todoist;
pub mod tui;
pub mod ui;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create an application.
    let application = App::new();
    let app = Arc::new(Mutex::new(application));

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(25);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let async_app = Arc::clone(&app);
    if async_app.lock().await.api_key.is_some() {
        let (tx, mut rx) = mpsc::channel(32);
        let refresh_interval = async_app.lock().await.refresh_interval;
        let key = async_app.lock().await.api_key.clone().unwrap();

        tokio::spawn(async move {
            let client = TodoistClient::new(key); // Remove Arc wrapper
            let mut interval = tokio::time::interval(Duration::from_secs(refresh_interval));

            loop {
                interval.tick().await;
                match client.get_tasks(None).await {
                    Ok(tasks) => {
                        if let Err(_) = tx.send(tasks).await {
                            break;
                        }
                    }
                    Err(e) => eprintln!("Error fetching tasks: {}", e),
                }
            }
        });

        // Spawn the task processing
        let process_app = Arc::clone(&app);
        tokio::spawn(async move {
            while let Some(tasks) = rx.recv().await {
                let mut app = process_app.lock().await;
                app.tasks = tasks;
            }
        });

        let process_app = Arc::clone(&app);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                // First tick completes immediately
                interval.tick().await;
                interval.tick().await;
                let mut ps = process_app.lock().await;
                let changes: Vec<PendingChange> = ps.pending_changes.iter().cloned().collect();
                ps.pending_changes = vec![];
                for change in &changes {
                    match change {
                        TaskCompletion { task_id, completed } => {
                            if let Some(client) = &ps.todoist_client {
                                let _ = client.update_task_completion(&task_id, *completed).await;
                            }
                        }
                    }
                }
            }
        });
    }

    // Start the main loop.
    while app.lock().await.running {
        // Render the user interface.
        tui.draw(&mut *app.lock().await)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => {}
            Event::Key(key_event) => handle_key_events(key_event, &mut *app.lock().await).await,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
