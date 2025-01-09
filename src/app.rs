use std::error;
use ratatui::widgets::ListState;
use crate::config::ApiKeyManager;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

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
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
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
                if i >= 4 {
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
                    4
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
