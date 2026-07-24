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

> **PR #16 honest re-pin map:** the canonical adjudication table for the
> honest-red workspace tests (694 unique names) — per-family evidence, A/B/C
> class calls, deny-lane/pin actions, and the coverage ledger — lives in
> [`pr16-honest-repin-inventory.md`](./pr16-honest-repin-inventory.md), not
> duplicated here. Consult it before instantiating any re-pin wave task.

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
- **Stage P2 — `structuredClone`**: **SHIPPED 2026-07-18**
  (`c893d5835..7e3aacc02`, 19 commits on `soundness-batch1-pra`). Deep clone
  over the sound envelope: flat fixed-shape objects of i64/f64 scalars +
  growable-i64 array fields (`Repr::GrowableArrayI64`, per-shape
  `__clone_shape_N` synthetics on `__alloc_global`, intern-time `clone_safe`
  allowlist bit), same-shape `===`/`!==` pointer identity, placeholder
  warn-build lane with const-provenance chains (E8001 guard pin). §8.4's
  deliberate flip advanced: the fixture now denies at `AbortController`'s
  `instanceof` (first P3 primitive), `success == false` preserved.
  Gate: 694-baseline held, 0 newly-red, double-enumerated, zero drift.
  P2 residual inventory (P3-relevant): envelope widening to string/nested
  object fields (identifier/call/`arr[i]` object-pointer fields fail closed
  by the clone-safe bit); named-growable alias `const b = a` still a
  tripwire-pinned fail-open; growable-field OOB index read `0` vs node
  `undefined`; object reassignment (`o = {…}`) zeroes reads (I-1 tripwire pin
  — the GrowableArrayI64/scalar intern AND-merge must be revisited when
  reassignment lands); general member-on-call placeholder hole
  (`mk().a` → 0) still open, `structuredClone(...)` callee scoped-denied.
- **Stage P3 — `AbortController`/`AbortSignal`**: **SHIPPED 2026-07-18**
  (`15c2b34f9..7cdbe2437`, 14 code/test commits on `soundness-batch1-pra`).
  Real lowering, not a placeholder: an 8-byte never-reclaimed
  `__alloc_global` abort cell; controller and signal share the same i64
  handle by compile-time provenance (`Repr::AbortHandle`, seeded in
  `kali_types` for both the const-declarator `new AbortController()` and a
  controller-origin `.signal` alias, via one shadow-guard traversal covering
  decls/exprs/params/catch/switch/fn-expr bodies); codegen tracks
  `abort_handle_locals` with owner-keyed capture proof and a position
  allowlist at the bare-identifier choke point
  (`admit_abort_handle_read`, set only by `emit_abort_receiver_handle`).
  Surface: `.abort()` dispatch; `.signal` identity + `.aborted` cell read
  (Boolean 1/0); `const s = c.signal` alias lane; compile-time
  `instanceof AbortSignal` folding (both-sides-proven, five-namespace
  shadow guard); capture allowlist entry 3 for function-scoped deferred
  callbacks; module-boundary fail-closed via
  `is_module_scope_abort_handle` at the method-call, identifier-read, and
  member-read choke points (covers top-level bindings AND `_start`
  loop/block-body locals); `AbortSignal` statics denied (dot and
  computed); a 16-sink enumeration wave; 55 pins in `soundness_abort.rs`.
  Acceptance: `acceptance_web_baseline_prefix_matches_node_byte_for_byte`
  — the web-baseline fixture prefix runs byte-for-byte against node,
  function-scope wrap (module-scope capture stays fail-closed by design
  this stage). Fixture provenance (Task 7 note): the acceptance prefix is
  the web-baseline fixture MINUS the `Event`-type block (pre-existing gap)
  MINUS the `URLSearchParams`-onward tail (Stage P4/P5).
  Gate: honest-red stage-base baseline, 0 newly-red at every task and
  at close-out, double-enumerated with zero drift. **712 vs 694 resolved
  (see `pr16-honest-repin-inventory.md`):** 712 = failing test INSTANCES,
  694 = unique test NAMES. 18 root-scope test-fn names are defined in TWO
  test binaries each (same name, two files), so a per-instance count sees
  them twice; the enumeration recipe's `sort -u` is name-set based. Both
  are honest measurements of the same red set — neither is an undercount.
  **Whole-stage review (2-round fix wave):** the 5th consecutive stage
  where whole-stage review caught a CRITICAL no per-task review saw: a
  `_start` loop/block-body captured abort handle was silently
  miscompiled — the capture was admitted via ALLOWLIST 1 (by-value
  scalar), but the env cell was never written, so a deferred `c.abort()`
  no-oped and reads returned the placeholder `0`. Fixed across two commits
  (`8c675bce2` closing the write/method choke, `7cdbe2437` closing the
  read-side choke); round-1's ALLOWLIST-3 guard was proven inert and
  removed with an honest NOTE. Verified closed by fresh-probe adversarial
  verification: 4 reproducers now E5506, 8 capability shapes unregressed,
  a 7-sink admittance sweep all denied.
  Residual inventory:
  1. Deferred P3b bundle from the plan: receiver widening, both
     total-deny conversions, p39, an `Event`-object repr, and
     signal-as-`EventTarget` (including re-greening the 3 deliberately
     flipped `browser_corpus` web-baseline build pins — flip commit
     `5a7fb5faa`, fixtures left untouched as tripwires); also
     `instanceof AbortController`, `s.reason`/`throwIfAborted`/statics, and
     abort-handle `===` identity.
  2. Plain alias `const b = c` stays fail-closed (deliberate).
  3. Dynamic-boolean 1/0 render divergence for a printed `.aborted` is a
     ratified convention and excluded from acceptance.
  4. `AbortSignal[<operator-token>](x)` computed static call fails open to
     `ran:0` — a degenerate case (node throws `TypeError`); the
     `!is_binary_operator_text` guard can't be naively removed because real
     `AbortSignal + 5` binary nodes hit the same predicate.
  5. Inference shadow-scan blind spots: destructured/defaulted param
     patterns and import/export declaration names have no Phase-B
     `visit_stmt` arm. Benign today via the escape chokes; a latent
     divergence tripwire.
  6. TRIPWIRE: `_start`-owned abort-handle captures remain ADMITTED by
     capture ALLOWLIST 1 (by-value scalar) — soundness rests entirely on
     the three consumer choke points. Any future consumer path that reads
     an env cell without going through the identifier lane (the
     `try_emit_captured_*` family, today blocked only by the
     const-mutability gate) must re-check `is_module_scope_abort_handle`.
  7. Pre-existing, surfaced but out of scope: the `new Event('tick')` /
     `event.type` silent-0 gap (the fixture's new fail-closed point;
     `runtime_smoke` flip pins re-pointed in `b71ae25a2` to the
     bare-callback-trap token, weakening the progression pin — revisit
     when `Event` lands); the generic warning-only undefined-call
     fallback's silent-sink breadth beyond `AbortSignal`; and
     module-scope non-const-foldable heap bindings (read was already
     fail-closed; this stage closed the write side for abort handles
     only).
