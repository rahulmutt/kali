# Kali — Implementation Plan

`PLAN.md` is the implementation playbook for [`SPEC.md`](./SPEC.md). It translates the normalized spec set into an implementation order that keeps the repository usable after every stage.

## Plan contract

After every stage the repository must remain in a workable state:

1. `cargo build` succeeds
2. `cargo test --workspace` passes
3. At least one user-visible capability is demonstrable
4. Hard invariants still hold: AOT-only, pure Rust, no tracing/background GC, sandbox-first honesty, deterministic machine contracts

Normative ownership remains unchanged:
- [`SPEC.md`](./SPEC.md) defines cross-spec rules and phase contracts
- the owning chapter in [`specs/`](./specs) defines subsystem behavior
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) defines public availability
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) defines the current proof-backed boundary
- this plan defines sequencing, dependencies, milestones, and completion gates only

---

## Planning goals

This plan is optimized for four things:

1. **Simple before broad** — establish local-file compilation and execution before package and host breadth
2. **Workable milestones** — every packet should leave the repo demoable, not just internally refactored
3. **Explicit ownership** — each stage maps back to named spec chapters and evidence lanes
4. **Safe parallelism** — parallel work starts only after the core compiler/runtime loop is stable

---

## Stage layout

```text
plan/
├── README.md
├── 00-planning-conventions.md
├── 01-repository-layout.md
├── 02-workstreams-and-handoffs.md
├── 03-spec-to-stage-traceability.md
├── 04-stage-dependency-matrix.md
├── 05-delivery-increments.md
├── 06-current-workspace-rollout.md
├── 07-roadmap-status-and-next-steps.md
├── 08-fresh-implementation-roadmap.md
├── 09-stage-acceptance-checklists.md
├── 10-risk-register.md
├── phase-1/
│   ├── README.md
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
│   ├── README.md
│   ├── 01-mir-and-ownership.md
│   ├── 02-public-effect-reporting.md
│   ├── 03-public-embedding-surface.md
│   ├── 04-lean-model-foundation.md
│   └── 05-test-coverage-and-reporting.md
├── phase-3/
│   ├── README.md
│   ├── 01-optimization-and-specialization.md
│   ├── 02-node-compatibility.md
│   ├── 03-ecosystem-breadth.md
│   └── 04-host-capability-expansion.md
├── phase-4/
│   ├── README.md
│   ├── 01-dynamic-compatibility.md
│   └── 02-formal-verification-depth.md
└── phase-5/
    ├── README.md
    ├── 01-threaded-runtime-profile.md
    ├── 02-standalone-browser-runtime-and-host-expansion.md
    ├── 03-programmable-policy-and-algebraic-effects.md
    ├── 04-late-host-and-object-compatibility.md
    └── 05-pgo-and-language-bindings.md
```

Total planning surface: 30 stage documents, 5 phase indexes, and 11 cross-phase planning guides.

---

## Suggested implementation directory structure

The repository already has a sensible fine-grained workspace. The plan keeps that structure and grows it deliberately instead of forcing an early reorganization.

### Long-lived logical ownership model

```text
.
├── Cargo.toml
├── mise.toml
├── SPEC.md
├── PLAN.md
├── specs/
├── plan/
├── proofs/
├── schemas/
├── types/
├── bindings/
├── crates/
│   ├── kali_cli
│   ├── kali_common
│   ├── kali_error
│   ├── kali_lexer
│   ├── kali_parser
│   ├── kali_ast
│   ├── kali_types
│   ├── kali_hir
│   ├── kali_mir
│   ├── kali_lir
│   ├── kali_codegen
│   ├── kali_runtime
│   ├── kali_sandbox
│   ├── kali_npm
│   ├── kali_fmt
│   ├── kali_lint
│   ├── kali_embed
│   ├── kali_capi
│   ├── kali_optimize
│   ├── kali_api_web
│   ├── kali_api_deno
│   └── kali_api_node
├── tests/
│   ├── integration/
│   ├── conformance/
│   ├── package-corpus/
│   ├── browser-smoke/
│   └── determinism/
└── fixtures/
    ├── cli/
    ├── compiler/
    ├── runtime/
    ├── browser/
    └── packages/
```

