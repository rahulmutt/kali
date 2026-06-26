# kali_sandbox Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the 937-line `crates/kali_sandbox/src/lib.rs` and the 1078-line `crates/kali_sandbox/src/effects.rs` into thin facades plus focused per-concern modules, with zero behavior change and a byte-identical public API.

**Architecture:** Pure code-motion. `lib.rs` splits into 8 sibling modules (`policy`, `operation`, `predicate`, `validation`, `loading`, `enforcement`, `matching`, `diagnostics`); the big `impl SandboxPolicy` block is dismantled into per-concern `impl SandboxPolicy { … }` blocks. `effects.rs` becomes `effects/mod.rs` over 4 submodules (`report`, `inference`, `scan`, `compare`). Both `lib.rs` and `effects/mod.rs` end as declarations + re-exports only. The existing 41-test suite (`tests.rs`) is the regression oracle and must stay green after every task.

**Tech Stack:** Rust (edition 2021), Cargo workspace. Dependencies: `kali_ast`, `kali_common` (`FileId`, `Span`), `kali_error` (`Diagnostic`, `_error_codes::{e4, e5, e8, e9}`), `kali_lexer` (`Lexer`, `Token`, `TokenType`), `kali_parser` (`Parser`), `serde`, `serde_json`.

## Global Constraints

- **Verbatim moves only.** Type/method/fn bodies are moved byte-identical (cut from the source file, paste into the new module). Do NOT retype, reformat, reorder, or "improve" any moved code. The only edits permitted are: visibility prefixes (Task 1 and the per-task widenings called out explicitly), `mod`/`use` wiring, and re-export lines.
- **Do NOT run `cargo fmt`.** The repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates). Verbatim moves + the mandated `pub(crate)` prefix may push some lines >100 cols or leave stray blank lines — these are accepted cosmetic minors, not regressions. Running fmt would violate the verbatim mandate.
- **Every task ends green:** `cargo build -p kali_sandbox` with **0 warnings** and `cargo test -p kali_sandbox` showing **41 passed**. Remove any `use` line that goes unused as code leaves the source file (the build will flag it); add any `use` a moved item now needs.
- **Shared types are imported from `crate::` re-exports**, mirroring the existing `effects.rs` convention (`use crate::{AccessRule, PatternKind, SandboxPolicy};`). Methods are called via `self.` and need no import.
- **Public surface stays byte-identical.** Crate-root `pub`: `SandboxPolicy`, `EffectsPolicy`, `FileSystemPolicy`, `NetworkPolicy`, `ProcessPolicy`, `TimerPolicy`, `ResourceLimits`, `AccessRule`, `HostOperation`, `PolicyPredicateContext`, `PolicyPredicateRegistry`, `HostPredicate`, `PolicyValidation`, plus their existing `pub` methods; `pub mod effects;` and the exact 12-symbol `pub use effects::{…}`. `pub(crate)`: `PatternKind`, `AccessRule::allows_candidate`. No module exposes any other `pub` item.
- **Commit message convention:** `refactor(kali_sandbox): <description> [refactor]`.
- **Integration:** work on branch `refactor/kali-sandbox-modularization` off `main`. Local-main ff-merge only — NEVER push to origin. (Branch is created in Task 1 Step 0; the final ff-merge is Task 10.)
- **Known pre-existing failure (not a regression):** `kali_cli` integration test `array_from_bracketed_root_wrappers` has 2 `build_bundles_*` failures (codegen/bundling). Confirm reproduction on the branch base; never attribute to this refactor. `kali_sandbox` has zero consumer edits.

---

## File Structure (end state)

`lib.rs` modules:
- `crates/kali_sandbox/src/lib.rs` — thin facade: crate doc, `pub mod effects;`, 8 `mod` decls, `pub use`/`pub(crate) use` re-exports, test wiring.
- `crates/kali_sandbox/src/policy.rs` — `SandboxPolicy`, `EffectsPolicy`, `FileSystemPolicy`, `NetworkPolicy`, `ProcessPolicy`, `TimerPolicy`, `ResourceLimits`, `AccessRule` (type defs), `default_schema_version`, `default_base_dir`.
- `crates/kali_sandbox/src/operation.rs` — `HostOperation`, `PolicyPredicateContext`, `impl PolicyPredicateContext { from_operation }`.
- `crates/kali_sandbox/src/predicate.rs` — `PolicyPredicateRegistry`, `RegisteredPredicate`, `HostPredicate`, `Default` impl, `enabled`/`disabled`/`is_enabled`/`register`/`evaluate`.
- `crates/kali_sandbox/src/validation.rs` — `PolicyValidation`, `impl SandboxPolicy { validate, validate_with_runtime_profiles, validate_policy }`, `validate_positive_u64`, `validate_zero_capable_u64`.
- `crates/kali_sandbox/src/loading.rs` — `impl SandboxPolicy { from_file, from_file_with_runtime_profiles, to_canonical_json, to_canonical_json_bytes, to_embedded_json_bytes }`.
- `crates/kali_sandbox/src/enforcement.rs` — `impl SandboxPolicy { check_operation, check_operation_with_predicates, check_path_access, check_url_access, check_exact_access, effective_thread_budget, effective_spawn_budget, network_max_connections }`.
- `crates/kali_sandbox/src/matching.rs` — `impl AccessRule { is_enabled, allows_path, allows_candidate }`, `PatternKind`, `resolve_pattern`, `normalize_text`, `glob_match`, `glob_match_inner`.
- `crates/kali_sandbox/src/diagnostics.rs` — `sandbox_violation`, `unavailable_capability`, `host_predicate_violation`, `resource_limit_violation`.

