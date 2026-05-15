use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn assert_map_iteration(command: &str, filename: &str, source: &str, expected: &str) {
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

fn map_iteration_run_source() -> &'static str {
    "const values = [[1, 2], [1, 3], [4, 5]]; const mapAlias = Map; const wrappedMapAlias = (mapAlias); for (const entry of new Map(values)) { console.log('map constructor iteration ok'); } for (const entry of new mapAlias(values)) { console.log('map constructor iteration ok'); } for await (const entry of new (wrappedMapAlias)(values)) { console.log('map constructor iteration ok'); } const frozenMapValues = Object.freeze(values); for (const entry of new Map(frozenMapValues)) { console.log('map constructor iteration ok'); }\n"
}

fn map_iteration_test_source() -> &'static str {
    r#"Kali.test('map constructor iteration', () => { const values = [[1, 2], [1, 3], [4, 5]]; const mapAlias = Map; const wrappedMapAlias = (mapAlias); for (const entry of new Map(values)) { console.log('map constructor iteration ok'); } for (const entry of new mapAlias(values)) { console.log('map constructor iteration ok'); } for await (const entry of new (wrappedMapAlias)(values)) { console.log('map constructor iteration ok'); } const frozenMapValues = Object.freeze(values); for (const entry of new Map(frozenMapValues)) { console.log('map constructor iteration ok'); } });
"#
}

#[test]
fn run_supports_map_constructor_iteration_in_js_input() {
    assert_map_iteration(
        "run",
        "main.js",
        map_iteration_run_source(),
        "map constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\n",
    );
}

#[test]
fn run_supports_map_constructor_iteration_in_ts_input() {
    assert_map_iteration(
        "run",
        "main.ts",
        map_iteration_run_source(),
        "map constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\n",
    );
}

#[test]
fn test_supports_map_constructor_iteration_in_js_input() {
    assert_map_iteration("test", "smoke.test.js", map_iteration_test_source(), "ok 1");
}

#[test]
fn test_supports_map_constructor_iteration_in_ts_input() {
    assert_map_iteration("test", "smoke.test.ts", map_iteration_test_source(), "ok 1");
}

#[test]
fn run_supports_map_constructor_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_map_iteration(
            "run",
            filename,
            map_iteration_run_source(),
            "map constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\nmap constructor iteration ok\n",
        );
    }
}

#[test]
fn test_supports_map_constructor_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_map_iteration("test", filename, map_iteration_test_source(), "ok 1");
    }
}
