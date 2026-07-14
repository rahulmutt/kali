# throw-fallout Stage 5 triage — dynamic import member typeof (pinning the target set empirically)

Stage 5 of the throw-fallout program (plan:
`docs/superpowers/plans/2026-07-14-throw-fallout-stage5-dynamic-import-typeof.md`).
Branch `soundness-batch1-pra`, Stage-5 BASE `ad7ab7c92` — this is the stage-entry HEAD commit the
snapshot below was taken at (this triage doc's own commit necessarily lands on top of it).
**Main-worktree cross-check is deferred to the Task-9 checkpoint gate per the plan** (that task
runs the full-workspace enumeration against a main worktree, not Task 1) — it is NOT measured
here and no "0 failures" claim is made for main at this stage.

The reproducers and counts below were each verified by running a command on a freshly-built
branch binary (`cargo build -p kali_cli`, `./target/debug/kali`, code identical to `ad7ab7c92`
— the working tree was clean at HEAD `ad7ab7c92` for every run recorded in this doc). Per the
program's established lesson (Stage 1–4 each had forecast falsifications; triage now precedes
implementation), baseline reproducers are empirically verified, not assumed — see the "Baseline
reproducers" section for the four kali-vs-node comparisons actually run (main9/main10/main11/main12)
and the "Pre-stage count" section for the denominator reconciliation, including isolation-run
evidence for every disputed name.

## Pre-stage count + drift

- Branch (`ad7ab7c92`): `cargo test --workspace --no-fail-fast` → **763 unique FAILED names**
  after dedup (`$SCRATCH/stage5-pre.txt`, sorted, `sort -u`).
- Two independent fresh-binary runs (`stage5-pre-run1-new.txt` / `stage5-pre-run2-new.txt`):
  both raw captures are 763 lines, already unique internally (no dupes) — `comm -3` diff = 0.
