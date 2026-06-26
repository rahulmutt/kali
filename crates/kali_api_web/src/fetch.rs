//! Web `fetch` API family: `Headers`, `Request`, `Response`, and `fetch`.

use ::url::{ParseError as UrlParseError, Url};
use std::sync::{Arc, Mutex};

fn normalize_header_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

/// A deterministic in-memory Web `Headers` baseline.
#[derive(Clone, Debug, Default)]
pub struct Headers {
    entries: Arc<Mutex<Vec<(String, String)>>>,
}

impl Headers {
    /// Create an empty header bag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a header while preserving insertion order.
    pub fn append(&self, name: impl Into<String>, value: impl Into<String>) {
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .push((normalize_header_name(&name.into()), value.into()));
    }

    /// Replace all matching headers with a single value.
    pub fn set(&self, name: impl Into<String>, value: impl Into<String>) {
        let name = normalize_header_name(&name.into());
        self.delete(&name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .push((name, value.into()));
    }

    /// Return whether a matching header exists.
    pub fn has(&self, name: &str) -> bool {
        let name = normalize_header_name(name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .iter()
            .any(|(entry_name, _)| entry_name == &name)
    }

    /// Return the first matching header value, if present.
    pub fn get(&self, name: &str) -> Option<String> {
        let name = normalize_header_name(name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .iter()
            .find(|(entry_name, _)| entry_name == &name)
            .map(|(_, value)| value.clone())
    }

    /// Remove all matching headers.
    pub fn delete(&self, name: &str) {
        let name = normalize_header_name(name);
        self.entries
            .lock()
            .expect("headers mutex poisoned")
            .retain(|(entry_name, _)| entry_name != &name);
    }

    /// Return a deterministic snapshot of the current entries.
    pub fn entries(&self) -> Vec<(String, String)> {
        self.entries.lock().expect("headers mutex poisoned").clone()
    }
}

/// A deterministic in-memory Web `Request` baseline.
#[derive(Clone, Debug)]
pub struct Request {
    url: Url,
    method: String,
    headers: Headers,
    body: Arc<[u8]>,
}

impl Request {
    /// Create a GET request with no body or headers.
    pub fn new(url: impl AsRef<str>) -> Result<Self, UrlParseError> {
        Self::with_parts(url, "GET", Headers::new(), [])
    }

    /// Create a request with explicit method, headers, and body.
    pub fn with_parts(
        url: impl AsRef<str>,
        method: impl Into<String>,
        headers: Headers,
        body: impl AsRef<[u8]>,
    ) -> Result<Self, UrlParseError> {
        Ok(Self {
            url: Url::parse(url.as_ref())?,
            method: method.into().to_ascii_uppercase(),
            headers,
            body: Arc::from(body.as_ref().to_vec()),
        })
    }

    /// Return the request URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Return the HTTP method.
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Return the request headers.
    pub fn headers(&self) -> Headers {
        self.headers.clone()
    }

    /// Return the request body bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Decode the request body as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.to_vec())
    }
}

/// A deterministic in-memory Web `Response` baseline.
#[derive(Clone, Debug)]
pub struct Response {
    url: Url,
    status: u16,
    status_text: String,
    headers: Headers,
    body: Arc<[u8]>,
}

impl Response {
    /// Create a basic 200 OK response with no associated URL.
    pub fn new(body: impl AsRef<[u8]>) -> Result<Self, UrlParseError> {
        Self::with_parts("https://example.invalid/", 200, "OK", Headers::new(), body)
    }

    /// Create a response with explicit status, headers, and body.
    pub fn with_parts(
        url: impl AsRef<str>,
        status: u16,
        status_text: impl Into<String>,
        headers: Headers,
        body: impl AsRef<[u8]>,
    ) -> Result<Self, UrlParseError> {
        Ok(Self {
            url: Url::parse(url.as_ref())?,
            status,
            status_text: status_text.into(),
            headers,
            body: Arc::from(body.as_ref().to_vec()),
        })
    }

    /// Return the response URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Return the HTTP status.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Return the HTTP status text.
    pub fn status_text(&self) -> &str {
        &self.status_text
    }

    /// Return whether the status is in the 2xx range.
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Return the response headers.
    pub fn headers(&self) -> Headers {
        self.headers.clone()
    }

    /// Return the response body bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Decode the response body as UTF-8 text.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.to_vec())
    }

    /// Create a response that deterministically echoes the request payload.
    pub fn from_request(request: &Request) -> Self {
        Self {
            url: request.url.clone(),
            status: 200,
            status_text: "OK".to_string(),
            headers: request.headers.clone(),
            body: Arc::from(request.body.as_ref().to_vec()),
        }
    }
}

/// Return a deterministic fetch result for the provided request.
pub fn fetch(request: &Request) -> Response {
    Response::from_request(request)
}

#[cfg(test)]
#[path = "fetch_tests.rs"]
mod fetch_tests;
