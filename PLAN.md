# Kali — Implementation Plan

This document is the top-level implementation plan that maps the spec's four phases onto concrete,
incrementally workable stages. After every stage the project should be in a workable state that:

1. **Compiles** — `cargo build` succeeds with no warnings that would block merge
2. **Passes tests** — Existing test suite passes (`cargo test --workspace`)
3. **Provides end-user value** — At least one new capability is demonstrable via CLI
4. **Maintains invariants** — AOT-only, pure Rust, no tracing GC, sandbox-first, deterministic

**Spec authority:** [`SPEC.md`](./SPEC.md) and the owning chapters in [`specs/`](./specs/) are the
normative source of truth. This plan translates their *Recommended Phase-1 Implementation Order*
and the phase contracts from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) into
a concrete build sequence.

---

## Directory Structure

This plan uses a flat directory structure under `plan/<phase>/` where each stage is a numbered file:

```
plan/
├── phase-1/  (Core Compiler & Toolchain MVP)
│   ├── 01-workspace-scaffold.md
│   ├── 02-lexer.md
│   ├── 03-parser-and-ast.md
│   ├── 04-name-resolution.md
│   ├── 05-type-checker.md
│   ├── 06-hir-lir-lowering.md
│   ├── 07-wasm-codegen.md
│   ├── 08-runtime-execution.md
│   ├── 09-sandbox-and-policy.md
│   ├── 10-package-management.md
│   ├── 11-build-artifacts.md
│   ├── 12-developer-workflow.md
│   ├── 13-diagnostics-and-schemas.md
│   └── 14-evidence-hardening.md
├── phase-2/  (Ownership, Effects & Public Embedding)
│   ├── 01-mir-and-ownership.md
│   ├── 02-public-effect-reporting.md
│   ├── 03-public-embedding-surface.md
│   └── 04-lean-model-foundation.md
├── phase-3/  (Specialization, Optimization and Ecosystem Breadth)
│   ├── 01-optimization-and-specialization.md
│   ├── 02-node-compatibility.md
│   └── 03-ecosystem-breadth.md
└── phase-4/  (Advanced Compatibility & Deep Verification)
    ├── 01-dynamic-compatibility.md
    └── 02-formal-verification-depth.md
```

Total: **20 stage documents** across **4 phases** with **4 stage groups** explicitly noting parallelizable work.

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
| 5 — Developer workflow foundation | 1.12 → 1.13 | `init`/`fmt`/`lint`, canonical `W2xxx` lint registry, diagnostics & schemas |
| 6 — Evidence hardening | 1.14 | Conformance, corpus, determinism, proof-ready, proof-ready CI pipeline |

Stage 1.1 (workspace scaffold) is a prerequisite shared across all steps.

The same pattern continues into Phases 2-4, though with fewer stages per phase:

| Spec Focus | Plan Phases |
|---|---|
| Core compiler MVP | Phase 1 (14 stages) |
| Ownership, Effects, Embedding, Verification baseline | Phase 2 (4 stages) |
| Specialization, Optimization, Ecosystem | Phase 3 (3 stages) |
| Dynamic Compatibility, Deep Verification | Phase 4 (2 stages) |

### Definition of "Workable State"

The meaning of "workable" evolves across phases:

| Phase | Workable State Criteria |
|---|---|
| **Phase 1** | Can compile .ts/.js files to WASM; `run`, `check`, `build`, `test` work for local file-based programs; basic package install works |
| **Phase 2** | Can produce verifiable ownership semantics; public effect-reporting commands work; can generate embedding artifacts (WIT/CABI) |
| **Phase 3** | Can produce optimized builds with measurable perf gains; can run programs with Node APIs; cross-module optimization works |
| **Phase 4** | Supports `eval`/`Function()` safely; has non-empty published verification boundary; can make proof-backed claims |



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

**Goal:** a dependable, end-to-end TypeScript/JavaScript → WebAssembly compiler that can check,
run, test, and bundle real programs in the Deno-oriented standalone context and the Phase-1
browser-targeted command set, with sandbox enforcement, basic package management, and a
proof-ready repository baseline.

**Workable state:** Can compile local .ts/.js files to WASM; `run`, `check`, `build`, `test` work for local file-based programs; sandbox policies enforce at runtime; basic package install works for npm/JSR packages under the pure JS/TS contract. All Phase 1 commands work in the **Default standalone context** or **browser-targeted** variants.

**Dependencies to complete Phase 1:** Stages 1.1–1.8 must complete before 1.9–1.14 begin, as later stages depend on the execution capability. Parallel development groups exist but require careful coordination on shared schemas and diagnostics registry.

### Foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.1 | [Workspace & Crate Scaffold](plan/phase-1/01-workspace-scaffold.md) | `cargo build` succeeds; `kali --version` prints a version string; `proofs/BOUNDARY.md` establishes the verification-boundary discipline **✅ COMPLETE** |
| 1.2 | [Lexer](plan/phase-1/02-lexer.md) | Tokenises valid TS/JS source; emits stable `E1xxx` lex errors **✅ COMPLETE** |
| 1.3 | [Parser & AST](plan/phase-1/03-parser-and-ast.md) | Parses full ECMA-262 + TypeScript grammar; AST node types defined **✅ COMPLETE** |

**NOTE:** Stage 1.1 establishes the **proof-ready** baseline. In the current repository state, later work has advanced beyond that baseline and the repository is now **proof-backed for the published boundary**; Stage 1.1 still owns the original boundary-discipline bootstrap.

### Spec Step 1 — Frontend + checking foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.2 | [Lexer](plan/phase-1/02-lexer.md) | Tokenises valid TS/JS source; emits stable `E1xxx` lex errors |
| 1.3 | [Parser & AST](plan/phase-1/03-parser-and-ast.md) | Parses full ECMA-262 + TypeScript grammar; AST node types defined |
| 1.4 | [Name Resolution](plan/phase-1/04-name-resolution.md) | `kali check` reports unresolved identifiers and import errors **✅ COMPLETE** |
| 1.5 | [Type Checker](plan/phase-1/05-type-checker.md) | `kali check` reports type errors under the bounded inference contract **✅ COMPLETE** |

### Spec Step 3 — Kali-hosted execution foundation *(before step 2 for workability; see ordering note)*

| Stage | Document | Workable milestone |
|---|---|---|
| 1.6 | [HIR & LIR Lowering](plan/phase-1/06-hir-lir-lowering.md) | Full compiler pipeline exists; LIR can be inspected / round-tripped **✅ COMPLETE** |
| 1.7 | [WASM Code Generation](plan/phase-1/07-wasm-codegen.md) | Simple programs compile to runnable WASM modules **✅ COMPLETE** |
| 1.8 | [Runtime & Execution](plan/phase-1/08-runtime-execution.md) | `kali run` and `kali test` work in the Default standalone context **✅ COMPLETE** |

### Spec Steps 2 & 4 — Package, sandbox & build foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.9 | [Sandbox & Policy](plan/phase-1/09-sandbox-and-policy.md) | `--sandbox` flag enforced at runtime; policy files validated at check/build time on the **Phase-1 static policy-validation surface** **✅ COMPLETE** |
| 1.10 | [Package Management](plan/phase-1/10-package-management.md) | `kali install` resolves npm/JSR/raw-URL deps; lock file is deterministic; package compatibility follows the **package-support decision order** **✅ COMPLETE** |
| 1.11 | [Build Artifacts](plan/phase-1/11-build-artifacts.md) | `kali build` emits executables; `--bundle` emits browser bundles in the **Phase-1 browser-targeted command set**; `--lib` emits the **base library artifact** for exact-version consumers **✅ COMPLETE** |

### Spec Step 5 — Developer workflow foundation

| Stage | Document | Workable milestone |
|---|---|---|
| 1.12 | [Developer Workflow](plan/phase-1/12-developer-workflow.md) | `kali init`, `kali fmt`, `kali lint` (with canonical `W2xxx` lint diagnostics) all functional **✅ COMPLETE** |
| 1.13 | [Diagnostics & Schemas](plan/phase-1/13-diagnostics-and-schemas.md) | Stable error codes; `--output json` emits schema-v1 envelopes **✅ COMPLETE** |

### Spec Step 6 — Evidence hardening

| Stage | Document | Workable milestone |
|---|---|---|
| 1.14 | [Evidence Hardening](plan/phase-1/14-evidence-hardening.md) | Conformance suite (unit/integration), TypeScript/JavaScript checker baseline, package-corpus checks under the **linked-artifact model**, browser-targeted smoke tests, determinism checks, and a passing proof-ready CI pipeline **✅ COMPLETE** |

### Phase 1 parallelism and coordination

Within Phase 1, certain stages can be developed in parallel **after the critical path** (1.1-1.8) is complete. This allows teams to work on independent streams while maintaining a coherent project state.

**Critical path (sequential):** 1.1 → 1.2 → 1.3 → 1.4 → 1.5 → 1.6 → 1.7 → 1.8

**Parallelizable after critical path:** 1.9-1.14 can begin once 1.8 completes, with the following coordination requirements:

