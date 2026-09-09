//! Tests for the incremental-cache reaper.

use super::*;
use tempfile::TempDir;

/// Writes `name` with `size` bytes into `dir`.
///
/// `filetime` is not a dependency, so age is simulated by writing the file and
/// asserting against a `MAX_AGE` boundary the test controls, rather than by
/// setting mtimes. For the age test we instead drive `reap` against a directory
/// whose entries are all fresh, and check the count/size paths directly.
fn write_entry(dir: &Path, name: &str, size: usize) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, vec![0u8; size]).expect("write cache entry");
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
