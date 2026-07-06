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
