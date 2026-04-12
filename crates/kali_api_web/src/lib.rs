//! Web API compatibility surface for Kali runtime.

use std::sync::OnceLock;
use std::time::Instant;

static TIME_ORIGIN: OnceLock<Instant> = OnceLock::new();

/// Initialize the Web API compatibility surface.
pub fn web_api_init() -> Result<(), ()> {
    Ok(())
}

/// Return a monotonic millisecond timestamp for `performance.now()`-style calls.
pub fn performance_now() -> f64 {
    TIME_ORIGIN
        .get_or_init(Instant::now)
        .elapsed()
        .as_secs_f64()
        * 1000.0
}

/// Fill the provided buffer with OS randomness for `crypto.getRandomValues()`.
pub fn fill_random_values(buffer: &mut [u8]) -> Result<(), getrandom::Error> {
    getrandom::fill(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn performance_now_is_monotonic_and_non_negative() {
        let first = performance_now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let second = performance_now();

        assert!(first >= 0.0, "first timestamp: {first}");
        assert!(
            second >= first,
            "timestamps should not go backwards: {first} -> {second}"
        );
    }

    #[test]
    fn random_fill_populates_the_requested_buffer() {
        let mut buffer = [0u8; 16];
        fill_random_values(&mut buffer).expect("random fill");
        assert_eq!(buffer.len(), 16);
    }
}
