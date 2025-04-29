use hex;
use log::error;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use std::str::FromStr;

pub fn get_address(private_key: &str) -> String {
    if is_valid_private_key(private_key) {
        create_address_from_private_key(private_key).unwrap_or_else(|e| {
            error!("Error creating address from private key: {}", e);
            String::new()
        })
    } else {
        let key_length = if !private_key.is_empty() {
            private_key.len()
        } else {
            0
        };
        error!(
            "Cannot get address from private key [privateKeyLength:{}]",
            key_length
        );
        String::new()
    }
}

fn is_valid_private_key(private_key: &str) -> bool {
    let key = if let Some(stripped) = private_key.strip_prefix("0x") {
        stripped
    } else {
        error!("Private key must start with '0x'.");
        return false;
    };

    if hex::decode(key).is_err() {
        error!("Private key is not valid hex.");
        return false;
    }

    if key.len() != 64 {
        error!("Private key length is {}, expected 64.", key.len());
        return false;
    }

    if key.chars().all(|c| c == '0') {
        error!("Private key cannot be all zeros.");
        return false;
    }

    match SecretKey::from_str(key) {
        Ok(_) => true,
        Err(e) => {
            error!("Failed to parse SecretKey: {}", e);
            false
        }
    }
}

fn create_address_from_private_key(
    private_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let key = private_key.strip_prefix("0x").unwrap_or(private_key);
    let secret_key = SecretKey::from_str(key)?;
    let secp = Secp256k1::new();

    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let serialized_pub_key = public_key.serialize_uncompressed();

    let mut hasher = Keccak256::new();
    hasher.update(&serialized_pub_key[1..]);
    let hash = hasher.finalize();

    let address = format!("0x{}", hex::encode(&hash[12..32]));

    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    const VALID_PRIVATE_KEY: &str =
        "0x0010000000000000000000000000000000000000000000000000000000000001";

    #[test]
    fn test_get_address() {
        assert_eq!(
            get_address(VALID_PRIVATE_KEY),
            "0xae2e9def8b48ba414fc57614f4683f008572226c"
        );

        // Test with invalid private key
        let invalid_key = "invalid";
        assert_eq!(get_address(invalid_key), "");
    }

    #[test]
    fn test_is_valid_private_key() {
        assert!(is_valid_private_key(VALID_PRIVATE_KEY));

        let valid_key = "0010000000000000000000000000000000000000000000000000000000000001";
        assert!(!is_valid_private_key(valid_key));

        let invalid_length = "0x0123";
        assert!(!is_valid_private_key(invalid_length));

        let invalid_chars = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg";
        assert!(!is_valid_private_key(invalid_chars));

        let all_zeros = "0x0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!is_valid_private_key(all_zeros));
    }
}
