# Stage Dependency Matrix

This document makes the roadmap executable at a glance.

Use it when you need to answer:
- what a stage truly depends on,
- what can proceed in parallel after it,
- which repository areas should carry most of the change,
- and what concrete demo should still work when the stage closes.

It complements rather than replaces the phase READMEs and stage files:
- [`../PLAN.md`](../PLAN.md) owns the overall ordering,
- phase READMEs own phase-local context,
- stage files own detailed tasks and definitions of done,
- this file owns the compact dependency view.

## Reading rule

For each row, read the columns in this order:
1. **Hard dependencies** — prerequisites that must exist before the stage is meaningful.
2. **Then opens / parallel window** — what work becomes safe or sensible after the stage lands.
3. **Primary code areas** — where most of the implementation should live.
4. **Repository demo** — the minimum concrete capability that should still be showable.
5. **Primary evidence** — the lane that should prove the milestone.

## Phase 1 matrix

| Stage | Hard dependencies | Then opens / parallel window | Primary code areas | Repository demo | Primary evidence |
|---|---|---|---|---|---|
| 1.1 | none | 1.2 | `crates/kali`, `crates/cli`, `proofs/`, workspace config | `kali --version` | workspace build/test, proof-ready baseline |
| 1.2 | 1.1 | 1.3 | lexer/frontend crates, diagnostics | deterministic token dump or lexer diagnostics | lexer fixtures, diagnostic snapshots |
| 1.3 | 1.2 | 1.4 | parser/AST crates | parse supported TS/JS into a stable AST form | parser baselines, AST snapshots |
| 1.4 | 1.3 | 1.5 and package-resolution handoff | name-resolution/import logic, CLI check wiring | `kali check` resolves symbols/imports and reports failures | resolution tests, import fixtures |
| 1.5 | 1.4 | 1.6 | type-checker, diagnostics | `kali check` reports type errors under bounded inference | checker baselines, JS inference fixtures |
| 1.6 | 1.5 | 1.7 | lowering pipeline, HIR/LIR, memory-strategy scaffolding | typed programs lower through internal IR | lowering snapshots, IR validation |
| 1.7 | 1.6 | 1.8 and early build plumbing | wasm codegen, artifact emission | simple programs compile to validated WASM | wasm validation, reproducibility checks |
| 1.8 | 1.7 | 1.9-1.14 parallel zone | runtime, CLI run/test wiring, host adapters | `kali run` and `kali test` work on local programs | runtime integration tests |
| 1.9 | 1.8 | stronger product hardening and later effect work | sandbox crate, runtime enforcement, schemas | runtime sandbox enforcement plus static `--sandbox` validation | sandbox policy tests, browser-targeted policy smoke |
| 1.10 | 1.4, 1.8 | broader package/evidence work and Phase-3 ecosystem breadth | packages crate, lock/install/cache plumbing | `kali install` materializes deterministic supported packages | install/lock tests, cache determinism |
| 1.11 | 1.7, 1.8 | Phase-2 embedding stabilization | build artifact plumbing, embed artifact metadata, browser bundle path | `kali build` emits executable, bundle, and base-library artifacts | artifact manifest tests, build reproducibility |
| 1.12 | 1.1 and enough CLI plumbing from 1.8/1.10/1.11 | smoother end-user workflows and scaffold-based demos | CLI/workflow crates, project scaffolds, formatter/linter | `kali init`, `kali fmt`, and `kali lint` work end to end | CLI workflow tests, snapshot tests |
| 1.13 | user-visible command slices from 1.8-1.12 | stable public machine contracts | CLI envelopes, diagnostics, schemas | `--output json` and stable diagnostic codes work consistently | JSON schema validation, snapshot determinism |
| 1.14 | 1.8 as minimum, ideally 1.9-1.13 in flight or landed | Phase-1 completion gate | tests, proofs, fixtures, CI | Phase-1 CLI surface works with passing evidence lanes | conformance, package corpus, browser smoke, determinism, proof CI |

## Phase 2 matrix

