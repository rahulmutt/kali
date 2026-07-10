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
// nullish condition with `I64Eqz` — a FALSY test that fires on ordinal `0` — so
// `last`, holding the first key's ordinal `0`, was wrongly overwritten with `5`
// and `table[5]` read garbage (printed `0`; node prints `7`). This pins that the
// surviving `??=` lane is SENTINEL-AWARE (`== -1`), not falsy. Lives here because
// this file owns the whole `??=` surface (rejects + the one live lane).
#[test]
fn for_in_key_alias_nullish_assign_sentinel_aware_not_falsy() {
    let out = run_source(
        "var table = {a:7, b:8};\nvar last = null;\nfor (var c in table) { last = c; break; }\nlast ??= 5;\nconsole.log(table[last]);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}
