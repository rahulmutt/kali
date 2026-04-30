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
- Progress: schema-document assertions already cover the core result payloads, the native JSON emission path now validates the `effects`, `package-effects`, and `package-audit` payload shapes before printing, the `check`, `run`, and `test` JSON payloads now also validate before emission, and the envelope timing checks now also pin a non-string `phase` field so the timing-item contract stays explicit. The `test --coverage` payload validator now also rejects malformed coverage file entries and incomplete coverage summary objects, and the schema-doc drift net now also pins the coverage container object itself as an open object, keeping the nested coverage contract explicit alongside the top-level JSON-envelope checks. The envelope validator now also walks warning diagnostics explicitly, so malformed warning entries fail alongside malformed errors. The build-result schema now also documents the shared build-artifact item shape and its optional `role` field across the `lib` / `bundle` / `capi` / `component` variants, and the build-result validation tests now also reject non-string artifact roles while positively exercising role-bearing payloads across the `lib` / `bundle` / `capi` / `component` variants; the validator now also rejects unsupported `artifactKind` values before entering the shape-specific branch logic, keeping the artifact-mode contract aligned with the emitted build payloads. Effects payload validation now also rejects unexpected keys at the top level plus nested `analysisContext` and effect-location keys, keeping the schema-v1 `additionalProperties: false` contract explicit in the unit tests too. Package-effects validation now also rejects unexpected keys in both the nested package coordinate and nested report objects, keeping the schema-v1 `additionalProperties: false` contract explicit in the unit tests too. The package-audit payload validator now also rejects non-null payloads, keeping the envelope-only JSON command contract explicit at the validator layer. The direct schema-validation tests now also pin the package-audit null-only contract and the duplicate primary-role build-result guard, keeping the narrow validator-level regressions explicit alongside the existing CLI smoke coverage. The doctor payload validator now also rejects non-string `browserHarness.command` / `browserHarness.args` entries, non-string `browserRuntimeContract.hostLabel` / `browserRuntimeContract.hostDescription` / `browserRuntimeContract.supportedCommands` / `browserRuntimeContract.diagnosticNotes` entries, and the `browserRuntimeContract.hostDescriptionNote` field, and now also rejects empty `browserRuntimeContract.hostDescriptionNote` strings and unsupported `browserRuntimeContract.supportedCommands` labels outside the run/test pair, keeping the browser-harness snapshot contract explicit across both scalar and array-item checks. The phase-6 conformance-dashboard drift test now also pins the browser-requested boolean conjunction / disjunction rows explicitly, keeping that browser-runtime snapshot row from drifting silently out of the supported bucket. The phase-10 README wording is now also pinned by the schema-doc drift net in `crates/kali_cli/tests/schema_docs.rs`, and that drift net now also records the browser-component browser-API-surface contradiction examples, the explicit `kali build --bundle --api browser main.ts` spec example, and the `kali test --api browser` browser-contract example, keeping the hardening summary deterministic.
- Progress: generated artifact metadata, generated C ABI metadata, and binding-package manifests now round-trip through their schema parsers before the helpers return, so the artifact-manifest contract is validated at generation time instead of only when a file is later reloaded from disk. The doctor browser-runtime-contract validator now also rejects duplicate `supportedCommands` and `diagnosticNotes` entries, keeping the browser-harness metadata snapshot deterministic instead of silently accepting repeated labels.
- Progress: the `run` and `test` JSON payload validators now also reject non-string `hostContract` / `runtimeBackend` provenance fields, keeping the machine-readable execution payload contract explicit when those optional fields are present.
- Progress: the `init` JSON payload validator now also rejects non-string `root` / `manifestPath` / `sourcePath` fields and non-boolean `library` values, and the `install` JSON payload validator now also rejects non-string `manifestPath` / `lockPath` values, keeping the scaffold/install machine-contract explicit beyond the happy-path schema shape.
- Progress: the build-result JSON payload now validates its browser-bundle `bundleFormat` field before emission, now rejects unexpected top-level and artifact-entry keys before printing, and the schema-doc drift net now pins that `esm` / `cjs` variant explicitly, keeping the browser bundle contract aligned across code and schema. The build-result schema validation tests now also round-trip the `lib` and `capi` artifact kinds, keeping the remaining artifact-mode variants covered alongside the existing bundle/component cases, and the validator now also rejects non-string `bundleFormat` values on bundle outputs. The build-result artifact-array validator now also rejects duplicate `primary-executable` / `primary-library` / `primary-component` roles, keeping the one-primary-artifact rule explicit in code as well as in the schema docs. The build-result validation tests now also reject fractional `sizeBytes` values, keeping the integer-only build-size contract explicit alongside the other payload-shape checks. The artifact-metadata validator now also rejects non-string `profileDataHash`, non-string `runtimeProfiles` entries, and fractional `maxSpecializations` values in unit coverage, keeping the build-side provenance contract explicit alongside the rest of the manifest validation. The schema-doc drift net now also records the current snapshot's implementation-specific build-result artifact-kind labels as an exact ordered set (`meta-json` plus browser-bundle `chunk-*` labels), keeping the checked-in payload vocabulary explicit without promoting those labels to new canonical kinds. The CLI spec drift test now also pins the explicit `kali build --bundle --api node main.ts`, `kali build --lib --api browser lib.ts`, and `kali build --capi --api browser lib.ts` contradiction examples, keeping the browser-only bundle / non-browser API-surface split visible in the documented command examples.
- Progress: emitted `analysisContext.runtimeProfiles` / `analysisContext.compatFeatures` and artifact-metadata `runtimeProfiles` arrays now validate as deduplicated, lexically sorted set-like lists, keeping the machine-emitted provenance arrays aligned with the deterministic ordering contract.
- Progress: the generated artifact metadata now rejects unexpected top-level keys before serialization, while the `doctor` JSON payload now also validates its browser-harness and browser-runtime-contract blocks before emission, including nested unexpected-key rejection coverage and the non-empty browserHarness.command / browserRuntimeContract.supportedCommands minima, and the `init`, `fmt`, `lint`, and `install` JSON payloads now validate their schema-v1 shapes before printing, keeping the command/result contract explicit at the surface instead of only in schema files. The `fmt`, `lint`, `check`, `run`, and `test` payload validators now also have explicit fractional-count / fractional-runtimeMs rejection coverage, keeping the integer-only machine-contract edges honest across the common JSON result envelopes. The README command reference now also includes `kali doctor --output json` and `kali effects --output json main.ts`, keeping the documented JSON examples aligned with the schema-v1 doctor and native effect payload contracts. The CLI envelope/diagnostic validator now also rejects backwards `SourceSpan` ranges while still allowing zero-length spans, and now also rejects unexpected `SourceSpan` keys, keeping machine-readable span ordering explicit in the same schema-hardening lane.

## Exit gate

- `mise run lean-proofs` passes for any proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No release or README wording exceeds the published proof boundary.
