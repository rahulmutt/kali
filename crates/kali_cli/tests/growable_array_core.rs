//! Stage 4 Task 2: growable runtime array core — `push` accumulates, `.length`
//! and `x[i]` read it back (i64 elements). Modeled on
//! `array_callback_identity_map.rs` (kali_bin() helper, tempdir, `run`).

use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Basic core: three pushes, then `.length`, two index reads, and a summing
/// loop over `o[i]`.
fn growable_core_source() -> &'static str {
    r#"function main() {
  const o = [];
  o.push(1);
  o.push(2);
  o.push(3);
  console.log(o.length);
  console.log(o[0]);
  console.log(o[2]);
  let s = 0;
  for (let i = 0; i < o.length; i++) {
    s += o[i];
  }
  console.log(s);
}
main();
"#
}

/// Growth across the realloc boundary (INITIAL_CAP = 4): ten pushes force a
/// 4→8→16 capacity doubling; `o[4]` proves the copy preserved earlier slots
/// written before the realloc, `o[9]` proves the post-realloc appends landed.
fn growable_realloc_boundary_source() -> &'static str {
    r#"function main() {
  const o = [];
  for (let i = 0; i < 10; i++) {
    o.push(i * 2);
  }
  console.log(o.length);
  console.log(o[9]);
  console.log(o[4]);
}
main();
"#
}

fn assert_run_stdout(source: &str, extension: &str, expected: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "run failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, expected, "unexpected stdout: {stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
}

#[test]
fn run_supports_growable_array_push_length_index_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(growable_core_source(), extension, "3\n1\n3\n6\n");
    }
}

#[test]
fn run_supports_growable_array_growth_across_realloc_boundary_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(growable_realloc_boundary_source(), extension, "10\n18\n8\n");
    }
}

/// Seeded literal `const o = [5, 7]`: `len = seed_len`, seeds land in the
/// data block, and a push appends after them.
fn growable_seeded_literal_source() -> &'static str {
    r#"function main() {
  const o = [5, 7];
  o.push(9);
  console.log(o.length);
  console.log(o[0]);
  console.log(o[2]);
}
main();
"#
}

#[test]
fn run_supports_growable_array_seeded_literal_declarator_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(growable_seeded_literal_source(), extension, "3\n5\n9\n");
    }
}

/// Task 4: `for (const v of o)` over a growable array runs a real runtime
/// counted loop (not a compile-time unroll of the stale declarator literal).
/// The SAME growable is exercised as a for-of source AND `out` is both a push
/// sink inside the body and a for-of source of its own — the fixture's exact
/// shape.
fn growable_for_of_source() -> &'static str {
    r#"function main() {
  const o = [];
  o.push(10);
  o.push(20);
  o.push(30);
  const out = [];
  for (const v of o) {
    out.push(v);
  }
  console.log(out.length);
  for (const v of out) {
    console.log(v);
  }
}
main();
"#
}

#[test]
fn run_supports_for_of_over_growable_array_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(growable_for_of_source(), extension, "3\n10\n20\n30\n");
    }
}

/// Task 4 fail-closed (review fix): pushing into the array being iterated
/// grows the iteration under node but NOT under the counted loop's
/// once-snapshotted length — must reject E5506 at compile time, never run
/// with fixed-count semantics (node prints 5; the miscompile printed 2).
fn growable_self_push_in_for_of_source() -> &'static str {
    r#"function main() {
  const o = [];
  o.push(1);
  o.push(2);
  let count = 0;
  for (const v of o) {
    count++;
    if (o.length < 5) o.push(v);
  }
  console.log(count);
}
main();
"#
}

#[test]
fn run_rejects_push_into_the_growable_array_being_iterated_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(&source_path, growable_self_push_in_for_of_source()).expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            !output.status.success(),
            "expected a self-push for-of to be rejected, not silently run with a fixed count"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.is_empty(),
            "expected NO stdout (never a silent node-divergent run): {stdout}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("iterating"),
            "expected the self-push-specific message, stderr: {stderr}"
        );
    }
}

