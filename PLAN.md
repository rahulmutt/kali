# Kali — Implementation Plan

`PLAN.md` is the implementation playbook for [`SPEC.md`](./SPEC.md). It maps the spec phases onto concrete, workable stages and defines the recommended execution order.

## Plan contract

After every stage the repository must remain in a workable state:

1. `cargo build` succeeds
2. `cargo test --workspace` passes
3. At least one user-visible capability is demonstrable
4. Hard invariants still hold: AOT-only, pure Rust, no tracing/background GC, sandbox-first honesty, deterministic machine contracts

**Normative source of truth:** [`SPEC.md`](./SPEC.md), the owning chapter in [`specs/`](./specs/), and actual public availability in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

---

## Stage layout

```text
plan/
├── phase-1/
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
├── phase-2/
│   ├── 01-mir-and-ownership.md
│   ├── 02-public-effect-reporting.md
│   ├── 03-public-embedding-surface.md
│   └── 04-lean-model-foundation.md
├── phase-3/
│   ├── 01-optimization-and-specialization.md
│   ├── 02-node-compatibility.md
│   └── 03-ecosystem-breadth.md
└── phase-4/
    ├── 01-dynamic-compatibility.md
    └── 02-formal-verification-depth.md
```

Total: 20 stage documents across 4 phases.

---

## How to use this document

- Use `SPEC.md` to decide what Kali promises.
- Use this plan to decide implementation order and dependencies.
- Use `specs/19-feature-maturity.md` to answer whether something is publicly available.
- Use the stage files under `plan/` for detailed tasks and definitions of done.

Each stage document should define:
- Goal
- Workable milestone
- Depends on
- Tasks
- Out of scope
- Definition of done

---

## Planning ownership

Planning material lives here, not in `SPEC.md`.

- `SPEC.md` defines cross-spec normalization, phase contracts, and release-claim rules.
- `PLAN.md` defines implementation order, dependency structure, phase sequencing, and completion gates.
- `plan/**/*.md` defines the detailed stage tasks and concrete definitions of done.

If a sentence is primarily about **what gets built when**, **what depends on what**, or **what a stage must prove before moving on**, it belongs in the plan set.

Recent compaction rule:
- `specs/16-testing.md` now owns the normative evidence requirements only; concrete CI rollout, benchmark automation, and staged evidence expansion live in [phase-1/14](./plan/phase-1/14-evidence-hardening.md) and later stage files.
- `specs/17-verification.md` now owns proof-boundary discipline only; concrete Lean milestones, first proof-backed scope growth, and deeper verification rollout live in [phase-2/04](./plan/phase-2/04-lean-model-foundation.md) and [phase-4/02](./plan/phase-4/02-formal-verification-depth.md).

---

## Implementation strata

The spec set is easiest to implement in four broad delivery strata:

| Stratum | Purpose | Primary chapters | Plan ownership |
|---|---|---|---|
| Bootstrap normalization + cross-spec rules | Turn `BOOTSTRAP.md` into phase-correct claims and shared vocabulary | `SPEC.md`, `specs/19-feature-maturity.md` | this document sets the delivery sequencing that respects those claims |
| Frontend + semantics | Parse TS/JS, build typed meaning, and enforce the bounded inference contract | `specs/01`-`specs/04` | Phase 1 stages 1.1-1.5 |
| Lowering + runtime core | Lower through IR, generate WASM, execute safely, and define host/runtime behavior | `specs/05`-`specs/11` | Phase 1 stages 1.6-1.11, then Phase 2 stage 2.1 |
| Product/tooling surface | CLI, packages, diagnostics, schemas, testing, embedding, verification | `specs/12`-`specs/18` | Phase 1 stages 1.9-1.14, Phase 2+, and later evidence/depth work |

This stratum view is for implementation planning only. It does not change normative ownership or public availability.

---

## Phase map

| Phase | Focus | Workable outcome |
|---|---|---|
| 1 | Core compiler and toolchain MVP | Check, run, build, test, install, sandbox, and basic browser-targeted build path |
| 2 | Ownership, effects, embedding, verification foundation | MIR ownership model, public effect reporting, stable embedding artifacts, Lean foundation |
| 3 | Optimization and ecosystem breadth | Specialization, stronger release modes, Node compatibility, broader package support |
| 4 | Advanced compatibility and deeper verification | Dynamic compatibility features and proof-backed published boundary |

---

## Important ordering decisions

### Phase contracts vs implementation order

Kali uses two different orderings on purpose:

- **phase contracts** describe the earliest user-visible support promise for a feature;
- **implementation order** describes the recommended engineering sequence for getting there.

A feature may be documented early for naming stability without being publicly available. This plan never overrides `specs/19-feature-maturity.md`; it only explains the recommended build order.

