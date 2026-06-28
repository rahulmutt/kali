# kali_cli `runtime_smoke.rs` modularization — design (22nd in series; kali_cli sub-project 2 of 3)

Date: 2026-06-28
Status: approved
Crate: `kali_cli` (22nd in the kali workspace modularization series; kali_cli sub-project 1 of 3 — production `src/` — landed at `5b6c7ff82`).
Scope: **sub-project 2 of 3** for kali_cli. This spec covers splitting the `tests/runtime_smoke.rs` integration-test monolith (73,625 lines, 1,816 `#[test]` fns) into per-command submodules. Sub-project 3 (subdirectory grouping for the ~370 small per-behavior files under `tests/`) and the other test monoliths (`package_corpus.rs`, `node_api_surface.rs`, `schema_docs.rs`, `late_compat_browser_*`) are deferred to later specs — each big file is its own spec, per the sub-project 1 deferral.

## Goal & invariant

Pure code-motion. Decompose `tests/runtime_smoke.rs` into a thin-ish root (shared helpers + `use` block + `mod` decls) plus per-command submodules, with **zero behavior change**: identical test count (1,816), identical pass/fail, identical scheduling/serialization scope. The file stays a **single integration-test binary** — no new test binaries, no N× link cost, no change to shared-state semantics.

Allowed changes only: `mod`/`#[path]` declarations, `use super::*;` at the head of each submodule, and verbatim relocation of `#[test]` fn bodies (attributes intact). Do **not** run `cargo fmt` (verbatim moves + the root's existing >100-col lines are already red on baseline, so not regressions; running fmt would violate the verbatim mandate).

## Baseline (branch base)

On a `refactor/kali-cli-runtime-smoke-modularization` branch off main. Confirm `cargo build -p kali_cli` clean (0 warnings); `cargo test -p kali_cli` green except the 2 pre-existing `build_bundles_*` failures in `array_from_bracketed_root_wrappers` (codegen/bundling, unrelated to this refactor — confirm reproduction on the branch base before starting, do not attribute; note these live in a different test binary, not `runtime_smoke`). Capture `cargo test -p kali_cli --test runtime_smoke -- --list` baseline (exact test count + names) in the SDD ledger before starting. Record exact branch-base HEAD.

## Current shape

- `tests/runtime_smoke.rs` (73,625 lines): a single flat integration-test binary that subprocess-drives the compiled `kali` binary (`CARGO_BIN_EXE_kali`).
- **1,816 `#[test]` fns**, all top-level (0 nested `mod` blocks — verified). 0 `#[tokio::test]` (no async), 0 `#[serial]`, 42 `#[cfg(unix)]` tests (browser-entrypoint + browser-summary-fallback, all `run_*`/`test_*` prefixed).
- **~183 shared helper fns** (non-test): `kali_bin`, `fixture_root`/`fixture_path`, `parse_json_stdout`, `write_browser_api_surface_manifest`, fixture writers (`write_valid_policy`, `write_browser_runtime_*_fixture`, `build_package_tarball`, …), test-servers (`start_registry_metadata_server`, `start_binary_response_server`), wasm inspectors (`count_i64_adds`, `count_tag_boxing_ops`, `count_wasm_instructions`), and a large cross-cutting `assert_*` / `*_source` cluster (`assert_browser_*`, `assert_permission_escalation_*`, `permission_escalation_*_source`, `late_*_source`, `browser_runtime_*_source`, …). 4 cfg-gated helpers (`shell_quote_path`, `browser_entrypoint_smoke`, `run_browser_entrypoint_smoke`, `test_browser_entrypoint_smoke`, all `#[cfg(unix)]`).
- `use` block imports `std` (collections/fs/io/net/path/process/sync/thread/time), `base64`, `flate2`, `serde_json`, `sha2`, `tar`, `wasmparser`, `kali_common`, `kali_optimize`, `kali_runtime`, `tempfile`.
- Helpers are **genuinely cross-cutting**: a single `run_*` test routinely calls `kali_bin` + `fixture_path` + `parse_json_stdout` + `start_registry_metadata_server` + `assert_browser_*` + `permission_escalation_*_source`. They cannot跟随 a single command module.

### Key structural finding — clean command-based fault lines (unlike `build_tests.rs`)

Sub-project 1 deliberately left `build_tests.rs` / `output_tests.rs` monolithic because they are flat black-box suites keyed on **input semantics**, where per-module partitioning would invent arbitrary boundaries against the grain (high risk, no navigability payoff). `runtime_smoke.rs` is different: its 1,816 tests cluster strongly by **CLI command prefix**, with `json_<cmd>` as the JSON-output variant of the same command. The first-token breakdown (test fns only):

| prefix group | tests | note |
|---|---:|---|
| `run` + `json_run` | ~582 | the `run` command |
| `test` + `json_test` | ~473 | the `test` command |
| `build` + `json_build` | ~320 | the `build` command |
| `check` + `json_check` | ~164 | the `check` command |
| `package` + `json_package` | ~123 | the `package` command (`package_audit`/`_effects`/`_registry`) |
| `effects` + `json_effects` | ~70 | the `effects` command |
| `install` + `json_install` | ~23 | the `install` command |
| long-tail | 43 | `browser_*` (11), `late_*` (8), `node_*` (4), `permission_*` (3), `release_*` (3), `threaded`/`standalone`/`default` (2 each), + 9 singletons |
| `fmt`/`lint`/`doctor`/`init` | 18 | small commands (4/4/3/7), folded into `misc.rs` |

`json_<cmd>` second-token breakdown confirms `json_X` belongs with `X`: `json_run` 183, `json_test` 178, `json_build` 99, `json_check` 44, `json_effects` 8, `json_package` 6, `json_init`/`json_fmt`/`json_lint`/`json_install` 1 each.

This mirrors sub-project 1's `bin/cmd_*.rs` per-command split — a real navigability win is available, so splitting is justified here where it was not for `build_tests.rs`.

## Approach — `#[path]` module includes, single test binary (Approach A)

`tests/runtime_smoke.rs` stays **one integration-test binary**. It becomes the root: the `use` block (unchanged) + the ~183 shared helpers (unchanged) + `mod` declarations that `#[path]`-include per-command submodules from `tests/runtime_smoke/<group>.rs`. Each submodule begins with `use super::*;` and holds verbatim-moved `#[test]` fns.

**Why single-binary (not multiple `tests/*.rs` binaries):**
- **Zero behavior change:** every test stays in the same binary, so any `static`/`OnceLock`/`AtomicBool`/`Mutex` shared state, `#[serial]` scope (0 today), and `CARGO_BIN_EXE_kali` subprocess scaffolding behave identically. Multiple binaries would split `#[serial]` scope per-binary — a real scheduling behavior change.
- **No compile-cost regression:** `runtime_smoke` is already the workspace's most expensive test binary; N binaries would N× relink the whole crate graph.
- **Series-consistent:** sub-project 1 used the same `#[path] mod` mechanism for the co-located `build_tests.rs` / `output_tests.rs`.

**Why helpers stay in the root (not extracted to `helpers.rs`):**
- **Zero `pub(crate)` churn** — helper signatures are untouched (the verbatim mandate, maximally honored). Child modules see their parent's private items in Rust, so submods reach root helpers via `use super::*` with **no visibility widening**. This makes the sub-project the cleanest in the series on the visibility axis (phase-1's `bin/` needed `pub(crate)` for cross-sibling helpers; this needs none).
- `use super::*` also re-imports the root's `use` block (std, `serde_json`, `kali_common` fns, `kali_optimize`, …), so submodules need **no `use` block of their own**. Unused glob-imported names are not flagged by `unused_imports` → the 0-warning gate holds.
- The root is intentionally **not a thin facade**. Unlike sub-project 1's lib `mod.rs` (which re-exports a public surface), this root is the test binary's **shared-scaffolding scope** — there is no public API to re-export. A non-thin root is the correct shape.

