# Stage D — Block-Arrow Un-flatten + Deferred-Callback Lane: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Block-bodied arrows parse as real functions (closing the eager-inline-execution silent miscompile), and `queueMicrotask`/`setTimeout`/`setInterval` (+`clearTimeout`/`clearInterval`) actually run their callbacks capture-aware under a virtual-clock, budget-bounded event loop — everything else fails closed E5506.

**Architecture:** Capability-first, parser flip last (spec §2). The runtime already has a real-time event loop with the full env_ptr ABI (`kali:rt` `queueMicrotask (i32,i64)`, `setTimeout/setInterval (i32,i32,i64)->i32`, `clearTimeout/clearInterval (i32)`, host state in `kali_runtime/src/state.rs`, drain in `kali_runtime/src/host/enforce.rs`); codegen emits calls to NONE of them. Tasks 2–3 convert the loop to virtual time + budget; Tasks 4–6 wire codegen registrations (fn-expr callbacks, exercised end-to-end pre-parser-change); Tasks 7–8 land the parser un-flatten + re-pins; Task 9 is the whole-stage review + final gate.

**Tech Stack:** Rust workspace (`cargo`), wasm via `wasm-encoder`/`wasmtime`, node v26.5.0 as the behavioral oracle.

**Spec:** `docs/superpowers/specs/2026-07-16-stageD-unflatten-design.md`. Stage C context: `docs/superpowers/followups/stageC-closures-triage.md` (§6 pin rows, §13 guard inversion). Seed patch: `docs/superpowers/followups/task5-block-arrows-WIP.patch` — parser + kali_types portions land in Task 7; its `queue_microtask` codegen wiring is **superseded** (pre-dates the env_ptr ABI) and must NOT be applied.

## Global Constraints

- Branch: `soundness-batch1-pra`. Never merge to main; the branch carries a 731-test honest-red baseline by design.
- **Gate command** (referenced as "the full gate" in every task):
  ```bash
  SCRATCH=/tmp/claude-1000/-workspace/3f4d161d-65f3-40b7-9770-135e7244af0f/scratchpad
  cd /workspace && rm -rf .kali-cache && cargo build -p kali_cli 2>/dev/null
  (cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > "$SCRATCH/stageD-post-<task>.txt"; touch "$SCRATCH/stageD-<task>.done") &
  # poll for the .done marker (NEVER pgrep -f); then:
  comm -13 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-<task>.txt"   # newly-red — must be EMPTY (exceptions: Task 7, see task text)
  comm -23 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-<task>.txt"   # drain — record, explain
  ```
  Always `sort -u` (18 dual-harness duplicate names; plain `sort` fabricates entries). Always `--no-fail-fast`. Always `rm -rf .kali-cache` first (stale ABI-mismatched cache once produced 34 false newly-red). Full run takes ~10–20 min; run detached with a `.done` marker file and poll.
- `$SCRATCH/stageD-pre.txt` is the canonical 731-entry entry baseline created in Task 1. Every gate diffs against it.
- Node oracle: every end-to-end fixture's expected output MUST be verified by running the same source through `node` before writing the assertion (`node /tmp/fixture.js`).
- Any new `kali:rt` import emitted by codegen MUST be added to all 4 hand-mirrored JS import lists (`crates/kali_runtime/src/browser/harness.rs` ×2 sites, `crates/kali_cli/src/bin/cmd_build.rs` ×2 sites) or browser tests LinkError (Task 6).
- Kali is GC-less by design: no tracing/copying collection anywhere; env records live in the never-reset `__alloc_global` region (unchanged here).
- Commit after every task with a `[stageD]` suffix, matching the existing convention (`git log --oneline` for examples).
- `crates/kali_cli/tests/` test census: adding SYNTHETIC guest functions requires syncing the `SYNTHETIC_FUNCTIONS` allowlist in `count_tag_boxing_ops` (runtime_smoke.rs). **This stage adds imports only, no new synthetics — no census change is expected; if a census test goes red, that is a desync to investigate, not a product regression.**

---

### Task 1: D0 — entry baseline + scratch blast-radius measurement (no product code)

**Files:**
- Create: `docs/superpowers/followups/stageD-triage.md`
- No source changes. Scratch tree work is applied and fully reverted.

**Interfaces:**
- Produces: `$SCRATCH/stageD-pre.txt` (canonical 731 baseline used by every later gate); `$SCRATCH/stageD-parser-newly-red.txt` (candidate re-pin list consumed by Tasks 7–8); the triage doc.

- [ ] **Step 1: Fresh entry baseline, two independent enumerations**

```bash
SCRATCH=/tmp/claude-1000/-workspace/3f4d161d-65f3-40b7-9770-135e7244af0f/scratchpad
cd /workspace && cargo build -p kali_cli
for i in 1 2; do
  rm -rf .kali-cache
  (cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > "$SCRATCH/stageD-pre-run$i.txt"; touch "$SCRATCH/stageD-pre-run$i.done") &
  wait
done
diff "$SCRATCH/stageD-pre-run1.txt" "$SCRATCH/stageD-pre-run2.txt"   # MUST be empty
wc -l "$SCRATCH/stageD-pre-run1.txt"                                  # expected: 731
sort -u "$SCRATCH/stageD-pre-run1.txt" "$SCRATCH/stageD-pre-run2.txt" > "$SCRATCH/stageD-pre.txt"
wc -l "$SCRATCH/stageD-pre.txt"                                       # expected: 731
```

Expected: 731, zero drift. **If ≠731 or drift: STOP — reconcile before anything lands** (spec D0). Record the numbers in the triage doc.

- [ ] **Step 2: Parser-only scratch measurement**

Apply ONLY the parser + kali_types portions of the WIP patch (the codegen wiring half is stale — pre-env_ptr-ABI — and must be excluded):

```bash
cd /workspace
git apply --include='crates/kali_parser/*' --include='crates/kali_types/*' \
  docs/superpowers/followups/task5-block-arrows-WIP.patch
git status   # exactly 2 modified files: kali_parser/src/expression/primary.rs, kali_types/src/resolve/call.rs
cargo build -p kali_cli
rm -rf .kali-cache
(cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stageD-parser-post.txt"; touch "$SCRATCH/stageD-parser.done") &
# wait for .done, then:
comm -13 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-parser-post.txt" > "$SCRATCH/stageD-parser-newly-red.txt"
wc -l "$SCRATCH/stageD-parser-newly-red.txt"
```

Note: the applied kali_types gate still treats `setTimeout`/`setInterval` as "deferred_but_unwired" (rejects anonymous callbacks to them) — Tasks 4–5 wire them, and Task 7 lands the gate WITHOUT that carve-out, so this measurement slightly OVER-counts the final red set. That direction is safe for planning.

- [ ] **Step 3: Revert and verify clean**

```bash
git checkout -- crates/kali_parser/src/expression/primary.rs crates/kali_types/src/resolve/call.rs
git status   # clean
git apply --check docs/superpowers/followups/task5-block-arrows-WIP.patch && echo STILL-APPLIES
```

- [ ] **Step 4: Classify the newly-red set**

