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
- Current regression coverage also asserts empty `errors` arrays on successful browser-targeted `check` / `build` and browser-requested `run` / `test` JSON paths for the promise-allSettled smoke slice, the browser for-of / for await iterator smoke slices, the browser Object.is / Object.hasOwn helper slices, the browser Object.fromEntries / Reflect.ownKeys helper slices, the browser Reflect.ownKeys helper slice now also spans JSX and TSX input across the browser-harness and browser-bundle success paths, the Deno env-set / env-delete executable build JSON smoke slices, the standalone object-enumeration / `Object.fromEntries` JSON smoke slices, the browser Math.round JSON smoke slice now also spans TS, JS, JSX, and TSX input, and the browser Math.hypot empty-identity / perfect-square smoke slice now also carries the same empty-errors assertion on the browser-harness JSON path, keeping the success-envelope shape explicit without narrowing extension posture.

## Exit gate

- `mise run lean-proofs` passes for proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No README, release, or plan wording exceeds the published boundary.