- **Stage P4 — `URL` + `URLSearchParams`**: **SHIPPED 2026-07-21**
  (`a372b754e..af7ae9c1f` on `soundness-stage-p4`; spec/plan
  `e7fc4fb0a`/`a372b754e`, 12 implementation commits
  `b607099cb..af7ae9c1f`). Self-contained in-wasm hybrid:
  compile-time-parsed `new URL(<lit>)` lowered to a 6-slot arena
  struct and `new URLSearchParams(<lit>)` to a growable pair-store;
  URL component reads; `get`/`getAll`/`has`/`set`/`append`/`toString`
  via synthetic WASM fns (`__usp_get`/`__usp_has`/`__usp_getall`/
  `__usp_set`/`__usp_append`/`__usp_tostring` + `__percent_encode`)
  over `__streq`; `u.searchParams.get(...)` read-only composition;
  USP `get`/`toString` results admitted to the `__streq`
  content-equality lane (closing a raw handle-identity compare
  fail-open on dynamically-set values); the null sentinel
  materialized as `"null"` in the print/concat/store-arg lanes.
  Acceptance:
  `acceptance_web_baseline_with_url_matches_node_byte_for_byte` — the
  web-baseline URL/USP block runs byte-for-byte vs node
  (`web baseline url ok`).
  Gate: the baseline at stage base was 100% GREEN — 0 failed (the
  plan's "694 expected" was stale; PR #16 re-pinned the reds to
  green), so every task gated on "workspace stays 0-failed"; final
  double-enumerated gate 2×(0 failed / 9146 passed / 374 binaries),
  zero drift, with the W-1 follow-up re-verified per-suite
  (`soundness_url` 53/53, `kali_codegen` 355, `runtime_smoke` 1826,
  fmt clean). 53 pins in `soundness_url.rs`.
  **Whole-stage review:** 4 CRITICAL (C-1 composition mutation
  desyncing `u.search`/`u.href`; C-2 leading `?` not stripped; C-3
  `append` mutating between key push and value evaluation; C-4
  block-scope shadow redeclaration wrong-value) + 5 Important (I-5
  null-sentinel rendering 0/empty; I-6 `get().length` child-count;
  I-7 `getAll()[0]` silent 0; I-8 empty-string truthiness; I-9
  assignment-into-binding wild load) — the 7th consecutive stage
  where the adversarial whole-stage review caught what per-task
  reviews missed. All fixed in wave `1640ffaf1` (plus F10 deny-now
  and the F11 free multibyte pin) + the W-1 follow-up `af7ae9c1f`;
  all VERIFIED-CLOSED by fresh-probe re-review (no over-deny;
  positive controls green).
  Recorded fixture adaptations (Task 7): `String(count)` →
  `'' + count` (G6 `String` deny); the get-compare → `!== '1'`
  (runtime-vs-dynamic string compare is E3200 fail-closed by design);
  untaken-path template-literal throws → plain concat (from the
  plan's literal fixture text); `function main()` wrapper (P3
  precedent). Flip pins remain at the compile-time `String` E5506
  frontier (5 sites) — the TextEncoder advance is blocked on the
  String deny, NOT yet at the P5 frontier.
  Ratified conventions: `.has` renders 1/0 (P3 `.aborted` precedent);
  `.set`/`.append` in value position render the void placeholder 0.
  Residual inventory:
  1. P4-R1: `u.searchParams` composition READ-ONLY (set/append denied
     at the `OfUrl` arm); tripwire: any future admit of composition
     mutation MUST re-derive `u.search`/`u.href` from the live store
     or deny those reads after mutation.
  2. P4-R2: repr seeds `usp_bindings` for `const sp = u.searchParams`
     but codegen denies at the declarator (sound). Tripwire:
     admitting the alias requires codegen provenance +
     arena-lifetime proof, never repr alone.
  3. P4-R3: CLOSED by F10 — inline/`let` ctor deny (deliberate
     deny-upgrade of the Task-6 honest-behavior pin, recorded at the
     pin).
  4. P4-R4: general receiver-dropping placeholder lane
     (`(u+1).get(...)` → 0) — pre-existing whole-compiler debt, not
     URL-keyed, tracked outside P4.
  5. P4-R5: CLOSED by C-3 — atomic `__usp_append` synthetic; no
     emitter scratch live across argument emissions.
  6. P4-R6: CLOSED by F11 — multibyte/reserved percent-encode pinned.
  7. P4-R7: deliberate over-denies as supported-surface boundaries
     (all E5506, never wrong-value): zero/two-arg/non-literal ctors,
     ctor outside the const-declarator shape, getAll
     binding/element/join, `.length` + condition position on
     get/toString results, for-of, destructuring, typeof, assignment
     into a URL/USP binding, redeclaration of a URL/USP name (both
     orders), OfUrl mutation.
  8. P4-R8: CLOSED — null-sentinel materialized in
     print/concat/condition/store-arg (I-5 + W-1).
  Outside-P4 (general inventory, not §8.6-scoped): generic
  block-scope flatten (const redeclaration in an inner block leaks
  the value outward, no URL involved) — pre-existing; C-4 closed only
  the URL/USP slice. Also `is_new_abort_controller`'s
  `args.is_empty()` looseness (Task-2 discovery: the parser folds
  args into the callee `CallExpression`).
  Note: after P5 (`TextEncoder`) AND a String-builtin lane land, the
  final byte-for-byte `webBaselineSmoke` acceptance runs the whole
  fixture three ways (`kali run` + browser + flipped build tests) —
  see the acceptance bullet below.
