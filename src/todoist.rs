use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub content: String,
    pub description: String,
    pub is_completed: bool,
    pub due: Option<DueDate>,
}

#[derive(Debug)]
pub enum PendingChange {
    TaskCompletion { task_id: String, completed: bool },
}

#[derive(Debug, Deserialize)]
pub struct DueDate {
    pub string: String,
    pub date: String,
}

#[derive(Debug, Clone)]
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

        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            eprintln!("API request failed with status {}: {}", status, error_body);
            return Err(format!("API request failed: {}", status).into());
        }

        // Print raw response for debugging
        let raw_json = response.text().await?;

        // Try to parse the JSON
        let tasks: Vec<Task> = serde_json::from_str(&raw_json)?;
        Ok(tasks)
    }
}
