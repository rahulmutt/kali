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
        "kali-object-call-result-args-{label}-{unique}-{}-{seq}",
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
fn object_call_result_passed_directly_as_argument_is_correct() {
    // No bound-identifier call site anywhere: the param shape must come from
    // the call-result argument itself. Depth 10 => itemCheck = 2^11 - 1.
    let source = write_temp_source(
        "call_result_arg",
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
  console.log(itemCheck(bottomUpTree(10)));
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2047\n");
}

#[test]
fn inline_nested_literal_field_read_is_rejected_not_miscompiled() {
    // `t.left` folds to the inline nested literal `{ v: 5 }`, whose emission
    // lands in the drop-only aggregate fallback (placeholder 0) — so `.v`
    // reads 0, a silent wrong answer. Must reject with E5506 instead.
    let source = write_temp_source(
        "e5506_inline_nested_read",
        "const t = { left: { v: 5 }, right: null };\nconsole.log(t.left.v);\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("E5506"));
}

#[test]
fn inline_nested_literal_field_comparison_is_rejected_not_miscompiled() {
    // Same base as above, but the nested field value is compared to null:
    // the placeholder 0 makes `t.left === null` print `true` (a wrong
    // answer). Must reject with E5506 instead.
    let source = write_temp_source(
        "e5506_inline_nested_cmp",
        "const t = { left: { v: 5 }, right: null };\nconsole.log(t.left === null);\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("E5506"));
}

#[test]
fn array_literal_element_field_value_read_is_rejected_not_miscompiled() {
    // The object field's value is `arr[0]` — a static index into a
    // const-bound array literal whose element aliases another object
    // literal. The fold lane resolves the element to `leaf`, which emits as
    // the placeholder 0, so `.v` reads 0. Must reject with E5506 instead.
    let source = write_temp_source(
        "e5506_array_elem_field",
        "const leaf = { v: 42 };\nconst arr = [leaf];\nconst t = { left: arr[0], right: null };\nconsole.log(t.left.v);\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("E5506"));
}

#[test]
fn unclassified_object_shape_member_read_is_rejected_not_miscompiled() {
    let source = write_temp_source(
        "e5506_backstop",
        "const leafA = { left: null, right: null };\nconst leafB = { left: null, right: null };\nconst t = { left: leafA, right: leafB };\nconsole.log(t.left === null);\n",
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("E5506"));
}
