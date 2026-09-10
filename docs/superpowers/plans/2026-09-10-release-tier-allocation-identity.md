# Release-Tier Allocation Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop the release tiers from substituting a binding's initializer for its name, so that an array produced by an allocating initializer keeps the identity `--fast` gives it — and make the benchmark harness execute what it measures, so this class cannot hide again.

**Architecture:** Two stages in `kali_optimize`. Stage 1 replaces `is_array_literal`'s negative-space definition (`layout.rs:177`, "any text-less `Value` that is not an object literal") with a positive element check, so a lowered `new Array(n).fill(v)` stops qualifying as a specializable binding and never enters the spec env at `specialize.rs:82`/`:109`. Stage 2 makes array-literal-ness *represented* rather than inferred, by adding `LirNodeKind::ArrayLiteral` on the `ComputedMember` precedent. Alongside both, `crates/kali_cli/tests/inprocess/` gains a benchmark-execution gate that compiles and RUNS every benchmark fixture at all three tiers against a `node` oracle.

**Tech Stack:** Rust (workspace, edition per `Cargo.toml`), `wasmtime 47` (linked only into the `inprocess` test target), `node` as the differential oracle, `cargo test`.

**Spec:** `docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md`

## Global Constraints

- **Branch:** `release-tier-allocation-identity`. **Baseline:** `eec408d000`.
- **One cargo target directory.** Do NOT create git worktrees or additional `CARGO_TARGET_DIR`s — extra target dirs have exhausted this pod's disk and killed a run. Check `df -h /workspace` if a build fails oddly.
- **`scripts/test-gate.sh` does not cover clippy or fmt.** `GATE OK` is not CI-green. Before any push run **all three**:
  - `bash scripts/test-gate.sh`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
- **wasmtime links into exactly one test target:** `crates/kali_cli/tests/inprocess.rs` (~450 MB, statically linked once). Every test that EXECUTES wasm goes in a module under `crates/kali_cli/tests/inprocess/` and is registered in `inprocess.rs`. Never create a second wasm-executing test target.
- **Oracle:** `node`. The local version is `v26.8.2`; the followup documents pin `v26.8.1`; CI installs `latest`. Record the version actually used in any measurement written to a document; never claim `v26.8.1` unless it was.
- **Fail-closed direction:** every predicate change in this plan works by *declining* to classify something as a materializable literal. A wrong answer must cost an optimization, never a correct program. If a change would make the optimizer accept MORE shapes, stop — that is not this plan.
- **Fixture sources are immutable.** Never edit a file under `crates/kali_cli/tests/fixtures/benchmarks/` — `sourceSha256` in the sibling `.json` pins it and the measured workload depends on it. Observability is achieved by writing a WRAPPED COPY to a temp file.
- **Line references** in this plan are as of `eec408d000`.

---

## File Structure

| file | responsibility | tasks |
|---|---|---|
| `crates/kali_cli/tests/inprocess/release_allocation_identity.rs` | **create** — the defect's own tests: silent half, loud declarator half, loud parameter half, and the §2.5 shape that must keep refusing | 1, 3, 4 |
| `crates/kali_cli/tests/inprocess.rs` | **modify** — register the two new modules | 1, 5 |
| `crates/kali_optimize/src/layout.rs:177` | **modify** — `is_array_literal` gains the positive element check | 2 |
| `crates/kali_optimize/src/layout_tests.rs` | **modify** — unit tests for the predicate | 2 |
| `crates/kali_cli/tests/inprocess/benchmark_execution.rs` | **create** — the executing gate: wrap, run under node, compile+run at three tiers, compare | 5, 6 |
| `crates/kali_cli/tests/runtime_smoke.rs:6061` | **modify** — drop the two exclusion arms | 6 |
| `crates/kali_cli/tests/runtime_smoke/misc.rs:1374`, `:1631` | **modify** — restore fixtures to the measured list, replace the re-pin block | 6 |
| `crates/kali_cli/tests/clbg_fannkuch_runtime.rs` | **modify** — three-mode assertion back to three | 6 |
| `crates/kali_lir/src/node.rs` | **modify** — add `LirNodeKind::ArrayLiteral` | 7 |
| `crates/kali_hir/src/lowering/expression.rs` | **modify** — tag array literals at lowering | 7 |
| `docs/superpowers/followups/*.md` | **modify/create** — ledger obligations | 8 |

---

## Task 1: Measure the silent half

**This task changes no production code.** Its output is a measurement that either justifies the rest of the plan or falsifies it. Spec §1.2 and §7 row 1.

**Files:**
- Create: `crates/kali_cli/tests/inprocess/release_allocation_identity.rs`
- Modify: `crates/kali_cli/tests/inprocess.rs`

**Interfaces:**
- Consumes: `kali_cli::build::{compile_source_file, BuildMode}`, `kali_cli::ApiSurface`, `kali_runtime::RuntimeCtx`
- Produces: `compile_at(source: &str, mode: BuildMode) -> Result<Vec<u8>, Vec<Diagnostic>>` and `run_at(source: &str, mode: BuildMode) -> String`, used by Tasks 3 and 4.

- [ ] **Step 1: Create the module with its helpers and the failing test**

Create `crates/kali_cli/tests/inprocess/release_allocation_identity.rs`:

