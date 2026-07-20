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

> **⚠ Framing correction (final stage review, §13).** The close-out's implicit
> claim that the SYNCHRONOUS capture surface was sound was FALSIFIED: the rows
> above are all OWNER-DIRECT or transparent-intermediate invocation shapes. A
> capturer invoked while a SIBLING env-owner's record is active silently
> addressed the WRONG record (cross-binding memory corruption — spec §3.4's
> dynamic-parent premise vs the lexical capture analysis). What is true NOW:
> the safe subset (owner-direct + transparent-intermediate chains, proven by
> an interprocedural fixpoint) is lowered; every sibling-dynamic /
> unprovable invocation shape fails closed E5506 (`kali_codegen::env_safety`).
> See §13 for the corruption reproducers and the gate.

## 6. Adversarial sweep — node-vs-kali evidence (Task 8 Step 3 + amendments)

Each row is a permanent test in `soundness_closures.rs`. Fixtures that pin a
DIVERGENCE from node are labelled with their class and follow-up.

> **⚠ Sweep-coverage correction (final stage review, §13).** This sweep never
> exercised a capturer invoked from a NON-owner env-owning context, so it
> could not observe the dynamic-env corruption class — its rows are all
> owner-rooted call chains. The class is now closed fail-closed (rows t–w
> below, §13); rows a–l remain sound because their shapes are exactly the
> provable subset the safety fixpoint admits. Rows o/p/q/q2/q3's guard was
> additionally INVERTED from a capturing-denylist to a provably-safe
> allowlist (§13 IMPORTANT-1); their E5506 verdicts are unchanged.

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
| o | `deferred_queue_microtask_capturing_callback_fails_closed` | `queueMicrotask(fn)` capturing | `sync=5`/`mt=6` | E5506 | ✅ fail-closed (Concern-2 re-close of the unmasked gap) |
| p | `deferred_set_timeout_capturing_callback_fails_closed` | `setTimeout(fn,0)` capturing | `sync=5`/`st=6` | E5506 | ✅ fail-closed (Concern-2 re-close) |
| q | `deferred_add_event_listener_capturing_callback_fails_closed` | `addEventListener`+`dispatchEvent` capturing | `ev=6`/`sync=6` | E5506 | ✅ fail-closed (Concern-2 re-close) |
| q2 | `deferred_set_interval_capturing_callback_fails_closed` | `setInterval(fn,0)` capturing | `sync=5`/`iv=6` | E5506 | ✅ fail-closed (Finding B — 4th surface pinned) |
| q3 | `deferred_set_timeout_indirect_capturing_callback_fails_closed` | `let cb=fn; setTimeout(cb,0)` capturing | `1` | E5506 | ✅ fail-closed (Finding C — indirect binding resolved by provenance) |
| bg1 | `deferred_queue_microtask_module_scope_capture_still_runs` | module-global `count+=1` via `queueMicrotask` | `sync=0`/`mt=1` | `sync=0` (runs) | ✅ boundary guard — module global has empty `captured`, guard does NOT fire |
| bg2 | `deferred_queue_microtask_non_capturing_callback_still_runs` | non-capturing inline callback | `sync`/`mt` | `sync` (runs) | ✅ boundary guard — empty `captured`, guard does NOT fire |
| bg3 | `deferred_set_timeout_indirect_non_capturing_callback_still_runs` | `let cb=nonCapturingFn; setTimeout(cb,0)` | `sync`/`cb` | `sync` (runs) | ✅ boundary guard (Finding C) — provenance resolves cb, empty `captured`, runs |
| r | `array_callback_capture_fails_closed` | `[..].map(cb)` capture | `12,13,14`… | E5506 | ✅ fail-closed (array-callback ABI stage) |
| s | `exotic_array_element_indirect_call_is_preexisting_zero_not_garbage` | `arr[0]()` capture-INDEPENDENT | `9` | `0` | ⚠️ PRE-EXISTING indirect-call zero (out of sweep-fix scope, amendment 3c) |
| t | `dynamic_env_sibling_write_capturer_fails_closed` | capture WRITE invoked from sibling env-owner | `101`/`1` | E5506 | ✅ fail-closed (§13 CRITICAL — was silent corruption `102`/`0`) |
| u | `dynamic_env_sibling_read_capturer_fails_closed` | capture READ invoked from sibling env-owner | `7` | E5506 | ✅ fail-closed (§13 CRITICAL — was silent corruption `101`) |
| v | `dynamic_env_owning_capturer_from_sibling_env_owner_fails_closed` | env-OWNING capturer invoked from sibling env-owner | `7` | E5506 | ✅ fail-closed (§13 CRITICAL — was silent corruption `11`) |
| w | `dynamic_env_test_registration_from_sibling_env_owner_fails_closed` | `Kali.test` registration inside sibling env-owner | `c=41` | E5506 | ✅ fail-closed (§13 CRITICAL registration variant — was `c=8`, wrong env_ptr) |
| x | `deferred_set_timeout_aliased_capturing_callback_fails_closed` | `let cb2 = cb; setTimeout(cb2, 0)` capturing | `1` | E5506 | ✅ fail-closed (§13 IMPORTANT-1 — was silent drop, `0`) |
| y | `deferred_set_timeout_call_result_callback_fails_closed` | `setTimeout(makeCb(), 0)` capturing | `1` | E5506 | ✅ fail-closed (§13 IMPORTANT-1 — was silent drop, `0`) |
| z | `deferred_reassigned_callback_provenance_fails_closed` | reassigned `cb` then `setTimeout(cb, 0)` capturing | `1` | E5506 | ✅ fail-closed (§13 IMPORTANT-1 — was the §12-item-9 fail-open tripwire, `0`) |
| z2 | `kali_test_unresolvable_callback_fails_closed` | `Kali.test("a", cb)` with `cb` a parameter | runs the test | E5506 | ✅ fail-closed (§13 IMPORTANT-2 — was warning + `ok 1` with zero tests) |

