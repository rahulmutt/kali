# kali_runtime Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Break `kali_runtime` (one 4,599-line `lib.rs` built around a big `impl RuntimeCtx`, two ~770/~800-line host-import registration fns, and a ~1,500-line `browser_*` cluster, plus a 7,149-line flat `tests.rs` of 158 tests) into small, single-purpose modules with co-located sibling test files and a shared `kali_test_support` dev-dependency — with zero behavior change.

**Architecture:** `lib.rs` becomes a thin facade (crate docs, the `use` import surface, and `pub use` re-exports of every current `pub` item). The `impl RuntimeCtx` block splits into accessor (`ctx.rs`) and execution (`execute.rs`) `impl RuntimeCtx` blocks (multiple `impl` blocks for one type are legal in one crate). Free functions move as `pub(crate) fn` into domain modules; the host-import + guest-plumbing code goes under a `host/` directory module and the browser-backend code under a `browser/` directory module. The giant functions (`execute_inner`, `register_default_host_imports`, `register_node_host_imports`) move **byte-for-byte intact** — cracking them is out of scope. Tests move into sibling `*_tests.rs` files wired with `#[cfg(test)] #[path = "…"] mod`.

**Tech Stack:** Rust 2021, Cargo workspace, `wasmtime`, `reqwest`, `serde_json`, `base64`, `url`, `tempfile`; dev: `wat`, `tempfile`, and (newly added) `kali_test_support`.

## Global Constraints

- **Zero behavior change.** Pure structural refactor. The set of tests that exist and pass is identical before and after (renames excepted, tracked explicitly). Function bodies are moved verbatim — never rewritten or split.
- **Green at every commit.** `cargo test -p kali_runtime` must pass after every task. Never commit a red tree.
- **Public API preserved.** Every current `pub` item must keep resolving at its existing path `kali_runtime::<Name>`. The facade re-exports them all with `pub use`. The full public surface (33 items) is: types `RuntimeCtx`, `KaliHostState`, `ScheduledTimer`, `RuntimeHostContract`, `RuntimeBackend`, `BrowserRuntimeContract`, `BrowserRuntimeContractDescriptor`, `RuntimeOutcome`, `BrowserRuntimeExecutionOutcome`, `BrowserHarnessInvocation`, `BrowserHarnessOutcome`, `BrowserHarnessError`; const `BROWSER_HARNESS_COMMAND_ENV`; fns `browser_runtime_contract_value`, `normalize_runtime_profiles`, `browser_runtime_unavailable_diagnostic`, `browser_runtime_request_context`, `split_command_spec`, `browser_harness_command_parts_checked`, `browser_harness_command_parts_for`, `browser_bundle_harness_prelude`, `browser_bundle_harness_script`, `browser_bundle_runtime_harness_module_script`, `browser_bundle_runtime_harness_page`, `browser_bundle_runtime_harness_script`, `browser_bundle_runtime_execute_checked`, `browser_runtime_harness_script`, `browser_runtime_harness_page`, `browser_runtime_execute_checked`, `browser_harness_invocation_checked`, `browser_harness_run_checked`, `browser_harness_run_checked_with_env`, `browser_harness_command_parts`.
- **Text-movement only.** No function body is rewritten or cracked into helpers. Cross-referenced items are widened to `pub(crate)` in Task 2, after which calls resolve across modules anywhere in the crate.
- **Test convention.** Unit tests live in sibling `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod …;` — never inline `#[cfg(test)] mod tests { … }` blocks.
- **No new runtime dependencies.** `kali_test_support` is added as a `dev-dependency` only; it already exists in the workspace.
- **Branch.** Create and work on `refactor/kali-runtime-modularization` off `main`. All paths are relative to repo root `/workspace`.
- **`--list` includes module-path prefixes.** Co-location changes the prefix of every moved test, so a raw `diff` of `--list` output is non-empty *by design*. Prove the invariant by comparing **basenames** (strip the `…::` prefix), as Task 15 does.
- **Verification triad per task:** `cargo build -p kali_runtime` → `cargo test -p kali_runtime` → `cargo clippy -p kali_runtime --all-targets -- -D warnings` (the clippy gate may be deferred to Task 11/15 if a transient `pub(crate)`-could-be-private style lint fires mid-split; build + test stay green every commit).

---

### Task 1: Branch + baseline test snapshot

**Files:**
- Create: `docs/superpowers/baselines/kali_runtime-tests-before.txt`

**Interfaces:**
- Produces: `kali_runtime-tests-before.txt` — the authoritative list of test basenames before refactor, diffed against in Task 15.

- [ ] **Step 1: Create and switch to the refactor branch**

```bash
cd /workspace
git checkout -b refactor/kali-runtime-modularization
git branch --show-current
```
Expected: `refactor/kali-runtime-modularization`.

- [ ] **Step 2: Confirm the suite is green before any change**

```bash
cargo test -p kali_runtime 2>&1 | tail -5
```
Expected: `test result: ok.` with 158 passed and no failures.

- [ ] **Step 3: Snapshot the exact set of test basenames**

```bash
mkdir -p docs/superpowers/baselines
cargo test -p kali_runtime -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort > docs/superpowers/baselines/kali_runtime-tests-before.txt
wc -l docs/superpowers/baselines/kali_runtime-tests-before.txt
```
Expected: `158`.

- [ ] **Step 4: Commit the baseline**

```bash
git add docs/superpowers/baselines/kali_runtime-tests-before.txt
git commit -m "chore(kali_runtime): snapshot test baseline [refactor]"
```

---

### Task 2: Widen internal visibility to `pub(crate)` (the enabling step)

