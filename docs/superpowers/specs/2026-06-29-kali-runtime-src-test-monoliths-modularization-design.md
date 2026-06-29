# kali_runtime co-located src test-monolith modularization — design

**Series:** 27th crate-modularization entry. Third entry of the post-kali_cli frontier
(other crates' co-located src unit-test monoliths; kali_optimize was 25th, kali_types 26th).
**Date:** 2026-06-29
**Branch base / main HEAD at start:** `318279f1a`

## Goal

Split six of kali_runtime's co-located `src/*_tests.rs` unit-test monoliths into a thin facade +
per-concern `#[path] mod` submodules grouped on a **semantic axis**. **Pure verbatim
code-motion, zero behavior change**, identical compiled test set, byte-identical public API (the
crate and its consumers compile unedited).

| file | lines | `#[test]` fns | declared from | facade model |
|---|---|---|---|---|
| `src/browser/summary_tests.rs` | 2,521 | 60 | `browser/summary.rs:210` | drain to 0 |
| `src/execute_tests.rs` | 1,748 | 35 | `execute.rs:364` | drain to 0 |
| `src/browser/execute_tests.rs` | 606 | 11 | `browser/execute.rs:395` | drain to 0 |
| `src/browser/command_tests.rs` | 580 | 10 | `browser/command.rs:287` | drain to 0 |
| `src/state_tests.rs` | 532 | 13 | `state.rs:403` | drain to 0 |
| `src/profiles_tests.rs` | 312 | 12 | `profiles.rs:96` | drain to 0 |

141 `#[test]` fns total. This is **not** TDD. No new product code, no new tests, no renames, no
reformatting. **All six facades drain to 0 module-level fns** — every module-level fn in these
files is a `#[test]` (shared helpers live in `src/test_support.rs`; any nested helper travels
with its parent test body).

## Approach

The proven series recipe (28 facades split this way across kali_cli + kali_optimize + kali_types),
applied to kali_runtime's co-located src unit tests.

For each file `F`:

- **Facade** `src/.../F.rs`: keeps its original header `use` lines verbatim (`use
  crate::test_support::*;` / `use crate::*;` / `use std::fs;` / `use std::os::unix::fs::symlink;`
  exactly as each file presents them) + appended `#[path = "F/<mod>.rs"] mod <mod>;` decls.
  Contains **zero** `#[test]` fns and **zero** module-level helpers.
- **Submodules** `src/.../F/<mod>.rs`: each begins with exactly `use super::*;` (nothing else),
  followed by verbatim-moved `#[test]` fns (attribute lines + body + one trailing blank).

### Facade-drain model — all six drain to 0

Inventory of module-level non-`#[test]` fns across the six files: **none**. Every module-level
fn carries `#[test]`. The five mid-tier files (`execute_tests`, `state_tests`, `profiles_tests`,
`browser/execute_tests`, `browser/command_tests`) were each scanned: zero module-level helpers.
`summary_tests` likewise. So every facade ends with **zero** fns — the simplest drain flavor,
matching kali_optimize's fully-drained facades.

The retained header `use` lines do **not** warn as unused when consumed only through children's
`use super::*;` — Rust's descendant-visibility re-exports the facade's private `use` items
through the child glob, marking them used. This is the exact mechanism proven clean (0 new
warnings, no `#[allow]`, no import deletion) in kali_optimize and kali_types' fully-drained
facades.

### No `include_*!` gotcha here

`grep -rn 'include_str!\|include_bytes!\|include!' src/` is **0** across the whole crate —
nothing embeds a file-relative `include_*!`, so there is nothing to pin in the facade and the
mover's pin (3rd) arg is unused for this sub-project. (The simplest split flavor in the series —
no env carve-outs, no pins.)

### Wiring

- The production siblings declare each test file as
  `#[cfg(test)]` + `#[path = "F_tests.rs"]` + `mod F_tests;` (e.g. `execute.rs:362-364`,
  `browser/summary.rs:208-210`). These decls stay **unchanged** — they still name the facade
  file, which now re-exports its children.
- The facade's appended `#[path = "F/<mod>.rs"] mod <mod>;` decls resolve **relative to the
  facade file's own directory**: `src/execute_tests.rs` → `src/execute_tests/<mod>.rs`;
  `src/browser/summary_tests.rs` → `src/browser/summary_tests/<mod>.rs`, etc.
- Submodule module paths become `execute::execute_tests::<mod>::`,
  `browser::summary::summary_tests::<mod>::`, `state::state_tests::<mod>::`, and so on.
- `use super::*;` in each submodule reaches the facade's private `use` imports via Rust
  descendant-visibility — the same mechanism every prior split relied on.

## Module groupings (semantic axis)

The tables below state intent and approximate counts; the implementation plan enumerates exact
per-group membership. The decisive gate is that each file's `--list` multiset is preserved
(60 / 35 / 13 / 12 / 11 / 10).

### browser/summary_tests.rs (60) → `src/browser/summary_tests/` — `startswith` (leading prefix)

