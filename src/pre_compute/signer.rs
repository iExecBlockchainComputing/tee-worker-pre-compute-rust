use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use crate::utils::env_utils::{get_env_var_or_throw, TeeSessionEnvironmentVariable};
use crate::utils::hash_utils::concatenate_and_hash;
use alloy_primitives::hex;
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
        .sign_message_sync(message_hash.as_ref())
        .map_err(|_| PreComputeError::new(ReplicateStatusCause::PreComputeInvalidTeeSignature))?;

    let signature_bytes = signature.as_bytes();

    let signature_hex = format!("0x{}", hex::encode(signature_bytes));

    Ok(signature_hex)
}

pub fn get_challenge(chain_task_id: &str) -> Result<String, PreComputeError> {
    let worker_address = get_env_var_or_throw(
        &TeeSessionEnvironmentVariable::SignWorkerAddress,
        ReplicateStatusCause::PreComputeWorkerAddressMissing,
    )?;

    let tee_challenge_private_key = get_env_var_or_throw(
        &TeeSessionEnvironmentVariable::SignTeeChallengePrivateKey,
        ReplicateStatusCause::PreComputeTeeChallengePrivateKeyMissing,
    )?;

    let message_hash = concatenate_and_hash(&[chain_task_id, &worker_address]);

    Ok(sign_enclave_challenge(
        &message_hash,
        &tee_challenge_private_key,
    )?)
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
    const EXPECTED_CHALLENGE: &str = "0xc81d2535a0d0a5e6d3046f7b8109ac9d05b216b7205ad865403549334a39f0981bc6c48b8e3d76f70cbd8a62f530442645592e4293c0309fbb3506e7ce6ae24a1b";

    #[test]
    fn should_sign_enclave_challenge() {
        let result = sign_enclave_challenge(MESSAGE_HASH, ENCLAVE_CHALLENGE_PRIVATE_KEY).unwrap();
        assert_eq!(result, EXPECTED_CHALLENGE);
    }

    #[test]
    fn should_get_challenge() {
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

                let actual_challenge = get_challenge(CHAIN_TASK_ID).unwrap();
                assert_eq!(actual_challenge, expected_signature);
            },
        );
    }

    #[test]
    fn should_throw_when_worker_address_missing() {
        with_vars(
            vec![(
                "SignTeeChallengePrivateKey",
                Some(ENCLAVE_CHALLENGE_PRIVATE_KEY),
            )],
            || {
                let err = get_challenge(CHAIN_TASK_ID).unwrap_err();
                assert_eq!(
                    *err.exit_cause(),
                    ReplicateStatusCause::PreComputeWorkerAddressMissing
                );
            },
        );
    }

    #[test]
    fn should_throw_when_challenge_private_key_missing() {
        with_vars(vec![("SignWorkerAddress", Some(WORKER_ADDRESS))], || {
            let err = get_challenge(CHAIN_TASK_ID).unwrap_err();
            assert_eq!(
                *err.exit_cause(),
                ReplicateStatusCause::PreComputeTeeChallengePrivateKeyMissing
            );
        });
    }
}
