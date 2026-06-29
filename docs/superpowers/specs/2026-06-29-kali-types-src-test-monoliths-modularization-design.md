# kali_types co-located src test-monolith modularization — design

**Series:** 26th crate-modularization entry. Second entry of the post-kali_cli frontier
(other crates' co-located src unit-test monoliths; kali_optimize was the first, 25th).
**Date:** 2026-06-29
**Branch base / main HEAD at start:** `e0a3416ef`

## Goal

Split kali_types's seven co-located `src/*_tests.rs` unit-test monoliths (≥750 lines) into a
thin facade + per-concern `#[path] mod` submodules grouped on a **semantic axis**. **Pure
verbatim code-motion, zero behavior change**, identical compiled test set, byte-identical
public API (the crate and its consumers compile unedited).

| file | lines | `#[test]` fns | declared from | facade model |
|---|---|---|---|---|
| `src/static_analysis/array_tests.rs` | 3,289 | 91 | `static_analysis/array.rs:1308` | retain 1 helper |
| `src/static_analysis/object_tests.rs` | 2,849 | 52 | `static_analysis/object.rs:897` | retain 1 helper |
| `src/static_analysis/math_tests.rs` | 2,398 | 65 | `static_analysis/math.rs:870` | drain to 0 |
| `src/late_host_tests.rs` | 2,254 | 39 | `late_host.rs:667` | drain to 0 |
| `src/resolve/expression_tests.rs` | 1,430 | 42 | `resolve/expression.rs:511` | drain to 0 |
| `src/static_analysis/string_tests.rs` | 1,078 | 30 | `static_analysis/string.rs:943` | drain to 0 |
| `src/resolve/function_tests.rs` | 778 | 13 | `resolve/function.rs:87` | drain to 0 |

332 `#[test]` fns total. This is **not** TDD. No new product code, no new tests, no renames,
no reformatting.

## Approach

The proven series recipe (20 facades split this way across kali_cli + kali_optimize), applied
to kali_types's co-located src unit tests.

For each file `F`:

- **Facade** `src/.../F.rs`: keeps its original header `use` lines verbatim (the multi-line
  `use crate::*;` / `use kali_ast::{…}` / `use kali_error::_error_codes::…;` / `use std::fs;` /
  `use crate::test_support::*;` / `use kali_test_support::fixtures;` block as it appears in
  each file) + appended `#[path = "F/<mod>.rs"] mod <mod>;` decls. Contains **zero** `#[test]`
  fns (and, for the two retain-helper files, exactly the one module-level `assert_*` helper).
- **Submodules** `src/.../F/<mod>.rs`: each begins with exactly `use super::*;` (nothing else),
  followed by verbatim-moved `#[test]` fns (attribute lines + body + one trailing blank).

### Facade-drain model — five drain-to-0, two retain-one-helper

Inventory of module-level non-`#[test]` fns:

- `array_tests.rs` — **one** module-level helper
  `assert_resolution_accepts_frozen_iterator_protocol_edge` (line 64). It stays in the facade;
  children reach it through `use super::*;`.
- `object_tests.rs` — **one** module-level helper
  `assert_object_helper_iteration_with_let_binding_in_js_input` (line 2698). Stays in the
  facade.
- `late_host_tests.rs` — the `fn member` / `fn const_descriptor` / `fn permission_query` at
  lines 1204+ are **nested inside** individual test bodies (indented), so they move with their
  parent test. The facade drains to **0** module-level fns.
- `math_tests.rs`, `expression_tests.rs`, `string_tests.rs`, `function_tests.rs` — only
  `#[test]` fns at module level; facades drain to **0**.

The retained header `use` lines do **not** warn as unused when consumed only through children's
`use super::*;` — Rust's descendant-visibility re-exports the facade's private `use` items
through the child glob, marking them used. This is the exact mechanism proven clean (0 new
warnings, no `#[allow]`, no import deletion) in kali_optimize's fully-drained facades. The two
retained `assert_*` helpers additionally consume some imports directly.

### No `include_*!` gotcha here

`grep -rn 'include_str!\|include_bytes!\|include!' src/` is **0** across the whole crate —
nothing embeds a file-relative `include_*!`, so there is nothing to pin in the facade and the
mover's pin (3rd) arg is unused for this sub-project. (This is the simplest split flavor in the
series — no env carve-outs, no pins.)

