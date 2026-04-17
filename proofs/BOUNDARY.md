# Proof Boundary Manifest

Status: **proof-backed proof-boundary manifest**.

This file is the canonical repository location for Kali's published **proof-boundary manifest**. The repository now contains a checked-in Lean 4 proof tree under `proofs/`, and the published boundary is mechanized for the widened closed fragment plus a small ownership / RC snapshot safety slice described below. The repository is therefore **proof-backed for the published boundary**, while remaining intentionally narrower than the later Stage 4.2 ownership/memory-safety and lowering-correctness target.

Current repository-state note:
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](../SPEC.md): the Lean project tree now exists under `proofs/` and is built from `proofs/lakefile.lean`
- the proof sources are organized around `proofs/KaliCore.lean` and `proofs/KaliIR.lean`, which import the provisional model files listed below
- the current proof claims now cover the widened closed fragment (literals, variables, closed functions, application, sequencing, and conditionals) and the proof file compiles without `sorry` placeholders

Canonical verification state (following the shared **proof-ready vs proof-backed split** from [SPEC.md](../SPEC.md)):

| Item | Current state |
|---|---|
| proof-ready | **yes** — this manifest exists, truthfully declares the current claim boundary, and publishes the repository's current proof-CI trigger policy |
| proof-backed | **yes** — the published boundary names mechanized claims for the widened closed fragment and the repository may cite proof coverage for that boundary |
| repository claim | **proof-backed for the published boundary** |
| canonical short summary | **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.** |

Release rule:
- this manifest is acceptable because it keeps the repository honest about the currently modeled slice while still publishing mechanized theorem/property claims for that slice
- before any release markets the later Stage 4.2 ownership/memory-safety or lowering-correctness target as shipped evidence, this manifest must widen to name those additional theorem/property claims explicitly
- the Lean tree is proof-backed for the published boundary, but it is still a modeling aid for the later widening work rather than evidence that the whole repository is already covered

## Modelled boundary

### Core type calculus (`proofs/KaliCore/Types.lean`, `proofs/KaliCore/Semantics.lean`, `proofs/KaliCore/Soundness.lean`)
- Type syntax: `Ty`, `LitVal`, and the provisional function/object/union/intersection forms
- Expression syntax: literals, variables, annotated functions, application, sequencing, conditionals, assignment, throw, and try/catch
- Runtime model: value predicate and small-step reduction for the bounded typed fragment; the current proof boundary models the closed literals / variables / closed-functions slice plus the application, sequencing, and conditional subfragment that is now mechanised in `KaliCore.Soundness`
- Claimed theorem inventory: progress and preservation for the widened closed typed core fragment
- Current proof state: theorem statements are present and the core proof file now compiles, but the mechanised scope still stops short of assignment, exceptions, and the full Stage 4.2 memory/lowering target

### Ownership model (`proofs/KaliCore/Safety.lean`)
- Ownership classes: `stack`, `ownedHeap`, `sharedHeap`, `borrowed`
- Model shape: `RcCell` heap entries, `RcSnapshot` ownership/heap/live-reference state, and released-reference tracking
- Claimed property inventory: no dangling references for well-formed RC snapshots; released references are not live
- Current proof state: the `noDanglingReference` and `releasedNotLive` theorems are mechanised for the current RC snapshot model, but the model remains narrower than the eventual Stage 4.2 ownership / RC target

### HIR lowering model (`proofs/KaliIR/HIRModel.lean`, `proofs/KaliIR/LoweringCorrectness.lean`)
- Provisional HIR syntax and a core lowering projection for future lowering-correctness work
- Claimed property inventory: structural lowering equations (`lower_core`, `lower_let1`, `lower_seq`, `lower_if`) plus a small-step lowering-preservation bridge for the current HIR subset
- Current proof state: the structural equations are mechanised, and `KaliIR.LoweringCorrectness.lower_preserves_step` now proves the current HIR step relation is preserved by lowering for the modeled subset; the subset still stops short of the full Stage 4.2 semantic-preservation target

## Claimed theorems/properties
- `KaliCore.Soundness.progress` — progress for the widened closed typed core fragment
- `KaliCore.Soundness.preservation` — preservation for the widened closed typed core fragment
- `KaliCore.Safety.noDanglingReference` — mechanised no-dangling-reference theorem for the current RC snapshot model
- `KaliCore.Safety.releasedNotLive` — mechanised theorem that released references are not live in the current RC snapshot model
- `KaliIR.HIRModel.lower_core`, `lower_let1`, `lower_seq`, `lower_if` — structural lowering equations for the provisional HIR projection
- `KaliIR.LoweringCorrectness.lower_preserves_step` — lowering-preservation bridge for the current HIR step subset

## Trusted assumptions
- The proof tree is a proof-backed modeling aid for the published closed-fragment boundary.
- The current closed-fragment proof boundary is intentionally narrower than the eventual Stage 4.2 ownership/memory-safety and lowering-correctness target and must be widened before any claim about that later target.
- The ownership slice currently models a small RC snapshot with live-reference and release tracking; it is still narrower than the eventual full ownership / reference-counting story.
- The lowering-correctness bridge is intentionally limited to the current HIR subset, not the later full HIR → LIR semantic-preservation target.
- The currently mechanised fragment now includes application, sequencing, and conditionals in addition to the original closed-literal/variable/closed-function slice.
- No mechanized proof coverage is claimed for Rust implementation code outside `proofs/`.

## Explicitly unmodeled features
- Full ECMAScript/TypeScript surface semantics
- Host integrations and OS behavior
- Dynamic compatibility features such as `eval` / `Function()`
- Browser/Node compatibility layers beyond the future modeled core
- Full lowering/codegen correctness end to end

## Covered implementation/spec paths
- `proofs/lakefile.lean`
- `proofs/KaliCore.lean`
- `proofs/KaliIR.lean`
- `proofs/KaliCore/Types.lean`
- `proofs/KaliCore/Semantics.lean`
- `proofs/KaliCore/Soundness.lean`
- `proofs/KaliCore/Safety.lean`
- `proofs/KaliIR/HIRModel.lean`
- `proofs/KaliIR/LoweringCorrectness.lean`

## Required implementation/spec alignment scope
- `proofs/lakefile.lean` must continue to declare the Lean roots that keep the provisional proof tree complete
- `proofs/KaliCore.lean` and `proofs/KaliIR.lean` serve as the root import surfaces for the provisional Lean model
- any change to the named proof files above should keep the boundary text and trigger policy in sync with the current modeled slice

## Proof-CI trigger policy
- Trigger proof CI when `proofs/**` changes.
- Because the published boundary is still provisional and only names Lean model files under `proofs/`, no broader Rust/spec trigger set is currently claimed.
- If the boundary later names covered implementation/spec subsystems outside `proofs/`, proof CI must also trigger for changes to those covered areas.

## Boundary-maintenance rule
- once the published boundary becomes non-provisional, a change to any covered path must land with matching proof updates or an explicit narrowing of the boundary first
- widening the boundary also requires updating the named theorem/property inventory; new Lean files alone do not widen the claim surface
- release/support wording must always follow this file's current boundary immediately after such a change
