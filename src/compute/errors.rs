use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, Serialize, Deserialize)]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
#[allow(clippy::enum_variant_names)]
pub enum ReplicateStatusCause {
    #[error("Unexpected error occurred")]
    PreComputeFailedUnknownIssue,
    #[error("Invalid TEE signature")]
    PreComputeInvalidTeeSignature,
    #[error("Task ID related environment variable is missing")]
    PreComputeTaskIdMissing,
    #[error("TEE challenge private key related environment variable is missing")]
    PreComputeTeeChallengePrivateKeyMissing,
    #[error("Worker address related environment variable is missing")]
    PreComputeWorkerAddressMissing,
}
