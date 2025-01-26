use crate::config::ApiKeyManager;
use crate::todoist::PendingChange::TaskCompletion;
use crate::todoist::{PendingChange, Task, TodoistClient};
use ratatui::widgets::ListState;
use std::error;

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum AppError {
    TaskNotFound(String),
    // Add other error variants as needed
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::TaskNotFound(id) => write!(f, "Task with id {} not found", id),
        }
    }
}

impl error::Error for AppError {}

pub struct AppState {
    pub today_tasks: Vec<Task>,
    pub inbox_tasks: Vec<Task>,
    pub pending_tasks: Vec<PendingChange>,
}

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
    pub app_state: AppState,
    pub selected_task: Option<String>,
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
            pending_changes: Vec::new(),
            app_state: AppState {
                today_tasks: Vec::new(),
                inbox_tasks: Vec::new(),
                pending_tasks: Vec::new(),
            },
            selected_task: None,
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

        app
    }

    pub fn today_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| {
                if let Some(due) = &task.due {
                    due.date == chrono::Local::now().date_naive().to_string()
                } else {
                    false
                }
            })
            .collect()
    }

    /// Handles the tick event of the terminal.
    pub async fn tick(&mut self) {}

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
            self.selected_task = Some(self.tasks[i].id.clone());
        } else {
            let today_tasks_count = self
                .app_state
                .today_tasks
                .iter()
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
            let selected_task_id = self.today_tasks().get(i).unwrap().id.clone();
            self.selected_task = self
                .tasks
                .iter()
                .filter(|task| task.id == selected_task_id)
                .map(|task| task.id.clone())
                .next();
        }
    }

    pub async fn toggle_task_completion(&mut self, task_id: String) -> AppResult<()> {
        let task_pos = self.tasks.iter().position(|task| task.id == task_id);
        if let Some(task) = &mut self
            .tasks
            .iter_mut()
            .filter(|task| task.id == *task_id)
            .next()
        {
            task.is_completed = !task.is_completed;
            self.pending_changes.push(TaskCompletion {
                task_id: task.id.clone(),
                completed: task.is_completed,
            });
            self.tasks.remove(task_pos.unwrap());
            if task_pos.unwrap() < self.tasks.len() {
                self.selected_task = self.tasks.get(task_pos.unwrap()).map(|task| task.id.clone());
            } else {
                self.selected_task = self.tasks.get(0).map(|task| task.id.clone());
            }
            Ok(())
        } else {
            Err(AppError::TaskNotFound(task_id.clone()))?
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
            self.selected_task = Some(self.tasks[i].id.clone());
        } else {
            let today_tasks_count = self
                .tasks
                .iter()
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
            let selected_task_id = self.today_tasks().get(i).unwrap().id.clone();
            self.selected_task = self
                .tasks
                .iter()
                .filter(|task| task.id == selected_task_id)
                .map(|task| task.id.clone())
                .next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toggle_task_completion() {
        let mut app = App {
            tasks: vec![Task {
                id: "1".to_string(),
                content: "Test task".to_string(),
                description: "".to_string(),
                is_completed: false,
                labels: vec![],
                due: None,
            }],
            ..Default::default()
        };

        // Initial state
        assert!(!app.tasks[0].is_completed);

        // First toggle
        // How to run this blocking, so I can use .await. Test cannot be async AI?
        app.toggle_task_completion("1".to_string()).await.unwrap();
        assert!(app.tasks[0].is_completed);

        // Test error case
        assert!(app.toggle_task_completion("999".to_string()).await.is_err());
    }
}
