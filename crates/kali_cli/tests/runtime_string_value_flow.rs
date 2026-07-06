use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // A per-process AtomicU64 counter makes the slug unique even when two
    // sources share a length (sharing a length previously collided the dir and
    // caused macOS CI temp-slug flakes — repo convention is a counter).
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-strflow-{}-{}-{}",
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

#[test]
fn string_variable_concat_prints() {
    let out = run_source("let x = \"GG\";\nx = x + \"CC\";\nconsole.log(x);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "GGCC\n");
}

#[test]
fn string_param_return_roundtrip_prints() {
    let out = run_source("function f(s) { return s + \"!\"; }\nconsole.log(f(\"hi\"));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hi!\n");
}

#[test]
fn string_accumulation_loop_prints() {
    let out = run_source(
        "let a = \"\";\nfor (let i = 0; i < 3; i = i + 1) { a = a + \"y\"; }\nconsole.log(a);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "yyy\n");
}

#[test]
fn string_then_number_binding_is_rejected() {
    // A binding used as both string and number must fail to compile (fail-closed),
    // not silently miscompile.
    let out = run_source("let x = \"a\";\nx = 5;\nconsole.log(x);\n");
    assert!(
        !out.status.success(),
        "mixed string/number binding must be rejected"
    );
}

#[test]
fn consumed_mixed_return_used_as_string_is_rejected() {
    // Finding 1: a mixed return (string branch + plain/int branch) is downgraded
    // to I64, but string-reachability still flows through the call-result into a
    // capturing scalar, which would classify `Repr::String` over the runtime int
    // — codegen would then read the raw int as a string handle (base printed
    // `99?`, the miscompile printed `?`). Consumed mixed returns must FAIL CLOSED.
    let out = run_source(
        "function g(v, k) { if (k > 0) return \"yes\"; return v; }\nlet r = g(99, 0);\nconsole.log(r + \"?\");\n",
    );
    assert!(
        !out.status.success(),
        "consumed mixed return used as a string must be rejected; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// ---- Final-review Finding 1: proven-string values in non-`+` consumers ----

#[test]
fn concat_result_equality_is_rejected() {
    // `b == "xy"` compares a FRESH runtime concat handle against the interned
    // "xy" by handle identity → wrong (base rejected the `+` with E3200; the
    // narrowed head printed 0). Must reject fail-closed.
    let out = run_source("let a = \"x\";\nlet b = a + \"y\";\nconsole.log(b == \"xy\");\n");
    assert!(
        !out.status.success(),
        "equality on a runtime concat result must be rejected; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn concat_result_truthiness_is_rejected() {
    // Empty-string falsiness is lost: a concat handle is always non-zero, so
    // `if (b)` for an empty concat took the wrong branch on head. Reject.
    let out = run_source(
        "let a = \"\";\nlet b = a + \"\";\nif (b) { console.log(1); } else { console.log(0); }\n",
    );
    assert!(
        !out.status.success(),
        "truthiness of a runtime concat result must be rejected; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn concat_result_relational_is_rejected() {
    // Ordering compares raw handles, not string content. Reject.
    let out = run_source("let a = \"x\";\nlet b = a + \"y\";\nconsole.log(b < \"b\");\n");
    assert!(
        !out.status.success(),
        "relational on a runtime concat result must be rejected; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn interned_literal_equality_is_preserved() {
    // A literal-rooted string variable is an INTERNED handle: identity equality
    // == value equality, so `s == "hi"` is correct on the base compiler and
    // must stay correct here (only runtime-concat handles are rejected).
    let out = run_source("let s = \"hi\";\nconsole.log(s == \"hi\");\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

// ---- Final-review Finding 2: array-element read must not leak the string axis ----

#[test]
fn element_read_captor_is_not_string() {
    // `s` captures an array element read; the element was later stored a string,
    // but `s` holds the raw int it read. It must NOT be classified `Repr::String`
    // (head printed garbage `!`); it stays on the int lane, matching the base
    // compiler's `0!`.
    let out = run_source("let a = [1];\nlet s = a[0];\na[0] = \"x\";\nconsole.log(s + \"!\");\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0!\n");
}

// ---- Final-review Finding 3: string `+=` lowers through string concat ----

#[test]
fn string_add_assign_concatenates() {
    // `a += "y"` on a string target must concatenate (base miscompiled it; head
    // vouched a non-handle). Correct output is `xy!`.
    let out = run_source("let a = \"x\";\na += \"y\";\nconsole.log(a + \"!\");\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "xy!\n");
}

#[test]
fn consumed_mixed_return_used_as_number_is_rejected() {
    // Same defect on the numeric-consumer side (base printed `43`, the miscompile
    // printed `1`): the capturing scalar is still poisoned to `Repr::String`, so
    // the program must be rejected rather than silently miscompiled.
    let out = run_source(
        "function g(v, k) { if (k > 0) return \"yes\"; return v; }\nlet r = g(42, 0);\nconsole.log(r + 1);\n",
    );
    assert!(
        !out.status.success(),
        "consumed mixed return used as a number must be rejected; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