`effects` submodules:
- `crates/kali_sandbox/src/effects/mod.rs` — thin facade: 4 `mod` decls + `pub use` re-exports (was `effects.rs`).
- `crates/kali_sandbox/src/effects/report.rs` — report data types + `EffectAnalysisContext` impl + `effect_report_from_inference` + `package_effects_report` + `normalize_semantic_axis` + `normalize_entry_points` + `location_sort_key`.
- `crates/kali_sandbox/src/effects/inference.rs` — `infer_effects_from_roots`, `visit_source_root`, import resolution, `dedupe_effects`, `effect_sort_cmp`, `has_errors`, `SOURCE_EXTENSIONS`.
- `crates/kali_sandbox/src/effects/scan.rs` — `scan_tokens_for_effects`, `observed_effect`, `EffectMatch`, all `is_*`/`read_*` recognizers, `call_string_argument`, `unquote_token_value`.
- `crates/kali_sandbox/src/effects/compare.rs` — `compare_effects_to_policy`, `policy_suggestion`, `effect_allowed`, `rule_allows`.
- `crates/kali_sandbox/src/tests.rs` — unchanged; stays declared in the `lib.rs` facade with `use super::*`.

**Source line map — `lib.rs`** (current, for verbatim cut/paste):

| Item | Lines |
|---|---|
| crate doc `//!` | 1 |
| `use std::{…}` | 3–8 |
| `pub mod effects;` | 10 |
| `pub use effects::{…}` | 12–16 |
| `use kali_error::{…}` | 18–21 |
| `use serde::{…}` | 22 |
| `SandboxPolicy` struct | 24–44 |
| `EffectsPolicy` struct | 46–58 |
| `FileSystemPolicy` struct | 60–66 |
| `NetworkPolicy` struct | 68–77 |
| `ProcessPolicy` struct | 79–88 |
| `TimerPolicy` struct | 90–99 |
| `ResourceLimits` struct | 101–115 |
| `AccessRule` enum | 117–123 |
| `HostOperation` enum | 125–178 |
| `PolicyPredicateContext` struct | 180–191 |
| `impl PolicyPredicateContext { from_operation }` | 193–276 |
| `PolicyPredicateRegistry` struct | 278–283 |
| `RegisteredPredicate` struct | 285–289 |
| `HostPredicate` type alias | 291–292 |
| `impl Default for PolicyPredicateRegistry` | 294–298 |
| `impl PolicyPredicateRegistry { enabled, disabled, is_enabled, register, evaluate }` | 300–361 |
| `PolicyValidation` struct | 363–368 |
| `impl SandboxPolicy {` (open) | 370 |
| &nbsp;&nbsp;`from_file` | 372–374 |
| &nbsp;&nbsp;`from_file_with_runtime_profiles` | 377–409 |
| &nbsp;&nbsp;`to_canonical_json` | 412–419 |
| &nbsp;&nbsp;`validate` | 422–424 |
| &nbsp;&nbsp;`validate_with_runtime_profiles` | 427–523 |
| &nbsp;&nbsp;`effective_thread_budget` | 531–538 |
| &nbsp;&nbsp;`effective_spawn_budget` | 542–549 |
| &nbsp;&nbsp;`validate_policy` | 552–563 |
| &nbsp;&nbsp;`check_operation` | 566–673 |
| &nbsp;&nbsp;`check_operation_with_predicates` | 680–688 |
| &nbsp;&nbsp;`to_embedded_json_bytes` | 691–696 |
| &nbsp;&nbsp;`to_canonical_json_bytes` | 699–701 |
| &nbsp;&nbsp;`check_path_access` | 703–718 |
| &nbsp;&nbsp;`check_url_access` | 720–734 |
| &nbsp;&nbsp;`check_exact_access` | 736–750 |
| &nbsp;&nbsp;`network_max_connections` | 752–754 |
| `}` (close `impl SandboxPolicy`) | 755 |
| `impl AccessRule { is_enabled, allows_path, allows_candidate }` | 757–792 |
| `PatternKind` enum | 794–799 |
| `default_schema_version` | 801–803 |
| `default_base_dir` | 805–807 |
| `validate_positive_u64` | 809–818 |
| `validate_zero_capable_u64` | 820–823 |
| `unavailable_capability` | 825–833 |
| `host_predicate_violation` | 835–848 |
| `sandbox_violation` | 850–852 |
| `resource_limit_violation` | 854–856 |
| `resolve_pattern` | 858–872 |
| `normalize_text` | 874–876 |
| `glob_match` | 878–882 |
| `glob_match_inner` | 884–932 |
| test wiring (`#[cfg(test)] … mod tests;`) | 934–936 |

**Source line map — `effects.rs`** (current, for verbatim cut/paste):

| Item | Lines |
|---|---|
| `use std::{…}` | 1–5 |
| `use kali_ast::{…}` | 7 |
| `use kali_common::FileId;` | 8 |
| `use kali_error::{…}` | 9 |
| `use kali_lexer::{…}` | 10 |
| `use kali_parser::Parser;` | 11 |
| `use serde::{…}` | 12 |
| `use crate::{AccessRule, PatternKind, SandboxPolicy};` | 14 |
| `SOURCE_EXTENSIONS` const | 16–18 |
| `EffectAnalysisContext` struct | 20–27 |
| `impl EffectAnalysisContext { new, normalized }` | 29–44 |
| `EffectLocation` struct | 47–55 |
| `EffectOccurrence` struct | 57–62 |
| `EffectReport` struct | 64–75 |
| `PackageCoordinate` struct | 77–84 |
| `PackageEffectsReport` struct | 86–94 |
| `ObservedEffect` struct | 96–102 |
| `EffectInference` struct | 104–109 |
| `infer_effects_from_roots` | 111–137 |
| `effect_report_from_inference` | 139–182 |
| `package_effects_report` | 184–194 |
| `compare_effects_to_policy` | 196–224 |
| `policy_suggestion` | 226–270 |
| `normalize_semantic_axis` | 272–281 |
| `normalize_entry_points` | 283–286 |
| `effect_allowed` | 288–344 |
| `rule_allows` | 346–365 |
| `visit_source_root` | 367–421 |
| `collect_relative_imports` | 423–473 |
| `is_relative_specifier` | 475–477 |
| `resolve_relative_import` | 479–504 |
| `resolve_with_extensions` | 506–537 |
| `scan_tokens_for_effects` | 539–667 |
| `observed_effect` | 669–697 |
| `is_eval_call` | 699–705 |
| `is_function_constructor` | 707–710 |
| `is_proxy_constructor` | 712–715 |
| `is_proxy_revocable_call` | 717–728 |
| `read_proxy_root` | 730–745 |
| `is_console_write_call` | 747–759 |
| `EffectMatch` struct | 761–766 |
| `read_property_segment` | 768–796 |
| `read_deno_root` | 798–813 |
| `read_process_root` | 815–830 |
| `is_deno_command_constructor` | 832–848 |
| `is_deno_permissions_query` | 850–867 |
| `is_deno_host_call` | 869–925 |
| `is_process_env_assignment` | 927–942 |
| `is_global_effect_call` | 944–988 |
| `is_require_call` | 990–1012 |
| `call_string_argument` | 1014–1025 |
| `unquote_token_value` | 1027–1044 |
| `dedupe_effects` | 1046–1059 |
| `effect_sort_cmp` | 1061–1066 |
| `location_sort_key` | 1068–1074 |
| `has_errors` | 1076–1078 |

