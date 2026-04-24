use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
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
    assert_eq!(build["anyOf"].as_array().expect("anyOf array").len(), 6);

    let test_result: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("schemas/result/test/v1.json")).expect("read test schema"),
    )
    .expect("parse test schema");
    assert!(test_result["properties"]["coverage"].is_object());

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
