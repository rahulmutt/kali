use super::*;

#[test]
fn active_plan_tracks_language_semantics_phase() {
    let root = repo_root();
    let plan = fs::read_to_string(root.join("PLAN.md")).expect("read active plan");
    let phase =
        fs::read_to_string(root.join("plan/phase-21/README.md")).expect("read phase 21 README");

    for expected in [
        "Phase 21 — Semantic Completeness and Conformance",
        "Generators and async generators",
        "Iterator and async-iterator protocols",
        "Dynamic language and built-in semantics",
        "Bounded TS/JS inference",
        "Conformance hygiene",
    ] {
        assert!(
            phase.contains(expected),
            "phase 21 README should mention {expected}"
        );
    }

    assert!(
        plan.contains("phase-21/README.md"),
        "top-level PLAN should link the active language phase"
    );
}

#[test]
fn active_plan_removes_historical_phase_dashboards() {
    let root = repo_root();

    for phase in 1..=20 {
        let removed = format!("plan/phase-{phase}");
        assert!(
            !root.join(&removed).exists(),
            "historical phase directory should not remain active: {removed}"
        );
    }

    for removed in [
        "plan/phase-4/02-formal-verification-depth.md",
        "plan/phase-6/conformance-dashboard.md",
        "plan/phase-8/package-corpus-matrix.md",
        "plan/phase-9/optimization-inventory.md",
        "plan/phase-10/README.md",
    ] {
        assert!(
            !root.join(removed).exists(),
            "historical progress snapshot should not remain active: {removed}"
        );
    }
}

#[test]
fn active_plan_tracks_runtime_host_phase() {
    let root = repo_root();
    let phase =
        fs::read_to_string(root.join("plan/phase-22/README.md")).expect("read phase 22 README");

    for expected in [
        "Phase 22 — Host/Runtime Capability Contracts",
        "Threaded runtime semantics",
        "Browser runtime contract",
        "Late host APIs and resources",
        "Late object/runtime APIs",
        "Keep browser-targeted `check` / `build --bundle`",
    ] {
        assert!(
            phase.contains(expected),
            "phase 22 README should mention {expected}"
        );
    }
}

#[test]
fn active_plan_tracks_ecosystem_phase_without_package_matrix_journal() {
    let root = repo_root();
    let phase =
        fs::read_to_string(root.join("plan/phase-23/README.md")).expect("read phase 23 README");

    for expected in [
        "Phase 23 — Ecosystem Compatibility by Rung",
        "Package-corpus stewardship",
        "Node ecosystem breadth",
        "Browser package deployability",
        "Registry-analysis boundaries",
        "support rung",
    ] {
        assert!(
            phase.contains(expected),
            "phase 23 README should mention {expected}"
        );
    }
}

#[test]
fn active_plan_tracks_verification_and_contract_hardening_phase() {
    let root = repo_root();
    let phase =
        fs::read_to_string(root.join("plan/phase-25/README.md")).expect("read phase 25 README");

    for expected in [
        "Phase 25 — Verification and Machine Contracts",
        "Proof-boundary hygiene",
        "Model widening",
        "Proof CI triggers",
        "Schema and CLI contract hardening",
        "proofs/BOUNDARY.md",
    ] {
        assert!(
            phase.contains(expected),
            "phase 25 README should mention {expected}"
        );
    }
}

#[test]
fn active_plan_tracks_optimization_phase_without_inventory_journal() {
    let root = repo_root();
    let phase =
        fs::read_to_string(root.join("plan/phase-24/README.md")).expect("read phase 24 README");

    for expected in [
        "Phase 24 — Optimization and Performance Evidence",
        "Optimization inventory upkeep",
        "Specialization depth",
        "PGO input hardening",
        "Benchmark promotion",
        "fast`, `release`, and `release-advanced`",
        "math-floor-builtin-js",
        "math-round-builtin-js",
        "math-pow-builtin-js",
        "math-trunc-builtin-js",
        "math-ceil-builtin-js",
        "folded-arithmetic-variant-js",
    ] {
        assert!(
            phase.contains(expected),
            "phase 24 README should mention {expected}"
        );
    }
}

#[test]
fn active_plan_tracks_current_state_and_gap_map_benchmark_inventory_updates() {
    let root = repo_root();
    for (doc, expected) in [
        (
            "plan/00-current-state.md",
            "the new `math-round-builtin` / `math-round-builtin-js` pair now does the same for `Math.round`",
        ),
        (
            "plan/00-current-state.md",
            "folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`)",
        ),
        (
            "plan/02-spec-gap-map.md",
            "folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`)",
        ),
    ] {
        let contents = fs::read_to_string(root.join(doc)).expect("read plan doc");
        assert!(
            contents.contains(expected),
            "{doc} should mention the benchmark inventory update"
        );
    }
}
