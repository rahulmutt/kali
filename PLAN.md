# Kali — Implementation Plan

This document is the top-level implementation plan. It maps the spec's four phases onto concrete,
incrementally workable stages. After every stage the project should be in a state that compiles,
passes its current tests, and provides a meaningful subset of end-user value.

**Spec authority:** [`SPEC.md`](./SPEC.md) and the owning chapters in [`specs/`](./specs/) are the
normative source of truth. This plan translates their *Recommended Phase-1 Implementation Order*
and the phase contracts from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) into
a concrete build sequence.

---

## Reading This Plan

Each stage document lives under `plan/<phase>/<stage>.md` and follows a consistent structure:

- **Goal** — one-paragraph summary of what the stage delivers.
- **Workable milestone** — the concrete user- or developer-visible capability that exists when
  the stage is done; every prior milestone stays intact.
- **Depends on** — the earlier stages that must be complete before this stage begins.
- **Tasks** — implementation steps, design decisions, and code examples.
- **Out of scope** — explicit list of things intentionally deferred to later stages.
- **Definition of done** — checkbox list used for stage sign-off.

### How stages relate to spec phases

`SPEC.md` defines four phases (1–4) and, within Phase 1, six recommended implementation steps.
The stages in this plan map onto those spec steps as follows:

| Spec Phase 1 step | Plan stages | Notes |
|---|---|---|
| 1 — Frontend + checking foundation | 1.2 → 1.5 | Lexer, parser, name resolution, type checker |
| 2 — Deterministic package/install foundation | 1.10 | Placed *after* execution stages; see ordering note below |
| 3 — Kali-hosted execution foundation | 1.6 → 1.8 | HIR/LIR pipeline, WASM codegen, runtime |
| 4 — Build/artifact foundation | 1.9, 1.11 | Sandbox + policy, build artifacts |
| 5 — Developer workflow foundation | 1.12 → 1.13 | `init`/`fmt`/`lint`, diagnostics & schemas |
| 6 — Phase-1 evidence hardening | 1.14 | Conformance, corpus, determinism, proof-ready |

Stage 1.1 (workspace scaffold) is a prerequisite shared across all steps.

### Ordering note: package management after execution

`SPEC.md`'s recommended order places the package/install foundation (spec step 2) *before*
the execution foundation (spec step 3). This plan intentionally reverses those two groups:
stages 1.6–1.8 (execution) come before stage 1.10 (packages).

**Rationale:** every intermediate stage must leave the project in a *workable state*.
An execution environment without any installed packages is immediately useful — developers
can compile and run local `.ts`/`.js` files and verify the pipeline end-to-end. A package
installer without an execution environment has no user-visible outcome to validate it against.
The name-resolver in stage 1.4 already stubs bare-specifier resolution (returning `E3010` for
unresolved specifiers) so all later stages remain consistent until stage 1.10 fills in the
real resolver.

This deviation does not affect the Phase-1 contract — all six spec steps are completed
within Phase 1.

---

## Phase 1 — Core Compiler & Toolchain MVP

Goal: a dependable, end-to-end TypeScript/JavaScript → WebAssembly compiler that can check, run,
test, and bundle real programs in the Deno-oriented standalone context and the Phase-1
browser-targeted command set, with sandbox enforcement, basic package management, and a
proof-ready repository baseline.

Each stage below leaves the project in a *workable* state — it compiles, existing tests pass, and
the user can exercise at least one new capability.

### Foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.1 | [Workspace & Crate Scaffold](plan/phase-1/01-workspace-scaffold.md) | `cargo build` succeeds; `kali --version` prints a version string; `proofs/BOUNDARY.md` exists with proof-ready placeholder |

### Spec Step 1 — Frontend + checking foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.2 | [Lexer](plan/phase-1/02-lexer.md) | Tokenises valid TS/JS source; emits stable `E1xxx` lex errors |
| 1.3 | [Parser & AST](plan/phase-1/03-parser-and-ast.md) | Parses full ECMA-262 + TypeScript grammar; AST node types defined |
| 1.4 | [Name Resolution](plan/phase-1/04-name-resolution.md) | `kali check` reports unresolved identifiers and import errors |
| 1.5 | [Type Checker](plan/phase-1/05-type-checker.md) | `kali check` reports type errors under the bounded inference contract |

### Spec Step 3 — Kali-hosted execution foundation *(before step 2 for workability; see ordering note)*

| Stage | Document | Workable milestone |
|---|---|---|
| 1.6 | [HIR & LIR Lowering](plan/phase-1/06-hir-lir-lowering.md) | Full compiler pipeline exists; LIR can be inspected / round-tripped |
| 1.7 | [WASM Code Generation](plan/phase-1/07-wasm-codegen.md) | Simple programs compile to runnable WASM modules |
| 1.8 | [Runtime & Execution](plan/phase-1/08-runtime-execution.md) | `kali run` and `kali test` work in the Default standalone context |

