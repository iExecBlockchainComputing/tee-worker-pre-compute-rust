use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use std::env;

pub enum TeeSessionEnvironmentVariable {
    SIGN_WORKER_ADDRESS,
    SIGN_TEE_CHALLENGE_PRIVATE_KEY,
}

impl TeeSessionEnvironmentVariable {
    pub fn name(&self) -> &str {
        match self {
            TeeSessionEnvironmentVariable::SIGN_WORKER_ADDRESS => "SIGN_WORKER_ADDRESS",
            TeeSessionEnvironmentVariable::SIGN_TEE_CHALLENGE_PRIVATE_KEY => {
                "SIGN_TEE_CHALLENGE_PRIVATE_KEY"
            }
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