### Directory-structure rules

- Keep `specs/` and `plan/` as documentation-only trees
- Keep `proofs/`, `schemas/`, `types/`, and `bindings/` as first-class contract trees
- Prefer adding evidence directories before adding new crates
- Add a new crate only when it maps to a durable subsystem boundary owned by a spec chapter
- Reuse the current `kali_*` crate split unless it actively blocks stage ownership or testing

See [plan/01-repository-layout.md](./plan/01-repository-layout.md) for the detailed ownership guide and [plan/06-current-workspace-rollout.md](./plan/06-current-workspace-rollout.md) for the concrete growth order.

---

## How to use this plan

Start here, then read the supporting guide that matches the planning question:

- [plan/README.md](./plan/README.md) — navigation across the plan set
- [plan/00-planning-conventions.md](./plan/00-planning-conventions.md) — stage-writing rules and completion packets
- [plan/01-repository-layout.md](./plan/01-repository-layout.md) — long-lived ownership and directory strategy
- [plan/02-workstreams-and-handoffs.md](./plan/02-workstreams-and-handoffs.md) — safe parallel streams and handoffs
- [plan/03-spec-to-stage-traceability.md](./plan/03-spec-to-stage-traceability.md) — spec-to-stage mapping
- [plan/04-stage-dependency-matrix.md](./plan/04-stage-dependency-matrix.md) — compact stage prerequisites and demos
- [plan/05-delivery-increments.md](./plan/05-delivery-increments.md) — milestone-sized implementation slices
- [plan/06-current-workspace-rollout.md](./plan/06-current-workspace-rollout.md) — how the current workspace should grow
- [plan/07-roadmap-status-and-next-steps.md](./plan/07-roadmap-status-and-next-steps.md) — near-term execution priorities
- [plan/08-fresh-implementation-roadmap.md](./plan/08-fresh-implementation-roadmap.md) — the shortest fresh-start route through the stage graph
- [plan/09-stage-acceptance-checklists.md](./plan/09-stage-acceptance-checklists.md) — concrete acceptance criteria for closing each stage family
- [plan/10-risk-register.md](./plan/10-risk-register.md) — cross-spec implementation risks and required mitigations
- the relevant phase README under `plan/phase-*/README.md`
- the exact stage file you are implementing

Use this document to answer **what should be built when**. Use `SPEC.md` and the owning chapter to answer **what the system promises**.

---

## Implementation strata

The spec set is easiest to realize in five broad strata:

| Stratum | Purpose | Primary chapters | Main plan ownership |
|---|---|---|---|
| Contract baseline | freeze vocabulary, maturity boundaries, schemas, and verification discipline | `SPEC.md`, `specs/17`, `specs/18`, `specs/19` | pre-1.1 planning packet |
| Frontend + semantics | parse TS/JS, resolve names, type-check, and establish deterministic diagnostics | `specs/01`-`specs/04`, `specs/15` | Phase 1.1-1.5 |
| Lowering + runtime core | lower typed programs, emit deterministic WASM, and execute safely | `specs/05`-`specs/11` | Phase 1.6-1.11, then Phase 2.1 |
| Product/tooling surface | complete CLI, packages, schemas, evidence, embedding, and verification plumbing | `specs/12`-`specs/18` | Phase 1.9-1.14, Phase 2 |
| Compatibility breadth | optimize and widen support one surface at a time | `specs/07`, `specs/09`-`specs/14`, `specs/19` | Phases 3-5 |

This view is for implementation order only. It does not change public maturity.

---

## Recommended execution lanes in the current workspace