**Alternative A2 (rejected, available as fallback):** extract helpers into `tests/runtime_smoke/helpers.rs` as `pub(crate)`, mirroring `bin/shared.rs`, for a thinner root. Rejected as higher-churn (~183 `pub(crate)` prefixes + extra wiring) for no functional gain. The series values minimal verbatim motion.

## Target layout

```
tests/runtime_smoke.rs          # root: use block + 183 helpers (unchanged) + mod decls
tests/runtime_smoke/run.rs      # run_*  + json_run_*   (~582 tests)
tests/runtime_smoke/test.rs     # test_* + json_test_*  (~473 tests)
tests/runtime_smoke/build.rs    # build_* + json_build_* (~320 tests)
tests/runtime_smoke/check.rs    # check_* + json_check_* (~164 tests)
tests/runtime_smoke/package.rs  # package_audit/_effects/_registry + json_package (~123 tests)
tests/runtime_smoke/effects.rs  # effects_* + json_effects_* (~70 tests)
tests/runtime_smoke/install.rs  # install_* + json_install_* (~23 tests)
tests/runtime_smoke/misc.rs     # long-tail + fmt/lint/doctor/init (~43 + 18 tests)
```

### Root shape

```rust
// tests/runtime_smoke.rs  (head, unchanged)
use std::{ … };
#[cfg(unix)] use std::os::unix::fs::symlink;
use base64::Engine;
… // full use block, unchanged
use kali_common::{ … };
use kali_optimize::{ … };
use kali_runtime::split_command_spec;
use tempfile::tempdir;

fn kali_bin() -> PathBuf { … }
… // all ~183 helpers, unchanged, private

#[path = "runtime_smoke/run.rs"] mod run;
#[path = "runtime_smoke/test.rs"] mod test;
#[path = "runtime_smoke/build.rs"] mod build;
#[path = "runtime_smoke/check.rs"] mod check;
#[path = "runtime_smoke/package.rs"] mod package;
#[path = "runtime_smoke/effects.rs"] mod effects;
#[path = "runtime_smoke/install.rs"] mod install;
#[path = "runtime_smoke/misc.rs"] mod misc;
```

