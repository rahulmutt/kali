# kali_codegen Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Break `kali_codegen` (one 10,331-line `lib.rs` built around a single ~201-method `impl<'a> FunctionEmitter<'a>`, plus an 8,024-line flat `tests.rs` of 324 tests) into small, single-purpose modules with co-located sibling test files, a shared `kali_test_support` dev-dependency, and a crate-local LIR-builder `test_support` — with zero behavior change.

**Architecture:** `lib.rs` becomes a thin facade (module declarations, import-index `const`s, and `pub use` re-exports of `CodegenCtx`/`TargetConfig`/`CodegenResult`/`lower_lir_to_wasm`). The single ~9,000-line `impl<'a> FunctionEmitter<'a>` is split across focused files, each carrying its own `impl<'a> FunctionEmitter<'a> { … }` block (legal within one crate). The crate driver and program-analysis free fns move to `lower.rs`. Tests move into sibling `*_tests.rs` files wired with `#[cfg(test)] #[path = "…"] mod`.

**Tech Stack:** Rust 2021, Cargo workspace, `wasm-encoder`, `wasmparser`, `semver`, `serde`/`serde_json`; dev: `wasmprinter`, and the existing `kali_hir`/`kali_lexer`/`kali_mir`/`kali_parser` pipeline crates.

## Global Constraints

- **Zero behavior change.** Pure structural refactor. The set of tests that exist and pass is identical before and after (renames excepted, tracked explicitly).
- **Green at every commit.** `cargo test -p kali_codegen` must pass after every task. Never commit a red tree.
- **Public API preserved.** External paths `kali_codegen::CodegenCtx`, `kali_codegen::TargetConfig`, `kali_codegen::CodegenResult`, and `kali_codegen::lower_lir_to_wasm` must keep resolving. The facade re-exports them with `pub use`. (`FunctionEmitter` and the `Static*` result enums are crate-internal — never `pub`.)
- **Text-movement only.** No method body is rewritten. Cross-referenced items are widened to `pub(crate)` in Task 2, after which `self.foo()` resolves across `impl` blocks anywhere in the crate.
- **Test convention.** Unit tests live in sibling `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod …;` — never inline `#[cfg(test)] mod tests { … }` blocks.
- **No new runtime dependencies.** `kali_test_support` is a `dev-dependency` only; it already exists in the workspace.
- **Branch already exists.** Work continues on `refactor/kali-codegen-modularization` (created during brainstorming; the design spec is already committed there). All paths are relative to repo root `/workspace`.
- **Verification triad per task:** `cargo build -p kali_codegen` → `cargo test -p kali_codegen` → `cargo clippy -p kali_codegen --all-targets -- -D warnings` (the clippy gate may be deferred to Task 10/14 if a transient `pub(crate)`-could-be-private style lint fires mid-split; build + test stay green every commit).

---

### Task 1: Baseline test snapshot

**Files:**
- Create: `docs/superpowers/baselines/kali_codegen-tests-before.txt`

**Interfaces:**
- Produces: `kali_codegen-tests-before.txt` — the authoritative list of test names before refactor, diffed against in Task 14.

- [ ] **Step 1: Confirm we are on the refactor branch**

```bash
cd /workspace
git branch --show-current
```
Expected: `refactor/kali-codegen-modularization`. (If not: `git checkout refactor/kali-codegen-modularization`.)

- [ ] **Step 2: Confirm the suite is green before any change**

```bash
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: `test result: ok.` with ≈324 passed and no failures.

- [ ] **Step 3: Snapshot the exact set of test names**

```bash
mkdir -p docs/superpowers/baselines
cargo test -p kali_codegen -- --list 2>/dev/null | grep ': test$' | sed 's/: test$//' | sort > docs/superpowers/baselines/kali_codegen-tests-before.txt
wc -l docs/superpowers/baselines/kali_codegen-tests-before.txt
```
Expected: a line count matching the unit-test count (≈324).

- [ ] **Step 4: Commit the baseline**

```bash
git add docs/superpowers/baselines/kali_codegen-tests-before.txt
git commit -m "chore(kali_codegen): snapshot test baseline [refactor]"
```

---

### Task 2: Widen internal visibility to `pub(crate)` (the enabling step)

**Why first:** Once methods live in sibling modules, Rust privacy blocks cross-module access to *private* items. Promoting crate-internal items to `pub(crate)` up front turns every later extraction into pure text movement. This task changes only visibility keywords — no code moves, no behavior change.

**Files:**
- Modify: `crates/kali_codegen/src/lib.rs`

**Interfaces:**
- Produces: `FunctionEmitter<'a>` (+ all its fields), the support types `FunctionPlan`/`ValueShape`/`ObjectEnumerationMode`/`ControlFlowLabelKind`/`LoopFrame`/`EmittedValue`/`StringPool` (+ fields/methods), the five `Static*` result enums (+ impls), every private `impl FunctionEmitter` method, and the cross-referenced trailing free fns all become `pub(crate)`.

- [ ] **Step 1: Promote the `FunctionEmitter` struct and its fields**

