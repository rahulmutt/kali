use std::process::Command;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
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
