use crate::pre_compute::errors::ReplicateStatusCause;
use reqwest::Client;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Serialize)]
pub struct ExitMessage {
    pub cause: ReplicateStatusCause,
}

pub struct WorkerApiClient {
    base_url: String,
    client: Client,
}

impl WorkerApiClient {
    pub fn new(base_url: &str) -> Self {
        WorkerApiClient {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    pub async fn send_exit_cause_for_pre_compute_stage(
        &self,
        authorization: &str,
        chain_task_id: &str,
        exit_message: &ExitMessage,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/compute/pre/{}/exit", self.base_url, chain_task_id);
        self.client
            .post(&url)
            .header("Authorization", authorization)
            .json(exit_message)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

// Singleton manager
pub static WORKER_API: OnceLock<WorkerApiClient> = OnceLock::new();

pub fn get_worker_api_client() -> &'static WorkerApiClient {
    WORKER_API.get_or_init(|| {
        let base_url = std::env::var("WORKER_API_BASE_URL")
            .unwrap_or_else(|_| "http://worker:13100".to_string());
        WorkerApiClient::new(&base_url)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_send_exit_cause_for_pre_compute_stage() {
        let mock_server = MockServer::start().await;

        let chain_task_id = "1234";
        let authorization = "Bearer authorization";
        let exit_message = ExitMessage {
            cause: ReplicateStatusCause::PreComputeFailedUnknownIssue,
        };

        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{}/exit", chain_task_id)))
            .and(header("Authorization", authorization))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = WorkerApiClient::new(&mock_server.uri());
        let result = client
            .send_exit_cause_for_pre_compute_stage(authorization, chain_task_id, &exit_message)
            .await;

        assert!(result.is_ok());
    }
}