In `crates/kali_codegen/src/lib.rs`, change `struct FunctionEmitter<'a> {` (line ≈254) to `pub(crate) struct FunctionEmitter<'a> {`, and prefix every field inside it with `pub(crate)` (fields: `program`, `node_lookup`, `scratch_nodes`, `functions`, `env_set_import_index`, `env_delete_import_index`, `env_get_import_index`, `env_has_import_index`, `cwd_set_import_index`, `process_exit_import_index`, `diagnostics`, `strings`, `source_path`, `current_function_flavor`, `locals`, `bindings`, `reported_placeholder_fallbacks`, `control_frames`, `loop_frames`, and any remaining fields through the end of the struct).

- [ ] **Step 2: Promote the support types and their fields/methods**

Change each of these to `pub(crate)` (the type keyword and every field/variant/method):
- `struct FunctionPlan {` (≈181) — fields `name`, `params`, `locals`, `body`, `result`, `is_entry`, `flavor`.
- `enum ValueShape {` (≈192), `enum ObjectEnumerationMode {` (≈199), `enum ControlFlowLabelKind {` (≈207).
- `struct LoopFrame {` (≈214) — fields `break_index`, `continue_index`.
- `struct EmittedValue {` (≈220) — fields `produced`, `shape`.
- `struct StringPool {` (≈225) — fields `entries`, `offsets`, `next_offset`; and `impl StringPool` (≈231) methods `new`, `intern` → `pub(crate) fn`.

- [ ] **Step 3: Promote the five `Static*` result enums and their impls**

Change to `pub(crate)`: `enum StaticObjectIdentityValue {` (≈46), `enum StaticArraySearchResult {` (≈56), `enum StaticArrayAtResult {` (≈62), `enum StaticStringAtResult {` (≈68), `enum StaticIndexMemberResult {` (≈74). In `impl StaticObjectIdentityValue` (≈80), promote each private method (`same_value`, `strict_eq`, `same_value_zero`, `is_nullish`, `truthiness`) to `pub(crate) fn`.

- [ ] **Step 4: Promote every private method inside `impl<'a> FunctionEmitter<'a>`**

The impl spans lines ≈276–9356. Mechanically promote each top-level (4-space-indented) private method:

```bash
cd /workspace
awk 'NR>=276 && NR<=9356 && /^    fn /{sub(/^    fn /, "    pub(crate) fn ")} {print}' crates/kali_codegen/src/lib.rs > /tmp/cg_vis.rs && mv /tmp/cg_vis.rs crates/kali_codegen/src/lib.rs
grep -c "^    pub(crate) fn " crates/kali_codegen/src/lib.rs
```
Expected: ≈201 (every previously-private method).

- [ ] **Step 5: Promote the cross-referenced trailing free fns**

These top-level free fns (lines ≈9357–10300) are called by methods/driver code that will live in other modules; promote each to `pub(crate) fn` (locate by name): `generator_lowering_unavailable_message`, `collect_functions`, `program_uses_env_get`, `program_uses_env_has`, `is_process_root`, `process_env_property_key`, `program_uses_env_set`, `program_uses_env_delete`, `program_uses_cwd_set`, `program_uses_process_exit`, `collect_functions_from_node`, `function_plan`, `is_function_like`, `collect_function_locals`, `collect_function_locals_from_node`, `top_level_children`, `emit_literal`, `encode_string_handle`, `quote_string_literal`, `semver_min_version`, `strip_string_delimiters`, `parse_number_literal`, `parse_numeric_literal_value`, `is_supported_static_ascii_char_code`, `static_parse_float_ascii_integer`, `static_parse_int_ascii`. Leave `pub fn lower_lir_to_wasm` as-is (already public).

- [ ] **Step 6: Build and test**

```bash
cargo build -p kali_codegen 2>&1 | tail -5
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: build succeeds (any "could be private" style warnings are acceptable), `test result: ok.` with the same count as Task 1.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen/src/lib.rs
git commit -m "refactor(kali_codegen): widen internals to pub(crate) for module split [refactor]"
```

---

## Source-extraction tasks (Tasks 3–9)

**Shared procedure for every source-extraction task below.** Each task creates one module file (or directory module), moves a named set of items out of `lib.rs` into it, adds the module declaration (+ re-exports for public types) to the facade, and proves green. The recipe is identical; only the file name and item list change:

1. Create the new file with a `//!` doc line and `use crate::*;` (for nested directory modules) or `use super::*;` (for top-level sibling modules) — both correctly bring the crate import surface into scope for method-only modules. Add any extra `use` only if a name is reported missing.
2. For method modules, wrap the moved methods in `impl<'a> FunctionEmitter<'a> { … }`. For type/free-fn modules, paste at file top level.
3. Cut the listed items out of `lib.rs`, paste into the new file.
4. In `lib.rs`, add `mod <name>;` (private — methods attach to `FunctionEmitter` regardless of module path), or `mod <name>; pub use <name>::{…};` for modules defining public types.
5. `cargo build -p kali_codegen && cargo test -p kali_codegen 2>&1 | tail -5`; both green, same count.
6. Commit `refactor(kali_codegen): extract <module> [refactor]`.

