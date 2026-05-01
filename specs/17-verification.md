# 17 — Formal Verification

Kali uses Lean 4 to verify selected high-value invariants over time. Verification is iterative and bounded: public claims are limited to the currently published boundary in [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).

Planning ownership:
- this chapter defines the **verification program**, proof-boundary discipline, and claim rules
- [`PLAN.md`](../PLAN.md) and [`plan/`](../plan) own milestone sequencing, proof-work ordering, and implementation tasks
- [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) owns the repository's **current** proof-backed scope

## Core verification rule

Lean proofs justify claims only for the subsystem slice named by the published proof boundary. They do **not** automatically:
- widen feature maturity rows,
- replace implementation or conformance testing,
- imply blanket coverage of the full JS/TS surface or all host behavior.

## Proof-ready vs proof-backed

Kali distinguishes two verification states:

| State | Minimum requirement | Allowed public claim |
|---|---|---|
| **proof-ready** | published `proofs/BOUNDARY.md`, honest proof-CI trigger policy, and no-overclaim discipline | the repository is prepared for phased verification work |
| **proof-backed** | non-empty published boundary naming concrete modeled subsystems plus mechanized theorem/property inventory | formal verification may be cited, but only for the published boundary |

Rules:
- Phase 1 requires the **proof-ready** baseline.
- Proof-backed release/support wording requires a non-empty published boundary.
- The current verification state is always read from [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), not inferred from roadmap prose.

## Scope discipline

Verification targets a **core Kali calculus** and selected high-value implementation invariants first. Early proof work should not overclaim the full language surface.

Late-compatibility features such as dynamic code execution, dynamic module loading, weak/finalization semantics, and concrete browser/OS host behavior remain outside the proof boundary until their semantics are stable enough to model honestly.

## Published proof boundary

`proofs/BOUNDARY.md` is the canonical published statement of:
- the modeled calculus/subsystem slice currently covered,
- the named theorems/properties currently claimed,
- trusted assumptions and explicitly unmodeled features,
- covered implementation/spec paths, and
- the proof-CI trigger rule.

Release notes, README summaries, and maturity claims must treat that manifest as the single source of truth for current proof status.
- canonical repository summary: **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**
- theorem/property inventory, covered paths, trusted assumptions, and current mechanized scope remain owned exclusively by [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md)

## Covered-boundary edit discipline

Once the published boundary is non-empty:
- if a change touches a subsystem or invariant named inside the boundary, the same change must either update the matching Lean model/proofs or narrow the published boundary first
- widening the boundary requires explicitly naming the new covered paths and theorem inventory in `proofs/BOUNDARY.md`
- shrinking the boundary is allowed as an honesty move, but all public wording must immediately follow the narrower boundary

## Verification focus areas

### Type-system soundness
For the modeled core fragment, verification should prioritize:
- progress
- preservation
- realistic structural-typing lemmas needed by the proved fragment
- termination/decidability results only for the explicit inference fragment being verified

Do not overclaim principality or full-language soundness for the whole TypeScript-compatible surface.

### Effect-system correctness
For the modeled capability subset, verification should prioritize:
- conservative effect inference
- sound sandbox-policy decision/enforcement behavior

### Memory safety
For the modeled ownership/reference-counting fragment, verification should prioritize:
- no-dangling-reference style safety invariants
- soundness of ownership/escape analysis assumptions used by the model
- refcount-helper invariants for the published RC snapshot slice named in `proofs/BOUNDARY.md`

The exact current theorem inventory belongs in the published boundary, not duplicated here.

### Selective lowering correctness
Where modeled, prove high-value lowering/desugaring steps preserve the intended semantics of the verified fragment.

## Proof-backed support boundary

A proof claim is one evidence lane among several. Public support wording should require both:
1. the proof claim staying inside the published proof boundary, and
2. the matching implementation/testing evidence from [16 — Testing](./16-testing.md) for the command/profile being claimed.

This prevents proof prose from outpacing runtime, package, sandbox, or CLI evidence.

## Methodology

### Modeling
- Define a simplified operational semantics for Kali's core language in Lean.
- Model only the fragment needed for the current proof claim.
- Keep unmodeled features explicit in the proof boundary.

### Proof ↔ implementation link
Lean proves properties of the model. The implementation is kept aligned through:
- tests derived from the model,
- regression cases informed by proof work, and
- review of the implementation/spec correspondence for covered paths.

### Iterative workflow
1. define or revise the model
2. prove the target properties
3. align the implementation with the model
4. add or update evidence showing the covered behavior matches the claim

## CI discipline

Proof CI follows the published boundary:
- proof jobs always trigger for changes under `proofs/`
- once the boundary names covered implementation/spec paths, proof jobs also trigger for changes to those covered areas
- areas outside the current boundary may evolve without widening proof claims

Hosted CI layout and milestone sequencing belong to the implementation plan rather than this chapter.

## Non-goals

This chapter does **not** claim:
- full ECMA-262 formalization,
- full-host end-to-end verification,
- blanket proof coverage for all TypeScript-compatible surface features,
- verification of every backend/tooling component merely because a core calculus fragment is proved.

## Practical implementation note

Concrete proof milestones, Lean-project staging, and deeper verification expansion belong to the active plan set, primarily:
- [`PLAN.md`](../PLAN.md)
- [`plan/phase-15/README.md`](../plan/phase-15/README.md)

The theorem/property inventory itself remains owned only by [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).
