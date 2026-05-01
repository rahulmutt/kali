# Phase 15 — Verification and Machine-Contract Widening

## Goal

Widen proof-backed and machine-contract confidence while keeping claims exact.

## Owning specs

- `specs/16-testing.md`
- `specs/17-verification.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`
- `proofs/BOUNDARY.md`

## Work packets

### 15.1 Proof-boundary hygiene

- Keep `proofs/BOUNDARY.md` as the sole theorem/property inventory.
- Remove duplicate proof inventories from plan files.
- Update boundary text whenever covered Lean paths or theorem claims change.

### 15.2 Model widening

- Widen core semantics, ownership/RC, effects, and lowering models in small named slices.
- Pair every widened proof-backed claim with mechanized theorem inventory and proof-CI evidence.
- Keep full-language/full-host proof wording out of release claims until those semantics are modeled.

### 15.3 Proof CI triggers

- Use `mise run lean-proofs` for Lean builds.
- Expand proof-trigger paths only when `proofs/BOUNDARY.md` claims implementation or spec paths outside `proofs/`.

### 15.4 Schema and CLI contract hardening

- Continue validating JSON payloads, artifact manifests, diagnostics, source spans, and schema-v1 envelopes at emission boundaries.
- Keep docs/schema drift tests aligned with README and CLI examples.
- Respect schema extension posture; do not make validators narrower than published schemas.

## Exit gate

- `mise run lean-proofs` passes for proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No README, release, or plan wording exceeds the published boundary.
