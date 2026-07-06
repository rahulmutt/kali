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