For each name in `stageD-parser-newly-red.txt`, bucket by family (grep the test name back to its fixture, extract the diagnostic class by running the fixture through the freshly built patched binary if needed — rebuild in a scratch worktree if that's simpler than re-applying). Expected buckets, per spec D0:
  (a) closed by Tasks 4–6 (deferred-surface families: fixtures whose failure is a capturing/anonymous callback to `queueMicrotask`/`setTimeout`/`setInterval`);
  (b) re-pin candidates (main-green-via-miscompile: `addEventListener` browser families + the two `json_test_supports_object_type_*` diagnostic-shape pins);
  (c) anything else → **investigate before proceeding; do not start Task 2 with an unexplained bucket-c entry.**

- [ ] **Step 5: Write the triage doc and commit**

`docs/superpowers/followups/stageD-triage.md`: baseline numbers, measurement method, the full newly-red list with per-family classification, the bucket-c disposition. Then:

```bash
git add docs/superpowers/followups/stageD-triage.md
git commit -m "docs(soundness): stageD triage — entry 731 baseline, parser-only blast radius measured [stageD]"
```

---

### Task 2: D1 — virtual-clock timer queue (state.rs + enforce.rs)

**Files:**
- Modify: `crates/kali_runtime/src/state.rs` (struct `ScheduledTimer` ~line 72, `KaliHostState` fields ~line 34, `schedule_timer` ~line 212)
- Modify: `crates/kali_runtime/src/host/enforce.rs` (`drain_event_loop` ~line 23)
- Test: `crates/kali_runtime/src/execute_tests/timers.rs`

**Interfaces:**
- Consumes: existing `KaliHostState.pending_timers: BTreeMap<u32, ScheduledTimer>`, `pending_microtasks`, `cancelled_timers`, `invoke_callback`.
- Produces: `ScheduledTimer { callback_id: i32, env_ptr: i64, due_at_ms: u64, seq: u64, repeat_interval_ms: Option<u64> }`; new `KaliHostState` fields `virtual_clock_ms: u64`, `next_timer_seq: u64` (both default 0). Task 3 and Task 6 (JS mirror) rely on these exact semantics.

- [ ] **Step 1: Write the failing ordering + virtual-time tests**

Add to `crates/kali_runtime/src/execute_tests/timers.rs` (follow the existing WAT-fixture style in that file):

```rust
#[test]
fn runtime_timers_fire_in_delay_order_not_registration_order() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Registers delay-10 BEFORE delay-5; the delay-5 callback must fire first.
    // State machine: cb_5 moves state 0->1; cb_10 requires state==1 else traps.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1") ;; the delay-10 callback
                    global.get $state
                    i32.const 1
                    i32.eq
                    if
                        i32.const 2
                        global.set $state
                    else
                        unreachable
                    end)
                (func (export "__kali_callback_2") ;; the delay-5 callback
                    i32.const 1
                    global.set $state)
                (func (export "_start")
                    i32.const 1
                    i32.const 10
                    i64.const 0
                    call $set_timeout
                    drop
                    i32.const 2
                    i32.const 5
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_large_delays_complete_instantly_under_the_virtual_clock() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (func (export "__kali_callback_1"))
                (func (export "_start")
                    i32.const 1
                    i32.const 60000
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let started = std::time::Instant::now();
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    // Real-time drain would sleep 60s; the virtual clock must not sleep at all.
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "drain slept on a virtual timer: {:?}",
        started.elapsed()
    );
}

#[test]
fn runtime_equal_due_times_fire_in_registration_order() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Two delay-0 timers: first-registered must fire first (seq tiebreak).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    global.get $state
                    i32.const 0
                    i32.eq
                    if
                        i32.const 1
                        global.set $state
                    else
                        unreachable
                    end)
                (func (export "__kali_callback_2")
                    global.get $state
                    i32.const 1
                    i32.eq
                    if
                        i32.const 2
                        global.set $state
                    else
                        unreachable
                    end)
                (func (export "_start")
                    i32.const 1
                    i32.const 0
                    i64.const 0
                    call $set_timeout
                    drop
                    i32.const 2
                    i32.const 0
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_negative_delays_clamp_and_fire_instead_of_trapping() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Node parity: setTimeout(fn, -1) clamps to the 1ms minimum and FIRES.
    // (Flips the old reject-negative-delay semantics — a deliberate Stage D
    // decision; the two old `runtime_rejects_negative_*` tests are retargeted
    // in Step 3.)
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (global $state (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    i32.const 1
                    global.set $state)
                (func (export "_start")
                    i32.const 1
                    i32.const -1
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

```bash
cargo test -p kali_runtime timers -- --test-threads=2 2>&1 | tail -20
```

Expected: `runtime_timers_fire_in_delay_order_not_registration_order` FAILS (real-time drain sleeps 5ms/10ms and fires in due order — actually this one may PASS on real time; the load-bearing failures are `runtime_large_delays_complete_instantly_under_the_virtual_clock` — hangs 60s then passes, or trips the 5s assert — and `runtime_negative_delays_clamp_and_fire_instead_of_trapping` — FAILS with exit 1, the schedule-time reject). At least those two must be red before the implementation lands.

- [ ] **Step 3: Implement the virtual clock**

In `crates/kali_runtime/src/state.rs`:

Replace the `ScheduledTimer` struct:

```rust
/// A scheduled timer callback.
#[derive(Clone, Debug)]
pub struct ScheduledTimer {
    /// Guest callback id.
    pub callback_id: i32,
    /// `current_env` captured at scheduling time (Stage C C3); restored into the
    /// `current_env` global by `invoke_callback` before the callback runs.
    pub env_ptr: i64,
    /// Virtual due time in milliseconds (`KaliHostState::virtual_clock_ms`
    /// coordinates; Stage D). The drain advances the clock directly to the
    /// next due timer — no real sleeping, ordering is the observable.
    pub due_at_ms: u64,
    /// Monotonic scheduling sequence. Ties on `due_at_ms` fire in seq order,
    /// and a re-armed interval takes a FRESH seq so already-due timers run
    /// before the re-armed tick (node heap-insertion parity).
    pub seq: u64,
    /// Repeat interval in virtual ms for setInterval-like timers.
    pub repeat_interval_ms: Option<u64>,
}
```

Add two fields to `KaliHostState` (next to `next_timer_id`) and their `Default` entries (`0` each):

```rust
    /// Virtual event-loop clock in ms (Stage D): starts at 0, advanced by the
    /// drain to each fired timer's due time. Never consults wall time.
    pub virtual_clock_ms: u64,
    /// Monotonic per-(re)arm sequence for timer tie-breaking.
    pub next_timer_seq: u64,
```

Replace the body of `schedule_timer` (keep the policy check and the id allocation exactly as they are):

```rust
    pub(crate) fn schedule_timer(
        &mut self,
        callback_id: i32,
        delay_ms: i32,
        repeat: bool,
        env_ptr: i64,
    ) -> wasmtime::Result<i32> {
        // Node parity (Stage D): delays below 1ms — including negative —
        // clamp to node's documented 1ms minimum. The clamp also prevents a
        // zero-delay interval re-arm from starving strictly-later timers
        // under the virtual clock.
        let effective_delay_ms: u64 = if delay_ms < 1 { 1 } else { delay_ms as u64 };

        let active_timers = self.pending_timers.len();
        if let Some(policy) = self.policy.as_ref() {
            policy
                .check_operation(HostOperation::TimerSchedule {
                    delay_ms: effective_delay_ms,
                    active_timers,
                })
                .map_err(|diagnostic| {
                    self.pending_diagnostic = Some(diagnostic.clone());
                    wasmtime::Error::msg(format!("KALI_E4003: {}", diagnostic.message))
                })?;
        }

        let timer_id = self.next_timer_id;
        self.next_timer_id = self
            .next_timer_id
            .checked_add(1)
            .ok_or_else(|| wasmtime::Error::msg("timer id overflow"))?;
        let seq = self.next_timer_seq;
        self.next_timer_seq += 1;

        self.pending_timers.insert(
            timer_id,
            ScheduledTimer {
                callback_id,
                env_ptr,
                due_at_ms: self.virtual_clock_ms + effective_delay_ms,
                seq,
                repeat_interval_ms: repeat.then_some(effective_delay_ms),
            },
        );

        Ok(timer_id as i32)
    }
```

In `crates/kali_runtime/src/host/enforce.rs`, replace the timer half of `drain_event_loop`'s loop body (from `let next_timer = …` through the re-arm block) with:

```rust
        let next_timer = {
            let state = store.data();
            state
                .pending_timers
                .iter()
                .min_by_key(|(_, timer)| (timer.due_at_ms, timer.seq))
                .map(|(timer_id, timer)| (*timer_id, timer.clone()))
        };

        let Some((timer_id, timer)) = next_timer else {
            break;
        };

        {
            let state = store.data_mut();
            // Advance the virtual clock directly to the due time — no sleeping.
            state.virtual_clock_ms = state.virtual_clock_ms.max(timer.due_at_ms);
            state.pending_timers.remove(&timer_id);
        }

        invoke_callback(instance, store, timer.callback_id, timer.env_ptr)?;

        if let Some(interval_ms) = timer.repeat_interval_ms {
            let cancelled = {
                let state = store.data_mut();
                state.cancelled_timers.remove(&timer_id)
            };

            if !cancelled {
                let state = store.data_mut();
                let seq = state.next_timer_seq;
                state.next_timer_seq += 1;
                let due_at_ms = state.virtual_clock_ms + interval_ms;
                state.pending_timers.insert(
                    timer_id,
                    ScheduledTimer {
                        callback_id: timer.callback_id,
                        env_ptr: timer.env_ptr,
                        due_at_ms,
                        seq,
                        repeat_interval_ms: Some(interval_ms),
                    },
                );
            }
        }
```

Delete the `let now = Instant::now(); if timer.due_at > now { thread::sleep(...) }` block entirely. Remove now-unused `Instant`/`Duration`/`thread` imports from both files if the compiler flags them (`cargo build -p kali_runtime` and fix `unused_imports` warnings — check `Instant`/`Duration` are not used elsewhere in `state.rs` first; `Instant::now()` had two uses in the deleted code paths only).

- [ ] **Step 4: Retarget the two negative-delay tests**

The existing `runtime_rejects_negative_timer_delays` and `runtime_rejects_negative_interval_delays` (in `timers.rs`) pinned the trap-on-negative semantics. Under clamp-and-fire, their `unreachable` callback bodies now RUN and trap — same exit 1, but through a different mechanism, which makes the pins misleading. Retarget them: rename to `runtime_negative_timeout_delay_fires_its_callback` / `runtime_negative_interval_delay_fires_its_callback`, keep the WAT unchanged, and update each doc comment to say the trap now comes from the CALLBACK body (proof the clamped timer fired), citing the Stage D node-parity clamp decision. For the interval variant, the `unreachable` callback traps on its first tick, so the drain terminates — no budget interaction. Assert additionally that the diagnostic message contains `"__kali_callback_"` (callback-attributed trap, not a schedule-time reject):

```rust
    assert!(
        diagnostic.message.contains("__kali_callback_"),
        "expected the trap to come from the FIRED callback, got: {}",
        diagnostic.message
    );
```

- [ ] **Step 5: Run the runtime test suite**

```bash
cargo test -p kali_runtime 2>&1 | tail -5
```

Expected: all pass, including the four new tests and the two retargeted ones.

- [ ] **Step 6: Full gate**

Run the full gate per Global Constraints (`stageD-post-task2.txt`). Newly-red MUST be empty (codegen emits no timer calls yet, so guest-visible behavior is unchanged; only the hand-written WAT runtime tests exercise this code).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_runtime
git commit -m "feat(runtime): virtual-clock timer queue — (due,seq) ordering, 1ms clamp, no sleeping (D1) [stageD]"
```

---

### Task 3: D1 — bounded drain (event-loop budget trap)

**Files:**
- Modify: `crates/kali_runtime/src/host/enforce.rs` (`drain_event_loop`)
- Test: `crates/kali_runtime/src/execute_tests/timers.rs`

**Interfaces:**
- Consumes: Task 2's virtual-clock drain.
- Produces: `pub(crate) const EVENT_LOOP_INVOCATION_BUDGET: u64 = 100_000;` in `enforce.rs`; on exceed, `drain_event_loop` returns `Err(Diagnostic)` with code `e4::RESOURCE_LIMIT_EXCEEDED` and a message containing `"event loop did not quiesce"`. Task 5's end-to-end budget fixture and Task 6's JS mirror rely on the constant and the message text.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn runtime_uncleared_interval_trips_the_drain_budget() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // An interval that never clears would drain forever (node parity would
    // hang); the bounded drain must trap loudly instead (Stage D decision).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32 i64) (result i32)))
                (memory (export "memory") 1)
                (func (export "__kali_callback_1"))
                (func (export "_start")
                    i32.const 1
                    i32.const 0
                    i64.const 0
                    call $set_interval
                    drop)
            )
            "#,
    );
    let started = std::time::Instant::now();
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    let diagnostic = outcome.trap.expect("budget exhaustion must surface a diagnostic");
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
    assert!(
        diagnostic.message.contains("event loop did not quiesce"),
        "got: {}",
        diagnostic.message
    );
    // 100k no-op invocations under a virtual clock must be fast.
    assert!(started.elapsed() < std::time::Duration::from_secs(60));
}

