use super::*;

#[test]
fn validate_init_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "root": "/workspace/example",
        "manifestPath": "/workspace/example/kali.json",
        "sourcePath": "/workspace/example/src/main.ts",
        "library": false,
    });

    validate_init_payload_value(&value).expect("init payload should validate");
}

#[test]
fn validate_init_payload_value_rejects_blank_paths() {
    for (field, value) in [
        ("root", json!("")),
        ("manifestPath", json!("  \t ")),
        ("sourcePath", json!("\n")),
    ] {
        let payload = json!({
            "root": "/workspace/example",
            "manifestPath": "/workspace/example/kali.json",
            "sourcePath": "/workspace/example/src/main.ts",
            "library": false,
        });
        let mut payload = payload.as_object().expect("init payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_init_payload_value(&serde_json::Value::Object(payload))
            .expect_err("blank init payload paths should fail");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_init_payload_value_rejects_padded_paths() {
    for (field, value) in [
        ("root", json!(" /workspace/example ")),
        ("manifestPath", json!(" /workspace/example/kali.json ")),
        ("sourcePath", json!(" /workspace/example/src/main.ts ")),
    ] {
        let payload = json!({
            "root": "/workspace/example",
            "manifestPath": "/workspace/example/kali.json",
            "sourcePath": "/workspace/example/src/main.ts",
            "library": false,
        });
        let mut payload = payload.as_object().expect("init payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_init_payload_value(&serde_json::Value::Object(payload))
            .expect_err("padded init payload paths should fail");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_fmt_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "filesFormatted": 2,
        "filesChecked": 3,
    });

    validate_fmt_payload_value(&value).expect("fmt payload should validate");
}

#[test]
fn validate_fmt_payload_value_rejects_fractional_counts() {
    let value = json!({
        "filesFormatted": 2.5,
        "filesChecked": 3,
    });

    let err = validate_fmt_payload_value(&value)
        .expect_err("fractional fmt counts should fail validation");
    assert!(err.contains("filesFormatted"), "unexpected error: {err}");
}

#[test]
fn validate_lint_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "filesLinted": 4,
        "errorCount": 1,
        "warningCount": 2,
        "fixedCount": 3,
    });

    validate_lint_payload_value(&value).expect("lint payload should validate");
}

#[test]
fn validate_lint_payload_value_rejects_fractional_counts() {
    let value = json!({
        "filesLinted": 4.25,
        "errorCount": 1,
        "warningCount": 2,
        "fixedCount": 3,
    });

    let err = validate_lint_payload_value(&value)
        .expect_err("fractional lint counts should fail validation");
    assert!(err.contains("filesLinted"), "unexpected error: {err}");
}

#[test]
fn validate_install_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "manifestPath": "/workspace/example/kali.json",
        "lockPath": null,
        "installed": ["semver"],
        "updated": [],
        "removed": [],
    });

    validate_install_payload_value(&value).expect("install payload should validate");
}

