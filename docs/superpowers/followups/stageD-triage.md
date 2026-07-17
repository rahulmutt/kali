# Stage D triage — entry baseline + parser-only blast-radius measurement

> Task 1 (D0) of the Stage D plan: no product code lands in this task. This
> doc records the frozen failure baseline every later Stage D gate diffs
> against, and the blast radius of applying ONLY the parser + kali_types
> hunks of the un-flatten WIP patch (the codegen half is stale — pre
> env_ptr-ABI — and is excluded).
> Plan: `docs/superpowers/plans/2026-07-16-stageD-unflatten.md`.
> Design: `docs/superpowers/specs/2026-07-16-stageD-unflatten-design.md`.

## 1. Branch / baseline commit

- Branch: `soundness-batch1-pra`.
- HEAD at triage time: `1a3a1ae80` ("docs(soundness): stageD implementation
  plan — 9 tasks, capability-first, parser flip last [stageD]").
- Working tree clean before and after this task; no product code changed.

## 2. Frozen entry baseline — 731, zero drift

Built `kali_cli` fresh (`cargo build -p kali_cli`), then ran two independent
full-workspace enumerations, each detached and polled to completion (cache
cleared before each):

```
rm -rf .kali-cache
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > stageD-pre-runN.txt
```

- `$SCRATCH/stageD-pre-run1.txt`: **731** failing test names.
- `$SCRATCH/stageD-pre-run2.txt`: **731** failing test names.
- `diff stageD-pre-run1.txt stageD-pre-run2.txt` → **empty** (zero drift).
- `sort -u stageD-pre-run1.txt stageD-pre-run2.txt > $SCRATCH/stageD-pre.txt`
  → **731** lines (union equals either run — confirms identical sets).

**Canonical entry baseline: `$SCRATCH/stageD-pre.txt`, 731 entries.** This is
the honest-red baseline this branch deliberately carries (never merge to
main); every later Stage D gate diffs against it via `comm -13` (newly-red)
and `comm -23` (drain).

## 3. Parser-only scratch measurement

Applied ONLY the parser + kali_types portions of
`docs/superpowers/followups/task5-block-arrows-WIP.patch`:

```
git apply --include='crates/kali_parser/*' --include='crates/kali_types/*' \
  docs/superpowers/followups/task5-block-arrows-WIP.patch
```

`git status` confirmed exactly the 2 expected modified files:
`crates/kali_parser/src/expression/primary.rs`,
`crates/kali_types/src/resolve/call.rs`. `cargo build -p kali_cli` succeeded
clean. Ran the same two-step enumeration (cache cleared) on the patched
binary:

- `$SCRATCH/stageD-parser-post.txt`: **700** failing test names (31 fewer
  than the 731 baseline gross count — the un-flatten parser/types change
  fixes more than it breaks, net; see newly-green note below).
- `comm -13 stageD-pre.txt stageD-parser-post.txt` →
  `$SCRATCH/stageD-parser-newly-red.txt`: **6** entries (candidate re-pin
  list for Tasks 7-8).
- For context (not part of the required interface, informational only):
  `comm -23 stageD-pre.txt stageD-parser-post.txt` → **37** entries newly
  GREEN under the patch (tests that fail on baseline and pass once the
  parser stops flattening block-arrows). Net: 731 − 37 + 6 = 700, consistent
  with the raw counts above.

Per the brief's note: the applied `kali_types` gate still treats
`setTimeout`/`setInterval` as `deferred_but_unwired` (rejects anonymous
callbacks to them) — Tasks 4-5 wire them, and Task 7 lands the gate WITHOUT
that carve-out, so this measurement slightly OVER-counts the final red set.
That direction is safe for planning purposes.

### Revert and clean-tree verification (Step 3)

```
git checkout -- crates/kali_parser/src/expression/primary.rs crates/kali_types/src/resolve/call.rs
git status                                                            # clean
git apply --check docs/superpowers/followups/task5-block-arrows-WIP.patch && echo STILL-APPLIES
```

- `git status` after revert: **clean** (nothing to commit).
- `git apply --check` on the full WIP patch: **STILL-APPLIES** (revert did
  not leave the tree in a state that desyncs the patch's other hunks).
- `kali_cli` rebuilt clean on the reverted (baseline) tree, confirming no
  residual state.

## 4. Classification of the newly-red set (Step 4)

All 6 entries were investigated by rebuilding the patched binary, re-running
each failing test with `--nocapture`, and reading the actual JSON
error/stderr payload — not inferred from the name alone.

| # | test name | family | bucket | evidence |
|---|---|---|---|---|
| 1 | `build::build_emits_browser_bundle_web_baseline_primitives` | `addEventListener` capturing callback in the browser web-baseline fixture | (b) re-pin candidate | `E5506`: *"a callback passed to 'addEventListener' is unavailable unless it is provably non-capturing: codegen emits no call to this scheduling surface... an argument with unresolvable provenance fails closed"* |
| 2 | `build::build_emits_browser_bundle_web_baseline_primitives_in_js_input` | same fixture, `.js` input variant | (b) | same `E5506` addEventListener rejection (JS-input variant of #1) |
| 3 | `build::json_build_emits_browser_bundle_web_baseline_primitives` | same fixture, `--output json` variant | (b) | same `E5506` addEventListener rejection, JSON envelope: `errors[0].code == "E5506"`, message identical to #1 |
| 4 | `build::json_build_emits_browser_bundle_web_baseline_primitives_in_js_input` | same fixture, JSON + JS-input variant | (b) | same `E5506` addEventListener rejection |
| 5 | `test::json_test_supports_object_type_and_constructor_semantics` | `instanceof`/constructor-semantics diagnostic-shape pin | (b) re-pin candidate | Before patch: top-level `errors[0].code` is `E4000`/`E5506`. After patch (un-flattened `Kali.test` callback body): `errors` is `[]`, `payload.failed == 1`, `payload.passed == 0`, trap attributes to `stderr` as `"runtime trap in callback '__kali_callback_34'"` — a strictly better, more specific shape (per-callback attribution vs bare top-level reject), still fails closed (exit != 0, `success: false`) |
| 6 | `test::json_test_supports_object_type_and_constructor_semantics_in_js_input` | same pin, `.js` input variant | (b) | identical shape change to #5 |

**Bucket sizes: (a) = 0, (b) = 6, (c) = 0.**

- **Bucket (a) — deferred-surface families (queueMicrotask/setTimeout/
  setInterval capturing callbacks) closed by Tasks 4-6: 0 entries.** No
  currently-green (baseline-passing) test exercises an anonymous/capturing
  callback to those three surfaces, so applying the parser+types-only patch
  does not newly-red anything in this family. (The `deferred_but_unwired`
  carve-out noted in §3 means any such fixtures that DO exist are either
  already red on baseline or pinned as fail-closed `E5506` expectations that
  hold under both trees — not new fallout from this measurement.)
- **Bucket (b) — re-pin candidates: 6 of 6 entries**, exactly the two
  families the brief predicted: 4 `addEventListener` browser-bundle
  variants (`build`/`json_build` × `.ts`/`.js` input) and the 2
  `json_test_supports_object_type_and_constructor_semantics` diagnostic-shape
  pins. Both families are documented in the full WIP patch itself: the
  `addEventListener` fixture's capturing callback is exactly the shape the
  un-flatten patch's `kali_types` gate is designed to reject fail-closed
  (main is currently only "green" because the flattened-arrow lane silently
  drops the callback — a miscompile, not a real pass); the
  `object_type_and_constructor_semantics` pins assert the OLD top-level
  error-envelope shape, and the WIP patch's own `runtime_smoke.rs` hunk
  (not applied in this measurement — it's outside `kali_parser`/
  `kali_types`) already contains a `test_mode` branch documenting and
  re-pinning the NEW per-callback-attributed shape verbatim. No new
  investigation was required to explain either family; both are expected,
  named re-pins for whichever later task lands the parser flip.
- **Bucket (c) — anything else: 0 entries.** Nothing unexplained; not
  BLOCKED.

## 5. Interfaces produced

- `$SCRATCH/stageD-pre.txt` — canonical 731-entry baseline (consumed by
  every later Stage D gate).
- `$SCRATCH/stageD-parser-newly-red.txt` — 6-entry candidate re-pin list
  (consumed by Tasks 7-8).
- This doc.

## 6. Concerns / follow-ups for later tasks

- Bucket (a) being empty means Tasks 4-6's "closes deferred-surface
  families" framing has no positive baseline evidence from THIS
  measurement — it is a forward-looking claim about capability work not yet
  landed, not something this parser-only slice could exercise (no
  currently-green fixture uses a capturing callback to
  `queueMicrotask`/`setTimeout`/`setInterval`, so there was nothing for the
  parser-only patch to newly-red in that family). Later tasks should
  re-measure once those surfaces are wired.
- The 37 newly-green tests (informational, §3) are not required by the
  brief's interface but are worth Task 7-8 awareness: the parser+types-only
  patch fixes real baseline failures as a side effect, which is expected
  (un-flattening block-arrows closes silent-miscompile pins that were
  previously red for the RIGHT reason).

---

# Task 7 (D3) — measured full-foundation blast radius (parser flip LANDED)

> Task 7 lands the block-arrow un-flatten on the FULL Stage D foundation
> (Tasks 2–6 live). Unlike the Task 1 measurement (parser+types hunks over a
> bare branch), this is the real gate: all codegen deferred-callback lanes,
> virtual-clock drain, env_safety registration edges, and browser glue are
> present. Gate command per the Task 7 brief (fresh `.kali-cache` +
> `cargo build -p kali_cli`, then `cargo test --workspace --no-fail-fast`).

## 7.1 Totals

- Baseline (`stageD-pre.txt`): **731** honest-red.
- Post-Task-7 (`stageD-post-task7.txt`): **698** red.
- **Newly-red: 4** (`stageD-task7-newly-red.txt`).
- **Drain (newly-green): 37.** Net −33.

Every one of the 4 newly-red was already predicted by Task 1's parser-only
measurement (`stageD-parser-newly-red.txt`) — `comm -23` of the Task 7
newly-red against the Task 1 list is EMPTY. The 2
`json_test_supports_object_type_and_constructor_semantics{,_in_js_input}`
entries Task 1 also predicted are now GREEN, pre-handled by the WIP patch's
`runtime_smoke.rs` `test_mode` re-pin hunk applied in Step 4 (verified: the
`json_test_supports_object_type_*` pair passes post-patch).

## 7.2 Classified newly-red (all bucket-b — Task 8 re-pin batch)

| Test (all in `runtime_smoke/build.rs`) | Bucket | Evidence |
|---|---|---|
| `build::build_emits_browser_bundle_web_baseline_primitives` | b | see below |
| `build::build_emits_browser_bundle_web_baseline_primitives_in_js_input` | b | see below |
| `build::json_build_emits_browser_bundle_web_baseline_primitives` | b | see below |
| `build::json_build_emits_browser_bundle_web_baseline_primitives_in_js_input` | b | see below |

**Root cause (shared by all 4):** `browser_bundle_web_baseline_source()`
(`runtime_smoke.rs:4462`) contains
`target.addEventListener('tick', () => { count += 1; controller.abort(); })`
— an anonymous BLOCK-ARROW callback on the `addEventListener` surface, the
ONE remaining undrained scheduling surface (`is_undrained_scheduling_surface`,
Task 5). Pre-D3 the arrow flattened to a `Value("unknown")` placeholder and
`scheduling_call_args_provably_safe` returned `true` for it (the resolve-to-
nothing tail this task flips to `false`), so the bundle BUILD succeeded by
SILENTLY DROPPING the callback (+ its captured `count`/`controller`). Post-D3
the un-flatten compiles the arrow as a real CAPTURING function, and the
flipped guard fails closed:

```
error[E5506]: a callback passed to 'addEventListener' is unavailable unless
it is provably non-capturing: codegen emits no call to this scheduling
surface, so the callback — and any captured environment — would be silently
dropped; an argument with unresolvable provenance fails closed
```

These 4 pins assert `output.status.success()` (the OLD masking shape: build
succeeds via dropped callback). The new behavior — a fail-closed build reject
of a callback that would otherwise vanish — is STRICTLY BETTER (a silent
miscompile becomes a clean diagnostic). They are legitimate re-pins, NOT
defects, and are deferred to **Task 8's user-approval re-pin batch** (this
task does NOT re-pin them beyond the one pre-approved `runtime_smoke.rs`
`assert_json_object_type_and_constructor_semantics` hunk landed in Step 4).

## 7.3 Bucket (a) — deferred-surface families that should now be GREEN: 0 still red

The 37 drained tests ARE the bucket-a family, now green as intended — all
`test_supports_*object_keys/values*` / `*for_of_break_continue*` /
`*set_constructor*` / `*frozen_object*` iteration fixtures (browser-harness
and direct variants). None remained red, so there is NO Task 4–6 defect.
(These went red pre-D3 for the RIGHT reason — the flattened-arrow lane — and
the un-flatten is their real fix.) Full list: `comm -23 stageD-pre.txt
stageD-post-task7.txt`.

## 7.4 Bucket (c) — unexplained: 0

Nothing unexplained; not BLOCKED.

## 7.5 Probe adaptation note (feature-rich deferred-ordering probe)

The Step 5 probe `a_feature_rich_block_arrow_callback_defers_with_correct_
ordering` was adapted from the brief's verbatim snippet: the brief wrote the
callback's module write as `acc = value + b.n` (a module-scope `let` `=` an
expression CONTAINING a `.field` member read). That specific shape
pre-existingly mis-parses (`E8001 unsupported unary operator 'n'` + `E8001
binary operator '='`, then misclassifies the WRITE as a module-binding READ →
`E5506`) from inside ANY function body — CONFIRMED to reproduce identically
with a plain SYNCHRONOUS NAMED function, i.e. wholly orthogonal to arrows /
the un-flatten. The member access was moved into an unobserved local
(`let probe = b.n + value;`) and the module write became `acc = value` (no
member in RHS → the working lane); the body stays feature-rich (`+=`, `*=`,
`new`, `.field` read, module write) and the load-bearing DEFERRED-ORDERING
property is unchanged (node + kali agree byte-for-byte:
`MODULE-END-acc\n0\nINSIDE-CALLBACK\n15\n`). New pre-existing follow-up
inventoried: `<module-let> = <expr with member access>` mis-parse, and a
sibling `.field` read inside a function lowering to `0`.

---

# Task 8 resolution — event-surface lane

> Closes out the 4 pins Task 7 (§7.2 above) deferred to "Task 8's
> user-approval re-pin batch". This section is the close-out record Task 9
> (whole-stage adversarial review) consumes; it also records the EV lane
> (Tasks 1-5 of `docs/superpowers/plans/2026-07-17-stageD-event-surface.md`)
> that superseded the originally-planned simple re-pin.

## 8.1 User decision trail

1. **Re-pin batch presented, REJECTED.** The straightforward Task 8 move —
   flip the 4 `browser_bundle_web_baseline_primitives{,_in_js_input}` /
   `json_build_…` pins from "build succeeds via silently-dropped callback" to
   "build succeeds, callback genuinely compiled" (or accept the new E5506 as
   correct) — was presented as the default close-out. The user rejected a
   bare re-pin: a re-pin alone leaves the underlying capability gap (no
   `EventTarget`/`addEventListener`/`dispatchEvent` runtime lane) unclosed,
   and the corpus fixture (`browser_bundle_web_baseline_source`,
   `runtime_smoke.rs:4462`) exercises `structuredClone`, `AbortController`,
   `EventTarget`, `URLSearchParams`, `URL`, `TextEncoder`/`TextDecoder` — a
   web-baseline API surface kali's browser target claims to support but
   mostly does not actually execute.
2. **Full `webBaselineSmoke` parity chosen as the destination**, decomposed
   into API-family stages (events, `structuredClone`, `AbortController`,
   `URL`/`URLSearchParams`, `TextEncoder`) rather than attempted in one
   sweep — each family has an independent capability build (registry model,
   provenance rules, codegen emit arms) and its own soundness envelope.
3. **Events chosen as the first stage** ("events-first"): `EventTarget` /
   `addEventListener` / `dispatchEvent` is the one family that is a genuine
   NEW async-shaped surface for kali (registration + later invocation, same
   shape as the timer/microtask lanes already landed in Stage D Tasks 1-4)
   and it is the surface actually triggering the Task 7 fail-closed reject
   (the `count`/`controller` capturing callback), so closing it first both
   unblocks the 4 pins AND proves out the pattern the later stages
   (`structuredClone`, `AbortController`, `URL`/`USP`, `TextEncoder`) will
   reuse.
4. **Design → plan → execution.**
   - Design: `docs/superpowers/specs/2026-07-17-stageD-event-surface-design.md`
     (commit `91dbeaec1`) — host-registry + synchronous dispatch, fail-closed
     envelope, Task 8 fallback framing.
   - Plan: `docs/superpowers/plans/2026-07-17-stageD-event-surface.md`
     (commit `34dbf638d`) — 5 tasks, glue-before-emit sequencing, explicit
     gate-restoration criteria (drain must return to 37, newly-red must
     return to empty).
   - EV Task 1 (`092d7d3fe`): runtime event registry + synchronous re-entrant
     dispatch (handles, `(handle, type)` listener keys, snapshot semantics,
     env restore).
   - EV Task 2 (`53b573cfa` + `771cfec8e`): mirrored the lane into all 4
     browser JS import lists (registry, sync dispatch, snapshot + dedup),
     then closed a guest-string-reader fail-open (align all 4 mirrors to
     fail closed on missing memory).
   - EV Task 3 (`8795ced80`): `EventTarget` construction lane — types 15-17,
     conditional imports, declarator provenance, handle-escape choke point.
   - EV Task 4 (`8da4d734b`): `addEventListener`/`dispatchEvent` emit arms —
     receiver provenance, literal-type gates, zero-param gate, `env_safety`
     member edge; full e2e/envelope/preservation pin suite; full-gate
     restoration first achieved here (newly-red EMPTY, drain 37).
5. **Dispatch-arg reconciliation, USER-RATIFIED.** EV Task 4 found the
   plan's two `addEventListener`-out-of-lane-argument-shape E5506 pins
   irreconcilable with the browser corpus (which dispatches out-of-lane
   argument shapes — e.g. `new CustomEvent('tick', {detail:1})`, a bound
   captured `Event` — on an in-lane, registered-listener `EventTarget`, and
   the corpus's "70+ packages stay deployable" contract cannot regress).
   Reconciliation: an out-of-lane dispatch *argument* on an in-lane receiver
   falls through to the pre-existing scheduling backstop (silent drop of
   that one dispatch call, not the registration) rather than E5506;
   `addEventListener` out-of-lane arguments still fail closed (no corpus
   needs a failing build there, and compile-time rejection is strictly
   safer at that surface). This was presented back to the user and
   ratified; spec §2 was amended in commit `e9fba32b0`
   ("dispatch-arg reconciliation (user-ratified) + free-receiver
   fail-closed widening note") to record the residual as an inventoried
   Stage P3 item (below) rather than a silently-accepted gap.
6. **EV Task 5 (this task)**: verification and close-out on the final tree
   (`e9fba32b0`, one docs commit past Task 4's gate) — the 4 build tests
   stay green UNTOUCHED, a browser-lane execute test is added, the gate is
   re-confirmed restored, and this section records the trail.

## 8.2 Lane envelope (what actually compiles and runs)

In-lane (compiles to a real runtime registration/dispatch, all node-verified
byte-for-byte against kali):

- `new EventTarget()` assigned to a `let`/`const` in the constructing
  function (or module scope) — a provable, non-escaping handle.
- `target.addEventListener(<string literal>, <callback>)` where the
  callback has zero parameters and a provable-provenance body (named
  function, arrow, function expression, or a resolvable alias) — same
  capture-closure machinery as Stage C/D's timer and microtask lanes
  (scalar + object capture, depth 0/1).
- `target.dispatchEvent(new CustomEvent(<string literal>))` on an in-lane
  receiver — runs every registered listener synchronously, in registration
  order, deduplicated, returns the DOM-standard boolean.
  - CORRECTION (Task 9 review): the parenthetical claim that the
    `new Event(<string literal>)` shape is ALSO in-lane is FALSE. `event_dispatch_literal`
    recognizes only `new CustomEvent(...)`; a `new Event("tick")` argument falls
    OUT of lane and is SILENTLY dropped (no listener fires, no diagnostic).
    Probe p21: `dispatchEvent(new Event("tick"))` → node `fired\ndone`, kali
    `done`. This is a residual of the out-of-lane-argument class (§8.6), not an
    in-lane shape.
  - CORRECTION (Task 9 review): "Module-scope listener registration + later
    in-function dispatch" is likewise NOT reliably in-lane. Probe p39
    (module-scope `const t = new EventTarget()`, module-scope
    `t.addEventListener(...)`, dispatch from inside a `function go(){ ... }`)
    SILENTLY under-fires: node `fired\ndone`, kali `done`. The listener registry
    (`event_target_locals`) is per-`FunctionEmitter`, so a receiver whose handle
    provenance is recorded in the MODULE emitter is not visible to the dispatch
    site inside `go`'s emitter (split-scope limitation) — the dispatch falls to
    the backstop and drops. Its mirror p39b (register from inside a function,
    dispatch at module scope) drops the same way. Both are residuals of the
    registered-but-under-fired class (§8.6), fully silent (no E3100).

Fail-closed (E5506, a provable soundness gap — NOT a silent drop):

- Non-literal event-name argument to `addEventListener`.
- A listener callback with a parameter (no `Event`-object repr exists yet,
  so the parameter's value would silently be `undefined`/wrong).
- A 3rd (`options`) argument to `addEventListener`.
- `removeEventListener` on an in-lane handle (an escape-discipline guard —
  allowing it to build while not implementing it would let a later dispatch
  silently diverge from node, which still fires the "removed" listener).
- `dispatchEvent` on a captured (cross-function/closed-over) receiver — a
  proven silent-miscompile class closed during EV Task 4 TDD (register in
  an outer function, dispatch from a captured inner one: node fires, kali
  was silently no-op).

Preserved-but-inert (out-of-lane; build succeeds, the specific call is a
silent no-op at that call site — the inventoried residual, §8.4):

- `dispatchEvent` with an out-of-lane argument (e.g. `CustomEvent` with a
  `detail` object literal, or a bound/captured `Event` value) called on an
  otherwise in-lane, registered receiver.
- Any receiver whose EventTarget-ness cannot be statically proven (e.g. a
  bare parameter like `signal` in `signal.addEventListener(...)`).

## 8.3 Gate numbers

| Checkpoint | newly-red (`comm -13` vs `stageD-pre.txt`, 731) | drain (`comm -23`) |
|---|---|---|
| Task 7 (parser flip landed, pre-EV) | 4 (the 4 pins re-pinned/closed by this lane) | 37 |
| EV Task 4 (lane landed) | **0 (EMPTY)** | **37** |
| EV Task 5 (this task, final tree `e9fba32b0`) | **0 (EMPTY)** — re-confirmed | **37** — re-confirmed, identical set to Task 4's |

So the headline is **4 → 0 newly-red**, drain steady at **37** throughout
(the drain is the pre-existing block-arrow-un-flatten iteration-lane fix
carried from Task 7, unrelated to and unperturbed by the event lane — see
§8.3.1). `cargo build -p kali_cli`: zero warnings, both at Task 4 and at
this task's re-run.

### 8.3.1 Drain family classification (37, unchanged by the EV lane)

All 37 are pre-existing object/for-of/Set iteration fixtures that went red
at Task 7 (the block-arrow un-flatten) for the RIGHT reason and are fixed by
it — see §7.3. None involve `EventTarget`/events; the EV lane neither added
nor removed any of them. Families (by shared root):

| Family | Count | Representative |
|---|---|---|
| `for_of_break_continue` (browser-harness, `test`/`json_test` × js/jsx/ts/tsx) | 8 | `test_supports_for_of_break_continue_when_browser_harness_is_configured_in_js_input` |
| `object_keys` iteration (direct/global/from_entries/break_continue/literal, js/ts/jsx+tsx) | 13 | `test_supports_object_keys_iteration_in_js_input` |
| `integer_like_object_keys_iteration` (browser-harness, `test`/`json_test`) | 4 | `test_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_js_input` |
| `object_values` iteration (direct/from_entries, js/ts/jsx+tsx) | 6 | `test_supports_object_values_iteration_in_js_input` |
| `frozen_object` enumeration/values iteration | 2 | `test_supports_frozen_object_enumeration_iteration_in_js_ts_jsx_tsx_input` |
| `object_string_enumeration` iteration | 1 | `test_supports_object_string_enumeration_iteration_in_js_ts_jsx_tsx_input` |
| `set_constructor` iteration (js/ts/jsx+tsx) | 3 | `test_supports_set_constructor_iteration_in_js_input` |
| **Total** | **37** | |

## 8.4 Corpus audit table (from EV Task 4's Step 7 sweep)

Full detail in `/workspace/.superpowers/sdd/ev-task-4-report.md`; summarized:

| Fixture | EventTarget usage | Disposition |
|---|---|---|
| `write_web_baseline_interop_source` (misc/utility/browser_corpus) | in-lane construct + registration + in-lane dispatch + bound/out-of-lane dispatch args + out-of-lane (`signal`) receiver | Out-of-lane dispatch args preserved (build succeeds, backstop no-op) → 3 `browser_corpus…deployable_through_host*` tests stay/return green. |
| `write_web_baseline_test_source` (Kali.test wrapper) | same, inside `Kali.test` | Already-red baseline (other unsupported APIs — `AbortController`/`URL`/etc.); stays red, no flip needed. |
| `write_browser_string_web_baseline_package` (browser_corpus/browser_runtime) | in-lane construct/register/dispatch | Builds; corpus tests already red-in-baseline for other reasons; no flip needed. |
| `structured_clone_and_event_primitives_source` (`runtime_smoke/test.rs` ×3) | fully in-lane events | **Deliberate flip**: events now build+run; fail-closed shifted to the `structuredClone` deep-clone runtime throw (E4000). Assertion broadened to accept `E4000` in stderr; `success==false` invariant preserved. |
| `browser_bundle_web_baseline_source` (`runtime_smoke.rs:4462`, the 4 Task-7 pins) | fully in-lane events + out-of-lane `URLSearchParams`/`URL`/`TextEncoder`/`TextDecoder`/`AbortController` at runtime | Build succeeds (events genuinely compile); the pins assert build-success only, unchanged by this lane — see §8.5. Execution still traps on the not-yet-supported families (unaffected by this lane; those are Stages P2-P5, §8.6). |

The 11 baseline web-baseline corpus tests already red pre-EV (unsupported
`AbortController`/`WebSocket`/`structuredClone`/`URL`/…) stay red — not
newly-red, no regression.

## 8.5 EV Task 5 deliverables (this task)

1. **4 build tests green, UNTOUCHED**: `cargo test -p kali_cli --test
   runtime_smoke browser_bundle_web_baseline_primitives` → 4 passed, 0
   failed. `git diff` on `crates/kali_cli/tests/runtime_smoke/build.rs`
   confirmed a pure insertion (77 lines added, 0 removed, 0 modified) — the
   4 pre-existing test bodies are byte-for-byte unchanged.
2. **Browser-lane execute test**: `build::browser_bundle_event_lane_executes`
   added immediately after the 4 web-baseline build tests. Fixture (node
   v26.5.0-verified first, plain and tree-shake-wrapped forms, both produce
   `before=0\nafter=1\n`):
   ```js
   // kali-tree-shake: eventLaneSmoke
   function eventLaneSmoke(left, right) {
     const t = new EventTarget();
     let n = 0;
     t.addEventListener("tick", function () { n += 1; });
     console.log("before=" + n);
     t.dispatchEvent(new CustomEvent("tick"));
     console.log("after=" + n);
     return left - left;
   }
   ```
   Built with `--bundle --api browser`, executed by mirroring
   `assert_browser_bundle_executes_with_result`'s helper calls
   (`kali_runtime::browser_bundle_harness_script`,
   `browser_bundle_harness_command_parts`, `Command::new(&harness_executable)`)
   inline, asserting the FULL captured stdout equals `"before=0\nafter=1\n"`
   exactly (not a `contains` check — the load-bearing property is the
   *ordering* of the two `console.log`s around the synchronous dispatch,
   proving the callback ran exactly once, exactly before the second log).
   `cargo test -p kali_cli --test runtime_smoke
   browser_bundle_event_lane_executes` → 1 passed.
3. **Gate restored**: §8.3 above — newly-red EMPTY, drain 37, re-confirmed
   on `e9fba32b0`.
4. **This triage section.**

## 8.6 Follow-up inventory

Carried forward for later stages / later tasks (none block this lane's
soundness — every item here is either an explicit preserved-but-inert
residual with a closing plan, or unimplemented API surface that fails
closed or stays pre-existing-red rather than miscompiling):

- **Out-of-envelope dispatch-arg silent-drop residual** (§8.1 item 5,
  user-ratified): an out-of-lane `dispatchEvent` argument on an in-lane,
  registered receiver is a silent no-op at that call site rather than
  E5506. Not observed by any current test; closing plan is Stage P3
  converting this to a total-deny (fail closed instead of fall through)
  once the receiver-widening and captured-receiver work below lands enough
  provenance to make total-deny non-regressive against the corpus.
- **Out-of-lane NON-CAPTURING listener silent-drop residual (pre-existing,
  distinct from the dispatch-arg item above)**: `x.addEventListener(lit, cb)`
  on an UNPROVEN receiver (e.g. a `signal` param, any unknown object) with a
  non-capturing callback still takes the pre-lane backstop and is a FULLY
  SILENT no-op registration (capturing callbacks on such receivers stay
  E5506). CORRECTION (Task 9 review): the earlier wording claimed this emits an
  "E3100 placeholder warning" — that is WRONG. The drop is completely silent:
  no E3100, no diagnostic at all. Probe p39b (register from inside a function,
  dispatch at module scope) and p03c (a user `class EventTarget` shadowing the
  builtin) both print only `done` with an empty stderr — node fires the
  listener, kali drops it silently. This is the design spec's named "top
  inventory item for Stage P3": receiver widening plus the backstop →
  total-deny conversion closes it. The dispatch-ARG item above is a different,
  newer, user-ratified residual (out-of-envelope argument on an IN-lane
  receiver); do not conflate the two.
- **`Kali.test` member-expression callback wrong-function class — CLOSED
  (Task 9, I-4)**: `Kali.test("x", obj.m)` previously resolved the callback
  node's PROPERTY text (`m`) to an unrelated module function `m`, RAN it, and
  printed a false `ok 1` (worse than a vacuous ok — it ran the wrong function).
  `kali_test_callback_index` now applies the same bare-identifier structural
  gate the scheduling resolver uses (resolve by text only for an inline
  function-plan node or a childless bare identifier); a member/index-expression
  callback (a `Value` node WITH children) routes to the unregisterable-value
  deny lane (E5506) instead. Pinned by
  `soundness_block_arrows.rs::kali_test_member_expression_callback_fails_closed`
  (asserts E5506 and that the wrong function never runs).
- **Registered-but-under-fired divergence class**: shapes where a listener
  is registered but a subsequent in-lane dispatch fires it fewer times
  than node (as distinct from the out-of-lane-argument silent-drop above —
  this is about receiver/handle aliasing across `FunctionEmitter` scopes, not
  argument shape). CORRECTION (Task 9 review): the earlier "not observed by any
  current test" note is STALE — probe p39 observes it directly (module-scope
  registration, dispatch from inside `function go(){…}` → node `fired\ndone`,
  kali `done`; the per-`FunctionEmitter` `event_target_locals` registry is not
  shared across the module and `go` emitters). Still flagged for Stage P3
  alongside the backstop hardening.
- **Stage P2 — `structuredClone`**: deep-clone runtime primitive; currently
  traps (E4000) wherever it's the first unsupported call in a fixture (see
  §8.4's deliberate flip).
- **Stage P3 — `AbortController`/`AbortSignal`**, bundled with:
  - receiver widening (proving more `EventTarget`-shaped receivers in-lane,
    e.g. `signal` params from an `AbortController`),
  - backstop → total-deny for the out-of-envelope dispatch-arg residual
    above,
  - captured-receiver support (the currently-denied captured-handle case,
    §8.2, promoted from deny to a real cross-function dispatch once the
    env-pointer/closure machinery can prove it safe),
  - an `Event`-object repr (lifting the current zero-parameter-listener
    restriction — `preventDefault`/`cancelable`/`target`/`type` on the
    callback's argument all depend on this).
- **Stage P4 — `URL` + `URLSearchParams`**.
- **Stage P5 — `TextEncoder`/`TextDecoder`**.
- **Final byte-for-byte `webBaselineSmoke` acceptance**: once P2-P5 land,
  execute `browser_bundle_web_baseline_source` (or its `webBaselineSmoke`
  export) end-to-end — via `kali run`, the browser lane, AND by flipping
  the 4 Task-7 build tests (§8.5.1) to also execute and assert real output
  — byte-for-byte against node, closing the loop this task's brief opened.
- **`removeEventListener`**: currently fail-closed (escape-discipline
  guard, §8.2); a real implementation is future work, not required for
  soundness (fail-closed is safe).
- **`preventDefault` / `cancelable`**: depend on the Event-object repr
  above; currently unreachable (no listener can observe an event object at
  all under the zero-parameter gate).
- **Non-lowered SCALAR-capture deferred-callback fail-open — RESOLVED
  (scalar-only, user-ratified) (Task 9, C-1)**: the four registration surfaces
  (`queueMicrotask` / `setTimeout` / `setInterval` / `addEventListener`) resolve
  any stable-provenance callback, but `env_safety` only CONSTRAINS captures whose
  closure lowering is engaged (`depth == 1 && cell_is_promotable`). A callback
  capturing a non-lowered SCALAR-class binding — a PARAM, a STRING-repr, or a
  FLOAT-repr — registered and RAN reading a placeholder 0, diverging from node
  which computes a real value (probes p36e `i=6`→`i=0`, p53b `hi`→``, p56
  `1.5`→``, p55 `i=3`→`i=0`, p54 `p=7`→`p=0`). **Fix (SCALAR-ONLY deny at the
  shared `scheduling_callback_at` choke point, inherited by all four surfaces):**
  a resolved callback whose env plan carries a non-lowered capture that is
  SCALAR-class fails closed E5506 with the class named
  (`scalar_unlowered_capture_class` in `intrinsics/host.rs`; the shared deny
  emitter `deny_deferred_scalar_capture` in `emit/call.rs`). SCALAR-class =
  `is_scalar` slots (String→`"string"`, F64→`"float"`, surviving I64→`"number"`
  — depth-1 I64 scalars are always lowered) PLUS captured PARAMETERS. NB a
  captured param is `is_scalar == false` with default `Repr::I64` at the env-plan
  level — the SAME shape as a `new AbortController()` capture — so the two are
  separated ONLY by whether the binding names a declared parameter of its owner
  (the newly-threaded `function_param_names` consult). Pinned by 5 E5506 tests
  (`deferred_settimeout_captured_param`/`_string`/`_float`,
  `deferred_queuemicrotask_captured_param`,
  `deferred_event_listener_captured_param` in `soundness_events.rs`).
  **DOCUMENTED RESIDUAL (deliberately ALLOWED):** a non-lowered NON-scalar
  capture that is NOT a param — a zero-placeholder unsupported construct
  (`new AbortController()`), or a captured array/object local — stays allowed.
  Rationale: no correct value exists to be wrongly replaced. Such a binding is a
  placeholder 0 in its OWNER's scope too (it reached the E3100 fallback / has no
  supported repr), so its in-callback read equals its out-of-callback read of the
  same placeholder — there is nothing to diverge, unlike a scalar where node has
  a real value. This is exactly what preserves the 4
  `browser_bundle_web_baseline_primitives` build tests: `webBaselineSmoke`'s
  listener `() => { count += 1; controller.abort(); }` captures `count` (a
  promotable I64 scalar → lowered, correct) AND `controller` (the AbortController
  placeholder → allowed residual), so its "unsupported constructs must still
  BUILD (warn, not error)" invariant holds. Pinned by
  `deferred_listener_nonscalar_placeholder_capture_still_builds`
  (`soundness_events.rs`). **Lifting plan:** Stage P3 (`Object` repr for these
  constructs) promotes them into the lowered/constrained set — at which point the
  capture becomes either genuinely lowered (correct) or `env_safety`-constrained,
  closing the residual by construction rather than by deny.
- **Negative-clear-id deliberate-loud divergence (Task 9 note)**:
  `clearTimeout(-1)` / `clearInterval(-1)` is a NO-OP in node (prints `ok`),
  but `kali run` traps LOUDLY — `KaliHostState::cancel_timer` does
  `u32::try_from(timer_id)` and a negative id fails it, surfacing as an
  E4000 runtime trap (probe p33: node `ok`, kali `error[E4000]: runtime
  trap`). This is a DELIBERATE fail-LOUD divergence (a trap, never a silent
  miscompile), inventoried here rather than "fixed": accepting a negative id
  as a no-op is a precision follow-up, not a soundness gap. The JS mirrors
  were previously LENIENT here (they accepted a negative id and no-op'd it,
  diverging from Rust's trap); after I-1 they still do not throw, but the
  insert-gate (`id >= 0 && id < kaliNextTimerId`) means a negative id can no
  longer land in `kaliCancelledTimers` (it is a clean no-op on both sides
  except for Rust's loud trap surface). Aligning the two — either making JS
  trap or making Rust no-op — is deferred to the same precision follow-up.
- **Timer-id base drift — ALIGNED (Task 9, I-1)**: the Rust runtime started
  `next_timer_id` at 0 (`state.rs`) while all 4 JS glue mirrors started
  `kaliNextTimerId` at 1 — an opaque-id divergence that was benign for id
  VALUES but masked the I-1 stale-clear bug asymmetrically (the first Rust
  interval was id 0, the exact id a pre-registration `clearInterval(0)`
  poisoned; the JS base of 1 hid it). No test pinned the JS base, so the 4
  mirrors (`harness.rs` ×2, `cmd_build.rs` ×2) were aligned to base 0,
  matching Rust. Combined with the shared insert-gate, the Rust and JS
  timer-cancellation semantics are now identical by construction.