| Parallel Group | Stages | Dependency | Coordination requirements |
|---|---|---|---|
| Static validation | 1.9 | 1.5, 1.8 | Sandbox policy schema in 18-schemas.md; E9xx diagnostics |
| Lowering pipeline (early) | 1.6, 1.7 | 1.5 | Share IR definitions; HIR/LIR round-trip tests pass |
| Package management | 1.10 | 1.8, 1.6-1.7 | E6xx diagnostics; package-resolution interfaces in 14-packages.md |
| Build artifacts | 1.11 | 1.9, 1.6-1.7 | Artifact modes in 11-build-artifacts.md; WIT output schema |
| Developer workflow | 1.12 | 1.11 | CLI flags in 12-cli.md; must pass fixture suite |
| Diagnostics & schemas | 1.13 | 1.11 | E5xx/E9xx registry; JSON envelopes in 18-schemas.md |
| Evidence infrastructure | 1.14 | 1.11, 1.12 | Conformance suite; determinism checks; CI pipeline config |

**Workability coordination notes:**
- **Critical path awareness:** All parallel work waits for 1.8 (runtime execution) because packages (1.10) cannot demonstrate end-to-end without execution; build artifacts (1.11) cannot validate without sandbox (1.9); evidence (1.14) cannot measure without CLI surface (1.11-1.12).
- **Schema coordination:** All parallel streams must reference the canonical diagnostic registry (E5xx, E6xx, E9xx namespaces) from [specs/15-errors.md](./specs/15-errors.md) and JSON envelopes from [specs/18-schemas.md](./specs/18-schemas.md). Changes to shared definitions require cross-review.
- **Test suite validation:** Before any parallel stream commits work, run `cargo test --workspace` to ensure it does not break existing functionality. Parallel streams may add tests but must not remove or modify existing test expectations.
- **CLI command registry:** Each parallel stream must update the canonical command surface in [specs/12-cli.md](./specs/12-cli.md) with matching implementations. A command documented as "defined early" before it ships must not claim phase availability in [specs/19-feature-maturity.md](./specs/19-feature-maturity.md).

### Phase 1 completion gate

Phase 1 is complete when **all** of the following conditions are met:

- [x] All stages 1.1–1.14 have passed their individual Definitions of Done
- [x] Every Phase-1 maturity label in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) is backed by a passing evidence track from stage 1.14
- [x] `cargo test --workspace` passes with no regressions
- [x] End-to-end smoke tests for `kali check`, `build`, `run`, `test`, `init`, `fmt`, `lint`, `install` succeed on at least one real-world TypeScript project
- [x] `proofs/BOUNDARY.md` exists with the Phase-1 proof-ready boundary (non-empty for proof-backed claims)
- [x] Phase-1 browser-targeted smoke tests pass for the **Phase-1 browser-targeted command set**
- [x] Determinism checks pass for all CLI outputs and generated artifacts

---

## Phase 2 — Ownership, Effects & Public Embedding

**Goal:** MIR-backed memory management with deterministic ownership/escape analysis; the stable public effect-report surface (`kali effects`, `kali package-effects`); compile-time inferred-effect-vs-policy validation; the stable public embedding surface (Rust API, WIT-first `--lib`, `--capi`, `--component`); and the Lean 4 core-type-calculus model that begins the formal verification programme.

**Workable state:** Can produce WASM with verifiable ownership semantics; public effect-reporting commands work (`kali effects <file>`, `kali package-effects <pkg>`); can generate embedding artifacts (WIT sidecar, C ABI headers, Component Model); check/build with `--sandbox` rejects programs that would violate policy at runtime.

**Critical path:** 2.1 (MIR/ownership) must complete before 2.2 and 2.3 can proceed. Stage 2.4 (Lean foundation) has a dependency on 2.1 for the memory-safety proof, but the type-calculus model can begin as soon as Phase 1 completes since it models the type system rather than implementation specifics.

