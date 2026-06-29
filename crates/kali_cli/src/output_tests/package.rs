use super::*;

#[test]
fn validate_package_effects_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    validate_package_effects_payload_value(&value)
        .expect("package-effects payload should validate");
}

#[test]
fn validate_package_effects_payload_value_rejects_unknown_analysis_context_api_surface() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "desktop",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("unknown analysisContext apiSurface should fail validation");
    assert!(err.contains("apiSurface"), "unexpected error: {err}");
    assert!(
        err.contains("default")
            && err.contains("deno")
            && err.contains("node")
            && err.contains("browser"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_whitespace_padded_analysis_context_api_surface() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": " browser ",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("whitespace-padded analysisContext apiSurface should fail validation");
    assert!(
        err.contains("must not have leading or trailing whitespace"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_accepts_jsr_canonical_report_labels() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "@std/path",
            "version": "1.0.8",
            "registry": "jsr",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["jsr:@std/path"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    validate_package_effects_payload_value(&value)
        .expect("jsr package-effects payload should validate");
}

#[test]
fn validate_package_effects_payload_value_rejects_mismatched_report_labels() {
    for (package, entry_point, canonical_label) in [
        (
            json!({
                "name": "@std/path",
                "version": "1.0.8",
                "registry": "jsr",
            }),
            "@std/path",
            "jsr:@std/path",
        ),
        (
            json!({
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            }),
            "npm:semver",
            "semver",
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "package": package,
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "browser",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": [entry_point],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("mismatched report labels should fail validation");
        assert!(
            err.contains("canonical registry package identifier") && err.contains(canonical_label),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_empty_or_whitespace_report_entry_points() {
    for entry_point in ["", "   "] {
        let value = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "browser",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": [entry_point],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("blank report entryPoint should fail validation");
        assert!(err.contains("entryPoints[0]"), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_whitespace_padded_jsr_report_entry_points() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "@std/path",
            "version": "1.0.8",
            "registry": "jsr",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": [" jsr:@std/path "],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("whitespace-padded jsr report entryPoint should fail validation");
    assert!(
        err.contains("entryPoints[0]")
            && err.contains("must not have leading or trailing whitespace"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_unsupported_registry_names() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "pnpm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("unsupported package registry should fail validation");
    assert!(
        err.contains("package registry") && err.contains("`npm` or `jsr`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_non_stable_semver_versions() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3-beta.1",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("prerelease package version should fail validation");
    assert!(
        err.contains("stable SemVer release string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_non_object_payloads() {
    for value in [serde_json::Value::Null, json!("oops"), json!(1)] {
        let err = validate_package_effects_payload_value(&value)
            .expect_err("non-object package-effects payloads should fail");
        assert!(
            err.contains("must be a JSON object"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_non_single_root_reports() {
    for entry_points in [json!([]), json!(["semver", "semver-helpers"])] {
        let value = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": entry_points,
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("non-single-root package-effects payloads should fail validation");
        assert!(
            err.contains("entryPoints must contain exactly one item"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_unexpected_keys() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
        "unexpected": true,
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("unexpected package-effects keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_package_effects_payload_value_rejects_unexpected_nested_keys() {
    let invalid_package = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
            "unexpected": true,
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&invalid_package)
        .expect_err("unexpected package keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");

    let invalid_report = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
            "unexpected": true,
        },
    });

    let err = validate_package_effects_payload_value(&invalid_report)
        .expect_err("unexpected report keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_package_effects_payload_value_rejects_unexpected_analysis_context_keys() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": [],
                "unexpected": true,
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("unexpected analysisContext keys should fail validation");
    assert!(err.contains("analysisContext"), "unexpected error: {err}");
    assert!(
        err.contains("unexpected key `unexpected`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_non_string_package_coordinate_fields() {
    for (field, value) in [
        ("name", json!(1)),
        ("version", json!(false)),
        ("registry", json!(["npm"])),
    ] {
        let payload = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });
        let mut payload = payload
            .as_object()
            .expect("package-effects payload object")
            .clone();
        payload
            .get_mut("package")
            .expect("package coordinate")
            .as_object_mut()
            .expect("package coordinate object")
            .insert(field.to_string(), value);

        let err = validate_package_effects_payload_value(&serde_json::Value::Object(payload))
            .expect_err("invalid package coordinate field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_whitespace_package_coordinate_fields() {
    for (field, value) in [
        ("name", json!("   ")),
        ("version", json!("\n")),
        ("registry", json!("\t")),
    ] {
        let payload = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });
        let mut payload = payload
            .as_object()
            .expect("package-effects payload object")
            .clone();
        payload
            .get_mut("package")
            .expect("package coordinate")
            .as_object_mut()
            .expect("package coordinate object")
            .insert(field.to_string(), value);

        let err = validate_package_effects_payload_value(&serde_json::Value::Object(payload))
            .expect_err("whitespace package coordinate field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_registry_package_names_with_internal_whitespace()
{
    for name in ["semi ver", "@types/ node"] {
        let value = json!({
            "schemaVersion": 1,
            "package": {
                "name": name,
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("internal whitespace package names should fail validation");
        assert!(
            err.contains("package-effects payload package name")
                && err.contains("without whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_registry_prefixed_package_names() {
    for (registry, name) in [("npm", "npm:semver"), ("jsr", "jsr:@std/path")] {
        let value = json!({
            "schemaVersion": 1,
            "package": {
                "name": name,
                "version": "7.6.3",
                "registry": registry,
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": {
                    "apiSurface": "default",
                    "runtimeProfiles": [],
                    "compatFeatures": [],
                },
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("registry-prefixed package names should fail validation");
        assert!(
            err.contains("registry-native") && err.contains("prefix"),
            "unexpected error: {err}"
        );
        assert!(err.contains(name), "unexpected error: {err}");
        assert!(err.contains(registry), "unexpected error: {err}");
    }
}

#[test]
fn validate_package_effects_payload_value_rejects_duplicate_analysis_context_sets() {
    let value = json!({
        "schemaVersion": 1,
        "package": {
            "name": "semver",
            "version": "7.6.3",
            "registry": "npm",
        },
        "report": {
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "default",
                "runtimeProfiles": ["wasm-threads", "wasm-threads"],
                "compatFeatures": ["eval", "eval"],
            },
            "entryPoints": ["semver"],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        },
    });

    let err = validate_package_effects_payload_value(&value)
        .expect_err("duplicate analysisContext set items should fail validation");
    assert!(
        err.contains("runtimeProfiles") || err.contains("compatFeatures"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_package_effects_payload_value_rejects_whitespace_analysis_context_sets() {
    for (field, analysis_context) in [
        (
            "runtimeProfiles",
            json!({
                "apiSurface": "default",
                "runtimeProfiles": ["   "],
                "compatFeatures": [],
            }),
        ),
        (
            "compatFeatures",
            json!({
                "apiSurface": "default",
                "runtimeProfiles": [],
                "compatFeatures": ["\n"],
            }),
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "package": {
                "name": "semver",
                "version": "7.6.3",
                "registry": "npm",
            },
            "report": {
                "schemaVersion": 1,
                "analysisContext": analysis_context,
                "entryPoints": ["semver"],
                "effects": [],
                "dynamicEffects": false,
                "dynamicReasons": [],
            },
        });

        let err = validate_package_effects_payload_value(&value)
            .expect_err("whitespace analysisContext set item should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_package_audit_payload_value_accepts_null() {
    validate_package_audit_payload_value(&serde_json::Value::Null)
        .expect("package-audit payload should validate");
}

#[test]
fn validate_package_audit_payload_value_rejects_non_null_payloads() {
    let value = json!({"unexpected": true});

    let err = validate_package_audit_payload_value(&value)
        .expect_err("non-null package-audit payloads should fail");
    assert!(err.contains("must be null"), "unexpected error: {err}");
}
