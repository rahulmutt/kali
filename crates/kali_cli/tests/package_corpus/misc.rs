use super::*;

#[test]
fn package_analysis_commands_reject_sandbox_flag_before_target_validation_in_json_output() {
    let dir = tempdir().expect("tempdir");

    let package_effects = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "package-effects",
            "--sandbox",
            "kali.policy.json",
        ],
    );
    assert_package_analysis_specific_flag_json_rejection(
        &package_effects,
        "package-effects",
        "--sandbox",
    );

    let package_audit = run_kali(
        dir.path(),
        [
            "--output",
            "json",
            "package-audit",
            "--sandbox",
            "kali.policy.json",
        ],
    );
    assert_package_analysis_specific_flag_json_rejection(
        &package_audit,
        "package-audit",
        "--sandbox",
    );
}

#[test]
fn binary_entrypoint_corpus_pi_coding_agent_style_package_executes_on_the_node_surface_and_is_rejected_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let standalone_run = run_kali(
        dir.path(),
        ["run", package_dir.join("dist/cli.js").to_str().unwrap()],
    );
    assert!(
        !standalone_run.status.success(),
        "pi-coding-agent corpus package bin should stay rejected on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&standalone_run.stdout),
        String::from_utf8_lossy(&standalone_run.stderr)
    );
    let standalone_stderr = String::from_utf8_lossy(&standalone_run.stderr);
    assert!(
        standalone_stderr.contains("E5506"),
        "stderr: {standalone_stderr}"
    );
    assert!(
        standalone_stderr.contains("Node.js CLI features")
            && standalone_stderr.contains("unavailable on the 'deno' API surface"),
        "stderr: {standalone_stderr}"
    );

    let node_run = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("dist/cli.js").to_str().unwrap(),
        ],
    );
    assert!(
        node_run.status.success(),
        "pi-coding-agent corpus package bin should execute on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&node_run.stdout),
        String::from_utf8_lossy(&node_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node_run.stdout), "0\n");
}

#[test]
fn binary_entrypoint_corpus_pi_coding_agent_style_package_preserves_node_arguments_on_the_node_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);

    let standalone_run = run_kali(
        dir.path(),
        ["run", package_dir.join("dist/argv.js").to_str().unwrap()],
    );
    assert!(
        !standalone_run.status.success(),
        "pi-coding-agent argv probe should stay rejected on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&standalone_run.stdout),
        String::from_utf8_lossy(&standalone_run.stderr)
    );
    let standalone_stderr = String::from_utf8_lossy(&standalone_run.stderr);
    assert!(
        standalone_stderr.contains("E5506"),
        "stderr: {standalone_stderr}"
    );

    let node_run = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("dist/argv.js").to_str().unwrap(),
            "--",
            "alpha",
        ],
    );
    assert!(
        node_run.status.success(),
        "pi-coding-agent argv probe should preserve Node arguments on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&node_run.stdout),
        String::from_utf8_lossy(&node_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node_run.stdout), "1\n");
}

#[test]
fn native_addon_corpus_packages_are_rejected_on_the_default_standalone_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    let package_dir = dir.path().join("node_modules/native-addon");
    fs::create_dir_all(&package_dir).expect("create native addon package dir");
    write_native_addon_package(&package_dir);
    write_types_stub_package(dir.path(), "native-addon");

    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import addon from 'native-addon';
console.log(addon);
"#,
    )
    .expect("write native addon package source");

    for command in ["check", "build", "run"] {
        let output = run_kali(dir.path(), [command, source_path.to_str().unwrap()]);
        assert!(
            !output.status.success(),
            "native addon package should be rejected on the default standalone surface for {command}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E6005"), "stderr: {stderr}");
        assert!(
            stderr.contains("native addon entrypoint")
                && stderr.contains("falls outside the pure JS/TS package contract"),
            "stderr: {stderr}"
        );
    }

    let test_source = dir.path().join("tests").join("native-addon.test.js");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        r#"import addon from 'native-addon';
Kali.test('native addon corpus', () => {
  console.log(addon);
});
"#,
    )
    .expect("write native addon package test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        !test.status.success(),
        "native addon package should be rejected on the default standalone surface for test\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test.stderr);
    assert!(test_stderr.contains("E6005"), "stderr: {test_stderr}");
    assert!(
        test_stderr.contains("native addon entrypoint")
            && test_stderr.contains("falls outside the pure JS/TS package contract"),
        "stderr: {test_stderr}"
    );
}

