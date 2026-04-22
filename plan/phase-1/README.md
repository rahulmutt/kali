# Phase 1 — Core Compiler & Toolchain MVP

**Implements:** `specs/01` through `specs/18` for the shipped Phase-1 surface, with actual availability controlled by [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)

## Objective

Deliver the first fully workable Kali toolchain:
- parse and check TypeScript/JavaScript,
- lower to internal IR,
- generate deterministic WebAssembly,
- execute through the Kali-hosted runtime,
- validate and enforce the Phase-1 sandbox contract,
- build executable, bundle, and base-library artifacts,
- install supported packages deterministically,
- expose the core CLI and schema-v1 machine-readable outputs,
- back the shipped surface with evidence.

## Entry criteria

Before starting this phase, the repository should already have the normative spec set in place and the team should agree that Phase-1 availability is read from [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md), not from raw implementation progress.

## Why this order

Phase 1 starts with the frontend because every later subsystem depends on a stable parsed and typed program model. It then moves into lowering, code generation, and runtime so the repository reaches a local-file end-to-end compiler before package management and broader workflow polish. Once runtime execution exists, the remaining streams can proceed in parallel around the shared CLI/schema/error contracts.

## Stage graph

### Critical path

`1.1 → 1.2 → 1.3 → 1.4 → 1.5 → 1.6 → 1.7 → 1.8`

### Parallelizable after 1.8

`1.9`, `1.10`, `1.11`, `1.12`, `1.13`, and `1.14` may proceed in parallel if they coordinate on:
- [`specs/12-cli.md`](../../specs/12-cli.md)
- [`specs/15-errors.md`](../../specs/15-errors.md)
- [`specs/18-schemas.md`](../../specs/18-schemas.md)
- [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)
- `cargo test --workspace`

## Stages

| Stage | Focus | Primary spec owners |
|---|---|---|
| [1.1 — Workspace & Crate Scaffold](./01-workspace-scaffold.md) | workspace shape, CLI entrypoint, proof-ready baseline | `specs/01`, `specs/17` |
| [1.2 — Lexer](./02-lexer.md) | tokenization and lexical diagnostics | `specs/02`, `specs/15` |
| [1.3 — Parser & AST](./03-parser-and-ast.md) | grammar acceptance and source representation | `specs/02`, `specs/03` |
| [1.4 — Name Resolution](./04-name-resolution.md) | scopes, imports, symbol binding | `specs/03`, `specs/04`, `specs/14` |
| [1.5 — Type Checker](./05-type-checker.md) | bounded inference, TS/JS checking | `specs/04`, `specs/15` |
| [1.6 — HIR & LIR Lowering](./06-hir-lir-lowering.md) | typed lowering pipeline | `specs/05`, `specs/06` |
| [1.7 — WASM Code Generation](./07-wasm-codegen.md) | deterministic WASM emission | `specs/08` |
| [1.8 — Runtime & Execution](./08-runtime-execution.md) | `run` / `test` on the default standalone surface | `specs/10`, `specs/11`, `specs/12` |
| [1.9 — Sandbox & Policy](./09-sandbox-and-policy.md) | runtime enforcement and static validation | `specs/09`, `specs/18` |
| [1.10 — Package Management](./10-package-management.md) | install/lock/materialization | `specs/14`, `specs/18` |
| [1.11 — Build Artifacts](./11-build-artifacts.md) | executable, bundle, and base-library outputs | `specs/08`, `specs/11`, `specs/13` |
| [1.12 — Developer Workflow](./12-developer-workflow.md) | `init`, `fmt`, `lint`, and project ergonomics | `specs/12` |
| [1.13 — Diagnostics & Schemas](./13-diagnostics-and-schemas.md) | deterministic diagnostics and schema-v1 JSON | `specs/15`, `specs/18` |
| [1.14 — Evidence Hardening](./14-evidence-hardening.md) | conformance, browser smoke, determinism, CI evidence | `specs/16`, `specs/17`, `specs/19` |

## Phase-level workable-state ladder

| After stage | The repository should be able to demonstrate |
|---|---|
| 1.1 | workspace builds/tests plus `kali --version` |
| 1.3 | deterministic frontend parsing over fixture inputs |
| 1.5 | `kali check` on local TS/JS files |
| 1.7 | validated WASM emission from local programs |
| 1.8 | `kali run` and `kali test` in the default standalone context |
| 1.11 | `kali build` for executable, bundle, and base-library artifact modes |
| 1.14 | the full Phase-1 CLI/evidence surface working together |

## Workable-state rule for this phase

After every Phase-1 stage:
1. `cargo build` must succeed,
2. `cargo test --workspace` must pass,
3. at least one user-visible capability must remain demonstrable,
4. no stage may violate the hard invariants from [`SPEC.md`](../../SPEC.md).

## Exit gate

Phase 1 is complete only when:
- all stage files `1.1` through `1.14` are closed,
- the Phase-1 browser-targeted command set has smoke coverage,
- deterministic artifacts and CLI outputs are checked,
- package-corpus evidence matches the linked-artifact model,
- the repository remains at least proof-ready, and
- every public claim is aligned with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
