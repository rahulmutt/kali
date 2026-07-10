use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-nullish-reject-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.ts");
    std::fs::write(&path, src).unwrap();
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

// `??=` lowers with a FALSY test (`I64Eqz`) and null/undefined both lower to
// i64 `0` for a scalar, so kali cannot distinguish `null` from `0` — a correct
// nullish test is unrepresentable without a nullable-scalar type. Must reject
// fail-closed, never miscompile `let x = 0; x ??= 1` to `1` (node: `0`).
#[test]
fn scalar_local_nullish_assign_rejects() {
    let out = run_source("let x = 0;\nx ??= 1;\nconsole.log(x);\n");
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
}

#[test]
fn numeric_param_nullish_assign_rejects() {
    let out = run_source("function f(p) { p ??= 1; return p; }\nconsole.log(f(0));\n");
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
}

// The ONLY surviving `??=` lowering: a for-in-key ALIAS binding, which has a
// real null sentinel (`-1`) so null and a valid key ordinal stay distinct.
#[test]
fn for_in_key_alias_nullish_assign_still_runs() {
    let out = run_source(
        "var table = { a: 1, b: 2 };\nvar last = null;\nfor (var c in table) {\n  last = c;\n}\nlast ??= null;\nif (last) { console.log(\"set\"); }\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "set\n");
}

// POSITIVE-BEHAVIOR pin (despite this file's name): after Spec 7 Task 3 the ONLY
// `??=` shape that reaches codegen is a for-in-key alias, whose null sentinel is
// `-1` (key ordinals are 0-based). The `??=` codegen arm previously tested the
// nullish condition with `I64Eqz` — a FALSY test that fires on ordinal `0`.
// This pins that the surviving `??=` lane is SENTINEL-AWARE (`== -1`), not
// falsy: `last` holds the first key's ordinal `0`, so `last ??= null` must NOT
// fire (0 is a valid key, not null) and `table[last]` must read key 0's value.
//
// DISCRIMINATION: this pin's RHS is `null` (the only RHS the resolve admit now
// accepts), and a fired `??= null` stores the `-1` sentinel (see the null-RHS
// store pin below). So under a regressed falsy `I64Eqz` compare, alias == 0
// FIRES, stores `-1`, and `table[-1]` reads out-of-table garbage (≠ 7) — the
// pin still catches the falsy-compare bug even though the RHS is null.
// Lives here because this file owns the whole `??=` surface (rejects + the one
// live lane).
#[test]
fn for_in_key_alias_nullish_assign_sentinel_aware_not_falsy() {
    let out = run_source(
        "var table = {a:7, b:8};\nvar last = null;\nfor (var c in table) { last = c; break; }\nlast ??= null;\nconsole.log(table[last]);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}

// A FIRED `??= null` must store the `-1` null sentinel, not a raw `0` (the
// generic null lowering): the sentinel-aware compare (pin above) correctly
// ENTERS the assignment branch when the alias is genuinely null (-1), and a
// bare-`null` RHS emitted via the generic path would store `0` — a VALID key
// ordinal — flipping the alias's truthiness from false to true. (Pre-sentinel,
// the falsy compare never took the branch on -1, which was accidentally
// correct.) Mirrors the `=` arm's null-store special case (literal.rs).
// Node prints `falsy`; a raw-0 store prints `truthy`.
#[test]
fn for_in_key_alias_fired_nullish_null_rhs_stores_sentinel() {
    let out = run_source(
        "var table = {a:1};\nvar last = null;\nvar go = 0;\nfor (var c in table) { if (go) { last = c; } }\nlast ??= null;\nif (last) { console.log(\"truthy\"); } else { console.log(\"falsy\"); }\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "falsy\n");
}

// A for-in-key alias `??=` with a NON-NULL RHS must reject: if it fired it
// would store a raw number into an ordinal-repr binding, and every downstream
// ordinal consumer (`table[last]`, truthiness) diverges from node (which holds
// a string key or the raw number, e.g. `table[last]` after `??= 1` → node
// `undefined`). The admit is narrowed to a `null`-literal RHS — the only RHS
// whose fired store (-1 sentinel) is representable.
#[test]
fn for_in_key_alias_nullish_assign_non_null_rhs_rejects() {
    let out = run_source(
        "var table = {a:7, b:8};\nvar last = null;\nfor (var c in table) { last = c; break; }\nlast ??= 5;\nconsole.log(table[last]);\n",
    );
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// `??= undefined` must ALSO reject: bare `undefined` parses as an IDENTIFIER,
// not a literal, and codegen's null recognizer (`is_null_or_undefined_literal`)
// only matches LITERAL nodes — so an admitted `??= undefined` would slip past
// the sentinel-store special case into the generic emit and store raw `0` (a
// valid key ordinal): kali `truthy`, node `falsy`. Until BOTH recognizer twins
// agree on bare `undefined`, the resolve admit accepts only the `null` literal
// — fail-closed.
#[test]
fn for_in_key_alias_nullish_assign_undefined_rhs_rejects() {
    let out = run_source(
        "var table = {a:1};\nvar last = null;\nvar go = 0;\nfor (var c in table) { if (go) { last = c; } }\nlast ??= undefined;\nif (last) { console.log(\"truthy\"); } else { console.log(\"falsy\"); }\n",
    );
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