**Do not change any method body.** If a moved item references a name that compiles only because everything was one module, the fix is to add `pub(crate)` to *that* item back in `lib.rs` — never to alter a body. **Borderline names** (a method whose name suggests one domain but whose only caller lives in another) go with their **primary caller**, confirmed by a quick `grep -n "self\.<name>(" crates/kali_codegen/src/*.rs crates/kali_codegen/src/**/*.rs` at extraction time. Flagged borderlines: `resolve_static_array_to_string_call` / `static_array_join_element_to_string` (array-driven but produce strings → keep in `string.rs` with the other string producers), `perfect_square_root_i128` and `emit_integer_math_arg` (math helpers grouped under `emit/`/`number` here — leave unless their sole caller is `math.rs`).

---

### Task 3: Extract `ctx.rs` (context, config, result, string pool, static-result enums)

**Files:**
- Create: `crates/kali_codegen/src/ctx.rs`
- Modify: `crates/kali_codegen/src/lib.rs`

**Interfaces:**
- Produces (re-exported from facade): `CodegenCtx`, `TargetConfig`, `CodegenResult`. Also moves (crate-internal): `StringPool`, and the five `Static*` result enums.

- [ ] **Step 1:** Create `ctx.rs` with header `use super::*;`. Move into it, verbatim:
  - `pub struct CodegenCtx` + `impl CodegenCtx` (the `new` constructor).
  - `pub struct TargetConfig` + `impl Default for TargetConfig`.
  - `pub struct CodegenResult`.
  - `pub(crate) struct StringPool` + `impl StringPool`.
  - The five enums `StaticObjectIdentityValue` (+ its impl), `StaticArraySearchResult`, `StaticArrayAtResult`, `StaticStringAtResult`, `StaticIndexMemberResult`.
- [ ] **Step 2:** In `lib.rs`: add `mod ctx; pub use ctx::{CodegenCtx, CodegenResult, TargetConfig};`. Keep the import-index `const`s and the `use` block in `lib.rs` for now (they are still referenced by code remaining there).
- [ ] **Step 3:** `cargo build -p kali_codegen && cargo test -p kali_codegen 2>&1 | tail -5`. Green, same count.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_codegen): extract ctx module [refactor]"`

---

### Task 4: Extract `emitter.rs` (emitter struct + lifecycle/support types)

**Files:**
- Create: `crates/kali_codegen/src/emitter.rs`
- Modify: `crates/kali_codegen/src/lib.rs`

**Interfaces:**
- Produces (crate-internal): `FunctionEmitter<'a>` (the type all later `impl` blocks attach to) plus its lifecycle methods and the support types.

- [ ] **Step 1:** Create `emitter.rs` with `use super::*;`. Move into it:
  - `pub(crate) struct FunctionEmitter<'a>` (the struct definition).
  - The support types `FunctionPlan`, `ValueShape`, `ObjectEnumerationMode`, `ControlFlowLabelKind`, `LoopFrame`, `EmittedValue`.
  - An `impl<'a> FunctionEmitter<'a> { … }` block containing the **lifecycle** methods: `new`, `node`, `alloc_scratch_node`, `push_control_frame`, `pop_control_frame`, `control_frame_depth`, `push_placeholder_fallback_diagnostic`.
- [ ] **Step 2:** In `lib.rs`: add `mod emitter;` (no re-export; type is crate-internal).
- [ ] **Step 3:** Build + test green, same count.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_codegen): extract emitter module [refactor]"`

---

### Task 5: Extract `emit/` (core emission)

**Files:**
- Create: `crates/kali_codegen/src/emit/mod.rs`
- Modify: `crates/kali_codegen/src/lib.rs`

**Interfaces:**
- Produces: `impl<'a> FunctionEmitter<'a>` cluster; no new public types.

- [ ] **Step 1:** Create `emit/mod.rs` with `use crate::*;` and one `impl<'a> FunctionEmitter<'a> { … }` block. Move into it these methods:
  `assignment_target_name`, `emit_aggregate_literal`, `emit_assignment`, `emit_binary`, `emit_branch`, `emit_break_or_continue`, `emit_call`, `emit_exponentiation_expression`, `emit_function_body`, `emit_node`, `emit_sequence`, `emit_unary`, `emit_update_expression`, `emit_value`, `for_of_binding_name`, `for_of_binding_name_from_node`, `is_supported_callable_reference`, `perfect_square_root_i128`, `resolve_bound_member_callable_node`, `resolve_bound_node`, `resolve_literal_aggregate`, `resolve_static_index_member`, `resolve_static_reference_root_name`, `resolve_transparent_callable_node`, `static_member_index`, `unwrap_transparent_value_node`.
- [ ] **Step 2:** In `lib.rs`: add `mod emit;`. Build + test green, same count.
- [ ] **Step 3:** `git add -A && git commit -m "refactor(kali_codegen): extract emit core module [refactor]"`

---

### Task 6: Extract `intrinsics/string.rs` and `intrinsics/array.rs`

**Files:**
- Create: `crates/kali_codegen/src/intrinsics/mod.rs`
- Create: `crates/kali_codegen/src/intrinsics/string.rs`
- Create: `crates/kali_codegen/src/intrinsics/array.rs`
- Modify: `crates/kali_codegen/src/lib.rs`

**Interfaces:**
- Produces: `impl<'a> FunctionEmitter<'a>` clusters; no new public types.

