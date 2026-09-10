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
use kali_error::_error_codes::e5;
use kali_error::Diagnostic;
use kali_runtime::RuntimeCtx;
use kali_sandbox::SandboxPolicy;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
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
    /// Refused at every tier, on purpose, with the SPECIFIC diagnostic code
    /// every tier's refusal must carry. Without this, a surface mismatch or
    /// any other harness-caused refusal is indistinguishable from a genuine,
    /// intentional fail-closed rejection -- the gate would read either as
    /// "refused," which is not the same claim.
    RefusedEverywhere { code: u32 },
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

/// Every fixture this gate compiles under a real API surface, not a default
/// that happens to be convenient. `fasta-benchmark-v1` reads `process.argv`,
/// a Node-only global; no fixture's committed metadata declares an API
/// surface (checked across all 68 `.json` files -- none has such a field), so
/// this is an empirical, by-name determination rather than a schema-driven
/// default: compiling fasta under `ApiSurface::Deno` (this gate's otherwise
/// uniform choice) made `process` an undefined identifier and produced a
/// harness artifact (E3100), not a measurement of the fixture. Every other
/// fixture is unaffected by this and keeps `ApiSurface::Deno`.
fn api_surface_for(stem: &str) -> ApiSurface {
    if stem == "fasta-benchmark-v1" {
        ApiSurface::Node
    } else {
        ApiSurface::Deno
    }
}

/// `<stem>.policy.json` next to a fixture's source, when one exists, loaded
/// as a real `SandboxPolicy` (not fabricated capability: `RuntimeCtx::new`
/// already takes `Option<SandboxPolicy>`, and three fixtures already ship a
/// policy file for exactly this purpose -- `fasta-benchmark-v1` and
/// `mandelbrot-benchmark-v1` ship one and both are used here. Reading a
/// committed `.policy.json` is reading, not editing -- the fixtures-immutable
/// constraint is unaffected.
///
/// `binary-trees-benchmark-v1` is the one EXCEPTION. Not because of its
/// policy or the sandbox at all -- the actual finding is more severe than
/// what this function exists to solve: COMPILING (not executing) this
/// fixture at `--release`/`--release-advanced` crashes the whole PROCESS with
/// a native stack overflow (`fatal runtime error: stack overflow, aborting`,
/// SIGABRT), isolated all the way down to a bare `compile_source_file` call
/// with no `RuntimeCtx` involved. Confirmed via an external `kali build
/// --release` SUBPROCESS on the real fixture too (crashed there, safely). A
/// native stack overflow cannot be caught by a Rust assertion, so this
/// fixture's `--release`/`--release-advanced` tiers are never compiled
/// in-process at all -- see the dedicated, subprocess-isolated check in the
/// `KnownBroken` arm below. Returning `None` here for this stem is
/// precautionary (nothing currently calls `RuntimeCtx::new` with this
/// fixture's policy at any tier), not the actual mitigation.
fn sibling_policy(stem: &str) -> Option<SandboxPolicy> {
    if stem == "binary-trees-benchmark-v1" {
        return None;
    }
    let path = benchmarks_dir().join(format!("{stem}.policy.json"));
    if !path.exists() {
        return None;
    }
    Some(
        SandboxPolicy::from_file(&path)
            .unwrap_or_else(|d| panic!("{stem}: failed to load {path:?}: {d:?}")),
    )
}

/// The oracle's `node --version`, captured once and surfaced in every panic
/// message that compares against it -- a different `node` in CI silently
/// changes the oracle for all 68 entries, so a mismatch should say which
/// node produced the expected value.
fn node_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION.get_or_init(|| {
        let output = Command::new("node")
            .arg("--version")
            .output()
            .expect("run node --version -- node must be on PATH; CI installs it");
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    })
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
        "{stem}: node ({}) failed on the wrapped fixture; stderr: {}",
        node_version(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Compiles `source` (the WRAPPED copy) under `mode`. Writes to a temp file
/// named with the fixture's REAL extension (`.ts` or `.js`, matching
/// `fixture_source`), not a fixed `.ts` -- otherwise the 15 `.js`-sourced
/// fixtures would never traverse the `.js` input path they exist to cover.
/// Uses `api_surface_for(stem)`, not a fixed surface, for the same reason.
fn compile_wrapped(source: &str, stem: &str, mode: BuildMode) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let extension = fixture_source(stem)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("ts")
        .to_string();
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join(format!("{stem}.{extension}"));
    fs::write(&path, source).expect("write wrapped source");
    compile_source_file(&path, mode, api_surface_for(stem), &[], false, false)
}

