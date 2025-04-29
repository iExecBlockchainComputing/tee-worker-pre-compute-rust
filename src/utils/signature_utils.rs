use crate::utils::signature::Signature;
use hex::FromHexError;
use secp256k1::ecdsa::RecoveryId;
use secp256k1::{ecdsa::RecoverableSignature, Message, PublicKey, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use tiny_keccak::{Hasher, Keccak};

pub fn bytes_to_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

pub fn string_to_bytes(hex_str: &str) -> Result<Vec<u8>, FromHexError> {
    let stripped = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(stripped)
}

pub fn sign_message_hash_and_get_signature(
    message_hash: &str,
    private_key: &str,
) -> Result<Signature, Box<dyn std::error::Error>> {
    // 1. Decode hex inputs
    let message_bytes = string_to_bytes(message_hash)?;
    let private_key_bytes = string_to_bytes(private_key)?;

    // 2. Create Ethereum prefixed message
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message_bytes.len());
    let mut prefixed_message = Vec::with_capacity(prefix.len() + message_bytes.len());
    prefixed_message.extend_from_slice(prefix.as_bytes());
    prefixed_message.extend_from_slice(&message_bytes);

    // 3. Hash with Keccak256 (second hashing)
    let mut hasher = Keccak256::new();
    hasher.update(&prefixed_message);
    let final_hash = hasher.finalize();

    // 4. Create signing objects
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&private_key_bytes)?;
    let message = Message::from_digest_slice(&final_hash)?;

    // 5. Sign and split components
    let signature: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &secret_key);
    let (recovery_id, sig_bytes) = signature.serialize_compact();

    Ok(Signature::from_parts(
        sig_bytes[0..32].try_into()?,
        sig_bytes[32..64].try_into()?,
        &[27 + recovery_id.to_i32() as u8], // Ethereum-style V
    ))
}

pub fn is_expected_signer_on_signed_message_hash(
    message_hash: &str,
    signature: &Signature,
    expected_signer: &str,
) -> bool {
    let signer_address = signed_message_hash_to_signer_address(message_hash, signature);
    signer_address.to_lowercase() == expected_signer.to_lowercase()
}

pub fn signed_message_hash_to_signer_address(message_hash: &str, signature: &Signature) -> String {
    // Create a prefixed message
    let message_bytes = match string_to_bytes(message_hash) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Failed to decode message hash: {}", e);
            return String::new();
        }
    };

    // Create the Ethereum prefixed message
    let prefixed_message = create_ethereum_prefixed_message(&message_bytes);

    // Create a message object from the prefixed hash
    let msg = match Message::from_digest_slice(&prefixed_message) {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to create message from bytes: {}", e);
            return String::new();
        }
    };

    // Create a recoverable signature from v, r, s components
    let recovery_id = match RecoveryId::from_i32((signature.v().unwrap()[0] as i32) - 27) {
        Ok(id) => id,
        Err(_) => {
            log::error!(
                "Failed to create recovery ID from v: {}",
                signature.v().unwrap()[0]
            );
            return String::new();
        }
    };

    let mut combined_rs = [0u8; 64];
    combined_rs[0..32].copy_from_slice(&signature.r().unwrap());
    combined_rs[32..64].copy_from_slice(&signature.s().unwrap());

    let recoverable_sig = match RecoverableSignature::from_compact(&combined_rs, recovery_id) {
        Ok(sig) => sig,
        Err(e) => {
            log::error!("Failed to create recoverable signature: {}", e);
            return String::new();
        }
    };

    // Recover the public key
    let secp = Secp256k1::new();
    let public_key = match secp.recover_ecdsa(&msg, &recoverable_sig) {
        Ok(pk) => pk,
        Err(e) => {
            log::error!("Failed to recover public key from signature: {}", e);
            return String::new();
        }
    };

    // Convert public key to Ethereum address
    let signer_address = public_key_to_eth_address(&public_key);

    // Format address as hex string with "0x" prefix
    format!("0x{}", hex::encode(signer_address))
}

