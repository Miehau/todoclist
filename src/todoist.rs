use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: String,
    pub content: String,
    pub description: String,
    pub is_completed: bool,
    pub due: Option<DueDate>,
}

#[derive(Debug, Deserialize)]
pub struct DueDate {
    pub string: String,
    pub date: String,
}

pub struct TodoistClient {
    api_key: String,
    client: reqwest::Client,
}

impl TodoistClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_inbox_tasks(&self) -> Result<Vec<Task>, Box<dyn Error>> {
        let response = self.client
            .get("https://api.todoist.com/rest/v2/tasks")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let tasks: Vec<Task> = response.json().await?;
        Ok(tasks)
    }
}
