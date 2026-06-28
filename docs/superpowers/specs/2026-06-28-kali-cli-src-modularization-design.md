# kali_cli production `src/` modularization — design (21st in series; kali_cli sub-project 1 of 3)

Date: 2026-06-28
Status: approved
Crate: `kali_cli` (21st crate in the kali workspace modularization series; kali_lir was the 20th at `ccc469424`)
Scope: **sub-project 1 of 3** for kali_cli. This spec covers the production `src/` split only. Sub-projects 2 (integration-test monoliths under `tests/`: `runtime_smoke.rs` 73K, `package_corpus.rs` 15K, …) and 3 (`tests/` directory grouping for the ~370 per-behavior files) are deferred to later specs.

## Goal & invariant

Pure code-motion. Decompose kali_cli's three oversized production files — `src/output.rs` (3,135), `src/build.rs` (5,128), `src/bin/kali.rs` (5,667) — into per-concern modules with **zero behavior change** and a **byte-identical public API**. External consumers (`bin/kali.rs`'s `use kali_cli::{build, output, …}`, and every `tests/` integration test driving the `kali` binary) MUST compile and behave unedited.

Allowed changes only: `mod`/`pub mod` declarations, `pub use` re-exports, `use` relocation, `pub(crate)` widening of previously-private items, and verbatim item-body moves. Do **not** run `cargo fmt` (verbatim moves + `pub(crate)` prefixes push some lines over 100 columns; the repo's `cargo fmt --all --check` gate is already red on baseline, so these are not regressions).

`pub(crate)` is invisible to external consumers, so widening previously-private helpers does not change the public surface. This crate needs more `pub(crate)` widenings than the prior small IR crates (which had 0–1 each), simply because these files have many private helpers that cross new module boundaries — explicitly allowed by the series mandate.

## Baseline (branch base)

`cargo build -p kali_cli` clean (0 warnings); `cargo test -p kali_cli` green except the 2 pre-existing `build_bundles_*` failures in `array_from_bracketed_root_wrappers` (codegen/bundling, unrelated to this refactor — confirm reproduction on the branch base before starting, do not attribute). Capture `cargo test -p kali_cli -- --list` baseline. Record exact branch-base HEAD and test count in the SDD ledger before starting.

## Current shape

- `src/lib.rs` (560 lines): the lib facade. Declares `pub mod build; pub mod init; pub mod output;` (no glob re-export). Holds the CLI definition (`Args`/`Commands`/`BundleFormat`/`CliOutputOptions`/`ColorChoice`/`OutputFormat`/`ApiSurface`) and source-discovery helpers (`discover_source_files`, `discover_test_files`, `is_declaration_only_source_file`, `build::ArtifactMetadata` accessor). Unchanged by this refactor.
- `src/init.rs` (123 lines) + `src/init_tests.rs` (95 lines): already a leaf. **Unchanged.**
- `src/output.rs` (3,135 lines): flat — `CliOutputOptions`, the public envelope/payload validators (`emit_envelope_value`, `validate_envelope_value`, 12 `validate_*_payload_value`, `merge_thread_topology_snapshot_values`, `emit_envelope`, `diagnostic_to_text`, `diagnostic_to_json`, `json_source_path`, `json_string_list`), and ~50 private schema helpers. Heavily cross-cutting: `reject_unexpected_keys` (28 refs), `validate_non_empty_string_value` (12 refs). Co-located `output_tests.rs` (8,509 lines, 229 tests) wired via `#[cfg(test)] #[path] mod tests`; flat, keyed on the public validator fns. One inline `mod tests` (14 tests) targeting `validate_browser_runtime_contract_value`.
- `src/build.rs` (5,128 lines): ~124 fns across ~20 responsibility clusters (A–U). Public surface: `BuildMode`, `ArtifactMetadata`, `DynamicImportTarget`, `LibraryExport`, and ~22 `pub fn`. Heavily interdependent: clusters P/R/S/T/U (export/signature/type-inference collection) are mutually recursive; clusters F/G (eval/Function-ctor rewriting + dynamic-import) are textually interleaved. Co-located `build_tests.rs` (15,535 lines, 845 tests) wired via `#[cfg(test)] #[path] mod tests`; flat, keyed on input semantics (`build_source_file_supports_<feature>_in_<inputClass>_input`), NOT on cluster boundaries.
- `src/bin/kali.rs` (5,667 lines): the binary. `#![allow(...)]`, `use` block (13 imports incl. `kali_cli::{build, init, output, Args, BundleFormat, Commands}`), `fn main` (dispatch, ~286 lines), ~96 private free fns, one `impl BuildResult` (4 methods), one `const`, and an inline `mod tests` (23 tests). All items private today. CLI definition lives in the lib; the binary is purely command implementations. Integration tests under `tests/` drive the compiled `kali` binary as a subprocess (`CARGO_BIN_EXE_kali`); the inline `mod tests` covers the small set of pure-helper logic not reachable via the CLI surface.

### Key structural finding — co-located test files resist re-splitting

`build_tests.rs` (845 tests) and `output_tests.rs` (229 tests) are flat black-box suites keyed on **input semantics** and **public validator fns** respectively — NOT on the production cluster boundaries. Co-locating their tests per-module would mean inventing arbitrary partitions against the grain: high risk, no navigability payoff. **Both stay monolithic and untouched.** They reach the public surface via `use super::*` (build) and `crate::output::{…}` (output, via the test file's own imports), all of which this refactor preserves at the module root. Only the inline `mod tests` blocks that target *specific private helpers* move with their helpers: output's 14 browser-runtime tests move into `browser_runtime.rs`; bin/kali.rs's 23 tests **stay in `kali.rs`** (see Section 3).

## Approach — Hybrid (per-command for the binary, functional tree for the lib modules)

Mirror the two established precedents by file shape:

- `kali_types` precedent (functional directory tree): one big impl split by responsibility into nested per-concern modules. Applied to `build.rs` and `output.rs`, whose clusters are responsibility-based.
- A per-command split for `bin/kali.rs`, whose natural fault line is the command handler (the binary is already a flat list of `*_command` fns dispatched from `main`).

A binary may declare sibling modules (`mod foo;` → `src/bin/foo.rs`), so the bin split is pure code-motion with no structural compromise.

Each of the three files is an independent split, executed in risk order **output → build → bin**: output first (cleanest fault lines, lowest risk, warms up the pattern), build middle (hardest — most interdependence; the `exports/` web), bin last (isolated binary; depends only on the lib public API being stable, which the prior two splits preserve).

## Target layout

### Section 1 — `output.rs` (3,135 → `output/` directory, ~10 modules)

`output.rs` becomes `output/mod.rs` (facade) + sibling modules. `output_tests.rs` is untouched and re-wired from `mod.rs`.

| Module | Cluster | ~Lines | Contents / notes |
|---|---|---|---|
| `mod.rs` | facade | ~20 | `pub use` public surface; declares submods; `#[cfg(test)] #[path="output_tests.rs"] mod tests` |
| `options.rs` | — | ~15 | `CliOutputOptions` + `impl is_json` |
| `envelope.rs` | envelope dispatch | ~200 | `emit_envelope_value`, `validate_envelope_value`, `emit_envelope` (all pub) |
| `payload.rs` | 12 validators | ~420 | the 12 `pub validate_*_payload_value` |
| `thread_topology.rs` | — | ~195 | `merge_thread_topology_snapshot_values` (pub) + `validate_thread_topology_snapshot_value` + `validate_thread_topology_instance_snapshot_value` (priv) |
| `coverage.rs` | — | ~160 | `validate_test_payload_coverage_value` (priv) |
| `schema.rs` | cross-cutting helpers | ~415 | `reject_unexpected_keys` (28 refs) + 13 scalar/array validators (`validate_schema_version_one`, `validate_string_array_value`, `validate_unique_string_array_value`, `validate_non_empty_string_value`, `validate_canonical_non_empty_string_value`, `validate_registry_package_name_value`, `validate_canonical_absolute_url_string_value`, `validate_optional_non_empty_string_value`, `validate_analysis_context_value`, `validate_effect_location_value`, `validate_effect_occurrences_value`, `validate_package_coordinate_value`, `validate_stable_semver_version_value`) → **`pub(crate)`** |
| `browser_runtime.rs` | — | ~385 + tests | `validate_browser_harness_value`, `validate_browser_runtime_contract_value` + supported-commands/notes/trimmed-string helpers; **inline 14-test `mod tests` moves here** |
| `diagnostic.rs` | diagnostic-object shape | ~565 | `validate_diagnostic_value`, `validate_source_span`, `validate_label_value`/`_label_array`, `validate_related_info_*`, `validate_text_edit_value`, `validate_source_location*`, `validate_suggested_fix*`, `validate_diagnostic_context`, `source_location_position`, `is_positive_integer`, `positive_integer_value` |
| `artifacts_timings.rs` | envelope-internal arrays | ~375 | `validate_diagnostic_array`, `sort_diagnostic_array_value`, `diagnostic_sort_key`, `validate_cli_artifacts_array`, `is_canonical_artifact_role`, `artifact_sort_key`, `artifact_role_rank`, `timing_sort_key`, `timing_phase_rank`, `validate_timings_array`, `validate_timing_value` |
| `serialize.rs` | Diagnostic→JSON/text | ~135 | `diagnostic_to_text`/`diagnostic_to_json` (pub) + private `diagnostics_to_json`, `synthetic_span`, `span_to_json`, `byte_offset_to_line_col`, `code_prefix`, `json_source_path`, `json_string_list` (pub) |

**Visibility:** ~14 `pub(crate)` widenings, all on previously-private helpers in `schema.rs` (called by envelope/payload/thread_topology/coverage/browser_runtime/diagnostic/artifacts_timings). Public surface stays `pub`, re-exported from `mod.rs`. Within-cluster privates (e.g. `synthetic_span` in serialize.rs) stay private.

### Section 2 — `build.rs` (5,128 → `build/` directory, ~8 modules + `exports/` sub-dir)

`build.rs` becomes `build/mod.rs` + siblings. `build_tests.rs` (845 tests) is untouched, re-wired from `mod.rs`; it reaches the public surface via `use super::*`, all re-exported.

| Module | Cluster(s) | ~Lines | Contents |
|---|---|---|---|
| `mod.rs` | facade | ~25 | `pub use` public surface; `#[cfg(test)] #[path="build_tests.rs"] mod tests` |
| `compile.rs` | B+C+D+E+J+K+H | ~1500 | `BuildMode`/`BuildOutput`/`CompileOutput`, `check_source_file`, `normalize_compiler_source`/`read_compiler_source_file`, `load_profile_data_file`+`profile_data_hash`, 8 `compile_source_file*` + `compile_source_file_uncached`, `validation_diagnostic`, `validate_hir/mir/lir_program`, `incremental_cache_path`, `project_root_for_source`, `analyze_source_file`, `source_uses_optional_chain_math_pow`, `source_uses_process_env_mutation`, `AnalyzedSource`, `build_source_file`, `build_mode_from_flags`, `source_hash_for_file`, `build_mode_name`, `validate_runtime_profiles` |
| `eval.rs` | F+G | ~790 | `EvalConst`+impl, `rewrite_eval_compat_source`, `rewrite_static_eval_calls`/`rewrite_static_function_constructor_calls`, `is_bare_function_constructor_spelling`, `find_immediate_invocation_end`, `parse_function_constructor_body_snippet`, `find_call_end`, `collect_constant_bindings`, `source_uses_eval_compat`, 8 `matches_*`, `parse_eval_source_snippet`, 6 `parse_constant_*`, `eval_plus`, `parse_template_constant_value`, `unquote_string_literal`, `find_statement_end`, `discover_dynamic_import_targets` (pub) + `parse_static_dynamic_import_specifier`/`find_token_call_end`/`resolve_dynamic_import_target`/`canonicalize_dynamic_import_candidate`, `DynamicImportTarget` |
| `metadata.rs` | L+M+N | ~790 | `ArtifactMetadata`, `build_artifact_metadata`, `validate_generated_artifact_metadata`, `validate_build_result_value` (pub) + `validate_build_result_artifacts_array`/`validate_name_signature_object`/`validate_build_result_exports_array`/`build_result_artifact_sort_key`/`build_result_artifact_role_rank`/`validate_no_unexpected_keys`/`validate_canonical_non_empty_string_field`/`is_canonical_artifact_role`, `serialize_artifact_metadata`, `append_metadata_section` |
| `exports/` | P+R+S+T | ~1520 | the interdependent export/signature/type web — sub-split (mirrors kali_types' nested `resolve/`, `static_analysis/`): |
| ↳ `exports/collect.rs` | P+R | ~1000 | `collect_library_exports` (pub) + `collect_library_exports_from_source_path_with_context`/`_from_statements_with_context`/`_from_statements`, `resolve_library_export_source_path`/`resolve_relative_library_export_source`, `collect_browser_bundle_exports` (pub), `collect_tree_shake_roots`, `collect_reachable_bundle_exports`, 5 `collect_direct_bundle_calls_from_*` |
| ↳ `exports/signatures.rs` | S | ~350 | `collect_declared_function_signatures`/`_binding_signatures`, `infer_function_signature`/`_binding_signature`, `is_object_freeze_call`, `call_member_access_name`, `member_access_name`, `function_signature` |
| ↳ `exports/types.rs` | T | ~170 | `infer_block_return_type`, `infer_static_truthiness`, `infer_expression_type`, `infer_unary_expression_type`, `infer_binary_expression_type`, `is_numeric_like_type`, `infer_literal_type` |
| `helpers.rs` | U+Q1 | ~70 | cross-cutting misc helpers → **`pub(crate)`**: `parse_source_file` (used by P/R/Q2), `source_stem` (→paths), `has_errors` (→compile/analyze/collect), `signature_from_export_specifier` (→P/R), `invalid_export_surface` (→P) |
| `entrypoint.rs` | A+Q2+Q3 | ~930 | `block_contains_yield_delegation`/`statement_`/`expression_`, `generator_function_unavailable_message`, `reject_async_and_generator_class_methods_in_runtime_entrypoint` (pub), `validate_unique_export_names_from_statements` |
| `paths.rs` | I | ~131 | `executable_output_path_for`, `library_output_paths_for`, `bundle_output_paths_for`, `bundle_chunk_output_dir_for`, `capi_output_paths_for`, `binding_package_manifest_output_path_for`, `component_output_paths_for` |
| `wit.rs` | O | ~45 | `library_wit_for` (pub), `browser_bundle_source_map` (pub), `LibraryExport`, `sanitize_wit_identifier` (wit-only — stays **private** within `wit.rs`, no widening) |

**Visibility:** cross-cluster private helpers confirmed shared by grep → `pub(crate)`: `has_errors` (B/E/P), `source_stem` (I), `parse_source_file` (P/R/Q2), `collect_declared_function_signatures`/`infer_function_signature` (S→P/R), `profile_data_hash` (D→L), `build_mode_name`/`source_hash_for_file` (J→B/L), `incremental_cache_path`/`project_root_for_source`/`analyze_source_file` (E→B/C), `rewrite_eval_compat_source`/`source_uses_eval_compat` (F→E), `validate_unique_export_names_from_statements` (Q3→E), `collect_constant_bindings` (F→G). Within `exports/`, the three sub-modules (collect/signatures/types) mutually recurse → their shared items are `pub(crate)` within the `build` crate (e.g. `infer_function_signature` called by collect); the build-level `helpers.rs` holds the cross-cluster misc helpers (`parse_source_file`, `has_errors`, `source_stem`, …) as `pub(crate)`. Public surface (~22 `pub fn` + `BuildMode`/`ArtifactMetadata`/`DynamicImportTarget`/`LibraryExport`) stays `pub`, re-exported from `mod.rs`.

**`exports/` sub-split rationale:** the ~1500-line export/signature/type-inference web is the kali_types-style "one big mutually-recursive impl" problem. Splitting it into collect/signatures/types gives honest per-concern boundaries; the cross-sub-module helpers they share (`parse_source_file`, `infer_function_signature`, `has_errors`, etc.) live in the build-level `helpers.rs` as `pub(crate)`. If the implementer finds the three-way split introduces more churn than clarity at execution time, collapsing to a single `exports.rs` is an acceptable fallback — the cluster boundaries are the target shape, not a frozen contract (same caveat as kali_types' design).

### Section 3 — `bin/kali.rs` (5,667 → per-command modules, ~13 modules)

`kali.rs` keeps `#![allow(...)]`, the `use` block, and `fn main` (dispatch); declares `mod` for each sibling under `src/bin/`. All items are private today → cross-module helpers become `pub(crate)` within the bin (invisible outside the binary; no external consumers — the lib already exposes the CLI surface). The inline 23-test `mod tests` **stays in `kali.rs`**; the ~10 helpers it references across `shared`/`config`/`cmd_package` are `pub(crate)` so `super::shared::…` / `super::config::…` / `super::cmd_package::…` resolve. (Co-locating those 23 tests into per-command modules was considered and rejected: they're narrow helper tests spanning several modules, and keeping them in `kali.rs` with `pub(crate)` targets is the minimal-churn choice.)

| Module | ~Lines | Contents |
|---|---|---|
| `kali.rs` | ~360 | attrs, `use`, `fn main` dispatch, `mod` decls, inline `mod tests` (23) |
| `shared.rs` | ~440 | `print_envelope`, `diagnostics_exit_code`, `emit_native_json_payload`, `emit_diagnostics_and_exit`, `split_and_convert_diagnostics`, `single_diagnostic_to_values`, `load_policy_or_exit`, `ensure_project_ready_or_exit`, `selected_source_files`, `single_or_error`, `validate_runtime_entrypoint`, `reject_workflow_context_flags`, `reject_install_context_flags`, `command_allows_pretty_without_json`, `matches_test_filter` — all `pub(crate)` |
| `config.rs` | ~485 | `resolve_effective_api_surface`/`_compat_features`/`_runtime_profiles`/`_max_specializations`, `config_diagnostic_context`/`_with_value`, `manifest_api_surface`/`manifest_compat_features`/`_runtime_profiles`/`_max_specializations`, `normalize_compat_features`, `reject_unavailable_compat_features`/`_runtime_profiles`/`_browser_runtime`/`_spawned_process_budget`/`_zero_capable_budgets`, `browser_runtime_harness_command_available`, `resolve_profile_data` — `pub(crate)` |
| `cmd_doctor.rs` | ~86 | `doctor_command` |
| `cmd_check.rs` | ~154 | `check_command` |
| `cmd_build.rs` | ~1660 | `build_command`, `BuildArtifactSelection`/`BundleArtifact`/`BrowserBundleBuild`/`BuildResult`+`impl`+`build_result_artifact_sort_key`/`_role_rank`, `build_executable_artifact`/`_library`/`_capi`/`_component`/`_browser_bundle`, `write_browser_bundle_files`, `collect_browser_bundle_chunk_artifacts`, `browser_bundle_dynamic_import_map`, `normalize_dynamic_import_specifier`, `relative_path`, `generate_browser_bundle_js` |
| `cmd_run.rs` | ~260 | `run_command`, `browser_stdout_thread_topology_snapshot_value` |
| `cmd_test.rs` | ~485 | `test_command`, `coverage_function_count_from_wasm`, `normalize_coverage_report_path`, `sort_coverage_reports`, `coverage_percent` |
| `cmd_fmt.rs` | ~98 | `fmt_command` |
| `cmd_lint.rs` | ~105 | `lint_command` |
| `cmd_effects.rs` | ~116 | `effects_command` |
| `cmd_package.rs` | ~1170 | `package_effects_command`, `package_audit_command`, `package_analysis_specific_flag_context`, `reject_package_analysis_specific_flags`, `require_single_registry_package_target`, `PACKAGE_AUDIT_PREVIEW_MESSAGE`, `package_audit_preview_diagnostic`, `sort_package_audit_findings`, `diagnostic_span_sort_key`, `PackageBinEntrypoint`+`validate_package_bin_runtime_entrypoint`/`detect_package_bin_entrypoint`, `package_root_for_node_modules_source`, `collect_unsupported_node_bin_markers`, `source_mentions_identifier`, `ParsedRegistryPackageTarget`+`parse_registry_package_target`/`is_version_suffixed_package_spec`, `find_package_root`, `read_package_version`, `analysis_context_for_api`, `validate_source_effects_against_policy`/`_for_roots` |
| `cmd_install.rs` | ~81 | `install_command`, `reject_install_context_flags` |

**Note on the two largest command modules:** `cmd_build.rs` (~1660) and `cmd_package.rs` (~1170) are single modules in this first pass. Per-command grouping was the chosen strategy. If either feels too large after the split ships green, a later sub-split (e.g. `cmd_build/{artifacts,browser_bundle}.rs`, `cmd_package/{effects,audit,registry}.rs`) is a separate, lower-risk refinement — the same logic as the `exports/` sub-directory in build.rs.

## Visibility summary

- **Public API byte-identical:** `lib.rs` already declares `pub mod build; pub mod init; pub mod output;`. `build.rs` and `output.rs` become directories (`build/mod.rs`, `output/mod.rs`) whose facades re-export the **complete previously-public surface** — every item that is `pub` today stays reachable at `kali_cli::build::*` / `kali_cli::output::*`, including pub items no external caller happens to use (e.g. `BuildOutput`, `CompileOutput`, `build_source_file`). Dropping an unused-but-`pub` item would shrink the public API and is forbidden. (The "externally-used" symbol lists under each section are the *proof* that consumers compile unedited, not an exhaustive re-export list — the facade re-exports the full set.) `bin/kali.rs`'s `use kali_cli::{build, output, Args, BundleFormat, Commands, …}` is unchanged.
- **`pub(crate)`** is applied only to previously-private items that cross new module boundaries. It is invisible to external consumers → public API unchanged. `pub use module::*` globs re-export `pub` items only; `pub(crate)` items are not promoted to `pub` by re-export, so the widening stays crate-internal. More widenings than the prior small crates because these files have many private helpers; explicitly allowed by the series mandate.
- **`bin/kali.rs` items** are all private today; splitting them requires `pub(crate)` within the bin. The lib crate and `tests/` integration tests never see them (they don't need to — the lib exposes the CLI surface). Only the inline `mod tests` needs access, satisfied by `pub(crate)`.
- **Exact per-fn placement:** the module → cluster mapping above is the target shape. Individual function placement follows the cluster boundaries documented in the structural inventory (clusters A–U for build.rs; the public/private validator sets for output.rs; the per-command groups for bin/kali.rs). If a fn straddles two clusters, place it with its primary caller; this is settled at execution time, not a frozen file-by-file contract (same caveat as kali_types' design).

## Test handling

- `build_tests.rs` (845) and `output_tests.rs` (229): **unchanged**, re-wired via `#[cfg(test)] #[path] mod tests` from the new `mod.rs` facades. They reach the public surface via `use super::*` / `crate::output::{…}`, all preserved.
- Output's inline 14-test `mod tests` (browser-runtime-contract): moves into `browser_runtime.rs`.
- Bin's inline 23-test `mod tests`: stays in `kali.rs`; the ~10 helpers it references become `pub(crate)`.
- `init_tests.rs` (95): unchanged (init.rs is a leaf, untouched).
- `tests/` integration dir: **out of scope** for this sub-project (sub-projects 2 & 3).

## Execution & verification rhythm

On a `refactor/kali-cli-src-modularization` branch off main; confirm baseline build+test+`--list` green (modulo the 2 pre-existing `build_bundles_*` failures) before starting. Three tasks in order:

1. **output → `output/` directory** (Section 1). After: `cargo build -p kali_cli` (0 warnings); `cargo test -p kali_cli` (same `--list` count vs baseline, modulo the 2 known failures); `cargo build -p kali_cli --bin kali` (binary compiles — its `use kali_cli::output::{…}` resolves via the facade).
2. **build → `build/` directory** (Section 2). Same verification.
3. **bin → `src/bin/` per-command modules** (Section 3). After: `cargo build -p kali_cli --bin kali` (0 warnings); `cargo test -p kali_cli` (same `--list` count — the inline 23 tests + the 845 build_tests + 229 output_tests all unchanged); spot-run a CLI-subprocess integration test (e.g. `tests/version_smoke.rs`, `tests/doctor.rs`) to confirm binary behavior.

After all three: `cargo build -p kali_cli` 0 warnings; full `cargo test -p kali_cli` matches baseline (`--list` diff empty except the 2 known failures); `--list` diff vs baseline = EMPTY (zero behavior/test change).

Integration is **local-main ff-merge only — NEVER push to origin** (origin/main intentionally lags). Re-verify build+test on merged main, then delete the branch.

## Out of scope (deferred to sub-projects 2 & 3)

- Sub-project 2: split the integration-test monoliths under `tests/` (`runtime_smoke.rs` 73,625 / 1,816 tests; `package_corpus.rs` 15K; `node_api_surface.rs` 4K; `late_compat_browser_*`; `schema_docs.rs` 2.8K). Each big file is its own spec.
- Sub-project 3: subdirectory grouping for the ~370 small per-behavior files under `tests/` (`array/`, `browser/`, …).
