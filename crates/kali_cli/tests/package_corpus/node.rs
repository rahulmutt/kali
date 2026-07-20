use super::*;

#[test]
fn node_runner_corpus_semver_style_package_bin_executes_on_the_node_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), Some("node"));
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);

    let run = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("bin/semver.js").to_str().unwrap(),
        ],
    );
    assert!(
        run.status.success(),
        "semver corpus package bin should execute on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn node_assuming_corpus_packages_are_rejected_on_the_default_standalone_surface() {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_node_assuming_package(
        dir.path(),
        "chalk",
        r#"import { createHash } from "node:crypto";
export default function chalk() {
    createHash("sha256").update("chalk").digest("hex");
    return "chalk";
}
"#,
    );
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import chalk from 'chalk';
console.log(chalk());
"#,
    )
    .expect("write node package source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        !check.status.success(),
        "node-assuming package should be rejected on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    assert!(check_stderr.contains("E6005"), "stderr: {check_stderr}");
    assert!(
        check_stderr.contains("Node-only host API"),
        "stderr: {check_stderr}"
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        !run.status.success(),
        "node-assuming package should stay rejected at runtime on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let run_stderr = String::from_utf8_lossy(&run.stderr);
    assert!(run_stderr.contains("E6005"), "stderr: {run_stderr}");
    assert!(
        run_stderr.contains("Node-only host API"),
        "stderr: {run_stderr}"
    );

    let test_source = dir.path().join("tests").join("main.test.js");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        r#"import chalk from 'chalk';
Kali.test('node-assuming corpus', () => {
  console.log(chalk());
});
"#,
    )
    .expect("write node package test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        !test.status.success(),
        "node-assuming package should stay rejected at test time on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stderr = String::from_utf8_lossy(&test.stderr);
    assert!(test_stderr.contains("E6005"), "stderr: {test_stderr}");
    assert!(
        test_stderr.contains("Node-only host API"),
        "stderr: {test_stderr}"
    );
}

#[test]
fn node_runner_corpus_packages_remain_gated_on_the_node_surface() {
    for package in ["vitest", "jest", "mocha", "ava"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_stub_package(
            dir.path(),
            package,
            r#"import assert from "node:assert";
export default assert;
"#,
        );
        write_types_stub_package(dir.path(), package);
        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import '{}';\nKali.test('{} corpus', () => {{\n  console.log('node corpus: {}');\n}});\n",
                package, package, package
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let stdout = String::from_utf8_lossy(&test.stdout);
        assert!(
            stdout.contains(&format!("node corpus: {}", package)),
            "stdout: {stdout}"
        );
    }
}

/// Deferred class-B (PR #16 rev2): kali silently miscompiles this construct.
/// Not re-pinned — asserting kali's wrong value would bless a falsehood. Ignored until the fix lands.
#[ignore = "corpus silent miscompile; tracked https://github.com/rahulmutt/kali/issues/18; see pr16-honest-repin-inventory.md"]
#[test]
fn node_runner_corpus_packages_with_exports_maps_remain_gated_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:root'; }}\n",
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
        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} with exports map should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

