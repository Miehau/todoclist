use std::{error, time::{SystemTime, UNIX_EPOCH}};
use ratatui::widgets::ListState;
use crate::config::ApiKeyManager;
use crate::todoist::{TodoistClient, Task, PendingChange};

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    pub list_state: ListState,
    pub today_list_state: ListState,
    /// Is onboarding complete?
    pub onboarding_complete: bool,
    /// Input buffer for onboarding
    pub input_buffer: String,
    /// Temporary storage for API key
    pub api_key: Option<String>,
    /// API key manager
    pub api_key_manager: ApiKeyManager,
    /// Todoist client
    pub todoist_client: Option<TodoistClient>,
    /// Inbox tasks from Todoist
    pub tasks: Vec<Task>,
    /// Pending changes to sync
    pub pending_changes: Vec<PendingChange>,
    pub refresh_interval: u64,
    last_refresh: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            list_state: ListState::default(),
            today_list_state: ListState::default(),
            onboarding_complete: false,
            input_buffer: String::new(),
            api_key: None,
            api_key_manager: ApiKeyManager::new(),
            todoist_client: None,
            tasks: Vec::new(),
            refresh_interval: 10, // Default to 10 seconds
            last_refresh: 0,
            pending_changes: Vec::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut app = Self::default();
        // Check if we have a saved API key
        if let Ok(key) = app.api_key_manager.load_api_key("todoist") {
            app.api_key = Some(key.clone());
            app.todoist_client = Some(TodoistClient::new(key));
            app.onboarding_complete = true;
            
            // Load refresh interval from config if available
            if let Ok(config) = app.api_key_manager.load_config() {
                app.refresh_interval = config.refresh_interval()
            }
        }
        
        // Set initial selection to Today list if there are tasks
        if !app.today_tasks().is_empty() {
            app.today_list_state.select(Some(0));
        } else if !app.tasks.is_empty() {
            app.list_state.select(Some(0));
        }
        
        app
    }

    pub fn today_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|task| {
            if let Some(due) = &task.due {
                due.date == chrono::Local::now().date_naive().to_string()
            } else {
                false
            }
        }).collect()
    }

    /// Load tasks from Todoist
    pub async fn load_tasks(&mut self) -> AppResult<()> {
        if let Some(client) = &self.todoist_client {
            // Get Inbox tasks (no filter)
            let inbox_tasks = client.get_tasks(None).await?;

            // Store both sets of tasks
            self.tasks = inbox_tasks;

            // Set initial selection to Today list if there are tasks
            if !self.today_tasks().is_empty() {
                self.today_list_state.select(Some(0));
                self.list_state.select(None);
            } else if !self.tasks.is_empty() {
                self.list_state.select(Some(0));
                self.today_list_state.select(None);
            }
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub async fn tick(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        // Check if it's time to refresh
        if now - self.last_refresh >= self.refresh_interval {
            if let Some(client) = &self.todoist_client {
                match tokio::try_join!(
                    client.get_tasks(None),
                    client.get_tasks(Some("today"))
                ) {
                    Ok((inbox_tasks, today_tasks)) => {
                        self.update_tasks(inbox_tasks, today_tasks);
                    }
                    Err(e) => eprintln!("Failed to refresh tasks: {}", e),
                }
            }
            self.last_refresh = now;
        }
    }

    /// Update tasks list from async refresh
    fn update_tasks(&mut self, new_inbox_tasks: Vec<Task>, new_today_tasks: Vec<Task>) {
        // Update Inbox tasks
        let mut existing_inbox = std::collections::HashMap::new();
        for task in &self.tasks {
            existing_inbox.insert(&task.id, task);
        }

        let mut merged_inbox = Vec::new();
        for new_task in new_inbox_tasks {
            if let Some(existing) = existing_inbox.get(&new_task.id) {
                merged_inbox.push(Task {
                    is_completed: existing.is_completed,
                    ..new_task
                });
            } else {
                merged_inbox.push(new_task);
            }
        }
        self.tasks = merged_inbox;

        // Update Today tasks
        let mut existing_today = std::collections::HashMap::new();
        for &mut task in &mut self.today_tasks() {
            existing_today.insert(&task.id, task);
        }

        let mut merged_today = Vec::new();
        for new_task in new_today_tasks {
            if let Some(existing) = existing_today.get(&new_task.id) {
                merged_today.push(Task {
                    is_completed: existing.is_completed,
                    ..new_task
                });
            } else {
                merged_today.push(new_task);
            }
        }
    }

    /// Validate API key format
    pub fn is_valid_api_key(&self) -> bool {
        !self.input_buffer.is_empty()
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }

    pub fn next(&mut self) {
        if self.list_state.selected().is_some() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= self.tasks.len().saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        } else {
            let today_tasks_count = self.tasks.iter()
                .filter(|task| {
                    if let Some(due) = &task.due {
                        let today = chrono::Local::now().date_naive();
                        let task_date = chrono::NaiveDate::parse_from_str(&due.date, "%Y-%m-%d").ok();
                        task_date == Some(today)
                    } else {
                        false
                    }
                })
                .count();
                
            let i = match self.today_list_state.selected() {
                Some(i) => {
                    if i >= today_tasks_count.saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.today_list_state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if self.list_state.selected().is_some() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.tasks.len().saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        } else {
            let today_tasks_count = self.tasks.iter()
                .filter(|task| {
                    if let Some(due) = &task.due {
                        let today = chrono::Local::now().date_naive();
                        let task_date = chrono::NaiveDate::parse_from_str(&due.date, "%Y-%m-%d").ok();
                        task_date == Some(today)
                    } else {
                        false
                    }
                })
                .count();
                
            let i = match self.today_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        today_tasks_count.saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.today_list_state.select(Some(i));
        }
    }
}
