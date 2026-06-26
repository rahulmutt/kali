//! URL parsing and resolution helpers (Node.js `url` module surface).

use url::Url;

/// Parse a URL string using the shared support library's URL parser.
pub fn parse_url(input: &str) -> Result<Url, url::ParseError> {
    Url::parse(input)
}

/// Resolve a URL against a base URL string.
pub fn resolve_url(base: &str, input: &str) -> Result<Url, url::ParseError> {
    Url::parse(base)?.join(input)
}

/// Namespace-style wrapper for URL helpers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeUrl;

impl NodeUrl {
    pub fn parse(input: impl AsRef<str>) -> Result<Url, url::ParseError> {
        parse_url(input.as_ref())
    }

    pub fn resolve(base: impl AsRef<str>, input: impl AsRef<str>) -> Result<Url, url::ParseError> {
        resolve_url(base.as_ref(), input.as_ref())
    }
}
