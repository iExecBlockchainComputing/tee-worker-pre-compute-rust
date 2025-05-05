use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use crate::utils::env_utils::{TeeSessionEnvironmentVariable, get_env_var_or_error};
use crate::utils::hash_utils::{concatenate_and_hash, hex_string_to_byte_array};
use alloy_signer::{Signature, SignerSync};
use alloy_signer_local::PrivateKeySigner;

pub fn sign_enclave_challenge(
    message_hash: &str,
    enclave_challenge_private_key: &str,
) -> Result<String, PreComputeError> {
    let signer: PrivateKeySigner = enclave_challenge_private_key
        .parse::<PrivateKeySigner>()
        .map_err(|_| {
            PreComputeError::new(ReplicateStatusCause::PreComputeTeeChallengePrivateKeyMissing)
        })?;

    let signature: Signature = signer
        .sign_message_sync(&hex_string_to_byte_array(&message_hash))
        .map_err(|_| PreComputeError::new(ReplicateStatusCause::PreComputeInvalidTeeSignature))?;

    Ok(signature.to_string())
}

pub fn challenge(chain_task_id: &str) -> Result<String, PreComputeError> {
    let worker_address = get_env_var_or_error(
        TeeSessionEnvironmentVariable::SIGN_WORKER_ADDRESS,
        ReplicateStatusCause::PreComputeWorkerAddressMissing,
    )?;

    let tee_challenge_private_key = get_env_var_or_error(
        TeeSessionEnvironmentVariable::SIGN_TEE_CHALLENGE_PRIVATE_KEY,
        ReplicateStatusCause::PreComputeTeeChallengePrivateKeyMissing,
    )?;

    let message_hash = concatenate_and_hash(&[chain_task_id, &worker_address]);
    sign_enclave_challenge(&message_hash, &tee_challenge_private_key)
}

#[cfg(test)]
mod env_utils_tests {
    use super::*;
    use temp_env::with_vars;

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";
    const WORKER_ADDRESS: &str = "0xabcdef123456789";
    const ENCLAVE_CHALLENGE_PRIVATE_KEY: &str =
        "0xdd3b993ec21c71c1f6d63a5240850e0d4d8dd83ff70d29e49247958548c1d479";
    const MESSAGE_HASH: &str = "0x5cd0e9c5180dd35e2b8285d0db4ded193a9b4be6fbfab90cbadccecab130acad";
    const EXPECTED_CHALLENGE: &str = "0xfcc6bce5eb04284c2eb1ed14405b943574343b1abda33628fbf94a374b18dd16541c6ebf63c6943d8643ff03c7aa17f1cb17b0a8d297d0fd95fc914bdd0e85f81b";

    #[test]
    fn test_sign_enclave_challenge() {
        let result = sign_enclave_challenge(MESSAGE_HASH, ENCLAVE_CHALLENGE_PRIVATE_KEY).unwrap();
        assert_eq!(result, EXPECTED_CHALLENGE);
    }

    #[test]
    fn test_get_challenge() {
        with_vars(
            vec![
                ("SignWorkerAddress", Some(WORKER_ADDRESS)),
                (
                    "SignTeeChallengePrivateKey",
                    Some(ENCLAVE_CHALLENGE_PRIVATE_KEY),
                ),
            ],
            || {
                let message_hash = concatenate_and_hash(&[CHAIN_TASK_ID, WORKER_ADDRESS]);
                let expected_signature =
                    sign_enclave_challenge(&message_hash, ENCLAVE_CHALLENGE_PRIVATE_KEY).unwrap();

                let actual_challenge = challenge(CHAIN_TASK_ID).unwrap();
                assert_eq!(actual_challenge, expected_signature);
            },
        );
    }

    #[test]
    fn error_when_worker_address_missing() {
        with_vars(
            vec![(
                "SignTeeChallengePrivateKey",
                Some(ENCLAVE_CHALLENGE_PRIVATE_KEY),
            )],
            || {
                let err = challenge(CHAIN_TASK_ID).unwrap_err();
                assert_eq!(
                    *err.exit_cause(),
                    ReplicateStatusCause::PreComputeWorkerAddressMissing
                );
            },
        );
    }

    #[test]
    fn error_when_challenge_private_key_missing() {
        with_vars(vec![("SignWorkerAddress", Some(WORKER_ADDRESS))], || {
            let err = challenge(CHAIN_TASK_ID).unwrap_err();
            assert_eq!(
                *err.exit_cause(),
                ReplicateStatusCause::PreComputeTeeChallengePrivateKeyMissing
            );
        });
    }
}
