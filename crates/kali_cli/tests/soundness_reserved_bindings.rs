//! Soundness pin: reserved words must reject as binding names.
//!
//! `parse_variable_declarator` used to take ANY next token's value as the
//! binding name, so `const if = 1` bound a variable literally named "if".
//! Contextual keywords that are legal JS binding identifiers (e.g. `type`,
//! `of`, `from`) must keep working.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-reserved-{}-{}-{}",
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

// `parse_variable_declarator` took ANY next token's value as the binding
// name, so `const if = 1` bound a variable literally named "if".
#[test]
fn reserved_word_binding_rejects() {
    for src in ["const if = 1;\n", "let for = 2;\n", "var function = 3;\n"] {
        let out = run_source(src);
        assert!(!out.status.success(), "{src:?} must reject, got: {out:?}");
    }
}

// Contextual keywords that are legal JS binding names must keep working —
// the lexer keywordizes them, but `const type = 1` is valid JS.
#[test]
fn contextual_keyword_bindings_still_work() {
    let out = run_source(
        "const type = 1;\nconst of = 2;\nconst from = 3;\nconsole.log(type + of + from);\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "6");
}

// Bind+read symmetry pin for the ENTIRE `is_binding_name_token` allowlist:
// every token legal in binding position must also read back as a plain
// identifier expression. The bind gate (statement.rs via call.rs allowlist)
// and the readback arms (primary.rs) are hand-mirrored — a token added to
// one side without the other silently miscompiles (readback falls into the
// primary catch-all and yields Identifier("unknown"), i.e. 0), so this pins
// all 8 tokens end-to-end with an exact sum only correct reads can produce.
#[test]
fn full_binding_allowlist_binds_and_reads_back() {
    let out = run_source(
        "const plain = 1;\nconst type = 2;\nconst interface = 3;\nconst enum = 4;\nconst from = 5;\nconst as = 6;\nconst of = 7;\nconst async = 8;\nconsole.log(plain + type + interface + enum + from + as + of + async);\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "36", "{out:?}");
}
