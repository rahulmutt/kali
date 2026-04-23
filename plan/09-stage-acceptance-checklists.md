# Stage Acceptance Checklists

This guide defines the repeatable acceptance checklist that every implementation stage should satisfy before it is treated as complete.

Use it together with:
- [`../PLAN.md`](../PLAN.md) for phase sequencing and completion gates
- [`00-planning-conventions.md`](./00-planning-conventions.md) for stage-writing rules
- the relevant phase README and stage document for stage-specific tasks

## Core rule

A stage is only complete when its implementation, docs, and evidence all agree.

No stage closes on code motion alone.

## Universal checklist

Every stage should satisfy all of the following:

### 1. Implementation slice
- the scoped code/config changes for the stage are present
- the stage leaves the repo in a buildable state
- earlier working demos still work

### 2. Spec-coordination slice
- the owning spec chapter is updated when public behavior changed
- `specs/12-cli.md` is updated when CLI shape changed
- `specs/15-errors.md` is updated when diagnostics changed
- `specs/18-schemas.md` is updated when JSON/config/artifact contracts changed
- `specs/19-feature-maturity.md` is updated when availability claims changed
- `README.md` is updated when user-facing usage materially changed

### 3. Evidence slice
- unit and integration coverage exists for the new behavior
- any required checker, browser, package, determinism, or proof lanes are updated
- new evidence matches the exact support rung being claimed

### 4. Operator proof
- at least one concrete command or demo fixture shows the stage milestone working now
- the demo uses the documented command shape rather than an internal-only path

### 5. Regression proof
- `cargo test --workspace` passes
- any relevant `mise` task for the stage passes
- no previously claimed surface regresses

## Batch-promotion checklist

Use this condensed gate before promoting work from one major batch to the next:

| Batch boundary | Minimum close-out proof |
|---|---|
| planning baseline → frontend spine | command/diagnostic/schema owners are stable enough that frontend work will not redefine them ad hoc |
| frontend spine → end-to-end local execution | `kali check` is deterministic and fixture-backed for explicit local inputs |
| end-to-end local execution → Phase-1 product parallel zone | `kali build`, `kali run`, and `kali test` form a repeatable local demo loop |
| Phase-1 product parallel zone → evidence closure | shared CLI/error/schema/maturity owners are aligned across sandbox, package, artifact, and workflow streams |
| Phase-1 evidence closure → Phase-2 semantic stabilization | evidence proves the shipped Phase-1 surface, not only isolated demos |
| Phase-2 stabilization → breadth phases | MIR/ownership, effects, embedding, proof foundation, and coverage are all settled enough to support widening |

## Additional checklist by stage family

### Frontend stages
- deterministic fixtures exist for parsing/checking output
- diagnostics use canonical codes and source locations
- `.js` handling stays aligned with the bounded inference contract

### Lowering/runtime stages
- emitted WASM is deterministic for equivalent inputs
- runtime failures flow through stable diagnostics
- AOT-only and no-tracing-GC invariants remain intact

### CLI/schema/tooling stages
- text and JSON paths both behave as documented
- schema examples or snapshots are updated
- help text matches actual behavior

### Package/browser/host stages
- support claims name the exact rung and context
- unsupported contexts fail honestly through the documented gate
- browser-targeted evidence is separate from standalone runtime evidence
- package-install success is not used as a proxy for buildable or executable support
- Node-path work does not widen browser or Deno-oriented claims by implication

### Verification stages
- proof-ready vs proof-backed wording stays honest
- `proofs/BOUNDARY.md` remains the single owner of the published proof boundary
- proof-related summary docs do not overstate coverage

## High-risk review triggers

Require an explicit cross-owner review before closing a stage when any of these are true:
- the change widens browser or Node wording
- the change adds or changes a JSON-producing mode
- the change touches both runtime behavior and CLI shape
- the change moves a package to a higher support rung
- the change modifies proof or verification wording

Use [`10-risk-register.md`](./10-risk-register.md) to identify which mitigations must ship with the stage.

## Exit-criteria handoff rule

Before marking a stage complete, verify:
1. the stage document's definition of done is met
2. the phase README exit assumptions still hold
3. the top-level plan does not need sequencing or dependency updates
4. follow-on stages have the prerequisites they expect

## Maintenance rule

Update this guide when the repository's minimum definition of stage completion changes.
