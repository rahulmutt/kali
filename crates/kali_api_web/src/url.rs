//! Web URL API family: `UrlMutationError`, `parse_url`, `resolve_url`, `URL`, `URLSearchParams`.

use std::{
    fmt,
    sync::{Arc, Mutex},
};
use url::{form_urlencoded, Url};

/// Errors returned when mutating a deterministic URL baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UrlMutationError {
    InvalidProtocol,
    InvalidHost,
    InvalidPort,
}

impl fmt::Display for UrlMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidProtocol => "invalid URL protocol",
            Self::InvalidHost => "invalid URL host",
            Self::InvalidPort => "invalid URL port",
        };
        f.write_str(message)
    }
}

impl std::error::Error for UrlMutationError {}

/// Parse a URL string using the shared support library's URL parser.
pub fn parse_url(input: &str) -> Result<Url, url::ParseError> {
    Url::parse(input)
}

/// Resolve a URL against a base URL string.
pub fn resolve_url(base: &str, input: &str) -> Result<Url, url::ParseError> {
    Url::parse(base)?.join(input)
}

/// A deterministic in-memory Web `URL` baseline.
#[derive(Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct URL {
    url: Url,
}

impl PartialEq for URL {
    fn eq(&self, other: &Self) -> bool {
        self.url.as_str() == other.url.as_str()
    }
}

impl Eq for URL {}

impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl URL {
    /// Create a new URL from an absolute or base-resolved URL string.
    pub fn new(input: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Self::parse(input)
    }

    /// Parse a URL string into the deterministic baseline wrapper.
    pub fn parse(input: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Url::parse(input.as_ref()).map(Self::from_url)
    }

    /// Resolve a relative URL against a base URL string.
    pub fn resolve(base: impl AsRef<str>, input: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Url::parse(base.as_ref())?
            .join(input.as_ref())
            .map(Self::from_url)
    }

    /// Wrap an existing parsed URL value.
    pub fn from_url(url: Url) -> Self {
        Self { url }
    }

    /// Unwrap the inner parsed URL value.
    pub fn into_inner(self) -> Url {
        self.url
    }

    /// Return the underlying parsed URL.
    pub fn as_url(&self) -> &Url {
        &self.url
    }

    /// Return the serialized URL string.
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    /// Return the canonical URL href string.
    pub fn href(&self) -> &str {
        self.as_str()
    }

    /// Return the current protocol with the trailing `:` suffix.
    pub fn protocol(&self) -> String {
        format!("{}:", self.url.scheme())
    }

    /// Update the protocol/scheme component.
    pub fn set_protocol(&mut self, protocol: impl AsRef<str>) -> Result<(), UrlMutationError> {
        let protocol = protocol.as_ref().trim_end_matches(':');
        self.url
            .set_scheme(protocol)
            .map_err(|_| UrlMutationError::InvalidProtocol)
    }

    /// Return the current pathname component.
    pub fn pathname(&self) -> &str {
        self.url.path()
    }

    /// Update the pathname component.
    pub fn set_pathname(&mut self, pathname: impl AsRef<str>) {
        self.url.set_path(pathname.as_ref());
    }

    /// Return the current query string with the leading `?`, if present.
    pub fn search(&self) -> String {
        self.url
            .query()
            .map(|query| format!("?{}", query))
            .unwrap_or_default()
    }

    /// Update the query string.
    pub fn set_search(&mut self, search: impl AsRef<str>) {
        let search = search.as_ref().strip_prefix('?').unwrap_or(search.as_ref());
        self.url.set_query((!search.is_empty()).then_some(search));
    }

    /// Return the current fragment with the leading `#`, if present.
    pub fn hash(&self) -> String {
        self.url
            .fragment()
            .map(|fragment| format!("#{}", fragment))
            .unwrap_or_default()
    }

    /// Update the fragment component.
    pub fn set_hash(&mut self, hash: impl AsRef<str>) {
        let hash = hash.as_ref().strip_prefix('#').unwrap_or(hash.as_ref());
        self.url.set_fragment((!hash.is_empty()).then_some(hash));
    }

    /// Return the current host component, if any.
    pub fn host(&self) -> Option<&str> {
        self.url.host_str()
    }

    /// Update the host component.
    pub fn set_host(&mut self, host: impl AsRef<str>) -> Result<(), UrlMutationError> {
        self.url
            .set_host(Some(host.as_ref()))
            .map_err(|_| UrlMutationError::InvalidHost)
    }

    /// Return the current port component, if any.
    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    /// Update the port component.
    pub fn set_port(&mut self, port: Option<u16>) -> Result<(), UrlMutationError> {
        self.url
            .set_port(port)
            .map_err(|_| UrlMutationError::InvalidPort)
    }
}

/// A deterministic in-memory Web `URLSearchParams` baseline.
#[derive(Clone, Debug, Default)]
pub struct URLSearchParams {
    entries: Arc<Mutex<Vec<(String, String)>>>,
}

impl URLSearchParams {
    /// Create an empty parameter bag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a parameter bag from a query string.
    pub fn from_query(query: impl AsRef<str>) -> Self {
        let params = Self::new();
        for (name, value) in form_urlencoded::parse(query.as_ref().as_bytes()) {
            params.append(name.into_owned(), value.into_owned());
        }
        params
    }

    /// Append a parameter while preserving insertion order.
    pub fn append(&self, name: impl Into<String>, value: impl Into<String>) {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .push((name.into(), value.into()));
    }

    /// Replace all matching parameters with a single value.
    pub fn set(&self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        self.delete(&name);
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .push((name, value.into()));
    }

    /// Return whether a matching parameter exists.
    pub fn has(&self, name: &str) -> bool {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
            .any(|(entry_name, _)| entry_name == name)
    }

    /// Return the first matching value, if present.
    pub fn get(&self, name: &str) -> Option<String> {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
            .find(|(entry_name, _)| entry_name == name)
            .map(|(_, value)| value.clone())
    }

    /// Return all matching values in insertion order.
    pub fn get_all(&self, name: &str) -> Vec<String> {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
            .filter(|(entry_name, _)| entry_name == name)
            .map(|(_, value)| value.clone())
            .collect()
    }

    /// Remove all matching parameters.
    pub fn delete(&self, name: &str) {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .retain(|(entry_name, _)| entry_name != name);
    }

    /// Return a deterministic snapshot of the current entries.
    pub fn entries(&self) -> Vec<(String, String)> {
        self.entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .clone()
    }

    fn serialize(&self) -> String {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (name, value) in self
            .entries
            .lock()
            .expect("urlsearchparams mutex poisoned")
            .iter()
        {
            serializer.append_pair(name, value);
        }
        serializer.finish()
    }
}

impl fmt::Display for URLSearchParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.serialize())
    }
}

#[cfg(test)]
#[path = "url_tests.rs"]
mod url_tests;