#[test]
fn runtime_self_requeueing_microtask_trips_the_drain_budget() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "queueMicrotask" (func $queue_microtask (param i32 i64)))
                (memory (export "memory") 1)
                (func (export "__kali_callback_1")
                    i32.const 1
                    i64.const 0
                    call $queue_microtask)
                (func (export "_start")
                    i32.const 1
                    i64.const 0
                    call $queue_microtask)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    let diagnostic = outcome.trap.expect("budget exhaustion must surface a diagnostic");
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::RESOURCE_LIMIT_EXCEEDED as u32)
    );
}

#[test]
fn runtime_zero_delay_interval_does_not_starve_later_timers() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // interval(0) clamps to 1ms; a timeout at 5ms must still get scheduled
    // (the timeout's callback clears the interval, so the drain terminates).
    // If the clamp regressed to 0, the interval would re-arm at the same
    // virtual instant forever and the budget trap (exit 1) would fire instead.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "setInterval" (func $set_interval (param i32 i32 i64) (result i32)))
                (import "kali:rt" "setTimeout" (func $set_timeout (param i32 i32 i64) (result i32)))
                (import "kali:rt" "clearInterval" (func $clear_interval (param i32)))
                (memory (export "memory") 1)
                (global $interval_id (mut i32) (i32.const -1))
                (func (export "__kali_callback_1")) ;; interval tick: no-op
                (func (export "__kali_callback_2") ;; timeout: clears the interval
                    global.get $interval_id
                    call $clear_interval)
                (func (export "_start")
                    i32.const 1
                    i32.const 0
                    i64.const 0
                    call $set_interval
                    global.set $interval_id
                    i32.const 2
                    i32.const 5
                    i64.const 0
                    call $set_timeout
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}
```

- [ ] **Step 2: Run to verify the budget tests fail**

```bash
timeout 120 cargo test -p kali_runtime runtime_uncleared_interval -- --nocapture 2>&1 | tail -5
```

Expected: without the budget, the drain loops forever → the `timeout 120` kills it (or the test hangs). That IS the red state. The starvation test should PASS already (clamp landed in Task 2) — it is a regression pin.

- [ ] **Step 3: Implement the budget**

In `enforce.rs`, above `drain_event_loop`:

```rust
/// Total scheduled-callback invocations (microtasks + timers) one drain may
/// perform before failing loudly. Terminating programs sit far below this;
/// only programs that would hang node (an uncleared `setInterval`, a
/// self-requeueing microtask) ever reach it — Stage D's bounded-drain
/// decision: trap distinctly instead of hanging the process and the gate.
pub(crate) const EVENT_LOOP_INVOCATION_BUDGET: u64 = 100_000;
```

At the top of `drain_event_loop`'s body add `let mut invocations: u64 = 0;`, and at the top of the `loop` add:

```rust
        if invocations >= EVENT_LOOP_INVOCATION_BUDGET {
            return Err(Diagnostic::error(
                e4::RESOURCE_LIMIT_EXCEEDED as u32,
                format!(
                    "event loop did not quiesce: {EVENT_LOOP_INVOCATION_BUDGET} scheduled-callback \
                     invocations were drained and callbacks are still pending (an uncleared \
                     setInterval or a self-requeueing microtask?)"
                ),
            ));
        }
```

Increment `invocations += 1;` immediately before EACH of the two `invoke_callback(...)` calls (microtask and timer).

- [ ] **Step 4: Run the runtime suite**

```bash
cargo test -p kali_runtime 2>&1 | tail -5
```

Expected: all pass. If the budget-trap assertions on `diagnostic.code` fail because `execute.rs` re-wraps drain errors, inspect the actual outcome shape printed by the failing assert and adjust ONLY the assertion to the observed wrapping — the load-bearing requirements are exit 1 + the "did not quiesce" text + a code distinct from a plain callback trap.

- [ ] **Step 5: Full gate** (`stageD-post-task3.txt`) — newly-red MUST be empty.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_runtime
git commit -m "feat(runtime): bounded event-loop drain — 100k invocation budget, distinct non-quiescence trap (D1) [stageD]"
```

---

### Task 4: D2 — codegen `queueMicrotask` registration lane + all import plumbing + env_safety edges

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (flags, import indices, type 14, import section entries, emitter ctor args, probe fn)
- Modify: `crates/kali_codegen/src/emitter.rs` (5 new `Option<u32>` fields + ctor params)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (`SchedulingSurface`, `scheduling_surface()`, `SchedulingCallback`, `scheduling_callback()`)
- Modify: `crates/kali_codegen/src/emit/call.rs` (emit arm after the `Kali.test` arm; remove `queueMicrotask` from `is_undrained_scheduling_surface`)
- Modify: `crates/kali_codegen/src/env_safety.rs` (registration edges for all three surfaces)
- Test: `crates/kali_cli/tests/soundness_closures.rs`

**Interfaces:**
- Consumes: Task 2/3 runtime semantics; existing emitter namespaces `self.functions: HashMap<String, u32>` (fn name → wasm index), `self.fn_valued_locals`, `self.unstable_provenance_names`, `self.locals`, `self.bindings`, `self.module_binding_names`, `self.env_plans`, `self.current_env_global()`; `TEST_REGISTER_TYPE_INDEX = 13` (`(i32,i64) -> ()`).
- Produces (Task 5 relies on these exact names): emitter fields `queue_microtask_import_index`, `set_timeout_import_index`, `set_interval_import_index`, `clear_timeout_import_index`, `clear_interval_import_index` (all `Option<u32>`); `const SCHEDULING_TIMER_SET_TYPE_INDEX: u32 = 14` (`(i32,i32,i64) -> i32`); `pub(crate) enum SchedulingSurface { QueueMicrotask, SetTimeout, SetInterval, ClearTimeout, ClearInterval }`; `fn scheduling_surface(&self, callee_node: &LirNode) -> Option<SchedulingSurface>`; `pub(crate) enum SchedulingCallback { Resolved(u32), LegacyPlaceholder, Deny }`; `fn scheduling_callback(&self, node: &LirNode) -> SchedulingCallback`; `fn emit_scheduling_call(&mut self, function: &mut Function, node: &LirNode, surface: SchedulingSurface) -> EmittedValue`.

- [ ] **Step 1: Write the failing end-to-end tests**

Add to `crates/kali_cli/tests/soundness_closures.rs` (it already has a `run_kali`-style helper — reuse the file's existing helper; if it differs in name, adapt these tests to it):

```rust
/// Stage D: a NON-capturing function-expression microtask callback must be
/// deferred and actually RUN during the post-_start drain.
/// node v26.5.0: "sync\nmt\n".
#[test]
fn deferred_queue_microtask_fn_expr_runs_after_sync_code() {
    let out = run_kali(
        r#"queueMicrotask(function () {
  console.log("mt");
});
console.log("sync");
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\nmt\n");
}

/// Stage D: a CAPTURING microtask callback runs with its owner's env record
/// (the C3 env_ptr restore), reading and writing the captured cell correctly
/// AFTER the owner has returned (never-reset-region property).
/// node v26.5.0: "sync=5\nmt=6\n".
#[test]
fn deferred_queue_microtask_capturing_callback_runs_with_env() {
    let out = run_kali(
        r#"function owner() {
  let base = 5;
  queueMicrotask(function () {
    base += 1;
    console.log("mt=" + base);
  });
  console.log("sync=" + base);
}
owner();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nmt=6\n");
}
```

Verify both expected outputs against node FIRST (`node /tmp/f.js` with the same source).

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p kali_cli --test soundness_closures deferred_queue_microtask_fn_expr -- --nocapture 2>&1 | tail -10
cargo test -p kali_cli --test soundness_closures deferred_queue_microtask_capturing_callback_runs -- --nocapture 2>&1 | tail -10
```

Expected: FAIL — today the non-capturing one silently drops the callback (prints only `sync`) and the capturing one is E5506 (Stage C guard).

- [ ] **Step 3: Add the type, flags, import indices, and import-section entries in `lower.rs`**

First locate the anchors:

```bash
grep -n "TEST_REGISTER_TYPE_INDEX\|repr-directed" crates/kali_codegen/src/lower.rs | head
grep -n "crypto_subtle_digest_import_index = if" crates/kali_codegen/src/lower.rs
```

(a) **Type 14.** Immediately after the `TEST_REGISTER_TYPE_INDEX` type (index 13, `(i32,i64) -> ()`), add:

```rust
    // Type 14: setTimeout / setInterval
    // `(callback_index: i32, delay_ms: i32, env_ptr: i64) -> i32` (Stage D) —
    // registers a timer with the env active at the scheduling site and
    // returns the i32 timer id. Registered unconditionally so the type index
    // is stable; the imports are conditional. This is now the last fixed
    // type, so the repr-directed function types start at index 15.
    const SCHEDULING_TIMER_SET_TYPE_INDEX: u32 = 14;
    type_section
        .ty()
        .function(vec![ValType::I32, ValType::I32, ValType::I64], vec![ValType::I32]);
```

Then find the repr-directed type dedup base (the code the type-13 comment "repr-directed function types start at index 14" refers to — grep `14` in the ~100 lines after the type section) and bump it to 15, updating both the constant/expression and the type-13 comment sentence ("last fixed type" moves to type 14's comment).

(b) **Flags.** Next to `let uses_crypto_subtle_digest = …` add:

```rust
    // Stage D: scheduling-surface conditional imports, appended LAST (after
    // crypto_subtle_digest) in declaration order queueMicrotask, setTimeout,
    // setInterval, clearTimeout, clearInterval — so no earlier import or
    // function index shifts.
    let uses_queue_microtask = program_calls_bare_identifier(lir, "queueMicrotask");
    let uses_set_timeout = program_calls_bare_identifier(lir, "setTimeout");
    let uses_set_interval = program_calls_bare_identifier(lir, "setInterval");
    let uses_clear_timeout = program_calls_bare_identifier(lir, "clearTimeout");
    let uses_clear_interval = program_calls_bare_identifier(lir, "clearInterval");
```

and extend `function_index_offset` with five `+ if uses_… { 1 } else { 0 }` terms in that order.

(c) **Probe.** Alongside the existing `program_uses_*` probes:

```rust
/// Program-wide probe for a bare-identifier call to `name` (Stage D
/// scheduling surfaces). The callee is a PLAIN identifier, not a member
/// expression. Kept a SUPERSET of the emit-time recognizer
/// (`scheduling_surface` additionally requires the name be unshadowed): if
/// this were ever false where emit fires, the conditional import would be
/// undeclared and the emitted `Call` invalid wasm — over-inclusive is the
/// safe side.
pub(crate) fn program_calls_bare_identifier(lir: &LirProgram, name: &str) -> bool {
    lir.nodes.iter().any(|node| {
        if node.kind != LirNodeKind::Call {
            return false;
        }
        let Some(&callee) = node.children.first() else {
            return false;
        };
        let Some(callee_node) = lir.nodes.get(callee.0 as usize) else {
            return false;
        };
        callee_node.text.as_deref() == Some(name)
    })
}
```

(d) **Import indices.** Immediately after the `crypto_subtle_digest_import_index` block, add five blocks following its exact pattern. Each index = the full flag chain of every conditional import declared before it. Copy the chain from `crypto_subtle_digest_import_index`, then:

```rust
    let queue_microtask_import_index = if uses_queue_microtask {
        Some(/* crypto_subtle_digest's full chain */ + if uses_crypto_subtle_digest { 1 } else { 0 })
    } else {
        None
    };
    let set_timeout_import_index = if uses_set_timeout {
        Some(/* queue_microtask's full chain */ + if uses_queue_microtask { 1 } else { 0 })
    } else {
        None
    };
    // …set_interval adds `+ if uses_set_timeout {1} else {0}`,
    // clear_timeout adds `+ if uses_set_interval {1} else {0}`,
    // clear_interval adds `+ if uses_clear_timeout {1} else {0}`.
```

Write each chain out in full (no shared `base` variable — match the file's existing style verbatim). **Verify the invariant against the CURRENT file, not this plan:** each conditional import's index = `COVERAGE_HIT_IMPORT_INDEX` + one term per conditional flag declared before it in the import section. The end-to-end tests in Step 7 catch any arithmetic error as a wasmtime instantiation failure.

(e) **Import section entries.** After the `crypto_subtle_digest` `import_section.import(...)` block:

```rust
    if queue_microtask_import_index.is_some() {
        // `(callback_index: i32, env_ptr: i64) -> ()` — same shape as
        // test_register: pushes the callback id + the scheduling-site
        // `current_env` onto the host microtask FIFO; drained after `_start`
        // (`kali_runtime::host::enforce::drain_event_loop`).
        import_section.import(
            "kali:rt",
            "queueMicrotask",
            EntityType::Function(TEST_REGISTER_TYPE_INDEX),
        );
    }
    if set_timeout_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "setTimeout",
            EntityType::Function(SCHEDULING_TIMER_SET_TYPE_INDEX),
        );
    }
    if set_interval_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "setInterval",
            EntityType::Function(SCHEDULING_TIMER_SET_TYPE_INDEX),
        );
    }
    if clear_timeout_import_index.is_some() {
        // `(timer_id: i32) -> ()` — same shape as coverage_hit (type 0).
        import_section.import("kali:rt", "clearTimeout", EntityType::Function(0));
    }
    if clear_interval_import_index.is_some() {
        import_section.import("kali:rt", "clearInterval", EntityType::Function(0));
    }