fn assert_fixture(stem: &str, expectation: Expectation) {
    let committed = fs::read_to_string(fixture_source(stem))
        .unwrap_or_else(|e| panic!("{stem}: read source: {e}"));
    let wrapped = wrap_terminal_call(&committed, stem);

    // Every fixture's own sibling sandbox policy, when it ships one --
    // applied uniformly, not just for the one fixture that happens to need
    // it. See `sibling_policy`'s doc comment.
    let policy = sibling_policy(stem);

    match expectation {
        Expectation::Runs => {
            let expected = node_oracle(&wrapped, stem);
            for mode in [
                BuildMode::Fast,
                BuildMode::Release,
                BuildMode::ReleaseAdvanced,
            ] {
                let wasm = compile_wrapped(&wrapped, stem, mode)
                    .unwrap_or_else(|d| panic!("{stem} {mode:?}: expected a build, got {d:?}"));
                let outcome = RuntimeCtx::new(policy.clone())
                    .execute(&wasm)
                    .unwrap_or_else(|d| panic!("{stem} {mode:?}: module did not run: {d:?}"));
                assert_eq!(
                    outcome.stdout,
                    expected,
                    "{stem} {mode:?}: disagrees with node ({})",
                    node_version()
                );
            }
        }
        Expectation::RefusedAtRelease => {
            let expected = node_oracle(&wrapped, stem);
            let wasm = compile_wrapped(&wrapped, stem, BuildMode::Fast)
                .unwrap_or_else(|d| panic!("{stem} --fast: must still build, got {d:?}"));
            let outcome = RuntimeCtx::new(policy.clone())
                .execute(&wasm)
                .unwrap_or_else(|d| panic!("{stem} --fast: must still run: {d:?}"));
            assert_eq!(
                outcome.stdout,
                expected,
                "{stem} --fast: disagrees with node ({})",
                node_version()
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
        Expectation::RefusedEverywhere { code } => {
            for mode in [
                BuildMode::Fast,
                BuildMode::Release,
                BuildMode::ReleaseAdvanced,
            ] {
                let diagnostics = compile_wrapped(&wrapped, stem, mode)
                    .err()
                    .unwrap_or_else(|| panic!("{stem} {mode:?}: expected a refusal at every tier"));
                assert!(
                    diagnostics.iter().any(|d| d.code == Some(code)),
                    "{stem} {mode:?}: expected a refusal carrying diagnostic code {code}, got \
                     {diagnostics:?}"
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
            //
            // But a node failure alone must never be sufficient TO PASS this
            // arm: every tier is compiled and executed regardless of whether
            // an oracle exists, and this arm requires having OBSERVED at
            // least one genuine tier failure (a build refusal, an execute
            // failure, or a value disagreement) before it may pass. Without
            // that requirement, a fixture whose oracle simply cannot run
            // would pass vacuously -- without the gate ever having compiled
            // anything -- which is exactly the blindness this gate exists to
            // remove.
            //
            // `binary-trees-benchmark-v1` gets its own dedicated, falsifiable
            // check instead of the generic path below, and deliberately NEVER
            // calls `compile_wrapped` (`compile_source_file`) for its
            // `--release`/`--release-advanced` tiers in-process. Measured
            // directly, THREE times: compiling (not executing -- this was
            // isolated all the way down to a bare `compile_wrapped` call with
            // no `RuntimeCtx` involved at all) this fixture's `--release`
            // crashes the whole PROCESS with a native stack overflow (`fatal
            // runtime error: stack overflow, aborting`, SIGABRT). Confirmed a
            // fourth time via a `kali build --release`/`--release-advanced`
            // SUBPROCESS on the real, committed fixture (crashed there too,
            // safely -- a subprocess's abort cannot bring this process down
            // with it). This is a genuine COMPILER crash on this specific
            // fixture's deeply recursive `bottomUpTree`/`itemCheck` shape at
            // N=21, unrelated to execution, fuel, or any sandbox policy.
            // `--fast` is unaffected and compiles in-process safely.
            if stem == "binary-trees-benchmark-v1" {
                compile_wrapped(&wrapped, stem, BuildMode::Fast)
                    .unwrap_or_else(|d| panic!("{stem} Fast: expected a clean build, got {d:?}"));

                let kali_bin = std::env::var("CARGO_BIN_EXE_kali")
                    .expect("CARGO_BIN_EXE_kali -- kali_cli's own [[bin]] must be built");
                for flag in ["--release", "--release-advanced"] {
                    let out_dir = tempdir().expect("tempdir");
                    let output = Command::new(&kali_bin)
                        .arg("build")
                        .arg(flag)
                        .arg("--out-dir")
                        .arg(out_dir.path())
                        .arg(fixture_source(stem))
                        .output()
                        .expect("run kali build in a subprocess");
                    #[cfg(unix)]
                    let crashed = {
                        use std::os::unix::process::ExitStatusExt;
                        output.status.signal().is_some()
                    };
                    #[cfg(not(unix))]
                    let crashed = !output.status.success();
                    assert!(
                        crashed,
                        "{stem} {flag}: this KNOWN-BROKEN entry no longer crashes the compiler \
                         (subprocess exit {:?}, stderr: {}). Do NOT delete this entry -- \
                         verify independently, then reclassify. Recorded reason: {reason}",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                return;
            }

            let dir = tempdir().expect("tempdir");
            let path = dir.path().join("oracle.js");
            fs::write(&path, &wrapped).expect("write oracle source");
            let node_output = Command::new("node")
                .arg(&path)
                .output()
                .expect("run node -- the oracle must be on PATH; CI installs it");
            let expected: Option<String> = node_output
                .status
                .success()
                .then(|| String::from_utf8_lossy(&node_output.stdout).into_owned());

            // `mandelbrot-benchmark-v1` has no node oracle at all
            // (`Kali.writeStdoutBytes` is a Kali-only host API, so node
            // throws instead of running) but DOES ship a golden: its raw
            // stdout BYTES (the separate `stdout_bytes` channel
            // `Kali.writeStdoutBytes` populates, not the `console.log`
            // string channel `outcome.stdout` is) are checked directly
            // against the committed `mandelbrot-benchmark-v1.expected.pbm`,
            // so this entry's pin is a real, falsifiable measurement rather
            // than an inference from node's failure.
            let golden_bytes = if stem == "mandelbrot-benchmark-v1" {
                Some(
                    fs::read(benchmarks_dir().join(format!("{stem}.expected.pbm")))
                        .unwrap_or_else(|e| panic!("{stem}: read golden .expected.pbm: {e}")),
                )
            } else {
                None
            };

            let mut observed_failure = false;
            let mut all_tiers_agree = expected.is_some() || golden_bytes.is_some();
            for mode in [
                BuildMode::Fast,
                BuildMode::Release,
                BuildMode::ReleaseAdvanced,
            ] {
                let tier_ok = match compile_wrapped(&wrapped, stem, mode) {
                    Ok(wasm) => match RuntimeCtx::new(policy.clone()).execute(&wasm) {
                        Ok(outcome) => {
                            if let Some(golden) = &golden_bytes {
                                outcome.stdout_bytes == *golden
                            } else if let Some(exp) = &expected {
                                outcome.stdout == *exp
                            } else {
                                // No oracle of any kind for this fixture: compiling
                                // and running cleanly is the most this arm can
                                // observe for this tier.
                                true
                            }
                        }
                        Err(_) => false,
                    },
                    Err(_) => false,
                };
                if !tier_ok {
                    observed_failure = true;
                    all_tiers_agree = false;
                }
            }

            assert!(
                observed_failure,
                "{stem}: this KNOWN-BROKEN pin never observed a single failing tier -- every \
                 tier this arm checked compiled, ran, and (where an oracle exists) agreed \
                 cleanly. The pin is now vacuous, not measured. Verify independently and \
                 reclassify -- do not leave a KnownBroken entry that cannot fail. Recorded \
                 reason: {reason}"
            );
            assert!(
                !all_tiers_agree,
                "{stem}: this KNOWN-BROKEN entry now builds, runs and agrees with node ({}) at \
                 all three tiers. Do NOT delete this entry -- verify independently, then move \
                 it to Expectation::Runs. Recorded reason: {reason}",
                node_version()
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
    // runtime_smoke/misc.rs; kept here so the table covers the directory. The
    // refusal must carry E5506 specifically -- any other code would mean this
    // gate is looking at a different (possibly harness-caused) refusal.
    (
        "array-literal-arguments-benchmark-v1",
        Expectation::RefusedEverywhere {
            code: e5::FEATURE_UNAVAILABLE as u32,
        },
    ),
    // Unclassified fixture, classified empirically. `fastaRandom`/`fastaRepeat`
    // read `process.argv[2]`, a Node-only global; compiled under
    // `ApiSurface::Node` (`api_surface_for`; no fixture metadata declares a
    // surface, so this is by-name) it builds and runs at `--fast`, matching
    // node's header lines. `--release`/`--release-advanced` refuse outright:
    // `error[E5506]: reading module binding 'ALU'/'IUB'/'HomoSap' from a
    // function is only available for compile-time-constant const
    // initializers`, plus `error[E5506]: for..in is only supported over an
    // object with a compile-time-known shape` and an `E3100` fallback warning
    // for `.join`. A genuine, tier-split refusal over closing over module-level
    // `const` bindings from inside a function -- unrelated to arrays or to
    // this project's fix. See task-5-report.md.
    (
        "fasta-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Under the correct ApiSurface::Node surface: Fast builds and runs (header \
                     lines match node); Release/ReleaseAdvanced refuse with E5506 (module-level \
                     const bindings read from inside a function) plus for..in shape errors. \
                     Unrelated to this project. See task-5-report.md.",
        },
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
    // Unclassified fixture, classified empirically, then reclassified twice --
    // the second time overturning the first. Round 2 pinned this as a
    // harness/default-fuel-budget mismatch (the canonical N=21 workload needs
    // ~32B fuel; the no-policy default grants 60M). Round 4 measured deeper:
    // this fixture's `--release` COMPILE (not execution -- isolated down to a
    // bare `compile_source_file` call, no `RuntimeCtx` involved) crashes the
    // whole process with a native stack overflow (SIGABRT), confirmed a
    // second way via an external `kali build --release` subprocess on the
    // real fixture (crashed there too, safely). This is a genuine COMPILER
    // defect on this fixture's deeply recursive shape at N=21, unrelated to
    // fuel, sandboxing, or execution. A native stack overflow cannot be
    // caught by a Rust assertion, so `assert_fixture`'s dedicated special
    // case for this stem never compiles `--release`/`--release-advanced`
    // in-process at all -- its "observed failure" evidence comes from
    // subprocess-isolated `kali build` calls instead, asserting the
    // subprocess terminated by signal (a real crash), not merely a nonzero
    // exit. See `sibling_policy`'s doc comment for the full account.
    (
        "binary-trees-benchmark-v1",
        Expectation::KnownBroken {
            reason: "Compiling (not executing) this fixture at --release/--release-advanced \
                     crashes the compiler with a native stack overflow (SIGABRT) -- confirmed \
                     in-process (isolated to a bare compile_source_file call) and via an \
                     external kali-build subprocess. Never compiled in-process for those two \
                     tiers by this gate. --fast is unaffected. See task-5-report.md.",
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

/// Committed `.ts`/`.js` sources with NO fixture metadata -- `on_disk` in
/// `the_table_covers_every_fixture_with_metadata` above is built from `.json`
/// files, so a source file with a `.policy.json` but no metadata `.json` is
/// invisible to that test, to `FIXTURES`, and to this gate. `join-loop-peak.ts`
/// and `string-loop-peak.ts` are exactly this: each has a `.policy.json` but no
/// fixture `.json`. This is an explicit, commented out-of-scope list, not a
/// silent gap -- the test below fails the moment a new source file is added
/// without either metadata or an entry here.
const SOURCES_WITHOUT_METADATA: &[&str] = &["join-loop-peak", "string-loop-peak"];

#[test]
fn every_source_file_has_metadata_or_is_explicitly_out_of_scope() {
    let mut stems_on_disk: Vec<String> = fs::read_dir(benchmarks_dir())
        .expect("read benchmarks dir")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            let stem = name
                .strip_suffix(".ts")
                .or_else(|| name.strip_suffix(".js"))?;
            Some(stem.to_string())
        })
        .collect();
    stems_on_disk.sort();
    stems_on_disk.dedup();

    let mut accounted: Vec<String> = FIXTURES
        .iter()
        .map(|(s, _)| s.to_string())
        .chain(SOURCES_WITHOUT_METADATA.iter().map(|s| s.to_string()))
        .collect();
    accounted.sort();
    accounted.dedup();

    assert_eq!(
        stems_on_disk, accounted,
        "a `.ts`/`.js` source file in the benchmarks directory has neither a `FIXTURES` \
         entry nor an entry in `SOURCES_WITHOUT_METADATA` -- this gate, and the coverage \
         test above, would silently never see it. Add a `FIXTURES` entry (with metadata) or \
         an out-of-scope entry (without) -- do not filter."
    );
}
