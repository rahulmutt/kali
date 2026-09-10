//! The benchmark fixtures are COMPILED AND RUN at all three tiers, and their
//! output is compared against node.
//!
//! WHY THIS EXISTS. `assert_optimization_benchmark_fixture`
//! (`crates/kali_cli/tests/runtime_smoke.rs:6061`) builds each fixture at three
//! tiers and counts instructions, adds and tag-boxing ops in the emitted
//! `.wasm`. It never executes the module. So a build that emitted zeros
//! measured as a passing benchmark, and a build that emitted an UNLOADABLE
//! module did too -- which is how the release-tier allocation-identity defect
//! stayed invisible while three Benchmarks Game fixtures were silently wrong or
//! refused. A benchmark harness that never runs the artefact is not a
//! correctness gate, and that one was being read as one.
//!
//! Spec: `docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md` §4
//!
//! MEASURED FALSE, 2026-09-10. This module's own design assumed that a
//! wrapped fixture whose call returns `undefined` is a liveness check rather
//! than a value oracle, because "node prints `undefined` and so should every
//! tier." It does not: `console.log(undefined)` prints `"0"` under every
//! build tier of this compiler, not `"undefined"`. Confirmed via a minimal
//! repro (`function f() {} console.log(f());`), uniform across `--fast`,
//! `--release` and `--release-advanced`. Any fixture this would affect is
//! classified `Expectation::KnownBroken` rather than `Runs`, with that
//! diagnosis as its reason.

use kali_cli::build::{compile_source_file, BuildMode};
use kali_cli::ApiSurface;
use kali_runtime::RuntimeCtx;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Expectation {
    /// Builds and runs at all three tiers, agreeing with node.
    Runs,
    /// Builds at `--fast` and is REFUSED at both release tiers. Every entry
    /// here needs a comment saying why the refusal is the correct outcome.
    ///
    /// No fixture in the current directory needs this arm: Ruling 1 restores
    /// the three Benchmarks Game fixtures to `Runs`, and every other
    /// non-`Runs` fixture this gate found is either a full-tier refusal
    /// (`RefusedEverywhere`) or a load-time failure rather than a diagnosed
    /// build refusal (`KnownBroken`). Kept for the next fixture that is a
    /// genuine, diagnosed release-only refusal; `#[allow(dead_code)]` below is
    /// deliberate, not an oversight.
    #[allow(dead_code)]
    RefusedAtRelease,
    /// Refused at every tier, on purpose.
    RefusedEverywhere,
    /// A fixture whose measured behaviour is BROKEN, pre-existing, and not
    /// caused or fixed by this project. Pinned rather than excluded: the gate
    /// asserts the brokenness is still there, so whoever fixes the underlying
    /// defect is told by a red test to reclassify this entry to `Runs`.
    /// Every entry carries a comment with the exact diagnostic, the tier
    /// pattern, and the followup it is filed under.
    KnownBroken { reason: &'static str },
}

fn benchmarks_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/benchmarks")
}

/// Resolve a fixture stem to its committed source, `.ts` preferred over `.js`.
fn fixture_source(stem: &str) -> PathBuf {
    let dir = benchmarks_dir();
    let ts = dir.join(format!("{stem}.ts"));
    if ts.exists() {
        return ts;
    }
    let js = dir.join(format!("{stem}.js"));
    assert!(
        js.exists(),
        "no .ts or .js source for benchmark stem {stem}"
    );
    js
}

