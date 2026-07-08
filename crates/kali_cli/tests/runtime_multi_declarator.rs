use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // A per-process AtomicU64 counter makes the slug unique even when two
    // sources share a length (a shared length previously collided the dir and
    // caused macOS CI temp-slug flakes — repo convention is a counter).
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-multidecl-{}-{}-{}",
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
fn two_declarators_both_bind() {
    // RED anchor: pre-fix the parser dropped the second declarator `b`, so `b`
    // was an undefined identifier (E3100). Both must bind and sum to 3.
    let out = run_source("var a = 1, b = 2;\nconsole.log(a + b);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}

#[test]
fn three_declarators_all_bind() {
    let out = run_source("var a = 1, b = 2, c = 3;\nconsole.log(a + b + c);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "6\n");
}

#[test]
fn mixed_init_and_no_init_declarator() {
    // The capstone's `selectRandom` shape: `var r = <init>, c;` — the second
    // declarator has no initializer but must still be a declared binding that
    // can be assigned later.
    let out = run_source("var r = 5, c;\nc = r + 1;\nconsole.log(r);\nconsole.log(c);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n6\n");
}

#[test]
fn let_multi_declarator() {
    let out = run_source("let x = 10, y = 20;\nconsole.log(x + y);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "30\n");
}

#[test]
fn const_multi_declarator() {
    let out = run_source("const p = 4, q = 5;\nconsole.log(p * q);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "20\n");
}

#[test]
fn single_declarator_unchanged() {
    // Regression guard: the single-declarator path must stay identical.
    let out = run_source("var a = 42;\nconsole.log(a);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}
