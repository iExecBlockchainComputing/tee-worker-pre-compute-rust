use crate::pre_compute::errors::ReplicateStatusCause;
use crate::utils::env_utils::{get_env_var_or_error, TeeSessionEnvironmentVariable};
use reqwest::header::AUTHORIZATION;
use reqwest::{blocking::Client, Error};
use serde::Serialize;

/// Represents payload that can be sent to the worker API to report the outcome of the
/// pre‑compute stage.
///
/// The JSON structure expected by the REST endpoint is:
/// ```json
/// {
///   "cause": "<ReplicateStatusCause as string>"
/// }
/// ```
///
/// # Arguments
///
/// * `cause` - A reference to the ReplicateStatusCause indicating why the pre-compute operation exited
///
/// # Example
///
/// ```
/// use crate::pre_compute::worker_api::ExitMessage;
/// use crate::pre_compute::errors::ReplicateStatusCause;
///
/// let exit_message = ExitMessage::from(&ReplicateStatusCause::PreComputeInvalidTeeSignature);
/// ```
#[derive(Serialize, Debug)]
pub struct ExitMessage<'a> {
    #[serde(rename = "cause")]
    pub cause: &'a ReplicateStatusCause,
}

impl<'a> From<&'a ReplicateStatusCause> for ExitMessage<'a> {
    fn from(cause: &'a ReplicateStatusCause) -> Self {
        Self { cause }
    }
}

/// Thin wrapper around a [`Client`] that knows how to reach the iExec worker API.
///
/// This client can be created directly with a base URL using [`new()`], or
/// configured from environment variables using [`from_env()`].
///
/// # Example
///
/// ```
/// use crate::pre_compute::worker_api::WorkerApiClient;
///
/// let client = WorkerApiClient::new("http://worker:13100");
/// ```
pub struct WorkerApiClient {
    base_url: String,
    client: Client,
}

const DEFAULT_WORKER_HOST: &str = "worker:13100";

impl WorkerApiClient {
     fn new(base_url: &str) -> Self {
        WorkerApiClient {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    /// Creates a new WorkerApiClient instance with configuration from environment variables.
    ///
    /// This method retrieves the worker host from the [`WORKER_HOST_ENV_VAR`] environment variable.
    /// If the variable is not set or empty, it defaults to `"worker:13100"`.
    ///
    /// # Returns
    ///
    /// * `WorkerApiClient` - A new client configured with the appropriate base URL
    ///
    /// # Example
    ///
    /// ```
    /// use crate::api::worker_api::WorkerApiClient;
    ///
    /// let client = WorkerApiClient::from_env();
    /// ```
    pub fn from_env() -> Self {
        let worker_host = get_env_var_or_error(
            TeeSessionEnvironmentVariable::WORKER_HOST_ENV_VAR,
            ReplicateStatusCause::PreComputeWorkerAddressMissing,
        )
            .unwrap_or_else(|_| DEFAULT_WORKER_HOST.to_string());

        let base_url = format!("http://{}", &worker_host);
        Self::new(&base_url)
    }

    /// Sends an exit cause for a pre-compute operation to the Worker API.
    ///
    /// This method reports the exit cause of a pre-compute operation to the Worker API,
    /// which can be used for tracking and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `authorization` - The authorization token to use for the API request
    /// * `chain_task_id` - The chain task ID for which to report the exit cause
    /// * `exit_cause` - The exit cause to report
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the exit cause was successfully reported
    /// * `Err(Error)` - If the exit cause could not be reported due to an HTTP error
    ///
    /// # Errors
    ///
    /// This function will return an [`Error`] if the request could not be sent or
    /// the server responded with a non‑success status.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::pre_compute::worker_api::{ExitMessage, WorkerApiClient};
    /// use crate::pre_compute::errors::ReplicateStatusCause;
    ///
    /// let client = WorkerApiClient::new("http://worker:13100");
    /// let exit_message = ExitMessage::from(&ReplicateStatusCause::PreComputeInvalidTeeSignature);
    ///
    /// match client.send_exit_cause_for_pre_compute_stage(
    ///     "authorization_token",
    ///     "0x123456789abcdef",
    ///     &exit_message,
    /// ) {
    ///     Ok(()) => println!("Exit cause reported successfully"),
    ///     Err(error) => eprintln!("Failed to report exit cause: {}", error),
    /// }
    /// ```
    pub fn send_exit_cause_for_pre_compute_stage(
        &self,
        authorization: &str,
        chain_task_id: &str,
        exit_cause: &ExitMessage,
    ) -> Result<(), Error> {
        let url = format!("{}/compute/pre/{}/exit", self.base_url, chain_task_id);
        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, authorization)
            .json(exit_cause)
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(response.error_for_status().unwrap_err())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_string};
    use serial_test::serial;
    use temp_env::with_vars;
    use wiremock::{
        matchers::{body_json, header, method, path}, Mock, MockServer,
        ResponseTemplate,
    };

    // region ExitMessage()
    #[test]
    fn should_serialize_exit_message() {
        let causes = [
            ReplicateStatusCause::PreComputeInvalidTeeSignature,
            ReplicateStatusCause::PreComputeWorkerAddressMissing,
            ReplicateStatusCause::PreComputeFailedUnknownIssue,
        ];

        for cause in causes {
            let exit_message = ExitMessage::from(&cause);
            let serialized = to_string(&exit_message).expect("Failed to serialize");
            let expected = format!("{{\"cause\":{}}}", to_string(&cause).unwrap());
            assert_eq!(serialized, expected);
        }
    }
    // endregion

    // region get_worker_api_client
    #[test]
    #[serial]
    fn should_get_worker_api_client_with_env_var() {
        with_vars(
            vec![(
                TeeSessionEnvironmentVariable::WORKER_HOST_ENV_VAR.name(),
                Some("custom-worker-host:9999"),
            )],
            || {
                let client = WorkerApiClient::from_env();
                assert_eq!(client.base_url, "http://custom-worker-host:9999");
            },
        );
    }

    #[test]
    #[serial]
    fn should_get_worker_api_client_without_env_var() {
        let client = WorkerApiClient::from_env();
        assert_eq!(client.base_url, format!("http://{}", DEFAULT_WORKER_HOST));
    }
    // endregion

    // region send_exit_cause_for_pre_compute_stage()
    const CHALLENGE: &str = "challenge";
    const CHAIN_TASK_ID: &str = "0x123456789abcdef";

    #[tokio::test]
    async fn should_send_exit_cause() {
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        let expected_body = json!({
            "cause": ReplicateStatusCause::PreComputeInvalidTeeSignature,
        });

        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{}/exit", CHAIN_TASK_ID)))
            .and(header("Authorization", CHALLENGE))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let exit_message =
                ExitMessage::from(&ReplicateStatusCause::PreComputeInvalidTeeSignature);
            let worker_api_client = WorkerApiClient::new(&server_url);
            worker_api_client.send_exit_cause_for_pre_compute_stage(
                CHALLENGE,
                CHAIN_TASK_ID,
                &exit_message,
            )
        })
            .await
            .expect("Task panicked");

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_send_exit_cause() {
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{}/exit", CHAIN_TASK_ID)))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let exit_message =
                ExitMessage::from(&ReplicateStatusCause::PreComputeFailedUnknownIssue);
            let worker_api_client = WorkerApiClient::new(&server_url);
            worker_api_client.send_exit_cause_for_pre_compute_stage(
                CHALLENGE,
                CHAIN_TASK_ID,
                &exit_message,
            )
        })
            .await
            .expect("Task panicked");

        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.status().unwrap(), 404);
        }
    }
    // endregion
}