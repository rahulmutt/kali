//! Node.js `crypto` module compatibility helpers.

use getrandom::fill as fill_random_bytes;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha384, Sha512};

/// Compute a SHA-256 digest as a lowercase hex string.
pub fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(bytes.as_ref());
    format!("{:x}", digest)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeCryptoError {
    message: String,
}

impl NodeCryptoError {
    fn unsupported_algorithm(algorithm: &str) -> Self {
        Self {
            message: format!("unsupported Node crypto algorithm '{}'", algorithm),
        }
    }

    fn invalid_key_length(algorithm: &str, error: impl std::fmt::Display) -> Self {
        Self {
            message: format!("failed to initialize {} HMAC: {}", algorithm, error),
        }
    }
}

impl std::fmt::Display for NodeCryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NodeCryptoError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeDigestAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl NodeDigestAlgorithm {
    fn parse(algorithm: impl AsRef<str>) -> Result<Self, NodeCryptoError> {
        match algorithm.as_ref().to_ascii_lowercase().as_str() {
            "sha256" => Ok(Self::Sha256),
            "sha384" => Ok(Self::Sha384),
            "sha512" => Ok(Self::Sha512),
            other => Err(NodeCryptoError::unsupported_algorithm(other)),
        }
    }

    fn digest_hex(self, bytes: impl AsRef<[u8]>) -> String {
        match self {
            Self::Sha256 => format!("{:x}", Sha256::digest(bytes.as_ref())),
            Self::Sha384 => format!("{:x}", Sha384::digest(bytes.as_ref())),
            Self::Sha512 => format!("{:x}", Sha512::digest(bytes.as_ref())),
        }
    }

    fn hmac_hex(
        self,
        key: impl AsRef<[u8]>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<String, NodeCryptoError> {
        match self {
            Self::Sha256 => {
                type HmacSha256 = Hmac<Sha256>;
                let mut mac = HmacSha256::new_from_slice(key.as_ref())
                    .map_err(|error| NodeCryptoError::invalid_key_length("sha256", error))?;
                mac.update(bytes.as_ref());
                Ok(format!("{:x}", mac.finalize().into_bytes()))
            }
            Self::Sha384 => {
                type HmacSha384 = Hmac<Sha384>;
                let mut mac = HmacSha384::new_from_slice(key.as_ref())
                    .map_err(|error| NodeCryptoError::invalid_key_length("sha384", error))?;
                mac.update(bytes.as_ref());
                Ok(format!("{:x}", mac.finalize().into_bytes()))
            }
            Self::Sha512 => {
                type HmacSha512 = Hmac<Sha512>;
                let mut mac = HmacSha512::new_from_slice(key.as_ref())
                    .map_err(|error| NodeCryptoError::invalid_key_length("sha512", error))?;
                mac.update(bytes.as_ref());
                Ok(format!("{:x}", mac.finalize().into_bytes()))
            }
        }
    }
}

/// Namespace-style projection of the common Node crypto helpers.
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeCrypto;

impl NodeCrypto {
    pub fn create_hash(
        algorithm: impl AsRef<str>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<String, NodeCryptoError> {
        NodeDigestAlgorithm::parse(algorithm).map(|algo| algo.digest_hex(bytes))
    }

    pub fn create_hmac(
        algorithm: impl AsRef<str>,
        key: impl AsRef<[u8]>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<String, NodeCryptoError> {
        NodeDigestAlgorithm::parse(algorithm)?.hmac_hex(key, bytes)
    }

    pub fn random_bytes(length: usize) -> Result<Vec<u8>, getrandom::Error> {
        random_bytes(length)
    }

    pub fn random_uuid_v4() -> Result<String, getrandom::Error> {
        random_uuid_v4()
    }
}

/// Return cryptographically random bytes.
pub fn random_bytes(length: usize) -> Result<Vec<u8>, getrandom::Error> {
    let mut bytes = vec![0u8; length];
    fill_random_bytes(&mut bytes)?;
    Ok(bytes)
}

/// Return a random UUIDv4 string.
pub fn random_uuid_v4() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    fill_random_bytes(&mut bytes)?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ))
}

#[cfg(test)]
#[path = "crypto_tests.rs"]
mod crypto_tests;
