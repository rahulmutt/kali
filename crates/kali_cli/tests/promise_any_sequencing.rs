use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn promise_any_sequencing_source(command: &str) -> String {
    if command == "test" {
        return r#"async function promiseAnySmoke() {
  const winner = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);
  if (winner !== 1) {
    throw new Error('unexpected Promise.any sequencing');
  }
}

Kali.test('promise any', () => promiseAnySmoke());
"#
        .to_string();
    }

    r#"async function promiseAnySmoke() {
  const winner = await Promise.any([Promise.reject('boom'), Promise.resolve(1)]);
  if (winner !== 1) {
    throw new Error('unexpected Promise.any sequencing');
  }
}

async function main() {
  await promiseAnySmoke();
  console.log('promise any ok');
}

main();
"#
    .to_string()
}

fn assert_promise_any(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, promise_any_sequencing_source(command)).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
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
    match command {
        "run" => assert!(stdout.contains("promise any ok"), "stdout: {stdout}"),
        "test" => assert!(stdout.contains("ok 1"), "stdout: {stdout}"),
        _ => unreachable!("unsupported command: {command}"),
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_supports_promise_any_in_js_input() {
    assert_promise_any("run", "main.js");
}

#[test]
fn run_supports_promise_any_in_ts_input() {
    assert_promise_any("run", "main.ts");
}

#[test]
fn test_supports_promise_any_in_js_input() {
    assert_promise_any("test", "smoke.test.js");
}

#[test]
fn test_supports_promise_any_in_ts_input() {
    assert_promise_any("test", "smoke.test.ts");
}
