use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-strarr-{}-{}-{}",
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
fn string_element_store_and_read_roundtrip() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(2);\n  a[0] = s.substring(0, 2);\n  a[1] = \"!\";\n  console.log(a[0]);\n  console.log(a[0] + a[1]);\n}\nf(\"hey\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "he\nhe!\n");
}

#[test]
fn string_element_read_feeds_length_and_substring() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = s.substring(0, 2);\n  console.log(a[0].length);\n  console.log(a[0].substring(1, 2));\n}\nf(\"hey\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\ne\n");
}

#[test]
fn interned_literal_element_identity_equality_stays_green() {
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nif (a[0] == \"x\") {\n  console.log(7);\n}\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}

#[test]
fn tainted_element_equality_is_rejected() {
    let out = run_source(
        "function f(s) {\n  const a = new Array(1);\n  a[0] = s + \"y\";\n  if (a[0] == \"xy\") {\n    console.log(1);\n  }\n}\nf(\"x\");\n",
    );
    assert!(
        !out.status.success(),
        "concat-tainted element == must reject"
    );
}

#[test]
fn mixed_element_array_is_rejected() {
    let out = run_source("const a = new Array(2);\na[0] = \"x\";\na[1] = 1;\nconsole.log(a[0]);\n");
    assert!(
        !out.status.success(),
        "mixed string/number elements must reject"
    );
}

#[test]
fn object_field_string_store_still_rejected() {
    let out = run_source("function f(s) {\n  const o = { v: 0 };\n  o.v = s;\n}\nf(\"x\");\n");
    assert!(
        !out.status.success(),
        "field stores stay gated (arrays only in Spec 3)"
    );
}

#[test]
fn non_ascii_element_length_is_rejected() {
    let out = run_source("const a = new Array(1);\na[0] = \"héllo\";\nconsole.log(a[0].length);\n");
    assert!(
        !out.status.success(),
        "byte-len .length on non-ASCII element must reject"
    );
}

#[test]
fn array_alloc_reassignment_int_elements() {
    // fastaRandom's partial-last-line shape; silent-wrong 0 on main 745a3ecea.
    let out = run_source(
        "function g(n) {\n  let a = new Array(60);\n  if (n < 60) {\n    a = new Array(n);\n  }\n  for (let i = 0; i < a.length; i = i + 1) {\n    a[i] = i * 10;\n  }\n  console.log(a[1]);\n  console.log(a.length);\n}\ng(3);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n3\n");
}

#[test]
fn array_alloc_reassignment_string_elements() {
    let out = run_source(
        "function g(n, s) {\n  let a = new Array(4);\n  if (n < 4) {\n    a = new Array(n);\n  }\n  for (let i = 0; i < a.length; i = i + 1) {\n    a[i] = s.substring(0, 1);\n  }\n  console.log(a[0] + a[1]);\n  console.log(a.length);\n}\ng(2, \"xy\");\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "xx\n2\n");
}

#[test]
fn array_to_array_binding_copy() {
    let out = run_source(
        "function g() {\n  const b = new Array(2);\n  b[0] = 5;\n  b[1] = 6;\n  let a = new Array(1);\n  a = b;\n  console.log(a[1]);\n  console.log(a.length);\n}\ng();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "6\n2\n");
}

#[test]
fn scalar_reassignment_of_array_binding_is_rejected() {
    let out = run_source("let a = new Array(2);\na[0] = 1;\na = 5;\nconsole.log(a[0]);\n");
    assert!(
        !out.status.success(),
        "scalar into array binding must reject, not clobber the handle"
    );
}
