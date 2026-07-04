use super::*;

#[test]
fn schema_documents_exist_and_parse() {
    let files = [
        "schemas/README.md",
        "schemas/envelope/v1.json",
        "schemas/diagnostic/v1.json",
        "schemas/manifest/v1.json",
        "schemas/lock/v1.json",
        "schemas/policy/v1.json",
        "schemas/artifact-meta/v1.json",
        "schemas/artifact-meta/lib-wit/v1.json",
        "schemas/artifact-meta/capi/v1.json",
        "schemas/artifact-meta/component/v1.json",
        "schemas/artifact-meta/binding-package/v1.json",
        "schemas/result/check/v1.json",
        "schemas/result/build/v1.json",
        "schemas/result/run/v1.json",
        "schemas/result/test/v1.json",
        "schemas/result/install/v1.json",
        "schemas/result/init/v1.json",
        "schemas/result/fmt/v1.json",
        "schemas/result/lint/v1.json",
        "schemas/result/doctor/v1.json",
        "schemas/benchmark/v1.json",
        "schemas/result/effects/v1.json",
        "schemas/result/package-effects/v1.json",
        "schemas/result/package-audit/v1.json",
    ];

    let root = repo_root();
    for relative in files {
        let path = root.join(relative);
        assert!(path.exists(), "missing schema document: {}", path.display());

        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            let raw = fs::read_to_string(&path).expect("read schema document");
            let value: serde_json::Value = serde_json::from_str(&raw).expect("parse schema json");
            assert!(
                value.get("$schema").is_some(),
                "missing $schema in {}",
                path.display()
            );
            assert!(
                value.get("title").is_some(),
                "missing title in {}",
                path.display()
            );
        }
    }
}

