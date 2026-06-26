//! Node-style byte buffer (`Buffer`).

use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Lightweight buffer wrapper for Node-style byte handling.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodeBuffer(Vec<u8>);

impl NodeBuffer {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn from_utf8(text: impl AsRef<str>) -> Self {
        Self(text.as_ref().as_bytes().to_vec())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn to_base64(&self) -> String {
        STANDARD.encode(&self.0)
    }

    pub fn from_base64(text: impl AsRef<str>) -> Result<Self, base64::DecodeError> {
        STANDARD.decode(text.as_ref()).map(Self)
    }

    pub fn to_hex(&self) -> String {
        use std::fmt::Write as _;

        let mut output = String::with_capacity(self.0.len() * 2);
        for byte in &self.0 {
            write!(&mut output, "{:02x}", byte).expect("hex formatting should be infallible");
        }
        output
    }

    pub fn from_hex(text: impl AsRef<str>) -> Result<Self, String> {
        let text = text.as_ref();
        if text.len() % 2 != 0 {
            return Err("hex input must contain an even number of digits".to_string());
        }

        let mut bytes = Vec::with_capacity(text.len() / 2);
        for chunk in text.as_bytes().chunks_exact(2) {
            let hi = hex_digit(chunk[0])
                .ok_or_else(|| format!("invalid hex digit '{}'", chunk[0] as char))?;
            let lo = hex_digit(chunk[1])
                .ok_or_else(|| format!("invalid hex digit '{}'", chunk[1] as char))?;
            bytes.push((hi << 4) | lo);
        }

        Ok(Self(bytes))
    }

    pub fn to_utf8(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.0.clone())
    }
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "buffer_tests.rs"]
mod buffer_tests;
