# Stage D Event-Surface Lane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `new EventTarget()` / `target.addEventListener(lit, cb)` / `target.dispatchEvent(new CustomEvent(lit))` compile and RUN with node-parity semantics inside a fail-closed envelope, making the 4 `browser_bundle_web_baseline_primitives` build tests green legitimately and restoring the Stage D gate.

**Architecture:** Host-registry + synchronous re-entrant dispatch (spec §3): EventTarget = opaque i64 host handle; listeners stored host-side keyed `(handle, event_type)`; `event_dispatch` re-enters the guest mid-`_start` via a `wasmtime::Caller`-based invoke (empirically verified on wasmtime 24.0.7). Codegen mirrors the Stage D Task 4/5 mechanics (probes → conditional imports → recognizers → emit arms → env_safety edges), plus a declarator-recorded receiver-provenance map and a handle-escape choke point. Browser glue lands BEFORE the emit arms so no gate window can LinkError.

**Tech Stack:** Rust workspace (`cargo`), wasm via `wasm-encoder`/`wasmtime 24.0.7`, node v26.5.0 as the behavioral oracle.

**Spec:** `docs/superpowers/specs/2026-07-17-stageD-event-surface-design.md`. Verified facts this plan relies on (probe evidence in `.superpowers/sdd/progress.md`):
- Sync re-entrancy from a `func_wrap` closure WORKS (`caller.get_export` → `.typed()` → `.call(&mut caller, ())` mid-`_start`); constraint: NEVER hold a `caller.data_mut()` borrow across the re-entrant call (E0499) — snapshot first.
- LIR member call = `Call` node, children `[callee, name, cb]`; callee = `Value`, text = method name, children = `[receiver]`. `new CustomEvent('tick')` argument = `Value`, NO text, children = `[Value("CustomEvent"), Literal("tick")]`. `is_kali_test_call` (`intrinsics/host.rs:769-778`) is the byte-identical shape precedent.
- No reusable fixed wasm type: new types 15 `() -> i64`, 16 `(i64,i32,i32,i32,i64) -> ()`, 17 `(i64,i32,i32) -> i32`; repr-directed base bumps 15 → 18 (`lower.rs:905-909`).
- Guest strings read host-side via `read_guest_string(caller, ptr, len)` (`kali_runtime/src/host/memory.rs:15-22`).

## Global Constraints

- Branch: `soundness-batch1-pra`. Never merge to main. Commits carry the `[stageD]` suffix.
- **Gate command** (referenced as "the full gate"; `stageD-pre.txt` is the canonical 731-line Stage D baseline):
  ```bash
  SCRATCH=/tmp/claude-1000/-workspace/8914ce72-e24a-40bc-9a7a-aa137ef87a2d/scratchpad
  cd /workspace && rm -rf .kali-cache && cargo build -p kali_cli 2>/dev/null
  (cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > "$SCRATCH/stageD-post-ev<task>.txt"; touch "$SCRATCH/stageD-ev<task>.done") &
  timeout 1800 bash -c "until [ -f '$SCRATCH/stageD-ev<task>.done' ]; do sleep 30; done"
  comm -13 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-ev<task>.txt"   # newly-red
  comm -23 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-ev<task>.txt"   # drain — record
  ```
  Always `sort -u`, always `--no-fail-fast`, always `rm -rf .kali-cache` first. Poll the `.done` marker with the blocking `timeout … until` call — never `pgrep -f`, never end a turn to "wait".
- **Per-task gate expectation:** Tasks 1–4: newly-red = EXACTLY the 4 inherited `browser_bundle_web_baseline_primitives` entries (the Task 7 mid-phase state) — no additions, no substitutions. Task 5: newly-red EMPTY (gate restored); drain = Task 7's 37 (record).
- Node oracle: every e2e fixture's expected output MUST be verified by running the same source through `node` BEFORE writing the assertion.
- Any new `kali:rt` import emitted by codegen must already be present in all 4 hand-mirrored JS import lists — this plan lands glue (Task 2) BEFORE emit arms (Task 4) by design; do not reorder.
- Kali is GC-less by design; nothing here allocates guest-heap objects (handles are scalars; listener storage is host-side).
- Imports only, no new synthetic guest functions — if a `count_tag_boxing_ops` census test goes red, that is a desync to investigate, not a product regression.
- Fail-closed rule for every codegen path in this plan: any shape not explicitly in-lane → `Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, …)` (E5506) with a message naming the unsupported feature. No warnings-and-continue on the new lane.

---

### Task 1: Runtime — event registry, 3 host imports, synchronous re-entrant invoke

**Files:**
- Modify: `crates/kali_runtime/src/state.rs` (registry fields + methods, near the timer state)
- Modify: `crates/kali_runtime/src/host/enforce.rs` (new `invoke_callback_reentrant`)
- Modify: `crates/kali_runtime/src/host/imports_default.rs` (3 `func_wrap` entries, after the `clearInterval` import)
- Test: create `crates/kali_runtime/src/execute_tests/events.rs` (register in the `execute_tests` mod the same way `timers.rs` is)

**Interfaces:**
- Consumes: `read_guest_string` (`host/memory.rs:15-22`), the `invoke_callback` env save/restore contract (`enforce.rs:109-191`), `KaliHostState`.
- Produces (Tasks 2 and 4 rely on these exact semantics): imports `kali:rt`::`event_target_new () -> i64`, `event_listener_add (i64,i32,i32,i32,i64) -> ()`, `event_dispatch (i64,i32,i32) -> i32`; state methods `register_event_target() -> u32`, `add_event_listener(target: u32, event_type: String, callback_id: i32, env_ptr: i64)` (dedup by exact `(callback_id, env_ptr)` pair), `event_listener_snapshot(target: u32, event_type: &str) -> Vec<(i32, i64)>`.

- [ ] **Step 1: Write the failing WAT tests**

Create `crates/kali_runtime/src/execute_tests/events.rs`, following `timers.rs`'s style (`RuntimeCtx::with_host_context`, `compile_wat`, `capture_env`, same imports):

