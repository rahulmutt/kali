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
    // Dual deny: the G6 value-builtin deny-set rejects `JSON.stringify` for ALL
    // inputs, so E5506 alone would pass even if the URL/USP gate regressed.
    // Assert the URL/USP-specific identifier-choke message TOO, so the URL gate
    // cannot hide behind G6.
    let stderr = run_kali_run_expect_error(&sink_src("console.log(JSON.stringify(q));"));
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("URL/URLSearchParams handle cannot be read in this position"),
        "URL/USP identifier-choke deny missing (G6 alone is not enough): {stderr}"
    );
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

// --- Task 6 review-fix wave: receiver-path shapes + getAll binding ----------
//
// Review probes falsified the member-receiver arm's "ANY method denies" claim:
// the generic call fallback DROPS the receiver (never emits it), so a receiver
// path the recognizers don't admit reached the placeholder terminals with NO
// choke firing — silent 0. Closed by ROOT PROVENANCE at the terminal choke:
// the member chain (dot AND computed, any depth) is walked to its root
// identifier; a URL/USP root denies by construction.

#[test]
fn url_computed_searchparams_method_fails_closed() {
    // Computed receiver at plain `_start` scope: `u['searchParams'].get(...)`
    // silently printed 0 pre-fix (node prints 1).
    let src = "const u = new URL('https://example.com/browser?alpha=1');\nconsole.log(u['searchParams'].get('alpha'));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_computed_searchparams_method_from_fn_fails_closed() {
    let src = "const u = new URL('https://example.com/browser?alpha=1');\nfunction f() { console.log(u['searchParams'].get('alpha')); }\nf();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_computed_searchparams_method_captured_fails_closed() {
    let src = "function m() { const u = new URL('https://example.com/browser?alpha=1'); setTimeout(function() { console.log(u['searchParams'].get('alpha')); }, 0); }\nm();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn url_two_hop_member_method_fails_closed() {
    // 2-hop dot chain over a URL root from inside a fn: silently 0'd pre-fix.
    let src = "const u = new URL('https://example.com/browser?alpha=1');\nfunction f() { console.log(u.searchParams.x.get('alpha')); }\nf();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_getall_binding_fails_closed() {
    // Review IMPORTANT: `const a = q.getAll('a'); a.length` silently printed 0
    // (node prints 2) — the binding loses the growable classification at the
    // declarator. Denied at the declarator choke; the direct
    // `q.getAll(k).length` composition keeps working (see
    // `usp_set_replaces_and_get_reflects_dynamic_value`).
    let src = "const q = new URLSearchParams('a=1&a=2');\nconst a = q.getAll('a');\nconsole.log(a.length);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn inline_url_construct_in_argument_position_is_placeholder_zero() {
    // DELIBERATE PIN FLIP (stage-review F10, adjudicated deny-now): this was
    // the Task-6 HONEST-BEHAVIOR pin recording that an inline value-position
    // `new URL(...)` lowered to the silent `0` placeholder. The fix-wave
    // upgraded the class from placeholder-zero to E5506 at the `emit_call`
    // ctor choke (a deny upgrade — silent-wrong → fail-closed), so the pin
    // now asserts the deny. Name kept so the flip is visible in history.
    let src = "function f(x) { return x; }\nconsole.log(f(new URL('https://x/')));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn let_url_construct_fails_closed() {
    // Stage-review F10: a `let`/`var` declarator is outside the admitted
    // `const` intercept; its init falls to the generic call path where the
    // ctor deny fires (previously: bound `0`, `u.pathname` printed 0; node
    // prints `/p`).
    let src = "let u = new URL('https://x/p');\nconsole.log(u.pathname);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn inline_url_member_read_fails_closed() {
    // Stage-review F10: an inline member read over an un-bound construction
    // (`new URL(s).pathname`) previously printed 0 with zero diagnostics.
    let src = "console.log(new URL('https://x/p').pathname);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

// --- Task 8 fix wave: stage-review findings (C-1..I-9, F11) ------------------

#[test]
fn url_searchparams_composition_mutation_fails_closed() {
    // Stage-review C-1: mutating through the composition
    // (`u.searchParams.set/append`) desyncs the compile-time-frozen
    // `u.search`/`u.href` slots from the live store (kali read stale `?a=1`
    // where node reads `?a=2&b=z`). Composition is read-only this phase —
    // both mutators deny E5506; standalone-USP mutation is unchanged.
    for method_call in [
        "u.searchParams.set('a', '2')",
        "u.searchParams.append('b', 'z')",
    ] {
        let src = format!(
            "const u = new URL('https://example.com/p?a=1');\n{method_call};\nconsole.log(u.search);\n"
        );
        let stderr = run_kali_run_expect_error(&src);
        assert!(stderr.contains("E5506"), "case {method_call}: {stderr}");
    }
}

#[test]
fn usp_leading_question_mark_stripped() {
    // Stage-review C-2 (WHATWG): exactly one leading `?` is stripped by the
    // constructor. Before the fix `get('a')` missed (null sentinel printed 0)
    // and toString rendered `%3Fa=1`. Node oracle: `1` / `a=1`.
    let src =
        "const q = new URLSearchParams('?a=1');\nconsole.log(q.get('a'));\nconsole.log(q.toString());\n";
    assert_eq!(run_kali_run(src).trim(), "1\na=1");
}

#[test]
fn usp_append_evaluates_args_before_mutation() {
    // Stage-review C-3: `append` must evaluate BOTH argument expressions
    // before mutating the store (JS argument-evaluation order). The reentrant
    // value expression observes the PRE-append state: node prints `no` /
    // `a=1&b=no`; the old inline two-push lowering pushed the key first and
    // printed `yes`. Closed by the single `__usp_append(store, key, val)`
    // synthetic (mirrors `.set`'s already-atomic shape).
    let src = "const q = new URLSearchParams('a=1');\nq.append('b', q.has('b') ? 'yes' : 'no');\nconsole.log(q.get('b'));\nconsole.log(q.toString());\n";
    assert_eq!(run_kali_run(src).trim(), "no\na=1&b=no");
}

#[test]
fn url_binding_name_shadowing_fails_closed() {
    // Stage-review C-4: URL/USP provenance is name-keyed and FLAT (no block
    // scoping). A block-scoped shadow of a URL binding read 0 where node
    // reads the shadow's value (7). Denied at the declarator choke; the
    // REVERSE order (generic binding first, URL shadow second) is refused at
    // the intercept and falls to the F10 ctor deny — both directions E5506.
    let shadow_over_url = "const u = new URL('https://x/first');\n{\n  const u = { pathname: 7 };\n  console.log(u.pathname);\n}\n";
    let stderr = run_kali_run_expect_error(shadow_over_url);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");

    let url_over_generic = "const u = { pathname: 7 };\n{\n  const u = new URL('https://x/first');\n  console.log(u.pathname);\n}\n";
    let stderr = run_kali_run_expect_error(url_over_generic);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_get_absent_prints_null() {
    // Stage-review I-5 (materialization): `q.get(absent)` returns the 0
    // null-sentinel; the print lane substitutes the interned `"null"` handle
    // at runtime, so kali prints node's `null` (previously `0`). The present
    // key stays a plain handle pass-through (flagship admitted surface).
    let src = "const q = new URLSearchParams('a=1');\nconsole.log(q.get('nope'));\nconsole.log(q.get('a'));\n";
    assert_eq!(run_kali_run(src).trim(), "null\n1");
}

#[test]
fn usp_get_absent_concat_renders_null() {
    // Stage-review I-5 (concat lane): `'v=' + q.get(absent)` must render
    // node's `v=null`, not `v=` (the 0 sentinel concatenated as empty).
    let src = "const q = new URLSearchParams('a=1');\nconsole.log('v=' + q.get('a'));\nconsole.log('v=' + q.get('nope'));\n";
    assert_eq!(run_kali_run(src).trim(), "v=1\nv=null");
}

#[test]
fn usp_get_result_length_fails_closed() {
    // Stage-review I-6: `q.get(k).length` statically rendered the CALL node's
    // child count (2; node prints the value's length, 1 here). No static or
    // ASCII-provable runtime length exists for a USP string result — the
    // static render bails and the member arm denies E5506.
    let src = "const q = new URLSearchParams('a=1');\nconsole.log(q.get('a').length);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_getall_element_read_fails_closed() {
    // Stage-review I-7: `q.getAll('a')[0]` silently printed 0 (node prints
    // `1`). Element reads of the fresh growable result are unsupported this
    // phase — denied at the computed-member arm keyed on the same
    // `is_usp_getall_call` recognizer as the admitted `.length` composition.
    let src = "const q = new URLSearchParams('a=1&a=22');\nconsole.log(q.getAll('a')[0]);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_get_in_condition_fails_closed() {
    // Stage-review I-8: a bare `q.get(k)` result in condition position tested
    // HANDLE truthiness — `'a='` (empty-string value) was truthy where node
    // is falsy (and the absent-key case matched node only by the sentinel
    // coincidence). Denied at the shared condition choke
    // (`reject_string_condition`), which covers if/while/for/ternary at once.
    for src in [
        "const q = new URLSearchParams('a=');\nif (q.get('a')) { console.log('t'); } else { console.log('f'); }\n",
        "const q = new URLSearchParams('a=');\nconsole.log(q.get('a') ? 1 : 0);\n",
        "const q = new URLSearchParams('a=');\nwhile (q.get('a')) { console.log('x'); }\n",
    ] {
        let stderr = run_kali_run_expect_error(src);
        assert!(stderr.contains("E5506"), "case {src}: {stderr}");
    }
}

#[test]
fn url_binding_reassignment_fails_closed() {
    // Stage-review I-9: `u = 5` overwrote the binding's handle local; the
    // admitted `u.pathname` read then wild-loaded from address 5+16 and
    // printed 0 (node throws on const reassignment). Denied at the
    // identifier-assignment choke keyed on all four URL/USP provenance
    // classifiers — the write-position twin of the member-write gate.
    let src = "const u = new URL('https://example.com/p');\nu = 5;\nconsole.log(u.pathname);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");

    let usp = "const q = new URLSearchParams('a=1');\nq = 5;\nconsole.log(q.get('a'));\n";
    let stderr = run_kali_run_expect_error(usp);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_tostring_multibyte_percent_encoding() {
    // Stage-review F11 (adjudicated pin-now, free pin): multibyte and
    // reserved-character percent-encoding is already node-identical.
    // Node oracle (verified 2026-07-21): `a=caf%C3%A9&sym=%26%3D` / `café`
    // — the `é` round-trips (decode at parse, re-encode in toString) and the
    // reserved `&`/`=` in the VALUE stay encoded.
    let src = "const q = new URLSearchParams('a=caf%C3%A9&sym=%26%3D');\nconsole.log(q.toString());\nconsole.log(q.get('a'));\n";
    assert_eq!(run_kali_run(src).trim(), "a=caf%C3%A9&sym=%26%3D\ncafé");
}

// --- Task 7: acceptance (byte-for-byte vs node) ------------------------------

/// Runs `node <main_path>` with `dir` as the working directory. The acceptance
/// fixture is valid, unmodified ES that node executes directly, so a straight
/// `node` run is a faithful oracle for "what should this program print".
/// (Copied from `soundness_abort.rs` / `module_namespace_link.rs`.)
fn node_output(dir: &std::path::Path, main_path: &std::path::Path) -> std::process::Output {
    Command::new("node")
        .current_dir(dir)
        .arg(main_path)
        .output()
        .expect("run node")
}

/// Task-7 acceptance fix, pinned: `q.get(k)` / `q.toString()` results admitted
/// into the `__streq` CONTENT-equality lane (`is_usp_string_call`, mirroring
/// the env-get precedent). Before the fix the usp-vs-literal compare fell
/// through Stage 1's both-proven-string lane to a raw `i64.eq` HANDLE-IDENTITY
/// compare — every Task 4-6 pin passed only because parse-time/literal-set
/// values and the compared literal dedup to the SAME interned handle, while a
/// DYNAMICALLY-set value (`q.set(k, '' + n)`) gets a fresh `string_concat`
/// handle: equal content, different handle, silently-wrong branch (observed as
/// the acceptance fixture's untaken USP throw firing). Node-oracle: node
/// prints ok for every line here.
#[test]
fn usp_get_after_dynamic_set_content_equality() {
    let src = "let n = 1;\nconst q = new URLSearchParams('a=x');\nq.set('a', '' + n);\nif (q.get('a') === '1') { console.log('eq'); } else { console.log('ne'); }\nif (q.get('a') !== '1') { console.log('ne2'); } else { console.log('eq2'); }\n";
    assert_eq!(run_kali_run(src).trim(), "eq\neq2");
}

/// Sibling pin: `.append` with a dynamic value, plus `.toString()` in the
/// equality lane (a `__usp_tostring` result is ALWAYS a fresh global-heap
/// handle, so identity compare would be wrong for it on every path, not just
/// the dynamic-set one).
#[test]
fn usp_dynamic_append_and_tostring_content_equality() {
    let src = "let n = 1;\nconst q = new URLSearchParams('a=x');\nq.append('g', '' + n);\nif (q.get('g') === '1') { console.log('append-eq'); }\nif (q.toString() === 'a=x&g=1') { console.log('tostring-eq'); }\n";
    assert_eq!(run_kali_run(src).trim(), "append-eq\ntostring-eq");
}

/// Stage P4 acceptance: the web-baseline smoke prefix (P2 structuredClone + P3
/// abort + Stage D events) EXTENDED with the URL/USP block, byte-for-byte
/// against a real `node` oracle. This is the stage's integration evidence: the
/// URL/USP surface works in composition (dynamic values through `append`/`set`,
/// `get`/`getAll().length`/`has` in taken-path if-conditions, and the
/// `u.searchParams.get` composition), not just in the isolated per-lane pins.
///
/// FIXTURE PROVENANCE: the live web-baseline fixture
/// `structured_clone_and_event_primitives_source` (runtime_smoke.rs) MINUS the
/// `new Event('tick')`/`event.type` block (pre-existing out-of-scope gap — see
/// `soundness_abort.rs` acceptance provenance note) and MINUS the TextEncoder
/// tail (P5 scope), wrapped `function main() { ... } main();` (module-scope
/// capture stays fail-closed by design; node prints identically — see the P3
/// acceptance's recorded wrap adaptation).
///
/// FIXTURE ADAPTATIONS (recorded for the Task-8 doc entry; each keeps
/// node-identical semantics — node takes the same branches and prints the same
/// bytes for the adapted shapes):
///   1. `String(count)` (3 sites in the brief's text) — `String(x)` is
///      FAIL-CLOSED on this branch (G6 value-builtin deny-set). In the two
///      ARGUMENT sites (`append`/`set`) the Task-4-ratified dynamic-string
///      shape `'' + count` substitutes (same runtime-computed "1").
///   2. The COMPARISON site `query.get('beta') !== String(count)` becomes
///      `query.get('beta') !== '1'`: a runtime-string vs DYNAMIC-string
///      compare (`!== ('' + count)`) is E3200 fail-closed by design (Stage 1
///      lowered only literal/proven operands into `__streq`), and `count` is
///      deterministically `1` here (single dispatched listener), so the
///      literal-RHS compare — the Stage-1-ratified shape — is value-identical
///      in node.
#[test]
fn acceptance_web_baseline_with_url_matches_node_byte_for_byte() {
    let src = r#"function main() {
  const original = { count: 1, values: [1, 2, 3] };
  const cloned = structuredClone(original);
  if (cloned === original || cloned.values === original.values) {
    throw new Error('structuredClone should deep-clone object graphs');
  }
  original.values.push(4);
  if (cloned.count !== 1 || cloned.values.join(',') !== '1,2,3') {
    throw new Error('unexpected structuredClone result');
  }
  const controller = new AbortController();
  if (!(controller.signal instanceof AbortSignal)) {
    throw new Error('expected AbortSignal from AbortController');
  }
  const target = new EventTarget();
  let count = 0;
  target.addEventListener('tick', () => {
    count += 1;
    controller.abort();
  });
  const dispatched = target.dispatchEvent(new CustomEvent('tick'));
  if (!dispatched || count !== 1 || !controller.signal.aborted) {
    throw new Error('unexpected event primitive behavior');
  }
  const query = new URLSearchParams('alpha=1&beta=two+words');
  query.append('gamma', '' + count);
  query.set('beta', '' + count);
  if (query.get('alpha') !== '1' || query.get('beta') !== '1' || query.getAll('beta').length !== 1 || !query.has('gamma')) {
    throw new Error('unexpected URLSearchParams behavior ' + query.toString());
  }
  const browserUrl = new URL('https://example.com/browser?alpha=1#fragment');
  if (browserUrl.origin !== 'https://example.com' || browserUrl.pathname !== '/browser' || browserUrl.search !== '?alpha=1' || browserUrl.hash !== '#fragment' || browserUrl.searchParams.get('alpha') !== '1') {
    throw new Error('unexpected URL behavior ' + browserUrl.href);
  }
  console.log('web baseline url ok');
}
main();
"#;
    let dir = tempdir().expect("tempdir");
    let main_path = dir.path().join("main.js");
    fs::write(&main_path, src).expect("write");
    let kali = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");
    assert!(
        kali.status.success(),
        "kali stderr: {}",
        String::from_utf8_lossy(&kali.stderr)
    );
    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&kali.stdout),
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
    assert_eq!(
        String::from_utf8_lossy(&kali.stdout).trim(),
        "web baseline url ok"
    );
}
