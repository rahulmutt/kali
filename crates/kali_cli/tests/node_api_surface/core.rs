use super::*;

#[test]
fn node_api_surface_accepts_threaded_runtime_globals_with_wasm_threads_in_js_input() {
    for inherited in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.js");
        let test_path = dir.path().join("main.test.js");
        fs::write(
            &source_path,
            "SharedArrayBuffer; Atomics; console.log('threaded globals ok');
",
        )
        .expect("write source");
        fs::write(
            &test_path,
            "Kali.test('threaded globals', () => { SharedArrayBuffer; Atomics; console.log('threaded globals ok'); });
",
        )
        .expect("write test source");

        if inherited {
            fs::write(
                dir.path().join("kali.json"),
                r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node",
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
            )
            .expect("write manifest");
        }

        for command in ["check", "build", "run", "test"] {
            let input_path = if command == "test" {
                &test_path
            } else {
                &source_path
            };

            let mut cli_command = Command::new(kali_bin());
            cli_command.current_dir(dir.path()).arg(command);
            if !inherited {
                cli_command.args(["--api", "node", "--wasm-threads"]);
            }
            cli_command.arg(input_path);

            let output = cli_command.output().expect("run kali");
            assert!(
                output.status.success(),
                "{command} should accept threaded globals on the Node surface (inherited={inherited})\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );

            let stdout = String::from_utf8_lossy(&output.stdout);
            match command {
                "check" => assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}"),
                "build" => {
                    assert!(
                        stdout.contains("Built executable artifact at"),
                        "stdout: {stdout}"
                    );
                    assert!(
                        source_path.with_file_name("main.wasm").exists(),
                        "expected build artifact"
                    );
                }
                "run" | "test" => {
                    assert!(stdout.contains("threaded globals ok"), "stdout: {stdout}")
                }
                _ => unreachable!("unexpected command"),
            }
        }

        for command in ["run", "test"] {
            let input_path = if command == "test" {
                &test_path
            } else {
                &source_path
            };

            let mut cli_command = Command::new(kali_bin());
            cli_command
                .current_dir(dir.path())
                .arg("--output")
                .arg("json")
                .arg(command);
            if !inherited {
                cli_command.args(["--api", "node", "--wasm-threads"]);
            }
            cli_command.arg(input_path);

            let output = cli_command.output().expect("run kali");
            assert!(
                output.status.success(),
                "{command} should accept threaded globals on the Node surface in JSON output (inherited={inherited})\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );

            let json = parse_json_stdout(&output);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], true);
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
            assert_eq!(json["payload"]["hostContract"], "kali-hosted");
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .contains("threaded globals ok"),
                "json: {json}"
            );
            match command {
                "run" => assert_eq!(json["payload"]["exitCode"], 0),
                "test" => {
                    assert_eq!(json["payload"]["failed"], 0);
                    assert_eq!(json["payload"]["passed"], 1);
                    assert_eq!(json["payload"]["total"], 1);
                }
                _ => unreachable!("unexpected command"),
            }
        }
    }
}

#[test]
fn node_api_surface_rejects_bundle_build_commands_in_js_input() {
    for inherited in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, "console.log(1);\n").expect("write source");

        if inherited {
            fs::write(
                dir.path().join("kali.json"),
                r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
            )
            .expect("write manifest");
        }

        let mut text_command = Command::new(kali_bin());
        text_command
            .current_dir(dir.path())
            .arg("build")
            .arg("--bundle");
        if !inherited {
            text_command.arg("--api").arg("node");
        }
        text_command.arg(&source_path);

        let text_output = text_command.output().expect("run kali");
        assert!(
            !text_output.status.success(),
            "build --bundle should be rejected on the Node surface (inherited={inherited})\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&text_output.stdout),
            String::from_utf8_lossy(&text_output.stderr)
        );
        assert_eq!(text_output.status.code(), Some(5));
        let text_stderr = String::from_utf8_lossy(&text_output.stderr);
        assert!(text_stderr.contains("E5508"), "stderr: {text_stderr}");
        assert!(
            text_stderr.contains("browser API surface"),
            "stderr: {text_stderr}"
        );

        let mut json_command = Command::new(kali_bin());
        json_command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg("build")
            .arg("--bundle");
        if !inherited {
            json_command.arg("--api").arg("node");
        }
        json_command.arg(&source_path);

        let json_output = json_command.output().expect("run kali");
        assert!(
            !json_output.status.success(),
            "json build --bundle should be rejected on the Node surface (inherited={inherited})\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_output.stdout),
            String::from_utf8_lossy(&json_output.stderr)
        );
        assert_eq!(json_output.status.code(), Some(5));
        let json = parse_json_stdout(&json_output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 5);
        assert_eq!(json["errors"][0]["code"], "E5508");
        assert!(
            json["errors"][0]["message"]
                .as_str()
                .expect("error message")
                .contains("browser API surface"),
            "json: {json}"
        );
    }
}

