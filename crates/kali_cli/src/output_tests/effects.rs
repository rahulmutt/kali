use super::*;

#[test]
fn validate_effects_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads"],
            "compatFeatures": [],
        },
        "entryPoints": ["src/main.ts"],
        "effects": [{
            "kind": "Network.Fetch",
            "locations": [{
                "file": "src/main.ts",
                "line": 12,
                "column": 3,
                "function": "main",
            }],
        }],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    validate_effects_payload_value(&value).expect("effects payload should validate");
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_effect_kind() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": ["src/main.ts"],
        "effects": [{
            "kind": "   ",
            "locations": [{
                "file": "src/main.ts",
                "line": 12,
                "column": 3,
            }],
        }],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("whitespace effect kind should fail validation");
    assert!(err.contains("effects[0] kind"), "unexpected error: {err}");
    assert!(
        err.contains("non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_invalid_effect_locations() {
    for (field, location, expected_fragment) in [
        (
            "file",
            json!({"file": "   ", "line": 12, "column": 3, "function": "main"}),
            "non-empty, non-whitespace string",
        ),
        (
            "line",
            json!({"file": "src/main.ts", "line": 0, "column": 3, "function": "main"}),
            "line must be a positive integer",
        ),
        (
            "column",
            json!({"file": "src/main.ts", "line": 12, "column": 0, "function": "main"}),
            "column must be a positive integer",
        ),
        (
            "function",
            json!({"file": "src/main.ts", "line": 12, "column": 3, "function": "   "}),
            "non-empty, non-whitespace string",
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": ["src/main.ts"],
            "effects": [{
                "kind": "Network.Fetch",
                "locations": [location],
            }],
            "dynamicEffects": false,
            "dynamicReasons": [],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("invalid effect location should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_analysis_context_api_surface() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "   ",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("whitespace analysisContext apiSurface should fail validation");
    assert!(
        err.contains("analysisContext apiSurface"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("non-empty, non-whitespace string"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_analysis_context_sets() {
    for (field, analysis_context, expected_fragment) in [
        (
            "runtimeProfiles",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": ["   "],
                "compatFeatures": [],
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "runtimeProfiles",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": [" wasm-threads "],
                "compatFeatures": [],
            }),
            "leading or trailing whitespace",
        ),
        (
            "compatFeatures",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": ["\n"],
            }),
            "non-empty, non-whitespace string",
        ),
        (
            "compatFeatures",
            json!({
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [" eval "],
            }),
            "leading or trailing whitespace",
        ),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": analysis_context,
            "entryPoints": [],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("whitespace analysisContext set item should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_entry_points() {
    for (invalid_value, expected_fragment) in [
        ("   ", "non-empty, non-whitespace string"),
        (" src/main.ts ", "leading or trailing whitespace"),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": [invalid_value],
            "effects": [],
            "dynamicEffects": false,
            "dynamicReasons": [],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("whitespace entryPoints should fail validation");
        assert!(err.contains("entryPoints[0]"), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_whitespace_dynamic_reasons() {
    for (invalid_value, expected_fragment) in [
        ("   ", "non-empty, non-whitespace string"),
        (" eval ", "leading or trailing whitespace"),
    ] {
        let value = json!({
            "schemaVersion": 1,
            "analysisContext": {
                "apiSurface": "browser",
                "runtimeProfiles": [],
                "compatFeatures": [],
            },
            "entryPoints": [],
            "effects": [],
            "dynamicEffects": true,
            "dynamicReasons": [invalid_value],
        });

        let err = validate_effects_payload_value(&value)
            .expect_err("whitespace dynamicReasons should fail validation");
        assert!(err.contains("dynamicReasons[0]"), "unexpected error: {err}");
        assert!(err.contains(expected_fragment), "unexpected error: {err}");
    }
}

#[test]
fn validate_effects_payload_value_rejects_duplicate_entry_points() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": ["src/main.ts", "src/main.ts"],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("duplicate entryPoints should fail validation");
    assert!(
        err.contains("entryPoints") && err.contains("duplicate item `src/main.ts`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_unexpected_keys() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
        "unexpected": true,
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("unexpected effects keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_effects_payload_value_rejects_unexpected_nested_keys() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
            "unexpected": true,
        },
        "entryPoints": [],
        "effects": [{
            "kind": "Network.Fetch",
            "locations": [{
                "file": "src/main.ts",
                "line": 12,
                "column": 3,
                "function": "main",
                "unexpected": true,
            }],
        }],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("unexpected nested effects keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_effects_payload_value_rejects_dynamic_reasons_when_dynamic_effects_is_false() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": ["eval"],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("non-empty dynamicReasons should fail when dynamicEffects is false");
    assert!(
        err.contains("dynamicReasons") && err.contains("dynamicEffects is false"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_unsorted_dynamic_reasons() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": [],
            "compatFeatures": [],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": true,
        "dynamicReasons": ["proxy-traps", "eval"],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("unsorted dynamicReasons should fail validation");
    assert!(
        err.contains("deduplicated and sorted in lexical order"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_effects_payload_value_rejects_duplicate_analysis_context_sets() {
    let value = json!({
        "schemaVersion": 1,
        "analysisContext": {
            "apiSurface": "browser",
            "runtimeProfiles": ["wasm-threads", "wasm-threads"],
            "compatFeatures": ["eval", "eval"],
        },
        "entryPoints": [],
        "effects": [],
        "dynamicEffects": false,
        "dynamicReasons": [],
    });

    let err = validate_effects_payload_value(&value)
        .expect_err("duplicate analysisContext set items should fail validation");
    assert!(
        err.contains("runtimeProfiles") || err.contains("compatFeatures"),
        "unexpected error: {err}"
    );
}
