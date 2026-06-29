use super::*;

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_bin_entrypoint_is_rejected_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(package_dir.join("dist/cli.js").to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        !run.status.success(),
        "browser pi-coding-agent package bin entrypoint should be rejected on the browser surface at runtime\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run_stderr.contains("E5506"), "stderr: {run_stderr}");
    assert!(
        run_stderr.contains("Node.js CLI features")
            && run_stderr.contains("unavailable on the 'browser' API surface"),
        "stderr: {run_stderr}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_bin_entrypoint_is_rejected_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured_for_test(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(package_dir.join("dist/cli.js").to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        !test.status.success(),
        "browser pi-coding-agent package bin entrypoint should be rejected on the browser surface at test time\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test.stderr);
    assert!(test_stderr.contains("E5506"), "stderr: {test_stderr}");
    assert!(
        test_stderr.contains("Node.js CLI features")
            && test_stderr.contains("unavailable on the 'browser' API surface"),
        "stderr: {test_stderr}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_bin_entrypoint_is_rejected_on_the_inherited_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_manifest(dir.path(), Some("browser"));
    write_pi_coding_agent_style_package(&package_dir);

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(package_dir.join("dist/cli.js").to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        !run.status.success(),
        "browser pi-coding-agent package bin entrypoint should be rejected on the inherited browser surface at runtime\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run_stderr.contains("E5506"), "stderr: {run_stderr}");
    assert!(
        run_stderr.contains("Node.js CLI features")
            && run_stderr.contains("unavailable on the 'browser' API surface"),
        "stderr: {run_stderr}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_bin_entrypoint_is_rejected_on_the_inherited_browser_surface_in_js_input_when_a_harness_command_is_configured_for_test(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_manifest(dir.path(), Some("browser"));
    write_pi_coding_agent_style_package(&package_dir);

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(package_dir.join("dist/cli.js").to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        !test.status.success(),
        "browser pi-coding-agent package bin entrypoint should be rejected on the inherited browser surface at test time\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test.stderr);
    assert!(test_stderr.contains("E5506"), "stderr: {test_stderr}");
    assert!(
        test_stderr.contains("Node.js CLI features")
            && test_stderr.contains("unavailable on the 'browser' API surface"),
        "stderr: {test_stderr}"
    );
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("1.2.3\n1\n1.2.3"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write browser runtime semver source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser semver package should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn json_browser_runtime_corpus_semver_style_package_remains_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
Kali.test('browser semver package', () => { 1 + 1; });
"#,
    )
    .expect("write browser runtime semver test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser semver package should stay testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("1.2.3\n1\n1.2.3"),
        "json: {json}"
    );
}

#[test]
fn json_browser_runtime_corpus_packages_with_exports_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [("date-fns", "formatISO"), ("@vueuse/core", "helpers")] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser exports-map package {package} should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let json = parse_json_stdout(&run);
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"].as_str().expect("stdout").contains("0\n"),
            "json: {json}"
        );

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser exports-map package', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser exports-map package {package} should stay testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let json = parse_json_stdout(&test);
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["skipped"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"].as_str().expect("stdout").contains("0\n0\n"),
            "json: {json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_module_only_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should be executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_module_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-only corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-only package {package} should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-only package {package} should be testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entry_chains_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:module-chain'; }}\n",
                package = package
            ),
        )
        .expect("write browser internal module");
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-entry-chain corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-chain package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-chain package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entry_chains_remain_executable_and_testable_on_the_browser_surface_in_js_input_with_json_output_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:module-chain'; }}\n",
                package = package
            ),
        )
        .expect("write browser internal module");
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-entry-chain corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-chain package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-chain package {package} should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_module_entry_chains_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_module_only_package(
            dir.path(),
            package,
            "import helper from './internal.mjs';\nexport default function widget() { return helper(); }\n",
        );
        write_types_stub_package(dir.path(), package);
        fs::write(
            dir.path()
                .join("node_modules")
                .join(package)
                .join("internal.mjs"),
            format!(
                "export default function helper() {{ return '{package}:module-chain'; }}\n",
                package = package
            ),
        )
        .expect("write browser internal module");
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} module-entry-chain corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser module-chain package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser module-chain package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_explicit_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_mixed_format_package(
            dir.path(),
            package,
            "module.exports = function root() { return 0; }\n",
            "export default function root() { return 0; }\n",
            subpath,
            "module.exports = function subpath() { return 0; }\n",
            "export default function subpath() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser mixed-format package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let run_json = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run_json.status.success(),
            "browser mixed-format package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run_json.stdout),
            String::from_utf8_lossy(&run_json.stderr)
        );
        assert_browser_runtime_json_output(&run_json, "run", "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser mixed-format package', () => {{\n  if (root() !== 0 || subpath() !== 0) {{\n    throw new Error('browser mixed-format package export mismatch');\n  }}\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser mixed-format package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(
            stdout.contains("0") || stdout.contains("1"),
            "stdout: {stdout}"
        );

        let test_json = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test_json.status.success(),
            "browser mixed-format package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test_json.stdout),
            String::from_utf8_lossy(&test_json.stderr)
        );
        assert_browser_runtime_json_output(&test_json, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_mixed_format_package(
            dir.path(),
            package,
            "module.exports = function root() { return 0; }\n",
            "export default function root() { return 0; }\n",
            subpath,
            "module.exports = function subpath() { return 0; }\n",
            "export default function subpath() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser mixed-format package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let run_json = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run_json.status.success(),
            "browser mixed-format package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run_json.stdout),
            String::from_utf8_lossy(&run_json.stderr)
        );
        assert_browser_runtime_json_output(&run_json, "run", "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser mixed-format package', () => {{\n  if (root() !== 0 || subpath() !== 0) {{\n    throw new Error('browser mixed-format package export mismatch');\n  }}\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser mixed-format package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");

        let test_json = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test_json.status.success(),
            "browser mixed-format package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test_json.stdout),
            String::from_utf8_lossy(&test_json.stderr)
        );
        assert_browser_runtime_json_output(&test_json, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_string_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:exports'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-exports package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-exports package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-exports package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-exports package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_remain_executable_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_remain_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }
",
                "export default function describe() { return 0; }
",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }
",
                "export default function describe() { return 1; }
",
                "const describe = require('./index.js');
module.exports = describe;
",
                "index",
                "export default function describe() { return 0; }
",
                "export default function describe() { return 1; }
",
                "const describe = require('./index.js');
module.exports = describe;
",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';
console.log(describe());
",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in JS input
stdout: {}
stderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "0
"
        );

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';
console.log(describe());
Kali.test('browser runtime package', () => {{ 1 + 1; }});
",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input
stdout: {}
stderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_fixtures_remain_executable_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_fixtures_remain_testable_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.ts");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_fixtures_remain_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_remain_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_internal_browser_rewrites_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "import helper from './internal.js';\nexport default function describe() { return 'node:' + helper(); }\n",
            "import helper from './internal.js';\nexport default function describe() { return 'browser:' + helper(); }\n",
            "internal",
            &format!("export default function helper() {{ return '{package}:node'; }}\n", package = package),
            &format!("export default function helper() {{ return '{package}:browser'; }}\n", package = package),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('{package} corpus', () => {{\n  console.log(describe());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser internal-browser-rewrite package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(
            run_stdout.as_ref(),
            "0
"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser internal-browser-rewrite package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_internal_browser_rewrites_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "import helper from './internal.js';\nexport default function describe() { return 'node:' + helper(); }\n",
            "import helper from './internal.js';\nexport default function describe() { return 'browser:' + helper(); }\n",
            "internal",
            &format!("export default function helper() {{ return '{package}:node'; }}\n", package = package),
            &format!("export default function helper() {{ return '{package}:browser'; }}\n", package = package),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('{package} corpus', () => {{\n  console.log(describe());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser internal-browser-rewrite package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(
            run_stdout.as_ref(),
            "0
"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser internal-browser-rewrite package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_browser_package_fixtures_remain_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured_in_js_run(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_run(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn json_browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn json_browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let json = parse_json_stdout(&run);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_packages_remain_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_test(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured_in_js_test(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_packages_with_exports_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [("date-fns", "formatISO"), ("@vueuse/core", "helpers")] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser exports-map package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('browser exports-map package', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser exports-map package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_pattern_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_pattern_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser pattern-export package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser pattern-export package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_pattern_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_pattern_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser pattern-export package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser pattern-export package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_string_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-entry package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-entry package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-entry package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_browser_string_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_package(
            dir.path(),
            package,
            "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() { return 1; }\n",
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-entry package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-entry package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-entry package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_string_entries_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_package(
            dir.path(),
            package,
            "export default function root() { return 1; }\n",
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-entry package {package} should be executable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-entry package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-entry package {package} should be testable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_string_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_string_exports_package(
            dir.path(),
            package,
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-exports package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-exports package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-exports package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_string_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_string_exports_package(
            dir.path(),
            package,
            "export default function root() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string-exports package {package} should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_browser_runtime_json_output(&run, "run", "0\n");

        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser string-exports package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string-exports package {package} should stay testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_browser_runtime_json_output(&test, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_string_and_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_web_baseline_package(dir.path(), package);
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string/web-baseline package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            stdout.contains("0") || stdout.contains("1"),
            "stdout: {stdout}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string/web-baseline package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(
            stdout.contains("0") || stdout.contains("1"),
            "stdout: {stdout}"
        );
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_browser_string_and_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_web_baseline_package(dir.path(), package);
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string/web-baseline package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        let run_stdout = run_json["stdout"].as_str().expect("json stdout");
        if run_stdout.contains("1") {
            assert_browser_runtime_json_output(&run, "run", "1\n");
        } else {
            assert_browser_runtime_json_output(&run, "run", "0\n");
        }

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string/web-baseline package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        let test_stdout = test_json["stdout"].as_str().expect("json stdout");
        if test_stdout.contains("1") {
            assert_browser_runtime_json_output(&test, "test", "1\n");
        } else {
            assert_browser_runtime_json_output(&test, "test", "0\n");
        }
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_string_and_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_web_baseline_package(dir.path(), package);
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string/web-baseline package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            stdout.contains("0") || stdout.contains("1"),
            "stdout: {stdout}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string/web-baseline package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(
            stdout.contains("0") || stdout.contains("1"),
            "stdout: {stdout}"
        );
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_browser_string_and_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_web_baseline_package(dir.path(), package);
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser string/web-baseline package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        let run_stdout = run_json["stdout"].as_str().expect("json stdout");
        if run_stdout.contains("1") {
            assert_browser_runtime_json_output(&run, "run", "1\n");
        } else {
            assert_browser_runtime_json_output(&run, "run", "0\n");
        }

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser string/web-baseline package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        let test_stdout = test_json["stdout"].as_str().expect("json stdout");
        if test_stdout.contains("1") {
            assert_browser_runtime_json_output(&test, "test", "1\n");
        } else {
            assert_browser_runtime_json_output(&test, "test", "0\n");
        }
    }
}

#[test]
fn browser_runtime_corpus_packages_remain_testable_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    for package in ["browserpkg", "browserexports"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));

        match package {
            "browserpkg" => write_browser_string_package(
                dir.path(),
                package,
                "export default function describe() { return 1; }\n",
                "export default function describe() { return 0; }\n",
            ),
            "browserexports" => write_browser_condition_exports_package(
                dir.path(),
                package,
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
                "index",
                "export default function describe() { return 0; }\n",
                "export default function describe() { return 1; }\n",
                "const describe = require('./index.js');\nmodule.exports = describe;\n",
            ),
            _ => unreachable!("unexpected browser runtime package fixture"),
        }
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.test.ts");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nconsole.log(describe());\nKali.test('browser runtime package', () => {{ 1 + 1; }});\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.ts");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_browser_deno_preference_remain_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return 1; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\nKali.test('browser vs deno package', () => { 1 + 1; });\n",
    )
    .expect("write browser/deno runtime source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package browser-deno should prefer the browser condition over deno on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let json = parse_json_stdout(&test);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        json["stdout"].as_str().expect("stdout").contains("0\n"),
        "json: {json}"
    );
}

#[test]
fn browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn json_browser_runtime_corpus_packages_with_browser_replacement_maps_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "export default function describe() { return 1; }\n",
            "export default function describe() { return 0; }\n",
            "internal",
            "export default function helper() { return 1; }\n",
            "export default function helper() { return 0; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import describe from '{package}';\nimport helper from '{package}/internal';\nKali.test('{package} corpus', () => {{\n  console.log(describe(), helper());\n}});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser replacement-map package {package} should stay executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser replacement-map package {package} should stay testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_packages_with_dual_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_dual_exports_package(
            dir.path(),
            package,
            &format!("export default function root() {{ return '{package}:import'; }}\n", package = package),
            &format!("module.exports = function root() {{ return '{package}:require'; }};\n", package = package),
            subpath,
            &format!("export default function subpath() {{ return '{package}:{subpath}:import'; }}\n", package = package, subpath = subpath),
            &format!("module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n", package = package, subpath = subpath),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser dual package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser dual package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_dual_exports_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_dual_exports_package(
            dir.path(),
            package,
            &format!("export default function root() {{ return '{package}:import'; }}\n", package = package),
            &format!("module.exports = function root() {{ return '{package}:require'; }};\n", package = package),
            subpath,
            &format!("export default function subpath() {{ return '{package}:{subpath}:import'; }}\n", package = package, subpath = subpath),
            &format!("module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n", package = package, subpath = subpath),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser dual package {package} should be executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");
        assert_eq!(run_stdout.lines().count(), 1, "stdout: {run_stdout}");

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser dual package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(stdout.contains("0"), "stdout: {stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_typed_export_branches_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("solid-js", "web"),
        ("@apollo/client", "cache"),
        ("@emotion/react", "jsx-runtime"),
        ("@emotion/styled", "styled"),
        ("@floating-ui/react", "dom"),
        ("@reduxjs/toolkit", "query"),
        ("@tanstack/react-query", "query-core"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_typed_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser package {package} with typed export branches should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser package {package} with typed export branches should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_typed_export_branches_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("solid-js", "web"),
        ("@apollo/client", "cache"),
        ("@emotion/react", "jsx-runtime"),
        ("@emotion/styled", "styled"),
        ("@floating-ui/react", "dom"),
        ("@reduxjs/toolkit", "query"),
        ("@tanstack/react-query", "query-core"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_typed_export_map_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser test source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser package {package} with typed export branches should be executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser package {package} with typed export branches should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

#[test]
fn browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn json_browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0")
                || run_json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .lines()
                    .all(|line| line == "1"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0")
                || test_json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .lines()
                    .all(|line| line == "1"),
            "json: {test_json}"
        );
    }
}

#[test]
fn browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert!(
            stdout.contains("0") || stdout.contains("1"),
            "stdout: {stdout}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        assert!(
            stdout.contains("0") || stdout.contains("1"),
            "stdout: {stdout}"
        );
    }
}

#[test]
fn json_browser_runtime_corpus_web_baseline_packages_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("main.test.js");
        write_web_baseline_test_source(&test_path, package);

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser web-baseline package {package} should be executable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_json = parse_json_stdout(&run);
        assert_eq!(run_json["command"], "run");
        assert_eq!(run_json["success"], true);
        assert_eq!(run_json["exitCode"], 0);
        assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            run_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0")
                || run_json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .lines()
                    .all(|line| line == "1"),
            "json: {run_json}"
        );

        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser web-baseline package {package} should be testable on the browser surface in JS input with json output when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_json = parse_json_stdout(&test);
        assert_eq!(test_json["command"], "test");
        assert_eq!(test_json["success"], true);
        assert_eq!(test_json["exitCode"], 0);
        assert_eq!(test_json["payload"]["passed"], 1);
        assert_eq!(test_json["payload"]["total"], 1);
        assert_eq!(test_json["payload"]["failed"], 0);
        assert_eq!(test_json["payload"]["skipped"], 0);
        assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
        assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            test_json["stdout"]
                .as_str()
                .expect("stdout")
                .lines()
                .all(|line| line == "0")
                || test_json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .lines()
                    .all(|line| line == "1"),
            "json: {test_json}"
        );
    }
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_jsx_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in JSX input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_tsx_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in TSX input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.test.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
Kali.test('pi-coding-agent browser runtime package', () => { 1 + 1; });
"#,
    )
    .expect("write pi-coding-agent browser runtime test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_json = parse_json_stdout(&test);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {test_json}"
    );
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_testable_on_the_browser_surface_in_jsx_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.test.jsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
Kali.test('pi-coding-agent browser runtime package', () => { 1 + 1; });
"#,
    )
    .expect("write pi-coding-agent browser runtime test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the browser surface in JSX input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_json = parse_json_stdout(&test);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {test_json}"
    );
}

