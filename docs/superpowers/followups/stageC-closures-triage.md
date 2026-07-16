# Stage C triage — entry baseline + four capture-miscompile probes

> Investigation-first stage entry. This doc records the frozen failure
> baseline that gates every Stage C task and the four baseline
> capture-miscompile probes named in the plan.
> Plan: `docs/superpowers/plans/2026-07-16-stageC-closures.md`
> (env-pointer closures C1-C4).
> Design: `docs/superpowers/specs/2026-07-16-stageC-closures-design.md`.

## 1. Branch / baseline commit

- Branch: `soundness-batch1-pra`.
- HEAD at triage time: `cf56ee382` ("docs(soundness): stageC implementation
  plan — env-pointer closures C1-C4 (8 tasks) [stageC]").
- `192984c39` confirmed an ancestor of HEAD (`git merge-base --is-ancestor
  192984c39 HEAD` → `OK`).

## 2. Frozen failure baseline — 731, zero drift

Built `kali_cli` fresh (`cargo build -p kali_cli`, clean success, no product
code changes). Then ran two independent full-workspace enumerations, each
detached and polled to completion:

```
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > stageC-pre-runN.txt
```

- `$SCRATCH/stageC-pre-run1.txt`: **731** failing test names.
- `$SCRATCH/stageC-pre-run2.txt`: **731** failing test names.
- `diff stageC-pre-run1.txt stageC-pre-run2.txt` → **empty** (zero drift).
- `sort -u stageC-pre-run1.txt stageC-pre-run2.txt > $SCRATCH/stageC-pre.txt`
  → **731** lines (same set both runs — union equals either run).

**Canonical entry baseline: `$SCRATCH/stageC-pre.txt`, 731 entries.** This is
the set every later Stage C gate diffs against (`comm -13` for newly-red,
`comm -23` for drain).

## 3. Four baseline capture-miscompile probes

Each probe was written to a standalone file under `$SCRATCH`, executed via
`target/debug/kali run <file>` on the freshly-built binary, and cross-checked
against `node` (v26.5.0) on the same source. Values below are the actual
observed stdout/stderr/exit code — not copied from the spec.

### Probe 1 — synchronous write to enclosing scalar (`c += 1` from nested fn)

```js
function o(){ let c=0; function inc(){ c+=1; } inc(); inc(); console.log(c); } o();
```

- **node**: stdout `2`, exit 0.
- **kali**: fails closed, exit 1:
  ```
  error[E5506]: compound assignment lowering is unavailable for binding 'c' unless it is a mutable local binding; use a mutable variable or the later compatibility path
  ```
- Verdict: **fail-closed E5506** on the write path, as expected — not a silent
  miscompile. This is the write-path fail site C1 must close.

### Probe 2 — synchronous read of enclosing scalar (`return c` from nested fn)

```js
function o(){ let c=7; function rd(){ return c; } console.log(rd()); } o();
```

- **node**: stdout `7`, exit 0.
- **kali**: stdout `0`, exit 0.
- Verdict: **silent miscompile** — kali runs to completion and prints the
  wrong value (`0` instead of `7`) with no diagnostic. This is the read-path
  fail site C1 must close.

### Probe 3 — heap read via nested fn (`obj.n` captured, not the scalar itself)

```js
function o(){ let obj={n:1}; function rd(){ return obj.n; } console.log(rd()); } o();
```

- **node**: stdout `1`, exit 0.
- **kali**: stdout `0`, exit 0.
- Verdict: **silent miscompile**, same shape as probe 2 but through a heap
  object field read instead of a scalar. Confirms the read-path gap covers
  heap-object capture too, not only scalars.

### Probe 4 — module-scope `queueMicrotask` capture (control — must stay working)

```js
let count=0; queueMicrotask(function(){ count+=1; }); console.log("sync="+count);
```

- **node**: stdout `sync=0`, exit 0 (the microtask callback runs later and
  drains after the synchronous log line, so no further stdout is produced by
  this program before/at the point `console.log` runs).
- **kali**: stdout `sync=0`, exit 0.
- Verdict: **matches** — this module-scope path already works (module
  globals, no function-nesting env needed) and must **not** regress as Stage C
  lands nested-function env-pointer closures.

## 4. Summary

| probe | shape | kali | node | class |
|---|---|---|---|---|
| 1 | write to enclosing scalar, sync call | E5506, exit 1 | `2` | fail-closed (write path) |
| 2 | read of enclosing scalar, sync call | `0`, exit 0 | `7` | **silent miscompile** (read path) |
| 3 | read of enclosing heap-object field, sync call | `0`, exit 0 | `1` | **silent miscompile** (heap read path) |
| 4 | module-scope capture via `queueMicrotask` | `sync=0`, exit 0 | `sync=0`, exit 0 | already correct — control, must stay green |

Probes 1-3 are the exact shapes Stage C (env-pointer closures, Phase C1
onward) targets: the write path currently fails closed (safe, but blocks
valid programs); the read paths (scalar and heap-object) currently miscompile
silently and must become either correct or fail-closed as part of the same
fix. Probe 4 isolates the pre-existing module-scope capture path, which must
remain unaffected.

---

# Stage C close-out (Task 8) — sweep, gate, headline outcome

> Added by Task 8 (headline flip decision, stage gate, adversarial sweep).
> Everything below was measured on the FINAL Stage C binary (HEAD
> `27e22bb47`, freshly built, `.kali-cache` cleared) and cross-checked against
> node v26.5.0. Escaping-closure and deferred-surface cases were additionally
> run on the pre-Stage-C base (`a57cd09d5`, `/workspace/.worktrees/kali-base`)
> to classify each divergence as PRE-EXISTING vs Stage-C-introduced.

## 5. Four baseline probes — before → after (measured on final binary)

| probe | shape | base `a57cd09d5` | final `27e22bb47` | node | verdict |
|---|---|---|---|---|---|
| 1 | write to enclosing scalar (`c += 1`), sync | E5506, exit 1 | `2` | `2` | **FIXED** (C1 write) |
| 2 | read of enclosing scalar (`return c`), sync | `0` (silent) | `7` | `7` | **FIXED** (C1 read) |
| 3 | read of enclosing heap field (`obj.n`), sync | `0` (silent) | `1` | `1` | **FIXED** (C2 heap read) |
| 4 | module-scope `queueMicrotask` capture (control) | `sync=0` | `sync=0` | `sync=0` | **unchanged** (correct) |

All three targeted miscompiles are closed; the control path is unaffected.
These are pinned as permanent tests in `crates/kali_cli/tests/soundness_closures.rs`
(`sync_scalar_capture_write_is_visible_to_owner`,
`sync_scalar_capture_read_returns_value_not_zero`,
`sync_heap_capture_reads_field`, plus the module-global companions).

## 6. Adversarial sweep — node-vs-kali evidence (Task 8 Step 3 + amendments)

Each row is a permanent test in `soundness_closures.rs`. Fixtures that pin a
DIVERGENCE from node are labelled with their class and follow-up.

| # | fixture (test name) | source shape | node | kali | class |
|---|---|---|---|---|---|
| a | `sync_scalar_capture_read_returns_value_not_zero` | read scalar, sync | `7` | `7` | ✅ sound |
| b | `sync_scalar_capture_write_is_visible_to_owner` | write scalar (`+=`), sync | `2` | `2` | ✅ sound |
| c | `sync_heap_capture_reads_field` | read heap field, sync | `1` | `1` | ✅ sound |
| d | `sync_heap_capture_field_write_visible_to_owner` | write heap field, sync | `2` | `2` | ✅ sound |
| e | `env_chain_grandparent_read` | capture skips a no-cell intermediate (depth-1) | `5` | `5` | ✅ sound |
| f | `env_chain_owning_capturer_parent_walk` | genuine one-hop parent walk (env-walk depth 1) | `15` | `15` | ✅ sound |
| g | `sync_scalar_capture_env_does_not_leak_across_activations` | `outer()` twice → distinct envs | `2\n2` | `2\n2` | ✅ sound |
| h | `sync_scalar_capture_restore_survives_sibling_env_owner` | sibling env-owner between two calls | `2` | `2` | ✅ sound |
| i | `capture_free_nested_fn_allocates_no_env` | nested fn captures NOTHING | `42` | `42` | ✅ sound (no env churn) |
| j | `deferred_test_callback_runs_with_its_env` | `Kali.test` deferred capture (the ONE reachable deferred surface) | `a=41`/`b=7` | `a=41`/`b=7` | ✅ sound (C3) |
| k | `deferred_drain_cleanliness_post_drain_capture_resolves` | post-drain capture-owning call inside a 3rd callback | `a=41`/`b=7`/`c=2` | `a=41`/`b=7`/`c=2` | ✅ sound (env restore across drain) |
| l | `module_global_compound_assign_still_routes_to_global` | module-scope capture (spec §3.3 case 4) | `1` | `1` | ✅ sound (unchanged) |
| m | `recursion_distinct_envs_is_preexisting_escaping_zero` | returned closures `make(10)`/`make(20)` | `10 20` | `0 0` | ⚠️ PRE-EXISTING escaping-closure miscompile |
| n | `returned_closure_late_read_is_preexisting_escaping_zero` | closure returned + late-read after loop scratch | `7` | `0` | ⚠️ PRE-EXISTING escaping-closure miscompile |
| o | `deferred_queue_microtask_capture_callback_dropped_preexisting_gap` | `queueMicrotask(cb)` | `sync=5`/`mt=6` | `sync=5` | ⚠️ event/timer-lowering gap (callback dropped) |
| p | `deferred_set_timeout_capture_callback_dropped_preexisting_gap` | `setTimeout(cb,0)` | `sync=5`/`st=6` | `sync=5` | ⚠️ event/timer-lowering gap (callback dropped) |
| q | `deferred_add_event_listener_capture_callback_dropped_preexisting_gap` | `addEventListener`+`dispatchEvent` | `ev=6`/`sync=6` | `sync=5` | ⚠️ event/timer-lowering gap (listener dropped) |
| r | `array_callback_capture_fails_closed` | `[..].map(cb)` capture | `12,13,14`… | E5506 | ✅ fail-closed (array-callback ABI stage) |
| s | `exotic_array_element_indirect_call_is_preexisting_zero_not_garbage` | `arr[0]()` capture-INDEPENDENT | `9` | `0` | ⚠️ PRE-EXISTING indirect-call zero (out of sweep-fix scope, amendment 3c) |

Rows a–l are sound (node parity or correct fail-closed). Rows m–s are
divergences, each pinned so a future stage trips the tripwire. Their
classification (pre-existing vs introduced) is §7.

## 7. Divergence classification — base cross-check (a57cd09d5)

**Escaping / returned-closure miscompile (rows m, n).** kali prints clean `0`
where node returns the captured value. Cause: the env-pointer design resolves
captures through the `current_env` global, which is only correct while the
owning activation is live (synchronous same-activation calls, or the
`invoke_callback` deferred path that restores `env_ptr`). A closure RETURNED
out of its owner and invoked later via a plain call (`a()`, `g()`) sees
`current_env` = the module env, so the promoted cell reads `0`. Run on base
`a57cd09d5`: **byte-identical** (`0 0` and `0`). Therefore **PRE-EXISTING, not
Stage-C-introduced** — the same first-class-function-value / escaping-capture
class as the already-pinned `arr[0]()` and `o?.f()` tripwires (Phase C4). The
load-bearing soundness property holds: clean `0`, NOT a garbage/stale-heap
leak. Amendment-2 note: the observed behavior is silent-wrong (not
fail-closed); failing escaping closures closed requires closure-value escape
analysis at codegen (the escaping-capture-region follow-up), which is beyond
this close-out task's scope and carries gate risk — so it is pinned + surfaced
rather than force-failed, consistent with the C4 precedent.

**Note on the never-reset region (Task 4 reviewer's unproven risk).** Rows m/n
CANNOT exercise the "captured cells survive after the owner returns" property,
because the escaping closure never reaches its env at all. That survival
property IS proven by the DEFERRED path instead: in
`deferred_test_callback_runs_with_its_env`, `suiteA` writes `base=41` into its
env cell and RETURNS; the callback is invoked later during the drain and reads
`41` correctly — i.e. the cell lived in the never-reset `__alloc_global`
region past its owner's return. Row k additionally proves a fresh
capture-owning call resolves correctly AFTER a drain.

**Event/timer-lowering gap + UNMASK (rows o, p, q).** Codegen emits NO call to
`queueMicrotask` / `setTimeout` / `addEventListener` (Task 6 finding: the host
imports exist but no generated module imports them; unknown call targets lower
through the pre-existing E3100 "zero-placeholder compatibility fallback"
no-op). So the scheduled callback never fires — kali prints only the
synchronous line. On base `a57cd09d5` these SAME programs FAILED CLOSED (E5506
on the callback's `base += 1` capture-write). Stage C lowered that
capture-write, removing the E5506 and **unmasking** the pre-existing scheduler
no-op: the program now RUNS and silently drops the callback instead of
rejecting. The drop itself is pre-existing (module-scope probe 4 already
dropped its microtask on base); Stage C only changed the FIRST guard these
programs hit. This is the event/timer-lowering follow-up; the callback-drop is
a wholesale no-op, not a capture-lowering miscompile. **Surfaced as a concern.**

## 8. Headline test — NON-FLIP (deferred, with evidence)

`structured_clone_and_event_primitives_source` (`runtime_smoke.rs:424`) was to
be flipped to node-parity success. It is NOT flippable and the fail-closed
pins are KEPT. Evidence (final binary, `kali run` on the non-test variant):

```
$ kali run hl-nontest.js
Uncaught Error: structuredClone should deep-clone object graphs
error[E4000]: runtime trap (unreachable — allocation failure or an
  unsupported-path guard) ...
exit=1
$ node hl-nontest.js
web baseline ok
exit=0
```

The program fails closed at the FIRST check — `structuredClone` does not
deep-clone (kali's `cloned.values === original.values`), throwing before the
`addEventListener` closure is ever reached. Even past that it depends on
`instanceof AbortSignal` (no prototype chain → E4000), synchronous
`dispatchEvent` of a listener (the row-q event-lowering gap: listener dropped,
so `count` stays 0 and `count !== 1` throws), `URL`/`URLSearchParams`/
`TextEncoder`. The closure lowering (Stage C) is IRRELEVANT to whether this
test passes. All 6 `web_baseline` run/test pins (`run.rs:379/420/465`,
`test.rs:628/669/716`) already assert `!success` with `E4000 || E5506 ||
"RuntimeError: unreachable"`, which matches today's exact behavior (E4000,
exit 1) — they were NOT modified. **The flip is deferred to the
event/structuredClone-lowering follow-up stage.**

## 9. depth-counts-env-owning reconciliation (Task 5)

MIR `depth` (`kali_mir/src/env_plan.rs`) counts STRUCTURAL env-owning
ancestors — repr-independent, because kali_mir cannot see repr. The RUNTIME
env chain links only records that were actually allocated, i.e. functions with
a PROMOTABLE cell (`cell_is_promotable`, repr-dependent). The two agree exactly
when no intermediate ancestor owns a cell that is a structural env-owner but
NOT promotable (a captured `F64` scalar, or a `Closure`/`Array` heap cell) —
such a frame allocates no record, so a MIR depth that counted it would
over-walk the chain. Codegen (`emit/closure_access.rs::env_walk_depth_for`)
therefore lowers ONLY `mir_depth == 1` (owner is the single env-owning
ancestor; every strictly-intermediate frame owns no cell and is transparent) →
env-walk depth 0 if the capturer owns no record, or 1 if it owns its own
record (a genuine one-hop `parent_env` walk). `mir_depth >= 2` is NOT proven
sound and falls through to baseline unchanged. Pinned end-to-end by
`env_chain_grandparent_read` (depth-1, transparent intermediate, env-walk 0)
and `env_chain_owning_capturer_parent_walk` (depth-1, owning capturer, env-walk
1).

## 10. Re-mask probes across the stage (permanent guards)

- **Per-activation alloc**: `sync_scalar_capture_env_does_not_leak_across_activations`
  — `outer()` twice prints `2\n2`, not `2\n4` (fresh env each activation).
- **Epilogue restore vs sibling env-owner**:
  `sync_scalar_capture_restore_survives_sibling_env_owner` — a sibling function
  that clobbers `current_env` in its prologue must restore it on exit, else the
  final read addresses the wrong record. Prints `2`.
- **Deferred env restore across a drain**:
  `deferred_drain_cleanliness_post_drain_capture_resolves` — `c=2` after two
  prior callbacks drained proves `invoke_callback` restores `current_env`.
- **Owner-repr capture gate (F64 phantom-cell)**:
  `capture_gate_owner_f64_compound_assign_rejects_not_miscompiles` /
  `..._update_expr..` — an owner-`F64` capture must reject (E5506), not write a
  phantom i64 cell.
- **F-AB-2 string/growable in exotic body**:
  `exotic_string_capture_in_array_element_fails_closed` (E3200) /
  `exotic_growable_capture_in_array_element_fails_closed` (E5506) — the
  string-handle-lowers-to-i64 danger is NOT reachable.

## 11. Stage gate — two enumerations + primary gate + main cross-check

Method: two detached full-workspace enumerations on the FINAL binary,
`.kali-cache` cleared before EACH (amendment 4 — a stale ABI-mismatched cache
produced 34 false newly-red last time):

```
rm -rf .kali-cache
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > stageC-post-runN.txt
```

- `stageC-post-run1.txt`: **731** failing.
- `stageC-post-run2.txt`: **731** failing.
- `diff run1 run2` → **empty (ZERO DRIFT)**.
- union `sort -u` → `stageC-post.txt`: **731** failing.

**PRIMARY GATE** — `comm -13 stageC-pre.txt stageC-post.txt` (newly-red vs the
731 entry baseline): **EMPTY — zero newly-red. GATE PASSES.**

**DRAIN** (measured, not forecast) — `comm -23 stageC-pre.txt stageC-post.txt`:
**EMPTY — 0 drained.** The pre and post sets are IDENTICAL (Task 8 added only
passing tests + docs; the 731 pre-existing failures — the branch's throw-unmask
poisoned baseline — are untouched by Stage C's closure work).

**Main-worktree cross-check** — `/workspace/.worktrees/kali-main` @ `b48a067d3`,
`cargo test --workspace --no-fail-fast` (`.kali-cache` cleared): **0 failing**,
unchanged from b48a067d3's known-0. Cross-check filter empty. ✅

Entry **731** → exit **731** (zero newly-red, zero drift, zero drain; main 0).

## 12. Follow-up inventory

1. **Array per-element callback ABI stage** — `[..].map/forEach/filter(cb)` with
   a capture currently fails closed E5506 (`array_callback_capture_fails_closed`).
   The follow-up wires the per-element callback ABI so captures lower.
2. **Event/timer lowering stage** — emit scheduler recognizers for
   `queueMicrotask` / `setTimeout` / `addEventListener`+`dispatchEvent` and a
   post-run drain; today they lower to the E3100 zero-placeholder no-op and drop
   the callback (rows o/p/q). This is also the headline-flip unblocker (§8) and
   must resolve the §7 UNMASK (either run the callback or fail closed).
3. **Reclaimable escaping-capture region / first-class function values** — a
   closure returned out of its owner (or stored in an array/object) and invoked
   later via a plain call reads `current_env` = module env → clean `0` (rows
   m/n; `arr[0]()`, `o?.f()` in C4). The follow-up gives escaping closures a
   captured env pointer (or fails them closed via escape analysis). Amendment-2
   flagged this class as needing fail-closed-or-correct; it is currently
   pre-existing silent-`0`.
4. **F-AB-1** — pre-existing expr-arrow-return miscompile (from Stage AB
   close-out), independent of captures, carried forward.
5. **Capture-independent indirect invocation → `0`** — `arr[0]()` / `o?.f()`
   return `0` even WITHOUT a capture (amendment 3c: explicitly OUT of the
   sweep-fix scope; pinned as pre-existing tripwires only).
6. **depth ≥ 2 general env chain** — `mir_depth >= 2` falls through to baseline
   (fails closed / unchanged); the general multi-hop parent walk with
   non-promotable intermediates is future work (§9, Task 5 boundary).
7. **`.kali-cache` ABI keying** — the Task 6 ABI change means stale cached wasm
   must be invalidated; until the cache keys on the ABI version, enumerations
   MUST `rm -rf .kali-cache` first (amendment 4).
8. **Duplicate-label scope IDs / twin owner computations** — carried from prior
   reviews; EnvPlan nesting keys on analysis labels (`8171e2081`); revisit if
   duplicate labels or twin owner derivations surface.
