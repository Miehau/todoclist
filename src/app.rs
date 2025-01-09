use std::error;
use ratatui::widgets::ListState;
use crate::config::ApiKeyManager;
use crate::todoist::{TodoistClient, Task};

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    pub list_state: ListState,
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
    /// Tasks from Todoist
    pub tasks: Vec<Task>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            list_state: ListState::default(),
            onboarding_complete: false,
            input_buffer: String::new(),
            api_key: None,
            api_key_manager: ApiKeyManager::new(),
            todoist_client: None,
            tasks: Vec::new(),
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
        }
        app
    }

    /// Load tasks from Todoist
    pub async fn load_tasks(&mut self) -> AppResult<()> {
        if let Some(client) = &self.todoist_client {
            self.tasks = client.get_inbox_tasks().await?;
            if self.tasks.is_empty() {
                println!("Warning: No tasks found - check your API key and Todoist account");
            }
        } else {
            println!("Warning: No Todoist client available");
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

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
    }

    pub fn previous(&mut self) {
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
    }
}
