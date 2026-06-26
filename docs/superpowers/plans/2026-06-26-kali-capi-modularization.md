# kali_capi Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the monolithic `crates/kali_capi/src/lib.rs` (1203 lines) into a thin facade plus 4 public per-artifact modules (`header`, `metadata`, `manifest`, `bundle`) + 1 internal `validate` module, and split `src/tests.rs` (37 tests) into 5 co-located `*_tests.rs` files — zero behavior change, preserved public API.

**Architecture:** FLAT FUNCTION-PILE grouped by output artifact (a new shape for the series — `kali_capi` is a deterministic C-header/JSON-sidecar generator, not FFI). The facade re-exports each public family via a glob (`pub use <mod>::*;`) so every `kali_capi::Name` flat path is preserved → zero consumer edits. The crate-level `pub const HOST_ABI_VERSION` stays at the facade root. The 8 shared JSON field-validators move into an internal `validate` module (`pub(crate)`, **no glob**) — the one predicted widening (deno `path` precedent).

**Tech Stack:** Rust 2021, cargo workspace, `serde_json`, std only. Dep `kali_common` is declared but unused (left untouched — out of scope).

## Global Constraints

- **Zero behavior change, preserved public API.** The public surface must be identical: same names, same flat `kali_capi::Name` paths. The set is exactly `HOST_ABI_VERSION` (const) + `Export` (struct, with `Export::new`) + **31** public free fns = **33** flat names. The only visibility change is the 8 validators going private → `pub(crate)` (internal, never reachable as `kali_capi::…`).
- **No changes to any consumer.** Sole consumer: `crates/kali_cli/src/bin/kali.rs`, importing `arity_from_signature, generate_binding_package_manifest_with_provenance, generate_header, generate_metadata_with_provenance as generate_capi_metadata, parse_binding_package_manifest, parse_metadata, Export as CApiExport`. It must compile and pass **without edits**.
- **Facade stays logic-free** except the one item that must live at the crate root: `pub const HOST_ABI_VERSION: u32 = 2;` (referenced by `metadata` and `manifest` as `crate::HOST_ABI_VERSION`).
- **Functions are interleaved in the source — cut by item name, never by absolute line range.** After each task the line numbers shift; re-locate the next item with `grep -n 'fn <name>' src/lib.rs`.
- **Test co-location mechanics:** each module file ends with
  ```rust
  #[cfg(test)]
  #[path = "<name>_tests.rs"]
  mod <name>_tests;
  ```
  and `binding_tests` is declared the same way from the **facade** (`lib.rs`). **Each `*_tests.rs` begins with `use crate::*;`** (series convention — the facade glob-exports every public family + `HOST_ABI_VERSION`, so `crate::*` gives a test file all capi items regardless of which module owns them) **plus any explicit `std`/external imports its bodies reference** (`std::fs`, `std::path::PathBuf`, `std::process::Command`, `std::time::{SystemTime, UNIX_EPOCH}`, `serde_json::json`). **Self-sufficiency rule:** add every `std`/external import the moved bodies use — `cargo build` skips `cfg(test)`, so a missing import compiles under build but fails under `cargo test`. **Per-test-split task you MUST run `cargo test -p kali_capi`, not just build.** (The internal `validate` fns are `pub(crate)`, not re-exported, so `crate::*` does not expose them — fine, no test calls them directly.)
- **Per-task verification:** every task ends with `cargo build -p kali_capi` + `cargo test -p kali_capi` green, then a commit. Mid-plan unused-import warnings on the crate-root `use` block are acceptable; that block is trimmed in the finalize task once empty.
- **Module extraction order constraint:** `validate` (Task 1) MUST precede `metadata` (Task 3) and `manifest` (Task 4). `bundle` (Task 5) MUST follow both `metadata` and `manifest`.

### Item → module map (cut by name)

