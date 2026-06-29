use super::*;

#[test]
fn validate_test_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
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
    });

    validate_test_payload_value(&value).expect("test payload should validate");
}

#[test]
fn validate_test_payload_value_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", json!(null)),
        ("runtimeBackend", json!(["wasmtime"])),
        ("hostContract", json!("")),
        ("runtimeBackend", json!("")),
        ("hostContract", json!("   ")),
        ("runtimeBackend", json!("   ")),
    ] {
        let payload = json!({
            "total": 4,
            "passed": 3,
            "failed": 1,
            "skipped": 0,
            "runtimeMs": 27,
            field: value,
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
        });

        let err = validate_test_payload_value(&payload)
            .expect_err("invalid test payload provenance field should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_test_payload_value_rejects_fractional_runtime_ms() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27.5,
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
    });

    let err = validate_test_payload_value(&value)
        .expect_err("fractional test runtimeMs should fail validation");
    assert!(err.contains("runtimeMs"), "unexpected error: {err}");
}

#[test]
fn validate_test_payload_value_rejects_malformed_coverage() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "branch",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unsupported coverage mode should fail validation");
    assert!(err.contains("coverage mode"), "unexpected error: {err}");
}

#[test]
fn validate_test_payload_value_rejects_malformed_coverage_files() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [1],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("malformed coverage file entry should fail validation");
    assert!(
        err.contains("coverage files[0] must be an object"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_empty_or_whitespace_coverage_file_paths() {
    for file in ["", "  \t "] {
        let value = json!({
            "total": 4,
            "passed": 3,
            "failed": 1,
            "skipped": 0,
            "runtimeMs": 27,
            "coverage": {
                "mode": "function",
                "files": [
                    {
                        "file": file,
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
        });

        let err = validate_test_payload_value(&value)
            .expect_err("empty or whitespace coverage file path should fail validation");
        assert!(
            err.contains("coverage files[0].file"),
            "unexpected error: {err}"
        );
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_test_payload_value_rejects_duplicate_coverage_file_rows() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [
                {
                    "file": "src/main.ts",
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                },
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
    });

    let err = validate_test_payload_value(&value)
        .expect_err("duplicate coverage file rows should fail validation");
    assert!(
        err.contains("coverage files[1].file must be unique"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_malformed_coverage_summary() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("malformed coverage summary should fail validation");
    assert!(
        err.contains("coverage summary is missing required key `functionsTotal`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_unexpected_coverage_root_keys() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
            "metadata": {"kind": "extra"},
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unexpected coverage root keys should fail validation");
    assert!(
        err.contains("coverage contains unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_unexpected_coverage_row_keys() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [
                {
                    "file": "src/main.ts",
                    "functionsTotal": 4,
                    "functionsCovered": 3,
                    "functionsMissed": 1,
                    "metadata": true,
                }
            ],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unexpected coverage row keys should fail validation");
    assert!(
        err.contains("coverage files[0] contains unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_test_payload_value_rejects_unexpected_coverage_summary_keys() {
    let value = json!({
        "total": 4,
        "passed": 3,
        "failed": 1,
        "skipped": 0,
        "runtimeMs": 27,
        "coverage": {
            "mode": "function",
            "files": [],
            "summary": {
                "functionsTotal": 4,
                "functionsCovered": 3,
                "functionsMissed": 1,
                "coveragePercent": 75.0,
                "metadata": {"kind": "extra"},
            },
        },
    });

    let err = validate_test_payload_value(&value)
        .expect_err("unexpected coverage summary keys should fail validation");
    assert!(
        err.contains("coverage summary contains unexpected key `metadata`"),
        "unexpected error: {err}"
    );
}
