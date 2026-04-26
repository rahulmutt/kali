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
- Progress: schema-document assertions now cover the core result payloads for `check`, `run`, `install`, `fmt`, `lint`, `effects`, `build`, `test`, `package-effects`, and `package-audit`, and now also pin the supporting diagnostic, manifest, lockfile, sandbox-policy, and specialized artifact-metadata projection schema documents plus the binding-package manifest schema, keeping the JSON-schema drift net aligned with the current command and config surface. The `effects` and `package-effects` payload schemas are now exact rather than reserved shells, so the native JSON effect-report shapes stay pinned alongside the other result envelopes. The build result schema is now also checked variant-by-variant for the executable, lib, bundle, capi, component, and artifact-only fallbacks, so the artifact-kind contract stays explicit as the build surface widens, and the artifact-only fallback's `artifacts` field is now explicitly pinned as an array. The build result contract now also names the optional provenance extras that the CLI already emits (`profileDataHash`, `witPath`, and `bindingPackagePath`) so the JSON envelope stays aligned with the generated artifact metadata. The test-result schema assertions now also pin the nested function-coverage payload shape (`mode`, `files`, and `summary`) so the coverage contract stays deterministic at the schema-doc layer too. The package-effects regression set now also pins combined `computed-host-access` + `eval` reports to the canonical lexical `dynamicReasons` order, so the native JSON package-analysis contract stays deterministic when multiple dynamic reasons coexist. The package-audit schema contract is now also pinned at the description/title layer, keeping the envelope-only JSON payload wording aligned with the null result schema. The README command reference is now also covered by a drift test, including the `doctor` and `test --coverage` code-block examples, so the documented public CLI surface stays aligned with the current command set. The `specs/12-cli.md` CLI spec example drift net now also pins the canonical `effects` / `package-effects` / `package-audit` command-family examples plus the live browser/node/library embedding markers and the build-sandbox orthogonality examples for `--lib`, `--capi`, `--component`, and browser-targeted `--bundle`, so the normative CLI chapter stays aligned with the checked-in surface. The `kali doctor` JSON payload schema is now also pinned in the schema-doc drift net, runtime-smoke coverage exercises the `doctor` JSON envelope with a deterministic browser-harness override, and the dedicated doctor corpus now also covers the auto-detected browser-harness baseline in JSON; `doctor` now also stays machine-readable under `--quiet` and `--pretty` together, and the payload now also carries the declarative browser runtime contract snapshot, keeping the environment/debug contract aligned with the rest of the machine-readable CLI surface. The browser bundle console-assert JS-input smoke now also has JSON-output coverage, so the browser bundle envelope path stays pinned alongside the existing execution harness. The package-corpus matrix drift test now also pins representative browser runtime corpus rows, the browser condition-preference slice, the default standalone `semver` `.js` slice, the default standalone package-content `.js` test row, and representative Node/Deno rows so the phase-8 corpus snapshot stays deterministic as that evidence set grows.

## Exit gate

- `mise run lean-proofs` passes for any proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No release or README wording exceeds the published proof boundary.
