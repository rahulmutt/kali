//! Task 8 (binary-trees CLBG fixture): the end-to-end acceptance gate for the
//! whole per-loop/per-function arena-reclamation effort (Tasks 5-7). The
//! canonical CLBG binary-trees benchmark at N=21 allocates ~9.4GB cumulative
//! against wasm32's 4GB linear-memory ceiling — it cannot complete without
//! genuine arena reclamation (post-arena peak ~= 270MB). The escape gate
//! grants `main()` a function arena plus its inner depth loop a loop arena;
//! `bottomUpTree`/`itemCheck` get NO arena of their own (their returned/walked
//! subtrees live in the caller's arena), and the long-lived tree (held across
//! the loop, built before it, read after it) survives untouched.
//!
//! Uses DIRECT call-result arguments (`itemCheck(bottomUpTree(depth))`, no
//! intermediate bindings) throughout, matching the vendored fixture and the
//! Task 7 (`object_call_result_args_runtime.rs`)-style P0b coverage.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

/// Process-wide-unique temp-fixture slug. Uniqueness must NOT depend on
/// wall-clock resolution alone: tests run multi-threaded, and on platforms
/// with a coarse `SystemTime` clock (e.g. macOS) two concurrent calls can
/// observe the same `as_nanos()` value and collide on the same temp dir. A
/// process-wide monotonic counter guarantees uniqueness independently of the
/// wall-clock's resolution (mirrors `arena_reclamation_runtime.rs`).
fn unique_fixture_slug(label: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    format!(
        "kali-binary-trees-{label}-{unique}-{}-{seq}",
        std::process::id()
    )
}

fn write_temp_source(label: &str, source: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(unique_fixture_slug(label));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    fs::write(&path, source).expect("write source fixture");
    path
}

/// Builds the canonical binary-trees program body for an arbitrary `n`
/// (`minDepth` fixed at 4, matching CLBG convention), so the small-N test
/// exercises the exact same shape (direct call-result args, no intermediate
/// bindings) as the vendored N=21 fixture rather than a hand-simplified
/// stand-in.
fn binary_trees_source(n: i64) -> String {
    format!(
        r#"function bottomUpTree(depth) {{
  if (depth > 0) {{
    return {{ left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) }};
  }}
  return {{ left: null, right: null }};
}}

function itemCheck(t) {{
  if (t.left === null) {{
    return 1;
  }}
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}}

function main() {{
  const n = {n};
  const minDepth = 4;
  const maxDepth = n;
  const stretchDepth = maxDepth + 1;
  console.log(`stretch tree of depth ${{stretchDepth}}\t check: ${{itemCheck(bottomUpTree(stretchDepth))}}`);
  const longLivedTree = bottomUpTree(maxDepth);
  for (let depth = minDepth; depth <= maxDepth; depth = depth + 2) {{
    const iterations = 1 << (maxDepth - depth + minDepth);
    let check = 0;
    for (let i = 1; i <= iterations; i = i + 1) {{
      check = check + itemCheck(bottomUpTree(depth));
    }}
    console.log(`${{iterations}}\t trees of depth ${{depth}}\t check: ${{check}}`);
  }}
  console.log(`long lived tree of depth ${{maxDepth}}\t check: ${{itemCheck(longLivedTree)}}`);
}}

main();
"#,
        n = n
    )
}

/// `iterations` for one depth-loop row: `1 << (maxDepth - depth + minDepth)`.
fn iterations_for(max_depth: i64, min_depth: i64, depth: i64) -> i64 {
    1i64 << (max_depth - depth + min_depth)
}

/// `itemCheck` of a full binary tree of the given depth is `2^(depth+1) - 1`
/// (a perfect binary tree has that many nodes); a depth-loop row sums that
/// count `iterations` times.
fn full_tree_check(depth: i64) -> i64 {
    (1i64 << (depth + 1)) - 1
}

