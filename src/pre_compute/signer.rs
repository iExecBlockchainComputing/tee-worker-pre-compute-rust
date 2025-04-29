use crate::pre_compute::errors::{PreComputeError, ReplicateStatusCause};
use crate::utils::credentials_utils;
use crate::utils::env_utils::{get_env_var_or_throw, TeeSessionEnvironmentVariable};
use crate::utils::hash_utils::concatenate_and_hash;
use crate::utils::signature_utils::{
    is_expected_signer_on_signed_message_hash, sign_message_hash_and_get_signature,
};

pub struct Signer {}

impl Signer {
    pub fn sign_enclave_challenge(
        message_hash: &str,
        enclave_challenge_private_key: &str,
    ) -> Result<String, PreComputeError> {
        let enclave_challenge_signature =
            sign_message_hash_and_get_signature(message_hash, enclave_challenge_private_key)
                .unwrap();
        let is_signature_valid = is_expected_signer_on_signed_message_hash(
            &message_hash,
            &enclave_challenge_signature,
            &credentials_utils::get_address(enclave_challenge_private_key),
        );

        if !is_signature_valid {
            return Err(PreComputeError::new(
                ReplicateStatusCause::PreComputeInvalidTeeSignature,
            ));
        }
        Ok("0x".to_string() + enclave_challenge_signature.value())
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

        Ok(Signer::sign_enclave_challenge(&message_hash, &tee_challenge_private_key)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";
    const WORKER_ADDRESS: &str = "0xabcdef123456789";
    const ENCLAVE_CHALLENGE_PRIVATE_KEY: &str = "0xdd3b993ec21c71c1f6d63a5240850e0d4d8dd83ff70d29e49247958548c1d479";
    const MESSAGE_HASH: &str = "0x5cd0e9c5180dd35e2b8285d0db4ded193a9b4be6fbfab90cbadccecab130acad";
    const EXPECTED_CHALLENGE: &str = "0xfcc6bce5eb04284c2eb1ed14405b943574343b1abda33628fbf94a374b18dd16541c6ebf63c6943d8643ff03c7aa17f1cb17b0a8d297d0fd95fc914bdd0e85f81b";

    #[test]
    fn should_sign_enclave_challenge() {
        let result = Signer::sign_enclave_challenge(MESSAGE_HASH, ENCLAVE_CHALLENGE_PRIVATE_KEY);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), EXPECTED_CHALLENGE);
    }

    #[test]
    fn should_get_challenge() {
        temp_env::with_vars(
            vec![
                ("SignWorkerAddress", Some(WORKER_ADDRESS)),
                ("SignTeeChallengePrivateKey", Some(ENCLAVE_CHALLENGE_PRIVATE_KEY)),
            ],
            || {
                let expected_message_hash = concatenate_and_hash(&[CHAIN_TASK_ID, WORKER_ADDRESS]);
                let expected_signature = Signer::sign_enclave_challenge(
                    &expected_message_hash,
                    ENCLAVE_CHALLENGE_PRIVATE_KEY,
                )
                    .unwrap();

                let actual_challenge = Signer::get_challenge(CHAIN_TASK_ID).unwrap();
                assert_eq!(actual_challenge, expected_signature);
            },
        );
    }

    #[test]
    fn should_throw_when_worker_address_environment_variable_missing() {
        temp_env::with_vars(
            vec![
                ("SignWorkerAddress", None),
                ("SignTeeChallengePrivateKey", Some(ENCLAVE_CHALLENGE_PRIVATE_KEY)),
            ],
            || {
                let result = Signer::get_challenge(CHAIN_TASK_ID);
                assert!(result.is_err());
                let error = result.unwrap_err();
                assert_eq!(
                    error.exit_cause(),
                    &ReplicateStatusCause::PreComputeWorkerAddressMissing
                );
            },
        );
    }

    #[test]
    fn should_throw_when_challenge_private_key_environment_variable_missing() {
        temp_env::with_vars(
            vec![
                ("SignWorkerAddress", Some(WORKER_ADDRESS)),
                ("SignTeeChallengePrivateKey", None),
            ],
            || {
                let result = Signer::get_challenge(CHAIN_TASK_ID);
                assert!(result.is_err());
                let error = result.unwrap_err();
                assert_eq!(
                    error.exit_cause(),
                    &ReplicateStatusCause::PreComputeTeeChallengePrivateKeyMissing
                );
            },
        );
    }
}
