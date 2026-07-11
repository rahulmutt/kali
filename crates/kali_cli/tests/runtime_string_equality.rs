use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // Per-process AtomicU64 counter slug — repo convention against the macOS
    // CI temp-dir collision flake (see runtime_string_value_flow.rs).
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-streq-{}-{}-{}",
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

fn assert_ok(out: &std::process::Output) {
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok\n");
}

// Every expectation below was derived by running the same source under node
// (self-check `throw` shapes, so the pre-existing 1/0-vs-true/false boolean
// print divergence never matters).

#[test]
fn concat_equality_compares_content() {
    // node: "x" + "y" == "xy" → true (fresh handle vs interned literal).
    let out = run_source(
        "let a = \"x\";\nlet b = a + \"y\";\nif (b !== \"xy\") { throw new Error(\"content equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn concat_inequality_same_length_different_bytes() {
    // node: "x" + "z" === "xy" → false.
    let out = run_source(
        "let a = \"x\";\nlet b = a + \"z\";\nif (b === \"xy\") { throw new Error(\"same-length different bytes compared equal\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn concat_inequality_different_length() {
    // node: "x" + "yz" === "xy" → false (length pre-check path).
    let out = run_source(
        "let a = \"x\";\nlet b = a + \"yz\";\nif (b === \"xy\") { throw new Error(\"different lengths compared equal\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn empty_concat_equals_empty_literal() {
    // node: "" + "" === "" → true (len-0 path: fresh empty handle at a
    // DIFFERENT offset than the interned "" — must still be equal).
    let out = run_source(
        "let a = \"\";\nlet b = a + \"\";\nif (b !== \"\") { throw new Error(\"empty strings compared unequal\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn empty_vs_nonempty_is_unequal() {
    // node: "" + "" === "x" → false.
    let out = run_source(
        "let a = \"\";\nlet b = a + \"\";\nif (b === \"x\") { throw new Error(\"empty equalled nonempty\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn interned_literal_equality_still_true() {
    // node: s = "hi"; s === "hi" → true (the __streq identity fast path —
    // must not regress the interned lane).
    let out = run_source(
        "let s = \"hi\";\nif (s !== \"hi\") { throw new Error(\"interned equality regressed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn substring_equality_compares_content() {
    // node: "GGCC".substring(0, 1) === "G" → true (zero-copy slice handle vs
    // interned literal; previously E3200-rejected).
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet s = a.substring(0, i);\nif (s !== \"G\") { throw new Error(\"substring equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn join_equality_compares_content() {
    // node: ["x"].join("") === "x" → true (fresh __join buffer; previously
    // E3200-rejected).
    let out = run_source(
        "const a = new Array(1);\na[0] = \"x\";\nif (a.join(\"\") !== \"x\") { throw new Error(\"join equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

fn run_node_source_with_args(src: &str, args: &[&str]) -> std::process::Output {
    // node-API surface + `--` guest-arg separator, same shape as
    // runtime_argv.rs's helper.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-streq-argv-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let mut cmd = Command::new(kali_bin());
    cmd.arg("run").arg("--api").arg("node").arg(&path).arg("--");
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("run kali")
}

#[test]
fn argv_element_equality_compares_content() {
    // node: process.argv[2] === "hello" → true when invoked with "hello"
    // (argv elements are fresh args_get buffers; previously handle-compared).
    let out = run_node_source_with_args(
        "if (process.argv[2] !== \"hello\") { throw new Error(\"argv equality failed\"); }\nconsole.log(\"ok\");\n",
        &["hello"],
    );
    assert_ok(&out);
}

#[test]
fn double_negation_lanes_agree() {
    // node: both ("a"+x) == "az" and !(("a"+x) != "az") are true — the ==
    // and != lowerings must be exact complements.
    let out = run_source(
        "let x = \"z\";\nlet b = \"a\" + x;\nif (b == \"az\") { if (b != \"az\") { throw new Error(\"eq and ne disagree\"); } console.log(\"ok\"); } else { throw new Error(\"eq false\"); }\n",
    );
    assert_ok(&out);
}