#[test]
fn node_api_surface_rejects_node_net_module_in_js_input_on_check_build_run_and_test_commands() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import net from 'node:net';\nconsole.log(typeof net);\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import net from 'node:net';\nKali.test('node net', () => console.log(typeof net));\n",
    )
    .expect("write test file");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let expected_message =
        "node builtin 'node:net' is not available on the explicit Node API surface";

    for command in ["check", "build", "run", "test"] {
        let input_path = if command == "test" {
            &test_file
        } else {
            &run_file
        };

        let mut text_command = Command::new(kali_bin());
        text_command.current_dir(dir.path()).arg(command);
        text_command.arg("--api").arg("node");
        text_command.arg(input_path);

        let text_output = text_command.output().expect("run kali");
        assert!(
            !text_output.status.success(),
            "{command} should be rejected on the Node surface for node:net\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&text_output.stdout),
            String::from_utf8_lossy(&text_output.stderr)
        );
        let text_stderr = String::from_utf8_lossy(&text_output.stderr);
        assert!(
            text_stderr.contains(expected_message),
            "{command} stderr for node:net: {text_stderr}"
        );

        let mut json_command = Command::new(kali_bin());
        json_command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg("--api")
            .arg("node")
            .arg(input_path);

        let json_output = json_command.output().expect("run kali");
        assert!(
            !json_output.status.success(),
            "json {command} should surface the Node net rejection as machine-readable output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_output.stdout),
            String::from_utf8_lossy(&json_output.stderr)
        );
        let json = parse_json_stdout(&json_output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        assert_json_node_builtin_rejection(
            json["errors"].as_array().expect("errors array"),
            expected_message,
        );
    }
}

#[test]
fn node_api_surface_rejects_node_net_module_on_inherited_node_api_surface_in_js_input_on_check_build_run_and_test_commands(
) {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import net from 'node:net';\nconsole.log(typeof net);\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import net from 'node:net';\nKali.test('node net', () => console.log(typeof net));\n",
    )
    .expect("write test file");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let expected_message =
        "node builtin 'node:net' is not available on the explicit Node API surface";

    for command in ["check", "build"] {
        let mut text_command = Command::new(kali_bin());
        text_command.current_dir(dir.path()).arg(command);
        text_command.arg(&run_file);

        let text_output = text_command.output().expect("run kali");
        assert!(
            !text_output.status.success(),
            "{command} should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&text_output.stdout),
            String::from_utf8_lossy(&text_output.stderr)
        );
        let text_stderr = String::from_utf8_lossy(&text_output.stderr);
        assert!(
            text_stderr.contains(expected_message),
            "{command} stderr for node:net: {text_stderr}"
        );

        let mut json_command = Command::new(kali_bin());
        json_command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg(&run_file);

        let json_output = json_command.output().expect("run kali");
        assert!(
            !json_output.status.success(),
            "json {command} should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_output.stdout),
            String::from_utf8_lossy(&json_output.stderr)
        );
        let json = parse_json_stdout(&json_output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        assert_json_node_builtin_rejection(
            json["errors"].as_array().expect("errors array"),
            expected_message,
        );
    }

    let mut run = Command::new(kali_bin());
    run.current_dir(dir.path()).arg("run").arg(&run_file);
    let run_output = run.output().expect("run kali");
    assert!(
        !run_output.status.success(),
        "run should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stderr = String::from_utf8_lossy(&run_output.stderr);
    assert!(
        run_stderr.contains(expected_message),
        "run stderr: {run_stderr}"
    );

    let run_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&run_file)
        .output()
        .expect("run kali");
    assert!(
        !run_json_output.status.success(),
        "json run should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_json_output.stdout),
        String::from_utf8_lossy(&run_json_output.stderr)
    );
    let run_json = parse_json_stdout(&run_json_output);
    assert_eq!(run_json["command"], "run");
    assert_eq!(run_json["success"], false);
    assert_eq!(run_json["exitCode"], 1);
    assert_json_node_builtin_rejection(
        run_json["errors"].as_array().expect("errors array"),
        expected_message,
    );

    let mut test = Command::new(kali_bin());
    test.current_dir(dir.path()).arg("test").arg(&test_file);
    let test_output = test.output().expect("run kali");
    assert!(
        !test_output.status.success(),
        "test should be rejected on the inherited Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_output.stdout),
        String::from_utf8_lossy(&test_output.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test_output.stderr);
    assert!(
        test_stderr.contains(expected_message),
        "test stderr: {test_stderr}"
    );

    let test_json_output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&test_file)
        .output()
        .expect("run kali");
    assert!(
        !test_json_output.status.success(),
        "json test should surface the inherited Node builtin rejection as machine-readable output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_json_output.stdout),
        String::from_utf8_lossy(&test_json_output.stderr)
    );
    let test_json = parse_json_stdout(&test_json_output);
    assert_eq!(test_json["command"], "test");
    assert_eq!(test_json["success"], false);
    assert_eq!(test_json["exitCode"], 1);
    assert_json_node_builtin_rejection(
        test_json["errors"].as_array().expect("errors array"),
        expected_message,
    );
}

#[test]
fn node_api_surface_rejects_node_dns_module_in_js_input_on_check_build_run_and_test_commands() {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import dns from 'node:dns';\nconsole.log(typeof dns.lookup);\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import dns from 'node:dns';\nKali.test('node dns', () => console.log(typeof dns));\n",
    )
    .expect("write test file");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let expected_message =
        "node builtin 'node:dns' is not available on the explicit Node API surface";

    for command in ["check", "build", "run", "test"] {
        let input_path = if command == "test" {
            &test_file
        } else {
            &run_file
        };

        let mut text_command = Command::new(kali_bin());
        text_command.current_dir(dir.path()).arg(command);
        text_command.arg("--api").arg("node");
        text_command.arg(input_path);

        let text_output = text_command.output().expect("run kali");
        assert!(
            !text_output.status.success(),
            "{command} should be rejected on the Node surface for node:dns\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&text_output.stdout),
            String::from_utf8_lossy(&text_output.stderr)
        );
        let text_stderr = String::from_utf8_lossy(&text_output.stderr);
        assert!(
            text_stderr.contains(expected_message),
            "{command} stderr for node:dns: {text_stderr}"
        );

        let mut json_command = Command::new(kali_bin());
        json_command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg("--api")
            .arg("node")
            .arg(input_path);

        let json_output = json_command.output().expect("run kali");
        assert!(
            !json_output.status.success(),
            "json {command} should surface the Node dns rejection as machine-readable output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_output.stdout),
            String::from_utf8_lossy(&json_output.stderr)
        );
        let json = parse_json_stdout(&json_output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        assert_json_node_builtin_rejection(
            json["errors"].as_array().expect("errors array"),
            expected_message,
        );
    }
}

