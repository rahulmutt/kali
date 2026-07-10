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
