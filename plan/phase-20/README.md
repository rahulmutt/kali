# Phase 20 — Verification and Machine Contracts

## Goal

Widen proof-backed and machine-contract confidence while keeping claims exact.

## Owning specs

- `specs/16-testing.md`
- `specs/17-verification.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`
- `proofs/BOUNDARY.md`

## Work packets

### 20.1 Proof-boundary hygiene

- Keep `proofs/BOUNDARY.md` as the sole theorem/property inventory.
- Remove duplicate proof inventories from plan files.
- Update boundary text whenever covered Lean paths or theorem claims change.

### 20.2 Model widening

- Widen core semantics, ownership/RC, effects, type-system, and lowering models in small named slices.
- Pair every widened proof-backed claim with mechanized theorem inventory and proof-CI evidence.
- Keep full-language/full-host proof wording out of release claims until those semantics are modeled.

### 20.3 Proof CI triggers

- Use `mise run lean-proofs` for Lean builds.
- Expand proof-trigger paths only when `proofs/BOUNDARY.md` claims implementation or spec paths outside `proofs/`.

### 20.4 Schema and CLI contract hardening

- Continue validating JSON payloads, artifact manifests, diagnostics, source spans, and schema-v1 envelopes at emission boundaries.
- Keep docs/schema drift tests aligned with README and CLI examples.
- Respect schema extension posture; do not make validators narrower than published schemas.
- Implemented: diagnostic JSON now mirrors `span.file` into the top-level `file` convenience field, schema validation rejects any mismatch between the mirror and the canonical span file, diagnostic objects now reject unexpected top-level keys, diagnostic labels and related-info items now reject unexpected extension keys, diagnostic-context objects now reject unexpected extension keys, source-location objects in suggested-fix edits now reject unexpected extension keys, suggested-fix objects now reject unexpected top-level keys, suggested-fix edit locations now require their nested `start.file` / `end.file` mirrors to match the edit file as well, suggested-fix edit ranges now reject reversed start/end ordering and overlapping edits, and the end.file mirror path now has a dedicated regression test; CLI envelopes now reject unexpected top-level keys while validating optional `artifacts` entries and now also reject unexpected keys inside those artifact objects, test coverage payloads now reject unexpected keys at the coverage root plus per-file and summary rows, and the CLI's E5506 runtime-profile / compatibility gates now carry canonical flag/config path notes for `--compat`, `--wasm-threads`, and the related config keys.

## Exit gate

- `mise run lean-proofs` passes for proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No README, release, or plan wording exceeds the published boundary.
