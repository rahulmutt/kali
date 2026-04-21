# Stage 2.5 — Test Coverage & Reporting

**Phase:** 2 — Ownership, Effects & Public Embedding  
**Spec refs:** [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/16-testing.md`](../../specs/16-testing.md), [`specs/18-schemas.md`](../../specs/18-schemas.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.8 — Runtime & Execution](../phase-1/08-runtime-execution.md), [1.13 — Diagnostics & Schemas](../phase-1/13-diagnostics-and-schemas.md), and preferably [2.1 — MIR & Ownership Analysis](01-mir-and-ownership.md) so instrumentation can attach to a stable mid-pipeline IR

## Goal

Open the Phase-2 `kali test --coverage` surface with one explicit, deterministic reporting contract.
This stage exists because the spec intentionally leaves coverage **out of schema v1** until Kali can
name a stable machine-readable format instead of treating ad hoc runner text as the contract.

## Workable Milestone

- `kali test --coverage` is supported for the documented standalone execution contexts.
- Coverage output has one canonical machine-readable shape documented in `specs/18-schemas.md`.
- Human output and JSON output report the same underlying coverage result instead of diverging into
  separate semantics.
- Repeated runs over identical pinned inputs converge on deterministic coverage ordering and totals.

## Tasks

### 1. Coverage data model and schema ownership

Define the stable coverage payload before wiring the CLI:

- choose the canonical granularity for Phase 2 (`line`, `function`, and optional `branch` support)
- define per-file and aggregate totals
- decide how uncovered generated/shim code is excluded or marked
- add the coverage schema to [`specs/18-schemas.md`](../../specs/18-schemas.md) rather than
  leaving it implicit in the test runner
- keep the normal command-envelope rules from schema v1/vNext explicit: `--output json` wraps the
  coverage payload in the standard envelope, and text mode remains human-oriented only

### 2. Instrumentation strategy

Add deterministic coverage instrumentation to the execution pipeline:

- inject counters at a stable IR/codegen stage
- ensure instrumentation preserves source mapping back to user files
- keep instrumentation deterministic across repeated builds of the same pinned inputs
- make the instrumentation mode explicit so ordinary `kali test` stays lean when coverage is not
  requested

### 3. `kali test --coverage` CLI path

Implement the user-visible command path:

```bash
kali test --coverage
kali test --coverage src/math.test.ts
kali test --coverage --output json
```

Rules to preserve:

- `--coverage` extends `kali test`; it does not create a second test runner
- declaration-only file rejection, `--filter`, sandbox attachment, API-surface gating, and exit
  codes still follow the ordinary `test` command rules
- unsupported command/profile combinations keep using the same maturity gates and diagnostics

### 4. Report merging and determinism

Coverage often spans multiple test modules, so the merge rules must be explicit:

- stable file ordering
- deterministic aggregation of line/function/branch counters
- normalized paths rooted at the effective project root when available
- reproducible totals regardless of discovery order or parallel execution order

### 5. Evidence lane expansion

Extend the testing evidence required by the spec:

- positive integration coverage for `kali test --coverage`
- schema validation / golden tests for JSON output
- deterministic repeated-run tests
- negative tests for still-gated contexts (for example browser-runtime or other later profiles)
- fixture coverage proving the command respects `--filter`, explicit file sets, and sandboxed test
  execution rather than bypassing the ordinary test runner path

## Out of Scope

- browser-runtime coverage before the standalone browser test contract exists
- profiler-style performance coverage or tracing visualizers
- turning ad hoc text output into the only canonical contract

## Status

Planned.
