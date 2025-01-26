use serde::Deserialize;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub content: String,
    pub description: String,
    pub is_completed: bool,
    pub labels: Vec<String>,
    pub due: Option<DueDate>,
}

#[derive(Debug)]
pub enum PendingChange {
    TaskCompletion { task_id: String, completed: bool },
}

impl Clone for PendingChange {
    fn clone(&self) -> Self {
        match &self {
            PendingChange::TaskCompletion { task_id, completed } => PendingChange::TaskCompletion {
                task_id: task_id.clone(),
                completed: completed.clone(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DueDate {
    pub string: String,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct TodoistClient {
    api_key: String,
    client: reqwest::Client,
}

unsafe impl Send for TodoistClient {}
unsafe impl Sync for TodoistClient {}

impl TodoistClient {
    pub(crate) async fn update_task_completion(
        &self,
        task_id: &String,
        completed: bool,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let endpoint = format!("https://api.todoist.com/rest/v2/tasks/{}/close", task_id);

        if completed {
            // Close the task
            let response = self
                .client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_body = response.text().await?;
                eprintln!("Failed to close task: {} - {}", status, error_body);
                return Err(format!("API request failed: {}", status).into());
            }
        } else {
            // Reopen the task
            let endpoint = format!("https://api.todoist.com/rest/v2/tasks/{}/reopen", task_id);
            let response = self
                .client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let error_body = response.text().await?;
                eprintln!("Failed to reopen task: {} - {}", status, error_body);
                return Err(format!("API request failed: {}", status).into());
            }
        }

        Ok(())
    }
}

impl TodoistClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn get_tasks(
        &self,
        filter: Option<&str>,
    ) -> Result<Vec<Task>, Box<dyn Error + Send + Sync + 'static>> {
        let mut request = self
            .client
            .get("https://api.todoist.com/rest/v2/tasks")
            .header("Authorization", format!("Bearer {}", self.api_key));

        if let Some(filter) = filter {
            request = request.query(&[("filter", filter)]);
        }

        let response = request.send().await?;

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