### Wiring

- `#[path]` decls resolve **relative to the facade file's own directory**:
  `src/static_analysis/array_tests.rs` → `src/static_analysis/array_tests/<mod>.rs`,
  `src/resolve/expression_tests.rs` → `src/resolve/expression_tests/<mod>.rs`,
  `src/late_host_tests.rs` → `src/late_host_tests/<mod>.rs`, etc.
- The `#[path = "F_tests.rs"] mod F_tests;` decls in each production sibling
  (`static_analysis/array.rs:1308`, …) stay **unchanged** — they still name the facade file,
  which now re-exports its children. Submodule module paths become
  `static_analysis::array::array_tests::<mod>::` and `resolve::function::function_tests::<mod>::`.
- `use super::*;` in each submodule reaches the facade's private `use` imports via Rust
  descendant-visibility — the same mechanism every prior split relied on.

## Module groupings (semantic axis)

Every `#[test]` fn shares the `test_resolution_` prefix, so the discriminator sits **mid-name**
— grouping is by **explicit `#[test]`-name set membership**, not leading-prefix (the
kali_optimize exact-name-partition variant). The tables below state intent and approximate
counts; the implementation plan enumerates exact per-group membership. The decisive gate is
that each file's `--list` multiset is preserved (91 / 52 / 65 / 39 / 42 / 30 / 13).

### array_tests.rs (91) → `src/static_analysis/array_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `set_map_targets` | ~9 | Set/Map iteration-target construction: `new_set`/`new_map`/`frozen_set`/`frozen_map`/`global_this_set_and_map`/`array_from_new_set_and_new_map` |
| `array_from` | ~30 | `Array.from` callable recognition + `array_from` iteration (the `recognizes_*_array_from_callable_name` + `supports_*_array_from_iteration` cluster; for-await `array_from` variants stay here) |
| `for_of` | ~20 | `for_of` array iteration variants (binding/alias/spread/decorated-wrapper/reject-identifier) |
| `for_await` | ~13 | `for_await(_of)` array iteration variants (excluding `array_from`-named, which group under `array_from`) |
| `methods` | ~19 | static array-method resolution: reduce/filter/search/join/concat/at/map/some/every/find/flat_map (`allows_static_*`, `rejects_dynamic_*`, `allows_identity_*`) |

### object_tests.rs (52) → `src/static_analysis/object_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `object_is` | ~14 | `Object.is` family (alias spellings, primitive/object literals, signed-zero, optional-chain/sequence wrappers) |
| `has_own_entries` | ~10 | `Object.hasOwn` + `Object.fromEntries` (bracketed/frozen aliases, conditional/satisfies wrappers, has_own over fromEntries results) |
| `enumeration` | ~12 | `Object.keys/values/entries` iteration + let-binding rebind accept/reject + bracket-root enumeration aliases |
| `freeze_late_model` | ~16 | `Object.freeze`-wrapped helpers, proxy-revocable / late object-model globals, freeze-wrapped Set/Map constructor targets, transparent/decorated wrappers |

### math_tests.rs (65) → `src/static_analysis/math_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `pow` | ~12 | `Math.pow` family (integer/zero-exponent/const-alias/negative-base, unsupported negative/optional-chain rejects) |
| `transcendental` | ~33 | exp/log/exp2/expm1/log1p/fround/sqrt/cbrt/hypot/imul/sin/cos/tan/asin/acos/atan/sinh/cosh/tanh/asinh/acosh/atanh/atan2/log2/log10 (supports + non-identity/unsupported rejects) |
| `rounding` | ~12 | floor/round/ceil/trunc/sign/clz32 member calls (incl. global_this/optional-chain/sequence/conditional round wrappers) |
| `wrappers` | ~8 | frozen/global_this callable-alias wrappers across js-like extensions for math helpers (abs/sign/pow/round/expm1/log1p, global_this math builtin slices) |