| Lane | Stages | Main crates |
|---|---|---|
| CLI and shared plumbing | 1.1, 1.12, 1.13 | `kali_cli`, `kali_common`, `kali_error`, `kali_fmt`, `kali_lint` |
| Frontend acceptance | 1.2-1.5 | `kali_lexer`, `kali_parser`, `kali_ast`, `kali_types` |
| Lowering + codegen | 1.6-1.7, 2.1 | `kali_hir`, `kali_mir`, `kali_lir`, `kali_codegen` |
| Runtime + sandbox | 1.8-1.9, 2.2, 3.4, 4.1, 5.x runtime work | `kali_runtime`, `kali_sandbox`, `kali_api_deno`, `kali_api_web`, `kali_api_node` |
| Packages + build artifacts | 1.10-1.11, 3.3 | `kali_npm`, `kali_embed`, `kali_capi`, `kali_codegen` |
| Optimization + later breadth | 3.1-5.5 | `kali_optimize`, `kali_runtime`, `kali_api_node`, `bindings/` |
| Evidence + proofs | 1.14, 2.4, 2.5, 4.2 | `tests/`, `fixtures/`, `proofs/`, `schemas/` |

---

## Phase map

For quick navigation:
- [Phase 1 index](./plan/phase-1/README.md)
- [Phase 2 index](./plan/phase-2/README.md)
- [Phase 3 index](./plan/phase-3/README.md)
- [Phase 4 index](./plan/phase-4/README.md)
- [Phase 5 index](./plan/phase-5/README.md)

| Phase | Focus | Workable outcome |
|---|---|---|
| 1 | Core compiler and toolchain MVP | deterministic `check`, `run`, `test`, `build`, `install`, sandboxing, browser-targeted build/check, and schema-v1 outputs |
| 2 | Ownership, effects, embedding, verification foundation | canonical MIR, public effect reporting, stable embedding surfaces, Lean foundation, and stable coverage reporting |
| 3 | Optimization and ecosystem breadth | stronger release modes, Node path, broader packages, and widened host capabilities |
| 4 | Dynamic compatibility and proof-backed depth | gated dynamic features, `package-audit`, and proof-backed published-boundary claims |
| 5 | Deferred platform/runtime expansion | threads, standalone browser runtime, programmable policy extensions, late object-model breadth, PGO, and language bindings |

### Recommended staffing split after the critical path

After stage `1.8`, use the parallel window deliberately instead of opening every stream with the same weight.
A practical split is:

| Stream | Main stages | Shared owners that must stay synchronized |
|---|---|---|
| Runtime policy stream | `1.9`, later `2.2`, `3.4`, `5.3` | `specs/09`, `specs/12`, `specs/18`, `specs/19` |
| Package and artifact stream | `1.10`, `1.11`, later `2.3`, `3.3` | `specs/08`, `specs/11`, `specs/13`, `specs/14`, `specs/18`, `specs/19` |
| Workflow and machine-contract stream | `1.12`, `1.13`, later `2.5` | `specs/12`, `specs/15`, `specs/18`, `specs/19` |
| Evidence and verification stream | `1.14`, later `2.4`, `4.2` | `specs/16`, `specs/17`, `proofs/BOUNDARY.md`, `specs/19` |

The critical-path frontend/lowering/runtime owners should keep final review on any change that affects stage workability for the entire repo.

---

## Recommended implementation batches

For active execution, group the roadmap into these repository-safe batches:

