# Implementation Risk Register

This document captures the main cross-spec risks that can make Kali appear more complete than it really is or can damage the workable-state rule.

Use it together with:
- [`../PLAN.md`](../PLAN.md)
- [`00-planning-conventions.md`](./00-planning-conventions.md)
- [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md)
- [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md)

It is a planning aid, not a second spec. Public behavior is still owned by `SPEC.md` and `specs/`.

## Risk scale

- **High** — likely to cause overclaiming, expensive rework, or broken stage workability
- **Medium** — manageable but requires explicit stage planning
- **Low** — mostly a consistency/hardening concern

## Active risks

Use the `First stages affected` column as the earliest point where the mitigation must already be part of the implementation packet, not as permission to ignore the risk before then.

| ID | Risk | Level | First stages affected | Required mitigation |
|---|---|---|---|---|
| R1 | Spec wording drifts from implementation order, causing support overclaims | High | pre-1.1 onward | always route availability wording through `specs/19-feature-maturity.md`; keep `PLAN.md` sequencing-only |
| R2 | Frontend work begins before command/diagnostic/error owners are stable enough | Medium | 1.1-1.5 | close the CLI/error spine first and snapshot diagnostics from the start |
| R3 | Lowering/codegen decisions get baked in before ownership semantics are coherent | High | 1.6-2.1 | keep Phase-1 lowering simple, then make MIR canonical in 2.1 before widening effect/embedding claims |
| R4 | Browser-targeted build/check support is mistaken for standalone browser runtime support | High | 1.9, 1.11, 5.2 | use the exact browser-targeted terminology from `SPEC.md`; keep `run/test --api browser` on the proper gated path until Phase 5 |
| R5 | Sandbox static validation, runtime enforcement, and effect reporting blur together | High | 1.9, 2.2, 5.3 | preserve the workflow-owner split and test each owner separately |
| R6 | Package install success is mistaken for executable support of the same package | High | 1.10, 3.3, 4.1 | use the package-support ladder in docs/tests and require rung-specific evidence |
| R7 | JSON envelopes, diagnostics, and artifact schemas diverge across commands | High | 1.13 onward | treat `specs/12`, `specs/15`, and `specs/18` as one review packet for user-visible command changes |
| R8 | Proof-ready process work is described as proof-backed product assurance | High | 1.1, 2.4, 4.2 | keep verification wording anchored to `proofs/BOUNDARY.md` and audit docs for overreach |
| R9 | Node-compatibility work starts before runtime and optimization foundations are stable | Medium | 3.2 | keep 3.2 behind 3.1 and 1.8/1.9 foundations; validate with real package fixtures |
| R10 | Late dynamic compatibility work weakens AOT-only or sandbox guarantees | High | 4.1 | require explicit gating, negative tests, and spec review before opening dynamic paths |
| R11 | Deferred threaded/runtime breadth leaks into default execution earlier than intended | Medium | 5.1-5.4 | keep thread-aware features profile-gated and prove the default profile did not widen |
| R12 | Planning docs become a second source of truth for current repository state | Medium | all phases | keep current-state notes clearly labeled and defer support claims to maturity/proof owners |

## Risk details and stage guidance

### R1 — Support overclaim through plan/spec confusion

**Failure mode**
- a stage file reads like a shipping commitment
- README or examples copy plan wording directly
- implemented internals are mistaken for public availability

**Mitigation**
- whenever a command family is mentioned in a planning doc, keep the wording implementation-oriented
- review `specs/19-feature-maturity.md` before changing README or user-facing command docs
- update maturity rows when and only when support is actually opened

### R3 — Premature semantic lock-in below the checker

**Failure mode**
- Phase-1 lowerings make implicit ownership assumptions
- later MIR introduction requires broad rewrites
- effect and embedding work build on unstable semantics

**Mitigation**
- keep 1.6-1.8 focused on the minimum workable path
- do not over-specialize the memory model before 2.1
- add IR snapshots early so later semantic changes are visible

### R4 — Browser wording drift

**Failure mode**
- docs say “browser support” without naming command/rung
- bundle success is confused with Kali-hosted browser runtime support
- browser ambient types are confused with mediated sandbox coverage

**Mitigation**
- always name the command: `check --api browser` or `build --bundle --api browser`
- keep browser smoke tests aligned to the Phase-1 browser-targeted command set
- reject contradictory browser command shapes through command-shape validation, not fallback behavior

### R5 — Sandbox/effects workflow blur

**Failure mode**
- `check/build --sandbox` is described as runtime enforcement
- `effects` output is described as policy enforcement
- registry analysis and source-graph analysis become conflated

**Mitigation**
- keep separate tests for runtime enforcement, static validation, and reporting
- treat `package-effects` and `package-audit` as explicit registry-analysis commands
- require schema review when effect-related output changes

### R6 — Package-rung inflation

**Failure mode**
- one package installs, so docs imply it runs
- one browser bundle builds, so docs imply browser execution support
- package evidence is not tied to a host/API context

**Mitigation**
- require package corpus entries to name command, context, and rung
- keep install tests separate from run/build tests
- document failures honestly for native/binary/bootstrap-heavy packages

### R7 — Machine-contract drift

**Failure mode**
- JSON envelope differences accumulate between commands
- diagnostic codes move without registry updates
- artifact metadata becomes non-deterministic

**Mitigation**
- review CLI/error/schema changes as one packet
- keep snapshot/schema validation tests for every JSON-producing path
- add determinism checks whenever artifact manifests change

### R8 — Verification overreach

**Failure mode**
- planning docs imply mechanized assurance beyond the published boundary
- non-empty Lean code is mistaken for proof-backed release language
- stage files duplicate proof inventory outside `proofs/BOUNDARY.md`

**Mitigation**
- keep `plan/phase-4/02-formal-verification-depth.md` as a reference back to `proofs/BOUNDARY.md`
- treat `mise run lean-proofs` as necessary but not sufficient for proof-backed wording
- review release-facing docs for proof language specifically

### R10 — Dynamic compatibility breaks hard invariants

**Failure mode**
- `eval`/`Function()` support bypasses the AOT-only rule
- dynamic loading widens host access without sandbox mediation
- compatibility shims become silent fallback paths

**Mitigation**
- require explicit feature gates
- add negative tests proving the default path still rejects unsupported forms
- pair runtime work with CLI/schema/error updates so the gating is visible

## Risk ownership by implementation stream

| Stream | Primary risks to watch first |
|---|---|
| Frontend + checker | `R1`, `R2`, `R3`, `R7` |
| Lowering + runtime | `R3`, `R4`, `R5`, `R7`, `R10`, `R11` |
| Packages + artifacts | `R1`, `R4`, `R6`, `R7`, `R9` |
| Workflow + machine contracts | `R1`, `R7`, `R8`, `R12` |
| Evidence + proofs | `R6`, `R8`, `R12` |

Use this table when staffing the post-`1.8` parallel window so every stream knows which risks it owns by default.

## Review triggers

Do an explicit risk review when any of these happen:
- a stage touches both CLI shape and runtime behavior
- a command starts producing new machine-readable output
- package support wording broadens
- browser or Node wording changes
- proof or verification wording changes
- a later-phase feature gets partial implementation before its maturity row opens

## Practical rule

If a change intersects a **High** risk area, do not treat it as a normal isolated implementation task. Ship it as a coordinated packet across code, docs, evidence, and maturity/proof owners.

If two or more **High** risks intersect in the same change, prefer reducing scope or splitting the work into smaller stage-aligned packets before merging.