### Phase 1 recommended implementation order

Phase 1 should be approached in this order:

1. **Frontend + checking foundation** — lexer, parser, AST, name resolution, TypeScript-compatible checking, and first-class JavaScript handling.
2. **Deterministic package/install foundation** — lock/materialization rules and strict non-mutating behavior for non-install commands.
3. **Kali-hosted execution foundation** — one AOT pipeline to one linked WASM payload, `run`/`test` in the default standalone context, and the Phase-1 sandbox/runtime contract.
4. **Build/artifact foundation** — executable builds, browser bundles, and the Phase-1 base library artifact.
5. **Developer workflow foundation** — `init`, `check`, `fmt`, `lint`, diagnostics, and schema-v1 machine-readable outputs.
6. **Phase-1 evidence hardening** — conformance, package corpus, browser smoke, determinism, and proof-ready CI maintenance.

This ordering is reflected by the stage graph below, even where practical sequencing differs slightly for workability.

### Execution before package management

Within Phase 1, execution work comes before package management:

- `1.6 → 1.8` (lowering, codegen, runtime)
- then `1.10` (package management)

This is intentional. An end-to-end compiler/runtime for local files is already workable. Package installation is more valuable once execution exists. This does **not** change the Phase-1 contract; it only changes implementation order.

### Proof-readiness starts at Stage 1.1

Proof-readiness is not a final cleanup task. The repository should publish `proofs/BOUNDARY.md` and maintain proof-CI discipline from the beginning of the spec-first repository state; later stages harden and expand that baseline.

---

## Phase 1 — Core Compiler & Toolchain MVP

**Goal:** deliver an end-to-end TypeScript/JavaScript → WebAssembly toolchain with checking, execution, build artifacts, sandboxing, package installation, workflow commands, and evidence hardening.

**Critical path:** `1.1 → 1.8`

**Parallelizable after 1.8:** `1.9 → 1.14`, with shared coordination on CLI definitions, diagnostics, schemas, and tests.

### Phase 1 stages

| Stage | Document | Milestone |
|---|---|---|
| 1.1 | [Workspace & Crate Scaffold](plan/phase-1/01-workspace-scaffold.md) | Workspace builds; CLI entrypoint exists; proof boundary discipline established |
| 1.2 | [Lexer](plan/phase-1/02-lexer.md) | Valid TS/JS tokenization; stable lex diagnostics |
| 1.3 | [Parser & AST](plan/phase-1/03-parser-and-ast.md) | AST produced for supported TS/JS grammar |
| 1.4 | [Name Resolution](plan/phase-1/04-name-resolution.md) | `kali check` resolves identifiers/imports and reports failures |
| 1.5 | [Type Checker](plan/phase-1/05-type-checker.md) | `kali check` reports type errors under the bounded inference contract |
| 1.6 | [HIR & LIR Lowering](plan/phase-1/06-hir-lir-lowering.md) | End-to-end compiler pipeline exists through internal IR |
| 1.7 | [WASM Code Generation](plan/phase-1/07-wasm-codegen.md) | Simple programs compile to runnable WASM |
| 1.8 | [Runtime & Execution](plan/phase-1/08-runtime-execution.md) | `kali run` and `kali test` work in the default standalone context |
| 1.9 | [Sandbox & Policy](plan/phase-1/09-sandbox-and-policy.md) | Runtime sandbox enforcement and static policy validation surface |
| 1.10 | [Package Management](plan/phase-1/10-package-management.md) | Deterministic install/lock flow for supported packages |
| 1.11 | [Build Artifacts](plan/phase-1/11-build-artifacts.md) | `kali build` emits executable, bundle, and base library artifacts |
| 1.12 | [Developer Workflow](plan/phase-1/12-developer-workflow.md) | `kali init`, `kali fmt`, `kali lint` work |
| 1.13 | [Diagnostics & Schemas](plan/phase-1/13-diagnostics-and-schemas.md) | Stable diagnostics and schema-v1 JSON output |
| 1.14 | [Evidence Hardening](plan/phase-1/14-evidence-hardening.md) | Conformance, determinism, package corpus, browser smoke, and proof-ready CI |

### Phase 1 coordination rules

After 1.8, parallel work is allowed only if all streams coordinate on:

- [`specs/12-cli.md`](./specs/12-cli.md)
- [`specs/15-errors.md`](./specs/15-errors.md)
- [`specs/18-schemas.md`](./specs/18-schemas.md)
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)
- `cargo test --workspace`

### Phase 1 completion gate

Phase 1 is complete when:

