use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use crate::pre_compute::signer::get_challenge;
use crate::pre_compute::worker_api::{ExitMessage, WorkerApiClient};
use crate::utils::env_utils::get_env_var_or_error;
use crate::utils::env_utils::TeeSessionEnvironmentVariable::IEXEC_TASK_ID;
use log::{error, info};

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

    match WorkerApiClient::from_env()
        .send_exit_cause_for_pre_compute_stage(&authorization, &chain_task_id, &exit_message)
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
mod tests {
    use super::*;
    use crate::utils::env_utils::TeeSessionEnvironmentVariable;
    use crate::utils::env_utils::TeeSessionEnvironmentVariable::*;
    use mockito;
    use serial_test::serial;
    use std::env;

    // Hellpers
    struct EnvVarGuard {
        key: String,
        original_value: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: TeeSessionEnvironmentVariable, value: &str) -> Self {
            let key_name = key.name().to_string();
            let original_value = env::var(&key_name).ok();
            unsafe {
                env::set_var(&key_name, value);
            }
            EnvVarGuard { key: key_name, original_value }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(val) = &self.original_value {
                unsafe {
                    env::set_var(&self.key, val);
                }
            } else {
                unsafe {
                    env::remove_var(&self.key);
                }
            }
        }
    }

    // Helper to clear an environment variable and restore it (if it existed)
    struct EnvVarClearGuard {
        key: String,
        original_value: Option<String>,
    }

    impl EnvVarClearGuard {
        fn clear(key: TeeSessionEnvironmentVariable) -> Self {
            let key_name = key.name().to_string();
            let original_value = env::var(&key_name).ok();
            unsafe {
                env::remove_var(&key_name);
            }
            EnvVarClearGuard { key: key_name, original_value }
        }
    }

    impl Drop for EnvVarClearGuard {
        fn drop(&mut self) {
            if let Some(val) = &self.original_value {
                unsafe {
                    env::set_var(&self.key, val);
                }
            }
        }
    }


    const MOCK_TASK_ID: &str = "0x123456789abcdef";
    const MOCK_WORKER_ADDRESS: &str = "0xabcdef123456789";
    const MOCK_TEE_CHALLENGE_KEY: &str = "0xdd3b993ec21c71c1f6d63a5240850e0d4d8dd83ff70d29e49247958548c1d479";

    #[test]
    #[serial]
    fn test_start_iexec_task_id_missing() {
        let _guard = EnvVarClearGuard::clear(IEXEC_TASK_ID);

        assert_eq!(start(), 3, "Should return 3 if IEXEC_TASK_ID is missing");
    }

    #[test]
    #[serial]
    fn test_start_run_fails_and_get_challenge_fails() {
        let _task_id_guard = EnvVarGuard::set(IEXEC_TASK_ID, MOCK_TASK_ID);

        let _worker_addr_guard = EnvVarClearGuard::clear(SIGN_WORKER_ADDRESS);
        let _challenge_key_guard = EnvVarClearGuard::clear(SIGN_TEE_CHALLENGE_PRIVATE_KEY);

        assert_eq!(start(), 2, "Should return 2 if run fails and get_challenge fails");
    }

    #[test]
    #[serial]
    fn test_start_run_fails_get_challenge_succeeds_send_exit_cause_succeeds() {
        let mut server = mockito::Server::new();
        let mock_server_url = server.url();

        let host_port = mock_server_url.split("://").nth(1).expect("Invalid mock server URL");

        let _task_id_guard = EnvVarGuard::set(IEXEC_TASK_ID, MOCK_TASK_ID);
        let _worker_addr_guard = EnvVarGuard::set(SIGN_WORKER_ADDRESS, MOCK_WORKER_ADDRESS);
        let _challenge_key_guard = EnvVarGuard::set(SIGN_TEE_CHALLENGE_PRIVATE_KEY, MOCK_TEE_CHALLENGE_KEY);
        let _worker_host_guard = EnvVarGuard::set(WORKER_HOST_ENV_VAR, host_port);

        let endpoint_path = format!("/compute/pre/{}/exit", MOCK_TASK_ID);
        let mock_api = server.mock("POST", endpoint_path.as_str())
            .with_status(200) // Simulate successful reporting
            .with_header("content-type", "application/json")
            .with_body("{\"status\":\"reported\"}")
            .create();

        assert_eq!(start(), 1, "Should return 1 on successful error reporting");

        mock_api.assert();
    }

    #[test]
    #[serial]
    fn test_start_run_fails_get_challenge_succeeds_send_exit_cause_fails() {
        let mut server = mockito::Server::new();
        let mock_server_url = server.url();
        let host_port = mock_server_url.split("://").nth(1).expect("Invalid mock server URL");

        let _task_id_guard = EnvVarGuard::set(IEXEC_TASK_ID, MOCK_TASK_ID);
        let _worker_addr_guard = EnvVarGuard::set(SIGN_WORKER_ADDRESS, MOCK_WORKER_ADDRESS);
        let _challenge_key_guard = EnvVarGuard::set(SIGN_TEE_CHALLENGE_PRIVATE_KEY, MOCK_TEE_CHALLENGE_KEY);
        let _worker_host_guard = EnvVarGuard::set(WORKER_HOST_ENV_VAR, host_port);

        let endpoint_path = format!("/compute/pre/{}/exit", MOCK_TASK_ID);
        let mock_api = server.mock("POST", endpoint_path.as_str())
            .with_status(500)
            .with_body("{\"error\":\"failed to report\"}")
            .create();

        assert_eq!(start(), 2, "Should return 2 if reporting exit cause fails");

        mock_api.assert();
    }

}
