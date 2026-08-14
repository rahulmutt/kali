# Test Binary Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Shrink `target/` from 79 GB to ~8 GB and reduce `kali_cli`'s 328 integration test targets to ~9, by unlinking wasmtime from the test binaries and replacing hand-written black-box CLI tests with TOML case files driven by one runner.

**Architecture:** Two independent phases. Phase 1 extracts the four wasmtime-free modules that 154 test binaries import (`profiles.rs`, `browser/{contract,command,harness}.rs`) into a new leaf crate `kali_runtime_contract`, which `kali_runtime` re-exports so no `src/**` consumer changes. Phase 2 adds a `kali_case_runner` library crate (parser, matrix expander, assertion evaluator, three step kinds) and one `harness = false` test target in `kali_cli` that discovers `tests/cases/**/*.toml`, then migrates families behind a literal-coverage audit gate.

**Tech Stack:** Rust 1.97.1 (pinned in `mise.toml`), `libtest-mimic` 0.8 (custom test harness), `toml` 0.9 (case files), `serde`/`serde_json`, `tempfile`. Python 3.14 for the one-shot audit script, matching the existing `scripts/split_inline_rust_tests.py` precedent.

**Spec:** `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`

**Branch:** `test-binary-consolidation-design` (already created; the spec is committed there as `576b47152c`).

## Global Constraints

- **Pure Rust only.** No embedded C/C++ dependencies (`AGENTS.md` §3 hard invariant). `libtest-mimic` 0.8.2 and `toml` 0.9 are both pure Rust; verified available from crates.io.
- **Rust unit tests live in sibling `*tests.rs` files, not inline `#[cfg(test)]` modules** (`AGENTS.md` §5). Every moved module keeps its `#[cfg(test)] #[path = "x_tests.rs"] mod x_tests;` footer, and every new module gets one.
- **Cargo target dir is `/workspace/.cache/cargo-target`**, set by `.cargo/config.toml`. All size measurements read from there, never from `./target` (which does not exist).
- **Workspace dependency versions are declared once** in the root `Cargo.toml` `[workspace.dependencies]` and referenced as `{ workspace = true }`. Never pin a version in a member crate.
- **New workspace members must be added to the root `Cargo.toml` `members` list** and, if depended upon by another member, to `[workspace.dependencies]` with a `path`.
- **Phase 1 changes no test assertion.** Its verification is `bash scripts/test-gate.sh` reporting `GATE OK: 0 failing tests` with an unchanged total test count.
- **Do not edit `crates/kali_cli/src/**` in Phase 1.** The re-export in `kali_runtime` exists precisely so that stays true.
- **Do not modify `scripts/check-determinism.sh`, `scripts/test-gate.sh`, `mise.toml`, or `.github/workflows/ci.yml`.** They reference `--test runtime_smoke`, `--test browser_cdp_smoke`, and `--test package_corpus`, all of which survive as hand-written targets. If a task appears to require changing them, stop and report.
- **Commit after every task.** Use Conventional Commits with the scope conventions already in the log (`feat(codegen):`, `fix(types):`, `docs(register):`, `test(switch):`).

---

# File Structure

## Phase 1 — new crate

```
crates/kali_runtime_contract/
  Cargo.toml                        # deps: kali_error, serde_json, base64
  src/lib.rs                        # module decls + explicit pub re-exports
  src/profiles.rs                   # MOVED from kali_runtime/src/profiles.rs
  src/profiles_tests.rs             # MOVED (+ profiles_tests/ dir)
  src/browser/mod.rs                # new: 3 pub(crate) mod decls
  src/browser/contract.rs           # MOVED
  src/browser/contract_tests.rs     # MOVED
  src/browser/command.rs            # MOVED
  src/browser/command_tests.rs      # MOVED (+ command_tests/ dir)
  src/browser/harness.rs            # MOVED
  src/browser/harness_tests.rs      # MOVED
```

## Phase 1 — modified

```
Cargo.toml                                  # members + workspace.dependencies
crates/kali_runtime/Cargo.toml              # + kali_runtime_contract dep
crates/kali_runtime/src/lib.rs              # mod decls removed, re-exports added
crates/kali_runtime/src/browser/mod.rs      # 3 mod decls removed
crates/kali_cli/Cargo.toml                  # + kali_runtime_contract dev-dep, + [[test]] inprocess
crates/kali_cli/tests/*.rs                  # 162 files: import path rewrite
crates/kali_cli/tests/inprocess.rs          # new: consolidates 3 fat targets
crates/kali_cli/tests/inprocess/            # new: the 3 moved files as modules
crates/kali_cli/tests/schema_validation.rs  # kali_cli::build -> subprocess
```

## Phase 2 — new crate

Each file has one responsibility and its own sibling test file. The runner lives
in a library crate, not in the test target, so its logic is unit-testable without
a `harness = false` binary.

```
crates/kali_case_runner/
  Cargo.toml
  src/lib.rs                # pub struct RunnerConfig; pub fn trials(); pub fn run()
  src/model.rs              # serde types for a case file (deny_unknown_fields)
  src/model_tests.rs
  src/expand.rs             # matrix product + ${...} substitution
  src/expand_tests.rs
  src/jsonpath.rs           # dotted-path lookup + toml/json value equality
  src/jsonpath_tests.rs
  src/assertions.rs         # the 8 assertion keys against a captured Output
  src/assertions_tests.rs
  src/discover.rs           # walk the case tree, sorted; error on empty
  src/discover_tests.rs
  src/steps.rs              # dispatch: cli | file_json | browser_bundle_harness
  src/steps_tests.rs
```

## Phase 2 — modified / added in `kali_cli`

```
crates/kali_cli/Cargo.toml            # + [[test]] cases (harness = false), + dev-deps
crates/kali_cli/tests/cases.rs        # thin main: ~20 lines
crates/kali_cli/tests/cases/          # the .toml case tree, by family
scripts/audit-case-migration.py       # the §6.2 literal-coverage gate
```

---

# Phase 1 — Unlink wasmtime from the test binaries

Phase 1 order is forced by intra-module dependencies: `profiles.rs` defines
`RuntimeBackend`/`RuntimeHostContract` used by `contract.rs`; `contract.rs`
defines `BROWSER_HARNESS_COMMAND_ENV` used by `command.rs`. Move in that order so
every intermediate state compiles.

Four `pub(crate)` items are consumed by files that stay behind and must be
promoted to `pub`. This list is complete — it was derived by grepping every
`pub(crate)` item in the four moved files against every consumer outside them:

| item | file | consumer that stays |
| --- | --- | --- |
| `parse_optional_runtime_host_contract_label` | `profiles.rs` | `browser/summary.rs` |
| `parse_optional_runtime_backend_label` | `profiles.rs` | `browser/summary.rs` |
| `BROWSER_HARNESS_SUMMARY_FILE_ENV` | `browser/contract.rs` | `browser/execute.rs` |
| `browser_harness_uses_html_entrypoint` | `browser/command.rs` | `browser/execute.rs` |

All four moved files currently begin with `use crate::*;`. Replacing that glob
with explicit imports is the substantive work. The exact required imports, derived
by identifier survey, are given per task.

---

### Task 1: Scaffold `kali_runtime_contract` and move `profiles.rs`

**Files:**
- Create: `crates/kali_runtime_contract/Cargo.toml`
- Create: `crates/kali_runtime_contract/src/lib.rs`
- Move: `crates/kali_runtime/src/profiles.rs` → `crates/kali_runtime_contract/src/profiles.rs`
- Move: `crates/kali_runtime/src/profiles_tests.rs` → `crates/kali_runtime_contract/src/profiles_tests.rs`
- Move: `crates/kali_runtime/src/profiles_tests/` → `crates/kali_runtime_contract/src/profiles_tests/`
- Modify: `Cargo.toml` (root — members, workspace.dependencies)
- Modify: `crates/kali_runtime/Cargo.toml`
- Modify: `crates/kali_runtime/src/lib.rs`

**Interfaces:**
- Consumes: nothing (first task).
- Produces: crate `kali_runtime_contract` exporting `RuntimeHostContract`,
  `RuntimeBackend`, `normalize_runtime_profiles(Vec<String>) -> Vec<String>`,
  `parse_optional_runtime_host_contract_label`,
  `parse_optional_runtime_backend_label`. `kali_runtime` re-exports
  `RuntimeHostContract`, `RuntimeBackend`, `normalize_runtime_profiles` at their
  existing paths.

- [ ] **Step 1: Capture the baseline test count**

This number is the Phase 1 contract. Every task in Phase 1 must preserve it.

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/baseline.txt | grep -E '^test result:' | \
  awk -F'[ ;]' '{p+=$4; f+=$7} END {print "passed:", p, "failed:", f}'
```

Record the output in the commit message for Step 8. Expect `failed: 0`.

- [ ] **Step 2: Create the crate manifest**

`crates/kali_runtime_contract/Cargo.toml`:

```toml
[package]
name = "kali_runtime_contract"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
kali_error = { workspace = true }
serde_json = { workspace = true }
base64 = { workspace = true }

[dev-dependencies]
kali_test_support = { workspace = true }
```

`base64` is unused until Task 4 (`harness.rs`); declaring it now avoids a second
manifest edit. Rust will warn about an unused crate only under
`-W unused-crate-dependencies`, which this workspace does not enable.

- [ ] **Step 3: Register the crate in the workspace**

In the root `Cargo.toml`, add to `members` immediately after
`"crates/kali_runtime",` (members are grouped by pipeline order, not alphabetical):

```toml
    "crates/kali_runtime_contract",
