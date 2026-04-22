# Workstreams and Handoffs

This document describes the cross-phase implementation streams that cut across individual stages.

Use it together with [`../PLAN.md`](../PLAN.md):
- `PLAN.md` tells you *when* a stage should happen,
- this file tells you *which streams must coordinate* when that stage moves.

## Why this exists

Kali's phases are linear enough to be workable, but the real implementation work falls into a few streams that repeatedly hand work to each other:
- frontend semantics
- lowering and runtime
- packages and host surfaces
- CLI/schema/diagnostics
- evidence and verification

Naming those streams explicitly helps prevent one stream from racing ahead and creating support claims that the others cannot yet uphold.

## Stream map

| Stream | Primary areas | First heavy phase | Main outputs |
|---|---|---|---|
| Frontend semantics | `kali_lexer`, `kali_parser`, `kali_ast`, `kali_types`, parser/checker fixtures | Phase 1 | tokens, AST, names, types, early IR |
| Lowering and execution | `kali_hir`, `kali_lir`, `kali_codegen`, `kali_runtime`, later `kali_mir` | Phase 1 | HIR/MIR/LIR, wasm, runtime behavior |
| Sandbox and effects | `kali_sandbox`, `kali_runtime`, `schemas/` | Phase 1 | policy validation, enforcement, later effect reports |
| Packages and ecosystem fit | `kali_npm`, frontend crates, host adapters | Phase 1 | install/lock/materialization, package resolution, corpus breadth |
| Product surface | `kali_cli`, `kali_error`, `schemas/`, README/help text | Phase 1 | commands, flags, JSON envelopes, user-facing behavior |
| Embedding and host integration | `kali_embed`, `kali_capi`, `bindings/`, artifact metadata | Phase 2 | stable library/API/C ABI/component outputs |
| Verification and evidence | `proofs/`, `tests/`, `fixtures/`, CI wiring | Phase 1 | evidence lanes, proof jobs, determinism gates |
| Optimization | `kali_optimize`, IR/codegen crates, benchmarks | Phase 3 | specialization depth, release-mode gains, PGO later |

## Current workspace lane mapping

To keep the present repository workable, contributors should treat the current fine-grained crates as one of the main guardrails for parallel work:
- do frontend changes primarily in `kali_lexer` / `kali_parser` / `kali_ast` / `kali_types`,
- do lowering/runtime changes primarily in `kali_hir` / `kali_lir` / `kali_codegen` / `kali_runtime`,
- do host-surface widening in the dedicated API crates (`kali_api_deno`, `kali_api_web`, `kali_api_node`),
- do command-shape and machine-contract changes in `kali_cli`, `kali_error`, and `schemas/` together.

## Canonical handoff points

### 1. Frontend semantics → lowering and execution

Handoff opens after Stage 1.5.

The lowering/runtime stream should not treat parser acceptance as semantic support. Before consuming frontend work, confirm:
- names resolve deterministically,
- type facts are stable enough for lowering,
- unsupported semantics are still diagnosed instead of silently lowered.

### 2. Lowering and execution → sandbox and effects

Handoff opens after Stage 1.8 for runtime enforcement, and deepens again after Stage 2.1 for stable effect reporting.

The sandbox/effects stream needs:
- a canonical execution model,
- explicit host-capability mediation points,
- stable ownership/layout semantics before publishing effect facts broadly.

### 3. Packages and ecosystem fit → frontend/runtime

Package work starts in earnest after the local-file pipeline is already workable.

The package stream hands over:
- deterministic materialized dependency graphs,
- bare-specifier resolution inputs,
- package-shape and host-fit facts,
- package corpus evidence per support rung.

The frontend/runtime streams must not widen package claims beyond the exact rung the package stream has proved.

### 4. Product surface ↔ every other stream

This is the most frequent coordination boundary.

Any stream that changes user-visible behavior must coordinate on:
- [`../specs/12-cli.md`](../specs/12-cli.md)
- [`../specs/15-errors.md`](../specs/15-errors.md)
- [`../specs/18-schemas.md`](../specs/18-schemas.md)
- [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md)

If these are not aligned, the stage is not really complete.

### 5. Verification and evidence ↔ every other stream

No stream gets to call its work supported without matching evidence.

Evidence handoff means:
- the stage has positive tests for what shipped,
- negative tests for what remains gated,
- deterministic output assertions where machine contracts exist,
- proof updates if the verification boundary changed.

## Safe parallelism by phase

### Phase 1
- Sequential critical path: 1.1 → 1.8
- Safe parallel zone: 1.9 → 1.14
- Required shared coordination: CLI, diagnostics, schemas, maturity, and workspace tests

### Phase 2
- Stage 2.1 is the semantic hinge
- After 2.1, effect reporting, embedding, verification-foundation, and coverage work can proceed in parallel if they share the same MIR/ownership assumptions

### Phase 3
- Stage 3.1 goes first
- Node compatibility and host-capability expansion can then proceed in parallel
- ecosystem breadth should consume those results rather than inventing separate compatibility rules

### Phase 4
- Dynamic compatibility and proof depth are separate streams, but both depend on earlier-phase stability
- neither should become a dumping ground for Phase-5 deferred breadth

### Phase 5
- additive only, feature by feature
- every stream must preserve the earlier command and schema contracts rather than forking them

## Stream-specific anti-patterns

### Frontend semantics
Do not claim runtime support just because syntax parses.

### Runtime/host work
Do not emulate unsupported host features loosely just to keep execution going.

### Package work
Do not translate one successful install or one passing package into blanket npm compatibility wording.

### CLI/schema work
Do not document a command as publicly available just because the shape is already defined.

### Verification work
Do not let proof-ready language drift into proof-backed marketing.

### Optimization work
Do not trade away deterministic or auditable behavior for a premature performance claim.

## Handoff checklist for parallel streams

When two streams need to land near each other, review this checklist before merging:

1. Are the same command names and flags used everywhere?
2. Are diagnostics using the canonical code families?
3. Are JSON envelopes and payload schemas still version-consistent?
4. Does `specs/19-feature-maturity.md` still describe the exact public status honestly?
5. Do tests prove both the positive path and the still-gated path?
6. If proof claims changed, does `proofs/BOUNDARY.md` match?

If any answer is no, keep coordinating before merging.
