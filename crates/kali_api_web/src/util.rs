//! Utility helpers: initialization, text codec, structured-clone, and performance timer.

use std::{sync::OnceLock, time::Instant};

static TIME_ORIGIN: OnceLock<Instant> = OnceLock::new();

/// Initialize the Web API compatibility surface.
pub fn web_api_init() {}

/// Encode text as UTF-8 bytes for the Web baseline text encoder.
pub fn text_encode(input: &str) -> Vec<u8> {
    input.as_bytes().to_vec()
}

/// Decode UTF-8 bytes for the Web baseline text decoder.
pub fn text_decode(bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(bytes.to_vec())
}

/// Clone a support-library value using the host's ordinary `Clone` semantics.
pub fn structured_clone<T: Clone>(value: &T) -> T {
    value.clone()
}

/// Return a monotonic millisecond timestamp for `performance.now()`-style calls.
pub fn performance_now() -> f64 {
    TIME_ORIGIN
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs_f64()
        * 1000.0
}

#[cfg(test)]
#[path = "util_tests.rs"]
mod util_tests;