- [ ] **Step 1:** Create `intrinsics/mod.rs`:
```rust
//! Static recognition and constant-folding of JS/host intrinsic call shapes.
use crate::*;
mod string;
mod array;
```
- [ ] **Step 2:** `intrinsics/string.rs` (`use crate::*;`, one `impl<'a> FunctionEmitter<'a>` block) — move these methods:
  `is_string_at_call_with_literal_receiver`, `is_string_char_at_call_with_literal_receiver`, `is_string_char_code_at_call_with_literal_receiver`, `is_string_code_point_at_call_with_literal_receiver`, `is_string_concat_call_with_literal_receiver`, `is_string_from_char_code_callable`, `is_string_normalize_call_with_literal_receiver`, `is_string_repeat_call_with_literal_receiver`, `is_string_split_call_with_literal_receiver`, `render_static_string_value`, `resolve_static_array_to_string_call`, `resolve_static_string_at_call`, `resolve_static_string_case_call`, `resolve_static_string_char_at_call`, `resolve_static_string_char_code_at_call`, `resolve_static_string_code_point_at_call`, `resolve_static_string_concat_call`, `resolve_static_string_from_char_code_call`, `resolve_static_string_identity_call`, `resolve_static_string_normalize_call`, `resolve_static_string_pad_call`, `resolve_static_string_repeat_call`, `resolve_static_string_replace_call`, `resolve_static_string_search_call`, `resolve_static_string_slice_call`, `resolve_static_string_split_call`, `resolve_static_string_split_parts_from_id`, `resolve_static_string_substring_call`, `resolve_static_string_trim_call`, `static_array_join_element_to_string`, `static_ascii_string_relational_result`, `string_case_call_method`, `string_identity_call_method_with_literal_receiver`, `string_pad_call_method`, `string_replace_call_method_with_literal_receiver`.
  Also move the string-helper **free fns** `quote_string_literal` and `strip_string_delimiters` to the top level of this file.
- [ ] **Step 3:** `intrinsics/array.rs` (`use crate::*;`, one `impl<'a> FunctionEmitter<'a>` block) — move these methods:
  `collect_for_of_array_iteration_items`, `collect_static_array_concat_operand`, `emit_for_of_array_iteration`, `is_array_at_call_with_literal_receiver`, `is_array_callback_iteration_call`, `is_array_from_call`, `is_array_from_callable_node`, `is_array_is_array_call`, `is_array_literal`, `is_array_object`, `is_frozen_array_from_call`, `is_identity_array_flat_map_callback`, `is_identity_array_flat_map_expression`, `is_identity_array_map_callback`, `is_supported_for_of_array_iteration_item`, `is_truthy_array_literal`, `resolve_identity_array_callback_source`, `resolve_static_array_at_call`, `resolve_static_array_callback_identity_operand`, `resolve_static_array_callback_numeric_operand`, `resolve_static_array_callback_truthiness`, `resolve_static_array_callback_truthiness_expr`, `resolve_static_array_concat_element`, `resolve_static_array_filter_items`, `resolve_static_array_find_call`, `resolve_static_array_join_call`, `resolve_static_array_join_receiver`, `resolve_static_array_reduce_call`, `resolve_static_array_search_call`, `resolve_static_array_slice_bounds`, `resolve_static_array_slice_element`, `resolve_static_array_some_every_call`, `resolve_truthy_identity_array_filter_source`, `static_array_at_literal_receiver`, `static_array_is_array_result`.
- [ ] **Step 4:** In `lib.rs`: add `mod intrinsics;`. Build + test green, same count.
- [ ] **Step 5:** `git add -A && git commit -m "refactor(kali_codegen): extract intrinsics string+array [refactor]"`

---

### Task 7: Extract `intrinsics/math.rs` and `intrinsics/number.rs`

**Files:**
- Create: `crates/kali_codegen/src/intrinsics/math.rs`
- Create: `crates/kali_codegen/src/intrinsics/number.rs`
- Modify: `crates/kali_codegen/src/intrinsics/mod.rs`

**Interfaces:**
- Produces: `impl<'a> FunctionEmitter<'a>` clusters.

- [ ] **Step 1:** `intrinsics/math.rs` (`use crate::*;`, one `impl` block) — move these methods:
  `math_abs_import_index`, `math_abs_static_literal_value`, `math_atan2_zero_slice_value`, `math_cbrt_constant_root`, `math_clz32_import_index`, `math_clz32_static_literal_value`, `math_exp2_constant_value`, `math_exp_constant_value`, `math_expm1_constant_value`, `math_extrema_static_literal_value`, `math_fround_zero_constant_value`, `math_hyperbolic_zero_constant_value`, `math_hypot_constant_root`, `math_imul_import_index`, `math_imul_static_literal_value`, `math_inverse_hyperbolic_constant_value`, `math_inverse_trig_constant_value`, `math_log10_constant_exponent`, `math_log1p_constant_value`, `math_log2_constant_exponent`, `math_log_constant_value`, `math_max_import_index`, `math_member_method`, `math_min_import_index`, `math_pow_import_index`, `math_round_import_index`, `math_round_like_static_literal_value`, `math_sign_import_index`, `math_sign_static_literal_value`, `math_sin_cos_zero_constant_value`, `math_sqrt_constant_root`.
