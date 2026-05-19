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
- Browser runtime contract validation now trims surrounding whitespace around the canonical host fields before comparison, keeping the generated `doctor` payload robust against incidental padding; the public `doctor` payload regression now covers both the trimmed-label path and the whitespace-only rejection path, including explicit `hostLabel` coverage. Field-specific regressions now also pin padded `hostDescriptionNote`, `diagnosticHint`, `supportedCommands`, and `diagnosticNotes` values so the browser-runtime contract stays single-sourced and independently trimmed across all canonical string fields and array-valued notes, and duplicate-item regressions now keep whitespace-padded `supportedCommands` / `diagnosticNotes` duplicates on the same canonical failure path. The lower-level browser-runtime contract validator now also rejects whitespace-only `diagnosticHint` values on that same shared path, and the four canonical scalar checks now flow through one shared descriptor loop. The CLI validator now also sources the browser runtime contract descriptor and note list from `kali_runtime::BrowserRuntimeContract`, so the canonical contract text stays single-sourced, and the shared browser-runtime JSON fixture now comes from `kali_runtime::browser_runtime_contract_value()` so the CLI/runtime test payload stays aligned with the runtime descriptor. The runtime contract test now also checks that the shared fixture exactly matches the canonical descriptor values end-to-end. The CLI `doctor` payload now reuses that shared fixture directly as well. The doctor regression now also accepts a whitespace-padded browser harness override string in the `env` source path, keeping the raw override text compatible with the command-splitting parser. Browser runtime summary-fallback coverage now also pins the trimmed `hostContract` / `runtimeBackend` labels across the browser-requested `run` / `test` lanes so the summary parser stays aligned with the canonical labels. Thread-topology snapshots are now also validated for ascending, duplicate-free `liveInstances` ordering and coherent `totalInstances = terminatedInstances + liveInstances.length` counts so the JSON observability payload stays deterministic, and the test payload merger now renumbers appended live-instance ids when aggregating multi-file runs so the emitted `threadTopology` remains schema-valid; worker script URLs now trim surrounding whitespace before parsing so direct topology snapshots stay aligned with the normalized spawn helper, and the validator tests now also pin whitespace-only `scriptUrl` rejection on the same path plus canonical absolute-URL rejection for relative spellings; the runtime host-state test corpus now also pins a whitespace-padded script URL and the absolute-URL guard for relative worker URLs, and the runtime summary parser now rejects whitespace-padded `threadTopology.liveInstances[].scriptUrl` spellings before they can be normalized into a snapshot. The schema-doc drift test now also cross-checks the browser-runtime trimming rules and the canonical threadTopology scriptUrl wording in both `specs/18-schemas.md` and `specs/12-cli.md`, keeping the CLI and schema chapters aligned with the shared validator behavior. The browser harness command parser now also trims surrounding whitespace around the override string before splitting while preserving the raw override text in malformed-override diagnostics, and the runtime unit tests pin that normalized parsing path; the parser regression now also covers quoted harness arguments after the whitespace trim so the normalized split keeps shell-like grouping intact. The `doctor` JSON regression now also checks that a whitespace-padded browser harness override still parses to the trimmed command parts while preserving the raw override string. The browser-runtime `diagnosticHint` is now exact-match validated too, so the doctor contract stays aligned with the canonical browser-targeted command reference. The README command reference now also names the explicit browser-targeted `kali check --api browser` and `kali build --bundle --api browser` spellings, and the drift test tracks those examples. The shared late-process-control prefix helper now also has a dedicated regression, keeping the prefix assembly and zero-probe append path independently checkable. The shared `process.kill(0)` alias inventory now also has a dedicated prefix-free/single-source regression, so the zero-probe union stays isolated from the late-process-control preamble.
- The browser-runtime contract validator now also trims `hostLabel` alongside `hostDescription`, `hostDescriptionNote`, and `diagnosticHint`, keeping the canonical browser-runtime metadata stable under incidental padding. The four canonical scalar checks now run through one shared trimmed-string helper so the comparison path stays single-sourced, and the human `doctor` summary now reads the browser-runtime contract fields from the shared JSON fixture so the printed snapshot stays aligned with the same canonical contract source. The doctor payload regression now also pins trimmed `diagnosticHint` acceptance on the same canonical browser-runtime contract path. The schema-doc drift test now also pins the `BrowserRuntimeContractDoctor` trimming rules alongside the `threadTopology.liveInstances[].scriptUrl` canonical absolute-URL wording, and the `threadTopology.liveInstances[].scriptUrl` validator now also rejects leading/trailing whitespace so the schema-v1 payload stays on the canonical absolute-URL spelling. Empty `supportedCommands` and `diagnosticNotes` arrays now also fail through the browser-runtime contract validator, keeping those contract tuples non-empty as the schema/docs already promise, and the public doctor regression now also pins out-of-order `diagnosticNotes` rejection on the same ordered-note path. The CLI contract prose now also calls out trimmed `supportedCommands` / `diagnosticNotes` items explicitly so the doctor behavior stays aligned with the validator.
- Browser-requested `test` summary parsing now also accepts padded `hostContract` / `runtimeBackend` labels in the browser summary file across the JS, TS, JSX, and TSX browser-summary fixtures, keeping the trimmed label parser aligned with the harness fallback merge path.
- `specs/12-cli.md` and `specs/18-schemas.md` now also say that `diagnosticHint` follows the same trim-on-compare rule as the other browser runtime contract string fields, and the schema-doc drift test now checks that wording stays aligned.
- `kali package-audit --pretty lodash` now has a regression guard for the required `E5508` invalid-usage path when JSON mode is absent, and the `--preview` shim is unit-tested to fail before target validation or registry lookup; when both flags are present without JSON mode, `--pretty` still wins, the registry is never queried, and the rejection text stays preview-free. The README and CLI-spec drift tests now also pin the `kali package-audit --pretty --output json lodash` example so the envelope-only pretty-JSON shape stays documented alongside the existing package-audit contract lines.
- `kali package-effects --pretty browserpkg` now also keeps the pretty-native-JSON path covered under inherited browser resolution without requiring `--output json`, matching the native JSON contract.
- The browser-runtime doctor schema/docs now spell out that `hostDescription` is trimmed like the other canonical host fields, keeping the schema chapter aligned with the existing validator coverage.
- The pretty-JSON doctor regression now also asserts the shared `browserRuntimeContract` fixture in both env-selected and auto-selected quiet output paths, keeping the JSON and helper-backed contract snapshots aligned.
- The human doctor summary now also prints the browser runtime contract from the shared JSON fixture, keeping the text summary and machine payload single-sourced.
- The browser-analysis `effects` wasm-threads rejection matrix now also spans JS, JSX, and TSX input on both the explicit browser-API surface and the inherited browser-analysis path, and the JSON-output matrix now mirrors that same browser-profile gate across JS/TS/JSX/TSX input, keeping the shared source-graph evidence aligned with the browser runtime profile gate.
- The CLI doctor human-output regression now also derives its browser-runtime contract assertions from the shared `kali_runtime::BrowserRuntimeContract` helper, and the CLI human summary now reads the same descriptor directly so the text output stays aligned with the shared contract vocabulary without re-parsing the JSON fixture.

## Exit gate

- `mise run lean-proofs` passes for proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No README, release, or plan wording exceeds the published boundary.
