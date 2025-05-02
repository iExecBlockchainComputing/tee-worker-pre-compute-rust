#[derive(Debug, Clone, PartialEq)]
pub enum ReplicateStatusCause {
    PreComputeTeeChallengePrivateKeyMissing,
    PreComputeInvalidTeeSignature,
    PreComputeWorkerAddressMissing,
    PreComputeTaskIdMissing,
}

#[derive(Debug)]
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

impl std::fmt::Display for PreComputeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PreComputeException: {:?}", self.exit_cause)
    }
}

impl std::error::Error for PreComputeError {}