### Submodule shape

```rust
// tests/runtime_smoke/run.rs
use super::*;

#[test]
fn run_supports_…() { … }   // verbatim move incl. #[test] / #[cfg(unix)] attrs
…
```

### Assignment rule (deterministic, exhaustive — so the `--list` diff is provably empty)

1. A test named `json_<cmd>_…` → the `<cmd>` module (JSON-output is a variant, not a separate command).
2. A test named `<cmd>_…` for `cmd ∈ {run, test, build, check, package, effects, install}` → that command's module.
3. `fmt_*` / `lint_*` / `doctor_*` / `init_*` (incl. `json_<cmd>` variants) → `misc.rs`.
4. Everything else (the 43 long-tail theme tests) → `misc.rs`.

No test straddles two modules under this rule — every test name has exactly one home. If an implementer finds a test whose primary affinity is a different command than its prefix suggests, that's settled at execution time (sub-project 1 precedent), but the prefix rule is the default and keeps the `--list` diff trivially verifiable.

**Flexibility (same caveat as sub-project 1's `exports/`):** the 4 tiny command groups (`fmt` 4 / `lint` 4 / `doctor` 3 / `init` 7) are folded into `misc.rs` rather than given 4 near-empty files. If the implementer prefers symmetry with `bin/cmd_*.rs`, dedicated `fmt.rs` / `lint.rs` / `doctor.rs` / `init.rs` modules are an acceptable execution-time choice — the command-based partition is the target shape, not a frozen file-by-file contract. Likewise, if `misc.rs` later feels rag-bag, sub-splitting it (`browser.rs`, `late.rs`, `permissions.rs`) is a lower-risk refinement.

## Visibility & test handling

- **No public API** exists for an integration-test binary → no byte-identical-surface obligation. The obligation is **zero behavior change** (count + pass/fail + scheduling scope).
- **Helpers:** all stay private in the root. **Zero `pub(crate)` widenings** in this sub-project.
- **Submodule wiring:** each `tests/runtime_smoke/<group>.rs` begins with exactly `use super::*;` — no per-module `use` block. Test bodies move **verbatim** (including `#[test]` / `#[cfg(unix)]` attributes).
- **Root `mod` decls:** explicit `#[path = "runtime_smoke/<group>.rs"] mod <group>;` (not bare `mod <group>;`) so file placement is unambiguous and doesn't rely on rustc's directory-naming resolution.
- **`build_tests.rs` / `output_tests.rs` / `init_tests.rs`:** untouched (sub-project 1's co-located lib tests, already finalized).
- **Other `tests/` monoliths** (`package_corpus.rs`, `node_api_surface.rs`, `schema_docs.rs`, `late_compat_browser_*`): untouched — each is its own future spec.
- **Cosmetic minors (do NOT run `cargo fmt`):** verbatim moves + the root's existing >100-col lines are not regressions.

## Execution & verification rhythm

**Single task** (one file, one mechanism). Move test fns by the assignment rule in size order so the biggest payoff lands first: `run` → `test` → `build` → `check` → `package` → `effects` → `install` → `misc`. Helpers and `use` block stay in root untouched. Wire root `mod` decls + `#[path]`, add `use super::*;` to each submod.

**Verification after the task:**
- `cargo build -p kali_cli --tests` (0 warnings).
- `cargo test -p kali_cli --test runtime_smoke -- --list` → **diff vs baseline = EMPTY** (same 1,816 tests, same names — provably zero behavior/test change).
- `cargo test -p kali_cli --test runtime_smoke` → same pass/fail as baseline.
- Spot-run a couple of `runtime_smoke` tests by filter name to confirm they still resolve after the move.

After the task: `cargo build -p kali_cli` 0 warnings; full `cargo test -p kali_cli` matches baseline; `--list` diff vs baseline = EMPTY.

Integration is **local-main ff-merge only — NEVER push to origin** (origin/main intentionally lags). Re-verify build + `--list` + test on merged main, then delete the branch. SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch, overwrite per sub-project).

## Out of scope (deferred)

- Sub-project 2's other test monoliths: `package_corpus.rs` (15K / 38 tests), `node_api_surface.rs` (4K / 30), `schema_docs.rs` (2.8K / 23), `late_compat_browser_*` / `browser_*` (1K–4K each). Each is its own spec.
- Sub-project 3: subdirectory grouping for the ~370 small per-behavior files under `tests/` (`array/`, `browser/`, …).