### Spec Steps 2 & 4 — Package, sandbox & build foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.9 | [Sandbox & Policy](plan/phase-1/09-sandbox-and-policy.md) | `--sandbox` flag enforced at runtime; policy files validated statically |
| 1.10 | [Package Management](plan/phase-1/10-package-management.md) | `kali install` resolves npm/JSR/raw-URL deps; lock file is deterministic |
| 1.11 | [Build Artifacts](plan/phase-1/11-build-artifacts.md) | `kali build` emits executables; `--bundle` emits browser bundles; `--lib` emits base library artifacts |

### Spec Step 5 — Developer workflow foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.12 | [Developer Workflow](plan/phase-1/12-developer-workflow.md) | `kali init`, `kali fmt`, `kali lint` all functional |
| 1.13 | [Diagnostics & Schemas](plan/phase-1/13-diagnostics-and-schemas.md) | Stable error codes; `--output json` emits schema-v1 envelopes |

### Spec Step 6 — Evidence hardening

| Stage | Document | Workable milestone |
|---|---|---|
| 1.14 | [Evidence Hardening](plan/phase-1/14-evidence-hardening.md) | Conformance suite, package corpus, browser smoke tests, determinism checks, proof-ready baseline |

### Phase 1 parallelism

Within Phase 1 the following parallel opportunities exist:
- **1.9 static-validation work** (policy parsing, schema validation, `kali check --sandbox`) depends
  only on stage 1.5 and may begin while stages 1.6–1.8 are still in progress; the runtime
  enforcement portion of 1.9 still requires 1.8.
- **1.12 and 1.13** are largely independent and may be worked on in parallel once 1.11 is complete.

All other Phase 1 stages should be treated as sequential unless noted above.

### Phase 1 completion gate

Phase 1 is complete when all stages 1.1–1.14 have passed their Definitions of Done *and* every
Phase-1 maturity label in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) is backed
by a passing evidence track from stage 1.14.

---

## Phase 2 — Ownership, Effects & Public Embedding

Goal: MIR-backed memory management with deterministic ownership/escape analysis; the stable public
effect-report surface (`kali effects`, `kali package-effects`); compile-time inferred-effect-vs-policy
validation; and the stable public embedding surface (Rust API, WIT-first `--lib`, `--capi`,
`--component`).

Stages 2.2 and 2.3 both depend on the MIR pipeline from 2.1; they can proceed in parallel once
2.1 is complete.

### Spec chapter mapping

| Spec chapter | Plan stage | Key deliverable |
|---|---|---|
| [`05 — IR`](./specs/05-ir.md) | 2.1 | MIR as canonical mid-stage; `HIR → MIR → LIR` path replaces direct lowering |
| [`06 — Memory Management`](./specs/06-memory.md) | 2.1 | Escape analysis; deterministic ownership classes (`stack`, `owned heap`, `shared heap`, `borrowed`) |
| [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md) | 2.2 | Public effect-report surface (reporting half + policy-comparison half) |
| [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md) | 2.3 | Stable Rust embedding API; WIT-first `--lib`; `--capi`; `--component` |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | all | Phase-2 maturity rows open |

| Stage | Document | Workable milestone |
|---|---|---|
| 2.1 | [MIR & Ownership Analysis](plan/phase-2/01-mir-and-ownership.md) | MIR is the canonical mid-stage; escape analysis drives stack/heap/shared decisions |
| 2.2 | [Public Effect Reporting](plan/phase-2/02-public-effect-reporting.md) | `kali effects <file>` and `kali package-effects <pkg>` emit stable JSON; `check/build --sandbox` adds inferred-effect-vs-policy rejection |
| 2.3 | [Public Embedding Surface](plan/phase-2/03-public-embedding-surface.md) | Stable Rust embedding API; WIT sidecar on `--lib`; `--capi` and `--component` artifact modes |

### Phase 2 completion gate

Phase 2 is complete when stages 2.1–2.3 have passed their Definitions of Done, the public
effect-report surface and public embedding surface are stable (stable semver published), and the
Phase-2 maturity rows in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) are
updated to reflect passing evidence.

---

## Phase 3 — Specialisation, Optimisation & Ecosystem Breadth

Goal: generic/function/layout specialisation at compile time; stronger optimisation tiers;
incremental compilation; broader npm/Node compatibility beyond the Phase-1 pure-JS/TS baseline;
and broader browser packaging.

