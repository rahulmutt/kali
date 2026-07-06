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
        "kali-strsub-{}-{}-{}",
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
fn substring_two_arg_runtime_bounds_prints() {
    let out =
        run_source("let a = \"GGCCAATT\";\nlet i = 2;\nconsole.log(a.substring(i, i + 4));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CCAA\n");
}

#[test]
fn substring_one_arg_and_concat_roundtrip() {
    // The fastaRepeat wrap shape: substring-to-end + `+` + substring prefix.
    let out = run_source(
        "function wrap(seq, i) { return seq.substring(i) + seq.substring(0, i); }\nconsole.log(wrap(\"GGCCAATT\", 6));\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "TTGGCCAA\n");
}

#[test]
fn substring_swaps_and_clamps_bounds_like_js() {
    // JS substring: start > end swaps; negative -> 0; > len -> len.
    let out = run_source(
        "let a = \"GGCCAATT\";\nlet hi = 99;\nlet lo = 0 - 5;\nconsole.log(a.substring(6, 2));\nconsole.log(a.substring(lo, 3));\nconsole.log(a.substring(4, hi));\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CCAA\nGGC\nAATT\n");
}

#[test]
fn chained_substring_prints() {
    let out = run_source(
        "let a = \"GGCCAATT\";\nlet i = 1;\nconsole.log(a.substring(i).substring(i));\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CCAATT\n");
}

#[test]
fn substring_on_non_ascii_receiver_is_rejected() {
    // Byte-offset slicing of non-ASCII text diverges from JS code-unit
    // semantics: must reject, never miscompile.
    let out = run_source("let a = \"héllo\";\nlet i = 1;\nconsole.log(a.substring(i, 3));\n");
    assert!(!out.status.success(), "non-ASCII receiver must be rejected");
}

#[test]
fn substring_with_float_bound_is_rejected() {
    // JS ToInteger on fractional bounds is deliberately unimplemented.
    let out = run_source("let a = \"GGCC\";\nlet f = 1 / 2;\nconsole.log(a.substring(f, 3));\n");
    assert!(!out.status.success(), "float-repr bound must be rejected");
}

#[test]
fn substring_result_equality_is_rejected() {
    // A slice is a non-interned runtime string: handle-identity == would be
    // wrong. Pin as a rejection.
    let out = run_source(
        "let a = \"GGCC\";\nlet i = 1;\nlet s = a.substring(0, i);\nif (s == \"G\") { console.log(1); }\n",
    );
    assert!(
        !out.status.success(),
        "substring == must be rejected, not compared by handle"
    );
}

#[test]
fn static_substring_fold_still_prints() {
    // Base fold lane byte-identical.
    let out = run_source("console.log(\"GGCCAATT\".substring(2, 4));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "CC\n");
}

#[test]
fn string_param_length_prints() {
    let out = run_source("function f(seq) { return seq.length; }\nconsole.log(f(\"GGCCAATT\"));\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "8\n");
}

#[test]
fn substring_result_length_prints() {
    // The fastaRepeat shape: `seqi = lenOut - s.length` on a slice.
    let out = run_source(
        "let a = \"GGCCAATT\";\nlet i = 6;\nlet s = a.substring(i);\nconsole.log(10 - s.length);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "8\n");
}

#[test]
fn let_string_length_prints_directly() {
    // Direct `console.log(a.length)` takes the console static-render lane
    // first (`render_length`); a runtime string receiver must defer to the
    // dynamic string-length arm, not bake in a static 0.
    let out = run_source("let a = \"GGCC\";\nconsole.log(a.length);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}

#[test]
fn non_ascii_string_length_is_rejected() {
    // handle len is a byte count; "héllo".length must be 5, the handle says 6.
    let out = run_source("let a = \"héllo\";\nlet b = a + \"\";\nconsole.log(b.length);\n");
    assert!(
        !out.status.success(),
        "non-ASCII runtime .length must be rejected"
    );
}

#[test]
fn static_non_ascii_literal_length_still_prints_utf16_count() {
    // Base fold lane: emit_unary counts UTF-16 units — correct for literals.
    let out = run_source("console.log(\"héllo\".length);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}

#[test]
fn array_length_still_prints() {
    // NOTE: adapted from the brief's `let a = [1, 2, 3]` to `const` — a `let`
    // numeric-array literal is not registered as an array binding today (a
    // pre-existing gap unrelated to this task; only the `const` alias/fold
    // path and `new Array`/`.fill` declarators register `array_bindings`).
    // `const` exercises the actual working array-`.length` lane this task
    // must leave untouched.
    let out = run_source("const a = [1, 2, 3];\nconsole.log(a.length);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}
