# Stage D Event-Surface Lane (Task 8 fallback): EventTarget / addEventListener / dispatchEvent / CustomEvent — Design

**Status:** approved design, pre-plan.
**Branch:** `soundness-batch1-pra` (Stage D mid-flight: Tasks 1–7 landed, gate at 4 measured newly-red).
**Provenance of this work:** Stage D Task 8 presented the 4-test re-pin batch (`browser_bundle_web_baseline_primitives` family) for user approval. The user REJECTED the re-pin in favor of fixing the underlying lane, then chose (a) **full `webBaselineSmoke` runtime parity** as the end goal, (b) **decomposition** with the event surface first, and (c) the **host-registry + synchronous re-entrant dispatch** architecture. This spec covers sub-project 1 (the event surface) only.

## 1. Goal and non-goals

**Goal:** `new EventTarget()`, `target.addEventListener(lit, cb)`, and `target.dispatchEvent(new CustomEvent(lit))` compile and RUN with node-parity semantics inside a fail-closed envelope. As a consequence the 4 `browser_bundle_web_baseline_primitives` build tests go green legitimately (no re-pin), restoring the Stage D gate (newly-red vs `stageD-pre.txt` = empty) so Task 9 close-out can run.

**Non-goals (later parity stages, in planned order):**
- Stage P2: `structuredClone` (deep clone + identity semantics).
- Stage P3: `AbortController`/`AbortSignal` (`.abort()`, `.signal.aborted`, `instanceof`; `signal.addEventListener` — widens this lane's receiver provenance to signals and adds host-fired events).
- Stage P4: `URL` + `URLSearchParams`.
- Stage P5: `TextEncoder`/`TextDecoder` runtime round-trip (host wiring partially exists from throw-fallout Stage 3 — re-assess then).
- Final acceptance: `webBaselineSmoke(3, 4)` byte-for-byte vs node (`kali run` + browser lane), and flipping the 4 build tests to also execute.

Each later stage gets its own brainstorm → spec → plan. This ordering is user-approved; only the event stage is specced here.

## 2. Envelope and semantics (fail-closed everywhere outside it)

### In-lane (compiles + runs, node parity)

1. **Construction:** `new EventTarget()` where `EventTarget` is the unshadowed builtin global and the call has zero args, appearing as a declarator init with stable provenance (`const t = new EventTarget()`; `let` allowed while unreassigned — reassignment makes the binding unstable → deny at every later lane use). The value is an opaque host handle (i64).
2. **Registration:** `t.addEventListener(<string literal>, <callback>)` where:
   - `t` resolves through the `event_target_locals` provenance map (declarator-recorded, `fn_valued_locals` pattern);
   - the event name is a string literal;
   - the callback resolves by the timer-lane provenance rules (`scheduling_callback`-equivalent): inline function expression / block arrow (post-un-flatten they are the same), declarator-recorded function local, or unshadowed function name. Capturing callbacks are allowed — `current_env` is captured at the registration site and env_safety proves the record (or denies E5506);
   - the callback's plan declares **zero parameters** (an `(event) => …` listener is E5506 — no Event-object repr exists yet);
   - exactly 2 args (an options/`once`/`capture` third arg is E5506).
3. **Dispatch:** `t.dispatchEvent(new CustomEvent(<string literal>))` where the argument is an **inline** `new CustomEvent` with a literal type and no further args. **[Amended post-implementation, user-ratified 2026-07-17:]** an out-of-envelope dispatch ARGUMENT on an in-lane receiver (bound event, `detail`/options, non-CustomEvent) takes the pre-existing out-of-lane silent-drop backstop instead of E5506 — the deployable browser corpus (`package_corpus.rs` web-baseline interop source, asserted build-green across 70+ packages ×3 suites) dispatches bound and non-CustomEvent events on in-lane targets, so the deny was unachievable without breaking it. Listeners can only UNDER-fire (never mis-fire); the residual is inventoried for Stage P3's total-deny conversion. Semantics of the in-lane form:
   - listeners for `(t, type)` are invoked **synchronously, in registration order**, before `dispatchEvent` returns;
   - the listener list is **snapshotted at dispatch entry** (node parity: listeners added during the dispatch do not fire in that dispatch);
   - duplicate registration dedups by `(callback_id, env_ptr)` — the function-identity analog (node dedups by listener identity);
   - the return value is boolean `true` (i64 1) — CustomEvent without `cancelable` can never be canceled; `preventDefault`/`cancelable` are out of envelope;
   - dispatch with zero listeners returns `true` (node parity);
   - recursive dispatch (a listener dispatching again) is permitted and bounded only by the wasm stack — overflow traps loudly (fail-closed, never silent).
4. **Handle-escape discipline (Spec-4a lesson — allowlist at the read choke point):** an EventTarget binding may be read ONLY as the receiver of the two lane methods. Any other read position (`console.log(t)`, `t + 1`, `f(t)`, `t === u`, property read, return value, …) is E5506. The handle must never leak as a number.

### Out-of-lane (behavior unchanged this stage)

`*.addEventListener(...)` on a receiver that does NOT resolve to an EventTarget local (e.g. `signal.addEventListener` — AbortSignal has no lowering yet, or any unknown object) keeps today's exact behavior: capturing callback → E5506 via the Stage C guard (`is_undrained_scheduling_surface` name-text backstop), non-capturing → pre-existing E3100 placeholder warning + no-op. This preserves the package-corpus fixtures untouched. **The non-capturing silent-drop residual is a known pre-existing fail-open, inventoried for Stage P3** (which widens receiver provenance and should convert the backstop to a total deny once signals are in-lane). `removeEventListener` in-lane is E5506 (message names the follow-up); shadowed `EventTarget`/`CustomEvent` identifiers take normal user lanes (recognizers require unshadowed global provenance). **[Post-implementation note:]** one deliberate fail-CLOSED widening of "unchanged": a FREE/captured identifier receiver named `addEventListener`/`dispatchEvent` now denies E5506 even out-of-lane (`member_receiver_is_free_identifier`) — the param/local-receiver case keeps the old warn+build behavior, and the gate confirmed no corpus depends on the widened case.

## 3. Runtime architecture

**Host state (`crates/kali_runtime/src/state.rs`):**
- `next_event_target_id: u32` (handles allocated 1..);
- `event_listeners: BTreeMap<(u32, String), Vec<(i32, i64)>>` — key `(target_handle, event_type)`, value registration-ordered `(callback_id, env_ptr)` pairs (BTreeMap for deterministic iteration; dedup on insert by exact pair).

**New `kali:rt` imports** (declared after `clearInterval`, in this order):
- `event_target_new() -> i64` — allocates and returns a fresh handle;
- `event_listener_add(target: i64, name_ptr: i32, name_len: i32, callback_id: i32, env_ptr: i64)` — reads the event name from guest memory (existing host-string ptr/len mechanism), dedup-inserts;
- `event_dispatch(target: i64, name_ptr: i32, name_len: i32) -> i32` — snapshots the listener vec, then for each entry synchronously re-enters the guest: save `__current_env`, set to the stored `env_ptr`, call `__kali_callback_<id>`, restore on both ok/err paths (the `invoke_callback` contract). Returns 1. Listener traps propagate loudly (fail-closed).

**Re-entrancy:** the Rust import is implemented with a `wasmtime::Caller`-based invoke variant (the existing `invoke_callback` takes `(instance, store)`; a `Caller` can fetch the memory, the `__current_env` global, and the callback export mid-`_start`). The JS glue calls `instance.exports.__kali_callback_${id}()` directly (wasm exports are synchronous; mid-`_start` re-entry is legal).

**Budget:** dispatch-invoked listeners do NOT count against `EVENT_LOOP_INVOCATION_BUDGET` — they are synchronous calls (same cost model as a direct function call), not drained callbacks. Timers/microtasks a listener schedules drain normally afterward and are budgeted there.

**Browser glue:** all 4 hand-mirrored JS import lists (`browser/harness.rs` ×2, `cmd_build.rs` ×2) gain the three entries plus a small shared registry prelude (`kaliEventTargets` counter + `kaliEventListeners` Map keyed `"<handle> <type>"`, snapshot-per-dispatch, same dedup), mirroring the Rust semantics exactly (the Task 6 discipline).

## 4. Codegen and analysis

Mechanics mirror Stage D Tasks 4–5 exactly:

- **lower.rs:** three program-wide probes (`new EventTarget` construction; member calls named `addEventListener`/`dispatchEvent` — probes stay SUPERSETS of the emit recognizers); conditional import indices extending the existing chains after `clear_interval`; new fixed wasm type indices for the two new signatures (`() -> i64` may reuse an existing type if one matches — verify against the live type section; `(i64,i32,i32,i32,i64) -> ()` and `(i64,i32,i32) -> i32` are new), with the repr-directed type base bumped accordingly and comments updated; indices threaded through the emitter ctor.
- **emitter.rs:** three `Option<u32>` import-index fields; an `event_target_locals` provenance map (declarator-emit recording, `fn_valued_locals` pattern; reassignment routes through `unstable_provenance_names` → deny).
- **intrinsics/host.rs:** `is_event_target_new(node)` (unshadowed global `EventTarget`, zero args); `event_target_receiver(&LirNode) -> Option<…>` resolving a member callee's receiver through the provenance map; `event_dispatch_argument(node)` validating the inline `new CustomEvent(lit)` shape. Every helper's unresolvable tail = deny.
- **emit/call.rs:** emit arms placed with the other scheduling arms — `addEventListener` (in-lane: name literal → data segment ptr/len; callback via the Task 4 resolver; zero-param plan check; emit registration; produce nothing) and `dispatchEvent` (in-lane: emit dispatch; produce i64 bool). Out-of-lane member calls with these names fall through to the existing backstop unchanged. Every deny is E5506 with a message naming the unsupported feature and the follow-up stage.
- **Declarator lane (lower.rs):** `const t = new EventTarget()` allocates a real local holding the handle (the Task 5 side-effecting-init promotion precedent applies — never fold-alias a constructor with host side effects).
- **Handle-escape choke point:** EventTarget-provenance bindings are denied at the identifier-read allowlist (the Spec-4a mechanism): allowed positions are exactly lane-method receiver and declarator init; everything else E5506.
- **env_safety.rs:** a member-aware registration predicate (`is_event_registration_callee`: callee text `addEventListener`, callee HAS children) with edge target `children[2]` (member call layout `[callee, name, cb]` — verify against LIR at plan time). Over-approximation on shadowed names is safe (reject-only analysis).
- **kali_types:** `EventTarget`/`CustomEvent` are already builtin globals (no resolve-side change expected); the anonymous-argument gate ignores member callees (verified in exploration) — no change.

## 5. Testing and gating

- **Runtime WAT tests** (`kali_runtime/src/execute_tests/`, new `events.rs`): registration order, snapshot-during-dispatch, dedup by pair, sync re-entry mid-`_start`, env save/restore around listener invocation, zero-listener dispatch returns 1, listener trap propagates.
- **e2e node-verified pins** (new `crates/kali_cli/tests/soundness_events.rs`, `run_kali` helper pattern): capturing listener mutates an outer cell and the mutation is visible immediately after `dispatchEvent` (sync proof: log lines straddling the dispatch); two listeners fire in registration order; a listener added during dispatch does not fire that round but fires the next; same function registered twice fires once; zero-listener dispatch returns true; recursive dispatch (depth 2) works. EVERY expected output verified via `node` first.
- **Envelope fail-closed pins:** non-literal event name; listener with a parameter; options third arg; `CustomEvent` with detail; bound (non-inline) event argument; `removeEventListener`; reassigned target binding; handle escape (`console.log(t)`); unresolvable callback (alias/call-result — the x/y/z analog); out-of-lane receiver preserved-behavior pin (non-capturing `signal.addEventListener`-shaped fixture still warns + builds — guards the corpus).
- **Pin flips:** Stage C row q (`deferred_add_event_listener_capturing_callback_fails_closed`) → `_now_runs`, node-verified; add the missing module-scope boundary pin (exploration found none exists).
- **Browser lane:** one `assert_browser_bundle_executes`-style test running an event fixture through the bundle glue (the digestSmoke precedent).
- **The 4 build tests:** go green untouched — that is the acceptance criterion for this sub-project.
- **Full gate:** newly-red vs `stageD-pre.txt` (731) = EMPTY; drain stays at Task 7's 37 (record). Census: imports only, no new synthetics expected.
- **Close-out:** commits tagged `[stageD]` on this branch; Stage D Task 9's whole-stage adversarial review then covers Tasks 2–7 + this lane in one scope, followed by the stage close-out.

## 6. Risks / uncertainty made explicit

- **Sync re-entrancy in wasmtime** via `Caller` mid-`_start` is believed supported (host imports may call back into the instance); if a borrow/store conflict surfaces, the fallback is dispatch-via-`pending_microtasks`-drained-inline-at-the-dispatch-site (a synchronous mini-drain), which preserves observable ordering — flag at plan time if needed.
- **Member-call LIR layout** (`[callee, name, cb]`, callee children = receiver) is asserted from exploration (`resolve_bound_member_callable_node`, `is_kali_test_call` precedents) — the plan's first codegen task must verify against a real LIR dump before wiring `children[2]` edges.
- **Type-section reuse** for `() -> i64` must be checked against the live file (do not assume a new index is needed).
- **The out-of-lane silent-drop residual** (non-capturing callbacks to unknown receivers warn + no-op) is pre-existing and deliberately unchanged; it is the top inventory item for Stage P3.