```rust
//! The release tiers substitute a const binding's initializer for its name
//! (`kali_optimize/src/specialize.rs:120-127`), so an array produced by an
//! allocating initializer loses the identity `--fast` gives it.
//!
//! Spec: `docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md`
//!
//! These tests must EXECUTE the module, not merely build it: the defect's
//! dangerous half is a literal-index read that builds clean and returns the
//! wrong value. `kali run` takes a source file and has no tier flag, so this
//! cannot be observed through the CLI — hence an in-process test here.

use kali_cli::build::{compile_source_file, BuildMode};
use kali_cli::ApiSurface;
use kali_error::Diagnostic;
use kali_runtime::RuntimeCtx;
use std::fs;
use tempfile::tempdir;

/// Compile `source` at `mode`, returning the wasm bytes or the diagnostics.
pub(crate) fn compile_at(source: &str, mode: BuildMode) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.ts");
    fs::write(&path, source).expect("write source");
    compile_source_file(&path, mode, ApiSurface::Deno, &[], false, false)
}

/// Compile and execute `source` at `mode`, returning captured guest stdout.
pub(crate) fn run_at(source: &str, mode: BuildMode) -> String {
    let wasm = compile_at(source, mode)
        .unwrap_or_else(|d| panic!("compile failed under {mode:?}: {d:?}"));
    RuntimeCtx::new(None)
        .execute(&wasm)
        .unwrap_or_else(|d| panic!("execute failed under {mode:?}: {d:?}"))
        .stdout
}

/// The SILENT half. A literal index has a static name, so it takes the
/// ordinary member lane and never reaches the computed-member gateway — it
/// builds clean at every tier. node prints `7`.
const LITERAL_INDEX: &str = "\
const u = new Array(4).fill(7);
console.log(u[0]);
";

#[test]
fn literal_index_read_off_an_allocation_agrees_with_node_at_every_tier() {
    for mode in [BuildMode::Fast, BuildMode::Release, BuildMode::ReleaseAdvanced] {
        assert_eq!(
            run_at(LITERAL_INDEX, mode),
            "7\n",
            "{mode:?}: a literal-index read off `new Array(4).fill(7)` must be 7, \
             as it is at --fast and under node"
        );
    }
}
```

- [ ] **Step 2: Register the module**

Modify `crates/kali_cli/tests/inprocess.rs`, adding after the existing `#[path]` module declarations (keep them alphabetical):

```rust
#[path = "inprocess/release_allocation_identity.rs"]
mod release_allocation_identity;
```

- [ ] **Step 3: Run the test and RECORD what release actually prints**

Run: `cargo test -p kali_cli --test inprocess literal_index_read_off_an_allocation -- --nocapture`

Expected: `Fast` passes; `Release` fails. **Write down the exact value in the failure message** — that number is this task's deliverable and goes into Task 8's documentation.

**STOP CONDITION.** If `Release` and `ReleaseAdvanced` both print `7`, the silent half does not exist. Do not continue to Task 2. Report the result, and re-scope the project to the loud half only per spec §7 row 1.

- [ ] **Step 4: Commit the failing test**

```bash
git add crates/kali_cli/tests/inprocess/release_allocation_identity.rs crates/kali_cli/tests/inprocess.rs
git commit -m "test(optimize): pin the silent half of the release-tier allocation identity defect

A literal-index read off \`new Array(4).fill(7)\` builds clean at every tier and
is wrong at the release tiers. Nothing in this repository pinned it, because
\`kali run\` takes source only and has no tier flag -- the divergence is not
observable through the CLI. Fails at --release and --release-advanced.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Task 2: Stage 1 — the positive element check

**Files:**
- Modify: `crates/kali_optimize/src/layout.rs:177-186`
- Modify: `crates/kali_optimize/src/layout_tests.rs` (append)

**Interfaces:**
- Consumes: `Optimizer::is_object_literal` (`layout.rs:150`), `parse_literal_text` (`constant_fold.rs:321`)
- Produces: `Optimizer::is_array_literal` with unchanged signature `(&self, &LirProgram, LirNodeId) -> bool`, now positive. Callers (`is_specializable_binding`, `specialize.rs:82`/`:109`) are unchanged.

- [ ] **Step 1: Write the failing unit tests**

Append to `crates/kali_optimize/src/layout_tests.rs`:

```rust
#[test]
fn array_literal_accepts_a_node_whose_children_are_all_literals() {
    let mut builder = LirBuilder::new();
    let array = builder.alloc(LirNodeKind::Value);
    let one = literal(&mut builder, "1");
    let two = literal(&mut builder, "2");
    builder.node_mut(array).unwrap().children = vec![one, two];

    let program = LirProgram {
        root: array,
        nodes: builder.into_nodes(),
    };

    assert!(Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, array));
}

