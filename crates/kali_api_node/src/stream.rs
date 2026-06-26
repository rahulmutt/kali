//! Node-style stream byte helpers.

/// A minimal namespace of stream-style byte helpers for Node compatibility.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeStream;

impl NodeStream {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Vec<u8> {
        bytes.into()
    }

    pub fn from_utf8(text: impl AsRef<str>) -> Vec<u8> {
        text.as_ref().as_bytes().to_vec()
    }

    pub fn concat(left: impl AsRef<[u8]>, right: impl AsRef<[u8]>) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(left.as_ref().len() + right.as_ref().len());
        bytes.extend_from_slice(left.as_ref());
        bytes.extend_from_slice(right.as_ref());
        bytes
    }

    pub fn concat_bytes(&self, left: impl AsRef<[u8]>, right: impl AsRef<[u8]>) -> Vec<u8> {
        Self::concat(left, right)
    }

    pub fn to_utf8(bytes: impl AsRef<[u8]>) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(bytes.as_ref().to_vec())
    }
}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod stream_tests;
