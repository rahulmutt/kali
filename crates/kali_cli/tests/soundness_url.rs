//! Stage P4 (URL + URLSearchParams lane) — Task 3: compile-time-parse
//! construction of both handles, the URL component reads (pure loads over the
//! parsed-at-compile-time arena struct), USP construction, and the shared
//! escape-restricted position gate (a raw read of a URL/USP handle fails closed
//! E5506). USP methods land in Task 4; here construction only proves it
//! compiles + runs.

use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_kali(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// Run `kali run`, assert it succeeded, and return stdout (caller trims).
fn run_kali_run(source: &str) -> String {
    let out = run_kali(source);
    assert!(
        out.status.success(),
        "expected success; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Run `kali run` expecting a fail-closed compile (nonzero exit); return stderr
/// so the caller can assert the diagnostic code (E5506).
fn run_kali_run_expect_error(source: &str) -> String {
    let out = run_kali(source);
    assert!(
        !out.status.success(),
        "expected a fail-closed compile (nonzero exit), got success; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn url_pathname_reads_the_parsed_component() {
    let src = "const u = new URL('https://example.com/browser?alpha=1#fragment');\nconsole.log(u.pathname);\n";
    assert_eq!(run_kali_run(src).trim(), "/browser");
}

#[test]
fn url_origin_search_hash_href_read_parsed_components() {
    let src = "const u = new URL('https://example.com/browser?alpha=1#fragment');\nconsole.log(u.origin);\nconsole.log(u.search);\nconsole.log(u.hash);\nconsole.log(u.href);\n";
    assert_eq!(
        run_kali_run(src).trim(),
        "https://example.com\n?alpha=1\n#fragment\nhttps://example.com/browser?alpha=1#fragment"
    );
}

#[test]
fn usp_construction_builds_and_runs() {
    // USP methods land in Task 4; here we only prove construction compiles/runs.
    let src = "const q = new URLSearchParams('alpha=1&beta=two+words');\nconsole.log('ok');\n";
    assert_eq!(run_kali_run(src).trim(), "ok");
}

#[test]
fn url_raw_print_fails_closed() {
    let src = "const u = new URL('https://x/');\nconsole.log(u);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_raw_print_fails_closed() {
    let src = "const q = new URLSearchParams('a=1');\nconsole.log(q);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_from_url_searchParams_raw_print_fails_closed() {
    let src = "const u = new URL('https://example.com/p?a=1');\nconsole.log(u.searchParams);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn non_literal_url_arg_fails_closed() {
    let src = "const s = 'https://x/';\nconst u = new URL(s);\nconsole.log(u.href);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must fail closed: {out:?}");
}

// --- Task 4: URLSearchParams query/mutation methods + composition -----------

#[test]
fn usp_get_returns_first_value() {
    let src = "const q = new URLSearchParams('alpha=1&beta=two+words');\nconsole.log(q.get('alpha'));\nconsole.log(q.get('beta'));\n";
    assert_eq!(run_kali_run(src).trim(), "1\ntwo words");
}

#[test]
fn usp_set_replaces_and_get_reflects_dynamic_value() {
    // Brief authored this with `String(count)`, but `String(x)` is fail-closed
    // on this branch (the G6 value-builtin deny-set: `{String,Boolean,toString,
    // split,JSON.stringify}` — a deliberate, shipped deny). The runtime int→
    // string concat coercion `'' + count` is the equivalent SUPPORTED dynamic-
    // string primitive: it produces the same runtime-computed string "7" (not a
    // const-fold — `q.set`/`q.get` are runtime scans), preserving the test's
    // intent (a dynamic value flows through `set` and `get` reflects it, and a
    // duplicate-free single entry via `getAll(...).length == 1`). Verified
    // byte-for-byte against `node`.
    let src = "let count = 7;\nconst q = new URLSearchParams('alpha=1&beta=x');\nq.set('beta', '' + count);\nconsole.log(q.get('beta'));\nconsole.log(q.getAll('beta').length);\n";
    assert_eq!(run_kali_run(src).trim(), "7\n1");
}

#[test]
fn usp_append_and_has() {
    // Dynamic booleans render `1`/`0` (the P3-ratified convention — see
    // `soundness_abort::aborted_flag_reads_zero_then_one`; node prints
    // true/false, a documented divergence never used in byte-for-byte
    // acceptance). `.has` flows through the same boolean-print lane as
    // `.aborted`, so it renders `1`/`0` too.
    let src = "const q = new URLSearchParams('alpha=1');\nq.append('gamma', 'g');\nconsole.log(q.has('gamma'));\nconsole.log(q.has('nope'));\n";
    assert_eq!(run_kali_run(src).trim(), "1\n0");
}

#[test]
fn url_search_params_composition_get() {
    let src = "const u = new URL('https://example.com/browser?alpha=1#fragment');\nconsole.log(u.searchParams.get('alpha'));\n";
    assert_eq!(run_kali_run(src).trim(), "1");
}

#[test]
fn usp_set_result_does_not_leak_the_store_handle() {
    // `.set` returns undefined (WHATWG); `__usp_set` carries the live tagged
    // store handle internally, but the call site drops it and renders the
    // void-method placeholder `0` — the raw i64 handle must NEVER escape as an
    // observable value (the Global Constraint). A leak would print a tagged
    // handle integer >= 2^62 here, not `0`.
    let src = "const q = new URLSearchParams('a=1');\nconsole.log(q.set('a', 'b'));\nconsole.log(q.get('a'));\n";
    assert_eq!(run_kali_run(src).trim(), "0\nb");
}

// --- Task 5: URLSearchParams.toString() serialization ------------------------

#[test]
fn usp_tostring_serializes_form_urlencoded() {
    let src =
        "const q = new URLSearchParams('alpha=1&beta=two+words');\nconsole.log(q.toString());\n";
    // application/x-www-form-urlencoded: space -> '+', pairs joined by '&'.
    assert_eq!(run_kali_run(src).trim(), "alpha=1&beta=two+words");
}

#[test]
fn usp_tostring_after_mutation() {
    let src = "const q = new URLSearchParams('alpha=1');\nq.append('g', 'a b');\nconsole.log(q.toString());\n";
    assert_eq!(run_kali_run(src).trim(), "alpha=1&g=a+b");
}

#[test]
fn unknown_method_on_usp_fails_closed() {
    let src = "const q = new URLSearchParams('a=1');\nq.sort();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

// --- Task 6: fail-closed enumeration wave (store sites + generic sinks) ------
//
// The standing lesson from 6 prior stages executed up front: pin EVERY store
// site and generic value sink for BOTH handle classes NOW. Each pin proves the
// Task-3 position gate (allowlist at the identifier/member choke point) denies
// the position; any leak is fixed AT the choke, never per-sink.

/// Shared two-binding prelude for the sink pins (one URL, one USP).
fn sink_src(line: &str) -> String {
    format!(
        "const u = new URL('https://example.com/browser?alpha=1');\nconst q = new URLSearchParams('a=1');\n{line}\n"
    )
}

#[test]
fn url_string_concat_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("console.log(\"v=\" + u);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_string_concat_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("console.log(\"v=\" + q);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_template_interp_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("console.log(`v=${u}`);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_arithmetic_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("console.log(u + 1);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_identity_compare_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("console.log(u === u ? 1 : 0);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_json_stringify_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("console.log(JSON.stringify(q));"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_return_position_fails_closed() {
    let src = "function f() { const u = new URL('https://x/'); return u; }\nf();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_argument_position_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("function f(x) { return 1; }\nf(q);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_object_literal_field_fails_closed() {
    // Store-site gating: the deny must fire even though the object is never
    // observed afterwards — a silent green compile here IS the leak.
    let stderr = run_kali_run_expect_error(&sink_src("const o = { h: u };"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_array_element_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("const a = [q];"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_growable_push_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("const a = [];\na.push(q);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_computed_member_read_fails_closed() {
    let stderr =
        run_kali_run_expect_error(&sink_src("const k = \"pathname\";\nconsole.log(u[k]);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_member_write_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("u.pathname = \"/x\";"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_captured_in_deferred_callback_fails_closed() {
    let src = "function m() { const q = new URLSearchParams('a=1'); setTimeout(function() { q.get('a'); }, 0); }\nm();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_unknown_member_read_fails_closed() {
    let stderr = run_kali_run_expect_error(&sink_src("console.log(u.protocol);"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn inline_url_construct_in_argument_position_is_placeholder_zero() {
    // HONEST-BEHAVIOR pin (flagged, not a deny pin): an inline value-position
    // `new URL(...)` that never became a `const` binding takes the PRE-EXISTING
    // non-const construct placeholder path (Task-3 documented class: cf.
    // `let u = new URL(...)` → placeholder) — the argument lowers to `0`, so NO
    // raw handle escapes (0 is not a struct pointer / tagged store). Fixing the
    // class (deny or real lowering) is outside the Task-6 choke-point scope;
    // this pin makes today's behavior visible so a future change is deliberate.
    let src = "function f(x) { return x; }\nconsole.log(f(new URL('https://x/')));\n";
    assert_eq!(run_kali_run(src).trim(), "0");
}
