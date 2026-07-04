use super::*;
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

fn write_source_fixture(source: &str) -> PathBuf {
    write_source_fixture_with_extension(source, "ts")
}

/// Build a process-wide-unique directory name for a test source fixture.
///
/// Uniqueness must NOT depend on wall-clock resolution: tests run multi-threaded,
/// and on platforms with a coarse `SystemTime` clock (e.g. macOS) two concurrent
/// calls can observe the same `as_nanos()` value, collide on the same directory,
/// and clobber each other's `main.<ext>` — making one test read another's source.
fn unique_fixture_slug() -> String {
    // A process-wide monotonic counter guarantees uniqueness independently of the
    // wall-clock's resolution, so concurrent calls never collide even when their
    // `as_nanos()` readings are identical.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    format!("kali-sandbox-{unique}-{}-{seq}", std::process::id())
}

fn write_source_fixture_with_extension(source: &str, extension: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(unique_fixture_slug());
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(format!("main.{extension}"));
    fs::write(&path, source).expect("write source fixture");
    path
}

#[test]
fn fixture_slugs_are_unique_under_concurrency() {
    use std::collections::HashSet;

    // Regression guard for a macOS-only CI flake: when the fixture-directory slug
    // derived uniqueness solely from `SystemTime::now().as_nanos()` + pid, two
    // concurrent test threads could generate the SAME slug, clobber each other's
    // fixture file, and cause a test to infer effects from the wrong source. Hammer
    // the generator from several threads and assert every slug is distinct.
    const THREADS: usize = 8;
    const PER_THREAD: usize = 10_000;

    let seen = Arc::new(Mutex::new(HashSet::with_capacity(THREADS * PER_THREAD)));
    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let seen = Arc::clone(&seen);
            std::thread::spawn(move || {
                for _ in 0..PER_THREAD {
                    let slug = unique_fixture_slug();
                    assert!(
                        seen.lock().expect("seen lock").insert(slug.clone()),
                        "duplicate fixture slug generated: {slug}"
                    );
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("fixture-slug worker thread panicked");
    }
    assert_eq!(seen.lock().expect("seen lock").len(), THREADS * PER_THREAD);
}

fn valid_policy() -> SandboxPolicy {
    SandboxPolicy {
        schema_version: 1,
        schema_uri: None,
        effects: EffectsPolicy {
            file_system: FileSystemPolicy {
                read: AccessRule::AllowList(vec!["/data/**".into()]),
                write: AccessRule::Deny(false),
            },
            network: NetworkPolicy {
                fetch: AccessRule::AllowList(vec!["https://example.com/**".into()]),
                connect: AccessRule::Deny(false),
                listen: AccessRule::Deny(false),
                max_connections: Some(4),
            },
            process: ProcessPolicy {
                spawn: AccessRule::Deny(false),
                env_read: AccessRule::AllowList(vec!["PATH".into()]),
                env_write: AccessRule::Deny(false),
            },
            timer: TimerPolicy {
                schedule: true,
                max_timeout_ms: Some(5_000),
                max_active_timers: Some(8),
            },
            eval: false,
            random: true,
            console: true,
        },
        resources: ResourceLimits {
            max_memory_mb: Some(256),
            max_cpu_time_ms: Some(10_000),
            max_open_files: Some(16),
            max_spawned_processes: Some(0),
            max_threads: Some(0),
        },
        base_dir: PathBuf::from("/workspace"),
        serialized_source: None,
    }
}

#[path = "tests/policy.rs"]
mod policy;

#[path = "tests/predicates.rs"]
mod predicates;

#[path = "tests/effect_analysis.rs"]
mod effect_analysis;

#[path = "tests/effect_reports.rs"]
mod effect_reports;