| Batch | Stages | Why it is grouped this way | Must be true before opening the next batch |
|---|---|---|---|
| B0 — Contract lock | planning baseline | freezes vocabulary, availability reading rules, schemas, and proof-boundary discipline | `SPEC.md`, `PLAN.md`, `specs/`, `plan/`, and `proofs/BOUNDARY.md` are internally aligned |
| B1 — Frontend spine | 1.1-1.5 | creates the first deterministic `check` path and the minimum semantic backbone | `kali check` works on local TS/JS files with stable diagnostics |
| B2 — End-to-end local execution | 1.6-1.8 | closes the local source → IR → WASM → runtime loop before widening product surface | `kali build`, `kali run`, and `kali test` work on local fixtures |
| B3 — Phase-1 product parallel zone | 1.9-1.13 | opens sandbox, install, artifact, workflow, and JSON-contract work after runtime exists | each stream coordinates on CLI/error/schema/maturity owners and preserves the B2 demos |
| B4 — Phase-1 evidence closure | 1.14 | turns the MVP into a supportable release packet rather than a demo-only compiler | browser, package, determinism, and proof-ready evidence lanes pass |
| B5 — Semantic stabilization | 2.1-2.5 | settles ownership, effects, embedding, proofs, and coverage on canonical semantics | MIR is canonical and Phase-2 public surfaces are coherent |
| B6 — Breadth expansion | 3.1-5.5 | widens optimization and compatibility one support rung at a time | every widened surface has explicit evidence and an updated maturity row when needed |

Use [plan/09-stage-acceptance-checklists.md](./plan/09-stage-acceptance-checklists.md) before closing a batch and [plan/10-risk-register.md](./plan/10-risk-register.md) when deciding where extra hardening is required.

### Promotion checkpoints between batches

Do not advance between major batches until the earlier batch has cleared its promotion check:

| From | To | Promotion check |
|---|---|---|
| B1 | B2 | `kali check` is deterministic on explicit TS/JS fixtures and diagnostic snapshots are stable |
| B2 | B3 | `kali build`, `kali run`, and `kali test` all work on local fixtures in the default standalone context |
| B3 | B4 | sandbox, install, build-artifact, workflow, and JSON-output workstreams no longer fight over CLI/schema owners |
| B4 | B5 | Phase-1 evidence lanes pass and the repo is supportable rather than only demoable |
| B5 | B6 | MIR/ownership, effect reporting, embedding, proof foundation, and coverage are all coherent enough to widen breadth safely |

---

## Stage-completion packet

Every stage should land with the same minimum packet:

1. **Implementation slice** — code/config/docs for the stage land together
2. **Spec-coordination slice** — update owning specs when public behavior changed
3. **Evidence slice** — add or extend the proving test lane
4. **Operator proof** — record the command/demo that shows the stage is workable
5. **Regression proof** — rerun `cargo test --workspace` and stage-specific canonical tasks

Stage files may add more, but they should not drop any of these five parts.

---

## Workable-state ladder

| Stage range | Minimum demonstration after the range closes |
|---|---|
| 1.1 | `kali --version` works; workspace builds/tests |
| 1.2-1.3 | deterministic tokenization and parsing fixtures |
| 1.4-1.5 | `kali check` reports name/type diagnostics on local files |
| 1.6-1.7 | local programs compile to validated WASM artifacts |
| 1.8 | `kali run` and `kali test` execute in the default standalone context |
| 1.9-1.14 | sandboxing, install/build/workflow commands, JSON output, and evidence all work together |
| 2.1-2.5 | MIR/ownership, effects, embedding, Lean, and coverage are externally coherent |
| 3.1-3.4 | optimization, Node, host breadth, and package breadth are evidence-backed |
| 4.1-4.2 | dynamic compatibility and proof-backed claims are explicitly bounded |
| 5.1-5.5 | deferred breadth opens one surface at a time without weakening earlier guarantees |

---

## Important ordering decisions

### 1. Phase contracts vs implementation order

Kali intentionally separates:
- **phase contracts** — earliest user-visible support promise
- **implementation order** — recommended engineering sequence

