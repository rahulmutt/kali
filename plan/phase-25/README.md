# Phase 25 — Verification and Machine Contracts

## Goal

Widen proof-backed and machine-contract confidence while keeping claims exact.

## Owning specs

- `specs/16-testing.md`
- `specs/17-verification.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`
- `proofs/BOUNDARY.md`

## Work packets

### 25.1 Proof-boundary hygiene

- Keep `proofs/BOUNDARY.md` as the sole theorem/property inventory.
- Remove duplicate proof inventories from plan files.
- Update boundary text whenever covered Lean paths or theorem claims change.

### 25.2 Model widening

- Widen core semantics, ownership/RC, effects, type-system, and lowering models in small named slices.
- Pair every widened proof-backed claim with mechanized theorem inventory and proof-CI evidence.
- Keep full-language/full-host proof wording out of release claims until those semantics are modeled.

### 25.3 Proof CI triggers

- Use `mise run lean-proofs` for Lean builds.
- Expand proof-trigger paths only when `proofs/BOUNDARY.md` claims implementation or spec paths outside `proofs/`.

### 25.4 Schema and CLI contract hardening

- Continue validating JSON payloads, artifact manifests, diagnostics, source spans, and schema-v1 envelopes at emission boundaries.
- Keep docs/schema drift tests aligned with README and CLI examples.
- Respect schema extension posture; do not make validators narrower than published schemas.
- Keep command-shape, arity, JSON-mode, and diagnostic-context regressions explicit for every newly promoted surface.
- Browser runtime contract validation now trims surrounding whitespace around the canonical host-label fields before comparison, keeping the generated `doctor` payload robust against incidental padding; the public `doctor` payload regression now covers both the trimmed-label path and the whitespace-only rejection path, including explicit `hostLabel` coverage. Thread-topology snapshots are now also validated for ascending, duplicate-free `liveInstances` ordering and coherent `totalInstances = terminatedInstances + liveInstances.length` counts so the JSON observability payload stays deterministic, and worker script URLs now trim surrounding whitespace before parsing so direct topology snapshots stay aligned with the normalized spawn helper.

## Exit gate

- `mise run lean-proofs` passes for proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No README, release, or plan wording exceeds the published boundary.