/// Builds the expected canonical stdout for the given `n` (minDepth fixed at
/// 4) purely from the formulas above — never hand-typed per line.
fn expected_output(n: i64) -> String {
    let min_depth = 4i64;
    let max_depth = n;
    let stretch_depth = max_depth + 1;
    let mut lines = Vec::new();
    lines.push(format!(
        "stretch tree of depth {stretch_depth}\t check: {}",
        full_tree_check(stretch_depth)
    ));
    let mut depth = min_depth;
    while depth <= max_depth {
        let iterations = iterations_for(max_depth, min_depth, depth);
        let check = iterations * full_tree_check(depth);
        lines.push(format!(
            "{iterations}\t trees of depth {depth}\t check: {check}"
        ));
        depth += 2;
    }
    lines.push(format!(
        "long lived tree of depth {max_depth}\t check: {}",
        full_tree_check(max_depth)
    ));
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

#[test]
fn binary_trees_small_n_matches_canonical_output() {
    // n=10 (minDepth=4, maxDepth=10, stretchDepth=11): a fast always-on
    // acceptance pin, exercising the exact same program shape as the
    // vendored N=21 fixture (direct call-result args, no intermediate
    // bindings) at a size that runs in well under a second.
    let source = write_temp_source("small_n10", &binary_trees_source(10));
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = expected_output(10);
    // Pin the derivation against the brief's hand-verified values, in real
    // tab characters (not the two-character `\`+`t` sequence): stretch 11 ->
    // 4095; 1024/depth4 -> 31744; 256/depth6 -> 32512; 64/depth8 -> 32704;
    // 16/depth10 -> 32752; long lived 10 -> 2047.
    assert_eq!(
        expected,
        "stretch tree of depth 11\t check: 4095\n\
         1024\t trees of depth 4\t check: 31744\n\
         256\t trees of depth 6\t check: 32512\n\
         64\t trees of depth 8\t check: 32704\n\
         16\t trees of depth 10\t check: 32752\n\
         long lived tree of depth 10\t check: 2047\n"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn binary_trees_canonical_n21_matches_output() {
    // The canonical CLBG parameter (n=21, minDepth=4): ~9.4GB cumulative heap
    // allocation against wasm32's 4GB ceiling, completing ONLY because
    // per-loop (Task 6) and per-function (Task 7) arenas reclaim pages as
    // `main`'s depth loop and each `bottomUpTree`/`itemCheck` call retire.
    //
    // NOT `#[ignore]`d: the observed wall-clock is ~8s (the wasm runs under
    // the wasmtime JIT, so it is the same in debug and release), well under
    // the spec's 30s always-on threshold — so this end-to-end acceptance runs
    // as a real gate on every `cargo test -p kali_cli`, not an opt-in. The
    // policy's `maxCpuTimeMs: 64000000` (~64B fuel) is ~2x the measured ~32B
    // fuel need (bisected: 31B traps E4003, 32B passes), the same scoped-
    // policy precedent mandelbrot set to clear the ~60B default runaway guard.
    let source = fixture("binary-trees-benchmark-v1.ts");
    let policy = fixture("binary-trees-benchmark-v1.policy.json");
    let expected = expected_output(21);
    // Sanity-check the formula-derived expectation against the brief's
    // independently-stated canonical values before trusting it further.
    assert!(expected.starts_with("stretch tree of depth 22\t check: 8388607\n"));
    assert!(expected
        .trim_end()
        .ends_with("long lived tree of depth 21\t check: 4194303"));
    assert_eq!(expected.lines().count(), 11, "expected 11 canonical lines");

    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        expected.as_bytes(),
        "canonical N=21 output mismatch\nstdout: {}\nexpected: {}",
        String::from_utf8_lossy(&output.stdout),
        expected
    );
}

#[test]
fn binary_trees_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("binary-trees-benchmark-v1.json")).expect("read metadata"),
    )
    .expect("parse metadata");
    assert_eq!(meta["benchmark"], "binary-trees");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "binary-trees-benchmark-v1.ts");
    assert_eq!(
        meta["buildModes"],
        serde_json::json!(["--fast", "--release", "--release-advanced"])
    );
    let src = fs::read(fixture("binary-trees-benchmark-v1.ts")).expect("read source");
    let digest_bytes = Sha256::digest(&src);
    let digest = format!(
        "sha256-{}",
        digest_bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert_eq!(
        meta["sourceSha256"], digest,
        "metadata sha256 must match the source file"
    );
}
