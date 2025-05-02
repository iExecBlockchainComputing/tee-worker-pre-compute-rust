use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use std::env;

pub enum TeeSessionEnvironmentVariable {
    SignWorkerAddress,
    SignTeeChallengePrivateKey,
}

impl TeeSessionEnvironmentVariable {
    pub fn name(&self) -> &str {
        match self {
            TeeSessionEnvironmentVariable::SignWorkerAddress => "SignWorkerAddress",
            TeeSessionEnvironmentVariable::SignTeeChallengePrivateKey => {
                "SignTeeChallengePrivateKey"
            }
        }
    }
}

pub fn get_env_var_or_throw(
    env_var: &TeeSessionEnvironmentVariable,
    status_cause_if_missing: ReplicateStatusCause,
) -> Result<String, PreComputeError> {
    get_env_var_or_throw_by_name(env_var.name(), status_cause_if_missing)
}

pub fn get_env_var_or_throw_by_name(
    env_var_name: &str,
    status_cause_if_missing: ReplicateStatusCause,
) -> Result<String, PreComputeError> {
    match env::var(env_var_name) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(PreComputeError::new(status_cause_if_missing)),
    }
}

#[cfg(test)]
mod env_utils_tests {
    use super::*;
    use temp_env;

    const ENVIRONMENT_VAR: &str = "envVar";
    const ENVIRONMENT_VAR_VALUE: &str = "envVarValue";

    #[test]
    fn should_get_env_var_or_throw() {
        temp_env::with_var(ENVIRONMENT_VAR, Some(ENVIRONMENT_VAR_VALUE), || {
            let result = get_env_var_or_throw_by_name(
                ENVIRONMENT_VAR,
                ReplicateStatusCause::PreComputeTaskIdMissing,
            );

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), ENVIRONMENT_VAR_VALUE);
        });
    }

    #[test]
    fn should_not_get_env_var_or_throw_since_empty_var() {
        temp_env::with_var(ENVIRONMENT_VAR, Some(""), || {
            let result = get_env_var_or_throw_by_name(
                ENVIRONMENT_VAR,
                ReplicateStatusCause::PreComputeTaskIdMissing,
            );

            assert!(result.is_err());
            let error = result.unwrap_err();
            assert_eq!(
                error.exit_cause(),
                &ReplicateStatusCause::PreComputeTaskIdMissing
            );
        });
    }
}
