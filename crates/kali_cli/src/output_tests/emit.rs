use super::*;

#[test]
fn emitted_cli_envelopes_satisfy_the_schema_v1_top_level_shape() {
    let value = emit_envelope_value(
        "doctor",
        true,
        json!([]),
        json!([]),
        json!({"answer": 42}),
        Some("stdout text".to_string()),
        None,
        0,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["schemaVersion"], json!(1));
    assert_eq!(object["command"], json!("doctor"));
    assert_eq!(object["success"], json!(true));
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["payload"], json!({"answer": 42}));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], serde_json::Value::Null);
    assert_eq!(object["exitCode"], json!(0));
}

#[test]
fn emitted_cli_envelopes_sort_diagnostics_before_validation() {
    let source_path = Path::new("/tmp/example.ts");
    let source_text = "first\nsecond\nthird\n";

    let later = diagnostic_to_json(
        &Diagnostic::error(e5::INVALID_CLI_USAGE as u32, "later finding").with_span(Span::new(
            FileId::new(0),
            12,
            13,
        )),
        Some(source_path),
        Some(source_text),
        "error",
    );
    let earlier =
        diagnostic_to_json(
            &Diagnostic::error(e5::INVALID_CLI_USAGE as u32, "earlier finding")
                .with_span(Span::new(FileId::new(0), 0, 1)),
            Some(source_path),
            Some(source_text),
            "error",
        );

    let value = emit_envelope_value(
        "check",
        false,
        json!([later, earlier]),
        json!([]),
        serde_json::Value::Null,
        None,
        None,
        1,
    );

    validate_envelope_value(&value).expect("sorted diagnostics should validate");

    let errors = value["errors"].as_array().expect("errors array");
    assert_eq!(errors[0]["message"], json!("earlier finding"));
    assert_eq!(errors[1]["message"], json!("later finding"));
}

#[test]
fn emitted_cli_envelopes_reject_empty_or_whitespace_command() {
    for command in ["", " \n\t "] {
        let mut value = emit_envelope_value(
            "doctor",
            true,
            json!([]),
            json!([]),
            json!({"answer": 42}),
            None,
            None,
            0,
        );
        value
            .as_object_mut()
            .expect("envelope object")
            .insert("command".to_string(), json!(command));

        let error = validate_envelope_value(&value)
            .expect_err("empty or whitespace command should fail validation");
        assert!(error.contains("command"), "unexpected error: {error}");
        assert!(
            error.contains("non-empty, non-whitespace string"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn emitted_cli_envelopes_reject_non_string_command_stdout_and_stderr_fields() {
    for (field, replacement, expected_fragment) in [
        (
            "command",
            json!(42),
            "CLI envelope command must be a non-empty, non-whitespace string",
        ),
        (
            "stdout",
            json!(false),
            "CLI envelope stdout must be string or null",
        ),
        (
            "stderr",
            json!(["not", "a", "string"]),
            "CLI envelope stderr must be string or null",
        ),
    ] {
        let mut value = emit_envelope_value(
            "doctor",
            true,
            json!([]),
            json!([]),
            json!({"answer": 42}),
            Some("stdout text".to_string()),
            Some("stderr text".to_string()),
            0,
        );
        value
            .as_object_mut()
            .expect("envelope object")
            .insert(field.to_string(), replacement);

        let error = validate_envelope_value(&value)
            .expect_err("non-string CLI envelope field should fail validation");
        assert!(
            error.contains(expected_fragment),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn emitted_cli_envelopes_accept_omitted_stdout_stderr_and_exit_code_fields() {
    let value = json!({
        "schemaVersion": 1,
        "command": "doctor",
        "success": true,
        "errors": [],
        "warnings": [],
        "payload": {"answer": 42}
    });

    validate_envelope_value(&value)
        .expect("externally sourced envelopes may omit stdout, stderr, and exitCode");
}

#[test]
fn emitted_cli_envelopes_preserve_empty_diagnostic_arrays_for_run_text_output() {
    let value = emit_envelope_value(
        "run",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        Some("stdout text".to_string()),
        Some("stderr text".to_string()),
        0,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["command"], json!("run"));
    assert_eq!(object["success"], json!(true));
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["payload"], json!({"result": "ok"}));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], json!("stderr text"));
    assert_eq!(object["exitCode"], json!(0));
}

#[test]
fn emitted_cli_envelopes_accept_artifacts_arrays() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7}
        ]),
    );

    validate_envelope_value(&value).expect("artifacts array should validate");
}