```rust
#[test]
fn runtime_event_dispatch_invokes_listeners_synchronously_in_registration_order() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Two listeners registered for "tick"; dispatch must run them IN ORDER,
    // synchronously (the state checks after the dispatch call happen inside
    // _start, before it returns).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
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
                    (local $t i64)
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 2
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    i32.const 1
                    i32.ne
                    if unreachable end        ;; dispatch must return 1 (true)
                    global.get $state
                    i32.const 2
                    i32.ne
                    if unreachable end)       ;; both listeners already ran — SYNCHRONOUS
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_dispatch_snapshot_excludes_listeners_added_during_dispatch() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // callback_1 (fired by dispatch #1) registers callback_2. Dispatch #1 must
    // NOT invoke callback_2 (snapshot semantics); dispatch #2 invokes both.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (global $t (mut i64) (i64.const 0))
                (global $ones (mut i32) (i32.const 0))
                (global $twos (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    global.get $ones
                    i32.const 1
                    i32.add
                    global.set $ones
                    global.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 2
                    i64.const 0
                    call $el_add)
                (func (export "__kali_callback_2")
                    global.get $twos
                    i32.const 1
                    i32.add
                    global.set $twos)
                (func (export "_start")
                    call $et_new
                    global.set $t
                    global.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    global.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    ;; after dispatch #1: ones=1, twos=0 (snapshot excluded cb2)
                    global.get $twos
                    if unreachable end
                    global.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    ;; after dispatch #2: ones=2, twos=1
                    global.get $ones
                    i32.const 2
                    i32.ne
                    if unreachable end
                    global.get $twos
                    i32.const 1
                    i32.ne
                    if unreachable end)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_duplicate_registration_dedups_by_callback_and_env() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // Same (callback_id, env_ptr) registered twice → fires ONCE per dispatch
    // (node dedups by listener identity). A different env_ptr is a different
    // listener and fires separately.
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (global $count (mut i32) (i32.const 0))
                (func (export "__kali_callback_1")
                    global.get $count
                    i32.const 1
                    i32.add
                    global.set $count)
                (func (export "_start")
                    (local $t i64)
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add          ;; exact duplicate — dedup
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 7
                    call $el_add          ;; different env_ptr — distinct listener
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    global.get $count
                    i32.const 2
                    i32.ne
                    if unreachable end)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_dispatch_restores_current_env_and_zero_listeners_returns_true() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    // The listener runs with __current_env = its registration env (42) and
    // _start's env (7) is restored afterward. Dispatching an event with no
    // listeners returns 1 (node: dispatchEvent with no listeners → true).
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (data (i32.const 24) "none")
                (global $env (export "__current_env") (mut i64) (i64.const 0))
                (global $seen (mut i64) (i64.const -1))
                (func (export "__kali_callback_1")
                    global.get $env
                    global.set $seen)
                (func (export "_start")
                    (local $t i64)
                    i64.const 7
                    global.set $env
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 42
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop
                    global.get $seen
                    i64.const 42
                    i64.ne
                    if unreachable end        ;; listener saw its registration env
                    global.get $env
                    i64.const 7
                    i64.ne
                    if unreachable end        ;; _start's env restored
                    local.get $t
                    i32.const 24
                    i32.const 4
                    call $ev_dispatch
                    i32.const 1
                    i32.ne
                    if unreachable end)       ;; zero listeners → still true
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_event_listener_trap_propagates_loudly() {
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "event_target_new" (func $et_new (result i64)))
                (import "kali:rt" "event_listener_add" (func $el_add (param i64 i32 i32 i32 i64)))
                (import "kali:rt" "event_dispatch" (func $ev_dispatch (param i64 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 16) "tick")
                (func (export "__kali_callback_1")
                    unreachable)
                (func (export "_start")
                    (local $t i64)
                    call $et_new
                    local.set $t
                    local.get $t
                    i32.const 16
                    i32.const 4
                    i32.const 1
                    i64.const 0
                    call $el_add
                    local.get $t
                    i32.const 16
                    i32.const 4
                    call $ev_dispatch
                    drop)
            )
            "#,
    );
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    // The trap surfaces through the _start error path (the listener trapped
    // while _start was on the stack). Assert the diagnostic names the
    // callback; adapt ONLY the outcome-shape access to what execute.rs
    // actually returns for a _start-time trap (this is a _start-time trap,
    // NOT a drain-time one — see the timers.rs trap tests for both shapes).
    let diagnostic_text = format!("{:?}", outcome);
    assert!(
        diagnostic_text.contains("__kali_callback_1"),
        "expected the trap to be attributed to the listener, got: {diagnostic_text}"
    );
}
```

Wire the module: add `pub mod events;` (or `mod events;`) wherever `timers.rs` is declared (grep `mod timers` under `crates/kali_runtime/src/execute_tests`).

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p kali_runtime events 2>&1 | tail -8
```
Expected: FAIL — the three imports don't exist yet, so instantiation errors ("unknown import: `kali:rt::event_target_new`").

- [ ] **Step 3: Implement the state registry**

In `crates/kali_runtime/src/state.rs`, next to the timer fields add:

```rust
    /// Stage D event-surface lane: next EventTarget handle (allocated 1..).
    pub next_event_target_id: u32,
    /// Listener registry keyed (target_handle, event_type); values are
    /// registration-ordered (callback_id, env_ptr) pairs. BTreeMap for
    /// deterministic iteration; duplicates dedup by exact pair (node dedups
    /// by listener identity — (callback_id, env_ptr) is the closest analog).
    pub event_listeners: BTreeMap<(u32, String), Vec<(i32, i64)>>,
```

with `Default` entries (`1` for `next_event_target_id` so 0 is never a valid handle, empty map). Add methods next to `schedule_timer`:

```rust
    pub(crate) fn register_event_target(&mut self) -> u32 {
        let id = self.next_event_target_id;
        self.next_event_target_id = self.next_event_target_id.wrapping_add(1);
        id
    }

    pub(crate) fn add_event_listener(
        &mut self,
        target: u32,
        event_type: String,
        callback_id: i32,
        env_ptr: i64,
    ) {
        let listeners = self.event_listeners.entry((target, event_type)).or_default();
        if !listeners.contains(&(callback_id, env_ptr)) {
            listeners.push((callback_id, env_ptr));
        }
    }

    pub(crate) fn event_listener_snapshot(
        &self,
        target: u32,
        event_type: &str,
    ) -> Vec<(i32, i64)> {
        self.event_listeners
            .get(&(target, event_type.to_string()))
            .cloned()
            .unwrap_or_default()
    }
```

(If the borrow checker or existing style prefers a `(u32, &str)` lookup without the `to_string`, match the file's existing map-key idiom — behavior is what's pinned.)

- [ ] **Step 4: Implement `invoke_callback_reentrant` in `enforce.rs`**

Next to `invoke_callback` (~line 109), following its export-lookup and env save/restore logic exactly, but `Caller`-based:

```rust
/// Synchronously invoke `__kali_callback_<id>` from INSIDE a host import,
/// while guest code (e.g. `_start`) is still on the wasm stack (Stage D
/// event dispatch). Mirrors `invoke_callback`'s contract: set the exported
/// `__current_env` global to the registration-time `env_ptr`, invoke, restore
/// the previous value on both ok and err paths. NEVER call this while holding
/// a `caller.data_mut()` borrow — snapshot host state first (E0499 otherwise).
pub(crate) fn invoke_callback_reentrant(
    caller: &mut wasmtime::Caller<'_, crate::state::KaliHostState>,
    callback_id: i32,
    env_ptr: i64,
) -> wasmtime::Result<()> {
    let export_name = format!("__kali_callback_{callback_id}");
    let func = caller
        .get_export(&export_name)
        .and_then(|e| e.into_func())
        .ok_or_else(|| {
            wasmtime::Error::msg(format!("missing callback export '{export_name}'"))
        })?;
    let typed = func.typed::<(), ()>(&mut *caller)?;

    let env_global = caller.get_export("__current_env").and_then(|e| e.into_global());
    let saved = env_global.map(|g| g.get(&mut *caller));
    if let Some(global) = env_global {
        global.set(&mut *caller, wasmtime::Val::I64(env_ptr))?;
    }

    let result = typed.call(&mut *caller, ()).map_err(|err| {
        wasmtime::Error::msg(format!("runtime trap in callback '{export_name}': {err}"))
    });

    if let (Some(global), Some(value)) = (env_global, saved) {
        global.set(&mut *caller, value)?;
    }

    result
}
```

Adapt the error wrapping to match `invoke_callback`'s exact wording (read it first — the Task-2 retargeted timer tests pinned "runtime trap in callback"). If `Global`/`Func` handles are not `Copy` in wasmtime 24, bind them before use as the borrow checker requires (the probe crate at `/tmp/claude-1000/-workspace/8914ce72-e24a-40bc-9a7a-aa137ef87a2d/scratchpad/probe-spec6` has a compiling reference).

- [ ] **Step 5: Implement the three imports in `imports_default.rs`**

Immediately after the `clearInterval` `func_wrap` block, following the file's existing style:

```rust
    linker.func_wrap(
        "kali:rt",
        "event_target_new",
        |mut caller: Caller<'_, KaliHostState>| -> i64 {
            i64::from(caller.data_mut().register_event_target())
        },
    )?;

    linker.func_wrap(
        "kali:rt",
        "event_listener_add",
        |mut caller: Caller<'_, KaliHostState>,
         target: i64,
         name_ptr: i32,
         name_len: i32,
         callback_id: i32,
         env_ptr: i64|
         -> wasmtime::Result<()> {
            let event_type = read_guest_string(&mut caller, name_ptr, name_len)?;
            caller
                .data_mut()
                .add_event_listener(target as u32, event_type, callback_id, env_ptr);
            Ok(())
        },
    )?;

    linker.func_wrap(
        "kali:rt",
        "event_dispatch",
        |mut caller: Caller<'_, KaliHostState>,
         target: i64,
         name_ptr: i32,
         name_len: i32|
         -> wasmtime::Result<i32> {
            let event_type = read_guest_string(&mut caller, name_ptr, name_len)?;
            // Snapshot BEFORE re-entering the guest: node parity (listeners
            // added during dispatch don't fire this round) AND the borrow
            // constraint (no data_mut borrow may live across the call).
            let snapshot = caller
                .data()
                .event_listener_snapshot(target as u32, &event_type);
            for (callback_id, env_ptr) in snapshot {
                invoke_callback_reentrant(&mut caller, callback_id, env_ptr)?;
            }
            Ok(1)
        },
    )?;
