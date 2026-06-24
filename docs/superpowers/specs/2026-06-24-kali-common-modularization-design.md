# kali_common modularization — design

**Date:** 2026-06-24
**Crate:** `kali_common` (5th crate in the kali modularization effort)
**Branch:** `refactor/kali-common-modularization`
**Predecessors:** kali_types (pilot), kali_codegen, kali_runtime, kali_optimize — all done + merged.

## Goal

Break `kali_common`'s 3,251-line `lib.rs` and 4,083-line `tests.rs` monolith into focused,
single-purpose modules with co-located tests, following the established kali-crate pattern —
**zero behavior change, public API preserved, green + clippy-clean at every commit.**

## Shape (differs from prior four crates)

The first four crates were a single large `impl Ctx` split method-by-method. kali_common is
instead a **flat pile of 156 free functions** — almost entirely JS-feature **source-snippet
generators** consumed as cross-crate test fixtures (e.g. `process_kill_zero_probe_source`,
`math_pow_source`, `late_process_control_source`), plus a small core of real utility types.

Consequences:
- Extraction is **pure text-movement of free `pub fn`/`pub const fn`** — no `pub(crate)`
  receiver-widening needed (there is no shared method receiver).
- External consumers reference **flat paths** (`kali_common::math_pow_source`, confirmed by
  grep across the workspace). Therefore every new module is re-exported from the facade with
  `pub use <mod>::*;` → **zero consumer edits**.
- Cross-module fixture calls (composite source fns calling alias/inventory fns in other
  modules) resolve through the facade re-exports, so each module gets `use crate::*;`.

## Invariants (carried over unchanged from the pilot)

- Zero behavior change; functions moved byte-for-byte verbatim.
- Public API preserved exactly (all current `pub` items remain reachable at their current
  flat paths).
- Identical whole-crate `cargo test --list` **basename** set (strip module-path prefixes
  before comparing — a raw diff is non-empty by design after co-location).
- Green + warning-free + `clippy -D warnings` clean at **every** commit.
- Baseline captured to `docs/superpowers/baselines/` before any movement.

**Authoritative test count: 102** (94 in `tests.rs` + 8 in the existing co-located
`interner_tests`/`span_tests`/`source_map_tests`). The proof baseline is the **whole-crate
`--list`**, not the `tests.rs` `#[test]` count.

## Module decomposition

**Core utilities** — keep existing `interner` / `span` / `source_map` / `template` modules
as-is; extract one new module:

- `registry.rs` — `SourceRegistry`, `FileId`, `SourceFile`, the `lib.rs` `SourceMap`,
  the `GLOBAL_INTERNER` / `SOURCE_REGISTRY` statics, `format_file_ref`,
  `bytewise_shared_memory_is_lock_free`.
  - **Care:** a *different* `SourceMap` type already lives in `source_map.rs` (built `From<SourceRegistry>`).
    The two coexist in separate modules today; preserve that — do **not** merge or rename.

**Fixture generators** — one flat module per JS-feature cluster:

| Module | Fns (approx) | Tests (approx) | Covers |
|---|---|---|---|
| `messages.rs` | 10 | 7 | async/generator lowering-unavailable messages |
| `process_kill.rs` | 33 | 21 | `process_kill_zero_probe_*` |
| `late.rs` | 27 | 16 | `late_process`/`object`/`threaded`/`permission`/`env`/`subprocess`/`network`/`compat` |
| `math.rs` | 44 | 20 | `math_abs`/`floor`/`round`/`pow`/`cbrt`/`hypot`/`exp2` |
| `object.rs` | 13 | 9 | `object_has_own_*`/`object_enumeration_*` + `reflect_own_keys_*` |
| `number.rs` | 5 | 1 | `number_predicates_*` |
| `array.rs` | 6 | 4 | `array_from_*` |
| `collections.rs` | 10 | 2 | set + map constructors |
| `promise.rs` | 4 | 4 | `promise_*_browser_body` |
| `intl.rs` | 2 | 1 | `broader_intl_*` |
| `template_literal.rs` | 2 | 1 | JS template-literal snippet generators (distinct from `template.rs`) |

(Counts are approximate and will be reconciled against the actual file during planning; the
authoritative whole-crate total stays 102.)

Each fixture module:
- header `use crate::*;` (resolves sibling-module fixture calls via facade re-exports);
- byte-for-byte verbatim function bodies;
- one sibling `<mod>_tests.rs` wired via `#[cfg(test)] #[path = "<mod>_tests.rs"] mod <mod>_tests;`
  at the bottom of the source module; test submodule uses `use crate::*;`.

`lib.rs` → thin facade (~60 lines): crate doc + module decls + the crate's import surface +
`pub use <mod>::*;` per module (preserving every flat public path) + existing
`pub use interner::{InternedString, Interner}` / `pub use span::Span`.

`tests.rs` (94 tests) splits into the 11 fixture `*_tests.rs` + registry tests, by cluster.

## GLOB rule

A module exporting free fns that remaining-in-`lib.rs` callers still use keeps
`pub use <mod>::*;`. Once `lib.rs` is a pure facade, every module is re-exported, so each glob
is live. **Verify with clippy** after each task; if clippy flags a glob `unused_imports`,
delete it (do not `#[allow]`). Drop `use crate::*;` from any module/facade that references no
crate items (the kali_runtime test_support lesson).

## Fixture-adoption decision (resolved)

**Skip `kali_test_support` for this crate.** kali_common's tests are pure-string assertions
with zero filesystem/tempdir usage, and it is the foundational crate everyone depends on.
Adding the dev-dep + a token round-trip test (the kali_codegen/kali_optimize precedent) would
introduce a dependency that earns nothing here. Hold the suite at exactly **102 tests** —
strict identical-set invariant. (No dependency-cycle risk either way: `kali_test_support`
depends only on `tempfile`.)

## Verification (per-commit gate)

```
cargo test -p kali_common
cargo build --workspace
cargo clippy -p kali_common --all-targets -- -D warnings
```

Identical-set proof: capture whole-crate `cargo test -p kali_common --all-targets -- --list`
basenames to a baseline before movement; after each task, compare basenames (prefix-stripped)
— must match exactly.

## Execution

Subagent-driven-development (same as cycles 2–4), one task per module extraction
(source fns → module + co-located `*_tests.rs`), green at each commit, on
`refactor/kali-common-modularization`. Final whole-branch opus review before merge.