#[test]
fn node_api_surface_rejects_node_worker_threads_module_in_js_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let run_file = dir.path().join(format!("main.{extension}"));
        let test_file = dir.path().join(format!("main.test.{extension}"));
        fs::write(
            &run_file,
            r#"import { Worker } from 'node:worker_threads';
console.log(typeof Worker);
"#,
        )
        .expect("write run file");
        fs::write(
            &test_file,
            r#"import { Worker } from 'node:worker_threads';
Kali.test('node worker_threads', () => console.log(typeof Worker));
"#,
        )
        .expect("write test file");

        fs::write(
            dir.path().join("kali.json"),
            r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
        )
        .expect("write manifest");

        let expected_message =
            "node builtin 'node:worker_threads' is not available on the explicit Node API surface";

        for command in ["check", "build", "run", "test"] {
            let input_path = if command == "test" {
                &test_file
            } else {
                &run_file
            };

            let mut text_command = Command::new(kali_bin());
            text_command.current_dir(dir.path()).arg(command);
            text_command.arg("--api").arg("node");
            text_command.arg(input_path);

            let text_output = text_command.output().expect("run kali");
            assert!(
                !text_output.status.success(),
                "{command} should be rejected on the Node surface for node:worker_threads\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&text_output.stdout),
                String::from_utf8_lossy(&text_output.stderr)
            );
            let text_stderr = String::from_utf8_lossy(&text_output.stderr);
            assert!(
                text_stderr.contains(expected_message),
                "{command} stderr for node:worker_threads: {text_stderr}"
            );

            let mut json_command = Command::new(kali_bin());
            json_command
                .current_dir(dir.path())
                .arg("--output")
                .arg("json")
                .arg(command)
                .arg("--api")
                .arg("node")
                .arg(input_path);

            let json_output = json_command.output().expect("run kali");
            assert!(
                !json_output.status.success(),
                "json {command} should surface the Node worker_threads rejection as machine-readable output\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&json_output.stdout),
                String::from_utf8_lossy(&json_output.stderr)
            );
            let json = parse_json_stdout(&json_output);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            assert_eq!(json["exitCode"], 1);
            assert_json_node_builtin_rejection(
                json["errors"].as_array().expect("errors array"),
                expected_message,
            );
        }
    }
}

#[test]
fn node_api_surface_supports_process_env_property_mutation_in_js_input_on_check_build_run_and_test_commands(
) {
    let source_variants = [
        "process.env.KALI_NODE_ENV_MUTATION = 'set'; delete process.env.KALI_NODE_ENV_MUTATION; console.log('node env mutation');",
        "process[\"env\"].KALI_NODE_ENV_MUTATION = 'set'; delete process[\"env\"].KALI_NODE_ENV_MUTATION; console.log('node env mutation');",
        "process[\"env\"][\"KALI_NODE_ENV_MUTATION\"] = 'set'; delete process[\"env\"][\"KALI_NODE_ENV_MUTATION\"]; console.log('node env mutation');",
        "globalThis.process.env.KALI_NODE_ENV_MUTATION = 'set'; delete globalThis.process.env.KALI_NODE_ENV_MUTATION; console.log('node env mutation');",
        "globalThis.process[\"env\"].KALI_NODE_ENV_MUTATION = 'set'; delete globalThis.process[\"env\"].KALI_NODE_ENV_MUTATION; console.log('node env mutation');",
        "globalThis.process[\"env\"][\"KALI_NODE_ENV_MUTATION\"] = 'set'; delete globalThis.process[\"env\"][\"KALI_NODE_ENV_MUTATION\"]; console.log('node env mutation');",
        "globalThis[\"process\"].env.KALI_NODE_ENV_MUTATION = 'set'; delete globalThis[\"process\"].env.KALI_NODE_ENV_MUTATION; console.log('node env mutation');",
        "globalThis[\"process\"].env[\"KALI_NODE_ENV_MUTATION\"] = 'set'; delete globalThis[\"process\"].env[\"KALI_NODE_ENV_MUTATION\"]; console.log('node env mutation');",
        "globalThis[\"process\"][\"env\"].KALI_NODE_ENV_MUTATION = 'set'; delete globalThis[\"process\"][\"env\"].KALI_NODE_ENV_MUTATION; console.log('node env mutation');",
        "globalThis[\"process\"][\"env\"][\"KALI_NODE_ENV_MUTATION\"] = 'set'; delete globalThis[\"process\"][\"env\"][\"KALI_NODE_ENV_MUTATION\"]; console.log('node env mutation');",
    ];

    for source in source_variants {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join("main.js");
            let test_file = dir.path().join("main.test.js");
            fs::write(&run_file, format!("{source}\n")).expect("write run file");
            fs::write(
                &test_file,
                format!("Kali.test('node env mutation', () => {{ {source} }});\n"),
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} should be supported on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&text_output.stdout),
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stderr = String::from_utf8_lossy(&text_output.stderr);
                assert!(
                    !text_stderr.contains("E5506") && !text_stderr.contains("process.env"),
                    "{command} stderr for {source} (inherited={inherited}): {text_stderr}"
                );

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["exitCode"], 0);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
        }
    }
}