| module | public items | private items moved in |
|---|---|---|
| `header` | `Export` (+`impl Export`), `arity_from_signature`, `generate_header`, `sanitize_identifier` | — |
| `metadata` | `generate_metadata`, `generate_metadata_with_provenance`, `parse_metadata`, `cabi_metadata_summary`, `load_metadata`, `load_metadata_summary`, `discover_metadata_path`, `discover_metadata_path_with_name`, `load_metadata_from_root`, `load_metadata_summary_from_root`, `load_metadata_from_root_with_name`, `load_metadata_summary_from_root_with_name` | `validate_generated_cabi_metadata` |
| `manifest` | `generate_binding_package_manifest`, `generate_binding_package_manifest_with_provenance`, `parse_binding_package_manifest`, `binding_package_manifest_summary`, `discover_binding_package_manifest_path`, `discover_binding_package_manifest_path_with_name`, `load_binding_package_manifest`, `load_binding_package_manifest_summary`, `load_binding_package_manifest_from_root`, `load_binding_package_manifest_summary_from_root`, `load_binding_package_manifest_from_root_with_name`, `load_binding_package_manifest_summary_from_root_with_name` | `validate_generated_binding_package_manifest` |
| `bundle` | `binding_package_bundle_summary`, `load_binding_package_bundle_summary`, `load_binding_package_bundle_summary_from_root`, `load_binding_package_bundle_summary_from_root_with_name` | — |
| `validate` (internal) | — | `reject_unexpected_keys`, `validate_string_field`, `validate_non_empty_string_field`, `validate_integer_field`, `validate_non_negative_integer_field`, `integer_value`, `validate_host_abi_version_window`, `normalize_string_list_value` (all → `pub(crate)`) |

### Test → file map (37 tests)

| test file | wired from | count | tests |
|---|---|---|---|
| `header_tests.rs` | `header.rs` | 2 | `header_generation_produces_c_compatible_prototypes`, `identifier_sanitization_is_deterministic` |
| `metadata_tests.rs` | `metadata.rs` | 9 | `metadata_generation_includes_expected_artifacts`, `metadata_generation_with_provenance_keeps_optional_fields_deterministic`, `cabi_metadata_helpers_load_and_summarize_generated_payloads`, `cabi_metadata_helpers_discover_load_and_summarize_root_sidecars`, `cabi_metadata_helpers_reject_incompatible_host_abi_version_windows`, `cabi_metadata_helpers_reject_ambiguous_auto_discovery`, `cabi_metadata_parsing_rejects_unexpected_keys`, `cabi_metadata_parsing_rejects_negative_max_specializations`, `cabi_metadata_helpers_reject_empty_provenance_fields` |
| `manifest_tests.rs` | `manifest.rs` | 19 + helper | all `binding_package_manifest_*` tests (incl. `*_summary_*`, parsing/rejection, `*_helpers_*`) + the non-`#[test]` helper fn `valid_binding_package_manifest()` |
| `binding_tests.rs` | **`lib.rs`** | 7 | `python_binding_package_metadata_is_present`, `python_binding_wraps_generated_header_exports`, `python_binding_auto_discovers_stem_specific_binding_package_manifest`, `python_binding_rejects_incompatible_host_abi_metadata`, `python_unittest_smoke_covers_the_binding_helper_package`, `javascript_binding_package_metadata_is_present`, `javascript_node_test_smoke_covers_the_binding_helper_package` |

`bundle` has no standalone tests — covered inside `binding_package_manifest_helpers_load_discover_and_summarize_manifests` (in `manifest_tests.rs`).

---

### Task 1: Internal `validate` module (the shared helper — extract first)

**Files:**
- Create: `crates/kali_capi/src/validate.rs`
- Modify: `crates/kali_capi/src/lib.rs`

**Interfaces:**
- Produces (all `pub(crate)`, importable via `use crate::validate::{…}`):
  - `reject_unexpected_keys(value: &Value, context: &str, allowed: &[&str]) -> Result<(), String>`
  - `validate_string_field(value: &Value, context: &str, field_name: &str) -> Result<(), String>`
  - `validate_non_empty_string_field(value: &Value, context: &str, field_name: &str) -> Result<(), String>`
  - `validate_integer_field(value: &Value, context: &str, field_name: &str) -> Result<(), String>`
  - `validate_non_negative_integer_field(value: &Value, context: &str, field_name: &str) -> Result<(), String>`
  - `integer_value(value: &Value, context: &str, field_name: &str) -> Result<i128, String>`
  - `validate_host_abi_version_window(host_abi_version: &Value, min_host_abi_version: Option<&Value>, context: &str) -> Result<Value, String>`
  - `normalize_string_list_value(value: &Value, context: &str, field_name: &str) -> Result<Value, String>`
  - Consumed later by `metadata` (Task 3) and `manifest` (Task 4).