#[test]
fn json_browser_runtime_corpus_pi_coding_agent_style_package_remains_testable_on_the_browser_surface_in_tsx_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.test.tsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
Kali.test('pi-coding-agent browser runtime package', () => { 1 + 1; });
"#,
    )
    .expect("write pi-coding-agent browser runtime test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the browser surface in TSX input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_json = parse_json_stdout(&test);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {test_json}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_js_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_jsx_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in JSX input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_remains_executable_on_the_browser_surface_in_tsx_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent browser runtime source");

    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the browser surface in TSX input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = parse_json_stdout(&run);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], true);
    assert_eq!(run_json["exitCode"], 0);
    assert_eq!(run_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(run_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        run_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {run_json}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_remains_testable_on_the_browser_surface_in_jsx_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.test.jsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
Kali.test('pi-coding-agent browser runtime package', () => { 1 + 1; });
"#,
    )
    .expect("write pi-coding-agent browser runtime test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the browser surface in JSX input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_json = parse_json_stdout(&test);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {test_json}"
    );
}

#[test]
fn browser_runtime_corpus_pi_coding_agent_style_package_remains_testable_on_the_browser_surface_in_tsx_input_when_the_browser_api_surface_is_inherited_and_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.test.tsx");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
Kali.test('pi-coding-agent browser runtime package', () => { 1 + 1; });
"#,
    )
    .expect("write pi-coding-agent browser runtime test source");

    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the browser surface in TSX input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_json = parse_json_stdout(&test);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], true);
    assert_eq!(test_json["exitCode"], 0);
    assert_eq!(test_json["payload"]["passed"], 1);
    assert_eq!(test_json["payload"]["total"], 1);
    assert_eq!(test_json["payload"]["failed"], 0);
    assert_eq!(test_json["payload"]["skipped"], 0);
    assert_eq!(test_json["payload"]["hostContract"], "browser-requested");
    assert_eq!(test_json["payload"]["runtimeBackend"], "browser-harness");
    assert!(
        test_json["stdout"]
            .as_str()
            .expect("stdout")
            .lines()
            .all(|line| line == "0"),
        "json: {test_json}"
    );
}