### late_host_tests.rs (39) → `src/late_host_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `globals` | ~10 | browser/web/shared-baseline/threaded-runtime globals + late-host control globals + late subprocess/network globals |
| `process_env` | ~11 | process/deno cwd/chdir/exit + env snapshot/materialization + env mutation accept/reject across surfaces |
| `permissions` | ~6 | permission-query descriptors (supported/unsupported/const-binding) + permission-escalation members |
| `intl_imports_kill` | ~12 | Intl member access, node builtin/timers imports (in/out of node context), `process_kill_zero` probe variants |

### expression_tests.rs (42) → `src/resolve/expression_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `exports` | ~16 | unresolved public/default exports, export-all sources, re-export sources, alias variants, unresolved identifiers inside default-export functions |
| `operators` | ~10 | nullish coalescing, remainder, update expressions (mutable/immutable/decorated), compound assignment, missing imports |
| `dynamic_import` | ~16 | dynamic-import targets (static/template/const-bound/parenthesized/sequence/directory-index/logical/constant-template; reject non-literal/unknown/no-index) |

### string_tests.rs (30) → `src/static_analysis/string_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `iteration` | ~6 | for-of/for-await string-concatenation / template-literal / const-string-alias iteration |
| `methods` | ~24 | static ASCII string methods: search/slice/substring/concat/at/char_at/char_code_at/trim/case/replace/split (`allows_static_ascii_*`, `rejects_dynamic_or_non_ascii_*`) |

### function_tests.rs (13) → `src/resolve/function_tests/`

| module | ~count | members (by intent) |
|---|---|---|
| `generator_functions` | ~7 | standalone generator/async-generator/mixed generator function lowering rejections (incl. yield-delegation, js/tsx variants) |
| `class_methods` | ~6 | class-method / class-expression generator lowering (sync/async/mixed-collapse) + async class-method lowering support |

> Final per-module counts are whatever the mover's `--list` baseline diff proves; the tables
> state intent. The decisive gate is that each file's `--list` multiset is preserved.

## Tooling

`.superpowers/sdd/move_fns.py` + `.superpowers/sdd/verify.py` (git-ignored scratch; re-created
from the documented design). **Keep `FN_RE` / `IDENT_CHARS` / `find_close_line` byte-identical**
— the string/comment/raw-string-aware brace lexer is required (these files contain `r#"..."#`
JS/TS templates with `}` at column 0; a naive column-0 close-brace scan breaks). Filter by the
`#[test]` **attribute**, never name prefix alone (the two retain-helper files have module-level
`assert_*` fns that must NOT move).

**Exact-name partition** (kali_optimize variant): because the semantic token is mid-name, the
mover's group assignment is **exact `#[test]`-name set membership** (equality), not leading-prefix
`startswith`. Each group is an explicit set of full fn names. This touches only the
GROUPS parsing / assignment in `main()`; `FN_RE` / `IDENT_CHARS` / `find_close_line` stay
byte-identical. The mover writes `src/<...>/<stem>/<mod>.rs` (each `use super::*;` + verbatim
fns) and rewrites the facade to drop moved fns + append `#[path] mod` decls. For the two
retain-helper files the module-level `assert_*` fn is **not** in any group set, so it stays in
the facade. The pin (3rd) arg exists but is unused (no `include_*!`).

`verify.py` (`python3 verify.py <orig_rs> "<submodule_glob>"`) reuses the same lexer to prove
`{name: body}` from the original == from the submodules, exiting non-zero on any
name-set/body mismatch — the decisive byte-identity gate. No facade pins, so no facade glob.

## Verification gates (this sandbox)