- **Denominator reconciliation (763 vs 781):** an *earlier* pair of enumeration runs, captured
  before the "-new" pair, is also on disk in `$SCRATCH` (`stage5-pre-run1.txt` /
  `stage5-pre-run2.txt`, 781 raw lines each). See "Denominator reconciliation" below for the
  full investigation — the short version: 781 vs 763 is not a set-membership drift (deduped sets
  are byte-identical, `comm -13` both directions = empty), it is 18 test names that legitimately
  exist as identically-named `#[test]` functions in two different test binaries, so a single
  `cargo test --workspace` capture can show the same text line once (dedup-invisible collision)
  or twice (both binaries' FAILED lines survive output interleaving) depending on run-to-run
  buffering. All 18 names were isolation-verified RED in both owning binaries (2/2 stable each),
  so they correctly belong in the 763-name set either way — no exclusion applies.

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

## Runnable enumeration pipeline

House style per `docs/superpowers/followups/throw-fallout-stage4-triage.md:294-303`: capture two
independent full-workspace enumerations on a freshly-built binary, diff them for interleaving
noise, then union into the canonical sorted set.

```bash
cd /workspace && cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort > "$SCRATCH/stage5-pre-run1.txt"
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort > "$SCRATCH/stage5-pre-run2.txt"
comm -3 "$SCRATCH/stage5-pre-run1.txt" "$SCRATCH/stage5-pre-run2.txt"   # interleaving noise only

sort -u "$SCRATCH/stage5-pre-run1.txt" "$SCRATCH/stage5-pre-run2.txt" > "$SCRATCH/stage5-pre.txt"
wc -l "$SCRATCH/stage5-pre.txt"
grep -c "dynamic_import" "$SCRATCH/stage5-pre.txt"   # expect 32
```

## Denominator reconciliation (763 vs 781 vs plan-expected ~783)

`$SCRATCH` holds two enumeration pairs from this task's work: an earlier pair
(`stage5-pre-run1.txt` / `stage5-pre-run2.txt`, **781 raw lines each**) and a later, fresh-binary
pair (`stage5-pre-run1-new.txt` / `stage5-pre-run2-new.txt`, **763 raw lines each**). The
committed `stage5-pre.txt` is the union of the "-new" pair. This section shows the two pairs are
not actually in disagreement about which tests are failing.

**Step 1 — exact delta between the two pairs' deduped (unique) sets:**

```bash
comm -13 <(sort -u "$SCRATCH/stage5-pre-run1-new.txt" "$SCRATCH/stage5-pre-run2-new.txt") \
         <(sort -u "$SCRATCH/stage5-pre-run1.txt" "$SCRATCH/stage5-pre-run2.txt")
# -> 0 lines (nothing in the old-pair unique set is missing from the new-pair unique set)
comm -13 <(sort -u "$SCRATCH/stage5-pre-run1.txt" "$SCRATCH/stage5-pre-run2.txt") \
         <(sort -u "$SCRATCH/stage5-pre-run1-new.txt" "$SCRATCH/stage5-pre-run2-new.txt")
# -> 0 lines (nothing in the new-pair unique set is missing from the old-pair unique set)
```

Both directions are **empty**. The deduped (`sort -u`) unique-name sets from both pairs are
byte-identical at **763 names**. So the 781-vs-763 gap is not a membership drift at all — it
lives entirely inside `sort -u` collapsing something that raw `sort` does not.

**Step 2 — where the extra 18 raw lines in the old pair come from:**

```bash
sort "$SCRATCH/stage5-pre-run1.txt" | uniq -c | awk '$1>1'
```

18 test names appear **twice** in each raw old-pair file (781 raw − 18 dupes = 763 unique); the
new-pair raw files have zero internal dupes (763 raw = 763 unique). All 18 are
`string_primitive_iteration`/`object_values_spread_iteration` browser-harness names, e.g.
`test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_tsx_input` and
`test_supports_object_values_spread_iteration_when_browser_harness_is_configured`.

Reading the test sources explains why: these are not coincidental duplicate strings — they are
**identically-named `#[test]` functions defined in two different integration-test binaries**:

- All 16 `*string_primitive_iteration*` names exist verbatim in both
  `crates/kali_cli/tests/browser_for_await_object_string_enumeration_harness.rs` and
  `crates/kali_cli/tests/browser_object_string_enumeration_harness.rs`.
- Both 2 `*object_values_spread_iteration*` names exist verbatim in both
  `crates/kali_cli/tests/browser_object_values_harness.rs` and
  `crates/kali_cli/tests/browser_object_values_spread_harness.rs`.

`cargo test --workspace` runs each integration-test file as its own binary, and the
`test <name> ... FAILED` line format does not include the binary name — so when both binaries'
copies fail, `grep`+`sed` produces the *same text line twice* in one enumeration's raw output;
`sort -u` correctly collapses that to one name. Whether a given raw capture shows the line once
or twice is a function of which binary's output survived concurrent-process interleaving that
run, not of whether the test is actually failing.

**Step 3 — isolation-run every one of the 18 names, in both owning binaries, 2x each, for true
current state** (fresh binary, `--exact --test-threads=4`):

| name group | binary | run 1 | run 2 | verdict |
|---|---|---|---|---|
| 16× `*string_primitive_iteration*` | `browser_for_await_object_string_enumeration_harness` | 16/16 FAILED | 16/16 FAILED | RED 2/2 |
| 16× `*string_primitive_iteration*` | `browser_object_string_enumeration_harness` | 16/16 FAILED | 16/16 FAILED | RED 2/2 |
| 2× `*object_values_spread_iteration*` | `browser_object_values_harness` | 2/2 FAILED | 2/2 FAILED | RED 2/2 |
| 2× `*object_values_spread_iteration*` | `browser_object_values_spread_harness` | 2/2 FAILED | 2/2 FAILED | RED 2/2 |

Commands run (per binary):

```bash
cargo test -p kali_cli --test browser_for_await_object_string_enumeration_harness \
  -- --exact --test-threads=4 <16 names>
cargo test -p kali_cli --test browser_object_string_enumeration_harness \
  -- --exact --test-threads=4 <16 names>
cargo test -p kali_cli --test browser_object_values_harness \
  -- --exact --test-threads=4 <2 names>
cargo test -p kali_cli --test browser_object_values_spread_harness \
  -- --exact --test-threads=4 <2 names>
```

Every one of the 18 disputed names is RED in **both** of its owning binaries, stably across 2
runs each (8 isolation invocations total, 0 flips). No name is a load/interleaving false-positive
in the "these tests don't actually fail" sense — the interleaving artifact is purely about
duplicate *line rendering* within one `cargo test --workspace` capture, not about whether the
underlying tests fail.

**Decision rule applied:** a delta name GREEN in isolation (2/2) would stay OUT of
`stage5-pre.txt` (enumeration failure would be a load/interleaving artifact, fail-safe to drop —
a later real regression would be caught by the Task-9 gate and triaged then). A delta name RED in
isolation goes IN. All 18 names here are RED in isolation, so all 18 belong in the set — and they
already are: the committed `stage5-pre.txt` (union of the "-new" pair, `sort -u`'d) already
contains all 18 as single entries, since dedup makes the two-binaries-one-name collision moot for
set membership. **No rebuild of `stage5-pre.txt` was required** — the existing file is correct.

**Final denominator: 763** (verified: `wc -l "$SCRATCH/stage5-pre.txt"` → 763;
`grep -c "dynamic_import" "$SCRATCH/stage5-pre.txt"` → 32).

**Honest note on the plan's ~783 expectation:** the Stage-5 implementation plan projected the
Stage-4 exit count (783) would hold at Stage-5 entry. The measured stage-entry count is 763, a
drop of 20. This was not independently re-derived in this task (Task 1's job is to capture and
reconcile the *current* snapshot, not to explain every historical drift since Stage 4's exit
gate) — plausible contributors include ordinary test-suite drift between the Stage-4 checkpoint
commit and `ad7ab7c92` (intervening commits: `1fecb7839`, `5b763bf66`, both docs-only per `git
show --stat`, so a code-driven drop is not expected from those two commits specifically) and
count-methodology differences (Stage 4's 783 was itself flagged as "50 real + 1
output-interleaving false-drain" in its own checkpoint doc). This discrepancy is flagged, not
resolved, here; Task 9's main-worktree cross-check is the authoritative reconciliation point.

---

## Baseline reproducers — the main9/main10/main11/main12 mirage evidence

Four canonical shapes of namespace and member access via dynamic/static import, all reproduced and
run (kali + node) at stage entry. Scratch files under `$SCRATCH/dyn5/` are not committed; their
content is recorded inline below.

### Probe setup

Four reproducer files share a structure:
- **`main9.js` (static namespace)**: `import * as ns from "./util9.js"`; call `ns.lazyValue()`
  ```js
  import * as ns from "./util9.js";
  const value = ns.lazyValue();
  console.log(String(value));
  console.log("main loaded");
  ```
- **`main10.js` (dynamic await import, static string specifier, browser lane)**:
  `await import("./lazy10.js")`; call `chunk.lazyValue()`
  ```js
  async function main() {
    const chunk = await import("./lazy10.js");
    const value = await chunk.lazyValue();
    console.log(String(value));
    console.log("main loaded");
  }
  main();
  ```
- **`main11.js` (static **named** import — NOT a dynamic/template-literal import)**:
  `import { lazyValue } from "./util9.js"`; call `lazyValue()` directly
  ```js
  import { lazyValue } from "./util9.js";
  const value = lazyValue();
  console.log(String(value));
  console.log("main loaded");
  ```
- **`main12.js` (dynamic await import, template-literal specifier — the dominant bucket shape;
  26 of the 32 stage-5 target names are template-literal-harness tests)**:
  `await import(\`./${name}\`)`; call `chunk.lazyValue()`
  ```js
  async function main() {
    const name = "lazy10.js";
    const chunk = await import(`./${name}`);
    console.log(String(await chunk.lazyValue()));
    console.log("main loaded");
  }
  main();
  ```
- **`util9.js` / `lazy10.js` (exports, both already matched the required `7n`-probe shape — no
  adaptation needed)**:
  `export function lazyValue() { console.log("inside lazyValue"); return 7n; }`

The `7n` probe is a BigInt literal — a provably "live" value that distinguishes execution paths:
- If `lazyValue()` runs, it prints `"inside lazyValue"` and returns the BigInt `7n`.
- If the function is never called (namespace/member access fails), execution skips the log and does
  not return the BigInt.

### Observed baseline (branch `ad7ab7c92`; node v26.5.0)

Commands run for each kali cell (from `$SCRATCH/dyn5`):
- main9/main11 (no dynamic import, no browser lane needed): `/workspace/target/debug/kali run <file>`
- main10/main12 (dynamic `await import`, in-scope for the browser lane per the 32-target bucket):
  both `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node /workspace/target/debug/kali run --api browser
  --max-threads 0 --max-spawned-processes 0 <file>` (browser lane) AND plain
  `/workspace/target/debug/kali run <file>` (recorded separately for main12).
- node baseline: `node <file>` for all four.

| reproducer | kali output (mode) | kali stderr | kali exit | node output | node stderr | node exit |
|---|---|---|---|---|---|---|
| `main9.js` (static namespace) | `0`\n`main loaded` | — | 0 | `inside lazyValue`\n`7`\n`main loaded` | — | 0 |
| `main10.js` (dynamic import, browser lane) | `0`\n`main loaded` (browser lane) | — | 0 | `inside lazyValue`\n`7`\n`main loaded` | — | 0 |
| `main11.js` (static named import) | `0`\n`main loaded` (plain `run`) | — | 0 | `inside lazyValue`\n`7`\n`main loaded` | — | 0 |
| `main12.js` (dynamic import, template-literal specifier) | `0`\n`main loaded` (browser lane) | — | 0 | `inside lazyValue`\n`7`\n`main loaded` | — | 0 |
| `main12.js` (dynamic import, template-literal specifier) | `0`\n`main loaded` (plain `run`, non-browser) | — | 0 | *(same as above)* | — | — |

All four kali cells were run once each (main9 was previously recorded; this task re-ran it as a
spot check and got the identical result). main12 was additionally run under both the browser lane
and plain `kali run` — both modes produce the identical mirage (`0` instead of `7`), confirming
the mirage is not browser-lane-specific.

**Key observation:** kali's output is `0` (integer) in every reproducer, not `7n`/`7` (BigInt).
The `String(value)` call must be encountering a falsy default (0 / undefined / null) rather than
the exported BigInt, across static namespace access (main9), dynamic static-specifier import
(main10), static named import (main11), and dynamic template-literal-specifier import (main12) —
i.e. the mirage reproduces identically regardless of import shape, confirming this is a general
namespace/member-resolution gap rather than something specific to one import syntax. Node
correctly prints `7` in every case, indicating the function ran and the return value propagated.

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

2. **Baseline probes (main9/main10/main11/main12) empirically show the mirage** — static
   `import * as ns`, dynamic static-specifier import, static named import, and dynamic
   template-literal-specifier import all compile, but member/name access resolves to a falsy at
   runtime in every shape. This will be the DELTA that Tasks 2–9 must close: namespace/named
   members must resolve to actual exported functions, not falsy defaults.

3. **No new host imports anticipated** — like Stage 4's growable arrays (pure-wasm `__join`), call
   resolution and namespace folding are likely to stay within codegen (no new `kali:rt` host
   surface). Confirm at each task gate.

4. **Browser lane (dynamic `await import`) is in scope** — both harness tests (26 names) and
   runtime smoke tests (6 names) include dynamic import paths that route through the browser
   API lane. `main10.js`/`main12.js` were reproduced via `kali run --api browser
   --max-threads 0 --max-spawned-processes 0` with `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`
   (the same node-backed browser-lane harness the test suite uses) — no CDP/real-browser eval
   was needed for this triage; the mirage reproduces identically to the plain (non-browser) lane.
