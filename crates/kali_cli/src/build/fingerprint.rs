//! Compiler-build identity for the incremental cache key.
//!
//! The cache key in `compile.rs` USED TO be built entirely from *inputs* and
//! ended in a frozen `CARGO_PKG_VERSION`. Nothing in it identified the
//! compiler that produced the artifact, so a cached wasm file survived
//! arbitrary compiler-semantics changes and was served to a compiler that
//! would no longer produce it. This module supplies the missing
//! discriminator.
//!
//! Design: `docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md` §3.1.

use super::helpers::hex_encode;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;

/// Hex characters kept from the digest: long enough that a collision is not a
/// practical concern, short enough to keep cache filenames tractable.
pub(crate) const FINGERPRINT_LEN: usize = 16;

static FINGERPRINT: OnceLock<Option<String>> = OnceLock::new();

/// A discriminator for the *running compiler build*.
///
/// `None` means the compiler could not identify itself. Callers MUST fail
/// closed by declining the cache entirely — never by falling back to a value
/// that does not discriminate, which is the defect this module exists to fix.
pub(crate) fn compiler_build_fingerprint() -> Option<&'static str> {
    FINGERPRINT.get_or_init(compute_fingerprint).as_deref()
}

/// Hashes the running executable's identity.
///
/// `len` and `mtime` move on every relink — including a relink caused by a
/// change in a *dependency* crate such as `kali_optimize`, which is the case
/// that actually broke and the reason a `build.rs`-emitted constant would not
/// work: Cargo reruns a build script on its own package's files, not its
/// dependencies'.
///
/// `path` separates the `kali` binary from each in-process test binary. Those
/// are genuinely different compiler builds, each carrying an independently
/// linked copy of the compiler, so giving them separate namespaces is correct.
///
/// The binary's *contents* are deliberately not hashed: the debug binary is
/// over 500 MB, which costs a second or two per process.
fn compute_fingerprint() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let metadata = std::fs::metadata(&exe).ok()?;
    let mtime_nanos = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();

    let mut hasher = Sha256::new();
    hasher.update(exe.to_string_lossy().as_bytes());
    hasher.update(b"\0");
    hasher.update(metadata.len().to_le_bytes());
    hasher.update(mtime_nanos.to_le_bytes());

    let mut hex = hex_encode(hasher.finalize());
    hex.truncate(FINGERPRINT_LEN);
    Some(hex)
}

#[cfg(test)]
#[path = "fingerprint_tests.rs"]
mod fingerprint_tests;
