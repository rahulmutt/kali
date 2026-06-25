//! Web Crypto API helpers: randomness, UUID, and digest support.

use std::fmt::{self, Write as _};

use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

/// Fill the provided buffer with OS randomness for `crypto.getRandomValues()`.
pub fn fill_random_values(buffer: &mut [u8]) -> Result<(), getrandom::Error> {
    getrandom::fill(buffer)
}

/// Generate a v4 UUID string for `crypto.randomUUID()`-style calls.
pub fn random_uuid() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    fill_random_values(&mut bytes)?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let mut uuid = String::with_capacity(36);
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            uuid.push('-');
        }
        write!(&mut uuid, "{:02x}", byte).expect("writing to a String cannot fail");
    }

    Ok(uuid)
}

/// Errors returned by the deterministic Web Crypto helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebCryptoError {
    UnsupportedDigestAlgorithm(String),
}

impl fmt::Display for WebCryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDigestAlgorithm(algorithm) => {
                write!(f, "unsupported Web Crypto digest algorithm '{algorithm}'")
            }
        }
    }
}

impl std::error::Error for WebCryptoError {}

fn canonicalize_digest_algorithm(name: &str) -> String {
    name.chars()
        .filter(|character| {
            !character.is_ascii_whitespace() && *character != '-' && *character != '_'
        })
        .flat_map(char::to_uppercase)
        .collect()
}

/// Deterministic Web Crypto facade for the shared randomness and digest subset.
#[derive(Clone, Copy, Debug, Default)]
pub struct Crypto;

impl Crypto {
    /// Fill the provided buffer with randomness for `crypto.getRandomValues()`.
    pub fn get_random_values(&self, buffer: &mut [u8]) -> Result<(), getrandom::Error> {
        fill_random_values(buffer)
    }

    /// Generate a v4 UUID string for `crypto.randomUUID()`-style calls.
    pub fn random_uuid(&self) -> Result<String, getrandom::Error> {
        random_uuid()
    }

    /// Return the deterministic `subtle` helper namespace.
    pub fn subtle(&self) -> SubtleCrypto {
        SubtleCrypto
    }
}

/// Deterministic Web Crypto `subtle` facade for digest support.
#[derive(Clone, Copy, Debug, Default)]
pub struct SubtleCrypto;

impl SubtleCrypto {
    /// Compute a deterministic digest for the provided payload.
    pub fn digest(
        &self,
        algorithm: impl AsRef<str>,
        data: impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, WebCryptoError> {
        let algorithm_name = algorithm.as_ref();
        let normalized = canonicalize_digest_algorithm(algorithm_name);

        match normalized.as_str() {
            "SHA1" => Ok(Sha1::digest(data.as_ref()).to_vec()),
            "SHA224" => Ok(Sha224::digest(data.as_ref()).to_vec()),
            "SHA256" => Ok(Sha256::digest(data.as_ref()).to_vec()),
            "SHA384" => Ok(Sha384::digest(data.as_ref()).to_vec()),
            "SHA512" => Ok(Sha512::digest(data.as_ref()).to_vec()),
            _ => Err(WebCryptoError::UnsupportedDigestAlgorithm(
                algorithm_name.trim().to_string(),
            )),
        }
    }
}

/// Return the shared deterministic Web Crypto facade.
pub fn crypto() -> Crypto {
    Crypto
}

#[cfg(test)]
#[path = "crypto_tests.rs"]
mod crypto_tests;