**Why first:** Once items live in sibling modules, Rust privacy blocks cross-module access to *private* items. Promoting crate-internal items to `pub(crate)` up front turns every later extraction into pure text movement. This task changes only visibility keywords — no code moves, no behavior change. (The public structs and their fields are already `pub`; this task targets the *private* functions, methods, structs, and consts.)

**Files:**
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces: every private free `fn`, every private method in `impl RuntimeCtx` / `impl KaliHostState` / `impl RuntimeHostContract` / `impl RuntimeBackend` / `impl BrowserRuntimeContract`, the private `struct BrowserRuntimeSummary`, and the private consts (`BROWSER_HARNESS_SUMMARY_FILE_ENV`, `STRING_HANDLE_TAG`, `BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES`) all become `pub(crate)`.

- [ ] **Step 1: Promote every private top-level free fn**

```bash
cd /workspace
awk '/^fn /{sub(/^fn /, "pub(crate) fn ")} {print}' crates/kali_runtime/src/lib.rs > /tmp/rt_vis.rs && mv /tmp/rt_vis.rs crates/kali_runtime/src/lib.rs
grep -c "^pub(crate) fn " crates/kali_runtime/src/lib.rs
```
Expected: ≈40 (every previously-private free fn; the already-`pub fn`s are untouched).

- [ ] **Step 2: Promote every private method inside the impl blocks**

The 4-space-indented private methods live in `impl RuntimeCtx` (≈365–844), `impl KaliHostState` (≈4215–4502), and the small `impl RuntimeHostContract`/`impl RuntimeBackend`/`impl BrowserRuntimeContract`/`impl BrowserRuntimeExecutionOutcome`/`impl BrowserHarnessInvocation` blocks. Promote each `    fn ` (private method) to `    pub(crate) fn ` across the whole file (already-`pub` methods are untouched):

```bash
cd /workspace
awk '/^    fn /{sub(/^    fn /, "    pub(crate) fn ")} {print}' crates/kali_runtime/src/lib.rs > /tmp/rt_vis2.rs && mv /tmp/rt_vis2.rs crates/kali_runtime/src/lib.rs
grep -c "^    pub(crate) fn " crates/kali_runtime/src/lib.rs
```
Expected: ≈25 (the previously-private methods: `RuntimeCtx::{browser_harness_command, reject_unavailable_threaded_requests, execute_inner}`, `KaliHostState::{schedule_timer, cancel_timer, queue_microtask, register_event_listener, event_listener_callbacks, event_listener_count, begin_spawn, finish_spawn, has_threaded_runtime_profile, begin_thread, finish_thread, take_pending_exit_code}`, and any private label/contract helpers).

- [ ] **Step 3: Promote the private struct and consts**

In `crates/kali_runtime/src/lib.rs`, change:
- `struct BrowserRuntimeSummary {` (≈3652) → `pub(crate) struct BrowserRuntimeSummary {` and prefix its fields with `pub(crate)`.
- `const BROWSER_HARNESS_SUMMARY_FILE_ENV` (≈244) → `pub(crate) const …`
- `const STRING_HANDLE_TAG` (≈2646) → `pub(crate) const …`
- `const BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES` (≈2801) → `pub(crate) const …`

(`BROWSER_HARNESS_COMMAND_ENV` at ≈241 is already `pub` — leave it.)

- [ ] **Step 4: Build and test**

```bash
cargo build -p kali_runtime 2>&1 | tail -5
cargo test -p kali_runtime 2>&1 | tail -5
```
Expected: build succeeds (any "could be private" style warnings are acceptable), `test result: ok.` with 158 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_runtime/src/lib.rs
git commit -m "refactor(kali_runtime): widen internals to pub(crate) for module split [refactor]"
```

---

## Source-extraction tasks (Tasks 3–10)

**Shared procedure for every source-extraction task below.** Each task creates one or more module files, moves a named set of items out of `lib.rs` into them, adds the module declaration (+ `pub use` re-exports for public items) to the facade, and proves green. The recipe is identical; only the file names and item lists change:

1. Create the new file with a `//!` doc line and `use crate::*;` (this brings the crate import surface — the big `use` block in `lib.rs` plus everything re-exported at the crate root — into scope). Add an extra `use` only if a name is reported missing at build time.
2. For method modules, wrap the moved methods in `impl RuntimeCtx { … }` (or `impl KaliHostState { … }`). For type/free-fn/const modules, paste at file top level.
3. Cut the listed items out of `lib.rs`, paste into the new file verbatim — **including any private const a moved item needs** (consts move with the module that uses them; if a const is shared by two modules, leave it in `lib.rs` and qualify uses as `crate::<CONST>`).
4. In `lib.rs`, declare and re-export the module so name resolution matches the pre-split single-module behavior — see **Crate-root re-export rule** below.
5. `cargo build -p kali_runtime && cargo test -p kali_runtime 2>&1 | tail -5`; both green, 158.
6. Commit with the message given in the task.

**Crate-root re-export rule (critical for this crate).** Unlike a method (`self.foo()` resolves through the `impl` from any module), a **free function** called by bare name (e.g. `append_stdout(state, text)` inside `register_node_host_imports`) only resolves if it is in scope. `use crate::*;` brings in crate-*root* items, so every extracted module that holds free fns/types referenced elsewhere must be glob-re-exported at the root. For each extracted module add **two** lines in `lib.rs`:

```rust
mod <name>;
pub(crate) use <name>::*;          // surfaces ALL items (pub + pub(crate)) to `use crate::*;`
pub use <name>::{Foo, bar_fn};     // ONLY for items in the public API surface (omit if none)
```

