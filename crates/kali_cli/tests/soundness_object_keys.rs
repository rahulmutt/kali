use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-objkeys-{}-{}-{}",
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

// `{type: 3}` then `obj.type`: the object-literal parser only accepted
// Identifier/String/Numeric/computed keys; a keyword key hit the `_ =>`
// arm which silently DISCARDED the whole property, so the read yielded 0.
// Member access `obj.type` was fixed earlier (is_property_name_token);
// this closes the literal side with the same key set.
#[test]
fn keyword_key_in_object_literal_round_trips() {
    let out = run_source("const o = { type: 3, if: 4 };\nconsole.log(o.type + o.if);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "7");
}

// The old `_ =>` arm silently dropped ANY unrecognized property form.
// Fail-closed now: unknown forms are a hard reject, not a silent drop.
#[test]
fn unrecognized_property_form_rejects() {
    // `...spread` in an object literal is unsupported surface.
    let out = run_source("const a = { x: 1 };\nconst o = { ...a };\nconsole.log(o.x);\n");
    assert!(
        !out.status.success(),
        "spread property must reject, got: {out:?}"
    );
}

// A single-element ARRAY literal `[x]` is a text-less one-child `Value`,
// structurally identical to a transparent sequence/grouping wrapper. Codegen's
// receiver classifiers (`unwrap_transparent`, `resolve_static_object_identity_value`)
// tunneled straight through it into element `x`, so `.length` read `x`'s STRING
// length instead of the array length 1 — a silent wrong answer (exit 0).
// (throw-fallout Stage 2 review fix.)
#[test]
fn single_element_string_array_length_reports_one() {
    // node: 1 (the array has one element). Pre-fix kali: 6 ("abcdef".length).
    let out = run_source("const a = [\"abcdef\"];\nconsole.log(a.length);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}

// The folded `Object.keys(singleKeyObject)` is a one-element array literal, so
// it hit the exact same single-element tunneling. `"OTHER".length` is 5, which
// silently replaced the array length 1.
#[test]
fn single_key_object_keys_length_reports_one() {
    // node: 1. Pre-fix kali: 5 ("OTHER".length).
    let out = run_source("console.log(Object.keys({ OTHER: \"2\" }).length);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}

// The delete-timeline lane rebuilds the literal down to a single key, then folds
// `Object.keys` over it — the enumeration-fold + delete-timeline shape this
// review fix exists to make honest.
#[test]
fn single_key_delete_timeline_keys_length_reports_one() {
    // node: 1. Pre-fix kali: 5 ("OTHER".length after delete).
    let out = run_source(
        "const cfg = { PATH: \"1\", OTHER: \"2\" };\ndelete cfg.PATH;\nconsole.log(Object.keys(cfg).length);\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
}

// The `[0]` element read and the `Object.values` variant were already correct;
// pin them so the fix does not regress the neighboring lanes.
#[test]
fn single_key_delete_timeline_neighbors_stay_correct() {
    let out = run_source(
        "const cfg = { PATH: \"1\", OTHER: \"2\" };\ndelete cfg.PATH;\nconsole.log(Object.keys(cfg)[0]);\nconsole.log(Object.values(cfg)[0]);\nconsole.log(Object.values(cfg).length);\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "OTHER\n2\n1");
}
