# Phase 9 Optimization Inventory

This note is a current-evidence snapshot, not an exhaustive compiler-pass catalog. It records the optimization families that are already backed by checked-in tests and the mode boundaries that currently matter for the plan.

## Current checked-in evidence

| Build mode | Representative real optimizations today | Primary test anchors |
|---|---|---|
| `fast` | Minimal optimizer path that keeps simple literal binary expressions opaque rather than folding them, so the release-only reduction budget remains visibly higher | `crates/kali_optimize/src/tests.rs::fast_keeps_binary_expressions_opaque`, `crates/kali_cli/tests/runtime_smoke.rs::release_build_constant_folds_literal_expressions` |
| `release` | constant folding, dead branch elimination, small-function inlining, duplicate pure-expression elimination, duplicate literal canonicalization, const object/array specialization, layout-driven specialization, and PGO-guided inlining/branching hints | `crates/kali_optimize/src/tests.rs::release_constant_folds_binary_expressions`, `::release_eliminates_constant_branches`, `::release_inlines_simple_function_calls`, `::release_eliminates_duplicate_pure_expressions_within_basic_blocks`, `::release_eliminates_duplicate_literals_within_basic_blocks`, `::release_specializes_const_object_property_access`, `::release_specializes_const_array_element_access`, `::release_specializes_array_literal_arguments_by_shape`, plus the profile-data tests in the same file |
| `release-advanced` | everything above, plus algebraic-identity simplification, division-by-one elimination, dead inlined-function pruning, and the more aggressive specialization paths used by the later inventory tests | `crates/kali_optimize/src/tests.rs::release_advanced_eliminates_algebraic_identities`, `::release_advanced_eliminates_division_by_one`, `::release_advanced_prunes_dead_inlined_functions`, `::release_advanced_allows_generic_specialization_inside_mir_specialized_clones`, `::release_advanced_reuses_generic_specializations_across_layout_specialized_owners` |

## Reading rule

- Treat this as a living inventory of what is currently evidenced, not as a promise that the optimizer is done.
- If a future packet adds a new optimization family or changes mode behavior, update this note alongside the relevant tests and plan progress.
- Performance claims should still follow the spec rule: name the workload class, build mode, and baseline.
