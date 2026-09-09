//! Tests for incremental cache key composition.
//!
//! The key must name the compiler build that produced the artifact. These
//! tests pass synthetic fingerprints rather than relinking a binary mid-test,
//! which is why `incremental_cache_path_with_fingerprint` takes one.

use super::*;
use tempfile::TempDir;

/// A project root with a manifest, plus one compilable source file in it.
fn project_with_source() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp project root");
    std::fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#)
        .expect("write manifest");
    let source = dir.path().join("main.ts");
    std::fs::write(&source, "console.log(1);\n").expect("write source");
    (dir, source)
}

fn path_for(source: &Path, fingerprint: Option<&str>) -> Option<PathBuf> {
    incremental_cache_path_with_fingerprint(
        source,
        BuildMode::Fast,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
        fingerprint,
    )
    .expect("cache path computation must not error")
}

#[test]
fn a_missing_fingerprint_disables_the_cache_rather_than_falling_back() {
    let (_dir, source) = project_with_source();

    assert_eq!(
        path_for(&source, None),
        None,
        "a compiler that cannot identify itself must decline the cache entirely; \
         falling back to a version-only key is the defect this change removes"
    );
}

#[test]
fn distinct_fingerprints_yield_distinct_cache_paths() {
    let (_dir, source) = project_with_source();

    let first = path_for(&source, Some("aaaaaaaaaaaaaaaa")).expect("path for first build");
    let second = path_for(&source, Some("bbbbbbbbbbbbbbbb")).expect("path for second build");

    assert_ne!(
        first, second,
        "two compiler builds must not share a cache entry for the same source"
    );
}

#[test]
fn one_fingerprint_reproduces_one_path() {
    let (_dir, source) = project_with_source();

    let first = path_for(&source, Some("aaaaaaaaaaaaaaaa")).expect("first call");
    let second = path_for(&source, Some("aaaaaaaaaaaaaaaa")).expect("second call");

    assert_eq!(
        first, second,
        "the same compiler and the same inputs must hit the same entry, or the \
         cache never hits at all"
    );
}

#[test]
fn the_cache_filename_carries_the_fingerprint() {
    let (_dir, source) = project_with_source();

    let path = path_for(&source, Some("abcdef0123456789")).expect("cache path");
    let name = path
        .file_name()
        .expect("cache path has a filename")
        .to_string_lossy()
        .into_owned();

    assert!(
        name.contains("abcdef0123456789"),
        "the fingerprint must be visible in the key so a stale entry is \
         identifiable by name, got {name:?}"
    );
}
