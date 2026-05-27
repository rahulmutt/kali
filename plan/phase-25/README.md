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
- Package-analysis-specific `--api` / `--compat` / `--wasm-threads` / `--sandbox` rejections now carry canonical CLI flag context in JSON output so the single-package registry-analysis contract stays machine-auditable; the broad `runtime_smoke` coverage now also asserts that context on the `package-effects` and `package-audit` JSON rejection paths, including requested/effective values for concrete rejected flags, and padded single-target package rejections now carry normalized requested/effective package-argument context in JSON output.
- Keep docs/schema drift tests aligned with README and CLI examples; the browser runtime contract docs now also spell out trim-on-compare handling for the summary and scope note fields alongside the existing host label/description fields, and the `doctor` smoke now exact-matches the canonical browser diagnostic hint string in JSON output instead of only checking for a substring, including a pretty-JSON integration check for the same contract. The artifact-metadata schema/validator path now also treats `maxSpecializations` as a non-negative integer so generated manifests, bindings, and JSON snapshots stay in step.
- Respect schema extension posture; do not make validators narrower than published schemas.
- Keep command-shape, arity, JSON-mode, and diagnostic-context regressions explicit for every newly promoted surface.
- Package-analysis-specific `--sandbox` rejection is now pinned alongside the existing `--api` / `--compat` / `--wasm-threads` precedence checks for `package-effects` and `package-audit`; the dedicated flag-precedence harness now exercises the package-analysis flag set in both plain and JSON output, including pretty-bearing JSON forms, and the package-corpus smoke now also covers the `--sandbox` JSON rejection envelope with canonical CLI flag context. The sandbox branch now also preserves the requested/effective policy-path value in JSON diagnostics, and padded single-package JSON rejections on `package-effects` / `package-audit` now also carry normalized requested/effective package-argument context. Package-audit preview rejection diagnostics now also preserve the hidden `--preview` CLI flag context in JSON output so the legacy shim stays machine-auditable, and the direct `package_audit_preview` regression now asserts the CLI origin plus requested/effective values in JSON mode, including the pretty-bearing JSON order variant. The preview shim now also preempts package-analysis-specific flag validation when `--sandbox` is present, keeping that legacy path precedence explicit in JSON output; the dedicated preview suite now also covers the same sandbox ordering in pretty JSON mode. The preview path now also short-circuits before package-analysis-specific flag validation for `--api` / `--compat` / `--wasm-threads` in both text and pretty JSON modes.

## Exit gate

- `mise run lean-proofs` passes for proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No README, release, or plan wording exceeds the published boundary.
- Machine-readable outputs remain deterministic and schema-valid.