```

(f) Thread all five indices through the `FunctionEmitter` constructor call (after `crypto_subtle_digest_import_index`).

- [ ] **Step 4: Emitter fields**

In `crates/kali_codegen/src/emitter.rs`, after `crypto_subtle_digest_import_index`, add five fields, ctor params, and ctor assignments following the existing pattern exactly:

```rust
    /// Stage D scheduling-surface host import indices — `Some` only when the
    /// matching `program_calls_bare_identifier` probe found a call; appended
    /// after `crypto_subtle_digest` in declaration order (queueMicrotask,
    /// setTimeout, setInterval, clearTimeout, clearInterval).
    pub(crate) queue_microtask_import_index: Option<u32>,
    pub(crate) set_timeout_import_index: Option<u32>,
    pub(crate) set_interval_import_index: Option<u32>,
    pub(crate) clear_timeout_import_index: Option<u32>,
    pub(crate) clear_interval_import_index: Option<u32>,
```

- [ ] **Step 5: Recognizer + callback resolver in `intrinsics/host.rs`**

Add near `is_kali_test_call` / `kali_test_callback_index`:

```rust
/// Stage D scheduling surfaces codegen emits real registrations for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SchedulingSurface {
    QueueMicrotask,
    SetTimeout,
    SetInterval,
    ClearTimeout,
    ClearInterval,
}

/// How a scheduling call's callback argument resolved (Stage D).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SchedulingCallback {
    /// Stable provenance to a compiled function: its raw wasm index.
    Resolved(u32),
    /// An identifier resolving to NOTHING in any codegen namespace — the
    /// flattened block-arrow lane (`Value("unknown")`). Kept on the
    /// pre-existing placeholder warning path until Task 7 (the un-flatten)
    /// deletes this variant; after that every arrow is a real function.
    LegacyPlaceholder,
    /// Everything else: unresolvable/unstable provenance — fail closed E5506.
    Deny,
}
```

and the two methods on `FunctionEmitter`:

```rust
    /// Recognize a bare, UNSHADOWED global scheduling callee (Stage D
    /// provenance rule: "bare unshadowed global callee only"). Any user
    /// binding, local, or function of the same name shadows the global and
    /// the call takes the normal user-call lane.
    pub(crate) fn scheduling_surface(&self, callee_node: &LirNode) -> Option<SchedulingSurface> {
        if !callee_node.children.is_empty() {
            return None;
        }
        let name = callee_node.text.as_deref()?;
        let surface = match name {
            "queueMicrotask" => SchedulingSurface::QueueMicrotask,
            "setTimeout" => SchedulingSurface::SetTimeout,
            "setInterval" => SchedulingSurface::SetInterval,
            "clearTimeout" => SchedulingSurface::ClearTimeout,
            "clearInterval" => SchedulingSurface::ClearInterval,
            _ => return None,
        };
        if self.locals.contains_key(name)
            || self.bindings.contains_key(name)
            || self.module_binding_names.contains(name)
            || self.fn_valued_locals.contains_key(name)
            || self.functions.contains_key(name)
        {
            return None;
        }
        Some(surface)
    }

    /// Resolve a scheduling call's callback argument (`children[1]`) by
    /// STABLE provenance — the same rules as the Stage C
    /// `scheduling_call_args_provably_safe` guard, but yielding the function
    /// index for the registration emit. Capturing callbacks resolve too:
    /// their soundness is `env_safety`'s job (registration edges), not this
    /// resolver's.
    pub(crate) fn scheduling_callback(&self, node: &LirNode) -> SchedulingCallback {
        let Some(&cb) = node.children.get(1) else {
            return SchedulingCallback::Deny;
        };
        let cb = self.unwrap_transparent(cb);
        let cb_node = self.node(cb);
        let Some(text) = cb_node.text.as_deref() else {
            return SchedulingCallback::Deny;
        };
        match cb_node.kind {
            // Inline function expression/declaration lowered as a plan: its
            // node text is the `__kali_fn_N` / declared plan key.
            LirNodeKind::Instruction => match self.functions.get(text) {
                Some(&index) => SchedulingCallback::Resolved(index),
                None => SchedulingCallback::Deny,
            },
            LirNodeKind::Value if cb_node.children.is_empty() => {
                if self.unstable_provenance_names.contains(text) {
                    return SchedulingCallback::Deny;
                }
                if let Some(key) = self.fn_valued_locals.get(text) {
                    return match self.functions.get(key) {
                        Some(&index) => SchedulingCallback::Resolved(index),
                        None => SchedulingCallback::Deny,
                    };
                }
                if self.locals.contains_key(text)
                    || self.bindings.contains_key(text)
                    || self.module_binding_names.contains(text)
                {
                    // A live binding without function provenance: unknown value.
                    return SchedulingCallback::Deny;
                }
                if let Some(&index) = self.functions.get(text) {
                    // Bare unshadowed function name.
                    return SchedulingCallback::Resolved(index);
                }
                SchedulingCallback::LegacyPlaceholder
            }
            _ => SchedulingCallback::Deny,
        }
    }
```

(If `unwrap_transparent` lives on a different receiver or takes/returns different types, mirror how `scheduling_call_args_provably_safe` in `emit/call.rs:2918` calls it — that guard is the reference implementation for these provenance rules.)

- [ ] **Step 6: The emit arm (queueMicrotask only in this task)**

In `emit/call.rs`, immediately after the entire `is_kali_test_call` arm (after its closing brace, ~line 151) — i.e. BEFORE any argument values are pushed on the wasm stack — insert:

```rust
        if let Some(surface) = self.scheduling_surface(&callee_node) {
            if let Some(result) = self.try_emit_scheduling_call(function, node, surface) {
                return result;
            }
        }
```

and add to the same impl (near the guard functions at the bottom):

```rust
    /// Stage D scheduling-surface registration emit. Returns `None` only for
    /// surfaces this task has not wired yet (they fall through to the Stage C
    /// default-deny guard below); `Some` is a fully-handled call.
    fn try_emit_scheduling_call(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        surface: SchedulingSurface,
    ) -> Option<EmittedValue> {
        match surface {
            SchedulingSurface::QueueMicrotask => {
                Some(self.emit_queue_microtask_call(function, node))
            }
            // Timers: wired in the next task; keep the Stage C guard lane.
            _ => None,
        }
    }

    fn emit_queue_microtask_call(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        // node.children: [callee, callback]; extra args fail closed (node
        // ignores them, but silently divergent arity is exactly the class
        // this stage exists to eliminate — precision follow-up).
        if node.children.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "queueMicrotask requires exactly one callback argument in the current phase"
                    .to_string(),
            ));
            return EmittedValue { produced: false, shape: ValueShape::Unknown };
        }
        match self.scheduling_callback(node) {
            SchedulingCallback::Resolved(index) => {
                let Some(import) = self.queue_microtask_import_index else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "queueMicrotask import unavailable (probe/emit desync)".to_string(),
                    ));
                    return EmittedValue { produced: false, shape: ValueShape::Unknown };
                };
                function.instruction(&Instruction::I32Const(index as i32));
                // env_ptr: the `current_env` active at the registration site
                // (the C3 test_register pattern); the host restores it before
                // invoking the callback, and env_safety's registration edges
                // prove it is the capture owner's record (or E5506).
                function.instruction(&Instruction::GlobalGet(self.current_env_global()));
                function.instruction(&Instruction::Call(import));
                EmittedValue { produced: false, shape: ValueShape::Unknown }
            }
            SchedulingCallback::LegacyPlaceholder => {
                // Pre-un-flatten flattened-arrow lane (`Value("unknown")`):
                // preserve the pre-existing placeholder warning + zero result.
                // Task 7 deletes this branch (and the enum variant) — after
                // the un-flatten every arrow is a real compiled function.
                self.push_placeholder_fallback_diagnostic("call target", "queueMicrotask");
                function.instruction(&Instruction::I64Const(0));
                EmittedValue { produced: true, shape: ValueShape::Unknown }
            }
            SchedulingCallback::Deny => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "a queueMicrotask callback must resolve through stable provenance to a compiled function (an inline function expression, a declarator-recorded function local, or an unshadowed function name); an unresolvable callback would be silently dropped".to_string(),
                ));
                EmittedValue { produced: false, shape: ValueShape::Unknown }
            }
        }
    }