/// Task 3: uniform-String pushes promote with element repr `String` — the
/// index-read result feeding `console.log` is treated as a string handle.
fn growable_string_push_source() -> &'static str {
    r#"function main() {
  const o = [];
  o.push("a");
  o.push("b");
  console.log(o[0]);
  console.log(o.length);
}
main();
"#
}

#[test]
fn run_supports_growable_array_string_push_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(growable_string_push_source(), extension, "a\n2\n");
    }
}

/// Task 3 fail-closed: a MIXED i64+String push set is a shape conflict
/// (E5506) at compile time — never a silent fall-back to the pre-promotion
/// no-op lane (which would print `undefined`/`0` and exit 0).
fn growable_mixed_push_source() -> &'static str {
    r#"function main() {
  const o = [];
  o.push(1);
  o.push("a");
  console.log(o.length);
}
main();
"#
}

/// Task 5: `x.join(sep)` over a growable i64 array renders each element as a
/// decimal string (runtime `int_to_string`) and copies the separator between
/// them — two calls with different separators (`","` then `"\n"`) prove the
/// separator is a real runtime argument, not baked in.
fn growable_int_join_source() -> &'static str {
    r#"function main() {
  const o = [];
  o.push(1);
  o.push(2);
  o.push(3);
  console.log(o.join(","));
  console.log(o.join("\n"));
}
main();
"#
}

#[test]
fn run_supports_growable_array_int_join_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(growable_int_join_source(), extension, "1,2,3\n1\n2\n3\n");
    }
}

/// Task 5: `x.join(sep)` over a growable String array copies each element
/// string handle plus the separator between them (the byte `memory.copy` path,
/// adjusted for the growable header indirection).
fn growable_string_join_source() -> &'static str {
    r#"function main() {
  const o = [];
  o.push("a");
  o.push("b");
  console.log(o.join(","));
}
main();
"#
}

#[test]
fn run_supports_growable_array_string_join_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(growable_string_join_source(), extension, "a,b\n");
    }
}

/// Task 5 edge parity vs node: negative integers render with a sign, an empty
/// array joins to the empty string, a single element emits no separator, and an
/// empty separator concatenates. All checked byte-for-byte against node.
fn growable_join_edges_source() -> &'static str {
    r#"function main() {
  const neg = [];
  neg.push(-5);
  neg.push(2);
  neg.push(-3);
  console.log(neg.join(","));
  const empty = [];
  console.log(empty.join(","));
  const single = [];
  single.push(7);
  console.log(single.join(","));
  const cat = [];
  cat.push(1);
  cat.push(2);
  console.log(cat.join(""));
}
main();
"#
}

#[test]
fn run_supports_growable_array_join_edges_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(
            growable_join_edges_source(),
            extension,
            "-5,2,-3\n\n7\n12\n",
        );
    }
}

#[test]
fn run_rejects_growable_array_mixed_i64_and_string_push_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(&source_path, growable_mixed_push_source()).expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            !output.status.success(),
            "expected a mixed i64/String push set to be rejected, not silently run"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.is_empty(),
            "expected NO stdout (never a silent wrong-output run): {stdout}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("used as both strings and numbers"),
            "stderr: {stderr}"
        );
    }
}

/// Task 6 review fix (truthful enumeration-key inference): `for..of` over
/// `Object.keys(<string>)` iterates STRINGS, so a growable receiver pushing the
/// loop variable promotes with a String element axis and a bare `.join`
/// renders content ("0,1") — previously the element axis solved I64 and join
/// printed the raw string-handle bits. Byte-for-byte vs node.
fn growable_enumeration_key_join_source() -> &'static str {
    r#"function main() {
  const keys = [];
  for (const key of Object.keys("ab")) {
    keys.push(key);
  }
  console.log(keys.length);
  console.log(keys.join(","));
}
main();
"#
}

#[test]
fn run_supports_join_over_enumeration_key_pushes_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_run_stdout(
            growable_enumeration_key_join_source(),
            extension,
            "2\n0,1\n",
        );
    }
}
