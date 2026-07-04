//! Shared test-only helpers for building process-wide-unique fixture paths.
//!
//! Uniqueness must NOT depend on wall-clock resolution: tests run
//! multi-threaded, and on platforms with a coarse `SystemTime` clock (e.g.
//! macOS) two concurrent calls can observe the same `as_nanos()` value,
//! collide on the same temp directory, and clobber each other's fixture
//! files. Appending a process-wide monotonic counter to each fixture slug
//! guarantees uniqueness independently of the wall-clock's resolution.

use std::sync::atomic::{AtomicU64, Ordering};

/// Returns a process-wide monotonic sequence number. Combined with a
/// fixture's existing pid + nanos-derived slug, this guarantees uniqueness
/// even when concurrent callers observe identical `SystemTime` readings.
pub(crate) fn unique_fixture_seq() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[test]
fn fixture_seqs_are_unique_under_concurrency() {
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    // Regression guard for a macOS-only CI flake: hammer the sequence
    // generator from several threads and assert every returned value is
    // distinct.
    const THREADS: usize = 8;
    const PER_THREAD: usize = 10_000;

    let seen = Arc::new(Mutex::new(HashSet::with_capacity(THREADS * PER_THREAD)));
    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let seen = Arc::clone(&seen);
            std::thread::spawn(move || {
                for _ in 0..PER_THREAD {
                    let value = unique_fixture_seq();
                    assert!(
                        seen.lock().expect("seen lock").insert(value),
                        "duplicate fixture seq"
                    );
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("fixture-seq worker thread panicked");
    }
    assert_eq!(seen.lock().expect("seen lock").len(), THREADS * PER_THREAD);
}
