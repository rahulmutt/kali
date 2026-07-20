use super::*;

#[test]
fn utility_corpus_packages_remain_executable_on_the_default_standalone_surface() {
    for package in [
        "react",
        "preact",
        "ramda",
        "rambda",
        "rxjs",
        "immer",
        "uuid",
        "typescript",
        "esbuild",
        "date-fns",
        "dayjs",
        "luxon",
        "axios",
        "camelcase",
        "deepmerge",
        "cheerio",
        "graphql",
        "lodash",
        "lodash-es",
        "commander",
        "redux",
        "reselect",
        "recoil",
        "clsx",
        "classnames",
        "zustand",
        "mitt",
        "query-string",
        "formik",
        "jotai",
        "yup",
        "yaml",
        "xstate",
        "valtio",
        "react-hook-form",
        "msw",
        "superjson",
        "chart.js",
        "recharts",
        "d3",
        "@jridgewell/sourcemap-codec",
        "@emotion/react",
        "@emotion/styled",
        "@mantine/core",
        "lucide-react",
        "vite",
        "tailwindcss",
        "@tanstack/router",
        "@tanstack/query-core",
        "path-to-regexp",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
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
                "import '{}';\nconsole.log('utility corpus: {}');\n",
                package, package
            ),
        )
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_exports_maps_remain_executable_on_the_default_standalone_surface() {
    for (package, subpath) in [
        ("ramda", "add"),
        ("rxjs", "operators"),
        ("uuid", "v4"),
        ("commander", "command"),
        ("redux", "createStore"),
        ("reselect", "selectors"),
        ("xstate", "react"),
        ("lodash", "get"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
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
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility package {package} with exports map should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build_out_dir = dir.path().join("build");
        let build = run_kali(
            dir.path(),
            [
                "build",
                "--out-dir",
                build_out_dir.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ],
        );
        assert!(
            build.status.success(),
            "utility package {package} with exports map should be buildable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility package {package} with exports map should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_string_exports_remain_executable_on_the_default_standalone_surface()
{
    for package in ["ramda", "rxjs", "uuid", "commander", "redux", "lodash"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:exports'; }}\n",
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
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility string-exports package {package} should resolve its exports string\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility string-exports package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_string_exports_remain_testable_on_the_default_standalone_surface_in_js_input(
) {
    for package in ["ramda", "rxjs", "uuid", "commander", "redux", "lodash"] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_string_exports_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:exports'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        let test_path = dir.path().join("smoke.test.js");
        fs::write(
            &test_path,
            format!(
                "import root from '{package}';\nKali.test('string exports corpus', () => {{\n  console.log(root());\n}});\n",
                package = package
            ),
        )
        .expect("write utility test source");

        let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
        assert!(
            test.status.success(),
            "utility string-exports package {package} should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&test.stdout),
            String::from_utf8_lossy(&test.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&test.stdout), "0\nok 1\n");
    }
}

#[test]
fn utility_corpus_semver_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir.path().join("node_modules/semver");
    write_semver_style_package(&package_dir);
    write_types_stub_package(dir.path(), "semver");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
console.log(valid('1.2.3'));
console.log(satisfies('1.2.3', '^1.0.0'));
console.log(minVersion('^1.2.3')?.version);
"#,
    )
    .expect("write semver source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "semver corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "semver corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "semver corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1.2.3\n1\n1.2.3\n");
}

#[test]
fn utility_corpus_semver_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
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
    .expect("write semver source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
Kali.test('semver corpus', () => {
  console.log(valid('1.2.3'));
  console.log(satisfies('1.2.3', '^1.0.0'));
  console.log(minVersion('^1.2.3')?.version);
});
"#,
    )
    .expect("write semver test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "semver corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let check_json = run_kali(
        dir.path(),
        ["--output", "json", "check", source_path.to_str().unwrap()],
    );
    assert!(
        check_json.status.success(),
        "semver corpus package should be checkable on the default standalone surface in JS input with JSON output\nstdout: {}\nstderr: {}",
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

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "semver corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let build_json = run_kali(
        dir.path(),
        ["--output", "json", "build", source_path.to_str().unwrap()],
    );
    assert!(
        build_json.status.success(),
        "semver corpus package should be buildable on the default standalone surface in JS input with JSON output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build_json.stdout),
        String::from_utf8_lossy(&build_json.stderr)
    );
    let build_envelope = parse_json_stdout(&build_json);
    assert_eq!(build_envelope["schemaVersion"], 1);
    assert_eq!(build_envelope["command"], "build");
    assert_eq!(build_envelope["success"], true);
    assert_eq!(build_envelope["exitCode"], 0);
    let build_payload = build_envelope["payload"]
        .as_object()
        .expect("build payload object");
    assert_eq!(build_payload["artifactKind"], "executable");
    assert_eq!(build_payload["buildMode"], "fast");
    assert_eq!(
        PathBuf::from(
            build_payload["outputPath"]
                .as_str()
                .expect("build output path")
        ),
        source_path.with_extension("wasm")
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "semver corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1.2.3\n1\n1.2.3\n");
    let run_json = run_kali(
        dir.path(),
        ["--output", "json", "run", source_path.to_str().unwrap()],
    );
    assert!(
        run_json.status.success(),
        "semver corpus package should stay executable on the default standalone surface in JS input with JSON output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_json.stdout),
        String::from_utf8_lossy(&run_json.stderr)
    );
    let run_envelope = parse_json_stdout(&run_json);
    assert_eq!(run_envelope["command"], "run");
    assert_eq!(run_envelope["success"], true);
    assert_eq!(run_envelope["exitCode"], 0);
    assert_eq!(run_envelope["payload"]["hostContract"], "kali-hosted");
    assert_eq!(run_envelope["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        run_envelope["stdout"]
            .as_str()
            .expect("run stdout")
            .contains("1.2.3\n1\n1.2.3\n"),
        "json run: {run_envelope}"
    );

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "semver corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("1.2.3"), "stdout: {test_stdout}");
    let test_json = run_kali(
        dir.path(),
        ["--output", "json", "test", test_path.to_str().unwrap()],
    );
    assert!(
        test_json.status.success(),
        "semver corpus package should be testable on the default standalone surface in JS input with JSON output\nstdout: {}\nstderr: {}",
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
            .contains("1.2.3"),
        "json test: {test_envelope}"
    );
}

#[test]
fn utility_corpus_semver_style_package_remains_checkable_buildable_executable_and_testable_on_the_node_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
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
    .expect("write semver source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import { valid, satisfies, minVersion } from 'semver';
Kali.test('semver corpus', () => {
  console.log(valid('1.2.3'));
  console.log(satisfies('1.2.3', '^1.0.0'));
  console.log(minVersion('^1.2.3')?.version);
});
"#,
    )
    .expect("write semver test source");

    let check = run_kali(
        dir.path(),
        ["check", "--api", "node", source_path.to_str().unwrap()],
    );
    assert!(
        check.status.success(),
        "semver corpus package should be checkable on the Node surface\nstdout: {}\nstderr: {}",
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
        "semver corpus package should be buildable on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(
        dir.path(),
        ["run", "--api", "node", source_path.to_str().unwrap()],
    );
    assert!(
        run.status.success(),
        "semver corpus package should stay executable on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1.2.3\n1\n1.2.3\n");

    let test = run_kali(
        dir.path(),
        ["test", "--api", "node", test_path.to_str().unwrap()],
    );
    assert!(
        test.status.success(),
        "semver corpus package should be testable on the Node surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("1.2.3"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_date_fns_style_package_remains_checkable_buildable_testable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "date-fns",
        "export function addDays(date, amount) { return 0; }\nexport function format(date) { return 0; }\n",
        "formatISO",
        "export function formatISO(date) { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "date-fns");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
console.log(addDays('2024-01-01', 3));
console.log(format('2024-01-01'));
console.log(formatISO('2024-01-01'));
"#,
    )
    .expect("write date-fns source");
    let test_path = dir.path().join("tests").join("date-fns.test.js");
    fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
Kali.test('date-fns corpus', () => {
  console.log(addDays('2024-01-01', 3));
  console.log(format('2024-01-01'));
  console.log(formatISO('2024-01-01'));
});
"#,
    )
    .expect("write date-fns test source");
    let ts_test_path = dir.path().join("tests").join("date-fns.test.ts");
    fs::write(
        &ts_test_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
Kali.test('date-fns corpus', () => {
  console.log(addDays('2024-01-01', 3));
  console.log(format('2024-01-01'));
  console.log(formatISO('2024-01-01'));
});
"#,
    )
    .expect("write date-fns TS test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "date-fns corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "date-fns corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "date-fns corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n0\n0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");

    let test_json = run_kali(
        dir.path(),
        ["--output", "json", "test", test_path.to_str().unwrap()],
    );
    assert!(
        test_json.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_json.stdout),
        String::from_utf8_lossy(&test_json.stderr)
    );
    let json = parse_json_stdout(&test_json);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("0\n0\n0\n"),
        "json: {json}"
    );

    let ts_test = run_kali(dir.path(), ["test", ts_test_path.to_str().unwrap()]);
    assert!(
        ts_test.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in TS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&ts_test.stdout),
        String::from_utf8_lossy(&ts_test.stderr)
    );
    let ts_test_stdout = String::from_utf8_lossy(&ts_test.stdout);
    assert!(ts_test_stdout.contains("ok 1"), "stdout: {ts_test_stdout}");
    assert!(ts_test_stdout.contains("0"), "stdout: {ts_test_stdout}");

    let ts_test_json = run_kali(
        dir.path(),
        ["--output", "json", "test", ts_test_path.to_str().unwrap()],
    );
    assert!(
        ts_test_json.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in TS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&ts_test_json.stdout),
        String::from_utf8_lossy(&ts_test_json.stderr)
    );
    let ts_json = parse_json_stdout(&ts_test_json);
    assert_eq!(ts_json["command"], "test");
    assert_eq!(ts_json["success"], true);
    assert_eq!(ts_json["exitCode"], 0);
    assert_eq!(ts_json["payload"]["passed"], 1);
    assert_eq!(ts_json["payload"]["total"], 1);
    assert_eq!(ts_json["payload"]["failed"], 0);
    assert_eq!(ts_json["payload"]["skipped"], 0);
    assert_eq!(ts_json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(ts_json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        ts_json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("0\n0\n0\n"),
        "json: {ts_json}"
    );
}

#[test]
fn utility_corpus_date_fns_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "date-fns",
        "export function addDays(date, amount) { return 0; }\nexport function format(date) { return 0; }\n",
        "formatISO",
        "export function formatISO(date) { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "date-fns");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
console.log(addDays('2024-01-01', 3));
console.log(format('2024-01-01'));
console.log(formatISO('2024-01-01'));
"#,
    )
    .expect("write date-fns JS source");
    let test_path = dir.path().join("tests").join("date-fns.test.js");
    fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_path,
        r#"import { addDays, format } from 'date-fns';