For directory modules, glob each submodule: `pub(crate) use host::{memory::*, io::*, diagnostics::*, enforce::*, imports_default::*, imports_node::*};`. The explicit `pub use` of a public name shadows the same name from the `pub(crate)` glob (explicit beats glob — no ambiguity error). This exactly reproduces the old "everything in one module" name resolution.

**Do not change any function body.** If a moved item references a name that compiled only because everything was one module, the fix is to add `pub(crate)` to *that* item (and ensure its module is glob-re-exported per the rule above) — never to alter a body. Directory-module submodules (`host/*`, `browser/*`) use `use crate::*;`. If a bare `const` still isn't surfaced, qualify it at the use site as `crate::<CONST>` (mechanical, no logic change).

---

### Task 3: Extract `ctx.rs` (context, builders/accessors) and `outcome.rs`

**Files:**
- Create: `crates/kali_runtime/src/ctx.rs`
- Create: `crates/kali_runtime/src/outcome.rs`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces (re-exported): `RuntimeCtx`, `RuntimeOutcome`. Moves the `RuntimeCtx` struct, its `Default`, and its builder/accessor `impl RuntimeCtx` block (everything **except** the execution methods, which go to `execute.rs` in Task 10).

- [ ] **Step 1:** Create `outcome.rs` (`use crate::*;`). Move into it: `pub struct RuntimeOutcome` (≈326) and any `impl RuntimeOutcome` block.
- [ ] **Step 2:** Create `ctx.rs` (`use crate::*;`). Move into it:
  - `pub struct RuntimeCtx` (≈35) with its fields.
  - `impl Default for RuntimeCtx` (≈349).
  - An `impl RuntimeCtx { … }` block containing **only the builder/accessor methods**: `new`, `with_api_surface`, `with_host_context`, `with_host_context_with_api_surface`, `with_runtime_profiles`, `with_max_threads`, `with_max_spawned_processes`, `effective_thread_budget`, `env_has`, `has`, `env_snapshot`, `env_to_object`, `snapshot`, `env_snapshot_object_value`, `env_snapshot_value`, `env_snapshot_json_value`, `snapshot_value`, `snapshot_object_value`, `snapshot_json_value`, `env_to_json_value`, `canonical_runtime_profiles`, `host_contract`, `runtime_backend`, `process_id`.
  - The free helper fns `capture_env` (≈2468) and `env_snapshot_value` (the free fn at ≈2472 — note this is distinct from the same-named method; keep both).
- [ ] **Step 3:** In `lib.rs`, per the Crate-root re-export rule:
```rust
mod ctx;
pub(crate) use ctx::*;
pub use ctx::RuntimeCtx;
mod outcome;
pub(crate) use outcome::*;
pub use outcome::RuntimeOutcome;
```
- [ ] **Step 4:** Build + test green, 158.
- [ ] **Step 5:** `git add -A && git commit -m "refactor(kali_runtime): extract ctx + outcome modules [refactor]"`

---

### Task 4: Extract `state.rs` (host state + scheduled timer)

**Files:**
- Create: `crates/kali_runtime/src/state.rs`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces (re-exported): `KaliHostState`, `ScheduledTimer`. Moves their definitions, `impl Default for KaliHostState`, and the full `impl KaliHostState` block.

- [ ] **Step 1:** Create `state.rs` (`use crate::*;`). Move into it:
  - `pub struct KaliHostState` (≈58) with fields.
  - `pub struct ScheduledTimer` (≈117) with fields.
  - `impl Default for KaliHostState` (≈4181).
  - The entire `impl KaliHostState { … }` block (≈4215–4502): `runtime_backend`, `process_id`, `env_has`, `has`, `env_snapshot`, `env_to_object`, `env_snapshot_object_value`, `env_snapshot_value`, `env_snapshot_json_value`, `snapshot_value`, `env_to_json_value`, `thread_topology_snapshot_json_value`, `thread_topology_snapshot_object_value`, `snapshot_object_value`, `snapshot_json_value`, `snapshot`, `thread_topology_snapshot`, `thread_topology_snapshot_value`, `schedule_timer`, `cancel_timer`, `queue_microtask`, `register_event_listener`, `event_listener_callbacks`, `event_listener_count`, `begin_spawn`, `finish_spawn`, `spawn_thread_instance`, `release_thread_instance`, `has_threaded_runtime_profile`, `begin_thread`, `finish_thread`, `take_pending_exit_code`.
- [ ] **Step 2:** In `lib.rs`:
```rust
mod state;
pub(crate) use state::*;
pub use state::{KaliHostState, ScheduledTimer};
```
- [ ] **Step 3:** Build + test green, 158.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_runtime): extract state module [refactor]"`

---

### Task 5: Extract `profiles.rs` (runtime contract/backend + profile normalization)

**Files:**
- Create: `crates/kali_runtime/src/profiles.rs`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces (re-exported): `RuntimeHostContract`, `RuntimeBackend`, `normalize_runtime_profiles`. Moves the two enums (+ their impls) and the profile/label helpers.

- [ ] **Step 1:** Create `profiles.rs` (`use crate::*;`). Move into it:
  - `pub enum RuntimeHostContract` (≈128) + `impl RuntimeHostContract` (≈135).
  - `pub enum RuntimeBackend` (≈147) + `impl RuntimeBackend` (≈154).
  - `pub fn normalize_runtime_profiles` (≈2457).
  - The label helpers: `parse_runtime_host_contract_label` (≈3661), `parse_runtime_backend_label` (≈3669), `parse_optional_runtime_host_contract_label` (≈3821), `parse_optional_runtime_backend_label` (≈3832).