---

### Task 1: Visibility widening pass (in place)

Widen exactly the three methods that will be called across module boundaries after the split, with no code moved yet. (The diagnostic builder free fns get `pub(crate)` when they move to `diagnostics` in Task 4; the effects helpers `scan_tokens_for_effects`/`resolve_relative_import`/`location_sort_key` get widened in their extraction tasks.)

**Files:**
- Modify: `crates/kali_sandbox/src/lib.rs`

**Interfaces:**
- Produces: `pub(crate)` access to `PolicyPredicateRegistry::evaluate` (called by `enforcement::check_operation_with_predicates`), `SandboxPolicy::network_max_connections` (called by `validation`), and `AccessRule::allows_path` (called by `enforcement::check_path_access`).
- Unchanged: all existing `pub` methods stay `pub`; `check_path_access`/`check_url_access`/`check_exact_access` stay private (only called within `enforcement`); `validate_positive_u64`/`validate_zero_capable_u64`/`default_*`/`resolve_pattern`/`normalize_text`/`glob_match*`/`normalize_semantic_axis`/`normalize_entry_points` stay private (single-module callers).

- [ ] **Step 0: Create the work branch**

Confirm baseline green on `main`, then branch:

```bash
cargo test -p kali_sandbox 2>&1 | tail -3   # expect: 41 passed
git checkout -b refactor/kali-sandbox-modularization
```

- [ ] **Step 1: Widen the 3 cross-module methods**

Change each from `fn` to `pub(crate) fn` (signatures only; bodies untouched):
- `PolicyPredicateRegistry::evaluate` (line 338): `fn evaluate(` → `pub(crate) fn evaluate(`.
- `SandboxPolicy::network_max_connections` (line 752): `fn network_max_connections(` → `pub(crate) fn network_max_connections(`.
- `AccessRule::allows_path` (line 766): `fn allows_path(` → `pub(crate) fn allows_path(`.

- [ ] **Step 2: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: build with 0 warnings; `41 passed`. (Private→`pub(crate)` on already-used items never triggers dead-code warnings.)

- [ ] **Step 3: Commit**

```bash
git add crates/kali_sandbox/src/lib.rs
git commit -m "refactor(kali_sandbox): pub(crate) receiver-widening pass [refactor]"
```

---

### Task 2: Extract `policy.rs` (data model)

**Files:**
- Create: `crates/kali_sandbox/src/policy.rs`
- Modify: `crates/kali_sandbox/src/lib.rs`

**Interfaces:**
- Consumes: nothing (leaf data module).
- Produces: `crate::policy::{SandboxPolicy, EffectsPolicy, FileSystemPolicy, NetworkPolicy, ProcessPolicy, TimerPolicy, ResourceLimits, AccessRule}`, re-exported at crate root.

- [ ] **Step 1: Create `policy.rs` with the moved items**

Header, then the items moved **byte-identical** from `lib.rs`: the 7 structs (24–115), the `AccessRule` enum (117–123), `default_schema_version` (801–803), `default_base_dir` (805–807). The `#[serde(default = "default_schema_version")]` / `default_base_dir` attributes resolve because the two fns live in this same module.

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// <SandboxPolicy struct — verbatim, lib.rs 24–44>
// <EffectsPolicy struct — verbatim, lib.rs 46–58>
// <FileSystemPolicy struct — verbatim, lib.rs 60–66>
// <NetworkPolicy struct — verbatim, lib.rs 68–77>
// <ProcessPolicy struct — verbatim, lib.rs 79–88>
// <TimerPolicy struct — verbatim, lib.rs 90–99>
// <ResourceLimits struct — verbatim, lib.rs 101–115>
// <AccessRule enum — verbatim, lib.rs 117–123>
// <default_schema_version — verbatim, lib.rs 801–803>
// <default_base_dir — verbatim, lib.rs 805–807>
```

- [ ] **Step 2: Remove those items from `lib.rs` and wire the module**

Delete from `lib.rs`: the 7 structs + `AccessRule` enum (24–123), `default_schema_version` (801–803), `default_base_dir` (805–807). Add a `mod policy;` decl and the re-export after the existing `pub use effects::{…}` block:

```rust
mod policy;

pub use policy::{
    AccessRule, EffectsPolicy, FileSystemPolicy, NetworkPolicy, ProcessPolicy, ResourceLimits,
    SandboxPolicy, TimerPolicy,
};
```

The remaining `impl SandboxPolicy`/`impl AccessRule` blocks and `HostOperation` in `lib.rs` resolve these types via the re-export. Keep `lib.rs`'s `use std::{…}`, `use kali_error::{…}`, `use serde::{…}` for now — remaining code still needs them. If the build flags any as unused, delete the unused one.

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_sandbox/src/policy.rs crates/kali_sandbox/src/lib.rs
git commit -m "refactor(kali_sandbox): extract policy data-model module [refactor]"
```

---

### Task 3: Extract `operation.rs` (host-operation vocabulary)

**Files:**
- Create: `crates/kali_sandbox/src/operation.rs`
- Modify: `crates/kali_sandbox/src/lib.rs`

**Interfaces:**
- Consumes: nothing internal.
- Produces: `crate::operation::{HostOperation, PolicyPredicateContext}` (and `PolicyPredicateContext::from_operation`), re-exported at crate root.

- [ ] **Step 1: Create `operation.rs`**

Header, then verbatim moves: `HostOperation` (125–178), `PolicyPredicateContext` struct (180–191), `impl PolicyPredicateContext { from_operation }` (193–276).