| Stage | Hard dependencies | Then opens / parallel window | Primary code areas | Repository demo | Primary evidence |
|---|---|---|---|---|---|
| 2.1 | Phase-1 lowering/runtime baseline | 2.2, 2.3, 2.4, and most of 2.5 | MIR, ownership/escape analysis, memory strategy | MIR is canonical and drives allocation decisions | MIR validation, ownership tests |
| 2.2 | 2.1 | later policy/effect expansion | sandbox/effect reporting, CLI, schemas | `kali effects` / `kali package-effects` emit stable reports | effect snapshots, policy-comparison tests |
| 2.3 | 2.1 and 1.11 | 5.5 binding expansion | embedding crates, artifact metadata, ABI/version checks | stable `--lib` / `--capi` / `--component` outputs | embedding integration tests, ABI metadata checks |
| 2.4 | 1.1 proof-ready baseline and stable enough semantics from 2.1 | 4.2 proof-depth work | `proofs/`, semantics models, proof CI | `mise run lean-proofs` succeeds on a meaningful model | Lean proof builds, proof-CI checks |
| 2.5 | 1.8 and reporting plumbing from 2.1/1.13 | stronger release/evidence reporting later | test runner, CLI reporting, schemas | `kali test --coverage` emits deterministic function-coverage reports | coverage snapshots, deterministic reports |

## Phase 3 matrix

| Stage | Hard dependencies | Then opens / parallel window | Primary code areas | Repository demo | Primary evidence |
|---|---|---|---|---|---|
| 3.1 | 2.1 and Phase-1 codegen baseline | 3.2 and 3.4 in parallel, then 3.3 | optimize crate, MIR/LIR passes, release build pipeline | `--release` / `--release-advanced` show measurable gains | benchmarks, optimization regressions |
| 3.2 | 3.1 and 1.8 | contributes to 3.3 | node API surface, runtime adapters, CLI/context gating | documented `--api node` path works for supported subset | Node compatibility fixtures, host tests |
| 3.3 | 1.10 plus 3.2/3.4 foundations where relevant | later dynamic compatibility breadth | packages, import lowering, browser/build/package corpus | named packages move up explicit support rungs | package-corpus evidence, bundle/package smoke |
| 3.4 | 3.1 and 1.9 | contributes to 3.3 and later runtime breadth | runtime host adapters, sandbox/resource enforcement | mutable env, subprocess, socket/listener capabilities work under policy limits | host-capability tests, resource-limit tests |

## Phase 4 matrix

| Stage | Hard dependencies | Then opens / parallel window | Primary code areas | Repository demo | Primary evidence |
|---|---|---|---|---|---|
| 4.1 | runtime/package/host groundwork from Phases 1-3 | later compatibility expansion in Phase 5 | runtime compatibility gates, package analysis, CLI/schema work | gated `eval`/`Function()`/dynamic loading and public `package-audit` | late-compat tests, package-audit schema checks |
| 4.2 | 2.4 | proof-backed maintenance and later proof growth | proofs, boundary docs, CI | non-empty `proofs/BOUNDARY.md` with matching proof jobs | Lean proofs, proof-boundary review, claim audits |

## Phase 5 matrix

| Stage | Hard dependencies | Then opens / parallel window | Primary code areas | Repository demo | Primary evidence |
|---|---|---|---|---|---|
| 5.1 | Phase-4-stable runtime core | thread-aware browser/runtime breadth | runtime scheduler, memory/thread budgeting, host adapters | opt-in threaded execution with budget enforcement | threaded runtime tests |
| 5.2 | 1.11 browser-targeted build maturity; 5.1 when thread-aware paths matter | broader browser/runtime host work | browser runtime adapter, CLI gating, runtime host plumbing | standalone `run/test --api browser` works within documented scope | browser runtime smoke, policy-compat tests |
| 5.3 | 2.2 and 2.3 | richer policy/effect integrations | sandbox, embedding, effect machinery | programmable policy/effect extensions preserve earlier declarative guarantees | policy-extension tests, effect-surface regression tests |
| 5.4 | 5.1 for thread-sensitive corners; otherwise late runtime maturity | late object-model/host expansion | runtime semantics, host compatibility layers | weak refs, proxies, and other late semantics work only through explicit gates | compatibility corpus, negative gating tests |
| 5.5 | 2.3 and 3.1 | additive optimization/deployment depth | optimize, embed, bindings, build pipeline | deterministic PGO-assisted builds and broader bindings over the stable ABI | PGO reproducibility checks, binding integration tests |

## Practical use

Use this matrix before opening work on a stage:
1. verify the hard dependencies are already landed,
2. keep most code in the listed ownership areas,
3. decide the exact demo command before coding,
4. identify the evidence lane that must ship with the change,
5. update [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md) if the scope moves.

## Maintenance rule

If stage boundaries change, update this file together with:
- [`../PLAN.md`](../PLAN.md),
- the affected phase README,
- the affected stage docs,
- and any evidence/traceability references that changed.
