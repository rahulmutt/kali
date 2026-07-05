use std::{
    fs,
    path::PathBuf,
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
/// wall-clock's resolution.
fn unique_fixture_slug(label: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    format!(
        "kali-trap-diagnostics-{label}-{unique}-{}-{seq}",
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

fn write_temp_policy_json(label: &str, policy: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(unique_fixture_slug(label));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("policy.json");
    fs::write(&path, policy).expect("write policy fixture");
    path
}

#[test]
fn fuel_exhaustion_reports_e4003_with_actionable_message() {
    // Runs forever; exhausts the 60M default fuel budget in well under a second.
    let source = write_temp_source(
        "fuel_runaway",
        "let i = 0;\nwhile (true) {\n  i = i + 1;\n}\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E4003"), "stderr: {stderr}");
    assert!(
        stderr.contains("CPU fuel budget exhausted"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("resources.maxCpuTimeMs"),
        "stderr: {stderr}"
    );
    assert!(
        !stderr.contains("E4000"),
        "fuel exhaustion must not present as a bare runtime trap: {stderr}"
    );
}

#[test]
fn deep_object_workload_is_correct_to_64mb_under_raised_fuel_policy() {
    let source = write_temp_source(
        "trees_64mb",
        r#"function bottomUpTree(depth) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}
function itemCheck(t) {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}
function main() {
  let check = 0;
  for (let i = 1; i <= 8000; i = i + 1) {
    const tree = bottomUpTree(8);
    check = check + itemCheck(tree);
  }
  console.log(check);
}
main();
"#,
    );
    let policy = write_temp_policy_json(
        "fuel_grant",
        r#"{
  "schemaVersion": 1,
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": null },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": null, "maxActiveTimers": null },
    "eval": false,
    "random": false,
    "console": true
  },
  "resources": {
    "maxMemoryMB": null,
    "maxCpuTimeMs": 600000,
    "maxOpenFiles": null,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // 8000 iterations x itemCheck(depth-8 tree) = 8000 x 511
    assert_eq!(String::from_utf8_lossy(&output.stdout), "4088000\n");
}

#[test]
fn stdout_emitted_before_a_trap_is_not_lost() {
    let source = write_temp_source(
        "stdout_before_trap",
        "console.log(777);\nlet i = 0;\nwhile (true) {\n  i = i + 1;\n}\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("777"),
        "pre-trap stdout must be flushed; got stdout: {:?} stderr: {:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("E4003"));
}

#[test]
fn quiet_run_still_reports_the_trap_diagnostic_on_stderr() {
    // `--quiet` suppresses status text and output replay, not error output:
    // a trap must never become a silent nonzero exit.
    let source = write_temp_source(
        "quiet_trap",
        "let i = 0;\nwhile (true) {\n  i = i + 1;\n}\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg("--quiet")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E4003"),
        "trap diagnostics must survive --quiet; got stderr: {stderr:?}"
    );
}