```rust
use std::{collections::BTreeMap, path::PathBuf};

// <HostOperation enum — verbatim, lib.rs 125–178>
// <PolicyPredicateContext struct — verbatim, lib.rs 180–191>
// <impl PolicyPredicateContext { from_operation } — verbatim, lib.rs 193–276>
```

- [ ] **Step 2: Remove from `lib.rs` and wire the module**

Delete `lib.rs` lines 125–276. Add after the `policy` re-export:

```rust
mod operation;

pub use operation::{HostOperation, PolicyPredicateContext};
```

If `use std::{…}` in `lib.rs` now has an unused member (e.g. `BTreeMap` only fed `from_operation`), delete the unused member — the build will flag it.

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_sandbox/src/operation.rs crates/kali_sandbox/src/lib.rs
git commit -m "refactor(kali_sandbox): extract operation module [refactor]"
```

---

### Task 4: Extract `diagnostics.rs` + `matching.rs`

Extract both leaf utility modules together: `diagnostics` (depends on `operation::PolicyPredicateContext`, extracted in Task 3) and `matching` (depends on `policy::AccessRule`, extracted in Task 2). After this, `lib.rs`'s remaining methods call the diagnostic builders via `crate::diagnostics::…`.

**Files:**
- Create: `crates/kali_sandbox/src/diagnostics.rs`, `crates/kali_sandbox/src/matching.rs`
- Modify: `crates/kali_sandbox/src/lib.rs`

**Interfaces:**
- Consumes: `crate::PolicyPredicateContext` (diagnostics), `crate::AccessRule` (matching).
- Produces: `crate::diagnostics::{sandbox_violation, unavailable_capability, host_predicate_violation, resource_limit_violation}` (all `pub(crate)`); `crate::matching::PatternKind` (re-exported `pub(crate)` at crate root) and the `impl AccessRule` methods `is_enabled`/`allows_path`/`allows_candidate`.

- [ ] **Step 1: Create `diagnostics.rs`**

Header, then verbatim moves of the 4 builders (825–856), each gaining a `pub(crate)` prefix:

```rust
use kali_error::{
    _error_codes::{e4, e5},
    Diagnostic,
};

use crate::PolicyPredicateContext;

// <unavailable_capability — verbatim body, lib.rs 825–833, signature prefixed `pub(crate) fn`>
// <host_predicate_violation — verbatim body, lib.rs 835–848, signature prefixed `pub(crate) fn`>
// <sandbox_violation — verbatim body, lib.rs 850–852, signature prefixed `pub(crate) fn`>
// <resource_limit_violation — verbatim body, lib.rs 854–856, signature prefixed `pub(crate) fn`>
```

- [ ] **Step 2: Create `matching.rs`**

Header, then verbatim moves: `impl AccessRule { is_enabled, allows_path, allows_candidate }` (757–792, with the `pub(crate)` on `allows_path` from Task 1 and the existing `pub(crate)` on `allows_candidate`), `PatternKind` (794–799), `resolve_pattern` (858–872), `normalize_text` (874–876), `glob_match` (878–882), `glob_match_inner` (884–932).

```rust
use std::path::Path;

use crate::AccessRule;

// <impl AccessRule { is_enabled, allows_path, allows_candidate } — verbatim, lib.rs 757–792>
// <PatternKind enum — verbatim, lib.rs 794–799>
// <resolve_pattern — verbatim, lib.rs 858–872>
// <normalize_text — verbatim, lib.rs 874–876>
// <glob_match — verbatim, lib.rs 878–882>
// <glob_match_inner — verbatim, lib.rs 884–932>
```

- [ ] **Step 3: Remove from `lib.rs`, wire modules, fix call sites**

Delete from `lib.rs`: `impl AccessRule` (757–792), `PatternKind` (794–799), the 4 diagnostic builders (825–856), `resolve_pattern`/`normalize_text`/`glob_match`/`glob_match_inner` (858–932). Add the module decls + re-exports:

```rust
mod diagnostics;
mod matching;

pub(crate) use matching::PatternKind;
```

The methods still in `lib.rs` (`validate_with_runtime_profiles`, `check_operation`, `check_*_access`, and `PolicyPredicateRegistry::evaluate`) call the diagnostic builders by bare name. Add to `lib.rs`'s imports so they resolve from the new module:

```rust
use crate::diagnostics::{
    host_predicate_violation, resource_limit_violation, sandbox_violation, unavailable_capability,
};
```

`PatternKind` referenced by the remaining `check_*_access` methods resolves via the `pub(crate) use`. Delete any now-unused member of `lib.rs`'s `use std::{…}` (e.g. `Path`/`PathBuf` if only `matching` used them) and `use kali_error::{…}` if `e4`/`e5` are no longer referenced in `lib.rs` — the build will flag these.

- [ ] **Step 4: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_sandbox/src/diagnostics.rs crates/kali_sandbox/src/matching.rs crates/kali_sandbox/src/lib.rs
git commit -m "refactor(kali_sandbox): extract diagnostics and matching modules [refactor]"
```

---

### Task 5: Extract `predicate.rs` (predicate registry)

**Files:**
- Create: `crates/kali_sandbox/src/predicate.rs`
- Modify: `crates/kali_sandbox/src/lib.rs`

**Interfaces:**
- Consumes: `crate::PolicyPredicateContext` (operation), `crate::diagnostics::{unavailable_capability, host_predicate_violation}`.
- Produces: `crate::predicate::{PolicyPredicateRegistry, HostPredicate}` (re-exported at crate root) with `evaluate` `pub(crate)` (from Task 1).

- [ ] **Step 1: Create `predicate.rs`**

Header, then verbatim moves: `PolicyPredicateRegistry` struct (278–283), `RegisteredPredicate` struct (285–289), `HostPredicate` type alias (291–292), `impl Default for PolicyPredicateRegistry` (294–298), `impl PolicyPredicateRegistry { … }` (300–361, with `evaluate` already `pub(crate)`).

