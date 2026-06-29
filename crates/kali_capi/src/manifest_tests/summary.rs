use super::*;

#[test]
fn binding_package_manifest_summary_normalizes_string_lists() {
    let manifest = valid_binding_package_manifest();

    let summary = binding_package_manifest_summary(&manifest).expect("summarize manifest");

    assert_eq!(
        summary["runtimeProfiles"],
        serde_json::json!(["fiber-threads", "wasm-threads"])
    );
    assert_eq!(
        summary["artifacts"]["glue"],
        serde_json::json!(["a.py", "z.py"])
    );
}

#[test]
fn binding_package_manifest_summary_rejects_invalid_required_field_types() {
    let cases = [
        ("moduleName", serde_json::json!(1)),
        ("hostAbiVersion", serde_json::json!("two")),
        ("minHostAbiVersion", serde_json::json!(false)),
        ("artifacts.library", serde_json::json!(1)),
        ("artifacts.metadata", serde_json::json!(null)),
        ("artifacts.exportsHeader", serde_json::json!(["sample.h"])),
        ("artifacts.glue", serde_json::json!("sample.py")),
        ("artifacts.glue", serde_json::json!(["sample.py", 1])),
    ];

    for (field, value) in cases {
        let mut manifest = valid_binding_package_manifest();
        match field {
            "moduleName" => manifest["moduleName"] = value,
            "hostAbiVersion" => manifest["hostAbiVersion"] = value,
            "minHostAbiVersion" => manifest["minHostAbiVersion"] = value,
            "artifacts.library" => manifest["artifacts"]["library"] = value,
            "artifacts.metadata" => manifest["artifacts"]["metadata"] = value,
            "artifacts.exportsHeader" => manifest["artifacts"]["exportsHeader"] = value,
            "artifacts.glue" => manifest["artifacts"]["glue"] = value,
            _ => unreachable!("unexpected field: {field}"),
        }

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("invalid required field type should fail");

        assert!(error.contains(field), "unexpected error: {error}");
    }
}

#[test]
fn binding_package_manifest_summary_rejects_non_string_provenance_fields() {
    for (field, value) in [
        ("hostContract", serde_json::json!(1)),
        ("runtimeBackend", serde_json::json!(false)),
    ] {
        let mut manifest = valid_binding_package_manifest();
        manifest[field] = value;

        let error = binding_package_manifest_summary(&manifest)
            .expect_err("invalid provenance field should fail");

        assert!(error.contains(field), "unexpected error: {error}");
    }
}
