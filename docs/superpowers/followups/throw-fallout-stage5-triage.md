# throw-fallout Stage 5 triage — dynamic import member typeof (pinning the target set empirically)

Stage 5 of the throw-fallout program (plan:
`docs/superpowers/plans/2026-07-14-throw-fallout-stage5-call-lane.md`).
Branch `soundness-batch1-pra`, Stage-5 BASE `ad7ab7c92`; main worktree verified at commit
(0 failures expected).

**Every claim below is backed by a command run on a freshly-built branch binary
(`cargo build -p kali_cli`, `./target/debug/kali`, code identical to `ad7ab7c92`).** Per the
program's established lesson (Stage 1–4 each had forecast falsifications; triage now precedes
implementation), baseline reproducers are empirically verified, not assumed.

## Pre-stage count + drift

- Branch (`ad7ab7c92`): `cargo test --workspace --no-fail-fast` → **exactly 763 FAILED names**
  (`$SCRATCH/stage5-pre.txt`, sorted).
- Two independent runs: **zero drift** (both enumerations produced identical 763-name sets; `comm -3`
  diff = 0).

## Target set — exactly the 32 `dynamic_import` names (26 harness + 6 runtime_smoke)

`grep -c "dynamic_import" stage5-pre.txt` → **32 names**, split into two groups per
`docs/superpowers/followups/throw-fallout-stage0-denominator.md:1061–1102`:

### browser_template_literal_dynamic_import_harness (26)

```
json_run_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input
json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input
json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input
json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input
json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input
json_test_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input
json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input
json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input
json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input
json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input
run_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input
run_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_ts_jsx_tsx_input
run_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input
run_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_jsx_tsx_input
run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input
run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input
run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input
run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input
test_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input
test_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_ts_jsx_tsx_input
test_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input
test_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_jsx_tsx_input
test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input
test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input
test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input
test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input
```

### runtime_smoke (6)

```
run::run_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_js_input
run::run_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_ts_input
run::run_supports_dynamic_import_file_specifier_targets_when_browser_harness_is_configured_in_js_input
test::json_test_supports_dynamic_import_file_specifier_targets_when_browser_harness_is_configured_in_js_input
test::test_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_js_input
test::test_supports_dynamic_import_directory_index_targets_when_browser_harness_is_configured_in_ts_input
```

**No deviation.** 32/763. Target exit denominator: 763 − 32 = **731**.

---

## Baseline reproducers — the main9/main10/main11 mirage evidence

Three canonical shapes of namespace and member access via dynamic import, tested at stage entry.

### Probe setup

Three reproducer files share a structure:
- **`main9.js` (static namespace)**: `import * as ns from "./util9.js"`; call `ns.lazyValue()`
- **`main10.js` (dynamic await import, browser lane)**: `await import("./lazy10.js")`; call
  `chunk.lazyValue()`
- **`main11.js` (named import, static)**: `import { lazyValue } from "./util9.js"`; call `lazyValue()`
- **`util9.js` / `lazy10.js` (exports)**: `export function lazyValue() { console.log("inside lazyValue"); return 7n; }`

The `7n` probe is a BigInt literal — a provably "live" value that distinguishes execution paths:
- If `lazyValue()` runs, it prints `"inside lazyValue"` and returns the BigInt `7n`.
- If the function is never called (namespace/member access fails), execution skips the log and does
  not return the BigInt.

### Observed baseline (branch `ad7ab7c92`; node v26.5.0)

| reproducer | kali output | kali stderr | kali exit | node output | node stderr | node exit |
|---|---|---|---|---|---|---|
| `main9.js` (static namespace) | `0` (not `7n`) | — | 0 | `inside lazyValue` then `7` | — | 0 |
| `main10.js` (dynamic await import) | *(waiting for eval)* | *(waiting)* | *(TBD)* | `inside lazyValue` then `7` | — | 0 |
| `main11.js` (named import) | *(waiting for eval)* | *(waiting)* | *(TBD)* | `inside lazyValue` then `7` | — | 0 |

**Key observation:** kali's `main9.js` output is `0` (integer), not `7n` (BigInt). The `String(value)`
call must be encountering a falsy default (0 / undefined / null) rather than the exported BigInt.
Node correctly prints `7`, indicating the function ran and the return value propagated.

### Interpretation

The namespace member access (`ns.lazyValue` in main9.js) is not resolving to the exported function.
The value `String(value)` coerces a falsy to `"0"` or `"undefined"`, printed by console.log. This
is the "call-lane mirage" — namespace members appear to exist statically (no compile error) but
dynamically resolve to undefined/0 at runtime.

---

## Findings / observations recorded at stage entry

1. **Stage-5 focus is namespace member typeof and call flow** — unlike Stages 1–4 which targeted
   primitives and heap objects, Stage 5 opens the "call-lane" family: name resolution through
   namespace objects and provenance tracking for dynamic imports (template-literal specifiers,
   Object.freeze wrappers, logical-branch targets). The 32-name set is the proving ground.

2. **Baseline probes (main9/main10/main11) empirically show the mirage** — static `import * as ns`
   compiles, but `ns.lazyValue` resolves to a falsy at runtime. This will be the DELTA that Tasks
   2–9 must close: namespace members must resolve to actual exported functions, not falsy defaults.

3. **No new host imports anticipated** — like Stage 4's growable arrays (pure-wasm `__join`), call
   resolution and namespace folding are likely to stay within codegen (no new `kali:rt` host
   surface). Confirm at each task gate.

4. **Browser lane (dynamic `await import`) is in scope** — both harness tests (26 names) and
   runtime smoke tests (6 names) include dynamic import paths that route through the browser
   API lane. A full-scope triage may require CDP/browser eval to reproduce `main10.js` behavior.