```rust
use std::{collections::BTreeMap, sync::Arc};

use kali_error::Diagnostic;

use crate::diagnostics::{host_predicate_violation, unavailable_capability};
use crate::PolicyPredicateContext;

// <PolicyPredicateRegistry struct — verbatim, lib.rs 278–283>
// <RegisteredPredicate struct — verbatim, lib.rs 285–289>
// <HostPredicate type alias — verbatim, lib.rs 291–292>
// <impl Default for PolicyPredicateRegistry — verbatim, lib.rs 294–298>
// <impl PolicyPredicateRegistry { enabled, disabled, is_enabled, register, evaluate } — verbatim, lib.rs 300–361>
```

- [ ] **Step 2: Remove from `lib.rs` and wire the module**

Delete `lib.rs` lines 278–361. Add:

```rust
mod predicate;

pub use predicate::{HostPredicate, PolicyPredicateRegistry};
```

Remove now-unused `lib.rs` imports the registry exclusively needed (`Arc`; `unavailable_capability`/`host_predicate_violation` are still used by remaining `enforcement`/`validation` methods — keep whichever the build still requires). The build will flag unused imports.

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_sandbox/src/predicate.rs crates/kali_sandbox/src/lib.rs
git commit -m "refactor(kali_sandbox): extract predicate module [refactor]"
```

---

### Task 6: Extract `validation.rs` (policy validation)

**Files:**
- Create: `crates/kali_sandbox/src/validation.rs`
- Modify: `crates/kali_sandbox/src/lib.rs`

**Interfaces:**
- Consumes: `crate::SandboxPolicy`, `crate::diagnostics::unavailable_capability`. Calls `self.network_max_connections()` (still in `lib.rs`'s `impl SandboxPolicy` at this point; `pub(crate)`, resolves crate-wide) and `AccessRule::is_enabled` (method).
- Produces: `crate::validation::PolicyValidation` (re-exported at crate root) and `impl SandboxPolicy { validate, validate_with_runtime_profiles, validate_policy }`.

- [ ] **Step 1: Create `validation.rs`**

Header, then verbatim moves into a single `impl SandboxPolicy { … }` block plus the struct and two free fns: `PolicyValidation` struct (363–368), `validate` (422–424), `validate_with_runtime_profiles` (427–523), `validate_policy` (552–563), `validate_positive_u64` (809–818), `validate_zero_capable_u64` (820–823).

```rust
use kali_error::{_error_codes::e5, Diagnostic};

use crate::diagnostics::unavailable_capability;
use crate::SandboxPolicy;

// <PolicyValidation struct — verbatim, lib.rs 363–368>

impl SandboxPolicy {
    // <validate — verbatim, lib.rs 422–424>
    // <validate_with_runtime_profiles — verbatim, lib.rs 427–523>
    // <validate_policy — verbatim, lib.rs 552–563>
}

// <validate_positive_u64 — verbatim, lib.rs 809–818>
// <validate_zero_capable_u64 — verbatim, lib.rs 820–823>
```

- [ ] **Step 2: Remove from `lib.rs` and wire the module**

Delete from `lib.rs`: `PolicyValidation` (363–368), `validate` (422–424), `validate_with_runtime_profiles` (427–523), `validate_policy` (552–563), `validate_positive_u64` (809–818), `validate_zero_capable_u64` (820–823). Add:

```rust
mod validation;

pub use validation::PolicyValidation;
```

Delete any `lib.rs` import that validation took exclusively (e.g. `unavailable_capability` if no longer used in `lib.rs`'s remaining `enforcement` methods — note `check_operation` still uses it, so likely keep). Build flags unused.

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_sandbox/src/validation.rs crates/kali_sandbox/src/lib.rs
git commit -m "refactor(kali_sandbox): extract validation module [refactor]"
```

---

### Task 7: Extract `loading.rs` + `enforcement.rs`; `lib.rs` becomes the thin facade

Move the last two concerns out of `lib.rs`'s `impl SandboxPolicy`, emptying the block. After this, `lib.rs` holds only the crate doc, module decls, re-exports, and test wiring.

**Files:**
- Create: `crates/kali_sandbox/src/loading.rs`, `crates/kali_sandbox/src/enforcement.rs`
- Modify: `crates/kali_sandbox/src/lib.rs`

**Interfaces:**
- Consumes (loading): `crate::SandboxPolicy`; calls `self.validate_with_runtime_profiles(...)` (validation, method).
- Consumes (enforcement): `crate::{SandboxPolicy, HostOperation, PolicyPredicateContext, PolicyPredicateRegistry, AccessRule, PatternKind}`, `crate::diagnostics::{sandbox_violation, unavailable_capability, resource_limit_violation}`; calls `self.evaluate`-via-`predicates.evaluate(...)` (`pub(crate)`), `AccessRule::allows_candidate`/`allows_path` (methods).
- Produces: no new public symbols (all methods already `pub`/`pub(crate)` on `SandboxPolicy`).

- [ ] **Step 1: Create `loading.rs`**

Header, then a single `impl SandboxPolicy { … }` block with verbatim moves: `from_file` (372–374), `from_file_with_runtime_profiles` (377–409), `to_canonical_json` (412–419), `to_embedded_json_bytes` (691–696), `to_canonical_json_bytes` (699–701).

```rust
use std::{
    fs,
    path::{Path, PathBuf},
};

use kali_error::{_error_codes::e5, Diagnostic};

use crate::SandboxPolicy;

impl SandboxPolicy {
    // <from_file — verbatim, lib.rs 372–374>
    // <from_file_with_runtime_profiles — verbatim, lib.rs 377–409>
    // <to_canonical_json — verbatim, lib.rs 412–419>
    // <to_embedded_json_bytes — verbatim, lib.rs 691–696>
    // <to_canonical_json_bytes — verbatim, lib.rs 699–701>
}
```

- [ ] **Step 2: Create `enforcement.rs`**

Header, then a single `impl SandboxPolicy { … }` block with verbatim moves: `effective_thread_budget` (531–538), `effective_spawn_budget` (542–549), `check_operation` (566–673), `check_operation_with_predicates` (680–688), `check_path_access` (703–718), `check_url_access` (720–734), `check_exact_access` (736–750), `network_max_connections` (752–754, `pub(crate)` from Task 1).

