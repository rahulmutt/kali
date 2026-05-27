use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn array_callback_iteration_sources() -> [&'static str; 10] {
    [
        r#"function main() {
  const values = [1, 2];
  for (const item of values.filter((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.find((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.findIndex((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.findLast((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.findLastIndex((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.flatMap((value) => [value])) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.some((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.every((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.reduce((acc, value) => acc + value, 0)) {
    console.log(item);
  }
}
main();
"#,
        r#"function main() {
  const values = [1, 2];
  for (const item of values.reduceRight((acc, value) => acc + value, 0)) {
    console.log(item);
  }
}
main();
"#,
    ]
}

fn assert_array_callback_iteration_source_rejects(command: &str, source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("for-of array iteration lowering is unavailable"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_array_callback_iteration_lowering_in_js_input() {
    for source in array_callback_iteration_sources() {
        assert_array_callback_iteration_source_rejects("run", source);
    }
}

#[test]
fn test_rejects_array_callback_iteration_lowering_in_js_input() {
    for source in array_callback_iteration_sources() {
        assert_array_callback_iteration_source_rejects("test", source);
    }
}

#[test]
fn check_rejects_array_callback_iteration_lowering_in_js_input() {
    for source in array_callback_iteration_sources() {
        assert_array_callback_iteration_source_rejects("check", source);
    }
}

#[test]
fn build_rejects_array_callback_iteration_lowering_in_js_input() {
    for source in array_callback_iteration_sources() {
        assert_array_callback_iteration_source_rejects("build", source);
    }
}