#[test]
fn node_api_surface_rejects_deno_env_mutation_in_js_input_on_check_build_run_and_test_commands() {
    let cases = [
        (
            r#"Deno.env.set('KALI_NODE_DENO_ENV_MUTATION', 'set'); Deno.env.delete('KALI_NODE_DENO_ENV_MUTATION'); console.log('deno env mutation');"#,
            "Deno.env.set",
        ),
        (
            r#"Deno["env"]["set"]('KALI_NODE_DENO_ENV_MUTATION', 'set'); Deno["env"]["delete"]('KALI_NODE_DENO_ENV_MUTATION'); console.log('deno env mutation');"#,
            r#"Deno["env"]["set"]"#,
        ),
    ];

    for (source, expected_fragment) in cases {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join("main.js");
            let test_path = dir.path().join("main.test.js");
            fs::write(&source_path, format!("{source}\n")).expect("write source");
            fs::write(
                &test_path,
                format!("Kali.test('deno env mutation rejection', () => {{ {source} }});\n"),
            )
            .expect("write test source");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_path
                } else {
                    &source_path
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    !text_output.status.success(),
                    "{command} should be rejected on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&text_output.stdout),
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stderr = String::from_utf8_lossy(&text_output.stderr);
                assert!(
                    text_stderr.contains("environment mutation API"),
                    "{command} stderr missing environment mutation gate for {source} (inherited={inherited}): {text_stderr}"
                );
                assert!(
                    text_stderr.contains(expected_fragment),
                    "{command} stderr missing {expected_fragment} for {source} (inherited={inherited}): {text_stderr}"
                );

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    !json_output.status.success(),
                    "json {command} should surface the Node rejection as machine-readable output for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], false);
                assert_eq!(json["exitCode"], 1);
                assert_eq!(json["errors"][0]["code"], "E5506");
                let message = json["errors"][0]["message"]
                    .as_str()
                    .expect("json error message string");
                assert!(
                    message.contains("environment mutation API"),
                    "json {command} message missing environment mutation gate for {source} (inherited={inherited}): {message}"
                );
                assert!(
                    message.contains(expected_fragment),
                    "json {command} message missing {expected_fragment} for {source} (inherited={inherited}): {message}"
                );
            }
        }
    }
}

#[test]
fn node_api_surface_rejects_deno_env_to_object_in_js_input_on_check_build_run_and_test_commands() {
    let cases = [
        (
            r#"Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno["env"]["toObject"](); globalThis["Deno"]["env"]["toObject"](); globalThis.Deno["env"]["toObject"](); globalThis["Deno"].env.toObject();"#,
            "Deno.env.toObject",
        ),
        (
            r#"globalThis.Deno.env.toObject(); Deno["env"].toObject(); globalThis.Deno["env"].toObject(); globalThis["Deno"].env["toObject"](); globalThis["Deno"]["env"].toObject();"#,
            r#"Deno["env"]["toObject"]"#,
        ),
    ];

    for (source, expected_fragment) in cases {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join("main.js");
            let test_path = dir.path().join("main.test.js");
            fs::write(&source_path, format!("{source}\n")).expect("write source");
            fs::write(
                &test_path,
                format!("Kali.test('deno env toObject rejection', () => {{ {source} }});\n"),
            )
            .expect("write test source");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_path
                } else {
                    &source_path
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    !text_output.status.success(),
                    "{command} should be rejected on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&text_output.stdout),
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stderr = String::from_utf8_lossy(&text_output.stderr);
                assert!(
                    text_stderr.contains("environment snapshot materialization API"),
                    "{command} stderr missing environment snapshot materialization gate for {source} (inherited={inherited}): {text_stderr}"
                );
                assert!(
                    text_stderr.contains("object-aggregate lowering"),
                    "{command} stderr missing object-aggregate lowering gate for {source} (inherited={inherited}): {text_stderr}"
                );
                assert!(
                    text_stderr.contains(expected_fragment),
                    "{command} stderr missing {expected_fragment} for {source} (inherited={inherited}): {text_stderr}"
                );

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    !json_output.status.success(),
                    "json {command} should surface the Node rejection as machine-readable output for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], false);
                assert_eq!(json["exitCode"], 1);
                assert_eq!(json["errors"][0]["code"], "E5506");
                let message = json["errors"][0]["message"]
                    .as_str()
                    .expect("json error message string");
                assert!(
                    message.contains("environment snapshot materialization API"),
                    "json {command} message missing environment snapshot materialization gate for {source} (inherited={inherited}): {message}"
                );
                assert!(
                    message.contains("object-aggregate lowering"),
                    "json {command} message missing object-aggregate lowering gate for {source} (inherited={inherited}): {message}"
                );
                assert!(
                    message.contains(expected_fragment),
                    "json {command} message missing {expected_fragment} for {source} (inherited={inherited}): {message}"
                );
            }
        }
    }
}

