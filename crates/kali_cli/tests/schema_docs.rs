use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn required_fields(schema: &serde_json::Value) -> Vec<String> {
    schema["required"]
        .as_array()
        .expect("required array")
        .iter()
        .map(|value| value.as_str().expect("required string").to_owned())
        .collect()
}

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
    assert_eq!(envelope["additionalProperties"], true);
    assert_eq!(envelope["properties"]["schemaVersion"]["const"], 1);
    assert_eq!(envelope["properties"]["timings"]["type"], "array");
    assert_eq!(envelope["properties"]["timings"]["items"]["type"], "object");
    assert_eq!(
        envelope["properties"]["timings"]["items"]["additionalProperties"],
        true
    );
    assert_eq!(
        required_fields(&envelope["properties"]["timings"]["items"]),
        ["phase", "milliseconds"]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        envelope["properties"]["timings"]["items"]["properties"]["phase"]["type"],
        "string"
    );
    assert_eq!(
        envelope["properties"]["timings"]["items"]["properties"]["milliseconds"]["type"],
        "number"
    );
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
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );

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
        (0, vec!["profileDataHash"]),
        (1, vec!["witPath", "profileDataHash"]),
        (2, vec!["profileDataHash"]),
        (3, vec!["witPath", "profileDataHash"]),
        (4, vec!["witPath", "bindingPackagePath", "profileDataHash"]),
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

    for variant_index in [0, 2] {
        assert_eq!(
            build_variants[variant_index]["properties"]["profileDataHash"]["type"],
            "string"
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
        test_result["properties"]["coverage"]["properties"]["files"]["items"]
            ["additionalProperties"],
        true
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
        test_result["properties"]["coverage"]["properties"]["summary"]["additionalProperties"],
        true
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
    assert_eq!(artifact_meta["properties"]["entrypoint"]["type"], "string");
    assert_eq!(artifact_meta["properties"]["buildMode"]["type"], "string");
    assert_eq!(
        artifact_meta["properties"]["buildMode"]["enum"],
        serde_json::json!(["fast", "release", "release-advanced"])
    );
    assert_eq!(artifact_meta["properties"]["apiSurface"]["type"], "string");
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
        artifact_meta["properties"]["exports"]["items"]["properties"]["name"]["type"],
        "string"
    );
    assert_eq!(
        artifact_meta["properties"]["exports"]["items"]["properties"]["signature"]["type"],
        "string"
    );

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
        vec!["error", "warning", "info"]
    );
    assert_eq!(
        diagnostic["properties"]["code"]["pattern"],
        "^[EWI][0-9]{4}$"
    );
    assert_eq!(
        diagnostic["properties"]["help"],
        serde_json::json!({"type": ["string", "null"]})
    );
    assert_eq!(
        diagnostic["properties"]["span"]["required"]
            .as_array()
            .expect("span required array")
            .iter()
            .map(|value| value.as_str().expect("span required string"))
            .collect::<Vec<_>>(),
        vec!["file", "line", "column", "endLine", "endColumn"]
    );
    assert_eq!(
        diagnostic["properties"]["span"]["additionalProperties"],
        false
    );
    assert_eq!(
        diagnostic["properties"]["labels"]["items"]["required"]
            .as_array()
            .expect("label required array")
            .iter()
            .map(|value| value.as_str().expect("label required string"))
            .collect::<Vec<_>>(),
        vec!["file", "line", "column", "endLine", "endColumn"]
    );
    for key in ["line", "column", "endLine", "endColumn"] {
        assert_eq!(
            diagnostic["properties"]["span"]["properties"][key]["minimum"],
            1
        );
        assert_eq!(
            diagnostic["properties"]["labels"]["items"]["properties"][key]["minimum"],
            1
        );
    }
    for key in ["line", "column"] {
        assert_eq!(
            diagnostic["properties"]["related"]["items"]["properties"][key]["minimum"],
            1
        );
    }
    assert!(diagnostic["properties"]["context"].is_object());

    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/manifest/v1.json")).expect("read manifest schema"),
    )
    .expect("parse manifest schema");
    assert_eq!(manifest["type"], "object");
    assert_eq!(manifest["properties"]["schemaVersion"]["const"], 1);
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
    assert_eq!(lock["type"], "object");
    assert_eq!(lock["properties"]["version"]["const"], 1);
    assert_eq!(
        lock["required"]
            .as_array()
            .expect("lock required array")
            .iter()
            .map(|value| value.as_str().expect("lock required string"))
            .collect::<Vec<_>>(),
        vec!["version"]
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
    assert_eq!(policy["type"], "object");
    assert_eq!(policy["properties"]["schemaVersion"]["const"], 1);
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
            ["compatFeatures"]["type"],
        "array"
    );
    assert_eq!(
        package_effects["properties"]["report"]["properties"]["analysisContext"]["properties"]
            ["compatFeatures"]["items"]["type"],
        "string"
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
        effects["properties"]["analysisContext"]["properties"]["runtimeProfiles"]["type"],
        "array"
    );
    assert_eq!(
        effects["properties"]["analysisContext"]["properties"]["runtimeProfiles"]["items"]["type"],
        "string"
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
        effects["properties"]["effects"]["items"]["required"]
            .as_array()
            .expect("effects occurrence required array")
            .iter()
            .map(|value| value.as_str().expect("effects occurrence required string"))
            .collect::<Vec<_>>(),
        vec!["kind", "locations"]
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
            ["function"]["type"],
        "string"
    );
    assert_eq!(effects["properties"]["entryPoints"]["type"], "array");
    assert_eq!(
        effects["properties"]["entryPoints"]["items"]["type"],
        "string"
    );
    assert_eq!(effects["properties"]["dynamicEffects"]["type"], "boolean");
    assert_eq!(effects["properties"]["dynamicReasons"]["type"], "array");
    assert_eq!(
        effects["properties"]["dynamicReasons"]["items"]["type"],
        "string"
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
        serde_json::json!({"type": "string"})
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
        doctor["properties"]["browserHarness"]["properties"]["command"],
        serde_json::json!({
            "type": "array",
            "minItems": 1,
            "items": { "type": "string" }
        })
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["executable"],
        serde_json::json!({"type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["args"],
        serde_json::json!({
            "type": "array",
            "items": { "type": "string" }
        })
    );
    assert_eq!(
        doctor["properties"]["browserHarness"]["properties"]["args"]["items"],
        serde_json::json!({"type": "string"})
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
            "diagnosticNotes"
        ]
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["hostLabel"],
        serde_json::json!({"type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["hostDescription"],
        serde_json::json!({"type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["hostDescriptionNote"],
        serde_json::json!({"type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["supportedCommands"],
        serde_json::json!({
            "type": "array",
            "minItems": 1,
            "items": { "type": "string" }
        })
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["supportedCommands"]["items"],
        serde_json::json!({"type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["diagnosticHint"],
        serde_json::json!({"type": "string"})
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["diagnosticNotes"],
        serde_json::json!({
            "type": "array",
            "minItems": 1,
            "items": { "type": "string" }
        })
    );
    assert_eq!(
        doctor["properties"]["browserRuntimeContract"]["properties"]["diagnosticNotes"]["items"],
        serde_json::json!({"type": "string"})
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
fn specialized_artifact_metadata_schemas_share_the_base_artifact_contract() {
    let root = repo_root();

    let expected = [
        (
            "schemas/artifact-meta/lib-wit/v1.json",
            "Kali Library WIT Artifact Metadata v1",
        ),
        (
            "schemas/artifact-meta/capi/v1.json",
            "Kali C ABI Artifact Metadata v1",
        ),
        (
            "schemas/artifact-meta/component/v1.json",
            "Kali Component Artifact Metadata v1",
        ),
    ];

    for (relative, title) in expected {
        let schema: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(relative)).expect("read specialized artifact schema"),
        )
        .expect("parse specialized artifact schema");
        assert_eq!(schema["title"], title);
        assert_eq!(
            schema["description"],
            "Reserved schema shape for a later embedding projection."
        );
        assert_eq!(schema["allOf"].as_array().expect("allOf array").len(), 1);
        assert_eq!(
            schema["allOf"][0]["$ref"],
            "https://kali-lang.org/schemas/artifact-meta/v1"
        );
    }
}

#[test]
fn binding_package_metadata_schema_is_pinned() {
    let root = repo_root();
    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/artifact-meta/binding-package/v1.json"))
            .expect("read binding package schema"),
    )
    .expect("parse binding package schema");

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
fn proof_boundary_summary_docs_reference_the_canonical_boundary() {
    let root = repo_root();
    let summary =
        "Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.";
    let proof_backed_phrase = "proof-backed for the published boundary";

    let summary_docs = [
        ("README.md", summary),
        ("specs/16-testing.md", summary),
        ("proofs/BOUNDARY.md", summary),
        ("specs/17-verification.md", summary),
        ("specs/19-feature-maturity.md", proof_backed_phrase),
        ("plan/phase-4/02-formal-verification-depth.md", summary),
    ];

    for (relative, expected_summary) in summary_docs {
        let text = fs::read_to_string(root.join(relative)).expect("read summary doc");
        assert!(
            text.contains(expected_summary),
            "{relative} is missing the canonical proof-backed summary"
        );
    }

    let stage_doc = fs::read_to_string(root.join("plan/phase-4/02-formal-verification-depth.md"))
        .expect("read stage 4.2 doc");
    assert!(
        stage_doc.contains("[`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md)"),
        "stage 4.2 doc should reference the canonical proof boundary"
    );
    assert!(
        stage_doc.contains("authoritative source") || stage_doc.contains("historical"),
        "stage 4.2 doc should describe proofs/BOUNDARY.md as the canonical boundary source"
    );

    let boundary = fs::read_to_string(root.join("proofs/BOUNDARY.md")).expect("read boundary");
    assert!(
        boundary.contains("Status: **proof-backed proof-boundary manifest**."),
        "proof boundary manifest should declare the proof-backed state"
    );
    assert!(
        boundary.contains("| proof-ready | **yes**"),
        "proof boundary manifest should continue to claim proof-ready status"
    );
    assert!(
        boundary.contains("| proof-backed | **yes**"),
        "proof boundary manifest should now claim proof-backed status"
    );
}

fn collect_proof_sources(root: &Path) -> BTreeSet<String> {
    fn visit(dir: &Path, root: &Path, files: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).expect("read proof directory") {
            let entry = entry.expect("read proof directory entry");
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            if path.is_dir() {
                if name == ".lake" || name == "build" {
                    continue;
                }
                visit(&path, root, files);
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) == Some("lean") {
                let relative = path
                    .strip_prefix(root)
                    .expect("proof source under root")
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(format!("proofs/{relative}"));
            }
        }
    }

    let mut files = BTreeSet::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn phase_six_conformance_dashboard_is_present_and_deterministic() {
    let root = repo_root();
    let dashboard = fs::read_to_string(root.join("plan/phase-6/conformance-dashboard.md"))
        .expect("read conformance dashboard");

    for expected in [
        "# Phase 6 Conformance Dashboard",
        "## Supported today",
        "## Gated for later phases",
        "## Rejected by default",
        "Latest published ECMA-262 lexical grammar (tokenization)",
        "Current-edition non-Annex-B semantics for features Kali marks as supported in a given command/profile",
        "Static ESM `import` / `export`",
        "Generator function declarations / expressions and `yield` / `yield*` expressions",
        "First-class JavaScript compilation with bounded inference",
        "TypeScript assertion / satisfies expressions (`as`, `satisfies`)",
        "Budgeted local/intra-module constraint solving inside the shared bounded inference contract",
        "CommonJS module lowering",
        "`require(\"literal\")`",
        "Basic `Math.sign()` built-in semantics",
        "Basic nested Math call composition in `.js` input",
        "Basic `Math.trunc()` built-in semantics",
        "Basic `Math.ceil()` built-in semantics",
        "Basic `Math.clz32()` built-in semantics",
        "Basic `Object.keys()` enumeration semantics, including overwrite/delete-reinsert ordering",
        "Basic `Object.entries()` enumeration semantics, including overwrite/delete-reinsert ordering",
        "Basic `Object.values()` enumeration semantics, including overwrite/delete-reinsert ordering",
        "Basic `Object.keys()` / `Object.entries()` / `Object.values()` string-primitive enumeration semantics in `.js` input",
        "Basic unary prefix semantics (`!`, unary `-`, unary `+`, and `void`) in `.ts` and `.js` input, including JSON-output coverage on the standalone `run` / `test` lanes",
        "Basic object property deletion / `in`-operator semantics in `.ts` and `.js` input",
        "Runtime object type / constructor semantics (`typeof`, `instanceof`) in `.ts` and `.js` input",
        "Browser-requested `run` / `test` runtime object type / constructor semantics (`typeof`, `instanceof`) in `.ts` and `.js` input, including JSON-output coverage",
        "Browser bundle runtime object type / constructor semantics (`typeof`, `instanceof`) in `.ts` and `.js` input",
        "Basic queueMicrotask ordering semantics in `.js` input",
        "Browser-requested `run` / `test` object-enumeration semantics in `.ts` and `.js` input, including overwrite/delete-reinsert ordering plus string-primitive enumeration in `.js` input and JSON-output coverage",
        "Browser bundle `queueMicrotask` ordering in `.ts` and `.js` input, including JSON-output coverage",
        "Browser timing baseline / `performance.now()` monotonic ordering in `.ts` and `.js` input",
        "Browser bundle integer-like key ordering semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` integer-like key ordering semantics in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input",
        "Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input under the configured browser harness",
        "Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input, including JSON-output coverage",
        "Shared web-baseline primitives (`structuredClone`, `AbortController`, `AbortSignal`, `Event`, `EventTarget`, `CustomEvent`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`) in `.ts` and `.js` input",
        "Browser ambient typing for baseline host globals (`fetch`, `Headers`, `Request`, `Response`, `Blob`, `File`, `performance`, `crypto`) in `.ts` and `.js` input",
        "Read-only `Deno.permissions.query(...)` const-bound descriptor aliases in `.ts` and `.js` input",
        "Browser-requested `run` / `test` shared web-baseline primitives (`structuredClone`, `AbortController`, `AbortSignal`, `Event`, `EventTarget`, `CustomEvent`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`) in `.ts` and `.js` input, including inherited-browser-api-surface coverage and JSON-output coverage",
        "Browser-requested `run` / `test` shared web-baseline primitives (`structuredClone`, `AbortController`, `AbortSignal`, `Event`, `EventTarget`, `CustomEvent`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`) when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage",
        "Browser bundle shared web-baseline primitives (`structuredClone`, `AbortController`, `AbortSignal`, `Event`, `EventTarget`, `CustomEvent`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`) in `.ts` and `.js` input",
        "Browser-requested `run` / `test` basic try/catch exception handling and try/finally sequencing in `.ts` and `.js` input",
        "Browser-requested `run` / `test` basic try/catch exception handling and try/finally sequencing when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage",
        "Browser bundle basic object property deletion / `in`-operator semantics in `.ts` and `.js` input",
        "Browser bundle `try/catch` exception semantics in `.ts` and `.js` input",
        "Proxy.revocable",
        "Browser-requested `run` / `test` Web Crypto randomness subset via `crypto.getRandomValues()` in `.ts` and `.js` input",
        "Browser-requested `run` / `test` Web Crypto `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` zero-capable budget pair (`--max-threads 0` / `--max-spawned-processes 0`) in `.js` input",
        "Browser bundle `crypto.getRandomValues()` plus `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` queueMicrotask ordering in `.ts` and `.js` input, including JSON-output coverage",
        "Read-only `Deno.pid` / `Deno[\"pid\"]` / `globalThis.Deno.pid` handling in `.js` input",
        "Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.ceil()` semantics in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.ceil()` semantics when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.ceil()` semantics when the browser API surface is inherited in `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` object-enumeration semantics in `.ts` and `.js` input, including overwrite/delete-reinsert ordering plus string-primitive enumeration in `.js` input and JSON-output coverage",
        "Browser-requested `run` / `test` object-enumeration semantics when the browser API surface is inherited in `.ts` and `.js` input, including overwrite/delete-reinsert ordering plus string-primitive enumeration in `.js` input",
        "Browser-requested `run` / `test` basic object property deletion / `in`-operator semantics in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` integer-like key ordering semantics when the browser API surface is inherited in `.ts` and `.js` input",
        "Browser-requested `run` / `test` `Math.clz32()` semantics in `.ts` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.clz32()` semantics when the browser API surface is inherited in `.ts` input, including JSON-output coverage",
        "Browser-requested `run` `Math.clz32()` semantics in `.js` input, including JSON-output coverage",
        "Browser-requested `test` `Math.clz32()` semantics in `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.clz32()` semantics when the browser API surface is inherited in `.js` input",
        "Browser bundle `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` / `Math.clz32()` semantics in `.ts` and `.js` input",
        "Browser bundle `Math.ceil()` semantics in `.ts` and `.js` input, including JSON-output coverage in both source classes",
        "Browser bundle `Math.trunc()` semantics in `.ts` and `.js` input",
        "Browser bundle `async/await` sequencing in `.ts` and `.js` input, including JSON-output coverage",
        "Browser bundle `queueMicrotask` ordering in `.ts` and `.js` input, including JSON-output coverage",
        "Browser bundle basic strict equality / inequality semantics in `.ts` and `.js` input",
        "Browser bundle basic boolean conjunction / disjunction semantics in `.ts` and `.js` input",
        "Browser bundle `try/finally` sequencing in `.ts` and `.js` input",
        "Browser-requested `run` / `test` `Math.clz32()` semantics in `.ts` input, including JSON-output coverage",
        "Browser-requested `run` `Math.clz32()` semantics in `.js` input, including JSON-output coverage",
        "Browser-requested `test` `Math.clz32()` semantics in `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.trunc()` semantics in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.trunc()` semantics when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.trunc()` semantics in `.js` input, including JSON-output coverage",
        "Browser-requested `run` / `test` basic strict equality / inequality semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` boolean conjunction / disjunction semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` boolean conjunction / disjunction semantics when the browser API surface is inherited in `.js` input",
        "Browser bundle `Object.keys()` / `Object.entries()` / `Object.values()` enumeration semantics in `.ts` and `.js` input, including string-primitive enumeration and overwrite/delete-reinsert ordering, plus JSON-output coverage for string-primitive enumeration and overwrite/delete-reinsert ordering",
        "Browser-requested `run` / `test` console error / warn / info / debug routing plus `console.assert()` false-branch reporting in `.js` input",
        "Browser bundle console error / warn / info / debug routing plus `console.assert()` false-branch reporting",
        "Process identity and process-control/working-directory APIs (`process.pid`, `Deno.exit`, `Deno.cwd`, `Deno.chdir`, `process.chdir`, `process.exit`; read-only `Deno.pid` / `globalThis.Deno.pid` / bracketed `globalThis[\"Deno\"][\"pid\"]` stays the default standalone exception)",
        "Basic optional chaining member and element access",
        "Basic BigInt addition semantics",
        "Basic BigInt addition semantics in `.js` input",
        "Open-ended or unstable cross-module/public-API constraint solving",
        "Literal-string `import()`",
        "Directory-index dynamic-import targets in `.tsx` and `.jsx` input",
        "Unsupported `Deno.permissions.query(...)` descriptor kinds such as `ffi` / `sys`",
        "Environment snapshot materialization (`Deno.env.toObject`)",
        "Interactive permission escalation APIs (`Deno.permissions.request()` / `revoke()` and similar prompt-style flows)",
        "Non-literal `import(expr)`",
        "`eval`",
        "`Function()` constructor",
        "`Proxy`",
        "`WeakMap` / `WeakSet` / `WeakRef`",
        "`FinalizationRegistry`",
        "Stage-3+/draft TC39 proposals beyond the latest published ECMA-262 edition",
        "Dynamic `require()`",
    ] {
        assert!(dashboard.contains(expected), "dashboard missing expected row or heading: {expected}");
    }

    let supported = dashboard
        .find("## Supported today")
        .expect("supported heading");
    let gated = dashboard
        .find("## Gated for later phases")
        .expect("gated heading");
    let rejected = dashboard
        .find("## Rejected by default")
        .expect("rejected heading");
    assert!(
        supported < gated && gated < rejected,
        "dashboard buckets should remain ordered deterministically"
    );

    let supported_section = dashboard
        .split_once("## Supported today")
        .map(|(_, rest)| rest)
        .expect("supported section")
        .split_once("## Gated for later phases")
        .map(|(supported, _)| supported)
        .expect("supported section terminator");
    let mut supported_features = BTreeSet::new();
    for line in supported_section.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| ")
            || trimmed.starts_with("| Feature")
            || trimmed.starts_with("|---")
        {
            continue;
        }

        let feature = trimmed.split('|').nth(1).expect("feature column").trim();
        if feature.is_empty() {
            continue;
        }

        assert!(
            supported_features.insert(feature.to_string()),
            "supported dashboard row duplicated: {feature}"
        );
    }

    let supported_rows = [
        "Latest published ECMA-262 lexical grammar (tokenization)",
        "Current-edition non-Annex-B semantics for features Kali marks as supported in a given command/profile",
        "Static ESM `import` / `export`",
        "Generator function declarations / expressions and `yield` / `yield*` expressions",
        "First-class JavaScript compilation with bounded inference",
        "TypeScript assertion / satisfies expressions (`as`, `satisfies`)",
        "Read-only `Deno.permissions.query(...)` const-bound descriptor aliases in `.ts` and `.js` input",
        "Budgeted local/intra-module constraint solving inside the shared bounded inference contract",
        "Read-only `Deno.pid` / `Deno[\"pid\"]` / `globalThis.Deno.pid` handling in `.js` input",
        "Basic optional chaining member and element access",
        "Basic BigInt addition semantics",
        "Basic BigInt addition semantics in `.js` input",
        "Browser bundle basic object property deletion / `in`-operator semantics in `.ts` and `.js` input",
        "Basic unary prefix semantics (`!`, unary `-`, unary `+`, and `void`) in `.ts` and `.js` input, including JSON-output coverage on the standalone `run` / `test` lanes",
        "Browser bundle `try/catch` exception semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` `Math.clz32()` semantics in `.ts` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.clz32()` semantics when the browser API surface is inherited in `.ts` input, including JSON-output coverage",
        "Browser-requested `run` / `test` `Math.clz32()` semantics when the browser API surface is inherited in `.js` input",
        "Browser-requested `run` / `test` zero-capable budget pair (`--max-threads 0` / `--max-spawned-processes 0`) in `.js` input",
        "CommonJS module lowering",
        "`require(\"literal\")`",
    ];
    let mut last = 0;
    for row in supported_rows {
        let pos = dashboard.find(row).expect("supported row");
        assert!(
            pos >= last,
            "supported rows should be stable and sorted in their section"
        );
        last = pos;
    }
}

#[test]
fn phase_six_conformance_dashboard_tracks_additional_supported_browser_runtime_rows() {
    let root = repo_root();
    let dashboard = fs::read_to_string(root.join("plan/phase-6/conformance-dashboard.md"))
        .expect("read conformance dashboard");

    let supported_rows = [
        "Browser bundle basic object property deletion / `in`-operator semantics in `.ts` and `.js` input",
        "Browser bundle `try/catch` exception semantics in `.ts` and `.js` input",
        "Browser bundle `try/finally` sequencing in `.ts` and `.js` input",
        "Browser bundle basic strict equality / inequality semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` basic try/catch exception handling and try/finally sequencing in `.ts` and `.js` input, including JSON-output coverage, plus inherited browser-api-surface coverage in `.ts` and `.js` input",
        "Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage",
        "Browser bundle console error / warn / info / debug routing plus `console.assert()` false-branch reporting",
    ];

    let mut last = 0;
    for row in supported_rows {
        let pos = dashboard.find(row).expect("supported row");
        assert!(
            pos >= last,
            "supported rows should be stable and sorted in their section"
        );
        last = pos;
    }
}

fn collect_proof_theorem_names(root: &Path) -> BTreeSet<String> {
    fn visit(dir: &Path, names: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).expect("read proof directory") {
            let entry = entry.expect("read proof directory entry");
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            if path.is_dir() {
                if name == ".lake" || name == "build" {
                    continue;
                }
                visit(&path, names);
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("lean") {
                continue;
            }

            let text = fs::read_to_string(&path).expect("read proof source");
            for line in text.lines() {
                let trimmed = line.trim_start();
                let remainder = trimmed
                    .strip_prefix("theorem ")
                    .or_else(|| trimmed.strip_prefix("lemma "));
                let Some(remainder) = remainder else {
                    continue;
                };

                let name = remainder
                    .split(|ch: char| ch.is_whitespace() || ch == '(' || ch == ':')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    names.insert(name.to_string());
                }
            }
        }
    }

    let mut names = BTreeSet::new();
    visit(root, &mut names);
    names
}

fn parse_boundary_covered_paths(boundary: &str) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let section = boundary
        .split_once("## Covered implementation/spec paths")
        .map(|(_, rest)| rest)
        .expect("covered paths section");

    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            break;
        }
        if let Some(path) = trimmed.strip_prefix("- ") {
            let path = path.trim().trim_matches('`');
            if path.ends_with(".lean") {
                paths.insert(path.to_string());
            }
        }
    }

    paths
}

#[test]
fn proof_check_workflow_is_configured_for_proofs_changes() {
    let root = repo_root();
    let workflow =
        fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read workflow");

    assert!(
        workflow.contains("proof-check:"),
        "proof-check job is missing"
    );
    assert!(
        workflow.contains("id: proofs"),
        "proof-check job should name the proof-change filter step"
    );
    assert!(
        workflow.contains("proofs/**"),
        "proof-check job should watch the proofs directory"
    );
    assert!(
        workflow.contains("bash scripts/check-proof-tree.sh"),
        "proof-check job should verify the Lean proof tree layout"
    );
    assert!(
        workflow.contains("leanprover/lean-action@v1"),
        "proof-check job should install the Lean toolchain"
    );
    assert!(
        workflow.contains("lake-package-directory: proofs"),
        "proof-check job should build the proofs workspace from the proofs directory"
    );
}

#[test]
fn proof_check_script_verifies_lake_roots_against_proof_sources() {
    let root = repo_root();
    let script = fs::read_to_string(root.join("scripts/check-proof-tree.sh"))
        .expect("read proof tree script");

    assert!(
        script.contains("proof lakefile roots do not match the proof source directories"),
        "proof tree script should reject mismatched lake roots"
    );
    assert!(
        script.contains("sed -n 's/^lean_lib[[:space:]]\\+\\([A-Za-z0-9_][A-Za-z0-9_]*\\).*/\\1/p' lakefile.lean"),
        "proof tree script should extract lean_lib roots from the lakefile"
    );
}

#[test]
fn proof_boundary_manifest_tracks_the_actual_proof_sources() {
    let root = repo_root();
    let boundary = fs::read_to_string(root.join("proofs/BOUNDARY.md")).expect("read boundary");
    let documented = parse_boundary_covered_paths(&boundary);
    let actual = collect_proof_sources(&root.join("proofs"));

    assert_eq!(
        documented, actual,
        "proof boundary manifest should track the actual Lean proof source files"
    );
}

#[test]
fn proof_boundary_manifest_tracks_the_published_theorem_inventory() {
    let root = repo_root();
    let boundary = fs::read_to_string(root.join("proofs/BOUNDARY.md")).expect("read boundary");
    let published_theorems = collect_proof_theorem_names(&root.join("proofs"));
    let internal_helpers = [
        "subst_closed",
        "Context.lookup_remove_head",
        "Context.lookup_remove_head_other",
        "Context.lookup_remove_ne",
    ];

    for theorem in published_theorems {
        if internal_helpers.contains(&theorem.as_str()) {
            continue;
        }

        assert!(
            boundary.contains(&theorem),
            "proof boundary manifest is missing theorem or lemma: {theorem}"
        );
    }
}

#[test]
fn readme_command_reference_tracks_the_current_cli_surface() {
    let root = repo_root();
    let readme = fs::read_to_string(root.join("README.md")).expect("read README");

    for expected in [
        "kali doctor                         # inspect local tool/environment selection",
        "kali init",
        "kali init --lib                 # create the minimal library scaffold",
        "kali init --output json         # emit the schema-v1 init result envelope",
        "kali init --output json --lib    # emit the library scaffold result envelope",
        "kali install",
        "kali fmt",
        "kali lint",
        "kali check [files...]",
        "kali build <file>",
        "kali build --validate-ir <file>  # run internal HIR/MIR/LIR validators",
        "kali build --profile pgo-profile.json main.ts # load deterministic PGO profile data",
        "kali build --bundle <file>      # browser-targeted build lane",
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
        "kali effects <file>",
        "kali package-effects <package>",
        "kali package-audit <package>",
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
        "Status: **Phase 4 compatibility**. This section also documents a **defined command family** in schema v1;",
        "### `kali package-audit <package>`",
        "kali package-audit lodash",
        "kali package-audit --output json lodash",
        "kali run --api browser main.ts",
        "kali build --api node main.ts",
        "kali build --sandbox kali.policy.json main.ts",
        "kali build --bundle --api browser --sandbox kali.policy.json main.ts",
        "kali build --profile pgo-profile.json main.ts # Load deterministic PGO profile data and record its normalized hash in build metadata sidecars and JSON output",
        "kali build --lib --sandbox kali.policy.json lib.ts",
        "kali build --capi --sandbox kali.policy.json lib.ts",
        "kali build --component --sandbox kali.policy.json lib.ts",
        "kali build --capi lib.ts",
        "kali build --component lib.ts",
    ] {
        assert!(
            cli_spec.contains(expected),
            "specs/12-cli.md is missing CLI example or contract marker: {expected}"
        );
    }
}

#[test]
fn phase_7_readme_tracks_browser_harness_summary_fallback_coverage() {
    let root = repo_root();
    let readme =
        fs::read_to_string(root.join("plan/phase-7/README.md")).expect("read phase 7 README");

    assert!(
        readme.contains(
            "the browser harness sandbox-rejection matrix in JS input now covers all `run` / `test` × direct/inherited browser-api-surface quadrants, including JSON-output coverage"
        ),
        "phase 7 README should keep the browser-harness sandbox matrix note explicit"
    );
    assert!(
        readme.contains(
            "The browser-runtime and browser-bundle summary parsers now also merge missing `testsFailed` data from stdout when a summary file omits it"
        ),
        "phase 7 README should keep the summary-fallback note explicit"
    );
    assert!(
        readme
            .contains("and the CLI schema-doc drift net now also pins this summary-fallback note"),
        "phase 7 README should keep the schema-doc drift-net note explicit"
    );
}

#[test]
fn package_corpus_matrix_tracks_current_browser_and_default_rows() {
    let root = repo_root();
    let matrix = fs::read_to_string(root.join("plan/phase-8/package-corpus-matrix.md"))
        .expect("read package corpus matrix");

    for (row, message) in [
        (
            "| npm-style package corpus | pure JS package (`semver`) with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser semver corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | exports-map packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser exports-map corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | module-entry packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser module-entry corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | module-entry-chain packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser module-entry-chain corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser replacement-map packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser replacement-map corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-condition / browser-deno preference packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser condition / browser-deno corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-blocked packages with `.js` input | browser-targeted | `check`, `build --bundle` | rejected by default | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser blocked-package corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | typed export branch packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser typed export branch corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | scoped packages with exports maps | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser scoped package corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | exports-map packages with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser exports-map JS corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser pi-coding-agent corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser inherited pi-coding-agent corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser web-baseline primitive packages with `.js` input and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser web-baseline inherited-browser corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser package fixtures with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser package fixtures with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser-blocked packages with `.js` input | browser-targeted execution harness | `run`, `test` | rejected by default | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime blocked-package corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser internal browser-rewrite packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime internal-browser-rewrite corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser internal browser-rewrite packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser internal-browser-rewrite corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser exports-map packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime exports-map corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser exports-map packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser exports-map corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser replacement-map packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime replacement-map corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser replacement-map packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser replacement-map corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | typed export branch packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime typed export branch corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | typed export branch packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser typed export branch corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser string-entry packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime string-entry corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser string-entry packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser string-entry corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser string-export packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime string-export corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser string-export packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser string-export corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser pattern-exports packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime pattern-exports corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser pattern-exports packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser pattern-exports corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-only packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime module-only corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-only packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser module-only corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-entry-chain packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime module-entry-chain corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-entry-chain packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser module-entry-chain corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser-condition / browser-deno preference packages | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime browser-condition / browser-deno TS row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser-condition / browser-deno preference packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime browser-condition / browser-deno JS row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser-condition / browser-deno preference packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser browser-condition / browser-deno JS row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser dual-exports packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime dual-exports corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser dual-exports packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser dual-exports corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | mixed CommonJS/ESM interop packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser mixed-format runtime corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | mixed CommonJS/ESM interop packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser inherited mixed-format runtime corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser internal browser-rewrite packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser internal browser-rewrite corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-condition / browser-string / web-baseline packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser condition / string / web-baseline corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser pattern-exports packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser pattern-exports `.js` corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | mixed CommonJS/ESM interop packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser-targeted mixed-format corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | mixed CommonJS/ESM interop packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser-targeted mixed-format JS entrypoint corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser replacement-map packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser replacement-map `.js` corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser string-entry packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser string-entry `.js` corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser string-export packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser string-export `.js` corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-condition export packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser condition-export `.js` corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-condition export packages with `.js` entrypoints and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser inherited condition-export `.js` corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime pi-coding-agent corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser pi-coding-agent corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | web-baseline packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime web-baseline corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | web-baseline packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser web-baseline corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser package fixtures | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime corpus TS row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-entry packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime module-entry corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-entry packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime inherited-browser module-entry corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser web-baseline primitive packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser web-baseline TS corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser web-baseline primitive packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser web-baseline JS entrypoint corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | scoped packages with exports maps with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser scoped packages with exports maps and js input should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | mixed CommonJS/ESM interop packages | default standalone | `run`, `test`, `build` | executable / testable / buildable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone mixed-format corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | mixed CommonJS/ESM interop packages with `.js` input | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone mixed-format JS input corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS package (`semver`) with `.js` input | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone semver corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| binary-entrypoint probe | `semver` bin entrypoints | default standalone | `run` | rejected by default | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone semver bin-entrypoint row should be recorded in the package corpus matrix",
        ),
        (
            "| binary-entrypoint probe | `@mariozechner/pi-coding-agent` bin entrypoints | default standalone | `run` | rejected by default | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone pi-coding-agent bin-entrypoint row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | scoped packages with `.js` input | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone scoped corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-condition / browser-deno preference packages with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser condition preference corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-condition / browser-deno preference packages with `.js` input and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser condition preference inherited-browser corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser dual-exports packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser dual-exports corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input and test coverage | default standalone | `check`, `build`, `test` | checkable / buildable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone package-content test row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS utility package (`date-fns`) with `.js` input | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone date-fns JS utility row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS utility package (`date-fns`) with `.ts` input | default standalone | `test` | testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone date-fns TS utility row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS utility packages (`zod`, `plimit`, `ms`) with `.js` input | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone zod/p-limit/ms JS utility row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS package (`semver`) with `.js` input | Node | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "node semver corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| binary-entrypoint probe | `semver` bin entrypoints | Node | `run` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node semver bin-entrypoint row should be recorded in the package corpus matrix",
        ),
        (
            "| binary-entrypoint probe | `@mariozechner/pi-coding-agent` bin entrypoints | Node | `run` | executable on the Node surface; rejected on the default standalone surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node pi-coding-agent bin-entrypoint row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | runner packages with exports maps with `.js` input | Node | `run`, `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node runner exports-map JS corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | Node built-in packages (`node:buffer`, `node:assert`, `node:child_process`, `node:events`, `node:stream`, `node:fs/promises`, `node:path`, `node:os`, `node:crypto`, `node:http`, `node:timers`, `node:timers/promises`, `node:fs`, `node:url`, `node:util`) on `.js` input | Node | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "node built-in corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | Node built-in packages (`node:buffer`, `node:assert`, `node:events`, `node:timers`) with inherited Node `apiSurface` on `.js` input | Node | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "node inherited built-in corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | Node built-in packages (`node:timers/promises`) with inherited Node `apiSurface` on `.js` input | Node | `check`, `build`, `run`, `test` | rejected by default | `crates/kali_cli/tests/node_api_surface.rs` |",
            "node inherited timers/promises rejection row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | node-assuming packages with `.js` input | Node | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "node-assuming corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| Deno-host package corpus | host-control packages (`Deno.env`, `Deno.Command`, `Deno.listen`, `Deno.serve`) with `.js` input | Deno | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "deno host-control corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| JSR corpus | `jsr:` packages materialized as on-disk package entries with `.js` input | Deno | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "jsr corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | exports-map packages | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone exports-map corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | module-entry packages | default standalone | `check`, `run` | checkable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone module-entry corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | web-baseline primitive packages with `.js` input | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone web-baseline JS corpus row should be recorded in the package corpus matrix",
        ),
    ] {
        assert!(matrix.contains(row), "{message}");
    }

    let assert_row_once = |row: &str, message: &str| {
        assert_eq!(matrix.matches(row).count(), 1, "{message}");
    };

    let browser_semver_targeted_row =
        "| npm-style package corpus | pure JS package (`semver`) with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_semver_targeted_row,
        "browser targeted semver corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_exports_map_row =
        "| browser runtime corpus | browser exports-map packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_exports_map_row,
        "browser runtime exports-map corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_exports_map_inherited_row =
        "| browser runtime corpus | browser exports-map packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_exports_map_inherited_row,
        "browser runtime inherited-browser exports-map corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_replacement_map_row =
        "| browser runtime corpus | browser replacement-map packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_replacement_map_row,
        "browser runtime replacement-map corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_replacement_map_inherited_row =
        "| browser runtime corpus | browser replacement-map packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_replacement_map_inherited_row,
        "browser runtime inherited-browser replacement-map corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_string_entry_row =
        "| browser runtime corpus | browser string-entry packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_string_entry_row,
        "browser runtime string-entry corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_string_entry_inherited_row =
        "| browser runtime corpus | browser string-entry packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_string_entry_inherited_row,
        "browser runtime inherited-browser string-entry corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_string_export_row =
        "| browser runtime corpus | browser string-export packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_string_export_row,
        "browser runtime string-export corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_string_export_inherited_row =
        "| browser runtime corpus | browser string-export packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_string_export_inherited_row,
        "browser runtime inherited-browser string-export corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_internal_browser_rewrite_row =
        "| npm-style package corpus | browser internal browser-rewrite packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_internal_browser_rewrite_row,
        "browser internal-browser-rewrite corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_internal_browser_rewrite_runtime_row =
        "| browser runtime corpus | browser internal browser-rewrite packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_internal_browser_rewrite_runtime_row,
        "browser runtime internal-browser-rewrite corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_internal_browser_rewrite_inherited_runtime_row =
        "| browser runtime corpus | browser internal browser-rewrite packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_internal_browser_rewrite_inherited_runtime_row,
        "browser runtime inherited-browser internal-browser-rewrite corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_condition_export_row =
        "| npm-style package corpus | browser-condition export packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_condition_export_row,
        "browser condition-export corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_condition_export_inherited_row =
        "| npm-style package corpus | browser-condition export packages with `.js` entrypoints and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_condition_export_inherited_row,
        "browser inherited condition-export corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_condition_row =
        "| npm-style package corpus | browser-condition / browser-deno preference packages with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_condition_row,
        "browser condition / browser-deno corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_condition_inherited_check_row =
        "| npm-style package corpus | browser-condition / browser-deno preference packages with `.js` input and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_condition_inherited_check_row,
        "browser inherited browser-condition / browser-deno corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_condition_string_web_baseline_row =
        "| npm-style package corpus | browser-condition / browser-string / web-baseline packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_condition_string_web_baseline_row,
        "browser condition / browser-string / web-baseline corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_condition_inherited_row =
        "| browser runtime corpus | browser-condition / browser-deno preference packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_condition_inherited_row,
        "browser inherited browser-condition / browser-deno runtime corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_dual_exports_row =
        "| npm-style package corpus | browser dual-exports packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_dual_exports_row,
        "browser dual-exports corpus row should be recorded exactly once in the package corpus matrix",
    );

    let browser_semver_row =
        "| browser runtime corpus | pure JS package (`semver`) with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_semver_row,
        "browser semver runtime corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_semver_inherited_row =
        "| browser runtime corpus | pure JS package (`semver`) with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_semver_inherited_row,
        "browser inherited semver runtime corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_mixed_format_row =
        "| browser runtime corpus | mixed CommonJS/ESM interop packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_mixed_format_row,
        "browser mixed-format runtime corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_mixed_format_inherited_row =
        "| browser runtime corpus | mixed CommonJS/ESM interop packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_mixed_format_inherited_row,
        "browser inherited mixed-format runtime corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_package_content_row =
        "| browser runtime corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_package_content_row,
        "browser package-content runtime corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_package_content_inherited_row =
        "| browser runtime corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_package_content_inherited_row,
        "browser inherited package-content runtime corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_package_content_inherited_check_build_row =
        "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_package_content_inherited_check_build_row,
        "browser inherited pi-coding-agent corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_package_fixtures_row =
        "| browser runtime corpus | browser package fixtures | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_package_fixtures_row,
        "browser runtime package-fixtures corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_package_fixtures_inherited_row =
        "| browser runtime corpus | browser package fixtures with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_package_fixtures_inherited_row,
        "browser inherited package-fixtures corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_web_baseline_targeted_row =
        "| npm-style package corpus | browser web-baseline primitive packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_web_baseline_targeted_row,
        "browser web-baseline targeted corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_web_baseline_js_entrypoint_row =
        "| npm-style package corpus | browser web-baseline primitive packages with `.js` entrypoints | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_web_baseline_js_entrypoint_row,
        "browser web-baseline JS entrypoint corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_web_baseline_inherited_row =
        "| npm-style package corpus | browser web-baseline primitive packages with `.js` input and inherited browser `apiSurface` | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_web_baseline_inherited_row,
        "browser inherited web-baseline corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_runtime_web_baseline_row =
        "| browser runtime corpus | web-baseline packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_runtime_web_baseline_row,
        "browser runtime web-baseline corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_runtime_web_baseline_inherited_row =
        "| browser runtime corpus | web-baseline packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_runtime_web_baseline_inherited_row,
        "browser inherited runtime web-baseline corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_entry_row =
        "| browser runtime corpus | browser module-entry packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_entry_row,
        "browser runtime module-entry corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_entry_inherited_row =
        "| browser runtime corpus | browser module-entry packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_entry_inherited_row,
        "browser inherited module-entry corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_entry_chain_row =
        "| browser runtime corpus | browser module-entry-chain packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_entry_chain_row,
        "browser runtime module-entry-chain corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_entry_chain_inherited_row =
        "| browser runtime corpus | browser module-entry-chain packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_entry_chain_inherited_row,
        "browser inherited module-entry-chain corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_only_targeted_row =
        "| npm-style package corpus | browser module-only packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_only_targeted_row,
        "browser module-only targeted corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_only_targeted_js_row =
        "| npm-style package corpus | browser module-only packages with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_only_targeted_js_row,
        "browser module-only targeted JS corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_pattern_exports_row =
        "| browser runtime corpus | browser pattern-exports packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_pattern_exports_row,
        "browser runtime pattern-exports corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_pattern_exports_inherited_row =
        "| browser runtime corpus | browser pattern-exports packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_pattern_exports_inherited_row,
        "browser runtime inherited-browser pattern-exports corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_only_row =
        "| browser runtime corpus | browser module-only packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_only_row,
        "browser runtime module-only corpus row should be recorded exactly once in the package corpus matrix",
    );
    let browser_module_only_inherited_row =
        "| browser runtime corpus | browser module-only packages with `.js` input and inherited browser `apiSurface` | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        browser_module_only_inherited_row,
        "browser runtime inherited-browser module-only corpus row should be recorded exactly once in the package corpus matrix",
    );

    let default_exports_map_row =
        "| npm-style package corpus | exports-map packages | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        default_exports_map_row,
        "default standalone exports-map corpus row should be recorded exactly once in the package corpus matrix",
    );
    let default_module_entry_row =
        "| npm-style package corpus | module-entry packages | default standalone | `check`, `run` | checkable / executable | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        default_module_entry_row,
        "default standalone module-entry corpus row should be recorded exactly once in the package corpus matrix",
    );
    let default_web_baseline_js_row =
        "| npm-style package corpus | web-baseline primitive packages with `.js` input | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        default_web_baseline_js_row,
        "default standalone web-baseline JS corpus row should be recorded exactly once in the package corpus matrix",
    );

    let default_package_content_test_row =
        "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input and test coverage | default standalone | `check`, `build`, `test` | checkable / buildable / testable | `crates/kali_cli/tests/package_corpus.rs` |";
    assert_row_once(
        default_package_content_test_row,
        "default standalone package-content test corpus row should be recorded exactly once in the package corpus matrix",
    );

    for (row, message) in [
        (
            "| npm-style package corpus | pure JS package (`semver`) with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser semver runtime corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pattern-exports packages with `.js` entrypoints | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone pattern-exports corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | exports-map mixed-format interop packages with `.js` entrypoints | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone exports-map mixed-format corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | module-entry packages with `.js` entrypoints | default standalone | `check`, `run` | checkable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone module-entry JS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | module-entry packages and module-entry chains | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone module-entry / module-entry-chain corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | module-entry packages and module-entry chains with `.js` entrypoints | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone module-entry / module-entry-chain JS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | web-baseline primitive packages | default standalone | `build`, `run` | buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone web-baseline TS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | string-export packages | default standalone | `run` | executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone string-export corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | string-export packages with `.js` input | default standalone | `test` | testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone string-export JS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS utility packages (`date-fns`, `zod`, `plimit`, `ms`) | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone pure-JS utility corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS package (`semver`) | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone semver corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) | default standalone | `check`, `build` | checkable / buildable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone package-content TS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone package-content JS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | scoped packages | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone scoped corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | runner packages (`vitest`, `jest`, `mocha`, `ava`) | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node runner corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | runner packages with exports maps | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node runner exports-map corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | runner packages with exports maps and `node:buffer` built-in usage with `.js` entrypoints | Node | `run`, `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node runner exports-map JS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | runner packages with mixed-format entries | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node runner mixed-format corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | runner packages with mixed-format entries with `.js` entrypoints | Node | `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node runner mixed-format JS corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | Node built-in packages (`node:timers/promises`) with inherited Node `apiSurface` on `.js` input | Node | `check`, `build`, `run`, `test` | rejected by default | `crates/kali_cli/tests/node_api_surface.rs` |",
            "node inherited timers/promises rejection row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| package-resolution corpus | Node-assuming packages | Node vs default standalone contrast | `check`, `run` vs rejection paths | gated on the Node surface; rejected by default standalone | `crates/kali_cli/tests/package_corpus.rs` |",
            "node-versus-standalone package-resolution corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| Deno-host package corpus | host-control packages (`Deno.env`, `Deno.Command`, `Deno.listen`, `Deno.serve`) | Deno | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "deno host-control corpus row should be recorded exactly once in the package corpus matrix",
        ),
        (
            "| JSR corpus | `jsr:` packages materialized as on-disk package entries | Deno | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "jsr corpus row should be recorded exactly once in the package corpus matrix",
        ),
    ] {
        assert_row_once(row, message);
    }

    assert!(
        matrix.contains(
            "The browser runtime module-entry-chain rows also carry JSON-output coverage on the configured browser harness path, including the direct and inherited browser `apiSurface` variants."
        ),
        "package corpus matrix should document browser module-entry-chain JSON-output coverage"
    );
    assert!(
        matrix.contains(
            "The browser runtime module-only row also carries JSON-output coverage on the direct browser-harness variant."
        ),
        "package corpus matrix should document browser module-only JSON-output coverage"
    );
    assert!(
        matrix.contains(
            "The browser runtime mixed-format interop rows now also carry JSON-output coverage on the direct and inherited browser-harness variants."
        ),
        "package corpus matrix should document browser mixed-format interop JSON-output coverage"
    );
    assert!(
        matrix.contains(
            "The browser runtime package-content rows also carry JSON-output coverage on the direct and inherited browser-harness variants."
        ),
        "package corpus matrix should document browser package-content JSON-output coverage"
    );
    assert!(
        matrix.contains(
            "The browser runtime package fixture rows also carry JSON-output coverage on the direct browser-harness `run` / `test` variants."
        ),
        "package corpus matrix should document browser package fixture JSON-output coverage"
    );
    assert!(
        matrix.contains(
            "The browser runtime exports-map rows now also carry JSON-output coverage on the direct and inherited browser-harness variants."
        ),
        "package corpus matrix should document browser exports-map JSON-output coverage"
    );
    assert!(
        matrix.contains(
            "The browser runtime semver rows also carry JSON-output coverage on the configured browser harness path, including the direct and inherited browser `apiSurface` variants."
        ),
        "package corpus matrix should document browser semver runtime JSON-output coverage"
    );
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
        "^[a-z0-9-]+-benchmark-v1\\.ts$"
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
        "math-trunc-builtin",
        "math-imul-builtin",
        "math-clz32-builtin",
        "math-ceil-builtin",
        "math-abs-sign-builtin",
        "division-by-one-elimination",
        "dead-branch-elimination",
        "dead-inlined-function-pruning",
        "division-and-identity",
        "closure-inlining-and-folding",
        "object-enumeration-folding",
        "integer-like-object-enumeration-folding",
        "object-enumeration-alias-chain",
        "object-enumeration-delete-reinsert",
        "object-literal-property-order-canonicalization",
        "identity-chain-and-simplification",
        "nested-wrapper-pruning",
        "algebraic-simplification",
        "duplicate-pure-expression-elimination",
        "nullish-specialization-repeat",
        "specialization-reuse",
        "boolean-literal-arguments",
        "const-array-element-access",
        "const-object-property-access",
        "folded-arithmetic-variant",
        "string-concatenation",
        "template-literal-concatenation",
        "layout-specialization",
        "nullish-specialization",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect();
    let expected_benchmark_sources: BTreeSet<String> = [
        "math-benchmark-v1.ts",
        "math-trunc-benchmark-v1.ts",
        "math-imul-benchmark-v1.ts",
        "math-clz32-benchmark-v1.ts",
        "math-ceil-benchmark-v1.ts",
        "math-abs-sign-benchmark-v1.ts",
        "division-by-one-benchmark-v1.ts",
        "dead-branch-elimination-benchmark-v1.ts",
        "dead-inlined-function-pruning-benchmark-v1.ts",
        "call-inlining-benchmark-v1.ts",
        "closure-inlining-benchmark-v1.ts",
        "object-enumeration-benchmark-v1.ts",
        "integer-like-object-enumeration-benchmark-v1.ts",
        "object-enumeration-alias-chain-benchmark-v1.ts",
        "object-enumeration-delete-reinsert-benchmark-v1.ts",
        "object-literal-property-order-canonicalization-benchmark-v1.ts",
        "identity-chain-benchmark-v1.ts",
        "nested-wrapper-pruning-benchmark-v1.ts",
        "algebraic-simplification-benchmark-v1.ts",
        "duplicate-pure-expression-elimination-benchmark-v1.ts",
        "nullish-specialization-repeat-benchmark-v1.ts",
        "specialization-reuse-benchmark-v1.ts",
        "boolean-literal-arguments-benchmark-v1.ts",
        "const-array-element-access-benchmark-v1.ts",
        "const-object-property-access-benchmark-v1.ts",
        "math-variant-benchmark-v1.ts",
        "string-concatenation-benchmark-v1.ts",
        "template-literal-concatenation-benchmark-v1.ts",
        "layout-specialization-benchmark-v1.ts",
        "nullish-benchmark-v1.ts",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect();

    for entry in fs::read_dir(root.join("crates/kali_cli/tests/fixtures/benchmarks"))
        .expect("read benchmark fixture directory")
    {
        let path = entry.expect("benchmark fixture entry").path();
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
        assert_eq!(
            metadata["sourceFile"],
            serde_json::json!(format!(
                "{}.ts",
                path.file_stem().expect("benchmark stem").to_string_lossy()
            )),
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

#[test]
fn optimization_inventory_tracks_current_mode_rows() {
    let root = repo_root();
    let inventory = fs::read_to_string(root.join("plan/phase-9/optimization-inventory.md"))
        .expect("read optimization inventory");

    for expected in [
        "# Phase 9 Optimization Inventory",
        "## Current checked-in evidence",
        "| `fast` |",
        "| `release` |",
        "| `release-advanced` |",
        "fast_keeps_binary_expressions_opaque",
        "release_folds_object_keys_calls_over_literal_object_shapes",
        "release_folds_object_entries_calls_over_literal_object_shapes",
        "release_folds_object_values_calls_over_literal_object_shapes",
        "release_folds_object_enumeration_calls_over_const_bound_literal_object_shapes",
        "release_folds_object_enumeration_calls_over_const_alias_chains",
        "release_advanced_eliminates_algebraic_identities",
        "release_advanced_folds_object_enumeration_calls_over_literal_object_shapes",
        "release_advanced_folds_object_enumeration_calls_over_const_bound_literal_object_shapes",
        "release_advanced_folds_object_enumeration_calls_over_const_alias_chains",
        "math-benchmark-v1",
        "math-trunc-benchmark-v1",
        "math-imul-benchmark-v1",
        "math-clz32-benchmark-v1",
        "math-ceil-benchmark-v1",
        "math-abs-sign-benchmark-v1",
        "division-by-one-benchmark-v1",
        "dead-branch-elimination-benchmark-v1",
        "dead-inlined-function-pruning-benchmark-v1",
        "call-inlining-benchmark-v1",
        "closure-inlining-benchmark-v1",
        "object-enumeration-benchmark-v1",
        "integer-like-object-enumeration-benchmark-v1",
        "object-enumeration-alias-chain-benchmark-v1",
        "object-enumeration-delete-reinsert-benchmark-v1",
        "object-literal-property-order-canonicalization-benchmark-v1",
        "identity-chain-benchmark-v1",
        "nested-wrapper-pruning-benchmark-v1",
        "algebraic-simplification-benchmark-v1",
        "duplicate-pure-expression-elimination-benchmark-v1",
        "nullish-specialization-repeat-benchmark-v1",
        "specialization-reuse-benchmark-v1",
        "boolean-literal-arguments-benchmark-v1",
        "const-array-element-access-benchmark-v1",
        "const-object-property-access-benchmark-v1",
        "math-variant-benchmark-v1",
        "string-concatenation-benchmark-v1",
        "template-literal-concatenation-benchmark-v1",
        "layout-specialization-benchmark-v1",
        "nullish-benchmark-v1",
        "## Reading rule",
    ] {
        assert!(
            inventory.contains(expected),
            "optimization inventory missing expected text: {expected}"
        );
    }

    let fast = inventory.find("| `fast` |").expect("fast row");
    let release = inventory.find("| `release` |").expect("release row");
    let release_advanced = inventory
        .find("| `release-advanced` |")
        .expect("release-advanced row");
    assert!(
        fast < release && release < release_advanced,
        "optimization inventory rows should remain ordered deterministically"
    );
}
