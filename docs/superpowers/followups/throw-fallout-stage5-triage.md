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

---

## Task 2 — generic `typeof` fallback measured fail-closed (E5506), then REVERTED

**Final status: REVERTED.** This section originally documented a flip-and-keep. The flip was
implemented, census-measured (8 newly-red, 0 newly-green — evidence retained below as the
deferred follow-up's sizing data), and — per the Task-2 decision rule ("measure, close if
cheap, otherwise revert") — reverted once neither the implementer nor an independent reviewer
found a same-stage cheap fix for any of the three newly-red buckets. See "Decision: REVERTED"
below for the full rationale. The "what changed" writeup immediately below describes the flip
AS IT WAS WHILE LIVE, for evidentiary/historical purposes only — it does NOT describe the
current state of `operators.rs`, which is back to its pre-Task-2 form (verified byte-identical
to `e2bd098b5~1`).

### What changed (historical — while the flip was live)

`crates/kali_codegen/src/emit/operators.rs` `"typeof"` arm, final fallback only: the generic
warning (`e8::UNIMPLEMENTED`, "unsupported unary operator 'typeof'") + silent `I64Const(0)`
placeholder was made a **compile error**:

> `error[E5506]: typeof is only supported on statically-provable operands in the current
> direct-runtime path (this operand's type cannot be proven; a silent placeholder would
> miscompile comparisons)`

The placeholder instruction was still emitted so the wasm stayed structurally valid (mirrors the
`delete` default-deny arm's handling); the error diagnostic failed the build at
`kali_cli::build_source_file` (`has_errors` ⇒ `Err`, no artifact). The `delete`/`void` arms and
every other operator's fallback were untouched. Reproducer test (red-first):
`unsupported_typeof_operand_rejects_unproven_member_read` in
`crates/kali_codegen/src/emit/operators_tests.rs` — RED under the pre-flip code (two E8001
warnings, successful compile), GREEN while the flip was live. `cargo test -p kali_codegen` while
live: 356 passed, 0 failed. **Now that the flip is reverted, this test is kept but marked
`#[ignore = "generic typeof fail-open closure deferred; census attached in stage5 triage"]`**
(assertions untouched, ready to un-ignore when a real fix lands) — current
`cargo test -p kali_codegen`: 355 passed, 0 failed, 1 ignored.

### Census (full workspace, fresh binary)

```bash
cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage5-typeof-census.txt"        # NOTE sort -u (Task-1 dupe pitfall)
comm -13 "$SCRATCH/stage5-pre.txt" "$SCRATCH/stage5-typeof-census.txt"   # newly-red
comm -23 "$SCRATCH/stage5-pre.txt" "$SCRATCH/stage5-typeof-census.txt"   # newly-green
```

- `stage5-typeof-census.txt`: **771** unique FAILED names (baseline 763).
- **Newly-red: 8. Newly-green: 0.** (763 + 8 = 771, exact.)

All 8 isolation-run 2/2 RED (two full passes over all three owning binaries, `--exact`); every
failure message contains the new E5506 text verbatim, so attribution to this flip is direct.

### Newly-red names, mechanism per bucket

**Bucket A — `package_corpus` browser corpus, 3 names** (RED 2/2 in isolation):

```
browser_corpus::browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host
browser_corpus::browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host_on_js_input
browser_corpus::browser_corpus_packages_with_web_baseline_primitives_remain_checkable_and_deployable_through_host_on_js_input_when_the_browser_api_surface_is_inherited
```

Mechanism: the shared web-baseline interop fixture (`write_web_baseline_interop_source`,
`crates/kali_cli/tests/package_corpus.rs:201`) contains the canonical feature-detection idiom
`if (typeof indexedDB !== 'undefined') { ... }`. `indexedDB` is an identifier that resolves to
nothing in kali (its guard body's calls all hit the W3100/E3100 zero-placeholder lane), so
`typeof indexedDB` is unproven → E5506 → `kali check`/`kali build --bundle` now fail; the tests
pin build success for ~70 corpus packages. **This was a LIVE miscompile before the flip**: the
placeholder `0` compares `!== "undefined"` as TRUE, so kali took the guard branch node skips —
the exact bucket-#7 wrong-branch class, previously invisible because the test only pinned
buildability.

**Bucket B — `runtime_smoke` build, 3 names** (RED 2/2 in isolation):

```
build::build_emits_browser_bundle_object_type_and_constructor_semantics_in_js_input
build::build_emits_browser_bundle_object_type_and_constructor_semantics_in_json_output
build::build_emits_browser_bundle_object_type_and_constructor_semantics_in_ts_input
```

Mechanism: fixture (`browser_bundle_object_type_and_constructor_semantics_source`,
`crates/kali_cli/tests/runtime_smoke.rs:3807`) does `typeof box` (where `box = new Box()`) and
`typeof Box` (a module-level `function Box() {}` declaration) — two unproven operands, E5506
fires twice per build. (`typeof null` in the same fixture is already provable → "object".) The
tests pin "bundle build must succeed" (evaluation-trap layering for `in`/`instanceof`); the
typeof operands rode along on the fail-open lane.

**Bucket C — `node_api_surface` library builds, 2 names** (RED 2/2 in isolation):

```
explicit::explicit_node_api_surface_builds_library_artifacts_in_js_input
inherited::inherited_node_api_surface_builds_library_artifacts_in_js_input
```

Mechanism: fixture (`crates/kali_cli/tests/node_api_surface/{explicit,inherited}.rs`) is
`import * as path from 'node:path'; import * as timers from 'node:timers';` +
`typeof path.basename === 'function' && typeof timers.clearInterval === 'function' ? 0 : 1`.
The namespaces are unresolved at codegen (W3100 "undefined identifier 'path'/'timers'"), so the
member-read typeof operands are unproven → E5506 ×2 → the `--lib` build fails; the tests pin
build success. This is precisely the Stage-5 namespace-member-typeof shape, but over **node
builtins**, which the Task-6 AST module-link rewrite (user modules with real files) will NOT
cover. Note the old behavior was also a silent lie: placeholder `0 !== "function"` ⇒ `describe()`
returned 1 where node returns 0.

### Decision: **REVERTED** (rule applied — final)

The flip was measured (8 newly-red, 0 newly-green, mechanism per bucket below), then reverted.
`crates/kali_codegen/src/emit/operators.rs`'s `"typeof"` arm final fallback is restored to its
pre-Task-2 form (generic `e8::UNIMPLEMENTED` warning + operand-eval/`Drop` + `I64Const(0)`
placeholder), byte-identical to `e2bd098b5~1`. The reproducer test
`unsupported_typeof_operand_rejects_unproven_member_read` in
`crates/kali_codegen/src/emit/operators_tests.rs` is kept but marked
`#[ignore = "generic typeof fail-open closure deferred; census attached in stage5 triage"]` —
its assertions are untouched so it can be un-ignored the moment a real fix lands.

**Why revert, not keep-and-fix-later:**

- (a) **Census is the sizing evidence, not an in-stage obligation.** Newly-red = 8, all
  attributable to exactly one mechanism (unproven `typeof` operand that used to silently
  fall through to a `0` placeholder — see the three bucket writeups below, unchanged). This
  sizing data is preserved here for whoever picks up the deferred follow-up; it is NOT being
  treated as something Stage 5 must clear.
- (b) **No cheap provable-lane extension exists for any bucket.** Bucket A needs a resolver
  oracle for unresolvable-identifier-as-"undefined" wired at the identifier-resolution choke
  point (default-deny, not a hand-mirrored predicate — see typeof-F1). Bucket B needs two new
  operand classifications (source-function-declaration → "function", `new F()` with a
  non-function-returning constructor → "object" — typeof-F2/F3) that `typeof_static_text`
  structurally cannot reach without new machinery. Neither implementer nor reviewer judged
  either a same-stage one-liner.
- (c) **Bucket C would force a capability regression, not just an unlanded nice-to-have.**
  The only landable option for `typeof path.basename` / `typeof timers.clearInterval` over
  unresolved `node:*` builtin namespaces is to re-pin the two `node_api_surface` tests to
  expect a fail-closed E5506 reject — but that makes kali unable to BUILD a realistic node
  library fixture that exercises this exact (very common) feature-detection idiom. Trading a
  passing "kali can build this library" test for a failing one is a real regression in what
  kali can do, not a neutral test-suite edit, so it was rejected as the in-stage move.
- **Net effect of reverting:** the 8 tests return to green (their pre-flip state) by construction
  — the fallback is textually identical to what they were passing against before Task 2's
  commit. Verified directly (see the task-2-revert-report for exact commands/output): all 8
  green in isolation, plus `cargo test -p kali_codegen` 355 passed/0 failed/1 ignored.

**bucket A's `typeof indexedDB` remains a LIVE, UNCLOSED wrong-branch miscompile** — this is
the single highest-value item in the deferred follow-up. Pre-flip (and now again, post-revert),
`typeof indexedDB !== 'undefined'` compiles to a silent `I64Const(0)` placeholder that compares
`!== "undefined"` as **true**, so kali takes the guard's "feature present" branch when the
correct branch is "feature absent" — a real behavioral divergence from node with no compile-time
signal, not merely an unimplemented feature. typeof-F1 (unresolvable bare identifier →
`"undefined"`) is the fix that closes it; it was not implemented in Stage 5 because it needs the
identifier-resolution choke-point treatment described below, not a codegen one-liner.

**Stage 5's own namespace-member-typeof surface is unaffected by this revert.** `typeof
ns.member` / `typeof chunk.member` over a Task-6-linked module namespace is folded to a string
literal AT THE AST LEVEL by `rewrite_namespace_uses` / `try_fold_typeof_namespace_member`
(`crates/kali_cli/src/build/module_link.rs`) BEFORE codegen ever sees a `typeof` node for that
operand — so the generic codegen fallback (kept vs. reverted) never executes on Stage 5's own
target set. This is structural, not incidental: check 3 in the verification contract
(`module_namespace_link` 11/11, `browser_template_literal_dynamic_import_harness` 26/0,
`runtime_smoke dynamic_import` 45/0, all unchanged after the revert) is the direct proof — had
Stage 5's typeof lane secretly depended on the codegen flip, reverting it would have turned some
of those 82 tests red, and none did.

### Fix-or-extend items (deferred follow-up scope — NOT Stage-5 obligations)

1. **[typeof-F1] Unresolvable bare identifier → `"undefined"`** (closes bucket A, 3 names).
   JS-correct (`typeof undeclared` is `"undefined"`, the one non-throwing undeclared read) and
   truthful in kali's closed world (the same fall-through that today emits W3100 + placeholder
   proves the name resolves to nothing). MUST be implemented at the identifier-resolution
   choke point / by reusing the actual fall-through conclusion — NOT a hand-mirrored predicate
   (Spec-2 lesson: mirrored oracles fail open; Spec-4a lesson: allowlist at the single read
   site). Default-deny: locals, fold-bindings, module bindings/globals, function names, host
   globals all win first.
2. **[typeof-F2] Identifier resolving to a source-level function declaration → `"function"`**
   (half of bucket B). Guards: not shadowed by local/binding/module binding/global slot;
   allowlist source-declared functions only (the `functions` map also contains synthetics
   (`__join*`, `__streq`, …) and monomorphized `f${N}` clones — classify from the LIR
   function-declaration node, not map membership).
3. **[typeof-F3] `new F()` result → `"object"`** (other half of bucket B), gated on `F`
   resolving to a source function declaration whose body provably cannot return a function
   (constructor returning a non-function object still yields "object"; returning a primitive is
   ignored by `new`; only a function-returning constructor breaks the classification).
   Conservative first cut: no `return <expr>` at all (the fixture's `function Box() {}`
   qualifies).
4. **[typeof-F4] Bucket C decision needed** (2 names): `typeof <member>` over an unsupported
   node-builtin namespace cannot be proven without implementing the builtin. Options:
   (a) re-pin both tests to expect fail-closed E5506 (house precedent: Stage-2 Lane-C `delete`
   re-pins in the same test families) — trivially landable and honest, but this is precisely the
   option Task 2 rejected as a build-capability regression (kali could no longer build a
   realistic node-library fixture that uses this common feature-detection idiom), which is why
   Task 2 reverted rather than taking this path itself;
   (b) fold node-builtin namespaces into the Task-6 module-link design as a synthetic module
   surface, so `typeof path.basename` resolves the same way a real linked module's member would
   — preserves buildability. (b) is the only option that avoids a capability regression, and is
   the recommended starting point for the follow-up. These 2 names are NOT covered by items 1–3
   (the base identifiers are namespace imports, not undeclared globals — classifying them
   "undefined" would trade one silent node-divergence for another).

### Artifacts

- `$SCRATCH/stage5-typeof-census.txt` (771), `$SCRATCH/stage5-typeof-newlyred.txt` (8).
- Newly-red list = exactly the 8 names above; newly-green = 0.

---

## Task 8 — distinguishable-value acceptance suite + adversarial re-mask probes

### What was added

`crates/kali_cli/tests/module_namespace_link.rs` extended from 2 tests (Task-7 dependency-order
pins) to 11 (2 + 9 new): 5 GREEN distinguishable-value tests (each proves a real call into the linked
module's body ran and its actual return value propagated — not the pre-stage fail-open `0`),
3 REJECT tests (E5506, exit != 0), and 1 GREEN guard (unchanged pre-existing statement-form
behavior). All 5 GREEN tests additionally run the identical fixture under a real `node` and
assert kali's stdout byte-matches node's — every fixture used is plain, valid, unmodified ES.

Per the controller's mandatory fixture override (superseding the task-8 brief): fixtures return
a plain `Number` (`return 7`, not `7n`) and call the linked function DIRECTLY at the
`console.log` site (never through an intermediate `const`), to route around two pre-existing,
unrelated kali codegen bugs — `String(<bigint>)` folds to `0`, and a `const` bound to a call
re-evaluates the call at every use (duplicating side effects) — that would otherwise corrupt the
expected output for reasons having nothing to do with this stage.

### Test results (fresh binary, `cargo build -p kali_cli` at working-tree HEAD)

```
cargo test -p kali_cli --test module_namespace_link -- --test-threads=4
```
→ **11 passed; 0 failed** (the file's own count: the 2 pre-existing Task-7 pins +
9 new Task-8 tests — the brief listed exactly 5 GREEN + 3 REJECT + 1 GREEN-guard = 9; final file
has 11 tests total). Full list: `impure_target_module_rejected`, `namespace_value_leak_rejected`,
`non_export_member_call_rejected`, `run_supports_namespace_linked_sibling_call_export_declared_before_helper`,
`run_supports_namespace_linked_sibling_call_helper_declared_before_export`,
`statement_form_side_effect_import_stays_green`, `static_namespace_member_call_runs_body_and_returns_value`,
`two_modules_same_export_name_route_to_respective_bodies`,
`dynamic_import_template_literal_specifier_variant`, `dynamic_import_member_call_runs_body_and_returns_value`,
`typeof_missing_member_is_undefined_string` — all `ok`.

Every GREEN test's kali stdout was verified byte-identical to a real `node` run of the same
fixture (asserted in-test, not just eyeballed) — e.g. `static_namespace_member_call_runs_body_and_returns_value`
and node both print `inside lazyValue\n7\nmain loaded\n`.

Verification contract (all four commands, exact expected results, all met):

```
cargo test -p kali_cli --test module_namespace_link -- --test-threads=4       # 11 passed, 0 failed
cargo test -p kali_cli --test runtime_smoke dynamic_import -- --test-threads=4 # 45 passed, 0 failed
cargo test -p kali_cli --test browser_template_literal_dynamic_import_harness -- --test-threads=4 # 26 passed, 0 failed
cargo fmt --check -p kali_cli                                                  # clean, no output
```

### Adversarial re-mask probe 1 — typeof fold always "undefined"

**Sabotage** (`crates/kali_cli/src/build/module_link.rs`, `try_fold_typeof_namespace_member`):
replaced the real `module.exports.contains_key(...)` branch with an unconditional `"undefined"`
literal (kept the map lookup as a discarded `let _ = ...;` so the diff was a single-purpose
behavioral change, not a structural one).

**Rebuild:** `cargo build -p kali_cli` → succeeds (no type errors; this is a pure logic
regression, not a compile break — exactly the shape a review could miss without a live oracle).

**Command:** `/workspace/target/debug/kali run main.js` against the
`static_namespace_member_call_runs_body_and_returns_value` fixture (which asserts
`typeof ns.lazyValue !== 'function'` and throws `Error('missing lazyValue export')` if so).

**Observed failure (honest trap fired, as required):**
```
Uncaught Error: missing lazyValue export
error[E4000]: runtime trap (unreachable — allocation failure or an unsupported-path guard): error while executing at wasm backtrace:
    0:  0x479 - <unknown>!<wasm function 22>
exit=1
```

**Full-file confirmation:** `cargo test -p kali_cli --test module_namespace_link -- --test-threads=4`
→ **7 passed; 4 failed** under sabotage. The 4 failures were exactly the distinguishable tests
that depend on a correct typeof fold: `static_namespace_member_call_runs_body_and_returns_value`,
`dynamic_import_member_call_runs_body_and_returns_value`,
`dynamic_import_template_literal_specifier_variant` (all three guard on `typeof ns.lazyValue`/
`typeof chunk.lazyValue` before calling), and `typeof_missing_member_is_undefined_string` (asserted
`"undefined\nfunction\n"`, got `"undefined\nundefined\n"` — the "real export" branch silently
became indistinguishable from "missing member").

**Revert:** restored the original `if module.exports.contains_key(&member.property) { "function" }
else { "undefined" }` branch. `git diff --stat crates/kali_cli/src/build/module_link.rs` → empty
(byte-identical to pre-sabotage) after revert.

### Adversarial re-mask probe 2 — `append_linked_functions` skips appending

**Sabotage** (`crates/kali_cli/src/build/module_link.rs`, `append_linked_functions`): replaced the
final `statements.splice(0..0, cloned); Ok(())` with `let _ = cloned; Ok(())` — every fallible step
(collision guard, sibling rename, topo sort) still runs and still succeeds, but the mangled clones
are computed and then discarded instead of spliced into `statements`. Every namespace call site is
still rewritten to `__link{N}_{name}` by `rewrite_namespace_uses` (that pass runs downstream,
unaffected by this sabotage) — so the rewritten calls now target a mangled name that was never
declared anywhere in the program.

**Rebuild:** `cargo build -p kali_cli` → succeeds.

**Command:** `/workspace/target/debug/kali run main.js` against the same
`static_namespace_member_call_runs_body_and_returns_value` fixture.

**Observed failure (as required — no `inside lazyValue` in stdout; resolver catches the dangling reference):**
```
error[E3100]: undefined identifier '__link0_lazyValue'
  = help: declare the name in the current module or import it
exit=1
```

**Full-file confirmation:** `cargo test -p kali_cli --test module_namespace_link -- --test-threads=4`
→ **5 passed; 6 failed** under sabotage. Failures: both pre-existing Task-7 dependency-order pins
(`run_supports_namespace_linked_sibling_call_export_declared_before_helper`,
`run_supports_namespace_linked_sibling_call_helper_declared_before_export` — also depend on the
append), plus 4 Task-8 distinguishable tests that call through a linked namespace
(`static_namespace_member_call_runs_body_and_returns_value`,
`dynamic_import_member_call_runs_body_and_returns_value`,
`dynamic_import_template_literal_specifier_variant`,
`two_modules_same_export_name_route_to_respective_bodies`). The 3 REJECT tests, the typeof-only
test, and the statement-form GREEN guard correctly stayed green (none of them depend on a
successful append) — confirming the probe's failure surface is precisely "genuine linked calls",
not a blanket build break that would mask the finding's precision.

**Revert:** restored `statements.splice(0..0, cloned); Ok(())`. `git diff --stat
crates/kali_cli/src/build/module_link.rs` → empty after revert.

### Post-probe cleanliness

`git status --short` after both probes' reverts: only
`crates/kali_cli/tests/module_namespace_link.rs` shows as modified (the intended Task-8 test
addition) — `crates/kali_cli/src/build/module_link.rs` is byte-identical to pre-sabotage. Both full
verification-contract commands were re-run on the reverted product and confirmed green a second
time (11/0, 45/0, 26/0, fmt clean) before committing.
