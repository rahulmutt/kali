//! Stage 2 (throw-fallout) node-parity pins: static enumeration over
//! quoted keys, ES integer-first ordering, and delete+reinsert timelines.
//! Every expectation is node-derived (fresh `node` run on the same
//! source), NEVER reverse-engineered from kali's output.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // Per-process AtomicU64 counter slug convention (object_delete_gate.rs,
    // runtime_string_value_flow.rs) — a src-length-only slug previously
    // collided under concurrency.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-static-enum-stage2-{}-{}-{}",
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

/// Run and assert success; return stdout.
fn run_expect_ok(src: &str) -> String {
    let out = run_source(src);
    assert!(
        out.status.success(),
        "expected success, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Run and capture (stdout, exit code) without asserting outcome.
fn run_capture(src: &str) -> (String, i32) {
    let out = run_source(src);
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn delete_reinsert_enumeration_matches_node() {
    // The runtime_smoke.rs:954 core shape. node-verified fresh: prints
    // "ok" (exit 0).
    let stdout = run_expect_ok(
        "const r = { \"a\": 1, \"b\": 2, \"c\": 3 };\n\
         delete r.b;\n\
         r.b = 4;\n\
         const ks = Object.keys(r);\n\
         const es = Object.entries(r);\n\
         const vs = Object.values(r);\n\
         if (ks.length !== 3 || ks[0] !== 'a' || ks[1] !== 'c' || ks[2] !== 'b') throw new Error('keys');\n\
         if (es.length !== 3 || es[2][0] !== 'b' || es[2][1] !== 4) throw new Error('entries');\n\
         if (vs.length !== 3 || vs[0] !== 1 || vs[1] !== 3 || vs[2] !== 4) throw new Error('values');\n\
         console.log('ok');",
    );
    assert_eq!(stdout, "ok\n");
}

#[test]
fn quoted_and_numeric_like_keys_enumerate_in_es_order() {
    // The browser_reflect_own_keys core object. node-verified fresh:
    // prints "ok" (exit 0) via Object.keys, Reflect.ownKeys, and for..in
    // alike.
    let stdout = run_expect_ok(
        "const o = { \"b\": 1, \"2\": 2, \"a\": 3, \"1\": 4 };\n\
         const keys = Object.keys(o);\n\
         if (keys.length !== 4 || keys[0] !== '1' || keys[1] !== '2' || keys[2] !== 'b' || keys[3] !== 'a') throw new Error('keys order');\n\
         const own = Reflect.ownKeys(o);\n\
         if (own.length !== 4 || own[0] !== '1' || own[3] !== 'a') throw new Error('ownKeys order');\n\
         let seen = '';\n\
         for (var k in o) { seen = seen + k; }\n\
         if (seen !== '12ba') throw new Error('for-in order');\n\
         console.log('ok');",
    );
    assert_eq!(stdout, "ok\n");
}

#[test]
fn store_only_mutation_folds_fresh_values() {
    // No delete at all — the timeline must also kill the stale-VALUES
    // fold. node-verified fresh: prints "2" (exit 0).
    let stdout = run_expect_ok(
        "const r = { a: 1 };\n\
         r.a = 2;\n\
         const vs = Object.values(r);\n\
         if (vs[0] !== 2) throw new Error('stale value');\n\
         console.log(vs[0]);",
    );
    assert_eq!(stdout, "2\n");
}

#[test]
fn re_mask_guard_delete_reinsert_self_check_still_fires() {
    // Deliberately wrong expectation (ks[2] is 'b', not 'c'): the throw
    // MUST fire and the run MUST fail. If this exits 0, a fix re-masked
    // the self-check throw (program Invariant 3 violation). node-verified
    // fresh: throws, exit 1.
    let (stdout, code) = run_capture(
        "const r = { \"a\": 1, \"b\": 2, \"c\": 3 };\n\
         delete r.b;\n\
         r.b = 4;\n\
         const ks = Object.keys(r);\n\
         if (ks[2] !== 'c') throw new Error('expected mismatch');\n\
         console.log('MUST NOT PRINT');",
    );
    assert_ne!(code, 0);
    assert!(!stdout.contains("MUST NOT PRINT"), "stdout: {stdout}");
}

#[test]
fn re_mask_guard_es_order_self_check_still_fires() {
    // node-verified fresh: throws, exit 1.
    let (stdout, code) = run_capture(
        "const o = { \"b\": 1, \"1\": 4 };\n\
         const keys = Object.keys(o);\n\
         if (keys[0] !== 'b') throw new Error('expected mismatch: ES order puts 1 first');\n\
         console.log('MUST NOT PRINT');",
    );
    assert_ne!(code, 0);
    assert!(!stdout.contains("MUST NOT PRINT"), "stdout: {stdout}");
}
