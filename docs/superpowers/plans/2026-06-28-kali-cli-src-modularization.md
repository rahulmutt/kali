# kali_cli production `src/` modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_cli's three oversized production files (`output.rs` 3,135; `build.rs` 5,128; `bin/kali.rs` 5,667) into per-concern modules with zero behavior change and a byte-identical public API.

**Architecture:** Pure code-motion. `output.rs` → `output/` functional tree (~10 modules); `build.rs` → `build/` functional tree (~9 modules incl. an `exports/` sub-dir); `bin/kali.rs` → per-command sibling modules under `src/bin/` (~13 modules). Verbatim item-body moves + `mod`/`pub use` wiring + `pub(crate)` widening of previously-private items that cross new module boundaries. Co-located `build_tests.rs` (845) and `output_tests.rs` (229) stay monolithic and untouched. Executed in risk order output → build → bin.

**Tech Stack:** Rust (edition 2021), cargo workspace. Crate `kali_cli` at `crates/kali_cli/`. Binary `kali` at `src/bin/kali.rs` (`[[bin]]` in `Cargo.toml`).

## Global Constraints

(Copied verbatim from the spec `docs/superpowers/specs/2026-06-28-kali-cli-src-modularization-design.md`. Every task's requirements implicitly include these.)

- **Pure code-motion, zero behavior change.** Same set of tests exists and passes before and after. `cargo test -p kali_cli -- --list` diff vs baseline MUST be empty (except the 2 pre-existing `build_bundles_*` failures in `array_from_bracketed_root_wrappers`, which are baseline-red — confirm on branch base, never attribute to the refactor).
- **Byte-identical public API.** Every item that is `pub` today stays reachable at `kali_cli::build::*` / `kali_cli::output::*`, including pub items no external caller happens to use (e.g. `BuildOutput`, `CompileOutput`, `build_source_file`). Dropping an unused-but-pub item is forbidden.
- **Allowed changes only:** `mod`/`pub mod` declarations, `pub use`/`pub(crate) use` re-exports, `use` relocation, `pub(crate)` widening of previously-private items, verbatim item-body moves. No rewriting of bodies.
- **No `cargo fmt`.** Verbatim moves + `pub(crate)` prefixes push some lines >100 cols; repo `cargo fmt --all --check` is already red on baseline — not a regression.
- **0 warnings gate.** Every task ends with `cargo build -p kali_cli` (and `cargo build -p kali_cli --bin kali` where the binary is touched) at 0 warnings. Resolve `unused_imports` by trimming `use` lines (the recurring "lowerer `use crate::{...}` over-imports the node type" gotcha from the series — omit unused types from wiring `use` lines; match `kali_mir/src/lower.rs:7` precedent).
- **`use super::*` cutoff gotcha (build only):** `build_tests.rs` uses `use super::*`; after `build.rs` → `build/mod.rs`, `super` = `mod.rs`, so the tests only see mod.rs's re-exports. The 4 private build.rs items the tests reach directly MUST be widened to `pub(crate)` and re-exported from `build/mod.rs`: `profile_data_hash`, `incremental_cache_path`, `collect_library_exports_from_statements`, `collect_direct_bundle_calls_from_statements` (plus already-`pub(crate)` `validate_artifact_metadata_value`, just re-exported). `output_tests.rs` uses explicit `crate::output::{…}` imports — immune; unchanged.
- **Verbatim-move convention:** when a step says "Move verbatim from `<file>:<lines>`", copy those line ranges byte-for-byte (item bodies unchanged) into the new file. The only edits to moved text are the visibility prefix (`pub(crate) `) where this plan explicitly says "widen to `pub(crate)`". Line numbers refer to the **branch-base** file; re-confirm ranges by item name if they have shifted.
- **Integration:** local-main ff-merge only — **NEVER push to origin** (origin/main intentionally lags). Re-verify build+test on merged main, then delete the branch.
- **SDD ledger:** overwrite `.superpowers/sdd/progress.md` per task (git-ignored scratch) — it's the durable recovery map.

## File Structure

```
crates/kali_cli/src/
  lib.rs                 UNCHANGED (facade: pub mod build/init/output + CLI def + discovery)
  init.rs                UNCHANGED (leaf, 123 lines)
  init_tests.rs          UNCHANGED
  output.rs              DELETED → becomes output/mod.rs
  output/
    mod.rs               facade: mod decls + pub use (public surface) + pub(crate) use (sorted_string_array) + #[path] output_tests.rs
    options.rs           CliOutputOptions + impl is_json
    serialize.rs         Diagnostic→JSON/text (diagnostic_to_text/json, json_source_path/string_list, +private helpers)
    schema.rs            cross-cutting scalar/array validators (pub(crate)) incl. validate_sorted_string_array_value
    artifacts_timings.rs envelope-internal array validators + sort keys
    diagnostic.rs        diagnostic-object-shape validators (source-span/label/related-info/text-edit/suggested-fix/source-location)
    browser_runtime.rs   browser-harness + runtime-contract validators + inline 14-test mod tests
    coverage.rs          validate_test_payload_coverage_value
    thread_topology.rs   merge_thread_topology_snapshot_values + 2 validators
    payload.rs           12 pub validate_*_payload_value
    envelope.rs          emit_envelope_value, validate_envelope_value, emit_envelope
  output_tests.rs        UNCHANGED (re-wired via #[path] from output/mod.rs)
  build.rs               DELETED → becomes build/mod.rs
  build/
    mod.rs               facade: mod decls + pub use (public surface) + pub(crate) use (cutoff items) + #[path] build_tests.rs
    paths.rs             7 *_output_path*_for / *_output_dir_for
    wit.rs               library_wit_for, browser_bundle_source_map, LibraryExport, sanitize_wit_identifier (private)
    helpers.rs           pub(crate): parse_source_file, source_stem, has_errors, signature_from_export_specifier, invalid_export_surface
    compile.rs           BuildMode/BuildOutput/CompileOutput, check_source_file, compile_source_file* (8)+uncached, validate_hir/mir/lir_program, incremental_cache_path, project_root_for_source, analyze_source_file, AnalyzedSource, build_source_file, build_mode_from_flags, source_hash_for_file, build_mode_name, validate_runtime_profiles, load_profile_data_file, profile_data_hash, normalize/read_compiler_source
    eval.rs              EvalConst+impl, eval/Function-ctor rewriting, const-eval parse_constant_*, discover_dynamic_import_targets + dyn-import helpers, DynamicImportTarget
    metadata.rs          ArtifactMetadata, build_artifact_metadata, validate_build_result_value + helpers, validate_artifact_metadata_value (pub(crate)), serialize/append_metadata_section
    entrypoint.rs        yield-delegation detection (3 fns + msg), reject_async_and_generator_class_methods_in_runtime_entrypoint, validate_unique_export_names_from_statements
    exports/
      mod.rs             pub use collect/signatures/types::* (re-export for build/mod.rs)
      collect.rs         collect_library_exports (pub) + _with_context/_from_statements/resolve_*, collect_browser_bundle_exports, tree-shake/bundle-call collectors (incl. cutoff items pub(crate))
      signatures.rs      collect_declared_function_signatures/_binding, infer_function_signature/_binding, is_object_freeze_call, *_member_access_name, function_signature
      types.rs           infer_block_return_type, infer_static_truthiness, infer_expression_type, infer_unary/binary_expression_type, is_numeric_like_type, infer_literal_type
  build_tests.rs         UNCHANGED (re-wired via #[path] from build/mod.rs)
  bin/
    kali.rs              attrs + use + fn main (dispatch) + mod decls + inline mod tests (23)
    shared.rs            pub(crate): print_envelope, diagnostics_exit_code, emit_native_json_payload, emit_diagnostics_and_exit, split_and_convert_diagnostics, single_diagnostic_to_values, load_policy_or_exit, ensure_project_ready_or_exit, selected_source_files, single_or_error, validate_runtime_entrypoint, reject_workflow_context_flags, reject_install_context_flags, command_allows_pretty_without_json, matches_test_filter
    config.rs            pub(crate): resolve_effective_*, config_diagnostic_context*, manifest_*, normalize_compat_features, reject_unavailable_*, browser_runtime_harness_command_available, resolve_profile_data
    cmd_doctor.rs        doctor_command
    cmd_check.rs         check_command
    cmd_build.rs         build_command + BuildResult/artifacts/browser-bundle generation
    cmd_run.rs           run_command + browser_stdout_thread_topology_snapshot_value
    cmd_test.rs          test_command + coverage_* helpers
    cmd_fmt.rs           fmt_command
    cmd_lint.rs          lint_command
    cmd_effects.rs       effects_command
    cmd_package.rs       package_effects_command + package_audit_command + registry-target/bin-entrypoint families
    cmd_install.rs       install_command + reject_install_context_flags
```

## Verification commands (used in every task)

```bash
# 0 warnings
cargo build -p kali_cli 2>&1 | tee /tmp/kali_cli_build.txt
# binary compiles (touch when bin/ changed)
cargo build -p kali_cli --bin kali 2>&1 | tee /tmp/kali_bin_build.txt
# tests: same set as baseline
cargo test -p kali_cli -- --list 2>/dev/null | sort > /tmp/kali_cli_tests_now.txt
diff /tmp/kali_cli_baseline_tests.txt /tmp/kali_cli_tests_now.txt && echo "TEST LIST UNCHANGED"
# run the suite (baseline-red build_bundles_* failures are expected)
cargo test -p kali_cli 2>&1 | tee /tmp/kali_cli_test.txt
```

---

### Task 0: Baseline & branch setup

**Files:**
- Modify: `.superpowers/sdd/progress.md` (git-ignored scratch — overwrite)

**Interfaces:** Produces the baseline test-list snapshot + branch that all later tasks build on.

- [ ] **Step 1: Create the refactor branch off main**

```bash
cd /workspace
git checkout main
git checkout -b refactor/kali-cli-src-modularization
git rev-parse HEAD   # record as branch-base in the ledger
```

- [ ] **Step 2: Capture the baseline test list + confirm pre-existing failures**

```bash
cargo test -p kali_cli -- --list 2>/dev/null | sort > /tmp/kali_cli_baseline_tests.txt
wc -l /tmp/kali_cli_baseline_tests.txt   # record count
cargo build -p kali_cli 2>&1 | tee /tmp/kali_cli_baseline_build.txt   # expect 0 warnings
cargo test -p kali_cli 2>&1 | tee /tmp/kali_cli_baseline_test.txt
```

Expected: build clean (0 warnings); the test run has exactly 2 failures in `array_from_bracketed_root_wrappers` (`build_bundles_*`) — these are pre-existing baseline-red (codegen/bundling). Confirm their names in the output. If a DIFFERENT failure appears, STOP — the baseline is not what the spec assumes; reconcile before proceeding.

- [ ] **Step 3: Write the SDD ledger**

Overwrite `.superpowers/sdd/progress.md` with: branch name, branch-base HEAD, baseline test count, the 2 known baseline-red failures, and an empty task list (Tasks 1–10 pending). This file is git-ignored scratch.

- [ ] **Step 4: Commit nothing** (no source changes; ledger is git-ignored). Proceed to Task 1.

---

### Task 1: output — scaffold `output/` + extract `serialize.rs`, `options.rs`, `schema.rs`

**Files:**
- Create: `crates/kali_cli/src/output/mod.rs`, `output/options.rs`, `output/serialize.rs`, `output/schema.rs`
- Delete: `crates/kali_cli/src/output.rs` (its remaining content moves into `output/mod.rs` temporarily — see Step 2)

**Interfaces:**
- Consumes: the original `output.rs` (3,135 lines). Branch-base imports header: `use kali_error::Diagnostic; use semver::Version; use serde_json::{json, Map, Value}; use std::{collections::{BTreeSet, HashSet}, path::Path}; use url::Url; use crate::{ColorChoice, OutputFormat};`
- Produces: `output::schema::validate_sorted_string_array_value` re-exported `pub(crate)` at `crate::output::validate_sorted_string_array_value` (build.rs depends on this path).

- [ ] **Step 1: Create `output/options.rs`**

Move verbatim from `output.rs:13-25` (`pub struct CliOutputOptions` + `impl CliOutputOptions { pub fn is_json ... }`).
Add header `//! CLI output options bag.` and the `use` lines its fields reference (determine via the 0-warning gate; candidates from the original header: `crate::{ColorChoice, OutputFormat}`).

- [ ] **Step 2: Create `output/serialize.rs`**

Move verbatim from `output.rs:2766-2900`: `diagnostic_to_text` (pub), `diagnostics_to_json` (priv), `diagnostic_to_json` (pub), `synthetic_span` (priv), `span_to_json` (priv), `byte_offset_to_line_col` (priv), `code_prefix` (priv), `json_source_path` (pub), `json_string_list` (pub).
Header `//! Diagnostic → JSON/text serialization.` Add `use` lines (candidates: `kali_error::Diagnostic`, `serde_json::{json, Value}`, `std::path::Path`, `crate::{ColorChoice, OutputFormat}` — keep only what the compiler requires). Private helpers stay private.

- [ ] **Step 3: Create `output/schema.rs`**

Move verbatim from `output.rs:961-1374`: `reject_unexpected_keys`, `validate_schema_version_one`, `validate_string_array_value`, `validate_unique_string_array_value`, `validate_non_empty_string_value`, `validate_canonical_non_empty_string_value`, `validate_registry_package_name_value`, `validate_canonical_absolute_url_string_value`, `validate_optional_non_empty_string_value`, `validate_analysis_context_value`, `validate_effect_location_value`, `validate_effect_occurrences_value`, `validate_package_coordinate_value`, `validate_stable_semver_version_value`, AND `validate_sorted_string_array_value` (output.rs:1135, already `pub(crate)`).
**Widen to `pub(crate)`** every one of these (they are private today except `validate_sorted_string_array_value` which is already `pub(crate)`). Header `//! Cross-cutting envelope/payload schema validators (crate-internal).` Add `use` lines (candidates: `serde_json::{Map, Value}`, `semver::Version`, `url::Url`, `std::collections::{BTreeSet, HashSet}`, `std::path::Path`).

- [ ] **Step 4: Convert `output.rs` → `output/mod.rs` (intermediate facade)**

`git mv crates/kali_cli/src/output.rs crates/kali_cli/src/output/mod.rs` — but first ensure the directory exists. Concretely:
```bash
mkdir -p crates/kali_cli/src/output
git mv crates/kali_cli/src/output.rs crates/kali_cli/src/output/mod.rs
```
In `output/mod.rs`: **delete** the items you just moved (options, serialize, schema ranges). Add at the top of `mod.rs`:
```rust
mod options;
mod schema;
mod serialize;

pub use options::CliOutputOptions;
pub use serialize::{diagnostic_to_json, diagnostic_to_text, json_source_path, json_string_list};
pub(crate) use schema::*;   // keeps crate::output::validate_sorted_string_array_value resolving for build.rs
```
Keep the `#[cfg(test)] #[path = "../output_tests.rs"] mod tests;` wiring — note the path is now `../output_tests.rs` since mod.rs is one dir deeper. (If `output_tests.rs` is at `src/output_tests.rs`, the path from `src/output/mod.rs` is `../output_tests.rs`.) The remaining (not-yet-extracted) items stay in `mod.rs` for now; they call schema helpers via the `pub(crate) use schema::*` re-export (no extra `use` needed since the re-export puts them in scope at the module root).

- [ ] **Step 5: Build + verify**

```bash
cargo build -p kali_cli 2>&1 | tee /tmp/o1_build.txt
cargo build -p kali_cli --bin kali 2>&1 | tee /tmp/o1_bin.txt
cargo test -p kali_cli -- --list 2>/dev/null | sort > /tmp/o1_list.txt
diff /tmp/kali_cli_baseline_tests.txt /tmp/o1_list.txt && echo "TEST LIST UNCHANGED"
cargo test -p kali_cli 2>&1 | tee /tmp/o1_test.txt
```
Expected: 0 warnings; test list diff EMPTY; only the 2 known baseline-red failures. If `build.rs` errors on `output::validate_sorted_string_array_value` not found, the `pub(crate) use schema::*;` line is missing — add it.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/src/output/
git commit -m "refactor(kali_cli): extract output serialize/options/schema modules [refactor]"
```

- [ ] **Step 7: Update SDD ledger** — mark Task 1 done with the commit SHA.

---

### Task 2: output — extract `artifacts_timings.rs`, `diagnostic.rs`, `browser_runtime.rs` (+tests), `coverage.rs`, `thread_topology.rs`

**Files:**
- Create: `crates/kali_cli/src/output/{artifacts_timings,diagnostic,browser_runtime,coverage,thread_topology}.rs`
- Modify: `crates/kali_cli/src/output/mod.rs` (remove moved items, add mod/pub use)

**Interfaces:**
- Consumes: `output::schema::*` (pub(crate) helpers from Task 1).
- Produces: these modules' validators are called by `payload.rs`/`envelope.rs` (Task 3). Public item to preserve: `merge_thread_topology_snapshot_values` (pub).

- [ ] **Step 1: Create `output/artifacts_timings.rs`**

Move verbatim from `output.rs:1784-2159`: `validate_diagnostic_array`, `sort_diagnostic_array_value`, `diagnostic_sort_key`, `validate_cli_artifacts_array`, `is_canonical_artifact_role`, `artifact_sort_key`, `artifact_role_rank`, `timing_sort_key`, `timing_phase_rank`, `validate_timings_array`, `validate_timing_value`. All private today → **widen to `pub(crate)`** (payload/envelope call them). Header + `use` lines via 0-warning gate (schema helpers resolve via `use super::schema::*;` or the crate re-export — use `use super::schema::*;`).

- [ ] **Step 2: Create `output/diagnostic.rs`**

Move verbatim from `output.rs:2160-2724`: `validate_diagnostic_value`, `is_positive_integer`, `positive_integer_value`, `validate_source_span`, `validate_label_value`, `validate_diagnostic_label_array`, `validate_related_info_value`, `validate_related_info_array`, `validate_text_edit_value`, `validate_source_location_file_mirror`, `validate_source_location`, `validate_text_edit_location_order`, `validate_suggested_fix_edits_non_overlapping`, `source_location_position`, `validate_suggested_fix`, `validate_diagnostic_context`. All private → **widen to `pub(crate)`**. Header + `use` (incl. `use super::schema::*;`).

- [ ] **Step 3: Create `output/browser_runtime.rs` (with its inline tests)**

Move verbatim from `output.rs:1398-1783`: `validate_browser_harness_value`, `trimmed_string_matches`, `validate_trimmed_string_field`, `validate_browser_runtime_supported_commands_value`, `browser_runtime_supported_commands_message`, `browser_runtime_contract_notes_message`, `validate_browser_runtime_diagnostic_notes_value`, `validate_browser_runtime_contract_value`. All private → **widen to `pub(crate)`**. ALSO move the inline `mod tests` block from `output.rs:2902-3135` (14 tests for `validate_browser_runtime_contract_value`) verbatim into `browser_runtime.rs` (it stays `#[cfg(test)] mod tests`). Header + `use`.

- [ ] **Step 4: Create `output/coverage.rs`**

Move verbatim from `output.rs:803-960`: `validate_test_payload_coverage_value`. Private → **widen to `pub(crate)`**. Header + `use` (incl. `use super::schema::*;`, `use super::diagnostic::*;` if it calls diagnostic validators).

- [ ] **Step 5: Create `output/thread_topology.rs`**

Move verbatim from `output.rs:609-802`: `merge_thread_topology_snapshot_values` (keep **`pub`**), `validate_thread_topology_snapshot_value` (priv), `validate_thread_topology_instance_snapshot_value` (priv). The two private validators → **widen to `pub(crate)`** (called by payload/envelope). Header + `use`.

- [ ] **Step 6: Update `output/mod.rs`**

Delete the moved ranges from `mod.rs`. Add:
```rust
mod artifacts_timings;
mod browser_runtime;
mod coverage;
mod diagnostic;
mod thread_topology;

pub use thread_topology::merge_thread_topology_snapshot_values;
pub(crate) use artifacts_timings::*;
pub(crate) use browser_runtime::*;
pub(crate) use coverage::*;
pub(crate) use diagnostic::*;
pub(crate) use thread_topology::*;
```
(Remove the now-empty inline `mod tests` from mod.rs since browser_runtime's tests moved with it. Keep `output_tests.rs` `#[path]` wiring.)

- [ ] **Step 7: Build + verify** — run the Verification commands block; expect 0 warnings, test-list diff EMPTY, only the 2 known failures.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_cli/src/output/
git commit -m "refactor(kali_cli): extract output validator cluster modules [refactor]"
```

- [ ] **Step 9: Update SDD ledger.**

---

### Task 3: output — extract `payload.rs`, `envelope.rs`; finalize `mod.rs` facade

**Files:**
- Create: `crates/kali_cli/src/output/{payload,envelope}.rs`
- Modify: `crates/kali_cli/src/output/mod.rs` (now a thin facade)

**Interfaces:**
- Produces: the complete `kali_cli::output::*` public surface, byte-identical to baseline. After this task `output/mod.rs` is a thin facade; all production code lives in submodules; `output_tests.rs` is untouched.

- [ ] **Step 1: Create `output/payload.rs`**

Move verbatim the 12 `pub fn validate_*_payload_value` from `output.rs`: doctor (192-211), effects (213-284), package_effects (285-361), init (362-394), fmt (395-422), lint (423-450), install (451-498), package_audit (499-506), check (507-534), run (535-574), test (575-608). Keep all **`pub`**. Header `//! Per-command payload validators (public surface).` Add `use` lines (schema/validator helpers resolve via `use super::schema::*; use super::diagnostic::*; use super::artifacts_timings::*; use super::browser_runtime::*; use super::coverage::*; use super::thread_topology::*;` — keep only what the compiler requires; trim unused per the 0-warning gate).

- [ ] **Step 2: Create `output/envelope.rs`**

Move verbatim from `output.rs`: `emit_envelope_value` (28-68), `validate_envelope_value` (70-191), `emit_envelope` (2725-2764). Keep all **`pub`**. Header `//! CLI envelope construction + shape validation (public surface).` Add `use` (serialize via `use super::serialize::{diagnostic_to_json, ...};`, schema/validators via `use super::*;` style — trim to what's used).

- [ ] **Step 3: Finalize `output/mod.rs` as a thin facade**

Delete the moved ranges from `mod.rs`. Final `mod.rs`:
```rust
//! CLI output envelope: construction, per-payload validation, and serialization.

mod artifacts_timings;
mod browser_runtime;
mod coverage;
mod diagnostic;
mod envelope;
mod options;
mod payload;
mod schema;
mod serialize;
mod thread_topology;

pub use envelope::{emit_envelope, emit_envelope_value, validate_envelope_value};
pub use options::CliOutputOptions;
pub use payload::{
    validate_check_payload_value, validate_doctor_payload_value, validate_effects_payload_value,
    validate_fmt_payload_value, validate_init_payload_value, validate_install_payload_value,
    validate_lint_payload_value, validate_package_audit_payload_value,
    validate_package_effects_payload_value, validate_run_payload_value, validate_test_payload_value,
};
pub use serialize::{diagnostic_to_json, diagnostic_to_text, json_source_path, json_string_list};
pub use thread_topology::merge_thread_topology_snapshot_values;

// Crate-internal re-exports (build.rs reaches validate_sorted_string_array_value via this path).
pub(crate) use schema::*;

#[cfg(test)]
#[path = "../output_tests.rs"]
mod tests;
```
(`mod.rs` should now contain NO production item bodies — only the facade above. Verify with `grep -nE '^\s*(fn|struct|enum|impl)\s' crates/kali_cli/src/output/mod.rs` → no hits.)

- [ ] **Step 4: Build + verify** — run the Verification commands block. CRITICAL: test-list diff vs baseline MUST be empty. `cargo build -p kali_cli --bin kali` must succeed (its `use kali_cli::output::{…}` resolves via the facade).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/src/output/
git commit -m "refactor(kali_cli): extract output payload/envelope; thin mod.rs facade [refactor]"
```

- [ ] **Step 6: Update SDD ledger** — Phase A (output) complete.

---

### Task 4: build — scaffold `build/` + extract `paths.rs`, `wit.rs`, `helpers.rs`

**Files:**
- Create: `crates/kali_cli/src/build/{mod.rs,paths.rs,wit.rs,helpers.rs}`
- Delete: `crates/kali_cli/src/build.rs` (→ `build/mod.rs`)

**Interfaces:**
- Consumes: original `build.rs` (5,128 lines). Branch-base imports header: `use sha2::{Digest, Sha256}; use std::borrow::Cow; use std::collections::{BTreeMap, BTreeSet}; use std::fs; use std::path::{Path, PathBuf}; use kali_ast::{...}; use kali_codegen::{lower_lir_to_wasm, CodegenCtx, TargetConfig}; use kali_common::{...}; use kali_error::{_error_codes::{e5,e8}, Diagnostic, DiagnosticContext, DiagnosticContextOrigin}; use kali_hir::HirLowerer; use kali_lexer::{Lexer, Token, TokenType}; use kali_lir::LirLowerer; use kali_mir::MirLowerer; use kali_optimize::{OptimizationLevel, Optimizer, ProfileData, PROFILE_DATA_VERSION}; use kali_parser::Parser; use kali_runtime::{normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract}; use kali_sandbox::SandboxPolicy; use kali_types::TypeContext; use serde::Serialize; use serde_json::{json, Value}; use wasm_encoder::{CustomSection, Section}; use crate::{is_declaration_only_source_file, output::validate_sorted_string_array_value, ApiSurface, BundleFormat};`
- Produces: `build::helpers::*` (pub(crate)) used by compile/exports/entrypoint; `build::paths::*` and `build::wit::*` (pub) used by bin/kali.rs.

- [ ] **Step 1: Create `build/paths.rs`**

Move verbatim from `build.rs:1886-2016`: `executable_output_path_for`, `library_output_paths_for`, `bundle_output_paths_for`, `bundle_chunk_output_dir_for`, `capi_output_paths_for`, `binding_package_manifest_output_path_for`, `component_output_paths_for` (all pub). Header `//! Output path computation for build artifacts.` Add `use` (candidates: `std::path::{Path, PathBuf}`, `crate::BundleFormat`).

- [ ] **Step 2: Create `build/wit.rs`**

Move verbatim: `library_wit_for` (pub, build.rs:2900-2913), `browser_bundle_source_map` (pub, 2871-2899), `LibraryExport` (pub struct, 394-399), `sanitize_wit_identifier` (private, 5101-5121 — keep private within wit.rs; only `library_wit_for` calls it). Header `//! WIT emission + browser-bundle sourcemaps.` Add `use` (candidates: `kali_ast::...`, `serde_json::{json, Value}`, `std::path::Path`).

- [ ] **Step 3: Create `build/helpers.rs`**

Move verbatim: `parse_source_file` (build.rs:3322-3337), `signature_from_export_specifier` (5077-5080), `invalid_export_surface` (5081-5092), `source_stem` (5093-5100), `has_errors` (5122-5126). All private → **widen to `pub(crate)`**. Header `//! Cross-cutting build helpers (crate-internal).` Add `use` (candidates: `kali_ast::Statement`, `kali_lexer::{Lexer, Token, TokenType}`, `kali_parser::Parser`, `kali_error::Diagnostic`).

- [ ] **Step 4: Convert `build.rs` → `build/mod.rs`**

```bash
mkdir -p crates/kali_cli/src/build
git mv crates/kali_cli/src/build.rs crates/kali_cli/src/build/mod.rs
```
In `build/mod.rs`: delete the moved items (paths, wit, helpers ranges). Add at top:
```rust
mod helpers;
mod paths;
mod wit;

pub use paths::{
    binding_package_manifest_output_path_for, bundle_chunk_output_dir_for, bundle_output_paths_for,
    capi_output_paths_for, component_output_paths_for, executable_output_path_for,
    library_output_paths_for,
};
pub use wit::{browser_bundle_source_map, library_wit_for, LibraryExport};
pub(crate) use helpers::*;   // parse_source_file, source_stem, has_errors, etc. for sibling modules
```
Keep the `#[cfg(test)] #[path = "../build_tests.rs"] mod tests;` wiring (path now `../build_tests.rs`). The remaining items in `mod.rs` call helpers via the `pub(crate) use helpers::*` re-export.

- [ ] **Step 5: Build + verify** — run the Verification commands block. `build_tests.rs` uses `use super::*`; after this task `super` = `build/mod.rs`, which re-exports `helpers::*` pub(crate) and paths/wit pub. The 4 cutoff items (`profile_data_hash`, `incremental_cache_path`, `collect_library_exports_from_statements`, `collect_direct_bundle_calls_from_statements`) and `validate_artifact_metadata_value` are NOT yet moved (still in mod.rs as private/pub(crate)) — so `use super::*` still resolves them while they live in mod.rs. Expect 0 warnings, test-list diff EMPTY.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/src/build/
git commit -m "refactor(kali_cli): extract build paths/wit/helpers modules [refactor]"
```

- [ ] **Step 7: Update SDD ledger.**

---

### Task 5: build — extract `compile.rs`, `eval.rs`, `metadata.rs`, `entrypoint.rs` (+ cutoff widenings)

**Files:**
- Create: `crates/kali_cli/src/build/{compile,eval,metadata,entrypoint}.rs`
- Modify: `crates/kali_cli/src/build/mod.rs`

**Interfaces:**
- Produces: `build::compile::*`, `build::eval::*`, `build::metadata::*`, `build::entrypoint::*`. CRITICAL cutoff: `profile_data_hash` + `incremental_cache_path` (→compile.rs) and `validate_artifact_metadata_value` (→metadata.rs) MUST be `pub(crate)` and re-exported from `build/mod.rs` so `build_tests.rs`'s `use super::*` keeps resolving them.

- [ ] **Step 1: Create `build/compile.rs`**

Move verbatim (cluster B+C+D+E+J+K+H):
- `BuildMode` (pub enum, 375-381), `BuildOutput` (pub struct, 382-387), `CompileOutput` (pub struct, 471-476)
- `check_source_file` (pub, 428-444), `normalize_compiler_source` (pub, 445-454), `read_compiler_source_file` (pub, 455-470)
- `load_profile_data_file` (pub, 477-528), `profile_data_hash` (private, 529-538) → **widen to `pub(crate)`** (build_tests.rs uses it)
- `compile_source_file_with_cache_state` (pub, 539-562), `..._and_profile_data` (pub, 563-587), `..._and_profile_data_and_validation` (pub, 588-685), `compile_source_file` (pub, 686-704), `compile_source_file_with_specialization_cap` (pub, 705-728), `..._and_validation` (pub, 729-753), `..._and_profile_data` (pub, 754-778), `..._and_profile_data_and_validation` (pub, 779-806), `compile_source_file_uncached` (private, 807-874)
- `validation_diagnostic` (priv, 875-881), `validate_hir_tree` (priv, 882-886), `validate_mir_program` (priv, 887-891), `validate_lir_program` (priv, 892-897)
- `incremental_cache_path` (private, 898-949) → **widen to `pub(crate)`** (build_tests.rs uses it), `project_root_for_source` (priv, 950-959)
- `analyze_source_file` (priv, 960-1035), `source_uses_optional_chain_math_pow` (priv, 1036-1088), `source_uses_process_env_mutation` (priv, 1089-1136)
- `AnalyzedSource` (private struct, 1807-1812) → widen to `pub(crate)` (used by `build_source_file`, same module, so can stay private if both in compile.rs — keep private)
- `build_source_file` (pub, 1813-1885)
- `build_mode_from_flags` (pub, 2017-2027), `source_hash_for_file` (priv, 2028-2034), `build_mode_name` (priv, 2035-2042), `validate_runtime_profiles` (pub, 2043-2082)

Header `//! Compile pipeline, source analysis, incremental cache, profile data.` Add `use` lines (the big import header from build.rs — keep only what compile.rs uses; cross-module helpers via `use super::helpers::*;` and `use super::{...}` for eval/entrypoint calls; `output::validate_sorted_string_array_value` via `use crate::output::validate_sorted_string_array_value;`). Resolve via 0-warning gate.

- [ ] **Step 2: Create `build/eval.rs`**

Move verbatim (cluster F+G, build.rs:1137-1806): `EvalConst` (priv enum, 1137-1143) + `impl EvalConst` (1144-1178), `rewrite_eval_compat_source` (priv, 1179-1183), `rewrite_static_eval_calls` (priv, 1184-1225), `rewrite_static_function_constructor_calls` (priv, 1226-1285), `is_bare_function_constructor_spelling` (priv, 1286-1292), `find_immediate_invocation_end` (priv, 1293-1302), `parse_function_constructor_body_snippet` (priv, 1303-1312), `find_call_end` (priv, 1313-1350), `collect_constant_bindings` (priv, 1351-1382), `discover_dynamic_import_targets` (pub, 1383-1419), `parse_static_dynamic_import_specifier` (priv, 1420-1433), `find_token_call_end` (priv, 1434-1451), `resolve_dynamic_import_target` (priv, 1452-1477), `canonicalize_dynamic_import_candidate` (priv, 1478-1503), `find_statement_end` (priv, 1504-1540), `source_uses_eval_compat` (priv, 1541-1554), the 8 `matches_*` (1555-1624), `parse_eval_source_snippet` (priv, 1625-1639), the 6 `parse_constant_*` (1640-1712), `eval_plus` (priv, 1756-1766), `parse_template_constant_value` (priv, 1767-1790), `unquote_string_literal` (priv, 1791-1806). Also move `DynamicImportTarget` (pub struct, 388-393).
Private helpers stay private (only `discover_dynamic_import_targets` + `DynamicImportTarget` are pub). `collect_constant_bindings` is called by `discover_dynamic_import_targets` (same module) — stays private. Header + `use`.

- [ ] **Step 3: Create `build/metadata.rs`**

Move verbatim (cluster L+M+N, build.rs:2083-2870): `build_artifact_metadata` (pub, 2083-2317), `validate_generated_artifact_metadata` (priv, 2318-2323), `validate_build_result_value` (pub, 2324-2576), `validate_build_result_artifacts_array` (priv, 2577-2704), `validate_name_signature_object` (priv, 2705-2719), `validate_build_result_exports_array` (priv, 2720-2763), `build_result_artifact_sort_key` (priv, 2764-2784), `build_result_artifact_role_rank` (priv, 2785-2799), `validate_no_unexpected_keys` (priv, 2800-2813), `validate_canonical_non_empty_string_field` (priv, 2814-2836), `is_canonical_artifact_role` (priv, 2837-2851), `serialize_artifact_metadata` (pub, 2852-2857), `append_metadata_section` (pub, 2858-2870). Also move `ArtifactMetadata` (pub struct, 400-427) and `validate_artifact_metadata_value` (already `pub(crate)`, 2142 — keep `pub(crate)`; **build_tests.rs uses it via super::***).
Header + `use`.

- [ ] **Step 4: Create `build/entrypoint.rs`**

Move verbatim (cluster A+Q2+Q3): `block_contains_yield_delegation` (priv, 37-40), `statement_contains_yield_delegation` (priv, 41-193), `expression_contains_yield_delegation` (priv, 194-362), `generator_function_unavailable_message` (priv, 363-374), `reject_async_and_generator_class_methods_in_runtime_entrypoint` (pub, 3338-3849), `validate_unique_export_names_from_statements` (priv, 3850-3930).
`reject_async_...` calls the yield-delegation helpers (same module) — they stay private. `validate_unique_export_names_from_statements` is called by `analyze_source_file` in compile.rs → **widen to `pub(crate)`**. Header + `use` (kali_ast types, kali_error, kali_common's `generator_class_method_yield_lowering_unavailable_message_for_flavors`).

- [ ] **Step 5: Update `build/mod.rs`**

Delete the moved ranges from `mod.rs`. Add:
```rust
mod compile;
mod entrypoint;
mod eval;
mod metadata;

pub use compile::{
    build_mode_from_flags, build_source_file, check_source_file, compile_source_file,
    compile_source_file_with_cache_state, compile_source_file_with_cache_state_and_profile_data,
    compile_source_file_with_cache_state_and_profile_data_and_validation,
    compile_source_file_with_specialization_cap,
    compile_source_file_with_specialization_cap_and_profile_data,
    compile_source_file_with_specialization_cap_and_profile_data_and_validation,
    compile_source_file_with_specialization_cap_and_validation, load_profile_data_file,
    normalize_compiler_source, read_compiler_source_file, validate_runtime_profiles,
    BuildMode, BuildOutput, CompileOutput,
};
pub use entrypoint::reject_async_and_generator_class_methods_in_runtime_entrypoint;
pub use eval::{discover_dynamic_import_targets, DynamicImportTarget};
pub use metadata::{
    append_metadata_section, build_artifact_metadata, serialize_artifact_metadata,
    validate_build_result_value, ArtifactMetadata,
};
// Cutoff re-exports for build_tests.rs (use super::*):
pub(crate) use compile::{incremental_cache_path, profile_data_hash};
pub(crate) use metadata::validate_artifact_metadata_value;
```
(The `pub use` lists must cover EVERY pub item that was in build.rs — verify with `grep -nE '^\s*pub (fn|struct|enum)' crates/kali_cli/src/build/mod.rs` returning only the facade's own items, and cross-check that the union of submodule `pub` items equals the baseline `pub` set. If a pub item is missing from the re-export, add it — dropping it is forbidden by Global Constraints.)

- [ ] **Step 6: Build + verify** — run the Verification commands block. CRITICAL cutoff check: `cargo test -p kali_cli --tests` must compile build_tests.rs (it uses `profile_data_hash`, `incremental_cache_path`, `validate_artifact_metadata_value` via `super::*`). If "cannot find function" → the re-export line is missing. Expect 0 warnings, test-list diff EMPTY.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/src/build/
git commit -m "refactor(kali_cli): extract build compile/eval/metadata/entrypoint modules [refactor]"
```

- [ ] **Step 8: Update SDD ledger.**

---

### Task 6: build — extract `exports/{collect,signatures,types}.rs`; finalize `mod.rs` facade

**Files:**
- Create: `crates/kali_cli/src/build/exports/{mod,collect,signatures,types}.rs`
- Modify: `crates/kali_cli/src/build/mod.rs`

**Interfaces:**
- Produces: `build::collect_library_exports`, `build::collect_browser_bundle_exports` (pub) + the cutoff items `collect_library_exports_from_statements`, `collect_direct_bundle_calls_from_statements` (pub(crate), re-exported for build_tests.rs). After this task `build/mod.rs` is a thin facade.

- [ ] **Step 1: Create `build/exports/collect.rs`**

Move verbatim (cluster P+R):
- `collect_library_exports` (pub, 2914-2928), `collect_library_exports_from_source_path_with_context` (priv, 2929-2962), `collect_library_exports_from_statements_with_context` (priv, 2963-3204), `resolve_library_export_source_path` (priv, 3205-3227), `resolve_relative_library_export_source` (priv, 3228-3290)
- `collect_browser_bundle_exports` (pub, 3291-3321)
- `collect_library_exports_from_statements` (private, 3931-4083) → **widen to `pub(crate)`** (build_tests.rs uses it)
- `collect_tree_shake_roots` (priv, 4084-4113), `collect_reachable_bundle_exports` (priv, 4114-4154), `collect_direct_bundle_calls_from_statements` (private, 4155-4165) → **widen to `pub(crate)`** (build_tests.rs uses it), `collect_direct_bundle_calls_from_statement` (priv, 4166-4348), `collect_direct_bundle_calls_from_block` (priv, 4349-4358), `collect_direct_bundle_calls_from_variable_declaration` (priv, 4359-4370), `collect_direct_bundle_calls_from_expression` (priv, 4371-4553)

Header `//! Library/browser-bundle export + tree-shake collection.` Add `use` (kali_ast types, `use super::super::helpers::*;` for parse_source_file/source_stem, `use super::signatures::*;` and `use super::types::*;` for the signature/type helpers they call — trim to what's used).

- [ ] **Step 2: Create `build/exports/signatures.rs`**

Move verbatim (cluster S, build.rs:4554-4905): `collect_declared_function_signatures` (priv, 4554-4599), `collect_declared_function_binding_signatures` (priv, 4600-4631), `infer_function_signature` (priv, 4632-4636), `infer_function_binding_signature` (priv, 4637-4864), `is_object_freeze_call` (priv, 4865-4871), `call_member_access_name` (priv, 4872-4890), `member_access_name` (priv, 4891-4895), `function_signature` (priv, 4896-4905).
These are called by collect.rs (P/R) → **widen to `pub(crate)`** the ones collect.rs calls (`collect_declared_function_signatures`, `infer_function_signature`, and any others the compiler reports). Header + `use`.

- [ ] **Step 3: Create `build/exports/types.rs`**

Move verbatim (cluster T, build.rs:4906-5076): `infer_block_return_type` (priv, 4906-4920), `infer_static_truthiness` (priv, 4921-4965), `infer_expression_type` (priv, 4966-5012), `infer_unary_expression_type` (priv, 5013-5022), `infer_binary_expression_type` (priv, 5023-5062), `is_numeric_like_type` (priv, 5063-5066), `infer_literal_type` (priv, 5067-5076). Widen to `pub(crate)` the ones signatures.rs/collect.rs call (per compiler). Header + `use`.

- [ ] **Step 4: Create `build/exports/mod.rs`**

```rust
//! Library export, signature, and type inference collection.

mod collect;
mod signatures;
mod types;

pub use collect::{collect_browser_bundle_exports, collect_library_exports};
pub(crate) use collect::{
    collect_direct_bundle_calls_from_statements, collect_library_exports_from_statements,
};
pub(crate) use signatures::*;   // collect.rs calls these
pub(crate) use types::*;        // signatures.rs/collect.rs call these
```
(Trim the `pub(crate) use signatures::*;` / `types::*;` to explicit lists if the compiler flags a glob as unused — but globs are fine for 0 warnings.)

- [ ] **Step 5: Finalize `build/mod.rs` as a thin facade**

Delete the moved ranges from `mod.rs`. Add `mod exports;` and:
```rust
pub use exports::{collect_browser_bundle_exports, collect_library_exports};
// Cutoff re-exports for build_tests.rs (use super::*):
pub(crate) use exports::{collect_direct_bundle_calls_from_statements, collect_library_exports_from_statements};
```
`mod.rs` should now be a thin facade (verify: `grep -nE '^\s*(fn|struct|enum|impl)\s' crates/kali_cli/src/build/mod.rs` → no hits). Final `mod.rs` declares: `mod {compile, entrypoint, eval, exports, helpers, metadata, paths, wit};` + all the `pub use`/`pub(crate) use` lines from Tasks 4–6 + `#[cfg(test)] #[path = "../build_tests.rs"] mod tests;`.

- [ ] **Step 6: Build + verify** — run the Verification commands block. CRITICAL: `cargo test -p kali_cli --tests` compiles build_tests.rs (cutoff items `collect_library_exports_from_statements` + `collect_direct_bundle_calls_from_statements` resolve via `super::*`). Test-list diff EMPTY. `cargo build -p kali_cli --bin kali` succeeds (all `build::*` pub paths resolve).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/src/build/
git commit -m "refactor(kali_cli): extract build exports submodules; thin mod.rs facade [refactor]"
```

- [ ] **Step 8: Update SDD ledger** — Phase B (build) complete.

---

### Task 7: bin — extract `shared.rs`, `config.rs`

**Files:**
- Create: `crates/kali_cli/src/bin/{shared,config}.rs`
- Modify: `crates/kali_cli/src/bin/kali.rs`

**Interfaces:**
- Produces: `shared::*` and `config::*` as `pub(crate)` (within the bin). The command modules (Tasks 8–9) and the inline `mod tests` (Task 9) call these via `super::shared::*` / `super::config::*`.

- [ ] **Step 1: Create `bin/shared.rs`**

Move verbatim from `bin/kali.rs`: `print_envelope` (5055-5088), `diagnostics_exit_code` (5032-5044), `emit_native_json_payload` (5002-5031), `emit_diagnostics_and_exit` (5089-5118), `split_and_convert_diagnostics` (5119-5136), `single_diagnostic_to_values` (5137-5146), `load_policy_or_exit` (4575-4594), `ensure_project_ready_or_exit` (4595-4614), `selected_source_files` (4615-4627), `single_or_error` (4628-4656), `validate_runtime_entrypoint` (4657-4677), `reject_workflow_context_flags` (4474-4493), `reject_install_context_flags` (4494-4510), `command_allows_pretty_without_json` (4351-4357), `matches_test_filter` (5045-5054). All private → **widen to `pub(crate)`**. Header `//! Shared CLI helpers: envelope printing, exit codes, preflight (crate-internal).` Add `use` lines (the bin's import header — keep only what shared.rs uses; `crate::output::{...}` and `kali_error::{...}` etc.).

- [ ] **Step 2: Create `bin/config.rs`**

Move verbatim from `bin/kali.rs`: `resolve_effective_api_surface` (794-810), `config_diagnostic_context` (811-814), `config_diagnostic_context_with_value` (815-821), `manifest_api_surface` (822-863), `resolve_effective_compat_features` (864-878), `resolve_effective_runtime_profiles` (879-901), `resolve_effective_max_specializations` (902-917), `resolve_profile_data` (918-924), `manifest_compat_features` (925-990), `manifest_runtime_profiles` (991-1042), `manifest_max_specializations` (1043-1077), `normalize_compat_features` (1078-1088), `reject_unavailable_compat_features` (1089-1124), `reject_unavailable_runtime_profiles` (1125-1161), `browser_runtime_harness_command_available` (1162-1168), `reject_unavailable_browser_runtime` (1169-1192), `reject_unavailable_spawned_process_budget` (1193-1229), `reject_unavailable_zero_capable_budgets` (1230-1278). All private → **widen to `pub(crate)`**. Header `//! CLI+manifest config resolution + runtime availability validation (crate-internal).` Add `use`.

- [ ] **Step 3: Update `bin/kali.rs`**

Delete the moved ranges from `kali.rs`. Add near the top (after the `use` block):
```rust
mod config;
mod shared;
```
The remaining items in `kali.rs` (main + command handlers) call shared/config helpers — they now resolve via `shared::print_envelope(...)` etc. **Update each call site** that referenced these as bare names (e.g. `print_envelope(...)`) to `shared::print_envelope(...)` and `resolve_effective_*` to `config::resolve_effective_*`. (Search: `grep -nE '\b(print_envelope|diagnostics_exit_code|emit_native_json_payload|emit_diagnostics_and_exit|split_and_convert_diagnostics|single_diagnostic_to_values|load_policy_or_exit|ensure_project_ready_or_exit|selected_source_files|single_or_error|validate_runtime_entrypoint|reject_workflow_context_flags|reject_install_context_flags|command_allows_pretty_without_json|matches_test_filter|resolve_effective_|config_diagnostic_context|manifest_|normalize_compat_features|reject_unavailable_|browser_runtime_harness_command_available|resolve_profile_data)\b' crates/kali_cli/src/bin/kali.rs` — prefix each non-definition occurrence with `shared::` or `config::` as appropriate.) This is wiring, not body rewriting.

- [ ] **Step 4: Build + verify** — run the Verification commands block (incl. `cargo build -p kali_cli --bin kali`). The inline `mod tests` (still in kali.rs) references some of these helpers via `super::` — they now live in `shared`/`config`, so update those test references to `super::shared::...` / `super::config::...` (the tests stay in kali.rs per the spec). Expect 0 warnings, test-list diff EMPTY.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/src/bin/
git commit -m "refactor(kali_cli): extract bin shared/config helper modules [refactor]"
```

- [ ] **Step 6: Update SDD ledger.**

---

### Task 8: bin — extract simple command modules (`cmd_doctor`, `cmd_check`, `cmd_fmt`, `cmd_lint`, `cmd_effects`, `cmd_install`, `cmd_run`, `cmd_test`)

**Files:**
- Create: `crates/kali_cli/src/bin/cmd_{doctor,check,fmt,lint,effects,install,run,test}.rs`
- Modify: `crates/kali_cli/src/bin/kali.rs`

**Interfaces:**
- Produces: 8 command-handler modules, each `pub(crate)` so `main` (in kali.rs) can call them.

- [ ] **Step 1: Create the 8 command modules** (one file each, verbatim move + widen to `pub(crate)`)

| File | Move verbatim from `bin/kali.rs` | Items |
|---|---|---|
| `cmd_doctor.rs` | 345-430 | `doctor_command` |
| `cmd_check.rs` | 431-584 | `check_command` |
| `cmd_fmt.rs` | 3673-3770 | `fmt_command` |
| `cmd_lint.rs` | 3771-3875 | `lint_command` |
| `cmd_effects.rs` | 3876-3991 | `effects_command` |
| `cmd_install.rs` | 4494-4510 + 4511-4574 | `reject_install_context_flags` + `install_command` |
| `cmd_run.rs` | 2963-2974 + 2975-3222 | `browser_stdout_thread_topology_snapshot_value` + `run_command` |
| `cmd_test.rs` | 3223-3626 + 3627-3672 | `test_command` + `coverage_function_count_from_wasm`, `normalize_coverage_report_path`, `sort_coverage_reports`, `coverage_percent` |

Each: widen the moved `fn`/helper to **`pub(crate)`**. Header `//! <command> command handler.` Add `use` lines (bin import header subset + `use super::shared::*;` / `use super::config::*;` for the shared/config helpers they call — trim to what's used). `cmd_install` keeps `reject_install_context_flags` private within itself if only `install_command` calls it; else `pub(crate)`.

- [ ] **Step 2: Update `bin/kali.rs`**

Delete the moved ranges. Add `mod` declarations:
```rust
mod cmd_check;
mod cmd_doctor;
mod cmd_effects;
mod cmd_fmt;
mod cmd_install;
mod cmd_lint;
mod cmd_run;
mod cmd_test;
```
Update `main`'s dispatch call sites: bare `doctor_command(...)` → `cmd_doctor::doctor_command(...)`, etc. (wiring).

- [ ] **Step 3: Build + verify** — run the Verification commands block. Expect 0 warnings, test-list diff EMPTY.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_cli/src/bin/
git commit -m "refactor(kali_cli): extract bin simple command modules [refactor]"
```

- [ ] **Step 5: Update SDD ledger.**

---

### Task 9: bin — extract `cmd_build.rs`, `cmd_package.rs`; finalize `kali.rs` facade

**Files:**
- Create: `crates/kali_cli/src/bin/{cmd_build,cmd_package}.rs`
- Modify: `crates/kali_cli/src/bin/kali.rs` (now: attrs + use + main + mod decls + inline mod tests)

**Interfaces:**
- Produces: the finalized `kali` binary. `kali.rs` keeps only `#![allow(...)]`, the `use` block, `fn main` (dispatch), `mod` declarations, and the inline 23-test `mod tests`.

- [ ] **Step 1: Create `bin/cmd_build.rs`**

Move verbatim from `bin/kali.rs` (cluster build + artifacts + browser bundle):
- `build_command` (585-793)
- `BuildArtifactSelection` (priv enum, 1279-1287), `BundleArtifact` (priv struct, 1288-1292), `BrowserBundleBuild` (priv struct, 1293-1304), `BuildResult` (priv enum, 1305-1346), `build_result_artifact_sort_key` (priv, 1347-1368), `build_result_artifact_role_rank` (priv, 1369-1383), `impl BuildResult` (1384-1581)
- `build_executable_artifact` (priv, 1582-1663), `build_library_artifact` (priv, 1664-1772), `build_capi_artifact` (priv, 1773-1966), `build_component_artifact` (priv, 1967-2127), `build_browser_bundle_artifact` (priv, 2128-2187), `write_browser_bundle_files` (priv, 2188-2345), `collect_browser_bundle_chunk_artifacts` (priv, 2346-2427), `browser_bundle_dynamic_import_map` (priv, 2428-2454), `normalize_dynamic_import_specifier` (priv, 2455-2496), `relative_path` (priv, 2497-2525), `generate_browser_bundle_js` (priv, 2526-2962)

Widen `build_command` to **`pub(crate)`** (main calls it). The rest stay private within `cmd_build.rs`. Header `//! build command: artifact + browser-bundle generation.` Add `use` (bin import header subset + `use super::shared::*; use super::config::*;` + `use kali_cli::build::{...}` for the build API).

- [ ] **Step 2: Create `bin/cmd_package.rs`**

Move verbatim from `bin/kali.rs` (package effects/audit + registry-target/bin-entrypoint families):
- `package_analysis_specific_flag_context` (priv, 3992-4038), `reject_package_analysis_specific_flags` (priv, 4039-4065), `require_single_registry_package_target` (priv, 4066-4127), `package_effects_command` (priv, 4128-4315), `sort_package_audit_findings` (priv, 4316-4342), `diagnostic_span_sort_key` (priv, 4343-4350), `PACKAGE_AUDIT_PREVIEW_MESSAGE` (priv const, 4358-4360), `package_audit_preview_diagnostic` (priv, 4361-4369), `package_audit_command` (priv, 4370-4473)
- `PackageBinEntrypoint` (priv struct, 4678-4682), `validate_package_bin_runtime_entrypoint` (priv, 4683-4713), `detect_package_bin_entrypoint` (priv, 4714-4747), `package_root_for_node_modules_source` (priv, 4748-4771), `collect_unsupported_node_bin_markers` (priv, 4772-4782), `source_mentions_identifier` (priv, 4783-4807), `ParsedRegistryPackageTarget` (priv struct, 4808-4814), `parse_registry_package_target` (priv, 4815-4913), `is_version_suffixed_package_spec` (priv, 4914-4920), `find_package_root` (priv, 4921-4933), `read_package_version` (priv, 4934-4967), `analysis_context_for_api` (priv, 4968-4978), `validate_source_effects_against_policy` (priv, 4979-4986), `validate_source_effects_against_policy_for_roots` (priv, 4987-5001)

Widen `package_effects_command` + `package_audit_command` to **`pub(crate)`** (main calls them). The inline `mod tests` (23) references: `analysis_context_for_api`, `package_analysis_specific_flag_context`, `package_audit_command`, `package_audit_preview_diagnostic`, `package_effects_command`, `sort_package_audit_findings`, `PACKAGE_AUDIT_PREVIEW_MESSAGE`, `parse_registry_package_target`. Since the tests STAY in `kali.rs` (per spec), widen these 8 to **`pub(crate)`** so `super::cmd_package::*` resolves from the tests. Header `//! package-effects + package-audit command handlers.` Add `use`.

- [ ] **Step 3: Finalize `bin/kali.rs`**

Delete the moved ranges. Add `mod cmd_build; mod cmd_package;` to the mod declarations. Update `main` dispatch: `build_command(...)` → `cmd_build::build_command(...)`, `package_effects_command(...)` → `cmd_package::package_effects_command(...)`, `package_audit_command(...)` → `cmd_package::package_audit_command(...)`.
The inline `mod tests` references cross-module helpers — update those `super::<helper>` references to `super::shared::<helper>`, `super::config::<helper>`, `super::cmd_package::<helper>` as appropriate (the 8 package items listed above, plus `super::shared::diagnostics_exit_code`, `super::shared::emit_native_json_payload`, `super::shared::command_allows_pretty_without_json`, `super::config::manifest_*`). Verify with: `cargo test -p kali_cli --bin kali` compiles the inline tests.
`kali.rs` should now contain only: `#![allow(...)]`, `use` block, `fn main`, `mod` declarations (config, shared, cmd_*), and `#[cfg(test)] mod tests`.

- [ ] **Step 4: Build + verify** — run the Verification commands block. CRITICAL: `cargo build -p kali_cli --bin kali` 0 warnings; `cargo test -p kali_cli` — the inline 23 tests compile + pass; test-list diff EMPTY. Spot-check a CLI-subprocess integration test:
```bash
cargo test -p kali_cli --test version_smoke 2>&1 | tee /tmp/c9_version.txt
cargo test -p kali_cli --test doctor 2>&1 | tee /tmp/c9_doctor.txt
```
Expected: both pass (binary behavior unchanged).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/src/bin/
git commit -m "refactor(kali_cli): extract bin build/package command modules; finalize kali.rs [refactor]"
```

- [ ] **Step 6: Update SDD ledger** — Phase C (bin) complete.

---

### Task 10: Final verification + local-main ff-merge

**Files:** None (verification + merge only).

- [ ] **Step 1: Full workspace verification**

```bash
cargo build -p kali_cli 2>&1 | tee /tmp/final_build.txt          # 0 warnings
cargo build -p kali_cli --bin kali 2>&1 | tee /tmp/final_bin.txt  # 0 warnings
cargo test -p kali_cli -- --list 2>/dev/null | sort > /tmp/final_list.txt
diff /tmp/kali_cli_baseline_tests.txt /tmp/final_list.txt && echo "TEST LIST UNCHANGED (zero behavior/test change)"
cargo test -p kali_cli 2>&1 | tee /tmp/final_test.txt             # only the 2 known baseline-red failures
```
Confirm: build 0 warnings; test-list diff EMPTY; test failures == exactly the 2 baseline `build_bundles_*` failures (no new failures). Also confirm `cargo build -p kali_cli` consumers (e.g. any workspace member depending on kali_cli) still compile — `cargo build --workspace 2>&1 | tee /tmp/final_ws.txt` (or at minimum the reverse-dep set).

- [ ] **Step 2: Confirm no stray fmt / public-surface drift**

```bash
# mod.rs facades are thin (no production item bodies):
grep -nE '^\s*(fn|struct|enum|impl)\s' crates/kali_cli/src/output/mod.rs crates/kali_cli/src/build/mod.rs   # expect no hits
# public surface unchanged: union of pub items in submodules == baseline pub set
grep -rnE '^\s*pub (fn|struct|enum)' crates/kali_cli/src/output/ crates/kali_cli/src/build/ | wc -l   # cross-check vs baseline count
```

- [ ] **Step 3: ff-merge to local main (NEVER push to origin)**

```bash
git checkout main
git merge --ff-only refactor/kali-cli-src-modularization
git rev-parse HEAD   # record
cargo build -p kali_cli && cargo test -p kali_cli -- --list 2>/dev/null | sort | diff - /tmp/kali_cli_baseline_tests.txt && echo "MERGED MAIN GREEN"
git branch -d refactor/kali-cli-src-modularization
```
Expected: fast-forward; merged main build+test green; test-list diff EMPTY; branch deleted. `origin/main` UNCHANGED (local-main-only per series default).

- [ ] **Step 4: Final SDD ledger entry** — record: 21st crate (kali_cli) sub-project 1 of 3 done; merged main HEAD; local main N commits ahead of origin; sub-projects 2 (integration-test monoliths) and 3 (tests/ grouping) remain.

---

## Self-Review (run by plan author — done)

**1. Spec coverage:** Spec Section 1 (output → 10 modules) → Tasks 1–3. Section 2 (build → 9 modules incl. exports/) → Tasks 4–6. Section 3 (bin → 13 modules) → Tasks 7–9. Visibility/pub(crate) strategy → embedded in each task's widen instructions + the cutoff re-exports. Test handling (build_tests/output_tests untouched, inline tests move/stay) → Tasks 1–9. Execution rhythm (output→build→bin, verify per task, ff-merge) → Tasks 0–10. ✓ No gaps.

**2. Placeholder scan:** Verbatim moves reference exact line ranges + item names (not "TBD"/"implement later"). `use` lines that depend on per-fn body analysis are specified as "determine via the 0-warning compiler gate" with candidate imports listed — this matches the established code-motion convention (the recurring "trim unused types from `use crate::{...}`" gotcha is resolved empirically against the compiler, per series memory). Wiring code (`mod`/`pub use`/`pub(crate) use`) is shown verbatim. ✓

**3. Type consistency:** Public-surface re-export names cross-checked against the grep'd `pub fn`/`pub struct`/`pub enum` set of output.rs and build.rs and the bin's `build::*`/`output::*` call sites. Cutoff item names (`profile_data_hash`, `incremental_cache_path`, `collect_library_exports_from_statements`, `collect_direct_bundle_calls_from_statements`, `validate_artifact_metadata_value`, `validate_sorted_string_array_value`) verified present at the cited line numbers. ✓
