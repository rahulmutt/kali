use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

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
        "schemas/result/fmt/v1.json",
        "schemas/result/lint/v1.json",
        "schemas/result/doctor/v1.json",
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
    assert_eq!(envelope["properties"]["schemaVersion"]["const"], 1);
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

    let test_result: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/test/v1.json")).expect("read test schema"),
    )
    .expect("parse test schema");
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
    let enum_values = artifact_meta["properties"]["artifactKind"]["enum"]
        .as_array()
        .expect("artifactKind enum array")
        .iter()
        .map(|value| value.as_str().expect("enum string"))
        .collect::<Vec<_>>();
    assert_eq!(
        enum_values,
        vec!["executable", "lib", "bundle", "capi", "component"]
    );
    for property in [
        "runtimeProfiles",
        "maxSpecializations",
        "profileDataHash",
        "hostContract",
        "runtimeBackend",
    ] {
        assert!(
            artifact_meta["properties"].get(property).is_some(),
            "missing artifact metadata property: {property}"
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

    let check: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/check/v1.json")).expect("read check schema"),
    )
    .expect("parse check schema");
    assert_eq!(check["type"], "object");
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
    assert_eq!(
        effects["properties"]["dynamicReasons"]["items"]["type"],
        "string"
    );

    let doctor: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/doctor/v1.json"))
            .expect("read doctor schema"),
    )
    .expect("parse doctor schema");
    assert_eq!(doctor["title"], "Kali Doctor Result v1");
    assert_eq!(doctor["type"], "object");
    assert_eq!(doctor["additionalProperties"], false);
    assert_eq!(
        required_fields(&doctor),
        ["browserHarness"]
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

    let package_audit: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/package-audit/v1.json"))
            .expect("read package-audit schema"),
    )
    .expect("parse package-audit schema");
    assert_eq!(package_audit["title"], "Kali Package Audit Result v1");
    assert_eq!(
        package_audit["description"],
        "Envelope-only JSON command payload for the later package-audit surface."
    );
    assert_eq!(package_audit["type"], "null");
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
    assert_eq!(schema["properties"]["artifacts"]["type"], "object");
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
        "Budgeted local/intra-module constraint solving inside the shared bounded inference contract",
        "CommonJS module lowering",
        "`require(\"literal\")`",
        "Basic `Math.sign()` built-in semantics",
        "Basic queueMicrotask ordering semantics in `.js` input",
        "Browser bundle integer-like key ordering semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` integer-like key ordering semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input",
        "Browser-requested `run` / `test` basic try/catch exception handling and try/finally sequencing in `.ts` and `.js` input",
        "Browser-requested `run` / `test` Web Crypto randomness subset via `crypto.getRandomValues()` in `.js` input",
        "Browser-requested `run` / `test` basic strict equality / inequality semantics in `.ts` and `.js` input",
        "Browser-requested `run` / `test` console error / warn / info / debug routing plus `console.assert()` false-branch reporting in `.js` input",
        "Browser bundle console error / warn / info / debug routing plus `console.assert()` false-branch reporting",
        "Open-ended or unstable cross-module/public-API constraint solving",
        "Literal-string `import()`",
        "Non-literal `import(expr)`",
        "`eval`",
        "`Function()` constructor",
        "`Proxy`",
        "`WeakMap` / `WeakSet`",
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

    let supported_rows = [
        "Latest published ECMA-262 lexical grammar (tokenization)",
        "Current-edition non-Annex-B semantics for features Kali marks as supported in a given command/profile",
        "Static ESM `import` / `export`",
        "Generator function declarations / expressions and `yield` / `yield*` expressions",
        "First-class JavaScript compilation with bounded inference",
        "Budgeted local/intra-module constraint solving inside the shared bounded inference contract",
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
        "kali install",
        "kali fmt",
        "kali lint",
        "kali check [files...]",
        "kali build <file>",
        "kali build --validate-ir <file>  # run internal HIR/MIR/LIR validators",
        "kali build --bundle <file>      # browser-targeted build lane",
        "kali build --lib <file>         # base library artifact for exact-version consumers",
        "kali build --capi <file>        # stable public C-ABI embedding flow",
        "kali build --component <file>   # Component Model packaging flow",
        "kali run <file> [-- args...]",
        "kali test [files...]",
        "kali test --coverage [files...]",
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
            "| npm-style package corpus | exports-map packages with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser exports-map JS corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | host-heavier package-content probe (`@mariozechner/pi-coding-agent`) with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser pi-coding-agent corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser package fixtures with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-only packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime module-only corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| browser runtime corpus | browser module-entry-chain packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime module-entry-chain corpus row should be recorded in the package corpus matrix",
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
            "| browser runtime corpus | browser dual-exports packages with `.js` input | browser-targeted execution harness | `run`, `test` | executable / testable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser runtime dual-exports corpus row should be recorded in the package corpus matrix",
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
            "| npm-style package corpus | browser web-baseline primitive packages | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser web-baseline TS corpus row should be recorded in the package corpus matrix",
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
            "| npm-style package corpus | scoped packages with `.js` input | default standalone | `check`, `build`, `run` | checkable / buildable / executable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone scoped corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | browser-condition / browser-deno preference packages with `.js` input | browser-targeted | `check`, `build --bundle` | checkable / buildable / deployable-through-host | `crates/kali_cli/tests/package_corpus.rs` |",
            "browser condition preference corpus row should be recorded in the package corpus matrix",
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
            "| npm-style package corpus | pure JS utility packages (`zod`, `plimit`, `ms`) with `.js` input | default standalone | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "default standalone zod/p-limit/ms JS utility row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | pure JS package (`semver`) with `.js` input | Node | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "node semver corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | runner packages with exports maps with `.js` input | Node | `run`, `test` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node runner exports-map JS corpus row should be recorded in the package corpus matrix",
        ),
        (
            "| binary-entrypoint probe | `semver` bin entrypoints | Node | `run` | executable on the Node surface | `crates/kali_cli/tests/package_corpus.rs` |",
            "node semver bin-entrypoint row should be recorded in the package corpus matrix",
        ),
        (
            "| npm-style package corpus | Node built-in packages (`node:buffer`, `node:assert`, `node:path`, `node:crypto`, `node:fs`, `node:url`, `node:util`) on `.js` input | Node | `check`, `build`, `run`, `test` | checkable / buildable / executable / testable | `crates/kali_cli/tests/package_corpus.rs` |",
            "node built-in corpus row should be recorded in the package corpus matrix",
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
    ] {
        assert!(matrix.contains(row), "{message}");
    }
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
        "closure-inlining-benchmark-v1",
        "object-enumeration-benchmark-v1",
        "identity-chain-benchmark-v1",
        "algebraic-simplification-benchmark-v1",
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
