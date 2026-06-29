# kali_optimize co-located src test-monolith modularization — design

**Series:** 25th crate-modularization entry. First entry of the post-kali_cli frontier
(other crates' co-located src unit-test monoliths).
**Date:** 2026-06-29
**Branch base / main HEAD at start:** `fe2878947`

## Goal

Split kali_optimize's two co-located `src/*_tests.rs` unit-test monoliths ≥1000 lines into
a thin facade + per-concern `#[path] mod` submodules grouped on a **semantic axis**. **Pure
verbatim code-motion, zero behavior change**, identical compiled test set, byte-identical
public API (the crate and its consumers compile unedited).

| file | lines | `#[test]` fns | declared from |
|---|---|---|---|
| `src/specialize_tests.rs` | 5,344 | 37 | `src/specialize.rs:832` via `mod specialize_tests;` |
| `src/object_fold_tests.rs` | 1,654 | 48 | `src/object_fold.rs:379` via `mod object_fold_tests;` |

This is **not** TDD. No new product code, no new tests, no renames, no reformatting.

## Approach

The proven series recipe (18 facades split this way across the kali_cli sub-projects),
applied to kali_optimize's co-located src unit tests.

For each file `F` (`specialize_tests` / `object_fold_tests`):

- **Facade** `src/F.rs`: keeps its **three** original `use` lines verbatim
  (`use crate::test_support::*;` / `use crate::*;` / `use kali_lir::{LirBuilder, LirNodeKind};`)
  + appended `#[path = "F/<mod>.rs"] mod <mod>;` decls. Contains **zero** `#[test]` fns.
- **Submodules** `src/F/<mod>.rs`: each begins with exactly `use super::*;` (nothing else),
  followed by verbatim-moved `#[test]` fns (attribute lines + body + one trailing blank).

### Both facades drain *fully* (zero retained fns) — verified safe

Unlike every kali_cli facade (which retained module-level `assert_*` / builder helper fns
that consumed the imports), **both kali_optimize files contain only `#[test]` fns at module
level** — all shared helpers (`literal`, `build_object_enumeration_call`, …) live in
`src/test_support.rs` and reach the tests via `use crate::test_support::*;`; the
`append_literal_chain` / `build_object` helpers are **nested inside** individual test bodies
and move with their parent test. So each facade drains to **0 `#[test]` and 0 fns** — only
the 3 `use` lines + `#[path] mod` decls remain.

This raised the question: do the facade's retained `use` lines warn as unused when consumed
*only* through children's `use super::*;`? **Empirically verified no** — a throwaway crate
under `#![deny(warnings)]` with a zero-fn facade (`use crate::*;` glob **and** a named
`use crate::thing::{Builder, Kind};`) consumed solely via a `#[path]` child's `use super::*;`
compiles clean. Rust's descendant-visibility re-exports the facade's private `use` items
through the child glob, marking them used. No `#[allow]`, no `#[cfg(test)]` re-export, no
import deletion needed.

### No `include_*!` gotcha here

`grep -c 'include_str!\|include_bytes!\|include!'` is **0** for both files — neither embeds a
file-relative `include_*!`, so there is nothing to pin in the facade and the mover's pin
(3rd) arg is unused for this sub-project.

### Wiring

- `#[path]` decls resolve **relative to the facade file's own directory** (`src/`): so
  `src/specialize_tests.rs` → `src/specialize_tests/<mod>.rs`,
  `src/object_fold_tests.rs` → `src/object_fold_tests/<mod>.rs`.
- The `mod specialize_tests;` / `mod object_fold_tests;` decls at `src/specialize.rs:832`
  and `src/object_fold.rs:379` stay **unchanged** — they still name the facade file, which
  now re-exports its children.
- `use super::*;` in each submodule reaches the facade's private `use` imports via Rust
  descendant-visibility — the same mechanism every prior split relied on.

## Module groupings (semantic axis)

The semantic token sits **mid-name** (every fn starts `release_` / `release_advanced_` /
`fast_`), so grouping is by **explicit `#[test]`-name set membership**, not leading-prefix.
Each group below lists its exact member fns; all 37 + 48 are assigned with no leftovers.

### specialize_tests.rs (37) → `src/specialize_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `mir_layout` | 4 | core specialization via MIR layouts, recursion, scopes, literal-shaped call sites |
| `tagged_budget` | 5 | tagged-parameter + budget/limit + concrete-argument specialization |
| `generic_reuse` | 8 | generic-specialization reuse across owners / reexport chains / budget exhaustion |
| `literal_args` | 14 | specializing by literal-argument kind (array/string/regex/nullish/bool/numeric/bigint) |
| `layout_bindings` | 6 | closure / struct / array / object-literal layout-binding specialization |

Exact assignment:

- **mir_layout** (4): `release_specializes_large_function_using_mir_layouts`,
  `release_recursively_specializes_nested_mir_call_sites`,
  `release_specializes_same_binding_name_in_distinct_function_scopes`,
  `release_specializes_literal_shaped_mir_call_sites_without_layout_metadata`
- **tagged_budget** (5): `release_specializes_tagged_parameters_from_concrete_arguments`,
  `release_respects_zero_specialization_budget_for_tagged_parameters`,
  `release_advanced_limits_specialization_to_one_distinct_call_site_after_root_inlining`,
  `release_specializes_tagged_parameters_for_non_inlined_functions`,
  `release_specializes_concrete_arguments_without_mir_layouts`
- **generic_reuse** (8): `release_allows_generic_specialization_inside_mir_specialized_clones`,
  `release_advanced_allows_generic_specialization_inside_mir_specialized_clones`,
  `release_reuses_generic_specializations_across_layout_specialized_owners`,
  `release_advanced_reuses_generic_specializations_across_layout_specialized_owners`,
  `release_specializes_identical_generic_call_sites_across_owners_once`,
  `release_reuses_generic_specializations_across_reexport_chain`,
  `release_advanced_partially_specializes_reexport_chain`,
  `release_reuses_existing_mir_specializations_after_an_owner_spends_its_budget`
- **literal_args** (14): `release_specializes_array_literal_arguments_by_shape`,
  `release_specializes_string_literal_arguments`,
  `release_specializes_quoted_string_and_template_literal_arguments_distinctly`,
  `release_specializes_regex_literal_arguments`,
  `release_specializes_regex_literal_arguments_with_mir_layouts`,
  `release_specializes_nullish_literal_arguments`,
  `release_advanced_specializes_nullish_literal_arguments`,
  `fast_keeps_nullish_literal_arguments_unspecialized`,
  `release_specializes_infinity_and_nan_literal_arguments`,
  `release_specializes_boolean_literal_arguments`,
  `release_specializes_numeric_literal_arguments`,
  `release_specializes_negative_zero_literal_arguments`,
  `release_specializes_bigint_literal_arguments`,
  `release_advanced_specializes_bigint_literal_arguments`
- **layout_bindings** (6): `release_specializes_shared_closure_layout_bindings`,
  `release_specializes_distinct_closure_capture_bindings`,
  `release_specializes_nested_mir_bound_bindings_inside_object_literals`,
  `release_specializes_shared_struct_layout_bindings`,
  `release_specializes_distinct_struct_layout_bindings`,
  `release_specializes_distinct_array_layout_bindings`

### object_fold_tests.rs (48) → `src/object_fold_tests/`

Grouped by **primary fold operation**; const-bound / alias-chain / frozen variants stay with
their operation rather than forming a cross-cutting group (keeps assignment unambiguous).

| module | ~count | members (by intent) |
|---|---|---|
| `enumeration` | 20 | `Object.keys/entries/values/fromEntries` + generic object-enumeration folds (incl. const/frozen/alias variants) |
| `reflect_own_keys` | 16 | `Reflect.ownKeys` folds (plain/bracketed/mixed/globalThis/frozen/const/alias variants) |
| `object_has_own` | 12 | `Object.hasOwn` folds (optional-chain/frozen/from-entries/callable-wrapper/const variants) |

Exact assignment — **enumeration** (20): the 12 `*object_keys/entries/values/from_entries*`
and `*object_enumeration*` fns at original lines 6–651, plus
`fast_folds_object_enumeration_calls_over_literal_object_shapes`,
`release_folds_object_enumeration_calls_over_const_bound_literal_object_shapes`,
`release_folds_object_enumeration_calls_over_wrapped_const_bound_literal_object_shapes`,
`release_folds_object_enumeration_calls_over_const_alias_chains`,
`release_advanced_folds_object_enumeration_calls_over_const_alias_chains`,
`release_advanced_folds_object_enumeration_calls_over_const_bound_literal_object_shapes`,
`release_advanced_folds_object_enumeration_calls_over_frozen_literal_object_shapes`,
`release_advanced_folds_object_enumeration_calls_over_literal_object_shapes`.
**reflect_own_keys** (16): all 13 `*reflect_own_keys*` / `*reflect_bracketed_own_keys*` fns at
original lines 651–826, plus
`release_advanced_folds_reflect_own_keys_calls_over_const_bound_literal_object_shapes`,
`release_folds_reflect_own_keys_calls_over_const_alias_chains`,
`release_advanced_folds_reflect_own_keys_calls_over_const_alias_chains`.
**object_has_own** (12): all 12 `*object_has_own*` fns at original lines 872–1179 (incl.
`release_folds_object_has_own_calls_over_const_bound_literal_object_shapes`).

Final per-module counts are whatever the mover's `--list` baseline diff proves; the tables
state intent. The decisive gate is that the per-file `--list` multiset is preserved (37, 48).

## Tooling

`.superpowers/sdd/move_fns.py` + `.superpowers/sdd/verify.py` (git-ignored scratch; re-created
from the documented design). **Keep `FN_RE` / `IDENT_CHARS` / `find_close_line` byte-identical**
— the string/comment/raw-string-aware brace lexer is required (these files contain
`r#"..."#` JS/TS templates with `}` at column 0; a naive column-0 close-brace scan breaks).
Filter by the `#[test]` **attribute**, never name prefix alone.

**Generalization for this sub-project — exact-name partition.** Because the semantic token is
mid-name, the mover's group assignment changes from leading-prefix `startswith` to **exact
`#[test]`-name set membership**: each group is an explicit set of full fn names, matched by
equality. This touches only the GROUPS parsing / assignment in `main()` (the same region
sub-projects 3 & 4 already generalized) — `FN_RE` / `IDENT_CHARS` / `find_close_line` stay
byte-identical. The mover still writes `src/<stem>/<mod>.rs` (each `use super::*;` + verbatim
fns) and rewrites the facade to drop moved fns + append `#[path] mod` decls. The pin (3rd)
arg exists but is unused (no `include_*!`).

`verify.py` (`python3 verify.py <orig_rs> "<submodule_glob>"`) reuses the same lexer to prove
`{name: body}` from the original == from the submodules, exiting non-zero on any
name-set/body mismatch — the decisive byte-identity gate. No facade pins, so no facade glob.

## Verification gates (this sandbox)

- **G1 — facade drained:** `grep -c '#\[test\]' src/F.rs` == 0 for both files; facade ends
  with one `#[path] mod` decl per non-empty group and retains exactly its 3 original `use`
  lines (no `#[allow]`, no import deletion — verified clean under deny-warnings above).
- **G2 — submodule headers:** each `src/F/<mod>.rs` begins with exactly `use super::*;`.
- **G3 — baseline build green (capture at Task 1):** capture
  `cargo build -p kali_optimize --tests 2>&1 | grep -c '^warning'` on the clean base before
  any move; gate = **no-new-warnings** (count stays == that baseline). The lib-only build is
  expected clean; record the actual number at Task 1.
- **G4 — test-set identical (per file):** the lib-test `--list` basename multiset for the
  tests under `F` is unchanged before/after, via `cargo test -p kali_optimize --lib -- --list`
  filtered to the `F`-rooted module path, new `<mod>::` segment stripped (`s/^.*:://`), `sort`
  without `-u` (multiset), `diff` against the pre-split baseline → empty. Expected sizes:
  specialize_tests 37, object_fold_tests 48.
- **G5 — runtime pass/fail unchanged:** `cargo test -p kali_optimize --lib` pass/fail name-set
  identical before/after (strip new module prefix; shifted-but-unchanged panic messages are
  not regressions — code-motion moves line numbers, the message is the invariant).
- **G6 — byte-identity:** `verify.py` proves every moved `#[test]` body byte-identical
  base→submodules for both files.

> G4's exact `--list` filter is validated against real `cargo test --lib -- --list` output at
> plan Task 1 (baseline capture) before any move; the principle (per-file multiset preserved)
> is fixed.

## Constraints (verbatim-binding)

- Pure relocation. No new product code, no new tests, no renames, no reordering, no tidy.
- Verbatim moves only — `#[test]` attr lines + body + one trailing blank relocate
  byte-for-byte.
- Submodule header is exactly `use super::*;`. Facade keeps every original `use`. No
  per-submodule extern `use`s.
- Facade ends with **zero** `#[test]` fns (no `include_*!` pins needed here).
- No `pub`/`pub(crate)` widening (intra-crate child modules reach parent scope via
  `use super::*`; no visibility change needed).
- Do **not** run `cargo fmt` (repo fmt gate already red on baseline; accepted cosmetic minors
  are not regressions).
- Integration: **local-main ff-merge only — NEVER push origin** (origin/main intentionally
  lags). Re-verify on merged main, then delete the branch.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per task;
  durable recovery map.

## Out of scope

- kali_optimize's sub-1000-line co-located test files (`constant_fold_tests.rs` 306,
  `inline_tests.rs` 237, `layout_tests.rs` 219, `driver_tests.rs` 166,
  `fixture_support_tests.rs` 12) — below the series threshold; left as-is.
- Other crates' co-located src test monoliths (kali_types, kali_runtime, kali_codegen, …) —
  future series entries, not this sub-project.

## Branch & sequencing

- Branch `refactor/kali-optimize-src-test-monoliths` off `fe2878947`; baseline build+test
  captured (warning count + per-file `--list` multiset) before starting.
- Execute via superpowers:subagent-driven-development: implementer (sonnet) →
  review-package → task reviewer (sonnet; opus for finalize/whole-branch review).
- Two files = two task-groups (specialize_tests, then object_fold_tests), each split per the
  recipe, committed separately. Final opus whole-branch review proves all 85 `#[test]` bodies
  byte-identical base→head.
