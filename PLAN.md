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
├── README.md
├── 00-planning-conventions.md
├── 01-repository-layout.md
├── 02-workstreams-and-handoffs.md
├── 03-spec-to-stage-traceability.md
├── 04-stage-dependency-matrix.md
├── 05-delivery-increments.md
├── 06-current-workspace-rollout.md
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

Total: 27 stage documents, 5 phase index documents, and 7 cross-phase planning guides.

## Suggested implementation directory structure

A more detailed adoption guide for this layout lives in [plan/01-repository-layout.md](./plan/01-repository-layout.md).

The plan assumes a repository layout that keeps normative docs, implementation crates, evidence, and fixtures separate.

### Logical long-lived structure

```text
.
├── Cargo.toml
├── mise.toml
├── SPEC.md
├── PLAN.md
├── specs/
├── plan/
├── proofs/
├── crates/
│   ├── kali/                  # CLI binary / command dispatch
│   ├── cli/                   # arg parsing, help text, config discovery
│   ├── core/                  # lexer, parser, AST, typing, lowering, codegen
│   ├── runtime/               # wasm execution, host adapters, sandbox enforcement
│   ├── packages/              # install, lock, resolution, cache management
│   ├── sandbox/               # policy schema, validation, effect comparison
│   ├── embed/                 # stable embedding APIs, C ABI, component support
│   └── optimize/              # specialization, optimization, PGO plumbing
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
    └── packages/
```

### Current-workspace-aligned structure

The current repository already uses a finer-grained crate split. The plan should work with that shape instead of forcing an immediate crate merge:

```text
crates/
├── kali_cli                     # user-facing binary / top-level dispatch
├── kali_common                  # shared utilities, spans, IDs, interners, config helpers
├── kali_error                   # canonical diagnostics and error plumbing
├── kali_lexer                   # stage 1.2
├── kali_parser                  # stage 1.3
├── kali_ast                     # stage 1.3-1.4
├── kali_types                   # stages 1.4-1.5, then 2.x effect/type depth
├── kali_hir                     # stage 1.6
├── kali_mir                     # stage 2.1 canonicalization
├── kali_lir                     # stages 1.6-1.7
├── kali_codegen                 # stages 1.7, 1.11, 3.1, 5.5
├── kali_runtime                 # stages 1.8, 3.2, 3.4, 4.1, 5.x
├── kali_sandbox                 # stages 1.9, 2.2, 5.3
├── kali_npm                     # stages 1.10, 3.3, 4.1
├── kali_fmt                     # stage 1.12
├── kali_lint                    # stage 1.12
├── kali_embed                   # stages 1.11, 2.3, 5.5
├── kali_capi                    # stage 2.3+
├── kali_optimize                # stage 3.1+
├── kali_api_web                 # browser-targeted host surface
├── kali_api_deno                # default standalone/build host surface
└── kali_api_node                # phase-3 node host surface
```

This is the recommended practical directory strategy:
- keep the current fine-grained crates,
- treat them as implementations of the logical ownership buckets above,
- add top-level `tests/` and `fixtures/` lanes as the evidence matrix expands,
- keep `schemas/`, `types/`, `bindings/`, and `proofs/` as first-class companion trees rather than hiding those contracts inside code crates.

Notes:
- The logical ownership split matters more than whether the repo uses eight broad crates or many focused crates.
- Keep `specs/` and `plan/` as docs-only trees so release claims and implementation sequencing stay reviewable.
- Keep evidence-producing tests in dedicated top-level directories so each maturity row can point to a concrete lane.
- Prefer adding a new crate only when it represents a stable subsystem boundary that maps back to an owning spec chapter and a plan stage.

---

## How to use this document

Start here, then use the supporting plan docs for the specific planning question:
- [plan/README.md](./plan/README.md) — quick navigation across the whole plan set
- [plan/00-planning-conventions.md](./plan/00-planning-conventions.md) — stage-writing rules, workable-state discipline, and completion packets
- [plan/01-repository-layout.md](./plan/01-repository-layout.md) — target repository shape and when each area should appear
- [plan/02-workstreams-and-handoffs.md](./plan/02-workstreams-and-handoffs.md) — cross-phase workstreams, ownership boundaries, and parallel-stream handoffs
- [plan/03-spec-to-stage-traceability.md](./plan/03-spec-to-stage-traceability.md) — chapter-by-chapter mapping from the normative spec set to concrete plan stages and evidence lanes
- [plan/04-stage-dependency-matrix.md](./plan/04-stage-dependency-matrix.md) — stage-by-stage prerequisites, parallel windows, code areas, and demo expectations
- [plan/05-delivery-increments.md](./plan/05-delivery-increments.md) — higher-level increments that keep the repository usable while stages accumulate
- [plan/06-current-workspace-rollout.md](./plan/06-current-workspace-rollout.md) — concrete repository-growth order for the current fine-grained workspace

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
- Definition of done or status