> Confirm each signature against the source before moving (`reject_unexpected_keys`'s exact params come from `grep -n -A4 'fn reject_unexpected_keys' src/lib.rs`). The signatures above are transcribed from the monolith; copy bodies verbatim, prefixing each `fn` with `pub(crate)`.

- [ ] **Step 1: Create `validate.rs` and move the 8 validators verbatim, each `fn` → `pub(crate) fn`**

Cut these 8 fns out of `lib.rs` (locate by name: `reject_unexpected_keys`, `validate_string_field`, `validate_non_empty_string_field`, `validate_integer_field`, `validate_non_negative_integer_field`, `integer_value`, `validate_host_abi_version_window`, `normalize_string_list_value`). New file:

```rust
//! Shared JSON field-validators used by the `metadata` and `manifest` families.
//!
//! Internal only — `pub(crate)`, intentionally NOT glob-exported by the facade.

use serde_json::Value;

pub(crate) fn reject_unexpected_keys(/* exact params from source */) -> Result<(), String> {
    // ... body moved verbatim from lib.rs
}

// ... the other 7 validators, each `fn` → `pub(crate) fn`, bodies verbatim ...
```

These validators reference only `serde_json::Value` and one another (e.g. `validate_host_abi_version_window` calls `validate_integer_field` + `integer_value`; verified: none reference `HOST_ABI_VERSION`). No `std::fs`/`std::path` needed.

- [ ] **Step 2: Wire the module into `lib.rs` and point callers at it**

At the top of `lib.rs` (after the crate doc / `use` block), add:

```rust
mod validate;
```

Do **not** add a `pub use validate::*;` glob — it stays internal. Then, for every remaining fn in `lib.rs` that called a validator (the `parse_metadata`, `cabi_metadata_summary`, `parse_binding_package_manifest`, `binding_package_manifest_summary` bodies and their helpers), the calls now need the path. Simplest: add `use crate::validate::*;` near the top of `lib.rs` (these callers all still live in `lib.rs` at this point — they move out in Tasks 3–4, taking the import with them).

- [ ] **Step 3: Build**

Run: `cargo build -p kali_capi`
Expected: PASS (warnings about unused imports are acceptable; there should be none yet since validators are still used by lib.rs bodies).

- [ ] **Step 4: Test**

Run: `cargo test -p kali_capi`
Expected: PASS — all 37 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_capi/src/validate.rs crates/kali_capi/src/lib.rs
git commit -m "refactor(kali_capi): extract internal validate module [refactor]"
```

---

### Task 2: `header` module (independent; the only public type)

**Files:**
- Create: `crates/kali_capi/src/header.rs`
- Create: `crates/kali_capi/src/header_tests.rs`
- Modify: `crates/kali_capi/src/lib.rs`
- Source: split `crates/kali_capi/src/tests.rs`

**Interfaces:**
- Produces (glob-exported by facade → preserved as `kali_capi::…`):
  - `pub struct Export { pub name: String, pub arity: usize }` + `impl Export { pub fn new(name: impl Into<String>, arity: usize) -> Self }`
  - `pub fn arity_from_signature(signature: &str) -> usize`
  - `pub fn generate_header(module_name: &str, exports: &[Export]) -> String`
  - `pub fn sanitize_identifier(name: &str) -> String`

- [ ] **Step 1: Create `header.rs` and move the 4 items verbatim**

Cut `Export` (struct + `impl Export`), `arity_from_signature`, `generate_header`, `sanitize_identifier` out of `lib.rs`. These use only `String`/slice ops — no `serde_json`, no `std::fs`/`std::path`. New file:

```rust
//! C header text generation: the `Export` descriptor and the deterministic
//! header/identifier emitters.

/// Description of an exported entrypoint in the generated C header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// Exported symbol name.
    pub name: String,
    /// Number of C arguments emitted for the prototype.
    pub arity: usize,
}

impl Export {
    /// Create a new export description.
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self { name: name.into(), arity }
    }
}

// arity_from_signature, generate_header, sanitize_identifier — bodies verbatim from lib.rs
```

- [ ] **Step 2: Create `header_tests.rs` with the 2 header tests**

Move `header_generation_produces_c_compatible_prototypes` and `identifier_sanitization_is_deterministic` out of `tests.rs`. New file:

```rust
use crate::*;

#[test]
fn header_generation_produces_c_compatible_prototypes() {
    // ... body verbatim from tests.rs
}