/// Deferred class-B (PR #16 rev2): kali silently miscompiles this construct.
/// Not re-pinned — asserting kali's wrong value would bless a falsehood. Ignored until the fix lands.
#[ignore = "corpus silent miscompile; tracked https://github.com/rahulmutt/kali/issues/18; see pr16-honest-repin-inventory.md"]
#[test]
fn node_runner_corpus_packages_with_mixed_format_entries_remain_gated_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function root() {{ assert.ok(true); return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function subpath() {{ assert.ok(true); return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}:esm'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);
        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node mixed-format package {package} should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn node_assuming_corpus_packages_remain_gated_on_the_node_surface() {
    for (package, body) in [
        (
            "axios",
            r#"import path from "node:path";
export default function axios() {
    return path.basename("/tmp/axios.js");
}
"#,
        ),
        (
            "express",
            r#"import assert from "node:assert";
export default function express() {
    assert.ok(true);
    return "express";
}
"#,
        ),
        (
            "chalk",
            r#"import { createHash } from "node:crypto";
export default function chalk() {
    createHash("sha256").update("chalk").digest("hex");
    return "chalk";
}
"#,
        ),
        (
            "dotenv",
            r#"import fs from "node:fs";
export default function dotenv() {
    return fs.existsSync("/tmp/dotenv");
}
"#,
        ),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_node_assuming_package(dir.path(), package, body);
        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import {package} from '{package}';\nconsole.log({package}());\n",
                package = package
            ),
        )
        .expect("write node package source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "node package {package} should check on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "node",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "node package {package} should build on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "node package {package} should execute on the Node context\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );

        let test_source = dir.path().join("tests").join(format!("{package}.test.js"));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import {package} from '{package}';\nKali.test('{package} corpus', () => {{\n  console.log({package}());\n}});\n",
                package = package
            ),
        )
        .expect("write node package test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_source.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} should be testable on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert!(
            String::from_utf8_lossy(&test.stdout).contains("ok 1"),
            "stdout: {}",
            String::from_utf8_lossy(&test.stdout)
        );
    }
}

