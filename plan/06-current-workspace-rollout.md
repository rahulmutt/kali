# Current Workspace Rollout

This guide turns the phase/stage roadmap into a concrete repository-growth order for the workspace that already exists today.

Use it when the planning question is not only **what stage comes next?** but also:
- which current crates should absorb that work,
- which top-level directories should be created or expanded next,
- and what minimal repo shape should exist before a later stage starts.

It complements rather than replaces:
- [`../PLAN.md`](../PLAN.md) for global ordering,
- [`01-repository-layout.md`](./01-repository-layout.md) for the long-lived ownership model,
- [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md) for prerequisites and demos,
- and the phase/stage files for detailed definitions of done.

## Core rule

Prefer **growing the current fine-grained workspace** over reorganizing it early.

That means:
- keep the existing `kali_*` crate split,
- add top-level evidence directories as the plan reaches the stages that need them,
- keep `schemas/`, `types/`, `bindings/`, and `proofs/` as first-class contract trees,
- and only do structural consolidation when the current boundaries actively block stage ownership.

## Recommended repository-growth order

| Packet | Stages | Primary directories/crates | Why this order | Minimum demo after the packet |
|---|---|---|---|---|
| R0 — Spec and contract baseline | pre-1.1 | `SPEC.md`, `PLAN.md`, `specs/`, `plan/`, `proofs/`, `schemas/`, `types/` | lock the normalized product contract and machine-readable boundaries before implementation claims begin | specs and plan are internally consistent; proof boundary discipline is published |
| R1 — Workspace boot | 1.1 | `crates/kali_cli`, `crates/kali_common`, `crates/kali_error`, workspace config, `scripts/` | establish the buildable entrypoint, shared plumbing, and canonical developer tasks first | `cargo build`, `cargo test --workspace`, `kali --version` |
| R2 — Frontend acceptance | 1.2-1.5 | `crates/kali_lexer`, `crates/kali_parser`, `crates/kali_ast`, `crates/kali_types`, `fixtures/compiler`, `tests/integration`, `tests/conformance` | parser/checker semantics should settle before lowering, runtime, or package breadth | deterministic token/AST/checker output and `kali check` on local inputs |
| R3 — Lowering and local execution | 1.6-1.8 | `crates/kali_hir`, `crates/kali_lir`, `crates/kali_codegen`, `crates/kali_runtime`, `crates/kali_api_deno`, `fixtures/runtime` | get to one end-to-end local-file compiler/runtime before widening product surface area | validated WASM plus `kali run` / `kali test` |
| R4 — Phase-1 product surface | 1.9-1.13 | `crates/kali_sandbox`, `crates/kali_npm`, `crates/kali_embed`, `crates/kali_fmt`, `crates/kali_lint`, `schemas/`, `types/`, `bindings/`, `tests/browser-smoke`, `tests/determinism`, `fixtures/packages`, `fixtures/browser`, `fixtures/cli` | after runtime exists, product-facing work can grow in parallel around stable CLI/schema/error contracts | `kali run --sandbox`, `kali install`, `kali build --bundle`, `kali init`, JSON output modes |
| R5 — Phase-1 evidence closure | 1.14 | `tests/package-corpus`, `tests/browser-smoke`, `tests/determinism`, `proofs/`, CI/workflow files | Phase 1 is only complete when shipped claims are evidence-backed | the canonical Phase-1 evidence tasks pass |
| R6 — Semantic depth and public surfaces | 2.1-2.5 | `crates/kali_mir`, `crates/kali_sandbox`, `crates/kali_embed`, `crates/kali_capi`, `proofs/`, coverage fixtures/tests | MIR/ownership should become canonical before public effects, embedding stability, or deeper proof work | `kali effects`, stable embedding outputs, `mise run lean-proofs`, `kali test --coverage` |
| R7 — Optimization and compatibility breadth | 3.1-3.4 | `crates/kali_optimize`, `crates/kali_api_node`, runtime/package crates, package corpus, benchmark support in `scripts/` and `tests/` | only widen host/package breadth after optimization and runtime foundations are stable | measurable release-mode gains, documented Node path, broader package corpus evidence |
| R8 — Dynamic compatibility and proof-backed boundary | 4.1-4.2 | runtime/package crates, `proofs/`, schemas/CLI docs | late dynamic features and proof-backed claims should come only after earlier semantics are settled | gated dynamic compatibility features plus a non-empty published proof boundary |
| R9 — Deferred platform expansion | 5.1-5.5 | runtime/host/binding/optimization crates, browser fixtures, bindings, scripts | extend the platform one explicit surface at a time without weakening earlier guarantees | each deferred surface has its own demo and evidence lane |