The one file with **mutually-exclusive leading prefixes**, so the mover's native `startswith`
grouping applies directly (no exact-name set needed):

| module | ~count | leading prefix |
|---|---|---|
| `runtime_summary` | ~24 | `browser_runtime_summary_*` (base summary fallback/labels/json-line behaviors) |
| `bundle` | ~14 | `browser_bundle_*` (bundle-runtime summary: thread topology, tests-failed merge, fallbacks) |
| `requested` | ~22 | `browser_requested_*` (requested-runtime summary + thread-spawn topology/reject) |

### execute_tests.rs (35) → `src/execute_tests/` — exact-name set (mid-name)

Every fn shares the `runtime_` prefix with the discriminator mid-name, so grouping is by explicit
`#[test]`-name set membership (the kali_optimize/kali_types exact-name-partition variant):

| module | ~count | members (by intent) |
|---|---|---|
| `node_imports` | ~10 | `runtime_executes_node_*` / `runtime_enforces_node_*` / `runtime_rejects_node_*` host-import suites (fs/stream/http/process/child_process/util/event_emitter/path/url/crypto/os + budget enforcement) |
| `host_env` | ~12 | console/arguments/exit-code/env-var get/set/delete/presence, cwd, file writes, http fetch + mocked failure, math-pow guard |
| `timers` | ~6 | microtask drain order, repeating/clearable intervals, trap reporting, clear scheduled timers, negative timer/interval delay rejection |
| `crypto_random` | ~3 | `performance.now`, random fill, crypto random UUID |
| `test_runner` | ~2 | registered-test collection + failed-registered-test reporting |
| `misc` | ~2 | anything not captured above (e.g. console-policy denial) — catch-all, empty groups auto-skip |

### state_tests.rs (13) → `src/state_tests/` — exact-name set

| module | ~count | members (by intent) |
|---|---|---|
| `host_state` | ~7 | `runtime_host_state_*` budget bookkeeping, spawn/release, rollback, whitespace trimming, profile acceptance, whitespace-only rejection |
| `summary_parser` | ~2 | `runtime_summary_parser_rejects_*` (whitespace-padded / relative thread script URLs) |
| `thread_exec` | ~4 | thread-topology snapshot reporting, thread-spawn host-import execute/reject (budget zero / exhausted), execute-tests topology |

### profiles_tests.rs (12) → `src/profiles_tests/` — exact-name set

| module | ~count | members (by intent) |
|---|---|---|
| `browser_surface` | ~3 | browser host-contract reporting + browser-api-surface rejection (run + test-execution) |
| `thread_budget` | ~3 | threaded-runtime-profile request acceptance, positive-thread-budget accept/reject vs threaded profile |
| `normalization` | ~6 | outcome/test-outcome carry profiles, execute/execute-tests normalize from public-field mutation, canonical-profile exposure, `normalize_runtime_profiles_is_shared_between_callers` |

### browser/execute_tests.rs (11) → `src/browser/execute_tests/` — exact-name set

| module | ~count | members (by intent) |
|---|---|---|
| `execution` | ~6 | `browser_requested_*` execute/registered-callbacks, `browser_runtime_execution_helper_*` html-entrypoint/launch+summary, `browser_bundle_runtime_execute_checked_*` bundle exports/html-entrypoint |
| `harness` | ~4 | `browser_harness_invocation_checked_*` launch-plan/file-url, `browser_harness_run_checked_*` command capture, `browser_harness_launch_failure_*` resolved-command preservation |
| `diagnostic` | ~1 | `browser_runtime_unavailable_diagnostic_formats_command_context` |

### browser/command_tests.rs (10) → `src/browser/command_tests/` — exact-name set

| module | ~count | members (by intent) |
|---|---|---|
| `command_parts` | ~5 | `browser_harness_command_parts_*` override/default selection, malformed-override reporting, whitespace trim (+quote preservation), headless-mode for browser executables |
| `split_command` | ~2 | `split_command_spec_supports_shell_like_quoting` / `split_command_spec_rejects_malformed_inputs` |
| `harness_misc` | ~3 | `browser_harness_invocation_checked_preserves_html_entrypoint_file_urls_*`, `browser_harness_launch_failure_reports_*`, `browser_harness_recognizes_all_canonical_browser_executable_names` |

> Final per-module counts are whatever the mover's `--list` baseline diff proves; the tables
> state intent. The decisive gate is that each file's `--list` multiset is preserved.

## Tooling

`.superpowers/sdd/move_fns.py` + `.superpowers/sdd/verify.py` (git-ignored scratch; re-created
from the documented design). **Keep `FN_RE` / `IDENT_CHARS` / `find_close_line` byte-identical**
— the string/comment/raw-string-aware brace lexer is required (these files contain `r#"..."#`
JS/TS templates with `}` at column 0; a naive column-0 close-brace scan breaks). Filter by the
`#[test]` **attribute**, never name prefix alone.

**Two grouping modes in one mover:**

