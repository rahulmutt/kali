# kali_codegen co-located src test-monolith modularization — design

**Series:** 28th crate-modularization entry. Fourth entry of the post-kali_cli frontier
(other crates' co-located src unit-test monoliths; kali_optimize was 25th, kali_types 26th,
kali_runtime 27th).
**Date:** 2026-06-29
**Branch base / main HEAD at start:** `138aaa1a7`

## Goal

Split eight of kali_codegen's co-located `src/*_tests.rs` unit-test monoliths into a thin facade +
per-concern `#[path] mod` submodules grouped on a **semantic axis**. **Pure verbatim
code-motion, zero behavior change**, identical compiled test set, byte-identical public API (the
crate and its consumers compile unedited).

| file | lines | `#[test]` fns | declared from | facade model |
|---|---|---|---|---|
| `src/intrinsics/math_tests.rs` | 2,385 | 97 | `intrinsics/math.rs:399` | drain to 0 |
| `src/emit/call_tests.rs` | 2,146 | 88 | `emit/call.rs:2523` | drain to 0 |
| `src/intrinsics/host_tests.rs` | 819 | 30 | `intrinsics/host.rs:581` | drain to 0 |
| `src/intrinsics/array_tests.rs` | 578 | 23 | `intrinsics/array.rs:1638` | drain to 0 |
| `src/emit/control_flow_tests.rs` | 570 | 22 | `emit/control_flow.rs:413` | retain 1 helper |
| `src/intrinsics/object_tests.rs` | 400 | 18 | `intrinsics/object.rs:630` | drain to 0 |
| `src/intrinsics/string_tests.rs` | 438 | 16 | `intrinsics/string.rs:1157` | drain to 0 |
| `src/intrinsics/collections_tests.rs` | 242 | 14 | `intrinsics/collections.rs:402` | drain to 0 |

308 `#[test]` fns total across eight files. This is **not** TDD. No new product code, no new
tests, no renames, no reformatting. Seven facades drain to **0** module-level fns (shared helpers
live in `src/test_support.rs`; any nested helper travels with its parent test body). The lone
exception is `control_flow_tests.rs`, which retains **one** module-level non-`#[test]` helper.

### Out-of-scope files (kept whole)

The three smallest co-located test files stay as-is this sub-project — already small, focused,
single-concern, below the chosen ≥13-test scope line:

- `src/intrinsics/number_tests.rs` (85 lines, 3 tests)
- `src/emit/literal_tests.rs` (107 lines, 6 tests)
- `src/emit/operators_tests.rs` (154 lines, 8 tests)

## Approach

The proven series recipe (36+ facades split this way across kali_cli + kali_optimize + kali_types
+ kali_runtime), applied to kali_codegen's co-located src unit tests.

For each file `F`:

- **Facade** `src/.../F.rs`: keeps its original header `use` lines verbatim (e.g.
  `use crate::test_support::*;` / `use crate::*;` / `use wasmparser::Validator;` exactly as each
  file presents them) + appended `#[path = "F/<mod>.rs"] mod <mod>;` decls. Contains **zero**
  `#[test]` fns; zero module-level helpers (except `control_flow_tests`, see below).
- **Submodules** `src/.../F/<mod>.rs`: each begins with exactly `use super::*;` (nothing else),
  followed by verbatim-moved `#[test]` fns (attribute lines + body + one trailing blank).

### Facade-drain model — seven drain to 0, one retains a helper

Inventory of module-level non-`#[test]` fns across the eight files: **exactly one**, in
`control_flow_tests.rs` — the helper `legacy_phase1_baseline(program, mir) -> LirProgram`
(line 455), consumed by the `pipeline_basics`-group tests. Per the kali_types `HELPER=1` pattern,
this helper is **retained in the facade** (mover 3rd "pin" arg, repurposed for a non-`#[test]`
helper) so the submodules reach it via `use super::*;`. The other seven facades end with **zero**
fns — the simplest drain flavor, matching kali_optimize / kali_runtime fully-drained facades.

The retained header `use` lines do **not** warn as unused when consumed only through children's
`use super::*;` — Rust's descendant-visibility re-exports the facade's private `use` items
through the child glob, marking them used. This is the exact mechanism proven clean (0 new
warnings, no `#[allow]`, no import deletion) in kali_optimize, kali_types, and kali_runtime
fully-drained facades.

### No `include_*!` gotcha here

`grep -rn 'include_str!\|include_bytes!\|include!' src/**/*_tests.rs` is **0** across the in-scope
files — nothing embeds a file-relative `include_*!`, so there is nothing path-pinning to do and
the mover's pin (3rd) arg is used **only** for `control_flow_tests`'s `legacy_phase1_baseline`
helper retention, never for path rewriting.

### Wiring

