# Proof Boundary Manifest

Status: **provisional proof-boundary manifest**.

This file is the canonical repository location for Kali's published **proof-boundary manifest**. The repository now contains a checked-in Lean 4 proof tree under `proofs/`, but the manifest remains provisional and the repository is still **proof-ready**, not proof-backed.

Current repository-state note:
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](../SPEC.md): the Lean project tree now exists under `proofs/` and is built from `proofs/lakefile.lean`
- the proof sources are organized around `proofs/KaliCore.lean` and `proofs/KaliIR.lean`, which import the provisional model files listed below
- the current proof claims are still intentionally narrow and include documented `sorry` placeholders where later mechanization work is expected

Canonical verification state (following the shared **proof-ready vs proof-backed split** from [SPEC.md](../SPEC.md)):

| Item | Current state |
|---|---|
| proof-ready | **yes** — this manifest exists, truthfully declares the current claim boundary, and publishes the repository's current proof-CI trigger policy |
| proof-backed | **no** — the published boundary is provisional and the repository is not yet marketing mechanized proof coverage as a shipped capability |
| repository claim | **no mechanized proof coverage is claimed yet** |
| canonical short summary | **Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet.** |

Release rule:
- this manifest is acceptable during provisional Lean-model development because it keeps the repository honest about the currently modeled slice without overclaiming proof-backed support
- before any release markets formal verification as a shipped Kali capability, this manifest must move beyond the provisional state with a non-empty published theorem/property inventory that is intended for release/support claims
- until then, the Lean tree is a proof-ready modeling aid, not evidence that the whole repository is already proof-backed

## Modelled boundary

### Core type calculus (`proofs/KaliCore/Types.lean`, `proofs/KaliCore/Semantics.lean`, `proofs/KaliCore/Soundness.lean`)
- Type syntax: `Ty`, `LitVal`, and the provisional function/object/union/intersection forms
- Expression syntax: literals, variables, annotated functions, application, sequencing, conditionals, assignment, throw, and try/catch
- Runtime model: value predicate, substitution, and small-step reduction for the bounded typed fragment
- Claimed theorem inventory: progress and preservation for the closed typed core fragment
- Current proof state: theorem statements are present; the main soundness proofs are documented-sorry placeholders in `KaliCore/Soundness.lean`

### Ownership model (`proofs/KaliCore/Safety.lean`)
- Ownership classes: `stack`, `ownedHeap`, `sharedHeap`, `borrowed`
- Claimed property inventory: no dangling references for the provisional ownership model
- Current proof state: property statement present as a stub for later mechanization

### HIR model stub (`proofs/KaliIR/HIRModel.lean`)
- Provisional HIR syntax and a core lowering projection for future lowering-correctness work
- Current proof state: model stub only; no lowering-correctness claim is made here yet

## Claimed theorems/properties
- `KaliCore.Soundness.progress` — progress for the closed typed core fragment
- `KaliCore.Soundness.preservation` — preservation for the closed typed core fragment
- `KaliCore.Safety.NoDanglingReference` — provisional no-dangling-reference statement
- `KaliIR.HIRModel.lower_core` — sanity lemma for the provisional HIR lowering projection

## Trusted assumptions
- The proof tree is a provisional modeling aid; the release/support boundary remains proof-ready only.
- `sorry` placeholders are allowed in this stage and must be eliminated before any proof-backed marketing claim.
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