```

And to `[workspace.dependencies]`, immediately after the `kali_runtime` line:

```toml
kali_runtime_contract = { path = "crates/kali_runtime_contract" }
```

- [ ] **Step 4: Move `profiles.rs` and its tests with git mv**

Use `git mv` so history follows the file.

```bash
cd /workspace
mkdir -p crates/kali_runtime_contract/src
git mv crates/kali_runtime/src/profiles.rs        crates/kali_runtime_contract/src/profiles.rs
git mv crates/kali_runtime/src/profiles_tests.rs  crates/kali_runtime_contract/src/profiles_tests.rs
git mv crates/kali_runtime/src/profiles_tests     crates/kali_runtime_contract/src/profiles_tests
```

- [ ] **Step 5: Replace the `use crate::*;` glob and promote two items**

In `crates/kali_runtime_contract/src/profiles.rs`, replace line 3
(`use crate::*;`) with the only two external identifiers the file uses:

```rust
use serde_json::Value;
use std::collections::BTreeSet;
```

Then promote the two functions `browser/summary.rs` still needs. Change:

```rust
pub(crate) fn parse_optional_runtime_host_contract_label(
```

to:

```rust
pub fn parse_optional_runtime_host_contract_label(
```

and likewise for `parse_optional_runtime_backend_label`. Leave
`parse_runtime_host_contract_label` and `parse_runtime_backend_label` as
`pub(crate)` — they are used only inside this file.

- [ ] **Step 6: Write `src/lib.rs`**

`crates/kali_runtime_contract/src/lib.rs`:

```rust
//! Runtime contract surface without the runtime.
//!
//! Holds the declarative half of `kali_runtime`: host-contract and backend
//! labels, profile normalization, the browser runtime contract, browser harness
//! command resolution, and browser harness script generation. None of it links
//! wasmtime, which is the entire reason this crate exists — 154 `kali_cli`
//! integration test binaries import these items and would otherwise each carry
//! a ~400 MB statically linked wasmtime.
//!
//! See `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`.

mod profiles;
pub use profiles::{
    normalize_runtime_profiles, parse_optional_runtime_backend_label,
    parse_optional_runtime_host_contract_label, RuntimeBackend, RuntimeHostContract,
};
```

- [ ] **Step 7: Re-export from `kali_runtime` so no consumer changes**

In `crates/kali_runtime/Cargo.toml`, add to `[dependencies]` after the
`kali_sandbox` line:

```toml
kali_runtime_contract = { workspace = true }
```

In `crates/kali_runtime/src/lib.rs`, replace these three lines:

```rust
mod profiles;
pub(crate) use profiles::*;
pub use profiles::{normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract};
```

with:

```rust
pub(crate) use kali_runtime_contract::*;
pub use kali_runtime_contract::{
    normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract,
};
```

The `pub(crate) use ...::*` keeps `use crate::*;` in the remaining `kali_runtime`
modules resolving `RuntimeBackend`, `RuntimeHostContract`, and the two promoted
`parse_optional_*` functions exactly as before.

- [ ] **Step 8: Verify the workspace is green with an unchanged test count**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' | \
  awk -F'[ ;]' '{p+=$4; f+=$7} END {print "passed:", p, "failed:", f}'
```

Expected: identical `passed:` to Step 1, `failed: 0`. A *lower* passed count means
`profiles_tests` stopped being compiled — check that `profiles.rs` still ends with
its `#[cfg(test)] #[path = "profiles_tests.rs"] mod profiles_tests;` footer.

- [ ] **Step 9: Commit**

```bash
cd /workspace
git add -A crates/kali_runtime_contract crates/kali_runtime Cargo.toml Cargo.lock
git commit -m "refactor(runtime): extract profiles into kali_runtime_contract

First move into the new wasmtime-free leaf crate. profiles.rs needed only
serde_json::Value and BTreeSet, so it carries no runtime dependency. kali_runtime
re-exports every moved symbol at its existing path, so no src/** consumer changes.

Promotes parse_optional_runtime_{host_contract,backend}_label to pub because
browser/summary.rs stays behind and still calls them.

Test count unchanged: <N> passed, 0 failed."
```

---

### Task 2: Move `browser/contract.rs`

**Files:**
- Create: `crates/kali_runtime_contract/src/browser/mod.rs`
- Move: `crates/kali_runtime/src/browser/contract.rs` → `crates/kali_runtime_contract/src/browser/contract.rs`
- Move: `crates/kali_runtime/src/browser/contract_tests.rs` → `crates/kali_runtime_contract/src/browser/contract_tests.rs`
- Modify: `crates/kali_runtime_contract/src/lib.rs`
- Modify: `crates/kali_runtime/src/lib.rs`
- Modify: `crates/kali_runtime/src/browser/mod.rs`

**Interfaces:**
- Consumes: `RuntimeBackend`, `RuntimeHostContract` from Task 1.
- Produces: `kali_runtime_contract` additionally exports
  `BROWSER_HARNESS_COMMAND_ENV: &str`, `BROWSER_HARNESS_SUMMARY_FILE_ENV: &str`,
  `BrowserRuntimeContract`, `BrowserRuntimeContractDescriptor`,
  `browser_runtime_contract_value`, `browser_runtime_request_context`,
  `browser_runtime_unavailable_diagnostic`.

- [ ] **Step 1: Move the files**

```bash
cd /workspace
mkdir -p crates/kali_runtime_contract/src/browser
git mv crates/kali_runtime/src/browser/contract.rs       crates/kali_runtime_contract/src/browser/contract.rs
git mv crates/kali_runtime/src/browser/contract_tests.rs crates/kali_runtime_contract/src/browser/contract_tests.rs
```

- [ ] **Step 2: Replace the glob and promote one constant**

In `crates/kali_runtime_contract/src/browser/contract.rs`, replace line 2
(`use crate::*;`) with:

```rust
use crate::{RuntimeBackend, RuntimeHostContract};
use kali_error::{
    _error_codes::e5, Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
```

Then promote the constant `browser/execute.rs` still needs. Change:

```rust
pub(crate) const BROWSER_HARNESS_SUMMARY_FILE_ENV: &str = "KALI_BROWSER_HARNESS_SUMMARY_FILE";
```

to:

```rust
pub const BROWSER_HARNESS_SUMMARY_FILE_ENV: &str = "KALI_BROWSER_HARNESS_SUMMARY_FILE";
```

Leave `browser_runtime_contract_descriptor_is_canonical` as `pub(crate)` — it has
no consumer outside this file.

- [ ] **Step 3: Create the browser module and wire it into lib.rs**

`crates/kali_runtime_contract/src/browser/mod.rs`:

```rust
//! Declarative browser-runtime surface: contract, command resolution, and
//! harness script generation. No wasmtime, no reqwest, no sandbox.
pub(crate) mod contract;
```

In `crates/kali_runtime_contract/src/lib.rs`, add after the `profiles` block:

```rust
mod browser;
pub use browser::contract::{
    browser_runtime_contract_value, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, BrowserRuntimeContract,
    BrowserRuntimeContractDescriptor, BROWSER_HARNESS_COMMAND_ENV,
    BROWSER_HARNESS_SUMMARY_FILE_ENV,
};
```

- [ ] **Step 4: Remove the module from `kali_runtime` and re-export**

In `crates/kali_runtime/src/browser/mod.rs`, delete the line:

```rust
pub(crate) mod contract;
```

In `crates/kali_runtime/src/lib.rs`, change:

```rust
pub(crate) use browser::{command::*, contract::*, summary::*};
```

to:

```rust
pub(crate) use browser::{command::*, summary::*};
```

and replace the `pub use browser::contract::{...}` block with:

```rust
pub use kali_runtime_contract::{
    browser_runtime_contract_value, browser_runtime_request_context,
    browser_runtime_unavailable_diagnostic, BrowserRuntimeContract,
    BrowserRuntimeContractDescriptor, BROWSER_HARNESS_COMMAND_ENV,
};
```

`BROWSER_HARNESS_SUMMARY_FILE_ENV` reaches `browser/execute.rs` through the
existing `pub(crate) use kali_runtime_contract::*;` added in Task 1 Step 7. Do not
add it to the `pub use` list — it was not public before, and widening the public
API is out of scope.

- [ ] **Step 5: Verify**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' | \
  awk -F'[ ;]' '{p+=$4; f+=$7} END {print "passed:", p, "failed:", f}'
```

Expected: same `passed:` as Task 1 Step 1, `failed: 0`.

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add -A crates/kali_runtime_contract crates/kali_runtime
git commit -m "refactor(runtime): move browser contract into kali_runtime_contract

contract.rs needed only kali_error diagnostics, serde_json, BTreeSet, and the
profiles enums moved in the previous commit — no wasmtime path. Promotes
BROWSER_HARNESS_SUMMARY_FILE_ENV to pub for browser/execute.rs, which stays.

Test count unchanged: <N> passed, 0 failed."
```

---

### Task 3: Move `browser/command.rs`

**Files:**
- Move: `crates/kali_runtime/src/browser/command.rs` → `crates/kali_runtime_contract/src/browser/command.rs`
- Move: `crates/kali_runtime/src/browser/command_tests.rs` → `crates/kali_runtime_contract/src/browser/command_tests.rs`
- Move: `crates/kali_runtime/src/browser/command_tests/` → `crates/kali_runtime_contract/src/browser/command_tests/`
- Modify: `crates/kali_runtime_contract/src/browser/mod.rs`
- Modify: `crates/kali_runtime_contract/src/lib.rs`
- Modify: `crates/kali_runtime/src/lib.rs`
- Modify: `crates/kali_runtime/src/browser/mod.rs`

**Interfaces:**
- Consumes: `BROWSER_HARNESS_COMMAND_ENV` from Task 2.
- Produces: `kali_runtime_contract` additionally exports
  `browser_harness_command_parts() -> Vec<String>`,
  `browser_harness_command_parts_for(Option<&str>) -> Vec<String>`,
  `browser_harness_command_parts_checked(Option<&str>) -> Result<Vec<String>, String>`,
  `browser_harness_uses_html_entrypoint(&str) -> bool`,
  `split_command_spec`.

- [ ] **Step 1: Move the files**

```bash
cd /workspace
git mv crates/kali_runtime/src/browser/command.rs       crates/kali_runtime_contract/src/browser/command.rs
git mv crates/kali_runtime/src/browser/command_tests.rs crates/kali_runtime_contract/src/browser/command_tests.rs
git mv crates/kali_runtime/src/browser/command_tests    crates/kali_runtime_contract/src/browser/command_tests
```

- [ ] **Step 2: Replace the glob and promote one function**

In `crates/kali_runtime_contract/src/browser/command.rs`, replace `use crate::*;`
with:

```rust
use super::contract::BROWSER_HARNESS_COMMAND_ENV;
use std::path::Path;
```

Then promote the function `browser/execute.rs` still needs. Change:

```rust
pub(crate) fn browser_harness_uses_html_entrypoint(executable: &str) -> bool {
```

to:

```rust
pub fn browser_harness_uses_html_entrypoint(executable: &str) -> bool {
```

Leave `browser_harness_normalized_executable_name`,
`BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES`,
`browser_harness_is_browser_executable_name`,
`browser_harness_command_parts_for_browser_executable`,
`browser_harness_default_command_parts_from`, and
`browser_harness_default_command_parts` as `pub(crate)` — all are used only
inside this file.

- [ ] **Step 3: Wire the module**

In `crates/kali_runtime_contract/src/browser/mod.rs`, add:

```rust
pub(crate) mod command;
```

In `crates/kali_runtime_contract/src/lib.rs`, add:

```rust
pub use browser::command::{
    browser_harness_command_parts, browser_harness_command_parts_checked,
    browser_harness_command_parts_for, browser_harness_uses_html_entrypoint,
    split_command_spec,
};
```

- [ ] **Step 4: Remove from `kali_runtime` and re-export**

In `crates/kali_runtime/src/browser/mod.rs`, delete:

```rust
pub(crate) mod command;
```

In `crates/kali_runtime/src/lib.rs`, change:

```rust
pub(crate) use browser::{command::*, summary::*};
```

to:

```rust
pub(crate) use browser::summary::*;
```

and replace the `pub use browser::command::{...}` block with:

```rust
pub use kali_runtime_contract::{
    browser_harness_command_parts, browser_harness_command_parts_checked,
    browser_harness_command_parts_for, split_command_spec,
};
```

Note `browser_harness_uses_html_entrypoint` is deliberately absent from this
list: it was `pub(crate)` before and reaches `browser/execute.rs` via the glob.

- [ ] **Step 5: Verify**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' | \
  awk -F'[ ;]' '{p+=$4; f+=$7} END {print "passed:", p, "failed:", f}'
```

Expected: same `passed:`, `failed: 0`.

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add -A crates/kali_runtime_contract crates/kali_runtime
git commit -m "refactor(runtime): move browser command resolution into kali_runtime_contract

command.rs needed only std::path::Path and the contract's env-var constant.
Promotes browser_harness_uses_html_entrypoint to pub for browser/execute.rs.

Test count unchanged: <N> passed, 0 failed."
```

---

### Task 4: Move `browser/harness.rs`

This is the largest move (1,183 lines) but the simplest: `harness.rs` references
no symbol outside its own module except `serde_json`, `std::fs`, and the base64
engine.

**Files:**
- Move: `crates/kali_runtime/src/browser/harness.rs` → `crates/kali_runtime_contract/src/browser/harness.rs`
- Move: `crates/kali_runtime/src/browser/harness_tests.rs` → `crates/kali_runtime_contract/src/browser/harness_tests.rs`
- Modify: `crates/kali_runtime_contract/src/browser/mod.rs`
- Modify: `crates/kali_runtime_contract/src/lib.rs`
- Modify: `crates/kali_runtime/src/lib.rs`
- Modify: `crates/kali_runtime/src/browser/mod.rs`

**Interfaces:**
- Consumes: nothing from Tasks 1–3.
- Produces: `kali_runtime_contract` additionally exports
  `browser_bundle_harness_script(bundle_dir: &str, allow_subpaths: bool, body: &str) -> String`,
  `browser_bundle_harness_prelude(bundle_dir: &str, allow_subpaths: bool) -> String`,
  `browser_bundle_harness_page(bundle_dir: &str, body: &str) -> String`,
  `browser_bundle_runtime_harness_module_script`,
  `browser_bundle_runtime_harness_page`, `browser_bundle_runtime_harness_script`,
  `browser_runtime_harness_page`, `browser_runtime_harness_script`,
  `BROWSER_HARNESS_DONE_BINDING: &str`.

- [ ] **Step 1: Move the files**

```bash
cd /workspace
git mv crates/kali_runtime/src/browser/harness.rs       crates/kali_runtime_contract/src/browser/harness.rs
git mv crates/kali_runtime/src/browser/harness_tests.rs crates/kali_runtime_contract/src/browser/harness_tests.rs
```

- [ ] **Step 2: Replace the glob**

In `crates/kali_runtime_contract/src/browser/harness.rs`, replace `use crate::*;`
with:

```rust
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use std::fs;
```

`serde_json` is referenced by fully-qualified path (`serde_json::to_string`,
`serde_json::json!`) so it needs no `use`. No promotion is needed:
`browser_runtime_harness_module_script` is the only `pub(crate)` item and it is
used only inside this file.

- [ ] **Step 3: Wire the module**

In `crates/kali_runtime_contract/src/browser/mod.rs`, add:

```rust
pub(crate) mod harness;
```

In `crates/kali_runtime_contract/src/lib.rs`, add:

```rust
pub use browser::harness::{
    browser_bundle_harness_page, browser_bundle_harness_prelude, browser_bundle_harness_script,
    browser_bundle_runtime_harness_module_script, browser_bundle_runtime_harness_page,
    browser_bundle_runtime_harness_script, browser_runtime_harness_page,
    browser_runtime_harness_script, BROWSER_HARNESS_DONE_BINDING,
};
```

- [ ] **Step 4: Remove from `kali_runtime` and re-export**

In `crates/kali_runtime/src/browser/mod.rs`, delete:

```rust
pub(crate) mod harness;
```

The file should now read:

```rust
//! Browser-runtime backend: checked execution and summary parsing.
//! Contract, command resolution, and harness generation live in
//! `kali_runtime_contract`.
pub(crate) mod execute;
pub(crate) mod summary;
```

In `crates/kali_runtime/src/lib.rs`, replace the `pub use browser::harness::{...}`
block with the same symbol list sourced from the new crate:

```rust
pub use kali_runtime_contract::{
    browser_bundle_harness_page, browser_bundle_harness_prelude, browser_bundle_harness_script,
    browser_bundle_runtime_harness_module_script, browser_bundle_runtime_harness_page,
    browser_bundle_runtime_harness_script, browser_runtime_harness_page,
    browser_runtime_harness_script, BROWSER_HARNESS_DONE_BINDING,
};
```

- [ ] **Step 5: Verify, and confirm nothing wasmtime-shaped leaked in**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' | \
  awk -F'[ ;]' '{p+=$4; f+=$7} END {print "passed:", p, "failed:", f}'
# The new crate must not transitively depend on wasmtime.
cargo tree -p kali_runtime_contract 2>/dev/null | grep -E 'wasmtime|cranelift|reqwest' && \
  echo "LEAK: kali_runtime_contract reaches a heavy dep" || echo "OK: no heavy deps"
```

Expected: same `passed:`, `failed: 0`, and `OK: no heavy deps`.

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add -A crates/kali_runtime_contract crates/kali_runtime
git commit -m "refactor(runtime): move browser harness generation into kali_runtime_contract

Completes the extraction. harness.rs is the largest module moved (1183 lines) and
the most self-contained: it needed only base64, std::fs, and fully-qualified
serde_json. cargo tree confirms kali_runtime_contract reaches no wasmtime,
cranelift, or reqwest.

Test count unchanged: <N> passed, 0 failed."
```

---

### Task 5: Repoint the 162 test files and measure the drop

This is the task that actually reclaims the disk. It is mechanical: the symbols
are already re-exported, so the rewrite is a dependency swap, not a semantic
change.

**Files:**
- Modify: `crates/kali_cli/Cargo.toml`
- Modify: 162 files under `crates/kali_cli/tests/` (import path only)

**Interfaces:**
- Consumes: every symbol produced by Tasks 1–4.
- Produces: no new API. `crates/kali_cli/tests/**` no longer names
  `kali_runtime::`, except in `inprocess`-bound files handled by Task 6.

- [ ] **Step 1: Measure the before-state**

```bash
cd /workspace/.cache/cargo-target/debug/deps
ls -1 | while read f; do case "$f" in *.d|*.rlib|*.rmeta) continue;; esac
  s=$(stat -c%s "$f" 2>/dev/null || echo 0); [ "$s" -gt 104857600 ] && echo "$f"; done | wc -l
du -sh /workspace/.cache/cargo-target
```

Record both numbers. Expect ~162 and ~79G.

- [ ] **Step 2: Add the dev-dependency**

In `crates/kali_cli/Cargo.toml`, add to `[dev-dependencies]`:

```toml
kali_runtime_contract = { workspace = true }
```

- [ ] **Step 3: Rewrite the import paths**

Three files must be excluded because they need symbols that stayed behind
(`browser_runtime_execute_checked` lives in `browser/execute.rs`; `RuntimeCtx` in
`ctx.rs`). Task 6 handles them.

```bash
cd /workspace/crates/kali_cli/tests
KEEP='browser_harness_cdp_in_page_trap_propagates|release_constant_condition_loop|release_mutated_binding_specialization'
grep -rl 'kali_runtime::' . --include='*.rs' \
  | grep -vE "($KEEP)" \
  | xargs sed -i 's/\bkali_runtime::/kali_runtime_contract::/g'
echo "remaining kali_runtime:: references (should be only the 3 excluded):"
grep -rl 'kali_runtime::' . --include='*.rs' | sort
```

Expected output: exactly the three excluded filenames.

- [ ] **Step 4: Verify the workspace is still green**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' | \
  awk -F'[ ;]' '{p+=$4; f+=$7} END {print "passed:", p, "failed:", f}'
```

Expected: same `passed:` as Task 1 Step 1, `failed: 0`.

If a file fails to compile because it used a `kali_runtime` symbol that stayed
behind, do not add `kali_runtime` back as a dev-dependency — add that file to the
`KEEP` list and to Task 6's consolidation instead, and note it in the commit
message. The spec's §1.1 survey says only three such files exist; a fourth is new
information worth recording.

- [ ] **Step 5: Measure the after-state from a clean target dir**

The existing target dir holds stale 400 MB artifacts that will not be
overwritten. Measure from scratch, in a scratch target dir so the shared one is
not disturbed.

```bash
cd /workspace
CARGO_TARGET_DIR=/tmp/claude-1000/-workspace/71d27bbd-6773-42b4-8e01-64efb6b6aeb6/scratchpad/t1 \
  cargo test -p kali_cli --no-run 2>&1 | tail -3
du -sh /tmp/claude-1000/-workspace/71d27bbd-6773-42b4-8e01-64efb6b6aeb6/scratchpad/t1
cd /tmp/claude-1000/-workspace/71d27bbd-6773-42b4-8e01-64efb6b6aeb6/scratchpad/t1/debug/deps
ls -1 | while read f; do case "$f" in *.d|*.rlib|*.rmeta) continue;; esac
  s=$(stat -c%s "$f" 2>/dev/null || echo 0); [ "$s" -gt 104857600 ] && echo "$f"; done
```

Expected: the >100 MB list contains only the `kali` binary, the `kali_cli`
unit-test binary, and the 3 excluded test targets. Record the `du -sh`.

Then reclaim the scratch dir and the stale shared artifacts:

```bash
rm -rf /tmp/claude-1000/-workspace/71d27bbd-6773-42b4-8e01-64efb6b6aeb6/scratchpad/t1
cd /workspace && cargo clean && du -sh .cache/cargo-target
```

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): import the browser harness surface from kali_runtime_contract

Repoints 159 of 162 test files from kali_runtime:: to kali_runtime_contract::.
Each was importing a const &str and two pure string builders and paying ~400 MB
of statically linked wasmtime for it.

Three files are excluded because they need symbols that stayed behind:
browser_harness_cdp_in_page_trap_propagates (browser_runtime_execute_checked) and
release_{constant_condition_loop,mutated_binding_specialization} (RuntimeCtx).
The next commit consolidates those into one target.

Test count unchanged: <N> passed, 0 failed."
```

---

### Task 6: Consolidate the three runtime-linking tests into one target

Three separate ~410 MB binaries become one. Their contents are unchanged — this
is purely about linking wasmtime once.

**Files:**
- Create: `crates/kali_cli/tests/inprocess.rs`
- Move: `crates/kali_cli/tests/browser_harness_cdp_in_page_trap_propagates.rs` → `crates/kali_cli/tests/inprocess/browser_harness_cdp_in_page_trap_propagates.rs`
- Move: `crates/kali_cli/tests/release_constant_condition_loop.rs` → `crates/kali_cli/tests/inprocess/release_constant_condition_loop.rs`
- Move: `crates/kali_cli/tests/release_mutated_binding_specialization.rs` → `crates/kali_cli/tests/inprocess/release_mutated_binding_specialization.rs`
- Modify: `crates/kali_cli/Cargo.toml`

**Interfaces:**
- Consumes: nothing new.
- Produces: one test target named `inprocess`. Its test *names* are unchanged, so
  `scripts/test-gate.sh` and any `-- --exact` filter still match; only the
  containing binary differs.

- [ ] **Step 1: Move the three files into a subdirectory**

Files under `tests/<subdir>/` are not auto-discovered as test targets, so moving
them stops Cargo building three binaries.

```bash
cd /workspace/crates/kali_cli/tests
mkdir -p inprocess
for f in browser_harness_cdp_in_page_trap_propagates release_constant_condition_loop release_mutated_binding_specialization; do
  git mv "$f.rs" "inprocess/$f.rs"
done
```

- [ ] **Step 2: Write the aggregating target**

`crates/kali_cli/tests/inprocess.rs`:

```rust
//! The only `kali_cli` integration test target that links wasmtime.
//!
//! These three suites cannot be black-box: two drive `kali_runtime::RuntimeCtx`
//! in-process to assert release-profile codegen, and one calls
//! `browser_runtime_execute_checked` from `kali_runtime::browser::execute`.
//! They are aggregated into a single target so wasmtime is statically linked
//! once (~450 MB) instead of three times (~1.2 GB).
//!
//! Add a module here only when a suite genuinely needs in-process runtime
//! access. Everything else belongs in `tests/cases/` (see
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`).

