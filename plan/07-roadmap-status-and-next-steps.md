# Roadmap Status and Next Steps

This guide complements [`../PLAN.md`](../PLAN.md) by answering a narrower question:

> **Given the current spec set and workspace layout, what should implementation focus on next, what is safe to parallelize, and what should stay blocked until earlier demos are real?**

Use this file when the broad phase map is clear but day-to-day prioritization still needs a sharper answer.

## Core rule

- [`../PLAN.md`](../PLAN.md) and the phase/stage files own implementation order
- [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) owns public availability
- [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) owns the current proof-backed boundary
- this file is the **execution-priority overlay** for near-term work, not a second maturity matrix

## Current recommended execution order

Treat the roadmap as an active implementation queue with three levels of priority.

Current repository note:
- the phase checklists currently carried in this repository snapshot are all marked complete in their phase documents
- use this page as a prioritization overlay for future spec-led work, not as an open todo list for the closed stage packets

Recent hardening:
- package-audit and package-effects JSON envelopes are now pinned under inherited browser context and quiet mode, reducing machine-contract drift across analysis presentation flags
- browser-targeted static policy-validation coverage now exercises inherited browser API surfaces for both `check` and `build --bundle`, including the sandbox-attached variants that keep the browser-targeted command set aligned with inherited config
- the top-level CLI spine now has a dedicated `kali --version` smoke test, keeping the entrypoint contract pinned alongside the other command-shape regressions
- the global `--pretty` gate and the `package-audit --pretty` path now report the canonical `E5508` CLI-usage diagnostic, keeping the shared command-shape code aligned with `specs/15-errors.md`

### Priority A — finish the Phase-1 critical path

Do these first, in order, and keep them sequential unless a stage file explicitly says otherwise:

1. `1.1` workspace and CLI spine
2. `1.2` lexer
3. `1.3` parser and AST
4. `1.4` name resolution
5. `1.5` type checker
6. `1.6` HIR/LIR lowering
7. `1.7` WASM code generation
8. `1.8` runtime and execution

Why this is first:
- it is the shortest route to a believable local-file compiler/runtime loop
- it creates the semantic foundation every later package, sandbox, and browser claim depends on
- it keeps the repo in a continuously demoable state

### Priority B — use the post-1.8 parallel window carefully

Only after `1.8` is solid, open parallel work in:
- `1.9` sandbox and policy
- `1.10` package management
- `1.11` build artifacts
- `1.12` developer workflow
- `1.13` diagnostics and schemas
- `1.14` evidence hardening

These streams must stay synchronized on:
- [`../specs/12-cli.md`](../specs/12-cli.md)
- [`../specs/15-errors.md`](../specs/15-errors.md)
- [`../specs/18-schemas.md`](../specs/18-schemas.md)
- [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md)

### Priority C — only start post-MVP depth after evidence closure

After Phase 1 is complete, move in this order:
1. `2.1` MIR and ownership
2. `2.2`, `2.3`, `2.4`, `2.5`
3. `3.1`
4. `3.2` and `3.4` in parallel where safe
5. `3.3`
6. `4.1`
7. `4.2`
8. `5.x` one surface at a time

## Recommended next-read documents by planning question

| If the question is... | Read first | Then read |
|---|---|---|
| what should the team build immediately next? | [`08-fresh-implementation-roadmap.md`](./08-fresh-implementation-roadmap.md) | `../PLAN.md` + relevant phase README |
| what crates/directories should absorb that work? | [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md) | [`01-repository-layout.md`](./01-repository-layout.md) |
| can two streams proceed in parallel? | [`02-workstreams-and-handoffs.md`](./02-workstreams-and-handoffs.md) | `../PLAN.md` + relevant phase README |
| which stage owns a spec chapter or maturity row? | [`03-spec-to-stage-traceability.md`](./03-spec-to-stage-traceability.md) | exact stage file |
| what exact prerequisites and demo should one stage satisfy? | [`04-stage-dependency-matrix.md`](./04-stage-dependency-matrix.md) | exact stage file |
| what checklist should gate stage closure? | [`09-stage-acceptance-checklists.md`](./09-stage-acceptance-checklists.md) | [`00-planning-conventions.md`](./00-planning-conventions.md) + exact stage file |
| which cross-spec risks need extra hardening? | [`10-risk-register.md`](./10-risk-register.md) | relevant phase README + exact stage file |
| whether something is publicly shipped | [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) | owning spec chapter |
| what is proof-backed today | [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) | [`../specs/17-verification.md`](../specs/17-verification.md) |

## Near-term decision rules

Use these rules when picking work:

1. **Prefer the earliest missing demo over later breadth.**
   If `kali check` is not yet deterministic, do not prioritize Node or package breadth.

2. **Prefer closing command-shape owners before adding more commands.**
   If CLI/error/schema wording is drifting, fix that before opening more product surface.

3. **Prefer evidence closure before maturity promotion.**
   A working demo is not enough to widen public claims.

4. **Prefer explicit gates over partial emulation.**
   When a feature is phase-gated, add the honest gate path before adding half-support.

5. **Prefer stage packets that leave the repo usable.**
   Avoid large hidden internal rewrites that leave no stable demo behind.

## When to create a new planning document

Create a new plan doc instead of only editing an existing one when all of the following are true:

1. the work is larger than a hardening pass
2. it has its own dependency order or completion gate
3. it touches more than one subsystem or evidence lane
4. future contributors will benefit from a dedicated checklist

Otherwise, prefer updating the relevant phase README, stage file, or this prioritization guide.

## Maintenance rule

Keep this file compact and action-oriented.

When the plan is in a fully closed state, keep the prioritization notes short and point readers at the owning specs, maturity matrix, and evidence tracks instead of re-expanding the historical stage sequence.

- Do not duplicate spec contracts here
- Do not duplicate the proof-boundary inventory here
- Do not let this file become a second top-level plan
- Update it whenever the recommended near-term execution order changes
