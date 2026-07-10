use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-undef-{}-{}-{}",
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

// `last = undefined` on a for-in key alias must store the -1 null sentinel
// (matching `last = null`), NOT 0 — ordinal 0 is the FIRST KEY, so a 0
// store flips `if (last)` from false to true. node prints "none".
//
// NOTE: the for-in key binding uses `let k` rather than `const k` — a
// `const`-bound for-in key hits an unrelated pre-existing local-reservation
// gap ("for..in key binding 'k' has no reserved local", independent of this
// task's recognizer fix) and is not part of the admitted surface. `var`/`let`
// key bindings are the form used throughout the existing for-in-key test
// suite (see `crates/kali_cli/tests/runtime_forin.rs`).
#[test]
fn forin_alias_undefined_reassign_reads_falsy() {
    let out = run_source(
        "const o = { a: 1, b: 2 };\nlet last = null;\nfor (let k in o) { last = k; }\nlast = undefined;\nif (last) { console.log(\"some\"); } else { console.log(\"none\"); }\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "none");
}

// Declarator form: `let last = undefined` (identifier, not null literal).
#[test]
fn forin_alias_undefined_init_reads_falsy() {
    let out = run_source(
        "const o = { a: 1 };\nlet last = undefined;\nfor (let k in o) { if (false) { last = k; } }\nif (last) { console.log(\"some\"); } else { console.log(\"none\"); }\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "none");
}

// With the recognizer twins unified, `??= undefined` no longer needs its
// disagreement reject: it stores the -1 sentinel exactly like `??= null`.
// node: last is "b" (non-nullish) after the loop, so `??=` does not fire and
// `o[last]` reads `o.b`. (Reads through the computed-index position
// `o[last]`, not a bare `console.log(last)`, which hits an unrelated
// pre-existing materialization restriction on `??=`-assigned aliases —
// see `console.log(table[last])` in `nullish_assign_reject.rs` for the same
// idiom.)
#[test]
fn forin_alias_nullish_assign_undefined_rhs_admits() {
    let out = run_source(
        "const o = { a: 1, b: 2 };\nlet last = null;\nfor (let k in o) { last = k; }\nlast ??= undefined;\nconsole.log(o[last]);\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");
}