- [ ] **Step 2:** `intrinsics/number.rs` (`use crate::*;`, one `impl` block) — move these methods:
  `contains_negative_numeric_literal`, `contains_non_integer_numeric_literal`, `emit_integer_math_arg`, `global_number_predicate_callable_method`, `is_number_parse_callable`, `is_parse_float_callable`, `is_parse_int_callable`, `resolve_static_global_number_predicate_call`, `resolve_static_numeric_reducer_callback`, `resolve_static_numeric_reducer_expr`, `resolve_static_numeric_value`, `resolve_static_parse_float_call`, `resolve_static_parse_int_call`, `static_bigint_literal_value`, `to_uint32_literal_value`.
  Also move the number-parsing **free fns** `parse_number_literal`, `parse_numeric_literal_value`, `is_supported_static_ascii_char_code`, `static_parse_float_ascii_integer`, `static_parse_int_ascii` to the top level of this file.
- [ ] **Step 3:** In `intrinsics/mod.rs` add `mod math;` and `mod number;`. Build + test green, same count.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_codegen): extract intrinsics math+number [refactor]"`

---

### Task 8: Extract `intrinsics/object.rs`, `intrinsics/host.rs`, `intrinsics/collections.rs`

**Files:**
- Create: `crates/kali_codegen/src/intrinsics/object.rs`
- Create: `crates/kali_codegen/src/intrinsics/host.rs`
- Create: `crates/kali_codegen/src/intrinsics/collections.rs`
- Modify: `crates/kali_codegen/src/intrinsics/mod.rs`

**Interfaces:**
- Produces: `impl<'a> FunctionEmitter<'a>` clusters.

- [ ] **Step 1:** `intrinsics/object.rs` (`use crate::*;`, one `impl` block) — move:
  `collect_object_enumeration_iteration_items`, `collect_object_from_entries_iteration_items`, `is_math_object`, `is_number_object`, `is_object_enumeration_call`, `is_object_freeze_call`, `is_object_from_entries_call`, `is_object_has_own_call`, `is_object_identity_object`, `is_object_literal`, `object_literal_field`, `resolve_static_object_identity_value`, `resolve_transparent_object_root_node`, `static_object_from_entries_has_key`, `static_object_has_own`.
- [ ] **Step 2:** `intrinsics/host.rs` (`use crate::*;`, one `impl` block) — move:
  `console_import_index`, `cwd_import_index`, `cwd_set_import_index`, `emit_coverage_hit`, `env_delete_import_index`, `env_get_import_index`, `env_has_import_index`, `env_set_import_index`, `has_semver_import`, `is_console_assert`, `is_deno_args`, `is_deno_exit`, `is_deno_pid`, `is_kali_test_call`, `is_process_argv`, `is_process_cwd`, `is_process_exit`, `is_process_kill`, `is_process_pid`, `kali_test_callback_index`, `process_argv_slice_start`, `process_exit_import_index`, `render_console_arguments`, `render_console_call`, `render_length`, `render_package_json_version`, `render_package_json_version_access`, `render_semver_intrinsic`, `render_static_value`.
  Also move the **free fn** `semver_min_version` to the top level of this file.
- [ ] **Step 3:** `intrinsics/collections.rs` (`use crate::*;`, one `impl` block) — move:
  `collect_map_constructor_iteration_items`, `collect_set_constructor_iteration_items`, `is_map_constructor_call`, `is_set_constructor_call`, `resolve_map_constructor_call`, `resolve_set_constructor_call`, `static_map_entry_key`, `static_set_item_key`.
- [ ] **Step 4:** In `intrinsics/mod.rs` add `mod object;`, `mod host;`, `mod collections;`. Build + test green, same count.
- [ ] **Step 5:** `git add -A && git commit -m "refactor(kali_codegen): extract intrinsics object/host/collections [refactor]"`

---

### Task 9: Extract `lower.rs` (driver + program analysis) and verify the facade

**Files:**
- Create: `crates/kali_codegen/src/lower.rs`
- Modify: `crates/kali_codegen/src/lib.rs`

**Interfaces:**
- Produces (re-exported): `lower_lir_to_wasm`. Holds the program-level driver and the LIR-walking free fns.

- [ ] **Step 1:** Create `lower.rs` with `use crate::*;`. Move into it (all free fns, keeping `pub(crate)`/`pub` as set in Task 2):
  - `pub fn lower_lir_to_wasm` (the crate entry point).
  - `generator_lowering_unavailable_message`, `collect_functions`, `collect_functions_from_node`, `function_plan`, `is_function_like`, `collect_function_locals`, `collect_function_locals_from_node`, `top_level_children`.
  - `program_uses_env_get`, `program_uses_env_has`, `program_uses_env_set`, `program_uses_env_delete`, `program_uses_cwd_set`, `program_uses_process_exit`, `is_process_root`, `process_env_property_key`.
  - `emit_literal`, `encode_string_handle`.