```rust
use std::path::Path;

use kali_error::Diagnostic;

use crate::diagnostics::{resource_limit_violation, sandbox_violation, unavailable_capability};
use crate::{AccessRule, HostOperation, PatternKind, PolicyPredicateContext, PolicyPredicateRegistry, SandboxPolicy};

impl SandboxPolicy {
    // <effective_thread_budget — verbatim, lib.rs 531–538>
    // <effective_spawn_budget — verbatim, lib.rs 542–549>
    // <check_operation — verbatim, lib.rs 566–673>
    // <check_operation_with_predicates — verbatim, lib.rs 680–688>
    // <check_path_access — verbatim, lib.rs 703–718>
    // <check_url_access — verbatim, lib.rs 720–734>
    // <check_exact_access — verbatim, lib.rs 736–750>
    // <network_max_connections — verbatim, lib.rs 752–754>
}
```

Note: `PolicyPredicateContext` is used by `check_operation_with_predicates` (it builds the context). `AccessRule`/`PatternKind` are used by the `check_*_access` methods. Drop any import the build reports unused.

- [ ] **Step 3: Reduce `lib.rs` to the thin facade**

After deleting the loading + enforcement methods, the `impl SandboxPolicy { }` block is empty — delete the empty block (370 + 755 and everything that remained between, now gone). Delete all leftover `use std::…`/`use kali_error::…`/`use serde::…`/`use crate::diagnostics::…` lines (nothing in `lib.rs` references them anymore). The final `lib.rs`:

```rust
//! Sandbox and policy system for the Kali compiler.

pub mod effects;

pub use effects::{
    compare_effects_to_policy, effect_report_from_inference, infer_effects_from_roots,
    package_effects_report, EffectAnalysisContext, EffectInference, EffectLocation,
    EffectOccurrence, EffectReport, ObservedEffect, PackageCoordinate, PackageEffectsReport,
};

mod diagnostics;
mod enforcement;
mod loading;
mod matching;
mod operation;
mod policy;
mod predicate;
mod validation;

pub use operation::{HostOperation, PolicyPredicateContext};
pub use policy::{
    AccessRule, EffectsPolicy, FileSystemPolicy, NetworkPolicy, ProcessPolicy, ResourceLimits,
    SandboxPolicy, TimerPolicy,
};
pub use predicate::{HostPredicate, PolicyPredicateRegistry};
pub use validation::PolicyValidation;

pub(crate) use matching::PatternKind;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```

`tests.rs` is unchanged: its `use super::*` resolves every type via the crate-root re-exports, including `pub(crate) PatternKind` and the `pub(crate)` method `AccessRule::allows_candidate`.

- [ ] **Step 4: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_sandbox/src/loading.rs crates/kali_sandbox/src/enforcement.rs crates/kali_sandbox/src/lib.rs
git commit -m "refactor(kali_sandbox): extract loading and enforcement; lib.rs thin facade [refactor]"
```

---

### Task 8: effects — create `effects/` dir, extract `report.rs`

Convert `effects.rs` into a directory module and extract the report data layer (a clean leaf). Use `git mv` so history follows the file.

**Files:**
- Rename: `crates/kali_sandbox/src/effects.rs` → `crates/kali_sandbox/src/effects/mod.rs`
- Create: `crates/kali_sandbox/src/effects/report.rs`
- Modify: `crates/kali_sandbox/src/effects/mod.rs`

**Interfaces:**
- Consumes: nothing internal (report types are self-contained).
- Produces: `crate::effects::report::{EffectAnalysisContext, EffectLocation, EffectOccurrence, EffectReport, PackageCoordinate, PackageEffectsReport, ObservedEffect, EffectInference, effect_report_from_inference, package_effects_report}` (pub) and `location_sort_key` (`pub(crate)`), re-exported at the effects-mod level.

- [ ] **Step 1: Move the file into a directory module**

```bash
mkdir crates/kali_sandbox/src/effects
git mv crates/kali_sandbox/src/effects.rs crates/kali_sandbox/src/effects/mod.rs
```

Verify build is still green (pure move): `cargo build -p kali_sandbox 2>&1 | tail -3` → 0 warnings.

- [ ] **Step 2: Create `effects/report.rs`**

Header, then verbatim moves from `effects/mod.rs` (line numbers from the original `effects.rs` map): `EffectAnalysisContext` struct (20–27), `impl EffectAnalysisContext { new, normalized }` (29–44), `EffectLocation` (47–55), `EffectOccurrence` (57–62), `EffectReport` (64–75), `PackageCoordinate` (77–84), `PackageEffectsReport` (86–94), `ObservedEffect` (96–102), `EffectInference` (104–109), `effect_report_from_inference` (139–182), `package_effects_report` (184–194), `normalize_semantic_axis` (272–281), `normalize_entry_points` (283–286), `location_sort_key` (1068–1074, signature prefixed `pub(crate) fn`).

```rust
use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

// <EffectAnalysisContext struct + impl — verbatim, effects.rs 20–44>
// <EffectLocation — verbatim, effects.rs 47–55>
// <EffectOccurrence — verbatim, effects.rs 57–62>
// <EffectReport — verbatim, effects.rs 64–75>
// <PackageCoordinate — verbatim, effects.rs 77–84>
// <PackageEffectsReport — verbatim, effects.rs 86–94>
// <ObservedEffect — verbatim, effects.rs 96–102>
// <EffectInference — verbatim, effects.rs 104–109>
// <effect_report_from_inference — verbatim, effects.rs 139–182>
// <package_effects_report — verbatim, effects.rs 184–194>
// <normalize_semantic_axis — verbatim, effects.rs 272–281>
// <normalize_entry_points — verbatim, effects.rs 283–286>
// <location_sort_key — verbatim body, effects.rs 1068–1074, signature `pub(crate) fn`>
```

- [ ] **Step 3: Remove moved items from `effects/mod.rs`, wire `report`, fix the remaining `location_sort_key` caller**

Delete the moved items from `effects/mod.rs` (the structs/impls 20–109, `effect_report_from_inference` 139–182, `package_effects_report` 184–194, `normalize_semantic_axis` 272–281, `normalize_entry_points` 283–286, `location_sort_key` 1068–1074). At the top of `effects/mod.rs`, add the submodule decl + re-exports (replacing the report-type names that were defined inline):

```rust
mod report;