#[test]
fn array_literal_rejects_a_wrapper_around_a_call() {
    // The shape of a lowered `new Array(n).fill(v)` declarator initializer: a
    // text-less `Value` wrapping a `Call`. Under the pre-2026-09-10 definition
    // ("text-less Value that is not an object literal") this returned true, so
    // the binding entered the spec env and every read of the name was
    // overwritten with a clone of this node -- destroying the array identity.
    let mut builder = LirBuilder::new();
    let wrapper = builder.alloc(LirNodeKind::Value);
    let call = builder.alloc(LirNodeKind::Call);
    let callee = builder.alloc_text(LirNodeKind::Value, "Array");
    let arg = literal(&mut builder, "4");
    builder.node_mut(call).unwrap().children = vec![callee, arg];
    builder.node_mut(wrapper).unwrap().children = vec![call];

    let program = LirProgram {
        root: wrapper,
        nodes: builder.into_nodes(),
    };

    assert!(!Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, wrapper));
}

#[test]
fn array_literal_rejects_an_empty_text_less_value() {
    // A text-less `Value` with no children carries no elements to materialize.
    // Declining costs an optimization; accepting invents an empty array.
    let mut builder = LirBuilder::new();
    let empty = builder.alloc(LirNodeKind::Value);

    let program = LirProgram {
        root: empty,
        nodes: builder.into_nodes(),
    };

    assert!(!Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, empty));
}

#[test]
fn array_literal_accepts_a_nested_array_literal() {
    let mut builder = LirBuilder::new();
    let outer = builder.alloc(LirNodeKind::Value);
    let inner = builder.alloc(LirNodeKind::Value);
    let one = literal(&mut builder, "1");
    let two = literal(&mut builder, "2");
    builder.node_mut(inner).unwrap().children = vec![one];
    builder.node_mut(outer).unwrap().children = vec![inner, two];

    let program = LirProgram {
        root: outer,
        nodes: builder.into_nodes(),
    };

    assert!(Optimizer::new(OptimizationLevel::Release).is_array_literal(&program, outer));
}
```

- [ ] **Step 2: Run the unit tests to verify they fail**

Run: `cargo test -p kali_optimize array_literal_`

Expected: `array_literal_rejects_a_wrapper_around_a_call` and `array_literal_rejects_an_empty_text_less_value` FAIL (assertion failed: the predicate returns `true`). The two `accepts` tests pass already.

- [ ] **Step 3: Implement the positive check**

Replace `crates/kali_optimize/src/layout.rs:177-186` with:

```rust
    /// True when `id` is an array literal: a text-less `Value` node whose
    /// children are ALL materializable elements.
    ///
    /// POSITIVE BY CONSTRUCTION, deliberately. Until 2026-09-10 this was
    /// negative space -- "a text-less `Value` that is not an object literal" --
    /// which admitted every shape nobody had thought about. The shape that
    /// mattered was a lowered `new Array(n).fill(v)` declarator initializer (a
    /// text-less `Value` wrapping a `Call`): it qualified, so
    /// `is_specializable_binding` let the binding into the spec env at
    /// `specialize.rs:82`/`:109`, and `specialize.rs:120-127` then overwrote
    /// every read of the name with a clone of the initializer node. The array
    /// the reads indexed was no longer the array the declarator allocated.
    ///
    /// A predicate defined as "not the other thing" cannot be audited. This one
    /// declines anything it cannot positively account for, which costs an
    /// optimization and never a correct program.
    ///
    /// Spec: `docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md` §3.1
    pub(crate) fn is_array_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };
        if node.kind != LirNodeKind::Value || node.text.is_some() || node.children.is_empty() {
            return false;
        }
        if self.is_object_literal(program, id) {
            return false;
        }

        node.children
            .iter()
            .all(|child| self.is_materializable_element(program, *child))
    }

    /// True when `id` can be materialized as an array element without
    /// evaluating anything: a literal, a literal-valued `Value`, or a nested
    /// array/object literal. A `Call`, a `ComputedMember` or an `Unknown` is
    /// NOT materializable -- evaluating it can allocate, and substituting it
    /// would duplicate the allocation rather than copy a value.
    fn is_materializable_element(&self, program: &LirProgram, id: LirNodeId) -> bool {
        let Some(node) = program.nodes.get(id.0 as usize) else {
            return false;
        };

        match node.kind {
            LirNodeKind::Literal => true,
            LirNodeKind::Value if node.children.is_empty() => node
                .text
                .as_deref()
                .and_then(|text| parse_literal_text(Some(text)))
                .is_some(),
            LirNodeKind::Value if node.text.is_none() => {
                self.is_object_literal(program, id) || self.is_array_literal(program, id)
            }
            _ => false,
        }
    }