- **Stage P5 — `String()` coercion + `TextEncoder`/`TextDecoder`**:
  **SHIPPED 2026-07-23** (`38797be9e..2448dd883` on
  `soundness-stage-p5`; spec/plan `051f6b33f` and earlier —
  `docs/superpowers/plans/2026-07-22-p5-textcodec-string.md`;
  18 implementation/test commits `38797be9e..2448dd883`, list
  confirmed against `git log --oneline 051f6b33f..HEAD`:
  `38797be9e` `41b0156d0` (Task 1 String coercion),
  `ebfae19d7` (Task 2 `Repr::Bytes`),
  `c91ad51c6` `33926f5c9` (Task 3 encode→Bytes),
  `396e7f3b0` `7a6da215a` `06b6dcc87` (Task 4 decode),
  `8cd1f3c83` `b73a45c6d` `f5217e65a` `19c8c7274` (T-new-B
  encode-of-`String()`), `83c4a0c0c` `590187072` (T-new-A
  `getRandomValues` length), `95e43638d` `e14c40004` (T-new-C
  Event `.type`), `baf431e29` (T-new-D unified shadow guard),
  `2448dd883` (Task 5 re-pin)).
  What shipped: `String(x)` runtime coercion via `emit_as_string`
  (the terminal-deny-set `String` entry REMOVED), with a positive
  argument proof (`string_coercion_arg_is_proven`) so only shapes
  `emit_as_string` renders soundly are admitted and the
  function-valued/aggregate holes fail closed; an inert
  `Repr::Bytes` opaque provenance handle (Task 2, grouped with
  Url/USP → I64 at all lowering sites, never co-grouped with
  String/F64); `encode`→`Repr::Bytes` for bound+inline receivers
  with `crypto.subtle.digest` migrated to admit Bytes and the
  escape choke extended (produce-side `admit_bytes_handle_produce`
  twin for unbound producers); net-new `decode` relabelling
  Bytes→String with zero-arg-ctor + unshadowed-ctor guards;
  `encode` admitting a bare `String()` result as its argument via
  three POSITIVE `repr_infer` allowlists (`numeric_shape_fields` /
  `numeric_returns` / `numeric_bindings`) — each load-bearing
  against a measured wrong-value defect, because `Repr::I64` is the
  UNRECORDED DEFAULT, never evidence; `crypto.getRandomValues`
  result now carries its buffer length so `.length`/`.byteLength`
  read correctly (was silent 0 → bundle built but TRAPPED); Event
  `.type` via a compile-time marker riding `__streq` content
  equality; and a unified `stale_provenance_shadow_lane(name)` guard
  (`emitter.rs:651`) ORing all EIGHT name-keyed handle/marker lane
  predicates, called from BOTH binding chokes (declarator
  `control_flow.rs:465` + for-of/for-await `~:1774`), closing the
  block-redeclare + for-of shadow hijack family across every lane
  in one place.
  Acceptance: `webBaselineSmoke`
  (`browser_bundle_web_baseline_source`) and the crypto
  `digestSmoke` fixture BUILD and EXECUTE byte-for-byte vs node
  v26.5.0 — the crypto/web-baseline bundle pins assert EXECUTION
  (`assert_browser_bundle_executes_with_result` → `digestSmoke(1n,2n)
  == 0`), not just build, closing the build-only gap that let the
  original `getRandomValues` silent miscompile pin green. 44 stale
  fail-closed pins (the pre-P5 "`String()` fails closed E5506" era)
  reconciled to node-correct behavior across 5 test files, every
  assertion EXACT-EQUALITY so a wrong-reason pass is structurally
  unavailable.
  Gate: controller-independent `cargo test --workspace
  --no-fail-fast` = **9294 passed / 0 failed** (matches implementer
  + reviewer). 96 pins in `soundness_textcodec.rs` plus 12 Task-6
  boundary tripwires (`p5_boundary_*` / `p5_r_*`).
  **Per-task reviews (opus):** Task 3 APPROVED (1 Important
  fail-open closed: inline-unbound `encode().byteLength`/`.length`);
  Task 4 APPROVED round 3 (4 Criticals: C-1 TextDecoder ctor-arg
  ignored — non-UTF-8 labels silently decoded as UTF-8; C-2 inline
  ctor hijacked a user-defined `TextDecoder`; C-3 let/var-bound
  markers silent-0; C-4 UNBOUND encode result escaped the choke);
  T-new-B APPROVED round 4 (3 of 4 Criticals were the same
  `Repr::I64`-as-evidence fallacy in a new position, closed by the
  three positive allowlists; one FALSE-DRAIN caught — a `let`/`var`
  over-deny turned 3 tests green for the WRONG reason); T-new-A
  APPROVED round 2 (I-1 = the task's own silent-0 surviving one
  scope inwards, into a capturing closure); T-new-C APPROVED round 2
  (C-1 marker redeclaration lacked the guard the sibling URL lane
  already shipped, + a for-of sibling one lowering away); T-new-D
  APPROVED (hoisted the per-lane guards to one choke). Task 5
  APPROVED (execution-mutation reproduced: expected 0→7 makes the
  harness throw, proving the wasm runs and its return value is
  observed).
  **HEADLINE LESSONS:** (1) narrowing a recognizer is NOT a fix —
  the rejected remainder falls through to a silent-0 fallback;
  narrow AND explicitly deny. (2) An escape choke on BOUND handles
  does not cover UNBOUND producers — a value class needs a
  produce-side twin. (3) A probe that uses a bound handle MASKS the
  unbound hole; vary the binding form in probes, not just the shape.
  (4) `Repr::I64` is the UNRECORDED DEFAULT, not proof — a soundness
  proof must be a POSITIVE allowlist, never "no taint recorded". (5)
  A test turning green is not evidence of progress; check WHY it went
  green (the false-drain). (6) A name-keyed flat side-table
  redeclaration hazard is STRUCTURAL to every lane — hoist ONE guard
  to the binding chokes rather than let each lane remember two
  places (5 of 7 remembered neither).
  Ratified conventions: any `new TextDecoder(<arg>)` — including the
  explicit default `'utf-8'` — is E5506 (conservative over-deny);
  `.has`-style boolean reads render 1/0 (P3/P4 precedent).
  **USER DECISION (AskUserQuestion, 2026-07-23) — BINDING:** accept
  the re-pin to fail-closed for the two Stage-D pins
  `event_custom_event_with_detail_out_of_lane_*` and
  `event_bound_event_argument_out_of_lane_*` — they asserted
  `new CustomEvent(...)` BUILDS, but what it built was a silent
  listener DROP; CustomEvent stays unsupported but stops lying (blast
  radius exactly those 2 tests).

  **P5 residual inventory.** Each item is marked **[DELIBERATE
  OVER-DENY]** (fail-closed boundary, E5506, never a wrong value) or
  **[SILENT MISCOMPILE]** (exit 0, wrong value — the dangerous
  class). All measured on the fresh HEAD binary; pre-existing unless
  noted.

  Deliberate over-denies (pinned as Task-6 tripwires, all E5506):
  1. P5-R7-boundaries **[DELIBERATE OVER-DENY]**: zero-arg
     `String()`, multi-arg `String(1n, 2n)`, function-valued
     `String(() => 1n)` / `String(namedFn)` (the Task-1 hole),
     nested bytes handle `[b][0]` (the escape choke), any
     `new TextDecoder(<arg>)` incl. `'utf-8'` (the T4/T-new-C ctor
     boundary), `d.decode('hi')` / `d.decode(42n)` (non-bytes decode
     args), and the for-of + block-redeclaration shadows of a codec
     or bytes-handle name (the T-new-D unified guard). Never a wrong
     value; a future accidental admit turns the pinned tripwire red.
  2. P5-R-utf8-label **[DELIBERATE OVER-DENY]**: explicitly-`'utf-8'`
     TextDecoder labels are denied along with non-default labels (any
     ctor arg ⇒ E5506). Conservative by design; revisit if a fixture
     needs the default label spelled out.
  3. P5-R-let-array **[DELIBERATE OVER-DENY, partial]**: only `const`
     array aliases are guarded, so `let arr=[1,2,3]; String(arr)`
     prints `"0"` not E5506 — this ONE spelling is a silent
     miscompile (mirrors `console.log(arr)`=0; needs a kali_types
     let/var array-literal taint set, out of Task 1 scope). The
     function-value hole IS fixed; deferred WITH ticket.

  Silent miscompiles (exit 0, wrong value — DO NOT pin as expected):
  4. P5-R-globalthis-string **[SILENT MISCOMPILE]** (NEW, found
     Task 6): the member-call form `globalThis.String(1n)` prints
     `0` (exit 0, no warning) where node prints `1`; `globalThis.
     String(42)` and the const-bound spelling identical. Was listed
     in the brief as a boundary expected to deny — it does NOT deny,
     it silently folds the member call to 0 (the unresolved-member
     /call-folds-to-0 class, ≈ register R-02/G2). DROPPED from the
     tripwires and filed here instead.
  5. F-newB-1 **[CLOSED BY CONSTRUCTION — T-new-F 2026-07-24 (repr seed)]**:
     stage-introduced silent miscompile (merge-base `694607bb2` had `String`
     deny-set → fail-closed; P5 Task 1 de-denylisted it without a
     `repr_infer` `Repr::String` seed → `function g(y){return String(y)}
     const s=g(1n); 'x'+s` → `x-9223354375949254655`, node `x1`). T-new-E
     rounds 1-3 fail-closed it (a denylist of shapes+sinks, rejected 3× for
     leaking one position over — ternary/`&&`/`||`/sequence). **T-new-F is
     the STRUCTURAL close**: `repr_infer` now SEEDS `Repr::String` for a
     value proven MONOMORPHICALLY a `String()` result (every write /
     return-path / operative composite-arm is a String() result), reusing
     the round-2 value-flow fixpoint (`resolve_string_result_taint`) with
     added composite source arms (`ConditionalExpression`, and the
     BinaryExpression `&&`/`||`/`??` the parser really emits, and
     `SequenceExpression`). The seeded value renders correctly at EVERY
     string sink (`+`, template, console, `===`, `.length`) BY CONSTRUCTION
     — no sink enumeration. Numeric sinks fail closed on `is_string_valued`
     (node throws for BigInt/string). Non-monomorphic values
     (reassign-with-a-numeric-write, params, `&&`/`||`/`??`/sequence — the
     last two seed-UNSAFE because kali cannot test a string handle's
     truthiness and mis-emits sequence values) stay UNSEEDED and fail
     CLOSED via the round-2 taint BACKSTOP (kept). `let s=String(1n); 'x'+s`
     → `x1`; `s*2n` → fail closed. Gate 9349→9360, 0 newly-red; acceptance
     byte-for-byte. See the T-new-F report. Incidental fix: `is_string_valued`'s
     Call arm now resolves the fold-alias callee, closing a pre-existing
     silent raw-bit render for a fn-expr `const g=function(){return 'hi'}`
     (the expression-bodied ARROW literal twin remains — memory F-AB-1 — its
     return is never String-seeded by the normal solve; separate, out of scope).
  6. F-newB-2/3/4 **[SILENT MISCOMPILE]**: `String(v).byteLength`
     → 2 (node `undefined`); `String(v)[0]` / `String(v).repeat()`
     silent 0; `String(undefined)` → `false`, `String(null)` → `0`
     (node `"undefined"` / `"null"`).
  7. P5-R-modulescope-growable-push **[SILENT MISCOMPILE, HIGH —
     FILE AS ONE TASK]**: `push` on a module-scope growable binding
     is a silent NO-OP — element never lands, length header never
     increments. `const g=[]; g.push(7)` at module scope → `g.length`
     0, `g[0]` undefined, `g.join('-')` empty (node 1/7/7); ALSO
     dropped when the push is inside a function targeting a
     module-scope growable (`const g=[]; function add(){g.push(7)}
     add(); g.length` → 0 — the common real shape). Fixed-size
     module-scope arrays are fine. A silent WRITE loss is worse than
     a read divergence: every downstream reader sees a plausible
     empty array, `warnings:[]`, exit 0. Min close if the lane is
     expensive: fail closed on `push` whose receiver is a
     module-scope growable. (The earlier "module-scope growable join
     prints an empty line" is a DOWNSTREAM SYMPTOM, not an
     independent bug.)
  8. P5-R-aggregate-array-provenance **[SILENT MISCOMPILE, HIGH —
     OWN TASK]**: an array handle stored into an aggregate loses its
     length, silently, no crypto — `const rb=new Array(4);
     const o={buf:rb}; o.buf.length` → **1** (node 4); `holder[0]=rb;
     holder[0].length` → **2** (node 4). The emitted values are the
     CHILD COUNT / HOLDER LENGTH — maximally plausible wrong numbers,
     the worst class. T-new-A did only a cheap partial close (4
     chokes denying laundering of a deny-domain name into an
     aggregate). ≈ register R-14.
  9. P5-R-newA-residuals **[SILENT MISCOMPILE, Minor/Important]**:
     I-4 — the 4 aggregate chokes are a DENYLIST and 2 routes leak:
     `const z = fb; z.length` → 0 and `function mk(){return fb}
     mk().length` → 1 (node 4 both), silent; same array-provenance
     family as #8. Cheap partial for the alias route only; the return
     route is not name-keyable. M-3 (deny-seed over-attribution) is a
     sound over-deny; M-4 (`emit/object.rs:118` store gate) and M-5
     (`lower.rs:1571` assigns-not-extends the seed) are correct-by-
     inspection, unexercised — latent tripwires.
  10. P5-R-computed-length **[SILENT MISCOMPILE]**: computed
      `["length"]` on ANY runtime string → 0 —
      `id('hello')["length"]` → 0, `(t+'!')["length"]` → 0,
      `decoder.decode(b)["length"]` → 0, node 5/6/5. Static-fold
      `const t='hello'; t["length"]` → 5 correctly. Hits Task 4's own
      decode result. Not coercion-specific.
  11. P5-R-bytelength-undef **[SILENT MISCOMPILE]** (NEW, Task 4
      M-1): `.byteLength` on ANY runtime string returns a byte count
      where JS gives `undefined` (`const a='h'; (a+'i').byteLength`
      → 2). Pre-existing for all runtime strings, but the Task-4
      `is_string_valued` arm newly routes decode results into it, and
      it contradicts the `.length`-deny on decode results.
  12. P5-R-digest-operand-shape **[SILENT MISCOMPILE, HIGHER
      PRIORITY]** (Task 4 r3): digest-operand admittance is
      POSITION-scoped not SHAPE-scoped — the admit flag stays set
      across the whole operand subtree, so `digest('SHA-256', '' +
      e.encode('a'))` and `digest('SHA-256', id2(e.encode('a')))` are
      admitted and hash something node rejects (node needs a
      BufferSource; `''+Uint8Array` is a string → TypeError). Fix:
      give the digest operand the `arg_is_bytes_provenance` shape
      proof `decode` already has.
  13. P5-R-unbound-digest-member **[SILENT MISCOMPILE, HIGHER
      PRIORITY]** (Task 4 r3): `crypto.subtle.digest('SHA-256',
      new TextEncoder().encode('hi')).byteLength` → 0; bound form and
      node both → 32. Divergent number on the digest lane P5's OWN
      acceptance path uses. Fix = structural bail mirroring the
      `.length`-on-decode one.
  14. P5-R-array-elem-fold **[SILENT MISCOMPILE]** (Task 4 r3):
      `const a=[new TextEncoder()]; a.length` → 0, node → 1. NOT
      generic array behavior (`[new Foo()].length` → 1); a
      `render_length` single-element string-identity fold tunnelling
      into the element — same hazard class as the Task-3 fold bails.
      C-3's runtime choke cannot see it.
  15. P5-R-tostring-length **[SILENT MISCOMPILE]** (Task 4 M-4):
      `arr.toString().length` → **1** for both `new Uint8Array(4)`
      (node 7) and `new Array(4)` (node 3).
  16. F-newD-1 **[SILENT MISCOMPILE]** (T-new-D review): a BLOCK
      FUNCTION-DECLARATION shadow of a handle name bypasses BOTH
      chokes structurally — `{ function u(){} console.log(u.pathname)
      }` returns the outer handle's REAL `/p` (crypto: `8`), node
      `undefined`, exit 0. A hoisted fn decl is its own FunctionPlan
      and introduces its name through no declarator/for-of node, so
      `stale_provenance_shadow_lane` never sees it; closing needs a
      name-collision check at the RECORDING sites or a module-wide
      pre-pass over `functions` — its own task. Identical on parent +
      branch (pre-existing, neither introduced nor widened). ≈
      register R-10, but NOT closed by the T-new-D guard.
  17. P5-R-classmethod-zero **[SILENT MISCOMPILE]** (Task 4 fix
      wave): `class Foo{m(x){return 'A';}} new Foo().m('x')` prints 0.
      Surfaces now that a shadowing `class TextEncoder` correctly
      takes the user lane. Same class as the Stage-5 "class-method
      bodies return 0" finding; the `function` spelling of the same
      shadow fails closed.
  18. P5-R-destructuring-assign **[SILENT MISCOMPILE, HIGH]** (NEW,
      T-new-B wave 3): the PARSER SILENTLY DROPS destructuring
      assignment — `let a=0n; [a]=[1n]; console.log(a)` prints `0`,
      node `1n`; the AST dump shows the statement decaying into two
      unrelated ExpressionStatements, no diagnostic. A defensive taint
      is in place for when the parser stops dropping it; today it
      protects nothing observable and a dropped write cannot inject a
      handle into a proven binding, so `numeric_bindings`' induction
      is not unsound today — but it is only as complete as the AST,
      which MUST be written into the proof's doc comment.

  Scope-model tripwire (not itself a live wrong value on the new keys):
  19. P5-R-blockscope-numeric **[DELIBERATE — latent coupling]**
      (T-new-B r4): kali has NO BLOCK SCOPING — `let s=7n; function
      f(){ { let s=0n; s+=1n; } return s; } f()` → 1, node → 7n (no
      `String()` involved). Not reachable through the new proof keys
      (it misroutes the READ before any proof is consulted), so
      `numeric_bindings` is sound today — BUT it is keyed on a
      function-granular scope model codegen does not implement at
      block granularity, so **any future block-scoping fix MUST
      revisit both sides together.** ≈ register R-10.