**Stage dependencies:**
- 2.1 → 2.2 (effect reporting needs ownership semantics to analyze)
- 2.1 → 2.3 (embedding artifacts need stable export surface from ownership analysis)
- 2.1 → 2.4 (memory-safety proofs depend on the ownership model)
- 1.4 → 2.4 (type-calculus model depends on Phase 1's type system implementation)

### Spec chapter mapping

| Spec chapter | Plan stage | Key deliverable |
|---|---|---|
| [`05 — IR`](./specs/05-ir.md) | 2.1 | MIR as canonical mid-stage; `HIR → MIR → LIR` path replaces direct lowering |
| [`06 — Memory Management`](./specs/06-memory.md) | 2.1 | Escape analysis; deterministic ownership classes (`stack`, `owned heap`, `shared heap`, `borrowed`) |
| [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md) | 2.2 | Public effect-report surface (reporting half + policy-comparison half) |
| [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md) | 2.3 | Stable Rust embedding API; WIT-first `--lib`; `--capi`; `--component` |
| [`17 — Formal Verification`](./specs/17-verification.md) | 2.4 | Lean 4 workspace; core type-calculus model; progress + preservation proved; real CI proof jobs |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | all | Phase-2 maturity rows open |

| Stage | Document | Workable milestone |
|---|---|---|
| 2.1 | [MIR & Ownership Analysis](plan/phase-2/01-mir-and-ownership.md) | MIR is the canonical mid-stage; escape analysis drives stack/heap/shared decisions **✅ COMPLETE** |
| 2.2 | [Public Effect Reporting](plan/phase-2/02-public-effect-reporting.md) | `kali effects <file>` and `kali package-effects <pkg>` emit stable JSON; `check/build --sandbox` adds inferred-effect-vs-policy rejection **✅ COMPLETE** |
| 2.3 | [Public Embedding Surface](plan/phase-2/03-public-embedding-surface.md) | Stable Rust embedding API; WIT sidecar on `--lib`; `--capi` and `--component` artifact modes **✅ COMPLETE** |
| 2.4 | [Lean Model Foundation](plan/phase-2/04-lean-model-foundation.md) | Lean workspace and core type-calculus model established; proof CI runs; this stage's provisional-boundary groundwork later feeds the current proof-backed boundary **✅ COMPLETE** |

### Phase 2 completion gate

Phase 2 is complete when **all** of the following conditions are met:

- [x] All stages 2.1–2.4 have passed their individual Definitions of Done
- [x] Public effect-report surface is stable: `kali effects` and `kali package-effects` produce schema-v1 JSON
- [x] Public embedding surface is stable: `kali build --lib` emits WIT; `--capi` and `--component` artifact modes work
- [x] The stable semver boundary has been published for embedding APIs
- [x] Lean type-soundness proof (progress + preservation) is implemented and CI runs proof jobs
- [x] Phase-2 maturity rows in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) are updated to reflect passing evidence
- [x] `cargo test --workspace` passes with no regressions

---

## Phase 3 — Specialization, Optimization & Ecosystem Breadth

**Goal:** generic/function/layout specialization at compile time; stronger optimization tiers; incremental compilation; broader npm/Node compatibility beyond the Phase-1 pure-JS/TS baseline; and broader browser packaging.

**Workable state:** `--release` and `--release-advanced` produce measurably faster WASM than `--fast`; monomorphization pipeline produces specialized code; can run programs with Node APIs via `--api node`; can build with incremental compilation for faster rebuilds; broader npm/JSR packages work beyond the pure JS/TS baseline.

**Critical path:** 3.1 (optimization/specialization) is the foundational work — 3.3's code splitting and tree-shaking depend on 3.1's monomorphization being stable.

**Stage dependencies:**
- 3.1 → 3.3 (code splitting needs monomorphization for safe export analysis)
- Phase 2 → 3.2 (Node compatibility can begin after Phase 2; has fewer dependencies on Phase 3 internals)

**Parallel development:** 3.1, 3.2, and 3.3 **cannot** all run in parallel. The correct sequence is:
1. Complete 3.1 first (optimization infrastructure)
2. Then begin 3.2 **or** 3.3 in parallel (Node compatibility and Ecosystem breadth are independent once specialization works)

### Spec chapter mapping

| Spec chapter | Plan stage | Key deliverable |
|---|---|---|
| [`05 — IR`](./specs/05-ir.md) | 3.1 | Layout specialization via MIR layout descriptors |
| [`07 — Optimization & Specialization`](./specs/07-specialization.md) | 3.1 | Monomorphization; `--release` / `--release-advanced` optimization passes |
| [`08 — WASM Code Generation`](./specs/08-wasm-codegen.md) | 3.3 | Code splitting; tree-shaking; dynamic `import()` bundle boundaries |
| [`10 — Runtime`](./specs/10-runtime.md) | 3.2 | Node compatibility surface; common Node built-ins |
| [`11 — Standard APIs`](./specs/11-standard-apis.md) | 3.2 | Node compatibility surface; broader Node built-ins |
| [`12 — CLI`](./specs/12-cli.md) | 3.2 | CLI flag wiring for `--api node` command path |
| [`14 — Package Management`](./specs/14-packages.md) | 3.2, 3.3 | Broader npm corpus; Node-assuming package support |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | all | Phase-3 maturity rows open |

| Stage | Document | Workable milestone |
|---|---|---|
| 3.1 | [Optimization & Specialization](plan/phase-3/01-optimization-and-specialization.md) | `--release` and `--release-advanced` produce measurably faster WASM; monomorphization pipeline stable; incremental compilation reduces rebuild times |
| 3.2 | [Node Compatibility](plan/phase-3/02-node-compatibility.md) | `--api node` command path supported; **Node compatibility surface** available; Node-assuming packages that fit the **pure JS/TS package contract** become executable |
| 3.3 | [Ecosystem Breadth](plan/phase-3/03-ecosystem-breadth.md) | Incremental compilation; broader npm / JSR package corpus; open-ended cross-module constraint solving within the **bounded inference contract**; deeper browser-bundle tooling |

