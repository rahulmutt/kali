# kali_cli sub-project 3 — remaining test-monolith modularization (23rd in series)

**Date:** 2026-06-29
**Crate:** `kali_cli` (sub-project 3 of 3)
**Status:** design approved, pending spec review

## Context

This is the 23rd entry in the long-running kali workspace crate-modularization
series. `kali_cli` is a three-sub-project crate:

- **Sub-project 1** (21st) — production `src/` split. DONE (`5b6c7ff82`).
- **Sub-project 2** (22nd) — `tests/runtime_smoke.rs` monolith split into a
  facade + per-command `#[path] mod` submodules. DONE (`c32905256`).
- **Sub-project 3** (this doc) — split the remaining large integration-test
  monoliths in `tests/`.

The `tests/` root holds 249 auto-discovered integration-test files (each its own
test binary; no `[[test]]` stanzas in `Cargo.toml`). Of these, 16 files are
≥1000 lines. One — `runtime_smoke.rs` (now 6,416 lines, 0 `#[test]`) — is
already the sub-project-2 facade and is excluded. The remaining **15 files** are
the scope of this sub-project.

Small-file subdir grouping / binary consolidation is **explicitly out of scope**
(it is a different kind of change — binary-boundary semantics — and is deferred).

## Goal

Decompose each of the 15 monolithic test files into a thin facade + per-concern
`#[path] mod` submodules with **zero behavior change** and a byte-identical test
name-set (modulo the new module prefix). Pure code-motion + `mod`/`use` wiring;
`#[test]` fn bodies moved verbatim.

## Architecture & method

Carried verbatim from sub-project 2. Each monolith `tests/F.rs` becomes:

### Facade — `tests/F.rs`

Keeps all top-level `use` imports and every non-`#[test]` helper fn, and declares
its submodules:

```rust
#[path = "F/build.rs"] mod build;
#[path = "F/check.rs"] mod check;
// ...one line per module...
```

The root file ends with **zero `#[test]` fns**. Helpers stay private in root —
`#[path] mod` children are descendants of the root module, so they can see the
root's private items. This is the "zero `pub(crate)`" property that held for
runtime_smoke: no visibility widening is required for submodules to call root
helpers.

### Submodules — `tests/F/<mod>.rs`

Each holds the verbatim-moved `#[test]` fns for one concern, prefixed with:

- `use super::*;` — to reach the root's private helper fns (items visible to
  descendants).
- Each module's own extern `use`s (e.g. `serde_json::{json, Value}`,
  `tempfile::tempdir`) — because `use super::*` only re-exports the root's
  **public** items, and the root's `use`-imports are private. Each submodule
  re-declares exactly the externs its moved fns reference, matching the
  runtime_smoke submodule precedent.

### Binary topology

One test binary per source file is preserved (cargo auto-discovers by file
stem; the `#[path] mod` submodules compile into that single binary). No
`Cargo.toml` changes.

### Tooling

Extraction uses the existing `.superpowers/sdd/move_fns.py` (git-ignored
scratch, from sub-project 2) **unchanged** — a deterministic, byte-faithful
`#[test]`-fn extractor with a string/comment-aware brace-counting
`find_close_line` lexer. Filter by the `#[test]` *attribute*, never name prefix
alone (cfg-gated helpers can share a verb prefix but lack `#[test]` and must stay
in root). The naive column-0 `}` close-brace scan must NOT be reintroduced —
these files contain raw-string `r#"..."#` JS/TS templates with `}` at column 0.

## Grouping rule & per-file scheme