pub use report::{
    effect_report_from_inference, package_effects_report, EffectAnalysisContext, EffectInference,
    EffectLocation, EffectOccurrence, EffectReport, ObservedEffect, PackageCoordinate,
    PackageEffectsReport,
};
```

The code still in `effects/mod.rs` (`infer_effects_from_roots`, `compare_effects_to_policy`, the scanners, etc.) references the report types — they now resolve via these `pub use` re-exports. The remaining `effect_sort_cmp` (still in `mod.rs`) calls `location_sort_key`: change that bare call to `report::location_sort_key(...)` (it now lives in `report`, `pub(crate)`). Drop any `use` line in `mod.rs` that the moved items took exclusively; the build flags unused imports.

- [ ] **Step 4: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_sandbox/src/effects/
git commit -m "refactor(kali_sandbox): effects dir module; extract report submodule [refactor]"
```

---

### Task 9: effects — extract `scan.rs` + `inference.rs`

These two are mutually dependent (`inference::visit_source_root` calls `scan_tokens_for_effects`; `scan_tokens_for_effects` calls `inference::resolve_relative_import`), so extract them in one task.

**Files:**
- Create: `crates/kali_sandbox/src/effects/scan.rs`, `crates/kali_sandbox/src/effects/inference.rs`
- Modify: `crates/kali_sandbox/src/effects/mod.rs`

**Interfaces:**
- Consumes: `super::report::{ObservedEffect, EffectLocation, EffectInference, location_sort_key}`; `kali_lexer::{Lexer, Token, TokenType}`, `kali_parser::Parser`, `kali_ast::*`, `kali_common::FileId`, `kali_error::{Diagnostic, _error_codes::{e5, e8}}`.
- Produces: `super::scan::scan_tokens_for_effects` (`pub(crate)`), `super::inference::{infer_effects_from_roots (pub, re-exported), resolve_relative_import (pub(crate))}`.

- [ ] **Step 1: Create `effects/scan.rs`**

Header, then verbatim moves (effects.rs line numbers): `scan_tokens_for_effects` (539–667, signature prefixed `pub(crate) fn`), `observed_effect` (669–697), `is_eval_call` (699–705), `is_function_constructor` (707–710), `is_proxy_constructor` (712–715), `is_proxy_revocable_call` (717–728), `read_proxy_root` (730–745), `is_console_write_call` (747–759), `EffectMatch` struct (761–766), `read_property_segment` (768–796), `read_deno_root` (798–813), `read_process_root` (815–830), `is_deno_command_constructor` (832–848), `is_deno_permissions_query` (850–867), `is_deno_host_call` (869–925), `is_process_env_assignment` (927–942), `is_global_effect_call` (944–988), `is_require_call` (990–1012), `call_string_argument` (1014–1025), `unquote_token_value` (1027–1044).

```rust
use std::{collections::BTreeSet, path::Path};

use kali_lexer::{Token, TokenType};

use super::inference::resolve_relative_import;
use super::report::{EffectAnalysisContext, EffectLocation, ObservedEffect};

// <scan_tokens_for_effects — verbatim body, effects.rs 539–667, signature `pub(crate) fn`>
// <observed_effect — verbatim, effects.rs 669–697>
// <is_eval_call … unquote_token_value — verbatim, effects.rs 699–1044 (incl. EffectMatch struct 761–766)>
```

(`EffectAnalysisContext` is a parameter type of `scan_tokens_for_effects`; `BTreeSet` is its `dynamic_reasons` arg type.)

- [ ] **Step 2: Create `effects/inference.rs`**

Header, then verbatim moves: `SOURCE_EXTENSIONS` const (16–18), `infer_effects_from_roots` (111–137), `visit_source_root` (367–421), `collect_relative_imports` (423–473), `is_relative_specifier` (475–477), `resolve_relative_import` (479–504, signature prefixed `pub(crate) fn`), `resolve_with_extensions` (506–537), `dedupe_effects` (1046–1059), `effect_sort_cmp` (1061–1066), `has_errors` (1076–1078).

```rust
use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use kali_ast::{ExportAllDeclaration, ExportNamedDeclaration, ImportDeclaration, Statement};
use kali_common::FileId;
use kali_error::{_error_codes::e5, _error_codes::e8, Diagnostic};
use kali_lexer::Lexer;
use kali_parser::Parser;

use super::report::{location_sort_key, EffectAnalysisContext, EffectInference, ObservedEffect};
use super::scan::scan_tokens_for_effects;

// <SOURCE_EXTENSIONS — verbatim, effects.rs 16–18>
// <infer_effects_from_roots — verbatim, effects.rs 111–137>
// <visit_source_root — verbatim, effects.rs 367–421>
// <collect_relative_imports — verbatim, effects.rs 423–473>
// <is_relative_specifier — verbatim, effects.rs 475–477>
// <resolve_relative_import — verbatim body, effects.rs 479–504, signature `pub(crate) fn`>
// <resolve_with_extensions — verbatim, effects.rs 506–537>
// <dedupe_effects — verbatim, effects.rs 1046–1059>
// <effect_sort_cmp — verbatim, effects.rs 1061–1066 (calls location_sort_key from report)>
// <has_errors — verbatim, effects.rs 1076–1078>
```

Note: in Task 8 the `mod.rs` copy of `effect_sort_cmp` was patched to call `report::location_sort_key`; here the moved copy lives beside the `use super::report::location_sort_key;` import, so the bare `location_sort_key(...)` call resolves directly — restore it to the verbatim original (bare call).

- [ ] **Step 3: Remove moved items from `effects/mod.rs` and wire modules**

Delete from `effects/mod.rs` all items now in `scan.rs` and `inference.rs` (everything except the `compare` group: `compare_effects_to_policy`, `policy_suggestion`, `effect_allowed`, `rule_allows`). Add the submodule decls and the `infer_effects_from_roots` re-export:

```rust
mod inference;
mod scan;

pub use inference::infer_effects_from_roots;
```