```

Import `invoke_callback_reentrant` at the top of the file the way `invoke_callback`-adjacent items are imported. If `func_wrap` closures returning plain `i64` don't fit the file's error-handling idiom, match the idiom (some entries return `wasmtime::Result<T>` uniformly).

- [ ] **Step 6: Run the runtime suite**

```bash
cargo test -p kali_runtime 2>&1 | tail -5
```
Expected: all pass, including the 5 new tests. Fix `unused`/warning noise to keep output pristine.

- [ ] **Step 7: Full gate** (`stageD-post-ev1.txt`): newly-red must be EXACTLY the 4 inherited `browser_bundle_web_baseline_primitives` entries. Record drain.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_runtime
git commit -m "feat(runtime): event-surface registry + sync re-entrant dispatch — handles, (handle,type) listeners, snapshot semantics, env restore (evD) [stageD]"
```

---

### Task 2: Browser glue parity — 3 entries + registry prelude at all 4 sites

**Files:**
- Modify: `crates/kali_runtime/src/browser/harness.rs` (2 `importObject` sites)
- Modify: `crates/kali_cli/src/bin/cmd_build.rs` (2 `importObject` sites)

**Interfaces:**
- Consumes: Task 1's exact semantics (snapshot, dedup by pair, env save/restore, return 1).
- Produces: browser modules importing any of the 3 surfaces instantiate and behave identically to the Rust host. No codegen emits these yet — the entries are inert until Task 4 (deliberate: kills the LinkError window that would otherwise open when existing corpus fixtures become in-lane).

**Background:** Rust `format!` templates — every JS `{`/`}` doubled (`{{`/`}}`). Task 6 (Stage D) placed the scheduling prelude before each site's `const importObject = {{`; put the event prelude adjacent to it.

- [ ] **Step 1: Locate the 4 sites**

```bash
grep -n "clearInterval(timerId)" crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs
```
Expected: 4 matches (2 per file). Each site also has the `kaliPendingMicrotasks`/`kaliInvokeCallback` prelude from Stage D Task 6 nearby.

- [ ] **Step 2: Add the shared event-registry prelude at each site** (before/next to the existing scheduling prelude):

```js
const kaliEventListeners = new Map();
let kaliNextEventTargetId = 1;
function kaliEventKey(target, type) {{
  return `${{Number(target)}} ${{type}}`;
}}
function kaliInvokeCallbackSync(instance, callbackId, envPtr) {{
  // Mirrors kali_runtime::host::enforce::invoke_callback_reentrant: set the
  // exported __current_env to the registration-time env, restore after —
  // SYNCHRONOUSLY (dispatchEvent runs listeners before it returns).
  const envGlobal = instance.exports.__current_env;
  const saved = envGlobal ? envGlobal.value : null;
  if (envGlobal) {{ envGlobal.value = envPtr; }}
  try {{
    instance.exports[`__kali_callback_${{callbackId}}`]();
  }} finally {{
    if (envGlobal) {{ envGlobal.value = saved; }}
  }}
}}
function kaliEventDispatch(instance, target, type) {{
  // Snapshot first: listeners added during dispatch don't fire this round
  // (node parity; mirrors event_dispatch in imports_default.rs).
  const listeners = kaliEventListeners.get(kaliEventKey(target, type));
  const snapshot = listeners ? listeners.slice() : [];
  for (const entry of snapshot) {{
    kaliInvokeCallbackSync(instance, entry.callbackId, entry.envPtr);
  }}
  return 1;
}}
```

**Note:** `kaliEventDispatch`/`kaliInvokeCallbackSync` need `instance`, which the importObject closures must see. Check how the site's existing entries reference the instance (Stage D Task 6's `kaliDrainEventLoop(instance)` is called after `_start`, but import-object entries that need the instance at call time rely on the `instance` binding being assigned before `_start` runs — verify the site's declaration order allows it, i.e. `instance` is a `let`/`const` in scope declared before the importObject and assigned at instantiation; if a site declares importObject before `instance`, hoist a `let instance;` above the importObject and assign it after instantiation — that is the standard WebAssembly JS pattern and is safe because imports only run during `_start`, after assignment).

- [ ] **Step 3: Add the three importObject entries at each site** (next to `clearInterval`):

```js
    event_target_new() {{
      return BigInt(kaliNextEventTargetId++);
    }},
    event_listener_add(target, namePtr, nameLen, callbackId, envPtr) {{
      const type = kaliReadGuestString(namePtr, nameLen);
      const key = kaliEventKey(target, type);
      const existing = kaliEventListeners.get(key) || [];
      // Dedup by exact (callbackId, envPtr) pair — node's listener-identity rule.
      if (!existing.some((e) => e.callbackId === callbackId && e.envPtr === envPtr)) {{
        existing.push({{ callbackId, envPtr }});
      }}
      kaliEventListeners.set(key, existing);
    }},
    event_dispatch(target, namePtr, nameLen) {{
      return kaliEventDispatch(instance, target, nameLen === undefined ? '' : kaliReadGuestString(namePtr, nameLen));
    }},
```

**`kaliReadGuestString`:** each site already reads guest strings for `console_log`-family imports — find the site's existing ptr/len string-read helper (grep `TextDecoder` or `getUint8` near the importObject) and use ITS name; only add a helper if the site truly has none (then mirror the site's memory-access idiom). The arrow function inside `.some()` uses single braces INSIDE a format string — escape as the file's existing JS arrows do (check the Task 6 prelude for precedent; if arrows are awkward to escape, use a plain `for` loop instead). Sanity-check the emitted JS of one bundle by building any fixture and reading the generated glue.