Each phase index document should define:
- the phase objective and what changes in that phase,
- the stage ordering and safe parallelism inside the phase,
- the main spec chapters that phase is implementing,
- the phase exit gate and evidence expectations.

---

## Planning ownership

Planning material lives here, not in `SPEC.md`.

- `SPEC.md` defines cross-spec normalization, phase contracts, and release-claim rules.
- `PLAN.md` defines implementation order, dependency structure, phase sequencing, and completion gates.
- `plan/**/*.md` defines the detailed stage tasks and concrete definitions of done.

If a sentence is primarily about **what gets built when**, **what depends on what**, or **what a stage must prove before moving on**, it belongs in the plan set.

Compaction rule:
- `specs/16-testing.md` owns the normative evidence requirements only; concrete CI rollout, benchmark automation, and staged evidence expansion live in [phase-1/14](./plan/phase-1/14-evidence-hardening.md) and later stage files.
- `specs/17-verification.md` owns proof-boundary discipline only; concrete Lean milestones, first proof-backed scope growth, and deeper verification rollout live in [phase-2/04](./plan/phase-2/04-lean-model-foundation.md) and [phase-4/02](./plan/phase-4/02-formal-verification-depth.md).
- later-compatibility items documented in the spec but intentionally outside Phases 1-4 are tracked in [Phase 5](#phase-5--later-compatibility--platform-expansion) so the plan set covers the full spec surface without falsely promoting those items into earlier public promises.

---

## Recommended execution lanes in the current workspace

When turning the spec set into code in the current repository, use these lanes to keep the work incremental and workable:

| Lane | Stages | Main current crates |
|---|---|---|
| Frontend acceptance | 1.2-1.5 | `kali_lexer`, `kali_parser`, `kali_ast`, `kali_types`, `kali_error` |
| Lowering + codegen | 1.6-1.7 | `kali_hir`, `kali_lir`, `kali_codegen`, later `kali_mir` |
| Runtime + host baseline | 1.8-1.9 | `kali_runtime`, `kali_sandbox`, `kali_api_deno`, `kali_api_web` |
| Package + build surface | 1.10-1.11 | `kali_npm`, `kali_codegen`, `kali_embed`, host API crates |
| Workflow + machine contracts | 1.12-1.14 | `kali_cli`, `kali_fmt`, `kali_lint`, `kali_error`, `schemas/`, `proofs/` |
| Ownership/effects/embedding depth | 2.x | `kali_mir`, `kali_sandbox`, `kali_embed`, `kali_capi` |
| Optimization + compatibility breadth | 3.x-5.x | `kali_optimize`, `kali_api_node`, `kali_runtime`, `bindings/` |

This lane view is a practical complement to the phase map: it tells contributors where code should accumulate first without changing the normative phase contracts.

For a more concrete “what directories and crates should grow next in this exact repository?” view, use [plan/06-current-workspace-rollout.md](./plan/06-current-workspace-rollout.md). That guide translates the phase/stage roadmap into a repository-growth sequence over the current `kali_*` crates, `schemas/`, `types/`, `bindings/`, `scripts/`, `tests/`, and `fixtures/` trees.

## Implementation strata

The spec set is easiest to implement in five broad delivery strata:

| Stratum | Purpose | Primary chapters | Plan ownership |
|---|---|---|---|
| Bootstrap normalization + cross-spec rules | Turn `prompts/bootstrap.md` into phase-correct claims and shared vocabulary | `SPEC.md`, `specs/19-feature-maturity.md` | this document sets the delivery sequencing that respects those claims |
| Frontend + semantics | Parse TS/JS, build typed meaning, and enforce the bounded inference contract | `specs/01`-`specs/04` | Phase 1 stages 1.1-1.5 |
| Lowering + runtime core | Lower through IR, generate WASM, execute safely, and define host/runtime behavior | `specs/05`-`specs/11` | Phase 1 stages 1.6-1.11, then Phase 2 stage 2.1 |
| Product/tooling surface | CLI, packages, diagnostics, schemas, testing, embedding, verification | `specs/12`-`specs/18` | Phase 1 stages 1.9-1.14, Phase 2+, and later evidence/depth work |
| Later-compatibility expansion | Implement spec-deferred host/runtime/object-model breadth without overclaiming early support | later-compatibility rows in `specs/09`-`specs/14`, `specs/19` | Phase 5 stages 5.1-5.5 |

This stratum view is for implementation planning only. It does not change normative ownership or public availability.

## Repository-area mapping

To keep implementation work aligned with the specs, each stratum should predominantly touch these repository areas:

| Stratum | Primary code areas | Primary evidence areas |
|---|---|---|
| Bootstrap normalization + cross-spec rules | `SPEC.md`, `specs/`, `PLAN.md`, `plan/` | doc review, schema/claim consistency checks |
| Frontend + semantics | `crates/core`, parser/type fixtures | `tests/integration`, `tests/conformance` |
| Lowering + runtime core | `crates/core`, `crates/runtime`, `crates/sandbox` | runtime fixtures, wasm validation, sandbox tests |
| Product/tooling surface | `crates/cli`, `crates/packages`, `crates/embed` | CLI snapshots, package corpus, JSON schema checks |
| Later-compatibility expansion | all of the above plus host-specific adapters | targeted compatibility lanes, benchmarks, proof growth |

This mapping is intentionally coarse. Stage files remain the authoritative place for exact work items.

---

## Phase map

For quick navigation, start with the phase index that matches the stream you are working on:
- [Phase 1 index](./plan/phase-1/README.md)
- [Phase 2 index](./plan/phase-2/README.md)
- [Phase 3 index](./plan/phase-3/README.md)
- [Phase 4 index](./plan/phase-4/README.md)
- [Phase 5 index](./plan/phase-5/README.md)


| Phase | Focus | Workable outcome |
|---|---|---|
| 1 | Core compiler and toolchain MVP | Check, run, build, test, install, sandbox, and basic browser-targeted build path |
| 2 | Ownership, effects, embedding, verification foundation | MIR ownership model, public effect reporting, stable embedding artifacts, Lean foundation, and stable coverage reporting |
| 3 | Optimization and ecosystem breadth | Specialization, stronger release modes, Node compatibility, broader package support, and Phase-3 host-capability expansion |
| 4 | Dynamic compatibility and proof-backed claims | `eval`/`Function()`, non-literal dynamic loading, public `package-audit`, and a proof-backed published boundary |
| 5 | Later compatibility and platform expansion | Threaded runtime, standalone browser runtime, programmable policy/effect extensions, late host/object-model breadth, PGO, and language-binding expansion |

Phase-5 interpretation rule:
- Phase 5 is a **planning bucket for spec-deferred later-compatibility work**.
- It exists so the plan set tracks all currently documented spec surfaces.
- It does **not** by itself turn any “Later compatibility” maturity row into a public commitment; `specs/19-feature-maturity.md` still controls actual availability wording.

## Stage-completion packet

Every stage should ship with a small, repeatable completion packet so the repository stays reviewable and phase-correct:

1. **Implementation slice** — code/config/docs for the stage land together.
2. **Spec-coordination slice** — update the owning chapter plus CLI/error/schema/maturity docs when public behavior changed.
3. **Evidence slice** — add or extend the test lane that proves the claimed milestone.
4. **Operator proof** — record the command/demo that shows the stage is workable right now.
5. **Regression proof** — rerun `cargo test --workspace` and any stage-specific canonical tasks.

Stage files may add more requirements, but they should not skip any of these five packet elements.

## Workable-state ladder

The main sequencing principle is that each stage should unlock or preserve a concrete demonstration, not just internal code motion.

| Stage range | Minimum repository demonstration after the range closes |
|---|---|
| 1.1 | `kali --version` works and the workspace builds/tests |
| 1.2-1.3 | parsing/tokenization fixtures produce deterministic frontend output |
| 1.4-1.5 | `kali check` reports name/type diagnostics on local files |
| 1.6-1.7 | local programs compile to validated WASM artifacts |
| 1.8 | `kali run` and `kali test` execute in the default standalone context |
| 1.9-1.14 | sandboxed execution, install/build/workflow/JSON outputs, and evidence lanes all function together |
| 2.1-2.5 | MIR/ownership, effects, embedding, Lean foundation, and coverage reporting are stable enough for external consumers |
| 3.1-3.4 | release-mode gains, Node path, broader packages, and host-capability growth are all evidence-backed |
| 4.1-4.2 | late dynamic features and a proof-backed published boundary are available within documented gates |
| 5.1-5.5 | deferred runtime/platform breadth is opened one surface at a time without weakening early guarantees |

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

### Threaded runtime precedes weak-reference and worker-heavy compatibility

Later-compatibility work has one important dependency chain that is not obvious from the phase labels alone:

- `5.1` (threaded runtime profile) should land before any work that depends on real `SharedArrayBuffer` / `Atomics` semantics or thread-aware runtime budgeting.
- later host/runtime work that wants workers, thread-aware browser execution, or thread-aware object/runtime guarantees should build on that stage rather than inventing a second concurrency model.

### Standalone browser runtime follows browser-targeted build maturity

The Phase-1 browser story is intentionally **build/analysis first**. A standalone browser runtime/test contract is planned only after:

- browser-targeted bundle/build behavior is already stable,
- the browser host adapter is explicit and tested, and
- the plan can preserve the spec's browser ambient-typing vs mediated-capability split without pretending Kali embeds a browser engine early.

That is why standalone browser runtime work is deferred to Phase 5 instead of being folded into Phase 1 or 3.

## Cross-phase dependency matrix

The most important inter-phase dependencies are:

| Later stage | Must not start in earnest until | Why |
|---|---|---|
| 2.2 Public effect reporting | 2.1 MIR & ownership | stable effect facts need canonical mid-level semantics |
| 2.3 Public embedding surface | 2.1 MIR & ownership, 1.11 build artifacts | stable exports and ABI metadata depend on settled lowering/artifact shape |
| 2.4 Lean foundation depth work | 1.1 proof-ready baseline, 2.1 canonical semantics | proofs should target the semantics the compiler actually commits to |
| 2.5 Stable coverage reporting | 1.8 runtime execution | coverage without a working test runner is speculative |
| 3.2 Node compatibility | 3.1 optimization baseline, 1.8 runtime core | host widening should build on the runtime/compiler shape Kali intends to keep |
| 3.3 Ecosystem breadth | 3.2 and 3.4 foundations where applicable | package breadth depends on host/API fit, not package resolution alone |
| 4.1 Dynamic compatibility | Phases 1-3 runtime, package, and host groundwork | late dynamic features amplify earlier runtime decisions |
| 4.2 Proof-backed depth | 2.4 Lean foundation | proof-backed claims require an already-running proof program |
| 5.2 Standalone browser runtime | 1.11 browser-targeted build maturity, 5.1 if thread-aware browser paths are desired | build-first browser support avoids overclaiming an embedded browser runtime |
| 5.5 PGO & language bindings | 2.3 public embedding surface, 3.1 optimization baseline | feedback-guided optimization and generated bindings need stable surfaces |

---

## Phase 1 — Core Compiler & Toolchain MVP

Phase index: [plan/phase-1/README.md](./plan/phase-1/README.md)

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

Phase index: [plan/phase-2/README.md](./plan/phase-2/README.md)

**Goal:** add MIR-backed ownership semantics, public effect reporting, stable embedding outputs, the Lean verification foundation, and the stable function-coverage reporting contract required by `kali test --coverage`.

**Dependency shape:** `2.1` is the main prerequisite; `2.2`, `2.3`, most of `2.4`, and the machine-readable half of `2.5` build on it.

### Phase 2 stages

| Stage | Document | Milestone |
|---|---|---|
| 2.1 | [MIR & Ownership Analysis](plan/phase-2/01-mir-and-ownership.md) | MIR becomes canonical mid-stage; ownership/escape analysis drives memory strategy |
| 2.2 | [Public Effect Reporting](plan/phase-2/02-public-effect-reporting.md) | `kali effects` / `kali package-effects`, policy comparison, and explicit built-in effect annotations are stable |
| 2.3 | [Public Embedding Surface](plan/phase-2/03-public-embedding-surface.md) | Stable Rust embedding API and stable `--lib` / `--capi` / `--component` artifact modes |
| 2.4 | [Lean Model Foundation](plan/phase-2/04-lean-model-foundation.md) | Lean workspace, proof CI, and core type-calculus model |
| 2.5 | [Test Coverage & Reporting](plan/phase-2/05-test-coverage-and-reporting.md) | `kali test --coverage` gains a stable function-coverage contract and evidence-backed output |

### Phase 2 completion gate

Phase 2 is complete when:

- Stages `2.1–2.5` are complete
- Public effect-report outputs are stable and schema-backed
- Effect-annotation checking and inferred-effect-vs-policy validation are stable for their documented built-in capability subset
- Embedding artifact modes are stable, including host-ABI metadata/version checks where applicable
- Lean proof jobs run in CI
- `kali test --coverage` is stable for its documented function-coverage command contexts, with schema-backed output and deterministic reports
- Phase-2 maturity rows are opened only with matching evidence

---

## Phase 3 — Specialization, Optimization & Ecosystem Breadth

Phase index: [plan/phase-3/README.md](./plan/phase-3/README.md)

**Goal:** improve performance and broaden compatibility without changing the core invariants.

**Ordering:** `3.1` first, then `3.2` and `3.4` may proceed in parallel; `3.3` consumes the stronger package/browser/runtime breadth once those foundations are in place.

### Phase 3 stages

| Stage | Document | Milestone |
|---|---|---|
| 3.1 | [Optimization & Specialization](plan/phase-3/01-optimization-and-specialization.md) | Stable monomorphization and faster `--release` / `--release-advanced` output |
| 3.2 | [Node Compatibility](plan/phase-3/02-node-compatibility.md) | `--api node` path and the documented Phase-3 Node subset |
| 3.3 | [Ecosystem Breadth](plan/phase-3/03-ecosystem-breadth.md) | Broader package corpus, literal-string `import()` lowering, and deeper bundle/package support |
| 3.4 | [Host Capability Expansion](plan/phase-3/04-host-capability-expansion.md) | Mutable env, subprocess, socket/listener, and broader Deno-oriented host capabilities become evidence-backed |

### Phase 3 completion gate

Phase 3 is complete when:

- Stages `3.1–3.4` are complete
- Release modes show measurable gains over `--fast`
- `--api node` is stable for its documented support surface
- Mutable env, subprocess, socket/listener, and broader Deno-oriented host capability rows are backed by sandbox/resource-limit evidence
- Incremental compilation and ecosystem-breadth evidence are in place
- Phase-3 maturity rows are opened only with matching evidence

---

## Phase 4 — Dynamic Compatibility & Deep Verification

Phase index: [plan/phase-4/README.md](./plan/phase-4/README.md)

**Goal:** add the hardest phase-promised dynamic features and move from proof-ready/proof-foundational work to a proof-backed published boundary.

**Dependency shape:** `4.2` depends on the Lean foundation from `2.4`; `4.1` depends on the runtime, package, and optimization work from Phases 1-3.

### Phase 4 stages

| Stage | Document | Milestone |
|---|---|---|
| 4.1 | [Dynamic Compatibility](plan/phase-4/01-dynamic-compatibility.md) | `--compat eval`, `Function()`, non-literal dynamic loading, and public `package-audit` |
| 4.2 | [Formal Verification Depth](plan/phase-4/02-formal-verification-depth.md) | Non-empty published proof boundary with proof-backed release claims |

### Phase 4 completion gate

Phase 4 is complete when:

- Stages `4.1–4.2` are complete
- Dynamic compatibility features are available only within their documented gates
- `proofs/BOUNDARY.md` documents a non-empty published proof boundary
- Proof-backed claims are limited to the boundary actually proved
- Phase-4 maturity rows are opened only with matching evidence

---

## Phase 5 — Later Compatibility & Platform Expansion

Phase index: [plan/phase-5/README.md](./plan/phase-5/README.md)

**Goal:** track and implement the spec surfaces intentionally marked as later compatibility or future work, without retroactively widening earlier promises.

**Dependency shape:** this phase is mostly additive after Phase 4, but its own internal order matters:
- `5.1` establishes the threaded runtime profile.
- `5.2` and `5.4` build on the stronger runtime/host foundation.
- `5.3` builds on the public effect and embedding surfaces from Phase 2.
- `5.5` uses the stable runtime/embedding/tooling foundations for optimization feedback loops and language bindings.

### Phase 5 stages

| Stage | Document | Milestone |
|---|---|---|
| 5.1 | [Threaded Runtime Profile](plan/phase-5/01-threaded-runtime-profile.md) | `--wasm-threads`, `SharedArrayBuffer`, `Atomics`, and thread-budget enforcement work under the documented opt-in profile |
| 5.2 | [Standalone Browser Runtime & Host Expansion](plan/phase-5/02-standalone-browser-runtime-and-host-expansion.md) | Later `run/test --api browser` and broader host-backend work gain an explicit runtime contract |
| 5.3 | [Programmable Policy & Algebraic Effects](plan/phase-5/03-programmable-policy-and-algebraic-effects.md) | Host-registered narrowing predicates and any algebraic-effect surface are introduced without breaking the declarative sandbox contract |
| 5.4 | [Late Host & Object Compatibility](plan/phase-5/04-late-host-and-object-compatibility.md) | Later host APIs, weak/finalization/proxy semantics, and legacy/web-compat corners are implemented behind explicit gates |
| 5.5 | [PGO & Language Bindings](plan/phase-5/05-pgo-and-language-bindings.md) | Profile-guided optimization and post-Phase-2 language-binding expansion are evidence-backed |

### Phase 5 completion gate

Phase 5 is complete when:

- Stages `5.1–5.5` are complete
- Each later-compatibility surface has explicit evidence matching the exact maturity row that was opened
- New runtime/backend/embedding breadth does not weaken the core invariants or machine contracts
- The maturity matrix is updated feature-by-feature rather than with one blanket “Phase 5 support” claim

## Cross-phase planning guides

The stage files are the authoritative step-by-step plan, but six cross-phase guides keep the whole roadmap coherent:

- [plan/README.md](./plan/README.md) — entrypoint and navigation map for the plan set
- [plan/00-planning-conventions.md](./plan/00-planning-conventions.md) — shared stage vocabulary, readiness rules, and update packets
- [plan/01-repository-layout.md](./plan/01-repository-layout.md) — repository area ownership and when each area should be introduced
- [plan/02-workstreams-and-handoffs.md](./plan/02-workstreams-and-handoffs.md) — the frontend/runtime/tooling/evidence workstreams and their handoff contracts
- [plan/03-spec-to-stage-traceability.md](./plan/03-spec-to-stage-traceability.md) — a spec-coverage audit map so each normative chapter has an explicit implementation home
- [plan/04-stage-dependency-matrix.md](./plan/04-stage-dependency-matrix.md) — the exact prerequisite graph and demo/evidence expectations for each stage
- [plan/05-delivery-increments.md](./plan/05-delivery-increments.md) — the larger implementation slices that should be reviewable, workable, and demoable to users
- [plan/06-current-workspace-rollout.md](./plan/06-current-workspace-rollout.md) — the concrete directory/crate growth order for this repository

## Current post-completion follow-up lanes

The repo still tracks a few closed-stage follow-up lanes explicitly so the plan set and the proof-summary docs stay in sync:

- **Stage 3.1 specialization depth follow-up** — see [`plan/phase-3/01-optimization-and-specialization.md`](./plan/phase-3/01-optimization-and-specialization.md) → `Remaining Work`
- **Stage 3.3 ecosystem/package/browser breadth follow-up** — see [`plan/phase-3/03-ecosystem-breadth.md`](./plan/phase-3/03-ecosystem-breadth.md) → `Remaining Work`
- **Stage 4.2 verification-depth follow-up** — see [`plan/phase-4/02-formal-verification-depth.md`](./plan/phase-4/02-formal-verification-depth.md) → `Remaining Work`

Canonical proof-summary pin for those follow-up lanes:

> **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**

The proof-summary theorem inventory itself remains authored by [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) and mirrored in the summary docs/tests that pin the current published boundary.

---

## Cross-cutting rules

- **Hard invariants never bend.** AOT-only, pure Rust, no tracing/background GC, sandbox-first honesty, deterministic machine contracts.
- **Each stage must preserve workability.** No stage may regress existing working commands or tests.
- **Availability is controlled by the maturity matrix.** Implemented does not automatically mean public.
- **Proof-ready starts in Stage 1.1.** Proof-backed claims require a non-empty published boundary.
- **Parallelism is opt-in.** If a stage file does not say work may proceed in parallel, assume sequential ordering.
- **Shared surfaces must stay aligned.** CLI definitions, diagnostics, schemas, and verification claims must match their owning spec chapters.
- **Later-compatibility planning is explicit.** If the spec defines a later or future surface, it should either have a stage in this plan or be explicitly called out as deferred-by-design rather than silently omitted.

---

## Practical reading rule

- Use this file to understand **implementation order**.
- Use stage documents to understand **what to build next**.
- Use `specs/19-feature-maturity.md` to understand **what is publicly available**.
- Use `proofs/BOUNDARY.md` to understand **what is actually proof-backed today**.