- `summary_tests` uses the native **leading-prefix `startswith`** grouping
  (`name=browser_runtime_summary_;bundle=browser_bundle_;requested=browser_requested_`) — its
  three discriminators are mutually-exclusive leading prefixes.
- The other five use the **exact-name partition** (kali_optimize variant): because the
  discriminator is mid-name, group assignment is exact `#[test]`-name set membership (equality),
  not leading-prefix. Each group is an explicit set of full fn names.

Either way this touches only the GROUPS parsing / assignment in `main()`; `FN_RE` / `IDENT_CHARS`
/ `find_close_line` stay byte-identical. The mover writes `src/<...>/<stem>/<mod>.rs` (each `use
super::*;` + verbatim fns) and rewrites the facade to drop moved fns + append `#[path] mod`
decls. The pin (3rd) arg exists but is unused (no `include_*!`).

`verify.py` (`python3 verify.py <orig_rs> "<submodule_glob>"`) reuses the same lexer to prove
`{name: body}` from the original == from the submodules, exiting non-zero on any
name-set/body mismatch — the decisive byte-identity gate. No facade pins, so no facade glob.

## Verification gates (this sandbox)

Baseline captured on the clean base (`318279f1a`): `cargo build -p kali_runtime --tests` =
**0 warnings**; `cargo test -p kali_runtime --lib` = **158 pass / 0 fail**. The literal series
"0 warnings / fully green" gates hold here — no env-failure carve-outs (unlike kali_cli's
chromium-sandbox). The **operative gates remain no-new-warnings + pass/fail unchanged** against
this baseline; the plan re-confirms the numbers at Task 1.

- **G1 — facade drained:** `grep -c '#\[test\]' src/.../F.rs` == 0 for all 6 files; each facade
  ends with one `#[path] mod` decl per non-empty group, retains exactly its original header
  `use` lines (no `#[allow]`, no import deletion), and zero module-level fns.
- **G2 — submodule headers:** each `src/.../F/<mod>.rs` begins with exactly `use super::*;`.
- **G3 — no new warnings:** `cargo build -p kali_runtime --tests 2>&1 | grep -c '^warning'`
  stays == the captured baseline.
- **G4 — test-set identical (per file):** the lib-test `--list` basename multiset for the tests
  under `F` is unchanged before/after, via `cargo test -p kali_runtime --lib -- --list` filtered
  to the `F`-rooted module path, new `<mod>::` segment stripped (`s/^.*:://`), `sort` without
  `-u` (multiset), `diff` against the pre-split baseline → empty. Expected sizes: 60 / 35 / 13 /
  12 / 11 / 10.
- **G5 — runtime pass/fail unchanged:** `cargo test -p kali_runtime --lib` pass/fail name-set
  identical before/after (strip new module prefix; shifted-but-unchanged panic messages are not
  regressions — code-motion moves line numbers, the message is the invariant). Expected total
  unchanged at **158 pass / 0 fail**.
- **G6 — byte-identity:** `verify.py` proves every moved `#[test]` body byte-identical
  base→submodules for all 6 files.

> G4's exact `--list` filter is validated against real `cargo test --lib -- --list` output at
> plan Task 1 (baseline capture) before any move; the principle (per-file multiset preserved)
> is fixed.

## Constraints (verbatim-binding)

- Pure relocation. No new product code, no new tests, no renames, no reordering, no tidy.
- Verbatim moves only — `#[test]` attr lines + body + one trailing blank relocate
  byte-for-byte.
- Submodule header is exactly `use super::*;`. Facade keeps every original `use`. No
  per-submodule extern `use`s.
- Facade ends with **zero** `#[test]` fns and zero module-level helpers (no `include_*!` pins
  needed here).
- No `pub`/`pub(crate)` widening (intra-crate child modules reach parent scope via
  `use super::*`; no visibility change needed).
- Do **not** run `cargo fmt` (repo fmt gate already red on baseline; accepted cosmetic minors
  are not regressions).
- Integration: **local-main ff-merge only — NEVER push origin** (origin/main intentionally
  lags). Re-verify on merged main, then delete the branch.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per task;
  durable recovery map.

## Out of scope

- kali_runtime's three sub-threshold co-located test files left as-is this sub-project:
  `src/browser/harness_tests.rs` (401 lines, 9 tests), `src/ctx_tests.rs` (213 lines, 6 tests),
  `src/browser/contract_tests.rs` (229 lines, 2 tests) — below the chosen ≥10-test scope line.
- Other crates' co-located src test monoliths (kali_codegen, …) — future series entries, not
  this sub-project.

## Branch & sequencing

- Branch `refactor/kali_runtime-modularization` off `318279f1a`; baseline build+test captured
  (warning count + per-file `--list` multiset + pass/fail count) before starting.
- Execute via superpowers:subagent-driven-development: implementer (sonnet) → review-package →
  task reviewer (sonnet; opus for finalize/whole-branch review).
- Six files = six task-groups (largest→smallest: summary, execute, browser/execute,
  browser/command, state, profiles), each split per the recipe, committed separately. Final opus
  whole-branch review proves all 141 `#[test]` bodies byte-identical base→head.