```

Note: `is_array_literal` now rejects an empty child list, where the old one accepted it. That is the fail-closed direction — an empty text-less `Value` carries no elements to materialize.

- [ ] **Step 4: Run the unit tests to verify they pass**

Run: `cargo test -p kali_optimize array_literal_`
Expected: all four PASS.

- [ ] **Step 5: Run the whole optimizer suite for fallout**

Run: `cargo test -p kali_optimize`

Expected: PASS. If a layout/specialize/object-fold test now fails, read it before changing it: a test that asserted a *fold* which this predicate now declines is telling you the fold was reaching a shape it should not have. Record which, and do not weaken the predicate to restore a fold without understanding what shape it was folding.

- [ ] **Step 6: Verify Task 1's test now passes**

Run: `cargo test -p kali_cli --test inprocess literal_index_read_off_an_allocation`
Expected: PASS at all three tiers.

- [ ] **Step 7: Check whether the E3100 warnings went with it**

Run:

```bash
cd /tmp && printf 'const u = new Array(4).fill(7); let i = 0; console.log(u[i]);\n' > t.js
cargo run -q -p kali_cli --bin kali -- build --release t.js 2>&1 | grep -E 'E5506|E3100' || echo "no diagnostics"
```

Expected: `no diagnostics`. **If any `E3100` survives** ("undefined call target 'Array' ... lowered through a zero placeholder compatibility fallback"), that is a fail-open lowering independent of this substitution. Record it verbatim; it becomes an entry in Task 8. Do NOT absorb it silently and do NOT fix it here.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_optimize/src/layout.rs crates/kali_optimize/src/layout_tests.rs
git commit -m "fix(optimize): is_array_literal is positive, so an allocation is not a literal

The predicate was negative space -- any text-less \`Value\` that is not an object
literal -- so a lowered \`new Array(n).fill(v)\` initializer qualified, entered the
spec env, and had every read of its binding overwritten with a clone of the
initializer node. It now requires every child to be a materializable element,
and declines an empty child list.

Closes the silent half pinned in the previous commit.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Task 3: Pin both loud halves

The `E5506` refusals — declarator and parameter — are the same defect as Task 1's silent half (spec §2.3). Task 2 should already have fixed them; this task proves it and pins it.

**Files:**
- Modify: `crates/kali_cli/tests/inprocess/release_allocation_identity.rs` (append)

**Interfaces:**
- Consumes: `run_at` from Task 1.

- [ ] **Step 1: Write the tests**

Append to `crates/kali_cli/tests/inprocess/release_allocation_identity.rs`:

```rust
/// The LOUD declarator half. A non-literal index reaches the computed-member
/// gateway, which cannot resolve a base with no name -- `E5506` at the release
/// tiers before 2026-09-10.
const DYNAMIC_INDEX: &str = "\
const u = new Array(4).fill(7);
let i = 0;
console.log(u[i]);
";

/// The LOUD parameter half. Ruling R22 recorded this as a SECOND class,
/// "deliberately NOT admitted". It is the same substitution arriving through an
/// inlined argument: measured `fast=0 / release>=1`, and correct at the default
/// tier. Spec §2.4 overturns R22 on that evidence.
const PARAMETER_READ: &str = "\
function f(a) { let i = 0; return a[i]; }
const u = new Array(2).fill(8);
console.log(f(u));
";

/// The parameter half with a STORE through the alias, which is the shape
/// spectral-norm's `Au(u, v)` actually has.
const PARAMETER_STORE: &str = "\
function g(a, b) { for (let i = 0; i < a.length; i = i + 1) { b[i] = a[i]; } }
const u = new Array(3).fill(5);
const v = new Array(3);
g(u, v);
console.log(v[0]);
";

/// The spectral-norm inner-loop shape at n=3, reduced.
const SPECTRAL_SHAPE: &str = "\
function Au(u, v) {
  for (let i = 0; i < u.length; i = i + 1) {
    let t = 0;
    for (let j = 0; j < u.length; j = j + 1) { t = t + u[j]; }
    v[i] = t;
  }
}
const u = new Array(3).fill(1);
const v = new Array(3);
Au(u, v);
console.log(v[0]);
";

