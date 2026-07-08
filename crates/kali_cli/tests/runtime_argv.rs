use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_node_source_with_args(src: &str, args: &[&str]) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-argv-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let mut cmd = Command::new(kali_bin());
    cmd.arg("run").arg("--api").arg("node").arg(&path).arg("--");
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("run kali")
}

#[test]
fn process_argv_element_read_yields_the_string() {
    // On the node surface argv == ["node", <src>, ...guestArgs], so argv[2] is
    // the first guest arg. Printing it must echo "hello".
    let out = run_node_source_with_args("console.log(process.argv[2]);\n", &["hello"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello\n");
}

#[test]
fn process_argv_element_length_is_the_byte_count() {
    let out = run_node_source_with_args("console.log(process.argv[2].length);\n", &["abcd"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}