### Phase 3 completion gate

Phase 3 is complete when **all** of the following conditions are met:

- [x] All stages 3.1–3.3 have passed their individual Definitions of Done
- [x] `--release` and `--release-advanced` modes produce measurably better performance than `--fast` on CI benchmark suite
- [x] `--api node` command path is available and stable
- [x] The optimization/specialization pipeline (3.1) is stable and incremental compilation reduces rebuild times
- [x] Phase-3 maturity rows in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) are updated to reflect passing evidence
- [x] `cargo test --workspace` passes with no regressions

---

## Phase 4 — Advanced Compatibility & Deep Verification

**Goal:** hardest dynamic features (`eval`, `Function()`, non-literal dynamic imports); deeper API coverage; and proof-backed release claims with a non-empty published Lean boundary.

**Workable state:** `eval`/`Function()` work safely behind `compat.features.eval`; can use non-literal dynamic imports with gated availability; `kali package-audit` is stable without `--preview` gate; can make **proof-backed** release claims for a non-empty published verification boundary in `proofs/BOUNDARY.md`.

**Critical path:** 4.2 (formal verification depth) depends on 2.4 (Lean Model Foundation from Phase 2), which establishes the Lean workspace and CI proof jobs. Stage 2.4 must complete before 4.2 can produce **proof-backed** claims.

**Stage dependencies:**
- 2.4 → 4.2 (4.2 builds on the Lean foundation from 2.4; the proof CI jobs and core calculus model must exist first)
- 4.1 depends on no other Phase 4 stage, but `eval` support requires the type system from Phase 1 and the CLI framework developed throughout

### Spec chapter mapping

| Spec chapter | Plan stage | Key deliverable |
|---|---|---|
| [`10 — Runtime`](./specs/10-runtime.md) | 4.1 | `eval` / `Function()` behind `compat.features.eval`; non-literal dynamic imports |
| [`12 — CLI`](./specs/12-cli.md) | 4.1 | CLI flag wiring for `compat.features` system |
| [`14 — Package Management`](./specs/14-packages.md) | 4.1 | `kali package-audit` publicly available (no `--preview` gate) |
| [`17 — Formal Verification`](./specs/17-verification.md) | 4.2 | Non-empty Lean proof boundary; proof CI passes; repository may claim proof-backed |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | all | Phase-4 maturity rows open |

| Stage | Document | Workable milestone |
|---|---|---|
| 4.1 | [Dynamic Compatibility](plan/phase-4/01-dynamic-compatibility.md) | `eval` / `Function()` executable behind `compat.features.eval`; non-literal dynamic loading gated similarly; `package-audit` command stable |
| 4.2 | [Formal Verification Depth](plan/phase-4/02-formal-verification-depth.md) | Non-provisional Lean boundary published; the repository is proof-backed for the published boundary, while wider verification depth remains a follow-up lane |

### Phase 4 completion gate

Phase 4 is complete when **all** of the following conditions are met:

- [x] All stages 4.1–4.2 have passed their individual Definitions of Done
- [x] `eval`/`Function()` support works safely behind `compat.features.eval` compatibility flag
- [x] `kali package-audit` is stable and no longer gated behind `--preview`
- [x] `proofs/BOUNDARY.md` documents a non-empty modelled subsystem with passing Lean proof jobs
- [x] Repository may claim **proof-backed** status for the published boundary (not merely proof-ready)
- [x] The type-soundness proof (progress + preservation) is fully implemented for the Phase-1 core
- [x] Phase-4 maturity rows in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) are updated to reflect passing evidence
- [x] `cargo test --workspace` passes with no regressions

### Current post-completion follow-up lanes

All four phase groups are implemented in the current repository state, but completed phase gates do **not** mean every long-term breadth target is exhausted. The current top-level follow-up lanes are:

