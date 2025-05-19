use crate::compute::errors::ReplicateStatusCause;
use std::env;

pub enum TeeSessionEnvironmentVariable {
    IexecTaskId,
    SignWorkerAddress,
    SignTeeChallengePrivateKey,
    WorkerHostEnvVar,
}

impl TeeSessionEnvironmentVariable {
    pub fn name(&self) -> &str {
        match self {
            TeeSessionEnvironmentVariable::IexecTaskId => "IEXEC_TASK_ID",
            TeeSessionEnvironmentVariable::SignWorkerAddress => "SIGN_WORKER_ADDRESS",
            TeeSessionEnvironmentVariable::SignTeeChallengePrivateKey => {
                "SIGN_TEE_CHALLENGE_PRIVATE_KEY"
            }
            TeeSessionEnvironmentVariable::WorkerHostEnvVar => "WORKER_HOST_ENV_VAR",
        }
    }
}

pub fn get_env_var_or_error(
    env_var: TeeSessionEnvironmentVariable,
    status_cause_if_missing: ReplicateStatusCause,
) -> Result<String, ReplicateStatusCause> {
    match env::var(env_var.name()) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(status_cause_if_missing),
    }
}
