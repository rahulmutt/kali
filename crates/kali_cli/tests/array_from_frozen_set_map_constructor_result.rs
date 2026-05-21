use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn array_from_frozen_set_map_constructor_result_source() -> &'static str {
    r##"async function arrayFromFrozenSetMapWrappers() {
  const setValues = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  for (const value of Array.from(Object.freeze(new Set(setValues)))) {
    console.log(value);
  }
  for await (const value of Array.from(Object.freeze((new Set(setValues))))) {
    console.log(value);
  }
  for (const entry of Array.from(Object.freeze(new Map(mapValues)))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(Object.freeze((new Map(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

arrayFromFrozenSetMapWrappers();
"##
}

fn array_from_frozen_set_map_constructor_result_test_source() -> &'static str {
    r##"Kali.test('array.from frozen set/map constructor results', () => {
  async function arrayFromFrozenSetMapWrappers() {
    const setValues = [1, 2, 1];
    const mapValues = [[1, 2], [1, 3], [4, 5]];
    for (const value of Array.from(Object.freeze(new Set(setValues)))) {
      console.log(value);
    }
    for await (const value of Array.from(Object.freeze((new Set(setValues))))) {
      console.log(value);
    }
    for (const entry of Array.from(Object.freeze(new Map(mapValues)))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for await (const entry of Array.from(Object.freeze((new Map(mapValues))))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return arrayFromFrozenSetMapWrappers();
});
"##
}

fn assert_run_supports_array_from_frozen_set_map_constructor_results_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        array_from_frozen_set_map_constructor_result_source(),
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
        stdout, "1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n",
        "stdout: {stdout}"
    );
}

fn assert_test_supports_array_from_frozen_set_map_constructor_results_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(
        &source_path,
        array_from_frozen_set_map_constructor_result_test_source(),
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
        stdout.contains("1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn run_supports_array_from_frozen_set_map_constructor_results_in_js_input() {
    assert_run_supports_array_from_frozen_set_map_constructor_results_in_input("js");
}

#[test]
fn run_supports_array_from_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_run_supports_array_from_frozen_set_map_constructor_results_in_input(extension);
    }
}

#[test]
fn test_supports_array_from_frozen_set_map_constructor_results_in_js_input() {
    assert_test_supports_array_from_frozen_set_map_constructor_results_in_input("js");
}

#[test]
fn test_supports_array_from_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_test_supports_array_from_frozen_set_map_constructor_results_in_input(extension);
    }
}