import { formatISO } from 'date-fns/formatISO';
Kali.test('date-fns corpus', () => {
  console.log(addDays('2024-01-01', 3));
  console.log(format('2024-01-01'));
  console.log(formatISO('2024-01-01'));
});
"#,
    )
    .expect("write date-fns test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "date-fns corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "date-fns corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "date-fns corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n0\n0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");

    let test_json = run_kali(
        dir.path(),
        ["--output", "json", "test", test_path.to_str().unwrap()],
    );
    assert!(
        test_json.status.success(),
        "date-fns corpus package should be testable on the default standalone surface in JS input with json output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_json.stdout),
        String::from_utf8_lossy(&test_json.stderr)
    );
    let json = parse_json_stdout(&test_json);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["payload"]["hostContract"], "kali-hosted");
    assert_eq!(json["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        json["stdout"]
            .as_str()
            .expect("stdout")
            .contains("0\n0\n0\n"),
        "json: {json}"
    );
}

#[test]
fn utility_corpus_zod_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "zod",
        "export default function zod() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "zod");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import zod from 'zod';
console.log(zod());
"#,
    )
    .expect("write zod source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "zod corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "zod corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "zod corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_plimit_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "p-limit",
        "export default function pLimit() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "p-limit");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import pLimit from 'p-limit';
console.log(pLimit());
"#,
    )
    .expect("write p-limit source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "p-limit corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "p-limit corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "p-limit corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_ms_style_package_remains_checkable_buildable_and_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "ms",
        "export default function ms() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "ms");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import ms from 'ms';
console.log(ms());
"#,
    )
    .expect("write ms source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "ms corpus package should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "ms corpus package should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "ms corpus package should stay executable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_zod_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "zod",
        "export default function zod() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "zod");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import zod from 'zod';
console.log(zod());
"#,
    )
    .expect("write zod JS source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import zod from 'zod';
Kali.test('zod corpus', () => {
  console.log(zod());
});
"#,
    )
    .expect("write zod test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "zod corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "zod corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "zod corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "zod corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_plimit_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "p-limit",
        "export default function pLimit() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "p-limit");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import pLimit from 'p-limit';
console.log(pLimit());
"#,
    )
    .expect("write p-limit JS source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import pLimit from 'p-limit';
Kali.test('p-limit corpus', () => {
  console.log(pLimit());
});
"#,
    )
    .expect("write p-limit test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "p-limit corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "p-limit corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "p-limit corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "p-limit corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_ms_style_package_remains_checkable_buildable_executable_and_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_module_only_package(
        dir.path(),
        "ms",
        "export default function ms() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "ms");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import ms from 'ms';
console.log(ms());
"#,
    )
    .expect("write ms JS source");
    let test_path = dir.path().join("smoke.test.js");
    fs::write(
        &test_path,
        r#"import ms from 'ms';
Kali.test('ms corpus', () => {
  console.log(ms());
});
"#,
    )
    .expect("write ms test source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "ms corpus package should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "ms corpus package should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "ms corpus package should stay executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "ms corpus package should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_remains_checkable_and_buildable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
console.log(codingAgent());
"#,
    )
    .expect("write pi-coding-agent source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be buildable on the default standalone surface\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_remains_checkable_and_buildable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
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
    .expect("write pi-coding-agent JS source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "pi-coding-agent corpus package content should be checkable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "pi-coding-agent corpus package content should be buildable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_is_executable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
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
    .expect("write pi-coding-agent JS source");

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "pi-coding-agent corpus package content should be executable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");

    let run_json = run_kali(
        dir.path(),
        ["--output", "json", "run", source_path.to_str().unwrap()],
    );
    assert!(
        run_json.status.success(),
        "pi-coding-agent corpus package content should be executable on the default standalone surface in JS input with JSON output\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run_json.stdout),
        String::from_utf8_lossy(&run_json.stderr)
    );
    let run_envelope = parse_json_stdout(&run_json);
    assert_eq!(run_envelope["command"], "run");
    assert_eq!(run_envelope["success"], true);
    assert_eq!(run_envelope["exitCode"], 0);
    assert_eq!(run_envelope["payload"]["hostContract"], "kali-hosted");
    assert_eq!(run_envelope["payload"]["runtimeBackend"], "wasmtime");
    assert!(
        run_envelope["stdout"]
            .as_str()
            .expect("run stdout")
            .contains("0\n"),
        "json run: {run_envelope}"
    );
}