Note on signatures: wasm `i64` crosses into JS as `BigInt`. `event_target_new` must return a BigInt; `target`/`envPtr` arrive as BigInt (the Stage D Task 6 review verified the existing entries' BigInt handling — follow the same conventions; `kaliEventKey` uses `Number(target)` which is safe for small handle counts).

- [ ] **Step 4: Verify the mirror lists**

```bash
for f in crates/kali_runtime/src/browser/harness.rs crates/kali_cli/src/bin/cmd_build.rs; do
  for n in event_target_new event_listener_add event_dispatch; do
    echo "$f $n: $(grep -c "    $n(" $f)"
  done
done
```
Expected: count 2 per name per file.

- [ ] **Step 5: Build + spot-check emitted JS**

```bash
cargo build -p kali_cli -p kali_runtime 2>&1 | tail -3
```
Then build any small fixture with `--bundle --api browser` and `node --check` the emitted JS glue file (or run it) to prove the format-string escaping produced valid JS.

- [ ] **Step 6: Full gate** (`stageD-post-ev2.txt`): newly-red = exactly the 4 inherited entries.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_runtime/src/browser crates/kali_cli/src/bin/cmd_build.rs
git commit -m "feat(browser): mirror the event-surface lane in all 4 JS import lists — registry, sync dispatch, snapshot + dedup (evD) [stageD]"
```

---

### Task 3: Codegen plumbing — types 15–17, conditional imports, construction lane, receiver provenance, handle-escape choke point

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (types, probes, flags, indices, import entries, ctor threading, declarator promotion)
- Modify: `crates/kali_codegen/src/emitter.rs` (3 import-index fields + `event_target_locals`)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (`is_event_target_new`)
- Test: `crates/kali_cli/tests/soundness_events.rs` (create; escape/reassign/non-declarator pins)

**Interfaces:**
- Consumes: Task 1's import names/signatures; the Stage D Task 4 plumbing patterns (`program_calls_bare_identifier`, conditional-index chains, `SCHEDULING_TIMER_SET_TYPE_INDEX = 14`); the Task 5 declarator-promotion precedent (`declarator_init_call_callee_name` + the `matches!` exclusion chain, `lower.rs:667-680`).
- Produces (Task 4 relies on these exact names): emitter fields `event_target_new_import_index`, `event_listener_add_import_index`, `event_dispatch_import_index` (all `Option<u32>`); `const EVENT_TARGET_NEW_TYPE_INDEX: u32 = 15`, `const EVENT_LISTENER_ADD_TYPE_INDEX: u32 = 16`, `const EVENT_DISPATCH_TYPE_INDEX: u32 = 17`; emitter field `event_target_locals: BTreeSet<String>`; `pub(crate) fn is_event_target_new(&self, node: &LirNode) -> bool` on the emitter (unshadowed global `EventTarget`, zero args).

- [ ] **Step 1: Write the failing pins**

Create `crates/kali_cli/tests/soundness_events.rs` with a `run_kali` helper copied from `soundness_closures.rs`'s (same shape: build + run a temp fixture, return Output). Add:

```rust
/// Stage D event lane: `new EventTarget()` in a declarator compiles and the
/// program runs (the handle is opaque; nothing observable yet).
/// node v26.5.0: "done\n".
#[test]
fn event_target_construction_in_declarator_compiles_and_runs() {
    let out = run_kali(
        r#"const t = new EventTarget();
console.log("done");
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "done\n");
}

/// Handle-escape discipline (spec §2.4): an EventTarget binding read outside
/// the lane's allowed positions fails closed — the handle must never leak as
/// a number (node prints `EventTarget {}`; kali would print the raw handle).
#[test]
fn event_target_handle_escape_fails_closed() {
    let out = run_kali(
        r#"const t = new EventTarget();
console.log(t);
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Reassigned target bindings lose stable provenance — every later lane use
/// fails closed (the unstable_provenance_names rule).
#[test]
fn event_target_reassigned_binding_fails_closed() {
    let out = run_kali(
        r#"let t = new EventTarget();
t = new EventTarget();
t.addEventListener("tick", function () {});
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `new EventTarget()` OUTSIDE a declarator has no recordable provenance —
/// fail closed, don't emit an untracked handle.
#[test]
fn event_target_non_declarator_construction_fails_closed() {
    let out = run_kali(
        r#"new EventTarget();
console.log("done");
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

Node-verify the first fixture (`node /tmp/f.js` → `done`). Note: pre-implementation, the FIRST test may already "pass" via the placeholder-warning lane (build succeeds, prints done) — the load-bearing red states are the three deny pins (they currently BUILD fine via placeholders → exit 0 → the asserts fail). Record which are red.

- [ ] **Step 2: Run to verify the deny pins fail**

```bash
cargo test -p kali_cli --test soundness_events 2>&1 | tail -8
```

- [ ] **Step 3: Types 15–17 + repr base bump in `lower.rs`**

After the type-14 registration (`SCHEDULING_TIMER_SET_TYPE_INDEX`, ~line 678):

```rust
    // Type 15: event_target_new `() -> i64` (Stage D event lane) — returns a
    // fresh opaque EventTarget handle.
    const EVENT_TARGET_NEW_TYPE_INDEX: u32 = 15;
    type_section.ty().function(vec![], vec![ValType::I64]);
    // Type 16: event_listener_add `(target: i64, name_ptr: i32, name_len: i32,
    // callback_index: i32, env_ptr: i64) -> ()`.
    const EVENT_LISTENER_ADD_TYPE_INDEX: u32 = 16;
    type_section.ty().function(
        vec![ValType::I64, ValType::I32, ValType::I32, ValType::I32, ValType::I64],
        vec![],
    );
    // Type 17: event_dispatch `(target: i64, name_ptr: i32, name_len: i32) -> i32`
    // — synchronously invokes the snapshot of listeners, returns 1 (true).
    // This is now the last fixed type: repr-directed function types start at 18.
    const EVENT_DISPATCH_TYPE_INDEX: u32 = 17;
    type_section.ty().function(vec![ValType::I64, ValType::I32, ValType::I32], vec![ValType::I32]);
```

Bump the repr-directed base at `lower.rs:905-909` from `SCHEDULING_TIMER_SET_TYPE_INDEX + 1` to `EVENT_DISPATCH_TYPE_INDEX + 1`, and move the "last fixed type" comment sentence from type 14's block to type 17's (shown above).

- [ ] **Step 4: Probes, flags, indices, import entries, ctor threading**

(a) Probes alongside `program_calls_bare_identifier`:

```rust
/// Program-wide probe for `new EventTarget(...)` (Stage D event lane).
/// New-expressions lower to a text-less Value whose children[0] is the
/// constructor identifier. SUPERSET of the emit-time recognizer (which
/// additionally requires unshadowed + zero args + declarator position).
pub(crate) fn program_constructs_event_target(lir: &LirProgram) -> bool {
    lir.nodes.iter().any(|node| {
        node.text.is_none()
            && node.children.first().is_some_and(|&c| {
                lir.nodes
                    .get(c.0 as usize)
                    .is_some_and(|n| n.text.as_deref() == Some("EventTarget"))
            })
    })
}

/// Program-wide probe for a MEMBER call named `name` (Stage D event lane):
/// any node whose text is `name` and which has children (the receiver).
/// SUPERSET of the emit-time recognizer (receiver provenance unchecked here).
pub(crate) fn program_calls_member_named(lir: &LirProgram, name: &str) -> bool {
    lir.nodes
        .iter()
        .any(|node| node.text.as_deref() == Some(name) && !node.children.is_empty())
}
```

**Verify the New-expression LIR shape against reality before trusting the probe** (it was source-verified, not dump-verified): compile `const t = new EventTarget(); console.log("x");` and check the probe fires (a temporary `eprintln!` or a unit test on the lowered LIR — remove scaffolding after). If the shape differs (e.g. New keeps a text), adjust probe + recognizer together.

(b) Flags next to `uses_clear_interval`:

```rust
    let uses_event_target_new = program_constructs_event_target(lir);
    let uses_event_listener_add = program_calls_member_named(lir, "addEventListener");
    let uses_event_dispatch = program_calls_member_named(lir, "dispatchEvent");
```

Extend `function_index_offset` with the three `+ if uses_… { 1 } else { 0 }` terms in that order (after clear_interval's).

(c) Import indices after `clear_interval_import_index`, each chain = the previous import's full chain + one term, written out in full per the file's style (the Stage D Task 4 review verified this pattern verbatim — copy `clear_interval_import_index`'s chain as the base):

```rust
    let event_target_new_import_index = if uses_event_target_new {
        Some(/* clear_interval's full chain */ + if uses_clear_interval { 1 } else { 0 })
    } else {
        None
    };
    // event_listener_add adds `+ if uses_event_target_new {1} else {0}`,
    // event_dispatch adds `+ if uses_event_listener_add {1} else {0}`.
```

(d) Import-section entries after the `clearInterval` block:

```rust
    if event_target_new_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "event_target_new",
            EntityType::Function(EVENT_TARGET_NEW_TYPE_INDEX),
        );
    }
    if event_listener_add_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "event_listener_add",
            EntityType::Function(EVENT_LISTENER_ADD_TYPE_INDEX),
        );
    }
    if event_dispatch_import_index.is_some() {
        import_section.import(
            "kali:rt",
            "event_dispatch",
            EntityType::Function(EVENT_DISPATCH_TYPE_INDEX),
        );
    }
```

(e) Thread the three indices through the `FunctionEmitter` ctor (fields in Step 5).

- [ ] **Step 5: Emitter fields + recognizer**

`emitter.rs`, after `clear_interval_import_index`:

```rust
    /// Stage D event-lane host import indices (Some only when the matching
    /// program-wide probe fired; appended after clear_interval).
    pub(crate) event_target_new_import_index: Option<u32>,
    pub(crate) event_listener_add_import_index: Option<u32>,
    pub(crate) event_dispatch_import_index: Option<u32>,
    /// Locals/module bindings with stable provenance to `new EventTarget()`
    /// (declarator-recorded, the fn_valued_locals pattern). Reads outside the
    /// lane's allowed positions fail closed (handle-escape discipline).
    pub(crate) event_target_locals: BTreeSet<String>,
```

`intrinsics/host.rs`, near `is_kali_test_call`:

```rust
    /// Recognize `new EventTarget()` — a text-less Value whose children are
    /// exactly [Value("EventTarget")] with the name unshadowed in every
    /// codegen namespace and ZERO constructor args (spec §2.1). Everything
    /// else (args, shadowed name) is out of lane.
    pub(crate) fn is_event_target_new(&self, node: &LirNode) -> bool {
        if node.text.is_some() || node.children.len() != 1 {
            return false;
        }
        let Some(ctor) = self.node_opt(node.children[0]) else { return false };
        if ctor.text.as_deref() != Some("EventTarget") || !ctor.children.is_empty() {
            return false;
        }
        !(self.locals.contains_key("EventTarget")
            || self.bindings.contains_key("EventTarget")
            || self.module_binding_names.contains("EventTarget")
            || self.fn_valued_locals.contains_key("EventTarget")
            || self.functions.contains_key("EventTarget"))
    }
```

(`self.node_opt` — use whatever accessor the file already uses to fetch a child node, e.g. `self.node(id)`; mirror `scheduling_surface`'s namespace checks verbatim. Confirm the New node's child list is `[ctor]` for zero args vs `[ctor, ...args]` — from the verified layout, `new CustomEvent('tick')` had children `[Value("CustomEvent"), Literal("tick")]`, so zero-arg is `len() == 1`.)

- [ ] **Step 6: Declarator construction lane + promotion + provenance recording**

In `lower.rs`'s declarator handling (the Task 5 promotion site, ~667-680):
1. Extend the side-effecting-init promotion so a declarator whose init `is_event_target_new`-shaped (structural check at lower level: text-less node, single child `Value("EventTarget")` — lower.rs has no emitter; mirror the structural part, shadowing is enforced at emit) forces a REAL local (never fold-aliased — the Task 5 `matches!` chain precedent).
2. Where declarators record `fn_valued_locals` (the declarator-emit provenance site in the emitter), record the binding name into `event_target_locals` when the init is `is_event_target_new` (full check incl. shadowing). Reassignment handling: confirm the existing `unstable_provenance_names` mechanism covers any reassigned binding regardless of lane (it does for fn provenance — grep where names enter it); if it is fn-specific, add reassigned event-target names to it at the same site.

Emit arm for the construction itself (in the emitter's New/Value emit path — find where the placeholder fallback for construction fires, the E3100 warn lane): BEFORE the fallback, if `is_event_target_new(node)`:
- in a recorded declarator-init position: emit `Call(event_target_new_import_index)` (fail closed E5506 "probe/emit desync" if None, the Task 4 pattern) producing the i64 handle for the declarator's local store;
- in ANY other position: E5506 `"a 'new EventTarget()' must be bound by a declarator (const t = new EventTarget()); an unbound handle has no stable provenance"`.

How the emitter knows it's in a declarator-init position: pass/inspect the same signal the fn_valued_locals declarator-emit uses (the declarator emit calls into expression emit with the init node — hook the SAME place fn_valued_locals is recorded, where the init node identity is known). If the existing structure resolves declarator inits by node-id comparison, use that; do not invent a parallel mechanism.

- [ ] **Step 7: Handle-escape choke point**

At the identifier-read resolution point (the Spec-4a allowlist precedent — grep `resolve_identifier` / the for-in-key deny site in the emitter): if the name is in `event_target_locals` and the read is NOT one of the allowed positions (lane-method receiver — which Task 4's emit arms consume directly without routing through the generic identifier lane; declarator init RHS never reaches identifier-read), emit E5506:

```rust
            if self.event_target_locals.contains(name) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "'{name}' holds an EventTarget handle, which may only be used as the \
                         receiver of addEventListener/dispatchEvent in the current phase; any \
                         other use would leak the internal handle representation"
                    ),
                ));
                return EmittedValue { produced: false, shape: ValueShape::Unknown };
            }
