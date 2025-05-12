use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Error)]
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

#[derive(Debug, Clone, PartialEq, Error)]
#[error("PreCompute failed: {exit_cause}")]
pub struct PreComputeError {
    exit_cause: ReplicateStatusCause,
}

impl PreComputeError {
    pub fn new(exit_cause: ReplicateStatusCause) -> Self {
        Self { exit_cause }
    }

    pub fn exit_cause(&self) -> &ReplicateStatusCause {
        &self.exit_cause
    }
}