- The production siblings declare each test file as `#[cfg(test)]` + `#[path = "F_tests.rs"]` +
  `mod F_tests;` (e.g. `intrinsics/math.rs:397-399`, `emit/call.rs:2521-2523`). These decls stay
  **unchanged** — they still name the facade file, which now re-exports its children.
- The facade's appended `#[path = "F/<mod>.rs"] mod <mod>;` decls resolve **relative to the
  facade file's own directory**: `src/intrinsics/math_tests.rs` → `src/intrinsics/math_tests/<mod>.rs`;
  `src/emit/call_tests.rs` → `src/emit/call_tests/<mod>.rs`, etc.
- Submodule module paths become `intrinsics::math::math_tests::<mod>::`,
  `emit::call::call_tests::<mod>::`, `intrinsics::host::host_tests::<mod>::`, and so on.
- `use super::*;` in each submodule reaches the facade's private `use` imports (and, for
  `control_flow_tests`, the retained helper) via Rust descendant-visibility — the same mechanism
  every prior split relied on.

## Module groupings (semantic axis)

The tables below state intent and the **exact** counts validated by a token classifier over the
extracted `#[test]` names (every file's partition is exhaustive and non-overlapping — zero
unmatched). The implementation plan enumerates exact per-group membership as explicit fn-name
lists for the mover; the decisive gate is that each file's `--list` multiset is preserved.

### intrinsics/math_tests.rs (97) → `src/intrinsics/math_tests/` — exact-name set (mid-name)

The operation token is mid-name (after `math_` / `supported_math_` / `unsupported_math_`
prefixes), so grouping is by explicit `#[test]`-name set membership. Unsupported-feature tests
fold into their operation family (no separate `unsupported` group needed):

| module | count | members (by intent) |
|---|---|---|
| `pow` | 16 | every `*math_pow_*` (constant-folding identities, alias chains, single-arg / non-integer / negative-exponent rejections) |
| `rounding` | 18 | `*math_{round,floor,trunc,ceil,fround}_*` (host imports + member constant-folding through freeze/parenthesized wrappers) |
| `integer_ops` | 22 | `*math_{max,min,abs,sign,imul,clz32}_*` (host imports + static numeric-literal constant folds + alias chains) |
| `transcendental` | 41 | `*math_{sqrt,cbrt,hypot,log,log2,log10,log1p,exp,exp2,expm1,sin,cos,tan,asin,acos,atan,atan2,asinh,acosh,atanh,hyperbolic,inverse_*}_*` (supported lowering + unsupported-feature gates) |

### emit/call_tests.rs (88) → `src/emit/call_tests/` — exact-name set (mid-name)

| module | count | members (by intent) |
|---|---|---|
| `array_iteration` | 44 | `supported_for_{of,await}_array_iteration_*` + `unsupported_array_callback_iteration_*` (array-from / wrapper / spread / map-filter callback forms) |
| `object_enumeration` | 31 | `for_{of,await}` object enumeration / entries / keys / values / from_entries operands, string-literal enumeration, `object_enumeration_helper_*` |
| `reflect_own_keys` | 9 | every `*reflect_own_keys*` (frozen static-object literals/aliases, sequence/nullish/logical wrappers) |
| `diagnostics` | 4 | `unresolved_identifier_*`, `unresolved_call_target_*`, `duplicate_unresolved_identifier_*`, `source_path_in_temp_dir_*` |

### intrinsics/host_tests.rs (30) → `src/intrinsics/host_tests/` — exact-name set (mid-name)

The surface token (`console` / `process` / `deno` / `env`) appears after `global_this_` /
`bracketed_global_this_` / `mixed_global_this_` spelling prefixes; `env` membership wins over
`deno`/`process` for the `*_env_*` names:

| module | count | members (by intent) |
|---|---|---|
| `process` | 15 | `*process_{argv,pid,cwd,exit,kill}*` (runtime args/pid/cwd imports, exit, zero-probe kill via wrappers) |
| `deno` | 7 | `deno_{args,pid,cwd,chdir}*` (runtime args/pid/cwd/chdir imports incl. global_this spellings) |
| `env` | 6 | `*deno_env_{get,has,set,delete}*` (runtime env imports incl. bracketed / mixed-global_this spellings) |
| `console` | 2 | `console_member_calls_*`, `console_assert_member_*` |

### intrinsics/array_tests.rs (23) → `src/intrinsics/array_tests/` — exact-name set (mid-name)

| module | count | members (by intent) |
|---|---|---|
| `callbacks` | 12 | `*array_{some,every,find,find_index,find_last,find_last_index,reduce}*` callback-lowering forms |
| `static_ops` | 11 | `*static_array_{includes,index_of,last_index_of,join,to_string,concat,at}*` + `*string_split*` static lowering / gating |

