# Kali — Implementation Plan

This document is the top-level implementation plan. It maps the spec's four phases onto concrete,
incrementally workable stages. After every stage the project should be in a state that compiles,
passes its current tests, and provides a meaningful subset of end-user value.

**Spec authority:** [`SPEC.md`](./SPEC.md) and the owning chapters in [`specs/`](./specs/) are the
normative source of truth. This plan translates their *Recommended Phase-1 Implementation Order*
and the phase contracts from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) into
a concrete build sequence.

---

## Phase 1 — Core Compiler & Toolchain MVP

Goal: a dependable, end-to-end TypeScript/JavaScript → WebAssembly compiler that can check, run,
test, and bundle real programs in the Deno-oriented standalone context and the Phase-1
browser-targeted command set, with sandbox enforcement, basic package management, and a
proof-ready repository baseline.

Each stage below leaves the project in a *workable* state — it compiles, existing tests pass, and
the user can exercise at least one new capability.

| Stage | Document | Workable milestone |
|---|---|---|
| 1.1 | [Workspace & Crate Scaffold](plan/phase-1/01-workspace-scaffold.md) | `cargo build` succeeds; `kali --version` prints a version string |
| 1.2 | [Lexer](plan/phase-1/02-lexer.md) | Tokenises valid TS/JS source; emits stable `E1xxx` lex errors |
| 1.3 | [Parser & AST](plan/phase-1/03-parser-and-ast.md) | Parses full ECMA-262 + TypeScript grammar; AST node types defined |
| 1.4 | [Name Resolution](plan/phase-1/04-name-resolution.md) | `kali check` reports unresolved identifiers and import errors |
| 1.5 | [Type Checker](plan/phase-1/05-type-checker.md) | `kali check` reports type errors under the bounded inference contract |
| 1.6 | [HIR & LIR Lowering](plan/phase-1/06-hir-lir-lowering.md) | Full compiler pipeline exists; LIR can be inspected / round-tripped |
| 1.7 | [WASM Code Generation](plan/phase-1/07-wasm-codegen.md) | Simple programs compile to runnable WASM modules |
| 1.8 | [Runtime & Execution](plan/phase-1/08-runtime-execution.md) | `kali run` and `kali test` work in the Default standalone context |
| 1.9 | [Sandbox & Policy](plan/phase-1/09-sandbox-and-policy.md) | `--sandbox` flag enforced at runtime; policy files validated statically |
| 1.10 | [Package Management](plan/phase-1/10-package-management.md) | `kali install` resolves npm/JSR/raw-URL deps; lock file is deterministic |
| 1.11 | [Build Artifacts](plan/phase-1/11-build-artifacts.md) | `kali build` emits executables; `--bundle` emits browser bundles; `--lib` emits base library artifacts |
| 1.12 | [Developer Workflow](plan/phase-1/12-developer-workflow.md) | `kali init`, `kali fmt`, `kali lint` all functional |
| 1.13 | [Diagnostics & Schemas](plan/phase-1/13-diagnostics-and-schemas.md) | Stable error codes; `--output json` emits schema-v1 envelopes |
| 1.14 | [Evidence Hardening](plan/phase-1/14-evidence-hardening.md) | Conformance suite, package corpus, browser smoke tests, determinism checks, proof-ready baseline |

---

## Phase 2 — Ownership, Effects & Public Embedding

Goal: MIR-backed memory management with deterministic ownership/escape analysis; the stable public
effect-report surface (`kali effects`, `kali package-effects`); compile-time inferred-effect-vs-policy
validation; and the stable public embedding surface (Rust API, WIT-first `--lib`, `--capi`,
`--component`).

| Stage | Document | Workable milestone |
|---|---|---|
| 2.1 | [MIR & Ownership Analysis](plan/phase-2/01-mir-and-ownership.md) | MIR is the canonical mid-stage; escape analysis drives stack/heap/shared decisions |
| 2.2 | [Public Effect Reporting](plan/phase-2/02-public-effect-reporting.md) | `kali effects <file>` and `kali package-effects <pkg>` emit stable JSON; `check/build --sandbox` adds inferred-effect-vs-policy rejection |
| 2.3 | [Public Embedding Surface](plan/phase-2/03-public-embedding-surface.md) | Stable Rust embedding API; WIT sidecar on `--lib`; `--capi` and `--component` artifact modes |

---

## Phase 3 — Specialisation, Optimisation & Ecosystem Breadth

Goal: generic/function/layout specialisation at compile time; stronger optimisation tiers;
incremental compilation; broader npm/Node compatibility beyond the Phase-1 pure-JS/TS baseline;
and broader browser packaging.

| Stage | Document | Workable milestone |
|---|---|---|
| 3.1 | [Specialisation & Optimisation](plan/phase-3/01-specialisation-and-optimisation.md) | `--release` and `--release-advanced` produce measurably faster WASM; monomorphisation pipeline stable |
| 3.2 | [Node Compatibility](plan/phase-3/02-node-compatibility.md) | `--api node` command path supported; broader Node built-ins available |
| 3.3 | [Ecosystem Breadth](plan/phase-3/03-ecosystem-breadth.md) | Incremental compilation; broader package corpus; open-ended cross-module constraint solving |

---

## Phase 4 — Advanced Compatibility & Deep Verification

Goal: hardest dynamic features (`eval`, `Function()`, non-literal dynamic imports); deeper API
coverage; and proof-backed release claims with a non-empty published Lean boundary.

| Stage | Document | Workable milestone |
|---|---|---|
| 4.1 | [Dynamic Compatibility](plan/phase-4/01-dynamic-compatibility.md) | `eval`/`Function()` executable behind `compat.features.eval`; non-literal dynamic loading gated similarly |
| 4.2 | [Formal Verification Depth](plan/phase-4/02-formal-verification-depth.md) | Published Lean boundary names concrete modelled subsystems; CI runs proof jobs; repository is proof-backed |

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
