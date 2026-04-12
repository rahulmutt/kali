use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use wasmparser::{Parser, Payload};

use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture_path(relative: impl AsRef<Path>) -> PathBuf {
    fixture_root().join(relative)
}

fn write_valid_policy(path: &Path) {
    fs::write(
        path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy");
}

#[test]
fn check_accepts_a_resolved_file() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let value = 1 + 2; value;").expect("write source");

    let output = Command::new(kali_bin())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_discovers_fixture_tree_from_cwd() {
    let output = Command::new(kali_bin())
        .current_dir(fixture_root())
        .arg("check")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 3 file(s)"), "stdout: {stdout}");
}

#[test]
fn check_reports_unresolved_identifiers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "missing;").expect("write source");

    let output = Command::new(kali_bin())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
}

#[test]
fn run_executes_the_hello_fixture() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(fixture_path("run/hello.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_rejects_declaration_only_fixture_entrypoints() {
    let output = Command::new(kali_bin())
        .arg("run")
        .arg(fixture_path("run/decl.d.ts"))
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5007"), "stderr: {stderr}");
}

#[test]
fn test_reports_success_for_explicit_file_sets() {
    let output = Command::new(kali_bin())
        .arg("test")
        .arg(fixture_path("tests/smoke.test.ts"))
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_discovers_fixture_tree_from_cwd() {
    let output = Command::new(kali_bin())
        .current_dir(fixture_root())
        .arg("test")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_filters_selected_files_before_execution() {
    let dir = tempdir().expect("tempdir");
    let keep = dir.path().join("math.test.ts");
    let skip = dir.path().join("strings.test.ts");
    fs::write(&keep, "1 + 2;").expect("write keep source");
    fs::write(&skip, "3 + 4;").expect("write skip source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--filter")
        .arg("math")
        .arg(&keep)
        .arg(&skip)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn init_scaffolds_application_project() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("main.ts").exists());

    let manifest = fs::read_to_string(dir.path().join("kali.json")).expect("manifest");
    assert!(
        manifest.contains("\"schemaVersion\": 1"),
        "manifest: {manifest}"
    );
    let source = fs::read_to_string(dir.path().join("main.ts")).expect("source");
    assert!(source.contains("Hello, world!"), "source: {source}");
}

#[test]
fn init_scaffolds_library_project() {
    let dir = tempdir().expect("tempdir");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("init")
        .arg("--lib")
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("lib.ts").exists());

    let source = fs::read_to_string(dir.path().join("lib.ts")).expect("source");
    assert!(source.contains("export function add"), "source: {source}");
}

#[test]
fn test_rejects_coverage_flag_until_report_contract_exists() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, "1 + 2;").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg("--coverage")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5006"), "stderr: {stderr}");
}

#[test]
fn build_embeds_sandbox_policy_custom_section() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "1 + 2;").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let built = fs::read(dir.path().join("main.wasm")).expect("read wasm artifact");
    let mut seen_policy = false;
    for payload in Parser::new(0).parse_all(&built) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:policy" {
                seen_policy = true;
                break;
            }
        }
    }
    assert!(seen_policy, "custom section 'kali:policy' was not embedded");
}