### emit/control_flow_tests.rs (22) → `src/emit/control_flow_tests/` — exact-name set; facade retains 1 helper

| module | count | members (by intent) |
|---|---|---|
| `function_plans` | 9 | `function_plans_are_detected_*` + `function_plans_preserve_generator_flavor_metadata_*` (class methods/expressions, default-export generator/async-generator declarations) |
| `unsupported_generators` | 10 | `unsupported_*generator*`, `mixed_generator_and_async_generator_*`, `generator_function_without_yield_*` feature-unavailable gates |
| `pipeline_basics` | 3 | `generates_valid_wasm_for_simple_programs`, `boolean_branches_use_the_layout_fast_path`, `mir_backed_pipeline_reduces_legacy_overhead_on_escaping_locals` |

> Facade retains the non-`#[test]` helper `legacy_phase1_baseline` (used by the `pipeline_basics`
> module's `mir_backed_pipeline_*` test). The `pipeline_basics` submodule reaches it through
> `use super::*;`.

### intrinsics/object_tests.rs (18) → `src/intrinsics/object_tests/` — leading-prefix-derived

Two clean, mutually-exclusive leading prefixes (`object_is_` vs `object_has_own_`); encoded as
exact-name sets for the current mover:

| module | count | members (by intent) |
|---|---|---|
| `is` | 9 | `object_is_*` (primitive/reference/member-root/freeze/parenthesized/unary-plus forms) |
| `has_own` | 9 | `object_has_own_*` (from_entries operands, bracketed global_this spellings, callable-alias / freeze wrappers) |

### intrinsics/string_tests.rs (16) → `src/intrinsics/string_tests/` — exact-name set (mid-name)

| module | count | members (by intent) |
|---|---|---|
| `lookup` | 9 | `*string_{search,prefix_suffix,length,slice,substring,at,char_at,char_code_at}*` accessor lowering |
| `transform` | 7 | `*string_{repeat,concat,trim_family,case_family,replace,replace_all}*` transform lowering / gating |

### intrinsics/collections_tests.rs (14) → `src/intrinsics/collections_tests/` — leading-prefix-derived

Three mutually-exclusive constructor prefixes; encoded as exact-name sets:

| module | count | members (by intent) |
|---|---|---|
| `map` | 6 | `map_constructor_iteration_*` (frozen input / builtin & frozen-constructor alias / frozen result forms) |
| `set` | 5 | `set_constructor_iteration_*` (frozen input / builtin & frozen-constructor alias / parenthesized frozen result forms) |
| `combined` | 3 | `set_and_map_constructor_iteration_*` (nullish/logical-wrapped, global_this & bracketed global_this roots) |

> Final per-module counts are whatever the mover's `--list` baseline diff proves; the tables
> state intent and the classifier-validated counts. The decisive gate is that each file's `--list`
> multiset is preserved.

## Tooling

`.superpowers/sdd/move_fns.py` + `.superpowers/sdd/verify.py` (git-ignored scratch). **Keep
`FN_RE` / `IDENT_CHARS` / `find_close_line` byte-identical** — the string/comment/raw-string-aware
brace lexer is required (these files contain `r#"..."#` JS/TS templates with `}` at column 0; a
naive column-0 close-brace scan breaks). Filter by the `#[test]` **attribute**, never name prefix
alone (cfg-gated helpers must stay).

**Grouping mode:** the current mover uses **exact-name partition** — each group is an explicit set
of full `#[test]` fn names, assigned by equality (`fn_name == member`), first matching group in
spec order wins, `*` is the catch-all. All eight files use this mode (the two prefix-clean files,
`object_tests` and `collections_tests`, are still expressed as exact-name sets derived from their
leading prefixes). This touches only the GROUPS parsing/assignment in `main()`; `FN_RE` /
`IDENT_CHARS` / `find_close_line` stay byte-identical. The mover writes `src/<...>/<stem>/<mod>.rs`
(each `use super::*;` + verbatim fns) and rewrites the facade to drop moved fns + append
`#[path] mod` decls.

**Pin (3rd) arg:** unused for path-pinning (no `include_*!`). Used **only** for
`control_flow_tests` to keep `legacy_phase1_baseline` in the facade. The pin mechanism keeps a
named fn out of the moved set; for `control_flow_tests` it retains a non-`#[test]` helper (the
mover already excludes non-`#[test]` fns from moves, so the explicit pin is belt-and-suspenders
documentation of intent — confirm the helper stays in the facade post-split).

`verify.py` (`python3 verify.py <orig_rs> "<submodule_glob>" [facade_glob_for_pins]`) reuses the
same lexer to prove `{name: body}` from the original == from the submodules (+ the retained
facade helper for `control_flow_tests`), exiting non-zero on any name-set/body mismatch — the
decisive byte-identity gate.

## Verification gates (this sandbox)

Baseline captured on the clean base (`138aaa1a7`): `cargo build -p kali_codegen --tests` =
**0 warnings**; `cargo test -p kali_codegen --lib` = **325 pass / 0 fail** (308 in-scope tests +
17 in the three out-of-scope files). kali_codegen has **no chromium-sandbox dependency**, so the
literal series "0 warnings / fully green" gates hold — no env-failure carve-outs (unlike
kali_cli). The **operative gates remain no-new-warnings + pass/fail unchanged** against this
baseline; the plan re-confirms the numbers at Task 1.

- **G1 — facade drained:** `grep -c '#\[test\]' src/.../F.rs` == 0 for all 8 files; each facade
  ends with one `#[path] mod` decl per non-empty group, retains exactly its original header `use`
  lines (no `#[allow]`, no import deletion), and zero module-level fns — **except**
  `control_flow_tests`, which additionally retains `legacy_phase1_baseline`.
- **G2 — submodule headers:** each `src/.../F/<mod>.rs` begins with exactly `use super::*;`.
- **G3 — no new warnings:** `cargo build -p kali_codegen --tests 2>&1 | grep -c '^warning'`
  stays == the captured baseline.
- **G4 — test-set identical (per file):** the lib-test `--list` basename multiset for the tests
  under `F` is unchanged before/after, via `cargo test -p kali_codegen --lib -- --list` filtered
  to the `F`-rooted module path (anchored with `^` to avoid suffix-substring over-match), new
  `<mod>::` segment stripped (`s/^.*:://`), `sort` without `-u` (multiset), `diff` against the
  pre-split baseline → empty. Expected sizes: 97 / 88 / 30 / 23 / 22 / 18 / 16 / 14.
- **G5 — runtime pass/fail unchanged:** `cargo test -p kali_codegen --lib` pass/fail name-set
  identical before/after (strip new module prefix; shifted-but-unchanged panic messages are not
  regressions — code-motion moves line numbers, the message is the invariant). Expected total
  unchanged at **325 pass / 0 fail**.
- **G6 — byte-identity:** `verify.py` proves every moved `#[test]` body byte-identical
  base→submodules for all 8 files (+ the retained `control_flow_tests` helper).

> G4's exact `--list` filter is validated against real `cargo test --lib -- --list` output at
> plan Task 1 (baseline capture) before any move; the principle (per-file multiset preserved) is
> fixed. **Anchor the MODPATH filter with `^`** — kali_runtime hit a suffix-substring over-count
> when one module path was a substring of another (`execute::execute_tests` also matched
> `browser::execute::execute_tests`). kali_codegen has no such collision in the in-scope set, but
> anchor as a precaution.

## Constraints (verbatim-binding)

- Pure relocation. No new product code, no new tests, no renames, no reordering, no tidy.
- Verbatim moves only — `#[test]` attr lines + body + one trailing blank relocate byte-for-byte.
- Submodule header is exactly `use super::*;`. Facade keeps every original `use`. No
  per-submodule extern `use`s.
- Facade ends with **zero** `#[test]` fns and zero module-level helpers — except
  `control_flow_tests`, which retains exactly the `legacy_phase1_baseline` helper.
- No `pub`/`pub(crate)` widening (intra-crate child modules reach parent scope via
  `use super::*`; no visibility change needed).
- Do **not** run `cargo fmt` (repo fmt gate already red on baseline; accepted cosmetic minors are
  not regressions).
- Integration: **local-main ff-merge only — NEVER push origin.** (origin/main currently equals
  HEAD `138aaa1a7` from external syncing of prior work, but the standing local-only convention
  holds for this sub-project.) Re-verify on merged main, then delete the branch.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch) — overwrite per task;
  durable recovery map.

## Out of scope

- kali_codegen's three sub-threshold co-located test files left as-is this sub-project:
  `src/intrinsics/number_tests.rs` (3 tests), `src/emit/literal_tests.rs` (6 tests),
  `src/emit/operators_tests.rs` (8 tests) — below the chosen ≥13-test scope line.
- Other crates' co-located src test monoliths — future series entries, not this sub-project.

## Branch & sequencing

- Branch `refactor/kali_codegen-modularization` off `138aaa1a7`; baseline build+test captured
  (warning count + per-file `--list` multiset + pass/fail count) before starting.
- Execute via superpowers:subagent-driven-development: implementer (sonnet) → review-package →
  task reviewer (sonnet; opus for finalize/whole-branch review).
- Eight files = eight task-groups (largest→smallest: math, call, host, array, control_flow,
  object, string, collections), each split per the recipe, committed separately. Final opus
  whole-branch review proves all 308 `#[test]` bodies byte-identical base→head.
