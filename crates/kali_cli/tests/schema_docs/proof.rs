use super::*;

#[test]
fn specialized_artifact_metadata_schemas_share_the_base_artifact_contract() {
    let root = repo_root();

    let expected = [
        (
            "schemas/artifact-meta/lib-wit/v1.json",
            "https://kali-lang.org/schemas/artifact-meta/lib-wit/v1",
            "Kali Library WIT Artifact Metadata v1",
        ),
        (
            "schemas/artifact-meta/capi/v1.json",
            "https://kali-lang.org/schemas/artifact-meta/capi/v1",
            "Kali C ABI Artifact Metadata v1",
        ),
        (
            "schemas/artifact-meta/component/v1.json",
            "https://kali-lang.org/schemas/artifact-meta/component/v1",
            "Kali Component Artifact Metadata v1",
        ),
    ];

    for (relative, id, title) in expected {
        let schema: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(relative)).expect("read specialized artifact schema"),
        )
        .expect("parse specialized artifact schema");
        assert_eq!(schema["$id"], id);
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
        ("plan/phase-25/README.md", "proofs/BOUNDARY.md"),
    ];

    for (relative, expected_summary) in summary_docs {
        let text = fs::read_to_string(root.join(relative)).expect("read summary doc");
        assert!(
            text.contains(expected_summary),
            "{relative} is missing the canonical proof-backed summary"
        );
    }

    let stage_doc =
        fs::read_to_string(root.join("plan/phase-25/README.md")).expect("read phase 25 doc");
    assert!(
        stage_doc.contains("proofs/BOUNDARY.md")
            && stage_doc.contains("sole theorem/property inventory"),
        "phase 25 doc should point to the canonical proof boundary without duplicating it"
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
