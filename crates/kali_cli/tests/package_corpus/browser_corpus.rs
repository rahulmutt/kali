use super::*;

#[test]
fn browser_corpus_packages_remain_checkable_and_deployable_through_host() {
    for package in [
        "react",
        "preact",
        "vue",
        "react-dom",
        "chart.js",
        "framer-motion",
        "clsx",
        "classnames",
        "reselect",
        "rxjs",
        "vue-router",
        "@testing-library/react",
        "@testing-library/dom",
        "nanostores",
        "@storybook/react",
        "@remix-run/react",
        "path-to-regexp",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function widget() { return 'ok'; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import '{}';\nconsole.log('browser corpus: {}');\n",
                package, package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_semver_style_package_remains_checkable_and_deployable_through_host_on_js_input() {
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
    .expect("write browser source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser semver package should be checkable on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let check_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "check",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        check_json.status.success(),
        "browser semver package should be checkable on the browser surface with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], true);
    assert_eq!(check_envelope["exitCode"], 0);
    assert_eq!(check_envelope["payload"]["filesChecked"], 1);
    assert_eq!(check_envelope["payload"]["errorCount"], 0);
    assert_eq!(check_envelope["payload"]["warningCount"], 0);

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build_json.status.success(),
        "browser semver package should be deployable-through-host via bundle with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], true);
    assert_eq!(build_envelope["exitCode"], 0);
    let payload = build_envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
}

#[test]
fn browser_corpus_semver_style_package_bin_entrypoint_is_rejected_on_the_browser_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");

    let bin_path = package_dir.join("bin/semver.js");
    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", bin_path.to_str().unwrap()],
    );
    assert!(
        !check.status.success(),
        "browser semver package bin entrypoint should be rejected on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check_stderr.contains("E3100"), "stderr: {check_stderr}");
    assert!(check_stderr.contains("require"), "stderr: {check_stderr}");

    let check_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "check",
            "--api",
            "browser",
            bin_path.to_str().unwrap(),
        ],
    );
    assert!(
        !check_json.status.success(),
        "json browser check should surface the bin entrypoint rejection\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], false);
    assert_eq!(check_envelope["exitCode"], 1);
    assert_eq!(
        check_envelope["payload"],
        serde_json::json!({"errorCount": 1, "filesChecked": 1, "warningCount": 0})
    );
    assert!(
        check_envelope["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["code"] == "E3100"),
        "check json: {check_envelope}"
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            bin_path.to_str().unwrap(),
        ],
    );
    assert!(
        !build.status.success(),
        "browser semver package bin entrypoint should be rejected during bundle emission\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stderr = String::from_utf8_lossy(&build.stderr);
    assert!(build_stderr.contains("E3100"), "stderr: {build_stderr}");
    assert!(build_stderr.contains("require"), "stderr: {build_stderr}");

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            "--api",
            "browser",
            bin_path.to_str().unwrap(),
        ],
    );
    assert!(
        !build_json.status.success(),
        "json browser build should surface the bin entrypoint rejection\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], false);
    assert_eq!(build_envelope["exitCode"], 1);
    assert_eq!(build_envelope["payload"], serde_json::Value::Null);
    assert!(
        build_envelope["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["code"] == "E3100"),
        "build json: {build_envelope}"
    );
}

#[test]
fn browser_corpus_semver_style_package_bin_entrypoint_is_rejected_on_the_inherited_browser_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");

    let bin_path = package_dir.join("bin/semver.js");
    let check = run_kali(dir.path(), ["check", bin_path.to_str().unwrap()]);
    assert!(
        !check.status.success(),
        "browser semver package bin entrypoint should be rejected on the inherited browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check_stderr.contains("E3100"), "stderr: {check_stderr}");
    assert!(check_stderr.contains("require"), "stderr: {check_stderr}");

    let check_json = run_kali(
        dir.path(),
        ["--output", "json", "check", bin_path.to_str().unwrap()],
    );
    assert!(
        !check_json.status.success(),
        "json browser check should surface the inherited bin entrypoint rejection\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], false);
    assert_eq!(check_envelope["exitCode"], 1);
    assert_eq!(
        check_envelope["payload"],
        serde_json::json!({"errorCount": 1, "filesChecked": 1, "warningCount": 0})
    );
    assert!(
        check_envelope["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["code"] == "E3100"),
        "check json: {check_envelope}"
    );

    let build = run_kali(
        dir.path(),
        ["build", "--bundle", bin_path.to_str().unwrap()],
    );
    assert!(
        !build.status.success(),
        "browser semver package bin entrypoint should be rejected during inherited bundle emission\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stderr = String::from_utf8_lossy(&build.stderr);
    assert!(build_stderr.contains("E3100"), "stderr: {build_stderr}");
    assert!(build_stderr.contains("require"), "stderr: {build_stderr}");

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            bin_path.to_str().unwrap(),
        ],
    );
    assert!(
        !build_json.status.success(),
        "json browser build should surface the inherited bin entrypoint rejection\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], false);
    assert_eq!(build_envelope["exitCode"], 1);
    assert_eq!(build_envelope["payload"], serde_json::Value::Null);
    assert!(
        build_envelope["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["code"] == "E3100"),
        "build json: {build_envelope}"
    );
}

#[test]
fn browser_corpus_pi_coding_agent_style_package_bin_entrypoint_is_rejected_on_the_browser_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let bin_path = package_dir.join("dist/cli.js");
    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", bin_path.to_str().unwrap()],
    );
    assert!(
        !check.status.success(),
        "browser pi-coding-agent package bin entrypoint should be rejected on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check_stderr.contains("E3100"), "stderr: {check_stderr}");
    assert!(check_stderr.contains("require"), "stderr: {check_stderr}");

    let check_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "check",
            "--api",
            "browser",
            bin_path.to_str().unwrap(),
        ],
    );
    assert!(
        !check_json.status.success(),
        "json browser check should surface the pi-coding-agent bin entrypoint rejection\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], false);
    assert_eq!(check_envelope["exitCode"], 1);
    assert_eq!(
        check_envelope["payload"],
        serde_json::json!({"errorCount": 1, "filesChecked": 1, "warningCount": 0})
    );
    assert!(
        check_envelope["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["code"] == "E3100"),
        "check json: {check_envelope}"
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            bin_path.to_str().unwrap(),
        ],
    );
    assert!(
        !build.status.success(),
        "browser pi-coding-agent package bin entrypoint should be rejected during bundle emission\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let build_stderr = String::from_utf8_lossy(&build.stderr);
    assert!(build_stderr.contains("E3100"), "stderr: {build_stderr}");
    assert!(build_stderr.contains("require"), "stderr: {build_stderr}");

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            "--api",
            "browser",
            bin_path.to_str().unwrap(),
        ],
    );
    assert!(
        !build_json.status.success(),
        "json browser build should surface the pi-coding-agent bin entrypoint rejection\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], false);
    assert_eq!(build_envelope["exitCode"], 1);
    assert_eq!(build_envelope["payload"], serde_json::Value::Null);
    assert!(
        build_envelope["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["code"] == "E3100"),
        "build json: {build_envelope}"
    );
}

#[test]
fn browser_corpus_pi_coding_agent_style_package_bin_entrypoint_is_rejected_on_the_inherited_browser_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let bin_path = package_dir.join("dist/cli.js");
    let check = run_kali(dir.path(), ["check", bin_path.to_str().unwrap()]);
    assert!(
        !check.status.success(),
        "browser pi-coding-agent package bin entrypoint should be rejected on the inherited browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check_stderr.contains("E3100"), "stderr: {check_stderr}");
    assert!(check_stderr.contains("require"), "stderr: {check_stderr}");

    let check_json = run_kali(
        dir.path(),
        ["--output", "json", "check", bin_path.to_str().unwrap()],
    );
    assert!(
        !check_json.status.success(),
        "json browser check should surface the inherited pi-coding-agent bin entrypoint rejection\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], false);
    assert_eq!(check_envelope["exitCode"], 1);
    assert_eq!(
        check_envelope["payload"],
        serde_json::json!({"errorCount": 1, "filesChecked": 1, "warningCount": 0})
    );
    assert!(
        check_envelope["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["code"] == "E3100"),
        "check json: {check_envelope}"
    );
}

#[test]
fn browser_corpus_packages_that_block_the_selected_path_are_rejected_in_browser_context_with_inherited_browser_api_surface_on_js_input(
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

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            !check.status.success(),
            "browser-blocked package {package} should be rejected in browser context on JS input when the browser apiSurface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let stderr = String::from_utf8_lossy(&check.stderr);
        assert!(
            stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the import-resolution failure on JS input when the browser apiSurface is inherited\nstderr: {}",
            stderr
        );
        assert!(
            stderr.contains("could not be resolved"),
            "browser-blocked package {package} should not fall back to the non-browser entry on JS input when the browser apiSurface is inherited\nstderr: {}",
            stderr
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            !build.status.success(),
            "browser-blocked package {package} should also be rejected during bundle emission on JS input when the browser apiSurface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let build_stderr = String::from_utf8_lossy(&build.stderr);
        assert!(
            build_stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the bundle-time import-resolution failure on JS input when the browser apiSurface is inherited\nstderr: {}",
            build_stderr
        );
        assert!(
            build_stderr.contains("could not be resolved"),
            "browser-blocked package {package} should not fall back to the non-browser entry during bundle emission on JS input when the browser apiSurface is inherited\nstderr: {}",
            build_stderr
        );

        let json_check = run_kali(
            dir.path(),
            ["--output", "json", "check", source_path.to_str().unwrap()],
        );
        assert_browser_blocked_package_json_rejection(&json_check, "check");

        let json_build = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_build, "build");
    }
}

#[test]
fn browser_corpus_pi_coding_agent_style_package_remains_checkable_and_deployable_through_host_on_js_input(
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
    .expect("write pi-coding-agent browser JS source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the browser surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let check_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "check",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        check_json.status.success(),
        "pi-coding-agent corpus package content should be checkable on the browser surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], true);
    assert_eq!(check_envelope["exitCode"], 0);
    assert_eq!(check_envelope["payload"]["filesChecked"], 1);
    assert_eq!(check_envelope["payload"]["errorCount"], 0);
    assert_eq!(check_envelope["payload"]["warningCount"], 0);

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be deployable-through-host via browser bundle in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build_json.status.success(),
        "pi-coding-agent corpus package content should be deployable-through-host via browser bundle in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], true);
    assert_eq!(build_envelope["exitCode"], 0);
    assert_eq!(build_envelope["payload"]["artifactKind"], "bundle");
}

#[test]
fn browser_corpus_pi_coding_agent_style_package_remains_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
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
    .expect("write pi-coding-agent browser JS source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the browser surface in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let check_json = run_kali(
        dir.path(),
        ["--output", "json", "check", source_path.to_str().unwrap()],
    );
    assert!(
        check_json.status.success(),
        "pi-coding-agent corpus package content should be checkable on the browser surface in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], true);
    assert_eq!(check_envelope["exitCode"], 0);
    assert_eq!(check_envelope["payload"]["filesChecked"], 1);
    assert_eq!(check_envelope["payload"]["errorCount"], 0);
    assert_eq!(check_envelope["payload"]["warningCount"], 0);

    let build = run_kali(
        dir.path(),
        ["build", "--bundle", source_path.to_str().unwrap()],
    );
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be deployable-through-host via browser bundle in JS input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build_json.status.success(),
        "pi-coding-agent corpus package content should be deployable-through-host via browser bundle in JS input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], true);
    assert_eq!(build_envelope["exitCode"], 0);
    assert_eq!(build_envelope["payload"]["artifactKind"], "bundle");
}

#[test]
fn browser_corpus_packages_with_exports_maps_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("hono", "client"),
        ("svelte", "compiler"),
        ("lit", "decorators"),
        ("solid-js", "store"),
        ("vue-router", "history"),
        ("react-router", "dom"),
        ("react-router-dom", "dom"),
        ("jotai", "store"),
        ("xstate", "react"),
        ("@remix-run/react", "links"),
        ("react-dom", "client"),
    ] {
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
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with exports map should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with exports map should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_exports_maps_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("hono", "client"),
        ("svelte", "compiler"),
        ("lit", "decorators"),
        ("solid-js", "store"),
        ("vue-router", "history"),
        ("react-router", "dom"),
        ("react-router-dom", "dom"),
        ("jotai", "store"),
        ("xstate", "react"),
        ("@remix-run/react", "links"),
        ("react-dom", "client"),
    ] {
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
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with exports map should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with exports map should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_pattern_exports_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
        ("vue-router", "matcher"),
        ("react-router", "routes"),
        ("react-router-dom", "dom"),
        ("xstate", "react"),
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
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser pattern-export package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser pattern-export package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_pattern_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("hono", "client"),
        ("solid-js", "web"),
        ("vue-router", "matcher"),
        ("react-router", "routes"),
        ("react-router-dom", "dom"),
        ("xstate", "react"),
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
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser pattern-export package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser pattern-export package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_typed_export_branches_remain_checkable_and_deployable_through_host()
{
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
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with typed export branches should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with typed export branches should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_typed_export_branches_remain_checkable_and_deployable_through_host_on_js_input(
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser package {package} with typed export branches should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser package {package} with typed export branches should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_dual_exports_remain_checkable_and_deployable_through_host() {
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
            &format!(
                "export default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "module.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser dual package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser dual package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_dual_exports_remain_checkable_and_deployable_through_host_on_js_input(
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
            &format!(
                "export default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "module.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser dual package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser dual package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_exports_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("jotai", "store"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser conditional-exports package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser conditional-exports package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("jotai", "store"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser conditional-exports package {package} should resolve its browser branch on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser conditional-exports package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_exports_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("dayjs", "locale"),
        ("jotai", "store"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
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

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "browser conditional-exports package {package} should resolve its browser branch on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "browser conditional-exports package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_module_entries_remain_checkable_and_deployable_through_host() {
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
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser module-only package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser module-only package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_module_entries_remain_checkable_and_deployable_through_host_on_js_input(
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
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser module-only package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser module-only package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_module_entries_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
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
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "browser module-only package {package} should be checkable on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let check_json = run_kali(
            dir.path(),
            ["--output", "json", "check", source_path.to_str().unwrap()],
        );
        assert!(
            check_json.status.success(),
            "browser module-only package {package} should be checkable on js input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check_json.stdout),
            String::from_utf8_lossy(&check_json.stderr)
        );
        let check_envelope = parse_json_stdout(&check_json);
        assert_eq!(check_envelope["schemaVersion"], 1);
        assert_eq!(check_envelope["command"], "check");
        assert_eq!(check_envelope["success"], true);
        assert_eq!(check_envelope["exitCode"], 0);
        assert_eq!(check_envelope["payload"]["filesChecked"], 1);
        assert_eq!(check_envelope["payload"]["errorCount"], 0);
        assert_eq!(check_envelope["payload"]["warningCount"], 0);

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "browser module-only package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let build_json = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build_json.status.success(),
            "browser module-only package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build_json.stdout),
            String::from_utf8_lossy(&build_json.stderr)
        );
        let build_envelope = parse_json_stdout(&build_json);
        assert_eq!(build_envelope["schemaVersion"], 1);
        assert_eq!(build_envelope["command"], "build");
        assert_eq!(build_envelope["success"], true);
        assert_eq!(build_envelope["exitCode"], 0);
        let payload = build_envelope["payload"]
            .as_object()
            .expect("build payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
    }
}

#[test]
fn browser_corpus_packages_with_module_entry_chains_remain_checkable_and_deployable_through_host_on_js_input(
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser module-chain package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser module-chain package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_replacement_maps_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:node'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser replacement-map package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser replacement-map package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_js_entrypoints_with_browser_replacement_maps_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:node'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser JS entrypoint package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let check_json = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "check",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            check_json.status.success(),
            "browser JS entrypoint package {package} should resolve its browser branch for check on js input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check_json.stdout),
            String::from_utf8_lossy(&check_json.stderr)
        );
        let check_envelope = parse_json_stdout(&check_json);
        assert_eq!(check_envelope["schemaVersion"], 1);
        assert_eq!(check_envelope["command"], "check");
        assert_eq!(check_envelope["success"], true);
        assert_eq!(check_envelope["exitCode"], 0);
        assert_eq!(check_envelope["payload"]["filesChecked"], 1);
        assert_eq!(check_envelope["payload"]["errorCount"], 0);
        assert_eq!(check_envelope["payload"]["warningCount"], 0);

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser JS entrypoint package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let build_json = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build_json.status.success(),
            "browser JS entrypoint package {package} should be deployable-through-host via bundle on js input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build_json.stdout),
            String::from_utf8_lossy(&build_json.stderr)
        );
        let build_envelope = parse_json_stdout(&build_json);
        assert_eq!(build_envelope["schemaVersion"], 1);
        assert_eq!(build_envelope["command"], "build");
        assert_eq!(build_envelope["success"], true);
        assert_eq!(build_envelope["exitCode"], 0);
        let payload = build_envelope["payload"]
            .as_object()
            .expect("build payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
    }
}

#[test]
fn browser_corpus_js_entrypoints_with_minimized_cjs_esm_interop_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("react", "jsx-runtime"),
        ("preact", "hooks"),
        ("vue", "runtime-dom"),
        ("@reduxjs/toolkit", "query"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            &format!(
                "module.exports = function root() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:node'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser mixed-format package {package} should resolve its browser branch\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let check_json = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "check",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            check_json.status.success(),
            "browser mixed-format package {package} should resolve its browser branch for check on js input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check_json.stdout),
            String::from_utf8_lossy(&check_json.stderr)
        );
        let check_envelope = parse_json_stdout(&check_json);
        assert_eq!(check_envelope["schemaVersion"], 1);
        assert_eq!(check_envelope["command"], "check");
        assert_eq!(check_envelope["success"], true);
        assert_eq!(check_envelope["exitCode"], 0);
        assert_eq!(check_envelope["payload"]["filesChecked"], 1);
        assert_eq!(check_envelope["payload"]["errorCount"], 0);
        assert_eq!(check_envelope["payload"]["warningCount"], 0);

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser mixed-format package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let build_json = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build_json.status.success(),
            "browser mixed-format package {package} should be deployable-through-host via bundle on js input with json output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build_json.stdout),
            String::from_utf8_lossy(&build_json.stderr)
        );
        let build_envelope = parse_json_stdout(&build_json);
        assert_eq!(build_envelope["schemaVersion"], 1);
        assert_eq!(build_envelope["command"], "build");
        assert_eq!(build_envelope["success"], true);
        assert_eq!(build_envelope["exitCode"], 0);
        let payload = build_envelope["payload"]
            .as_object()
            .expect("build payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
    }
}

#[test]
fn browser_corpus_packages_with_browser_string_entries_remain_checkable_and_deployable_through_host(
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
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-entry package {package} should resolve its browser override\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-entry package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_string_entries_and_web_baseline_primitives_remain_checkable_and_deployable_through_host(
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
            &format!(
                "const controller = new AbortController();\nconst signal = controller.signal;\nsignal.addEventListener('abort', () => {{\n}});\nconst target = new EventTarget();\ntarget.addEventListener('tick', () => {{\n  controller.abort();\n}});\ntarget.dispatchEvent(new CustomEvent('tick'));\nconst query = new URLSearchParams('alpha=1&beta=two+words');\nquery.set('beta', 'browser');\nstructuredClone(new Blob(['browser corpus']));\nconst encoder = new TextEncoder();\nencoder.encode('browser corpus');\nexport default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string/web-baseline package {package} should resolve its browser override\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string/web-baseline package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_string_web_baseline_primitives_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
) {
    for package in ["react", "preact", "vue"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_string_web_baseline_package(dir.path(), package);
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "browser string/web-baseline package {package} should resolve its browser override on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "browser string/web-baseline package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_string_entries_remain_checkable_and_deployable_through_host_on_js_input(
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
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
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
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-entry package {package} should resolve its browser override on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-entry package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_browser_string_entries_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
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
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
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
        .expect("write browser source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "browser string-entry package {package} should resolve its browser override on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "browser string-entry package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_string_exports_remain_checkable_and_deployable_through_host() {
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
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-exports package {package} should resolve its exports string\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-exports package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_string_exports_remain_checkable_and_deployable_through_host_on_js_input(
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
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser string-exports package {package} should resolve its exports string on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser string-exports package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_string_exports_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
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
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "browser string-exports package {package} should resolve its exports string on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "browser string-exports package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host(
) {
    for package in [
        "react",
        "preact",
        "vue",
        "solid-js",
        "date-fns",
        "dayjs",
        "d3",
        "recharts",
        "luxon",
        "graphql",
        "lodash-es",
        "nanoid",
        "ramda",
        "rxjs",
        "uuid",
        "clsx",
        "react-router",
        "zustand",
        "zod",
        "svelte",
        "lit",
        "axios",
        "ajv",
        "immer",
        "next",
        "react-helmet-async",
        "hono",
        "@vueuse/core",
        "@apollo/client",
        "@emotion/react",
        "@reduxjs/toolkit",
        "@floating-ui/react",
        "@headlessui/react",
        "@chakra-ui/react",
        "@mantine/core",
        "@emotion/styled",
        "@heroicons/react",
        "lucide-react",
        "@storybook/react",
        "@stripe/react-stripe-js",
        "@mui/material",
        "@radix-ui/react-dialog",
        "@tanstack/react-query",
        "@tanstack/react-table",
        "@tanstack/table-core",
        "@tanstack/react-virtual",
        "@testing-library/dom",
        "@testing-library/user-event",
        "@playwright/test",
        "mobx",
        "redux",
        "recoil",
        "mitt",
        "swr",
        "formik",
        "jotai",
        "pinia",
        "xstate",
        "valtio",
        "superjson",
        "@jridgewell/sourcemap-codec",
        "@babel/runtime",
        "@npmcli/package-json",
        "query-string",
        "yup",
        "msw",
        "yaml",
        "react-hook-form",
        "@tanstack/react-form",
        "@tanstack/router",
        "@tanstack/react-router",
        "@tanstack/query-core",
        "path-to-regexp",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        write_web_baseline_interop_source(&source_path, package);

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser web-baseline package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser web-baseline package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in [
        "react",
        "preact",
        "vue",
        "solid-js",
        "date-fns",
        "dayjs",
        "d3",
        "recharts",
        "luxon",
        "graphql",
        "lodash-es",
        "nanoid",
        "ramda",
        "rxjs",
        "uuid",
        "clsx",
        "react-router",
        "zustand",
        "zod",
        "svelte",
        "lit",
        "axios",
        "ajv",
        "immer",
        "next",
        "react-helmet-async",
        "hono",
        "@vueuse/core",
        "@apollo/client",
        "@emotion/react",
        "@reduxjs/toolkit",
        "@floating-ui/react",
        "@headlessui/react",
        "@chakra-ui/react",
        "@mantine/core",
        "@emotion/styled",
        "@heroicons/react",
        "lucide-react",
        "@storybook/react",
        "@stripe/react-stripe-js",
        "@mui/material",
        "@radix-ui/react-dialog",
        "@tanstack/react-query",
        "@tanstack/react-table",
        "@tanstack/table-core",
        "@tanstack/react-virtual",
        "@testing-library/dom",
        "@testing-library/user-event",
        "@playwright/test",
        "mobx",
        "redux",
        "recoil",
        "mitt",
        "swr",
        "formik",
        "jotai",
        "pinia",
        "xstate",
        "valtio",
        "superjson",
        "@jridgewell/sourcemap-codec",
        "@babel/runtime",
        "@npmcli/package-json",
        "query-string",
        "yup",
        "msw",
        "yaml",
        "react-hook-form",
        "@tanstack/react-form",
        "@tanstack/router",
        "@tanstack/react-router",
        "@tanstack/query-core",
        "path-to-regexp",
    ] {
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser web-baseline package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser web-baseline package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
) {
    for package in [
        "react",
        "preact",
        "vue",
        "solid-js",
        "date-fns",
        "dayjs",
        "d3",
        "recharts",
        "luxon",
        "graphql",
        "lodash-es",
        "nanoid",
        "ramda",
        "rxjs",
        "uuid",
        "clsx",
        "react-router",
        "zustand",
        "zod",
        "svelte",
        "lit",
        "axios",
        "ajv",
        "immer",
        "next",
        "react-helmet-async",
        "hono",
        "@vueuse/core",
        "@apollo/client",
        "@emotion/react",
        "@reduxjs/toolkit",
        "@floating-ui/react",
        "@headlessui/react",
        "@chakra-ui/react",
        "@mantine/core",
        "@emotion/styled",
        "@heroicons/react",
        "lucide-react",
        "@storybook/react",
        "@stripe/react-stripe-js",
        "@mui/material",
        "@radix-ui/react-dialog",
        "@tanstack/react-query",
        "@tanstack/react-table",
        "@tanstack/table-core",
        "@tanstack/react-virtual",
        "@testing-library/dom",
        "@testing-library/user-event",
        "@playwright/test",
        "mobx",
        "redux",
        "recoil",
        "mitt",
        "swr",
        "formik",
        "jotai",
        "pinia",
        "xstate",
        "valtio",
        "superjson",
        "@jridgewell/sourcemap-codec",
        "@babel/runtime",
        "@npmcli/package-json",
        "query-string",
        "yup",
        "msw",
        "yaml",
        "react-hook-form",
        "@tanstack/react-form",
        "@tanstack/router",
        "@tanstack/react-router",
        "@tanstack/query-core",
        "path-to-regexp",
    ] {
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

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "browser web-baseline package {package} should be checkable on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "browser web-baseline package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_internal_browser_rewrites_remain_checkable_and_deployable_through_host(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "import helper from './internal.js';\nexport default function root() { return 'node:' + helper(); }\n",
            "import helper from './internal.js';\nexport default function root() { return 'browser:' + helper(); }\n",
            "internal",
            &format!(
                "export default function helper() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function helper() {{ return '{package}:browser'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser internal-browser-rewrite package {package} should resolve its browser rewrite chain\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser internal-browser-rewrite package {package} should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_internal_browser_rewrites_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "import helper from './internal.js';\nexport default function root() { return 'node:' + helper(); }\n",
            "import helper from './internal.js';\nexport default function root() { return 'browser:' + helper(); }\n",
            "internal",
            &format!(
                "export default function helper() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function helper() {{ return '{package}:browser'; }}\n",
                package = package
            ),
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
        .expect("write browser source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "browser internal-browser-rewrite package {package} should resolve its browser rewrite chain on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "browser internal-browser-rewrite package {package} should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_exports_maps_remain_checkable_and_deployable_through_host() {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
        ("chart.js", "auto"),
        ("zustand", "vanilla"),
    ] {
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
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with exports map should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with exports map should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_exports_maps_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
        ("chart.js", "auto"),
        ("zustand", "vanilla"),
    ] {
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
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with exports map should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with exports map should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_exports_maps_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited(
) {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
        ("chart.js", "auto"),
        ("zustand", "vanilla"),
    ] {
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
        .expect("write browser source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "scoped browser package {package} with exports map should be checkable on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            ["build", "--bundle", source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with exports map should be deployable-through-host via bundle on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_with_internal_browser_rewrites_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for package in ["solid-js", "lit"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_replacement_map_package(
            dir.path(),
            package,
            "import helper from './internal.js';\nexport default function root() { return 'node:' + helper(); }\n",
            "import helper from './internal.js';\nexport default function root() { return 'browser:' + helper(); }\n",
            "internal",
            &format!(
                "export default function helper() {{ return '{package}:node'; }}\n",
                package = package
            ),
            &format!(
                "export default function helper() {{ return '{package}:browser'; }}\n",
                package = package
            ),
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
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "browser internal-browser-rewrite package {package} should resolve its browser rewrite chain on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "browser internal-browser-rewrite package {package} should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_browser_condition_exports_remain_checkable_and_deployable_through_host(
) {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("zustand", "vanilla"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with browser condition exports should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with browser condition exports should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_scoped_packages_with_browser_condition_exports_remain_checkable_and_deployable_through_host_on_js_input(
) {
    for (package, subpath) in [
        ("@emotion/react", "jsx-runtime"),
        ("@apollo/client", "cache"),
        ("@chakra-ui/react", "system"),
        ("@floating-ui/react", "dom"),
        ("@headlessui/react", "dialog"),
        ("@heroicons/react", "solid"),
        ("@radix-ui/react-dialog", "dialog"),
        ("@storybook/react", "preview-api"),
        ("@mui/material", "styles"),
        ("@stripe/react-stripe-js", "elements"),
        ("@mantine/core", "styles"),
        ("@tanstack/react-query", "query-core"),
        ("@tanstack/query-core", "core"),
        ("@tanstack/table-core", "table"),
        ("@tanstack/router", "router"),
        ("@tanstack/react-router", "router"),
        ("zustand", "vanilla"),
        ("@remix-run/react", "links"),
        ("@vueuse/core", "index"),
        ("react-dom", "client"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("browser"));
        write_browser_condition_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function root() {{ return '{package}:browser'; }}\n",
                package = package
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function root() {{ return '{package}:import'; }}\n",
                package = package
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function root() {{ return '{package}:require'; }};\n",
                package = package
            ),
            subpath,
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:browser'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from 'node:assert';\nassert.ok(true);\nexport default function subpath() {{ return '{package}:{subpath}:import'; }}\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "const assert = require('node:assert');\nassert.ok(true);\nmodule.exports = function subpath() {{ return '{package}:{subpath}:require'; }};\n",
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "scoped browser package {package} with browser condition exports should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "scoped browser package {package} with browser condition exports should be deployable-through-host via bundle on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
    }
}

#[test]
fn browser_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return Deno.env.get('HOME') ? 1 : 2; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser condition package browser-deno should resolve its browser branch for check\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "browser condition package browser-deno should be deployable-through-host via bundle\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn browser_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return Deno.env.get('HOME') ? 1 : 2; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser condition package browser-deno should resolve its browser branch for check on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let check_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "check",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        check_json.status.success(),
        "browser condition package browser-deno should resolve its browser branch for check on js input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], true);
    assert_eq!(check_envelope["exitCode"], 0);
    assert_eq!(check_envelope["payload"]["filesChecked"], 1);
    assert_eq!(check_envelope["payload"]["errorCount"], 0);
    assert_eq!(check_envelope["payload"]["warningCount"], 0);

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build_json.status.success(),
        "browser condition package browser-deno should be deployable-through-host via bundle on js input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], true);
    assert_eq!(build_envelope["exitCode"], 0);
    let payload = build_envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
}

#[test]
fn browser_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_on_js_input_when_the_browser_api_surface_is_inherited(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("browser"));
    write_browser_and_deno_condition_package(
        dir.path(),
        "browser-deno",
        "export default function describe() { return 0; }\n",
        "export default function describe() { return Deno.env.get('HOME') ? 1 : 2; }\n",
    );
    write_types_stub_package(dir.path(), "browser-deno");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import describe from 'browser-deno';\nconsole.log(describe());\n",
    )
    .expect("write browser source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "browser condition package browser-deno should resolve its browser branch for check on js input when the browser api surface is inherited\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let check_json = run_kali(
        dir.path(),
        ["--output", "json", "check", source_path.to_str().unwrap()],
    );
    assert!(
        check_json.status.success(),
        "browser condition package browser-deno should resolve its browser branch for check on js input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check_json.stdout),
        String::from_utf8_lossy(&check_json.stderr)
    );
    let check_envelope = parse_json_stdout(&check_json);
    assert_eq!(check_envelope["schemaVersion"], 1);
    assert_eq!(check_envelope["command"], "check");
    assert_eq!(check_envelope["success"], true);
    assert_eq!(check_envelope["exitCode"], 0);
    assert_eq!(check_envelope["payload"]["filesChecked"], 1);
    assert_eq!(check_envelope["payload"]["errorCount"], 0);
    assert_eq!(check_envelope["payload"]["warningCount"], 0);

    let build_json = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "build",
            "--bundle",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build_json.status.success(),
        "browser condition package browser-deno should be deployable-through-host via bundle on js input when the browser api surface is inherited with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], true);
    assert_eq!(build_envelope["exitCode"], 0);
    let payload = build_envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(payload["artifactKind"], "bundle");
    assert_eq!(payload["bundleFormat"], "esm");
}

#[test]
fn browser_corpus_packages_prefer_browser_condition_over_deno_condition_on_the_browser_surface_when_a_harness_command_is_configured(
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
fn browser_corpus_packages_with_browser_replacement_maps_remain_checkable_and_deployable_through_host_on_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        for explicit_browser_surface in [true, false] {
            let dir = tempdir().expect("tempdir");
            let package = "browser-replacement-map";
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

            let source_path = dir.path().join(format!("main.{extension}"));
            fs::write(
                &source_path,
                format!(
                    "import describe from '{package}';\nimport helper from '{package}/internal';\nconsole.log(describe(), helper());\n",
                    package = package
                ),
            )
            .expect("write browser source");

            let source = source_path.to_str().unwrap();
            let check = if explicit_browser_surface {
                run_kali(dir.path(), ["check", "--api", "browser", source])
            } else {
                run_kali(dir.path(), ["check", source])
            };
            assert!(
                check.status.success(),
                "browser replacement-map package {package} should be checkable on the browser surface in {extension} input when the browser api surface is {}\nstdout: {}\nstderr: {}",
                if explicit_browser_surface { "explicit" } else { "inherited" },
                String::from_utf8_lossy(&check.stdout),
                String::from_utf8_lossy(&check.stderr)
            );

            let check_json = if explicit_browser_surface {
                run_kali(
                    dir.path(),
                    ["--output", "json", "check", "--api", "browser", source],
                )
            } else {
                run_kali(dir.path(), ["--output", "json", "check", source])
            };
            assert!(
                check_json.status.success(),
                "browser replacement-map package {package} should be checkable on the browser surface in {extension} input with json output when the browser api surface is {}\nstdout: {}\nstderr: {}",
                if explicit_browser_surface { "explicit" } else { "inherited" },
                String::from_utf8_lossy(&check_json.stdout),
                String::from_utf8_lossy(&check_json.stderr)
            );
            let check_envelope = parse_json_stdout(&check_json);
            assert_eq!(check_envelope["schemaVersion"], 1);
            assert_eq!(check_envelope["command"], "check");
            assert_eq!(check_envelope["success"], true);
            assert_eq!(check_envelope["exitCode"], 0);
            assert_eq!(check_envelope["payload"]["filesChecked"], 1);
            assert_eq!(check_envelope["payload"]["errorCount"], 0);
            assert_eq!(check_envelope["payload"]["warningCount"], 0);

            let build = if explicit_browser_surface {
                run_kali(
                    dir.path(),
                    ["build", "--bundle", "--api", "browser", source],
                )
            } else {
                run_kali(dir.path(), ["build", "--bundle", source])
            };
            assert!(
                build.status.success(),
                "browser replacement-map package {package} should be deployable-through-host via bundle on {extension} input when the browser api surface is {}\nstdout: {}\nstderr: {}",
                if explicit_browser_surface { "explicit" } else { "inherited" },
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            );

            let build_json = if explicit_browser_surface {
                run_kali(
                    dir.path(),
                    [
                        "--output", "json", "build", "--bundle", "--api", "browser", source,
                    ],
                )
            } else {
                run_kali(
                    dir.path(),
                    ["--output", "json", "build", "--bundle", source],
                )
            };
            assert!(
                build_json.status.success(),
                "browser replacement-map package {package} should be deployable-through-host via bundle on {extension} input with json output when the browser api surface is {}\nstdout: {}\nstderr: {}",
                if explicit_browser_surface { "explicit" } else { "inherited" },
                String::from_utf8_lossy(&build_json.stdout),
                String::from_utf8_lossy(&build_json.stderr)
            );
            let build_envelope = parse_json_stdout(&build_json);
            assert_eq!(build_envelope["schemaVersion"], 1);
            assert_eq!(build_envelope["command"], "build");
            assert_eq!(build_envelope["success"], true);
            assert_eq!(build_envelope["exitCode"], 0);
            let payload = build_envelope["payload"]
                .as_object()
                .expect("build payload object");
            assert_eq!(payload["artifactKind"], "bundle");
            assert_eq!(payload["bundleFormat"], "esm");
        }
    }
}

#[test]
fn browser_corpus_packages_that_block_the_selected_path_are_rejected_in_browser_context() {
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
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write browser source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            !check.status.success(),
            "browser-blocked package {package} should be rejected in browser context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let stderr = String::from_utf8_lossy(&check.stderr);
        assert!(
            stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the import-resolution failure\nstderr: {}",
            stderr
        );
        assert!(
            stderr.contains("could not be resolved"),
            "browser-blocked package {package} should not fall back to the non-browser entry\nstderr: {}",
            stderr
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            !build.status.success(),
            "browser-blocked package {package} should also be rejected during bundle emission\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let build_stderr = String::from_utf8_lossy(&build.stderr);
        assert!(
            build_stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the bundle-time import-resolution failure\nstderr: {}",
            build_stderr
        );

        let json_check = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "check",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_check, "check");

        let json_build = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_build, "build");
    }
}

#[test]
fn browser_corpus_packages_that_block_the_selected_path_are_rejected_in_browser_context_on_js_input(
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

        let check = run_kali(
            dir.path(),
            ["check", "--api", "browser", source_path.to_str().unwrap()],
        );
        assert!(
            !check.status.success(),
            "browser-blocked package {package} should be rejected in browser context on JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let stderr = String::from_utf8_lossy(&check.stderr);
        assert!(
            stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the import-resolution failure on JS input\nstderr: {}",
            stderr
        );
        assert!(
            stderr.contains("could not be resolved"),
            "browser-blocked package {package} should not fall back to the non-browser entry on JS input\nstderr: {}",
            stderr
        );

        let build = run_kali(
            dir.path(),
            [
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            !build.status.success(),
            "browser-blocked package {package} should also be rejected during bundle emission on JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let build_stderr = String::from_utf8_lossy(&build.stderr);
        assert!(
            build_stderr.contains("error[E3000]"),
            "browser-blocked package {package} should surface the bundle-time import-resolution failure on JS input\nstderr: {}",
            build_stderr
        );

        let json_check = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "check",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_check, "check");

        let json_build = run_kali(
            dir.path(),
            [
                "--output",
                "json",
                "build",
                "--bundle",
                "--api",
                "browser",
                source_path.to_str().unwrap(),
            ],
        );
        assert_browser_blocked_package_json_rejection(&json_build, "build");
    }
}

#[test]
fn browser_corpus_packages_with_spawn_tools_remain_checkable_and_bundleable_on_the_browser_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write browser host source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser host package {package} should remain checkable on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "browser host package {package} should remain bundleable on the browser surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn browser_corpus_packages_with_spawn_tools_remain_checkable_and_bundleable_on_the_browser_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let package = "spawn-tools";
    write_manifest(dir.path(), Some("browser"));
    write_deno_host_package(
        dir.path(),
        package,
        "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
    );
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        format!(
            "import root from '{package}';\nconsole.log(root());\n",
            package = package
        ),
    )
    .expect("write browser host TS source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "browser", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "browser host package {package} should remain checkable on the browser surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(
        dir.path(),
        [
            "build",
            "--bundle",
            "--api",
            "browser",
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "browser host package {package} should remain bundleable on the browser surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}