```

Then remove `Some("queueMicrotask")` from `is_undrained_scheduling_surface` (the emit arm now handles every bare-unshadowed `queueMicrotask` call; shadowed ones take the user-call lane and must not hit the guard either — they are not the scheduling surface). Update that function's doc comment: the un-emittable family is now `setTimeout`/`setInterval`/`addEventListener`.

- [ ] **Step 7: env_safety registration edges (all three surfaces at once)**

In `crates/kali_codegen/src/env_safety.rs`, next to `is_kali_test_callee` (~line 134) add:

```rust
/// True when `callee_id` is a bare-identifier scheduling callee whose call
/// REGISTERS its callback argument (`children[1]`) for a later host-driven
/// invocation (`queueMicrotask` / `setTimeout` / `setInterval`, Stage D).
/// The env active at the registration site is what the host restores before
/// invoking the callback, so the registration site inherits the same
/// Record(owner) requirement as a direct call — the `Kali.test` precedent.
/// Shadowing is ignored here: a spurious edge from a user-shadowed name is a
/// safe over-approximation (this analysis only ever REJECTS more).
fn is_scheduling_registration_callee(nodes: &[LirNode], callee: LirNodeId) -> bool {
    nodes.get(callee.0 as usize).is_some_and(|node| {
        node.children.is_empty()
            && matches!(
                node.text.as_deref(),
                Some("queueMicrotask") | Some("setTimeout") | Some("setInterval")
            )
    })
}
```

and in the edge-collection loop (~line 314), replace the `target_root` selection:

```rust
                    let target_root = if is_kali_test_callee(&lir.nodes, callee) {
                        node.children.get(2).copied()
                    } else if is_scheduling_registration_callee(&lir.nodes, callee) {
                        node.children.get(1).copied()
                    } else {
                        Some(callee)
                    };
```

- [ ] **Step 8: Build, run the two new tests**

```bash
cargo build -p kali_cli 2>&1 | tail -3
rm -rf .kali-cache
cargo test -p kali_cli --test soundness_closures deferred_queue_microtask_fn_expr deferred_queue_microtask_capturing_callback_runs 2>&1 | tail -5
```

Expected: PASS. A wasmtime "incompatible import type" or instantiation error here means the import-index arithmetic in Step 3(d) is wrong — recheck each chain against the file.

- [ ] **Step 9: Flip the Stage C queueMicrotask pins (deliberate capability flips, spec §3 D2)**

In `soundness_closures.rs`:
- `deferred_queue_microtask_capturing_callback_fails_closed` (§6 row o): rename to `deferred_queue_microtask_capturing_callback_now_runs`; replace the E5506 assertions with success + exact stdout. Its fixture prints `sync=5` then `mt=6` per the Stage C triage node column — verify against node first, then assert `"sync=5\nmt=6\n"` (adjust to the fixture's actual strings after reading it).
- `deferred_queue_microtask_module_scope_capture_still_runs` (bg1): the callback now RUNS (was: silent drop, printed `sync=0` only). Update to assert the full node-parity output (`sync=0` then the callback's line — read the fixture, run node, assert exactly).
- `deferred_queue_microtask_non_capturing_callback_still_runs` (bg2): same treatment — full node-parity stdout.

Keep every OTHER Stage C pin untouched — in particular rows p/q2/q3 (`setTimeout`/`setInterval` still E5506 until Task 5) and x/y/z (alias/call-result/reassigned: still Deny by provenance — permanent).

Each flipped pin's doc comment must state: pre-Stage-D behavior, the Stage D task that flipped it, and the node-verified expected output.

- [ ] **Step 10: Full test file + full gate**

```bash
cargo test -p kali_cli --test soundness_closures 2>&1 | tail -5
```
Expected: all pass. Then the full gate (`stageD-post-task4.txt`): newly-red MUST be empty.

- [ ] **Step 11: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/soundness_closures.rs
git commit -m "feat(codegen): queueMicrotask registration lane — env_ptr ABI, provenance-resolved callbacks, env_safety registration edges (D2) [stageD]"
```

---