#[test]
fn utility_corpus_pi_coding_agent_style_package_is_testable_on_the_default_standalone_surface_on_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let package_dir = dir
        .path()
        .join("node_modules/@mariozechner/pi-coding-agent");
    write_pi_coding_agent_style_package(&package_dir);
    write_types_stub_package(dir.path(), "@mariozechner/pi-coding-agent");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"import codingAgent from '@mariozechner/pi-coding-agent';
export function describeAgent() {
  return codingAgent();
}
"#,
    )
    .expect("write pi-coding-agent JS source");
    let test_path = dir.path().join("tests").join("pi-coding-agent.test.js");
    fs::create_dir_all(test_path.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_path,
        r#"import { describeAgent } from '../main.js';
Kali.test('pi-coding-agent corpus', () => {
  console.log(describeAgent());
});
"#,
    )
    .expect("write pi-coding-agent JS test source");

    let test = run_kali(dir.path(), ["test", test_path.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "pi-coding-agent corpus package content should be testable on the default standalone surface in JS input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let test_stdout = String::from_utf8_lossy(&test.stdout);
    assert!(test_stdout.contains("ok 1"), "stdout: {test_stdout}");
    assert!(test_stdout.contains("0"), "stdout: {test_stdout}");

    let test_json = run_kali(
        dir.path(),
        ["--output", "json", "test", test_path.to_str().unwrap()],
    );
    assert!(
        test_json.status.success(),
        "pi-coding-agent corpus package content should be testable on the default standalone surface in JS input with JSON output\nstdout: {}\nstderr: {}",
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
            .contains("0"),
        "json test: {test_envelope}"
    );
}

#[test]
fn utility_corpus_packages_with_pattern_exports_remain_executable_on_the_default_standalone_surface(
) {
    for (package, subpath) in [
        ("ramda", "add"),
        ("rxjs", "operators"),
        ("uuid", "v4"),
        ("commander", "command"),
        ("redux", "createStore"),
        ("reselect", "selectors"),
        ("xstate", "react"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
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
        for source_name in ["main.ts", "main.js"] {
            let source_path = dir.path().join(source_name);
            fs::write(
                &source_path,
                format!(
                    "import root from '{package}';\nimport subpath from '{package}/{subpath}';\nconsole.log(root(), subpath());\n",
                    package = package,
                    subpath = subpath
                ),
            )
            .expect("write utility source");

            let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
            assert!(
                check.status.success(),
                "utility pattern-export package {package} should be checkable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&check.stdout),
                String::from_utf8_lossy(&check.stderr)
            );

            let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
            assert!(
                build.status.success(),
                "utility pattern-export package {package} should be buildable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            );

            let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
            assert!(
                run.status.success(),
                "utility pattern-export package {package} should stay executable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
}

#[test]
fn utility_corpus_packages_with_module_entries_remain_executable_on_the_default_standalone_surface()
{
    for package in [
        "ramda",
        "rambda",
        "rxjs",
        "uuid",
        "commander",
        "immer",
        "typescript",
        "esbuild",
        "luxon",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_module_only_package(
            dir.path(),
            package,
            &format!(
                "export default function widget() {{ return '{package}:module'; }}\n",
                package = package
            ),
        );
        write_types_stub_package(dir.path(), package);
        for source_name in ["main.ts", "main.js"] {
            let source_path = dir.path().join(source_name);
            fs::write(
                &source_path,
                format!(
                    "import root from '{package}';\nconsole.log(root());\n",
                    package = package
                ),
            )
            .expect("write utility source");

            let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
            assert!(
                check.status.success(),
                "utility module-only package {package} should be checkable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&check.stdout),
                String::from_utf8_lossy(&check.stderr)
            );

            let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
            assert!(
                build.status.success(),
                "utility module-only package {package} should be buildable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            );

            let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
            assert!(
                run.status.success(),
                "utility module-only package {package} should stay executable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
}

#[test]
fn utility_corpus_packages_with_module_entry_chains_remain_executable_on_the_default_standalone_surface(
) {
    for package in [
        "ramda",
        "uuid",
        "commander",
        "immer",
        "typescript",
        "esbuild",
        "dayjs",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
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
                "export default function helper() {{ return '{package}:internal'; }}\n",
                package = package
            ),
        )
        .expect("write utility internal module");
        for source_name in ["main.ts", "main.js"] {
            let source_path = dir.path().join(source_name);
            fs::write(
                &source_path,
                format!(
                    "import root from '{package}';\nconsole.log(root());\n",
                    package = package
                ),
            )
            .expect("write utility source");

            let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
            assert!(
                check.status.success(),
                "utility module-chain package {package} should resolve its internal module dependency on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&check.stdout),
                String::from_utf8_lossy(&check.stderr)
            );

            let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
            assert!(
                build.status.success(),
                "utility module-chain package {package} should build while resolving its internal module dependency on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&build.stdout),
                String::from_utf8_lossy(&build.stderr)
            );

            let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
            assert!(
                run.status.success(),
                "utility module-chain package {package} should stay executable on {source_name}\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }
}

#[test]
fn utility_corpus_packages_with_web_baseline_primitives_remain_executable_on_the_default_standalone_surface(
) {
    for package in [
        "ramda",
        "uuid",
        "rxjs",
        "dayjs",
        "luxon",
        "zod",
        "nanoid",
        "axios",
        "ajv",
        "redux",
        "mitt",
        "swr",
        "nanostores",
        "pinia",
        "superjson",
        "yup",
        "chart.js",
        "recharts",
        "@emotion/styled",
        "@storybook/react",
        "@tanstack/react-table",
        "lodash",
        "vite",
        "yaml",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
            dir.path(),
            package,
            "export default function describe(value) { return value; }\n",
        );
        write_types_stub_package(dir.path(), package);
        let source_path = dir.path().join("main.ts");
        write_web_baseline_interop_source(&source_path, package);

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility web-baseline package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        // Honest re-pin (PR #16 rev2): kali fails closed/loud here (E5506) on this
        // web-baseline interop source; `check` genuinely stays green above (static
        // analysis only), but `build` reaches the unsupported-lowering codegen path.
        // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
        assert!(!build.status.success(), "must fail closed: {build:?}");
    }
}

#[test]
fn utility_corpus_packages_with_web_baseline_primitives_remain_checkable_executable_and_testable_on_the_default_standalone_surface_in_js_input(
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

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility web-baseline package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        // Honest re-pin (PR #16 rev2): kali fails closed/loud here (E5506) on this
        // web-baseline interop source; `check` genuinely stays green above (static
        // analysis only), but `build` reaches the unsupported-lowering codegen path.
        // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
        assert!(!build.status.success(), "must fail closed: {build:?}");
    }
}

#[test]
fn utility_corpus_packages_with_mixed_format_entries_remain_executable_on_the_default_standalone_surface(
) {
    for (package, subpath) in [
        ("ramda", "add"),
        ("rxjs", "operators"),
        ("uuid", "v4"),
        ("commander", "command"),
        ("immer", "produce"),
        ("typescript", "tsc"),
        ("esbuild", "build"),
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_mixed_format_package(
            dir.path(),
            package,
            &format!(
                "module.exports = function root() {{ return '{package}:cjs'; }};\n",
                package = package
            ),
            &format!(
                "export default function root() {{ return '{package}:esm'; }}\n",
                package = package
            ),
            subpath,
            &format!(
                "module.exports = function subpath() {{ return '{package}:{subpath}:cjs'; }};\n",
                package = package,
                subpath = subpath
            ),
            &format!(
                "export default function subpath() {{ return '{package}:{subpath}:esm'; }}\n",
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
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "utility mixed-format package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "utility mixed-format package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import root from 'interop-demo';\nimport feature from 'interop-demo/feature';\nif (feature() !== 0) { throw new Error('interop-demo feature export mismatch'); }\nconsole.log(root());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility mixed-format package interop-demo should be checkable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility mixed-format package interop-demo should be buildable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility mixed-format package interop-demo should stay executable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import root from 'interop-demo';\nimport feature from 'interop-demo/feature';\nif (feature() !== 0) { throw new Error('interop-demo feature export mismatch'); }\nconsole.log(root());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility mixed-format package interop-demo should be checkable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility mixed-format package interop-demo should be buildable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility mixed-format package interop-demo should stay executable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_exports_map_and_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "interop-export-map-demo",
        "module.exports = function root() { return 0; }\n",
        "feature",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-export-map-demo");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "import root from 'interop-export-map-demo';\nimport feature from 'interop-export-map-demo/feature';\nif (root() !== 0 || feature() !== 0) { throw new Error('interop-export-map-demo export mismatch'); }\nconsole.log(root() + feature());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be checkable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be buildable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should stay executable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_exports_map_and_minimized_cjs_esm_interop_remain_executable_on_the_default_standalone_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_export_map_package(
        dir.path(),
        "interop-export-map-demo",
        "module.exports = function root() { return 0; }\n",
        "feature",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-export-map-demo");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "import root from 'interop-export-map-demo';\nimport feature from 'interop-export-map-demo/feature';\nif (root() !== 0 || feature() !== 0) { throw new Error('interop-export-map-demo export mismatch'); }\nconsole.log(root() + feature());\n",
    )
    .expect("write utility source");

    let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
    assert!(
        check.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be checkable on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
    assert!(
        build.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should be buildable on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "utility export-map mixed-format package interop-export-map-demo should stay executable on js input\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "0\n");
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_testable_on_the_default_standalone_surface(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo-test",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo-test");
    let test_source = dir.path().join("tests").join("interop-demo.test.ts");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        "import root from 'interop-demo-test';\nimport feature from 'interop-demo-test/feature';\nKali.test('interop-demo-test corpus', () => {\n  if (root() !== 0 || feature() !== 0) { throw new Error('interop-demo-test export mismatch'); }\n  console.log(root() + feature());\n});\n",
    )
    .expect("write utility test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "utility mixed-format package interop-demo-test should be testable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn utility_corpus_packages_with_minimized_cjs_esm_interop_remain_testable_on_the_default_standalone_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    write_manifest(dir.path(), None);
    write_mixed_format_package(
        dir.path(),
        "interop-demo-test",
        "module.exports = function root() { return 0; }\n",
        "export default function root() { return 0; }\n",
        "feature",
        "module.exports = function feature() { return 0; }\n",
        "export default function feature() { return 0; }\n",
    );
    write_types_stub_package(dir.path(), "interop-demo-test");
    let test_source = dir.path().join("tests").join("interop-demo.test.js");
    fs::create_dir_all(test_source.parent().expect("test dir")).expect("create test dir");
    fs::write(
        &test_source,
        "import root from 'interop-demo-test';\nimport feature from 'interop-demo-test/feature';\nKali.test('interop-demo-test corpus', () => {\n  if (root() !== 0 || feature() !== 0) { throw new Error('interop-demo-test export mismatch'); }\n  console.log(root() + feature());\n});\n",
    )
    .expect("write utility test source");

    let test = run_kali(dir.path(), ["test", test_source.to_str().unwrap()]);
    assert!(
        test.status.success(),
        "utility mixed-format package interop-demo-test should be testable\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test.stdout),
        String::from_utf8_lossy(&test.stderr)
    );
    let stdout = String::from_utf8_lossy(&test.stdout);
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    assert!(stdout.contains("0"), "stdout: {stdout}");
}

#[test]
fn utility_corpus_scoped_packages_remain_executable_on_the_default_standalone_surface() {
    for package in [
        "@babel/runtime",
        "@npmcli/package-json",
        "@jridgewell/sourcemap-codec",
        "@reduxjs/toolkit",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
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
        .expect("write utility source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "scoped utility package {package} should be checkable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        assert!(
            build.status.success(),
            "scoped utility package {package} should be buildable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "scoped utility package {package} should stay executable\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn utility_corpus_scoped_packages_remain_checkable_and_executable_on_the_default_standalone_surface_on_js_input(
) {
    for package in [
        "@babel/runtime",
        "@npmcli/package-json",
        "@jridgewell/sourcemap-codec",
        "@reduxjs/toolkit",
    ] {
        let dir = tempdir().expect("tempdir");
        write_manifest(dir.path(), None);
        write_stub_package(
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
        .expect("write utility JS source");

        let check = run_kali(dir.path(), ["check", source_path.to_str().unwrap()]);
        assert!(
            check.status.success(),
            "scoped utility package {package} should be checkable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );

        let build = run_kali(dir.path(), ["build", source_path.to_str().unwrap()]);
        assert!(
            build.status.success(),
            "scoped utility package {package} should be buildable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = run_kali(dir.path(), ["run", source_path.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "scoped utility package {package} should stay executable on js input\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        assert!(run_stdout.contains("0"), "stdout: {run_stdout}");
    }
}