```

Because Task 4's emit arms read the receiver via a direct local/global access (not the generic identifier lane), this deny is TOTAL at the generic lane — the allowlist is structural, by construction (the Spec-4a lesson). Verify that claim when wiring Task 4 (if the arms DO route receiver loads through the generic lane, thread an explicit allow-flag instead — do not weaken the deny).

- [ ] **Step 8: Run the pins + suite**

```bash
cargo build -p kali_cli 2>&1 | tail -3 && rm -rf .kali-cache
cargo test -p kali_cli --test soundness_events 2>&1 | tail -6
cargo test -p kali_cli --test soundness_closures 2>&1 | tail -3
```
Expected: all 4 events pins pass (construction runs, escape/reassign/non-declarator deny); closures suite untouched. NOTE: after this task `t.addEventListener(...)` still takes the backstop (capturing → E5506) — the 4 build tests stay red until Task 4.

- [ ] **Step 9: Full gate** (`stageD-post-ev3.txt`): newly-red = exactly the 4 inherited entries. **Risk to check in the diff:** corpus fixtures with module-scope `const target = new EventTarget()` now take the construction lane + escape discipline — if any corpus test goes newly-red from an escape-deny (e.g. a fixture that logs the target), that fixture's use is OUT of the approved envelope: record it, and adjust the choke point ONLY if the gate demands it, preferring a pin-preserving deny message (never widening the allowed positions silently). STOP and investigate any unexplained entry.

- [ ] **Step 10: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/soundness_events.rs
git commit -m "feat(codegen): EventTarget construction lane — types 15-17, conditional imports, declarator provenance, handle-escape choke point (evD) [stageD]"
```

---

### Task 4: Emit arms — addEventListener + dispatchEvent, env_safety edge, full e2e suite, pin flips

**Files:**
- Modify: `crates/kali_codegen/src/emit/call.rs` (2 emit arms before the backstop; zero-param gate)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (receiver/argument validators)
- Modify: `crates/kali_codegen/src/env_safety.rs` (member-aware registration edge)
- Modify: `crates/kali_codegen/src/lower.rs` (only if the string-literal ptr/len emit needs plumbing)
- Test: `crates/kali_cli/tests/soundness_events.rs` (e2e + envelope pins), `crates/kali_cli/tests/soundness_closures.rs` (row q flip + module-scope boundary pin)

**Interfaces:**
- Consumes: Task 3's fields/recognizer; Task 1's import semantics; `scheduling_callback()` (Task 4/5, `intrinsics/host.rs`) for callback provenance; the emitter's existing string-literal ptr/len emission (locate: grep how string args reach `crypto_subtle_digest`/`console_log` imports in `emit/` and reuse that helper).
- Produces: the lane end-to-end; the 4 build tests' E5506 gone.