- **Stage 3.1 specialization depth** — continue broadening MIR-aware specialization from the current layout-stable clone path toward the fuller generic-instantiation planner and cross-module specialization model, while preserving deterministic specialization-budget enforcement and the existing benchmark/evidence suite; current progress now scopes MIR-backed binding layouts by function owner so same-named bindings in different functions can specialize independently, the pure-LIR release path can still clone deterministic generic/function helpers from literal-shaped call sites even when MIR layout metadata is unavailable, and the release-advanced MIR-specialized clone path keeps the generic-instantiation path available inside its specialized bodies so large generic callees can still clone and fold after the layout pass narrows the arguments, `null` / `undefined` literal arguments keep distinct signatures instead of collapsing onto the old zero-valued fallback, boolean literal arguments now keep their `true` / `false` identity in the specialization signature, quoted string-literal, no-substitution template-literal, and RegExp-literal arguments keep distinct signatures, and the MIR-aware specialization path preserves that same regex-literal split when it has to run without layout metadata, signed-zero `-0` stays distinct from `0` in literal signatures, `Infinity`, `-Infinity`, and `NaN` keep distinct signatures in the same literal-signature path, array-valued MIR bindings now preserve their element/length fingerprints during specialization so the array-layout widening can split into separate clones, direct array-literal call-site arguments now carry explicit `Value:array:len=...` shape signatures so the direct array-literal shape widening splits inline arrays with different lengths apart too, object-literal property order is canonicalized so semantically identical object shapes with reordered fields reuse one clone, the MIR-aware reuse path now checks the specialization cache before spending the current owner's remaining budget so already-materialized clones stay reusable even after the owner has exhausted its slots, and the nested-call regression `release_recursively_specializes_nested_mir_call_sites` keeps deeper monomorphisation layered inside a specialized clone, and the new explicit `public` / `bridge` / helper re-export-chain regression keeps the same generic helper clone reused once while the bridge wrapper itself still specializes once in release mode.
- **Stage 3.3 ecosystem breadth** — continue widening the representative package corpus where real-world browser/utility/Node package shapes are still missing, without changing package-audit availability or package-support rung claims until the evidence matrix says otherwise; current progress now also carries `classnames`, `mobx`, `@heroicons/react`, `lucide-react`, `react-dom`, `recoil`, `reselect`, `mitt`, `swr`, `formik`, `pinia`, `xstate`, `valtio`, and now also carries `xstate` through the browser and utility exports-map and pattern-exports slices so the state-management shape coverage stays concrete, `superjson`, `chart.js`, `recharts`, `@jridgewell/sourcemap-codec`, `@emotion/react`, `@emotion/styled`, `framer-motion`, `@storybook/react`, `@radix-ui/react-dialog`, `@tanstack/react-form`, `@tanstack/react-table`, `@apollo/client`, and `@testing-library/user-event` through the browser and utility web-baseline interop corpora alongside the existing scoped browser package names, now also carries `react-dom` through the browser exports-map and browser-condition slices, now also carries `dayjs` through the browser exports-map and browser-condition slices, now also carries `jotai` through the browser exports-map and browser-condition slices, now also carries `@chakra-ui/react` through the browser exports-map and browser-condition slices, now also carries `@mantine/core` through the browser exports-map and browser-condition slices, now also carries `@radix-ui/react-dialog` through the scoped browser exports-map and browser-condition slices, now also carries `hono` through the browser exports-map and pattern-exports slices, now also carries `redux` through the utility exports-map, string-exports, and pattern-exports slices, now also carries `lodash` through the utility plain-package, exports-map, string-exports, pattern-exports, and web-baseline slices, and the browser web-baseline corpus now explicitly exercises `@storybook/react` in the actual package-corpus test suite, while the scoped browser corpus now also carries `@storybook/react` through the exports-map and browser-condition slices, so browser-style package-shape coverage stays mirrored in code as well as in the progress note, and now also captures deterministic `EventTarget` listener removal — including safe removal during dispatch — in the shared browser/runtime support library so browser-style listener lifecycle tests can stay faithful without changing the documented support rungs, and the browser web-baseline interop corpus now also exercises `redux` as another representative browser state-management package name without changing the support-rung story, and now also exposes deterministic `navigator` metadata (`userAgent`, `language`, `onLine`) in the shared browser/runtime support library so browser-style code can inspect ambient browser metadata without changing the support-rung story, and now also exposes a `random_uuid` helper for `crypto.randomUUID()`-style calls while `kali_runtime` wires the matching `crypto_random_uuid` / `cryptoRandomUUID` host imports through that helper so the browser UUID slice stays covered without changing the support-rung story, and now also exposes an `IndexedDB` alias for the in-memory browser-runtime stub, and the Deno compatibility surface reexports that browser-aligned name too so Rust-facing code can mirror the docs while the lower-case `indexedDB` global stays the corpus source of truth, and now also carries `vite` through the utility plain-package and web-baseline interop slices so one more modern build-tool package name stays covered without changing the support-rung story, and now also carries `luxon` through the browser web-baseline, utility plain-package, utility web-baseline interop, and utility module-entry slices so one more date-time package name stays covered without changing the support-rung story, and now also carries `react` and `preact` through the utility plain-package slice on the default standalone surface so the representative React/Preact package breadth stays concrete without changing the support-rung story, and now also carries `rambda` through the browser web-baseline interop and utility plain-package slices so one more functional-utility package name stays covered without changing the support-rung story, and now also carries `rxjs` through the browser web-baseline interop and utility plain-package slices so one more observable/stream utility package name stays covered without changing the support-rung story, and now also carries `axios` through the utility plain-package slice on the default standalone surface so one more common pure-JS package stays covered without changing the support-rung story, and now also carries `msw` through the utility plain-package slice on the default standalone surface so one more browser/networking package name stays covered without changing the support-rung story, and now also carries `yaml` through the browser web-baseline interop and utility plain-package / web-baseline interop slices so one more pure-JS data-format package name stays covered without changing the support-rung story, and now also carries `query-string` through the browser web-baseline interop corpus and the utility plain-package surface so one more query-string package name stays covered without changing the support-rung story, and the utility exports-map and pattern-exports corpus now also carries `reselect` so one more state-management package shape stays covered without changing the support-rung story, and now also carries `graphql` through the browser web-baseline interop and utility plain-package slices so one more common JS package name stays covered without changing the support-rung story, and now also carries `@tanstack/router` through the browser and utility web-baseline interop slices so one more representative scoped routing package name stays covered without changing the support-rung story, and now also carries `@tanstack/react-router` through the browser web-baseline interop and scoped browser exports-map/browser-condition slices so one more representative scoped routing package name stays covered without changing the support-rung story, and now also carries `@tanstack/table-core` through the browser web-baseline interop and scoped browser exports-map/browser-condition slices so one more representative scoped table package name stays covered without changing the support-rung story, and now also carries `@tanstack/router` through the scoped browser exports-map and browser-condition slices so one more representative scoped routing package name stays covered without changing the support-rung story, and now also carries `zustand` through the scoped browser exports-map and browser-condition slices so one more representative scoped state-management package name stays covered without changing the support-rung story, and now also exercises `localStorage` / `sessionStorage` in the shared browser/runtime baseline so the browser/utility interoperability slice keeps its browser-state helpers concrete without changing the support-rung story, and now also exercises `atob` / `btoa` in the shared browser/runtime baseline so the browser/utility interoperability slice keeps its binary-string helpers concrete without changing the support-rung story, and now also exercises `@playwright/test` in the browser web-baseline interop corpus so one more representative browser test-runner package name stays covered without changing the support-rung story, and now also carries `@remix-run/react` through the browser web-baseline interop and scoped browser exports-map/browser-condition slices, and now also carries `react-router-dom` through the browser exports-map slice, and now also carries `react-helmet-async` through the browser web-baseline interop slice as another representative head-management package name, and now also carries `@stripe/react-stripe-js` through the browser web-baseline interop slice as another representative browser payment/UI package name, and the scoped browser corpus now also carries it through the exports-map and browser-condition slices so one more representative browser payment/UI package shape stays covered without changing the support-rung story, and now also carries `ajv` through the browser web-baseline interop and utility plain-package slices so one more representative validation package name stays covered without changing the support-rung story, and now also carries `tailwindcss` through the utility plain-package slice on the default standalone surface so one more build-tool package name stays covered without changing the support-rung story, and the Node-runner corpus now also exercises `ava` in the exports-map and mixed-format slices so the test-runner breadth stays concrete without changing the support-rung story, and the Node-assuming corpus now also exercises `dotenv` under the Node context so one more common Node-only package shape stays concrete without changing the support-rung story, and the scoped browser corpus now also exercises `@vueuse/core` across the exports-map and browser-condition slices so one more representative scoped browser utility package shape stays concrete without changing the support-rung story, and the browser web-baseline interop corpus now also exercises `@tanstack/query-core` alongside the utility plain-package slice so one more representative scoped query package name stays concrete without changing the support-rung story, and the browser web-baseline interop corpus now also exercises `@babel/runtime` and `@npmcli/package-json` as another representative scoped utility package name pair without changing the support-rung story.
- **Stage 3.1 specialization depth** — continue broadening MIR-aware specialization from the current layout-stable clone path toward the fuller generic-instantiation planner and cross-module specialization model, while preserving deterministic specialization-budget enforcement and the existing benchmark/evidence suite; current progress now scopes MIR-backed binding layouts by function owner so same-named bindings in different functions can specialize independently, the pure-LIR release path can still clone deterministic generic/function helpers from literal-shaped call sites even when MIR layout metadata is unavailable, and the release-advanced MIR-specialized clone path keeps the generic-instantiation path available inside its specialized bodies so large generic callees can still clone and fold after the layout pass narrows the arguments, and MIR-specialized clones now keep generic specialization enabled inside their bodies in `release` too, so a layout-specialized wrapper can still clone and fold a large generic callee after the MIR pass narrows its arguments, `null` / `undefined` literal arguments keep distinct signatures instead of collapsing onto the old zero-valued fallback, boolean literal arguments now keep their `true` / `false` identity in the specialization signature, quoted string-literal, no-substitution template-literal, and RegExp-literal arguments keep distinct signatures, and the MIR-aware specialization path preserves that same regex-literal split when it has to run without layout metadata, signed-zero `-0` stays distinct from `0` in literal signatures, `Infinity`, `-Infinity`, and `NaN` keep distinct signatures in the same literal-signature path, array-valued MIR bindings now preserve their element/length fingerprints during specialization so the array-layout widening can split into separate clones, direct array-literal call-site arguments now carry explicit `Value:array:len=...` shape signatures so the direct array-literal shape widening splits inline arrays with different lengths apart too, object-literal property order is canonicalized so semantically identical object shapes with reordered fields reuse one clone, the MIR-aware reuse path now checks the specialization cache before spending the current owner's remaining budget so already-materialized clones stay reusable even after the owner has exhausted its slots, and the nested-call regression `release_recursively_specializes_nested_mir_call_sites` keeps deeper monomorphisation layered inside a specialized clone.
- **Stage 3.3 scoped utility corpus breadth** — continue widening the default-standalone scoped utility package corpus where representative package shapes are still missing; current progress now also carries `@babel/runtime` and `@npmcli/package-json` through the scoped-package slice on the default standalone surface, keeping one more representative scoped utility package shape covered without changing the support-rung story.
- **Stage 3.3 plain-package breadth** — continue widening the default-standalone plain-package corpus where representative pure-JS package shapes are still missing; current progress now also carries `deepmerge` and `@tanstack/query-core` on the default standalone surface, keeping two more common package names covered without changing the support-rung story.
- **Stage 4.2 verification depth** — continue widening the proof model beyond the current published proof-backed boundary toward the fuller ownership / RC decrement-and-freeing story and the wider HIR → LIR semantic-preservation target, while keeping `proofs/BOUNDARY.md` and the proof-summary anti-drift guard honest about the narrower proof-backed slice that exists today; the published slice already includes the explicit linear-memory payload preservation corollaries, the combined wellformedness/linear-memory corollaries, the combined wellformedness/ownership/linear-memory corollaries, and the release-only helper's `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` companion alongside the decrement companion `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` and the collection companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, and the mechanized `KaliCore.Safety.noDanglingReference` theorem plus the helper-level no-dangling-reference corollaries `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`, so the remaining work now sits beyond that owned-payload bridge. The local collection helper now also carries the matching origin/positive-count + linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory`, the collection target-cell iff theorem `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, the collection target-cell allocation corollary `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, and the heap-filter + linear-memory corollary `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`, while the collection target-cell origin/ownership theorem `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCount` now also has its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`, and the release-and-decrement target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory`, and the target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, and the final-heap positivity theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` remains named explicitly in the current boundary summary.

