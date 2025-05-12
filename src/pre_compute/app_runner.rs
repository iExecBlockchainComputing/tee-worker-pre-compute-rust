use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use crate::pre_compute::signer::get_challenge;
use crate::pre_compute::worker_api::{ExitMessage, get_worker_api_client};
use crate::utils::env_utils::TeeSessionEnvironmentVariable::IEXEC_TASK_ID;
use crate::utils::env_utils::get_env_var_or_error;
use log::{error, info};

pub async fn start() -> i32 {
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
        cause: exit_cause.clone(),
    };

    match get_worker_api_client()
        .send_exit_cause_for_pre_compute_stage(&authorization, &chain_task_id, &exit_message)
        .await
    {
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
mod app_runner_tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";

    #[tokio::test]
    async fn test_missing_task_id_exits_3() {
        unsafe {
            std::env::remove_var("IEXEC_TASK_ID");
        }
        let exit_code = start().await;
        assert_eq!(exit_code, 3);
    }

    #[tokio::test]
    async fn test_exit_cause_sent_successfully_exits_1() {
        let worker_mock = MockServer::start().await;
        let challenge_mock = MockServer::start().await;

        unsafe {
            std::env::set_var("IEXEC_TASK_ID", CHAIN_TASK_ID);
            std::env::set_var("WORKER_API_BASE_URL", worker_mock.uri());
            std::env::set_var("CHALLENGE_SERVICE_URL", challenge_mock.uri());
        }

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("valid_auth"))
            .mount(&challenge_mock)
            .await;

        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{}/exit", CHAIN_TASK_ID)))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&worker_mock)
            .await;

        let exit_code = start().await;
        assert_eq!(exit_code, 1);
    }
}