- [ ] **Step 1: Write the failing e2e tests** (node-verify EVERY expected output first)

Append to `soundness_events.rs`:

```rust
/// Happy path, function scope, capturing listener: dispatch is SYNCHRONOUS
/// (the mutation is visible on the line after dispatchEvent) and the return
/// value is true. node v26.5.0: "before=0\nlistener n=1\nafter=1\ndispatched\n".
#[test]
fn event_dispatch_runs_capturing_listener_synchronously() {
    let out = run_kali(
        r#"function owner() {
  const t = new EventTarget();
  let n = 0;
  t.addEventListener("tick", () => {
    n += 1;
    console.log("listener n=" + n);
  });
  console.log("before=" + n);
  const ok = t.dispatchEvent(new CustomEvent("tick"));
  console.log("after=" + n);
  if (ok) {
    console.log("dispatched");
  }
}
owner();
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "before=0\nlistener n=1\nafter=1\ndispatched\n"
    );
}

/// Two listeners fire in registration order. node v26.5.0: "a\nb\n".
#[test]
fn event_listeners_fire_in_registration_order() {
    let out = run_kali(
        r#"const t = new EventTarget();
t.addEventListener("tick", function () { console.log("a"); });
t.addEventListener("tick", function () { console.log("b"); });
t.dispatchEvent(new CustomEvent("tick"));
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
}

/// The same function registered twice fires once per dispatch (identity
/// dedup). node v26.5.0: "hit\n".
#[test]
fn event_duplicate_listener_dedups() {
    let out = run_kali(
        r#"const t = new EventTarget();
function onTick() {
  console.log("hit");
}
t.addEventListener("tick", onTick);
t.addEventListener("tick", onTick);
t.dispatchEvent(new CustomEvent("tick"));
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hit\n");
}

/// Dispatch with zero listeners returns true. node v26.5.0: "ok\n".
#[test]
fn event_dispatch_with_no_listeners_returns_true() {
    let out = run_kali(
        r#"const t = new EventTarget();
if (t.dispatchEvent(new CustomEvent("none"))) {
  console.log("ok");
}
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok\n");
}
```

Module-scope receiver note: the last three fixtures use a module-scope `const t` read at module scope — same-scope reads, in-lane. Listener-internal receiver reads (a listener re-dispatching or adding listeners) are OUT of envelope this stage (captured receiver — the choke point denies them): snapshot-during-dispatch and recursion semantics are pinned at the WAT level (Task 1) instead, and the deny is pinned below.

Envelope pins (each expects `!success` + E5506 in stderr — same assertion shape as `event_target_handle_escape_fails_closed`; one test each, fixtures):

```text
event_non_literal_event_name_fails_closed:        const t = new EventTarget(); let name = "tick"; t.addEventListener(name, function () {});
event_listener_with_parameter_fails_closed:       const t = new EventTarget(); t.addEventListener("tick", (e) => { console.log("x"); });
event_listener_options_arg_fails_closed:          const t = new EventTarget(); t.addEventListener("tick", function () {}, true);
event_custom_event_with_detail_fails_closed:      const t = new EventTarget(); t.addEventListener("tick", function () {}); t.dispatchEvent(new CustomEvent("tick", { detail: 1 }));
event_bound_event_argument_fails_closed:          const t = new EventTarget(); const ev = new CustomEvent("tick"); t.dispatchEvent(ev);
event_remove_event_listener_fails_closed:         const t = new EventTarget(); function f() {} t.addEventListener("tick", f); t.removeEventListener("tick", f);
event_alias_callback_fails_closed:                const t = new EventTarget(); function f() { console.log("x"); } const g = f; t.addEventListener("tick", g);  // alias — x/y/z rule  [VERIFY: if the fn_valued_locals declarator lane makes `const g = f` RESOLVED provenance, this is in-lane and RUNS — check scheduling_callback's actual treatment of a fn-alias declarator and pin whichever the resolver soundly produces: Resolved → node-verified run, or Deny → E5506. Do not force Deny if provenance is genuinely stable.]
event_captured_receiver_fails_closed:             function outer() { const t = new EventTarget(); function inner() { t.dispatchEvent(new CustomEvent("tick")); } inner(); } outer();
```

(`event_bound_event_argument_fails_closed`'s `const ev = new CustomEvent("tick")` also exercises that CustomEvent construction outside a dispatch argument keeps its PRE-EXISTING behavior or denies — assert only on the overall failure + E5506, not the specific site.)

Out-of-lane preservation pin (guards the corpus — build MUST still succeed):

```rust
/// Out-of-lane receivers keep pre-lane behavior this stage (spec §2 out-of-lane):
/// a NON-capturing listener on an unknown receiver still warns + builds
/// (pre-existing residual, inventoried for Stage P3 — do NOT convert to deny here).
#[test]
fn event_unknown_receiver_non_capturing_listener_still_builds() {
    let out = run_kali(
        r#"const obj = { addEventListener: 0 };
console.log("built");
"#,
    );
    // Placeholder: replace with the ACTUAL out-of-lane corpus shape — read
    // one package_corpus addEventListener fixture (grep addEventListener in
    // crates/kali_cli/tests/package_corpus*) and pin a minimal equivalent of
    // ITS receiver shape (e.g. an AbortSignal-like unknown object), asserting
    // build success. The load-bearing property: the backstop lane did not widen.
    assert!(out.status.success());
}
```

**Replace that placeholder fixture during implementation** as its comment instructs (the corpus grep gives the real shape; the test must exercise a member `addEventListener` call on a non-EventTarget receiver with a non-capturing callback and assert build success).

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test -p kali_cli --test soundness_events 2>&1 | tail -10
```
Expected: happy-path tests fail (capturing → backstop E5506; non-capturing → silent no-op prints wrong output); envelope pins fail (fixtures currently BUILD via placeholders → exit 0).

- [ ] **Step 3: Receiver/argument validators in `intrinsics/host.rs`**

```rust
    /// Resolve a member-call receiver to an event-target local with stable
    /// provenance: callee children[0] is a bare Value identifier recorded in
    /// event_target_locals and not since made unstable. Returns the binding
    /// name for the emit arm's local/global load.
    pub(crate) fn event_target_receiver<'a>(&self, callee_node: &'a LirNode) -> Option<&'a str> {
        let &receiver = callee_node.children.first()?;
        let receiver_node = self.node(receiver);
        if !receiver_node.children.is_empty() {
            return None;
        }
        let name = receiver_node.text.as_deref()?;
        if self.unstable_provenance_names.contains(name) {
            return None;
        }
        self.event_target_locals.contains(name).then_some(name)
    }

    /// Validate a dispatchEvent argument: an INLINE `new CustomEvent(<lit>)`
    /// with an unshadowed CustomEvent and exactly one literal arg. Returns
    /// the event-type literal text.
    pub(crate) fn event_dispatch_literal<'a>(&self, node: &'a LirNode) -> Option<&'a str> {
        // Shape: Value, no text, children = [Value("CustomEvent"), Literal(type)].
        if node.text.is_some() || node.children.len() != 2 {
            return None;
        }
        let ctor = self.node(node.children[0]);
        if ctor.text.as_deref() != Some("CustomEvent") || !ctor.children.is_empty() {
            return None;
        }
        if self.locals.contains_key("CustomEvent")
            || self.bindings.contains_key("CustomEvent")
            || self.module_binding_names.contains("CustomEvent")
            || self.fn_valued_locals.contains_key("CustomEvent")
            || self.functions.contains_key("CustomEvent")
        {
            return None;
        }
        let arg = self.node(node.children[1]);
        if arg.kind != LirNodeKind::Literal || !arg.children.is_empty() {
            return None;
        }
        arg.text.as_deref()
    }
