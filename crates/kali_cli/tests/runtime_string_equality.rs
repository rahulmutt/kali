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

#[test]
fn negated_inequality_true_for_unequal_strings() {
    // node: ("x"+"z") != "xy" → true and ("x"+"z") !== "xy" → true. The other
    // tests only pin the negation lane's FALSE side (equal strings) or use
    // ===/== for unequal operands; this pins the negation TRUE outcome, so a
    // constant-false `!=`/`!==` lowering cannot pass the suite.
    let out = run_source(
        "let a = \"x\";\nlet b = a + \"z\";\nif (b != \"xy\") { if (b !== \"xy\") { console.log(\"ok\"); } else { throw new Error(\"strict negation false for unequal\"); } } else { throw new Error(\"loose negation false for unequal\"); }\n",
    );
    assert_ok(&out);
}

fn run_source_with_env(src: &str, key: &str, value: Option<&str>) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-streq-env-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let mut cmd = Command::new(kali_bin());
    cmd.arg("run").arg(&path).env_remove(key);
    if let Some(value) = value {
        cmd.env(key, value);
    }
    cmd.output().expect("run kali")
}

// Node analog for derivation: `process.env.K` (node has no Deno global);
// semantics asserted are plain JS string/undefined equality.

#[test]
fn env_get_equality_matches_set_value() {
    // node analog: process.env.K = "y"; process.env.K === "y" → true.
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_A\") !== \"y\") { throw new Error(\"env equality failed\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_A",
        Some("y"),
    );
    assert_ok(&out);
}

#[test]
fn env_get_equality_rejects_different_value() {
    // node analog: env K = "z"; K === "y" → false.
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_B\") === \"y\") { throw new Error(\"different env value compared equal\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_B",
        Some("z"),
    );
    assert_ok(&out);
}

#[test]
fn env_get_missing_is_unequal_to_every_string() {
    // node analog: undefined === "y" → false, and undefined === "" → false
    // (the __streq TAG guard: a 0 result is not a string handle).
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_MISSING\") === \"y\") { throw new Error(\"missing env equalled a string\"); }\nif (Deno.env.get(\"KALI_STREQ_MISSING\") === \"\") { throw new Error(\"missing env equalled empty string\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_MISSING",
        None,
    );
    assert_ok(&out);
}

#[test]
fn env_get_empty_value_equals_empty_literal() {
    // node analog: env K = ""; K === "" → true (present-but-empty is a REAL
    // empty string, distinct from missing/undefined).
    let out = run_source_with_env(
        "if (Deno.env.get(\"KALI_STREQ_EMPTY\") !== \"\") { throw new Error(\"empty env value unequal to empty literal\"); }\nconsole.log(\"ok\");\n",
        "KALI_STREQ_EMPTY",
        Some(""),
    );
    assert_ok(&out);
}

// The headline #2/#3 bucket shapes (throw-fallout denominator): enumeration
// keys are FRESH runtime buffers; `!==` against an interned literal was true
// by handle identity even when the text matched. All node-derived.

