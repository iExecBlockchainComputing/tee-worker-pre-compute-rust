use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, Serialize, Deserialize)]
pub enum ReplicateStatusCause {
    #[error("TEE challenge private key is missing")]
    PreComputeTeeChallengePrivateKeyMissing,
    #[error("Invalid TEE signature")]
    PreComputeInvalidTeeSignature,
    #[error("Worker address is missing")]
    PreComputeWorkerAddressMissing,
    #[error("Task ID is missing")]
    PreComputeTaskIdMissing,
    #[error("Pre Compute failed due to an unknown issue")]
    PreComputeFailedUnknownIssue,
}

#[derive(Debug, Error, Clone)]
#[error("PreCompute failed: {exit_cause}")]
pub struct PreComputeError {
    pub exit_cause: ReplicateStatusCause,
}

impl PreComputeError {
    pub fn new(cause: ReplicateStatusCause) -> Self {
        Self { exit_cause: cause }
    }

    pub fn exit_cause(&self) -> &ReplicateStatusCause {
        &self.exit_cause
    }
}
