# Phase 10 — Verification and Contract Hardening

## Goal

Widen proof-backed and machine-contract confidence while keeping claim boundaries exact.

## Owning specs

- `specs/16-testing.md`
- `specs/17-verification.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`
- `proofs/BOUNDARY.md`

## Work packets

### 10.1 Boundary hygiene

- Keep `proofs/BOUNDARY.md` as the sole theorem/property inventory.
- Remove duplicate proof inventories from plan docs; this phase index only points to the boundary.
- Update boundary text whenever covered Lean paths or theorem claims change.

### 10.2 Model widening

- Widen the core semantics model only with mechanized progress/preservation or equivalent property inventory.
- Widen ownership/RC proofs toward the actual implementation model in small named slices.
- Widen HIR/lowering correctness only for source fragments with corresponding Lean semantics.

### 10.3 Proof CI triggers

- Expand proof-trigger paths if and only if `proofs/BOUNDARY.md` starts claiming implementation/spec files outside `proofs/`.
- Keep `mise run lean-proofs` as the canonical proof build command.

### 10.4 Schema and machine-contract hardening

- Add schema validation for all JSON outputs and artifact manifests.
- Keep deterministic ordering and envelopes stable.
- Add docs/schema drift tests for CLI examples and result payloads.

## Exit gate

- `mise run lean-proofs` passes for any proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No release or README wording exceeds the published proof boundary.
