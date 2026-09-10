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
    std::fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
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

/// The old key, exactly as it was built before this change: every component an
/// input, ending in the frozen `CARGO_PKG_VERSION`. Real examples of this shape
/// were found in `crates/kali_cli/tests/fixtures/.kali-cache/incremental/`,
/// e.g. `sha256-18aab710...-release-deno-16-profiles:-profile:none-false-false-0.1.0.wasm`.
fn old_style_cache_filename(source: &Path) -> String {
    let source_hash = source_hash_for_file(source).expect("hash source");
    format!(
        "{}-fast-deno-16-profiles:-profile:none-false-false-{}.wasm",
        source_hash,
        env!("CARGO_PKG_VERSION")
    )
}

#[test]
fn an_artifact_written_under_the_old_key_is_never_served() {
    let (dir, source) = project_with_source();

    // Plant an artifact where the pre-fix compiler would have written one, with
    // contents that are obviously not a compile of this source. Before the fix,
    // `compile_source_file` returned exactly these bytes with `cache_hit: true`
    // and never reached codegen -- which is how a codegen diagnostic could not
    // be produced, and how a suite passed in ~0.00s.
    let planted = b"NOT-WASM-PLANTED-BY-AN-OLDER-COMPILER".to_vec();
    let cache_dir = dir.path().join(".kali-cache").join("incremental");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::write(cache_dir.join(old_style_cache_filename(&source)), &planted)
        .expect("plant stale artifact");

    let output = compile_source_file_with_cache_state(
        &source,
        BuildMode::Fast,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("compiling a trivial source must succeed");

    assert!(
        !output.cache_hit,
        "an artifact keyed without compiler identity must not be read"
    );
    assert_ne!(
        output.wasm_bytes, planted,
        "the planted bytes must never reach the caller"
    );
}

/// A project root whose manifest is exactly `manifest`, plus one source file.
fn project_with_manifest(manifest: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp project root");
    std::fs::write(dir.path().join("kali.json"), manifest).expect("write manifest");
    let source = dir.path().join("main.ts");
    std::fs::write(&source, "console.log(1);\n").expect("write source");
    (dir, source)
}

#[test]
fn a_project_can_decline_the_incremental_cache() {
    let (_dir, source) = project_with_manifest(r#"{"schemaVersion":1,"incrementalCache":false}"#);

    assert_eq!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")),
        None,
        "`incrementalCache: false` must decline the on-disk cache"
    );
}

#[test]
fn an_omitted_incremental_cache_field_leaves_the_cache_enabled() {
    let (_dir, source) = project_with_manifest(r#"{"schemaVersion":1}"#);

    assert!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")).is_some(),
        "omission must preserve the behaviour every existing project already has"
    );
}

#[test]
fn an_explicit_true_leaves_the_cache_enabled() {
    let (_dir, source) = project_with_manifest(r#"{"schemaVersion":1,"incrementalCache":true}"#);

    assert!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")).is_some(),
        "an explicit true must mean the same as omission"
    );
}

#[test]
fn an_unparseable_manifest_leaves_the_cache_enabled() {
    let (_dir, source) = project_with_manifest("{ this is not json");

    assert!(
        path_for(&source, Some("aaaaaaaaaaaaaaaa")).is_some(),
        "a broken manifest must not silently change caching behaviour; \
         `load_exclude_set` is tolerant the same way"
    );
}
