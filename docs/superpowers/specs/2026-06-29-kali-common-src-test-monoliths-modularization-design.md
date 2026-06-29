# kali_common co-located src test-monolith modularization — design

**Series:** 29th crate-modularization entry. Fifth entry of the post-kali_cli frontier
(other crates' co-located src unit-test monoliths; kali_optimize was 25th, kali_types 26th,
kali_runtime 27th, kali_codegen 28th).
**Date:** 2026-06-29
**Branch base / main HEAD at start:** `6a9507a0f`

## Goal

Split five of kali_common's co-located `src/*_tests.rs` unit-test monoliths into a thin facade +
per-concern `#[path] mod` submodules grouped on a **semantic axis**. **Pure verbatim
code-motion, zero behavior change**, identical compiled test set, byte-identical public API (the
crate and its consumers compile unedited).

| file | lines | `#[test]` fns | declared from | facade model |
|---|---|---|---|---|
| `src/late_tests.rs` | 1,013 | 18 | `late.rs:585` | drain to 0 |
| `src/math_tests.rs` | 828 | 21 | `math.rs:641` | drain to 0 |
| `src/process_kill_tests.rs` | 724 | 21 | `process_kill.rs:391` | drain to 0 |
| `src/object_tests.rs` | 376 | 9 | `object.rs:316` | drain to 0 |
| `src/promise_tests.rs` | 442 | 4 | `promise.rs:442` | drain to 0 |

73 `#[test]` fns total across five files. This is **not** TDD. No new product code, no new
tests, no renames, no reformatting. Every facade drains to **0** module-level fns: no file
contains a single non-`#[test]` module-level helper, so nothing is retained beyond the original
`use` line(s). Any nested helper travels with its parent test body.

### How the test modules are wired

Each monolith is declared at the foot of its product module via:

```rust
#[path = "<name>_tests.rs"]
mod <name>_tests;
```

`super::` inside a test file therefore resolves to the **product** module (e.g. `late`). The
facade keeps the original imports; each submodule opens with `use super::*;`, which re-propagates
everything the facade brought into scope — including `late_tests`'s
`use super::LATE_PROCESS_CONTROL_PREFIX_SEGMENTS;` (kept on the facade). This mirrors the
kali_codegen (28th) structure exactly.

### Out-of-scope files (kept whole)

kali_common's remaining co-located `src/*_tests.rs` files are already small, focused, and
single-concern; they stay as-is this entry (below the split line, untouched): `array_tests.rs`,
`collections_tests.rs`, `interner_tests.rs`, `intl_tests.rs`, `messages_tests.rs`,
`number_tests.rs`, `registry_tests.rs`, `source_map_tests.rs`, `span_tests.rs`,
`template_literal_tests.rs`.

## Per-file split (semantic axis)

### `late_tests.rs` (18 → 3 submodules) — axis: late-binding subject

- **`object_model`** (2): `test_late_object_model_aliases_and_source_are_canonical`,
  `test_late_object_model_own_property_aliases_and_source_are_canonical`
- **`capabilities`** (6): `test_late_threaded_runtime_aliases_and_source_are_canonical`,
  `test_late_permission_escalation_source_lists_request_and_revoke_aliases`,
  `test_late_env_materialization_source_lists_to_object_aliases`,
  `test_late_subprocess_source_lists_command_aliases`,
  `test_late_network_source_lists_connect_listen_and_serve_aliases`,
  `test_late_compat_object_has_own_source_lists_representative_aliases_in_order`
- **`process_control`** (10): the remaining `test_late_process_control_*`,
  `test_late_process_env_mutation_*` tests

### `math_tests.rs` (21 → 3 submodules) — axis: math operation family

- **`rounding`** (5): `abs_sign` (×2), `floor_trunc_ceil` (×2), `round` (×1)
- **`pow`** (13): all `test_math_pow_*` tests (source, alias inventory, browser alias inventory,
  bracketed, frozen-callable, invocation lines)
- **`roots`** (3): `cbrt`, `hypot`, `exp2` frozen-callable source tests

Note: `pow` tests are interleaved with `cbrt`/`hypot`/`exp2` in the original file; verbatim
per-fn motion groups them by concern regardless of original line order (same as prior entries).

### `process_kill_tests.rs` (21 → 3 submodules) — axis: zero-probe aspect

All tests are `test_process_kill_zero_probe_*`.

- **`inventory`** (6): `source`, `alias_inventory_source`, `unavailable_message`,
  `wrapped_zero_aliases`, `console_log`, `guard`
- **`parenthesized_freeze`** (8): `parenthesized_frozen_callable`, the
  `parenthesized_receiver_freeze_inventory*` set, the `parenthesized_receiver_freeze_bracket*`
  set, and `parenthesized_receiver`
- **`call_targets`** (7): `node_api_surface`, `call_target_aliases`, `typed_wrapper`,
  `wrapped_call_target`, `call_target_aliases_are_in_canonical_order`,
  `direct_call_target_binding_lines`, `sequence_call_target_binding_lines`

### `object_tests.rs` (9 → 2 submodules) — axis: reflection operation

- **`reflect`** (2): `test_reflect_own_keys_*`
- **`has_own`** (7): `test_object_has_own_*`, `test_object_enumeration_*`,
  `test_object_has_own_property_call_*`

### `promise_tests.rs` (4 → 2 submodules) — axis: combinator semantics

- **`aggregate`** (2): `test_promise_all_settled_browser_body_*`,
  `test_promise_all_browser_body_*` (combinators that aggregate every input result)
- **`select`** (2): `test_promise_race_browser_body_*`, `test_promise_any_browser_body_*`
  (combinators that settle on the first qualifying input)

**Total: 73 tests → 13 submodules.**

## Mechanical procedure (per file)

1. Create subdir `src/<name>_tests/`.
2. Move each `#[test]` fn **verbatim** into its target group file, prepending a single
   `use super::*;` line.
3. Reduce the facade to its original `use` line(s) plus one
   `#[path = "<name>_tests/<group>.rs"] mod <group>;` declaration per group.

## Verification

- `cargo test -p kali_common` — capture the test count before and after; counts identical, all
  pass.
- `cargo build -p kali_common` plus one dependent crate compile unedited (public API byte-identical).
- `cargo fmt --check` — accept known fmt nits per series convention.
- `git diff --stat` confirms only test-file code-motion (facades shrink; new submodule files
  added; no product-source changes beyond the unchanged `#[path] mod` decls).

## Commit shape

Mirrors the series:

1. `docs(kali_common): design spec for co-located src test-monolith modularization [spec]`
2. `docs(kali_common): implementation plan for src test-monolith modularization [plan]`
3. One `refactor(kali_common): split <file>_tests.rs into per-concern test submodules [refactor]`
   commit **per file** (5 refactor commits).

Local-main ff-merge only; no origin push.