- Stages `1.1–1.14` are complete
- Evidence in `specs/16-testing.md` backs all opened Phase-1 maturity rows
- Core CLI surface passes end-to-end smoke tests
- Browser-targeted smoke tests pass for the Phase-1 browser-targeted command set
- Determinism checks pass for CLI outputs and generated artifacts
- `proofs/BOUNDARY.md` exists and matches the claimed proof state

---

## Phase 2 — Ownership, Effects & Public Embedding

**Goal:** add MIR-backed ownership semantics, public effect reporting, stable embedding outputs, and the Lean verification foundation.

**Dependency shape:** `2.1` is the main prerequisite; `2.2`, `2.3`, and most of `2.4` build on it.

### Phase 2 stages

| Stage | Document | Milestone |
|---|---|---|
| 2.1 | [MIR & Ownership Analysis](plan/phase-2/01-mir-and-ownership.md) | MIR becomes canonical mid-stage; ownership/escape analysis drives memory strategy |
| 2.2 | [Public Effect Reporting](plan/phase-2/02-public-effect-reporting.md) | `kali effects` and `kali package-effects` emit stable public reports |
| 2.3 | [Public Embedding Surface](plan/phase-2/03-public-embedding-surface.md) | Stable embedding API and artifact modes (`--lib`, `--capi`, `--component`) |
| 2.4 | [Lean Model Foundation](plan/phase-2/04-lean-model-foundation.md) | Lean workspace, proof CI, and core type-calculus model |

### Phase 2 completion gate

Phase 2 is complete when:

- Stages `2.1–2.4` are complete
- Public effect-report outputs are stable and schema-backed
- Embedding artifact modes are stable
- Lean proof jobs run in CI
- Phase-2 maturity rows are opened only with matching evidence

---

## Phase 3 — Specialization, Optimization & Ecosystem Breadth

**Goal:** improve performance and broaden compatibility without changing the core invariants.

**Ordering:** `3.1` first, then `3.2` and/or `3.3`.

### Phase 3 stages

| Stage | Document | Milestone |
|---|---|---|
| 3.1 | [Optimization & Specialization](plan/phase-3/01-optimization-and-specialization.md) | Stable monomorphization and faster `--release` / `--release-advanced` output |
| 3.2 | [Node Compatibility](plan/phase-3/02-node-compatibility.md) | `--api node` path and supported Node-oriented package execution |
| 3.3 | [Ecosystem Breadth](plan/phase-3/03-ecosystem-breadth.md) | Broader package corpus, incremental compilation, and deeper bundle/package support |

### Phase 3 completion gate

Phase 3 is complete when:

- Stages `3.1–3.3` are complete
- Release modes show measurable gains over `--fast`
- `--api node` is stable for its documented support surface
- Incremental compilation and ecosystem-breadth evidence are in place
- Phase-3 maturity rows are opened only with matching evidence

---

## Phase 4 — Advanced Compatibility & Deep Verification

**Goal:** add the hardest compatibility features and move from proof-ready/proof-foundational work to a proof-backed published boundary.

**Dependency shape:** `4.2` depends on the Lean foundation from `2.4`.

### Phase 4 stages

| Stage | Document | Milestone |
|---|---|---|
| 4.1 | [Dynamic Compatibility](plan/phase-4/01-dynamic-compatibility.md) | Gated support for dynamic compatibility features such as `eval` / `Function()` |
| 4.2 | [Formal Verification Depth](plan/phase-4/02-formal-verification-depth.md) | Non-empty published proof boundary with proof-backed release claims |

### Phase 4 completion gate

Phase 4 is complete when:

- Stages `4.1–4.2` are complete
- Dynamic compatibility features are available only within their documented gates
- `proofs/BOUNDARY.md` documents a non-empty published proof boundary
- Proof-backed claims are limited to the boundary actually proved
- Phase-4 maturity rows are opened only with matching evidence

---

## Cross-cutting rules

- **Hard invariants never bend.** AOT-only, pure Rust, no tracing/background GC, sandbox-first honesty, deterministic machine contracts.
- **Each stage must preserve workability.** No stage may regress existing working commands or tests.
- **Availability is controlled by the maturity matrix.** Implemented does not automatically mean public.
- **Proof-ready starts in Stage 1.1.** Proof-backed claims require a non-empty published boundary.
- **Parallelism is opt-in.** If a stage file does not say work may proceed in parallel, assume sequential ordering.
- **Shared surfaces must stay aligned.** CLI definitions, diagnostics, schemas, and verification claims must match their owning spec chapters.

---

## Practical reading rule

- Use this file to understand **implementation order**.
- Use stage documents to understand **what to build next**.
- Use `specs/19-feature-maturity.md` to understand **what is publicly available**.
- Use `proofs/BOUNDARY.md` to understand **what is actually proof-backed today**.