#[test]
fn core_schema_documents_match_current_cli_contracts() {
    let root = repo_root();

    let envelope: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/envelope/v1.json")).expect("read envelope schema"),
    )
    .expect("parse envelope schema");
    assert_eq!(envelope["additionalProperties"], false);
    assert_eq!(
        required_fields(&envelope),
        [
            "schemaVersion",
            "command",
            "success",
            "errors",
            "warnings",
            "payload"
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(envelope["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(envelope["properties"]["command"]["type"], "string");
    assert_eq!(envelope["properties"]["success"]["type"], "boolean");
    assert_eq!(envelope["properties"]["errors"]["type"], "array");
    assert_eq!(envelope["properties"]["warnings"]["type"], "array");
    assert_eq!(
        envelope["properties"]["errors"]["items"]["$ref"],
        "https://kali-lang.org/schemas/diagnostic/v1"
    );
    assert_eq!(
        envelope["properties"]["warnings"]["items"]["$ref"],
        "https://kali-lang.org/schemas/diagnostic/v1"
    );
    assert_eq!(
        envelope["properties"]["payload"]["anyOf"]
            .as_array()
            .expect("payload variants")
            .iter()
            .map(|variant| variant["type"].as_str().expect("payload variant type"))
            .collect::<Vec<_>>(),
        ["object", "array", "string", "number", "boolean", "null"]
    );
    assert_eq!(envelope["properties"]["timings"]["type"], "array");
    assert_eq!(
        envelope["properties"]["timings"]["items"]["$ref"],
        "#/$defs/timing"
    );
    assert_eq!(envelope["$defs"]["timing"]["type"], "object");
    assert_eq!(envelope["$defs"]["timing"]["additionalProperties"], false);
    assert_eq!(
        required_fields(&envelope["$defs"]["timing"]),
        ["phase", "milliseconds"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        envelope["$defs"]["timing"]["properties"]["phase"]["type"],
        "string"
    );
    assert_eq!(
        envelope["$defs"]["timing"]["properties"]["milliseconds"]["type"],
        "number"
    );
    assert_eq!(envelope["properties"]["artifacts"]["type"], "array");
    assert_eq!(
        envelope["properties"]["artifacts"]["items"]["$ref"],
        "#/$defs/artifact"
    );
    assert_eq!(envelope["$defs"]["artifact"]["additionalProperties"], false);
    assert_eq!(
        envelope["properties"]["stdout"]["type"],
        serde_json::json!(["string", "null"])
    );
    assert_eq!(
        envelope["properties"]["stderr"]["type"],
        serde_json::json!(["string", "null"])
    );
    assert_eq!(envelope["properties"]["exitCode"]["type"], "integer");
    assert!(envelope["required"]
        .as_array()
        .expect("required array")
        .iter()
        .any(|value| value == "payload"));

    let build: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/build/v1.json")).expect("read build schema"),
    )
    .expect("parse build schema");
    let build_variants = build["anyOf"].as_array().expect("anyOf array");
    assert_eq!(build_variants.len(), 6);

    for variant in build_variants.iter().take(5) {
        assert_eq!(variant["properties"]["sizeBytes"]["type"], "integer");
        assert_eq!(variant["properties"]["sizeBytes"]["minimum"], 0);
    }

    assert_eq!(
        build_variants[0]["properties"]["artifactKind"]["const"],
        "executable"
    );
    assert_eq!(
        required_fields(&build_variants[0]),
        [
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );

    assert_eq!(
        build_variants[1]["properties"]["artifactKind"]["const"],
        "lib"
    );
    assert_eq!(
        required_fields(&build_variants[1]),
        [
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "metadataPath",
            "witPath",
            "artifacts",
            "exports",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );

    assert_eq!(
        build_variants[2]["properties"]["artifactKind"]["const"],
        "bundle"
    );
    assert_eq!(
        required_fields(&build_variants[2]),
        [
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "artifacts",
            "exports",
            "bundleFormat",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(
        build_variants[2]["properties"]["bundleFormat"]["type"],
        "string"
    );
    assert_eq!(
        build_variants[2]["properties"]["bundleFormat"]["enum"]
            .as_array()
            .expect("bundle format enum")
            .iter()
            .map(|value| value.as_str().expect("bundle format enum string"))
            .collect::<Vec<_>>(),
        vec!["esm", "cjs"]
    );
    for variant_index in [1, 2, 3, 4] {
        assert_eq!(
            build_variants[variant_index]["properties"]["exports"]["type"],
            "array"
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["exports"]["items"]["type"],
            "object"
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["exports"]["items"]["additionalProperties"],
            false
        );
        assert_eq!(
            required_fields(&build_variants[variant_index]["properties"]["exports"]["items"]),
            ["name", "signature"]
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["exports"]["items"]["properties"]["name"],
            serde_json::json!({"type": "string", "minLength": 1, "pattern": "^\\S(?:.*\\S)?$"})
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["exports"]["items"]["properties"]
                ["signature"],
            serde_json::json!({"type": "string", "minLength": 1, "pattern": "^\\S(?:.*\\S)?$"})
        );
    }

    let build_artifact = &build["$defs"]["buildArtifact"];
    assert_eq!(build_artifact["type"], "object");
    assert_eq!(build_artifact["additionalProperties"], false);
    assert_eq!(
        required_fields(build_artifact),
        ["kind", "path"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(build_artifact["properties"]["kind"]["type"], "string");
    assert_eq!(build_artifact["properties"]["path"]["type"], "string");
    assert_eq!(build_artifact["properties"]["role"]["type"], "string");

    for variant_index in [1, 2, 3, 4, 5] {
        assert_eq!(
            build_variants[variant_index]["properties"]["artifacts"]["type"],
            "array"
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["artifacts"]["items"]["$ref"],
            "#/$defs/buildArtifact"
        );
    }

    assert_eq!(
        build_variants[3]["properties"]["artifactKind"]["const"],
        "capi"
    );
    assert_eq!(
        required_fields(&build_variants[3]),
        [
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "metadataPath",
            "witPath",
            "headerPath",
            "artifacts",
            "exports",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );

    assert_eq!(
        build_variants[4]["properties"]["artifactKind"]["const"],
        "component"
    );
    assert_eq!(
        required_fields(&build_variants[4]),
        [
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "metadataPath",
            "witPath",
            "bindingPackagePath",
            "artifacts",
            "exports",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );

    assert_eq!(
        required_fields(&build_variants[5]),
        ["artifacts"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        build_variants[5]["properties"]["artifacts"]["type"],
        "array"
    );

    for (variant_index, variant) in build_variants.iter().enumerate() {
        assert_eq!(
            variant["additionalProperties"], true,
            "unexpected build schema additionalProperties at variant {variant_index}"
        );
    }

    for (variant_index, expected_properties) in [
        (0, vec!["hostContract", "runtimeBackend", "profileDataHash"]),
        (
            1,
            vec![
                "witPath",
                "hostContract",
                "runtimeBackend",
                "profileDataHash",
            ],
        ),
        (2, vec!["hostContract", "runtimeBackend", "profileDataHash"]),
        (
            3,
            vec![
                "witPath",
                "hostContract",
                "runtimeBackend",
                "profileDataHash",
            ],
        ),
        (
            4,
            vec![
                "witPath",
                "bindingPackagePath",
                "hostContract",
                "runtimeBackend",
                "profileDataHash",
            ],
        ),
    ] {
        for property in expected_properties {
            assert!(
                build_variants[variant_index]["properties"]
                    .get(property)
                    .is_some(),
                "missing build schema property: variant {variant_index} property {property}"
            );
        }
    }

    for variant_index in [0, 1, 2, 3, 4] {
        assert_eq!(
            build_variants[variant_index]["properties"]["buildMode"]["type"],
            "string"
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["buildMode"]["enum"],
            serde_json::json!(["fast", "release", "release-advanced"])
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["hostContract"]["type"],
            "string"
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["hostContract"]["minLength"],
            1
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["runtimeBackend"]["type"],
            "string"
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["runtimeBackend"]["minLength"],
            1
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["profileDataHash"]["type"],
            "string"
        );
        assert_eq!(
            build_variants[variant_index]["properties"]["profileDataHash"]["minLength"],
            1
        );
    }
    for variant_index in [1, 3, 4] {
        assert_eq!(
            build_variants[variant_index]["properties"]["witPath"]["type"],
            "string"
        );
    }
    assert_eq!(
        build_variants[4]["properties"]["bindingPackagePath"]["type"],
        "string"
    );

    let test_result: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/test/v1.json")).expect("read test schema"),
    )
    .expect("parse test schema");
    assert_eq!(test_result["additionalProperties"], true);
    assert!(test_result["properties"]["coverage"].is_object());
    assert_eq!(test_result["properties"]["coverage"]["type"], "object");
    assert_eq!(
        test_result["properties"]["coverage"]["additionalProperties"],
        false
    );
    assert_eq!(
        test_result["properties"]["coverage"]["required"]
            .as_array()
            .expect("coverage required array")
            .iter()
            .map(|value| value.as_str().expect("coverage required string"))
            .collect::<Vec<_>>(),
        vec!["mode", "files", "summary"]
    );
    assert_eq!(
        test_result["properties"]["coverage"]["properties"]["mode"]["const"],
        "function"
    );
    assert_eq!(
        test_result["properties"]["coverage"]["properties"]["files"]["items"]["type"],
        "object"
    );
    assert_eq!(
        test_result["properties"]["coverage"]["properties"]["files"]["items"]
            ["additionalProperties"],
        false
    );
    assert_eq!(
        test_result["properties"]["coverage"]["properties"]["files"]["items"]["required"]
            .as_array()
            .expect("coverage file required array")
            .iter()
            .map(|value| value.as_str().expect("coverage file required string"))
            .collect::<Vec<_>>(),
        vec![
            "file",
            "functionsTotal",
            "functionsCovered",
            "functionsMissed"
        ]
    );
    assert_eq!(
        test_result["properties"]["coverage"]["properties"]["summary"]["type"],
        "object"
    );
    assert_eq!(
        test_result["properties"]["coverage"]["properties"]["summary"]["additionalProperties"],
        false
    );
    assert_eq!(
        test_result["properties"]["coverage"]["properties"]["summary"]["required"]
            .as_array()
            .expect("coverage summary required array")
            .iter()
            .map(|value| value.as_str().expect("coverage summary required string"))
            .collect::<Vec<_>>(),
        vec![
            "functionsTotal",
            "functionsCovered",
            "functionsMissed",
            "coveragePercent"
        ]
    );
    assert_eq!(
        test_result["properties"]["threadTopology"]["type"],
        "object"
    );
    assert_eq!(
        test_result["properties"]["threadTopology"]["additionalProperties"],
        false
    );
    assert_eq!(
        test_result["properties"]["threadTopology"]["required"]
            .as_array()
            .expect("test thread topology required array")
            .iter()
            .map(|value| value
                .as_str()
                .expect("test thread topology required string"))
            .collect::<Vec<_>>(),
        vec!["totalInstances", "terminatedInstances", "liveInstances"]
    );
    assert_eq!(
        test_result["properties"]["threadTopology"]["properties"]["liveInstances"]["items"]
            ["properties"]["postedSharedBuffers"]["items"]["items"]["maximum"],
        255
    );

    let artifact_meta: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/artifact-meta/v1.json"))
            .expect("read artifact schema"),
    )
    .expect("parse artifact schema");
    assert_eq!(artifact_meta["title"], "Kali Artifact Metadata v1");
    assert_eq!(artifact_meta["type"], "object");
    assert_eq!(artifact_meta["additionalProperties"], true);
    assert_eq!(
        required_fields(&artifact_meta),
        [
            "schemaVersion",
            "artifactKind",
            "entrypoint",
            "buildMode",
            "apiSurface",
            "kaliVersion",
            "sourceHash",
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(artifact_meta["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(
        artifact_meta["properties"]["artifactKind"]["enum"],
        serde_json::json!(["executable", "lib", "bundle", "capi", "component"])
    );
    assert_eq!(
        artifact_meta["properties"]["entrypoint"],
        serde_json::json!({"type": "string", "minLength": 1, "pattern": "^\\S(?:.*\\S)?$"})
    );
    assert_eq!(artifact_meta["properties"]["buildMode"]["type"], "string");
    assert_eq!(
        artifact_meta["properties"]["buildMode"]["enum"],
        serde_json::json!(["fast", "release", "release-advanced"])
    );
    assert_eq!(
        artifact_meta["properties"]["apiSurface"],
        serde_json::json!({"type": "string", "minLength": 1, "pattern": "^\\S(?:.*\\S)?$"})
    );
    assert_eq!(
        artifact_meta["properties"]["runtimeProfiles"]["type"],
        "array"
    );
    assert_eq!(
        artifact_meta["properties"]["runtimeProfiles"]["items"]["type"],
        "string"
    );
    assert_eq!(
        artifact_meta["properties"]["maxSpecializations"]["type"],
        "integer"
    );
    assert_eq!(
        artifact_meta["properties"]["maxSpecializations"]["minimum"],
        0
    );
    assert_eq!(
        artifact_meta["properties"]["hostContract"]["type"],
        "string"
    );
    assert_eq!(
        artifact_meta["properties"]["runtimeBackend"]["type"],
        "string"
    );
    assert_eq!(artifact_meta["properties"]["kaliVersion"]["type"], "string");
    assert_eq!(artifact_meta["properties"]["sourceHash"]["type"], "string");
    assert_eq!(
        artifact_meta["properties"]["profileDataHash"]["type"],
        "string"
    );
    assert_eq!(artifact_meta["properties"]["exports"]["type"], "array");
    assert_eq!(
        artifact_meta["properties"]["exports"]["items"]["type"],
        "object"
    );
    assert_eq!(
        artifact_meta["properties"]["exports"]["items"]["additionalProperties"],
        false
    );
    assert_eq!(
        artifact_meta["properties"]["exports"]["items"]["required"],
        serde_json::json!(["name", "signature"])
    );
    assert_eq!(
        artifact_meta["properties"]["exports"]["items"]["properties"]["name"],
        serde_json::json!({"type": "string", "minLength": 1, "pattern": "^\\S(?:.*\\S)?$"})
    );
    assert_eq!(
        artifact_meta["properties"]["exports"]["items"]["properties"]["signature"],
        serde_json::json!({"type": "string", "minLength": 1, "pattern": "^\\S(?:.*\\S)?$"})
    );

    let schemas_18 =
        fs::read_to_string(root.join("specs/18-schemas.md")).expect("read schemas chapter");
    let snapshot_note = schemas_18
        .lines()
        .find(|line| line.contains("implementation-specific artifact-kind labels"))
        .expect("implementation-specific build-result artifact-kind labels note");
    assert!(
        snapshot_note.contains(
            "(`meta-json`, `chunk-wasm`, `chunk-js`, `chunk-source-map`, and `chunk-meta-json`)"
        ),
        "specs/18-schemas.md should record the current implementation-specific build-result artifact kinds in-order"
    );
    for label in [
        "`meta-json`",
        "`chunk-wasm`",
        "`chunk-js`",
        "`chunk-source-map`",
        "`chunk-meta-json`",
    ] {
        assert_eq!(
            snapshot_note.matches(label).count(),
            1,
            "snapshot-level artifact label {label} should appear exactly once"
        );
    }

    let binding_package: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/artifact-meta/binding-package/v1.json"))
            .expect("read binding package schema"),
    )
    .expect("parse binding package schema");
    assert_eq!(
        binding_package["properties"]["kind"]["const"],
        "binding-package"
    );
    for property in [
        "moduleName",
        "hostAbiVersion",
        "minHostAbiVersion",
        "maxSpecializations",
    ] {
        assert!(
            binding_package["properties"].get(property).is_some(),
            "missing binding package schema property: {property}"
        );
    }
    assert_eq!(
        binding_package["properties"]["maxSpecializations"]["minimum"],
        0
    );
    assert!(binding_package["properties"]["artifacts"]["required"]
        .as_array()
        .expect("binding package required array")
        .iter()
        .any(|value| value == "glue"));

    let diagnostic: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/diagnostic/v1.json"))
            .expect("read diagnostic schema"),
    )
    .expect("parse diagnostic schema");
    assert_eq!(diagnostic["type"], "object");
    assert_eq!(diagnostic["additionalProperties"], false);
    assert_eq!(
        diagnostic["required"]
            .as_array()
            .expect("diagnostic required array")
            .iter()
            .map(|value| value.as_str().expect("diagnostic required string"))
            .collect::<Vec<_>>(),
        vec!["severity", "code", "message", "span", "labels", "related", "fix", "notes"]
    );
    assert_eq!(
        diagnostic["properties"]["severity"]["enum"]
            .as_array()
            .expect("severity enum array")
            .iter()
            .map(|value| value.as_str().expect("severity enum string"))
            .collect::<Vec<_>>(),
        vec!["error", "warning", "info", "hint"]
    );
    assert_eq!(
        diagnostic["properties"]["code"]["pattern"],
        "^[EWIH][0-9]{4}$"
    );
    assert_eq!(
        diagnostic["properties"]["help"],
        serde_json::json!({"type": ["string", "null"]})
    );
    assert_eq!(
        diagnostic["properties"]["fix"],
        serde_json::json!({"anyOf": [{"type": "null"}, {"$ref": "#/$defs/suggestedFix"}]})
    );
    assert_eq!(
        diagnostic["properties"]["span"]["$ref"],
        "#/$defs/sourceSpan"
    );
    assert_eq!(
        diagnostic["properties"]["labels"]["items"]["$ref"],
        "#/$defs/label"
    );
    assert_eq!(
        diagnostic["properties"]["related"]["items"]["$ref"],
        "#/$defs/relatedInfo"
    );
    assert_eq!(
        diagnostic["properties"]["context"]["$ref"],
        "#/$defs/diagnosticContext"
    );
    for def_name in [
        "sourceLocation",
        "sourceSpan",
        "label",
        "relatedInfo",
        "textEdit",
        "suggestedFix",
        "diagnosticContext",
    ] {
        assert_eq!(
            diagnostic["$defs"][def_name]["additionalProperties"], false,
            "{def_name} should be fixed-shape"
        );
    }
    assert_eq!(
        required_fields(&diagnostic["$defs"]["sourceSpan"]),
        vec!["file", "line", "column", "endLine", "endColumn"]
    );
    assert_eq!(
        required_fields(&diagnostic["$defs"]["label"]),
        vec!["span", "message", "style"]
    );
    for key in ["line", "column", "endLine", "endColumn"] {
        assert_eq!(
            diagnostic["$defs"]["sourceSpan"]["properties"][key]["minimum"],
            1
        );
    }
    for key in ["line", "column"] {
        assert_eq!(
            diagnostic["$defs"]["sourceLocation"]["properties"][key]["minimum"],
            1
        );
    }
    assert_eq!(
        required_fields(&diagnostic["$defs"]["diagnosticContext"]),
        vec!["origin"]
    );
    assert_eq!(
        diagnostic["$defs"]["diagnosticContext"]["properties"]["origin"]["enum"]
            .as_array()
            .expect("diagnostic context origin enum array")
            .iter()
            .map(|value| value
                .as_str()
                .expect("diagnostic context origin enum string"))
            .collect::<Vec<_>>(),
        vec!["cli", "config", "default", "source"]
    );
    assert_eq!(
        diagnostic["$defs"]["diagnosticContext"]["properties"]["configPath"]["type"],
        serde_json::json!(["string", "null"])
    );
    assert_eq!(
        diagnostic["$defs"]["diagnosticContext"]["properties"]["flag"]["type"],
        serde_json::json!(["string", "null"])
    );
    assert_eq!(
        diagnostic["$defs"]["diagnosticContext"]["properties"]["requestedValue"],
        serde_json::json!({})
    );
    assert_eq!(
        diagnostic["$defs"]["diagnosticContext"]["properties"]["effectiveValue"],
        serde_json::json!({})
    );

    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/manifest/v1.json")).expect("read manifest schema"),
    )
    .expect("parse manifest schema");
    assert_eq!(manifest["$id"], "https://kali-lang.org/schemas/manifest/v1");
    assert_eq!(manifest["type"], "object");
    assert_eq!(manifest["additionalProperties"], true);
    assert_eq!(manifest["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(manifest["properties"]["$schema"]["type"], "string");
    assert_eq!(
        manifest["required"]
            .as_array()
            .expect("manifest required array")
            .iter()
            .map(|value| value.as_str().expect("manifest required string"))
            .collect::<Vec<_>>(),
        vec!["schemaVersion"]
    );
    for property in ["compilerOptions", "compat"] {
        assert_eq!(manifest["properties"][property]["type"], "object");
    }
    assert_eq!(
        manifest["properties"]["sandbox"]["type"],
        serde_json::json!(["string", "null"])
    );
    for property in ["include", "exclude"] {
        assert_eq!(manifest["properties"][property]["type"], "array");
        assert_eq!(manifest["properties"][property]["items"]["type"], "string");
    }
    for property in ["imports", "dependencies", "devDependencies"] {
        assert_eq!(manifest["properties"][property]["type"], "object");
        assert_eq!(
            manifest["properties"][property]["additionalProperties"]["type"],
            "string"
        );
    }

    let lock: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/lock/v1.json")).expect("read lock schema"),
    )
    .expect("parse lock schema");
    assert_eq!(lock["$id"], "https://kali-lang.org/schemas/lock/v1");
    assert_eq!(lock["type"], "object");
    assert_eq!(lock["additionalProperties"], true);
    assert_eq!(lock["properties"]["version"]["const"], 1);
    assert_eq!(lock["properties"]["schemaVersion"]["type"], "integer");
    assert_eq!(lock["properties"]["packages"]["type"], "object");
    assert_eq!(lock["properties"]["rawUrls"]["type"], "object");
    assert_eq!(
        lock["required"]
            .as_array()
            .expect("lock required array")
            .iter()
            .map(|value| value.as_str().expect("lock required string"))
            .collect::<Vec<_>>(),
        vec!["version"]
    );
    assert_eq!(
        lock["properties"]["packages"]["additionalProperties"]["type"],
        "object"
    );
    assert_eq!(
        lock["properties"]["packages"]["additionalProperties"]["additionalProperties"],
        true
    );
    for property in ["registry", "integrity", "resolved"] {
        assert_eq!(
            lock["properties"]["packages"]["additionalProperties"]["properties"][property]["type"],
            "string"
        );
    }
    assert_eq!(
        lock["properties"]["packages"]["additionalProperties"]["required"]
            .as_array()
            .expect("lock package required array")
            .iter()
            .map(|value| value.as_str().expect("lock package required string"))
            .collect::<Vec<_>>(),
        vec!["registry", "integrity", "resolved", "dependencies"]
    );
    assert_eq!(lock["properties"]["rawUrls"]["type"], "object");
    assert_eq!(
        lock["properties"]["rawUrls"]["additionalProperties"]["type"],
        "object"
    );
    assert_eq!(
        lock["properties"]["rawUrls"]["additionalProperties"]["additionalProperties"],
        true
    );
    assert_eq!(
        lock["properties"]["rawUrls"]["additionalProperties"]["required"]
            .as_array()
            .expect("lock rawUrls required array")
            .iter()
            .map(|value| value.as_str().expect("lock rawUrls required string"))
            .collect::<Vec<_>>(),
        vec!["integrity", "cached"]
    );

    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/policy/v1.json")).expect("read policy schema"),
    )
    .expect("parse policy schema");
    assert_eq!(policy["$id"], "https://kali-lang.org/schemas/policy/v1");
    assert_eq!(policy["type"], "object");
    assert_eq!(policy["additionalProperties"], true);
    assert_eq!(policy["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(policy["properties"]["effects"]["type"], "object");
    assert_eq!(
        policy["properties"]["effects"]["additionalProperties"],
        false
    );
    assert_eq!(policy["properties"]["resources"]["type"], "object");
    assert_eq!(
        policy["properties"]["resources"]["additionalProperties"],
        false
    );
    assert_eq!(
        policy["required"]
            .as_array()
            .expect("policy required array")
            .iter()
            .map(|value| value.as_str().expect("policy required string"))
            .collect::<Vec<_>>(),
        vec!["schemaVersion", "effects", "resources"]
    );
    assert_eq!(
        policy["properties"]["effects"]["required"]
            .as_array()
            .expect("policy effects required array")
            .iter()
            .map(|value| value.as_str().expect("policy effects required string"))
            .collect::<Vec<_>>(),
        vec![
            "fileSystem",
            "network",
            "process",
            "timer",
            "eval",
            "random",
            "console"
        ]
    );
    assert_eq!(
        policy["properties"]["resources"]["properties"]
            .as_object()
            .expect("policy resource properties")
            .len(),
        5
    );
    assert_eq!(
        policy["properties"]["resources"]["properties"]["maxThreads"]["minimum"],
        0
    );

    let package_effects: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/package-effects/v1.json"))
            .expect("read package-effects schema"),
    )
    .expect("parse package-effects schema");
    assert_eq!(package_effects["title"], "Kali Package Effects Result v1");
    assert_eq!(
        package_effects["description"],
        "Native JSON payload emitted by `kali package-effects`."
    );
    assert_eq!(package_effects["type"], "object");
    assert_eq!(package_effects["additionalProperties"], false);
    assert_eq!(package_effects["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(
        required_fields(&package_effects),
        ["schemaVersion", "package", "report"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        required_fields(&package_effects["properties"]["package"]),
        ["name", "version", "registry"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        package_effects["properties"]["package"]["additionalProperties"],
        false
    );
    assert_eq!(
        package_effects["properties"]["package"]["properties"]["name"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["package"]["properties"]["version"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["package"]["properties"]["version"]["pattern"],
        "^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)(?:\\+[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?$"
    );
    assert_eq!(
        package_effects["properties"]["package"]["properties"]["registry"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["required"]
            .as_array()
            .expect("package-effects analysisContext required array")
            .iter()
            .map(|value| value
                .as_str()
                .expect("package-effects analysisContext required string"))
            .collect::<Vec<_>>(),
        vec!["apiSurface", "runtimeProfiles", "compatFeatures"]
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["kind"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["kind"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["required"]
            .as_array()
            .expect("package-effects report required array")
            .iter()
            .map(|value| value
                .as_str()
                .expect("package-effects report required string"))
            .collect::<Vec<_>>(),
        vec![
            "schemaVersion",
            "analysisContext",
            "entryPoints",
            "effects",
            "dynamicEffects",
            "dynamicReasons"
        ]
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["schemaVersion"]["const"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["additionalProperties"],
        false
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["type"],
        "object"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]
            ["additionalProperties"],
        false
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["apiSurface"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["apiSurface"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["runtimeProfiles"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["runtimeProfiles"]["items"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["runtimeProfiles"]["items"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["compatFeatures"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["compatFeatures"]["items"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["compatFeatures"]["items"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["entryPoints"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["entryPoints"]["items"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]
            ["additionalProperties"],
        false
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["required"]
            .as_array()
            .expect("package-effects effect required array")
            .iter()
            .map(|value| value
                .as_str()
                .expect("package-effects effect required string"))
            .collect::<Vec<_>>(),
        vec!["kind", "locations"]
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["items"]["additionalProperties"],
        false
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["items"]["properties"]["file"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["items"]["properties"]["file"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["items"]["required"]
            .as_array()
            .expect("package-effects effect location required array")
            .iter()
            .map(|value| value
                .as_str()
                .expect("package-effects effect location required string"))
            .collect::<Vec<_>>(),
        vec!["file", "line", "column"]
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["items"]["properties"]["line"]["minimum"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["items"]["properties"]["column"]["minimum"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["effects"]["items"]["properties"]
            ["locations"]["items"]["properties"]["function"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["dynamicEffects"]["type"],
        "boolean"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["dynamicReasons"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["entryPoints"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["entryPoints"]["items"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["entryPoints"]["items"]["minLength"],
        1
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["dynamicEffects"]["type"],
        "boolean"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["dynamicReasons"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["dynamicReasons"]["items"]["type"],
        "string"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["dynamicReasons"]["items"]
            ["minLength"],
        1
    );

    let check: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/check/v1.json")).expect("read check schema"),
    )
    .expect("parse check schema");
    assert_eq!(check["type"], "object");
    assert_eq!(check["additionalProperties"], true);
    assert_eq!(
        check["required"]
            .as_array()
            .expect("check required array")
            .iter()
            .map(|value| value.as_str().expect("check required string"))
            .collect::<Vec<_>>(),
        vec!["filesChecked", "errorCount", "warningCount"]
    );
    assert_eq!(check["properties"]["filesChecked"]["type"], "integer");
    assert_eq!(check["properties"]["errorCount"]["minimum"], 0);
    assert_eq!(check["properties"]["warningCount"]["minimum"], 0);

    let run: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/run/v1.json")).expect("read run schema"),
    )
    .expect("parse run schema");
    assert_eq!(run["type"], "object");
    assert_eq!(run["additionalProperties"], true);
    assert_eq!(
        run["required"]
            .as_array()
            .expect("run required array")
            .iter()
            .map(|value| value.as_str().expect("run required string"))
            .collect::<Vec<_>>(),
        vec!["exitCode", "runtimeMs"]
    );
    assert_eq!(run["properties"]["exitCode"]["type"], "integer");
    assert_eq!(run["properties"]["runtimeMs"]["type"], "integer");
    assert_eq!(run["properties"]["runtimeMs"]["minimum"], 0);
    assert_eq!(run["properties"]["threadTopology"]["type"], "object");
    assert_eq!(
        run["properties"]["threadTopology"]["additionalProperties"],
        false
    );
    assert_eq!(
        run["properties"]["threadTopology"]["required"]
            .as_array()
            .expect("run thread topology required array")
            .iter()
            .map(|value| value.as_str().expect("run thread topology required string"))
            .collect::<Vec<_>>(),
        vec!["totalInstances", "terminatedInstances", "liveInstances"]
    );
    assert_eq!(
        run["properties"]["threadTopology"]["properties"]["totalInstances"]["minimum"],
        0
    );
    assert_eq!(
        run["properties"]["threadTopology"]["properties"]["liveInstances"]["items"]["properties"]
            ["postedSharedBuffers"]["items"]["items"]["maximum"],
        255
    );

    let install: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/install/v1.json"))
            .expect("read install schema"),
    )
    .expect("parse install schema");
    assert_eq!(install["type"], "object");
    assert_eq!(install["additionalProperties"], true);
    assert_eq!(
        install["required"]
            .as_array()
            .expect("install required array")
            .iter()
            .map(|value| value.as_str().expect("install required string"))
            .collect::<Vec<_>>(),
        vec!["installed", "updated", "removed"]
    );
    for property in ["manifestPath", "lockPath"] {
        let types = install["properties"][property]["type"]
            .as_array()
            .expect("install optional path type array")
            .iter()
            .map(|value| value.as_str().expect("install path type string"))
            .collect::<Vec<_>>();
        assert_eq!(
            types,
            vec!["string", "null"],
            "unexpected {property} type set"
        );
    }
    for property in ["installed", "updated", "removed"] {
        assert_eq!(install["properties"][property]["type"], "array");
        assert_eq!(install["properties"][property]["items"]["type"], "string");
    }

    let fmt: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/fmt/v1.json")).expect("read fmt schema"),
    )
    .expect("parse fmt schema");
    assert_eq!(fmt["type"], "object");
    assert_eq!(fmt["additionalProperties"], true);
    assert_eq!(
        fmt["required"]
            .as_array()
            .expect("fmt required array")
            .iter()
            .map(|value| value.as_str().expect("fmt required string"))
            .collect::<Vec<_>>(),
        vec!["filesFormatted", "filesChecked"]
    );
    assert_eq!(fmt["properties"]["filesFormatted"]["type"], "integer");
    assert_eq!(fmt["properties"]["filesChecked"]["type"], "integer");

    let lint: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/lint/v1.json")).expect("read lint schema"),
    )
    .expect("parse lint schema");
    assert_eq!(lint["type"], "object");
    assert_eq!(lint["additionalProperties"], true);
    assert_eq!(
        lint["required"]
            .as_array()
            .expect("lint required array")
            .iter()
            .map(|value| value.as_str().expect("lint required string"))
            .collect::<Vec<_>>(),
        vec!["filesLinted", "errorCount", "warningCount", "fixedCount"]
    );
    for property in ["filesLinted", "errorCount", "warningCount", "fixedCount"] {
        assert_eq!(lint["properties"][property]["type"], "integer");
        assert_eq!(lint["properties"][property]["minimum"], 0);
    }

    let effects: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/effects/v1.json"))
            .expect("read effects schema"),
    )
    .expect("parse effects schema");
    assert_eq!(effects["title"], "Kali Effects Result v1");
    assert_eq!(
        effects["description"],
        "Native JSON payload emitted by `kali effects`."
    );
    assert_eq!(effects["type"], "object");
    assert_eq!(effects["additionalProperties"], false);
    assert_eq!(effects["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(
        required_fields(&effects),
        [
            "schemaVersion",
            "analysisContext",
            "entryPoints",
            "effects",
            "dynamicEffects",
            "dynamicReasons"
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["required"]
            .as_array()
            .expect("effects analysisContext required array")
            .iter()
            .map(|value| value
                .as_str()
                .expect("effects analysisContext required string"))
            .collect::<Vec<_>>(),
        vec!["apiSurface", "runtimeProfiles", "compatFeatures"]
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["additionalProperties"],
        false
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["apiSurface"]["minLength"],
        1
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["runtimeProfiles"]["type"],
        "array"
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["runtimeProfiles"]["items"]["type"],
        "string"
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["runtimeProfiles"]["items"]
            ["minLength"],
        1
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["compatFeatures"]["type"],
        "array"
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["compatFeatures"]["items"]["type"],
        "string"
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["compatFeatures"]["items"]
            ["minLength"],
        1
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["required"]
            .as_array()
            .expect("effects occurrence required array")
            .iter()
            .map(|value| value.as_str().expect("effects occurrence required string"))
            .collect::<Vec<_>>(),
        vec!["kind", "locations"]
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["kind"]["type"],
        "string"
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["kind"]["minLength"],
        1
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]["required"]
            .as_array()
            .expect("effects location required array")
            .iter()
            .map(|value| value.as_str().expect("effects location required string"))
            .collect::<Vec<_>>(),
        vec!["file", "line", "column"]
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]
            ["additionalProperties"],
        false
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]["properties"]
            ["file"]["type"],
        "string"
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]["properties"]
            ["file"]["minLength"],
        1
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]["required"]
            .as_array()
            .expect("effects location required array")
            .iter()
            .map(|value| value.as_str().expect("effects location required string"))
            .collect::<Vec<_>>(),
        vec!["file", "line", "column"]
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]["properties"]
            ["line"]["minimum"],
        1
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]["properties"]
            ["column"]["minimum"],
        1
    );
    assert_eq!(
        effects["properties"]["effects"]["items"]["properties"]["locations"]["items"]["properties"]
            ["function"]["type"],
        "string"
    );
    assert_eq!(effects["properties"]["entryPoints"]["type"], "array");
    assert_eq!(
        effects["properties"]["entryPoints"]["items"]["type"],
        "string"
    );
    assert_eq!(
        effects["properties"]["entryPoints"]["items"]["minLength"],
        1
    );
    assert_eq!(effects["properties"]["dynamicEffects"]["type"], "boolean");
    assert_eq!(effects["properties"]["dynamicReasons"]["type"], "array");
    assert_eq!(
        effects["properties"]["dynamicReasons"]["items"]["type"],
        "string"
    );
    assert_eq!(
        effects["properties"]["dynamicReasons"]["items"]["minLength"],
        1
    );

    let doctor: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/doctor/v1.json"))
            .expect("read doctor schema"),
    )
    .expect("parse doctor schema");
    assert_eq!(
        doctor["$id"],
        "https://kali-lang.org/schemas/result/doctor/v1"
    );
    assert_eq!(doctor["title"], "Kali Doctor Result v1");
    assert_eq!(doctor["type"], "object");
    assert_eq!(doctor["additionalProperties"], false);
    assert_eq!(
        required_fields(&doctor),
        ["browserHarness", "browserRuntimeContract"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(doctor["properties"]["browserHarness"]["type"], "object");
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]
            .as_object()
            .expect("browser harness properties")
            .len(),
        7
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["additionalProperties"],
        false
    );
    assert_eq!(
        required_fields(&doctor["properties"]["browserHarness"]),
        [
            "envVar",
            "source",
            "override",
            "command",
            "executable",
            "args",
            "executableAvailable"
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["source"]["enum"],
        serde_json::json!(["env", "auto"])
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["command"]["minItems"],
        1
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["executableAvailable"]["type"],
        "boolean"
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["envVar"],
        serde_json::json!({"type": "string", "minLength": 1})
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["source"],
        serde_json::json!({"enum": ["env", "auto"], "type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["override"],
        serde_json::json!({"type": ["string", "null"]})
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["allOf"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["allOf"][0]["if"],
        serde_json::json!({
            "properties": { "source": { "const": "env" } },
            "required": ["source"]
        })
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["allOf"][0]["then"],
        serde_json::json!({
            "properties": { "override": { "type": "string", "minLength": 1 } }
        })
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["allOf"][0]["else"],
        serde_json::json!({
            "properties": { "override": { "const": null } }
        })
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["command"],
        serde_json::json!({
            "type": "array",
            "minItems": 1,
            "items": { "type": "string", "minLength": 1 }
        })
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["executable"],
        serde_json::json!({"type": "string", "minLength": 1})
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["args"],
        serde_json::json!({
            "type": "array",
            "items": { "type": "string", "minLength": 1 }
        })
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["args"]["items"],
        serde_json::json!({"type": "string", "minLength": 1})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["type"],
        "object"
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]
            .as_object()
            .expect("browser runtime contract properties")
            .len(),
        8
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["additionalProperties"],
        false
    );
    assert_eq!(
        required_fields(&doctor["properties"]["browserRuntimeContract"]),
        [
            "hostLabel",
            "hostDescription",
            "hostDescriptionNote",
            "supportedCommands",
            "diagnosticHint",
            "summaryNote",
            "contractScopeNote",
            "diagnosticNotes"
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["hostLabel"],
        serde_json::json!({"const": "browser-requested", "type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["hostDescription"],
        serde_json::json!({"type": "string", "minLength": 1})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["hostDescriptionNote"],
        serde_json::json!({"const": "browser runtime host description: real browser host"})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["supportedCommands"],
        serde_json::json!({
            "type": "array",
            "prefixItems": [
                { "const": "run", "type": "string" },
                { "const": "test", "type": "string" }
            ],
            "items": false,
            "minItems": 2,
            "maxItems": 2,
            "uniqueItems": true
        })
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["diagnosticHint"],
        serde_json::json!({"type": "string", "minLength": 1})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["summaryNote"],
        serde_json::json!({"type": "string", "minLength": 1})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["contractScopeNote"],
        serde_json::json!({"type": "string", "minLength": 1})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["diagnosticNotes"],
        serde_json::json!({
            "type": "array",
            "minItems": 1,
            "uniqueItems": true,
            "items": { "type": "string", "minLength": 1 }
        })
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["diagnosticNotes"]["items"],
        serde_json::json!({"type": "string", "minLength": 1})
    );

    let init: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/init/v1.json")).expect("read init schema"),
    )
    .expect("parse init schema");
    assert_eq!(init["$id"], "https://kali-lang.org/schemas/result/init/v1");
    assert_eq!(init["title"], "Kali Init Result v1");
    assert_eq!(init["type"], "object");
    assert_eq!(init["additionalProperties"], false);
    assert_eq!(
        required_fields(&init),
        ["root", "manifestPath", "sourcePath", "library"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(init["properties"]["root"]["type"], "string");
    assert_eq!(init["properties"]["manifestPath"]["type"], "string");
    assert_eq!(init["properties"]["sourcePath"]["type"], "string");
    assert_eq!(init["properties"]["library"]["type"], "boolean");

    let package_audit: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/package-audit/v1.json"))
            .expect("read package-audit schema"),
    )
    .expect("parse package-audit schema");
    assert_eq!(
        package_audit["$id"],
        "https://kali-lang.org/schemas/result/package-audit/v1"
    );
    assert_eq!(package_audit["title"], "Kali Package Audit Result v1");
    assert_eq!(
        package_audit["description"],
        "Envelope-only JSON command payload for the later package-audit surface."
    );
    assert_eq!(package_audit["type"], "null");
    let package_audit_keys = package_audit
        .as_object()
        .expect("package-audit schema object")
        .keys()
        .map(|key| key.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        package_audit_keys,
        std::collections::BTreeSet::from(["$schema", "$id", "title", "type", "description",])
    );
}

#[test]
fn binding_package_metadata_schema_is_pinned() {
    let root = repo_root();
    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/artifact-meta/binding-package/v1.json"))
            .expect("read binding package schema"),
    )
    .expect("parse binding package schema");

    assert_eq!(
        schema["$id"],
        "https://kali-lang.org/schemas/artifact-meta/binding-package/v1"
    );
    assert_eq!(schema["title"], "Kali Binding Package Manifest v1");
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], true);
    assert_eq!(
        schema["required"],
        serde_json::json!([
            "schemaVersion",
            "kind",
            "moduleName",
            "hostAbiVersion",
            "artifacts"
        ])
    );
    assert_eq!(schema["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(schema["properties"]["kind"]["const"], "binding-package");
    assert_eq!(schema["properties"]["moduleName"]["type"], "string");
    assert_eq!(schema["properties"]["hostAbiVersion"]["type"], "integer");
    assert_eq!(schema["properties"]["minHostAbiVersion"]["type"], "integer");
    assert_eq!(
        schema["properties"]["maxSpecializations"]["type"],
        "integer"
    );
    assert_eq!(schema["properties"]["artifacts"]["type"], "object");
    assert_eq!(
        schema["properties"]["artifacts"]["additionalProperties"],
        true
    );
    assert_eq!(
        schema["properties"]["artifacts"]["properties"]["library"]["type"],
        "string"
    );
    assert_eq!(
        schema["properties"]["artifacts"]["properties"]["metadata"]["type"],
        "string"
    );
    assert_eq!(
        schema["properties"]["artifacts"]["properties"]["exportsHeader"]["type"],
        "string"
    );
    assert_eq!(
        schema["properties"]["artifacts"]["properties"]["glue"]["type"],
        "array"
    );
    assert_eq!(
        schema["properties"]["artifacts"]["properties"]["glue"]["items"]["type"],
        "string"
    );
    assert_eq!(
        schema["properties"]["artifacts"]["required"],
        serde_json::json!(["library", "metadata", "exportsHeader", "glue"])
    );
}
#[test]
fn readme_command_reference_tracks_the_current_cli_surface() {
    let root = repo_root();
    let readme = fs::read_to_string(root.join("README.md")).expect("read README");

    for expected in [
        "kali doctor                         # inspect local tool/environment selection",
        "kali doctor --output json           # emit the schema-v1 doctor result envelope",
        "kali init",
        "kali init --lib                 # create the minimal library scaffold",
        "kali init --output json         # emit the schema-v1 init result envelope",
        "kali init --output json --lib    # emit the library scaffold result envelope",
        "kali install",
        "kali fmt",
        "kali lint",
        "kali check [files...]",
        "kali check --api browser main.ts # browser-targeted analysis lane",
        "kali check --api browser --sandbox kali.policy.json main.ts # browser-targeted analysis lane with static sandbox validation",
        "kali build <file>",
        "kali build --validate-ir <file>  # run internal HIR/MIR/LIR validators",
        "kali build --profile pgo-profile.json main.ts # load deterministic PGO profile data",
        "kali build --bundle --api browser <file> # browser-targeted build lane",
        "kali build --bundle --api browser --sandbox kali.policy.json main.ts # browser-targeted build lane with static sandbox validation",
        "kali build --bundle --format cjs <file> # browser-targeted CommonJS browser bundle wrapper",
        "kali build --lib <file>         # base library artifact for exact-version consumers",
        "kali build --capi <file>        # stable public C-ABI embedding flow",
        "kali build --component <file>   # Component Model packaging flow",
        "kali init --output json",
        "kali init --output json --lib",
        "kali run <file> [-- args...]",
        "kali test [files...]",
        "kali test --coverage [files...]",
        "kali check --sandbox kali.policy.json main.ts",
        "kali build --bundle --sandbox kali.policy.json main.ts",
        "kali build --lib --sandbox kali.policy.json lib.ts",
        "kali build --capi --sandbox kali.policy.json lib.ts",
        "kali build --component --sandbox kali.policy.json lib.ts",
        "kali build --lib --api browser lib.ts",
        "kali build --lib --api browser --sandbox kali.policy.json lib.ts",
        "kali build --capi --api browser lib.ts",
        "kali build --capi --api browser --sandbox kali.policy.json lib.ts",
        "kali build --component --api browser lib.ts",
        "kali build --component --api browser --sandbox kali.policy.json lib.ts",
        "kali effects <file>",
        "kali effects --output json main.ts",
        "kali package-effects <package>",
        "kali package-effects --output json lodash",
        "kali package-effects jsr:@std/path",
        "kali package-effects --pretty lodash",
        "kali package-effects --pretty browserpkg",
        "kali package-effects --pretty --output json lodash",
        "kali package-audit <package>",
        "kali package-audit --output json lodash",
        "kali package-audit jsr:@std/path",
        "kali package-audit --pretty lodash",
        "kali package-audit --pretty --output json lodash",
    ] {
        assert!(
            readme.contains(expected),
            "README is missing CLI example: {expected}"
        );
    }

    assert!(
        readme.contains("[`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)"),
        "README should point readers to the feature-maturity matrix for availability"
    );
    assert!(
        readme.contains("[`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md)"),
        "README should point readers to the proof boundary manifest for verification wording"
    );
}

#[test]
fn feature_maturity_current_repository_snapshot_tracks_the_live_surface() {
    let root = repo_root();
    let maturity = fs::read_to_string(root.join("specs/19-feature-maturity.md"))
        .expect("read feature maturity matrix");

    for expected in [
        "Current repository snapshot",
        "| Public effect reporting | `kali effects`, `kali package-effects`, and the policy-comparison half on `check/build --sandbox` are implemented",
        "| Embedding surface | `kali build --lib` now emits the stable WIT-sidecar form",
        "| Coverage reporting | `kali test --coverage` ships the stable deterministic function-coverage contract",
        "| Browser runtime nuance | browser-runtime harness/helper work exists",
        "| Browser package deployability | the browser package corpus now also mirrors the shared exports-map package set onto `.js` input for `check` and `build --bundle`",
        "| Node compatibility breadth | `run` / `test` remain live on the documented Node execution subset",
        "| Threaded runtime profile | `run` / `test` now accept the explicit `--wasm-threads` opt-in",
        "| `kali build --component --api browser lib.ts` | Rejected by default |",
        "| plain `kali build --component lib.ts` under an inherited browser API surface | Rejected by default |",
    ] {
        assert!(
            maturity.contains(expected),
            "specs/19-feature-maturity.md is missing maturity snapshot marker: {expected}"
        );
    }
}

#[test]
fn cli_spec_examples_track_the_current_repository_surface() {
    let root = repo_root();
    let cli_spec = fs::read_to_string(root.join("specs/12-cli.md")).expect("read CLI spec");

    for expected in [
        "Status: Phase 2 target. This section documents a **defined command family** in schema v1;",
        "### `kali effects <file>`",
        "kali effects main.ts",
        "kali effects --api node main.ts",
        "kali effects --output json main.ts",
        "Status: **Phase 2 target**. This section documents a **defined command family** in schema v1;",
        "### `kali package-effects <package>`",
        "kali package-effects lodash",
        "kali package-effects --output json lodash",
        "kali package-effects --pretty lodash",
        "kali package-effects --pretty browserpkg",
        "Status: **Phase 4 compatibility**. This section also documents a **defined command family** in schema v1;",
        "### `kali package-audit <package>`",
        "kali package-audit lodash",
        "kali package-audit --output json lodash",
        "kali package-audit --pretty --output json lodash",
        "kali run --api browser main.ts",
        "kali test --api browser",
        "kali build --bundle --api browser main.ts",
        "kali build --api node main.ts",
        "kali build --sandbox kali.policy.json main.ts",
        "kali build --bundle --api node main.ts",
        "kali build --bundle --api browser --sandbox kali.policy.json main.ts",
        "kali build --profile pgo-profile.json main.ts # Load deterministic PGO profile data and record its normalized hash in build metadata sidecars and JSON output",
        "kali build --lib --sandbox kali.policy.json lib.ts",
        "kali build --capi --sandbox kali.policy.json lib.ts",
        "kali build --component --sandbox kali.policy.json lib.ts",
        "kali build --lib --api browser --sandbox kali.policy.json lib.ts",
        "kali build --capi --api browser --sandbox kali.policy.json lib.ts",
        "kali build --capi lib.ts",
        "kali build --component lib.ts",
    ] {
        assert!(
            cli_spec.contains(expected),
            "specs/12-cli.md is missing CLI example or contract marker: {expected}"
        );
    }

    assert!(
        cli_spec.contains("The browser runtime contract `hostLabel`, `hostDescription`, `hostDescriptionNote`, `summaryNote`, `contractScopeNote`, and `diagnosticHint` are compared against their canonical strings after trimming surrounding whitespace, and the `supportedCommands` / `diagnosticNotes` items are compared after trimming too, but whitespace-only values still fail validation and duplicates remain forbidden."),
        "specs/12-cli.md should describe browser runtime contract trimming alongside the array-item rules"
    );
}

#[test]
fn browser_runtime_contract_docs_track_trimmed_validation_rules() {
    let root = repo_root();
    let docs = [
        (
            "specs/18-schemas.md",
            [
                "hostLabel: const",
                "`hostLabel: const` `\"browser-requested\"` — canonical label for the browser-requested host contract used in diagnostics; validators compare the emitted value after trimming surrounding whitespace, but whitespace-only values still fail. This matches the CLI validator's trim-on-compare rule for the browser runtime contract.",
                "hostDescription: non-empty, non-whitespace string",
                "`hostDescription: non-empty, non-whitespace string` — short human-readable description of the intended browser host; validators compare the emitted value after trimming surrounding whitespace, but whitespace-only values still fail. This matches the CLI validator's trim-on-compare rule for the browser runtime contract.",
                "hostDescriptionNote: const",
                "supportedCommands: string[]",
                "diagnosticHint: non-empty, non-whitespace string",
                "diagnosticNotes: string[]",
                "validators compare the emitted value after trimming surrounding whitespace, but whitespace-only values still fail",
                "validators compare each emitted note after trimming surrounding whitespace, but whitespace-only notes still fail",
            ]
            .as_slice(),
        ),
        (
            "specs/12-cli.md",
            [
                "The browser runtime contract `hostLabel`, `hostDescription`, `hostDescriptionNote`, `summaryNote`, `contractScopeNote`, and `diagnosticHint` are compared against their canonical strings after trimming surrounding whitespace, and the `supportedCommands` / `diagnosticNotes` items are compared after trimming too, but whitespace-only values still fail validation and duplicates remain forbidden.",
                "Each `threadTopology.liveInstances[].scriptUrl` is a canonical absolute URL string, not a relative or whitespace-padded spelling.",
            ]
            .as_slice(),
        ),
    ];

    for (doc, expectations) in docs {
        let contents = fs::read_to_string(root.join(doc)).expect("read browser runtime docs");
        for expected in expectations {
            assert!(
                contents.contains(expected),
                "{doc} is missing browser runtime contract marker: {expected}"
            );
        }
    }
}

#[test]
fn run_test_result_provenance_docs_track_canonical_thread_topology_script_urls() {
    let root = repo_root();
    let docs = ["specs/18-schemas.md", "specs/12-cli.md"];

    for doc in docs {
        let contents = fs::read_to_string(root.join(doc)).expect("read run/test provenance docs");
        assert!(
            contents.contains(
                "canonical absolute URL string, not a relative or whitespace-padded spelling"
            ),
            "{doc} should describe the canonical threadTopology scriptUrl form"
        );
    }
}

#[test]
fn benchmark_fixture_metadata_schema_tracks_current_fixture_contract() {
    let root = repo_root();
    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/benchmark/v1.json")).expect("read benchmark schema"),
    )
    .expect("parse benchmark schema");

    assert_eq!(schema["title"], "Kali Benchmark Fixture Metadata v1");
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        required_fields(&schema),
        [
            "benchmark",
            "version",
            "sourceFile",
            "sourceSha256",
            "buildModes"
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(schema["properties"]["benchmark"]["type"], "string");
    assert_eq!(schema["properties"]["benchmark"]["pattern"], "^[a-z0-9-]+$");
    assert_eq!(schema["properties"]["version"]["const"], 1);
    assert_eq!(schema["properties"]["sourceFile"]["type"], "string");
    assert_eq!(
        schema["properties"]["sourceFile"]["pattern"],
        "^[a-z0-9-]+-benchmark-v1\\.(?:ts|js)$"
    );
    assert_eq!(schema["properties"]["sourceSha256"]["type"], "string");
    assert_eq!(
        schema["properties"]["sourceSha256"]["pattern"],
        "^sha256-[0-9a-f]{64}$"
    );
    assert_eq!(schema["properties"]["buildModes"]["type"], "array");
    assert_eq!(
        schema["properties"]["buildModes"]["prefixItems"]
            .as_array()
            .expect("buildModes prefixItems array")
            .iter()
            .map(|item| item["const"]
                .as_str()
                .expect("buildModes prefixItem const")
                .to_owned())
            .collect::<Vec<_>>(),
        vec!["--fast", "--release", "--release-advanced"]
    );
    assert_eq!(schema["properties"]["buildModes"]["items"], false);
    assert_eq!(schema["properties"]["buildModes"]["minItems"], 3);
    assert_eq!(schema["properties"]["buildModes"]["maxItems"], 3);

    let mut benchmark_names = BTreeSet::new();
    let mut benchmark_sources = BTreeSet::new();

    let expected_benchmark_names: BTreeSet<String> = [
        "folded-arithmetic",
        "folded-arithmetic-js",
        "math-trunc-builtin",
        "math-trunc-builtin-js",
        "math-imul-builtin",
        "math-imul-builtin-js",
        "math-clz32-builtin",
        "math-clz32-builtin-js",
        "math-ceil-builtin",
        "math-ceil-builtin-js",
        "math-abs-sign-builtin",
        "math-abs-sign-builtin-js",
        "math-max-min-builtin",
        "math-max-min-builtin-js",
        "math-floor-builtin",
        "math-floor-builtin-js",
        "math-round-builtin",
        "math-round-builtin-js",
        "math-pow-builtin",
        "math-pow-builtin-js",
        "division-by-one-elimination",
        "multiplication-by-one-elimination",
        "dead-branch-elimination",
        "dead-inlined-function-pruning",
        "division-and-identity",
        "closure-inlining-and-folding",
        "object-enumeration-folding",
        "object-string-enumeration-folding",
        "reflect-own-keys-folding",
        "reflect-own-keys-const-bound-literal",
        "reflect-own-keys-alias-chain",
        "integer-like-object-enumeration-folding",
        "object-enumeration-alias-chain",
        "object-enumeration-const-bound-literal",
        "object-enumeration-delete-reinsert",
        "object-literal-property-order-canonicalization",
        "object-literal-property-order-canonicalization-js",
        "identity-chain-and-simplification",
        "nested-wrapper-pruning",
        "algebraic-simplification",
        "duplicate-pure-expression-elimination",
        "nullish-specialization-repeat",
        "specialization-reuse",
        "bigint-literal-arguments",
        "bigint-addition-chain",
        "bigint-multiplication-chain",
        "numeric-literal-arguments",
        "boolean-literal-arguments",
        "branch-specialization-repeat",
        "const-array-element-access",
        "const-object-property-access",
        "folded-arithmetic-variant",
        "folded-arithmetic-variant-js",
        "string-concatenation",
        "array-literal-arguments",
        "template-literal-concatenation",
        "template-literal-concatenation-js",
        "layout-specialization",
        "call-inlining-chain",
        "nested-call-inlining-chain",
        "object-enumeration-alias-chain-js",
        "nullish-specialization",
        "fannkuch-redux",
        "spectral-norm",
        "nbody",
        "mandelbrot",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect();
    let expected_benchmark_sources: BTreeSet<String> = [
        "math-benchmark-v1.ts",
        "math-benchmark-v1-js.js",
        "math-trunc-benchmark-v1.ts",
        "math-trunc-benchmark-v1-js.js",
        "math-imul-benchmark-v1.ts",
        "math-imul-benchmark-v1-js.js",
        "math-clz32-benchmark-v1.ts",
        "math-clz32-benchmark-v1-js.js",
        "math-ceil-benchmark-v1.ts",
        "math-ceil-benchmark-v1-js.js",
        "math-abs-sign-benchmark-v1.ts",
        "math-abs-sign-benchmark-v1-js.js",
        "math-max-min-benchmark-v1.ts",
        "math-max-min-benchmark-v1-js.js",
        "math-floor-benchmark-v1.ts",
        "math-floor-benchmark-v1-js.js",
        "math-round-benchmark-v1.ts",
        "math-round-benchmark-v1-js.js",
        "math-pow-benchmark-v1.ts",
        "math-pow-benchmark-v1-js.js",
        "division-by-one-benchmark-v1.ts",
        "multiplication-by-one-benchmark-v1.ts",
        "dead-branch-elimination-benchmark-v1.ts",
        "dead-inlined-function-pruning-benchmark-v1.ts",
        "call-inlining-benchmark-v1.ts",
        "closure-inlining-benchmark-v1.ts",
        "nested-call-inlining-chain-benchmark-v1.ts",
        "object-enumeration-benchmark-v1.ts",
        "object-string-enumeration-benchmark-v1.ts",
        "reflect-own-keys-benchmark-v1.ts",
        "reflect-own-keys-const-bound-literal-benchmark-v1.ts",
        "reflect-own-keys-alias-chain-benchmark-v1.ts",
        "integer-like-object-enumeration-benchmark-v1.ts",
        "object-enumeration-alias-chain-benchmark-v1.ts",
        "object-enumeration-alias-chain-benchmark-v1-js.js",
        "object-enumeration-const-bound-literal-benchmark-v1.ts",
        "object-enumeration-delete-reinsert-benchmark-v1.ts",
        "object-literal-property-order-canonicalization-benchmark-v1.ts",
        "object-literal-property-order-canonicalization-benchmark-v1-js.js",
        "identity-chain-benchmark-v1.ts",
        "nested-wrapper-pruning-benchmark-v1.ts",
        "algebraic-simplification-benchmark-v1.ts",
        "duplicate-pure-expression-elimination-benchmark-v1.ts",
        "nullish-specialization-repeat-benchmark-v1.ts",
        "specialization-reuse-benchmark-v1.ts",
        "bigint-literal-arguments-benchmark-v1.ts",
        "bigint-addition-chain-benchmark-v1.ts",
        "bigint-multiplication-chain-benchmark-v1.ts",
        "numeric-literal-arguments-benchmark-v1.ts",
        "boolean-literal-arguments-benchmark-v1.ts",
        "branch-specialization-repeat-benchmark-v1.ts",
        "const-array-element-access-benchmark-v1.ts",
        "const-object-property-access-benchmark-v1.ts",
        "math-variant-benchmark-v1.ts",
        "math-variant-benchmark-v1-js.js",
        "string-concatenation-benchmark-v1.ts",
        "array-literal-arguments-benchmark-v1.js",
        "template-literal-concatenation-benchmark-v1.ts",
        "template-literal-concatenation-benchmark-v1-js.js",
        "layout-specialization-benchmark-v1.ts",
        "call-inlining-chain-benchmark-v1.ts",
        "nullish-benchmark-v1.ts",
        "fannkuch-redux-benchmark-v1.ts",
        "spectral-norm-benchmark-v1.ts",
        "nbody-benchmark-v1.ts",
        "mandelbrot-benchmark-v1.ts",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect();

    let mut benchmark_entries: Vec<_> =
        fs::read_dir(root.join("crates/kali_cli/tests/fixtures/benchmarks"))
            .expect("read benchmark fixture directory")
            .map(|entry| entry.expect("benchmark fixture entry").path())
            .collect();
    benchmark_entries.sort();

    for path in benchmark_entries {
        let file_name = path.file_name().and_then(|name| name.to_str());
        if file_name.is_some_and(|name| name.ends_with(".policy.json")) {
            // Scoped `--sandbox` policy fixtures (e.g. mandelbrot-benchmark-v1.policy.json)
            // sit alongside benchmark metadata but follow a different schema; they are
            // reusable sandbox policies, not benchmark metadata files.
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&path).expect("read benchmark metadata fixture"),
        )
        .expect("parse benchmark metadata fixture");
        let metadata_object = metadata.as_object().expect("benchmark metadata object");
        assert_eq!(metadata_object.len(), 5, "{}", path.display());
        for expected_key in [
            "benchmark",
            "version",
            "sourceFile",
            "sourceSha256",
            "buildModes",
        ] {
            assert!(
                metadata_object.contains_key(expected_key),
                "{} missing expected key {}",
                path.display(),
                expected_key
            );
        }
        let benchmark_name = metadata["benchmark"]
            .as_str()
            .expect("benchmark string")
            .to_owned();
        assert!(
            benchmark_names.insert(benchmark_name.clone()),
            "duplicate benchmark slug: {} ({})",
            benchmark_name,
            path.display()
        );
        assert_eq!(metadata["version"], 1, "{}", path.display());
        let benchmark_stem = path.file_stem().expect("benchmark stem").to_string_lossy();
        let expected_source_file_js = format!("{}.js", benchmark_stem);
        let expected_source_file_ts = format!("{}.ts", benchmark_stem);
        assert!(
            metadata["sourceFile"] == serde_json::json!(expected_source_file_ts)
                || metadata["sourceFile"] == serde_json::json!(expected_source_file_js),
            "{}",
            path.display()
        );
        assert_eq!(
            metadata["buildModes"],
            serde_json::json!(["--fast", "--release", "--release-advanced"]),
            "{}",
            path.display()
        );
        let source_file_name = metadata["sourceFile"].as_str().expect("sourceFile string");
        assert!(
            benchmark_sources.insert(source_file_name.to_owned()),
            "duplicate benchmark source file: {} ({})",
            source_file_name,
            path.display()
        );
        let source_path = path
            .parent()
            .expect("benchmark metadata parent")
            .join(source_file_name);
        let source = fs::read_to_string(&source_path).expect("read benchmark source fixture");
        let source_hash = format!("sha256-{:x}", Sha256::digest(source.as_bytes()));
        assert_eq!(metadata["sourceSha256"], source_hash, "{}", path.display());
    }

    assert_eq!(
        benchmark_names, expected_benchmark_names,
        "benchmark slugs should match the checked-in benchmark corpus"
    );
    assert_eq!(
        benchmark_sources, expected_benchmark_sources,
        "benchmark source files should match the checked-in benchmark corpus"
    );
}