#[test]
fn identifier_sanitization_is_deterministic() {
    // ... body verbatim from tests.rs
}
```

`crate::*` exposes `Export`, `generate_header`, `sanitize_identifier` (re-exported from `header`). These two tests reference no `std::fs`/`Command`/etc., so `use crate::*;` alone suffices — verify by reading the bodies.

- [ ] **Step 3: Wire module + co-located tests in `lib.rs`**

Add to `lib.rs`:

```rust
mod header;
pub use header::*;
```

Append the test wiring at the **end of `header.rs`**:

```rust
#[cfg(test)]
#[path = "header_tests.rs"]
mod header_tests;
```

- [ ] **Step 4: Build**

Run: `cargo build -p kali_capi`
Expected: PASS.

- [ ] **Step 5: Test**

Run: `cargo test -p kali_capi`
Expected: PASS — all 37 tests (2 now run via `header::header_tests`).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_capi/src/header.rs crates/kali_capi/src/header_tests.rs crates/kali_capi/src/lib.rs crates/kali_capi/src/tests.rs
git commit -m "refactor(kali_capi): extract header module, co-locate tests [refactor]"
```

---

### Task 3: `metadata` module (depends on `validate` + `crate::HOST_ABI_VERSION`)

**Files:**
- Create: `crates/kali_capi/src/metadata.rs`
- Create: `crates/kali_capi/src/metadata_tests.rs`
- Modify: `crates/kali_capi/src/lib.rs`
- Source: split `crates/kali_capi/src/tests.rs`

**Interfaces:**
- Consumes: `crate::validate::{reject_unexpected_keys, validate_string_field, validate_non_empty_string_field, validate_integer_field, validate_non_negative_integer_field, integer_value, validate_host_abi_version_window, normalize_string_list_value}` (only the subset metadata actually calls), `crate::HOST_ABI_VERSION`.
- Produces (glob-exported): `generate_metadata`, `generate_metadata_with_provenance`, `parse_metadata`, `cabi_metadata_summary`, `load_metadata`, `load_metadata_summary`, `discover_metadata_path`, `discover_metadata_path_with_name`, `load_metadata_from_root`, `load_metadata_summary_from_root`, `load_metadata_from_root_with_name`, `load_metadata_summary_from_root_with_name`. Plus private `validate_generated_cabi_metadata` (stays private to this module).

- [ ] **Step 1: Create `metadata.rs` and move the 12 public fns + 1 private verbatim**

Cut these out of `lib.rs` by name: the 12 public fns above + `validate_generated_cabi_metadata`. Header of new file:

```rust
//! cabi-metadata sidecar: generation, parsing, summarizing, loading, discovery.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::validate::{
    integer_value, normalize_string_list_value, reject_unexpected_keys,
    validate_host_abi_version_window, validate_integer_field, validate_non_empty_string_field,
    validate_non_negative_integer_field, validate_string_field,
};
use crate::HOST_ABI_VERSION;

// ... the 12 public fns + private validate_generated_cabi_metadata, bodies verbatim ...
```

> Trim the `use crate::validate::{…}` import to exactly the validators `metadata` calls (remove any the compiler flags as unused). `parse_metadata`/`cabi_metadata_summary` reference the validators; the generate fns reference `HOST_ABI_VERSION`.

- [ ] **Step 2: Create `metadata_tests.rs` with the 9 metadata tests**

Move these 9 from `tests.rs`: `metadata_generation_includes_expected_artifacts`, `metadata_generation_with_provenance_keeps_optional_fields_deterministic`, `cabi_metadata_helpers_load_and_summarize_generated_payloads`, `cabi_metadata_helpers_discover_load_and_summarize_root_sidecars`, `cabi_metadata_helpers_reject_incompatible_host_abi_version_windows`, `cabi_metadata_helpers_reject_ambiguous_auto_discovery`, `cabi_metadata_parsing_rejects_unexpected_keys`, `cabi_metadata_parsing_rejects_negative_max_specializations`, `cabi_metadata_helpers_reject_empty_provenance_fields`.

```rust
use crate::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn metadata_generation_includes_expected_artifacts() {
    // ... verbatim
}
// ... remaining 8 tests verbatim ...
```

> **Self-sufficiency:** `crate::*` already exposes every capi item these tests call (`generate_metadata*`, `parse_metadata`, `cabi_metadata_summary`, `discover_*`, `load_*`, and `HOST_ABI_VERSION`). Add only the `std` imports the moved bodies use — the originals leaned on the file-level `use std::fs; use std::path::PathBuf;` in `tests.rs`. Read each body and match its `std`/`serde_json` usage.

- [ ] **Step 3: Wire module + tests**

Add to `lib.rs`:

```rust
mod metadata;
pub use metadata::*;
```

Append to end of `metadata.rs`:

```rust
#[cfg(test)]
#[path = "metadata_tests.rs"]
mod metadata_tests;
```

- [ ] **Step 4: Build**

Run: `cargo build -p kali_capi`
Expected: PASS. Fix any "unused import" by trimming the `use crate::validate::{…}` list to what metadata actually calls.

