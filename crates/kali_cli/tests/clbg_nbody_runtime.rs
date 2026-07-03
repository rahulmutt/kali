use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

#[test]
fn nbody_runs_and_matches_canonical_output() {
    let source = fixture("nbody-benchmark-v1.ts");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "-0.169075164\n-0.169087605\n"
    );
}

#[test]
fn nbody_metadata_is_consistent() {
    let meta: Value = serde_json::from_str(
        &fs::read_to_string(fixture("nbody-benchmark-v1.json")).expect("read metadata"),
    )
    .expect("parse metadata");
    assert_eq!(meta["benchmark"], "nbody");
    assert_eq!(meta["version"], 1);
    assert_eq!(meta["sourceFile"], "nbody-benchmark-v1.ts");
    assert_eq!(
        meta["buildModes"],
        serde_json::json!(["--fast", "--release", "--release-advanced"])
    );
    let src = fs::read(fixture("nbody-benchmark-v1.ts")).expect("read source");
    let digest = format!("sha256-{:x}", Sha256::digest(&src));
    assert_eq!(
        meta["sourceSha256"], digest,
        "metadata sha256 must match the source file"
    );
}
