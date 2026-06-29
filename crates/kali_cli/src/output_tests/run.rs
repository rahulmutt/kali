use super::*;

#[test]
fn validate_run_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "exitCode": 0,
        "runtimeMs": 12,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "threadTopology": {
            "totalInstances": 0,
            "terminatedInstances": 0,
            "liveInstances": [],
        },
    });

    validate_run_payload_value(&value).expect("run payload should validate");
}

#[test]
fn validate_run_payload_value_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", json!(true)),
        ("runtimeBackend", json!(42)),
        ("hostContract", json!("")),
        ("runtimeBackend", json!("")),
        ("hostContract", json!("   ")),
        ("runtimeBackend", json!("   ")),
    ] {
        let payload = json!({
            "exitCode": 0,
            "runtimeMs": 12,
            field: value,
        });

        let err = validate_run_payload_value(&payload)
            .expect_err("invalid run payload provenance field should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_run_payload_value_rejects_fractional_runtime_ms() {
    let value = json!({
        "exitCode": 0,
        "runtimeMs": 12.25,
    });

    let err = validate_run_payload_value(&value)
        .expect_err("fractional run runtimeMs should fail validation");
    assert!(err.contains("runtimeMs"), "unexpected error: {err}");
}

#[test]
fn validate_run_and_test_payload_value_rejects_malformed_thread_topology() {
    let malformed_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "https://e.co/worker.js",
            "postedMessages": [],
            "postedSharedBuffers": [[[999]]],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": malformed_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": malformed_thread_topology,
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
            }),
        ),
    ] {
        let err = validator(&payload).expect_err("malformed thread topology should fail");
        assert!(
            err.contains("threadTopology") || err.contains("postedSharedBuffers"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_unexpected_thread_topology_keys() {
    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": {
                    "totalInstances": 1,
                    "terminatedInstances": 0,
                    "liveInstances": [],
                    "metadata": true,
                },
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": {
                    "totalInstances": 1,
                    "terminatedInstances": 0,
                    "liveInstances": [],
                    "metadata": true,
                },
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
            }),
        ),
    ] {
        let err = validator(&payload).expect_err("unexpected thread topology keys should fail");
        assert!(
            err.contains("threadTopology contains unexpected key `metadata`"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_unexpected_thread_topology_instance_keys() {
    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": {
                    "totalInstances": 1,
                    "terminatedInstances": 0,
                    "liveInstances": [{
                        "instanceId": 0,
                        "scriptUrl": "https://e.co/worker.js",
                        "postedMessages": [],
                        "postedSharedBuffers": [],
                        "wasTerminated": false,
                        "metadata": true,
                    }],
                },
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": {
                    "totalInstances": 1,
                    "terminatedInstances": 0,
                    "liveInstances": [{
                        "instanceId": 0,
                        "scriptUrl": "https://e.co/worker.js",
                        "postedMessages": [],
                        "postedSharedBuffers": [],
                        "wasTerminated": false,
                        "metadata": true,
                    }],
                },
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
            }),
        ),
    ] {
        let err = validator(&payload)
            .expect_err("unexpected thread topology liveInstances item keys should fail");
        assert!(
            err.contains("threadTopology liveInstances item contains unexpected key `metadata`"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_duplicate_thread_topology_instance_ids() {
    let duplicated_thread_topology = json!({
        "totalInstances": 2,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-0.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-1.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": duplicated_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": duplicated_thread_topology,
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
            }),
        ),
    ] {
        let err =
            validator(&payload).expect_err("duplicate thread topology instance ids should fail");
        assert!(
            err.contains("instanceId must be unique"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_unsorted_thread_topology_instance_ids() {
    let unsorted_thread_topology = json!({
        "totalInstances": 2,
        "terminatedInstances": 0,
        "liveInstances": [
            {
                "instanceId": 1,
                "scriptUrl": "https://e.co/worker-1.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
            {
                "instanceId": 0,
                "scriptUrl": "https://e.co/worker-0.js",
                "postedMessages": [],
                "postedSharedBuffers": [],
                "wasTerminated": false,
            },
        ],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": unsorted_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": unsorted_thread_topology,
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
            }),
        ),
    ] {
        let err =
            validator(&payload).expect_err("unsorted thread topology instance ids should fail");
        assert!(
            err.contains("ordered by ascending instanceId"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_incoherent_thread_topology_counts() {
    let incoherent_thread_topology = json!({
        "totalInstances": 3,
        "terminatedInstances": 1,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "https://e.co/worker-0.js",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": incoherent_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": incoherent_thread_topology,
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
            }),
        ),
    ] {
        let err = validator(&payload).expect_err("incoherent thread topology counts should fail");
        assert!(
            err.contains("totalInstances must equal terminatedInstances + liveInstances.len()"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_whitespace_thread_topology_script_url() {
    let whitespace_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "   ",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": whitespace_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": whitespace_thread_topology,
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
            }),
        ),
    ] {
        let err =
            validator(&payload).expect_err("whitespace thread topology scriptUrl should fail");
        assert_eq!(
            err,
            "threadTopology liveInstances[0] scriptUrl must be a non-empty, non-whitespace string",
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_whitespace_padded_thread_topology_script_url() {
    let padded_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": " https://e.co/worker.js ",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": padded_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": padded_thread_topology,
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
            }),
        ),
    ] {
        let err = validator(&payload)
            .expect_err("whitespace-padded thread topology scriptUrl should fail");
        assert!(err.contains("scriptUrl"), "{kind} error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_non_url_thread_topology_script_url() {
    let malformed_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "worker.js",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": malformed_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": malformed_thread_topology,
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
            }),
        ),
    ] {
        let err = validator(&payload).expect_err("non-URL thread topology scriptUrl should fail");
        assert_eq!(
            err,
            "threadTopology liveInstances[0] scriptUrl must be a valid absolute URL, got worker.js",
            "{kind} error: {err}"
        );
    }
}

#[test]
fn validate_run_and_test_payload_value_rejects_non_canonical_thread_topology_script_url() {
    let non_canonical_thread_topology = json!({
        "totalInstances": 1,
        "terminatedInstances": 0,
        "liveInstances": [{
            "instanceId": 0,
            "scriptUrl": "https://e.co/worker/../worker.js",
            "postedMessages": [],
            "postedSharedBuffers": [],
            "wasTerminated": false,
        }],
    });

    for (kind, validator, payload) in [
        (
            "run",
            validate_run_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "exitCode": 0,
                "runtimeMs": 12,
                "threadTopology": non_canonical_thread_topology.clone(),
            }),
        ),
        (
            "test",
            validate_test_payload_value as fn(&serde_json::Value) -> Result<(), String>,
            json!({
                "total": 4,
                "passed": 3,
                "failed": 1,
                "skipped": 0,
                "runtimeMs": 27,
                "threadTopology": non_canonical_thread_topology,
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
            }),
        ),
    ] {
        let err =
            validator(&payload).expect_err("non-canonical thread topology scriptUrl should fail");
        assert_eq!(
            err,
            "threadTopology liveInstances[0] scriptUrl must be a canonical absolute URL, got https://e.co/worker/../worker.js",
            "{kind} error: {err}"
        );
    }
}
