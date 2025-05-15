use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use std::env;

pub enum TeeSessionEnvironmentVariable {
    IEXEC_TASK_ID,
    SIGN_WORKER_ADDRESS,
    SIGN_TEE_CHALLENGE_PRIVATE_KEY,
    WORKER_HOST_ENV_VAR,
}

impl TeeSessionEnvironmentVariable {
    pub fn name(&self) -> &str {
        match self {
            TeeSessionEnvironmentVariable::IEXEC_TASK_ID => "IEXEC_TASK_ID",
            TeeSessionEnvironmentVariable::SIGN_WORKER_ADDRESS => "SIGN_WORKER_ADDRESS",
            TeeSessionEnvironmentVariable::SIGN_TEE_CHALLENGE_PRIVATE_KEY => {
                "SIGN_TEE_CHALLENGE_PRIVATE_KEY"
            }
            TeeSessionEnvironmentVariable::WORKER_HOST_ENV_VAR => "WORKER_HOST_ENV_VAR",
        }
    }
}

pub fn get_env_var_or_error(
    env_var: TeeSessionEnvironmentVariable,
    status_cause_if_missing: ReplicateStatusCause,
) -> Result<String, PreComputeError> {
    match env::var(env_var.name()) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(PreComputeError::new(status_cause_if_missing)),
    }
}