#[test]
fn allocation_backed_reads_agree_with_node_at_every_tier() {
    for (label, source, expected) in [
        ("dynamic index off a declarator", DYNAMIC_INDEX, "7\n"),
        ("read through a parameter", PARAMETER_READ, "8\n"),
        ("store through a parameter", PARAMETER_STORE, "5\n"),
        ("the spectral-norm shape", SPECTRAL_SHAPE, "3\n"),
    ] {
        for mode in [BuildMode::Fast, BuildMode::Release, BuildMode::ReleaseAdvanced] {
            assert_eq!(
                run_at(source, mode),
                expected,
                "{label} under {mode:?}: must agree with node, as --fast already does"
            );
        }
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p kali_cli --test inprocess allocation_backed_reads`
Expected: PASS.

If any case still refuses with `E5506`, the binding is entering the spec env by a route other than `specialize.rs:82`/`:109`. **Re-open spec §2.1 before writing more code** — do not add a second patch on top of a wrong mechanism (spec §7 row 2).

- [ ] **Step 3: Commit**

```bash
git add crates/kali_cli/tests/inprocess/release_allocation_identity.rs
git commit -m "test(optimize): pin both loud halves, declarator and parameter

Ruling R22 recorded the parameter shape as a second class deliberately not
admitted. It is the same substitution arriving through an inlined argument:
fast=0/release>=1, and correct at the default tier. All four shapes now agree
with node at all three tiers.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Task 4: Pin what must still refuse

Spec §2.5. A plain array literal with a dynamic index refuses at BOTH tiers — no tier split, so it is not this defect. Pinning it stops a later reader mistaking it for fallout from this work, and catches a fix that accidentally widens the admitted set.

**Files:**
- Modify: `crates/kali_cli/tests/inprocess/release_allocation_identity.rs` (append)

**Interfaces:**
- Consumes: `compile_at` from Task 1.

- [ ] **Step 1: Write the test**

Append:

```rust
/// NOT this defect, and must keep refusing. Measured at `eec408d000`:
/// `fast=1 / release=1` -- no tier split, so nothing here is caused by the
/// binding substitution. This is the pre-existing computed-member gateway gap.
/// It is pinned so that a change which accidentally ADMITS it is caught, and so
/// that a later reader does not read the refusal as fallout from this project.
const ARRAY_LITERAL_DYNAMIC_INDEX: &str = "\
const u = [7, 7, 7, 7];
let i = 0;
console.log(u[i]);
";

#[test]
fn a_plain_array_literal_with_a_dynamic_index_still_refuses_at_every_tier() {
    for mode in [BuildMode::Fast, BuildMode::Release, BuildMode::ReleaseAdvanced] {
        let diagnostics = compile_at(ARRAY_LITERAL_DYNAMIC_INDEX, mode)
            .err()
            .unwrap_or_else(|| {
                panic!(
                    "{mode:?}: this shape is expected to REFUSE. If it now builds, that is a \
                     capability change this project did not intend and did not verify -- \
                     confirm the emitted module actually runs and agrees with node (7) \
                     before relaxing this test."
                )
            });
        assert!(
            diagnostics.iter().any(|d| {
                d.code == Some(u32::from(e5::FEATURE_UNAVAILABLE))
                    && d.message
                        .contains(&computed_member_access_unavailable_message())
            }),
            "{mode:?}: expected an E5506 carrying the canonical computed-member \
             wording; got {diagnostics:?}"
        );
    }
}
```

This uses the same idiom as `crates/kali_cli/tests/inprocess/release_constant_condition_loop.rs:245`. Add the two imports to the top of the module:

```rust
use kali_common::computed_member_access_unavailable_message;
use kali_error::_error_codes::e5;
```

`Diagnostic::code` is an `Option<u32>` (`crates/kali_error/src/diagnostic.rs:32`), not a string — do not compare it to `"E5506"`.

- [ ] **Step 2: Run the test**

Run: `cargo test -p kali_cli --test inprocess a_plain_array_literal_with_a_dynamic_index`
Expected: PASS.

If `computed_member_access_unavailable_message` takes arguments in this version, check its call site at `release_constant_condition_loop.rs:245` and match it.

- [ ] **Step 3: Commit**

```bash
git add crates/kali_cli/tests/inprocess/release_allocation_identity.rs
git commit -m "test(optimize): pin the pre-existing gateway gap that is NOT this defect

A plain array literal with a dynamic index refuses at both tiers -- no tier
split, so the binding substitution does not cause it. Pinned so a change that
accidentally admits it is caught, and so the refusal is not read as fallout.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Task 5: The executing benchmark gate

Spec §4.1-§4.2. `assert_optimization_benchmark_fixture` (`runtime_smoke.rs:6061`) builds each fixture at three tiers and counts instructions in the `.wasm` — **it never executes the module**. A build emitting zeros measured as a passing benchmark; for spectral-norm, so did a build emitting an unloadable module.

**Files:**
- Create: `crates/kali_cli/tests/inprocess/benchmark_execution.rs`
- Modify: `crates/kali_cli/tests/inprocess.rs`

**Interfaces:**
- Consumes: `compile_source_file`, `BuildMode`, `RuntimeCtx`.
- Produces: `Expectation` enum and the `FIXTURES` table, which Task 6 edits.

- [ ] **Step 1: Write the module**

Create `crates/kali_cli/tests/inprocess/benchmark_execution.rs`:

```rust
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
    RefusedAtRelease,
    /// Refused at every tier, on purpose.
    RefusedEverywhere,
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
    assert!(js.exists(), "no .ts or .js source for benchmark stem {stem}");
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
    assert!(
        call.ends_with(')') && call.contains('('),
        "{stem}: expected the fixture to end in a bare call statement, found: {trimmed}"
    );

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
            for mode in [BuildMode::Fast, BuildMode::Release, BuildMode::ReleaseAdvanced] {
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
            assert_eq!(outcome.stdout, expected, "{stem} --fast: disagrees with node");

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
            for mode in [BuildMode::Fast, BuildMode::Release, BuildMode::ReleaseAdvanced] {
                assert!(
                    compile_wrapped(&wrapped, stem, mode).is_err(),
                    "{stem} {mode:?}: expected a refusal at every tier"
                );
            }
        }
    }
}
```

- [ ] **Step 2: Add the fixture table and the coverage test**

Append to the same file. **Build `FIXTURES` from the 59 stems in the loop at `crates/kali_cli/tests/runtime_smoke/misc.rs:1374`** — read that list and transcribe every uncommented stem with `Expectation::Runs`, plus the four entries below. Do not guess the list; read it.

```rust
/// Every benchmark fixture with committed metadata, and what it is expected to
/// do. A fixture missing from this table fails `the_table_covers_every_fixture`
/// below -- adding a fixture without classifying it is a gate that silently
/// does not test it.
pub(crate) const FIXTURES: &[(&str, Expectation)] = &[
    // ... transcribe the 59 stems from runtime_smoke/misc.rs:1374, each as
    // (stem, Expectation::Runs) ...

    // Refused at the release tiers by the allocation-identity defect this
    // project closes. Task 6 flips these to Runs.
    ("spectral-norm-benchmark-v1", Expectation::RefusedAtRelease),
    ("nbody-benchmark-v1", Expectation::RefusedAtRelease),
    ("fannkuch-redux-benchmark-v1", Expectation::RefusedAtRelease),

    // Deliberately rejected fail-closed: passing an array literal to a function
    // had the callee reading zero placeholders, a silent miscompile. Pinned by
    // `array_literal_arguments_benchmark_is_rejected_fail_closed` in
    // runtime_smoke/misc.rs; kept here so the table covers the directory.
    ("array-literal-arguments-benchmark-v1", Expectation::RefusedEverywhere),
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
```

- [ ] **Step 3: Register the module**

Modify `crates/kali_cli/tests/inprocess.rs`, adding in alphabetical position:

```rust
#[path = "inprocess/benchmark_execution.rs"]
mod benchmark_execution;
```

- [ ] **Step 4: Run the coverage test first**

Run: `cargo test -p kali_cli --test inprocess the_table_covers_every_fixture -- --nocapture`

Expected: it will likely FAIL the first time, listing stems present on disk but absent from the table (or vice versa). Reconcile the table against the failure output until it passes. This is the intended workflow — the assertion is how you discover the true fixture set.

- [ ] **Step 5: Run the execution gate**

Run: `cargo test -p kali_cli --test inprocess every_benchmark_fixture_runs -- --nocapture`

Expected: PASS. Two failure modes worth distinguishing:
- **A fixture's tail does not match the wrapper's expectations** — `wrap_terminal_call` panics with the offending line. Widen the wrapper only if the tail is genuinely a call; otherwise the fixture needs its own handling and a comment.
- **A fixture returns `undefined`** — node prints `undefined` and so should every tier. That is a liveness check rather than a value oracle; it is accepted (spec §4.2) and needs no change.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/inprocess/benchmark_execution.rs crates/kali_cli/tests/inprocess.rs
git commit -m "test(bench): the benchmark fixtures are RUN, not just built

assert_optimization_benchmark_fixture counts instructions in the emitted wasm
and never executes it, so a build emitting zeros measured as a passing
benchmark and an unloadable module did too. Every fixture with metadata is now
compiled and executed at all three tiers and compared against node, with its
terminal call wrapped in a temp copy so sourceSha256 and the measured workload
are untouched. A fixture with no table entry fails the coverage test.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Task 6: Restore the three fixtures to the measured list

Spec §4.4 and §4.5. Only start this once Tasks 2, 3 and 5 are green.

**Files:**
- Modify: `crates/kali_cli/tests/inprocess/benchmark_execution.rs` (three table entries)
- Modify: `crates/kali_cli/tests/runtime_smoke.rs` (the `matches!` exclusion arms)
- Modify: `crates/kali_cli/tests/runtime_smoke/misc.rs:1374`, `:1631`
- Modify: `crates/kali_cli/tests/clbg_fannkuch_runtime.rs`

- [ ] **Step 1: Flip the three table entries**

In `benchmark_execution.rs`, change the three `Expectation::RefusedAtRelease` entries to `Expectation::Runs` and delete the comment above them that says Task 6 will do so.

- [ ] **Step 2: Run the gate to confirm they now run at every tier**

Run: `cargo test -p kali_cli --test inprocess every_benchmark_fixture_runs -- --nocapture`

Expected: PASS. spectral-norm should print `1.274219991`, nbody `-0.169075164` / `-0.169087605`, fannkuch `228` / `Pfannkuchen(7) = 16` — at all three tiers, matching node.

If a release tier still refuses, STOP and return to Task 3's step 2 guidance: the mechanism is not fully closed and the remaining route must be found before anything else changes.

- [ ] **Step 3: Restore the two stems to the measured list**

In `crates/kali_cli/tests/runtime_smoke/misc.rs`, inside the loop at `:1374`, replace the commented-out pair and the `RE-PINNED 2026-09-09` comment block above it with the live entries:

```rust
        // RESTORED 2026-09-10 by the release-tier-allocation-identity project
        // (docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md).
        // These two were re-pinned out of this list on 2026-09-09 under ruling
        // R22, because the release tiers refused them. The cause was the
        // negative-space `is_array_literal` predicate letting an allocating
        // initializer be substituted for its binding name; that is fixed, and
        // `inprocess/benchmark_execution.rs` now RUNS both at all three tiers
        // against node.
        ("spectral-norm-benchmark-v1", "spectral-norm"),
        ("nbody-benchmark-v1", "nbody"),
```

- [ ] **Step 4: Delete the dedicated re-pin block**

Delete the `for (fixture_stem, fast_stdout) in [...]` loop at `misc.rs:1631` together with its `THE TWO BENCHMARKS-GAME FIXTURES...` comment header. Its own failure message authorises exactly this: *"verify the emitted module actually RUNS and agrees with node, then move the fixture back into the list above."* Step 2 is that verification, and `benchmark_execution.rs` is a stronger standing form of what the block asserted.

- [ ] **Step 5: Remove the two exclusion arms**

In `crates/kali_cli/tests/runtime_smoke.rs`, inside `assert_optimization_benchmark_fixture`, delete the `| "spectral-norm"` and `| "nbody"` arms from the `matches!` list that gates the footprint assertion.

- [ ] **Step 6: Restore fannkuch's three-mode assertion**

In `crates/kali_cli/tests/clbg_fannkuch_runtime.rs`, replace the block asserting that `--release` and `--release-advanced` refuse with an assertion that all three modes build and run, and update the `RE-PINNED 2026-09-09` comment to record the restoration and its date.

- [ ] **Step 7: Run the affected suites**

Run:

```bash
cargo test -p kali_cli --test runtime_smoke
cargo test -p kali_cli --test clbg_fannkuch_runtime
cargo test -p kali_cli --test inprocess
```

Expected: PASS.

**THE BRANCH THIS STEP MUST BE READY FOR (spec §4.5).** Stage 1 works by REMOVING a release-tier fold, so release may no longer beat `--fast` on size, instructions or adds for the restored fixtures, and the footprint assertion added back in Step 5 may fail. If it does:

- Do NOT weaken the assertion.
- Put the failing names back on the `matches!` exclusion list **with a comment recording the measured numbers and the reason** — that the release tiers do not improve this shape once the unsound fold is gone. The list already carries one such honest entry (`const-object-property-access`, rewritten 2026-07-19); this would be a second.
- Record the numbers for Task 8.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_cli/tests/
git commit -m "test(bench): spectral-norm, nbody and fannkuch return to the measured list

All three build AND run at all three tiers, byte-identical to node, so the
2026-09-09 re-pin under ruling R22 is discharged. The dedicated re-pin block is
replaced by the standing execution gate, per the instruction that block carried
in its own failure message. fannkuch's three-mode assertion is three again.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Task 7: Stage 2 — represent array-literal-ness

Spec §3.2. Stage 1 fixed the measured defect by exclusion; it does not stop the next allocating construct (`Array.from`, `new X`, a spread) entering through the same hole, because array-literal-ness is still *inferred from node shape*. `LirNode` has only `kind`, `text`, `children`, `function_flavor` — there is nowhere to hang a flag — so the fix is a new node kind, on the precedent `node.rs:12-15` already sets for `ComputedMember`.

**This task begins with a sizing step and may legitimately end there** (spec §7 row 5).

**Files:**
- Modify: `crates/kali_lir/src/node.rs`
- Modify: `crates/kali_hir/src/lowering/expression.rs`
- Modify: `crates/kali_optimize/src/layout.rs`
- Modify: whatever the compiler names in Step 2

- [ ] **Step 1: Add the variant and let the compiler enumerate the blast radius**

In `crates/kali_lir/src/node.rs`, add to `LirNodeKind` after `ComputedMember`:

```rust
    /// An array literal `[a, b, c]`, whose children are its elements.
    ///
    /// Represented rather than inferred. Before 2026-09-10 an array literal was
    /// "a text-less `Value` that is not an object literal" -- negative space,
    /// which admitted a lowered `new Array(n).fill(v)` and cost a silent
    /// miscompile at the release tiers. Every recognizer that matches `Value`
    /// declines this kind by construction.
    ArrayLiteral,
```

- [ ] **Step 2: Run the build and COUNT the sites**

Run: `cargo build --workspace 2>&1 | grep -c "non-exhaustive patterns"`

and

Run: `cargo build --workspace 2>&1 | grep -A 2 "non-exhaustive patterns" | grep "\-\->"`

**DECISION POINT.** Record the count and the file list.

- **If the list is small enough to handle attentively** (roughly ≤ 15 sites), continue to Step 3.
- **If it is large**, stop. Revert this task's changes (`git checkout crates/kali_lir/src/node.rs`), and report the site list. Stage 2 becomes its own project with its own spec, and Stage 1 ships alone — spec §3.2 already records why a follow-on exists. This is an anticipated outcome, not a failure.

- [ ] **Step 3: Tag array literals at lowering**

In `crates/kali_hir/src/lowering/expression.rs`, find where an array-literal expression allocates its LIR node as a text-less `LirNodeKind::Value` and change it to `LirNodeKind::ArrayLiteral`. Leave the children (the elements) unchanged.

- [ ] **Step 4: Resolve every site the compiler named**

For each site from Step 2, decide explicitly and comment non-obvious ones. The default for a recognizer that matched text-less `Value` as "maybe an array literal" is to match `ArrayLiteral` instead; the default for codegen is to handle it exactly as it handled the text-less `Value` before.

- [ ] **Step 5: Reduce the predicate to a kind check**

Replace `is_array_literal`'s body in `crates/kali_optimize/src/layout.rs` with:

```rust
    /// True when `id` IS an array literal. Positive by construction: the kind
    /// is assigned at lowering, so no shape has to be guessed at. See
    /// `LirNodeKind::ArrayLiteral`'s doc comment for what this replaced.
    pub(crate) fn is_array_literal(&self, program: &LirProgram, id: LirNodeId) -> bool {
        program
            .nodes
            .get(id.0 as usize)
            .is_some_and(|node| node.kind == LirNodeKind::ArrayLiteral)
    }
```

Keep `is_materializable_element` from Task 2 — `is_specializable_binding` still needs it to decide whether an array literal's ELEMENTS can be materialized, which a kind check does not answer.

- [ ] **Step 6: Run everything**

Run:

```bash
cargo test -p kali_lir
cargo test -p kali_hir
cargo test -p kali_optimize
cargo test -p kali_cli --test inprocess
```

Expected: PASS, including every test from Tasks 1, 3, 4 and 5.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_lir/src/node.rs crates/kali_hir/src/lowering/expression.rs crates/kali_optimize/src/layout.rs
git commit -m "refactor(lir): array-literal-ness is represented, not inferred

Adds LirNodeKind::ArrayLiteral on the precedent ComputedMember sets, so every
recognizer that matches Value declines it by construction and is_array_literal
becomes a kind check. Stage 1 closed the hole for one shape by exclusion; this
closes the class, because the next allocating construct can no longer qualify
as a literal by failing to look like an object.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Task 8: Ledger obligations

Spec §6. Documentation this project owes. Do this LAST, so every number written down is one the finished branch produces.

**Files:**
- Modify: `docs/superpowers/followups/release-mode-optimizer-inlines-an-allocating-initializer.md`
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md`
- Create: `docs/superpowers/followups/mutation-scan-is-blind-to-parameter-aliases.md`

- [ ] **Step 1: Correct the optimizer followup**

In `release-mode-optimizer-inlines-an-allocating-initializer.md`:
- Correct the **title and §1**: the pass is layout-binding specialization (`specialize.rs:120-127`), not inlining.
- Rewrite **§6** ("What would close it") with the mechanism-accurate account and what actually closed it.
- Amend **§5** with the **R22 overturn** and spec §2.4's table. Withdraw the "second class deliberately NOT admitted" note: there is no second class.
- Add **Task 1's measured value** for the silent half — the number, the tier, the binary, and the node version actually used.

Follow the file's existing amendment convention: strike text with `~~...~~` and date the amendment inline rather than deleting history.

- [ ] **Step 2: File the alias-mutation hole**

Create `docs/superpowers/followups/mutation-scan-is-blind-to-parameter-aliases.md`, recording: `collect_mutated_binding_names` (`object_fold.rs:519`) is a program-wide, scope-blind scan for store BASE IDENTIFIERS; a const array mutated only through a differently-named parameter is never a store base under its own name; spectral-norm's `const w` is written as `v[i] = t` inside `Au`, so the name `w` is never seen. Note that Stage 1 removed the exposure for allocations and that the hole remains for genuine array literals. Include a runnable repro and its measured behaviour at each tier.

- [ ] **Step 3: File the register entries**

Per the optimizer followup's §7: the silent literal-index half to the register's **§2** (silent), the refusing half to **§7** (fail-loudly-but-wrong). Record with them the instrument gap §7 names: the register's oracle harness measures the **default tier only**, so a §0.2 row for this entry cannot be backed by an oracle case as the harness stands.

If Task 2 Step 7 found a surviving `E3100`, file it here too.

- [ ] **Step 4: Regenerate the ranking if the register changed**

Run: `cargo run -p kali_blast_radius --example rank`

Then: `cargo test -p kali_blast_radius spliced_document_matches_the_generator`

Expected: PASS. This test re-renders the generated regions of `blast-radius-ranking.md` and asserts they equal the committed text — it goes red if the register moved and the ranking was not regenerated, by design.

- [ ] **Step 5: Run the full gate**

Run all three, per Global Constraints:

```bash
bash scripts/test-gate.sh
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Expected: all three clean. `GATE OK` alone is not sufficient.

- [ ] **Step 6: Commit**

```bash
git add docs/
git commit -m "docs(followups): the mechanism, the R22 overturn, and the alias hole

Corrects the pass attribution (layout-binding specialization, not inlining),
records the measured value of the silent half, withdraws the second-class note
now that the parameter shape is known to be the same substitution, and files
the scope-blind mutation scan as its own entry.

Claude-Session: https://claude.ai/code/session_01Jdj1YHopdpFoRAhLWND4yv"
```

---

## Self-Review Notes

**Spec coverage.** §1.2 → Task 1. §2.1-2.3 → Tasks 1, 3. §2.4 → Task 3 + Task 8 Step 1. §2.5 → Task 4. §3.1 → Task 2. §3.2 → Task 7. §3.3 (rejected approach) → no task by design; it is recorded in the spec so it is not re-proposed. §4.1-4.2 → Task 5. §4.3 → Tasks 1, 3, 4, 2 Step 1. §4.4 → Task 6. §4.5 → Task 6 Step 7. §5 non-goals → Task 8 Step 2 files the one that needed filing. §6 → Task 8. §7 risks → stop conditions in Tasks 1, 2, 3, 6, 7. §8 → Global Constraints.

**Type consistency.** `compile_at` / `run_at` (Task 1) are used unchanged in Tasks 3 and 4. `Expectation` and `FIXTURES` (Task 5) are edited by Task 6. `is_materializable_element` (Task 2) survives Task 7 Step 5 deliberately, and Step 5 says why.

**Known soft spot.** Task 5 Step 2 asks the implementer to transcribe 59 stems from `misc.rs:1374` rather than listing them here. That is deliberate — transcribing a list into this document would create a second copy that can drift from the loop, and `the_table_covers_every_fixture_with_metadata` is the mechanism that makes the transcription verifiable. The implementer is told to read the list, not guess it, and a wrong transcription fails loudly on the first run.