#[test]
fn node_api_surface_supports_promise_all_settled_in_js_input_on_check_build_run_and_test_commands()
{
    let source_variants = [
        r#"console.log(Promise.allSettled([1, 2]));"#,
        r#"console.log(globalThis.Promise.allSettled([1, 2]));"#,
    ];

    for source in source_variants {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join("main.js");
            let test_file = dir.path().join("main.test.js");
            fs::write(&run_file, format!("{source}\n")).expect("write run file");
            fs::write(
                &test_file,
                format!("Kali.test('node promise allSettled', () => {{ {source} }});\n"),
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} should be supported on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&text_output.stdout),
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stderr = String::from_utf8_lossy(&text_output.stderr);
                assert!(
                    !text_stderr.contains("E5506") && !text_stderr.contains("Promise.allSettled"),
                    "{command} stderr for {source} (inherited={inherited}): {text_stderr}"
                );

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["exitCode"], 0);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
        }
    }
}

#[test]
fn node_api_surface_rejects_node_dns_module_on_inherited_node_api_surface_in_js_input_on_check_build_run_and_test_commands(
) {
    let dir = tempdir().expect("tempdir");
    let run_file = dir.path().join("main.js");
    let test_file = dir.path().join("main.test.js");
    fs::write(
        &run_file,
        "import dns from 'node:dns';\nconsole.log(typeof dns.lookup);\n",
    )
    .expect("write run file");
    fs::write(
        &test_file,
        "import dns from 'node:dns';\nKali.test('node dns', () => console.log(typeof dns));\n",
    )
    .expect("write test file");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
    )
    .expect("write manifest");

    let expected_message =
        "node builtin 'node:dns' is not available on the explicit Node API surface";

    for command in ["check", "build", "run", "test"] {
        let input_path = if command == "test" {
            &test_file
        } else {
            &run_file
        };

        let mut text_command = Command::new(kali_bin());
        text_command.current_dir(dir.path()).arg(command);
        text_command.arg(input_path);

        let text_output = text_command.output().expect("run kali");
        assert!(
            !text_output.status.success(),
            "{command} should be rejected on the inherited Node surface for node:dns\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&text_output.stdout),
            String::from_utf8_lossy(&text_output.stderr)
        );
        let text_stderr = String::from_utf8_lossy(&text_output.stderr);
        assert!(
            text_stderr.contains(expected_message),
            "{command} stderr for node:dns: {text_stderr}"
        );

        let mut json_command = Command::new(kali_bin());
        json_command
            .current_dir(dir.path())
            .arg("--output")
            .arg("json")
            .arg(command)
            .arg(input_path);

        let json_output = json_command.output().expect("run kali");
        assert!(
            !json_output.status.success(),
            "json {command} should surface the inherited Node dns rejection as machine-readable output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&json_output.stdout),
            String::from_utf8_lossy(&json_output.stderr)
        );
        let json = parse_json_stdout(&json_output);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        assert_json_node_builtin_rejection(
            json["errors"].as_array().expect("errors array"),
            expected_message,
        );
    }
}

#[test]
fn node_api_surface_supports_process_exit_in_js_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "jsx", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            fs::write(&run_file, "process.exit(7);\n").expect("write run file");
            fs::write(
                &test_file,
                "Kali.test('process exit', () => process.exit(7));\n",
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for process.exit (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stdout = String::from_utf8_lossy(&text_output.stdout);
                assert!(
                    text_stdout.contains(if command == "check" {
                        "Checked 1 file(s)"
                    } else {
                        "Built executable artifact at"
                    }),
                    "{command} stdout for process.exit (extension={extension}, inherited={inherited}): {text_stdout}"
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                let expected_code = if command == "run" { Some(7) } else { Some(0) };
                assert_eq!(
                    text_output.status.code(),
                    expected_code,
                    "{command} stderr for process.exit (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
            }
        }
    }
}

#[test]
fn node_api_surface_supports_bracketed_process_control_in_js_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "jsx", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            fs::write(
                &run_file,
                "globalThis.process.cwd(); globalThis[\"process\"][\"cwd\"](); globalThis.process[\"cwd\"](); process[\"cwd\"](); globalThis[\"process\"][\"cwd\"](); globalThis.process.chdir('.'); globalThis[\"process\"][\"chdir\"]('.'); globalThis.process[\"chdir\"]('.'); process[\"chdir\"]('.'); globalThis[\"process\"][\"chdir\"]('.'); globalThis.process.exit(7); globalThis[\"process\"][\"exit\"](7); globalThis.process[\"exit\"](7); process[\"exit\"](7); globalThis[\"process\"][\"exit\"](7);\n",
            )
            .expect("write run file");
            fs::write(
                &test_file,
                "Kali.test('process control', () => { globalThis.process.cwd(); globalThis[\"process\"][\"cwd\"](); globalThis.process[\"cwd\"](); process[\"cwd\"](); globalThis[\"process\"][\"cwd\"](); globalThis.process.chdir('.'); globalThis[\"process\"][\"chdir\"]('.'); globalThis.process[\"chdir\"]('.'); process[\"chdir\"]('.'); globalThis[\"process\"][\"chdir\"]('.'); globalThis.process.exit(7); globalThis[\"process\"][\"exit\"](7); globalThis.process[\"exit\"](7); process[\"exit\"](7); globalThis[\"process\"][\"exit\"](7); });\n"
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for bracketed process control (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stdout = String::from_utf8_lossy(&text_output.stdout);
                assert!(
                    text_stdout.contains(if command == "check" {
                        "Checked 1 file(s)"
                    } else {
                        "Built executable artifact at"
                    }),
                    "{command} stdout for bracketed process control (extension={extension}, inherited={inherited}): {text_stdout}"
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                let expected_code = if command == "run" { Some(7) } else { Some(0) };
                assert_eq!(
                    text_output.status.code(),
                    expected_code,
                    "{command} stderr for bracketed process control (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
            }
        }
    }
}

