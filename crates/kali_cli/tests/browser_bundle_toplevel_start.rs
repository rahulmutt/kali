//! A bare top-level program in a browser bundle runs via the glue's exported
//! `start()` helper, routing its console output through the `console_log`
//! import — and runs at most once no matter how many times `start()` is called.
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn toplevel_program_runs_once_via_glue_start_helper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "console.log(1 + 2);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let harness_path = dir.path().join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.start();
await mod.start();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3\n"), "stdout: {stdout:?}");
    assert_eq!(
        stdout.matches("3\n").count(),
        1,
        "top-level code must run exactly once across repeated start() calls; stdout: {stdout:?}"
    );
}

#[test]
fn start_helper_is_present_in_both_glue_formats() {
    for format in ["esm", "cjs"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("app.ts");
        fs::write(&source_path, "console.log(1 + 2);\n").expect("write source");

        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser");
        if format == "cjs" {
            command.arg("--format").arg("cjs");
        }
        let output = command.arg(&source_path).output().expect("run kali");
        assert!(
            output.status.success(),
            "[{format}] stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let glue_file = if format == "cjs" { "app.cjs" } else { "app.js" };
        let js = fs::read_to_string(dir.path().join("app").join(glue_file)).expect("read glue");
        match format {
            "esm" => assert!(
                js.contains("export async function start()"),
                "[esm] glue: {js}"
            ),
            _ => {
                assert!(js.contains("async function start()"), "[cjs] glue: {js}");
                assert!(
                    js.contains(
                        "const exported = { load, loadWithImports, loadDynamicImport, start };"
                    ),
                    "[cjs] glue: {js}"
                );
            }
        }
        assert!(
            js.contains("instance.exports._start()"),
            "[{format}] glue must invoke the _start export: {js}"
        );
    }
}