- [ ] **Step 2:** In `lib.rs`:
```rust
mod profiles;
pub(crate) use profiles::*;
pub use profiles::{normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract};
```
- [ ] **Step 3:** Build + test green, 158.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_runtime): extract profiles module [refactor]"`

---

### Task 6: Extract `host/` plumbing (`memory`, `io`, `diagnostics`, `enforce`)

**Files:**
- Create: `crates/kali_runtime/src/host/mod.rs`
- Create: `crates/kali_runtime/src/host/memory.rs`
- Create: `crates/kali_runtime/src/host/io.rs`
- Create: `crates/kali_runtime/src/host/diagnostics.rs`
- Create: `crates/kali_runtime/src/host/enforce.rs`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces (crate-internal): guest-memory read/write helpers, IO buffering, host diagnostics, sandbox enforcement + event-loop drain. No public types.

- [ ] **Step 1:** Create `host/mod.rs`:
```rust
//! Host-import registration and guest-memory plumbing for the wasmtime linker.
use crate::*;
pub(crate) mod memory;
pub(crate) mod io;
pub(crate) mod diagnostics;
pub(crate) mod enforce;
```
- [ ] **Step 2:** `host/memory.rs` (`use crate::*;`). Move: `read_guest_string` (≈2492), `read_guest_string_handle` (≈2503), `read_guest_bytes` (≈2519), `write_guest_bytes` (≈2537), `write_guest_string` (≈2559), `guest_memory` (≈2568), `checked_offset` (≈2575), `decode_spawn_args` (≈2480), and the `pub(crate) const STRING_HANDLE_TAG` (≈2646).
- [ ] **Step 3:** `host/io.rs` (`use crate::*;`). Move: `append_stdout` (≈2613), `append_stdout_raw` (≈2618), `append_stderr` (≈2622), `append_stderr_raw` (≈2627), `format_console_value` (≈2631).
- [ ] **Step 4:** `host/diagnostics.rs` (`use crate::*;`). Move: `host_import_error` (≈2648), `runtime_error_diagnostic` (≈2655), `resolve_host_path` (≈2579), `normalize_path` (≈2587).
- [ ] **Step 5:** `host/enforce.rs` (`use crate::*;`). Move: `enforce_operation` (≈4165), `drain_event_loop` (≈4503), `invoke_callback` (≈4567).
- [ ] **Step 6:** In `lib.rs`:
```rust
mod host;
pub(crate) use host::{diagnostics::*, enforce::*, io::*, memory::*};
```
Build + test green, 158.
- [ ] **Step 7:** `git add -A && git commit -m "refactor(kali_runtime): extract host plumbing (memory/io/diagnostics/enforce) [refactor]"`

---

### Task 7: Extract `host/imports_default.rs` and `host/imports_node.rs` (giant fns, intact)

**Files:**
- Create: `crates/kali_runtime/src/host/imports_default.rs`
- Create: `crates/kali_runtime/src/host/imports_node.rs`
- Modify: `crates/kali_runtime/src/host/mod.rs`

**Interfaces:**
- Produces (crate-internal): `register_default_host_imports`, `register_node_host_imports` — called from `execute.rs`. **Moved byte-for-byte; not cracked.**

- [ ] **Step 1:** `host/imports_default.rs` (`use crate::*;`). Move `register_default_host_imports` (≈890–1658) **verbatim, unchanged**.
- [ ] **Step 2:** `host/imports_node.rs` (`use crate::*;`). Move `register_node_host_imports` (≈1659–2456) **verbatim, unchanged**.
- [ ] **Step 3:** In `host/mod.rs` add `pub(crate) mod imports_default;` and `pub(crate) mod imports_node;`. In `lib.rs` extend the host glob to surface them: `pub(crate) use host::{imports_default::*, imports_node::*};` (alongside the existing `host::{diagnostics::*, …}` line). Build + test green, 158.
- [ ] **Step 4:** Confirm the bodies were not altered:
```bash
cd /workspace
git diff --stat HEAD~0 # sanity; then verify line counts moved, not changed
wc -l crates/kali_runtime/src/host/imports_default.rs crates/kali_runtime/src/host/imports_node.rs
```
Expected: `imports_default.rs` ≈770 lines, `imports_node.rs` ≈800 lines (header + one fn each).
- [ ] **Step 5:** `git add -A && git commit -m "refactor(kali_runtime): extract host import registration (intact) [refactor]"`

---

### Task 8: Extract `browser/` part 1 (`contract`, `command`, `summary`)

**Files:**
- Create: `crates/kali_runtime/src/browser/mod.rs`
- Create: `crates/kali_runtime/src/browser/contract.rs`
- Create: `crates/kali_runtime/src/browser/command.rs`
- Create: `crates/kali_runtime/src/browser/summary.rs`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces (re-exported): `BrowserRuntimeContract`, `BrowserRuntimeContractDescriptor`, `browser_runtime_contract_value`, `BROWSER_HARNESS_COMMAND_ENV`, `browser_runtime_unavailable_diagnostic`, `browser_runtime_request_context`, `split_command_spec`, `browser_harness_command_parts_checked`, `browser_harness_command_parts_for`, `browser_harness_command_parts`. Plus crate-internal `BrowserRuntimeSummary` and summary parsers.