#[test]
fn node_api_surface_supports_process_kill_zero_probe_in_js_ts_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            fs::write(&run_file, process_kill_zero_probe_run_source()).expect("write run file");
            fs::write(&test_file, process_kill_zero_probe_test_source()).expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for process.kill(0) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stdout = String::from_utf8_lossy(&text_output.stdout);
                assert!(
                    text_stdout.contains(if command == "check" {
                        "Checked 1 file(s)"
                    } else {
                        "Built executable artifact at"
                    }),
                    "{command} stdout for process.kill(0) (extension={extension}, inherited={inherited}): {text_stdout}"
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert_eq!(
                    text_output.status.code(),
                    Some(0),
                    "{command} stderr for process.kill(0) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                if command == "run" {
                    let stdout = String::from_utf8_lossy(&text_output.stdout);
                    // UPDATED by console-render-unification Task 5b: every probe in this
                    // fixture's run source is a directly-named `process.kill(...)` call, a
                    // shape the shared string-coercion ladder can now prove Boolean, so the
                    // probe renders `true` rather than the raw `1` i64 this check used to see.
                    assert!(stdout.contains("true"), "{command} stdout: {stdout}");
                }
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for process.kill(0) (extension={extension}, inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["exitCode"], 0);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
        }
    }
}

#[test]
fn node_api_surface_supports_optional_chain_wrapped_process_kill_zero_probe_in_js_ts_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            fs::write(
                &run_file,
                process_kill_optional_chain_zero_probe_run_source(),
            )
            .expect("write run file");
            fs::write(
                &test_file,
                process_kill_optional_chain_zero_probe_test_source(),
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for optional-chain process.kill(0) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stdout = String::from_utf8_lossy(&text_output.stdout);
                assert!(
                    text_stdout.contains(if command == "check" {
                        "Checked 1 file(s)"
                    } else {
                        "Built executable artifact at"
                    }),
                    "{command} stdout for optional-chain process.kill(0) (extension={extension}, inherited={inherited}): {text_stdout}"
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert_eq!(
                    text_output.status.code(),
                    Some(0),
                    "{command} stderr for optional-chain process.kill(0) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                if command == "run" {
                    let stdout = String::from_utf8_lossy(&text_output.stdout);
                    assert!(stdout.contains("1"), "{command} stdout: {stdout}");
                }
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for optional-chain process.kill(0) (extension={extension}, inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["exitCode"], 0);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
        }
    }
}

#[test]
fn node_api_surface_supports_process_kill_zero_probe_through_static_zero_aliases_in_js_ts_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            let run_source = process_kill_zero_probe_node_api_surface_run_source();
            fs::write(&run_file, run_source).expect("write run file");
            let test_source = process_kill_zero_probe_node_api_surface_test_source();
            fs::write(&test_file, test_source).expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for process.kill alias (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stdout = String::from_utf8_lossy(&text_output.stdout);
                assert!(
                    text_stdout.contains(if command == "check" {
                        "Checked 1 file(s)"
                    } else {
                        "Built executable artifact at"
                    }),
                    "{command} stdout for process.kill alias (extension={extension}, inherited={inherited}): {text_stdout}"
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert_eq!(
                    text_output.status.code(),
                    Some(0),
                    "{command} stderr for process.kill alias (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                if command == "run" {
                    let stdout = String::from_utf8_lossy(&text_output.stdout);
                    // UPDATED by console-render-unification Task 5b: the directly-named call
                    // targets in this fixture now render `true` (a shape the shared ladder can
                    // prove); the six sequence-expression-selected targets (`sequenceKill` and
                    // its aliases) are NOT a directly-provable call target and still render the
                    // raw `0`/`1` i64 -- unaffected by this fix, same as a plain binding read.
                    assert!(stdout.contains("true"), "{command} stdout: {stdout}");
                }
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for process.kill alias (extension={extension}, inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["exitCode"], 0);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
        }
    }
}

#[test]
fn node_api_surface_supports_process_kill_zero_probe_object_freeze_wrappers_in_js_ts_jsx_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            fs::write(&run_file, process_kill_zero_probe_console_log_source())
                .expect("write run file");
            fs::write(
                &test_file,
                format!(
                    "Kali.test('process kill freeze', () => {{ if ({}) {{ throw new Error('expected zero probe'); }} }});",
                    process_kill_zero_probe_guard_source()
                ),
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for process.kill freeze alias (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stdout = String::from_utf8_lossy(&text_output.stdout);
                assert!(
                    text_stdout.contains(if command == "check" {
                        "Checked 1 file(s)"
                    } else {
                        "Built executable artifact at"
                    }),
                    "{command} stdout for process.kill freeze alias (extension={extension}, inherited={inherited}): {text_stdout}"
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert_eq!(
                    text_output.status.code(),
                    Some(0),
                    "{command} stderr for process.kill freeze alias (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
                if command == "run" {
                    let stdout = String::from_utf8_lossy(&text_output.stdout);
                    // UPDATED by console-render-unification Task 5b: every probe in this
                    // fixture's run source is a directly-named `process.kill(...)` call, a
                    // shape the shared string-coercion ladder can now prove Boolean, so the
                    // probe renders `true` rather than the raw `1` i64 this check used to see.
                    assert!(stdout.contains("true"), "{command} stdout: {stdout}");
                }
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for process.kill freeze alias (extension={extension}, inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert_eq!(json["exitCode"], 0);
                let errors = json["errors"].as_array().expect("errors array");
                assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            }
        }
    }
}

#[test]
fn node_api_surface_supports_process_kill_zero_probe_satisfies_wrappers_in_ts_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["ts", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            fs::write(&run_file, process_kill_zero_probe_satisfies_source())
                .expect("write run file");
            fs::write(
                &test_file,
                "Kali.test('process kill satisfies', () => { if (!process.kill((0 satisfies number)) || !globalThis.process.kill((0 satisfies number)) || !globalThis.process[\"kill\"]((0 satisfies number)) || !globalThis[\"process\"].kill((0 satisfies number)) || !globalThis[\"process\"][\"kill\"]((0 satisfies number)) || !process[\"kill\"]((0 satisfies number))) { throw new Error('expected zero probe'); } });\n",
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for process.kill((0 satisfies number)) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert_eq!(
                    text_output.status.code(),
                    Some(0),
                    "{command} stderr for process.kill((0 satisfies number)) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for process.kill((0 satisfies number)) (extension={extension}, inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
            }
        }
    }
}