#[test]
fn emitted_cli_envelopes_reject_out_of_order_artifacts() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7},
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("out-of-order artifacts should fail");
    assert!(
        error.contains("must be sorted by role, kind, then path"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_duplicate_primary_artifact_roles() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42},
            {"path": "alt.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 11}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("duplicate primary role should fail");
    assert!(
        error.contains("duplicates primary-executable"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_unrecognized_artifact_roles() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "auxiliary", "bytes": 42},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7}
        ]),
    );

    let error =
        validate_envelope_value(&value).expect_err("unrecognized artifact roles should fail");
    assert!(
        error.contains("canonical schema-v1 role") && error.contains("auxiliary"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_whitespace_padded_artifact_roles() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": " browser-glue ", "bytes": 42},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7}
        ]),
    );

    let error =
        validate_envelope_value(&value).expect_err("whitespace padded artifact roles should fail");
    assert!(error.contains("role"), "unexpected error: {error}");
    assert!(
        error.contains("leading or trailing whitespace"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_accept_debug_source_map_artifact_roles() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7},
            {"path": "main.map", "kind": "source-map", "role": "debug-source-map", "bytes": 11}
        ]),
    );

    validate_envelope_value(&value).expect("debug-source-map artifact roles should validate");
}

#[test]
fn emitted_cli_envelopes_reject_unexpected_artifact_keys() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42, "extra": true},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("unexpected artifact keys should fail");
    assert!(
        error.contains("CLI envelope artifact") && error.contains("unexpected key `extra`"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_invalid_artifact_bytes() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": -1},
            {"path": "main.js", "kind": "js-glue", "role": "browser-glue", "bytes": 7.5}
        ]),
    );

    let error = validate_envelope_value(&value).expect_err("invalid artifact bytes should fail");
    assert!(
        error.contains("bytes must be a non-negative integer"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_empty_or_whitespace_artifact_paths_and_kinds() {
    for (field, invalid_artifact) in [
        (
            "artifacts[0].path",
            json!({"path": "", "kind": "wasm-module", "role": "primary-executable", "bytes": 42}),
        ),
        (
            "artifacts[0].kind",
            json!({"path": "main.js", "kind": "   ", "role": "browser-glue", "bytes": 7}),
        ),
    ] {
        let mut value = emit_envelope_value(
            "build",
            true,
            json!([]),
            json!([]),
            json!({"result": "ok"}),
            None,
            None,
            0,
        );
        value.as_object_mut().expect("envelope object").insert(
            "artifacts".to_string(),
            json!([
                invalid_artifact,
                {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42}
            ]),
        );

        let error = validate_envelope_value(&value)
            .expect_err("empty or whitespace artifact fields should fail");
        assert!(error.contains(field), "unexpected error: {error}");
        assert!(
            error.contains("non-empty, non-whitespace string"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn emitted_cli_envelopes_reject_duplicate_artifact_kind_path_pairs() {
    let mut value = emit_envelope_value(
        "build",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        None,
        None,
        0,
    );
    value.as_object_mut().expect("envelope object").insert(
        "artifacts".to_string(),
        json!([
            {"path": "main.wasm", "kind": "wasm-module", "role": "primary-executable", "bytes": 42},
            {"path": "main.wasm", "kind": "wasm-module", "role": "browser-glue", "bytes": 7}
        ]),
    );

    let error = validate_envelope_value(&value)
        .expect_err("duplicate artifact kind/path pairs should fail");
    assert!(
        error.contains("duplicates artifact `wasm-module` at `main.wasm`"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_reject_unexpected_top_level_keys() {
    let mut value = emit_envelope_value(
        "doctor",
        true,
        json!([]),
        json!([]),
        json!({"answer": 42}),
        None,
        None,
        0,
    );
    value
        .as_object_mut()
        .expect("envelope object")
        .insert("extensionKey".to_string(), json!("not allowed"));

    let error = validate_envelope_value(&value).expect_err("unexpected keys should fail");
    assert!(
        error.contains("unexpected key `extensionKey`"),
        "unexpected error: {error}"
    );
}

#[test]
fn emitted_cli_envelopes_preserve_empty_diagnostic_arrays_for_test_text_output() {
    let value = emit_envelope_value(
        "test",
        true,
        json!([]),
        json!([]),
        json!({"result": "ok"}),
        Some("stdout text".to_string()),
        Some("stderr text".to_string()),
        0,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["command"], json!("test"));
    assert_eq!(object["success"], json!(true));
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["payload"], json!({"result": "ok"}));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], json!("stderr text"));
    assert_eq!(object["exitCode"], json!(0));
}

#[test]
fn diagnostic_json_includes_the_top_level_file_mirror() {
    let diagnostic = Diagnostic::error(e5::INVALID_CLI_USAGE as u32, "message")
        .with_span(Span::new(FileId::new(1), 0, 4));
    let value = diagnostic_to_json(
        &diagnostic,
        Some(Path::new("src/main.ts")),
        Some("test"),
        "error",
    );

    assert_eq!(value["file"], json!("src/main.ts"));
    assert_eq!(value["span"]["file"], json!("src/main.ts"));
}

#[test]
fn diagnostic_json_rejects_a_top_level_file_mirror_mismatch() {
    let envelope = json!({
        "schemaVersion": 1,
        "command": "check",
        "success": false,
        "errors": [{
            "severity": "error",
            "code": "E5101",
            "message": "message",
            "file": "src/other.ts",
            "span": {
                "file": "src/main.ts",
                "line": 1,
                "column": 1,
                "endLine": 1,
                "endColumn": 1,
            },
            "labels": [],
            "related": [],
            "fix": null,
            "notes": [],
        }],
        "warnings": [],
        "payload": null,
        "stdout": null,
        "stderr": null,
        "exitCode": 1,
    });

    let err = validate_envelope_value(&envelope)
        .expect_err("mismatched file mirror should fail validation");
    assert!(
        err.contains("diagnostic file mirror must match span.file"),
        "unexpected error: {err}"
    );
}

#[test]
fn ordinary_cli_result_payloads_accept_schema_permitted_extension_keys() {
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "filesChecked": 3,
            "errorCount": 1,
            "warningCount": 2,
        }),
        validate_check_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "exitCode": 0,
            "runtimeMs": 12,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "threadTopology": {
                "totalInstances": 0,
                "terminatedInstances": 0,
                "liveInstances": [],
            },
        }),
        validate_run_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "total": 4,
            "passed": 3,
            "failed": 1,
            "skipped": 0,
            "runtimeMs": 27,
            "hostContract": "kali-hosted",
            "runtimeBackend": "wasmtime",
            "threadTopology": {
                "totalInstances": 0,
                "terminatedInstances": 0,
                "liveInstances": [],
            },
            "coverage": {
                "mode": "function",
                "files": [
                    {
                        "file": "src/main.ts",
                        "functionsTotal": 4,
                        "functionsCovered": 3,
                        "functionsMissed": 1,
                    }
                ],
                "summary": {
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                    "coveragePercent": 75.0,
                },
            },
        }),
        validate_test_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "filesFormatted": 12,
            "filesChecked": 4,
            "durationMs": 8,
        }),
        validate_fmt_payload_value,
    );
    assert_payload_accepts_schema_permitted_extension_key(
        json!({
            "filesLinted": 4,
            "errorCount": 0,
            "warningCount": 1,
            "fixedCount": 2,
            "durationMs": 9,
        }),
        validate_lint_payload_value,
    );
}