Rows a–l are sound (node parity or correct fail-closed). Rows o/p/q/q2/q3 are
now CORRECT FAIL-CLOSED (E5506) after the Concern-2 fix + Finding B/C follow-ups
(the guard rejects capturing callbacks to un-emittable scheduling surfaces —
inline AND indirect-via-binding); rows bg1–bg3 are boundary guards proving the
guard is capture-gated (module-global / non-capturing / indirect-non-capturing
callbacks still run). Rows m, n, s remain PRE-EXISTING escaping/indirect-call
zero divergences, each pinned so a future stage trips the tripwire. Their
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
a wholesale no-op, not a capture-lowering miscompile.

**→ CLOSED (Concern-2 fix, commit see below).** The reject-don't-miscompile
violation — base-E5506 shapes now running silently — is fixed. A single guard
at the choke point all four surfaces converge on
(`emit/call.rs::emit_call`, the generic zero-placeholder fallback) rejects
E5506 when `is_undrained_scheduling_surface(callee)` (`queueMicrotask` /
`setTimeout` / `setInterval` / `addEventListener` — the closed scheduling
family) AND `call_has_capturing_closure_arg(node)` (an argument function whose
`derive_env_plans` `captured` set is non-empty). Only the newly-unmasked class
is caught; the safe shapes keep working (see the CORRECTED behavior matrix in
task-8-report.md §Concern-2 fix):

- module-scope callbacks (`count += 1` on a module global → empty `captured`)
  and non-capturing callbacks: NOT caught — still run (silent drop is
  pre-existing, out of scope), pinned by two boundary guards.
- `Kali.test` (the one codegen-emitted deferred surface that threads
  `env_ptr`): handled far above the fallback, never reaches the guard.

FALSIFIED FACT: the Task-0 probe-4 claim that module-scope `queueMicrotask`
"drains" was an artifact of a NO-OUTPUT callback body — codegen emits no call
to ANY of the four surfaces in EITHER the `kali run` OR `kali test` lane, so the
callback is ALWAYS dropped (node always fires it). The distinguishing axis is
capture (fail-closed) vs no-capture (pre-existing silent drop), NOT lane and NOT
module-vs-function scope. Rows o/p/q retargeted to E5506 fail-closed pins
(`deferred_{queue_microtask,set_timeout,add_event_listener}_capturing_callback_fails_closed`).