- [ ] **Step 5: Test**

Run: `cargo test -p kali_capi`
Expected: PASS — all 37 tests. (If a test fails to compile with "cannot find `<symbol>`", it is a missing `std`/`serde_json` import — `crate::*` covers all capi items; add the std import the body uses.)

- [ ] **Step 6: Commit**

```bash
git add crates/kali_capi/src/metadata.rs crates/kali_capi/src/metadata_tests.rs crates/kali_capi/src/lib.rs crates/kali_capi/src/tests.rs
git commit -m "refactor(kali_capi): extract metadata module, co-locate tests [refactor]"
```

---

### Task 4: `manifest` module (depends on `validate` + `crate::HOST_ABI_VERSION`)

**Files:**
- Create: `crates/kali_capi/src/manifest.rs`
- Create: `crates/kali_capi/src/manifest_tests.rs`
- Modify: `crates/kali_capi/src/lib.rs`
- Source: split `crates/kali_capi/src/tests.rs`

**Interfaces:**
- Consumes: `crate::validate::{…}` (subset manifest calls), `crate::HOST_ABI_VERSION`.
- Produces (glob-exported): `generate_binding_package_manifest`, `generate_binding_package_manifest_with_provenance`, `parse_binding_package_manifest`, `binding_package_manifest_summary`, `discover_binding_package_manifest_path`, `discover_binding_package_manifest_path_with_name`, `load_binding_package_manifest`, `load_binding_package_manifest_summary`, `load_binding_package_manifest_from_root`, `load_binding_package_manifest_summary_from_root`, `load_binding_package_manifest_from_root_with_name`, `load_binding_package_manifest_summary_from_root_with_name`. Plus private `validate_generated_binding_package_manifest` (stays private to this module). Consumed later by `bundle` (Task 5).

- [ ] **Step 1: Create `manifest.rs` and move the 12 public fns + 1 private verbatim**

Cut by name: the 12 public manifest fns above + `validate_generated_binding_package_manifest`. Header:

```rust
//! binding-package manifest sidecar: generation, parsing, summarizing, loading, discovery.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::validate::{
    integer_value, normalize_string_list_value, reject_unexpected_keys,
    validate_host_abi_version_window, validate_integer_field, validate_non_empty_string_field,
    validate_non_negative_integer_field, validate_string_field,
};
use crate::HOST_ABI_VERSION;

// ... the 12 public fns + private validate_generated_binding_package_manifest, verbatim ...
```

