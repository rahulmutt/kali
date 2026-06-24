## emit/ subsplit test-name baseline diff

**Result:** 325 test function names identical before and after; no tests dropped or renamed.

**Note on module-path prefixes:** `cargo test -- --list` does include module-path prefixes in its
output, contrary to the assumption in the brief. As a result, the raw diff between before and after
baselines is non-empty — the prefixes changed from `emit::emit_tests::*` (monolithic module) to
`emit::call::call_tests::*`, `emit::control_flow::control_flow_tests::*`,
`emit::literal::literal_tests::*`, and `emit::operators::operators_tests::*` (co-located
sub-modules). These are expected structural renames caused by the emit/ subsplit refactor itself.

**Verification:** Stripping all `*::` prefixes and sorting both files produces identical output
(verified with `diff` → empty). All 325 test function names are preserved exactly.

No test was dropped, duplicated, or renamed. The module-path differences reflect the intended
co-location of tests into their respective sub-modules.
