use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, Serialize, Deserialize)]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
#[allow(clippy::enum_variant_names)]
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
