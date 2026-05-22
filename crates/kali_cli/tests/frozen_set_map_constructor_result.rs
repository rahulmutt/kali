use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn frozen_set_map_constructor_result_source() -> &'static str {
    "const values = [1, 2, 1]; for (const value of Object.freeze(new Set(values))) { console.log(value); } for (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0]); console.log(entry[1]); } for (const value of Object.freeze(new globalThis[\"Set\"](values))) { console.log(value); } for (const entry of Object.freeze(new globalThis['Map']([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0]); console.log(entry[1]); }\n"
}

fn parenthesized_frozen_set_map_constructor_result_source() -> &'static str {
    "const values = [1, 2, 1]; for (const value of Object.freeze((new Set(values)))) { console.log(value); } for (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); } for (const value of Object.freeze((new globalThis[\"Set\"](values)))) { console.log(value); } for (const entry of Object.freeze((new globalThis['Map']([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); } for (const value of Object.freeze((new (null ?? Set)(values)))) { console.log(value); } for (const value of Object.freeze((new (false || Set)(values)))) { console.log(value); } for (const entry of Object.freeze((new (null ?? Map)([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of Object.freeze((new (false || Map)([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); }\n"
}

fn parenthesized_frozen_set_map_constructor_result_test_source() -> &'static str {
    "Kali.test('parenthesized frozen set and map constructor results', () => { const values = [1, 2, 1]; for (const value of Object.freeze((new Set(values)))) { console.log(value); } for (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); } for (const value of Object.freeze((new globalThis[\"Set\"](values)))) { console.log(value); } for (const entry of Object.freeze((new globalThis['Map']([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); } for (const value of Object.freeze((new (null ?? Set)(values)))) { console.log(value); } for (const value of Object.freeze((new (false || Set)(values)))) { console.log(value); } for (const entry of Object.freeze((new (null ?? Map)([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of Object.freeze((new (false || Map)([[1, 2], [1, 3], [4, 5]])))) { console.log(entry[0]); console.log(entry[1]); } });\n"
}

fn frozen_object_helper_iteration_source() -> &'static str {
    "const object = Object.fromEntries([[\"b\", 1], [\"a\", 2]]); for (const key of Object.freeze(Object.keys(object))) { console.log(key); } for (const entry of Object.freeze(Object.entries(object))) { console.log(entry[0]); console.log(entry[1]); }\n"
}

fn frozen_set_map_constructor_result_test_source() -> &'static str {
    "Kali.test('frozen set and map constructor results', () => { const values = [1, 2, 1]; for (const value of Object.freeze(new Set(values))) { console.log(value); } for (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0]); console.log(entry[1]); } for (const value of Object.freeze(new globalThis[\"Set\"](values))) { console.log(value); } for (const entry of Object.freeze(new globalThis['Map']([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0]); console.log(entry[1]); } });\n"
}

fn frozen_object_helper_iteration_test_source() -> &'static str {
    "Kali.test('frozen object helper iteration targets', () => { const object = Object.fromEntries([[\"b\", 1], [\"a\", 2]]); for (const key of Object.freeze(Object.keys(object))) { console.log(key); } for (const entry of Object.freeze(Object.entries(object))) { console.log(entry[0]); console.log(entry[1]); } });\n"
}

fn assert_run_supports_frozen_set_map_constructor_results_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, frozen_set_map_constructor_result_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
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
    assert_eq!(
        stdout, "1\n2\n1\n3\n4\n5\n1\n2\n1\n3\n4\n5\n",
        "stdout: {stdout}"
    );
}

fn assert_test_supports_frozen_set_map_constructor_results_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(
        &source_path,
        frozen_set_map_constructor_result_test_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(
        stdout.contains("1\n2\n1\n3\n4\n5\n1\n2\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

fn assert_run_supports_frozen_object_helper_iteration_targets_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, frozen_object_helper_iteration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
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
    assert_eq!(stdout, "b\na\nb\n1\na\n2\n", "stdout: {stdout}");
}

fn assert_test_supports_frozen_object_helper_iteration_targets_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, frozen_object_helper_iteration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(stdout.contains("b\na\nb\n1\na\n2\n"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn run_supports_frozen_set_map_constructor_results_in_js_input() {
    assert_run_supports_frozen_set_map_constructor_results_in_input("js");
}

#[test]
fn run_supports_parenthesized_frozen_set_map_constructor_results_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        parenthesized_frozen_set_map_constructor_result_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
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
    assert_eq!(
        stdout, "1\n2\n1\n3\n4\n5\n1\n2\n1\n3\n4\n5\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n",
        "stdout: {stdout}"
    );
}

#[test]
fn run_supports_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_run_supports_frozen_set_map_constructor_results_in_input(extension);
    }
}

#[test]
fn run_supports_parenthesized_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            parenthesized_frozen_set_map_constructor_result_source(),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("run")
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
        assert_eq!(
            stdout, "1\n2\n1\n3\n4\n5\n1\n2\n1\n3\n4\n5\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n",
            "stdout: {stdout}"
        );
    }
}

#[test]
fn test_supports_frozen_set_map_constructor_results_in_js_input() {
    assert_test_supports_frozen_set_map_constructor_results_in_input("js");
}

#[test]
fn test_supports_parenthesized_frozen_set_map_constructor_results_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(
        &source_path,
        parenthesized_frozen_set_map_constructor_result_test_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
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
    assert!(
        stdout.contains("1\n2\n1\n3\n4\n5\n1\n2\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn test_supports_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_test_supports_frozen_set_map_constructor_results_in_input(extension);
    }
}

#[test]
fn test_supports_parenthesized_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &source_path,
            parenthesized_frozen_set_map_constructor_result_test_source(),
        )
        .expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("test")
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
        assert!(
            stdout.contains("1\n2\n1\n3\n4\n5\n1\n2\n1\n3\n4\n5\n"),
            "stdout: {stdout}"
        );
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_frozen_object_helper_iteration_targets_in_js_input() {
    assert_run_supports_frozen_object_helper_iteration_targets_in_input("js");
}

#[test]
fn run_supports_frozen_object_helper_iteration_targets_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_run_supports_frozen_object_helper_iteration_targets_in_input(extension);
    }
}

#[test]
fn test_supports_frozen_object_helper_iteration_targets_in_js_input() {
    assert_test_supports_frozen_object_helper_iteration_targets_in_input("js");
}

#[test]
fn test_supports_frozen_object_helper_iteration_targets_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_test_supports_frozen_object_helper_iteration_targets_in_input(extension);
    }
}