Baseline (captured on clean base before any move): `cargo build -p kali_types --tests` = **0
warnings**; `cargo test -p kali_types --lib` = **372 pass / 0 fail**. The literal series gates
hold here — no env-failure carve-outs (unlike kali_cli).

- **G1 — facade drained:** `grep -c '#\[test\]' src/.../F.rs` == 0 for all 7 files; each facade
  ends with one `#[path] mod` decl per non-empty group, retains exactly its original header
  `use` lines (no `#[allow]`, no import deletion), and — for `array_tests`/`object_tests` —
  exactly its one module-level `assert_*` helper.
- **G2 — submodule headers:** each `src/.../F/<mod>.rs` begins with exactly `use super::*;`.
- **G3 — no new warnings:** `cargo build -p kali_types --tests 2>&1 | grep -c '^warning'`
  stays == 0 (the captured baseline).
- **G4 — test-set identical (per file):** the lib-test `--list` basename multiset for the tests
  under `F` is unchanged before/after, via `cargo test -p kali_types --lib -- --list` filtered
  to the `F`-rooted module path, new `<mod>::` segment stripped (`s/^.*:://`), `sort` without
  `-u` (multiset), `diff` against the pre-split baseline → empty. Expected sizes: 91 / 52 / 65 /
  39 / 42 / 30 / 13.
- **G5 — runtime pass/fail unchanged:** `cargo test -p kali_types --lib` pass/fail name-set
  identical before/after (strip new module prefix; shifted-but-unchanged panic messages are not
  regressions — code-motion moves line numbers, the message is the invariant). Expected total
  unchanged at 372.
- **G6 — byte-identity:** `verify.py` proves every moved `#[test]` body byte-identical
  base→submodules for all 7 files; the two retained `assert_*` helpers remain in their facade.

> G4's exact `--list` filter is validated against real `cargo test --lib -- --list` output at
> plan Task 1 (baseline capture) before any move; the principle (per-file multiset preserved)
> is fixed.

## Constraints (verbatim-binding)

- Pure relocation. No new product code, no new tests, no renames, no reordering, no tidy.
- Verbatim moves only — `#[test]` attr lines + body + one trailing blank relocate
  byte-for-byte.
- Submodule header is exactly `use super::*;`. Facade keeps every original `use`. No
  per-submodule extern `use`s.
- Facade ends with **zero** `#[test]` fns (no `include_*!` pins needed here); the two
  retain-helper facades keep exactly their one module-level `assert_*` helper.
- No `pub`/`pub(crate)` widening (intra-crate child modules reach parent scope via
  `use super::*`; no visibility change needed).
- Do **not** run `cargo fmt` (repo fmt gate already red on baseline; accepted cosmetic minors
  are not regressions).
- Integration: **local-main ff-merge only — NEVER push origin** (origin/main intentionally
  lags). Re-verify on merged main, then delete the branch.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per task;
  durable recovery map.

## Out of scope

- kali_types's sub-750-line co-located test files (`typecheck_tests.rs` 411,
  `static_analysis/number_tests.rs` 328, `resolve/call_tests.rs` 128, `promise_tests.rs` 118,
  `resolve/member_tests.rs` 102, `context_tests.rs` 91, `resolve/jsx_tests.rs` 72,
  `scope_tests.rs` 50) — below the series threshold; left as-is.
- Other crates' co-located src test monoliths (kali_runtime, kali_codegen, …) — future series
  entries, not this sub-project.

## Branch & sequencing

- Branch `refactor/kali_types-modularization` off `e0a3416ef`; baseline build+test captured
  (0 warnings + per-file `--list` multiset + 372 pass) before starting.
- Execute via superpowers:subagent-driven-development: implementer (sonnet) → review-package →
  task reviewer (sonnet; opus for finalize/whole-branch review).
- Seven files = seven task-groups (largest→smallest: array, object, math, late_host,
  expression, string, function), each split per the recipe, committed separately. Final opus
  whole-branch review proves all 332 `#[test]` bodies byte-identical base→head.
</content>
</invoke>