> `generate_binding_package_manifest_with_provenance` uses `serde_json::Map` (built via `serde_json::Map::new()`) and `HOST_ABI_VERSION`. The dir-scan body of `discover_binding_package_manifest_path_with_name` is self-contained (duplicates metadata's scan — do **not** try to share it). Trim the `use crate::validate::{…}` list to what compiles without warnings.

- [ ] **Step 2: Create `manifest_tests.rs` with the 19 manifest tests + the helper**

Move from `tests.rs`, in order: `binding_package_manifest_orders_and_deduplicates_glue_deterministically`, `binding_package_manifest_with_provenance_uses_explicit_contract_labels`, `binding_package_manifest_helpers_load_discover_and_summarize_manifests`, the helper fn `valid_binding_package_manifest()`, `binding_package_manifest_parsing_normalizes_string_lists`, `binding_package_manifest_parsing_rejects_whitespace_padded_string_lists`, `binding_package_manifest_parsing_rejects_whitespace_padded_artifact_paths`, `binding_package_manifest_summary_normalizes_string_lists`, `binding_package_manifest_parsing_rejects_non_integer_max_specializations`, `binding_package_manifest_parsing_rejects_negative_max_specializations`, `binding_package_manifest_parsing_rejects_unexpected_keys`, `binding_package_manifest_parsing_rejects_invalid_required_field_types`, `binding_package_manifest_parsing_rejects_non_string_provenance_fields`, `binding_package_manifest_helpers_reject_whitespace_padded_module_name`, `binding_package_manifest_helpers_reject_empty_provenance_fields`, `binding_package_manifest_helpers_reject_empty_or_whitespace_artifact_paths`, `binding_package_manifest_rejects_incompatible_host_abi_version_window`, `binding_package_manifest_summary_rejects_invalid_required_field_types`, `binding_package_manifest_summary_rejects_non_string_provenance_fields`, `binding_package_manifest_helpers_reject_ambiguous_auto_discovery`.

```rust
use crate::*;
use serde_json::json;     // valid_binding_package_manifest() builds a Value via json!/Map
use std::fs;
use std::path::PathBuf;

fn valid_binding_package_manifest() -> serde_json::Value {
    // ... helper body verbatim
}

#[test]
fn binding_package_manifest_orders_and_deduplicates_glue_deterministically() {
    // ... verbatim
}
// ... remaining 17 tests verbatim ...
```

> `binding_package_manifest_helpers_load_discover_and_summarize_manifests` calls `load_binding_package_bundle_summary*` — those `bundle` fns do not exist until Task 5. **This test will not compile in this task.** Handle it: move this single test in **Task 5** (with `bundle`), not here. In Task 4 move the other **18** manifest tests + the helper, and leave `binding_package_manifest_helpers_load_discover_and_summarize_manifests` in `tests.rs` for now. (Re-confirm: `grep -n 'bundle_summary' src/tests.rs` shows all bundle calls are inside that one test.)
>
> **Self-sufficiency:** `crate::*` covers every capi symbol these tests call (`HOST_ABI_VERSION`, `parse_*`, `generate_*`, `*_summary`, `cabi_metadata_summary`, etc.). Add only the `std`/`serde_json` imports the bodies use (`fs`, `PathBuf`, `json!`).

- [ ] **Step 3: Wire module + tests**

Add to `lib.rs`:

```rust
mod manifest;
pub use manifest::*;
```

Append to end of `manifest.rs`:

```rust
#[cfg(test)]
#[path = "manifest_tests.rs"]
mod manifest_tests;
```

- [ ] **Step 4: Build**

Run: `cargo build -p kali_capi`
Expected: PASS (trim unused `validate` imports).

- [ ] **Step 5: Test**

Run: `cargo test -p kali_capi`
Expected: PASS — all 37 tests (the 18 moved manifest tests now run via `manifest::manifest_tests`; the 1 bundle-touching test still runs from `tests.rs`).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_capi/src/manifest.rs crates/kali_capi/src/manifest_tests.rs crates/kali_capi/src/lib.rs crates/kali_capi/src/tests.rs
git commit -m "refactor(kali_capi): extract manifest module, co-locate tests [refactor]"
```

---

### Task 5: `bundle` module (depends on `metadata` + `manifest` public surface)

**Files:**
- Create: `crates/kali_capi/src/bundle.rs`
- Modify: `crates/kali_capi/src/manifest_tests.rs` (add the one bundle-touching test)
- Modify: `crates/kali_capi/src/lib.rs`
- Source: split `crates/kali_capi/src/tests.rs`

**Interfaces:**
- Consumes (all public, via `crate::…`): `binding_package_manifest_summary`, `load_binding_package_manifest`, `discover_binding_package_manifest_path_with_name` (from `manifest`); `cabi_metadata_summary`, `load_metadata` (from `metadata`).
- Produces (glob-exported): `binding_package_bundle_summary`, `load_binding_package_bundle_summary`, `load_binding_package_bundle_summary_from_root`, `load_binding_package_bundle_summary_from_root_with_name`.

- [ ] **Step 1: Create `bundle.rs` and move the 4 fns verbatim**

Cut the 4 `*bundle_summary*` fns out of `lib.rs`. Header:

```rust
//! Binding-package "bundle" = manifest + metadata combined summary.
//! Composes the public surface of the `manifest` and `metadata` families.

use serde_json::Value;
use std::path::Path;

use crate::manifest::{
    binding_package_manifest_summary, discover_binding_package_manifest_path_with_name,
    load_binding_package_manifest,
};
use crate::metadata::{cabi_metadata_summary, load_metadata};

// ... the 4 fns verbatim ...
```

> Verify the exact consumed set against the bodies (`grep -n -A20 'fn load_binding_package_bundle_summary' src/lib.rs`): `binding_package_bundle_summary` calls `binding_package_manifest_summary` + `cabi_metadata_summary`; `load_binding_package_bundle_summary` calls `load_binding_package_manifest` + `binding_package_manifest_summary` + `load_metadata`; `load_binding_package_bundle_summary_from_root_with_name` calls `discover_binding_package_manifest_path_with_name`. Trim imports to what compiles warning-free.

- [ ] **Step 2: Move the bundle-touching test into `manifest_tests.rs`**

Move `binding_package_manifest_helpers_load_discover_and_summarize_manifests` from `tests.rs` into `manifest_tests.rs` (append after the other manifest tests). It now compiles: `bundle`'s fns exist and are glob-exported, so the file's `use crate::*;` (added in Task 4) already exposes every capi symbol the test calls — `load_binding_package_bundle_summary*` (bundle), `cabi_metadata_summary` / `load_metadata` (metadata), `binding_package_manifest_summary` / `generate_binding_package_manifest` / `discover_*` / `load_binding_package_manifest` (manifest), and `HOST_ABI_VERSION`. The **only** new import this body needs (not already at the top of `manifest_tests.rs`) is `std::time` for the temp-dir stamp:

```rust
use std::time::{SystemTime, UNIX_EPOCH};
```

(`use crate::*;`, `use serde_json::json;`, `use std::fs;`, `use std::path::PathBuf;` are already at the top from Task 4. `env!("CARGO_MANIFEST_DIR")` is a macro; `std::env::temp_dir()` is called fully-qualified. Re-read the body and add anything else it references.)

- [ ] **Step 3: Wire module in `lib.rs`**

Add to `lib.rs`:

```rust
mod bundle;
pub use bundle::*;
```

(`bundle` has no co-located test file — its coverage lives in `manifest_tests.rs`.)

- [ ] **Step 4: Build**

Run: `cargo build -p kali_capi`
Expected: PASS.

- [ ] **Step 5: Test**

Run: `cargo test -p kali_capi`
Expected: PASS — all 37 tests. `tests.rs` now contains only the 7 cross-cutting binding tests.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_capi/src/bundle.rs crates/kali_capi/src/manifest_tests.rs crates/kali_capi/src/lib.rs crates/kali_capi/src/tests.rs
git commit -m "refactor(kali_capi): extract bundle module, relocate bundle test [refactor]"
```

---

### Task 6: Finalize facade + co-locate the cross-cutting `binding_tests`

**Files:**
- Create: `crates/kali_capi/src/binding_tests.rs`
- Delete: `crates/kali_capi/src/tests.rs`
- Modify: `crates/kali_capi/src/lib.rs`

**Interfaces:**
- Produces: a thin `lib.rs` whose only non-`mod`/non-`use` content is `pub const HOST_ABI_VERSION: u32 = 2;`.

- [ ] **Step 1: Move the 7 remaining tests into `binding_tests.rs`**

`tests.rs` now holds exactly the 7 cross-cutting tests. Move them into a new `binding_tests.rs`:

```rust
use crate::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn python_binding_package_metadata_is_present() {
    // ... verbatim
}
// ... python_binding_wraps_generated_header_exports,
//     python_binding_auto_discovers_stem_specific_binding_package_manifest,
//     python_binding_rejects_incompatible_host_abi_metadata,
//     python_unittest_smoke_covers_the_binding_helper_package,
//     javascript_binding_package_metadata_is_present,
//     javascript_node_test_smoke_covers_the_binding_helper_package — all verbatim ...
```

`use crate::*;` exposes `generate_header`, `Export`, `generate_metadata`, etc. through the facade globs (here `binding_tests` is declared from `lib.rs`, so `crate::*` and `super::*` would coincide — use `crate::*` for series consistency). Keep the four file-level `std` imports from the original `tests.rs` (`fs`, `PathBuf`, `Command`, `SystemTime`/`UNIX_EPOCH`) — these bodies use `env!("CARGO_MANIFEST_DIR")`, `std::env::temp_dir()`, `std::process::id()`. Read each body and ensure every referenced `std` symbol is imported (self-sufficiency).

- [ ] **Step 2: Replace the old test wiring in `lib.rs` and delete `tests.rs`**

In `lib.rs`, replace the trailing
```rust
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```
with
```rust
#[cfg(test)]
#[path = "binding_tests.rs"]
mod binding_tests;
```
Then delete `tests.rs`.

- [ ] **Step 3: Trim the crate-root `use` block and confirm the thin facade**

The crate-root `use serde_json::{json, Value};`, `use std::fs;`, `use std::path::{Path, PathBuf};` are now unused (all code that used them moved out). Remove them. Confirm `lib.rs` consists of exactly:
- the `//!` crate doc,
- `pub const HOST_ABI_VERSION: u32 = 2;`,
- `mod validate;` (no glob),
- `mod header; pub use header::*;`
- `mod metadata; pub use metadata::*;`
- `mod manifest; pub use manifest::*;`
- `mod bundle; pub use bundle::*;`
- the `#[cfg(test)] #[path = "binding_tests.rs"] mod binding_tests;` wiring.

Verify: `cargo build -p kali_capi 2>&1 | grep -c warning` → expect `0`.

- [ ] **Step 4: Build + full test**

Run: `cargo build -p kali_capi && cargo test -p kali_capi`
Expected: PASS — all 37 tests, zero warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_capi/src/binding_tests.rs crates/kali_capi/src/lib.rs
git rm crates/kali_capi/src/tests.rs
git commit -m "refactor(kali_capi): finalize facade, co-locate binding tests, delete tests.rs [refactor]"
```

---

### Task 7: Whole-workspace verification + public-API proof

**Files:** none (verification only).

- [ ] **Step 1: Consumer compiles unchanged**

Run: `cargo build -p kali_cli && cargo test -p kali_cli`
Expected: PASS with **no edits** to `kali_cli` — proves the 7 flat names it imports are preserved.

- [ ] **Step 2: Whole-workspace green**

Run: `cargo build && cargo test`
Expected: PASS across the workspace.

- [ ] **Step 3: Basename-multiset proof (series invariant)**

Confirm the public surface is unchanged. Compare the set of `pub` items reachable as `kali_capi::Name` before vs after — expect **33** flat names (1 const + 1 struct + 31 fns), and the 8 validators absent from the public surface:

```bash
# the 8 validators must NOT be reachable publicly:
for v in reject_unexpected_keys validate_string_field validate_non_empty_string_field \
         validate_integer_field validate_non_negative_integer_field integer_value \
         validate_host_abi_version_window normalize_string_list_value; do
  grep -rq "pub fn $v" crates/kali_capi/src/validate.rs && echo "BAD: $v is pub" || true
done
# confirm validate.rs uses pub(crate), never pub:
grep -c 'pub(crate) fn' crates/kali_capi/src/validate.rs   # expect 8
grep -c '^pub fn'        crates/kali_capi/src/validate.rs   # expect 0
```

Also eyeball that `git grep -n 'pub fn ' crates/kali_capi/src/{header,metadata,manifest,bundle}.rs | wc -l` totals **31** and `pub struct Export` + `pub const HOST_ABI_VERSION` each appear once.

- [ ] **Step 4: Integrate to local main (no push to origin)**

```bash
git checkout main
git merge --ff-only <feature-branch>
cargo build && cargo test    # re-verify on merged main
git branch -d <feature-branch>
```

Do **not** push to origin (matches crates 2–10, 12, 13).

- [ ] **Step 5: No commit needed** — verification + integration only.

---

## Self-Review

**Spec coverage:**
- 4 public modules (`header`/`metadata`/`manifest`/`bundle`) → Tasks 2–5. ✓
- Internal `validate` module, `pub(crate)`, no glob → Task 1. ✓
- `HOST_ABI_VERSION` kept at facade root → Task 6 Step 3. ✓
- Glob facade preserves flat paths; consumer unchanged → Task 7 Steps 1, 3. ✓
- The one widening (`validate`) → Task 1; verified-clean-elsewhere (discover dup, bundle public-only, header no-validators) reflected in Task 4/5 notes. ✓
- 5-file test split incl. crate-level `binding_tests`; bundle has no standalone tests → Tasks 2–6. ✓
- No E0255 risk (module names don't clash with `std::fs`/`std::path`) → no collision step needed; crate-root `use` removed in Task 6. ✓
- Local-main-only integration → Task 7 Step 4. ✓
- Unused `kali_common` dep left untouched (out of scope) → not modified by any task. ✓

**Placeholder scan:** No "TBD"/"add error handling"/"similar to". Bodies are "move verbatim" (this is a pure code-motion refactor — the canonical content is the existing source, referenced by exact item name; reproducing 1200 lines inline would be error-prone, so each task names the exact items to cut and the exact imports/visibility changes to apply). ✓

**Type consistency:** `validate` signatures in Task 1 match the `Consumes` blocks in Tasks 3–4; bundle's consumed fns in Task 5 match `manifest`/`metadata` `Produces` in Tasks 3–4. Test counts sum to 37: header 2 + metadata 9 + manifest 19 (18 in Task 4 + 1 relocated in Task 5) + binding 7. The bundle-touching manifest test moves Task 4→5; all test files use `use crate::*;` so no per-test capi-import bookkeeping is needed (only `std`/`serde_json` imports). ✓

**Known cross-task subtlety (called out in-task):** the manifest test `binding_package_manifest_helpers_load_discover_and_summarize_manifests` touches `bundle` fns, so it is deferred from Task 4 to Task 5 — Task 4 moves 18 manifest tests + the helper and leaves this one in `tests.rs`; Task 5 relocates it into `manifest_tests.rs` with a crate-root `use`.
