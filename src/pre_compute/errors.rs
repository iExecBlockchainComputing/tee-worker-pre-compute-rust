use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReplicateStatusCause {
    #[error("TEE challenge private key is missing")]
    PreComputeTeeChallengePrivateKeyMissing,
    #[error("Invalid enclave challenge private key")]
    PreComputeInvalidEnclaveChallengePrivateKey,
    #[error("Invalid TEE signature")]
    PreComputeInvalidTeeSignature,
    #[error("Task ID is missing")]
    PreComputeTaskIdMissing,
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
