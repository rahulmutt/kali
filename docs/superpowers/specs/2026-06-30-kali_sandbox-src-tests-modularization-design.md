# kali_sandbox `src/tests.rs` Unit-Test Modularization (Series Entry 35)

**Date:** 2026-06-30
**Crate:** `kali_sandbox`
**Branch:** `refactor/kali_sandbox-srctests-modularization` (off local `main`)
**Integration:** local-`main` ff-merge only — **never push to origin** (origin/main intentionally lags).

## Goal

Decompose the co-located unit-test monolith `crates/kali_sandbox/src/tests.rs`
(41 `#[test]` fns, 1303 lines) into a thin **drained facade** plus four
per-concern `#[path] mod` submodules under `src/tests/`. Zero behavior change,
byte-identical test bodies, public API untouched, consumers compile unedited.

This is the 35th entry in the crate-modularization series and follows the exact
verbatim-code-motion recipe used for the prior 14+ entries (kali_optimize,
kali_types, kali_runtime, kali_codegen, kali_common, kali_mir, kali_api_web, …).
It extends the frontier from `src/*_tests.rs`-named files (now exhausted) to the
previously-missed `src/tests.rs`-named files.

## Baseline (verified)

- Declared from `lib.rs:36–38`: `#[cfg(test)]` / `#[path = "tests.rs"]` / `mod tests;` — **untouched** by this work.
- `cargo build -p kali_sandbox --tests` → **0 warnings**.
- `cargo test -p kali_sandbox --lib` → **41 passed; 0 failed**.
- **0 `include_*!` pins** in `tests.rs` (no facade pinning needed).
- 3 module-level helper fns are **not** `#[test]` and stay in the facade:
  `write_source_fixture`, `write_source_fixture_with_extension`, `valid_policy`.
  The mover leaves non-`#[test]` fns in place automatically; children reach them
  via `use super::*;` (Rust descendant-visibility re-exports the facade's private
  items through the child glob — proven at 0 warnings across prior entries).
- Facade currently retains these `use` lines (kept verbatim): `use super::*;` and
  `use std::{fs, path::PathBuf, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};`.

## Architecture

```
crates/kali_sandbox/src/
  lib.rs                         # line 36–38 decl UNCHANGED
  tests.rs                       # FACADE: use-lines + 3 helpers + 4 `#[path] mod` decls, 0 #[test]
  tests/
    policy.rs                    # use super::*; + 10 verbatim #[test] fns
    predicates.rs                # use super::*; + 12 verbatim #[test] fns
    effect_analysis.rs           # use super::*; + 10 verbatim #[test] fns
    effect_reports.rs            # use super::*; +  9 verbatim #[test] fns
```

The facade drains to **0 module-level `#[test]` fns** and appends:

```rust
#[path = "tests/policy.rs"]
mod policy;
#[path = "tests/predicates.rs"]
mod predicates;
#[path = "tests/effect_analysis.rs"]
mod effect_analysis;
#[path = "tests/effect_reports.rs"]
mod effect_reports;
```

## Grouping (all four groups partition by leading prefix)

The mover's native `name.startswith(prefix-tuple)` mode covers all 41 tests; the
four prefix-sets are mutually disjoint, so no exact-name-set mode and no `misc=*`
catch-all are required. Completeness is enforced by the drained-facade gate
(facade must reach 0 `#[test]`) and `verify.py`, not by a catch-all.

**move_fns.py groups-spec:**
```
policy=policy_;predicates=predicate_,registered_,declarative_,late_,access_;effect_analysis=effect_analysis_;effect_reports=effect_reports_,effect_inference_
```

| submodule | n | prefixes |
|-----------|--:|----------|
| `policy` | 10 | `policy_` |
| `predicates` | 12 | `predicate_`, `registered_`, `declarative_`, `late_`, `access_` |
| `effect_analysis` | 10 | `effect_analysis_` |
| `effect_reports` | 9 | `effect_reports_`, `effect_inference_` |

Exact membership (the decisive multiset; 10+12+10+9 = 41):