#[test]
fn merge_thread_topology_snapshot_values_renumbers_and_orders_live_instances() {
    let mut target = json!({
        "totalInstances": 2,
        "terminatedInstances": 1,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "https://e.co/worker-0.js",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });
    let source = json!({
        "totalInstances": 2,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-1.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 1,
                "scriptUrl": "https://e.co/worker-2.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });

    merge_thread_topology_snapshot_values(&mut target, &source);

    assert_eq!(target["totalInstances"], json!(4));
    assert_eq!(target["terminatedInstances"], json!(1));
    assert_eq!(
        target["liveInstances"]
            .as_array()
            .expect("live instances")
            .len(),
        3
    );
    assert_eq!(target["liveInstances"][0]["instanceId"], json!(0));
    assert_eq!(target["liveInstances"][1]["instanceId"], json!(1));
    assert_eq!(target["liveInstances"][2]["instanceId"], json!(2));

    validate_test_payload_value(&json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "threadTopology": target,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    }))
    .expect("merged thread topology should validate");
}

#[test]
fn merge_thread_topology_snapshot_values_renumbers_live_instances_after_gapped_ids() {
    let mut target = json!({
        "totalInstances": 3,
        "terminatedInstances": 1,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-0.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 2,
                "scriptUrl": "https://e.co/worker-2.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });
    let source = json!({
        "totalInstances": 2,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-3.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 1,
                "scriptUrl": "https://e.co/worker-4.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });

    merge_thread_topology_snapshot_values(&mut target, &source);

    assert_eq!(target["totalInstances"], json!(5));
    assert_eq!(target["terminatedInstances"], json!(1));
    assert_eq!(
        target["liveInstances"]
            .as_array()
            .expect("live instances")
            .iter()
            .map(|item| item["instanceId"].as_u64().expect("instance id"))
            .collect::<Vec<_>>(),
        vec![0, 2, 3, 4]
    );
    validate_test_payload_value(&json!({
        "total": 5,
        "passed": 4,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "threadTopology": target,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    }))
    .expect("merged thread topology with gapped ids should validate");
}

#[test]
fn emitted_cli_envelopes_preserve_stdout_and_stderr_strings() {
    let value = emit_envelope_value(
        "doctor",
        false,
        json!([]),
        json!([]),
        serde_json::Value::Null,
        Some("stdout text".to_string()),
        Some("stderr text".to_string()),
        1,
    );

    validate_envelope_value(&value).expect("constructed envelope should validate");

    let object = value.as_object().expect("envelope object");
    assert_eq!(object["errors"], json!([]));
    assert_eq!(object["warnings"], json!([]));
    assert_eq!(object["stdout"], json!("stdout text"));
    assert_eq!(object["stderr"], json!("stderr text"));
    assert_eq!(object["exitCode"], json!(1));
}
