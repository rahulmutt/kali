# Stage D design — land the block-arrow un-flatten + deferred-callback lane

**Date:** 2026-07-16
**Branch:** `soundness-batch1-pra` (entry HEAD `c48f1660a`)
**Predecessors:** Stage AB (repr_infer fn-body walk — gaps A/B),
Stage C (env-pointer closures — gap C, `docs/superpowers/specs/2026-07-16-stageC-closures-design.md`).
**Seed:** `docs/superpowers/followups/task5-block-arrows-WIP.patch` (605 lines,
verified to still `git apply --check` clean at entry HEAD).
**Prior measurements:** `docs/superpowers/followups/throw-fallout-stage6-triage.md`
(the parse defect, the pre-A/B/C 63-test blast radius),
`docs/superpowers/followups/stageC-closures-triage.md` (§12–§13: the scheduling
default-deny, the flattened-arrow residual, the dynamic-env fail-closed envelope).

---

## 1. Goal and non-goals

**Goal.** Close the block-arrow flatten — the parser defect that turns
`f(() => { body })` into a bogus `Identifier("unknown")` argument plus the
body hoisted to module scope, executing **eagerly, inline, at the wrong
time** (a silent miscompile class; the reason the A/B/C/D capability project
exists). Simultaneously, make the deferred surfaces those arrows most
commonly feed — `queueMicrotask`, `setTimeout`, `setInterval` (+
`clearTimeout`/`clearInterval`) — actually **run** their callbacks,
capture-aware, instead of silently dropping them or failing closed.

After Stage D, every program in this lane either behaves byte-for-byte like
node, fails closed E5506 at compile time, or (non-quiescent programs only)
traps loudly at runtime. No silent lane survives — including the §13.2/§13.3
flattened-arrow `Value("unknown")` residual, which ceases to exist.

**Non-goals (explicitly out of scope):**

- `addEventListener` / `dispatchEvent` / `EventTarget` object model — stays
  fail-closed E5506 (its own follow-up stage; the affected fixtures need an
  EventTarget instance model with synchronous dispatch, not event-loop work).
- Promise-based timers (`node:timers/promises`, `.then`) — untouched.
- The headline `structured_clone_and_event_primitives` flip — it fails on
  `structuredClone` deep-clone before ever reaching a listener (Stage C
  triage §8); not gated on this stage.
- Lexical parent links for the env chain (the dynamic-env "real fix") — the
  Stage C fail-closed `env_safety` envelope stays; it extends to the new
  registration surfaces.
- `setTimeout` argument forwarding (`setTimeout(fn, 0, x)`) — fail-closed.
- Class-method return lowering, escaping first-class closures, array-callback
  capture ABI, depth≥2 env chains — all carried follow-ups, unchanged.

## 2. Approach (chosen from three)

**Capability-first, parser flip last.** Build and gate the timer/microtask
lowering *before* touching the parser, exercised end-to-end with
`function(){}` callbacks (which parse correctly today). The parser un-flatten
lands last as a small diff onto ready machinery; block arrows simply join the
lane function expressions already exercise. Every intermediate commit gates
green against the frozen baseline.

Rejected alternatives: *un-flatten first, then drain the reds* (same tests
churn twice — re-pin to E5506 then un-re-pin; an interrupted stage leaves the
branch worse than either endpoint); *single combined diff* (parser + lowering
+ guard re-scope superposed — exactly the shape where the last three
whole-stage reviews found their CRITICALs).

**The WIP patch is a reference, not a rebase target, for its wiring half.**
Its parser + `reject_anonymous_function_argument` portions land in D3
(verbatim where still valid — the anonymity fix keys on the `__kali_fn_`
marker, not `id.is_none()`). Its `queueMicrotask` emit/import wiring predates
the C3 `env_ptr` ABI and the Stage C default-deny guard inversion; that half
is **superseded by D2** and does not land.

## 3. Stage structure (five phases)

### D0 — Fresh triage (measurement only, no product code)

1. Fresh entry baseline: two detached full-workspace enumerations
   (`--no-fail-fast`, `sort -u`, `.kali-cache` cleared each), expected 731
   and zero drift. If ≠731: **stop and reconcile** before anything lands.
2. Apply the WIP patch in a scratch tree on entry HEAD, build, run one full
   enumeration, record the **true** newly-red set on the A/B/C foundation
   (the stage-6 "63" was measured pre-A/B/C and is stale), then revert and
   verify the tree clean.
3. Classify the newly-red into: (a) closed by D1+D2 (deferred-surface
   families), (b) re-pin candidates (main-green-via-miscompile families,
   expected the `addEventListener` browser families + the two
   `json_test_supports_object_type_*` diagnostic-shape pins), (c) anything
   else → investigate before proceeding.

### D1 — Runtime deferred lane (host only, behavior-neutral)

Extend the existing drain (`kali_runtime`, C3 `invoke_callback` +
per-callback `current_env` restore; `execute_tests/timers.rs`) into a real
event loop:

- **Virtual clock, no sleeping.** Clock starts at 0; the drain advances it
  directly to the next due timer's expiry. Correct *ordering* is the
  observable, not wall time.
