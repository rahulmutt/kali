use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn write_stdout_bytes_emits_raw_bytes() {
    // Emits the 5 ASCII bytes for "P4\n4 " — includes 0x0A and 0x20 to prove
    // arbitrary (non-alphanumeric) bytes survive the sink verbatim.
    let src = "const out = new Array(5);\n\
        out[0] = 80; out[1] = 52; out[2] = 10; out[3] = 52; out[4] = 32;\n\
        Kali.writeStdoutBytes(out);";
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.ts");
    fs::write(&path, src).expect("write");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, vec![80u8, 52, 10, 52, 32]);
}