- [ ] **Step 1:** Create `browser/mod.rs`:
```rust
//! Browser-runtime backend: contract, command resolution, harness generation,
//! checked execution, and summary parsing.
use crate::*;
pub(crate) mod contract;
pub(crate) mod command;
pub(crate) mod summary;
```
- [ ] **Step 2:** `browser/contract.rs` (`use crate::*;`). Move: `pub struct BrowserRuntimeContract` (≈169), `pub struct BrowserRuntimeContractDescriptor` (≈173), `browser_runtime_contract_descriptor_is_canonical` (≈192), `pub fn browser_runtime_contract_value` (≈221), `impl BrowserRuntimeContract` (≈246), `browser_runtime_unavailable_diagnostic` (≈2671), `browser_runtime_request_context` (≈2705), and the consts `pub const BROWSER_HARNESS_COMMAND_ENV` (≈241) + `pub(crate) const BROWSER_HARNESS_SUMMARY_FILE_ENV` (≈244).
- [ ] **Step 3:** `browser/command.rs` (`use crate::*;`). Move: `split_command_spec` (≈2718), `browser_harness_normalized_executable_name` (≈2774), `browser_harness_is_browser_executable_name` (≈2897), `browser_harness_command_parts_for_browser_executable` (≈2901), `browser_harness_default_browser_command_parts` (≈2911), `browser_harness_default_command_parts` (≈2923), `browser_harness_command_parts_checked` (≈2944), `browser_harness_command_parts_for` (≈2977), `browser_harness_command_parts` (≈4161), `browser_harness_uses_html_entrypoint` (≈3347), and the const `pub(crate) const BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES` (≈2801).
- [ ] **Step 4:** `browser/summary.rs` (`use crate::*;`). Move: `pub(crate) struct BrowserRuntimeSummary` (≈3652), `parse_non_blank_string_array_field` (≈3677), `parse_browser_runtime_summary` (≈3690), `parse_thread_runtime_instance_snapshot_value` (≈3694), `parse_thread_runtime_shutdown_report_value` (≈3747), `parse_browser_runtime_summary_value` (≈3793), `parse_browser_runtime_summary_opt` (≈3843), `browser_runtime_summary_for_outcome` (≈3855).
- [ ] **Step 5:** In `lib.rs`: add
```rust
mod browser;
pub(crate) use browser::{command::*, contract::*, summary::*};
pub use browser::contract::{
    browser_runtime_contract_value, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, BrowserRuntimeContract,
    BrowserRuntimeContractDescriptor, BROWSER_HARNESS_COMMAND_ENV,
};
pub use browser::command::{
    browser_harness_command_parts, browser_harness_command_parts_checked,
    browser_harness_command_parts_for, split_command_spec,
};
```
- [ ] **Step 6:** Build + test green, 158.
- [ ] **Step 7:** `git add -A && git commit -m "refactor(kali_runtime): extract browser contract/command/summary [refactor]"`

---

### Task 9: Extract `browser/` part 2 (`harness`, `execute`)

**Files:**
- Create: `crates/kali_runtime/src/browser/harness.rs`
- Create: `crates/kali_runtime/src/browser/execute.rs`
- Modify: `crates/kali_runtime/src/browser/mod.rs`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces (re-exported): the harness-script generators and the checked-execution surface + outcome/invocation/error types.

- [ ] **Step 1:** `browser/harness.rs` (`use crate::*;`). Move (each generator **verbatim**, including its embedded JS template string): `browser_bundle_harness_prelude` (≈2986), `browser_bundle_harness_script` (≈3031), `browser_bundle_runtime_harness_module_script` (≈3044), `browser_bundle_runtime_harness_page` (≈3219), `browser_bundle_runtime_harness_script` (≈3248), `browser_runtime_harness_module_script` (≈3351), `browser_runtime_harness_script` (≈3587), `browser_runtime_harness_page` (≈3599).
- [ ] **Step 2:** `browser/execute.rs` (`use crate::*;`). Move: `pub struct BrowserRuntimeExecutionOutcome` (≈3620) + `impl` (≈3644), `pub struct BrowserHarnessInvocation` (≈3952) + `impl` (≈3968), `pub struct BrowserHarnessOutcome` (≈4029), `pub enum BrowserHarnessError` (≈4043) + `impl Display`/`impl Error` (≈4069/≈4091), `browser_bundle_runtime_execute_checked` (≈3266), `browser_runtime_execute_checked` (≈3890), `browser_harness_invocation_checked` (≈4094), `browser_harness_run_checked` (≈4139), `browser_harness_run_checked_with_env` (≈4149).
- [ ] **Step 3:** In `browser/mod.rs` add `pub(crate) mod harness;` and `pub(crate) mod execute;`.
- [ ] **Step 4:** In `lib.rs` add:
```rust
pub(crate) use browser::{execute::*, harness::*};
pub use browser::harness::{
    browser_bundle_harness_prelude, browser_bundle_harness_script,
    browser_bundle_runtime_harness_module_script, browser_bundle_runtime_harness_page,
    browser_bundle_runtime_harness_script, browser_runtime_harness_page,
    browser_runtime_harness_script,
};
pub use browser::execute::{
    browser_bundle_runtime_execute_checked, browser_harness_invocation_checked,
    browser_harness_run_checked, browser_harness_run_checked_with_env,
    browser_runtime_execute_checked, BrowserHarnessError, BrowserHarnessInvocation,
    BrowserHarnessOutcome, BrowserRuntimeExecutionOutcome,
};
```
- [ ] **Step 5:** Build + test green, 158.
- [ ] **Step 6:** `git add -A && git commit -m "refactor(kali_runtime): extract browser harness/execute [refactor]"`

---

### Task 10: Extract `execute.rs` (execution methods) and verify the facade

**Files:**
- Create: `crates/kali_runtime/src/execute.rs`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Produces (crate-internal): the `impl RuntimeCtx` execution block + `execute_browser_runtime`. `execute`/`execute_tests` stay public methods on the re-exported `RuntimeCtx`.