#[test]
fn node_builtin_corpus_packages_remain_checkable_buildable_executable_and_testable_on_the_node_surface_on_js_input(
) {
    let node_process_corpus_body = node_process_corpus_body();
    for (package, body, expected) in [
        (
            "node-buffer-corpus",
            "import { Buffer } from \"node:buffer\";\nexport default function root() { Buffer.from(\"node\"); return 0; }\n",
            "0",
        ),
        (
            "node-path-corpus",
            "import path from \"node:path\";\nexport default function root() { path.basename(\"/tmp/node-corpus.txt\"); return 0; }\n",
            "0",
        ),
        (
            "node-os-corpus",
            "import os from \"node:os\";\nexport default function root() { return typeof os.platform === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-crypto-corpus",
            "import { createHash } from \"node:crypto\";\nexport default function root() { createHash(\"sha256\").update(\"node-corpus\").digest(\"hex\"); return 0; }\n",
            "0",
        ),
        (
            "node-fs-corpus",
            "import fs from \"node:fs\";\nexport default function root() { return typeof fs.existsSync === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-fs-promises-corpus",
            "import { readFile } from \"node:fs/promises\";\nexport default function root() { return typeof readFile === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-url-corpus",
            "import { fileURLToPath } from \"node:url\";\nexport default function root() { fileURLToPath(new URL(\"file:///tmp/node-corpus.txt\")); return 0; }\n",
            "0",
        ),
        (
            "node-util-corpus",
            "import util from \"node:util\";\nexport default function root() { util.format(\"%s-%s\", \"node\", \"util\"); return 0; }\n",
            "0",
        ),
        (
            "node-http-corpus",
            "import http from \"node:http\";\nexport default function root() { return typeof http.get === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-process-corpus",
            node_process_corpus_body.as_str(),
            "0",
        ),
        (
            "node-timers-corpus",
            "import timers from \"node:timers\";\nexport default function root() { return typeof timers.setTimeout === \"function\" && typeof timers.clearTimeout === \"function\" && typeof timers.setInterval === \"function\" && typeof timers.clearInterval === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-timers-promises-corpus",
            "import { setTimeout as delay } from \"node:timers/promises\";\nexport default function root() { return typeof delay === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-events-corpus",
            "import { EventEmitter } from \"node:events\";\nexport default function root() { const emitter = new EventEmitter(); return typeof emitter.on === \"function\" && typeof emitter.emit === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-stream-corpus",
            "import { Readable } from \"node:stream\";\nexport default function root() { return typeof Readable === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-assert-corpus",
            "import assert from \"node:assert\";\nexport default function root() { assert.ok(true); return 0; }\n",
            "0",
        ),
        (
            "node-child-process-corpus",
            "import { spawn } from \"node:child_process\";\nexport default function root() { return typeof spawn === \"function\" ? 0 : 1; }\n",
            "0",
        ),
    ] {
        if package == "node-process-corpus" {
            assert!(
                body.contains("process.chdir"),
                "node process corpus should confirm process.chdir"
            );
            assert!(
                body.contains("process.exit"),
                "node process corpus should confirm process.exit"
            );
            assert!(
                body.contains(r#"typeof process["chdir"] === "function""#),
                r#"node process corpus should confirm process[\"chdir\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis.process["chdir"] === "function""#),
                r#"node process corpus should confirm globalThis.process[\"chdir\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis["process"]["chdir"] === "function""#),
                r#"node process corpus should confirm globalThis[\"process\"][\"chdir\"]"#
            );
            assert!(
                body.contains(r#"typeof process["exit"] === "function""#),
                r#"node process corpus should confirm process[\"exit\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis.process["exit"] === "function""#),
                r#"node process corpus should confirm globalThis.process[\"exit\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis["process"]["exit"] === "function""#),
                r#"node process corpus should confirm globalThis[\"process\"][\"exit\"]"#
            );
            assert_node_process_corpus_has_zero_probe_inventory(body);
        }
        if package == "node-timers-corpus" {
            assert!(
                body.contains("clearInterval"),
                "node timers corpus should confirm clearInterval"
            );
        }

        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_module_only_package(dir.path(), package, body);
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write node built-in source");

        let check = run_kali(
            dir.path(),
            ["check", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            check.status.success(),
            "node built-in package {package} should check on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--api",
                "node",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "node built-in package {package} should build on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(
            dir.path(),
            ["run", "--api", "node", source_path.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "node built-in package {package} should execute on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), format!("{expected}\n"));

        let test_source = dir.path().join("tests").join(format!("{package}.test.js"));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import root from '{package}';\nKali.test('{package} corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write node built-in test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_source.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node built-in package {package} should be testable on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains(expected), "stdout: {test_stdout}");
    }
}

#[test]
fn node_builtin_corpus_packages_remain_checkable_buildable_executable_and_testable_on_the_inherited_node_surface_on_js_input(
) {
    let node_process_corpus_body = node_process_corpus_body();
    for (package, body, expected) in [
        (
            "node-buffer-corpus",
            "import { Buffer } from \"node:buffer\";\nexport default function root() { Buffer.from(\"node\"); return 0; }\n",
            "0",
        ),
        (
            "node-assert-corpus",
            "import assert from \"node:assert\";\nexport default function root() { assert.ok(true); return 0; }\n",
            "0",
        ),
        (
            "node-http-corpus",
            "import http from \"node:http\";\nexport default function root() { return typeof http.get === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-process-corpus",
            node_process_corpus_body.as_str(),
            "0",
        ),
        (
            "node-timers-corpus",
            "import timers from \"node:timers\";\nexport default function root() { return typeof timers.setTimeout === \"function\" && typeof timers.clearTimeout === \"function\" && typeof timers.setInterval === \"function\" && typeof timers.clearInterval === \"function\" ? 0 : 1; }\n",
            "0",
        ),
        (
            "node-events-corpus",
            "import { EventEmitter } from \"node:events\";\nexport default function root() { const emitter = new EventEmitter(); return typeof emitter.on === \"function\" && typeof emitter.emit === \"function\" ? 0 : 1; }\n",
            "0",
        ),
    ] {
        if package == "node-process-corpus" {
            assert!(
                body.contains("process.chdir"),
                "node process corpus should confirm process.chdir"
            );
            assert!(
                body.contains("process.exit"),
                "node process corpus should confirm process.exit"
            );
            assert!(
                body.contains(r#"typeof process["chdir"] === "function""#),
                r#"node process corpus should confirm process[\"chdir\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis.process["chdir"] === "function""#),
                r#"node process corpus should confirm globalThis.process[\"chdir\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis["process"]["chdir"] === "function""#),
                r#"node process corpus should confirm globalThis[\"process\"][\"chdir\"]"#
            );
            assert!(
                body.contains(r#"typeof process["exit"] === "function""#),
                r#"node process corpus should confirm process[\"exit\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis.process["exit"] === "function""#),
                r#"node process corpus should confirm globalThis.process[\"exit\"]"#
            );
            assert!(
                body.contains(r#"typeof globalThis["process"]["exit"] === "function""#),
                r#"node process corpus should confirm globalThis[\"process\"][\"exit\"]"#
            );
            assert_node_process_corpus_has_zero_probe_inventory(body);
        }

        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_module_only_package(dir.path(), package, body);
        write_types_stub_package(dir.path(), package);

        let source_path = dir.path().join("main.js");
        fs::write(
            &source_path,
            format!(
                "import root from '{package}';\nconsole.log(root());\n",
                package = package
            ),
        )
        .expect("write inherited node built-in source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "node built-in package {package} should check on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            ["build", "--out-dir", build_out_dir.to_str().unwrap(), source_path.to_str().unwrap()],
        );
        assert!(
            build.status.success(),
            "node built-in package {package} should build on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "node built-in package {package} should execute on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), format!("{expected}\n"));

        let run_json = run_kali(
            dir.path(),
            ["--output", "json", "run", source_path.to_str().unwrap()],
        );
        assert!(
            run_json.status.success(),
            "node built-in package {package} should execute on the inherited Node surface in JS input with JSON output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run_json.stdout),
            String::from_utf8_lossy(&run_json.stderr)
        );
        let run_envelope = parse_json_stdout(&run_json);
        assert_eq!(run_envelope["command"], "run");
        assert_eq!(run_envelope["success"], true);
        assert_eq!(run_envelope["exitCode"], 0);
        assert_eq!(run_envelope["payload"]["exitCode"], 0);
        assert_eq!(run_envelope["payload"]["hostContract"], "kali-hosted");
        assert_eq!(run_envelope["payload"]["runtimeBackend"], "wasmtime");
        assert!(
            run_envelope["stdout"]
                .as_str()
                .expect("run stdout")
                .contains(expected),
            "json run: {run_envelope}"
        );

        let test_source = dir.path().join("tests").join(format!("{package}.test.js"));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import root from '{package}';\nKali.test('{package} corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write inherited node built-in test source");

        let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
        assert!(
            test.status.success(),
            "node built-in package {package} should be testable on the inherited Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
        assert!(test_stdout.contains(expected), "stdout: {test_stdout}");

        let test_json = run_kali(
            dir.path(),
            ["--output", "json", "test", test_source.to_str().unwrap()],
        );
        assert!(
            test_json.status.success(),
            "node built-in package {package} should be testable on the inherited Node surface in JS input with JSON output\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test_json.stdout),
            String::from_utf8_lossy(&test_json.stderr)
        );
        let test_envelope = parse_json_stdout(&test_json);
        assert_eq!(test_envelope["command"], "test");
        assert_eq!(test_envelope["success"], true);
        assert_eq!(test_envelope["exitCode"], 0);
        assert_eq!(test_envelope["payload"]["passed"], 1);
        assert_eq!(test_envelope["payload"]["total"], 1);
        assert_eq!(test_envelope["payload"]["failed"], 0);
        assert_eq!(test_envelope["payload"]["skipped"], 0);
        assert_eq!(test_envelope["payload"]["hostContract"], "kali-hosted");
        assert_eq!(test_envelope["payload"]["runtimeBackend"], "wasmtime");
        assert!(
            test_envelope["stdout"]
                .as_str()
                .expect("test stdout")
                .contains(expected),
            "json test: {test_envelope}"
        );
    }
}

/// Deferred class-B (PR #16 rev2): kali silently miscompiles this construct.
/// Not re-pinned — asserting kali's wrong value would bless a falsehood. Ignored until the fix lands.
#[ignore = "corpus silent miscompile; tracked https://github.com/rahulmutt/kali/issues/18; see pr16-honest-repin-inventory.md"]
#[test]
fn node_runner_corpus_packages_with_inherited_api_surface_remain_executable_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let run_source = dir.path().join("main.ts");
        fs::write(
            &run_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write inherited node run source");

        let run = run_kali(dir.path(), ["run", run_source.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "node package {package} with exports map should execute on the inherited Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(run_stdout.trim(), "0", "stdout: {run_stdout}");

        let test_source = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write inherited node test source");

        let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
        assert!(
            test.status.success(),
            "node package {package} with exports map should be testable on the inherited Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

/// Deferred class-B (PR #16 rev2): kali silently miscompiles this construct.
/// Not re-pinned — asserting kali's wrong value would bless a falsehood. Ignored until the fix lands.
#[ignore = "corpus silent miscompile; tracked https://github.com/rahulmutt/kali/issues/18; see pr16-honest-repin-inventory.md"]
#[test]
fn node_runner_corpus_packages_with_exports_maps_remain_executable_on_the_node_surface_in_js_input()
{
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_export_map_package(
            dir.path(),
            package,
            &format!(
                "import assert from \"node:assert\";\nimport {{ Buffer }} from \"node:buffer\";\nexport default function root() {{ assert.ok(true); Buffer.from('node'); return '{package}:root'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "import assert from \"node:assert\";\nimport {{ Buffer }} from \"node:buffer\";\nexport default function subpath() {{ assert.ok(true); Buffer.from('node'); return '{package}:{subpath}'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let run_source = dir.path().join("main.js");
        fs::write(
            &run_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node run source");

        let run = run_kali(
            dir.path(),
            ["run", "--api", "node", run_source.to_str().unwrap()],
        );
        assert!(
            run.status.success(),
            "node package {package} with exports map should execute on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert_eq!(run_stdout.trim(), "0", "stdout: {run_stdout}");

        let test_source = dir
            .path()
            .join("tests")
            .join(format!("{}.test.js", package));
        fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_source,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_source.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node package {package} with exports map should be testable on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        let test_stdout = String::from_utf8_lossy(&test.stdout);
        assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    }
}

/// Deferred class-B (PR #16 rev2): kali silently miscompiles this construct.
/// Not re-pinned — asserting kali's wrong value would bless a falsehood. Ignored until the fix lands.
#[ignore = "corpus silent miscompile; tracked https://github.com/rahulmutt/kali/issues/18; see pr16-honest-repin-inventory.md"]
#[test]
fn node_runner_corpus_packages_with_mixed_format_entries_remain_executable_on_the_node_surface() {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function root() {{ assert.ok(true); return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function subpath() {{ assert.ok(true); return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}:esm'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.ts", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node mixed-format package {package} should execute on the Node surface\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

/// Deferred class-B (PR #16 rev2): kali silently miscompiles this construct.
/// Not re-pinned — asserting kali's wrong value would bless a falsehood. Ignored until the fix lands.
#[ignore = "corpus silent miscompile; tracked https://github.com/rahulmutt/kali/issues/18; see pr16-honest-repin-inventory.md"]
#[test]
fn node_runner_corpus_packages_with_mixed_format_entries_remain_executable_on_the_node_surface_in_js_input(
) {
    for (package, subpath) in [
        ("vitest", "config"),
        ("jest", "globals"),
        ("mocha", "reporter"),
        ("ava", "config"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), Some("node"));
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function root() {{ assert.ok(true); return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function root() {{ assert.ok(true); return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "const assert = require(\"node:assert\");\nmodule.exports = function subpath() {{ assert.ok(true); return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "import assert from \"node:assert\";\nexport default function subpath() {{ assert.ok(true); return '{package}:{subpath}:esm'; }}\n",
                package = package,
                subpath = subpath
            ),
        );
        write_types_stub_package(dir.path(), package);

        let test_path = dir
            .path()
            .join("tests")
            .join(format!("{}.test.js", package));
        fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nKali.test('{package} corpus', () => {{\n  console.log(root(), subpath());\n}});\n",
                package = package,
                subpath = subpath
            ),
        )
        .expect("write node test source");

        let test = run_kali(
            dir.path(),
            ["test", "--api", "node", test_path.to_str().unwrap()],
        );
        assert!(
            test.status.success(),
            "node mixed-format package {package} should execute on the Node surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn node_corpus_executes_semver_style_package_bin_entrypoint() {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);

    let output = run_kali(
        dir.path(),
        [
            "run",
            "--api",
            "node",
            package_dir.join("bin/semver.js").to_str().unwrap(),
            "--",
            "1.2.3",
        ],
    );

    assert!(
        output.status.success(),
        "expected the Node surface to execute a published package bin entrypoint\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
