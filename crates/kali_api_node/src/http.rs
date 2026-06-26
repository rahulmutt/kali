//! Node-style HTTP client helpers.

/// Node-style HTTP error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeHttpError {
    message: String,
}

impl NodeHttpError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for NodeHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NodeHttpError {}

/// Minimal Node-style HTTP client helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeHttp;

/// Minimal Node-style HTTP response wrapper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeHttpResponse {
    status: u16,
    body: Vec<u8>,
}

impl NodeHttpResponse {
    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
}

impl NodeHttp {
    pub fn get(url: impl AsRef<str>) -> Result<NodeHttpResponse, NodeHttpError> {
        let response = reqwest::blocking::get(url.as_ref())
            .and_then(|resp| resp.error_for_status())
            .map_err(|error| {
                NodeHttpError::new(format!("failed to GET '{}': {}", url.as_ref(), error))
            })?;

        let status = response.status().as_u16();
        let body = response
            .bytes()
            .map_err(|error| {
                NodeHttpError::new(format!(
                    "failed to read '{}' response body: {}",
                    url.as_ref(),
                    error
                ))
            })?
            .to_vec();
        Ok(NodeHttpResponse { status, body })
    }

    pub fn request_get(&self, url: impl AsRef<str>) -> Result<NodeHttpResponse, NodeHttpError> {
        Self::get(url)
    }
}

#[cfg(test)]
#[path = "http_tests.rs"]
mod http_tests;