#[path = "inprocess/browser_harness_cdp_in_page_trap_propagates.rs"]
mod browser_harness_cdp_in_page_trap_propagates;

#[path = "inprocess/release_constant_condition_loop.rs"]
mod release_constant_condition_loop;

#[path = "inprocess/release_mutated_binding_specialization.rs"]
mod release_mutated_binding_specialization;
```

- [ ] **Step 3: Declare the target explicitly**

`kali_cli` sets `autobins = false` but relies on test auto-discovery. Adding an
explicit `[[test]]` does not disable discovery for the other files. In
`crates/kali_cli/Cargo.toml`, add after the `[[bin]]` block:

```toml
[[test]]
name = "inprocess"
path = "tests/inprocess.rs"
```

- [ ] **Step 4: Run to verify the same tests still run, now in one binary**

```bash
cd /workspace
cargo test -p kali_cli --test inprocess 2>&1 | tail -20
```

Expected: PASS, with the union of the three files' test counts. Confirm the
per-module test names appear (e.g.
`release_constant_condition_loop::<test_name>`) — the `mod` wrapper prefixes
them, which changes the *display* name.

- [ ] **Step 5: Confirm the name change did not break the determinism gate**

`scripts/check-determinism.sh` runs `--test runtime_smoke ... -- --exact`, which
is untouched. But verify no other script or workflow filters on the three moved
test names:

```bash
cd /workspace
grep -rn 'release_constant_condition_loop\|release_mutated_binding_specialization\|browser_harness_cdp_in_page_trap_propagates' \
  scripts/ .github/ mise.toml 2>/dev/null || echo "OK: no external references"
```

Expected: `OK: no external references`. If any reference exists, stop and report
— per Global Constraints those files must not be edited.

- [ ] **Step 6: Verify the full gate and commit**

```bash
cd /workspace
bash scripts/test-gate.sh
git add -A crates/kali_cli
git commit -m "test(cli): consolidate the three in-process suites into one target

browser_harness_cdp_in_page_trap_propagates needs browser_runtime_execute_checked
and the two release_* suites need RuntimeCtx, so none can go black-box. Linking
wasmtime once instead of three times saves ~800 MB. Contents are unchanged; the
mod wrapper prefixes the displayed test names.

Verified no script or workflow filters on the three former target names."
```

---

### Task 7: Convert `schema_validation` to a subprocess test

The last fat non-runtime target. It uses `kali_cli::{build, output}` only to
produce artifacts whose JSON it then asserts on, so the `kali` binary does the
same job.

**Files:**
- Modify: `crates/kali_cli/tests/schema_validation.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces: no API. `crates/kali_cli/tests/**` no longer names `kali_cli::`.

- [ ] **Step 1: Read the file and inventory what it asserts**

```bash
cd /workspace
grep -n 'kali_cli::\|build::\|output::\|fn ' crates/kali_cli/tests/schema_validation.rs | head -40
```

Record every `#[test]` name and every assertion. The conversion must preserve all
of them; this list is the acceptance criterion for Step 4.

- [ ] **Step 2: Replace each in-process `build` call with a subprocess invocation**

Use the idiom already used by the other 324 test files. The helper to add at the
top of the file:

```rust
fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
```

For each in-process call, the subprocess equivalent runs the same command the CLI
would. For example, an in-process browser bundle build becomes:

```rust
let output = std::process::Command::new(kali_bin())
    .current_dir(dir.path())
    .arg("--output")
    .arg("json")
    .arg("build")
    .arg("--api")
    .arg("browser")
    .arg(&source_path)
    .output()
    .expect("run kali");
assert!(
    output.status.success(),
    "stdout: {}\nstderr: {}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
);
let envelope: serde_json::Value =
    serde_json::from_slice(&output.stdout).expect("json stdout");
```

Then remove `use kali_cli::{build, output};`.

If any assertion inspects a value that the CLI does not surface on stdout or in a
written artifact, that assertion cannot be converted. Do not drop it: leave that
`#[test]` in place, move the file into `tests/inprocess/` and add a `mod` line to
`tests/inprocess.rs` following Task 6's pattern, and record in the commit message
which assertion forced it. A correct 495 MB binary beats a cheap weakened one.

- [ ] **Step 3: Run the target**

```bash
cd /workspace
cargo test -p kali_cli --test schema_validation 2>&1 | tail -20
```

Expected: PASS with the same test count as Step 1's inventory.

- [ ] **Step 4: Confirm no assertion was lost**

```bash
cd /workspace
git diff crates/kali_cli/tests/schema_validation.rs | grep -E '^-.*assert' | sort > /tmp/removed.txt
git diff crates/kali_cli/tests/schema_validation.rs | grep -E '^\+.*assert' | sort > /tmp/added.txt
diff <(sed 's/^-//' /tmp/removed.txt) <(sed 's/^+//' /tmp/added.txt) && echo "OK: assertions identical"
```

A non-empty diff is expected where an assertion's *subject* changed (in-process
return value → parsed stdout). Review each such line by hand and confirm it still
claims the same thing. Any assertion that disappears with no replacement is a
failure of this task.

- [ ] **Step 5: Verify Phase 1 is complete and measure**

```bash
cd /workspace
grep -rl 'kali_runtime::\|kali_cli::' crates/kali_cli/tests --include='*.rs' | grep -v '^crates/kali_cli/tests/inprocess' || echo "OK: no test outside inprocess/ links the runtime or CLI lib"
bash scripts/test-gate.sh
cargo clean && cargo test -p kali_cli --no-run 2>&1 | tail -2
du -sh .cache/cargo-target
cd .cache/cargo-target/debug/deps && ls -1 | while read f; do case "$f" in *.d|*.rlib|*.rmeta) continue;; esac
  s=$(stat -c%s "$f" 2>/dev/null || echo 0); [ "$s" -gt 104857600 ] && printf "%5s MB  %s\n" $((s/1048576)) "$f"; done
```

Expected: `GATE OK: 0 failing tests`; the >100 MB list contains only `inprocess`,
the `kali` binary, and in-crate unit-test binaries (`kali_cli`, `kali_embed`,
`kali_runtime`, `kali_types`, `kali_codegen`) — the ~2.6 GB floor the spec
declares out of scope. Record the `du -sh`; the spec predicts ~9 GB.

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): drive schema_validation through the kali binary

Was the last non-runtime test linking the CLI library, and so wasmtime, at
495 MB. It only needed artifacts and their JSON, which the subprocess produces
identically.

Phase 1 complete: target/ <BEFORE> -> <AFTER>. The only remaining >100 MB test
target is inprocess."
```

---

# Phase 2 — File-driven case runner

Phase 2 builds the runner before migrating anything, and each runner task is
TDD'd inside `kali_case_runner` where the logic is a plain library and testable
with ordinary unit tests. `crates/kali_cli/tests/cases.rs` stays a ~20-line
`main`.

---

### Task 8: Scaffold `kali_case_runner` and parse a case file

**Files:**
- Create: `crates/kali_case_runner/Cargo.toml`
- Create: `crates/kali_case_runner/src/lib.rs`
- Create: `crates/kali_case_runner/src/model.rs`
- Create: `crates/kali_case_runner/src/model_tests.rs`
- Modify: `Cargo.toml` (root)

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub struct CaseFile { pub constants: BTreeMap<String, String>, pub matrix: BTreeMap<String, Vec<String>>, pub source: BTreeMap<String, String>, pub case: Vec<Case> }`
  - `pub struct Case { pub name: String, pub rationale: Option<String>, pub ignore: bool, pub step: Vec<Step>, pub inline: Option<Step> }`
  - `pub struct Step { pub kind: StepKind, pub args: Vec<String>, pub env: BTreeMap<String, String>, pub exit: Option<Exit>, pub stdout: Option<String>, pub stdout_contains: Vec<String>, pub stdout_absent: Vec<String>, pub stderr_contains: Vec<String>, pub stderr_absent: Vec<String>, pub json: Option<toml::Value>, pub path: Option<String>, pub fields: Option<toml::Value>, pub entry: Option<String>, pub body: Option<String> }`
  - `pub enum StepKind { Cli, FileJson, BrowserBundleHarness }`
  - `pub enum Exit { Success, Failure, Code(i32) }`
  - `pub fn parse_case_file(text: &str) -> Result<CaseFile, String>`

- [ ] **Step 1: Write the failing tests**

`crates/kali_case_runner/src/model_tests.rs`:

```rust
use super::*;

#[test]
fn a_single_step_case_parses_with_inline_step_fields() {
    let text = r#"
[source]
"main.js" = "console.log(1);\n"

[[case]]
name = "run"
args = ["run", "main.js"]
exit = "success"
stdout = "1\n"
"#;
    let parsed = parse_case_file(text).expect("parse");
    assert_eq!(parsed.source["main.js"], "console.log(1);\n");
    assert_eq!(parsed.case.len(), 1);
    assert_eq!(parsed.case[0].name, "run");
    assert!(parsed.case[0].step.is_empty());
    let inline = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(inline.args, vec!["run", "main.js"]);
    assert_eq!(inline.exit, Some(Exit::Success));
    assert_eq!(inline.stdout.as_deref(), Some("1\n"));
}

#[test]
fn a_multi_step_case_parses_its_steps_in_order() {
    let text = r#"
[[case]]
name = "bundle_and_harness"

  [[case.step]]
  kind = "cli"
  args = ["build", "--bundle"]
  exit = "success"

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app"
  body = "await mod.f();"
  stdout_contains = ["1\n"]
"#;
    let parsed = parse_case_file(text).expect("parse");
    assert_eq!(parsed.case[0].step.len(), 2);
    assert_eq!(parsed.case[0].step[0].kind, StepKind::Cli);
    assert_eq!(parsed.case[0].step[1].kind, StepKind::BrowserBundleHarness);
    assert_eq!(parsed.case[0].step[1].entry.as_deref(), Some("app"));
}

#[test]
fn an_exact_exit_code_parses_as_a_code() {
    let text = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
exit = 2
"#;
    let parsed = parse_case_file(text).expect("parse");
    let inline = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(inline.exit, Some(Exit::Code(2)));
}

#[test]
fn dotted_json_keys_parse_into_a_nested_table() {
    let text = r#"
[[case]]
name = "c"
args = ["check", "main.ts"]
json.schemaVersion = 1
json.payload.artifactKind = "bundle"
"#;
    let parsed = parse_case_file(text).expect("parse");
    let json = parsed.case[0].inline.as_ref().unwrap().json.as_ref().expect("json");
    assert_eq!(json["schemaVersion"].as_integer(), Some(1));
    assert_eq!(json["payload"]["artifactKind"].as_str(), Some("bundle"));
}

// The format must not become a degradation vector: a typo'd key that silently
// asserts nothing is worse than no test at all (spec 5.10).
#[test]
fn an_unknown_key_is_a_hard_error_naming_the_key() {
    let text = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
stdout_contain = ["oops"]
"#;
    let err = parse_case_file(text).expect_err("must reject unknown key");
    assert!(err.contains("stdout_contain"), "error must name the key: {err}");
}

#[test]
fn a_case_file_with_no_cases_is_a_hard_error() {
    let text = r#"
[source]
"main.js" = "console.log(1);\n"
"#;
    let err = parse_case_file(text).expect_err("must reject zero cases");
    assert!(err.contains("no [[case]]"), "error must explain: {err}");
}

#[test]
fn a_matrix_axis_with_no_values_is_a_hard_error() {
    let text = r#"
[matrix]
ext = []

[[case]]
name = "c"
args = ["run", "main.js"]
"#;
    let err = parse_case_file(text).expect_err("must reject empty axis");
    assert!(err.contains("ext"), "error must name the axis: {err}");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cd /workspace && cargo test -p kali_case_runner
```

Expected: FAIL — the package does not exist yet.

- [ ] **Step 3: Create the manifest and register the crate**

`crates/kali_case_runner/Cargo.toml`:

```toml
[package]
name = "kali_case_runner"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
kali_runtime_contract = { workspace = true }
libtest-mimic = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
tempfile = "3.10"
toml = { workspace = true }
```

In the root `Cargo.toml`, add to `members` after `"crates/kali_test_support",`:

```toml
    "crates/kali_case_runner",
```

To `[workspace.dependencies]`, after the `kali_test_support` line:

```toml
kali_case_runner = { path = "crates/kali_case_runner" }
```

And to the external dependency block, after `getrandom`:

```toml
libtest-mimic = "0.8"
toml = "0.9"
```

- [ ] **Step 4: Write the model**

`crates/kali_case_runner/src/model.rs`:

```rust
//! Serde model for a `.toml` case file.
//!
//! Every struct is `deny_unknown_fields`. That is load-bearing: a typo'd
//! assertion key must fail the run, not silently assert nothing.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseFile {
    #[serde(default)]
    pub constants: BTreeMap<String, String>,
    #[serde(default)]
    pub matrix: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub source: BTreeMap<String, String>,
    #[serde(default)]
    pub case: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub name: String,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub ignore: bool,
    /// Multi-step form: `[[case.step]]`.
    #[serde(default)]
    pub step: Vec<Step>,
    /// Single-step shorthand: step fields written directly on `[[case]]`.
    #[serde(flatten)]
    pub inline: Option<Step>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Cli,
    FileJson,
    BrowserBundleHarness,
}

impl Default for StepKind {
    fn default() -> Self {
        Self::Cli
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(untagged)]
pub enum Exit {
    Status(ExitStatusWord),
    Code(i32),
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ExitStatusWord {
    Success,
    Failure,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(default)]
    pub kind: StepKind,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub exit: Option<Exit>,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stdout_contains: Vec<String>,
    #[serde(default)]
    pub stdout_absent: Vec<String>,
    #[serde(default)]
    pub stderr_contains: Vec<String>,
    #[serde(default)]
    pub stderr_absent: Vec<String>,
    #[serde(default)]
    pub json: Option<toml::Value>,
    /// `file_json` only.
    #[serde(default)]
    pub path: Option<String>,
    /// `file_json` only.
    #[serde(default)]
    pub fields: Option<toml::Value>,
    /// `browser_bundle_harness` only.
    #[serde(default)]
    pub entry: Option<String>,
    /// `browser_bundle_harness` only.
    #[serde(default)]
    pub body: Option<String>,
}

pub fn parse_case_file(text: &str) -> Result<CaseFile, String> {
    let parsed: CaseFile = toml::from_str(text).map_err(|error| error.to_string())?;
    if parsed.case.is_empty() {
        return Err("case file declares no [[case]] entries".to_string());
    }
    for (axis, values) in &parsed.matrix {
        if values.is_empty() {
            return Err(format!("matrix axis `{axis}` has no values"));
        }
    }
    Ok(parsed)
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod model_tests;
```

Note on `Exit`: `#[serde(untagged)]` makes `exit = "success"` and `exit = 2` both
parse. The test asserts `Exit::Code(2)` and `Exit::Success`; adjust the test's
expected variants to `Exit::Status(ExitStatusWord::Success)` and `Exit::Code(2)`,
or add `pub const SUCCESS: Exit = Exit::Status(ExitStatusWord::Success);` — pick
one and keep it consistent through Task 10.

`#[serde(flatten)]` on an `Option<Step>` combined with `deny_unknown_fields` on
`Case` conflicts: serde cannot deny unknown fields on a struct containing a
flattened field. Resolve by removing `deny_unknown_fields` from `Case` only, and
compensating with an explicit check in `parse_case_file` that each case has
either a non-empty `step` or a non-default `inline`, never both:

```rust
    for case in &parsed.case {
        let has_inline = case.inline.as_ref().is_some_and(|s| !s.args.is_empty()
            || s.path.is_some() || s.entry.is_some());
        if case.step.is_empty() && !has_inline {
            return Err(format!("case `{}` declares no step", case.name));
        }
        if !case.step.is_empty() && has_inline {
            return Err(format!(
                "case `{}` mixes [[case.step]] with inline step fields",
                case.name
            ));
        }
    }
```

Add a test for each of those two errors alongside the Step 1 tests.

- [ ] **Step 5: Write the crate root**

`crates/kali_case_runner/src/lib.rs`:

```rust
//! File-driven runner for `kali_cli`'s black-box CLI tests.
//!
//! One compiled target discovers `tests/cases/**/*.toml` at runtime, so adding
//! a test compiles nothing. See
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`.

mod model;
pub use model::{parse_case_file, Case, CaseFile, Exit, ExitStatusWord, Step, StepKind};
```

- [ ] **Step 6: Run the tests to verify they pass**

```bash
cd /workspace && cargo test -p kali_case_runner
```

Expected: PASS, all 9 tests.

- [ ] **Step 7: Commit**

```bash
cd /workspace
git add -A crates/kali_case_runner Cargo.toml Cargo.lock
git commit -m "feat(case-runner): parse TOML case files with deny_unknown_fields

