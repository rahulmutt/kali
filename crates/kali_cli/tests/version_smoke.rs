use std::{path::PathBuf, process::Command};

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

#[test]
fn kali_version_reports_package_version_and_exits_successfully() {
    let output = Command::new(kali_bin())
        .arg("--version")
        .output()
        .expect("run kali --version");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("kali {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
}