```

(Match lifetimes/accessors to the file's actual idiom — `self.node` vs indexing; adjust `LirNodeKind::Literal` to the real literal kind name used elsewhere, e.g. how `parse_numeric_literal_value` call sites check literal-ness. Apply `unwrap_transparent` to the argument before validation, mirroring `scheduling_callback`.)

- [ ] **Step 4: Emit arms in `emit/call.rs`**

Immediately after the `try_emit_scheduling_call` arm insertion point, add:

```rust
        if callee_node.text.as_deref() == Some("addEventListener")
            && !callee_node.children.is_empty()
        {
            if let Some(receiver) = self.event_target_receiver(&callee_node) {
                let receiver = receiver.to_string();
                return self.emit_event_listener_add(function, node, &receiver);
            }
            // No provable EventTarget receiver → fall through to the Stage C
            // backstop unchanged (out-of-lane behavior preserved this stage).
        }
        if callee_node.text.as_deref() == Some("dispatchEvent")
            && !callee_node.children.is_empty()
        {
            if let Some(receiver) = self.event_target_receiver(&callee_node) {
                let receiver = receiver.to_string();
                return self.emit_event_dispatch(function, node, &receiver);
            }
        }
```

and the two emit functions (near the timer emit fns, same fail-closed style):

```rust
    fn emit_event_listener_add(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        receiver: &str,
    ) -> EmittedValue {
        let fail_closed = |this: &mut Self, message: String| {
            this.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            EmittedValue { produced: false, shape: ValueShape::Unknown }
        };
        // node.children: [callee, name, callback] — exactly 2 args (an
        // options/once/capture third argument has no lowering — spec §2.2).
        if node.children.len() != 3 {
            return fail_closed(
                self,
                "addEventListener supports exactly (type, listener) in the current phase; \
                 options have no lowering"
                    .to_string(),
            );
        }
        // Event type: a string literal only.
        let Some(event_type) = self.string_literal_text(node.children[1]) else {
            return fail_closed(
                self,
                "an addEventListener event type must be a string literal in the current phase"
                    .to_string(),
            );
        };
        let event_type = event_type.to_string();
        // Callback: the scheduling-lane provenance resolver (Task 4/5 rules).
        let callback_index = match self.scheduling_callback(node) {
            SchedulingCallback::Resolved(index) => index,
            SchedulingCallback::Deny => {
                return fail_closed(
                    self,
                    "an addEventListener listener must resolve through stable provenance to a \
                     compiled function; an unresolvable listener would be silently dropped"
                        .to_string(),
                )
            }
        };
        // Zero-parameter listeners only (no Event-object repr yet — spec §2.2).
        if self.function_param_count_by_index(callback_index) != Some(0) {
            return fail_closed(
                self,
                "an addEventListener listener must declare zero parameters in the current \
                 phase (there is no Event-object lowering yet)"
                    .to_string(),
            );
        }
        let Some(import) = self.event_listener_add_import_index else {
            return fail_closed(self, "event_listener_add import unavailable (probe/emit desync)".to_string());
        };
        self.emit_event_target_handle_load(function, receiver);          // i64 handle
        self.emit_string_literal_ptr_len(function, &event_type);          // i32 ptr, i32 len
        function.instruction(&Instruction::I32Const(callback_index as i32));
        function.instruction(&Instruction::GlobalGet(self.current_env_global()));
        function.instruction(&Instruction::Call(import));
        EmittedValue { produced: false, shape: ValueShape::Unknown }
    }

    fn emit_event_dispatch(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        receiver: &str,
    ) -> EmittedValue {
        let fail_closed = |this: &mut Self, message: String| {
            this.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            EmittedValue { produced: false, shape: ValueShape::Unknown }
        };
        // node.children: [callee, event] — exactly 1 arg, an INLINE
        // `new CustomEvent(<string literal>)` (spec §2.3).
        if node.children.len() != 2 {
            return fail_closed(
                self,
                "dispatchEvent takes exactly one event argument".to_string(),
            );
        }
        let event_arg = self.unwrap_transparent(node.children[1]);
        let Some(event_type) = self.event_dispatch_literal(self.node(event_arg)) else {
            return fail_closed(
                self,
                "a dispatchEvent argument must be an inline `new CustomEvent(<string literal>)` \
                 in the current phase (bound events, detail, and options have no lowering)"
                    .to_string(),
            );
        };
        let event_type = event_type.to_string();
        let Some(import) = self.event_dispatch_import_index else {
            return fail_closed(self, "event_dispatch import unavailable (probe/emit desync)".to_string());
        };
        self.emit_event_target_handle_load(function, receiver);          // i64 handle
        self.emit_string_literal_ptr_len(function, &event_type);          // i32 ptr, i32 len
        function.instruction(&Instruction::Call(import));
        // Host returns i32 1 (true); kali booleans/scalars are i64.
        function.instruction(&Instruction::I64ExtendI32S);
        EmittedValue { produced: true, shape: ValueShape::Unknown }
    }
```

Helper notes (resolve each against the live file; report BLOCKED rather than inventing a mechanism if one is absent):
- `self.string_literal_text(id)`: fetch node, `unwrap_transparent`, return `Some(text)` iff it is a string-Literal node — mirror however `scheduling_call_args_provably_safe` or the console.log lane distinguishes string literals; write the small helper if none exists.
- `self.emit_string_literal_ptr_len(function, s)`: the SAME mechanism the codebase uses to pass literal strings to host imports (crypto digest / console paths) — find it by grepping the digest emit; it emits a data-segment reference as (ptr, len) i32 pair. If the existing helper returns a handle instead of ptr/len, follow how `read_guest_string`-consuming imports get their args (there is at least one — args flow to `console_log`-family imports; trace one end-to-end).
- `self.emit_event_target_handle_load(function, name)`: emit the local/global read for the recorded binding DIRECTLY (local.get / global.get per binding class — mirror how `fn_valued_locals` bindings or ordinary scalar locals are loaded), bypassing the generic identifier lane (the choke point denies it there — that is by design; confirm the bypass truly bypasses).
- `self.function_param_count_by_index(index)`: param count for a compiled function by wasm index — derive from the function's registered type/plan (grep how lower.rs records each `__kali_fn_N`'s param list when building its wasm type; expose the count via a map threaded like `self.functions` if not already available). Deny unless exactly `Some(0)`.
- `scheduling_callback` expects the callback at `node.children.get(1)` (bare-call layout); the member layout has it at children[2]. Add a thin wrapper or a `callback_child_index` parameter — do NOT duplicate the resolver logic (DRY); the Task 7 default-deny tail must apply identically.

- [ ] **Step 5: env_safety member edge**

In `env_safety.rs`, next to `is_scheduling_registration_callee`:

```rust
/// True when `callee_id` is a MEMBER callee named `addEventListener` (it has
/// a receiver child). The callback argument (`children[2]`) is registered for
/// later host-driven invocation with the env active at the registration site,
/// so it inherits the Record(owner) requirement (the Kali.test precedent).
/// Receiver provenance is deliberately ignored here: a spurious edge from an
/// out-of-lane receiver is a safe over-approximation (reject-only analysis).
fn is_event_registration_callee(nodes: &[LirNode], callee: LirNodeId) -> bool {
    nodes.get(callee.0 as usize).is_some_and(|node| {
        !node.children.is_empty() && node.text.as_deref() == Some("addEventListener")
    })
}
```

and extend the `target_root` selection chain:

```rust
                    } else if is_event_registration_callee(&lir.nodes, callee) {
                        node.children.get(2).copied()
                    }