#[test]
fn object_keys_element_equality() {
    // The browser_object_keys_harness self-check shape, round-tripped through
    // the SUPPORTED string-element array lane (`new Array(2)` + indexed store)
    // instead of `[]`+push — push into a `[]` literal is a pre-existing
    // silent no-op (bucket #10, Stage 4; see the Stage 1 triage doc),
    // unrelated to equality. node v26.5.0: prints "ok".
    let out = run_source(
        "const values = { \"b\": 1, \"a\": 2 };\nconst keys = new Array(2);\nlet i = 0;\nfor (const key of Object.keys(values)) {\n  keys[i] = key;\n  i = i + 1;\n}\nif (keys.length !== 2 || keys[0] !== \"b\" || keys[1] !== \"a\") {\n  throw new Error(\"unexpected Object.keys iteration semantics\");\n}\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn object_keys_loop_variable_equality() {
    // Direct compare of the for-of binding (no array round-trip).
    let out = run_source(
        "const o = { \"b\": 1, \"a\": 2 };\nlet seen = 0;\nfor (const key of Object.keys(o)) {\n  if (seen === 0 && key !== \"b\") { throw new Error(\"first key mismatch\"); }\n  seen = seen + 1;\n}\nif (seen !== 2) { throw new Error(\"key count mismatch\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

#[test]
fn for_in_key_equality() {
    // Spec 4a materialized for-in keys are repr-lifted `String`. UNQUOTED
    // object-literal keys (`{ b: 1, a: 2 }`): quoted keys never materialize a
    // Repr::Object shape (F-Stage1-4, pre-existing, fail-closed E5506 —
    // triage doc), and a `const`-bound for-in key hits the pre-existing "no
    // reserved local" reject (only var/let keys are in the admitted surface),
    // so the key is `let`-bound. Neither gap is equality-related; node
    // semantics are identical across the spellings. node v26.5.0: prints "ok".
    //
    // VERDICT (empirically settled, post-Task-5 review probe): behavior pin,
    // NOT load-bearing for Stage 1. Built the pre-__streq binary at 031fcda37
    // (last commit before the equality arm) and ran this exact fixture: it
    // also printed "ok" (exit 0, no stderr) — byte-identical to the
    // current-branch result. The for-in key's handle comes from Spec 4a's
    // interned key table and coincides with the literal "b"/"a" handle, so
    // `===` passes by pre-existing handle identity regardless of __streq.
    // Content-equality load-bearing coverage for for-in keys is carried by
    // nothing yet — recorded in throw-fallout-stage1-triage.md (Task 5
    // discoveries) as an open gap; candidate shape would need a for-in key
    // handle that does NOT coincide with any interned literal (not built
    // here).
    let out = run_source(
        "const o = { b: 1, a: 2 };\nlet matched = 0;\nfor (let k in o) {\n  if (k === \"b\") { matched = matched + 1; }\n  if (k === \"a\") { matched = matched + 1; }\n}\nif (matched !== 2) { throw new Error(\"for-in key equality failed\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}

// `object_entries_key_equality` (the fourth brief shape, `pair[0]` collected
// via `names.push(...)`) is intentionally ABSENT: its `[]`+push round-trip is
// blocked by the pre-existing push-no-op lane (bucket #10, Stage 4), a
// non-equality gap — recorded as expected-to-remain in
// docs/superpowers/followups/throw-fallout-stage1-triage.md rather than
// pinned wrong here.

// ---- Invariant 3 (no re-masking) + fail-closed backstops ----

#[test]
fn wrong_comparison_self_check_still_fails() {
    // Invariant 3: the fix must not re-silence self-check throws. A comparison
    // that is genuinely false must take the throw path and fail the run
    // (print-then-trap → non-zero exit).
    //
    // Empirically verified against the branch's debug binary (built at
    // f6c8f25ca, matching CARGO_BIN_EXE_kali's profile): the QUOTED key
    // `{ "b": 1 }` compiles and runs cleanly through this shape — no E5506
    // reject. Object.keys(...) produces "b"; "b" !== "nope" is true; the
    // throw fires: exit 1, stderr "Uncaught Error: honest failure", empty
    // stdout. The brief's quoted-key contingency did not trigger; kept
    // verbatim, no substitution to unquoted `{ b: 1 }` needed.
    let out = run_source(
        "const keys = Object.keys({ \"b\": 1 });\nif (keys[0] !== \"nope\") {\n  throw new Error(\"honest failure\");\n}\nconsole.log(\"unreachable ok\");\n",
    );
    assert!(
        !out.status.success(),
        "a false comparison's throw must fail the run; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        printed.contains("honest failure"),
        "throw's print-then-trap message missing; combined output: {printed}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("unreachable ok"),
        "execution continued past a throw"
    );
}

#[test]
fn mixed_tainted_equality_still_rejects_e3200() {
    // Fail-closed backstop: a tainted string against a NON-string operand
    // still hits the E3200 reject (the Task 2 arm requires BOTH sides
    // string-proven; the reject lane below it is retained for this residue).
    let out = run_source(
        "function f(s) {\n  if ((s + \"y\") == 5) {\n    console.log(1);\n  }\n}\nf(\"x\");\n",
    );
    assert!(
        !out.status.success(),
        "mixed tainted-string == number must stay rejected; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E3200"), "expected E3200, stderr: {stderr}");
}

#[test]
fn proven_string_vs_number_strict_equality_unchanged() {
    // Mixed lane pin (spec: out of scope, unchanged): an UNTAINTED proven
    // string against a number keeps today's handle-vs-number compare, which
    // agrees with node for `===` (false). node: "hi" === 5 → false. The `==`
    // coercion divergence ("5" == 5) is follow-up F-Stage1-1, NOT fixed here.
    //
    // Empirically verified: this expression is outside the Task 2 lane
    // (requires both sides string-proven), so branch == main for it. The
    // branch binary compiles it to the accidental-correct compare (does not
    // reject at the types layer): exit 0, stdout "ok\n". Kept the brief's
    // "ok path" pin verbatim; the E3200-reject reshape was not needed.
    let out = run_source(
        "let s = \"hi\";\nif (s === 5) { throw new Error(\"string equalled number\"); }\nconsole.log(\"ok\");\n",
    );
    assert_ok(&out);
}
