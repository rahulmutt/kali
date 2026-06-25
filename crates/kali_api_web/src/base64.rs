//! Base64 encoding/decoding helpers (`btoa`, `atob`).

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Error returned by the deterministic base64 helpers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Base64Error {
    message: String,
}

impl Base64Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Base64Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Base64Error {}

/// Encode a binary string as base64 using the browser's `btoa` semantics.
pub fn btoa(input: &str) -> Result<String, Base64Error> {
    let mut bytes = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let code = ch as u32;
        if code > 0xFF {
            return Err(Base64Error::new(
                "The string to be encoded contains characters outside of the Latin1 range.",
            ));
        }
        bytes.push(code as u8);
    }

    Ok(encode_base64(&bytes))
}

/// Decode a base64 string using the browser's `atob` semantics.
pub fn atob(input: &str) -> Result<String, Base64Error> {
    let mut normalized: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    match normalized.len() % 4 {
        0 => {}
        1 => {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ))
        }
        2 => normalized.push_str("=="),
        3 => normalized.push('='),
        _ => {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ))
        }
    }

    let decoded = decode_base64(&normalized)?;
    Ok(decoded.into_iter().map(char::from).collect())
}

fn encode_base64(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);

        output.push(char::from(BASE64_ALPHABET[(first >> 2) as usize]));
        output.push(char::from(
            BASE64_ALPHABET[((first & 0b0000_0011) << 4 | (second >> 4)) as usize],
        ));
        if chunk.len() > 1 {
            output.push(char::from(
                BASE64_ALPHABET[((second & 0b0000_1111) << 2 | (third >> 6)) as usize],
            ));
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(char::from(BASE64_ALPHABET[(third & 0b0011_1111) as usize]));
        } else {
            output.push('=');
        }
    }
    output
}

fn decode_base64(input: &str) -> Result<Vec<u8>, Base64Error> {
    let bytes = input.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err(Base64Error::new(
            "The string to be decoded is not correctly encoded.",
        ));
    }

    let mut output = Vec::with_capacity((bytes.len() / 4) * 3);
    let chunk_count = bytes.len() / 4;
    for (chunk_index, chunk) in bytes.chunks(4).enumerate() {
        let mut values = [0u8; 4];
        let mut padding = 0usize;

        for (index, byte) in chunk.iter().copied().enumerate() {
            if byte == b'=' {
                padding += 1;
                values[index] = 0;
                continue;
            }

            if padding > 0 {
                return Err(Base64Error::new(
                    "The string to be decoded is not correctly encoded.",
                ));
            }

            values[index] = decode_base64_value(byte).ok_or_else(|| {
                Base64Error::new("The string to be decoded contains invalid base64 characters.")
            })?;
        }

        if padding > 2 {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ));
        }
        if padding > 0 && chunk_index + 1 != chunk_count {
            return Err(Base64Error::new(
                "The string to be decoded is not correctly encoded.",
            ));
        }

        output.push((values[0] << 2) | (values[1] >> 4));
        if padding < 2 {
            output.push((values[1] << 4) | (values[2] >> 2));
        }
        if padding == 0 {
            output.push((values[2] << 6) | values[3]);
        }
    }

    Ok(output)
}

fn decode_base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
#[path = "base64_tests.rs"]
mod base64_tests;
