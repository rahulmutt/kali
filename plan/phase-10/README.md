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
- Progress: schema-document assertions already cover the core result payloads, the native JSON emission path now validates the `effects`, `package-effects`, and `package-audit` payload shapes before printing, the `check`, `run`, and `test` JSON payloads now also validate before emission, and the envelope timing checks now also pin a non-string `phase` field so the timing-item contract stays explicit. The build-result schema now also documents the shared build-artifact item shape and its optional `role` field across the `lib` / `bundle` / `capi` / `component` variants, and the build-result validation tests now also reject non-string artifact roles while positively exercising role-bearing payloads across the `lib` / `bundle` / `capi` / `component` variants, keeping the artifact-mode contract aligned with the emitted build payloads. The phase-6 conformance-dashboard drift test now also pins the browser-requested boolean conjunction / disjunction rows explicitly, keeping that browser-runtime snapshot row from drifting silently out of the supported bucket.
- Progress: generated artifact metadata, generated C ABI metadata, and binding-package manifests now round-trip through their schema parsers before the helpers return, so the artifact-manifest contract is validated at generation time instead of only when a file is later reloaded from disk.
- Progress: the `run` and `test` JSON payload validators now also reject non-string `hostContract` / `runtimeBackend` provenance fields, keeping the machine-readable execution payload contract explicit when those optional fields are present.
- Progress: the build-result JSON payload now validates its browser-bundle `bundleFormat` field before emission, now rejects unexpected top-level and artifact-entry keys before printing, and the schema-doc drift net now pins that `esm` / `cjs` variant explicitly, keeping the browser bundle contract aligned across code and schema. The build-result schema validation tests now also round-trip the `lib` and `capi` artifact kinds, keeping the remaining artifact-mode variants covered alongside the existing bundle/component cases.
- Progress: the generated artifact metadata now rejects unexpected top-level keys before serialization, while the `doctor` JSON payload now also validates its browser-harness and browser-runtime-contract blocks before emission, including nested unexpected-key rejection coverage, and the `init`, `fmt`, `lint`, and `install` JSON payloads now validate their schema-v1 shapes before printing, keeping the command/result contract explicit at the surface instead of only in schema files.

## Exit gate

- `mise run lean-proofs` passes for any proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No release or README wording exceeds the published proof boundary.
