# kali_sandbox modularization — design (17th in series)

Date: 2026-06-26
Status: approved
Crate: `kali_sandbox` (17th crate in the kali workspace modularization series; kali_lexer was 16th)

## Goal & invariant

Pure code-motion. Decompose the two monoliths — `src/lib.rs` (937 lines) and `src/effects.rs` (1078 lines) — into a thin facade plus per-concern modules with **zero behavior change** and a **byte-identical public API**. The three external consumers (`kali_cli`, `kali_embed`, `kali_runtime`) MUST compile unedited.

Allowed changes only: `mod` declarations, `use` wiring, and `pub(crate)` visibility widening on items that become cross-module. Item bodies are moved **verbatim**. Do **not** run `cargo fmt` (verbatim moves plus the mandated `pub(crate)` prefix push some signatures over 100 columns and leave stray blank lines; the repo's `cargo fmt --all --check` gate is already red on baseline, so these are not regressions).

## Baseline (branch base)

`cargo test -p kali_sandbox`: **41 passed, 0 failed, 0 ignored, 0 doc-tests, 0 warnings**. Record the exact branch-base HEAD in the SDD ledger before starting.

## Current shape

- `src/lib.rs` (937 lines): the policy data model, host-operation vocabulary, predicate registry, validation, file loading/serialization, operation enforcement, glob/pattern matching, and diagnostic builders — all in one file.
- `src/effects.rs` (1078 lines): already a separate `pub mod effects`, but itself a monolith mixing report data types, source-tree inference/traversal, token-level effect recognizers, and policy comparison.
- `src/tests.rs` (1303 lines, 41 tests): co-located, declared in `lib.rs`, uses `use super::*`. References no internals beyond the re-exported `PatternKind` and the `pub(crate)` method `AccessRule::allows_candidate`.

## Target layout

### `lib.rs` → thin facade + 8 modules

| Module | Contents |
|---|---|
| `policy` | `SandboxPolicy` + `EffectsPolicy`/`FileSystemPolicy`/`NetworkPolicy`/`ProcessPolicy`/`TimerPolicy`/`ResourceLimits`/`AccessRule` type definitions + `default_schema_version`/`default_base_dir` |
| `operation` | `HostOperation`, `PolicyPredicateContext`, `PolicyPredicateContext::from_operation` |
| `predicate` | `PolicyPredicateRegistry`, `RegisteredPredicate`, `HostPredicate`, `Default`/`enabled`/`disabled`/`is_enabled`/`register`/`evaluate` |
| `validation` | `PolicyValidation` + `SandboxPolicy::{validate, validate_with_runtime_profiles, validate_policy}` + `validate_positive_u64`/`validate_zero_capable_u64` |
| `loading` | `SandboxPolicy::{from_file, from_file_with_runtime_profiles, to_canonical_json, to_canonical_json_bytes, to_embedded_json_bytes}` |
| `enforcement` | `SandboxPolicy::{check_operation, check_operation_with_predicates, check_path_access, check_url_access, check_exact_access, effective_thread_budget, effective_spawn_budget, network_max_connections}` |
| `matching` | `AccessRule` impl (`is_enabled`/`allows_path`/`allows_candidate`), `PatternKind`, `resolve_pattern`, `normalize_text`, `glob_match`/`glob_match_inner` |
| `diagnostics` | `sandbox_violation`, `unavailable_capability`, `host_predicate_violation`, `resource_limit_violation` |

### `effects.rs` → `effects/mod.rs` facade + 4 submodules

| Submodule | Contents |
|---|---|
| `effects/report` | report data types (`EffectAnalysisContext`, `EffectLocation`, `EffectOccurrence`, `EffectReport`, `PackageCoordinate`, `PackageEffectsReport`, `ObservedEffect`, `EffectInference`) + `EffectAnalysisContext` impl + `effect_report_from_inference` + `package_effects_report` + `normalize_semantic_axis` + `normalize_entry_points` |
| `effects/inference` | `infer_effects_from_roots`, `visit_source_root`, import resolution (`collect_relative_imports`, `is_relative_specifier`, `resolve_relative_import`, `resolve_with_extensions`, `SOURCE_EXTENSIONS`), `dedupe_effects`, `effect_sort_cmp`, `location_sort_key`, `has_errors` |
| `effects/scan` | `scan_tokens_for_effects`, `observed_effect`, `EffectMatch`, all `is_*`/`read_*` token recognizers, `call_string_argument`, `unquote_token_value` |
| `effects/compare` | `compare_effects_to_policy`, `policy_suggestion`, `effect_allowed`, `rule_allows` |

### Tests

`src/tests.rs` stays co-located, declared in the `lib.rs` facade, with `use super::*` unchanged — all 41 tests preserved verbatim. Splitting tests per-module is **out of scope**: they are cross-cutting and reference no internals beyond the re-exported `PatternKind`.

## Public-surface contract (byte-identical)

- **Crate-root `pub` (unchanged):** `SandboxPolicy`, `EffectsPolicy`, `FileSystemPolicy`, `NetworkPolicy`, `ProcessPolicy`, `TimerPolicy`, `ResourceLimits`, `AccessRule`, `HostOperation`, `PolicyPredicateContext`, `PolicyPredicateRegistry`, `HostPredicate`, `PolicyValidation` — plus every existing `pub` method on them.
- **Facade keeps** `pub mod effects;` **and** the exact 12-symbol re-export: `compare_effects_to_policy`, `effect_report_from_inference`, `infer_effects_from_roots`, `package_effects_report`, `EffectAnalysisContext`, `EffectInference`, `EffectLocation`, `EffectOccurrence`, `EffectReport`, `ObservedEffect`, `PackageCoordinate`, `PackageEffectsReport`.
- **`pub(crate)`:** `PatternKind` and `AccessRule::allows_candidate` — `PatternKind` re-exported at crate root via `pub(crate) use matching::PatternKind;` so `tests.rs`'s `super::*` still resolves it.

## Known cross-module couplings (drive the widening pass)

- `resolve_relative_import` is defined alongside the inference traversal but is **also called by the scanner** (`scan_tokens_for_effects`, current `effects.rs:561`). It must become `pub(crate)` in `effects/inference` and be imported into `effects/scan`.
- `effects/*` submodules use `use crate::{AccessRule, PatternKind, SandboxPolicy};` — the facade must re-export those (the `pub` ones already; `PatternKind` via `pub(crate) use`).
- `lib.rs` `SandboxPolicy` methods and free helpers that become cross-module (e.g. `network_max_connections` used by `validation`, the diagnostic builders used across `enforcement`/`validation`/`predicate`, `validate_positive_u64` family, the `matching` helpers) get a leading `pub(crate)` widening pass (Task 1) before extraction, exactly as in prior crates.
- The report data types in `effects/report` are consumed by `effects/scan`, `effects/inference`, and `effects/compare`; they are already `pub` so no widening is needed — submodules reach them via `use super::report::{...}` (or `use crate::effects::...`).

## Error handling / risk

This is pure code-motion: no new error paths, no behavior change. The integrity guarantee is that bodies are byte-identical, the public `pub`/`pub use` set is unchanged, the consumer diff is empty, the workspace builds with 0 warnings, and all 41 tests pass. Any deviation (e.g. an import added to a test file because removing an unused root import severed a `super::*` source) must be minimal, import-only, and recorded in the SDD ledger as an accepted minor.

## Testing & verification

- After each task: `cargo test -p kali_sandbox` green + 0 warnings.
- Finalize: whole-workspace build 0 warnings; **consumer diff empty** across `kali_cli`/`kali_embed`/`kali_runtime`; API proof confirming the exact `pub` + `pub use` set is unchanged; 41/41 tests pass.
- Pre-existing `kali_cli` integration failures `array_from_bracketed_root_wrappers` / `build_bundles_*` are unrelated codegen/bundling issues — confirm reproduction on the branch base, do not attribute to this refactor.

## Process (series conventions)

- Work on `refactor/kali-sandbox-modularization` branched off main; baseline build+test green before starting.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per crate; durable recovery map.
- Per-task: implementer (sonnet) → review-package → task reviewer (sonnet; opus for the finalize/whole-branch reviews).
- **Do NOT run `cargo fmt`** — accept the cosmetic >100-col and stray-blank-line minors from verbatim moves + `pub(crate)` widening.
- Integration is **local-main ff-merge only — NEVER push to origin** (origin/main intentionally lags). Re-verify on merged main, then delete the branch.

## Proposed task outline (~10 tasks; sequenced in the implementation plan)

1. `pub(crate)` receiver-widening pass on `lib.rs` items that become cross-module.
2. Extract `policy` (data model).
3. Extract `operation`.
4. Extract `predicate`.
5. Extract `diagnostics` + `matching`.
6. Extract `validation`.
7. Extract `loading` + `enforcement`; `lib.rs` becomes the thin facade.
8. effects: create `effects/` dir, convert `effects.rs` → `effects/mod.rs`, extract `report`.
9. effects: extract `scan` + `inference` (resolve shared `resolve_relative_import` via `pub(crate)`).
10. effects: extract `compare`; `effects/mod.rs` becomes the thin facade; finalize + whole-branch review.
