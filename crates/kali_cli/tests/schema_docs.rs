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
}

#[test]
fn proof_boundary_summary_matches_readme_manifest_and_status_docs() {
    let root = repo_root();
    let summary =
        "Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.";
    let proof_backed_phrase = "proof-backed for the published boundary";
    let rc_predicate_vocabulary = ["hasOwnership", "allocated", "liveAnnotated"];
    let rc_theorem_names = [
        "releaseRefNoDanglingReference",
        "KaliCore.Safety.noDanglingReference",
        // Keep the release-only helper and the decrement/collection live-reference filtering corollaries pinned together.
        "releaseRefLiveRefsAreOwnedAndAllocated",
        "releaseRefLiveRefsAreLiveAnnotated",
        "releaseAndDecrementLiveRefsAreLiveAnnotated",
        "releaseAndCollectLiveRefsAreLiveAnnotated",
        "releaseAndCollectLiveRefsAreOwnedAndAllocated",
        "liveRefsAreOwnedAndAllocated",
        "releaseRefLiveRefsFiltered",
        "releaseAndDecrementLiveRefsFiltered",
        "releaseAndCollectLiveRefsFiltered",
        "releasePreservesWellFormed",
        "releaseRecorded",
        "releaseAndDecrementRecorded",
        "releaseAndDecrementDecrementsTargetCell",
        "releaseAndDecrementPreservesWellFormed",
        "releaseAndDecrementPreservesOwnership",
        "releaseAndDecrementLiveRefsAreOwnedAndAllocated",
        "releaseAndDecrementLiveRefsAreLiveAnnotated",
        "releaseAndDecrementReleasedNotLiveRef",
        "releaseAndDecrementZeroesLastTargetCell",
        "releaseRefReleasedRefsCons",
        "releaseRefPreservesReleasedRefs",
        "releaseRefHeapCharacterisation",
        "releaseRefHeapCharacterisationAndLinearMemory",
        "releaseRefHeapCellOrigin",
        "releaseRefHeapCellOriginAndOwnership",
        "releaseRefHeapCellOriginOwnershipAndPositiveCount",
        "releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory",
        "releaseRefPreservesOwnership",
        "releaseRefPreservesLinearMemory",
        "releaseRefPreservesOwnershipAndLinearMemory",
        "releaseAndCollectPreservesLinearMemory",
        "releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory",
        "releaseAndCollectPreservesOwnershipAndLinearMemory",
        "releaseRefPreservesWellFormedAndLinearMemory",
        "releaseAndDecrementPreservesWellFormedAndLinearMemory",
        "releaseAndCollectPreservesWellFormedAndLinearMemory",
        "releaseRefPreservesWellFormedAndOwnershipAndLinearMemory",
        "releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory",
        "releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory",
        "releaseRefReleasedNotLiveRef",
        "releaseAndDecrementNoDanglingReference",
        "releaseAndDecrementKeepsTargetCellWhenPositiveCount",
        "releaseAndDecrementKeepsOtherPositiveCountCells",
        "releaseAndDecrementTargetCellPositiveCountIff",
        "releaseAndDecrementTargetCellOrigin",
        "releaseAndDecrementKeepsOriginalPositiveCountCells",
        "releaseAndDecrementTargetCellOriginAndPositiveCount",
        "releaseAndDecrementKeepsOtherHeapEntries",
        "releaseAndDecrementPreservesOtherLiveRefs",
        "releaseAndDecrementHeapCellOrigin",
        "releaseAndDecrementHeapCellOriginAndOwnership",
        "releaseAndDecrementHeapCellOriginAndPositiveCount",
        "releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount",
        "releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory",
        "releaseAndDecrementTargetCellAllocatedWhenPositiveCount",
        "releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount",
        "releaseAndDecrementPreservesLinearMemory",
        "releaseAndDecrementPreservesOwnershipAndLinearMemory",
        "releaseAndCollectPreservesOwnership",
        "releaseAndDecrementReleasedRefsCons",
        "releaseAndDecrementPreservesReleasedRefs",
        "releaseAndCollectNoDanglingReference",
        "releasedNotLive",
        "releasedNotLiveRef",
        "releaseAndCollectRecorded",
        "releaseAndCollectDropsZeroCountCells",
        "releaseAndCollectRemovesZeroCountCells",
        "releaseAndCollectKeepsPositiveCountCells",
        "releaseAndCollectDropsOriginalZeroCountCells",
        "releaseAndCollectKeepsOtherPositiveCountCells",
        "releaseAndCollectKeepsOriginalPositiveCountCells",
        "releaseAndCollectKeepsOtherHeapEntries",
        "releaseAndCollectPreservesOtherLiveRefs",
        "releaseAndCollectKeepsTargetCellWhenPositiveCount",
        "releaseAndCollectTargetCellPresentIffPositiveCount",
        "releaseAndCollectPreservesWellFormed",
        "releaseAndCollectReleasedNotLiveRef",
        "releaseAndCollectHeapCellOrigin",
        "releaseAndCollectHeapCellOriginAndOwnership",
        "releaseAndCollectHeapCellOriginOwnershipAndPositiveCount",
        "releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory",
        "releaseAndCollectHeapCellOriginAndPositiveCount",
        "releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory",
        "releaseAndCollectHeapIsPositiveCountFilter",
        "releaseAndCollectHeapCellsHavePositiveCount",
        "releaseAndCollectTargetCellAllocatedWhenPositiveCount",
        "releaseAndCollectLiveRefsAreLiveAnnotated",
        "releaseAndCollectTargetCellOrigin",
        "releaseAndCollectTargetCellOriginOwnershipAndPositiveCount",
        "releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount",
        "releaseAndCollectPreservesLinearMemory",
        "releaseAndCollectPreservesOwnershipAndLinearMemory",
        "releaseAndCollectLiveRefsAreOwnedAndAllocated",
        "releaseAndCollectReleasedRefsCons",
        "releaseAndCollectPreservesReleasedRefs",
        "releaseAndDecrementHeapCharacterisation",
        "releaseAndDecrementHeapCharacterisationAndLinearMemory",
        "releaseAndCollectHeapCharacterisation",
        "releaseAndCollectHeapCharacterisationAndLinearMemory",
        "releaseAndCollectHeapCellsHavePositiveCount",
    ];
    let summary_docs = [
        ("README.md", summary),
        ("specs/16-testing.md", summary),
        ("proofs/BOUNDARY.md", summary),
        ("specs/17-verification.md", summary),
        ("specs/19-feature-maturity.md", proof_backed_phrase),
        ("plan/phase-4/02-formal-verification-depth.md", summary),
        ("PLAN-4.2-STATUS.md", summary),
    ];

    for (relative, expected_summary) in summary_docs {
        let text = fs::read_to_string(root.join(relative)).expect("read summary doc");
        assert!(
            text.contains(expected_summary),
            "{relative} is missing the canonical proof-backed summary"
        );
        for theorem in rc_theorem_names {
            assert!(
                text.contains(theorem),
                "{relative} is missing proof-summary theorem name: {theorem}"
            );
        }
        for term in rc_predicate_vocabulary {
            assert!(
                text.contains(term),
                "{relative} is missing RC predicate vocabulary term: {term}"
            );
        }
    }

    let lowering_theorem_names = [
        "KaliIR.Value",
        "KaliIR.LoweringCorrectness.lower_preserves_value",
        "KaliIR.LoweringCorrectness.lower_preserves_step",
        "KaliIR.LoweringCorrectness.lower_preserves_steps",
    ];
    let lowering_theorem_docs = [
        "proofs/BOUNDARY.md",
        "plan/phase-4/02-formal-verification-depth.md",
        "PLAN-4.2-STATUS.md",
        "TODO.md",
    ];

    for relative in lowering_theorem_docs {
        let text = fs::read_to_string(root.join(relative)).expect("read lowering doc");
        for theorem in lowering_theorem_names {
            assert!(
                text.contains(theorem),
                "{relative} is missing lowering theorem name: {theorem}"
            );
        }
    }

    let theorem_only_docs = ["TODO.md"];

    for relative in theorem_only_docs {
        let text = fs::read_to_string(root.join(relative)).expect("read tracker doc");
        for theorem in rc_theorem_names {
            assert!(
                text.contains(theorem),
                "{relative} is missing proof-summary theorem name: {theorem}"
            );
        }
        for term in rc_predicate_vocabulary {
            assert!(
                text.contains(term),
                "{relative} is missing RC predicate vocabulary term: {term}"
            );
        }
    }

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
        workflow.contains("leanprover/lean4-action@v1"),
        "proof-check job should install the Lean toolchain"
    );
    assert!(
        workflow.contains("cd proofs && lake build"),
        "proof-check job should build the proofs workspace"
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