- [ ] **Step 1:** Create `execute.rs` (`use crate::*;`). Move into it:
  - An `impl RuntimeCtx { … }` block with: `browser_harness_command` (≈529-ish), `reject_unavailable_threaded_requests`, `execute` (`pub fn`), `execute_tests` (`pub fn`), `execute_inner` (**moved byte-for-byte intact**).
  - The free fn `execute_browser_runtime` (≈845).
- [ ] **Step 2:** In `lib.rs`: add `mod execute;` (no re-export; methods attach to `RuntimeCtx`, the free fn is crate-internal).
- [ ] **Step 3:** Verify `lib.rs` is now a thin facade:
```bash
cd /workspace
grep -nE "^(pub )?(struct|enum|impl|fn|const) " crates/kali_runtime/src/lib.rs
wc -l crates/kali_runtime/src/lib.rs
```
Expected: only `mod`/`pub use` lines, the crate `use` block, crate docs, and (later) `#[cfg(test)]` wiring remain — no `struct`/`enum`/`impl`/`fn`/`const` definitions; `lib.rs` drops from 4,599 to roughly 80–130 lines. If any private const is still referenced from a moved module, qualify it as `crate::<CONST>` at the use site or leave the const in `lib.rs` as `pub(crate)`.
- [ ] **Step 4:** Build + test green, 158.
- [ ] **Step 5:** `git add -A && git commit -m "refactor(kali_runtime): extract execute; reduce lib.rs to facade [refactor]"`

---

### Task 11: `cargo fmt` normalization

**Files:**
- Modify: all `crates/kali_runtime/src/**/*.rs`

- [ ] **Step 1:** Normalize formatting after the split:
```bash
cd /workspace
cargo fmt -p kali_runtime
```
- [ ] **Step 2:** Confirm still green and clippy-clean:
```bash
cargo test -p kali_runtime 2>&1 | tail -5
cargo clippy -p kali_runtime --all-targets -- -D warnings 2>&1 | tail -15
```
Expected: `test result: ok.` 158; clippy reports no errors. If clippy flags a genuinely-now-private `pub(crate)` item, narrow that single item's visibility (no body change) and re-run.
- [ ] **Step 3:** `git add -A && git commit -m "style(kali_runtime): cargo fmt normalization after module split [refactor]"`

---

## Test co-location tasks (Tasks 12–14)

### Task 12: Add `kali_test_support` dev-dep and the crate-local `test_support` module

**Files:**
- Modify: `crates/kali_runtime/Cargo.toml` (add dev-dependency)
- Create: `crates/kali_runtime/src/test_support.rs`
- Modify: `crates/kali_runtime/src/lib.rs` (declare the module under `cfg(test)`)
- Modify: `crates/kali_runtime/src/tests.rs` (import from `test_support`, drop moved helpers)

**Interfaces:**
- Produces (all `pub(crate)`, available to every `*_tests.rs`): the helpers migrated from `tests.rs` — `compile_wat(wat: &str) -> Vec<u8>`, `wat_assert_buffer_eq(start: i32, expected: &str) -> String`, and the two `#[cfg(unix)]`/`#[cfg(not(unix))]` `browser_exit_status(code: i32) -> std::process::ExitStatus` variants.

- [ ] **Step 1: Add the dev-dependency.** In `crates/kali_runtime/Cargo.toml`, under `[dev-dependencies]`, add:
```toml
kali_test_support = { workspace = true }
```

- [ ] **Step 2: Enumerate the existing local test helpers to migrate.**
```bash
cd /workspace
grep -nE "^(#\[cfg.*\]\n)?fn (compile_wat|wat_assert_buffer_eq|browser_exit_status)" crates/kali_runtime/src/tests.rs
grep -nE "^fn (compile_wat|wat_assert_buffer_eq|browser_exit_status)" crates/kali_runtime/src/tests.rs
```
Expected: `compile_wat` (≈12), `wat_assert_buffer_eq` (≈16), and the two `browser_exit_status` cfg variants (≈29 / ≈35).

- [ ] **Step 3: Create `test_support.rs`.** Move the helpers here verbatim, each declared `pub(crate)` (keep the `#[cfg(unix)]` / `#[cfg(not(unix))]` attributes on the two `browser_exit_status` variants). Header:
```rust
//! kali_runtime-specific test builders (compiled under cfg(test)).
use crate::*;

// pub(crate) fn compile_wat(wat: &str) -> Vec<u8> { ... }
// pub(crate) fn wat_assert_buffer_eq(start: i32, expected: &str) -> String { ... }
// #[cfg(unix)] pub(crate) fn browser_exit_status(code: i32) -> std::process::ExitStatus { ... }
// #[cfg(not(unix))] pub(crate) fn browser_exit_status(code: i32) -> std::process::ExitStatus { ... }
```

- [ ] **Step 4: Declare the module under cfg(test).** In `crates/kali_runtime/src/lib.rs`, add (just above the `tests` wiring):
```rust
#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
```

- [ ] **Step 5: Update `tests.rs`.** Delete the moved helper fns from `tests.rs`; at the top replace `use super::*;` with:
```rust
use crate::*;
use crate::test_support::*;
```
Keep the `use std::{…}`, `#[cfg(unix)] use std::os::unix::fs::symlink;`, and any other existing imports. Leave all `#[test]` fns unchanged (they call the same helper names, now resolved from `test_support`).

- [ ] **Step 6: Build and test.**
```bash
cargo test -p kali_runtime 2>&1 | tail -5
```
Expected: `test result: ok.` 158.