- [ ] **Step 2:** In `lib.rs`: add `mod lower; pub use lower::lower_lir_to_wasm;`. The import-index `const`s (`TEST_REGISTER_IMPORT_INDEX` … `STRING_HANDLE_TAG`) remain in `lib.rs` and are referenced cross-module via `crate::<CONST>` — confirm references resolve; if any const is named unqualified from a moved module, either qualify it as `crate::<CONST>` at the call site (mechanical, no logic change) or keep `use crate::*;` which already re-exports them.
- [ ] **Step 3:** Verify `lib.rs` is now a thin facade:
```bash
cd /workspace
grep -nE "^(pub )?(struct|enum|impl|fn) " crates/kali_codegen/src/lib.rs || echo "facade clean — no definitions remain"
wc -l crates/kali_codegen/src/lib.rs
```
Expected: "facade clean" (only `mod`/`pub use` lines, the import-index `const`s, the `use` block, crate docs, and `#[cfg(test)]` wiring remain); `lib.rs` drops from 10,331 to roughly 70–110 lines.
- [ ] **Step 4:** Build + test green, same count.
- [ ] **Step 5:** `git add -A && git commit -m "refactor(kali_codegen): extract lower driver; reduce lib.rs to facade [refactor]"`

---

### Task 10: `cargo fmt` normalization

**Files:**
- Modify: all `crates/kali_codegen/src/**/*.rs`

- [ ] **Step 1:** Normalize formatting after the split:
```bash
cd /workspace
cargo fmt -p kali_codegen
```
- [ ] **Step 2:** Confirm still green and clippy-clean:
```bash
cargo test -p kali_codegen 2>&1 | tail -5
cargo clippy -p kali_codegen --all-targets -- -D warnings 2>&1 | tail -15
```
Expected: `test result: ok.`, same count; clippy reports no errors. If clippy flags a genuinely-now-private `pub(crate)` item, narrow that single item's visibility (no body change) and re-run.
- [ ] **Step 3:** `git add -A && git commit -m "style(kali_codegen): cargo fmt normalization after module split [refactor]"`

---

## Test co-location tasks (Tasks 11–13)

### Task 11: Add `kali_test_support` dev-dep and the crate-local `test_support` module (LIR builder + macros)

**Files:**
- Modify: `crates/kali_codegen/Cargo.toml` (add dev-dependency)
- Create: `crates/kali_codegen/src/test_support.rs`
- Modify: `crates/kali_codegen/src/lib.rs` (declare the module under `cfg(test)`)
- Modify: `crates/kali_codegen/src/tests.rs` (import from `test_support`, drop moved helpers)

**Interfaces:**
- Produces (all `pub(crate)`, available to every `*_tests.rs`):
  - The helpers migrated from `tests.rs`: `node(kind: LirNodeKind, text: Option<&str>, children: Vec<LirNodeId>) -> LirNode`, `sample_program() -> LirProgram`, `assert_nullish_assignment_lowers(source: &str)`, `assert_logical_assignment_lowers(source: &str)`, `assert_nullish_coalescing_lowers(source: &str)`, `compile_and_measure(program: &LirProgram) -> (Vec<u8>, usize)`, `wasm_instruction_count(bytes: &[u8]) -> usize`.
  - A `lir!{}` macro that builds a `LirProgram` from a flat node list, replacing the repetitive `LirNodeId(n)` + `nodes.push(node(...))` bookkeeping.

- [ ] **Step 1: Add the dev-dependency.** In `crates/kali_codegen/Cargo.toml`, under `[dev-dependencies]`, add:
```toml
kali_test_support = { workspace = true }
```

- [ ] **Step 2: Enumerate the existing local test helpers to migrate.**
```bash
cd /workspace
grep -nE "^fn (node|sample_program|assert_|compile_and_measure|wasm_instruction_count)" crates/kali_codegen/src/tests.rs
```
Expected: the seven helper fns listed in the Interfaces block.

- [ ] **Step 3: Create `test_support.rs`.** Move the seven helpers here verbatim, each declared `pub(crate)`. Header:
```rust
//! kali_codegen-specific test builders and macros (compiled under cfg(test)).
use crate::*;

// --- helpers migrated from tests.rs (each made pub(crate)) ---
// pub(crate) fn node(kind: LirNodeKind, text: Option<&str>, children: Vec<LirNodeId>) -> LirNode { ... }
// pub(crate) fn sample_program() -> LirProgram { ... }
// pub(crate) fn assert_nullish_assignment_lowers(source: &str) { ... }
// pub(crate) fn assert_logical_assignment_lowers(source: &str) { ... }
// pub(crate) fn assert_nullish_coalescing_lowers(source: &str) { ... }
// pub(crate) fn compile_and_measure(program: &LirProgram) -> (Vec<u8>, usize) { ... }
// pub(crate) fn wasm_instruction_count(bytes: &[u8]) -> usize { ... }

/// Build a `LirProgram` from a root id and a flat list of `(kind, text, children)`
/// tuples — collapses the repeated `LirNodeId(n)` + `nodes.push(node(...))` pattern.
///
/// `lir!(root: 0, nodes: [ (LirNodeKind::Program, None, vec![1]), (LirNodeKind::Value, Some("a"), vec![]) ])`
macro_rules! lir {
    (root: $root:expr, nodes: [ $( ($kind:expr, $text:expr, $children:expr) ),* $(,)? ]) => {{
        let mut nodes = Vec::new();
        $( nodes.push($crate::test_support::node($kind, $text, $children)); )*
        $crate::LirProgram { root: $crate::LirNodeId($root), nodes }
    }};
}
pub(crate) use lir;
```
> Before finalizing the macro, confirm `LirProgram`'s field names (`root`, `nodes`) and that `LirNode`/`LirNodeId`/`LirNodeKind` are re-exported on the crate root (they are imported in `lib.rs` from `kali_lir`). The `node()` helper sets `function_flavor: None`; if any test needs a non-`None` flavor it keeps building the `LirNode` literal directly rather than via the macro.

