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
- Implemented: diagnostic JSON now mirrors `span.file` into the top-level `file` convenience field, schema validation rejects any mismatch between the mirror and the canonical span file, diagnostic objects now reject unexpected top-level keys, diagnostic labels and related-info items now reject unexpected extension keys, diagnostic-context objects now reject unexpected extension keys, and the diagnostic-context unexpected-key path now has a dedicated regression test; diagnostic-context `configPath` / `flag` values now also reject empty or whitespace-only strings when present, with dedicated regression coverage; source-location objects in suggested-fix edits now reject unexpected extension keys, and the regression suite now also pins that nested source-location unexpected-key path inside suggested-fix edits, including the `end` location path, plus unexpected-key rejection in nested label and related-info source spans, text-edit objects inside suggested fixes now reject unexpected keys, suggested-fix objects now reject unexpected top-level keys, and suggested-fix edit locations now validate their `file` mirror plus their `line` / `column` fields directly before the range check, keeping the `start.file` / `end.file` mirror and position-order checks explicit; source-location validation now shares one helper across text-edit and source-location mirror checks and now also rejects empty or whitespace-only `file` values on source spans and nested source locations; suggested-fix edit ranges now reject reversed start/end ordering, overlapping edits, and duplicate zero-length insertions at the same file position, and the end.file mirror path now has a dedicated regression test; CLI envelopes now reject unexpected top-level keys while validating optional `artifacts` entries and now also reject unexpected keys inside those artifact objects, duplicate artifact kind/path pairs, noncanonical artifact roles, out-of-order artifact arrays, and negative `exitCode` values, build-result artifact roles now use the same canonical schema-v1 role set and reject noncanonical strings too, build-result artifact arrays now also reject out-of-order entries, and build-result emitters now sort artifact arrays before schema validation; build-result and artifact-metadata export objects now also reject unexpected keys, with dedicated build-result and artifact-metadata export-key regression coverage, build result `profileDataHash` now also rejects empty or whitespace-only strings, and build result `sourceHash` plus artifact-metadata `kaliVersion` / `sourceHash` now also reject empty or whitespace-only strings, keeping the generated sidecar/export metadata fixed-shape; install payloads now reject unexpected top-level keys, timing objects now reject unexpected keys and duplicate phase labels, and the timing unexpected-key path now has a dedicated regression test; the current repository snapshot now also rejects empty or whitespace-only timing phase labels to keep the schema-v1 contract explicit; effects payload `entryPoints` now also reject duplicate root labels to keep logical-root serialization deterministic; effects payload `dynamicReasons` now also enforce canonical deduplication/lexical ordering and the `dynamicEffects=false` empty-list rule, test coverage payloads now reject unexpected keys at the coverage root plus per-file and summary rows, the C-ABI metadata and binding-package sidecars now reject unexpected top-level and nested artifact keys, runtime/result provenance labels now require non-empty, non-whitespace string values when present, and the shared provenance-string helper now also backs the `run` / `test` payload `hostContract` / `runtimeBackend` checks so those fields stay aligned with the same non-empty-string contract; the browserRuntimeContract `hostLabel` now also remains pinned to the canonical `browser-requested` label in doctor JSON, the published doctor schema now also encodes that const plus the non-empty `browserHarness.envVar` / `browserHarness.executable` / `browserRuntimeContract.hostDescription` string constraints, `browserRuntimeContract.hostDescriptionNote` now also rejects empty or whitespace-only strings, `browserRuntimeContract.diagnosticHint` now also rejects empty or whitespace-only strings, and non-string `browserRuntimeContract.diagnosticHint` values now have dedicated regression coverage to keep the browser runtime guidance explicit, the CLI's E5506 runtime-profile / compatibility gates now carry canonical flag/config path notes for `--compat`, `--wasm-threads`, and the related config keys, and the regression suite now also explicitly pins the canonical diagnostic-context origin set (`cli`, `config`, `default`, and `source`) in schema-v1 JSON.

## Exit gate

- `mise run lean-proofs` passes for proof changes.
- `proofs/BOUNDARY.md` names exactly the widened proof-backed scope.
- No README, release, or plan wording exceeds the published boundary.
