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

/// Build a process-wide-unique directory name for a test fixture.
///
/// Uniqueness must NOT depend on wall-clock resolution: tests run
/// multi-threaded, and on platforms with a coarse `SystemTime` clock (e.g.
/// macOS) two concurrent calls can observe the same `as_nanos()` value,
/// collide on the same temp dir, and clobber each other's fixture file. A
/// process-wide monotonic counter guarantees uniqueness independently of the
/// wall-clock's resolution. Mirrors the identical idiom in
/// `object_call_result_args_runtime.rs` / `trap_diagnostics_runtime.rs`.
fn unique_fixture_slug(label: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    format!(
        "kali-heap-grow-{label}-{unique}-{}-{seq}",
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

#[test]
fn allocation_beyond_one_megabyte_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("grow.ts");
    // ~3 MB of i64 array storage: 24 arrays of 16384 elements = 24 * (16384+1)*8 bytes ~= 3.15 MB,
    // well past the old 1 MB (16-page) wall. Touch each so it is not folded away.
    std::fs::write(
        &src,
        r#"
let total = 0;
for (let k = 0; k < 24; k = k + 1) {
  const a = new Array(16384);
  a.fill(1);
  total = total + a.length;
}
console.log(total);
"#,
    )
    .expect("write");
    let out = Command::new(kali_bin())
        .arg("run")
        .arg(&src)
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        (24 * 16384).to_string()
    );
}

// NOTE: the task brief's Step 5 proposed a literal binary-trees smoke test
// (`bottomUpTree`/`itemCheck` on a depth-10 tree, expecting 2047). That
// program was tried verbatim during development and found to give the wrong
// answer (`1` instead of `2047`) both before AND after this change, and
// verified present on `main` at 2be3a0cac (pre-dating this whole branch) —
// i.e. it is an unrelated, pre-existing correctness bug, not something
// `memory.grow` touches. Root cause (isolated via a dozen minimal repros):
// reading an object-typed field back (a field whose *value* is itself an
// object handle, e.g. `node.left` where `left` was initialized from another
// object/variable rather than a `null` literal) does not reliably return the
// stored handle — `t.left === null` can spuriously read `true` even when
// `t.left` holds a live, non-null object reference. `bottomUpTree` depends on
// exactly this (each node's `left`/`right` holds a child *object*), so it is
// not usable as a growth-under-recursion smoke test today. This is a
// significant, separate bug in nested-object field codegen — flagged in the
// task report as a blocker for the binary-trees benchmark and for Phase 1
// (escape-analysis regions), not fixed here (out of scope for `__alloc`
// growth).
//
// This test exercises the same intent — recursion-driven allocation past the
// 1 MB wall, as opposed to Step 4's loop-driven one — using arrays (whose
// element/length storage has no such bug) instead of objects.
#[test]
fn recursive_allocation_beyond_wall_no_longer_traps() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("recurse_grow.ts");
    // 4001 calls * (64+1)*8 bytes/array ~= 2.08 MB, past the old 1 MB (16-page)
    // wall; each call allocates its own array and folds its length into an
    // accumulator so the allocation cannot be folded away.
    std::fs::write(
        &src,
        r#"
function recurse(depth, acc) {
  const a = new Array(64);
  a.fill(1);
  const bumped = acc + a.length;
  if (depth <= 0) { return bumped; }
  return recurse(depth - 1, bumped);
}
console.log(recurse(4000, 0));
"#,
    )
    .expect("write");
    let out = Command::new(kali_bin())
        .arg("run")
        .arg(&src)
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        (4001 * 64).to_string()
    ); // 256064
}

#[test]
fn oom_past_sandbox_cap_fails_cleanly() {
    let dir = tempfile::tempdir().expect("tempdir");
    let policy = dir.path().join("tiny.policy.json");
    std::fs::write(
        &policy,
        r#"{"schemaVersion":1,"effects":{"fileSystem":{"read":false,"write":false},"network":{"fetch":false,"connect":false,"listen":false,"maxConnections":null},"process":{"spawn":false,"envRead":false,"envWrite":false},"timer":{"schedule":false,"maxTimeoutMs":null,"maxActiveTimers":null},"eval":false,"random":false,"console":true},"resources":{"maxMemoryMB":4,"maxCpuTimeMs":100000,"maxOpenFiles":null,"maxSpawnedProcesses":0,"maxThreads":0}}"#,
    )
    .expect("write");
    let src = dir.path().join("oom.ts");
    std::fs::write(
        &src,
        "for (let k=0;k<10000;k=k+1){ const a=new Array(16384); a.fill(1); }\nconsole.log(0);",
    )
    .expect("write");
    let out = Command::new(kali_bin())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&src)
        .output()
        .expect("run");
    assert!(!out.status.success(), "expected clean OOM failure");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("panic"), "should not panic: {err}");
}

// Task 5 (page-pool allocator): a `new Array(20000)` is (20000+1)*8 = 160008
// bytes, past PAYLOAD (PAGE-8 = 65528 bytes), so each iteration exercises the
// multi-page SPAN allocation path in `__page_get` (n>1 pages, no free-list
// entries yet) rather than the single-page bump/frontier path. Arrays this
// size already work via today's flat `__heap` bump (recorded as the PASSing
// baseline below, before the page-pool machinery exists); the test's value is
// catching span-path regressions once `__page_get` replaces the bump.
#[test]
fn multi_page_array_allocations_are_correct() {
    let source = write_temp_source(
        "span_arrays",
        r#"function main() {
  let sum = 0;
  for (let round = 0; round < 4; round = round + 1) {
    const a = new Array(20000);
    for (let i = 0; i < 20000; i = i + 1) {
      a[i] = i + round;
    }
    sum = sum + a[19999];
  }
  console.log(sum);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "80002\n"); // 4*19999 + (0+1+2+3)
}
