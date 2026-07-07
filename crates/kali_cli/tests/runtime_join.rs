use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-join-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

#[test]
fn runtime_join_empty_default_and_static_separators() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(3);\n  for (let i = 0; i < 3; i = i + 1) {\n    a[i] = s.substring(i, i + 1);\n  }\n  console.log(a.join(\"\"));\n  console.log(a.join(\"-\"));\n  console.log(a.join());\n}\nf(\"xyz\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "xyz\nx-y-z\nx,y,z\n");
}

#[test]
fn runtime_join_runtime_separator_and_concat_consumer() {
    let out = run_source(
        "function g(s, sep) {\n  const a = new Array(2);\n  a[0] = s.substring(0, 1);\n  a[1] = s.substring(1, 2);\n  console.log(a.join(sep));\n  console.log(\"[\" + a.join(\"\") + \"]\");\n}\ng(\"ab\", \"::\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a::b\n[ab]\n");
}

#[test]
fn runtime_join_literal_string_elements() {
    // probe_a from the design investigation: silent 0 on main 745a3ecea.
    let out = run_source(
        "var line = new Array(3);\nfor (var i = 0; i < line.length; i = i + 1) {\n  line[i] = \"x\";\n}\nconsole.log(line.join(\"\"));\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "xxx\n");
}

#[test]
fn runtime_join_zero_length_array_prints_empty_line() {
    let out = run_source(
        "function f() {\n  const a = new Array(0);\n  console.log(a.join(\"-\"));\n}\nf();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "\n");
}

#[test]
fn runtime_join_single_element_array_copies() {
    // The always-copy rule (spec §4): a 1-element join returns a FRESH
    // buffer, never the element handle itself.
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = s.substring(0, 2);\n  console.log(a.join(\"-\"));\n}\nf(\"hey\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "he\n");
}

#[test]
fn join_result_feeds_length_and_substring() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(2);\n  a[0] = s.substring(0, 1);\n  a[1] = s.substring(1, 2);\n  const j = a.join(\"\");\n  console.log(j.length);\n  console.log(j.substring(1, 2));\n}\nf(\"ab\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\nb\n");
}

#[test]
fn static_fold_join_stays_green() {
    let out = run_source("const q = [\"x\", \"y\"];\nconsole.log(q.join(\",\"));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x,y\n");
}

#[test]
fn join_of_int_element_array_is_rejected() {
    let out =
        run_source("const a = new Array(2);\na[0] = 1;\na[1] = 2;\nconsole.log(a.join(\",\"));\n");
    assert!(
        !out.status.success(),
        "runtime join over number elements must reject"
    );
}

#[test]
fn join_with_non_ascii_element_is_rejected() {
    let out = run_source("const a = new Array(1);\na[0] = \"é\";\nconsole.log(a.join(\"\"));\n");
    assert!(
        !out.status.success(),
        "byte-length join over non-ASCII elements must reject"
    );
}

#[test]
fn join_with_unproven_separator_is_rejected() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nlet f = 1 / 2;\nconsole.log(a.join(f));\n",
    );
    assert!(!out.status.success(), "non-string separator must reject");
}

#[test]
fn static_receiver_with_variable_separator_is_rejected_not_silent() {
    // probe_b from the design investigation: printed 0 silently on main.
    let out = run_source(
        "var line = [\"a\", \"b\", \"c\"];\nvar sep = \"-\";\nconsole.log(line.join(sep));\n",
    );
    assert!(!out.status.success(), "was silent-wrong 0; must reject now");
}

#[test]
fn join_result_equality_is_rejected() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nif (a.join(\"\") == \"x\") {\n  console.log(1);\n}\n",
    );
    assert!(
        !out.status.success(),
        "join results are runtime concat — identity == must reject"
    );
}

#[test]
fn ternary_wrapped_join_receiver_is_rejected() {
    let out = run_source(
        "function f(c) {\n  const a = new Array(1);\n  a[0] = \"x\";\n  const b = new Array(1);\n  b[0] = \"y\";\n  console.log((c > 0 ? a : b).join(\"\"));\n}\nf(1);\n",
    );
    assert!(
        !out.status.success(),
        "non-identifier receivers hit the fail-closed default"
    );
}

#[test]
fn logical_or_wrapped_join_receiver_is_rejected() {
    // Fix round 1: was silent 0 — the parser lowers `||` to BinaryExpression,
    // which dodged a LogicalExpression-only reject arm.
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nconst b = new Array(1);\nb[0] = \"y\";\nconsole.log((a || b).join(\"-\"));\n",
    );
    assert!(
        !out.status.success(),
        "logical-|| wrapper receivers hit the fail-closed default (was silent 0)"
    );
}

#[test]
fn logical_and_wrapped_join_receiver_is_rejected() {
    // Fix round 1: was silent 0 (same BinaryExpression("&&") parse shape).
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nconst b = new Array(1);\nb[0] = \"y\";\nconsole.log((a && b).join(\"-\"));\n",
    );
    assert!(
        !out.status.success(),
        "logical-&& wrapper receivers hit the fail-closed default (was silent 0)"
    );
}