Scaffolds kali_case_runner and the serde model. Unknown keys, zero-case files,
and empty matrix axes are hard errors -- a case format that silently asserts
nothing is the degradation vector this whole effort exists to close."
```

---

### Task 9: Matrix expansion and `${...}` substitution

**Files:**
- Create: `crates/kali_case_runner/src/expand.rs`
- Create: `crates/kali_case_runner/src/expand_tests.rs`
- Modify: `crates/kali_case_runner/src/lib.rs`

**Interfaces:**
- Consumes: `CaseFile`, `Case`, `Step` from Task 8.
- Produces:
  - `pub struct Trial { pub id: String, pub rationale: Option<String>, pub ignore: bool, pub source: BTreeMap<String, String>, pub steps: Vec<Step> }`
  - `pub fn expand(stem: &str, file: &CaseFile) -> Result<Vec<Trial>, String>` —
    `stem` is the family-relative path without extension, e.g.
    `browser/math_exp_log_mixed_root`. `Trial::id` is
    `<stem>[<axis>=<value>,...]::<case name>`, with the bracket segment omitted
    when there is no matrix. Axes are sorted by name so ids are deterministic.

- [ ] **Step 1: Write the failing tests**

`crates/kali_case_runner/src/expand_tests.rs`:

```rust
use super::*;
use crate::parse_case_file;

#[test]
fn a_case_file_with_no_matrix_yields_one_trial_per_case() {
    let file = parse_case_file(r#"
[source]
"main.js" = "console.log(1);\n"

[[case]]
name = "run"
args = ["run", "main.js"]

[[case]]
name = "check"
args = ["check", "main.js"]
"#).expect("parse");
    let trials = expand("string/x", &file).expect("expand");
    let ids: Vec<&str> = trials.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["string/x::run", "string/x::check"]);
}