#[test]
fn validate_install_payload_value_rejects_unexpected_top_level_keys() {
    let value = json!({
        "manifestPath": "/workspace/example/kali.json",
        "lockPath": null,
        "installed": ["semver"],
        "updated": [],
        "removed": [],
        "extensionKey": true,
    });

    let err = validate_install_payload_value(&value)
        .expect_err("unexpected install payload keys should fail validation");
    assert!(
        err.contains("unexpected key `extensionKey`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_check_payload_value_accepts_the_current_contract_shape() {
    let value = json!({
        "filesChecked": 3,
        "errorCount": 1,
        "warningCount": 2,
    });

    validate_check_payload_value(&value).expect("check payload should validate");
}

#[test]
fn validate_check_payload_value_rejects_fractional_counts() {
    let value = json!({
        "filesChecked": 3.5,
        "errorCount": 1,
        "warningCount": 2,
    });

    let err = validate_check_payload_value(&value)
        .expect_err("fractional check counts should fail validation");
    assert!(err.contains("filesChecked"), "unexpected error: {err}");
}

#[test]
fn validate_init_payload_value_rejects_unexpected_keys() {
    let value = json!({
        "root": "/workspace/example",
        "manifestPath": "/workspace/example/kali.json",
        "sourcePath": "/workspace/example/src/main.ts",
        "library": false,
        "extra": true,
    });

    let err = validate_init_payload_value(&value).expect_err("unexpected init keys should fail");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_init_payload_value_rejects_non_string_and_non_boolean_fields() {
    for (field, value) in [
        ("root", json!(1)),
        ("manifestPath", json!(false)),
        ("sourcePath", json!(["src/main.ts"])),
        ("library", json!("yes")),
    ] {
        let payload = json!({
            "root": "/workspace/example",
            "manifestPath": "/workspace/example/kali.json",
            "sourcePath": "/workspace/example/src/main.ts",
            "library": false,
        });
        let mut payload = payload.as_object().expect("init payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_init_payload_value(&serde_json::Value::Object(payload))
            .expect_err("invalid init payload field should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_install_payload_value_rejects_non_string_entries() {
    let value = json!({
        "manifestPath": "/workspace/example/kali.json",
        "lockPath": null,
        "installed": ["semver", 1],
        "updated": [],
        "removed": [],
    });

    let err =
        validate_install_payload_value(&value).expect_err("non-string install entries should fail");
    assert!(err.contains("installed[1]"), "unexpected error: {err}");
}

#[test]
fn validate_install_payload_value_rejects_non_string_manifest_and_lock_paths() {
    for (field, value) in [
        ("manifestPath", json!(1)),
        ("lockPath", json!(["lock.json"])),
    ] {
        let payload = json!({
            "manifestPath": "/workspace/example/kali.json",
            "lockPath": null,
            "installed": [],
            "updated": [],
            "removed": [],
        });
        let mut payload = payload.as_object().expect("install payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_install_payload_value(&serde_json::Value::Object(payload))
            .expect_err("invalid install payload path should fail");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_install_payload_value_rejects_whitespace_manifest_and_lock_paths() {
    for (field, value) in [("manifestPath", json!("   ")), ("lockPath", json!("\n"))] {
        let payload = json!({
            "manifestPath": "/workspace/example/kali.json",
            "lockPath": null,
            "installed": [],
            "updated": [],
            "removed": [],
        });
        let mut payload = payload.as_object().expect("install payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_install_payload_value(&serde_json::Value::Object(payload))
            .expect_err("whitespace install payload path should fail");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_install_payload_value_rejects_padded_manifest_and_lock_paths() {
    for (field, value) in [
        ("manifestPath", json!(" /workspace/example/kali.json ")),
        ("lockPath", json!(" /workspace/example/kali.lock ")),
    ] {
        let payload = json!({
            "manifestPath": "/workspace/example/kali.json",
            "lockPath": null,
            "installed": [],
            "updated": [],
            "removed": [],
        });
        let mut payload = payload.as_object().expect("install payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_install_payload_value(&serde_json::Value::Object(payload))
            .expect_err("padded install payload path should fail");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("leading or trailing whitespace"),
            "unexpected error: {err}"
        );
    }
}

#[test]
fn validate_install_payload_value_rejects_empty_manifest_and_lock_paths() {
    for (field, value) in [("manifestPath", json!("")), ("lockPath", json!(""))] {
        let payload = json!({
            "manifestPath": "/workspace/example/kali.json",
            "lockPath": null,
            "installed": [],
            "updated": [],
            "removed": [],
        });
        let mut payload = payload.as_object().expect("install payload object").clone();
        payload.insert(field.to_string(), value);

        let err = validate_install_payload_value(&serde_json::Value::Object(payload))
            .expect_err("empty install payload path should fail");
        assert!(err.contains(field), "unexpected error: {err}");
        assert!(
            err.contains("non-empty, non-whitespace string"),
            "unexpected error: {err}"
        );
    }
}