- **policy** — `policy_validates_and_serializes`, `policy_thread_budget_helper_preserves_zero_cap_tightening`, `policy_rejects_thread_spawn_when_no_budget_is_available`, `policy_rejects_thread_spawn_when_the_budget_is_zero`, `policy_rejects_positive_spawn_budget_before_subprocess_support_exists`, `policy_spawn_budget_helper_combines_policy_and_override`, `policy_rejects_timer_schedule_when_scheduling_is_disabled`, `policy_rejects_timer_schedule_when_the_delay_exceeds_the_policy_limit`, `policy_rejects_timer_schedule_when_the_active_timer_limit_is_reached`, `policy_rejects_unavailable_capabilities`
- **predicates** — `predicate_registry_rejects_when_disabled`, `registered_predicates_run_after_declarative_allowance`, `predicate_context_records_process_spawn_details`, `predicate_context_records_file_network_and_env_details`, `predicate_context_records_remaining_host_specific_details`, `predicate_context_records_late_process_control_details`, `predicate_context_records_thread_spawn_details`, `late_process_control_operations_remain_feature_gated`, `registered_predicates_can_inspect_host_specific_context_details`, `declarative_denials_stay_primary_over_predicates`, `registered_predicates_run_in_registration_order`, `access_rules_match_globs`
- **effect_analysis** — `effect_analysis_tracks_phase_three_deno_host_capabilities`, `effect_analysis_marks_computed_bracketed_deno_command_constructors_as_dynamic_in_js_input`, `effect_analysis_marks_computed_deno_host_access_as_dynamic`, `effect_analysis_tracks_node_process_env_assignment_in_js_input`, `effect_analysis_tracks_direct_deno_network_calls_in_js_input`, `effect_analysis_marks_computed_bracketed_deno_network_calls_as_dynamic_in_js_input`, `effect_analysis_marks_computed_bracketed_deno_env_read_as_dynamic`, `effect_analysis_tracks_deno_env_to_object_as_env_read`, `effect_analysis_tracks_bracketed_deno_env_to_object_as_dynamic_env_read`, `effect_analysis_marks_proxy_constructor_and_revocable_calls_as_dynamic`
- **effect_reports** — `effect_reports_normalize_dynamic_reasons_and_analysis_context_axes`, `effect_reports_trim_and_deduplicate_semantic_axes_before_serialization`, `effect_inference_deduplicates_repeated_roots_before_serialization`, `effect_reports_deduplicate_entry_points_while_preserving_first_seen_order`, `effect_reports_treat_permissions_query_as_effect_free`, `effect_reports_treat_computed_permissions_query_as_effect_free`, `effect_reports_treat_permissions_query_as_effect_free_in_js_input`, `effect_reports_treat_computed_permissions_query_as_effect_free_in_js_input`, `effect_reports_sort_effect_groups_and_locations_deterministically`

## Method

Pure verbatim code-motion via the series' reusable tools (git-ignored scratch
under `.superpowers/sdd/`):

- **Task 0** establishes the toolchain for this entry: confirm `move_fns.py` is in
  leading-prefix mode (`group_for` uses `fn_name.startswith(prefs)`) and the
  matching `verify.py`; keep `FN_RE` / `IDENT_CHARS` / `find_close_line`
  byte-identical. Capture the baseline `--list` test-name set.
- **One task per submodule** (4 tasks): run the mover for the group, build+test gate.
  Implementer = haiku/sonnet (pure command-transcription); per-task review = sonnet.
- Bodies move **verbatim** — no `cargo fmt`, no path rewrites, no `pub`-widening,
  no `include_*!` changes. Nested helper fns inside a test body travel with their
  parent test.

## Invariants / gates (literal — this crate's baseline is clean)

1. `cargo build -p kali_sandbox --tests` → **0 warnings** (held at baseline and on merged main).
2. `cargo test -p kali_sandbox --lib` → **41 passed; 0 failed** (held throughout).
3. Facade `src/tests.rs` drains to **0 module-level `#[test]` fns** (3 helpers retained).
4. `verify.py` proves `{name: body}` extracted from `src/tests.rs@base` ==
   union of the four submodules — 41/41 bodies byte-identical, disjoint namespaces,
   0 collisions, net new lines = scaffold only (`use super::*;` + blank per file +
   4 `#[path]`/`mod` pairs).
5. `lib.rs:36–38` decl unchanged; no production `.rs` file touched; `kali_sandbox`
   consumers compile unedited.

## Out of scope

- The smaller `tests.rs` files in other crates (kali_embed 20, kali_lir 11, …) —
  future series entries.
- `kali_parser/tests/parser_integration.rs` integration monolith — different track.
- Any production-`src` refactor (kali_sandbox production was split in the 17th entry).

## Completion

Per-task reviews + opus whole-branch review → 0 findings; re-verify on merged
`main`; ff-merge to local `main`; delete branch; **do not push origin**. Update the
`crate-modularization-series` memory with the 35th entry.
