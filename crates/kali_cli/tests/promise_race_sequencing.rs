use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn promise_race_sequencing_source(command: &str) -> String {
    if command == "test" {
        return r#"async function promiseRaceSmoke() {
  const winner = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);
  if (winner !== 1) {
    throw new Error('unexpected Promise.race sequencing');
  }
}

Kali.test('promise race', () => promiseRaceSmoke());
"#
        .to_string();
    }

    r#"async function promiseRaceSmoke() {
  const winner = await Promise.race([Promise.resolve(1), Promise.resolve(2)]);
  if (winner !== 1) {
    throw new Error('unexpected Promise.race sequencing');
  }
}

async function main() {
  await promiseRaceSmoke();
  console.log('promise race ok');
}

main();
"#
    .to_string()
}

fn assert_promise_race(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, promise_race_sequencing_source(command)).expect("write source");

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
        "run" => assert!(stdout.contains("promise race ok"), "stdout: {stdout}"),
        "test" => assert!(stdout.contains("ok 1"), "stdout: {stdout}"),
        _ => unreachable!("unsupported command: {command}"),
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_supports_promise_race_in_js_input() {
    assert_promise_race("run", "main.js");
}

#[test]
fn run_supports_promise_race_in_ts_input() {
    assert_promise_race("run", "main.ts");
}

#[test]
fn test_supports_promise_race_in_js_input() {
    assert_promise_race("test", "smoke.test.js");
}

#[test]
fn test_supports_promise_race_in_ts_input() {
    assert_promise_race("test", "smoke.test.ts");
}