#[test]
fn node_api_surface_supports_process_kill_zero_probe_type_assertion_wrappers_in_ts_and_tsx_input_on_check_build_run_and_test_commands(
) {
    for extension in ["ts", "tsx"] {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let run_file = dir.path().join(format!("main.{extension}"));
            let test_file = dir.path().join(format!("main.test.{extension}"));
            fs::write(&run_file, process_kill_zero_probe_type_assertion_source())
                .expect("write run file");
            fs::write(
                &test_file,
                "Kali.test('process kill type assertion', () => { if (!process.kill((0 as number)) || !globalThis.process.kill((0 as number)) || !globalThis.process[\"kill\"]((0 as number)) || !globalThis[\"process\"].kill((0 as number)) || !globalThis[\"process\"][\"kill\"]((0 as number)) || !process[\"kill\"]((0 as number))) { throw new Error('expected zero probe'); } });\n",
            )
            .expect("write test file");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build"] {
                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(&run_file);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    text_output.status.success(),
                    "{command} stderr for process.kill((0 as number)) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
            }

            for command in ["run", "test"] {
                let input_path = if command == "run" {
                    &run_file
                } else {
                    &test_file
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert_eq!(
                    text_output.status.code(),
                    Some(0),
                    "{command} stderr for process.kill((0 as number)) (extension={extension}, inherited={inherited}): {}",
                    String::from_utf8_lossy(&text_output.stderr)
                );
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_file
                } else {
                    &run_file
                };

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    json_output.status.success(),
                    "json {command} should be supported on the Node surface for process.kill((0 as number)) (extension={extension}, inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
            }
        }
    }
}