**Rule.** Primary axis = command verb (`build` / `check` / `run` / `test` /
`package`). Each `json_<cmd>` output-variant test folds into its base-command
module. When a *single* command dominates (≥~90% of the file's tests), sub-split
by output mode instead (`<cmd>` plain vs `<cmd>_json`). Files with no command
axis use their dominant semantic prefix. Command-less stragglers collect in a
`misc` module.

| File | tests | modules |
|---|---|---|
| `package_corpus.rs` | 206 | run, check, package, build, test |
| `late_compat_browser_js_input.rs` | 118 | run, build, check, test, misc |
| `browser_non_literal_iterator_sources.rs` | 90 | build, check, misc |
| `node_api_surface.rs` | 45 | *(semantic)* core, explicit, inherited, process |
| `browser_reflect_own_keys.rs` | 44 | run, test, build, check |
| `browser_for_await_frozen_set_map_constructor_result.rs` | 40 | run, test, build |
| `browser_runtime_summary_fallback_js_input.rs` | 34 | run, test |
| `browser_math_atan2_bracketed_root.rs` | 29 | build, run |
| `browser_runtime_summary_fallback_tsx_input.rs` | 28 | run, test |
| `browser_runtime_summary_fallback_jsx_input.rs` | 28 | run, test |
| `browser_runtime_summary_fallback_ts_input.rs` | 27 | run, test |
| `browser_object_keys_iteration.rs` | 25 | build, build_json *(single-command → output-mode split)* |
| `late_compat_browser_tsx_input.rs` | 22 | run, build, check, test, misc |
| `schema_docs.rs` | 22 | *(semantic)* plan, proof, misc |
| `late_compat_browser_jsx_input.rs` | 18 | run, build, check, misc |

The exact `#[test]` fn → module assignment is deterministic from the rule plus
`move_fns.py` output. The **implementation plan** enumerates the assignment per
file; the spec fixes the method and the module set.

## Sequencing — one plan, 4 task-groups by size

- **TG1** — `package_corpus.rs` (the 14,972-line giant; isolated).
- **TG2** — `late_compat_browser_js_input.rs`, `node_api_surface.rs`,
  `schema_docs.rs`.
- **TG3** — `browser_non_literal_iterator_sources.rs`,
  `browser_reflect_own_keys.rs`,
  `browser_for_await_frozen_set_map_constructor_result.rs`,
  `browser_runtime_summary_fallback_js_input.rs`.
- **TG4** — the 7 remaining smaller files:
  `late_compat_browser_tsx_input.rs`, `late_compat_browser_jsx_input.rs`,
  `browser_runtime_summary_fallback_tsx_input.rs`,
  `browser_runtime_summary_fallback_jsx_input.rs`,
  `browser_runtime_summary_fallback_ts_input.rs`,
  `browser_object_keys_iteration.rs`, `browser_math_atan2_bracketed_root.rs`.

Per-task flow follows the series: implementer (sonnet) → review-package → task
reviewer (sonnet; opus for the finalize / whole-branch review). The work is
mechanical, so reviews are fast.

## Verification gates (corrected for this sandbox)

The plans' literal "0 warnings / fully green baseline" gates do NOT hold in this
sandbox. Use the human-approved corrected gates established in sub-project 2:

- **no-new-warnings** — the warning count stays at the baseline number (the 1
  pre-existing `build/mod.rs:40 profile_data_hash unused_imports` lib-test
  warning from sub-project 1); no NEW warnings introduced.
- **pass/fail-unchanged** — the same test *names* pass/fail before and after.
  There are 143 pre-existing chromium-sandbox browser-bundle env failures
  (`No usable sandbox! ... install the chromium-sandbox package`) on clean main
  (1673 pass / 143 fail across the suite). Diff the per-file fail-set; expect
  empty. For `#[path] mod` moves, strip the new module prefix
  (`sed 's/^.*:://'`, turning `build::foo` → `foo`) and the libtest ` ... FAILED`
  suffix before diffing. A shifted panic-site line number is expected
  (code-motion moves fn bodies) — the panic *message* is unchanged.
- **`--list` name-set identical** — `cargo test --test F -- --list` produces the
  same test names modulo the new `<mod>::` prefix.
- **Do NOT run `cargo fmt`** — the verbatim-move mandate forbids it, and the
  repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates),
  so accepted cosmetic minors (>100-col signatures, stray blank lines) are not
  regressions.

## Risks / edge cases

- **Single-command files** (`browser_object_keys_iteration.rs`) split by output
  mode (`build` / `build_json`), not by command, since one command verb covers
  ~all tests.
- **`use super::*` extern cutoff** — submodules must re-declare the externs their
  moved fns reference; `use super::*` does not bring the root's private
  `use`-imports into scope. Same handling as runtime_smoke submodules.
- **Raw-string `}` at column 0** — already handled by `move_fns.py`'s
  string/comment-aware lexer; must not regress to the naive brace scan.
- **`array_from_bracketed_root_wrappers`** — has 2 pre-existing
  `build_bundles_*` failures (codegen/bundling, unrelated). Confirm reproduction
  on the branch base; do not attribute to the refactor. (Not in this batch's
  file set, but watch if cross-binary effects appear.)

## Integration policy (series convention)

- Work on a `refactor/kali-cli-test-monoliths` branch off main; baseline
  build+test captured green-modulo-known-failures before starting.
- Integration is **local-main ff-merge only — NEVER push to origin** (origin/main
  intentionally lags). Re-verify on merged main, then delete the branch.
- SDD ledger lives at `.superpowers/sdd/progress.md` (git-ignored scratch) —
  overwrite for this sub-project; it is the durable recovery map.