## Directory-by-directory guidance

### `crates/`

Use the current focused crates as the primary implementation boundaries:
- `kali_cli`, `kali_common`, `kali_error` for command dispatch, spans, config discovery, and envelopes,
- frontend crates for syntax and semantics,
- IR/codegen/runtime crates for executable behavior,
- host API crates for surface-specific widening,
- dedicated crates for sandbox, packages, embedding, optimization, and C ABI work.

Do **not** merge these crates just to match a more abstract logical layout from the plan. The logical layout is for ownership clarity; the current crate split is already a sensible implementation structure.

### `tests/`

Grow top-level evidence lanes in this order:
1. `tests/integration`
2. `tests/conformance`
3. `tests/browser-smoke`
4. `tests/determinism`
5. `tests/package-corpus`
6. later benchmark/coverage/proof-adjacent support lanes as needed

The important rule is that test directories should mirror the evidence matrix from [`../specs/16-testing.md`](../specs/16-testing.md), not arbitrary subsystem boundaries.

### `fixtures/`

Create small, reviewable fixture trees that line up with the evidence lanes:
- `fixtures/compiler`
- `fixtures/runtime`
- `fixtures/packages`
- `fixtures/browser`
- `fixtures/cli`

Prefer many narrow fixtures over a few giant sample applications.

### `schemas/`, `types/`, and `bindings/`

Treat these as contract trees, not afterthoughts:
- `schemas/` should track the schema-v1 machine-readable contracts,
- `types/` should hold host-facing type packages or generated type helpers,
- `bindings/` should hold stable host-language projections once embedding surfaces open.

These should stay top-level so reviewers can inspect machine contracts without digging through implementation crates.

### `scripts/`

This repository already uses `scripts/`, so the plan should align to that path rather than inventing a parallel `tools/scripts/` tree.

Use `scripts/` only for helper automation behind canonical `mise` tasks or CI workflows. Day-to-day instructions should still point contributors at the stable `mise` entrypoints first.

## What not to do early

Avoid these common planning mistakes:
- creating a new top-level directory for every feature,
- merging focused crates before the dependency graph proves that necessary,
- placing normative behavior docs in crate-local notes instead of `specs/` and `plan/`,
- building broad package/browser/Node test corpora before the local compiler/runtime path is stable,
- or claiming maturity just because a directory now exists.

## Practical startup order for a fresh implementation push

If implementation restarted from the current spec set today, the recommended order would be:
1. verify `SPEC.md`, `specs/`, `PLAN.md`, and `plan/` stay aligned,
2. close or harden the `kali_cli` / `kali_common` / `kali_error` workspace foundation,
3. complete frontend crates through deterministic `kali check`,
4. complete lowering/codegen/runtime through deterministic local execution,
5. open the Phase-1 product-surface parallel zone,
6. close the Phase-1 evidence lanes,
7. move MIR/ownership to the center of post-MVP work,
8. widen compatibility only after the evidence matrix can prove each new rung.

## Maintenance rule

Update this file whenever the recommended **directory or crate growth order** changes.

That usually means updating it together with:
- [`../PLAN.md`](../PLAN.md),
- [`01-repository-layout.md`](./01-repository-layout.md),
- [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md),
- and any phase README whose implementation guidance moved.