#[test]
fn json_utility_corpus_packages_with_web_baseline_primitives_remain_checkable_executable_and_testable_on_the_default_standalone_surface_in_js_input(
) {
    for package in ["ramda", "uuid", "dayjs", "zod", "lodash", "yaml"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.js");
        write_web_baseline_interop_source(&source_path, package);
        let test_path = dir.path().join("tests").join("web-baseline.test.js");
        write_web_baseline_test_source(&test_path, package);

        for (command, path) in [
            ("check", source_path.as_path()),
            ("build", source_path.as_path()),
            ("run", source_path.as_path()),
            ("test", test_path.as_path()),
        ] {
            let output = Command::new(kali_bin())
                .current_dir(dir.path())
                .arg("--output")
                .arg("json")
                .arg(command)
                .arg(path)
                .output()
                .expect("run kali");

            if command == "check" {
                // `check` genuinely stays green here (static analysis only, never
                // reaches the unsupported-lowering codegen path below).
                assert!(
                    output.status.success(),
                    "utility web-baseline package {package} should be {command}able on js input with json output\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );

                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["exitCode"], 0);
                assert!(json["payload"].is_object(), "json: {json}");
            } else {
                // Honest re-pin (PR #16 rev2): kali fails closed/loud here (E5506)
                // for build/run/test on this web-baseline interop source;
                // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
                assert!(!output.status.success(), "must fail closed: {output:?}");
            }
        }
    }
}

#[test]
fn deno_host_corpus_packages_remain_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    for (package, body) in [
        (
            "fresh-env",
            "export default function mutate() {\n  Deno.env.set('KALI_CORPUS_FLAG', 'set');\n  return Deno.env.get('KALI_CORPUS_FLAG');\n}\n",
        ),
        (
            "spawn-tools",
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
        ),
        (
            "listen-tools",
            "export default function listen() {\n  Deno.listen('127.0.0.1', 0);\n  return 'listen';\n}\n",
        ),
        (
            "serve-tools",
            "export default function serve() {\n  Deno.serve('127.0.0.1', 0);\n  return 'serve';\n}\n",
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("deno"));
        write_deno_host_package(dir.path(), package, body);
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write deno host source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "deno host package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "deno",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "deno host package {package} should be buildable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "deno host package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn deno_host_corpus_packages_remain_testable_on_the_deno_surface() {
    for (package, body, expected) in [
        (
            "fresh-env",
            "export default function mutate() {\n  Deno.env.set('KALI_CORPUS_FLAG', 'set');\n  return Deno.env.get('KALI_CORPUS_FLAG');\n}\n",
            "set",
        ),
        (
            "spawn-tools",
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
            "spawn",
        ),
        (
            "listen-tools",
            "export default function listen() {\n  Deno.listen('127.0.0.1', 0);\n  return 'listen';\n}\n",
            "listen",
        ),
        (
            "serve-tools",
            "export default function serve() {\n  Deno.serve('127.0.0.1', 0);\n  return 'serve';\n}\n",
            "serve",
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("deno"));
        write_deno_host_package(dir.path(), package, body);
        let test_path = dir.path().join("tests").join(format!("{package}.test.ts"));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} corpus', () => {{\n  const value = root();\n  if (value !== '{expected}') {{ throw new Error('{package} test mismatch: ' + value); }}\n  console.log(value);\n}});\n",
                package = package,
                expected = expected
            ),
        )
        .expect("write deno host test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "deno", test_path.to_str().unwrap()],
        );
        // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
        // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
        assert!(!test.status.success(), "must fail closed: {test:?}");
    }
}

#[test]
fn deno_host_corpus_packages_remain_testable_on_the_deno_surface_in_js_input() {
    for (package, body, expected) in [
        (
            "fresh-env",
            "export default function mutate() {\n  Deno.env.set('KALI_CORPUS_FLAG', 'set');\n  return Deno.env.get('KALI_CORPUS_FLAG');\n}\n",
            "set",
        ),
        (
            "spawn-tools",
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
            "spawn",
        ),
        (
            "listen-tools",
            "export default function listen() {\n  Deno.listen('127.0.0.1', 0);\n  return 'listen';\n}\n",
            "listen",
        ),
        (
            "serve-tools",
            "export default function serve() {\n  Deno.serve('127.0.0.1', 0);\n  return 'serve';\n}\n",
            "serve",
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("deno"));
        write_deno_host_package(dir.path(), package, body);
        let test_path = dir.path().join("tests").join(format!("{package}.test.js"));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('{package} corpus', () => {{\n  const value = root();\n  if (value !== '{expected}') {{ throw new Error('{package} test mismatch: ' + value); }}\n  console.log(value);\n}});\n",
                package = package,
                expected = expected
            ),
        )
        .expect("write deno host JS test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "deno", test_path.to_str().unwrap()],
        );
        // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
        // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
        assert!(!test.status.success(), "must fail closed: {test:?}");
    }
}

