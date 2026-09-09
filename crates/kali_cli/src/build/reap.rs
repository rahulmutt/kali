//! Bounding the incremental cache directory.
//!
//! The compiler-build fingerprint in the cache key namespaces entries per
//! build, so every rebuild orphans a whole namespace and the directory grows
//! without bound. Entries are small (~7 KB), which makes FILE COUNT the acute
//! resource rather than bytes.
//!
//! This is eviction by *insertion* age, not by use: `std::fs` cannot set an
//! mtime, so there is no touch-on-hit and no dependency is added to get one. A
//! hot entry older than `MAX_AGE` is evicted and recompiled once. Calling it an
//! LRU would be wrong.
//!
//! Design: `docs/superpowers/specs/2026-09-09-incremental-cache-compiler-identity-design.md` §3.4.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

const MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MAX_ENTRIES: usize = 4096;
const MAX_BYTES: u64 = 128 * 1024 * 1024;

/// Sweeping on every write costs an O(N) `readdir` per compile; sweeping once
/// per process lets a single long test binary outrun the budget. This is the
/// compromise, and the number is arbitrary within an order of magnitude.
const SWEEP_INTERVAL: usize = 256;

static WRITES_SINCE_SWEEP: AtomicUsize = AtomicUsize::new(0);

/// Sweep on the first cache write in this process, and every
/// `SWEEP_INTERVAL` writes thereafter.
pub(crate) fn maybe_reap(cache_dir: &Path) {
    let previous = WRITES_SINCE_SWEEP.fetch_add(1, Ordering::Relaxed);
    if previous % SWEEP_INTERVAL == 0 {
        reap(cache_dir);
    }
}

/// Best-effort. Every IO error is ignored: a reaper that can fail a build is
/// worse than a full disk.
///
/// Needs no locking. A concurrent reader whose entry is deleted between path
/// computation and `fs::read` gets `ErrorKind::NotFound`, which the read path
/// already treats as a miss and falls through to a real compile (the
/// `NotFound` arm in `crates/kali_cli/src/build/compile.rs`). The worst
/// outcome of a race is a recompile.
pub(crate) fn reap(cache_dir: &Path) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    let now = SystemTime::now();
    let mut kept: Vec<(SystemTime, u64, PathBuf)> = Vec::new();

    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        let path = entry.path();
        // A file with no readable mtime is treated as brand new rather than
        // ancient, so an unreadable clock cannot cause mass deletion.
        let modified = metadata.modified().unwrap_or(now);
        let too_old = now
            .duration_since(modified)
            .map(|age| age > MAX_AGE)
            .unwrap_or(false);
        if too_old {
            let _ = fs::remove_file(&path);
            continue;
        }
        kept.push((modified, metadata.len(), path));
    }

    // Oldest first: those are evicted first when over budget.
    kept.sort_by(|left, right| left.0.cmp(&right.0));

    let mut count = kept.len();
    let mut bytes: u64 = kept.iter().map(|(_, len, _)| *len).sum();

    for (_, len, path) in &kept {
        if count <= MAX_ENTRIES && bytes <= MAX_BYTES {
            break;
        }
        if fs::remove_file(path).is_ok() {
            count -= 1;
            bytes = bytes.saturating_sub(*len);
        }
    }
}

#[cfg(test)]
#[path = "reap_tests.rs"]
mod reap_tests;