**Post-review follow-ups (Findings B/C).** Two boundary cases the initial fix
left open, both now closed at the SAME choke point:
- **Finding B — `setInterval` unpinned.** The guard already allowlisted
  `setInterval` (4th scheduling surface) but no fixture exercised it. Pinned by
  `deferred_set_interval_capturing_callback_fails_closed` (E5506).
- **Finding C — indirect capturing callback.** `call_has_capturing_closure_arg`
  originally resolved only the DIRECT inline form (`setTimeout(function(){…})`,
  where the arg node's own text is the `__kali_fn_N` plan key). The INDIRECT
  form `let cb = function(){ base += 1; }; setTimeout(cb, 0)` passed the callback
  by binding name; on the pre-fix binary it COMPILED and silently printed `0`
  (callback + captured `base` dropped) — the SAME reject-don't-miscompile class
  (base a57cd09d5 rejected E5506 on the identical `base += 1` capture-write; the
  binding indirection does not change what is captured). The guard was WIDENED
  to resolve an identifier argument to its closure plan by DECLARATION PROVENANCE
  (`fn_valued_locals`, recorded at declaration-emit time — not name-guessing);
  a capturing indirect callback now fails closed E5506, while a non-capturing
  indirect callback (empty `captured`) still runs unchanged. Pinned by
  `deferred_set_timeout_indirect_{capturing_callback_fails_closed,non_capturing_callback_still_runs}`.

**→ SUPERSEDED (final stage review, §13 IMPORTANT-1).** The Finding-B/C guard
was still a DENYLIST — it default-ALLOWED any argument it could not resolve
(aliases, call results, reassigned bindings — all live fail-opens on HEAD).
`call_has_capturing_closure_arg` no longer exists: it was inverted into
`scheduling_call_args_provably_safe` (an allowlist — E5506 unless every
argument is provably a non-callable literal or a stable-provenance,
non-capturing closure). The rows-o/p/q/q2/q3 verdicts and the bg1–bg3
boundary behaviors above are unchanged; the details and the one deliberate
residual (the flattened-arrow `Value("unknown")` placeholder lane) are §13.

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
   `queueMicrotask` / `setTimeout` / `setInterval` / `addEventListener`+
   `dispatchEvent` and a post-run drain so the callback actually RUNS. Today
   they lower to the zero-placeholder no-op and drop the callback. The §7 UNMASK
   half (capturing callbacks must not run silently) is now CLOSED — they fail
   closed E5506 (Concern-2 fix); this remaining item is the CORRECTNESS half
   (run the dropped callback), which also unblocks the headline flip (§8) and
   the NON-capturing / module-scope drops (still silent, pre-existing, pinned by
   the two boundary guards).
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
9. **`fn_valued_locals` reassignment/shadowing invalidation — RE-SCOPED to the
   default-deny disposition (CLOSED, §13 IMPORTANT-1).** The stale-provenance
   fail-open (a later `cb = function(){ …capture… }` or a shadowing re-`let`
   leaving the declarator-time mapping stale → silent drop) is closed
   structurally, not by mapping invalidation: the scheduling guard now
   REFUSES to resolve any name that is assigned outside its declarator or
   declared more than once (`unstable_provenance_names`, computed up front
   from the body — not emission-order-dependent) and fails closed E5506.
   The old tripwire pin
   `deferred_reassigned_callback_provenance_is_stale_fail_open_tripwire`
   (which pinned the fail-open precisely so this change would trip it) is
   replaced by the E5506 assertion
   `deferred_reassigned_callback_provenance_fails_closed` (§6 row z). The
   precise fix directions previously listed here (invalidate/update the map
   at assignment-emit; call-site binding provenance) remain valid future
   PRECISION work — they would turn some of these E5506s back into running
   programs — but the soundness hole itself no longer exists.
10. **Shadowing scope-granularity (pre-existing).** Capture analysis and name
   resolution are FUNCTION-granular: a block-level `let` shadow of a captured
   name inside the same function is not modeled as a distinct binding, so a
   capture of the outer binding can go undetected (no env cell, baseline
   local resolution). Pre-existing, independent of this wave; the
   `unstable_provenance_names` multi-declarator rule fails the SCHEDULING
   lane closed for such names, but direct-call capture shapes with block
   shadows remain function-granular.
11. **`kali test` prints `ok 1` with ZERO registered tests.** The harness
   summary does not distinguish "all tests passed" from "nothing was ever
   registered", which is exactly what masked the IMPORTANT-2 silent drop
   (and still masks the flattened-arrow warning lane, §13 residual). A
   zero-registered run should be distinguishable (e.g. `ok 0` or a
   diagnostic) — harness follow-up.
12. **Duplicate-label collisions shielded ACCIDENTALLY by E4201.** Everything
   in this stage keys on function-name labels (`env_plans`,
   `capture_owners`, the §13 safety fixpoint's name-keyed edges). Two
   same-named functions would conflate — today unreachable only because the
   export path rejects duplicate names (E4201 export-dedup), an accidental
   shield, not a designed invariant. If exports ever stop deduping (or a
   non-exported duplicate lane appears), the label keying needs its own
   collision poisoning (the `escape_flow.rs::poison_function` precedent).
13. **Twin owner computations.** "Owns a promotable env" is now computed in
   TWO places from the same inputs: the `lower.rs` promotion loop (locals
   mutation + save-local reservation) and `env_safety.rs::promotable_owner`
   (the fixpoint's Record(F) decision), both delegating to the single
   `closure::cell_is_promotable` predicate over the same plan cells. They
   cannot diverge today (same predicate, same data), but a future edit to
   one loop's FILTER (not the predicate) would desync them — fold into one
   shared helper when next touched.

---

# 13. Final stage review — fix wave (2026-07-16)

> Findings from the FINAL whole-stage review of Stage C (HEAD `3a1545b95`),
> fixed in one wave. Every reproducer below was verified on pre-fix HEAD (the
> recorded corrupted/silent output) and is a permanent E5506 fixture in
> `soundness_closures.rs` (§6 rows t–z2). Suite census: 33 → 40 tests.

## 13.1 CRITICAL — dynamic env chain vs lexical capture analysis

**The falsified premise.** `emit_function_env_prologue` links a new env
record's parent to the INCOMING `current_env` — the DYNAMIC caller's env —
and the capture lowering addresses cells against `current_env`; the capture
analysis (`derive_env_plans`, `mir_depth`) is LEXICAL (spec §3.4). The two
agree only for owner-rooted invocation chains. When a capturer runs while a
SIBLING env-owner's record is active, cell addressing resolves against the
wrong record — SILENT CROSS-BINDING MEMORY CORRUPTION. Verified on pre-fix
HEAD (node v26.5.0 disagrees; base `a57cd09d5` rejected all E5506):

| reproducer | pre-fix HEAD | node | class |
|---|---|---|---|
| sibling-invoked capture WRITE (`inc` from `sib`) | `102` / `0` | `101` / `1` | write landed in sibling's cell |
| sibling-invoked capture READ (`rd` from `sib`) | `101` | `7` | read sibling's cell |
| env-OWNING capturer from sibling env-owner | `11` | `7` | parent walk into sibling's record |
| `Kali.test` registration inside sibling env-owner | `c=8` + `ok 1` | `c=41` | wrong `env_ptr` stored at registration |

**The fix — FAIL CLOSED** (controller decision: the lexical-parent-links
rewrite is Stage D-adjacent, out of scope). New module
`crates/kali_codegen/src/env_safety.rs`, wired into `lower_lir_to_wasm`
after env-cell promotion. Interprocedural fixpoint (the `escape_flow.rs`
precedent):

- Abstract `current_env` during each function body: `Record(F)` for a
  promotable env owner (its prologue publishes its own record), else the
  JOIN of its callers' contexts (`_start` = `NoEnv`; conflicts = `Top`).
- Edge graph: direct calls + `Kali.test` registrations (the stored `env_ptr`
  is `current_env` at the registration site, so registration inherits the
  identical requirement), attributed to the lexically-enclosing function
  (nested-fn subtrees opaque). Callee names resolve through a name-keyed,
  whole-program alias closure (declarator initializers, `for-of` bindings)
  — a deliberate over-approximation.
- Verdict: every REACHABLE edge into an ENGAGED capturer (>=1
  `mir_depth == 1` ref whose owner-keyed cell is promotable — exactly the
  `resolve_capture_access` engagement predicate) must carry exactly
  `Record(owner)`; anything else is E5506 at compile time, un-lowering the
  program to the pre-Stage-C reject.
- A capturer with NO reachable invocation edge is vacuously safe: kali has
  no first-class invocation (indirect calls lower to the zero-placeholder
  no-op), so the escaping-closure pins (§6 rows m/n/s) keep their exact
  pinned pre-existing behavior.

All 33 pre-existing pins stay green: rows a–l are owner-direct or
transparent-intermediate chains, which the fixpoint proves `Record(owner)`
(the grandparent/one-hop-walk fixtures e/f are the transparent-intermediate
proof cases).

## 13.2 IMPORTANT-1 — scheduling guard default-ALLOWED unresolvable provenance

`call_has_capturing_closure_arg` returned false (= allow) on anything it
could not resolve. Live fail-opens on pre-fix HEAD, base E5506: alias
(`let cb2 = cb; setTimeout(cb2, 0)`) and call result
(`setTimeout(makeCb(), 0)`) — both compiled and silently dropped the
capturing callback (printed `0`). Fix: INVERTED to
`scheduling_call_args_provably_safe` at the single choke point all four
surfaces (`queueMicrotask`/`setTimeout`/`setInterval`/`addEventListener`)
converge on — E5506 UNLESS every argument is provably safe:

- a literal / numeric constant (not callable), or
- a callback resolving through STABLE provenance to an empty `captured`
  set: `fn_valued_locals` first (a local shadows a same-named module fn),
  then bindings-namespace checks, then a bare unshadowed function name.
  Names assigned outside their declarator or declared twice
  (`unstable_provenance_names`) are NEVER resolved — closing the
  reassignment-stale tripwire (§12 item 9, re-scoped) fail-closed.

bg1–bg3 boundary pins stay green (module-scope / inline / indirect
non-capturing callbacks still run). **Deliberate residual:** an identifier
resolving to NOTHING in any codegen namespace stays on the pre-existing
placeholder lane — this is the flattened block-arrow argument
(`setTimeout(() => {…})` lowers the arrow to `Value("unknown")`; the arrow
body's statements execute EAGERLY INLINE at the call site (verified:
`arrow=6` prints before `sync=6`; node defers), so no closure ever exists
in this lane — the wrong-TIMING eager execution is PRE-EXISTING and shared
with base). Denying it re-reds main-green
web-baseline bundle-build pins whose callback never existed as a function.
Stage D's un-flatten converts that lane into real closure plans, at which
point the allowlist catches them automatically.

## 13.3 IMPORTANT-2 — `Kali.test` fallback was a fifth unguarded surface

An unresolvable callback VALUE (`function suite(cb){ Kali.test("a", cb); }`)
produced a WARNING and registered nothing → `ok 1`, exit 0, the callback
never ran (verified pre-fix HEAD; base E5506 for capturing bodies). Fix:
folded into the default-deny — a bare identifier naming a live binding, or
a call expression, in callback position is now E5506
(`kali_test_unresolvable_callback_fails_closed`). Same narrowing as 13.2:
the flattened-arrow `Value("unknown")` placeholder callback keeps the
pre-existing warning lane (its body statements were never a compiled
function; a blanket deny re-reds hundreds of main-green browser-lane
fixtures). The `ok 1`-with-zero-tests masking is §12 item 11.

## 13.4 Docs corrected in the same wave

- `owns_promotable_env` + env-prologue doc comments claimed "scalar-i64"
  only; the predicate also fires for C2 fixed-shape object cells —
  corrected (`emitter.rs`, `emit/control_flow.rs`).
- §5/§6 framing banners (the falsified sync-surface soundness claim), §6
  rows t–z2, §7 Finding-B/C superseded note, §12 items 9–13.

## 13.5 Gate

`.kali-cache` cleared; ONE detached full-workspace enumeration on the final
binary → `.superpowers/sdd/scratch/stageC-final.txt`; primary gate
`comm -13 stageC-pre.txt stageC-final.txt` (newly-red) and `comm -23`
(drain) recorded in `.superpowers/sdd/stage-review-fix-report.md` alongside
per-finding RED/GREEN evidence.