/// Every benchmark fixture ends in a bare call -- `entry();`, `hot(1, 2);`,
/// `hot(true);` -- and NOT ONE of the fixtures in the measured list contains a
/// `console.log`. Executing them as committed proves only that the module
/// loads. So the terminal call is wrapped as `console.log(<call>)` in a COPY.
///
/// The copy is written to a temp file. It is never passed to
/// `assert_optimization_benchmark_fixture` and never hashed against the
/// fixture's `sourceSha256`, so the measured workload and the pinned metadata
/// are untouched. This mirrors the EXECUTION GUARD pattern already used for
/// three fixtures in `crates/kali_cli/tests/runtime_smoke/misc.rs`.
///
/// Fails loudly rather than skipping: a fixture whose tail does not match is a
/// fixture this gate would silently not be testing.
fn wrap_terminal_call(source: &str, stem: &str) -> String {
    // The three Benchmarks Game fixtures already print their real, meaningful
    // output via `console.log` calls INSIDE the function this tail calls
    // (spec §4.2: "The Benchmarks Game fixtures do print, which is why the
    // three being restored get a real value oracle without wrapping.").
    // Wrapping their tail anyway would only observe the call's RETURN value,
    // which is `undefined` for all three -- adding a meaningless trailing
    // line that exposes an unrelated, pre-existing defect (`console.log
    // (undefined)` prints `"0"` under this compiler, not `"undefined"` -- see
    // the module doc comment) instead of proving anything about the fixture.
    // So these three are used AS COMMITTED, unwrapped.
    if matches!(
        stem,
        "spectral-norm-benchmark-v1" | "nbody-benchmark-v1" | "fannkuch-redux-benchmark-v1"
    ) {
        return source.to_string();
    }

    let mut lines: Vec<&str> = source.lines().collect();
    let idx = lines
        .iter()
        .rposition(|line| {
            let t = line.trim();
            !t.is_empty() && !t.starts_with("//")
        })
        .unwrap_or_else(|| panic!("{stem}: source has no statements"));

    let trimmed = lines[idx].trim();
    let call = trimmed.strip_suffix(';').unwrap_or_else(|| {
        panic!("{stem}: expected the fixture to end in a bare call statement, found: {trimmed}")
    });
    // Two fixtures' tails are genuinely NOT a call: `division-by-one`'s is a
    // sum of two calls plus a constant (`hot(1) + hot(1) + folded;`), and
    // `object-enumeration-delete-reinsert`'s is a bare identifier reference to
    // a `const` computed just above it (`total;`). Both are side-effect-free
    // expression statements, safe to wrap in `console.log(...)` exactly like a
    // call would be. Per this task's own guidance, the call-shape assertion is
    // NOT widened to admit non-call tails generally -- these two are admitted
    // narrowly, by exact stem, so every other fixture still fails loudly if
    // its tail does not match.
    let is_known_non_call_tail = matches!(
        stem,
        "division-by-one-benchmark-v1" | "object-enumeration-delete-reinsert-benchmark-v1"
    );
    if !is_known_non_call_tail {
        assert!(
            call.ends_with(')') && call.contains('('),
            "{stem}: expected the fixture to end in a bare call statement, found: {trimmed}"
        );
    }

    let wrapped = format!("console.log({call});");
    lines[idx] = &wrapped;
    lines.join("\n") + "\n"
}