- [ ] **Step 7: Commit.**
```bash
git add crates/kali_runtime/Cargo.toml crates/kali_runtime/src/lib.rs crates/kali_runtime/src/test_support.rs crates/kali_runtime/src/tests.rs Cargo.lock
git commit -m "refactor(kali_runtime): add test_support (wat builders) [refactor]"
```

---

### Task 13: Split `tests.rs` into sibling `*_tests.rs` per module

**Files:**
- Create per destination module: `ctx_tests.rs`, `execute_tests.rs`, `state_tests.rs`, `profiles_tests.rs`, and under the directories `browser/contract_tests.rs`, `browser/command_tests.rs`, `browser/harness_tests.rs`, `browser/execute_tests.rs`, `browser/summary_tests.rs`, plus `host/*_tests.rs` only for modules that actually receive tests
- Modify: each corresponding source module (add the `#[cfg(test)] #[path] mod` wiring)
- Delete (at end): `crates/kali_runtime/src/tests.rs` and its facade wiring

**Classification rule:** For each `#[test]` fn, read its body and assign it to the module whose behavior it exercises. Name-prefix guide (verify against the body when ambiguous):

| Destination test file              | Test clusters (by name)                                                        |
|------------------------------------|--------------------------------------------------------------------------------|
| `ctx_tests.rs`                     | `runtime_context_*` (env snapshots, profiles, process identity)                |
| `execute_tests.rs`                 | `runtime_executes_*`, `runtime_exposes_*`, `runtime_records_*` (end-to-end run via `compile_wat`) |
| `profiles_tests.rs`                | `*_host_contract_*`, `normalize_runtime_profiles_*`, backend/contract label tests |
| `browser/contract_tests.rs`        | `browser_runtime_contract_*`                                                    |
| `browser/command_tests.rs`         | `split_command_spec_*`, `browser_harness_command_parts_*`, `browser_harness_recognizes_*`, executable-name tests |
| `browser/harness_tests.rs`         | `browser_runtime_harness_*`, `browser_bundle_harness_*`, `browser_*_harness_page/script_*` |
| `browser/execute_tests.rs`         | `browser_harness_invocation_*`, `browser_harness_launch_*`, `browser_requested_*`, `browser_runtime_execution_*` |
| `browser/summary_tests.rs`         | `browser_runtime_summary_*`, `browser_runtime_harness_summary_*`               |
| `state_tests.rs`                   | timer/microtask/event-listener/thread/spawn `KaliHostState` tests, if any      |

When a test drives the full pipeline, place it with the **most specific** surface it asserts on. When still ambiguous, place it with the source module whose function the test names most directly.

**Renaming rule:** Keep test names **identical** so the Task 15 basename diff is empty. If a collision arises after moving (two modules with same-named tests), rename minimally and record every rename for Task 15.

- [ ] **Step 1: Worklist.** Count tests to place:
```bash
cd /workspace
grep -cE "^\s*fn [a-z0-9_]+\(\)" crates/kali_runtime/src/tests.rs
grep -c '#\[test\]' crates/kali_runtime/src/tests.rs
```
Expected: 158 `#[test]` attributes.

- [ ] **Step 2:** For each destination module `<m>`, create `<m>_tests.rs` with header:
```rust
use crate::*;
use crate::test_support::*;
use std::{fs, io::{Read, Write}, net::TcpListener, thread};
```
Trim each file's `use` block to only what its tests reference (drop `TcpListener`/`thread`/`fs` where unused; add `#[cfg(unix)] use std::os::unix::fs::symlink;` only where a test uses it). Copy any other `use` the moved tests need from the current `tests.rs` import block.

- [ ] **Step 3:** Move the classified tests into their destination files. Wire each into its source module by adding at the **bottom of the source file**, e.g. in `browser/command.rs`:
```rust
#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;
```
`#[path]` is resolved relative to the directory of the file containing the `mod` declaration, so a directory-module source file uses the bare filename (`#[path = "command_tests.rs"]` → `browser/command_tests.rs`). Top-level source files (`ctx.rs`, `execute.rs`, …) wire `#[path = "ctx_tests.rs"]`, etc.

- [ ] **Step 4:** Move tests in batches by destination module, running after each and committing per module:
```bash
cargo test -p kali_runtime 2>&1 | tail -5
git add -A && git commit -m "test(kali_runtime): co-locate <module> tests [refactor]"
```
Expected after each batch: green, and the running total of co-located + remaining-in-`tests.rs` tests stays at 158.

- [ ] **Step 5:** When `tests.rs` has no `#[test]` fns left, delete it and remove its wiring:
```bash
rm crates/kali_runtime/src/tests.rs
```
Remove the `#[cfg(test)] #[path = "tests.rs"] mod tests;` lines from `lib.rs`.

- [ ] **Step 6:** `cargo test -p kali_runtime 2>&1 | tail -5`. Green, 158.
- [ ] **Step 7:** `git add -A && git commit -m "test(kali_runtime): remove monolithic tests.rs [refactor]"`

---

### Task 14: Adopt `kali_test_support::fixtures` where it reduces boilerplate

**Files:**
- Modify: the new `*_tests.rs` files