```

(placed after the `is_scheduling_registration_callee` branch, before the fallback).

- [ ] **Step 6: Row q flip + module-scope boundary pin in `soundness_closures.rs`**

- `deferred_add_event_listener_capturing_callback_fails_closed` (row q): read its fixture. If its receiver is a declarator-bound `new EventTarget()`, the callback now registers (and fires only if the fixture dispatches). Run the fixture through node AND the fresh binary; rename to `..._now_registers` (or `_now_runs` if it dispatches) and assert the exact node-parity stdout. If its receiver is NOT in-lane (unknown object), the pin stays E5506 UNCHANGED — verify and document which.
- Add the module-scope boundary pin the Stage C suite lacks (node-verify first):

```rust
/// Boundary pin: module-scope capturing listener registers and fires via
/// dispatch (the bg-series analog for the event lane).
/// node v26.5.0: "sync=0\nev=1\n".
#[test]
fn event_module_scope_capture_listener_now_runs() {
    let out = run_kali(
        r#"let base = 0;
const t = new EventTarget();
t.addEventListener("tick", function () {
  base += 1;
  console.log("ev=" + base);
});
console.log("sync=" + base);
t.dispatchEvent(new CustomEvent("tick"));
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=0\nev=1\n");
}
```

- [ ] **Step 7: Corpus audit (deliberate-flip sweep)**

```bash
grep -rn "addEventListener" crates/kali_cli/tests/package_corpus.rs crates/kali_cli/tests/package_corpus/ | grep -v "^Binary"
```
For each fixture: classify in-lane vs out-of-lane under the new recognizers (receiver provenance + literal + callback resolvability + zero params). For every in-lane one, determine what its test asserts; if observable behavior changes (listener now fires), node-verify the fixture and update the assertion as a DELIBERATE capability flip with a doc comment (the row-q treatment). Out-of-lane fixtures must be untouched. Record the audit table in the task report.

- [ ] **Step 8: Run the full events + closures suites**

```bash
rm -rf .kali-cache && cargo build -p kali_cli 2>&1 | tail -3
cargo test -p kali_cli --test soundness_events 2>&1 | tail -6
cargo test -p kali_cli --test soundness_closures 2>&1 | tail -3
```
Expected: all pass.

- [ ] **Step 9: Full gate** (`stageD-post-ev4.txt`): newly-red must now be EMPTY or exactly whichever of the 4 build tests remain red for an out-of-scope reason (expected: EMPTY — the E5506 that failed them is gone and webBaselineSmoke's event usage is fully in-lane: function-local const receiver, literal names, zero-param capturing arrow, inline CustomEvent). If any of the 4 is still red: STOP, read its stderr, fix (that is the acceptance criterion). If a corpus test is newly red: that is a Step-7 miss — audit and flip/fix deliberately.

- [ ] **Step 10: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests
git commit -m "feat(codegen): addEventListener/dispatchEvent emit arms — receiver provenance, literal types, zero-param gate, env_safety member edge (evD) [stageD]"
```

---

### Task 5: Verification & close-out — build tests green, browser execute test, gate restored, triage

**Files:**
- Modify: `crates/kali_cli/tests/runtime_smoke/build.rs` (browser execute test only — the 4 build tests stay UNTOUCHED)
- Modify: `docs/superpowers/followups/stageD-triage.md`

**Interfaces:**
- Consumes: everything above.
- Produces: the restored Stage D gate + the close-out record consumed by Stage D Task 9.

- [ ] **Step 1: Run the 4 build tests by name — must be green UNTOUCHED**

```bash
cargo test -p kali_cli --test runtime_smoke browser_bundle_web_baseline_primitives 2>&1 | tail -6
```
Expected: 4 passed, 0 failed, zero diffs to those tests in `git diff` (acceptance criterion: green legitimately).

- [ ] **Step 2: Browser-lane execute test**

Next to the digestSmoke execute test (`build.rs` ~3775, `assert_browser_bundle_executes` precedent — read it and mirror its harness invocation exactly), add an event-lane fixture that RUNS through the bundle glue (node-verify the expected stdout first):

```rust
/// Stage D event lane, browser glue end-to-end: the bundle's JS import list
/// registers and synchronously dispatches through kaliEventListeners.
/// node v26.5.0 (same source, plain node): "before=0\nafter=1\n".
#[test]
fn browser_bundle_event_lane_executes() {
    // Fixture:
    // const t = new EventTarget();
    // let n = 0;
    // t.addEventListener("tick", function () { n += 1; });
    // console.log("before=" + n);
    // t.dispatchEvent(new CustomEvent("tick"));
    // console.log("after=" + n);
    // Build with --bundle --api browser, execute via the same harness call
    // the digestSmoke execute test uses, assert stdout "before=0\nafter=1\n".
}
```

(Write the real body by mirroring the digestSmoke execute test's helper calls verbatim — fixture file creation, build invocation, harness execution, stdout assertion.)

- [ ] **Step 3: Full gate — RESTORED** (`stageD-post-ev5.txt`):

```bash
comm -13 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-ev5.txt"   # MUST be EMPTY
comm -23 "$SCRATCH/stageD-pre.txt" "$SCRATCH/stageD-post-ev5.txt" | wc -l   # drain — expected 37; record each family
```

- [ ] **Step 4: Update the triage doc**

`docs/superpowers/followups/stageD-triage.md`: a "Task 8 resolution — event-surface lane" section: the user decision trail (re-pin rejected → full-parity → events-first), the lane envelope, the gate numbers (4 → 0 newly-red, drain 37), the corpus-audit table, and the follow-up inventory: Stage P2 structuredClone / P3 Abort (+ receiver widening + backstop→total-deny + captured-receiver support) / P4 URL+USP / P5 TextEncoder / final byte-for-byte acceptance; Event-object repr (zero-param restriction lift); removeEventListener; `preventDefault`/`cancelable`; the out-of-lane non-capturing silent-drop residual (pre-existing, now inventoried).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "test(soundness): event lane close-out — 4 web-baseline build tests green legitimately, browser execute pin, gate restored 0 newly-red (evD) [stageD]"
```

Then hand back to the Stage D plan's Task 9 (whole-stage adversarial review) — its scope now includes Tasks 2–7 of the original plan PLUS this lane (Tasks 1–5 here).

---

## Plan Self-Review (performed at write time)

1. **Spec coverage:** §2 envelope → Task 3 (construction/provenance/escape) + Task 4 (registration/dispatch/all deny pins — each §2 deny has a named test); §2 out-of-lane preservation → Task 4 Step 1 corpus-guard pin + Step 7 audit; §3 runtime → Task 1 (registry/imports/re-entrancy/snapshot/dedup/env/budget-exclusion is implicit: dispatch never touches `invocations`) + Task 2 (glue mirror); §4 codegen → Tasks 3–4 (probes/types/indices/declarator/choke point/arms/env_safety edge); §5 testing → WAT (Task 1), e2e + pins + row q + module-scope (Task 4), browser execute + 4-greens + gate + triage (Task 5). §6 risks → resolved pre-plan (header) or carry explicit verify steps (New-node shape, Task 3 Step 4a; string emit helper, Task 4 Step 4).
2. **Placeholder scan:** two deliberate verify-against-live-file instructions remain (string ptr/len emit helper name; param-count source) — each names the concrete search path, the reference implementation to mirror, and a BLOCKED escalation instead of invention. The out-of-lane corpus pin explicitly instructs replacing its sketch fixture with the real corpus shape. The browser execute test mirrors a named precedent verbatim. No TBDs.
3. **Type consistency:** `event_target_new/listener_add/dispatch` names and signatures match across Tasks 1, 2, 3, 4; `event_target_locals`/`event_target_receiver`/`event_dispatch_literal`/`emit_event_listener_add`/`emit_event_dispatch` used only as defined; type indices 15/16/17 + base 18 consistent; dedup/snapshot semantics identical in state.rs, glue, WAT tests, and e2e comments.
4. **Sequencing soundness:** glue (Task 2) precedes emit arms (Task 4) — no LinkError window; the backstop stays intact for out-of-lane receivers at every point; the 4 build tests are expected red through Task 3's gate and green from Task 4's.
