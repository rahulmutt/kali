use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn assert_for_of_array_iteration_spread(
    command: &str,
    filename: &str,
    source: &str,
    expected: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

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
    assert!(stdout.contains(expected), "stdout: {stdout}");
}

#[test]
fn run_supports_for_of_array_iteration_spread_in_js_input() {
    assert_for_of_array_iteration_spread(
        "run",
        "main.js",
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }\n",
        "1\n2\n",
    );
}

#[test]
fn run_supports_for_of_array_iteration_spread_in_ts_input() {
    assert_for_of_array_iteration_spread(
        "run",
        "main.ts",
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }\n",
        "1\n2\n",
    );
}

#[test]
fn test_supports_for_of_array_iteration_spread_in_js_input() {
    assert_for_of_array_iteration_spread(
        "test",
        "smoke.test.js",
        "Kali.test('for-of spread', () => { const values = [1, 2]; for (const item of [...values]) { console.log(item); } });\n",
        "ok 1",
    );
}

#[test]
fn test_supports_for_of_array_iteration_spread_in_ts_input() {
    assert_for_of_array_iteration_spread(
        "test",
        "smoke.test.ts",
        "Kali.test('for-of spread', () => { const values = [1, 2]; for (const item of [...values]) { console.log(item); } });\n",
        "ok 1",
    );
}

#[test]
fn run_supports_for_of_array_iteration_spread_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [1, 2]; for (const item of [...values]) { console.log(item); }\n",
            "1\n2\n",
        );
    }
}

#[test]
fn test_supports_for_of_array_iteration_spread_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-of spread', () => { const values = [1, 2]; for (const item of [...values]) { console.log(item); } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_of_break_and_continue_in_js_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [0, 1, 1]; for (const value of values) { if (!value) continue; console.log(value); if (value) break; }\n",
            "1\n",
        );
    }
}

#[test]
fn test_supports_for_of_break_and_continue_in_js_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-of break/continue', () => { const values = [0, 1, 1]; for (const value of values) { if (!value) continue; console.log(value); if (value) break; } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_await_break_and_continue_in_js_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [0, 1, 1]; for await (const value of values) { if (!value) continue; console.log(value); if (value) break; }\n",
            "1\n",
        );
    }
}

#[test]
fn test_supports_for_await_break_and_continue_in_js_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-await break/continue', () => { const values = [0, 1, 1]; for await (const value of values) { if (!value) continue; console.log(value); if (value) break; } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [1, 2]; for (const value of Array.from(values)) { console.log(value); }\n",
            "1\n2\n",
        );
    }
}

#[test]
fn test_supports_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-of Array.from', () => { const values = [1, 2]; for (const value of Array.from(values)) { console.log(value); } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_await_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [1, 2]; for await (const value of Array.from(values)) { console.log(value); }\n",
            "1\n2\n",
        );
    }
}

#[test]
fn test_supports_for_await_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-await Array.from', () => { const values = [1, 2]; for await (const value of Array.from(values)) { console.log(value); } });\n",
            "ok 1",
        );
    }
}