#[test]
fn a_matrix_axis_substitutes_into_source_names_bodies_and_argv() {
    let file = parse_case_file(r#"
[matrix]
ext = ["js", "ts"]

[source]
"app.${ext}" = "// ${ext}\nconsole.log(1);\n"

[[case]]
name = "build"
args = ["build", "app.${ext}"]
"#).expect("parse");
    let trials = expand("browser/y", &file).expect("expand");
    assert_eq!(trials.len(), 2);
    assert_eq!(trials[0].id, "browser/y[ext=js]::build");
    assert!(trials[0].source.contains_key("app.js"));
    assert_eq!(trials[0].source["app.js"], "// js\nconsole.log(1);\n");
    assert_eq!(trials[0].steps[0].args, vec!["build", "app.js"]);
    assert_eq!(trials[1].id, "browser/y[ext=ts]::build");
    assert!(trials[1].source.contains_key("app.ts"));
}

#[test]
fn two_axes_form_a_cartesian_product_with_axes_sorted_by_name() {
    let file = parse_case_file(r#"
[matrix]
ext = ["js", "ts"]
api = ["browser", "node"]

[[case]]
name = "build"
args = ["build", "--api", "${api}", "app.${ext}"]
"#).expect("parse");
    let trials = expand("browser/z", &file).expect("expand");
    assert_eq!(trials.len(), 4);
    // `api` sorts before `ext`.
    assert_eq!(trials[0].id, "browser/z[api=browser,ext=js]::build");
    assert_eq!(trials[3].id, "browser/z[api=node,ext=ts]::build");
}

#[test]
fn constants_substitute_into_expected_strings() {
    let file = parse_case_file(r#"
[constants]
RULE_1 = "the discriminant is not a proven integer or string"

[[case]]
name = "float_discriminant"
args = ["run", "main.js"]
exit = "failure"
stderr_contains = ["E5506", "${RULE_1}"]
"#).expect("parse");
    let trials = expand("switch/fail_closed", &file).expect("expand");
    assert_eq!(
        trials[0].steps[0].stderr_contains,
        vec!["E5506", "the discriminant is not a proven integer or string"]
    );
}

// An unresolved placeholder must never survive into a comparison, or the test
// silently asserts a literal `${...}` nobody will ever emit.
#[test]
fn an_unresolved_placeholder_is_a_hard_error_naming_it() {
    let file = parse_case_file(r#"
[[case]]
name = "c"
args = ["run", "main.js"]
stderr_contains = ["${NOPE}"]
"#).expect("parse");
    let err = expand("x/y", &file).expect_err("must reject unresolved");
    assert!(err.contains("NOPE"), "error must name the placeholder: {err}");
}

#[test]
fn substitution_reaches_multi_step_cases_and_env_values() {
    let file = parse_case_file(r#"
[matrix]
ext = ["js"]

[constants]
CMD = "node"

[[case]]
name = "harness"

  [[case.step]]
  kind = "cli"
  args = ["run", "main.${ext}"]
  env = { KALI_BROWSER_BUNDLE_HARNESS_COMMAND = "${CMD}" }
"#).expect("parse");
    let trials = expand("browser/w", &file).expect("expand");
    assert_eq!(trials[0].steps[0].args, vec!["run", "main.js"]);
    assert_eq!(
        trials[0].steps[0].env["KALI_BROWSER_BUNDLE_HARNESS_COMMAND"],
        "node"
    );
}

#[test]
fn the_ignore_flag_and_rationale_carry_onto_every_expanded_trial() {
    let file = parse_case_file(r#"
[matrix]
ext = ["js", "ts"]

[[case]]
name = "c"
rationale = "why this exists"
ignore = true
args = ["run", "main.${ext}"]
"#).expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert!(trials.iter().all(|t| t.ignore));
    assert!(trials.iter().all(|t| t.rationale.as_deref() == Some("why this exists")));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

```bash
cd /workspace && cargo test -p kali_case_runner expand
```

Expected: FAIL with `cannot find function \`expand\``.

- [ ] **Step 3: Implement expansion**

`crates/kali_case_runner/src/expand.rs`:

```rust
//! Matrix expansion and `${...}` substitution.
//!
//! Substitution is closed at exactly two forms -- `${matrix_axis}` and
//! `${CONSTANT}` -- with no conditionals and no expressions. Variation that
//! changes assertions rather than substituting uniformly (text vs JSON output,
//! for instance) belongs in sibling `[[case]]` blocks, not here.

use crate::model::{Case, CaseFile, Step};
use std::collections::BTreeMap;

pub struct Trial {
    pub id: String,
    pub rationale: Option<String>,
    pub ignore: bool,
    pub source: BTreeMap<String, String>,
    pub steps: Vec<Step>,
}

/// Substitute every `${name}` from `bindings`; error on any survivor.
fn substitute(text: &str, bindings: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find('}')
            .ok_or_else(|| format!("unterminated `${{` in {text:?}"))?;
        let name = &after[..end];
        let value = bindings
            .get(name)
            .ok_or_else(|| format!("unresolved placeholder `${{{name}}}` in {text:?}"))?;
        out.push_str(value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn substitute_step(step: &Step, bindings: &BTreeMap<String, String>) -> Result<Step, String> {
    let list = |values: &Vec<String>| -> Result<Vec<String>, String> {
        values.iter().map(|v| substitute(v, bindings)).collect()
    };
    let opt = |value: &Option<String>| -> Result<Option<String>, String> {
        value.as_deref().map(|v| substitute(v, bindings)).transpose()
    };
    let mut env = BTreeMap::new();
    for (key, value) in &step.env {
        env.insert(substitute(key, bindings)?, substitute(value, bindings)?);
    }
    Ok(Step {
        kind: step.kind,
        args: list(&step.args)?,
        env,
        exit: step.exit,
        stdout: opt(&step.stdout)?,
        stdout_contains: list(&step.stdout_contains)?,
        stdout_absent: list(&step.stdout_absent)?,
        stderr_contains: list(&step.stderr_contains)?,
        stderr_absent: list(&step.stderr_absent)?,
        json: step.json.clone(),
        path: opt(&step.path)?,
        fields: step.fields.clone(),
        entry: opt(&step.entry)?,
        body: opt(&step.body)?,
    })
}

/// Every combination of the matrix axes, sorted by axis name so trial ids are
/// deterministic across runs.
fn matrix_cells(matrix: &BTreeMap<String, Vec<String>>) -> Vec<Vec<(String, String)>> {
    let mut cells: Vec<Vec<(String, String)>> = vec![Vec::new()];
    for (axis, values) in matrix {
        let mut next = Vec::with_capacity(cells.len() * values.len());
        for cell in &cells {
            for value in values {
                let mut extended = cell.clone();
                extended.push((axis.clone(), value.clone()));
                next.push(extended);
            }
        }
        cells = next;
    }
    cells
}

fn steps_of(case: &Case) -> Vec<&Step> {
    if case.step.is_empty() {
        case.inline.iter().collect()
    } else {
        case.step.iter().collect()
    }
}

pub fn expand(stem: &str, file: &CaseFile) -> Result<Vec<Trial>, String> {
    let mut trials = Vec::new();
    for cell in matrix_cells(&file.matrix) {
        let mut bindings = file.constants.clone();
        for (axis, value) in &cell {
            bindings.insert(axis.clone(), value.clone());
        }

        let suffix = if cell.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> =
                cell.iter().map(|(a, v)| format!("{a}={v}")).collect();
            format!("[{}]", pairs.join(","))
        };

        let mut source = BTreeMap::new();
        for (name, body) in &file.source {
            source.insert(substitute(name, &bindings)?, substitute(body, &bindings)?);
        }

        for case in &file.case {
            let steps = steps_of(case)
                .into_iter()
                .map(|step| substitute_step(step, &bindings))
                .collect::<Result<Vec<Step>, String>>()
                .map_err(|error| format!("{stem}{suffix}::{}: {error}", case.name))?;
            trials.push(Trial {
                id: format!("{stem}{suffix}::{}", case.name),
                rationale: case.rationale.clone(),
                ignore: case.ignore,
                source: source.clone(),
                steps,
            });
        }
    }
    Ok(trials)
}

#[cfg(test)]
#[path = "expand_tests.rs"]
mod expand_tests;
```

Add `#[derive(Clone, Copy)]` to `StepKind` in `model.rs` so `substitute_step` can
copy it, and `#[derive(Clone)]` is not needed on `Step` since a new one is built.

Export from `lib.rs`:

```rust
mod expand;
pub use expand::{expand, Trial};
```

- [ ] **Step 4: Run the tests to verify they pass**

```bash
cd /workspace && cargo test -p kali_case_runner
```

Expected: PASS, all tests from Tasks 8 and 9.

- [ ] **Step 5: Commit**

```bash
cd /workspace
git add -A crates/kali_case_runner
git commit -m "feat(case-runner): expand matrix axes and substitute placeholders

Substitution is closed at \${axis} and \${CONSTANT}. An unresolved placeholder is
a hard error -- letting one survive would assert a literal nobody emits. Axes
sort by name so trial ids are deterministic."
```

---

### Task 10: Assertion evaluation

**Files:**
- Create: `crates/kali_case_runner/src/jsonpath.rs`
- Create: `crates/kali_case_runner/src/jsonpath_tests.rs`
- Create: `crates/kali_case_runner/src/assertions.rs`
- Create: `crates/kali_case_runner/src/assertions_tests.rs`
- Modify: `crates/kali_case_runner/src/lib.rs`

**Interfaces:**
- Consumes: `Step`, `Exit`, `ExitStatusWord` from Task 8.
- Produces:
  - `pub fn flatten_expected(table: &toml::Value) -> Vec<(String, toml::Value)>` —
    a nested TOML table to dotted-path leaf pairs.
  - `pub fn lookup<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value>`
  - `pub fn values_equal(expected: &toml::Value, actual: &serde_json::Value) -> bool`
  - `pub struct Captured { pub code: Option<i32>, pub success: bool, pub stdout: String, pub stderr: String }`
  - `pub fn check(step: &Step, captured: &Captured) -> Result<(), String>`

- [ ] **Step 1: Write the failing tests for path handling**

`crates/kali_case_runner/src/jsonpath_tests.rs`:

```rust
use super::*;

fn toml_of(text: &str) -> toml::Value {
    text.parse::<toml::Value>().expect("toml")
}

#[test]
fn a_nested_table_flattens_to_dotted_leaf_paths() {
    let table = toml_of(r#"
schemaVersion = 1
[payload]
artifactKind = "bundle"
bundleFormat = "esm"
"#);
    let mut pairs = flatten_expected(&table);
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let paths: Vec<&str> = pairs.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        vec!["payload.artifactKind", "payload.bundleFormat", "schemaVersion"]
    );
}

// An empty array is a leaf, not a table to recurse into. `json.errors = []` is a
// real assertion used 245 times in the suite being migrated.
#[test]
fn an_empty_array_is_a_leaf() {
    let table = toml_of("errors = []");
    let pairs = flatten_expected(&table);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0, "errors");
    assert!(pairs[0].1.as_array().expect("array").is_empty());
}

#[test]
fn lookup_walks_a_dotted_path() {
    let actual: serde_json::Value =
        serde_json::json!({"payload": {"artifactKind": "bundle"}});
    assert_eq!(
        lookup(&actual, "payload.artifactKind").and_then(|v| v.as_str()),
        Some("bundle")
    );
    assert!(lookup(&actual, "payload.missing").is_none());
    assert!(lookup(&actual, "absent.deeper").is_none());
}

#[test]
fn values_equal_matches_across_toml_and_json_types() {
    assert!(values_equal(&toml::Value::Integer(1), &serde_json::json!(1)));
    assert!(values_equal(&toml::Value::Boolean(true), &serde_json::json!(true)));
    assert!(values_equal(
        &toml::Value::String("bundle".into()),
        &serde_json::json!("bundle")
    ));
    assert!(values_equal(
        &toml::Value::Array(vec![]),
        &serde_json::json!([])
    ));
    assert!(!values_equal(&toml::Value::Integer(1), &serde_json::json!("1")));
    assert!(!values_equal(&toml::Value::Integer(1), &serde_json::json!(2)));
}

// TOML has no null; JSON does. An expected empty string must not match null.
#[test]
fn a_json_null_matches_nothing_expressible_in_toml() {
    assert!(!values_equal(
        &toml::Value::String(String::new()),
        &serde_json::Value::Null
    ));
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cd /workspace && cargo test -p kali_case_runner jsonpath
```

Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement path handling**

`crates/kali_case_runner/src/jsonpath.rs`:

```rust
//! Dotted-path lookup into a JSON document, and equality between a TOML
//! expectation and a JSON actual.

/// Flatten a nested TOML table into `(dotted.path, leaf)` pairs. Arrays are
/// leaves, not tables -- `json.errors = []` asserts the whole array.
pub fn flatten_expected(table: &toml::Value) -> Vec<(String, toml::Value)> {
    fn walk(prefix: &str, value: &toml::Value, out: &mut Vec<(String, toml::Value)>) {
        match value {
            toml::Value::Table(map) => {
                for (key, child) in map {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(&path, child, out);
                }
            }
            leaf => out.push((prefix.to_string(), leaf.clone())),
        }
    }
    let mut out = Vec::new();
    walk("", table, &mut out);
    out
}

pub fn lookup<'a>(
    value: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

pub fn values_equal(expected: &toml::Value, actual: &serde_json::Value) -> bool {
    match (expected, actual) {
        (toml::Value::String(e), serde_json::Value::String(a)) => e == a,
        (toml::Value::Integer(e), serde_json::Value::Number(a)) => a.as_i64() == Some(*e),
        (toml::Value::Float(e), serde_json::Value::Number(a)) => a.as_f64() == Some(*e),
        (toml::Value::Boolean(e), serde_json::Value::Bool(a)) => e == a,
        (toml::Value::Array(e), serde_json::Value::Array(a)) => {
            e.len() == a.len()
                && e.iter().zip(a.iter()).all(|(e, a)| values_equal(e, a))
        }
        (toml::Value::Table(e), serde_json::Value::Object(a)) => {
            e.len() == a.len()
                && e.iter()
                    .all(|(k, e)| a.get(k).is_some_and(|a| values_equal(e, a)))
        }
        // TOML cannot express null, so nothing matches a JSON null.
        _ => false,
    }
}

#[cfg(test)]
#[path = "jsonpath_tests.rs"]
mod jsonpath_tests;
```

- [ ] **Step 4: Write the failing tests for assertion checking**

`crates/kali_case_runner/src/assertions_tests.rs`:

```rust
use super::*;
use crate::model::{Exit, ExitStatusWord, Step, StepKind};
use std::collections::BTreeMap;

fn blank_step() -> Step {
    Step {
        kind: StepKind::Cli,
        args: Vec::new(),
        env: BTreeMap::new(),
        exit: None,
        stdout: None,
        stdout_contains: Vec::new(),
        stdout_absent: Vec::new(),
        stderr_contains: Vec::new(),
        stderr_absent: Vec::new(),
        json: None,
        path: None,
        fields: None,
        entry: None,
        body: None,
    }
}

fn captured(success: bool, code: i32, stdout: &str, stderr: &str) -> Captured {
    Captured {
        code: Some(code),
        success,
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    }
}

#[test]
fn exit_success_passes_on_success_and_fails_on_failure() {
    let mut step = blank_step();
    step.exit = Some(Exit::Status(ExitStatusWord::Success));
    assert!(check(&step, &captured(true, 0, "", "")).is_ok());
    let err = check(&step, &captured(false, 1, "", "")).expect_err("must fail");
    assert!(err.contains("exit"), "{err}");
}

#[test]
fn an_exact_exit_code_must_match() {
    let mut step = blank_step();
    step.exit = Some(Exit::Code(2));
    assert!(check(&step, &captured(false, 2, "", "")).is_ok());
    assert!(check(&step, &captured(false, 1, "", "")).is_err());
}

#[test]
fn exact_stdout_must_match_byte_for_byte() {
    let mut step = blank_step();
    step.stdout = Some("hahaha\n\n".to_string());
    assert!(check(&step, &captured(true, 0, "hahaha\n\n", "")).is_ok());
    assert!(check(&step, &captured(true, 0, "hahaha\n", "")).is_err());
}

#[test]
fn contains_and_absent_are_both_enforced() {
    let mut step = blank_step();
    step.stdout_contains = vec!["1\n".to_string()];
    step.stdout_absent = vec!["E5506".to_string()];
    assert!(check(&step, &captured(true, 0, "1\n0\n", "")).is_ok());
    assert!(check(&step, &captured(true, 0, "0\n", "")).is_err());
    assert!(check(&step, &captured(true, 0, "1\nE5506", "")).is_err());
}

#[test]
fn stderr_claims_are_checked_against_stderr() {
    let mut step = blank_step();
    step.stderr_contains = vec!["E5506".to_string()];
    step.stderr_absent = vec!["is used as both a string and a number".to_string()];
    assert!(check(&step, &captured(false, 1, "", "E5506 denied")).is_ok());
    let err = check(
        &step,
        &captured(false, 1, "", "E5506 is used as both a string and a number"),
    )
    .expect_err("must fail on a present absence claim");
    assert!(err.contains("is used as both"), "{err}");
}

#[test]
fn json_fields_are_checked_by_dotted_path() {
    let mut step = blank_step();
    step.json = Some(
        r#"
schemaVersion = 1
success = true
[payload]
artifactKind = "bundle"
"#
        .parse()
        .expect("toml"),
    );
    let good = r#"{"schemaVersion":1,"success":true,"payload":{"artifactKind":"bundle"}}"#;
    assert!(check(&step, &captured(true, 0, good, "")).is_ok());
    let bad = r#"{"schemaVersion":2,"success":true,"payload":{"artifactKind":"bundle"}}"#;
    let err = check(&step, &captured(true, 0, bad, "")).expect_err("must fail");
    assert!(err.contains("schemaVersion"), "{err}");
}

#[test]
fn a_missing_json_path_fails_and_names_the_path() {
    let mut step = blank_step();
    step.json = Some(r#"payload.bundleFormat = "esm""#.parse().expect("toml"));
    let err = check(&step, &captured(true, 0, r#"{"payload":{}}"#, ""))
        .expect_err("must fail");
    assert!(err.contains("payload.bundleFormat"), "{err}");
}

#[test]
fn unparseable_stdout_under_a_json_claim_fails_rather_than_passing_vacuously() {
    let mut step = blank_step();
    step.json = Some("schemaVersion = 1".parse().expect("toml"));
    let err = check(&step, &captured(true, 0, "not json", "")).expect_err("must fail");
    assert!(err.to_lowercase().contains("json"), "{err}");
}

// Failure text must not emit lines matching `^    [A-Za-z_]`, which
// scripts/test-gate.sh parses as failed-test names.
#[test]
fn failure_text_never_uses_the_four_space_name_indent() {
    let mut step = blank_step();
    step.stdout = Some("expected".to_string());
    let err = check(&step, &captured(true, 0, "actual output here", "and stderr"))
        .expect_err("must fail");
    for line in err.lines() {
        assert!(
            !(line.starts_with("    ") && line.chars().nth(4).is_some_and(|c| c.is_alphabetic() || c == '_')),
            "line would be misparsed by test-gate.sh: {line:?}"
        );
    }
}
```

- [ ] **Step 5: Run to verify they fail**

```bash
cd /workspace && cargo test -p kali_case_runner assertions
```

Expected: FAIL — `check` and `Captured` do not exist.

- [ ] **Step 6: Implement assertion checking**

`crates/kali_case_runner/src/assertions.rs`:

```rust
//! Evaluate a step's eight assertion keys against captured process output.
//!
//! Failure messages are indented with two spaces, never four. `scripts/test-gate.sh`
//! parses `^    [A-Za-z_]` as a failed-test name, and a four-space-indented
//! detail line would be misread as a test that does not exist.

use crate::jsonpath::{flatten_expected, lookup, values_equal};
use crate::model::{Exit, ExitStatusWord, Step};

pub struct Captured {
    pub code: Option<i32>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Captured {
    fn context(&self) -> String {
        format!(
            "  --- stdout ---\n{}\n  --- stderr ---\n{}",
            indent(&self.stdout),
            indent(&self.stderr)
        )
    }
}

fn indent(text: &str) -> String {
    if text.is_empty() {
        return "  (empty)".to_string();
    }
    text.lines()
        .map(|line| format!("  | {line}"))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn check(step: &Step, captured: &Captured) -> Result<(), String> {
    let fail = |claim: String| -> String { format!("{claim}\n{}", captured.context()) };

    match step.exit {
        Some(Exit::Status(ExitStatusWord::Success)) if !captured.success => {
            return Err(fail(format!(
                "expected exit success, got code {:?}",
                captured.code
            )));
        }
        Some(Exit::Status(ExitStatusWord::Failure)) if captured.success => {
            return Err(fail("expected exit failure, but it succeeded".to_string()));
        }
        Some(Exit::Code(expected)) if captured.code != Some(expected) => {
            return Err(fail(format!(
                "expected exit code {expected}, got {:?}",
                captured.code
            )));
        }
        _ => {}
    }

    if let Some(expected) = &step.stdout {
        if &captured.stdout != expected {
            return Err(fail(format!(
                "stdout mismatch\n  expected: {expected:?}\n  actual:   {:?}",
                captured.stdout
            )));
        }
    }

    for needle in &step.stdout_contains {
        if !captured.stdout.contains(needle.as_str()) {
            return Err(fail(format!("stdout missing {needle:?}")));
        }
    }
    for needle in &step.stdout_absent {
        if captured.stdout.contains(needle.as_str()) {
            return Err(fail(format!("stdout must not contain {needle:?}")));
        }
    }
    for needle in &step.stderr_contains {
        if !captured.stderr.contains(needle.as_str()) {
            return Err(fail(format!("stderr missing {needle:?}")));
        }
    }
    for needle in &step.stderr_absent {
        if captured.stderr.contains(needle.as_str()) {
            return Err(fail(format!("stderr must not contain {needle:?}")));
        }
    }

    if let Some(expected) = &step.json {
        let actual: serde_json::Value = serde_json::from_str(&captured.stdout)
            .map_err(|error| fail(format!("stdout is not valid json: {error}")))?;
        check_json(expected, &actual).map_err(fail)?;
    }

    Ok(())
}

/// Shared by the `json` key and by `file_json`'s `fields` key.
pub fn check_json(
    expected: &toml::Value,
    actual: &serde_json::Value,
) -> Result<(), String> {
    for (path, leaf) in flatten_expected(expected) {
        match lookup(actual, &path) {
            None => return Err(format!("json path {path} is absent")),
            Some(found) if !values_equal(&leaf, found) => {
                return Err(format!(
                    "json path {path} mismatch\n  expected: {leaf}\n  actual:   {found}"
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "assertions_tests.rs"]
mod assertions_tests;
```

Export from `lib.rs`:

```rust
mod assertions;
mod jsonpath;
pub use assertions::{check, check_json, Captured};
pub use jsonpath::{flatten_expected, lookup, values_equal};
```

- [ ] **Step 7: Run to verify all tests pass**

```bash
cd /workspace && cargo test -p kali_case_runner
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cd /workspace
git add -A crates/kali_case_runner
git commit -m "feat(case-runner): evaluate the eight assertion keys

Covers the measured vocabulary: exit class/code, exact stdout, contains and
absent on both streams, and dotted-path JSON equality across toml and json
types. Unparseable stdout under a json claim fails rather than passing
vacuously, and failure text avoids the four-space indent that
scripts/test-gate.sh parses as a test name."
```

---

### Task 11: Step execution and the three step kinds

**Files:**
- Create: `crates/kali_case_runner/src/steps.rs`
- Create: `crates/kali_case_runner/src/steps_tests.rs`
- Modify: `crates/kali_case_runner/src/lib.rs`

**Interfaces:**
- Consumes: `Trial` (Task 9), `Captured`/`check`/`check_json` (Task 10),
  `browser_bundle_harness_script` and `browser_harness_command_parts_for` from
  `kali_runtime_contract` (Task 4).
- Produces:
  - `pub struct RunnerConfig { pub kali_bin: PathBuf, pub cases_dir: PathBuf }`
  - `pub fn run_trial(config: &RunnerConfig, trial: &Trial) -> Result<(), String>` —
    creates a temp dir, writes `trial.source`, runs each step in order, returns
    the first failure with the step index and rationale prepended.

- [ ] **Step 1: Write the failing tests**

These tests use a stub "kali" — a shell script — so they exercise step
orchestration without building the real compiler.

`crates/kali_case_runner/src/steps_tests.rs`:

```rust
use super::*;
use crate::{expand, parse_case_file};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

/// Write an executable stub that stands in for the `kali` binary.
fn stub_bin(dir: &std::path::Path, script: &str) -> std::path::PathBuf {
    let path = dir.join("stub-kali");
    let mut file = std::fs::File::create(&path).expect("create stub");
    writeln!(file, "#!/usr/bin/env bash").expect("write");
    write!(file, "{script}").expect("write");
    drop(file);
    let mut perms = std::fs::metadata(&path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod");
    path
}

fn config_for(bin: std::path::PathBuf) -> RunnerConfig {
    RunnerConfig { kali_bin: bin, cases_dir: std::path::PathBuf::from(".") }
}

#[test]
fn a_cli_step_writes_the_source_and_asserts_on_output() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "cat main.js\n");
    let file = parse_case_file(r#"
[source]
"main.js" = "hello\n"

[[case]]
name = "run"
args = ["run", "main.js"]
exit = "success"
stdout = "hello\n"
"#).expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    run_trial(&config_for(bin), &trials[0]).expect("trial should pass");
}

#[test]
fn a_failing_step_reports_the_step_index_and_the_rationale() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "echo wrong\n");
    let file = parse_case_file(r#"
[[case]]
name = "run"
rationale = "pins the folded literal"
args = ["run", "main.js"]
stdout = "right\n"
"#).expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(err.contains("step 1"), "must name the step: {err}");
    assert!(err.contains("pins the folded literal"), "must print rationale: {err}");
}

#[test]
fn later_steps_see_artifacts_written_by_earlier_steps() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(
        home.path(),
        "if [ \"$1\" = build ]; then mkdir -p app; echo '{\"apiSurface\":\"browser\"}' > app/app.meta.json; else cat app/app.meta.json; fi\n",
    );
    let file = parse_case_file(r#"
[[case]]
name = "build_then_read"

  [[case.step]]
  kind = "cli"
  args = ["build"]
  exit = "success"

  [[case.step]]
  kind = "file_json"
  path = "app/app.meta.json"
  fields = { apiSurface = "browser" }
"#).expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    run_trial(&config_for(bin), &trials[0]).expect("trial should pass");
}

#[test]
fn a_file_json_step_fails_when_the_file_is_absent() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "file_json"
  path = "app/app.meta.json"
  fields = { apiSurface = "browser" }
"#).expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(err.contains("app/app.meta.json"), "{err}");
}

#[test]
fn env_declared_on_a_step_reaches_the_child_process() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "echo \"$KALI_TEST_MARKER\"\n");
    let file = parse_case_file(r#"
[[case]]
name = "c"
args = ["run"]
env = { KALI_TEST_MARKER = "seen" }
stdout = "seen\n"
"#).expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    run_trial(&config_for(bin), &trials[0]).expect("trial should pass");
}

#[test]
fn a_browser_bundle_harness_step_requires_entry_and_body() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app"
"#).expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(err.contains("body"), "must name the missing key: {err}");
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cd /workspace && cargo test -p kali_case_runner steps
```

Expected: FAIL — `run_trial` and `RunnerConfig` do not exist.

- [ ] **Step 3: Implement step execution**

`crates/kali_case_runner/src/steps.rs`:

```rust
//! Trial execution: one temp dir per trial, steps run in order, first failure
//! wins.

use crate::assertions::{check, check_json, Captured};
use crate::expand::Trial;
use crate::model::{Step, StepKind};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RunnerConfig {
    pub kali_bin: PathBuf,
    pub cases_dir: PathBuf,
}

fn capture(mut command: Command, step: &Step) -> Result<Captured, String> {
    for (key, value) in &step.env {
        command.env(key, value);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to spawn: {error}"))?;
    Ok(Captured {
        code: output.status.code(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn run_cli(config: &RunnerConfig, dir: &Path, step: &Step) -> Result<(), String> {
    let mut command = Command::new(&config.kali_bin);
    command.current_dir(dir).args(&step.args);
    let captured = capture(command, step)?;
    check(step, &captured)
}

fn run_file_json(dir: &Path, step: &Step) -> Result<(), String> {
    let rel = step
        .path
        .as_deref()
        .ok_or_else(|| "file_json step requires `path`".to_string())?;
    let fields = step
        .fields
        .as_ref()
        .ok_or_else(|| "file_json step requires `fields`".to_string())?;
    let text = std::fs::read_to_string(dir.join(rel))
        .map_err(|error| format!("cannot read {rel}: {error}"))?;
    let actual: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("{rel} is not valid json: {error}"))?;
    check_json(fields, &actual)
}

fn run_browser_bundle_harness(
    dir: &Path,
    step: &Step,
) -> Result<(), String> {
    let entry = step
        .entry
        .as_deref()
        .ok_or_else(|| "browser_bundle_harness step requires `entry`".to_string())?;
    let body = step
        .body
        .as_deref()
        .ok_or_else(|| "browser_bundle_harness step requires `body`".to_string())?;

    let script = kali_runtime_contract::browser_bundle_harness_script(entry, false, body);
    let harness_path = dir.join("browser-bundle-smoke.mjs");
    std::fs::write(&harness_path, script)
        .map_err(|error| format!("cannot write harness: {error}"))?;

    let mut parts = kali_runtime_contract::browser_harness_command_parts_for(
        step.env
            .get(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV)
            .map(String::as_str)
            .or_else(|| None),
    );
    let executable = parts.remove(0);
    let mut command = Command::new(executable);
    command.current_dir(dir).args(&parts).arg(&harness_path);
    let captured = capture(command, step)?;
    check(step, &captured)
}

pub fn run_trial(config: &RunnerConfig, trial: &Trial) -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|error| format!("tempdir: {error}"))?;

    for (name, body) in &trial.source {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        std::fs::write(&path, body)
            .map_err(|error| format!("cannot write {name}: {error}"))?;
    }

    for (index, step) in trial.steps.iter().enumerate() {
        let result = match step.kind {
            StepKind::Cli => run_cli(config, dir.path(), step),
            StepKind::FileJson => run_file_json(dir.path(), step),
            StepKind::BrowserBundleHarness => {
                run_browser_bundle_harness(dir.path(), step)
            }
        };
        if let Err(detail) = result {
            let mut message = format!("step {} ({:?}) failed\n", index + 1, step.kind);
            if let Some(rationale) = &trial.rationale {
                message.push_str("  rationale:\n");
                for line in rationale.lines() {
                    message.push_str(&format!("  | {line}\n"));
                }
            }
            if !step.args.is_empty() {
                message.push_str(&format!("  argv: {:?}\n", step.args));
            }
            if !step.env.is_empty() {
                message.push_str(&format!("  env: {:?}\n", step.env));
            }
            message.push_str(&detail);
            return Err(message);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "steps_tests.rs"]
mod steps_tests;
```

Add `#[derive(Debug)]` to `StepKind` if Task 8 did not (the `{:?}` above needs
it), and export from `lib.rs`:

```rust
mod steps;
pub use steps::{run_trial, RunnerConfig};
```

- [ ] **Step 4: Run to verify all tests pass**

```bash
cd /workspace && cargo test -p kali_case_runner
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd /workspace
git add -A crates/kali_case_runner
git commit -m "feat(case-runner): execute trials with the three step kinds

One temp dir per trial, steps in order, first failure wins. cli shells out to
the kali binary; file_json reads a produced artifact; browser_bundle_harness
generates the .mjs via kali_runtime_contract and runs it under the resolved
harness command. Failures name the step index and print the case rationale.

Tested against an executable stub rather than the real compiler, so the suite
stays fast and hermetic."
```

---

### Task 12: Discovery, the libtest-mimic harness, and the `cases` target

This task makes the runner real: `cargo test -p kali_cli --test cases` runs.

**Files:**
- Create: `crates/kali_case_runner/src/discover.rs`
- Create: `crates/kali_case_runner/src/discover_tests.rs`
- Modify: `crates/kali_case_runner/src/lib.rs`
- Create: `crates/kali_cli/tests/cases.rs`
- Create: `crates/kali_cli/tests/cases/string/repeat_static_ascii.toml`
- Modify: `crates/kali_cli/Cargo.toml`

**Interfaces:**
- Consumes: `RunnerConfig`, `run_trial`, `expand`, `parse_case_file`.
- Produces:
  - `pub fn discover(cases_dir: &Path) -> Result<Vec<(String, CaseFile)>, String>` —
    sorted by stem; errors if the directory is missing or contains no `.toml`.
  - `pub fn main_with(config: RunnerConfig) -> std::process::ExitCode` — builds
    trials, hands them to `libtest_mimic::run`.

- [ ] **Step 1: Write the failing discovery tests**

`crates/kali_case_runner/src/discover_tests.rs`:

```rust
use super::*;

fn write(dir: &std::path::Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, text).expect("write");
}

const MINIMAL: &str = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
"#;

#[test]
fn discovery_returns_family_relative_stems_sorted() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "array/at.toml", MINIMAL);
    write(root.path(), "string/repeat.toml", MINIMAL);
    let found = discover(root.path()).expect("discover");
    let stems: Vec<&str> = found.iter().map(|(stem, _)| stem.as_str()).collect();
    assert_eq!(stems, vec!["array/at", "string/pad", "string/repeat"]);
}

#[test]
fn non_toml_files_are_ignored() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/README.md", "notes");
    let found = discover(root.path()).expect("discover");
    assert_eq!(found.len(), 1);
}

// A wrong discovery path must not report "0 tests, ok" and turn CI green.
#[test]
fn an_empty_case_tree_is_a_hard_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let err = discover(root.path()).expect_err("must reject empty tree");
    assert!(err.contains("no case files"), "{err}");
}

#[test]
fn a_missing_case_directory_is_a_hard_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let err = discover(&root.path().join("absent")).expect_err("must reject missing dir");
    assert!(err.contains("absent"), "{err}");
}

#[test]
fn a_malformed_case_file_errors_with_its_path() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/broken.toml", "[[case]]\nname = ");
    let err = discover(root.path()).expect_err("must reject malformed toml");
    assert!(err.contains("string/broken.toml"), "{err}");
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cd /workspace && cargo test -p kali_case_runner discover
```

Expected: FAIL — `discover` does not exist.

- [ ] **Step 3: Implement discovery and the harness entry point**

`crates/kali_case_runner/src/discover.rs`:

```rust
//! Case-tree discovery and the libtest-mimic entry point.

use crate::expand::expand;
use crate::model::{parse_case_file, CaseFile};
use crate::steps::{run_trial, RunnerConfig};
use libtest_mimic::{Arguments, Failed, Trial as MimicTrial};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

fn collect(dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            let nested = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            collect(&path, &nested, out)?;
        } else if path.extension().is_some_and(|ext| ext == "toml") {
            let stem = name.trim_end_matches(".toml").to_string();
            let full = if prefix.is_empty() {
                stem
            } else {
                format!("{prefix}/{stem}")
            };
            out.push((full, path));
        }
    }
    Ok(())
}

pub fn discover(cases_dir: &Path) -> Result<Vec<(String, CaseFile)>, String> {
    if !cases_dir.is_dir() {
        return Err(format!(
            "case directory {} does not exist",
            cases_dir.display()
        ));
    }
    let mut paths = Vec::new();
    collect(cases_dir, "", &mut paths)?;
    if paths.is_empty() {
        return Err(format!(
            "no case files found under {} -- refusing to report a green run over zero tests",
            cases_dir.display()
        ));
    }
    paths.sort_by(|a, b| a.0.cmp(&b.0));

    let mut files = Vec::with_capacity(paths.len());
    for (stem, path) in paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let parsed = parse_case_file(&text)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        files.push((stem, parsed));
    }
    Ok(files)
}

pub fn main_with(config: RunnerConfig) -> ExitCode {
    let args = Arguments::from_args();

    let files = match discover(&config.cases_dir) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("case discovery failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let config = Arc::new(config);
    let mut trials = Vec::new();
    for (stem, file) in &files {
        let expanded = match expand(stem, file) {
            Ok(expanded) => expanded,
            Err(error) => {
                eprintln!("case expansion failed: {error}");
                return ExitCode::FAILURE;
            }
        };
        for trial in expanded {
            let config = Arc::clone(&config);
            let ignore = trial.ignore;
            let id = trial.id.clone();
            trials.push(
                MimicTrial::test(id, move || {
                    run_trial(&config, &trial).map_err(Failed::from)
                })
                .with_ignored_flag(ignore),
            );
        }
    }

    libtest_mimic::run(&args, trials).exit_code()
}

#[cfg(test)]
#[path = "discover_tests.rs"]
mod discover_tests;
```

`Trial` must be `Send + 'static` for `MimicTrial::test`. Add `#[derive(Debug)]`
is not required, but if the compiler objects to moving `trial` into the closure,
the fix is to ensure `Trial` and `Step` hold only owned data — they do.

Export from `lib.rs`:

```rust
mod discover;
pub use discover::{discover, main_with};
```

- [ ] **Step 4: Write the thin test target and one real case file**

`crates/kali_cli/tests/cases.rs`:

```rust
//! File-driven CLI test target.
//!
//! Every case lives in `tests/cases/**/*.toml`; adding one compiles nothing.
//! Filter with the path: `cargo test -p kali_cli --test cases -- switch/`.
//!
//! Do not add Rust test logic here. Cases that the format cannot express stay
//! as their own hand-written target -- see
//! `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md` 5.11.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    kali_case_runner::main_with(kali_case_runner::RunnerConfig {
        kali_bin: PathBuf::from(env!("CARGO_BIN_EXE_kali")),
        cases_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases"),
    })
}
```

`crates/kali_cli/tests/cases/string/repeat_static_ascii.toml` — the first real
case, translated from `tests/string_repeat_static_ascii.rs`:

```toml
# Migrated from tests/string_repeat_static_ascii.rs.
[source]
"main.js" = '''
console.log("ha".repeat(3));
console.log("x".repeat(0));
'''
"main.ts" = '''
console.log("ha".repeat(3));
console.log("x".repeat(0));
'''

[[case]]
name = "run"
rationale = "`.repeat()` on an ASCII literal folds at compile time."
args = ["run", "main.js"]
exit = "success"
stdout = "hahaha\n\n"

[[case]]
name = "json_check"
args = ["--output", "json", "check", "main.ts"]
exit = "success"
json.schemaVersion = 1
json.command = "check"
json.success = true
json.errors = []
```

- [ ] **Step 5: Declare the target and the dev-dependency**

In `crates/kali_cli/Cargo.toml`, add to `[dev-dependencies]`:

```toml
kali_case_runner = { workspace = true }
```

And after the `[[test]] inprocess` block added in Task 6:

```toml
[[test]]
name = "cases"
path = "tests/cases.rs"
harness = false
```

- [ ] **Step 6: Run the new target**

```bash
cd /workspace
cargo test -p kali_cli --test cases 2>&1 | tail -20
```

Expected: `running 2 tests`, both PASS, named
`string/repeat_static_ascii::run` and `string/repeat_static_ascii::json_check`.

- [ ] **Step 7: Verify filtering, `--ignored`, and the empty-tree guard**

```bash
cd /workspace
cargo test -p kali_cli --test cases -- string/ 2>&1 | grep -E 'running|test result'
cargo test -p kali_cli --test cases -- --ignored 2>&1 | grep -E 'running|test result'
# The empty-tree guard must fail loudly, not report a green zero-test run.
mv crates/kali_cli/tests/cases crates/kali_cli/tests/cases-off
cargo test -p kali_cli --test cases 2>&1 | tail -3
echo "exit was: $?"
mv crates/kali_cli/tests/cases-off crates/kali_cli/tests/cases
```

Expected: the filter runs 2 tests; `--ignored` runs 0; the moved-away tree
produces `case directory ... does not exist` and a non-zero exit.

- [ ] **Step 8: Verify the runner's failure output does not confuse the gate**

Deliberately break the case, run the gate parser over the output, and confirm it
reports exactly one failure name.

```bash
cd /workspace
sed -i 's/stdout = "hahaha\\n\\n"/stdout = "WRONG\\n"/' crates/kali_cli/tests/cases/string/repeat_static_ascii.toml
cargo test -p kali_cli --test cases 2>&1 | awk '
    /^failures:$/ { collecting = 1; next }
    collecting && /^    [A-Za-z_]/ { print $1; next }
    collecting { collecting = 0 }' | sort -u
git checkout crates/kali_cli/tests/cases/string/repeat_static_ascii.toml
```

Expected: exactly one line, `string/repeat_static_ascii::run`. More than one line
means a failure-detail line is being misparsed as a test name — fix the
indentation in `assertions.rs`/`steps.rs` before proceeding.

- [ ] **Step 9: Commit**

```bash
cd /workspace
git add -A crates/kali_case_runner crates/kali_cli Cargo.lock
git commit -m "feat(case-runner): discover the case tree and run it under libtest-mimic

Adds the 'cases' target: one harness = false binary that walks
tests/cases/**/*.toml at runtime, so adding a case compiles nothing. Filtering,
--ignored, and parallel execution all work. An empty or missing case tree is a
hard error -- reporting '0 tests, ok' would turn CI green over nothing.

Verified scripts/test-gate.sh's failure parser reads exactly one name from a
deliberately failing case."
```

---

### Task 13: The migration audit script

The gate that makes the remaining migration safe. Built before any bulk
migration, and used by every migration task that follows.

**Files:**
- Create: `scripts/audit-case-migration.py`

**Interfaces:**
- Consumes: nothing in Rust.
- Produces: `python3 scripts/audit-case-migration.py <old.rs> <new.toml>...`
  exits 0 if every literal claim in the old file appears in the new case files,
  1 otherwise with a report of what is missing.

- [ ] **Step 1: Write the script**

`scripts/audit-case-migration.py`:

```python
#!/usr/bin/env python3
"""Fail if a migrated case file drops a claim its .rs predecessor made.

Migrating ~200k lines of assertions is where meaning gets silently dropped, and
this repository has already had two fail-closed tests degrade to asserting
nothing. So the migration gate is mechanical, not eyeballed: every string
literal the old test compared against, every JSON path it asserted on, and every
argv token it passed must still appear somewhere in the new case files.

This is a coverage check, not a proof of equivalence. It catches wholesale drops
and quiet weakenings (a rule constant vanishing while `contains("E5506")`
survives). It cannot catch a claim that was rewritten to be weaker while keeping
its literals. Read the diff too.

Usage: audit-case-migration.py OLD.rs NEW.toml [NEW.toml ...]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

# Literal arguments to .contains(...) — the dominant assertion form.
CONTAINS = re.compile(r'\.contains\(\s*(?:&)?(r?#*"(?:[^"\\]|\\.)*"#*)')
# assert_eq!(json["a"]["b"], value) — capture each bracketed key.
JSON_KEY = re.compile(r'\[\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\]')
# .arg("token")
ARG = re.compile(r'\.arg\(\s*"([^"]*)"\s*\)')
# const NAME: &str = "literal";
CONST = re.compile(r'const\s+[A-Z0-9_]+\s*:\s*&str\s*=\s*\n?\s*(r?#*"(?:[^"\\]|\\.)*"#*)')
TEST_FN = re.compile(r'#\[test\][^\n]*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*(?:async\s+)?fn\s+([a-z0-9_]+)')


def unquote(raw: str) -> str:
    """Turn a Rust string literal token into its text, best-effort."""
    raw = raw.strip()
    if raw.startswith("r"):
        raw = raw[1:]
        hashes = len(raw) - len(raw.lstrip("#"))
        return raw[hashes + 1 : len(raw) - hashes - 1]
    body = raw[1:-1]
    return (
        body.replace('\\"', '"')
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
    )


def claims(source: str) -> dict[str, set[str]]:
    return {
        "contains literals": {unquote(m) for m in CONTAINS.findall(source)},
        "rule constants": {unquote(m) for m in CONST.findall(source)},
        "json keys": set(JSON_KEY.findall(source)),
        "argv tokens": set(ARG.findall(source)),
    }


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2

    old_path = Path(sys.argv[1])
    new_paths = [Path(p) for p in sys.argv[2:]]

    old_source = old_path.read_text()
    new_text = "\n".join(p.read_text() for p in new_paths)

    # Trivially-common argv tokens carry no signal.
    boring = {"run", "check", "build", "test", "json", "--output"}

    missing: list[tuple[str, str]] = []
    for kind, values in claims(old_source).items():
        for value in sorted(values):
            if not value or value in boring:
                continue
            if value not in new_text:
                missing.append((kind, value))

    old_tests = sorted(set(TEST_FN.findall(old_source)))
    print(f"{old_path}: {len(old_tests)} #[test] fns")
    for kind, values in claims(old_source).items():
        print(f"  {kind}: {len(values)}")

    if missing:
        print(f"\nAUDIT FAILED — {len(missing)} claim(s) absent from the case files:")
        for kind, value in missing:
            print(f"  [{kind}] {value!r}")
        return 1

    print("\nAUDIT OK — every literal claim is present in the case files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

- [ ] **Step 2: Verify it passes on the already-migrated case**

Task 12 migrated `string_repeat_static_ascii`. The old file is still present, so
this is a real end-to-end check of the script.

```bash
cd /workspace
python3 scripts/audit-case-migration.py \
  crates/kali_cli/tests/string_repeat_static_ascii.rs \
  crates/kali_cli/tests/cases/string/repeat_static_ascii.toml
```

Expected: `AUDIT OK`. If it reports a genuine missing claim, the Task 12 case
file is incomplete — fix the `.toml`, not the script.

- [ ] **Step 3: Verify it actually fails when a claim is dropped**

A gate that cannot fail is not a gate.

```bash
cd /workspace
cp crates/kali_cli/tests/cases/string/repeat_static_ascii.toml /tmp/weakened.toml
sed -i '/json.command/d' /tmp/weakened.toml
python3 scripts/audit-case-migration.py \
  crates/kali_cli/tests/string_repeat_static_ascii.rs /tmp/weakened.toml
echo "exit: $?"
rm /tmp/weakened.toml
```

Expected: `AUDIT FAILED` naming the `command` json key, exit 1.

- [ ] **Step 4: Commit**

```bash
cd /workspace
chmod +x scripts/audit-case-migration.py
git add scripts/audit-case-migration.py
git commit -m "test(cli): add the case-migration literal-coverage audit

No .rs file gets deleted until this passes for its family. Extracts every
contains() literal, rule constant, JSON key, and argv token from the old test and
requires each to appear in the new case files. Verified it both passes on a
correct migration and fails on a deliberately weakened one."
```

---

### Task 14: Migrate the `string/` family

The first bulk migration, and the proof that the format handles a real family.
19 targets.

**Files:**
- Create: `crates/kali_cli/tests/cases/string/*.toml` (18 new, 1 exists)
- Delete: `crates/kali_cli/tests/string_*.rs` (19 files)

**Interfaces:**
- Consumes: the runner and audit script from Tasks 8–13.
- Produces: no API. `crates/kali_cli/tests/string_*.rs` no longer exists.

- [ ] **Step 1: List the family and record the baseline**

```bash
cd /workspace
ls crates/kali_cli/tests/string_*.rs | tee /tmp/string-family.txt | wc -l
cargo test -p kali_cli 2>&1 | grep -E '^test result:' | \
  awk -F'[ ;]' '{p+=$4} END {print "kali_cli tests passing:", p}'
```

Expected: 19 files. Record the passing count.

- [ ] **Step 2: Migrate one file and audit it, before doing the rest**

Pick `crates/kali_cli/tests/string_pad_static_ascii.rs`. Read it in full, then
write `crates/kali_cli/tests/cases/string/pad_static_ascii.toml` following the
Task 12 case file as the template. Run:

```bash
cd /workspace
cargo test -p kali_cli --test cases -- string/pad_static_ascii
python3 scripts/audit-case-migration.py \
  crates/kali_cli/tests/string_pad_static_ascii.rs \
  crates/kali_cli/tests/cases/string/pad_static_ascii.toml
```

Expected: tests PASS and `AUDIT OK`. Only proceed to Step 3 once both hold — if
the format cannot express something this file needs, that is information about
the format, and it is much cheaper to learn now than after 18 more files.

- [ ] **Step 3: Migrate the remaining 17 files**

For each remaining `crates/kali_cli/tests/string_*.rs`, create
`crates/kali_cli/tests/cases/string/<name without string_ prefix>.toml`. Where a
file loops over `["js", "ts", "jsx", "tsx"]`, use a `[matrix] ext = [...]` axis.
Where it varies text vs JSON output, use two sibling `[[case]]` blocks — not a
matrix axis (spec §5.6).

Preserve every `#[test]`'s claim. Carry over any explanatory comment as a
`rationale` field rather than a `#` comment, so it prints on failure.

- [ ] **Step 4: Run the family and audit every file**

```bash
cd /workspace
cargo test -p kali_cli --test cases -- string/ 2>&1 | tail -5
failed=0
while read -r old; do
  base=$(basename "$old" .rs); name=${base#string_}
  new="crates/kali_cli/tests/cases/string/$name.toml"
  if [ ! -f "$new" ]; then echo "MISSING: $new"; failed=1; continue; fi
  python3 scripts/audit-case-migration.py "$old" "$new" >/dev/null || { echo "AUDIT FAILED: $old"; failed=1; }
done < /tmp/string-family.txt
[ "$failed" -eq 0 ] && echo "ALL AUDITS OK" || echo "AUDITS INCOMPLETE — do not delete"
```

Expected: all cases pass and `ALL AUDITS OK`. If any audit fails, fix the case
file. Do not proceed to Step 5 otherwise.

- [ ] **Step 5: Confirm both suites are green together, then delete the old files**

Running both at once proves the new cases pass in the same tree state the old
ones do.

```bash
cd /workspace
bash scripts/test-gate.sh
xargs git rm < /tmp/string-family.txt
bash scripts/test-gate.sh
```

Expected: `GATE OK: 0 failing tests` both times.

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): migrate the string family to case files

19 test targets become 19 .toml case files under tests/cases/string/. Every
file's literal claims verified present by scripts/audit-case-migration.py before
deletion, and scripts/test-gate.sh green with both suites compiled.

First bulk migration -- the format handled the family without extension."
```

---

### Task 15: Migrate `array/`, `math/`, and `object/`

35 targets (21 + 1 + 13). Same procedure as Task 14, three families at once
because they share the same shape: build or run a snippet, assert stdout and the
JSON envelope.

**Files:**
- Create: `crates/kali_cli/tests/cases/array/*.toml` (21)
- Create: `crates/kali_cli/tests/cases/math/*.toml` (1)
- Create: `crates/kali_cli/tests/cases/object/*.toml` (13)
- Delete: `crates/kali_cli/tests/{array,math,object}_*.rs` (35 files)

**Interfaces:** unchanged from Task 14.

- [ ] **Step 1: List the families**

```bash
cd /workspace
ls crates/kali_cli/tests/array_*.rs crates/kali_cli/tests/math_*.rs \
   crates/kali_cli/tests/object_*.rs | tee /tmp/amo-family.txt | wc -l
```

Expected: 35.

- [ ] **Step 2: Migrate each file**

Same rules as Task 14 Step 3: one `.toml` per `.rs`, family prefix stripped from
the filename, `[matrix]` for uniform extension loops, sibling `[[case]]` blocks
for text-vs-JSON, comments become `rationale`.

- [ ] **Step 3: Run and audit**

```bash
cd /workspace
cargo test -p kali_cli --test cases -- array/ 2>&1 | tail -3
cargo test -p kali_cli --test cases -- math/ 2>&1 | tail -3
cargo test -p kali_cli --test cases -- object/ 2>&1 | tail -3
failed=0
while read -r old; do
  base=$(basename "$old" .rs)
  fam=${base%%_*}; name=${base#${fam}_}
  new="crates/kali_cli/tests/cases/$fam/$name.toml"
  if [ ! -f "$new" ]; then echo "MISSING: $new"; failed=1; continue; fi
  python3 scripts/audit-case-migration.py "$old" "$new" >/dev/null || { echo "AUDIT FAILED: $old"; failed=1; }
done < /tmp/amo-family.txt
[ "$failed" -eq 0 ] && echo "ALL AUDITS OK" || echo "AUDITS INCOMPLETE — do not delete"
```

Expected: all pass, `ALL AUDITS OK`.

- [ ] **Step 4: Delete and verify**

```bash
cd /workspace
bash scripts/test-gate.sh
xargs git rm < /tmp/amo-family.txt
bash scripts/test-gate.sh
```

Expected: `GATE OK: 0 failing tests` both times.

- [ ] **Step 5: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): migrate the array, math, and object families to case files

35 test targets become case files. All audits clean before deletion."
```

---

### Task 16: Migrate `soundness/`

29 targets, and the first family where prose matters: several of these files
carry commentary about what the test pins and why. Every comment becomes a
`rationale` field.

**Files:**
- Create: `crates/kali_cli/tests/cases/soundness/*.toml` (29)
- Delete: `crates/kali_cli/tests/soundness_*.rs` (29 files)

**Interfaces:** unchanged.

- [ ] **Step 1: List the family and note which files carry prose**

```bash
cd /workspace
ls crates/kali_cli/tests/soundness_*.rs | tee /tmp/soundness-family.txt | wc -l
echo "--- files with substantial commentary ---"
while read -r f; do
  n=$(grep -c '^//' "$f"); [ "$n" -gt 5 ] && printf "%4s comment lines  %s\n" "$n" "$f"
done < /tmp/soundness-family.txt
```

Expected: 29 files, with a handful flagged. Those need the most care.

- [ ] **Step 2: Migrate, moving every comment into `rationale`**

Same mechanics as Task 14. For flagged files, the `rationale` field on each case
carries the commentary that explains that specific claim. Do not summarize the
prose — move it. The whole point is that it prints on failure instead of sitting
invisible in a comment.

Where a file declares `const X: &str = "..."` for a pinned diagnostic message,
that becomes a `[constants]` entry referenced as `${X}`, so the "rule literals
are never hand-rolled" discipline stays greppable.

- [ ] **Step 3: Run and audit**

```bash
cd /workspace
cargo test -p kali_cli --test cases -- soundness/ 2>&1 | tail -3
failed=0
while read -r old; do
  base=$(basename "$old" .rs); name=${base#soundness_}
  new="crates/kali_cli/tests/cases/soundness/$name.toml"
  if [ ! -f "$new" ]; then echo "MISSING: $new"; failed=1; continue; fi
  python3 scripts/audit-case-migration.py "$old" "$new" >/dev/null || { echo "AUDIT FAILED: $old"; failed=1; }
done < /tmp/soundness-family.txt
[ "$failed" -eq 0 ] && echo "ALL AUDITS OK" || echo "AUDITS INCOMPLETE — do not delete"
```

Expected: all pass, `ALL AUDITS OK`. The audit's "rule constants" line is the one
to watch here — a dropped constant is exactly the degradation this family is
vulnerable to.

- [ ] **Step 4: Delete and verify**

```bash
cd /workspace
bash scripts/test-gate.sh
xargs git rm < /tmp/soundness-family.txt
bash scripts/test-gate.sh
```

- [ ] **Step 5: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): migrate the soundness family to case files

29 test targets become case files. Every explanatory comment moved into a
rationale field, so it prints on failure instead of sitting invisible; every
pinned diagnostic literal moved into [constants] and referenced as \${NAME}.

All audits clean before deletion, with particular attention to the rule-constant
count -- a dropped constant is the exact degradation this family risks."
```

---

### Task 17: Migrate `switch/`

Only 2 targets, but `switch_fail_closed.rs` is 1,492 lines carrying ~80 lines of
commentary that records two real degradation incidents. This task is small in
count and high in care.

**Files:**
- Create: `crates/kali_cli/tests/cases/switch/fail_closed.toml`
- Create: `crates/kali_cli/tests/cases/switch/runtime.toml`
- Delete: `crates/kali_cli/tests/switch_fail_closed.rs`
- Delete: `crates/kali_cli/tests/switch_runtime.rs`

**Interfaces:** unchanged.

- [ ] **Step 1: Read `switch_fail_closed.rs` in full and inventory it**

```bash
cd /workspace
grep -c '#\[test\]' crates/kali_cli/tests/switch_fail_closed.rs
grep -n 'const [A-Z0-9_]*:' crates/kali_cli/tests/switch_fail_closed.rs
```

Every `const` becomes a `[constants]` entry. Every `#[test]` becomes a
`[[case]]`. The file-level header comment explains the pinning discipline as a
whole — put it as a `#` comment at the top of the `.toml`, since it describes the
file rather than any one case. Per-case commentary goes in `rationale`.

- [ ] **Step 2: Note the three cases that deliberately do not pin via the helper**

The file header documents these explicitly and they must survive migration
intact:

- `a_labeled_break_in_a_clause_is_fail_closed` denies with **E3100** from name
  resolution and never reaches E5506.
- The two `NO_ENCLOSING_LOOP` cases assert their constant *and* that the other
  cause's message is **absent** — a disjointness claim.

The absence claims map onto `stderr_absent`. Do not collapse them into a single
`stderr_contains` list; the whole point is that they distinguish two causes.

- [ ] **Step 3: Write both case files**

The `fail_closed` skeleton, to be filled out to all its cases:

```toml
# Migrated from tests/switch_fail_closed.rs.
#
# Every E5506 cell pins the exact rule it denies by. A test that asserts only
# "some E5506" silently degrades the moment the cell it names starts denying for
# a DIFFERENT reason -- most often Rule 1, since a discriminant proof regression
# makes every switch in the file deny, and it keeps passing while having lost its
# entire purpose. That failure mode was found twice on this plan, so it is closed
# structurally: rule literals live in [constants] and are never hand-rolled.
#
# Three cells legitimately do not pin an E5506 rule and must not:
# labeled_break denies with E3100 from name resolution and never reaches E5506;
# the two NO_ENCLOSING_LOOP cells also assert the OTHER cause's message is
# ABSENT, which is a disjointness claim.

[constants]
RULE_1_DISCRIMINANT = "the discriminant is not a proven integer or string"
RULE_2_CASE_TEST = "a `case` test that is not a literal in the discriminant's domain"
RULE_4_TERMINATOR = "a clause that does not end in `return`, `break` or `continue`"
RULE_5_BLOCK_BINDING = "a `let`/`const` declaration in a clause body"
NO_ENCLOSING_LOOP = "`continue` inside a `switch` requires an enclosing loop; there is none here"
UNFAITHFUL_CONTINUE = "does not re-run the update/test faithfully in the current lowering (register R-09)"
REPR_MIXED_CONFLICT = "is used as both a string and a number"
TRAILING_EMPTY_GROUP = "an empty trailing clause with no body to group onto"

[[case]]
name = "float_discriminant"
rationale = """
Pins Rule 1 exactly. This cell asserted only contains("E5506") until the
pre-merge fix wave; it was the one test in this file that had already drifted.
"""
exit = "failure"
args = ["run", "main.js"]
stderr_contains = ["E5506", "${RULE_1_DISCRIMINANT}"]
```

Each case needs its own `[source]`. Since the cases differ in source rather than
sharing one, put per-case sources in separate files: give each `[[case]]` its own
snippet by naming distinct source files in the file-level `[source]` table and
pointing each case's argv at the right one. For example `[source]` holds
`"float.js"`, `"labeled-break.js"`, and so on, and the float case runs
`args = ["run", "float.js"]`.

- [ ] **Step 4: Run and audit**

```bash
cd /workspace
cargo test -p kali_cli --test cases -- switch/ 2>&1 | tail -5
python3 scripts/audit-case-migration.py \
  crates/kali_cli/tests/switch_fail_closed.rs \
  crates/kali_cli/tests/cases/switch/fail_closed.toml
python3 scripts/audit-case-migration.py \
  crates/kali_cli/tests/switch_runtime.rs \
  crates/kali_cli/tests/cases/switch/runtime.toml
```

Expected: all cases pass, both audits `AUDIT OK`. The rule-constant count in the
audit output must match the `const` count from Step 1.

- [ ] **Step 5: Verify each pinned rule still bites**

An audit proves the literal is present; it does not prove the case would fail if
the compiler stopped emitting it. Spot-check by corrupting one constant and
confirming the case goes red.

```bash
cd /workspace
sed -i 's/RULE_1_DISCRIMINANT = "the discriminant/RULE_1_DISCRIMINANT = "XX the discriminant/' \
  crates/kali_cli/tests/cases/switch/fail_closed.toml
cargo test -p kali_cli --test cases -- switch/fail_closed 2>&1 | grep -E 'test result|FAILED'
git checkout crates/kali_cli/tests/cases/switch/fail_closed.toml
```

Expected: at least one FAILED. A green run means the constant is not actually
being asserted anywhere.

- [ ] **Step 6: Delete and verify**

```bash
cd /workspace
bash scripts/test-gate.sh
git rm crates/kali_cli/tests/switch_fail_closed.rs crates/kali_cli/tests/switch_runtime.rs
bash scripts/test-gate.sh
```

- [ ] **Step 7: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): migrate the switch family to case files

switch_fail_closed.rs was the highest-risk file in the migration: 1492 lines
whose header records two incidents where a cell degraded to asserting 'some
E5506'. All rule literals moved to [constants]; the three cells that
deliberately do not pin an E5506 rule kept their distinct shapes, including the
two disjointness claims that now use stderr_absent.

Verified by corrupting a constant and confirming the case goes red -- the audit
proves presence, this proves the pin still bites."
```

---

### Task 18: Migrate `browser/`

162 targets, the largest family and the only one exercising all three step kinds.
Migrate in sub-batches so a broken assumption surfaces early rather than after
162 files.

**Files:**
- Create: `crates/kali_cli/tests/cases/browser/*.toml` (~159)
- Delete: `crates/kali_cli/tests/browser_*.rs` (~159)

Four `browser_*` targets are **not** migrated and must remain untouched:
`browser_cdp_smoke.rs` and `browser_harness_failing_test_propagates_failure.rs`
stay hand-written per spec §5.11; `browser_harness_cdp_in_page_trap_propagates.rs`
already moved into `tests/inprocess/` in Task 6; and any file using
`tests/cdp_driver/`.

**Interfaces:** unchanged.

- [ ] **Step 1: Build the work list, excluding the survivors**

```bash
cd /workspace/crates/kali_cli/tests
ls browser_*.rs > /tmp/browser-all.txt
grep -rl 'cdp_driver' . --include='*.rs' | sed 's|^\./||' > /tmp/browser-keep.txt
echo "browser_cdp_smoke.rs" >> /tmp/browser-keep.txt
echo "browser_harness_failing_test_propagates_failure.rs" >> /tmp/browser-keep.txt
sort -u /tmp/browser-keep.txt -o /tmp/browser-keep.txt
comm -23 <(sort /tmp/browser-all.txt) /tmp/browser-keep.txt > /tmp/browser-migrate.txt
wc -l /tmp/browser-migrate.txt /tmp/browser-keep.txt
```

Expected: ~159 to migrate, ~3 to keep.

- [ ] **Step 2: Migrate the first five as a pilot, then audit**

Pick five that between them use all three step kinds — one pure `build` bundle
check, one with an `app.meta.json` assertion, one with a
`browser_bundle_harness` step, and two that loop over extensions.

```bash
cd /workspace
head -5 /tmp/browser-migrate.txt
```

Migrate those five, then:

```bash
cd /workspace
cargo test -p kali_cli --test cases -- browser/ 2>&1 | tail -5
while read -r old; do
  base=$(basename "$old" .rs); name=${base#browser_}
  python3 scripts/audit-case-migration.py "crates/kali_cli/tests/$old" \
    "crates/kali_cli/tests/cases/browser/$name.toml" >/dev/null \
    && echo "OK   $old" || echo "FAIL $old"
done < <(head -5 /tmp/browser-migrate.txt)
```

Expected: cases pass, all five `OK`. If the `browser_bundle_harness` step kind
needs an option the format lacks (a different `allow_subpaths`, a page rather
than a module harness), add it to `Step` in `model.rs` with a test, in its own
commit, before continuing. Discovering that now is the purpose of the pilot.

- [ ] **Step 3: Commit the pilot separately**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): migrate five browser cases as a format pilot

Covers all three step kinds and the extension matrix before committing to the
remaining ~154."
```

- [ ] **Step 4: Migrate the remaining ~154 in batches of about 25**

For each batch: write the case files, run `cargo test -p kali_cli --test cases -- browser/`,
audit every file in the batch, then commit the batch. Do not delete any `.rs`
file yet — deletion happens once for the whole family in Step 5, so the gate can
compare both suites in one tree state.

Batch commit message template:

```
test(cli): migrate browser cases <N>-<M> to case files

<count> targets. All audits clean; old .rs files retained until the family
audit completes.
```

- [ ] **Step 5: Audit the whole family, then delete**

```bash
cd /workspace
failed=0
while read -r old; do
  base=$(basename "$old" .rs); name=${base#browser_}
  new="crates/kali_cli/tests/cases/browser/$name.toml"
  if [ ! -f "$new" ]; then echo "MISSING: $new"; failed=1; continue; fi
  python3 scripts/audit-case-migration.py "crates/kali_cli/tests/$old" "$new" >/dev/null \
    || { echo "AUDIT FAILED: $old"; failed=1; }
done < /tmp/browser-migrate.txt
[ "$failed" -eq 0 ] && echo "ALL AUDITS OK" || echo "AUDITS INCOMPLETE — do not delete"
bash scripts/test-gate.sh
```

Expected: `ALL AUDITS OK` and `GATE OK: 0 failing tests`. Only then:

```bash
cd /workspace/crates/kali_cli/tests
xargs git rm < /tmp/browser-migrate.txt
cd /workspace && bash scripts/test-gate.sh
```

- [ ] **Step 6: Confirm the survivors are intact**

```bash
cd /workspace
ls crates/kali_cli/tests/browser_*.rs
cargo test -p kali_cli --test browser_cdp_smoke -- --ignored 2>&1 | tail -3
grep -rn 'browser_cdp_smoke' mise.toml .github/workflows/ci.yml
```

Expected: only the ~3 kept files remain; the CDP target still builds; the
`mise.toml` and `ci.yml` references still resolve.

- [ ] **Step 7: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): migrate the browser family to case files

~159 test targets become case files under tests/cases/browser/. This was the
largest family and the only one using all three step kinds.

browser_cdp_smoke and browser_harness_failing_test_propagates_failure stay
hand-written per spec 5.11; both still build and the mise/CI references to them
still resolve."
```

---

### Task 19: Migrate the remaining families and the miscellaneous targets

Everything left that is black-box CLI-shaped: `clbg_` (6), `late_compat_` (5),
`for_of_` (4), `permission_` (3), `promise_`/`parse_`/`number_`/`nullish_`/
`module_`/`growable_` (2 each), `for_await_` (1), plus the 26 targets with no
family prefix.

Explicitly **not** migrated, per spec §5.11: `runtime_smoke`, `package_corpus`,
`schema_docs`, `node_api_surface`, `schema_validation`, and anything whose
assertions use `starts_with` or `lines()` in a way the eight keys cannot express.

**Files:**
- Create: `crates/kali_cli/tests/cases/<family>/*.toml`
- Create: `crates/kali_cli/tests/cases/misc/*.toml` (for the unprefixed targets)
- Delete: the migrated `.rs` files

**Interfaces:** unchanged.

- [ ] **Step 1: Build the work list and the keep list**

```bash
cd /workspace/crates/kali_cli/tests
ls *.rs | sed 's/\.rs$//' > /tmp/remaining-all.txt
cat > /tmp/remaining-keep.txt <<'EOF'
cases
inprocess
runtime_smoke
package_corpus
schema_docs
schema_validation
node_api_surface
browser_cdp_smoke
browser_harness_failing_test_propagates_failure
EOF
# Targets whose assertions may exceed the eight keys.
echo "--- uses starts_with or lines() ---"
grep -l 'starts_with(\|\.lines()' *.rs 2>/dev/null
comm -23 <(sort /tmp/remaining-all.txt) <(sort /tmp/remaining-keep.txt) > /tmp/remaining-migrate.txt
wc -l /tmp/remaining-migrate.txt
```

Review the `starts_with`/`lines()` list. For each, decide: either the assertion
maps onto `stdout_contains` without weakening (a `starts_with` on a full-line
prefix often does), or the target joins the keep list. Record the decision for
each in the Step 6 commit message. **Do not** weaken an assertion to fit the
format.

- [ ] **Step 2: Migrate the prefixed families**

For `clbg_`, `late_compat_`, `for_of_`, `for_await_`, `permission_`, `promise_`,
`parse_`, `number_`, `nullish_`, `module_`, `growable_`: one `.toml` per `.rs`
under `tests/cases/<prefix without trailing underscore>/`, prefix stripped from
the filename. Same rules as Task 14.

- [ ] **Step 3: Migrate the unprefixed targets into `misc/`**

The 26 targets with no shared prefix (`arena_reclamation_runtime`,
`binary_stdout_runtime`, `bitwise_operators_runtime`, `closure_return_isolation`,
`compound_assignment_wrapped_local_binding`, `doctor`,
`exponentiation_operator`, `float_console_runtime`,
`frozen_set_map_constructor_result`, `heap_grow_runtime`,
`imperative_core_runtime`, `init`,
`logical_assignment_wrapped_local_binding`, `map_iteration_runtime`,
`param_compound_assign`, `phase_three_host_apis`, `reclamation_bounded_peak`,
`reflect_own_keys_js_input`, `set_iteration_runtime`,
`standalone_non_literal_iterator_sources`, `static_enumeration_stage2`,
`template_literal_interpolation_runtime`, `thread_topology_json`,
`trap_diagnostics_runtime`, `version_smoke`, `wrapped_call_targets_wrappers`)
go to `tests/cases/misc/<name>.toml`, keeping their full names.

- [ ] **Step 4: Also migrate the `runtime_` family except `runtime_smoke`**

`runtime_` has 15 targets; `runtime_smoke` stays. The other 14 are ordinary
black-box tests.

- [ ] **Step 5: Run, audit everything, then delete**

> **Superseded — do not run the deletion loop below.** The stem-suffix match
> (`case "$stem" in *"$b")`) is a name scan, not a classifier: every stem ending
> in `runtime` matches `cases/switch/runtime.toml`. Measured against the
> deletion commit's parent and recorded in `t19_deletion_classify.py`'s
> docstring, the scan names 8 sources that no case file claims —
> `binary_stdout_runtime`, `imperative_core_runtime`, and all six
> `clbg_*_runtime` benchmark targets — and only a printed
> `AUDITS INCOMPLETE — do not delete` sits between that
> mis-mapping and the unconditional `git rm` loop directly beneath it, which
> would have deleted live, unmigrated coverage. Task 19 used
> `tools/migration/t19_deletion_classify.py` instead: it resolves each case
> file's own `Migrated from` line to partition every on-disk
> `crates/kali_cli/tests/*.rs` into DELETE / RETAINED / NOT MIGRATED, and hard-stops
> on an unparsed claim or an ambiguous audit result rather than guessing. The
> commands below are kept as the record of what was planned.

```bash
cd /workspace
cargo test -p kali_cli --test cases 2>&1 | tail -5
failed=0
while read -r stem; do
  old="crates/kali_cli/tests/$stem.rs"
  # Find the case file wherever it landed.
  new=$(find crates/kali_cli/tests/cases -name "*.toml" | while read -r c; do
    b=$(basename "$c" .toml)
    case "$stem" in *"$b") echo "$c"; break;; esac
  done | head -1)
  if [ -z "$new" ]; then echo "MISSING case for $stem"; failed=1; continue; fi
  python3 scripts/audit-case-migration.py "$old" "$new" >/dev/null \
    || { echo "AUDIT FAILED: $stem"; failed=1; }
done < /tmp/remaining-migrate.txt
[ "$failed" -eq 0 ] && echo "ALL AUDITS OK" || echo "AUDITS INCOMPLETE — do not delete"
bash scripts/test-gate.sh
```

Expected: `ALL AUDITS OK` and `GATE OK`. Then delete:

```bash
cd /workspace/crates/kali_cli/tests
while read -r stem; do git rm "$stem.rs"; done < /tmp/remaining-migrate.txt
cd /workspace && bash scripts/test-gate.sh
```

- [ ] **Step 6: Commit**

```bash
cd /workspace
git add -A crates/kali_cli
git commit -m "test(cli): migrate the remaining CLI families to case files

Covers clbg, late_compat, for_of, for_await, permission, promise, parse, number,
nullish, module, growable, the 14 non-smoke runtime targets, and the 26
unprefixed targets (now under cases/misc/).

Retained as hand-written per spec 5.11: runtime_smoke, package_corpus,
schema_docs, node_api_surface, schema_validation, browser_cdp_smoke,
browser_harness_failing_test_propagates_failure, inprocess.

starts_with/lines() decisions: <one line per affected target>."
```

---

### Task 20: Final measurement and documentation

**Files:**
- Modify: `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`
- Modify: `AGENTS.md`
- Create: `crates/kali_cli/tests/cases/README.md`

**Interfaces:** none.

- [ ] **Step 1: Measure the end state from a cold target dir**

```bash
cd /workspace
cargo clean
cargo test -p kali_cli --no-run 2>&1 | tail -3
du -sh .cache/cargo-target
du -sh .cache/cargo-target/debug/deps
echo "--- test targets in kali_cli ---"
ls .cache/cargo-target/debug/deps | grep -vE '\.(d|rlib|rmeta)$' | wc -l
echo "--- >100MB ---"
cd .cache/cargo-target/debug/deps && ls -1 | while read f; do
  case "$f" in *.d|*.rlib|*.rmeta) continue;; esac
  s=$(stat -c%s "$f" 2>/dev/null || echo 0)
  [ "$s" -gt 104857600 ] && printf "%5s MB  %s\n" $((s/1048576)) "$f"; done
```

Record every number. The spec predicts `deps` ~3.9 GB and whole-tree ~8 GB, with
`inprocess`, the `kali` binary, and five in-crate unit-test binaries as the only
items over 100 MB.

- [ ] **Step 2: Count the outcome**

```bash
cd /workspace
echo "hand-written test targets: $(ls crates/kali_cli/tests/*.rs | wc -l)"
echo "case files: $(find crates/kali_cli/tests/cases -name '*.toml' | wc -l)"
echo "trials: $(cargo test -p kali_cli --test cases -- --list 2>/dev/null | grep -c ': test$')"
echo "test lines deleted: $(git diff --stat main -- crates/kali_cli/tests | tail -1)"
```

- [ ] **Step 3: Write the case-tree README**

`crates/kali_cli/tests/cases/README.md`:

```markdown
# CLI test cases

Each `.toml` here is one black-box test of the `kali` binary. Adding a case
compiles nothing — the `cases` target discovers this tree at runtime.

```bash
cargo test -p kali_cli --test cases              # everything
cargo test -p kali_cli --test cases -- switch/   # one family
cargo test -p kali_cli --test cases -- --ignored # the gated cases
```

Format reference: `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`
§5. In short: `[constants]`, `[matrix]`, `[source]`, and one or more `[[case]]`,
each either a single inline step or an ordered `[[case.step]]` list.

Two rules worth stating up front:

- **Put the reason in `rationale`, not a `#` comment.** The runner prints
  `rationale` when the case fails. A comment explaining why a test exists is
  invisible exactly when someone needs it.
- **Pin diagnostic text through `[constants]`.** A hand-copied message prefix
  goes insensitive the moment the diagnostic widens. That has happened here
  before.

A case the format cannot express belongs in its own hand-written target, not in a
weakened `.toml`. The eight assertion keys are deliberately closed; see §5.11 for
what stays Rust and why.
```

- [ ] **Step 4: Record the outcome in the spec**

Append to `docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md`:

```markdown
## 9) Outcome as built

Measured on a cold target dir after Task 20:

| | predicted | actual |
| --- | --- | --- |
| `debug/deps` | ~3.9 GB | <fill in> |
| whole tree | ~8 GB | <fill in> |
| `kali_cli` test targets | ~9 | <fill in> |
| case files | — | <fill in> |
| expanded trials | — | <fill in> |
| test lines deleted | ~200k | <fill in> |

Targets over 100 MB: <fill in>.

Deviations from the design: <fill in, or "none">.
```

Fill every `<fill in>` from Steps 1–2. If a number missed its prediction by more
than ~30%, say so plainly and explain why — an unexplained miss is worth more to
the next reader than a rounded-down one.

- [ ] **Step 5: Point `AGENTS.md` at the new convention**

In `AGENTS.md` §5, under "Conformance expectations", after the line
`- Rust unit tests in sibling \`*tests.rs\` files, not inline \`#[cfg(test)]\` modules`,
add:

```markdown
- Black-box CLI tests are `.toml` case files under `crates/kali_cli/tests/cases/`,
  run by the single `cases` target — not new `tests/*.rs` integration targets.
  See `crates/kali_cli/tests/cases/README.md`. Add a hand-written target only
  when a test genuinely needs in-process runtime access or drives a real browser.
```

- [ ] **Step 6: Verify the whole gate one final time and commit**

```bash
cd /workspace
bash scripts/test-gate.sh
bash scripts/check-determinism.sh
cargo test -p kali_cli --test browser_cdp_smoke -- --ignored 2>&1 | tail -3
```

Expected: `GATE OK: 0 failing tests`, determinism lane green, CDP target builds.

```bash
cd /workspace
git add -A docs AGENTS.md crates/kali_cli/tests/cases/README.md
git commit -m "docs: record the test-consolidation outcome and the case convention

Adds the measured end state to the spec, a README for the case tree, and an
AGENTS.md line pointing new CLI tests at .toml cases rather than new integration
targets.

Verified: test-gate green, determinism lane green, browser_cdp_smoke still
builds."
```

---

# Self-Review

**Spec coverage.** Walking the spec section by section:

| spec § | covered by |
| --- | --- |
| §1 problem, §2 goals | plan preamble, Global Constraints |
| §4.1 the four moved files | Tasks 1–4 (one per file, dependency-ordered) |
| §4.1 the four `pub(crate)` promotions | Task 1 Step 5, Task 2 Step 2, Task 3 Step 2 (table in the Phase 1 header) |
| §4.2 compatibility by re-export | Task 1 Step 7, and the re-export step of Tasks 2–4 |
| §4.3 the `use crate::*;` cost | explicit import lists in Tasks 1–4 |
| §4.4 `schema_validation` → subprocess | Task 7 |
| §4.4 consolidate the three in-process tests | Task 6 |
| §4.5 Phase 1 result | Task 7 Step 5 |
| §5.1 layout, test ids | Task 9 (`expand`), Task 12 (`discover`) |
| §5.2 schema | Task 8 |
| §5.3 three step kinds | Task 11 |
| §5.4 eight assertion keys + `ignore` | Task 10, Task 12 Step 7 |
| §5.5 rationale and `[constants]` | Task 9 (substitution), Task 11 (printing), Tasks 16–17 (use) |
| §5.6 worked example, no matrix for output shape | Task 12 Step 4, Task 14 Step 3, Task 15 Step 2 |
| §5.7 substitution closed at two forms | Task 9 Step 3 |
| §5.8 runner, `harness = false`, deps | Task 8 Step 3, Task 12 Steps 4–5 |
| §5.9 failure output | Task 11 Step 3, verified Task 12 Step 8 |
| §5.10 five hard failures | Task 8 (unknown key, zero cases, empty axis), Task 9 (unresolved placeholder), Task 12 (zero files, missing dir) |
| §5.11 what stays Rust | Task 18 Step 1, Task 19 Step 1 |
| §6.1 migration order | Tasks 14 → 15 → 16 → 17 → 18 → 19 |
| §6.2 audit gate | Task 13, applied in every migration task |
| §6.3 CI surface unchanged | Global Constraints, verified Task 6 Step 5, Task 18 Step 6, Task 20 Step 6 |
| §7 expected outcome | Task 20 Steps 1–2, recorded in §9 |
| §8 risks | each risk's mitigation is a step: `use crate::*` (Task 1 Step 1 baseline), weakened case (Task 13), format as vector (Task 8), DSL pressure (Task 9), granularity (Task 12 Step 7), reference-count misread (Task 5 Step 4's escalation rule) |

No spec requirement is unimplemented.

**Two additions beyond the spec**, both discovered while planning:

1. **Task 6 consolidates three targets into one** rather than leaving three fat
   binaries. The spec §4.4 says to do this; the plan makes the `[[test]]` wiring
   explicit.
2. **Task 12 Step 8 checks the runner's failure text against `scripts/test-gate.sh`'s
   awk parser.** The gate treats `^    [A-Za-z_]` as a failed-test name, so a
   four-space-indented detail line would be misread as a nonexistent test. This
   is why `assertions.rs` indents with two spaces and `  | ` — it is a real
   coupling the spec did not name, and Task 10's last test pins it.

**Placeholder scan.** The only intentional fill-ins are `<N>`, `<BEFORE>`,
`<AFTER>`, and `<fill in>` in commit messages and the spec's §9 outcome table —
each is a measurement the executing engineer takes in the step immediately
before, and each step says which command produces it. No step defers work.

**Type consistency.** Checked across tasks:

- `Exit` is `Exit::Status(ExitStatusWord::Success)` / `Exit::Code(i32)`
  throughout. Task 8 Step 4 flags that the Step 1 test text uses the shorthand
  `Exit::Success` and must be reconciled to one spelling — called out explicitly
  rather than left to bite.
- `Step`, `Case`, `CaseFile`, `Trial`, `Captured`, `RunnerConfig`, `StepKind` are
  defined once (Tasks 8–11) and every later use matches those field names.
- `check` and `check_json` are both used by Task 11's `run_file_json`; `check_json`
  is `pub` in Task 10 for exactly that reason.
- `expand(stem, file)` produces `Trial::id` in the `<stem>[<axes>]::<case>` form
  that Task 12's filtering and Task 20's `--list` count both assume.
- `StepKind` needs `Clone + Copy + Debug + PartialEq + Eq`; noted in Tasks 9 and
  11 where each is first required.

**One known rough edge, flagged rather than hidden.** `Case` cannot carry both
`#[serde(flatten)]` (for the single-step shorthand) and `deny_unknown_fields`
— serde rejects that combination. Task 8 Step 4 resolves it by dropping
`deny_unknown_fields` from `Case` alone and adding explicit
no-step/mixed-step checks, with tests. `Step` keeps `deny_unknown_fields`, which
is where typo'd assertion keys would land, so §5.10's guarantee holds where it
matters. If the executing engineer finds the flatten approach fights serde
further, the fallback is an explicit `[[case.step]]` for every case and no inline
shorthand — more verbose case files, same guarantees.