Stages 3.2 and 3.3 can be developed in parallel with 3.1 once Phase 1 is complete; 3.1's
monomorphisation work is a prerequisite for the full layout-specialisation benefits in 3.3.

### Spec chapter mapping

| Spec chapter | Plan stage | Key deliverable |
|---|---|---|
| [`05 — IR`](./specs/05-ir.md) | 3.1 | Layout specialisation via MIR layout descriptors |
| [`07 — Specialisation`](./specs/07-specialization.md) | 3.1 | Monomorphisation; `--release` / `--release-advanced` optimisation passes |
| [`08 — WASM Codegen`](./specs/08-wasm-codegen.md) | 3.3 | Code splitting; tree-shaking; dynamic `import()` bundle boundaries |
| [`11 — Standard APIs`](./specs/11-standard-apis.md) | 3.2 | Node compatibility surface; common Node built-ins |
| [`14 — Packages`](./specs/14-packages.md) | 3.2, 3.3 | Broader npm corpus; Node-assuming package support |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | all | Phase-3 maturity rows open |

| Stage | Document | Workable milestone |
|---|---|---|
| 3.1 | [Specialisation & Optimisation](plan/phase-3/01-specialisation-and-optimisation.md) | `--release` and `--release-advanced` produce measurably faster WASM; monomorphisation pipeline stable |
| 3.2 | [Node Compatibility](plan/phase-3/02-node-compatibility.md) | `--api node` command path supported; broader Node built-ins available |
| 3.3 | [Ecosystem Breadth](plan/phase-3/03-ecosystem-breadth.md) | Incremental compilation; broader package corpus; open-ended cross-module constraint solving |

### Phase 3 completion gate

Phase 3 is complete when stages 3.1–3.3 have passed their Definitions of Done, `--release` and
`--release-advanced` are measurably better than `--fast` on the CI benchmark suite, `--api node`
is no longer gated, and the Phase-3 maturity rows in
[`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) are updated to reflect passing
evidence.

---

## Phase 4 — Advanced Compatibility & Deep Verification

Goal: hardest dynamic features (`eval`, `Function()`, non-literal dynamic imports); deeper API
coverage; and proof-backed release claims with a non-empty published Lean boundary.

Stage 4.2 (formal verification) can be started in parallel with earlier phases — the Lean model
can be developed alongside the implementation, but **proof-backed** claims require a non-empty
published boundary in `proofs/BOUNDARY.md` before they may appear in release notes or support
summaries.

### Spec chapter mapping

| Spec chapter | Plan stage | Key deliverable |
|---|---|---|
| [`10 — Runtime`](./specs/10-runtime.md) | 4.1 | `eval`/`Function()` behind `compat.features.eval`; non-literal dynamic imports |
| [`14 — Packages`](./specs/14-packages.md) | 4.1 | `kali package-audit` publicly available (no `--preview` gate) |
| [`17 — Formal Verification`](./specs/17-verification.md) | 4.2 | Non-empty Lean proof boundary; proof CI passes; repository may claim proof-backed |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | all | Phase-4 maturity rows open |

| Stage | Document | Workable milestone |
|---|---|---|
| 4.1 | [Dynamic Compatibility](plan/phase-4/01-dynamic-compatibility.md) | `eval`/`Function()` executable behind `compat.features.eval`; non-literal dynamic loading gated similarly |
| 4.2 | [Formal Verification Depth](plan/phase-4/02-formal-verification-depth.md) | Published Lean boundary names concrete modelled subsystems; CI runs proof jobs; repository is proof-backed |

### Phase 4 completion gate

Phase 4 is complete when stages 4.1–4.2 have passed their Definitions of Done,
`proofs/BOUNDARY.md` names a non-empty modelled subsystem with passing Lean proof jobs, and the
repository may honestly claim **proof-backed** status for the published boundary. Phase-4 maturity
rows in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) are updated accordingly.

---

## Cross-Cutting Rules

* **Hard invariants never bend.** AOT-only, pure-Rust implementation, no tracing/background GC,
  sandbox-first honesty, and deterministic machine contracts hold across all phases.
* **Each stage must leave the project workable.** No stage may break existing tests or make a
  previously-functional CLI command regress.
* **Availability follows `specs/19-feature-maturity.md`.** A stage completing its implementation
  work does not automatically promote a feature's maturity label — that requires the matching
  evidence from the canonical testing tracks in `specs/16-testing.md`.
* **Proof-ready from day one.** `proofs/BOUNDARY.md` and the proof-CI trigger policy must exist
  from Stage 1.1 onward; proof-backed claims require a non-empty published boundary.
* **Stage parallelism is opt-in.** Unless a stage document explicitly notes that its work can
  proceed in parallel with another, assume sequential ordering within each phase.
