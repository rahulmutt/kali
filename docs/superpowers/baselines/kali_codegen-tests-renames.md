# kali_codegen test-name baseline: before → after renames

## Summary

**No renames** — every original test basename is preserved.

## Module-path relocation (expected, not a rename)

All 324 original tests moved from the monolithic `tests::` module in `src/tests.rs`
to co-located `*_tests.rs` siblings of their respective modules.  Their bare function
names (basenames, i.e. the part after the last `::`) are **unchanged**.

What `cargo test -- --list` reports changed is only the module-path *prefix*:

| Before (`--list` output prefix) | After (`--list` output prefix) |
|---|---|
| `tests::math_abs_folds_constant` | `intrinsics::math::math_tests::math_abs_folds_constant` |
| `tests::emit_generates_valid_wasm_…` | `emit::emit_tests::emit_generates_valid_wasm_…` |
| … (all 324 tests) | … |

The raw `diff before after` therefore shows every line as changed — this is
**expected and not a regression**.  The meaningful comparison is on basenames only.

## Basename comparison result (Task 14)

```
=== only in BEFORE (dropped — must be EMPTY) ===
(empty)

=== only in AFTER (added — must be exactly the one new test) ===
source_path_in_temp_dir_attaches_to_unresolved_identifier_diagnostics
```

- 0 basenames dropped
- 0 basenames renamed
- 1 basename added (intentional, see below)

## Intentional addition

`source_path_in_temp_dir_attaches_to_unresolved_identifier_diagnostics`
(file: `src/emit/emit_tests.rs`)

Added in Task 13 to exercise the `kali_test_support` fixtures dev-dependency
(`tempdir` / `write_file`).  `kali_codegen` had no pre-existing filesystem tests
to convert, so a new test was written to verify that the `source_path` field on
an unresolved-identifier diagnostic is set correctly when the source file lives in
a temp directory.  This was an approved controller/user decision.

**Count: 324 → 325**