Remove the `mod.rs` top-of-file `use` lines that only the moved code needed (`kali_ast`, `kali_common::FileId`, `kali_lexer`, `kali_parser`, parts of `kali_error`, parts of `std`). Keep what the remaining `compare` group still needs (`use crate::{AccessRule, PatternKind, SandboxPolicy};`, `kali_error::{_error_codes::e9, Diagnostic}`, `std::path::Path`, `super::report::ObservedEffect`). The build flags any unused or missing import.

- [ ] **Step 4: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_sandbox/src/effects/
git commit -m "refactor(kali_sandbox): extract effects scan and inference submodules [refactor]"
```

---

### Task 10: effects — extract `compare.rs`; `effects/mod.rs` thin facade; finalize

Move the last concern out; `effects/mod.rs` becomes declarations + re-exports only. Then run the whole-branch verification and integrate.

**Files:**
- Create: `crates/kali_sandbox/src/effects/compare.rs`
- Modify: `crates/kali_sandbox/src/effects/mod.rs`

**Interfaces:**
- Consumes: `super::report::ObservedEffect`; `crate::{AccessRule, PatternKind, SandboxPolicy}`; `kali_error::{Diagnostic, _error_codes::e9}`.
- Produces: `super::compare::compare_effects_to_policy` (pub, re-exported).

- [ ] **Step 1: Create `effects/compare.rs`**

Header, then verbatim moves (effects.rs line numbers): `compare_effects_to_policy` (196–224), `policy_suggestion` (226–270), `effect_allowed` (288–344), `rule_allows` (346–365).

```rust
use std::path::Path;

use kali_error::{_error_codes::e9, Diagnostic};

use super::report::ObservedEffect;
use crate::{AccessRule, PatternKind, SandboxPolicy};

// <compare_effects_to_policy — verbatim, effects.rs 196–224>
// <policy_suggestion — verbatim, effects.rs 226–270>
// <effect_allowed — verbatim, effects.rs 288–344>
// <rule_allows — verbatim, effects.rs 346–365>
```

- [ ] **Step 2: Reduce `effects/mod.rs` to the thin facade**

Delete the `compare` group from `effects/mod.rs`. The final `effects/mod.rs`:

```rust
mod compare;
mod inference;
mod report;
mod scan;

pub use compare::compare_effects_to_policy;
pub use inference::infer_effects_from_roots;
pub use report::{
    effect_report_from_inference, package_effects_report, EffectAnalysisContext, EffectInference,
    EffectLocation, EffectOccurrence, EffectReport, ObservedEffect, PackageCoordinate,
    PackageEffectsReport,
};
```

All leftover top-level `use` lines in `mod.rs` are now gone (every concern lives in a submodule). The crate-root `pub use effects::{…}` in `lib.rs` is unchanged and still resolves all 12 symbols.

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_sandbox 2>&1 | tail -5 && cargo test -p kali_sandbox 2>&1 | tail -3`
Expected: 0 warnings; `41 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_sandbox/src/effects/
git commit -m "refactor(kali_sandbox): extract effects compare submodule; mod.rs thin facade [refactor]"
```

- [ ] **Step 5: API-surface proof (byte-identical public surface)**

Confirm the crate-root `pub` set is exactly the 13 types + `pub mod effects` + the 12 effects re-exports, and that `PatternKind` is `pub(crate)` only:

```bash
grep -rn "pub use\|pub mod\|pub(crate) use" crates/kali_sandbox/src/lib.rs
grep -rn "pub " crates/kali_sandbox/src/effects/mod.rs
```

Expected: the `lib.rs` re-exports match the Global Constraints public-surface list verbatim; `effects/mod.rs` exposes only the 12 documented symbols; no stray `pub` leaked from any submodule.

- [ ] **Step 6: Consumer-diff proof (consumers compile unedited)**

```bash
git diff --stat main -- crates/kali_cli crates/kali_embed crates/kali_runtime
cargo build -p kali_cli -p kali_embed -p kali_runtime 2>&1 | tail -10
```

Expected: **empty diff** for the three consumer crates; they build with no new warnings. (The pre-existing `kali_cli` `build_bundles_*` test failures are unrelated — do not treat as regressions.)

- [ ] **Step 7: Whole-workspace check**

```bash
cargo build --workspace 2>&1 | tail -10
cargo test -p kali_sandbox 2>&1 | tail -3   # 41 passed
```

Expected: workspace builds with 0 new warnings; `kali_sandbox` 41/41.

- [ ] **Step 8: Whole-branch review + integrate (local-main ff-merge only)**

After the finalize review passes (per series convention: opus reviewer on the whole branch), integrate to local main — **NEVER push to origin**:

```bash
git checkout main
git merge --ff-only refactor/kali-sandbox-modularization
cargo test -p kali_sandbox 2>&1 | tail -3   # re-verify on merged main: 41 passed
git branch -d refactor/kali-sandbox-modularization
```

Update the SDD ledger (`.superpowers/sdd/progress.md`) and the `crate-modularization-series` memory: 17th crate done, new local-main HEAD, origin still lagging.

---

## Self-Review

**Spec coverage:** Every spec section maps to a task — `lib.rs` 8-module split (Tasks 2–7), `effects.rs` 4-submodule split (Tasks 8–10), widening pass (Task 1 + per-task `pub(crate)` on `scan_tokens_for_effects`/`resolve_relative_import`/`location_sort_key`), `tests.rs` kept co-located (Task 7 Step 3), public-surface contract (Task 10 Step 5), consumer-diff-empty + workspace verification (Task 10 Steps 6–7), series process/integration (Task 1 Step 0 + Task 10 Step 8). The spec's task outline ordered predicate before diagnostics/matching; this plan swaps them (diagnostics+matching = Task 4, predicate = Task 5) so each module imports only already-extracted items, avoiding import churn — a sequencing refinement explicitly permitted by the spec ("sequenced in the implementation plan").

**Placeholder scan:** No `TBD`/`TODO`/"handle edge cases". The `// <… — verbatim, lines A–B>` markers are explicit cut/paste instructions against the line maps, not placeholders.

**Type consistency:** Module/type/fn names and the re-export lists are identical across the file-structure map, the per-task interfaces, and the final `lib.rs`/`effects/mod.rs` facades. The 12 effects symbols and 13 crate-root types match the Global Constraints list verbatim.