/// Run `source` under node and return its stdout.
fn node_oracle(source: &str, stem: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("oracle.js");
    fs::write(&path, source).expect("write oracle source");
    let output = Command::new("node")
        .arg(&path)
        .output()
        .expect("run node -- the oracle must be on PATH; CI installs it");
    assert!(
        output.status.success(),
        "{stem}: node failed on the wrapped fixture; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn compile_wrapped(source: &str, stem: &str, mode: BuildMode) -> Result<Vec<u8>, String> {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join(format!("{stem}.ts"));
    fs::write(&path, source).expect("write wrapped source");
    compile_source_file(&path, mode, ApiSurface::Deno, &[], false, false)
        .map_err(|d| format!("{d:?}"))
}

fn assert_fixture(stem: &str, expectation: Expectation) {
    let committed = fs::read_to_string(fixture_source(stem))
        .unwrap_or_else(|e| panic!("{stem}: read source: {e}"));
    let wrapped = wrap_terminal_call(&committed, stem);

    match expectation {
        Expectation::Runs => {
            let expected = node_oracle(&wrapped, stem);
            for mode in [
                BuildMode::Fast,
                BuildMode::Release,
                BuildMode::ReleaseAdvanced,
            ] {
                let wasm = compile_wrapped(&wrapped, stem, mode)
                    .unwrap_or_else(|d| panic!("{stem} {mode:?}: expected a build, got {d}"));
                let outcome = RuntimeCtx::new(None)
                    .execute(&wasm)
                    .unwrap_or_else(|d| panic!("{stem} {mode:?}: module did not run: {d:?}"));
                assert_eq!(
                    outcome.stdout, expected,
                    "{stem} {mode:?}: disagrees with node"
                );
            }
        }
        Expectation::RefusedAtRelease => {
            let expected = node_oracle(&wrapped, stem);
            let wasm = compile_wrapped(&wrapped, stem, BuildMode::Fast)
                .unwrap_or_else(|d| panic!("{stem} --fast: must still build, got {d}"));
            let outcome = RuntimeCtx::new(None)
                .execute(&wasm)
                .unwrap_or_else(|d| panic!("{stem} --fast: must still run: {d:?}"));
            assert_eq!(
                outcome.stdout, expected,
                "{stem} --fast: disagrees with node"
            );

            for mode in [BuildMode::Release, BuildMode::ReleaseAdvanced] {
                assert!(
                    compile_wrapped(&wrapped, stem, mode).is_err(),
                    "{stem} {mode:?}: expected the honest refusal, not a build. If this now \
                     builds, do NOT relax this entry -- verify the module RUNS and agrees \
                     with node, then move the entry to Expectation::Runs."
                );
            }
        }
        Expectation::RefusedEverywhere => {
            for mode in [
                BuildMode::Fast,
                BuildMode::Release,
                BuildMode::ReleaseAdvanced,
            ] {
                assert!(
                    compile_wrapped(&wrapped, stem, mode).is_err(),
                    "{stem} {mode:?}: expected a refusal at every tier"
                );
            }
        }
        Expectation::KnownBroken { reason } => {
            // A real assertion, not a hole: this fails the moment the underlying
            // defect is fixed, which is the direction that matters for a defect
            // pin. A node oracle may not even be obtainable (e.g. a fixture that
            // calls a Kali-only host API) -- that alone is sufficient evidence
            // the fixture does not satisfy `Runs`, so the oracle is attempted
            // without the `node_oracle` helper's hard `assert!` on success.
            let dir = tempdir().expect("tempdir");
            let path = dir.path().join("oracle.js");
            fs::write(&path, &wrapped).expect("write oracle source");
            let node_output = Command::new("node")
                .arg(&path)
                .output()
                .expect("run node -- the oracle must be on PATH; CI installs it");
            let expected = String::from_utf8_lossy(&node_output.stdout).into_owned();

            let mut fully_runs = node_output.status.success();
            if fully_runs {
                for mode in [
                    BuildMode::Fast,
                    BuildMode::Release,
                    BuildMode::ReleaseAdvanced,
                ] {
                    let agrees = match compile_wrapped(&wrapped, stem, mode) {
                        Ok(wasm) => match RuntimeCtx::new(None).execute(&wasm) {
                            Ok(outcome) => outcome.stdout == expected,
                            Err(_) => false,
                        },
                        Err(_) => false,
                    };
                    if !agrees {
                        fully_runs = false;
                        break;
                    }
                }
            }

            assert!(
                !fully_runs,
                "{stem}: this KNOWN-BROKEN entry now builds, runs and agrees with node at all \
                 three tiers. Do NOT delete this entry -- verify independently, then move it \
                 to Expectation::Runs. Recorded reason: {reason}"
            );
        }
    }
}

/// Every benchmark fixture with committed metadata, and what it is expected to
/// do. A fixture missing from this table fails `the_table_covers_every_fixture`
/// below -- adding a fixture without classifying it is a gate that silently
/// does not test it.
pub(crate) const FIXTURES: &[(&str, Expectation)] = &[
    (
        "math-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"24\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "math-benchmark-v1-js",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"24\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "math-trunc-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Fast/Release build but the wasm fails to load (E4201, wasm[0]::function[41]); ReleaseAdvanced loads and agrees with node (24). Pre-existing, confirmed at baseline eec408d000. See task-5-report.md.",
        },
    ),
    ("math-imul-benchmark-v1", Expectation::Runs),
    ("math-imul-benchmark-v1-js", Expectation::Runs),
    ("math-clz32-benchmark-v1", Expectation::Runs),
    ("math-clz32-benchmark-v1-js", Expectation::Runs),
    (
        "math-ceil-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Fast/Release build but the wasm fails to load (E4201, wasm[0]::function[41]); ReleaseAdvanced loads and agrees with node (24). Pre-existing, confirmed at baseline eec408d000. See task-5-report.md.",
        },
    ),
    ("math-abs-sign-benchmark-v1", Expectation::Runs),
    ("math-abs-sign-benchmark-v1-js", Expectation::Runs),
    ("math-max-min-benchmark-v1", Expectation::Runs),
    ("math-max-min-benchmark-v1-js", Expectation::Runs),
    ("math-floor-benchmark-v1", Expectation::Runs),
    ("math-floor-benchmark-v1-js", Expectation::Runs),
    ("math-round-benchmark-v1", Expectation::Runs),
    ("math-round-benchmark-v1-js", Expectation::Runs),
    ("math-pow-benchmark-v1", Expectation::Runs),
    ("math-pow-benchmark-v1-js", Expectation::Runs),
    ("division-by-one-benchmark-v1", Expectation::Runs),
    (
        "multiplication-by-one-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"24\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    ("dead-branch-elimination-benchmark-v1", Expectation::Runs),
    (
        "dead-inlined-function-pruning-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"39\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "call-inlining-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"24\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    ("closure-inlining-benchmark-v1", Expectation::Runs),
    (
        "nested-call-inlining-chain-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"2\" where node/Fast say \"42\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-enumeration-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"9\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-string-enumeration-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"12\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "reflect-own-keys-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"3\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "reflect-own-keys-const-bound-literal-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"3\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "reflect-own-keys-alias-chain-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"3\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "integer-like-object-enumeration-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"21\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-enumeration-alias-chain-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"9\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-enumeration-alias-chain-benchmark-v1-js",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"9\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-enumeration-const-bound-literal-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"9\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-enumeration-delete-reinsert-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"10\" where node/Fast say \"15\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-literal-property-order-canonicalization-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"9\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "object-literal-property-order-canonicalization-benchmark-v1-js",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"9\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "identity-chain-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"38\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    ("nested-wrapper-pruning-benchmark-v1", Expectation::Runs),
    (
        "algebraic-simplification-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"24\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "duplicate-pure-expression-elimination-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"140\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    ("nullish-specialization-repeat-benchmark-v1", Expectation::Runs),
    (
        "specialization-reuse-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"3\" where node/Fast say \"114\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "bigint-literal-arguments-benchmark-v1",
        Expectation::KnownBroken {
            reason: "BigInt literal prints \"112\" (missing the 'n' suffix) at every tier, node says \"112n\"; Release/ReleaseAdvanced additionally print \"3\". Pre-existing, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "bigint-addition-chain-benchmark-v1",
        Expectation::KnownBroken {
            reason: "BigInt literal prints \"112\" (missing the 'n' suffix) at every tier, node says \"112n\"; Release/ReleaseAdvanced additionally print \"3\". Pre-existing, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "bigint-multiplication-chain-benchmark-v1",
        Expectation::KnownBroken {
            reason: "BigInt literal prints \"93312\" (missing the 'n' suffix) at every tier, node says \"93312n\"; Release/ReleaseAdvanced additionally print \"1\". Pre-existing, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "numeric-literal-arguments-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"4\" where node/Fast say \"151\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "boolean-literal-arguments-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"36\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "branch-specialization-repeat-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"36\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "const-array-element-access-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"23\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "const-object-property-access-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"35\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "math-variant-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"24\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "math-variant-benchmark-v1-js",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"24\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "string-concatenation-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"start-ahead-of-time-end\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "template-literal-concatenation-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"start-ahead-of-time-end\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    (
        "template-literal-concatenation-benchmark-v1-js",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"start-ahead-of-time-end\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    ("layout-specialization-benchmark-v1", Expectation::Runs),
    (
        "call-inlining-chain-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Release/ReleaseAdvanced print \"1\" where node/Fast say \"27\" -- silent miscompile, no diagnostic. Pre-existing, confirmed at baseline eec408d000, unrelated to this project. See task-5-report.md.",
        },
    ),
    ("nullish-benchmark-v1", Expectation::Runs),

    // The three Benchmarks Game fixtures, per Ruling 1 (round 2). Each is
    // measured directly through this gate, unwrapped (see `wrap_terminal_call`).
    (
        "spectral-norm-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Builds at every tier; Fast agrees with node (1.274219991), but Release/ReleaseAdvanced fail to load the wasm (E4201, wasm[0]::function[46]/[41]). Confirmed NOT entering the spec env this project's fix narrowed (is_specializable_binding declines it) -- a different, still-open mechanism (Ruling 2, round 2: \"different mechanism\"). See task-5-report.md.",
        },
    ),
    (
        "nbody-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Fast builds and agrees with node (both energy lines); Release/ReleaseAdvanced refuse to build with E5506 (computed member access unavailable) plus an E8001 warning cascade. Confirmed NOT entering the spec env this project's fix narrowed -- a different, still-open mechanism (Ruling 2, round 2: \"different mechanism\"). See task-5-report.md.",
        },
    ),
    // Genuinely fixed by this project: builds, loads, runs, and matches node
    // byte-for-byte at all three tiers ("228\nPfannkuchen(7) = 16\n").
    ("fannkuch-redux-benchmark-v1", Expectation::Runs),
    // Deliberately rejected fail-closed: passing an array literal to a function
    // had the callee reading zero placeholders, a silent miscompile. Pinned by
    // `array_literal_arguments_benchmark_is_rejected_fail_closed` in
    // runtime_smoke/misc.rs; kept here so the table covers the directory.
    (
        "array-literal-arguments-benchmark-v1",
        Expectation::RefusedEverywhere,
    ),
    // Unclassified fixture (directory has metadata but was absent from the
    // measured list), classified empirically. `fastaRandom`/`fastaRepeat` read
    // `process.argv[2]`; under `ApiSurface::Deno` (which this gate always
    // compiles under) `process` is not a declared global, so this refuses to
    // build at every tier with the same diagnostic: `error[E3100]: undefined
    // identifier 'process'`. Measured at `--fast`, `--release` and
    // `--release-advanced` via `kali build`; uniform across all three, so this
    // is an API-surface mismatch, not a tier-specific defect.
    (
        "fasta-benchmark-v1",
        Expectation::RefusedEverywhere,
    ),
    (
        "math-ceil-benchmark-v1-js",
        Expectation::KnownBroken {
            reason: "Same wasm[0]::function[41] unloadable-module defect as math-ceil-benchmark-v1/math-trunc-benchmark-v1; Fast/Release fail to load, ReleaseAdvanced agrees with node (24). Pre-existing, baseline eec408d000. See task-5-report.md.",
        },
    ),
    (
        "math-trunc-benchmark-v1-js",
        Expectation::KnownBroken {
            reason: "Same wasm[0]::function[41] unloadable-module defect as math-ceil-benchmark-v1/math-trunc-benchmark-v1; Fast/Release fail to load, ReleaseAdvanced agrees with node (24). Pre-existing, baseline eec408d000. See task-5-report.md.",
        },
    ),
    (
        "mandelbrot-benchmark-v1",
        Expectation::KnownBroken {
            reason: "No node oracle exists (Kali.writeStdoutBytes has no Node equivalent); independently, Release/ReleaseAdvanced fail to load the wasm (E4201, wasm[0]::function[43]/[42]) while Fast matches the committed .expected.pbm golden exactly. Pre-existing, baseline eec408d000. See task-5-report.md.",
        },
    ),
    (
        "binary-trees-benchmark-v1",
        Expectation::KnownBroken {
            reason: "The canonical N=21 workload needs ~32B fuel; this gate's no-policy RuntimeCtx::new(None) grants only the 60M default, so every tier traps with E4003 CPU fuel budget exhausted. Not a compiler defect -- a harness/default-budget mismatch. See task-5-report.md.",
        },
    ),
];

#[test]
fn the_table_covers_every_fixture_with_metadata() {
    let mut on_disk: Vec<String> = fs::read_dir(benchmarks_dir())
        .expect("read benchmarks dir")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            // `*.policy.json` are sandbox policies, not fixture metadata.
            let stem = name.strip_suffix(".json")?;
            if stem.ends_with(".policy") {
                return None;
            }
            Some(stem.to_string())
        })
        .collect();
    on_disk.sort();

    let mut tabled: Vec<String> = FIXTURES.iter().map(|(s, _)| s.to_string()).collect();
    tabled.sort();

    assert_eq!(
        on_disk, tabled,
        "the fixture table and the benchmarks directory disagree. A fixture with \
         metadata but no table entry is one this gate does not execute; a table \
         entry with no fixture is dead. Add or remove entries -- do not filter."
    );
}

#[test]
fn every_benchmark_fixture_runs_and_agrees_with_node() {
    for (stem, expectation) in FIXTURES {
        assert_fixture(stem, *expectation);
    }
}