#[test]
fn node_api_surface_rejects_late_object_model_members_in_js_input_on_check_build_run_and_test_commands(
) {
    let cases = [
        (r#"Proxy;"#, "Proxy"),
        (r#"globalThis.Proxy;"#, "globalThis.Proxy"),
        (r#"globalThis["Proxy"];"#, "globalThis.Proxy"),
        (r#"Proxy.revocable({}, {});"#, "Proxy.revocable"),
        (
            r#"globalThis.Proxy.revocable({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (
            r#"globalThis["Proxy"]["revocable"]({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (
            r#"globalThis["Proxy"].revocable({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (
            r#"globalThis.Proxy["revocable"]({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (
            r#"Object.freeze(Proxy.revocable)({}, {});"#,
            "Proxy.revocable",
        ),
        (
            r#"Object.freeze(globalThis.Proxy.revocable)({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (
            r#"Object.freeze(globalThis["Proxy"]["revocable"])({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (
            r#"Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (
            r#"Object.freeze(globalThis.Proxy["revocable"])({}, {});"#,
            "globalThis.Proxy.revocable",
        ),
        (r#"WeakMap;"#, "WeakMap"),
        (r#"globalThis.WeakMap;"#, "globalThis.WeakMap"),
        (r#"globalThis["WeakMap"];"#, "globalThis.WeakMap"),
        (r#"WeakSet;"#, "WeakSet"),
        (r#"globalThis.WeakSet;"#, "globalThis.WeakSet"),
        (r#"globalThis["WeakSet"];"#, "globalThis.WeakSet"),
        (r#"WeakRef;"#, "WeakRef"),
        (r#"globalThis.WeakRef;"#, "globalThis.WeakRef"),
        (r#"globalThis["WeakRef"];"#, "globalThis.WeakRef"),
        (r#"FinalizationRegistry(() => {});"#, "FinalizationRegistry"),
        (
            r#"globalThis.FinalizationRegistry;"#,
            "globalThis.FinalizationRegistry",
        ),
        (
            r#"globalThis["FinalizationRegistry"];"#,
            "globalThis.FinalizationRegistry",
        ),
        (r#"Object.hasOwn(globalThis, "a");"#, "Object.hasOwn"),
        (
            r#"Object.prototype.hasOwnProperty.call(globalThis, "a");"#,
            "Object.prototype.hasOwnProperty.call",
        ),
    ];

    let assert_rejection = |source: &str, expected_fragment: &str| {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join("main.js");
            let test_path = dir.path().join("main.test.js");
            fs::write(&source_path, format!("{source}\n")).expect("write source");
            fs::write(
                &test_path,
                format!("Kali.test('late object model', () => {{ {source} }});\n"),
            )
            .expect("write test source");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_path
                } else {
                    &source_path
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    !text_output.status.success(),
                    "{command} should be rejected on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&text_output.stdout),
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stderr = String::from_utf8_lossy(&text_output.stderr);
                assert!(
                    text_stderr.contains("late object-model API"),
                    "{command} stderr missing late object-model gate for {source} (inherited={inherited}): {text_stderr}"
                );
                assert!(
                    text_stderr.contains(expected_fragment),
                    "{command} stderr missing {expected_fragment} for {source} (inherited={inherited}): {text_stderr}"
                );

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    !json_output.status.success(),
                    "json {command} should surface the Node rejection as machine-readable output for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], false);
                assert_eq!(json["exitCode"], 1);
                assert_eq!(json["errors"][0]["code"], "E5506");
                let message = json["errors"][0]["message"]
                    .as_str()
                    .expect("json error message string");
                assert!(
                    message.contains("late object-model API"),
                    "json {command} message missing late object-model gate for {source} (inherited={inherited}): {message}"
                );
                assert!(
                    message.contains(expected_fragment),
                    "json {command} message missing {expected_fragment} for {source} (inherited={inherited}): {message}"
                );
            }
        }
    };

    for (source, expected_fragment) in cases {
        assert_rejection(source, expected_fragment);
    }
}

#[test]
fn node_api_surface_rejects_broader_intl_members_in_js_input_on_check_build_run_and_test_commands()
{
    let cases = [
        (r#"Intl;"#, "Intl"),
        (r#"Intl.NumberFormat;"#, "Intl.NumberFormat"),
        (
            r#"globalThis.Intl.DateTimeFormat;"#,
            "globalThis.Intl.DateTimeFormat",
        ),
        (
            r#"globalThis.Intl.RelativeTimeFormat;"#,
            "globalThis.Intl.RelativeTimeFormat",
        ),
        (
            r#"globalThis.Intl.PluralRules;"#,
            "globalThis.Intl.PluralRules",
        ),
        (r#"globalThis.Intl.Collator;"#, "globalThis.Intl.Collator"),
        (
            r#"globalThis["Intl"]["RelativeTimeFormat"];"#,
            "globalThis.Intl.RelativeTimeFormat",
        ),
        (
            r#"globalThis["Intl"]["PluralRules"];"#,
            "globalThis.Intl.PluralRules",
        ),
        (
            r#"globalThis["Intl"]["Collator"];"#,
            "globalThis.Intl.Collator",
        ),
        (
            r#"globalThis["Intl"]["DisplayNames"];"#,
            "globalThis.Intl.DisplayNames",
        ),
        (r#"globalThis.Intl.Segmenter;"#, "globalThis.Intl.Segmenter"),
        (
            r#"globalThis["Intl"]["Segmenter"];"#,
            "globalThis.Intl.Segmenter",
        ),
        (r#"Intl.Locale;"#, "Intl.Locale"),
    ];

    let assert_rejection = |source: &str, expected_fragment: &str| {
        for inherited in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join("main.js");
            let test_path = dir.path().join("main.test.js");
            fs::write(&source_path, format!("{source}\n")).expect("write source");
            fs::write(
                &test_path,
                format!("Kali.test('broad intl', () => {{ {source} }});\n"),
            )
            .expect("write test source");

            if inherited {
                fs::write(
                    dir.path().join("kali.json"),
                    r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "node"
  }
}"#,
                )
                .expect("write manifest");
            }

            for command in ["check", "build", "run", "test"] {
                let input_path = if command == "test" {
                    &test_path
                } else {
                    &source_path
                };

                let mut text_command = Command::new(kali_bin());
                text_command.current_dir(dir.path()).arg(command);
                if !inherited {
                    text_command.arg("--api").arg("node");
                }
                text_command.arg(input_path);

                let text_output = text_command.output().expect("run kali");
                assert!(
                    !text_output.status.success(),
                    "{command} should be rejected on the Node surface for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&text_output.stdout),
                    String::from_utf8_lossy(&text_output.stderr)
                );
                let text_stderr = String::from_utf8_lossy(&text_output.stderr);
                assert!(
                    text_stderr.contains("broader Intl support"),
                    "{command} stderr missing broader Intl gate for {source} (inherited={inherited}): {text_stderr}"
                );
                assert!(
                    text_stderr.contains(expected_fragment),
                    "{command} stderr missing {expected_fragment} for {source} (inherited={inherited}): {text_stderr}"
                );

                let mut json_command = Command::new(kali_bin());
                json_command
                    .current_dir(dir.path())
                    .arg("--output")
                    .arg("json")
                    .arg(command);
                if !inherited {
                    json_command.arg("--api").arg("node");
                }
                json_command.arg(input_path);

                let json_output = json_command.output().expect("run kali");
                assert!(
                    !json_output.status.success(),
                    "json {command} should surface the Node rejection as machine-readable output for {source} (inherited={inherited})\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&json_output.stdout),
                    String::from_utf8_lossy(&json_output.stderr)
                );
                let json = parse_json_stdout(&json_output);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], false);
                assert_eq!(json["exitCode"], 1);
                assert_eq!(json["errors"][0]["code"], "E5506");
                let message = json["errors"][0]["message"]
                    .as_str()
                    .expect("json error message string");
                assert!(
                    message.contains("broader Intl support"),
                    "json {command} message missing broader Intl gate for {source} (inherited={inherited}): {message}"
                );
                assert!(
                    message.contains(expected_fragment),
                    "json {command} message missing {expected_fragment} for {source} (inherited={inherited}): {message}"
                );
            }
        }
    };

    for (source, expected_fragment) in cases {
        assert_rejection(source, expected_fragment);
    }
}
