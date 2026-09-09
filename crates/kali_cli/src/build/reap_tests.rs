//! Tests for the incremental-cache reaper.

use super::*;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Writes `name` with `size` bytes into `dir`.
///
/// `filetime` is not a dependency, so age is simulated by writing the file and
/// asserting against a `MAX_AGE` boundary the test controls, rather than by
/// setting mtimes. For the age test we instead drive `reap` against a directory
/// whose entries are all fresh, and check the count/size paths directly. The
/// file is sparse -- real bytes are never needed because `reap` only reads
/// `metadata.len()`, and `set_len` reports the full length without writing it.
fn write_entry(dir: &Path, name: &str, size: usize) -> PathBuf {
    let path = dir.join(name);
    fs::File::create(&path)
        .and_then(|file| file.set_len(size as u64))
        .expect("write cache entry");
    path
}

#[test]
fn a_missing_directory_is_a_no_op() {
    let dir = TempDir::new().expect("temp dir");
    let missing = dir.path().join("does-not-exist");

    reap(&missing); // must not panic
    assert!(!missing.exists());
}

#[test]
fn an_empty_directory_is_a_no_op() {
    let dir = TempDir::new().expect("temp dir");

    reap(dir.path());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn entries_within_budget_are_all_kept() {
    let dir = TempDir::new().expect("temp dir");
    for index in 0..16 {
        write_entry(dir.path(), &format!("entry-{index}.wasm"), 64);
    }

    reap(dir.path());

    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        16,
        "nothing may be evicted while under both budgets"
    );
}

#[test]
fn exceeding_the_entry_cap_evicts_down_to_it() {
    let dir = TempDir::new().expect("temp dir");
    for index in 0..(MAX_ENTRIES + 32) {
        write_entry(dir.path(), &format!("entry-{index:05}.wasm"), 1);
    }

    reap(dir.path());

    let remaining = fs::read_dir(dir.path()).unwrap().count();
    assert!(
        remaining <= MAX_ENTRIES,
        "the entry cap is the acute resource; got {remaining} entries"
    );
}

#[test]
fn a_directory_containing_a_subdirectory_ignores_it() {
    let dir = TempDir::new().expect("temp dir");
    fs::create_dir(dir.path().join("nested")).expect("create nested dir");
    write_entry(dir.path(), "entry.wasm", 64);

    reap(dir.path());

    assert!(
        dir.path().join("nested").exists(),
        "the reaper removes files, never directories"
    );
}

/// Pins that eviction over the entry cap removes the OLDEST entries first and
/// the NEWEST survive. Every file in `exceeding_the_entry_cap_evicts_down_to_it`
/// is 1 byte, so that test alone would pass even if the sort comparator in
/// `reap.rs` were reversed to evict newest-first; this test would then fail.
///
/// No sleep between individual writes -- 4128 of those would be far slower
/// than the rest of this suite for no benefit. Instead the entries are written
/// in two groups separated by one sleep, giving every "old" entry a strictly
/// earlier mtime than every "new" entry (subject to the filesystem's mtime
/// resolution, which the 20ms gap is chosen to clear).
#[test]
fn exceeding_the_entry_cap_evicts_the_oldest_and_keeps_the_newest() {
    let dir = TempDir::new().expect("temp dir");
    let old_count = MAX_ENTRIES + 32 - 28;
    let new_count = 28;

    for index in 0..old_count {
        write_entry(dir.path(), &format!("old-{index:05}.wasm"), 1);
    }
    thread::sleep(Duration::from_millis(20));
    for index in 0..new_count {
        write_entry(dir.path(), &format!("new-{index:02}.wasm"), 1);
    }

    reap(dir.path());

    for index in 0..new_count {
        assert!(
            dir.path().join(format!("new-{index:02}.wasm")).exists(),
            "newest entry new-{index:02}.wasm was evicted; eviction must prefer the oldest entries"
        );
    }
    let remaining = fs::read_dir(dir.path()).unwrap().count();
    assert!(
        remaining <= MAX_ENTRIES,
        "the entry cap is the acute resource; got {remaining} entries"
    );
}

/// Pins the `MAX_BYTES` half of the eviction condition independently of the
/// entry-count cap: a handful of large files, well under `MAX_ENTRIES`, must
/// still trigger eviction once their combined size passes `MAX_BYTES`. An
/// `&&` -> `||` regression in the break condition would otherwise go
/// undetected, since the only other over-budget test (the entry-cap test)
/// never approaches `MAX_BYTES`.
#[test]
fn exceeding_the_byte_budget_evicts_even_under_the_entry_cap() {
    let dir = TempDir::new().expect("temp dir");
    let entry_size = (MAX_BYTES / 10) as usize;
    let entry_count = 14; // 14 * (MAX_BYTES / 10) > MAX_BYTES, and 14 << MAX_ENTRIES.
    assert!((entry_count as u64) * (entry_size as u64) > MAX_BYTES);
    assert!(entry_count < MAX_ENTRIES);

    for index in 0..entry_count {
        write_entry(dir.path(), &format!("big-{index:02}.wasm"), entry_size);
    }

    reap(dir.path());

    let remaining: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .collect();
    assert!(
        remaining.len() < entry_count,
        "over the byte budget with entries to spare under the entry cap must still evict; \
         wrote {entry_count}, kept {}",
        remaining.len()
    );
    let remaining_bytes: u64 = remaining
        .iter()
        .map(|path| fs::metadata(path).unwrap().len())
        .sum();
    assert!(
        remaining_bytes <= MAX_BYTES,
        "surviving entries must fit the byte budget; got {remaining_bytes} bytes"
    );
}

/// Pins the "unreadable entry" half of directory-safety: a dangling symlink
/// alongside a real file must not panic `reap`, and the real file must
/// survive.
///
/// Empirically, `std::fs::DirEntry::metadata` on this platform does not
/// traverse symlinks (it is `lstat`-like), so a dangling symlink's own
/// metadata read succeeds and reports `is_file() == false` -- this exercises
/// the `!metadata.is_file()` skip at `reap.rs`, the same branch the
/// subdirectory test exercises, rather than the `entry.metadata()` `Err`
/// branch. That branch (a metadata *read* failing outright) would need an
/// entry removed between `read_dir` and `metadata()`, or a permissions
/// failure -- neither reliably constructible here without flakiness or root.
/// See the fix-round section of the task report for detail.
/// This test is kept because it still pins real, useful behavior (a symlink
/// in the cache directory cannot crash or corrupt the sweep), documented
/// honestly rather than mis-labeled.
#[test]
#[cfg(unix)]
fn a_dangling_symlink_does_not_panic_and_the_real_entry_survives() {
    use std::os::unix::fs::symlink;

    let dir = TempDir::new().expect("temp dir");
    let target = dir.path().join("nonexistent-target");
    let link = dir.path().join("dangling-link.wasm");
    symlink(&target, &link).expect("create dangling symlink");
    write_entry(dir.path(), "entry.wasm", 64);

    reap(dir.path()); // must not panic

    assert!(
        dir.path().join("entry.wasm").exists(),
        "the real file must survive a dangling symlink in the same directory"
    );
}
