//! Tests for the compiler-build fingerprint.
//!
//! These run inside a test binary, which is itself a compiler build, so
//! `compiler_build_fingerprint()` must succeed here for the same reason it must
//! succeed for the `kali` binary.

use super::*;

#[test]
fn fingerprint_is_available_and_stable_within_one_process() {
    let first =
        compiler_build_fingerprint().expect("a test binary must be able to identify itself");
    let second = compiler_build_fingerprint().expect("the second call must also succeed");

    assert_eq!(
        first, second,
        "the OnceLock must hand back exactly one value per process"
    );
}

#[test]
fn fingerprint_is_a_fixed_width_hex_string() {
    let fingerprint = compiler_build_fingerprint().expect("fingerprint must be available");

    assert_eq!(
        fingerprint.len(),
        FINGERPRINT_LEN,
        "the fingerprint is truncated to a fixed width so cache filenames stay tractable"
    );
    assert!(
        fingerprint.chars().all(|c| c.is_ascii_hexdigit()),
        "the fingerprint must be hex so it is safe in a filename, got {fingerprint:?}"
    );
}
