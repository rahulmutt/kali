# kali_cli co-located src test-monolith modularization — design

**Series:** 24th crate-modularization entry. kali_cli sub-project 4 of 4 (final).
**Date:** 2026-06-29
**Branch base / main HEAD at start:** `77704c7e7`

## Goal

Split kali_cli's two remaining co-located `src/*_tests.rs` unit-test monoliths —
the two largest files in the workspace — into a thin facade + per-concern `#[path] mod`
submodules. **Pure verbatim code-motion, zero behavior change**, identical compiled
test set. After this, kali_cli has no source file over ~2k lines and is fully modularized.

| file | lines | `#[test]` fns | declared from |
|---|---|---|---|
| `src/build_tests.rs` | 15,535 | 716 | `src/build/mod.rs:57` via `#[path="../build_tests.rs"] #[cfg(test)] mod tests;` |
| `src/output_tests.rs` | 8,509 | 229 | `src/lib.rs:559` via `#[path="output_tests.rs"] #[cfg(test)] mod output_tests;` |

This is **not** TDD. No new product code, no new tests, no renames, no reformatting.

## Approach

The proven series recipe (30 files split this way across kali_cli sub-projects 2 & 3),
applied to co-located src unit tests rather than `tests/` integration binaries.

For each file `F` (`build_tests` / `output_tests`):

- **Facade** `src/F.rs`: keeps **every** original `use` line + **all** non-`#[test]`
  helper fns (e.g. `assert_*`, `collect_*` raw-source builders, `assert_payload_*`)
  + appended `#[path = "F/<mod>.rs"] mod <mod>;` decls. Contains **zero** `#[test]` fns.
- **Submodules** `src/F/<mod>.rs`: each begins with exactly `use super::*;` (nothing else),
  followed by verbatim-moved `#[test]` fns (attribute lines + body + one trailing blank).

### Wiring

- `#[path]` decls and submodule files resolve **relative to the facade file's own
  directory** (`src/`), independent of where the `mod` decl that includes the facade
  lives. So `src/build_tests.rs` → `src/build_tests/<mod>.rs`, even though build_tests
  is declared from `src/build/mod.rs`.
- `use super::*;` in each submodule reaches the facade's **private** `use` imports via
  Rust descendant-visibility (private items are visible to child modules). This is the
  exact mechanism the integration split relied on; no per-submodule extern `use`s.
- **build_tests `super` chain:** build_tests' own `use super::*` resolves `super` = the
  `build` module, which carries the `#[cfg(test)]` "cutoff re-exports" at
  `src/build/mod.rs:37,47`. Those stay **untouched**; submodules reach them transitively
  through the facade's `use super::*`. (output_tests uses absolute `use crate::output::{…}`
  paths, so it has no `super`-chain subtlety.)
- The `mod` decls at `src/build/mod.rs:57` and `src/lib.rs:559` stay unchanged — they
  still point at the facade file, which now re-exports its children.

### File-relative `include_*!` gotcha (empirically verified)

`include_str!` / `include_bytes!` / `include!` resolve paths **relative to the source
file containing the macro**. Moving such a fn into `src/<stem>/<mod>.rs` adds one
directory level, so a file-relative path resolves one level short and fails to compile.
Rewriting the path would violate the verbatim mandate. **Resolution: pin any `#[test]`
fn whose body contains a file-relative `include_*!` to the facade** (the mover's 3rd
arg) — it stays in `src/<stem>.rs` at the original depth, verbatim and compiling.

- `build_tests.rs`: **no** `include_*!` macros → facade fully drains to 0 `#[test]`.
- `output_tests.rs`: exactly **two** `#[test]` fns embed
  `include_str!("../../../schemas/{envelope,diagnostic}/v1.json")` —
  `published_cli_envelope_schema_matches_fixed_shape_validator_posture` and
  `published_diagnostic_schema_matches_fixed_shape_validator_posture`. Both are pinned;
  the facade retains **exactly these two** `#[test]` fns.

This was verified end-to-end: applying the split to both files, `cargo build -p kali_cli
--tests` compiled clean (warnings == baseline 2), and the lib `--list` multiset was
preserved at 716 / 229.

## Module groupings (Balanced, ~10 + ~8)

Grouping is the mover's only partition axis: leading-prefix of the `#[test]` fn name,
**first match wins in spec order** — so specific prefixes must precede general ones.
`*` = catch-all (last). Empty groups auto-skip. Helpers (non-`#[test]`) never move.

### build_tests.rs (716 → ~10 modules), semantic axis

All tests are "build", so the dominant `build_source_file_*` positive/negative clusters
are subdivided. supports = positive cases, rejects = negative cases.

| module | leading prefix(es) | ~count |
|---|---|---|
| `supports_math` | `build_source_file_supports_math` | 118 |
| `supports_for` | `build_source_file_supports_for` | 121 |
| `supports_misc` | `build_source_file_supports` (remaining) | 121 |
| `rejects` | `build_source_file_rejects` | 142 |
| `check` | `check_source` | 43 |
| `collect` | `collect` | 53 |
| `validate` | `validate` | 46 |
| `runtime` | `runtime_entrypoint` | 26 |
| `discover` | `discover_dynamic` | 16 |
| `misc` | `*` (compile/incremental/build_artifact/build_browser/writes/output/load/component/capi/…) | ~30 |

Spec order: `supports_math` and `supports_for` **before** `supports_misc` so the
specific clusters bind first; `build_source_file_writes` and the `build_artifact`/
`build_browser` stragglers fall through to `misc`.

### output_tests.rs (229 → ~8 modules), payload/validator-kind axis