- **Ordering invariants (node parity):** all pending microtasks drain FIFO —
  including microtasks queued by microtasks — before each timer callback;
  timers fire in `(expiry, insertion-seq)` order; `setInterval` re-arms at
  `expiry + delay` with a fresh seq (already-due timers run before the
  re-armed tick). Missing/`0`/negative delays clamp to **1** — node's
  documented minimum — which under the virtual clock also prevents a
  zero-delay interval re-arm from starving every strictly-later timer.
- **Timer IDs:** monotonically increasing i64 handles. `clearTimeout` /
  `clearInterval` on an unknown, fired, or cleared ID is a silent no-op.
- **Bounded drain:** a total-callback-invocation budget (order 100k) per
  drain; exceeding it traps with a **distinct** diagnostic ("event loop did
  not quiesce"; exact code per kali_common conventions, chosen in the plan)
  and nonzero exit. Only programs that would hang node ever see it.
- **Per-callback env:** every invocation sets `current_env` to the
  registered `env_ptr` and restores after (C3 path); traps inside callbacks
  keep `__kali_callback_N` attribution.
- Both `kali run` and `kali test` lanes share the one queue. How the
  per-test drain boundary composes with the existing `execute_tests` drain is
  settled in the plan after reading `execute_tests/timers.rs`, under the
  invariant that today's `Kali.test` ordering pins stay green.

Codegen still emits no calls to these surfaces after D1, so it is
behavior-neutral and gates green in isolation. Rust unit tests pin the queue
semantics directly (§6).

### D2 — Codegen registration lane (fn-expr callbacks)

- **Recognizers** for global `queueMicrotask` / `setTimeout` / `setInterval`
  / `clearTimeout` / `clearInterval` as call targets — bare unshadowed global
  identifier callee only (Stage C provenance rules; a shadowed or aliased
  callee is not recognized and falls to the default-deny).
- **Callback resolution** through the existing stable-provenance machinery
  (`fn_valued_locals`, `unstable_provenance_names`) to `(fn_index, env_ptr)`
  — the `Kali.test`/C3 registration pattern, emitted against the
  already-threaded env_ptr host ABI.
- **`env_safety` registration edges** for the three surfaces, identical
  treatment to `Kali.test`: the stored `env_ptr` is `current_env` at the
  registration site, so a registration inside a sibling-dynamic context of an
  engaged capturer fails closed E5506.
- **Guard re-scope:** `scheduling_call_args_provably_safe` splits emittable
  from denied surfaces. Emittable (`queueMicrotask`/`setTimeout`/
  `setInterval`) admit provably-resolvable callbacks — **capturing
  included** (their soundness is env_safety's job now); denied
  (`addEventListener`) keeps the full default-deny. Unresolvable provenance
  is E5506 everywhere, unchanged.
- **Timer args:** delay must be a provably-numeric scalar or absent; else
  E5506. Extra args → E5506. `setTimeout`/`setInterval` return the i64 timer
  ID; `clearTimeout`/`clearInterval` take a provably-scalar arg or E5506.
- **Deliberate pin flips:** Stage C's branch-local pins asserting E5506 for
  capturing callbacks with STABLE provenance to these surfaces (stageC
  triage §6 rows o/p/q2/q3 — direct inline and stable-indirect forms) and
  silent-drop for non-capturing ones (bg1–bg3) retarget to node-parity
  assertions. These are capability flips of our own pins, enumerated
  one-by-one in the plan — NOT blast-radius re-pins. Rows x/y/z (alias,
  call-result, reassigned callbacks) keep their E5506 pins: their provenance
  is still unprovable under the unchanged allowlist; making aliases provable
  is future precision work, not Stage D.

Everything in D2 is exercised end-to-end with `function(){}` callbacks
before the parser changes.

### D3 — Parser un-flatten + re-pins

- Land the WIP patch's parser change: block-bodied arrows parse as real
  `ArrowFunctionExpression` nodes (no more phantom `Identifier("unknown")` +
  hoisted `BlockStatement`). Arrows then flow: `name_anonymous_functions`
  pre-pass (`__kali_fn_N`) → repr walk (Stage AB) → env plans (Stage C) →
  the D2 lane.
- Land the `reject_anonymous_function_argument` fix (keys on the
  `__kali_fn_` marker).
- **Delete the flattened-arrow residual carve-outs**: the `Value("unknown")`
  placeholder lane no longer exists, so the scheduling guard's and the
  `Kali.test` fallback's deliberate residuals (stageC triage §13.2/§13.3)
  are removed — the default-deny becomes total.
- `Kali.test('x', () => {...})` becomes a real registered test (`total: 1`);
  the vacuous-`ok 1` empty-shell class closes.
- **Re-pins**, batch-approved with node-vs-kali evidence per family against
  the D0 measurement re-verified on the real D3 tree: (a) main-green tests
  that pass today only via eager-inline execution and hit a still-denied
  surface; (b) the two `json_test_supports_object_type_*` pins wired to the
  flattened error-reporting shape (widen — the patched shape is strictly
  better: `total: 1`, callback-attributed trap). Anything newly-red outside
  those classes is **stop-and-investigate, not re-pin**.

### D4 — Whole-stage review + final gate

- Adversarial whole-branch review (the Stage AB/C pattern — most capable
  model, cross-task probes), explicitly covering: the D1–D3 diff, the
  interaction seams (guard × env_safety × drain × parser), **and the Stage-6
  Tasks 1–4 foundation** (naming pre-pass, repr-tracking, class-method pin)
  whose whole-branch review was deferred to this stage because the un-flatten
  is what first exercises it for real.
- Final gate: two detached enumerations, zero drift, `comm -13` empty vs the
  D0 baseline, drain (`comm -23`) measured and explained, main-worktree
  cross-check (fresh main HEAD, expected 0 failing).
- Triage doc records: D0 measurement, all deliberate pin flips, re-pin
  evidence, budget-trap fixture behavior, final gate numbers, follow-up
  inventory hand-off.

## 4. Components touched

| Component | Change |
|---|---|
| `kali_parser/src/expression/primary.rs` | un-flatten (WIP patch, D3) |
| `kali_types/src/resolve/call.rs` | `reject_anonymous_function_argument` marker fix + arrow-argument resolution (WIP patch portions, D3) |
| `kali_runtime` (`execute_tests/timers.rs`, `state.rs`, host imports) | virtual-clock timer queue, microtask checkpoint, IDs, budget trap, clear no-ops (D1) |
| `kali_codegen/src/emit/call.rs` | recognizers, registration emit, guard re-scope (D2); residual carve-out deletion (D3) |
| `kali_codegen/src/env_safety.rs` | registration edges for 3 new surfaces (D2) |
| `kali_codegen/src/intrinsics/host.rs`, `emitter.rs`, `lower.rs` | conditional imports + lowering plumbing (D2; WIP patch as reference only) |
| `kali_cli/tests/soundness_block_arrows.rs` | arrow probes as permanent pins (D3; the WIP patch's 4 tests rewritten fresh) |
| `kali_cli/tests/soundness_closures.rs` | deliberate pin flips (D2) |
| `kali_cli/tests/runtime_smoke.rs` + browser-family suites | approved re-pins only (D3) |
| new `kali_cli/tests/soundness_event_loop.rs` (or folded into closures suite — plan decides) | timer-lane end-to-end matrix (D2/D3) |

Browser-lane note: any new host import must be mirrored in the 4
hand-maintained `kali:rt` JS import lists (harness.rs ×2 + cmd_build.rs ×2)
or browser tests LinkError (standing convention).

## 5. Error handling — the complete envelope

Compile-time E5506 (fail-closed): denied surfaces (`addEventListener`);
unstable/unresolvable callback or callee provenance (total — no placeholder
residual after D3); env_safety verdicts other than `Record(owner)` at
registration sites of engaged capturers; owner-F64 / `mir_depth ≥ 2` /
non-promotable captures; non-scalar delay; extra timer args; non-scalar
clear args.

Runtime trap (loud, nonzero exit): drain budget exceeded (distinct
diagnostic); traps inside callbacks attributed `__kali_callback_N`
(existing).

Silent-correct only: node-parity execution. The eager-inline class and the
silent-drop class are both eliminated for this lane.

## 6. Testing

- **D1 Rust unit tests** (`kali_runtime`): ordering, virtual clock, interval
  re-arm, clear no-op, budget trap, microtask checkpoint.
- **D2 end-to-end fixtures** (node-vs-kali, fn-expr callbacks): capturing +
  non-capturing on all three surfaces; two-timer delay order + insertion
  tiebreak; nested-microtask-before-timer; interval tick count with
  clearInterval; clearTimeout cancellation; deferred capture read/write
  after owner returns (never-reset-region property via timers); full E5506
  envelope; budget trap (uncleared interval).
- **D3 fixtures:** arrow variants of the D2 matrix + stage-6 probes as pins
  (probe 1 prints `0` then the deferred line; probe 3a ordering restored;
  `Kali.test` arrow registers `total: 1`; throwing arrow attributes to
  `__kali_callback_N`).
- **Gate discipline** (every phase): fresh baseline, `comm -13`/`comm -23`,
  `sort -u`, `--no-fail-fast`, `.kali-cache` cleared every run, detached
  runs watched via `.done` marker files (never `pgrep -f`), main cross-check
  at close.

## 7. Decisions log (user-confirmed)

1. Scope: capture-aware `queueMicrotask` **in** (not minimal-land).
2. Blast-radius disposition: **lower the timer surfaces** rather than re-pin
   their main-green tests to E5506 — bounded to the timers deferred lane;
   `addEventListener` stays denied and its families re-pin with approval.
3. Non-quiescence: **bounded drain + trap**, not node-parity hang, not
   fail-closed setInterval.
4. Sequencing: **capability-first, parser flip last** (Approach 1).
5. Branch stays `soundness-batch1-pra`, unmerged by design (carries the 731
   honest-red baseline per the ci-gate-vs-poisoned-baseline rule).