- [ ] **Step 1: Find filesystem/manifest test sites.**
```bash
cd /workspace
grep -rnE "tempfile::tempdir|tempdir\(\)|fs::write|fs::create_dir|fs::File::create" crates/kali_runtime/src/*_tests.rs crates/kali_runtime/src/**/*_tests.rs | wc -l
```
Expected: a substantial count (the ~249 fs/tempdir sites noted in the design, now spread across the co-located files).
- [ ] **Step 2:** Where a test creates a temp dir / writes files, replace the hand-rolled setup with `kali_test_support::fixtures::{tempdir, write_file, write_manifest}` where it shortens the test without obscuring intent. Do **not** convert sites where the explicit `tempfile`/`fs` calls are clearer, or where a fixture helper doesn't match the exact shape needed (e.g. symlinks, `TcpListener`, custom permissions). Assertions and the bytes written must stay identical. This adoption keeps the **test count at 158** — no tests added or removed (unlike `kali_codegen`, which had no fs tests and needed a one-off added fixture test).
- [ ] **Step 3:** `cargo test -p kali_runtime 2>&1 | tail -5`. Green, 158.
- [ ] **Step 4:** `git add -A && git commit -m "test(kali_runtime): adopt kali_test_support fixtures in co-located tests [refactor]"`

---

### Task 15: Final verification, lint, and baseline diff

**Files:**
- Create: `docs/superpowers/baselines/kali_runtime-tests-after.txt`
- Create: `docs/superpowers/baselines/kali_runtime-tests-renames.md`

- [ ] **Step 1: Regenerate the after-snapshot (basenames, prefix stripped):**
```bash
cd /workspace
cargo test -p kali_runtime -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort > docs/superpowers/baselines/kali_runtime-tests-after.txt
```

- [ ] **Step 2: Diff before vs after (basename sets).** They must be identical (every test moved, none lost/renamed):
```bash
diff docs/superpowers/baselines/kali_runtime-tests-before.txt docs/superpowers/baselines/kali_runtime-tests-after.txt && echo "IDENTICAL"
wc -l docs/superpowers/baselines/kali_runtime-tests-*.txt
```
Expected: `IDENTICAL`, both files 158 lines. Record the result in `docs/superpowers/baselines/kali_runtime-tests-renames.md` (write "No renames — test basename set identical." if the diff is empty; otherwise list every before→after rename and why).

- [ ] **Step 3: Format and lint:**
```bash
cargo fmt -p kali_runtime
cargo clippy -p kali_runtime --all-targets -- -D warnings 2>&1 | tail -15
```
Expected: no clippy errors.

- [ ] **Step 4: Full-workspace sanity** (kali_runtime feeds `kali_cli`/`kali_capi`/…):
```bash
cargo build --workspace 2>&1 | tail -5
cargo test -p kali_runtime 2>&1 | tail -5
```
Expected: workspace builds; kali_runtime tests green, 158.

- [ ] **Step 5: Per-file size check** (confirm the monolith is gone):
```bash
find crates/kali_runtime/src -name '*.rs' | xargs wc -l | sort -rn | head -25
```
Expected: no single file near the old 4,599 / 7,149 line counts; `lib.rs` is a thin facade; `imports_default.rs` / `imports_node.rs` remain the largest source files (~770 / ~800, intact by design).

- [ ] **Step 6: Commit baselines:**
```bash
git add docs/superpowers/baselines/kali_runtime-tests-after.txt docs/superpowers/baselines/kali_runtime-tests-renames.md
git commit -m "test(kali_runtime): record post-refactor baseline + renames [refactor]"
```

- [ ] **Step 7: STOP for review.** Summarize: per-file line counts before/after, the rename mapping (ideally empty), and confirmation that the test count is unchanged (158) and the suite + full workspace are green. Recommend a whole-branch opus review before merge. Then update the `kali-crate-modularization` memory to mark `kali_runtime` done and note the next candidate crate.

---

## Self-Review Notes (for the implementer)

- **Giant functions move intact.** `execute_inner`, `register_default_host_imports`, `register_node_host_imports` are relocated byte-for-byte. Splitting them into helpers is a separate, out-of-scope logic refactor (mirrors how `kali_codegen` deferred cracking `emit_call`).
- If a moved item references a name that compiled only because everything lived in one module and you missed it in Task 2, fix it by adding `pub(crate)` to *that* item — never by altering a function body.
- `use crate::*;` is used in **every** extracted module (top-level and directory submodules) — it surfaces the crate's `use` import block **and all crate-root re-exports** added per the Crate-root re-export rule. That rule is what makes cross-module free-fn calls (e.g. `append_stdout`, `guest_memory`, `enforce_operation`, `browser_runtime_summary_for_outcome`) resolve after extraction. If a bare `const` isn't surfaced, qualify it as `crate::<CONST>` at the use site (mechanical, no logic change). Domain consts (`STRING_HANDLE_TAG`, `BROWSER_HARNESS_*`) move *with* their module; the public `BROWSER_HARNESS_COMMAND_ENV` is re-exported from the facade.
- If a `pub(crate) use <module>::*;` glob trips an `unused_imports`/clippy warning under `-D warnings` (because nothing outside the module references it), replace that glob with an explicit `pub(crate) use <module>::{the, names, actually, used, cross-module};` list — never drop a re-export a caller depends on. Resolve these at the Task 11 fmt/clippy pass.
- The `#[path]` attribute for a sibling test file is resolved **relative to the directory of the file containing the `mod` declaration** — always the bare filename (`#[path = "command_tests.rs"]`), never a path with directories.
- Two distinct items share the name `env_snapshot_value`: a free fn (`fn env_snapshot_value(env: &BTreeMap<…>)`) and methods on `RuntimeCtx`/`KaliHostState`. They are different items — keep all of them.
- Do not change any `Cargo.toml` dependency versions; the only manifest edit is the `kali_test_support` dev-dependency added in Task 12.
- Keep test names identical through the split so the Task 15 basename diff stays empty; that is the strongest proof of zero behavior change.
- Line numbers (≈) are from the pre-refactor `lib.rs` and drift as items are removed — locate items by **name**, using the numbers only as a starting hint.
```