Reading rule:
- use completed phase gates to answer whether a phase contract has been met,
- but use the stage docs, status trackers, and `proofs/BOUNDARY.md` to answer which breadth/depth follow-up work still remains after that gate.

---

## Cross-Cutting Rules

* **Hard invariants never bend.** AOT-only, pure-Rust implementation, no tracing/background GC,
  sandbox-first honesty, and deterministic machine contracts hold across all phases.
* **Each stage must leave the project workable.** No stage may break existing tests or make a
  previously-functional CLI command regress. A feature may land internally before it is marked
  as available in the maturity table — the public availability surface always reads from
  [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).
* **Availability follows `specs/19-feature-maturity.md`.** A stage completing its implementation
  work does not automatically promote a feature's maturity label — that requires the matching
  evidence from the canonical testing tracks in `specs/16-testing.md`.
* **Proof-ready from day one.** `proofs/BOUNDARY.md` and the proof-CI trigger policy must exist
  from Stage 1.1 onward; proof-backed claims require a non-empty published boundary.
* **Stage parallelism is opt-in.** Unless a stage document explicitly notes that its work can
  proceed in parallel with another, assume sequential ordering within each phase. When working
  in parallel, coordinate on:
  - Shared schema definitions (18-schemas.md) — diagnostic codes, JSON envelopes must not diverge
  - Command availability tracking — know which features are internally implemented vs. publicly
    available according to the maturity matrix
  - Existing test suite — validate changes against passing tests from prior sequential stages
  - Command shape consistency — CLI subcommand definitions and flags must align across parallel
    development streams
* **Spec chapter alignment.** References to spec chapters should use the exact chapter headings
  from [`specs/`](./specs/) (e.g., "Optimization & Specialization", "Package Management", "WASM Code Generation", etc.)
  to ensure consistent navigation across documents.