// Helper function
fn create_ethereum_prefixed_message(message_hash: &[u8]) -> [u8; 32] {
    let prefix = "\u{0019}Ethereum Signed Message:\n32";

    let mut hasher = Keccak::v256();
    let mut result = [0u8; 32];

    hasher.update(prefix.as_bytes());
    hasher.update(message_hash);
    hasher.finalize(&mut result);

    result
}

fn public_key_to_eth_address(public_key: &PublicKey) -> [u8; 20] {
    let serialized = public_key.serialize_uncompressed();
    let public_key_bytes = &serialized[1..];

    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(public_key_bytes);
    hasher.finalize(&mut hash);

    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..32]);
    address
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::signature::Signature;
    use std::error::Error;

    const PRIVATE_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const MESSAGE_HASH: &str = "0xf0cea2ffdb802c106aef2a032b01c7d271a454473709016c2e2c406097acdfd3";

    #[test]
    fn test_bytes_to_string() {
        // Test normal conversion
        let bytes = vec![0x01, 0x02, 0x03, 0x04, 0xff];
        assert_eq!(bytes_to_string(&bytes), "01020304ff");

        // Test empty bytes
        let empty_bytes: Vec<u8> = vec![];
        assert_eq!(bytes_to_string(&empty_bytes), "");
    }

    #[test]
    fn test_string_to_bytes() {
        // Test normal conversion
        assert_eq!(
            string_to_bytes("0102030400").unwrap(),
            vec![0x01, 0x02, 0x03, 0x04, 0x00]
        );

        // Test with '0x' prefix
        assert_eq!(
            string_to_bytes("0x0102030400").unwrap(),
            vec![0x01, 0x02, 0x03, 0x04, 0x00]
        );

        assert_eq!(string_to_bytes("").unwrap(), Vec::<u8>::new());
        assert!(string_to_bytes("0102030g").is_err());
        assert!(string_to_bytes("10203").is_err());
    }

    #[test]
    fn test_round_trip_signing() -> Result<(), Box<dyn Error>> {
        let mut hasher = sha3::Keccak256::new();
        hasher.update(MESSAGE_HASH.as_bytes());
        let hash = hasher.finalize();
        let message_hash = bytes_to_string(&hash);

        let signature = sign_message_hash_and_get_signature(&message_hash, PRIVATE_KEY)?;

        // Convert signature components to strings
        let r_str = bytes_to_string(&signature.r()?);
        let s_str = bytes_to_string(&signature.s()?);
        let v_str = bytes_to_string(&signature.v()?);

        // Convert back to bytes
        let r_bytes = string_to_bytes(&r_str)?;
        let s_bytes = string_to_bytes(&s_str)?;
        let v_bytes = string_to_bytes(&v_str)?;

        // Verify round trip consistency
        assert_eq!(r_bytes, signature.r()?);
        assert_eq!(s_bytes, signature.s()?);
        assert_eq!(v_bytes, signature.v()?);

        Ok(())
    }

    // This test ensures that the Signature struct can be properly constructed from parts
    #[test]
    fn test_signature_from_parts() {
        let r = [1u8; 32];
        let s = [2u8; 32];
        let v = [27u8; 1];

        let signature = Signature::from_parts(&r, &s, &v);

        assert_eq!(signature.r().unwrap(), r);
        assert_eq!(signature.s().unwrap(), s);
        assert_eq!(signature.v().unwrap(), v);
    }
    use crate::utils::credentials_utils;
    use secp256k1::{Message, Secp256k1, SecretKey};

    // Helper function
    fn create_test_signature(private_key: &str, message_hash: &str) -> Signature {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(private_key).unwrap()).unwrap();

        let message_bytes = string_to_bytes(message_hash).unwrap();

        let prefixed_message = create_ethereum_prefixed_message(&message_bytes);

        let message = Message::from_slice(&prefixed_message).unwrap();

        let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);

        // Convert to signature format with v = recovery_id + 27
        let (recovery_id, signature_bytes) = signature.serialize_compact();
        let v = recovery_id.to_i32() as u8 + 27;

        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&signature_bytes[0..32]);
        s.copy_from_slice(&signature_bytes[32..64]);

        // Create a signature object with a single v byte
        let mut v_array = [0u8; 1];
        v_array[0] = v;

        Signature::from_parts(&r.to_vec(), &s.to_vec(), &v_array.to_vec())
    }

    #[test]
    fn test_valid_signature() {
        // Get public address from private key for verification
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(PRIVATE_KEY).unwrap()).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let expected_address = public_key_to_eth_address(&public_key);
        let expected_address_hex = format!("0x{}", hex::encode(expected_address));

        // Create signature for the message
        let signature = create_test_signature(PRIVATE_KEY, MESSAGE_HASH);

        // Verify the signature
        let result = is_expected_signer_on_signed_message_hash(
            MESSAGE_HASH,
            &signature,
            &expected_address_hex,
        );

        assert!(result, "Valid signature should return true");
    }

    #[test]
    fn should_match_expected_signer() {
        let message_hash = "0xf0cea2ffdb802c106aef2a032b01c7d271a454473709016c2e2c406097acdfd3";
        let private_key = "0x6dacd24b3d49d0c50c555aa728c60a57aa08beb363e3a90cce2e4e5d327c6ee2";
        let address = credentials_utils::get_address(private_key);
        let signature = sign_message_hash_and_get_signature(message_hash, private_key).unwrap();

        let is_expected_signer =
            is_expected_signer_on_signed_message_hash(message_hash, &signature, &address);

        assert!(is_expected_signer);
    }

    #[test]
    fn test_invalid_signer() {
        // Wrong address - not the signer of our message
        let wrong_address = "0xd8da6bf26964af9d7eed9e03e53415d37aa96045"; // Random ETH address

        // Create signature for the message
        let signature = create_test_signature(PRIVATE_KEY, MESSAGE_HASH);

        // Verify the signature with wrong address
        let result =
            is_expected_signer_on_signed_message_hash(MESSAGE_HASH, &signature, wrong_address);

        assert!(!result, "Invalid signer should return false");
    }

    #[test]
    fn test_tampered_message() {
        // Get public address from private key for verification
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(PRIVATE_KEY).unwrap()).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let expected_address = public_key_to_eth_address(&public_key);
        let expected_address_hex = format!("0x{}", hex::encode(expected_address));

        // Create signature for the message
        let signature = create_test_signature(PRIVATE_KEY, MESSAGE_HASH);

        // Try to verify with a tampered message
        let tampered_message = "0x2234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let result = is_expected_signer_on_signed_message_hash(
            tampered_message,
            &signature,
            &expected_address_hex,
        );

        assert!(!result, "Tampered message should return false");
    }

    #[test]
    fn test_case_insensitive_address() {
        // Get public address from private key for verification
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hex::decode(PRIVATE_KEY).unwrap()).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let expected_address = public_key_to_eth_address(&public_key);
        let expected_address_hex = format!("0x{}", hex::encode(expected_address));

        // Create an uppercase version of the address
        let uppercase_address = expected_address_hex.to_uppercase();

        // Create signature for the message
        let signature = create_test_signature(PRIVATE_KEY, MESSAGE_HASH);

        // Verify with uppercase address
        let result =
            is_expected_signer_on_signed_message_hash(MESSAGE_HASH, &signature, &uppercase_address);

        assert!(
            result,
            "Case-insensitive address comparison should return true"
        );
    }

    #[test]
    fn test_invalid_message_format() {
        // Create an invalid message hash (not proper hex)
        let invalid_message = "0xZZZZ67890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let signature = Signature::from_parts(&[0; 32], &[0; 32], &[27, 1]);

        let address = "0x1234567890123456789012345678901234567890";

        let result =
            is_expected_signer_on_signed_message_hash(invalid_message, &signature, address);

        assert!(!result, "Invalid message format should return false");
    }

    #[test]
    fn test_invalid_recovery_id() {
        let signature = Signature::from_parts(&[1; 32], &[1; 32], &[255, 1]);

        let address = "0x1234567890123456789012345678901234567890";

        let result = is_expected_signer_on_signed_message_hash(MESSAGE_HASH, &signature, address);

        assert!(!result, "Invalid recovery ID should return false");
    }
}
