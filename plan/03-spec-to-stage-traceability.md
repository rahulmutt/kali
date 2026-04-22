# Spec-to-Stage Traceability

This document connects the normative spec set to the implementation plan.

Use it when you need to answer any of these planning questions:
- Which stage first gives a spec chapter a real implementation home?
- Which later stages deepen that chapter instead of replacing it?
- Which evidence lane should justify promotion of a maturity row?
- Did a new plan change accidentally leave part of the spec set orphaned?

This file is planning-only. It does **not** decide public availability.
Availability still comes from [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).

## Reading rule

For each chapter below, read the columns in this order:
1. **First implementation home** — where the chapter first becomes materially real in code.
2. **Follow-on stages** — where the same chapter deepens, stabilizes, or broadens.
3. **Primary evidence lanes** — the test/proof lanes that should exist before opening or widening maturity rows.

If a chapter has multiple phases listed, that means the spec intentionally defines stable vocabulary earlier than the full public support surface ships.

## Chapter traceability matrix

| Spec owner | First implementation home | Follow-on stages | Primary evidence lanes |
|---|---|---|---|
| [`SPEC.md`](../SPEC.md) | Phase 1 planning baseline before 1.1 | all phases whenever shared vocabulary, guardrails, or claim-shaping rules change | doc review, release-claim consistency, maturity alignment |
| [`specs/01-architecture.md`](../specs/01-architecture.md) | [1.1](./phase-1/01-workspace-scaffold.md) | [1.6](./phase-1/06-hir-lir-lowering.md), [1.7](./phase-1/07-wasm-codegen.md), [1.8](./phase-1/08-runtime-execution.md), [3.1](./phase-3/01-optimization-and-specialization.md) | workspace tests, pipeline integration, deterministic build checks |
| [`specs/02-lexer-parser.md`](../specs/02-lexer-parser.md) | [1.2](./phase-1/02-lexer.md), [1.3](./phase-1/03-parser-and-ast.md) | [4.1](./phase-4/01-dynamic-compatibility.md) for late dynamic grammar/runtime edges where applicable | parser fixtures, syntax baselines, fuzzing, `test262` parsing subsets |
| [`specs/03-ast.md`](../specs/03-ast.md) | [1.3](./phase-1/03-parser-and-ast.md) | [1.4](./phase-1/04-name-resolution.md), [1.6](./phase-1/06-hir-lir-lowering.md) | AST snapshots, frontend integration tests |
| [`specs/04-type-system.md`](../specs/04-type-system.md) | [1.4](./phase-1/04-name-resolution.md), [1.5](./phase-1/05-type-checker.md) | [2.1](./phase-2/01-mir-and-ownership.md), [2.2](./phase-2/02-public-effect-reporting.md), [4.1](./phase-4/01-dynamic-compatibility.md) | checker baselines, JS inference fixtures, conformance lanes, effect-annotation tests |
| [`specs/05-ir.md`](../specs/05-ir.md) | [1.6](./phase-1/06-hir-lir-lowering.md) | [2.1](./phase-2/01-mir-and-ownership.md), [3.1](./phase-3/01-optimization-and-specialization.md) | IR validation tests, lowering snapshots, optimization regression tests |
| [`specs/06-memory.md`](../specs/06-memory.md) | [1.6](./phase-1/06-hir-lir-lowering.md) | [2.1](./phase-2/01-mir-and-ownership.md), [5.1](./phase-5/01-threaded-runtime-profile.md), [5.4](./phase-5/04-late-host-and-object-compatibility.md) | ownership/escape tests, runtime memory safety tests, proof work where boundary applies |
| [`specs/07-specialization.md`](../specs/07-specialization.md) | [3.1](./phase-3/01-optimization-and-specialization.md) | [5.5](./phase-5/05-pgo-and-language-bindings.md) | benchmarks, determinism checks for release artifacts, optimization regression corpus |
| [`specs/08-wasm-codegen.md`](../specs/08-wasm-codegen.md) | [1.7](./phase-1/07-wasm-codegen.md) | [1.11](./phase-1/11-build-artifacts.md), [3.1](./phase-3/01-optimization-and-specialization.md), [5.5](./phase-5/05-pgo-and-language-bindings.md) | wasm validation, artifact reproducibility, source-map checks |
| [`specs/09-sandboxing.md`](../specs/09-sandboxing.md) | [1.9](./phase-1/09-sandbox-and-policy.md) | [2.2](./phase-2/02-public-effect-reporting.md), [3.4](./phase-3/04-host-capability-expansion.md), [5.3](./phase-5/03-programmable-policy-and-algebraic-effects.md) | sandbox policy tests, runtime enforcement tests, effect-report snapshots, browser-targeted policy smoke |
| [`specs/10-runtime.md`](../specs/10-runtime.md) | [1.8](./phase-1/08-runtime-execution.md) | [3.2](./phase-3/02-node-compatibility.md), [4.1](./phase-4/01-dynamic-compatibility.md), [5.1](./phase-5/01-threaded-runtime-profile.md), [5.2](./phase-5/02-standalone-browser-runtime-and-host-expansion.md), [5.4](./phase-5/04-late-host-and-object-compatibility.md) | runtime integration tests, async/test runner coverage, late-compat negative tests |
| [`specs/11-standard-apis.md`](../specs/11-standard-apis.md) | [1.8](./phase-1/08-runtime-execution.md), [1.11](./phase-1/11-build-artifacts.md) | [3.2](./phase-3/02-node-compatibility.md), [3.4](./phase-3/04-host-capability-expansion.md), [5.2](./phase-5/02-standalone-browser-runtime-and-host-expansion.md), [5.4](./phase-5/04-late-host-and-object-compatibility.md) | API compatibility fixtures, browser-targeted smoke, host-capability policy tests |
| [`specs/12-cli.md`](../specs/12-cli.md) | [1.1](./phase-1/01-workspace-scaffold.md) | [1.8](./phase-1/08-runtime-execution.md), [1.10](./phase-1/10-package-management.md), [1.11](./phase-1/11-build-artifacts.md), [1.12](./phase-1/12-developer-workflow.md), [1.13](./phase-1/13-diagnostics-and-schemas.md), then every later feature-opening stage | CLI integration tests, snapshot tests, command-shape negative tests, JSON envelope checks |
| [`specs/13-embedding.md`](../specs/13-embedding.md) | [1.11](./phase-1/11-build-artifacts.md) for the base library artifact | [2.3](./phase-2/03-public-embedding-surface.md), [5.5](./phase-5/05-pgo-and-language-bindings.md) | artifact manifest tests, ABI metadata checks, embedding integration tests |
| [`specs/14-packages.md`](../specs/14-packages.md) | [1.10](./phase-1/10-package-management.md) | [1.4](./phase-1/04-name-resolution.md) for import-resolution handoff, [3.3](./phase-3/03-ecosystem-breadth.md), [4.1](./phase-4/01-dynamic-compatibility.md) | install/lock tests, package corpus by rung, determinism and cache-layout checks |
| [`specs/15-errors.md`](../specs/15-errors.md) | [1.2](./phase-1/02-lexer.md) for early diagnostics, stabilized in [1.13](./phase-1/13-diagnostics-and-schemas.md) | all user-visible stages that add or widen diagnostics | diagnostic snapshots, JSON diagnostic schema checks, stable-code regression tests |
| [`specs/16-testing.md`](../specs/16-testing.md) | [1.1](./phase-1/01-workspace-scaffold.md) baseline test harness discipline | [1.14](./phase-1/14-evidence-hardening.md), [2.5](./phase-2/05-test-coverage-and-reporting.md), plus every phase gate | CI lanes, conformance suites, package corpus, browser smoke, determinism |
| [`specs/17-verification.md`](../specs/17-verification.md) | [1.1](./phase-1/01-workspace-scaffold.md) proof-ready baseline | [2.4](./phase-2/04-lean-model-foundation.md), [4.2](./phase-4/02-formal-verification-depth.md) | `mise run lean-proofs`, proof CI, proof-boundary review |
| [`specs/18-schemas.md`](../specs/18-schemas.md) | [1.9](./phase-1/09-sandbox-and-policy.md) and [1.13](./phase-1/13-diagnostics-and-schemas.md) for the main schema-v1 machine contracts | [2.2](./phase-2/02-public-effect-reporting.md), [2.3](./phase-2/03-public-embedding-surface.md), [2.5](./phase-2/05-test-coverage-and-reporting.md), [4.1](./phase-4/01-dynamic-compatibility.md) | schema validation tests, snapshot tests, determinism checks |
| [`specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) | planning baseline before implementation stages open | every phase gate and every public-surface change | evidence review against `specs/16`, claim audits, release-note/README alignment |

## Command-family implementation checkpoints

Some parts of the spec set cut across many chapters. Use these checkpoints when reviewing whether a user-visible command family is really ready.

| Command family | Minimum stage before internal implementation is sensible | Stage where public hardening should occur |
|---|---|---|
| `kali check` | [1.4](./phase-1/04-name-resolution.md) | [1.5](./phase-1/05-type-checker.md) + [1.13](./phase-1/13-diagnostics-and-schemas.md) |
| `kali run` / `kali test` | [1.8](./phase-1/08-runtime-execution.md) | [1.9](./phase-1/09-sandbox-and-policy.md) + [1.14](./phase-1/14-evidence-hardening.md) |
| `kali build` executable/bundle/lib | [1.7](./phase-1/07-wasm-codegen.md) | [1.11](./phase-1/11-build-artifacts.md) + [1.13](./phase-1/13-diagnostics-and-schemas.md) |
| `kali install` | [1.10](./phase-1/10-package-management.md) | [1.14](./phase-1/14-evidence-hardening.md) |
| `kali effects` / `kali package-effects` | [2.1](./phase-2/01-mir-and-ownership.md) | [2.2](./phase-2/02-public-effect-reporting.md) |
| `kali package-audit` | [4.1](./phase-4/01-dynamic-compatibility.md) | [4.1](./phase-4/01-dynamic-compatibility.md) + matching schema/error updates |

## Maintenance rule

When a new stage is added, split, renamed, or re-scoped:
1. update `../PLAN.md` and the relevant phase README,
2. update this matrix so the affected chapter still has an explicit implementation home,
3. check whether `../specs/19-feature-maturity.md` wording needs adjustment,
4. confirm the evidence lane named here still matches the real promotion criteria.

If this file and a stage document disagree, the stage document owns the implementation detail, but this file should be corrected quickly so the plan set stays reviewable as a whole.
