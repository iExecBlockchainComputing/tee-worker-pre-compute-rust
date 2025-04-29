use crate::utils::signature_utils::{bytes_to_string, string_to_bytes};
use hex::FromHexError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    value: String,
}

impl Signature {
    pub fn from_bytes(sign: &[u8]) -> Self {
        Self {
            value: bytes_to_string(sign),
        }
    }

    pub fn from_parts(r: &[u8], s: &[u8], v: &[u8]) -> Self {
        let mut full = Vec::with_capacity(65);
        full.extend_from_slice(r);
        full.extend_from_slice(s);
        full.extend_from_slice(v);
        Self::from_bytes(&full)
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn r(&self) -> Result<Vec<u8>, FromHexError> {
        let bytes = string_to_bytes(&self.value)?;
        if bytes.len() < 32 {
            return Err(FromHexError::InvalidStringLength);
        }
        Ok(bytes[0..32].to_vec())
    }

    pub fn s(&self) -> Result<Vec<u8>, FromHexError> {
        let bytes = string_to_bytes(&self.value)?;
        if bytes.len() < 64 {
            return Err(FromHexError::InvalidStringLength);
        }
        Ok(bytes[32..64].to_vec())
    }

    pub fn v(&self) -> Result<Vec<u8>, FromHexError> {
        let bytes = string_to_bytes(&self.value)?;
        if bytes.len() < 65 {
            return Err(FromHexError::InvalidStringLength);
        }
        Ok(vec![bytes[64]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_valid() -> Result<(), FromHexError> {
        let valid_signature = "1b0b90d9f17a30d42492c8a2f98a24374600729a98d4e0b663a44ed48b589cab0e445eec300245e590150c7d88340d902c27e0d8673f3257cb8393f647d6c75c1b";

        let sign = Signature {
            value: valid_signature.to_string(),
        };
        let r = sign.r()?;
        let s = sign.s()?;
        let v = sign.v()?;
        let sign2 = Signature::from_parts(&r, &s, &v);

        assert_eq!(sign, sign2);
        Ok(())
    }

    #[test]
    fn test_get_r_s_v_components() -> Result<(), FromHexError> {
        let valid_signature = "1b0b90d9f17a30d42492c8a2f98a24374600729a98d4e0b663a44ed48b589cab0e445eec300245e590150c7d88340d902c27e0d8673f3257cb8393f647d6c75c1b";

        let sign = Signature {
            value: valid_signature.to_string(),
        };

        // Check the R component
        let r = sign.r()?;
        let r_hex = bytes_to_string(&r);
        assert_eq!(
            "1b0b90d9f17a30d42492c8a2f98a24374600729a98d4e0b663a44ed48b589cab",
            r_hex
        );

        // Check the S component
        let s = sign.s()?;
        let s_hex = bytes_to_string(&s);
        assert_eq!(
            "0e445eec300245e590150c7d88340d902c27e0d8673f3257cb8393f647d6c75c",
            s_hex
        );

        // Check the V component
        let v = sign.v()?;
        let v_hex = bytes_to_string(&v);
        assert_eq!("1b", v_hex);

        Ok(())
    }

    #[test]
    fn test_from_bytes() {
        let valid_signature = "1b0b90d9f17a30d42492c8a2f98a24374600729a98d4e0b663a44ed48b589cab0e445eec300245e590150c7d88340d902c27e0d8673f3257cb8393f647d6c75c1b";

        // Convert valid_signature to bytes
        let bytes = match hex::decode(valid_signature) {
            Ok(b) => b,
            Err(_) => panic!("Failed to decode hex string"),
        };

        let sign = Signature::from_bytes(&bytes);

        assert_eq!(valid_signature, sign.value);
    }

    #[test]
    fn test_from_parts() -> Result<(), FromHexError> {
        let valid_signature = "1b0b90d9f17a30d42492c8a2f98a24374600729a98d4e0b663a44ed48b589cab0e445eec300245e590150c7d88340d902c27e0d8673f3257cb8393f647d6c75c1b";

        let sign = Signature {
            value: valid_signature.to_string(),
        };
        let r = sign.r()?;
        let s = sign.s()?;
        let v = sign.v()?;

        let reconstructed = Signature::from_parts(&r, &s, &v);

        assert_eq!(valid_signature, reconstructed.value);
        Ok(())
    }

    #[test]
    fn test_invalid_hex_string() {
        let invalid_signature = "invalid hex string";

        let sign = Signature {
            value: invalid_signature.to_string(),
        };

        assert!(sign.r().is_err());
        assert!(sign.s().is_err());
        assert!(sign.v().is_err());
    }

    #[test]
    fn test_short_signature() {
        let short_signature =
            "1b0b90d9f17a30d42492c8a2f98a24374600729a98d4e0b663a44ed48b589cab0e445eec";

        let sign = Signature {
            value: short_signature.to_string(),
        };

        let r_result = sign.r();
        assert!(r_result.is_ok()); // R should be extractable

        let s_result = sign.s();
        assert!(s_result.is_err()); // S should fail (not enough bytes)

        let v_result = sign.v();
        assert!(v_result.is_err()); // V should fail (not enough bytes)
    }
}