- [ ] **Step 4: Declare the module under cfg(test).** In `crates/kali_codegen/src/lib.rs`, just above the `#[path = "tests.rs"] mod tests;` wiring, add:
```rust
#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
```

- [ ] **Step 5: Update `tests.rs`.** Delete the seven now-moved helper fns from `tests.rs`; at the top replace `use super::*;` with:
```rust
use crate::*;
use crate::test_support::*;
use wasmparser::Validator;
```
Leave all `#[test]` fns unchanged for now (they still call the same helper names, resolved from `test_support`).

- [ ] **Step 6: Build and test.**
```bash
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: `test result: ok.`, same count as Task 1.

- [ ] **Step 7: Commit.**
```bash
git add crates/kali_codegen/Cargo.toml crates/kali_codegen/src/lib.rs crates/kali_codegen/src/test_support.rs crates/kali_codegen/src/tests.rs Cargo.lock
git commit -m "refactor(kali_codegen): add test_support (LIR builder + macros) [refactor]"
```

---

### Task 12: Split `tests.rs` into sibling `*_tests.rs` per module

**Files:**
- Create: `crates/kali_codegen/src/emit/emit_tests.rs`, `intrinsics/string_tests.rs`, `intrinsics/array_tests.rs`, `intrinsics/math_tests.rs`, `intrinsics/number_tests.rs`, `intrinsics/object_tests.rs`, `intrinsics/host_tests.rs`, `intrinsics/collections_tests.rs`, and (if any tests target them) `lower_tests.rs` / `ctx_tests.rs` (only those that receive tests)
- Modify: each corresponding source module (add the `#[cfg(test)] #[path] mod` wiring)
- Delete (at end): `crates/kali_codegen/src/tests.rs` and its facade wiring

**Classification rule:** For each `#[test]` fn, read its body and assign it to the module whose behavior it exercises. Name-prefix guide (verify against the body when ambiguous):

| Destination test file              | Test clusters (by name)                                  |
|------------------------------------|----------------------------------------------------------|
| `intrinsics/math_tests.rs`         | `math_*`, `*static*` math folding                        |
| `emit/emit_tests.rs`               | `for_*` (for-of iteration), update/logical/nullish/coalescing/exponentiation/compound/bitwise, generator/async lowering, `mixed_*`, `sample_*`, `wasm_*` |
| `intrinsics/object_tests.rs`       | `object_*`                                               |
| `intrinsics/host_tests.rs`         | `process_*`, `deno_*`, `console_*`, package/semver       |
| `intrinsics/array_tests.rs`        | `array_*`, `bracketed_*` array forms                     |
| `intrinsics/collections_tests.rs`  | `set_*`, `map_*`                                          |
| `intrinsics/number_tests.rs`       | `global_*`, parse-int/float, numeric predicates          |
| `intrinsics/string_tests.rs`       | string intrinsic tests                                   |

When a test drives the full pipeline through several intrinsics, place it with the **most specific** intrinsic it asserts on; pure `for-of`/control-flow lowering tests go to `emit/emit_tests.rs`. When still ambiguous, place with the source module whose method the test names most directly.

**Renaming rule:** Keep test names **identical** wherever possible so the Task 14 baseline diff is empty. If a collision arises after moving (two modules with same-named tests), rename minimally and record every rename for Task 14.

- [ ] **Step 1: Worklist.** Count tests to place:
```bash
cd /workspace
grep -cE "^fn [a-z0-9_]+\(\)" crates/kali_codegen/src/tests.rs
```
Expected: ≈324.