| module | leading prefix(es) | ~count |
|---|---|---|
| `envelope` | `validate_envelope`, `emit_envelope` | 67 |
| `doctor` | `validate_doctor` | 59 |
| `package` | `validate_package` | 22 |
| `run` | `validate_run` | 13 |
| `effects` | `validate_effects` | 13 |
| `test` | `validate_test` | 11 |
| `payloads_misc` | `validate_install`, `validate_init`, `validate_lint`, `validate_fmt`, `validate_check` | 18 |
| `emit` | `*` (emitted_cli/ordinary_cli/merge_thread/diagnostic_json) | 24 |

The 2 `published_*_schema_*` fns that would land in `emit` are pinned to the facade
(see the `include_*!` gotcha above), so `emit` holds 24, not 26; facade retains 2.

Final per-module counts are whatever the mover's `--list` baseline diff proves; the
tables above are the intent. Counts that shift between adjacent groups due to the
leading-prefix rule are acceptable as long as the per-file `--list` multiset is preserved.

## Tooling

`.superpowers/sdd/move_fns.py` (git-ignored scratch; cleaned between sessions) is
re-created from the documented design. **Keep `FN_RE` / `IDENT_CHARS` /
`find_close_line` byte-identical** — the string/comment/raw-string-aware brace lexer is
required (these files contain `r#"..."#` JS/TS templates with `}` at column 0; a naive
column-0 close-brace scan breaks). Filter by the `#[test]` **attribute**, never name
prefix alone (helpers like `collect_library_*`/`assert_*` start with grouped prefixes
but lack `#[test]` and must stay in the facade).

**One generalization for this sub-project:** the mover writes submodules **relative to
the input file's own directory** (`src/build_tests/<mod>.rs`), not hardcoded under
`tests/`. Only the path-derivation, docstring, ROOT/GROUPS/main() change; the lexer is
untouched.

CLI: `python3 move_fns.py <root_rs_relpath> "<groups-spec>" ["<pin1,pin2>"]` run from
`crates/kali_cli`. The optional 3rd arg is a comma-separated list of `#[test]` fn names
to pin in the facade (for the `include_*!` gotcha above).

## Verification gates (this sandbox)

Literal "0 warnings / fully green" does **not** hold here; use the corrected gates
established in sub-projects 2 & 3:

- **G1 — facade drained:** `grep -c '#\[test\]' src/F.rs` == 0 for `build_tests`, == 2
  for `output_tests` (the two pinned `published_*_schema_*` fns); facade ends with one
  `#[path] mod` decl per non-empty group.
- **G2 — submodule headers:** each `src/F/<mod>.rs` begins with exactly `use super::*;`.
- **G3 — no-new-warnings:** `cargo build -p kali_cli --tests 2>&1 | grep -c '^warning'`
  stays == baseline (**2** — a grep artifact for the single pre-existing
  `build/mod.rs:40 profile_data_hash unused_imports` lib-test warning plus cargo's
  "generated 1 warning" summary; only one real warning, untouched by this refactor).
- **G4 — test-set identical (per file):** the lib-test `--list` basename multiset for
  the tests under `F` is unchanged before/after. Because these are **lib** unit tests
  (not separate `--test` binaries), the list is captured via
  `cargo test -p kali_cli --lib -- --list`, filtered to the `F`-rooted module path,
  with the new `<mod>::` segment stripped (`s/^.*:://`), `sort` without `-u` (multiset),
  `diff` against the pre-split baseline → empty. Expected sizes: build_tests 716,
  output_tests 229.
- **G5 — runtime pass/fail unchanged:** lib-test run pass/fail name-set identical
  before/after (strip new module prefix; expect the same pre-existing sandbox
  env-failures, if any reach the lib test, with shifted-but-unchanged panic messages).

> Note: G4's exact `--list` filter for lib unit tests is validated against real output
> during plan Task 1 (baseline capture) before any move; if the lib `--list` path
> grouping differs from the `--test` binaries used in sub-project 3, the filter is
> adjusted there. The principle (per-file multiset preserved) is fixed.

## Constraints (verbatim-binding)

- Pure relocation. No new product code, no new tests, no renames, no reordering, no tidy.
- Verbatim moves only — `#[test]` attr lines + body + one trailing blank relocate
  byte-for-byte.
- Submodule header is exactly `use super::*;`. Facade keeps every original `use`. No
  per-submodule extern `use`s.
- Facade ends with **zero** `#[test]` fns, **except** fns pinned for the `include_*!`
  gotcha (output_tests keeps exactly 2). Non-`#[test]` helpers (incl. `#[cfg]`-gated
  helpers lacking `#[test]`) always stay in the facade.
- No `pub`/`pub(crate)` widening (these are intra-crate child modules reaching parent
  scope via `use super::*`; no visibility change needed).
- Do **not** run `cargo fmt` (repo fmt gate already red on baseline; accepted cosmetic
  minors are not regressions).
- Integration: **local-main ff-merge only — NEVER push origin** (origin/main
  intentionally lags). Re-verify on merged main, then delete the branch.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per
  task; durable recovery map.

## Out of scope

- The small co-located test files (`src/tests.rs`, `src/build/tests.rs`,
  `src/init` tests, etc., all < 1000 lines) — below the series threshold; left as-is.
- Any other crate's co-located src test monoliths (kali_optimize, kali_types,
  kali_runtime, kali_codegen, …) — future series entries, not this sub-project.

## Branch & sequencing

- Branch `refactor/kali-cli-src-test-monoliths` off `77704c7e7`; baseline build+test
  captured green-relative-to-baseline before starting.
- Execute via superpowers:subagent-driven-development: implementer (sonnet) →
  review-package → task reviewer (sonnet; opus for finalize/whole-branch review).
- Two files = two task-groups (build_tests, then output_tests), each split per the
  R1–R6 recipe, committed separately. Final opus whole-branch review proves all 945
  `#[test]` bodies byte-identical base→head.