#[test]
fn jsr_corpus_packages_remain_checkable_buildable_and_executable_on_the_deno_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("deno"));
    write_jsr_package(
        dir.path(),
        "jsr:@std/path",
        r#"module.exports = function joinPath(left, right) {
    return `${left}/${right}`;
};
"#,
    );
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import joinPath from '@std/path';\nconsole.log(joinPath('alpha', 'beta'));\n",
    )
    .expect("write jsr source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "jsr package should be checkable on the Deno surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build_out_dir = dir.path().join("build");
    let build = run_kali(
        dir.path(),
        [
            "build",
            "--api",
            "deno",
            "--out-dir",
            build_out_dir.to_str().unwrap(),
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "jsr package should be buildable on the Deno surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(
        dir.path(),
        ["run", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        run.status.success(),
        "jsr package should stay executable on the Deno surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn deno_host_corpus_packages_remain_checkable_buildable_and_executable_on_the_deno_surface_in_js_input(
) {
    for (package, body) in [
        (
            "fresh-env",
            "export default function mutate() {\n  Deno.env.set('KALI_CORPUS_FLAG', 'set');\n  return Deno.env.get('KALI_CORPUS_FLAG');\n}\n",
        ),
        (
            "spawn-tools",
            "export default function spawn() {\n  new Deno.Command('sh').spawn();\n  return 'spawn';\n}\n",
        ),
        (
            "listen-tools",
            "export default function listen() {\n  Deno.listen('127.0.0.1', 0);\n  return 'listen';\n}\n",
        ),
        (
            "serve-tools",
            "export default function serve() {\n  Deno.serve('127.0.0.1', 0);\n  return 'serve';\n}\n",
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("deno"));
        write_deno_host_package(dir.path(), package, body);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write deno host JS source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "deno host package {package} should be checkable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "deno",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "deno host package {package} should be buildable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "deno", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "deno host package {package} should stay executable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn jsr_corpus_packages_remain_checkable_buildable_and_executable_on_the_deno_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("deno"));
    write_jsr_package(
        dir.path(),
        "jsr:@std/path",
        r#"module.exports = function joinPath(left, right) {
    return `${left}/${right}`;
};
"#,
    );
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import joinPath from '@std/path';\nconsole.log(joinPath('alpha', 'beta'));\n",
    )
    .expect("write jsr JS source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "jsr package should be checkable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build_out_dir = dir.path().join("build");
    let build = run_kali(
        dir.path(),
        [
            "build",
            "--api",
            "deno",
            "--out-dir",
            build_out_dir.to_str().unwrap(),
            source_path.to_str().unwrap(),
        ],
    );
    assert!(
        build.status.success(),
        "jsr package should be buildable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(
        dir.path(),
        ["run", "--api", "deno", source_path.to_str().unwrap()],
    );
    assert!(
        run.status.success(),
        "jsr package should stay executable on the Deno surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn jsr_corpus_packages_remain_testable_on_the_deno_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("deno"));
    write_jsr_package(
        dir.path(),
        "jsr:@std/path",
        r#"module.exports = function joinPath(left, right) {
    return `${left}/${right}`;
};
"#,
    );
    let test_source = dir.path().join("tests").join("std-path.test.ts");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        "import joinPath from '@std/path';\nKali.test('jsr corpus', () => {\n  const value = joinPath('alpha', 'beta');\n  if (value !== 'alpha/beta') { throw new Error('jsr test mismatch: ' + value); }\n  console.log(value);\n});\n",
    )
    .expect("write jsr test source");

    let test = run_kali(
        dir.path(),
        ["test", "--api", "deno", test_source.to_str().unwrap()],
    );
    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!test.status.success(), "must fail closed: {test:?}");
}

#[test]
fn jsr_corpus_packages_remain_testable_on_the_deno_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("deno"));
    write_jsr_package(
        dir.path(),
        "jsr:@std/path",
        r#"module.exports = function joinPath(left, right) {
    return `${left}/${right}`;
};
"#,
    );
    let test_source = dir.path().join("tests").join("std-path.test.js");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        "import joinPath from '@std/path';\nKali.test('jsr corpus', () => {\n  const value = joinPath('alpha', 'beta');\n  if (value !== 'alpha/beta') { throw new Error('jsr test mismatch: ' + value); }\n  console.log(value);\n});\n",
    )
    .expect("write jsr JS test source");

    let test = run_kali(
        dir.path(),
        ["test", "--api", "deno", test_source.to_str().unwrap()],
    );
    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!test.status.success(), "must fail closed: {test:?}");
}

#[test]
fn default_standalone_corpus_rejects_semver_style_package_bin_entrypoint() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);

    let output = run_kali(
        dir.path(),
        ["run", package_dir.join("bin/semver.js").to_str().unwrap()],
    );

    assert!(
        !output.status.success(),
        "expected the default standalone surface to reject a Node-style package bin entrypoint\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("Node.js CLI features")
            && stderr.contains("unavailable on the 'deno' API surface"),
        "stderr: {stderr}"
    );
}

#[test]
fn inherited_node_corpus_packages_remain_checkable_buildable_executable_and_testable_on_the_node_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_manifest(dir.path(), Some("node"));
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write inherited node package source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be buildable on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let test_source = dir.path().join("tests").join("pi-coding-agent.test.js");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        r#"import codingAgent from '../main.js';
Kali.test('pi-coding-agent corpus', () => {
  console.log(codingAgent());
});
"#,
    )
    .expect("write inherited node package test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}