A feature may be documented before it is implemented, and implemented before it becomes publicly available. This plan does not override [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

### 2. Phase 1 should be built in six packets

1. **CLI/workspace spine** — shared entrypoint, diagnostics, config loading, proof-ready hygiene
2. **Frontend acceptance** — lexer, parser, AST, name resolution, type checking
3. **End-to-end local pipeline** — HIR/LIR lowering, codegen, runtime execution
4. **Phase-1 product surface** — sandboxing, packages, build artifacts, workflow commands, schemas
5. **Machine contracts** — stable diagnostics and JSON envelopes
6. **Evidence closure** — browser smoke, package corpus, determinism, proof-ready CI discipline

### 3. Execution before package breadth

Within Phase 1, end-to-end local execution should land before broad package work. That makes the compiler/runtime loop testable before install/registry complexity is introduced.

### 4. Proof-readiness starts at stage 1.1

Proof-ready discipline is an early repository requirement, not a final cleanup step. `proofs/BOUNDARY.md` and proof-trigger rules should exist from the start.

### 5. Browser support is build/analysis-first

Phase 1 browser work means the browser-targeted command set for `check` and `build --bundle`, not a standalone browser runtime contract. Standalone browser runtime work remains later.

### 6. Threading comes before thread-dependent breadth

Any real `SharedArrayBuffer`, `Atomics`, worker-style, or thread-budget semantics should build on the explicit threaded runtime profile from Phase 5.1 rather than inventing ad hoc concurrency paths earlier.

---

## Cross-phase dependency matrix

| Later stage | Must not begin in earnest until | Why |
|---|---|---|
| 2.2 Public effect reporting | 2.1 MIR & ownership | stable effect facts need canonical mid-level semantics |
| 2.3 Public embedding surface | 2.1 and 1.11 | stable exports depend on settled lowering and artifact shape |
| 2.4 Lean model foundation | 1.1 proof-ready baseline and 2.1 semantics | proofs should target committed semantics |
| 2.5 Stable coverage reporting | 1.8 runtime execution | coverage without a working runner is speculative |
| 3.2 Node compatibility | 3.1 optimization baseline and 1.8 runtime core | host widening should build on a stable compiler/runtime path |
| 3.3 Ecosystem breadth | 3.2 and 3.4 where applicable | package breadth depends on host/API fit, not only resolution |
| 4.1 Dynamic compatibility | Phases 1-3 runtime/package groundwork | late dynamic features amplify earlier design choices |
| 4.2 Proof-backed depth | 2.4 Lean foundation | proof-backed claims require an operating proof program |
| 5.2 Standalone browser runtime | 1.11 browser-targeted build maturity and 5.1 when thread-aware | browser runtime support should follow explicit build/runtime contracts |
| 5.5 PGO and language bindings | 2.3 embedding surface and 3.1 optimization baseline | feedback-guided optimization and bindings need stable public surfaces |

---

## Phase 1 — Core Compiler & Toolchain MVP

Phase index: [plan/phase-1/README.md](./plan/phase-1/README.md)

**Goal:** deliver the first dependable TS/JS → WASM toolchain with deterministic checking, execution, building, package installation, sandboxing, workflow commands, and evidence.

**Critical path:** `1.1 → 1.8`

**Parallel window:** `1.9 → 1.14` after `1.8` closes

| Stage | Document | Milestone |
|---|---|---|
| 1.1 | [Workspace & Crate Scaffold](plan/phase-1/01-workspace-scaffold.md) | buildable workspace, CLI spine, proof-ready baseline |
| 1.2 | [Lexer](plan/phase-1/02-lexer.md) | deterministic TS/JS tokenization |
| 1.3 | [Parser & AST](plan/phase-1/03-parser-and-ast.md) | AST for supported grammar |
| 1.4 | [Name Resolution](plan/phase-1/04-name-resolution.md) | imports and identifiers resolve with stable diagnostics |
| 1.5 | [Type Checker](plan/phase-1/05-type-checker.md) | `kali check` enforces bounded inference and TS/JS checking |
| 1.6 | [HIR & LIR Lowering](plan/phase-1/06-hir-lir-lowering.md) | typed lowering pipeline exists |
| 1.7 | [WASM Code Generation](plan/phase-1/07-wasm-codegen.md) | simple programs compile to validated WASM |
| 1.8 | [Runtime & Execution](plan/phase-1/08-runtime-execution.md) | `kali run` and `kali test` work in the default standalone context |
| 1.9 | [Sandbox & Policy](plan/phase-1/09-sandbox-and-policy.md) | runtime enforcement and static policy validation |
| 1.10 | [Package Management](plan/phase-1/10-package-management.md) | deterministic install/lock/materialization for supported packages |
| 1.11 | [Build Artifacts](plan/phase-1/11-build-artifacts.md) | executable, browser bundle, and base library artifact outputs |
| 1.12 | [Developer Workflow](plan/phase-1/12-developer-workflow.md) | `init`, `fmt`, and `lint` work |
| 1.13 | [Diagnostics & Schemas](plan/phase-1/13-diagnostics-and-schemas.md) | stable diagnostics and schema-v1 JSON contracts |
| 1.14 | [Evidence Hardening](plan/phase-1/14-evidence-hardening.md) | conformance, browser smoke, determinism, package corpus, proof-ready CI |

**Phase-1 completion gate:** stages `1.1–1.14` complete, canonical browser-targeted smoke coverage exists, determinism checks pass, and proof-ready discipline matches the published boundary.

---

## Phase 2 — Ownership, Effects & Public Embedding

Phase index: [plan/phase-2/README.md](./plan/phase-2/README.md)

**Goal:** stabilize post-MVP semantics and first public non-MVP contracts.

**Dependency shape:** `2.1` is the hinge; `2.2–2.5` build on it.

| Stage | Document | Milestone |
|---|---|---|
| 2.1 | [MIR & Ownership Analysis](plan/phase-2/01-mir-and-ownership.md) | MIR is canonical and ownership analysis drives memory strategy |
| 2.2 | [Public Effect Reporting](plan/phase-2/02-public-effect-reporting.md) | `effects` / `package-effects` and policy comparison are stable |
| 2.3 | [Public Embedding Surface](plan/phase-2/03-public-embedding-surface.md) | stable embedding API and `--lib` / `--capi` / `--component` flows |
| 2.4 | [Lean Model Foundation](plan/phase-2/04-lean-model-foundation.md) | proof workspace, CI, and semantic core exist |
| 2.5 | [Test Coverage & Reporting](plan/phase-2/05-test-coverage-and-reporting.md) | `kali test --coverage` is deterministic and schema-backed |

**Phase-2 completion gate:** MIR is canonical, public effects and embedding are stable, Lean jobs run in CI, and coverage reporting is deterministic for supported contexts.

---

## Phase 3 — Specialization, Optimization & Ecosystem Breadth

Phase index: [plan/phase-3/README.md](./plan/phase-3/README.md)

**Goal:** improve performance and widen compatibility without weakening the core invariants.

**Ordering:** `3.1` first; `3.2` and `3.4` can then proceed in parallel; `3.3` consumes those stronger foundations.

| Stage | Document | Milestone |
|---|---|---|
| 3.1 | [Optimization & Specialization](plan/phase-3/01-optimization-and-specialization.md) | release modes show measurable gains |
| 3.2 | [Node Compatibility](plan/phase-3/02-node-compatibility.md) | documented `--api node` path works end to end |
| 3.3 | [Ecosystem Breadth](plan/phase-3/03-ecosystem-breadth.md) | broader package support and dynamic import breadth are evidence-backed |
| 3.4 | [Host Capability Expansion](plan/phase-3/04-host-capability-expansion.md) | mutable env, subprocess, socket/listener capabilities are implemented honestly |

**Phase-3 completion gate:** release modes are evidence-backed, Node support is explicit and tested, widened host capabilities have sandbox/resource-limit coverage, and package breadth claims name exact support rungs.

---

## Phase 4 — Dynamic Compatibility & Deep Verification

Phase index: [plan/phase-4/README.md](./plan/phase-4/README.md)

**Goal:** add the hardest late compatibility paths and move from proof-ready/foundational verification to proof-backed published-boundary claims.

| Stage | Document | Milestone |
|---|---|---|
| 4.1 | [Dynamic Compatibility](plan/phase-4/01-dynamic-compatibility.md) | gated `eval`, `Function()`, harder dynamic loading, and `package-audit` |
| 4.2 | [Formal Verification Depth](plan/phase-4/02-formal-verification-depth.md) | non-empty published proof boundary with proof-backed claims |

**Phase-4 completion gate:** dynamic compatibility is only available through explicit gates, `package-audit` has a stable contract, and proof-backed wording is limited to the published boundary.

---

## Phase 5 — Deferred Platform Expansion

Phase index: [plan/phase-5/README.md](./plan/phase-5/README.md)

**Goal:** track and implement intentionally deferred runtime/platform breadth without back-solving it into earlier phases.

| Stage | Document | Milestone |
|---|---|---|
| 5.1 | [Threaded Runtime Profile](plan/phase-5/01-threaded-runtime-profile.md) | opt-in threaded runtime profile with thread budgets |
| 5.2 | [Standalone Browser Runtime & Host Expansion](plan/phase-5/02-standalone-browser-runtime-and-host-expansion.md) | later `run/test --api browser` contract |
| 5.3 | [Programmable Policy & Algebraic Effects](plan/phase-5/03-programmable-policy-and-algebraic-effects.md) | host-registered predicates and later effect extensions |
| 5.4 | [Late Host & Object Compatibility](plan/phase-5/04-late-host-and-object-compatibility.md) | weak refs, proxies, legacy corners, and late host APIs |
| 5.5 | [PGO & Language Bindings](plan/phase-5/05-pgo-and-language-bindings.md) | additive PGO and broader language bindings |

**Phase-5 completion gate:** each widened surface has its own evidence trail and maturity update; no blanket “Phase 5 support” claim is allowed.

---

## Cross-phase planning guides

The cross-phase guides under `plan/` are part of the active planning surface, not historical appendices:

- [plan/README.md](./plan/README.md) — navigation map
- [plan/00-planning-conventions.md](./plan/00-planning-conventions.md) — shared rules and completion packets
- [plan/01-repository-layout.md](./plan/01-repository-layout.md) — directory ownership
- [plan/02-workstreams-and-handoffs.md](./plan/02-workstreams-and-handoffs.md) — stream boundaries and handoffs
- [plan/03-spec-to-stage-traceability.md](./plan/03-spec-to-stage-traceability.md) — spec coverage audit
- [plan/04-stage-dependency-matrix.md](./plan/04-stage-dependency-matrix.md) — prerequisite graph and demos
- [plan/05-delivery-increments.md](./plan/05-delivery-increments.md) — workable milestone packets
- [plan/06-current-workspace-rollout.md](./plan/06-current-workspace-rollout.md) — concrete crate/directory growth order
- [plan/07-roadmap-status-and-next-steps.md](./plan/07-roadmap-status-and-next-steps.md) — recommended next execution lanes
- [plan/08-fresh-implementation-roadmap.md](./plan/08-fresh-implementation-roadmap.md) — compact fresh-start execution order
- [plan/09-stage-acceptance-checklists.md](./plan/09-stage-acceptance-checklists.md) — close-out checklist by stage family
- [plan/10-risk-register.md](./plan/10-risk-register.md) — cross-cutting risk and mitigation register

---

## Practical reading rule

- Use this file to understand **implementation order**
- Use phase README files to understand **what changes in a phase**
- Use stage files to understand **what to implement next**
- Use [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) to understand **what is publicly available**
- Use [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) to understand **what is actually proof-backed today**
