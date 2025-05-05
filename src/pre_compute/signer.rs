use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use crate::utils::env_utils::{get_env_var_or_error, TeeSessionEnvironmentVariable};
use crate::utils::hash_utils::{concatenate_and_hash, hex_string_to_byte_array};
use alloy_signer::{Signature, SignerSync};
use alloy_signer_local::PrivateKeySigner;

/// Signs a message hash using the enclave challenge private key.
///
/// This function takes a hex-encoded message hash and a private key, then signs the message
/// using the private key. It converts the signature to a string representation.
///
/// # Arguments
///
/// * `message_hash` - A hex-encoded string representing the hash of the message to sign
/// * `enclave_challenge_private_key` - A string containing the private key to use for signing
///
/// # Returns
///
/// * `Result<String, PreComputeError>` - A string representation of the signature if successful,
///   or an error if the private key is invalid or the signing operation fails
///
/// # Errors
///
/// * `PreComputeTeeChallengePrivateKeyMissing` - When the private key is invalid or cannot be parsed
/// * `PreComputeInvalidTeeSignature` - When the signing operation fails
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

/// Generates a challenge signature for a given chain task ID.
///
/// This function creates a challenge signature by:
/// 1. Retrieving the worker address and private key from environment variables
/// 2. Concatenating and hashing the chain task ID with the worker address
/// 3. Signing the resulting hash with the TEE challenge private key
///
/// # Arguments
///
/// * `chain_task_id` - A string representing the chain task ID to use in the challenge
///
/// # Returns
///
/// * `Result<String, PreComputeError>` - The challenge signature as a string if successful,
///   or an error if environment variables are missing or signing fails
///
/// # Errors
///
/// * `PreComputeWorkerAddressMissing` - When the worker address environment variable is missing
/// * `PreComputeTeeChallengePrivateKeyMissing` - When the private key environment variable is missing
/// * Other errors that may be propagated from `sign_enclave_challenge`
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