#[test]
fn browser_runtime_corpus_packages_that_block_the_selected_path_are_rejected_in_browser_context_on_js_input_when_a_harness_command_is_configured(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_blocked_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
        );
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser JS source");
        let source = source_path.to_str().unwrap();

        for command in ["run", "test"] {
            for explicit_browser_surface in [true, false] {
                let args = if explicit_browser_surface {
                    vec![command, "--api", "browser", source]
                } else {
                    vec![command, source]
                };
                let output = Command::new(kali_bin())
                    .current_dir(dir.path())
                    .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
                    .args(args)
                    .output()
                    .expect("run kali");
                assert!(
                    !output.status.success(),
                    "browser-blocked package {package} should be rejected during {command} on JS input with {} browser apiSurface\nstdout: {}\nstderr: {}",
                    if explicit_browser_surface { "explicit" } else { "inherited" },
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );

                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    stderr.contains("error[E3000]"),
                    "browser-blocked package {package} should surface the import-resolution failure during {command} on JS input with {} browser apiSurface\nstderr: {}",
                    if explicit_browser_surface { "explicit" } else { "inherited" },
                    stderr
                );
                assert!(
                    stderr.contains("could not be resolved"),
                    "browser-blocked package {package} should not fall back to the non-browser entry during {command} on JS input with {} browser apiSurface\nstderr: {}",
                    if explicit_browser_surface { "explicit" } else { "inherited" },
                    stderr
                );

                let json_args = if explicit_browser_surface {
                    vec!["--output", "json", command, "--api", "browser", source]
                } else {
                    vec!["--output", "json", command, source]
                };
                let json_output = Command::new(kali_bin())
                    .current_dir(dir.path())
                    .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
                    .args(json_args)
                    .output()
                    .expect("run kali");
                assert_browser_blocked_package_json_rejection(&json_output, command);
            }
        }
    }
}