#[test]
fn call_result_join_receiver_is_rejected() {
    // Fix round 1: was silent 0. Fold-eligible call receivers (e.g.
    // `Object.keys(staticObj).join(',')`) take the static fold lane and never
    // reach the runtime lane; a runtime call result has no lowering — reject.
    let out = run_source(
        "function mk() {\n  const a = new Array(2);\n  a[0] = \"x\";\n  a[1] = \"y\";\n  return a;\n}\nconsole.log(mk().join(\"-\"));\n",
    );
    assert!(
        !out.status.success(),
        "call-result receivers hit the fail-closed default (was silent 0)"
    );
}

#[test]
fn static_object_keys_join_stays_green() {
    // Pin the fold-lane disjointness the call-result reject relies on:
    // `Object.keys(staticObj)` is a static-lane receiver, not a runtime call.
    let out = run_source("const o = { a: 1, b: 2 };\nconsole.log(Object.keys(o).join(\",\"));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a,b\n");
}

// ---------------------------------------------------------------------------
// Task 8: fail-closed gates batch — object literals, `&&`/`||`, slice,
// literal-array mutation. Step 1 probe classifications (kali vs node):
//   1. object-literal `{ v: s }` construction   — was silent (exit 0, no diag) -> reject
//   2. `a[0] = 1 && s`                          — was silent-WRONG (printed "1", node "x") -> reject
//   3. `s.slice(1)` runtime string receiver     — ALREADY rejected (via the array-slice
//      catch-all firing on every `.slice(...)` call); this task adds a precise
//      String-specific diagnostic alongside it, not a new reject -> green pin (already-correct)
//   4. `a.slice(0)` runtime array receiver      — ALREADY rejected (same array-slice
//      catch-all) -> green pin (already-correct)
//   5. `a[k] = 42` literal-array runtime index  — was silent-WRONG (printed "0", node "42") -> reject
//   6. `a[1] = 42` literal-array, named function scope — was silent-WRONG (printed "2",
//      node "42"; NOT "0" as the design note guessed — recorded as observed) -> reject
//   7. `a[1] = 42` literal-array, top-level (`_start`) static index — kali prints "0",
//      node prints "42": this shape is ALSO silent-wrong relative to node, but it is a
//      PRE-EXISTING residual outside this task's scope (no new green lanes); pinned as
//      byte-identical-to-base (still "0", still exit 0) so this gate batch does not
//      regress it into either a reject or a different wrong value.
// ---------------------------------------------------------------------------

#[test]
fn object_literal_runtime_string_value_is_rejected() {
    let out = run_source("function f(s) {\n  const o = { v: s };\n}\nf(\"x\");\n");
    assert!(
        !out.status.success(),
        "object-literal construction store must reject"
    );
}

#[test]
fn logical_launder_into_element_store_is_rejected() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = 1 && s;\n  console.log(a[0]);\n}\nf(\"x\");\n",
    );
    assert!(
        !out.status.success(),
        "&&/|| must not launder runtime strings into stores"
    );
}

#[test]
fn runtime_string_slice_is_rejected() {
    let out = run_source("function f(s) {\n  console.log(s.slice(1));\n}\nf(\"abc\");\n");
    assert!(
        !out.status.success(),
        "slice on a runtime string receiver must reject (was silent 0)"
    );
}

#[test]
fn runtime_array_slice_is_rejected() {
    let out = run_source(
        "const a = new Array(2);\na[0] = 7;\nconst b = a.slice(0);\nconsole.log(b[0]);\n",
    );
    assert!(
        !out.status.success(),
        "slice on a runtime array receiver must reject"
    );
}

#[test]
fn literal_array_runtime_index_mutation_is_rejected() {
    let out = run_source(
        "function g(k) {\n  const a = [1, 2, 3];\n  a[k] = 42;\n  console.log(a[k]);\n}\ng(1);\n",
    );
    assert!(!out.status.success(), "was silent-wrong 0; must reject");
}

#[test]
fn literal_array_function_scope_mutation_is_rejected() {
    let out = run_source(
        "function h() {\n  const a = [1, 2, 3];\n  a[1] = 42;\n  console.log(a[1]);\n}\nh();\n",
    );
    assert!(
        !out.status.success(),
        "was silent-wrong (printed 2, node 42); must reject"
    );
}

#[test]
fn literal_array_top_level_static_index_mutation_stays_unchanged() {
    // Probe 7: pre-existing silent-wrong residual (node prints "42", kali
    // prints "0") — out of scope for this task (no new green lanes). Pinned
    // so the literal-array mutation gate (condition (a) index-foldable AND
    // (b) `_start`-only scope) does not regress this top-level fold-lane
    // shape into a reject or a different wrong value.
    let out = run_source("var a = [1, 2, 3];\na[1] = 42;\nconsole.log(a[1]);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n");
}