- **Final byte-for-byte `webBaselineSmoke` acceptance**: once P5 lands
  AND a String-builtin lane lands (P2-P4 shipped; the observed flip
  point is the compile-time `String` E5506, not `TextEncoder`),
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
- **Non-lowered-capture deferred-callback fail-open — RESOLVED via DEFAULT-DENY
  ALLOWLIST (Task 9, C-1 FINAL)**: the four registration surfaces
  (`queueMicrotask` / `setTimeout` / `setInterval` / `addEventListener`) resolve
  any stable-provenance callback, but the deferred lane restores captures through
  the OWNER's env-record pointer while the owner frame + its arena are already
  gone when the callback fires. The FIRST fix (scalar-only DENYLIST) leaked three
  whole classes — the stage review FALSIFIED its residual rationale with probes:
  captured OBJECTS read `0` (b2 `x=4`→`x=0`; b7 reads the field SYNC=`4` THEN
  deferred=`0`, kali self-contradicting — this DISPROVES the "in-callback read
  equals out-of-callback read" equality claim the old bullet asserted for
  objects), captured-object field MUTATION `0`s (b2b), a scalar LAUNDERED into an
  object field `0`s even though the object earned an `Object` repr and passed the
  old `if lowered` early-out (b5), and a param-ALIAS `let a = i` `0`s (b3=p36b —
  `is_scalar == false`, non-param, so the `function_param_names` consult missed
  it). **Fix (DEFAULT-DENY over an ALLOWLIST at the shared choke point,
  `unlowered_capture_denied` in `intrinsics/host.rs`; shared deny emitter
  `deny_deferred_unlowered_capture` in `emit/call.rs`, variant
  `DenyUnloweredCapture`):** EVERY captured binding is denied E5506 UNLESS it is
  provably safe. Two allowlist entries: (1) a BY-VALUE scalar cell — depth-1
  `is_scalar` i64 stored inline in the env record (the exact
  `cell_is_promotable` engagement predicate), the only class the deferred lane
  restores soundly (b4 `let a = i+1` → correct); (2) a PROVABLE ZERO-PLACEHOLDER
  construct ONLY — a `new X()` that lowers to the drop-and-push-`0` aggregate
  placeholder (`crate::lower::declarator_init_is_placeholder_construct`, excluding
  the real-value constructs `Array`/`Uint8Array`/`EventTarget`), proven per
  depth-1 capture whose owner is the registering function
  (`owner == self.function_name`, so `self.body` holds the declarator; walk stops
  at nested `is_function_like` subtrees so a nested `const c = new Foo()` cannot
  wrong-ALLOW an outer object of the same name). The 5 original param/string/float
  pins are now SUBSUMED by the default; the flip is pinned by 5 NEW E5506 tests
  (`deferred_settimeout_captured_object_read`/`_object_self_contradiction`/
  `_object_mutation`/`_scalar_laundered_into_object`/`_param_alias_capture` in
  `soundness_events.rs`, probes b2/b7/b2b/b5/b3).
  **DOCUMENTED RESIDUAL (deliberately ALLOWED — now ZERO-PLACEHOLDER CONSTRUCTS
  ONLY):** `new AbortController()` and other unsupported `new X()` that are `0` in
  the OWNER's own body too — the deferred read of the same `0` introduces no
  divergence. Growable arrays are INDEPENDENTLY gated (b1: the growable-array
  capture lane already rejects `.push` under nested-function capture, E5506).
  Captured OBJECTS/arrays are NO LONGER a residual — they are denied (the b2/b7
  falsification retired the object equality rationale). This preserves the 4
  `browser_bundle_web_baseline_primitives` build tests: `webBaselineSmoke`'s
  listener `() => { count += 1; controller.abort(); }` captures `count` (by-value
  i64 → allowlist entry 1) AND `controller` (AbortController placeholder →
  allowlist entry 2), so "unsupported constructs must still BUILD (warn, not
  error)" holds. Pinned by
  `deferred_listener_nonscalar_placeholder_capture_still_builds`.
  **RE-SCOPED (Task 9 C-1 final):** the captured-TIMER-ID self-clear form
  (`const t = setInterval(...); ... clearInterval(t)`) now FAILS CLOSED — `t` is
  `is_scalar == false`, non-lowered, so it reads a placeholder `0`; two pins
  (`deferred_set_interval_..._captured_timer_id_fails_closed`,
  `..._ticks_self_clear_captured_timer_id_fails_closed`) that previously passed
  ONLY because the sole timer's id coincidentally equalled the placeholder `0`
  were re-scoped to assert E5506 (forcing a non-zero id → `E4003` "did not
  quiesce" hang confirms the underlying miscompile). The base-capture capability
  stays covered by `..._row_q3_now_runs`. Timer-id closure lowering is deferred
  follow-up work. **Lifting plan:** Stage P3 (`Object` repr for constructs) and
  real closure lowering for is_scalar==false cells promote these into the
  genuinely-lowered/constrained set, closing the residual by construction.
- **Hand-mirrored exclusion list in `declarator_init_is_placeholder_construct`
  is a standing wrong-allow flip risk (Task 9 rider, near the C-1 RESOLVED
  bullet above)**: the `Array`/`Uint8Array`/`EventTarget` exclusion list
  inside `declarator_init_is_placeholder_construct`
  (`crates/kali_codegen/src/lower.rs:2145`) is a HAND-MIRRORED NAME LIST.
  `unlowered_capture_denied`'s allowlist branch 2
  (`crates/kali_codegen/src/intrinsics/host.rs`) admits a captured `new X()`
  binding only because its lowering is drop-and-push-0 TODAY; any future
  REAL-VALUE lowering for a bound constructor not added to the exclusion
  list (obvious candidates: `Set`, `Map` — a captured bound `new Set(...)` is
  admitted RIGHT NOW, sound only because it lowers to 0) silently flips the
  allowlist into a value-losing wrong-allow. Two tripwire pins in
  `soundness_events.rs` turn that flip red instead of silent:
  `deferred_capture_of_bound_set_placeholder_tripwire` (pins kali's current
  same-0-both-sides `sync=0`/`cb=0` output for a captured bound `new
  Set(...)` against node's real `sync=3`/`cb=3` — a DELIBERATE tripwire, not
  a correctness claim; goes red the day `Set` gains a real lowering, at
  which point `Set` must be added to the exclusion list) and
  `deferred_capture_nested_shadow_placeholder_denies` (reviewer probe c3:
  pins the `is_function_like` walk-stop in `binding_is_placeholder_construct`
  that stops a nested function's own placeholder declarator from being
  wrongly attributed to an outer binding of the same name — the safety net
  for the SAME hand-mirrored mechanism).
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

## 9. Stage D close-out (Task 9, D4) — CERTIFIED 2026-07-17

### 9.1 Final gate numbers

- Entry baseline: **731** (frozen, §2). Exit: **694** failing — newly-red vs
  the baseline **EMPTY**, drain (newly-green) **37** (the deferred-surface
  families, §8.3.1 classification). Two independent final enumerations on the
  final tree (`.kali-cache` cleared before each): 694/694, `diff` **empty**
  (zero drift). Main-worktree cross-check (`b48a067d3`): **0 FAILED**.
- Stage totals: 26 commits after the plan commit (`1a3a1ae80..9f001d922`) (D1 runtime 2, D2 codegen+glue 3,
  D3 un-flatten 1, EV lane 8 incl. spec/plan/fix rounds, Task 9 fix waves +
  rider 7, docs 6).

### 9.2 Deliberate pin-flip ledger

- Task 2: `runtime_rejects_negative_*` ×2 retargeted → `*_fires_its_callback`
  (clamp-and-fire node parity; trap now attributed to the FIRED callback).
- Task 4: Stage C rows o/bg1/bg2 (queueMicrotask) → `_now_runs`, node-verified.
- Task 5: rows p/q2/q3/bg3 (timers) → `_now_runs`, node-verified (q2/q3
  fixture sources adapted — the plan's literals did not node-verify; §ledger).
- Task 7: the pre-approved `json_test_supports_object_type_*` re-pin ×2
  (strictly-better per-callback-attributed shape).
- Task 8 (as re-scoped by the user): NO re-pins — the 4
  `browser_bundle_web_baseline_primitives` tests went green legitimately via
  the EV lane; row q (addEventListener) → `_now_runs` in EV Task 4.
- Task 9 C-1 final: `deferred_set_interval_*_row_q2_now_runs` +
  `*_ticks_until_cleared` re-scoped to E5506 reject-don't-miscompile pins
  (coincidence-green proven: captured own timer id == placeholder 0 only
  while the sole timer's id was 0; reviewer-reproduced c4 two-interval probe
  → wrong-timer-cancel E4003 on the pre-flip build). Capability retained via
  row_q3 and the `let id = 0; id = setInterval(...)` promoted-scalar spelling
  (p49 byte-for-byte).

### 9.3 Whole-stage review findings + dispositions (3 passes, most-capable model)

- **C-1 (CRITICAL, guard→resolver seam)**: non-lowered captures registered and
  read placeholder 0 through all 4 surfaces (base E5506'd). Round 1
  scalar-only deny (user-ratified) was FALSIFIED by the verifier (captured
  objects value-losing b2/b7/b2b, param-alias b3, laundered scalar b5) →
  round 2 flipped to the ALLOWLIST form (`unlowered_capture_denied`): deny
  everything non-lowered except by-value promotable scalars and provable
  zero-placeholder constructs. 10 E5506 pins + 1 keep-allowed pin + 2
  tripwire pins. The standing denylist-leaks lesson, re-learned at review
  scale.
- **I-1**: stale `clear*` id poisoned the next interval's re-arm — insert-gate
  fix (Rust + 4 JS mirrors), timer-id bases aligned to 0 everywhere.
- **I-2**: kali_types anonymous-callback gate was shadow-blind (shadowed
  builtin = total silent no-op) — builtin exemption now unshadowed-only.
- **I-4**: `Kali.test(name, obj.m)` ran the WRONG function with a false ok —
  bare-identifier gate; member-expression callbacks fail closed.
- **I-3**: §8.2/§8.6 doc-vs-behavior corrections (new-Event dispatch and
  module/function split-scope claims retracted; silent-drop wording fixed).
- Verifier-adjudicated: interval-test re-scope CORRECT (see 9.2); §8.6
  hand-mirror rider landed (`06cee1d67`) with the Set-lowering tripwire.

### 9.4 Follow-up inventory hand-off

§8.6 is the canonical inventory. Headline items: addEventListener/EventTarget
receiver widening + backstop→total-deny (Stage P3, with AbortController
Object repr lifting the placeholder-capture residual and the zero-param
listener restriction); parity stages P2 structuredClone / P4 URL+USP / P5
TextEncoder → byte-for-byte webBaselineSmoke acceptance; non-lowered
non-scalar closure lowering (timer-id capture class); delay-expression and
alias-provenance precision; escaping first-class closures, depth≥2 env
chains, lexical parent links (Stage C carry-over); F-AB-1 expr-arrow-return;
`ok 1`-with-zero-tests distinguishability — PARTIALLY mitigated: the D3+I-4
hardening closed the wrong-function and flattened-arrow vacuous-ok classes;
a zero-registration run still prints a bare summary (full fix = registration
count in the harness epilogue, future stage).

**Stage D CLOSED. Branch stays unmerged by design (honest-red 694 pending the
broader soundness project).**