#[test]
fn browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    write_types_stub_package(dir.path(), package);

    let run_source_path = dir.path().join("main.js");
    fs::write(
        &run_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write browser runtime run source");
    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(run_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package {package} should stay executable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let test_source_path = dir.path().join("main.test.js");
    fs::write(
        &test_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
            package = package
        ),
    )
    .expect("write browser runtime test source");
    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(test_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package {package} should stay testable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_browser_surface_in_ts_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    write_types_stub_package(dir.path(), package);

    let run_source_path = dir.path().join("main.ts");
    fs::write(
        &run_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write browser runtime TS run source");
    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(run_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package {package} should stay executable on the browser surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let test_source_path = dir.path().join("main.test.ts");
    fs::write(
        &test_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
            package = package
        ),
    )
    .expect("write browser runtime TS test source");
    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(test_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package {package} should stay testable on the browser surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn json_browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    write_types_stub_package(dir.path(), package);

    let run_source_path = dir.path().join("main.js");
    fs::write(
        &run_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write browser runtime JS run source");
    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(run_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package {package} should stay executable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_browser_runtime_json_output(&run, "run", "0\n");

    let test_source_path = dir.path().join("main.test.js");
    fs::write(
        &test_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
            package = package
        ),
    )
    .expect("write browser runtime JS test source");
    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(test_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package {package} should stay testable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    assert_browser_runtime_json_output(&test, "test", "0\n");
}

#[test]
fn json_browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_inherited_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    write_types_stub_package(dir.path(), package);

    let run_source_path = dir.path().join("main.js");
    fs::write(
        &run_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write inherited browser runtime run source");
    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--output")
        .arg("json")
        .arg(run_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package {package} should stay executable on the inherited browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_browser_runtime_json_output(&run, "run", "0\n");

    let test_source_path = dir.path().join("main.test.js");
    fs::write(
        &test_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
            package = package
        ),
    )
    .expect("write inherited browser runtime test source");
    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(test_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package {package} should stay testable on the inherited browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    assert_browser_runtime_json_output(&test, "test", "0\n");
}

#[test]
fn browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_inherited_browser_surface_in_js_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    write_types_stub_package(dir.path(), package);

    let run_source_path = dir.path().join("main.js");
    fs::write(
        &run_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write inherited browser runtime run source");
    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(run_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package {package} should stay executable on the inherited browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(stdout.trim(), "0", "stdout: {stdout}");

    let test_source_path = dir.path().join("main.test.js");
    fs::write(
        &test_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
            package = package
        ),
    )
    .expect("write inherited browser runtime test source");
    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(test_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package {package} should stay testable on the inherited browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_inherited_browser_surface_in_ts_input_when_a_harness_command_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    write_types_stub_package(dir.path(), package);

    let run_source_path = dir.path().join("main.ts");
    fs::write(
        &run_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write inherited browser runtime TS run source");
    let run = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(run_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        run.status.success(),
        "browser runtime package {package} should stay executable on the inherited browser surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(stdout.trim(), "0", "stdout: {stdout}");

    let test_source_path = dir.path().join("main.test.ts");
    fs::write(
        &test_source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
            package = package
        ),
    )
    .expect("write inherited browser runtime TS test source");
    let test = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(test_source_path.to_str().unwrap())
        .output()
        .expect("run kali");
    assert!(
        test.status.success(),
        "browser runtime package {package} should stay testable on the inherited browser surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("0"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
}

#[test]
fn browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_browser_surface_in_jsx_and_tsx_input_when_a_harness_command_is_configured(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let package = "spawn-tools";
        write_manifest(dir.path(), Some("browser"));
        write_deno_host_package(
            dir.path(),
            package,
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
        );
        write_types_stub_package(dir.path(), package);

        let run_source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &run_source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser runtime source");

        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(run_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in {extension} input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(stdout.trim(), "0", "stdout: {stdout}");

        let test_source_path = dir.path().join(format!("main.test.{extension}"));
        fs::write(
            &test_source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
                package = package
            ),
        )
        .expect("write browser runtime test source");
        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in {extension} input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("0"), "stdout: {stdout}");
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");

        let json_run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg("--api")
            .arg("browser")
            .arg(run_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            json_run.status.success(),
            "browser runtime package {package} should stay executable on the browser surface in {extension} input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_run.stdout),
            String::from_utf8_lossy(&json_run.stderr)
        );
        assert_browser_runtime_json_output(&json_run, "run", "0\n");

        let json_test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg("--api")
            .arg("browser")
            .arg(test_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            json_test.status.success(),
            "browser runtime package {package} should stay testable on the browser surface in {extension} input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_test.stdout),
            String::from_utf8_lossy(&json_test.stderr)
        );
        assert_browser_runtime_json_output(&json_test, "test", "0\n");
    }
}

#[test]
fn browser_runtime_corpus_packages_with_spawn_tools_remain_executable_and_testable_on_the_inherited_browser_surface_in_jsx_and_tsx_input_when_a_harness_command_is_configured(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let package = "spawn-tools";
        write_manifest(dir.path(), Some("browser"));
        write_deno_host_package(
            dir.path(),
            package,
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
        );
        write_types_stub_package(dir.path(), package);

        let run_source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &run_source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write inherited browser runtime source");
        let run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("run")
            .arg(run_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            run.status.success(),
            "browser runtime package {package} should stay executable on the inherited browser surface in {extension} input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(stdout.trim(), "0", "stdout: {stdout}");

        let test_source_path = dir.path().join(format!("main.test.{extension}"));
        fs::write(
            &test_source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\nKali.test('browser runtime package', () => {{ console.log(root()); }});\n",
                package = package
            ),
        )
        .expect("write inherited browser runtime test source");
        let test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("test")
            .arg(test_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            test.status.success(),
            "browser runtime package {package} should stay testable on the inherited browser surface in {extension} input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(stdout.contains("0"), "stdout: {stdout}");
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");

        let json_run = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("run")
            .arg(run_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            json_run.status.success(),
            "browser runtime package {package} should stay executable on the inherited browser surface in {extension} input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_run.stdout),
            String::from_utf8_lossy(&json_run.stderr)
        );
        assert_browser_runtime_json_output(&json_run, "run", "0\n");

        let json_test = Command::new(kali_bin())
            .current_dir(dir.path())
            .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
            .arg("--output")
            .arg("json")
            .arg("test")
            .arg(test_source_path.to_str().unwrap())
            .output()
            .expect("run kali");
        assert!(
            json_test.status.success(),
            "browser runtime package {package} should stay testable on the inherited browser surface in {extension} input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_test.stdout),
            String::from_utf8_lossy(&json_test.stderr)
        );
        assert_browser_runtime_json_output(&json_test, "test", "0\n");
    }
}