### Task 5: D2 — timer lanes (`setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`)

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (`try_emit_scheduling_call` timer arms; shrink `is_undrained_scheduling_surface` to `addEventListener` only)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (only if a helper is needed beyond Task 4's)
- Test: `crates/kali_cli/tests/soundness_closures.rs`

**Interfaces:**
- Consumes: Task 4's `SchedulingSurface`, `SchedulingCallback`, `scheduling_callback()`, import-index fields, `SCHEDULING_TIMER_SET_TYPE_INDEX`; Task 2/3 runtime semantics; `parse_numeric_literal_value` (already used in `emit/call.rs`).
- Produces: full timer emit; after this task the only surface left in `is_undrained_scheduling_surface` is `addEventListener`.

- [ ] **Step 1: Write the failing end-to-end tests** (verify EVERY expected output against node first)

```rust
/// Stage D: microtasks drain before timers; timers fire in delay order with
/// registration-order tiebreak — full ordering matrix in one fixture.
/// node v26.5.0: "sync\nm\na\nb\n".
#[test]
fn deferred_ordering_microtasks_then_timers_in_delay_order() {
    let out = run_kali(
        r#"setTimeout(function () { console.log("b"); }, 10);
setTimeout(function () { console.log("a"); }, 5);
queueMicrotask(function () { console.log("m"); });
console.log("sync");
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\nm\na\nb\n");
}

/// Stage D: a capturing setTimeout callback runs with its owner's env record
/// after the owner returned (the never-reset-region property via timers).
/// node v26.5.0: "sync=5\nst=6\n".
#[test]
fn deferred_set_timeout_capturing_callback_now_runs() {
    let out = run_kali(
        r#"function owner() {
  let base = 5;
  setTimeout(function () {
    base += 1;
    console.log("st=" + base);
  }, 0);
  console.log("sync=" + base);
}
owner();
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nst=6\n");
}

/// Stage D: setInterval ticks repeatedly and clearInterval (with the captured
/// timer id) stops it. Function-scope variant: `n` and `t` are env cells.
/// node v26.5.0: "sync\ntick=1\ntick=2\ntick=3\n".
#[test]
fn deferred_set_interval_ticks_until_cleared() {
    let out = run_kali(
        r#"function main() {
  let n = 0;
  const t = setInterval(function () {
    n += 1;
    console.log("tick=" + n);
    if (n >= 3) {
      clearInterval(t);
    }
  }, 0);
  console.log("sync");
}
main();
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "sync\ntick=1\ntick=2\ntick=3\n"
    );
}

/// Stage D: clearTimeout cancels a pending timer — the callback never runs.
/// node v26.5.0: "sync\n".
#[test]
fn deferred_clear_timeout_cancels_pending_callback() {
    let out = run_kali(
        r#"function main() {
  const t = setTimeout(function () {
    console.log("never");
  }, 0);
  clearTimeout(t);
  console.log("sync");
}
main();
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\n");
}

/// Stage D bounded drain, end to end: an uncleared interval must trap loudly
/// (exit != 0, "did not quiesce"), never hang. (node would hang here — the
/// one deliberate divergence, spec decision 3.)
#[test]
fn deferred_uncleared_interval_fails_loudly_not_hangs() {
    let out = run_kali(
        r#"setInterval(function () {}, 0);
console.log("sync");
"#,
    );
    assert!(!out.status.success(), "expected the non-quiescence trap, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("did not quiesce"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Stage D envelope: a non-literal delay fails closed (precision follow-up).
#[test]
fn deferred_set_timeout_non_literal_delay_fails_closed() {
    let out = run_kali(
        r#"let d = 5;
setTimeout(function () { console.log("x"); }, d);
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Stage D envelope: extra forwarded args fail closed (node passes them to
/// the callback; kali has no arg-forwarding lane — reject, don't drop).
#[test]
fn deferred_set_timeout_extra_args_fail_closed() {
    let out = run_kali(
        r#"setTimeout(function () { console.log("x"); }, 0, 42);
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
```

Module-scope fallback note for `deferred_set_interval_ticks_until_cleared`: if the function-scope fixture rejects for a reason OUTSIDE this task's scope (e.g. `const t = <call>` declarator-provenance interaction with capture promotion), record the exact diagnostic in the task report and use this node-verified module-scope variant instead, keeping the function-scope source as an `#[ignore = "..."]` pin naming the blocking lane:

```js
let n = 0;
var t = 0;
t = setInterval(function () {
  n += 1;
  console.log("tick=" + n);
  if (n >= 3) { clearInterval(t); }
}, 0);
console.log("sync");
```

(node: `sync\ntick=1\ntick=2\ntick=3\n` — re-verify before use.)

- [ ] **Step 2: Run to verify they fail**

```bash
rm -rf .kali-cache
cargo test -p kali_cli --test soundness_closures deferred_ordering deferred_set_timeout_capturing_callback_now_runs deferred_set_interval_ticks deferred_clear_timeout_cancels deferred_uncleared_interval 2>&1 | tail -8
```

Expected: all FAIL (capturing forms E5506 via the Stage C guard; non-capturing forms silently drop).

- [ ] **Step 3: Implement the timer arms**

In `try_emit_scheduling_call`, replace the `_ => None` catch-all:

```rust
            SchedulingSurface::SetTimeout | SchedulingSurface::SetInterval => {
                Some(self.emit_timer_set_call(function, node, surface))
            }
            SchedulingSurface::ClearTimeout | SchedulingSurface::ClearInterval => {
                Some(self.emit_timer_clear_call(function, node, surface))
            }
```

and add:

```rust
    fn emit_timer_set_call(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        surface: SchedulingSurface,
    ) -> EmittedValue {
        let surface_name = if surface == SchedulingSurface::SetTimeout {
            "setTimeout"
        } else {
            "setInterval"
        };
        let fail_closed = |this: &mut Self, message: String| {
            this.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            EmittedValue { produced: false, shape: ValueShape::Unknown }
        };
        // node.children: [callee, callback, optional delay]; anything else —
        // including node's arg-forwarding form setTimeout(fn, 0, x) — fails
        // closed (spec §5).
        if node.children.len() < 2 || node.children.len() > 3 {
            return fail_closed(
                self,
                format!("{surface_name} supports exactly (callback[, delay]) in the current phase; extra arguments have no forwarding lane"),
            );
        }
        // Delay: a NUMERIC LITERAL or absent (spec envelope: provably-numeric
        // only; widening to proven-scalar bindings is precision follow-up
        // work). Clamping below 1 happens host-side (node parity).
        let delay: i32 = match node.children.get(2) {
            None => 0,
            Some(&d) => {
                let d = self.unwrap_transparent(d);
                let d_node = self.node(d);
                let literal = d_node
                    .children
                    .is_empty()
                    .then(|| d_node.text.as_deref())
                    .flatten()
                    .and_then(parse_numeric_literal_value);
                match literal {
                    Some(value) => value.max(0.0).min(i32::MAX as f64) as i32,
                    None => {
                        return fail_closed(
                            self,
                            format!("a {surface_name} delay must be a numeric literal in the current phase (a computed delay has no provably-numeric lowering yet)"),
                        )
                    }
                }
            }
        };
        let import = if surface == SchedulingSurface::SetTimeout {
            self.set_timeout_import_index
        } else {
            self.set_interval_import_index
        };
        let Some(import) = import else {
            return fail_closed(self, format!("{surface_name} import unavailable (probe/emit desync)"));
        };
        match self.scheduling_callback(node) {
            SchedulingCallback::Resolved(index) => {
                function.instruction(&Instruction::I32Const(index as i32));
                function.instruction(&Instruction::I32Const(delay));
                function.instruction(&Instruction::GlobalGet(self.current_env_global()));
                function.instruction(&Instruction::Call(import));
                // The host returns the i32 timer id; kali scalars are i64.
                function.instruction(&Instruction::I64ExtendI32S);
                EmittedValue { produced: true, shape: ValueShape::Unknown }
            }
            SchedulingCallback::LegacyPlaceholder => {
                // Pre-un-flatten flattened-arrow lane — deleted in Task 7.
                self.push_placeholder_fallback_diagnostic("call target", surface_name);
                function.instruction(&Instruction::I64Const(0));
                EmittedValue { produced: true, shape: ValueShape::Unknown }
            }
            SchedulingCallback::Deny => fail_closed(
                self,
                format!("a {surface_name} callback must resolve through stable provenance to a compiled function; an unresolvable callback would be silently dropped"),
            ),
        }
    }

    fn emit_timer_clear_call(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        surface: SchedulingSurface,
    ) -> EmittedValue {
        let surface_name = if surface == SchedulingSurface::ClearTimeout {
            "clearTimeout"
        } else {
            "clearInterval"
        };
        if node.children.len() != 2 {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!("{surface_name} requires exactly one timer-id argument"),
            ));
            return EmittedValue { produced: false, shape: ValueShape::Unknown };
        }
        let import = if surface == SchedulingSurface::ClearTimeout {
            self.clear_timeout_import_index
        } else {
            self.clear_interval_import_index
        };
        let Some(import) = import else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!("{surface_name} import unavailable (probe/emit desync)"),
            ));
            return EmittedValue { produced: false, shape: ValueShape::Unknown };
        };
        // Any i64 value is SOUND as a cancel id: the host no-ops an unknown/
        // already-fired id (node parity), so no provenance proof is needed.
        let emitted = self.emit_node(function, node.children[1], true);
        if !emitted.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Call(import));
        EmittedValue { produced: false, shape: ValueShape::Unknown }
    }
```

Then shrink `is_undrained_scheduling_surface` to `addEventListener` only, and update its doc comment (the guard is now solely the `addEventListener` deny lane).

- [ ] **Step 4: Run the new tests** — expected: PASS (or the documented module-scope fallback for the interval fixture).

- [ ] **Step 5: Flip the Stage C timer pins** (same treatment as Task 4 Step 9; node-verify each expected output against the fixture's actual source first):
- `deferred_set_timeout_capturing_callback_fails_closed` (row p) → `_now_runs`, node output per triage: `sync=5` / `st=6`.
- `deferred_set_interval_capturing_callback_fails_closed` (row q2) → `_now_runs`, node output: `sync=5` / `iv=6`.
- `deferred_set_timeout_indirect_capturing_callback_fails_closed` (row q3, `let cb = fn; setTimeout(cb, 0)`) → `_now_runs`, node output: `1`.
- `deferred_set_timeout_indirect_non_capturing_callback_still_runs` (bg3): callback now RUNS — assert full node-parity stdout.
- **Leave x/y/z as E5506 pins** (alias `let cb2 = cb`, call-result `setTimeout(makeCb(), 0)`, reassigned `cb`): their provenance is Deny by construction. Run each of the three to confirm they still pass (the resolver must NOT have accidentally widened); if any fails, that is a fail-open — fix the resolver, do not re-pin.

- [ ] **Step 6: Full suite + full gate**

```bash
cargo test -p kali_cli --test soundness_closures 2>&1 | tail -5
```
Then the full gate (`stageD-post-task5.txt`): newly-red MUST be empty.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/soundness_closures.rs
git commit -m "feat(codegen): setTimeout/setInterval/clear* timer lanes — literal-delay gate, id return, guard shrunk to addEventListener (D2) [stageD]"
```

---

### Task 6: D2 — browser-lane JS glue parity (4 hand-mirrored import lists)

**Files:**
- Modify: `crates/kali_runtime/src/browser/harness.rs` (2 `importObject` sites — grep `test_register(`)
- Modify: `crates/kali_runtime/src/browser/execute_tests/harness.rs` (only if it contains its own `importObject`; check first)
- Modify: `crates/kali_cli/src/bin/cmd_build.rs` (2 `importObject` sites, lines ~1555 and ~1893)

**Interfaces:**
- Consumes: the 5 import names/signatures from Tasks 4–5; Task 2/3 drain semantics (the JS mirror must match: virtual clock, `(due, seq)` order, 1ms clamp, cancelled-set re-arm rule, 100k budget).
- Produces: browser modules that import any of the 5 surfaces instantiate and drain instead of LinkError-ing.

**Background:** These are Rust `format!` string literals — every JS `{`/`}` must be doubled (`{{`/`}}`) exactly as written below. The harness sites instantiate the module and `await instance.exports._start()` (harness.rs lines ~386 and ~805); the glue's `test_register` precedent shows the style.

- [ ] **Step 1: Locate all four sites and confirm the count**

```bash
grep -n "test_register(" crates/kali_runtime/src/browser/harness.rs crates/kali_runtime/src/browser/execute_tests/harness.rs crates/kali_cli/src/bin/cmd_build.rs
```

Expected: 4 matches total (harness.rs ×2, cmd_build.rs ×2; the execute_tests/harness.rs file had none as of Stage C — if it now has one, treat it as a 5th site with identical treatment).

- [ ] **Step 2: Add the shared scheduling state + drain, once per site's script prelude**

Insert BEFORE the `const importObject = {{` of each site (adjusting only surrounding whitespace):

```js
const kaliPendingMicrotasks = [];
const kaliPendingTimers = new Map();
const kaliCancelledTimers = new Set();
let kaliNextTimerId = 1;
let kaliNextTimerSeq = 1;
let kaliVirtualNowMs = 0;
const KALI_EVENT_LOOP_BUDGET = 100000;
function kaliScheduleTimer(callbackId, delayMs, repeat, envPtr) {{
  // Mirrors kali_runtime::state::schedule_timer: 1ms minimum clamp,
  // virtual-clock due time, fresh seq per (re)arm.
  const effective = Math.max(1, Number(delayMs) | 0);
  const id = kaliNextTimerId++;
  kaliPendingTimers.set(id, {{
    callbackId,
    envPtr,
    dueAtMs: kaliVirtualNowMs + effective,
    seq: kaliNextTimerSeq++,
    repeatMs: repeat ? effective : null,
  }});
  return id;
}}
function kaliCancelTimer(timerId) {{
  const id = Number(timerId);
  if (!kaliPendingTimers.delete(id)) {{
    kaliCancelledTimers.add(id);
  }}
}}
async function kaliInvokeCallback(instance, callbackId, envPtr) {{
  // Mirrors kali_runtime::host::enforce::invoke_callback: set the exported
  // __current_env to the registration-time env, restore after (both paths).
  const envGlobal = instance.exports.__current_env;
  const saved = envGlobal ? envGlobal.value : null;
  if (envGlobal) {{ envGlobal.value = envPtr; }}
  try {{
    await instance.exports[`__kali_callback_${{callbackId}}`]();
  }} finally {{
    if (envGlobal) {{ envGlobal.value = saved; }}
  }}
}}
async function kaliDrainEventLoop(instance) {{
  // Mirrors kali_runtime::host::enforce::drain_event_loop: all microtasks
  // FIFO before each timer; timers by (dueAtMs, seq); re-arm unless
  // cancelled while firing; bounded by the invocation budget.
  let invoked = 0;
  for (;;) {{
    if (invoked >= KALI_EVENT_LOOP_BUDGET) {{
      throw new Error('event loop did not quiesce: scheduled-callback budget exceeded');
    }}
    const microtask = kaliPendingMicrotasks.shift();
    if (microtask) {{
      invoked += 1;
      await kaliInvokeCallback(instance, microtask.callbackId, microtask.envPtr);
      continue;
    }}
    let next = null;
    for (const [id, timer] of kaliPendingTimers) {{
      if (
        next === null
        || timer.dueAtMs < next.timer.dueAtMs
        || (timer.dueAtMs === next.timer.dueAtMs && timer.seq < next.timer.seq)
      ) {{
        next = {{ id, timer }};
      }}
    }}
    if (next === null) {{ return; }}
    kaliPendingTimers.delete(next.id);
    kaliVirtualNowMs = Math.max(kaliVirtualNowMs, next.timer.dueAtMs);
    invoked += 1;
    await kaliInvokeCallback(instance, next.timer.callbackId, next.timer.envPtr);
    if (next.timer.repeatMs !== null && !kaliCancelledTimers.delete(next.id)) {{
      kaliPendingTimers.set(next.id, {{
        callbackId: next.timer.callbackId,
        envPtr: next.timer.envPtr,
        dueAtMs: kaliVirtualNowMs + next.timer.repeatMs,
        seq: kaliNextTimerSeq++,
        repeatMs: next.timer.repeatMs,
      }});
    }}
  }}
}}
```

- [ ] **Step 3: Add the five importObject entries at each site** (next to `test_register`):

```js
    queueMicrotask(callbackId, envPtr) {{
      kaliPendingMicrotasks.push({{ callbackId, envPtr }});
    }},
    setTimeout(callbackId, delayMs, envPtr) {{
      return kaliScheduleTimer(callbackId, delayMs, false, envPtr);
    }},
    setInterval(callbackId, delayMs, envPtr) {{
      return kaliScheduleTimer(callbackId, delayMs, true, envPtr);
    }},
    clearTimeout(timerId) {{
      kaliCancelTimer(timerId);
    }},
    clearInterval(timerId) {{
      kaliCancelTimer(timerId);
    }},
```

- [ ] **Step 4: Invoke the drain after `_start` at each site that calls `_start`**

Immediately after each `await instance.exports._start();` line add:

```js
await kaliDrainEventLoop(instance);
```

If a cmd_build site never invokes `_start` (pure bundle-glue definition), add the import entries anyway (signature parity — the `test_register` no-op precedent) and note in a code comment that the drain runs wherever the bundle's loader calls `_start`.

- [ ] **Step 5: Verify the mirror lists**

```bash
for f in crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs; do
  for n in queueMicrotask setTimeout setInterval clearTimeout clearInterval; do
    echo "$f $n: $(grep -c "    $n(" $f)"
  done
done
```

Expected: count 2 per name per file (one per site).

- [ ] **Step 6: Build + full gate**

```bash
cargo build -p kali_cli -p kali_runtime 2>&1 | tail -3
```
Then the full gate (`stageD-post-task6.txt`): newly-red MUST be empty. The browser families run under this gate (node-preferred harness) — a LinkError or JS syntax error in the glue shows up here. Note: no browser fixture exercises the new glue functionally until Task 7 un-flattens the arrows; the D3 gate is the real browser-lane proof.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_runtime/src/browser crates/kali_cli/src/bin/cmd_build.rs
git commit -m "feat(browser): mirror the deferred-callback lane in all 4 JS import lists — virtual-clock drain, env restore, budget (D2) [stageD]"
```

---

### Task 7: D3 — parser un-flatten + anonymous-argument gate + carve-out deletions

**Files:**
- Modify: `crates/kali_parser/src/expression/primary.rs` (WIP patch hunks, verbatim)
- Modify: `crates/kali_types/src/resolve/call.rs` (WIP patch hunk, minus the `deferred_but_unwired` carve-out)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` + `crates/kali_codegen/src/emit/call.rs` (delete `SchedulingCallback::LegacyPlaceholder`; harden the `Kali.test` fallback; `scheduling_call_args_provably_safe` resolve-to-nothing → deny)
- Modify: `crates/kali_cli/tests/runtime_smoke.rs` (WIP patch re-pin hunk for `assert_json_object_type_and_constructor_semantics` — pre-approved re-pin, stage-6 triage §4c item 1)
- Test: `crates/kali_cli/tests/soundness_block_arrows.rs` (WIP patch test hunks + the probes below)

**Interfaces:**
- Consumes: everything from Tasks 2–6.
- Produces: block-bodied arrows parse as unnamed `FunctionExpression` (the declarator-form desugar) in ALL expression positions; the flattened-arrow `Value("unknown")` lane no longer exists; the scheduling/`Kali.test` default-deny is total.

- [ ] **Step 1: Apply the parser + types hunks from the seed patch**

```bash
git apply --include='crates/kali_parser/*' --include='crates/kali_types/*' \
  docs/superpowers/followups/task5-block-arrows-WIP.patch
cargo build -p kali_cli 2>&1 | tail -3
```

- [ ] **Step 2: Remove the now-false `deferred_but_unwired` carve-out from the applied gate**

In `crates/kali_types/src/resolve/call.rs`, `reject_anonymous_function_argument`: Tasks 4–5 wired `setTimeout`/`setInterval` (registration + host invocation), so the WIP patch's premise for force-rejecting them is falsified. Delete the `deferred_but_unwired` binding and its use, leaving:

```rust
        if self.resolve_name(callee_name).is_none() || Self::is_builtin_global_name(callee_name) {
            return;
        }
```

and rewrite the two doc-comment paragraphs about `setTimeout`/`setInterval` to say: all three scheduling surfaces are WIRED as of Stage D Tasks 4–5 (codegen emits `queueMicrotask`/`setTimeout`/`setInterval` registrations; the runtime drains them), so they take the generic builtin exemption; the codegen-side provenance resolver (`scheduling_callback`) and `env_safety` own the fail-closed decisions for unresolvable/unsound callbacks.

- [ ] **Step 3: Delete the flattened-arrow carve-outs in codegen (the residual lane no longer exists)**

(a) In `intrinsics/host.rs`: remove the `LegacyPlaceholder` variant from `SchedulingCallback`; in `scheduling_callback`, the resolves-to-nothing tail becomes `SchedulingCallback::Deny` with the comment:

```rust
                // Post-un-flatten (Stage D Task 7): every arrow is a real
                // compiled function, so an identifier resolving to NOTHING in
                // any codegen namespace is a genuinely unresolvable value —
                // deny. (Pre-D3 this was the flattened-arrow placeholder lane.)
                SchedulingCallback::Deny
```

(b) In `emit/call.rs`: delete the two `LegacyPlaceholder` match arms (`emit_queue_microtask_call`, `emit_timer_set_call`) — the compiler's exhaustiveness check confirms the variant is gone.

(c) In `scheduling_call_args_provably_safe` (now `addEventListener`-only): change the final resolve-to-nothing `true` to `false`, updating its comment (the `Value("unknown")` lane is deleted; an unknown identifier is unresolvable → deny).

(d) In the `Kali.test` arm's `callback_is_unregisterable_value` closure: the `LirNodeKind::Value` arm drops its liveness conditions — any bare-identifier `Value` callback that `kali_test_callback_index` could not resolve is now a deny:

```rust
                    // Post-un-flatten: a bare identifier in callback position
                    // that `kali_test_callback_index` did not resolve is a
                    // real value this lane cannot register — deny. (The
                    // pre-D3 flattened-arrow `Value("unknown")` placeholder no
                    // longer exists.)
                    LirNodeKind::Value => cb_node.children.is_empty(),
```

Update the arm's doc comment (delete the "Deliberately narrower than a blanket deny" paragraph — it describes the deleted lane).

- [ ] **Step 4: Apply the WIP patch's test hunks**

```bash
git apply --include='crates/kali_cli/tests/*' docs/superpowers/followups/task5-block-arrows-WIP.patch
```

This lands: the two `soundness_block_arrows.rs` queueMicrotask-arrow probes (deferred-not-inline; actually-runs-during-drain), the uninvocable-anonymous-callback E5506 probe, the formatting fix in `class_method_bodies_return_their_value`, and the `runtime_smoke.rs` `assert_json_object_type_and_constructor_semantics` re-pin (pre-approved: stage-6 triage §4c — the patched error-reporting shape is strictly better: `total: 1`, callback-attributed trap).

- [ ] **Step 5: Add the remaining stage-6 probes as permanent pins**

Append to `soundness_block_arrows.rs` (node-verify each first):

```rust
/// Stage-6 probe 3a pinned: a feature-rich block-arrow callback body must
/// run DEFERRED with correct ordering, not inline — the module lines print
/// FIRST, the microtask drains after. node v26.5.0 (stage-6 triage §3):
/// "MODULE-END-acc\n0\nINSIDE-CALLBACK\n15\n". Re-verify with node before
/// asserting. Pre-D3 kali inverted the order and leaked the mutated `acc`.
#[test]
fn a_feature_rich_block_arrow_callback_defers_with_correct_ordering() {
    let out = run_kali(
        r#"class Box { constructor() { this.n = 4; } }
let acc = 0;
queueMicrotask(() => {
  let value = 1;
  value += 2;
  value *= 5;
  let b = new Box();
  acc = value + b.n;
  console.log("INSIDE-CALLBACK");
  console.log(value);
});
console.log("MODULE-END-acc");
console.log(acc);
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "MODULE-END-acc\n0\nINSIDE-CALLBACK\n15\n"
    );
}

/// Stage-6 probe 4 pinned: a block-arrow `Kali.test` callback registers a
/// REAL test (total: 1), and its body does NOT run inline at module scope.
/// Pre-D3: the body ran between the module lines and the harness printed a
/// vacuous `ok 1` with zero registered tests.
#[test]
fn a_block_arrow_kali_test_registers_a_real_test() {
    let out = run_kali_test(
        r#"console.log("A-module-start");
Kali.test('real-test', () => { console.log("B-inside-test-body"); });
console.log("C-module-end");
"#,
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The body line must come AFTER module end (deferred to the test run),
    // and the run must report a registered, passing test.
    let a = stdout.find("A-module-start").expect("module start printed");
    let c = stdout.find("C-module-end").expect("module end printed");
    let b = stdout.find("B-inside-test-body").expect("test body ran");
    assert!(a < c && c < b, "test body ran inline at module scope: {stdout}");
    assert!(stdout.contains("ok 1"), "expected a passing registered test: {stdout}");
}
```

`run_kali_test` helper (add next to `run_kali`, same shape but `arg("test")` instead of `arg("run")`). For BOTH probes: run the source through node (for the first) and through the freshly-built `kali test` (for the second) BEFORE finalizing assertions; if `kali test`'s summary line differs in exact text (e.g. TAP formatting), pin the observed passing-shape (the load-bearing asserts are the ordering and a registered non-vacuous test).

- [ ] **Step 6: Focused runs**

```bash
rm -rf .kali-cache && cargo build -p kali_cli 2>&1 | tail -3
cargo test -p kali_cli --test soundness_block_arrows 2>&1 | tail -5
cargo test -p kali_cli --test soundness_closures 2>&1 | tail -5
```

Expected: all pass. The Task 4/5 fixtures now ALSO cover the arrow spellings implicitly (arrows desugar to the same `FunctionExpression` lane); add arrow-spelling variants of the two capturing fixtures (`queueMicrotask(() => { base += 1; … })`, `setTimeout(() => { base += 1; … }, 0)`) as quick confirmations, node-verified:

```rust
/// Arrow spelling of deferred_queue_microtask_capturing_callback_runs_with_env —
/// the un-flatten routes `() => {…}` through the same FunctionExpression lane.
/// node v26.5.0: "sync=5\nmt=6\n".
#[test]
fn deferred_queue_microtask_capturing_block_arrow_runs_with_env() {
    let out = run_kali(
        r#"function owner() {
  let base = 5;
  queueMicrotask(() => {
    base += 1;
    console.log("mt=" + base);
  });
  console.log("sync=" + base);
}
owner();
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nmt=6\n");
}
```

(and the analogous `setTimeout` variant).

- [ ] **Step 7: Full enumeration — measured, NOT required empty**

Run the full gate (`stageD-post-task7.txt`). The newly-red set here is the REAL blast radius on the full foundation:

```bash
comm -13 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-task7.txt" > "$SCRATCH/stageD-task7-newly-red.txt"
wc -l "$SCRATCH/stageD-task7-newly-red.txt"
```

Classify every entry against Task 1's buckets: (a) should now be GREEN (deferred-surface families — if any bucket-a test is still red, that is a Task 4–6 defect: STOP and fix before proceeding); (b) re-pin candidates (`addEventListener` families, diagnostic-shape pins beyond the one pre-approved in Step 4); (c) unexplained — STOP and investigate. Record the classified list in the triage doc.

- [ ] **Step 8: Commit** (the branch carries the measured, classified red set until Task 8 — deliberate mid-phase state, per the spec's D3 sequencing):

```bash
git add crates/kali_parser crates/kali_types crates/kali_codegen crates/kali_cli docs/superpowers/followups/stageD-triage.md
git commit -m "feat(parser+codegen): land the block-arrow un-flatten; total default-deny (placeholder lane deleted) (D3) [stageD]"
```

---

### Task 8: D3 — re-pin batch (USER APPROVAL GATE) + restore the green gate

**Files:**
- Modify: the test files named by Task 7's measured re-pin list (expected: browser/web-baseline families in `runtime_smoke.rs` and the browser corpus suites)
- Modify: `docs/superpowers/followups/stageD-triage.md`

**Interfaces:**
- Consumes: `$SCRATCH/stageD-task7-newly-red.txt` + its classification.
- Produces: a green gate (`comm -13` empty vs `stageD-pre.txt`).

- [ ] **Step 1: Build the re-pin evidence table**

For each red family: extract the fixture source, run it through node and through the Stage D binary, and record (family, count, node behavior, kali-before = the eager-inline miscompile output, kali-after = the observed E5506/trap, proposed re-pin). Two classes are pre-authorized by the spec (§3 D3): (a) main-green-via-eager-inline tests hitting still-denied surfaces (`addEventListener`); (b) diagnostic-shape pins where the new shape is strictly better. Anything else is NOT re-pinnable — it must be fixed or escalated.

- [ ] **Step 2: STOP — present the batch to the user**

Present the evidence table and wait for explicit approval before changing any test. **Do not proceed on silence.** If the user rejects a family, the fallback is fixing the underlying lane or reverting Task 7 (their call).

- [ ] **Step 3: Land the approved re-pins**

Each re-pinned test gets: the E5506/fail-closed assertion replacing the behavioral one, and a doc comment naming (1) the pre-Stage-D eager-inline miscompile it used to pass through, (2) the node-verified correct behavior, (3) the follow-up stage that will flip it back (event/EventTarget lowering). No assertion deletions — every re-pin still asserts a SPECIFIC failure shape.

- [ ] **Step 4: Full gate — must be green**

Run the full gate (`stageD-post-task8.txt`):

```bash
comm -13 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-task8.txt"   # MUST be empty
comm -23 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-task8.txt"   # drain: record + explain each
```

Drain entries (previously-red tests now green) are expected here — the deferred-surface families the stage fixed. List each in the triage doc with its fixing task.

- [ ] **Step 5: Update the triage doc + commit**

```bash
git add -A
git commit -m "test(soundness): stageD re-pin batch (user-approved) — gate restored to 0 newly-red [stageD]"
```

---

### Task 9: D4 — whole-stage adversarial review + final gate + close-out

**Files:**
- Modify: `docs/superpowers/followups/stageD-triage.md` (close-out sections)
- Any fix-wave files the review demands.

**Interfaces:**
- Consumes: the full Stage D diff (`git diff <task1-commit>..HEAD`), the Stage-6 Tasks 1–4 foundation commits (`5a0cc82a0`, `d294ba8e6`, `51de2bb7a`+`52c4bc11e`, `3b12dc6cc`).
- Produces: the certified stage close-out.

- [ ] **Step 1: Dispatch the whole-stage adversarial review**

Scope for the reviewer (most capable model, cross-task probes — the Stage AB/C pattern that caught both prior CRITICALs):
1. The D1–D3 diff as a whole, with emphasis on the SEAMS: recognizer × shadowing (`scheduling_surface` vs user bindings), resolver × guard (`scheduling_callback` vs `scheduling_call_args_provably_safe` — do they agree everywhere both apply?), env_safety registration edges × the emit sites (is EVERY emitted registration's callee shape edge-modeled — including inside nested functions?), virtual clock × the JS glue mirror (semantics drift), budget × `Kali.test` drain interaction.
2. **The Stage-6 Tasks 1–4 foundation** (deferred review, spec D4): `name_anonymous_functions` pre-pass collision guard, the 3-site repr-tracking mirror, the `#[ignore]`'d class-method pin — now that the un-flatten exercises them for real.
3. Adversarial probes the fixtures don't cover — reviewer must RUN candidate reproducers through both kali and node (e.g.: capturing callback registered from a SIBLING env-owner's extent via setTimeout — must be E5506 via env_safety, never corruption; `let setTimeout = function(a){return 9}; console.log(setTimeout(1))` — user shadowing must take the normal lane; a block arrow in a non-callback expression position, e.g. assigned then passed; `clearInterval` called with a stale id after the interval already self-cleared; timer scheduled from INSIDE a drained microtask).
4. Fail-open hunting per the standing lesson: any path where an unresolvable/unproven shape RUNS instead of rejecting is a finding, regardless of how exotic.

- [ ] **Step 2: Fix wave** — every CONFIRMED finding fixed fail-closed (or explicitly pinned + inventoried with user sign-off), each with a red→green reproducer test, following the Stage C §13 format in the triage doc.

- [ ] **Step 3: Final double-enumeration gate**

Two independent full enumerations on the final binary (`.kali-cache` cleared before each), zero drift between them, `comm -13` vs `stageD-pre.txt` empty, drain recorded. Then the main-worktree cross-check:

```bash
cd /workspace/.worktrees/kali-main && git fetch origin main && git log --oneline -1
rm -rf .kali-cache
cargo test --workspace --no-fail-fast 2>&1 | grep -cE '^test .* \.\.\. FAILED'   # expected: 0
```

- [ ] **Step 4: Close out the triage doc**

Sections: final gate numbers (entry 731 → exit 731 + drain), the deliberate pin-flip ledger (Tasks 2/4/5/7 flips + Task 8 re-pins, each with its evidence row), review findings + dispositions, and the follow-up inventory hand-off — carry forward: `addEventListener`/EventTarget lowering stage (re-flips the Task 8 re-pins + unblocks the §8 headline), delay-expression precision (non-literal delays), alias-provenance precision (rows x/y/z), escaping first-class closures (rows m/n/s), depth≥2 env chains, lexical parent links (the env_safety envelope's real fix), F-AB-1 expr-arrow-return miscompile, `ok 1`-with-zero-tests harness distinguishability (§12 item 11 — partially mitigated by the Task 7 `Kali.test` hardening; verify and update its status).

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "docs(soundness): stageD close-out — un-flatten landed, deferred lane live, gate 731 held [stageD]"
```

---

## Plan Self-Review (performed at write time)

1. **Spec coverage:** D0→Task 1; D1 (virtual clock, ordering, IDs, clamp, budget, env restore)→Tasks 2–3; D2 (recognizers, resolver, env_safety edges, guard re-scope, timer args, pin flips)→Tasks 4–5; the 4 JS import lists (spec §4 browser note)→Task 6; D3 (parser, anon-gate, carve-out deletions, `Kali.test` real registration, stage-6 probes, re-pins)→Tasks 7–8; D4 (review incl. Stage-6 T1–4, final gate, triage)→Task 9. Envelope items each have a fixture: non-scalar/non-literal delay ✓, extra args ✓, unstable provenance (x/y/z retained) ✓, addEventListener deny (retained Stage C pin row q — verify untouched in Task 5 Step 6) ✓, budget trap ✓ (unit + e2e).
2. **Sequencing soundness:** every fail-open window checked — timers keep the Stage C guard until their emit arm lands (Task 5 removes them from `is_undrained_scheduling_surface` in the same commit as the arms); env_safety edges land BEFORE the first capturing registration can lower (Task 4); the LegacyPlaceholder lane preserves pre-D3 behavior byte-for-byte and dies in Task 7.
3. **Type consistency:** `SchedulingSurface`/`SchedulingCallback`/`scheduling_callback`/`emit_scheduling_call` names match across Tasks 4, 5, 7; `ScheduledTimer` fields match between Task 2 (Rust) and Task 6 (JS mirror); the budget constant and message text match between Task 3 and Tasks 5–6.
4. **Known uncertainty, made explicit rather than hidden:** import-index chain must be verified against the live file (Task 4 Step 3d); `const t = setInterval(...)` capture-promotion interaction has a documented node-verified fallback fixture (Task 5 Step 1); `kali test` summary text pinned to observed shape (Task 7 Step 5); budget-diagnostic wrapping (Task 3 Step 4).