- [ ] **Step 2:** For each destination module `<m>`, create `<m>_tests.rs` with header:
```rust
use crate::*;
use crate::test_support::*;
use wasmparser::Validator;
```
(Drop the `Validator` import in files whose tests don't validate WASM; add any extra `kali_hir`/`kali_mir`/`kali_parser`/`kali_lexer` `use` lines the moved tests need — copy from the current `tests.rs` import block.)

- [ ] **Step 3:** Move the classified tests into their destination files. Wire each into its source module by adding at the **bottom of the source file**, e.g. in `intrinsics/math.rs`:
```rust
#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;
```
For directory modules the `#[path]` is relative to the source file's directory, so `intrinsics/math.rs` → `#[path = "math_tests.rs"]` resolves to `intrinsics/math_tests.rs`. For `emit/`, the test wiring goes in `emit/mod.rs` with `#[path = "emit_tests.rs"]`.

- [ ] **Step 4:** Move tests in batches by destination module, running after each and committing per module:
```bash
cargo test -p kali_codegen 2>&1 | tail -5
git add -A && git commit -m "test(kali_codegen): co-locate <module> tests [refactor]"
```
Expected after each batch: green, and the running total of co-located + remaining-in-`tests.rs` tests stays at ≈324.

- [ ] **Step 5:** When `tests.rs` has no `#[test]` fns left, delete it and remove its wiring:
```bash
rm crates/kali_codegen/src/tests.rs
```
Remove the `#[cfg(test)] #[path = "tests.rs"] mod tests;` lines from `lib.rs`.

- [ ] **Step 6:** `cargo test -p kali_codegen 2>&1 | tail -5`. Green, ≈324.
- [ ] **Step 7:** `git add -A && git commit -m "test(kali_codegen): remove monolithic tests.rs [refactor]"`

---

### Task 13: Adopt the LIR builder/fixtures where it reduces boilerplate

**Files:**
- Modify: the new `*_tests.rs` files

- [ ] **Step 1:** In the co-located test files, replace repeated hand-rolled `LirProgram` construction with the `lir!{}` macro (and `sample_program`/`compile_and_measure`/`wasm_instruction_count` helpers) where it shortens the test without obscuring intent. Where a test touches the filesystem or a manifest, use `kali_test_support::fixtures::{tempdir, write_file, write_manifest}`. Do **not** macro-ize tests where the explicit node list is clearer or where a node needs a non-`None` `function_flavor`.
- [ ] **Step 2:** `cargo test -p kali_codegen 2>&1 | tail -5`. Green, same count.
- [ ] **Step 3:** `git add -A && git commit -m "test(kali_codegen): adopt LIR builder/fixtures in co-located tests [refactor]"`

---

### Task 14: Final verification, lint, and baseline diff

**Files:**
- Create: `docs/superpowers/baselines/kali_codegen-tests-after.txt`
- Create: `docs/superpowers/baselines/kali_codegen-tests-renames.md`

- [ ] **Step 1: Regenerate the after-snapshot:**
```bash
cd /workspace
cargo test -p kali_codegen -- --list 2>/dev/null | grep ': test$' | sed 's/: test$//' | sort > docs/superpowers/baselines/kali_codegen-tests-after.txt
```

- [ ] **Step 2: Diff before vs after.** Every difference must be an intentional rename (ideally none):
```bash
diff docs/superpowers/baselines/kali_codegen-tests-before.txt docs/superpowers/baselines/kali_codegen-tests-after.txt
wc -l docs/superpowers/baselines/kali_codegen-tests-*.txt
```
Record any before→after renames in `docs/superpowers/baselines/kali_codegen-tests-renames.md` (write "No renames — test name set identical." if the diff is empty, aside from module-path prefixes which `--list` does not include). Confirm the **count is unchanged** — no test dropped.

- [ ] **Step 3: Format and lint:**
```bash
cargo fmt -p kali_codegen
cargo clippy -p kali_codegen --all-targets -- -D warnings 2>&1 | tail -15
```
Expected: no new clippy errors; warnings no worse than the pre-refactor baseline.

- [ ] **Step 4: Full-workspace sanity** (kali_codegen feeds `kali_cli`/`kali_runtime`/…):
```bash
cargo build --workspace 2>&1 | tail -5
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: workspace builds; kali_codegen tests green.

- [ ] **Step 5: Per-file size check** (confirm the monolith is gone):
```bash
find crates/kali_codegen/src -name '*.rs' | xargs wc -l | sort -rn | head -20
```
Expected: no single file near the old 10,331 / 8,024 line counts; modules are small and focused.

- [ ] **Step 6: Commit baselines:**
```bash
git add docs/superpowers/baselines/kali_codegen-tests-after.txt docs/superpowers/baselines/kali_codegen-tests-renames.md
git commit -m "test(kali_codegen): record post-refactor baseline + renames [refactor]"
```

- [ ] **Step 7: STOP for review.** Summarize: per-file line counts before/after, the rename mapping (ideally empty), and confirmation that the test count is unchanged and the suite + full workspace are green. Then update the `kali-crate-modularization` memory to mark `kali_codegen` done and note the next candidate crate.

---

## Self-Review Notes (for the implementer)

- If a moved item references a name that compiled only because everything lived in one module and you missed it in Task 2, fix it by adding `pub(crate)` to *that* item — never by altering a method body.
- `use super::*;` (top-level sibling modules: `ctx`, `emitter`, `lower`) vs `use crate::*;` (nested directory modules: `emit/`, `intrinsics/*`): both work for method-only modules; prefer `use crate::*;` inside directories to avoid `super` resolving to the directory's `mod.rs`.
- The import-index `const`s and the `wasm_encoder`/`kali_lir`/etc. `use` block stay in `lib.rs`; moved modules reach them via `use crate::*;`. If `use crate::*;` doesn't surface a bare `const`, qualify it as `crate::<CONST>` at the use site (mechanical, no logic change).
- The `#[path]` attribute for a sibling test file is resolved **relative to the directory of the file containing the `mod` declaration** — always the bare filename (`#[path = "math_tests.rs"]`), never a path with directories.
- Do not change any `Cargo.toml` dependency versions; the only manifest edit is the `kali_test_support` dev-dependency added in Task 11.
- Keep test names identical through the split so the Task 14 baseline diff stays empty; that is the strongest proof of zero behavior change.
