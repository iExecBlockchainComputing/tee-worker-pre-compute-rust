use crate::api::worker_api::{ExitMessage, WorkerApiClient};
use crate::compute::{
    errors::{PreComputeError, ReplicateStatusCause},
    signer::get_challenge,
    utils::env_utils::{get_env_var_or_error, TeeSessionEnvironmentVariable::IEXEC_TASK_ID},
};
use log::{error, info};

/// Executes the pre-compute workflow.
///
/// This function orchestrates the full pre-compute process, handling environment
/// variable checks, execution of the main pre-compute logic, and error reporting.
/// It executes core operations and handles all the workflow states and transitions.
///
/// # Returns
///
/// * `i32` - An exit code indicating the result of the pre-compute process:
///   - 0: Success - pre-compute completed successfully
///   - 1: Failure with reported cause - pre-compute failed but the cause was reported
///   - 2: Failure with unreported cause - pre-compute failed and the cause could not be reported
///   - 3: Failure due to missing taskID context - pre-compute could not start due to missing task ID
///
/// # Example
///
/// ```
/// use crate::app_runner::{start};
///
/// let exit_code = start();
/// ```

pub fn start() -> i32 {
    info!("TEE pre-compute started");

    let mut exit_cause = ReplicateStatusCause::PreComputeFailedUnknownIssue;
    let chain_task_id =
        match get_env_var_or_error(IEXEC_TASK_ID, ReplicateStatusCause::PreComputeTaskIdMissing) {
            Ok(id) => id,
            Err(e) => {
                error!(
                    "TEE pre-compute cannot proceed without taskID context: {:?}",
                    e
                );
                return 3;
            }
        };

    match run() {
        Ok(_) => {
            info!("TEE pre-compute completed");
            return 0;
        }
        Err(e) => {
            exit_cause = e.exit_cause().clone();
            error!(
                "TEE pre-compute failed with known exit cause [{:?}]",
                exit_cause
            );
        }
    }

    let authorization = match get_challenge(&chain_task_id) {
        Ok(auth) => auth,
        Err(e) => {
            error!("Failed to sign exitCause message [{:?}]", exit_cause);
            return 2;
        }
    };

    let exit_message = ExitMessage {
        cause: &exit_cause.clone(),
    };

    match WorkerApiClient::from_env().send_exit_cause_for_pre_compute_stage(
        &authorization,
        &chain_task_id,
        &exit_message,
    ) {
        Ok(_) => 1,
        Err(e) => {
            error!("Failed to report exitCause [{:?}]", exit_cause);
            2
        }
    }
}

pub fn run() -> Result<(), PreComputeError> {
    Err(PreComputeError::new(
        ReplicateStatusCause::PreComputeFailedUnknownIssue,
    ))
}

#[cfg(test)]
mod pre_compute_start_tests {
    use super::*;
    use serde_json::json;
    use temp_env;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";
    const WORKER_ADDRESS: &str = "0xabcdef123456789";
    const ENCLAVE_CHALLENGE_PRIVATE_KEY: &str =
        "0xdd3b993ec21c71c1f6d63a5240850e0d4d8dd83ff70d29e49247958548c1d479";
    const ENV_IEXEC_TASK_ID: &str = "IEXEC_TASK_ID";
    const ENV_SIGN_WORKER_ADDRESS: &str = "SIGN_WORKER_ADDRESS";
    const ENV_SIGN_TEE_CHALLENGE_PRIVATE_KEY: &str = "SIGN_TEE_CHALLENGE_PRIVATE_KEY";
    const ENV_WORKER_HOST: &str = "WORKER_HOST_ENV_VAR";
    const DEFAULT_WORKER_HOST: &str = "localhost:8080";

    #[test]
    fn start_fails_when_task_id_missing() {
        temp_env::with_vars_unset(vec![ENV_IEXEC_TASK_ID], || {
            assert_eq!(start(), 3, "Should return 3 if IEXEC_TASK_ID is missing");
        });
    }

    #[test]
    fn start_fails_when_signer_address_missing() {
        let env_vars_to_set = vec![
            (ENV_IEXEC_TASK_ID, Some(CHAIN_TASK_ID)),
            (
                ENV_SIGN_TEE_CHALLENGE_PRIVATE_KEY,
                Some(ENCLAVE_CHALLENGE_PRIVATE_KEY),
            ),
        ];
        let env_vars_to_unset = vec![ENV_SIGN_WORKER_ADDRESS];

        temp_env::with_vars(env_vars_to_set, || {
            temp_env::with_vars_unset(env_vars_to_unset, || {
                assert_eq!(
                    start(),
                    2,
                    "Should return 2 if get_challenge fails due to missing signer address"
                );
            });
        });
    }

    #[test]
    fn start_fails_when_private_key_missing() {
        let env_vars_to_set = vec![
            (ENV_IEXEC_TASK_ID, Some(CHAIN_TASK_ID)),
            (ENV_SIGN_WORKER_ADDRESS, Some(WORKER_ADDRESS)),
        ];
        let env_vars_to_unset = vec![ENV_SIGN_TEE_CHALLENGE_PRIVATE_KEY];

        temp_env::with_vars(env_vars_to_set, || {
            temp_env::with_vars_unset(env_vars_to_unset, || {
                assert_eq!(
                    start(),
                    2,
                    "Should return 2 if get_challenge fails due to missing private key"
                );
            });
        });
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn start_fails_when_send_exit_cause_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{}/exit", CHAIN_TASK_ID)))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let mock_server_addr_string = mock_server.address().to_string();

        let result_code = tokio::task::spawn_blocking(move || {
            let env_vars = vec![
                (ENV_IEXEC_TASK_ID, Some(CHAIN_TASK_ID)),
                (ENV_SIGN_WORKER_ADDRESS, Some(WORKER_ADDRESS)),
                (
                    ENV_SIGN_TEE_CHALLENGE_PRIVATE_KEY,
                    Some(ENCLAVE_CHALLENGE_PRIVATE_KEY),
                ),
                (ENV_WORKER_HOST, Some(mock_server_addr_string.as_str())),
            ];

            temp_env::with_vars(env_vars, start)
        })
        .await
        .expect("Blocking task panicked");

        assert_eq!(
            result_code, 2,
            "Should return 2 if sending exit cause to worker API fails"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn start_succeeds_when_send_exit_cause_api_success() {
        let mock_server = MockServer::start().await;

        let expected_cause_enum = ReplicateStatusCause::PreComputeFailedUnknownIssue;
        let expected_exit_message_payload = json!({
            "cause": expected_cause_enum // Relies on ReplicateStatusCause's Serialize impl
        });

        // Mock the worker API to return success
        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{}/exit", CHAIN_TASK_ID)))
            .and(body_json(expected_exit_message_payload))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mock_server_addr_string = mock_server.address().to_string();

        // Move the blocking operations into spawn_blocking
        let result_code = tokio::task::spawn_blocking(move || {
            let env_vars = vec![
                (ENV_IEXEC_TASK_ID, Some(CHAIN_TASK_ID)),
                (ENV_SIGN_WORKER_ADDRESS, Some(WORKER_ADDRESS)),
                (
                    ENV_SIGN_TEE_CHALLENGE_PRIVATE_KEY,
                    Some(ENCLAVE_CHALLENGE_PRIVATE_KEY),
                ),
                (ENV_WORKER_HOST, Some(mock_server_addr_string.as_str())),
            ];

            temp_env::with_vars(env_vars, start)
        })
        .await
        .expect("Blocking task panicked");

        assert_eq!(
            result_code, 1,
            "Should return 1 if sending exit cause to worker API succeeds"
        );
    }
}
