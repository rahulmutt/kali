# throw-fallout Stage 6a — Honest `await` Suspension Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `await` genuinely suspend — an async function runs to its first `await`, enqueues a continuation on the existing host microtask FIFO, and returns a promise — so kali's async ordering matches node instead of silently running async bodies to completion inline.

**Architecture:** An AST-level pass (`crates/kali_cli/src/build/async_split.rs`) rewrites each in-lane `async function` into an ordinary head function plus one continuation function per `await`, with live locals held in a heap **frame**. Frames and promises are plain `new Array(n)` values, so the entire promise runtime is **generated JavaScript** compiled by the existing resolver — no new codegen synthetics. Two small codegen/runtime additions carry it: a `__fn_index(f)` recognizer (function name → wasm index constant) and a `queue_job(fn_index, frame)` host import.

**Tech Stack:** Rust (kali_cli / kali_codegen / kali_runtime / kali_ast), wasm (wasm-encoder), wasmtime, node (as the differential oracle).

**Spec:** `docs/superpowers/specs/2026-07-14-throw-fallout-stage6a-async-suspension-design.md`
**Branch:** `soundness-batch1-pra` · **Stage base:** `9fdf180a2`

---

## Global Constraints

- **Stage-entry denominator: 731** (measured, two enumerations, `sort -u`). **Target drain ≈ 49** (27 `async_await_sequencing` + 22 `queue_microtask`) → ~682.
- **PRIMARY GATE: zero newly-red.** `comm -13 pre post` on full `cargo test --workspace --no-fail-fast` enumerations must print **nothing**. Cross-check against a **`main` worktree**, never a mid-branch baseline.
- **Enumeration MUST use `sort -u`, never `sort`.** 18 test names legitimately exist in two harness binaries each; raw `sort` makes `comm` report false newly-red.
- **`let`, never `const`, for any call-bound value in generated code.** Measured this session: `const b = f()` evaluates `f` **uses + 1** times (declaration *plus* every use) — even a single-use `const` double-evaluates. `let a = f()` evaluates exactly once. This is a live pre-existing miscompile; generated code must route around it, not rely on it being fixed.
- **Never emit a direct call to `__alloc` / any synthetic from the AST pass.** It would resolve by name but push i64 args into an `(i32)->i32` callee, producing an **invalid module**. The only AST-drivable heap route is `new Array(n)` + subscripts.
- **Fixture-authoring:** never `String(<bigint>)` (folds to `0`), never bind a call to a `const`. Return plain `Number`, call directly at the `console.log` site.
- **Every new synthetic/generated function name must be added to the test-side mirror** `SYNTHETIC_FUNCTIONS` in `crates/kali_cli/tests/runtime_smoke.rs:806` (inside `count_tag_boxing_ops`) or the census miscounts and reports a *test-side* desync as a product regression.
- **Adding a host import shifts indices.** Add `queue_job` / `queue_microtask` **conditionally**, at the END of the conditional import chain in `crates/kali_codegen/src/lower.rs`, with a `+ if uses_x { 1 } else { 0 }` term added to `function_index_offset` (`lower.rs:92`). Then **no existing import or function index moves**.
- **A new import must be mirrored in FIVE places** or instantiation fails: `crates/kali_runtime/src/browser/harness.rs:233`, `harness.rs:649`, `crates/kali_cli/src/bin/cmd_build.rs:1554`, `cmd_build.rs:1892`, and the native linker `crates/kali_runtime/src/host/imports_default.rs`.
- **No `_ =>` arm in any census/deny walk.** Every no-op arm must cite `kali_ast`/`kali_parser` `file:line` proving the node cannot carry an `await` or an async function. Mirror `deny_import_positions_expression` (`crates/kali_cli/src/build/module_link.rs:2601`).
- **Opt-in, widening lane.** The transform rewrites only shapes it provably supports; every other async function keeps **today's** (eager) behavior until Task 8 flips out-of-lane shapes to a hard `E5506`. This keeps each task's blast radius small across the 283 currently-green async fixtures.

---

## File Structure

| file | responsibility |
|---|---|
| `crates/kali_cli/src/build/async_split.rs` | **new.** The AST pass: in-lane detection, frame layout, continuation split, generated promise runtime, census + default-deny. |
| `crates/kali_cli/src/build/mod.rs` | **modify.** `pub mod async_split;` |
| `crates/kali_cli/src/build/compile.rs:646` | **modify.** Wire the pass in **after** `module_link::link_provable_module_namespaces`, **before** `monomorphize_statements`. |
| `crates/kali_codegen/src/intrinsics/host.rs` | **modify.** `__fn_index(f)` and `queueMicrotask(cb)` recognizers (mirror `is_kali_test_call` / `kali_test_callback_index` at `:748-765`). |
| `crates/kali_codegen/src/emit/call.rs` | **modify.** Emit sites for the two recognizers (mirror `:61-75`). |
| `crates/kali_codegen/src/lower.rs` | **modify.** Conditional `queue_microtask` + `queue_job` imports; `program_uses_*` probes; `function_index_offset`. |
| `crates/kali_codegen/src/lib.rs:42-74` | **modify.** Import-index constants. |
| `crates/kali_runtime/src/state.rs:36` | **modify.** Queue entry `i32` → `(i32 fn_index, i64 frame)`. |
| `crates/kali_runtime/src/host/enforce.rs:87` | **modify.** `invoke_callback` passes the frame for arity-1 targets. |
| `crates/kali_runtime/src/host/imports_default.rs` | **modify.** `queue_job` host import. |
| `crates/kali_runtime/src/execute.rs:326` | **modify.** Attribute a drain-time trap to the running test. |
| `crates/kali_runtime/src/browser/harness.rs` | **modify.** Import lists (×2) + async test settlement. |
| `crates/kali_cli/src/bin/cmd_build.rs` | **modify.** Import lists (×2). |
| `crates/kali_cli/tests/async_suspension.rs` | **new.** Exact-stdout acceptance suite, byte-compared against real `node`. |
| `docs/superpowers/followups/throw-fallout-stage6a-triage.md` | **new.** Entry snapshot, adjudications, drain table, follow-ups. |

---

## Task 1: Stage-entry triage + snapshot

**Files:**
- Create: `docs/superpowers/followups/throw-fallout-stage6a-triage.md`

**Interfaces:**
- Produces: `$SCRATCH/stage6a-pre.txt` — the canonical sorted stage-entry failing-name set (**731**), consumed by every later task's gate.

- [ ] **Step 1: Capture two independent enumerations on a fresh binary**

```bash
cd /workspace && cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-pre-run1.txt"
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-pre-run2.txt"
diff "$SCRATCH/stage6a-pre-run1.txt" "$SCRATCH/stage6a-pre-run2.txt"   # interleaving noise only
sort -u "$SCRATCH/stage6a-pre-run1.txt" "$SCRATCH/stage6a-pre-run2.txt" > "$SCRATCH/stage6a-pre.txt"
wc -l "$SCRATCH/stage6a-pre.txt"        # expect 731
```

Expected: **731**. If it differs, STOP and reconcile before writing any code — the denominator is the gate.

- [ ] **Step 2: Confirm the six async sub-families**

```bash
grep -E "async|await|promise|microtask|queue" "$SCRATCH/stage6a-pre.txt" > "$SCRATCH/stage6a-async.txt"
wc -l "$SCRATCH/stage6a-async.txt"     # expect 177
```

Expected 177 = `promise_all` 48 · `promise_race` 28 · `promise_any` 28 · `async_await_sequencing` (`await_*`) 27 · `promise_all_settled` 24 · `queue_microtask` 22.

- [ ] **Step 3: Adjudicate the `await`-inside-`try` set (spec §3.2.1)**

For every fixture whose source has an `await` lexically inside a `try` block **within an `async function`**, record each owning test's CURRENT red/green state. Known: `object_enumeration_finalization.rs` (`object_enumeration_finalization_test_source`) → `test_supports_object_enumeration_finalization_in_{js,ts}_input` are **RED**; plus browser variants.

**Decision rule:** all-red ⇒ Task 8's `E5506` deny costs no newly-red. **If any is GREEN**, denying it is a capability regression — apply the Stage-5 `typeof` rule (record the census, do **not** flip, defer) and record it in this triage doc.

- [ ] **Step 4: Record the `for await`-outside-async green list (spec §3.2.2)**

These 10 are **GREEN today** and MUST stay green — they use top-level `for await`, which stays degenerate and is neither transformed nor denied:

```
run_supports_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input
test_supports_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input
run_supports_browser_harness_for_of_array_from_iteration_in_js_input
run_supports_browser_harness_for_of_array_from_iteration_in_ts_input
run_supports_browser_harness_for_of_array_from_iteration_in_jsx_and_tsx_input
json_run_supports_browser_harness_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input
test_supports_browser_harness_for_of_array_from_iteration_in_js_input
test_supports_browser_harness_for_of_array_from_iteration_in_ts_input
test_supports_browser_harness_for_of_array_from_iteration_in_jsx_and_tsx_input
json_test_supports_browser_harness_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input
```

- [ ] **Step 5: Record the measured baseline** (re-run each; do not copy from the spec)

| probe | node | kali |
|---|---|---|
| `main(); console.log("b")` with an await inside | `a b c` | `a c b` |
| `queueMicrotask` flag before/after await | `false` / `true` | `1` / `1` |
| `await Promise.all([...])` | `len=2 v0=1 v1=2` | `len=0 v0=0 v1=0` |
| `const b = f()` used twice | `calls=1` | `calls=3` |
| `let a = f()` used twice | `calls=1` | `calls=1` |

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/followups/throw-fallout-stage6a-triage.md
git commit -m "docs(soundness): stage6a triage — entry snapshot 731, async bucket 177 [stage6a]"
```

---

## Task 2: `queueMicrotask` honest deferral (non-capturing callbacks)

Ships the smallest end-to-end honest-ordering slice with **no ABI widening** — the existing nullary `__kali_callback_<index>` dispatch and the existing host `queue_microtask` suffice for a callback that captures nothing.

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/host.rs` (recognizer, near `:748`)
- Modify: `crates/kali_codegen/src/emit/call.rs` (emit site, near `:61`)
- Modify: `crates/kali_codegen/src/lower.rs` (conditional import + `function_index_offset`)
- Modify: `crates/kali_codegen/src/lib.rs` (index constant)
- Test: `crates/kali_cli/tests/async_suspension.rs` (new)

**Interfaces:**
- Consumes: nothing.
- Produces: `queue_microtask` conditional import; `program_uses_queue_microtask` probe. Task 3 reuses the probe pattern for `queue_job`.

- [ ] **Step 1: Write the failing test**

Create `crates/kali_cli/tests/async_suspension.rs`:

```rust
use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Run `source` under both kali and a real `node`, assert kali's stdout is
/// byte-identical to node's, and return that stdout.
fn assert_matches_node(source: &str, filename: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join(filename);
    fs::write(&path, source).expect("write source");

    let kali = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    let node = Command::new("node")
        .current_dir(dir.path())
        .arg(&path)
        .output()
        .expect("run node");

    let kali_out = String::from_utf8_lossy(&kali.stdout).to_string();
    let node_out = String::from_utf8_lossy(&node.stdout).to_string();
    assert!(
        kali.status.success(),
        "kali failed\nstdout: {kali_out}\nstderr: {}",
        String::from_utf8_lossy(&kali.stderr)
    );
    assert_eq!(
        kali_out, node_out,
        "kali stdout diverged from node\nkali: {kali_out:?}\nnode: {node_out:?}"
    );
    kali_out
}

#[test]
fn queue_microtask_defers_a_non_capturing_callback() {
    // node prints `sync` then `micro`. Eager (pre-stage) kali prints them reversed.
    let out = assert_matches_node(
        r#"queueMicrotask(() => {
  console.log("micro");
});
console.log("sync");
"#,
        "main.js",
    );
    assert_eq!(out, "sync\nmicro\n");
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL — kali prints `micro\nsync\n` (the arrow is inlined and runs eagerly), node prints `sync\nmicro\n`.

- [ ] **Step 3: Add the conditional `queue_microtask` import**

In `crates/kali_codegen/src/lower.rs`, add a `program_uses_queue_microtask` probe alongside the existing `program_uses_*` probes, append the import at the **END** of the conditional chain (after `crypto_subtle_digest`), and add its term to `function_index_offset` (`lower.rs:92`):

```rust
// lower.rs, with the other conditional-import probes
let uses_queue_microtask = program_uses_queue_microtask(&lir);

// function_index_offset (lower.rs:92) — append the new term LAST
let function_index_offset = crate::FUNCTION_INDEX_OFFSET
    + if ctx.target.coverage { 1 } else { 0 }
    // ... existing conditional terms, unchanged, in import order ...
    + if uses_crypto_subtle_digest { 1 } else { 0 }
    + if uses_queue_microtask { 1 } else { 0 };

// import section — AFTER the crypto_subtle_digest conditional import
if uses_queue_microtask {
    // `queue_microtask(callback_index: i32) -> ()` — type 0 is `(i32) -> ()`,
    // the same shape `test_register` uses.
    import_section.import("kali:rt", "queue_microtask", EntityType::Function(0));
}
```

and compute its index the way `crypto_subtle_digest_import_index` is computed — relative to `COVERAGE_HIT_IMPORT_INDEX` plus the count of preceding enabled conditionals. **Do not** touch the fixed 0..=21 import indices.

- [ ] **Step 4: Add the recognizer + emit site**

`crates/kali_codegen/src/intrinsics/host.rs`, mirroring `is_kali_test_call` / `kali_test_callback_index` (`:748-765`) — this is a **name lookup**, and HIR already names arrows `__kali_fn_N` (`kali_hir/src/lowering/mod.rs:88`), so both an inline arrow and an identifier naming a function resolve:

```rust
pub(crate) fn is_queue_microtask_call(&self, callee: &LirNode) -> bool {
    callee.text.as_deref() == Some("queueMicrotask")
}

/// `queueMicrotask(cb)` → the wasm function index of `cb`.
/// children = [callee, callback]; the callback's `text` is the function name
/// (HIR names anonymous arrows `__kali_fn_N`).
pub(crate) fn queue_microtask_callback_index(&self, node: &LirNode) -> Option<u32> {
    let callback_node = node.children.get(1).copied()?;
    let callback_name = self.node(callback_node).text.as_deref()?;
    self.functions.get(callback_name).copied()
}
```

`crates/kali_codegen/src/emit/call.rs`, next to the `is_kali_test_call` arm (`:61`):

```rust
if self.is_queue_microtask_call(&callee_node) {
    let Some(index) = self.queue_microtask_callback_index(node) else {
        // Fail closed: an unresolvable callback must NOT fall through to the
        // eager/placeholder path (that is the pre-stage miscompile).
        self.error_e5506(
            "queueMicrotask requires a callback lowered as a function \
             (arrow, function expression, or a name bound to one)",
        );
        return;
    };
    let Some(import_index) = self.queue_microtask_import_index() else {
        self.error_e5506("queueMicrotask is not available in this build");
        return;
    };
    function.instruction(&Instruction::I32Const(index as i32));
    function.instruction(&Instruction::Call(import_index));
    return;
}
```

- [ ] **Step 5: Run the test**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: PASS — kali prints `sync\nmicro\n`, byte-identical to node.

- [ ] **Step 6: Adversarial re-mask probe (non-vacuity)**

Temporarily change the emit site to call the callback inline instead of enqueuing. Rebuild. The test MUST go RED (`micro\nsync\n`). Revert; confirm `git diff --stat` on the codegen files is empty. A test that stays green under this sabotage is measuring nothing.

- [ ] **Step 7: Full-workspace gate**

```bash
cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t2.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t2.txt"   # MUST print nothing
```

Expected: **empty**. `queueMicrotask` was previously inlined, so any newly-red name is a real reordering regression — adjudicate against node, do not wave through.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen crates/kali_cli/tests/async_suspension.rs
git commit -m "feat(codegen): queueMicrotask defers through the host FIFO instead of inlining [stage6a]"
```

---

## Task 3: The frame + promise runtime, and the ABI widen

Adds the two primitives the continuation split needs. No async transform yet — this task is provable on hand-written JS that mimics what the pass will generate.

**Files:**
- Modify: `crates/kali_runtime/src/state.rs:36` (queue entry), `crates/kali_runtime/src/host/enforce.rs:87` (`invoke_callback`), `crates/kali_runtime/src/host/imports_default.rs` (`queue_job`)
- Modify: `crates/kali_codegen/src/{lower.rs, lib.rs, intrinsics/host.rs, emit/call.rs}` (`__fn_index` + `queue_job`)
- Modify: `crates/kali_runtime/src/browser/harness.rs:233,649`; `crates/kali_cli/src/bin/cmd_build.rs:1554,1892` (the four JS import lists)
- Test: `crates/kali_cli/tests/async_suspension.rs`

**Interfaces:**
- Consumes: Task 2's conditional-import pattern.
- Produces:
  - `__fn_index(f)` → i64 constant = `f`'s wasm function index. Recognized in codegen; **never** emitted by users.
  - `queue_job(fn_index: i32, frame: i64)` host import → pushes `(fn_index, frame)` onto the microtask FIFO.
  - Frame/promise layout consumed by Tasks 4–7: **`new Array(n)`**, handle = i64 base, slot `k` via `fr[k]`.
    - **Promise** = `new Array(4)`: `[0]=state (0 pending, 1 fulfilled)`, `[1]=value`, `[2]=waiter fn_index (0 = none)`, `[3]=waiter frame`.
    - **Frame** = `new Array(2 + L)`: `[0]=own promise`, `[1]=awaited-value landing slot`, `[2..]=locals live across an await`.

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_cli/tests/async_suspension.rs`. This is hand-written JS in exactly the shape Task 5 will generate — it proves the primitives before any pass depends on them.

```rust
#[test]
fn a_queued_job_resumes_with_its_frame() {
    // Hand-written in the shape the async_split pass will generate:
    // a frame array is allocated, a continuation is enqueued with it, and the
    // continuation resumes AFTER the synchronous tail — reading its frame back.
    let out = assert_matches_node(
        r#"function cont(fr) {
  console.log("resumed x=" + fr[2]);
}
function start() {
  let fr = new Array(3);
  fr[2] = 41;
  __queue_job(__fn_index(cont), fr);
  console.log("started");
}
start();
console.log("tail");
"#,
        "main.js",
    );
    assert_eq!(out, "started\ntail\nresumed x=41\n");
}
```

> **Node-parity note.** `__fn_index` / `__queue_job` do not exist in node. Provide the *node* run with a 3-line prelude defining them (`globalThis.__fn_index = f => f; globalThis.__queue_job = (f, fr) => queueMicrotask(() => f(fr));`) written to a separate `node_prelude.mjs` the test prepends, so the node oracle executes the same semantics. Keep the prelude in the test file, not in the fixture kali compiles.

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL — `__fn_index` / `__queue_job` are undefined identifiers (`E3100`) under kali.

- [ ] **Step 3: Widen the host queue entry**

`crates/kali_runtime/src/state.rs:36`:

```rust
/// Pending microtask jobs: (guest function index, frame handle).
/// `frame == 0` means "no frame" — a nullary callback (plain queueMicrotask,
/// timers, Kali.test), which stays arity-0 and is invoked with no arguments.
pub pending_microtasks: VecDeque<(i32, i64)>,
```

Update `queue_microtask(&mut self, callback_id: i32)` (`:254`) to push `(callback_id, 0)`, and add:

```rust
pub(crate) fn queue_job(&mut self, callback_id: i32, frame: i64) {
    self.pending_microtasks.push_back((callback_id, frame));
}
```

Timers keep pushing frame-less callbacks, so `pending_timers` is untouched.

- [ ] **Step 4: Pass the frame in `invoke_callback`**

`crates/kali_runtime/src/host/enforce.rs:87` — it already shapes a `results` vector from the callee's type; do the symmetric thing for params:

```rust
pub(crate) fn invoke_callback(
    instance: &Instance,
    store: &mut Store<KaliHostState>,
    callback_id: i32,
    frame: i64,
) -> Result<(), Diagnostic> {
    let export_name = format!("__kali_callback_{}", callback_id);
    let callback = instance.get_func(&mut *store, &export_name).ok_or_else(|| {
        Diagnostic::error(
            e4::UNCAUGHT_ERROR as u32,
            format!("missing callback '{}'", export_name),
        )
    })?;

    // Continuations are 1-param (the frame handle, i64). Plain callbacks
    // (queueMicrotask, timers, Kali.test) are nullary. Dispatch both.
    let params: Vec<Val> = match callback.ty(&*store).params().len() {
        0 => Vec::new(),
        1 => vec![Val::I64(frame)],
        n => {
            return Err(Diagnostic::error(
                e4::UNCAUGHT_ERROR as u32,
                format!("callback '{}' has unsupported arity {}", export_name, n),
            ))
        }
    };
    let mut results: Vec<Val> = callback
        .ty(&*store)
        .results()
        .map(|ty| match ty {
            wasmtime::ValType::I32 => Val::I32(0),
            wasmtime::ValType::F32 => Val::F32(0),
            wasmtime::ValType::F64 => Val::F64(0),
            _ => Val::I64(0),
        })
        .collect();
    // ... existing call + trap mapping, now passing `&params` ...
}
```

Update `drain_event_loop` (`:23`) to destructure `(callback_id, frame)` and pass both; timer call sites pass `0`.

- [ ] **Step 5: Add the `queue_job` host import (all five mirrors)**

`crates/kali_runtime/src/host/imports_default.rs`, next to the existing `queue_microtask` (`:875`):

```rust
linker
    .func_wrap(
        "kali:rt",
        "queue_job",
        |mut caller: Caller<'_, KaliHostState>, callback_id: i32, frame: i64| -> wasmtime::Result<()> {
            caller.data_mut().queue_job(callback_id, frame);
            Ok(())
        },
    )
    .map_err(|error| host_import_error("queue_job", error))?;
```

And in **all four** JS import lists (`browser/harness.rs:233`, `:649`; `cmd_build.rs:1554`, `:1892`) — the browser lane needs its own FIFO drained at the same points the native one is:

```js
queue_job(id, frame) {{
  pendingJobs.push([Number(id), frame]);
}},
queue_microtask(id) {{
  pendingJobs.push([Number(id), 0n]);
}},
```

with the drain calling `instance.exports["__kali_callback_" + id](frame)` when the export's arity is 1, else `()`.

> **Import-sync rule.** Miss any one of these five and the browser lane fails with a LinkError, not a test failure. Grep `kali:rt` across the repo and confirm exactly five definition sites carry `queue_job`.

- [ ] **Step 6: Add the `__fn_index` + `__queue_job` codegen recognizers**

`crates/kali_codegen/src/intrinsics/host.rs`:

```rust
/// `__fn_index(f)` → the wasm function index of `f`, as an i64 constant.
/// INTERNAL: emitted only by `async_split`. A name lookup, exactly like
/// `kali_test_callback_index` (host.rs:761).
pub(crate) fn fn_index_operand(&self, node: &LirNode) -> Option<u32> {
    let arg = node.children.get(1).copied()?;
    let name = self.node(arg).text.as_deref()?;
    self.functions.get(name).copied()
}
```

`crates/kali_codegen/src/emit/call.rs`, alongside the Task-2 arm:

```rust
if callee_node.text.as_deref() == Some("__fn_index") {
    let Some(index) = self.fn_index_operand(node) else {
        self.error_e5506("__fn_index requires a direct function name operand");
        return;
    };
    function.instruction(&Instruction::I64Const(index as i64));
    return;
}
if callee_node.text.as_deref() == Some("__queue_job") {
    // args: (fn_index: i64, frame: i64) → import wants (i32, i64)
    self.emit_expression(node.children[1], function);   // fn_index
    function.instruction(&Instruction::I32WrapI64);
    self.emit_expression(node.children[2], function);   // frame handle
    let Some(import_index) = self.queue_job_import_index() else {
        self.error_e5506("__queue_job is not available in this build");
        return;
    };
    function.instruction(&Instruction::Call(import_index));
    return;
}
```

Add `queue_job` to `lower.rs`'s conditional chain exactly as Task 2 did for `queue_microtask` (probe → import → `function_index_offset` term), appended **last**.

- [ ] **Step 7: Run the test**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: PASS — `started\ntail\nresumed x=41\n`, byte-identical to node-with-prelude.

- [ ] **Step 8: Full-workspace gate**

```bash
cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t3.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t3.txt"   # MUST print nothing
```

The queue-entry widen touches every timer and `Kali.test` dispatch — a newly-red here is most likely a broken frame-less path. Expected: **empty**.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_runtime crates/kali_codegen crates/kali_cli
git commit -m "feat(runtime): frame-carrying job queue + __fn_index/__queue_job primitives [stage6a]"
```

---

## Task 4: `async_split` — the single-await head/continuation split

The first real transform. In-lane shape only: an `async function` **declaration** whose `await` operands are all `Promise.resolve(v)` or plain values, with **no** `try` region and **no** capturing arrow. Everything else keeps today's behavior (opt-in lane).

**Files:**
- Create: `crates/kali_cli/src/build/async_split.rs`
- Modify: `crates/kali_cli/src/build/mod.rs`, `crates/kali_cli/src/build/compile.rs:646`
- Test: `crates/kali_cli/tests/async_suspension.rs`

**Interfaces:**
- Consumes: Task 3's `__fn_index` / `__queue_job` and the frame/promise layout.
- Produces:
  ```rust
  /// Rewrite in-lane `async function` declarations into head + continuation
  /// functions. Out-of-lane async functions are left UNCHANGED (Task 8 denies
  /// them). Appends generated functions to `statements`.
  pub fn split_async_functions(
      statements: &mut Vec<Statement>,
      diagnostics: &mut Vec<Diagnostic>,
  );
  ```
  Generated names: `__async{N}_{fn}__k{M}` for continuations. Add every generated prefix to the `SYNTHETIC_FUNCTIONS` test mirror (`runtime_smoke.rs:806`).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn await_suspends_so_sync_tail_runs_first() {
    // THE canonical ordering probe. Eager (pre-stage) kali prints `a c b`.
    let out = assert_matches_node(
        r#"async function main() {
  console.log("a");
  await Promise.resolve();
  console.log("c");
}
main();
console.log("b");
"#,
        "main.js",
    );
    assert_eq!(out, "a\nb\nc\n");
}

#[test]
fn await_delivers_the_resolved_value_to_the_continuation() {
    let out = assert_matches_node(
        r#"async function main() {
  let x = await Promise.resolve(41);
  console.log("x=" + (x + 1));
}
main();
console.log("tail");
"#,
        "main.js",
    );
    assert_eq!(out, "tail\nx=42\n");
}
```

- [ ] **Step 2: Run and watch them fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL — kali prints `a\nc\nb\n` and `x=42\ntail\n` (bodies run inline).

- [ ] **Step 3: Implement the pass**

Create `crates/kali_cli/src/build/async_split.rs`. Structure (mirroring `module_link.rs`):

1. **`fn is_in_lane(f: &FunctionDeclaration) -> bool`** — `f.is_async && !f.generator`, no `try` statement containing an `await` (spec §3.2.1), every `AwaitExpression` operand on the allowlist (`Promise.resolve(v)` | a call | a plain value), no `await` inside a nested function/arrow body.
2. **`fn live_across_await(f) -> Vec<String>`** — locals assigned before an `await` and read after one. Conservative and sound: **every** local declared in the body. (Over-approximating costs a frame slot; under-approximating loses a value.)
3. **`fn split(f) -> Vec<Statement>`** — emit the head + continuations.

The head for `async function main() { A; let x = await E; B }`, with frame layout `[0]=promise, [1]=landing, [2..]=locals`:

```js
function main() {
  let __fr = new Array(3);              // 2 + 1 local
  let __p = new Array(4);               // promise: [state, value, waiter_fn, waiter_frame]
  __p[0] = 0;
  __fr[0] = __p;
  A;
  __await_step(E, __fn_index(__async0_main__k1), __fr);
  return __fr[0];
}
function __async0_main__k1(__fr) {
  let x = __fr[1];                      // landing slot
  B;
  __settle(__fr[0], 0);
}
```

with two **generated helper functions** (ordinary JS, emitted once per module — no codegen support needed):

```js
function __await_step(v, k, fr) {
  // In-lane operands settle synchronously, so `v` is already a value.
  // Even so we ENQUEUE, never call: the one-tick deferral IS the semantics.
  fr[1] = v;
  __queue_job(k, fr);
}
function __settle(p, v) {
  p[0] = 1;
  p[1] = v;
  if (p[2] !== 0) {                      // a waiter is parked (Task 5)
    p[3][1] = v;
    __queue_job(p[2], p[3]);
  }
}
```

**Hard rules while generating (Global Constraints):** `let`, never `const`, for every call-bound value. `new Array(n)` for every frame/promise — never a call to a synthetic. `Promise.resolve(v)` folds to `v` at AST level (it is settled by definition; `kali_types/src/static_analysis/promise.rs:11` already treats it as a pass-through).

4. Wire into `compile.rs` **after** `module_link::link_provable_module_namespaces` (`:646`) and **before** `monomorphize_statements` (`:663`):

```rust
// Async continuation-splitting (throw-fallout Stage 6a). Runs AFTER
// module_link (which owns `await import(...)` — 39 green tests depend on the
// statement form staying untouched) and BEFORE monomorphize, so the generated
// head/continuation functions participate in specialization like any other.
// No in-lane async function (the overwhelming majority of sources) is a no-op.
crate::build::async_split::split_async_functions(&mut parsed.statements, &mut diagnostics);
if has_errors(&diagnostics) {
    return Err(diagnostics);
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: PASS — `a\nb\nc\n` and `tail\nx=42\n`, both byte-identical to node.

- [ ] **Step 5: Adversarial re-mask probe**

In `__await_step`, replace the enqueue with a direct call (`k(fr)` semantics — i.e. sabotage the deferral). Rebuild. **Both** tests MUST go red with the pre-stage ordering (`a c b`). This is the single most important non-vacuity probe in the stage: it proves the suite detects the exact bug the stage exists to fix. Revert and confirm an empty diff.

- [ ] **Step 6: Full-workspace gate**

```bash
cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t4.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t4.txt"   # newly-red
comm -23 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t4.txt"   # drain
```

**This is the highest-blast-radius gate in the stage** — it reorders every in-lane async fixture. Expect drain; newly-red MUST be empty. Adjudicate any newly-red name against a real `node` run: if node agrees with kali's NEW output, the old expectation encoded kali's wrong order and the test is **re-pinned with the node evidence recorded in the triage doc**; otherwise it is a real regression and the transform is wrong.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli
git commit -m "feat(build): async_split — await suspends via a continuation + heap frame [stage6a]"
```

---

## Task 5: Awaiting an async function (promise chaining)

**Files:**
- Modify: `crates/kali_cli/src/build/async_split.rs`
- Test: `crates/kali_cli/tests/async_suspension.rs`

**Interfaces:**
- Consumes: Task 4's frame/promise layout and `__settle`.
- Produces: parking — `__await_step` handles a **pending** promise operand by recording `(k, fr)` in the awaited promise's `[2]`/`[3]` slots. `__settle` (already written in Task 4) enqueues the parked waiter.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn awaiting_an_async_function_resumes_after_its_whole_chain() {
    let out = assert_matches_node(
        r#"async function inner() {
  console.log("inner-start");
  await Promise.resolve();
  console.log("inner-end");
  return 7;
}
async function outer() {
  let v = await inner();
  console.log("outer got " + v);
}
outer();
console.log("tail");
"#,
        "main.js",
    );
    assert_eq!(out, "inner-start\ntail\ninner-end\nouter got 7\n");
}
```

That expected order is node's, and it is the whole point: `outer` must not resume until `inner`'s *entire* continuation chain has settled its promise.

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL — `outer` resumes immediately (its operand is a promise handle, and Task 4's `__await_step` treats it as a settled value, so `v` is the raw array handle, not `7`).

- [ ] **Step 3: Teach `__await_step` to park on a pending promise**

The awaited operand is now either a plain value or a promise (an in-lane async call's return). Distinguish them with a tag: make `__promise_new` set `p[0] = 0` and have the pass mark promise-typed operands **statically** — the pass knows syntactically whether the operand is a call to an in-lane async function, so it emits the parking form directly rather than branching at runtime:

```js
// operand is a call to an in-lane async function → park on its promise
function __await_promise(p, k, fr) {
  if (p[0] === 1) {          // already fulfilled
    fr[1] = p[1];
    __queue_job(k, fr);
  } else {                   // pending — park; __settle will enqueue us
    p[2] = k;
    p[3] = fr;
  }
}
```

and the head emits `__await_promise(inner(), __fn_index(__async1_outer__k1), __fr)` — note `inner()` appears **exactly once** and is **not** bound to a `const` (Global Constraints).

Also complete `__settle` so a `return R` in any continuation settles the head's promise with `R` (Task 4 stubbed it with `0`): every `return <expr>` inside an in-lane async body becomes `__settle(__fr[0], <expr>); return;`.

- [ ] **Step 4: Run the test**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: PASS — `inner-start\ntail\ninner-end\nouter got 7\n`.

- [ ] **Step 5: Full-workspace gate + commit**

```bash
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t5.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t5.txt"   # MUST print nothing
git add crates/kali_cli && git commit -m "feat(build): await an async function — park + settle the promise chain [stage6a]"
```

---

## Task 6: Capturing arrows → the `queue_microtask` family (22 names)

The 22-name fixture captures a local of the enclosing async function. The frame **is** the captured scope: an arrow inside an in-lane async function is emitted as an ordinary function taking the same frame, with captured locals rewritten to frame slots.

**Files:**
- Modify: `crates/kali_cli/src/build/async_split.rs`
- Test: `crates/kali_cli/tests/async_suspension.rs`

**Interfaces:**
- Consumes: Task 4's frame layout.
- Produces: captured locals are promoted into frame slots; an arrow passed to `queueMicrotask` inside an in-lane async function is lifted to `__async{N}_{fn}__cb{M}(__fr)` and enqueued via `__queue_job(__fn_index(...), __fr)`.

- [ ] **Step 1: Write the failing test** (this is the bucket fixture, made distinguishable)

```rust
#[test]
fn queue_microtask_callback_sees_and_mutates_the_enclosing_frame() {
    let out = assert_matches_node(
        r#"async function main() {
  let ran = 0;
  queueMicrotask(() => {
    ran = 1;
  });
  console.log("before-await ran=" + ran);
  await Promise.resolve();
  console.log("after-await ran=" + ran);
}
main();
"#,
        "main.js",
    );
    assert_eq!(out, "before-await ran=0\nafter-await ran=1\n");
}
```

Both halves are load-bearing: `ran=0` before the await proves the callback was **deferred**; `ran=1` after proves it **ran and mutated the enclosing frame**. Neither can pass under eager semantics, and neither can pass if the callback is dropped.

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL — pre-stage kali prints `before-await ran=1` (eager). After Task 2 alone it would print `ran=0` then `ran=0`, because the lifted arrow mutates its own copy, not the frame.

- [ ] **Step 3: Promote captured locals into the frame**

In `is_in_lane`, now ADMIT an async function containing an arrow that captures its locals (previously out-of-lane). In the split:
- Any local of the async function referenced inside a nested arrow is **forced into a frame slot** (union it with `live_across_await`).
- The arrow is lifted to a top-level `function __async{N}_{fn}__cb{M}(__fr) { ... }` with every captured local rewritten `name` → `__fr[slot]` (reads **and** writes).
- The `queueMicrotask(arrow)` call becomes `__queue_job(__fn_index(__async0_main__cb0), __fr)`.
- The enclosing body's reads/writes of that local must use the **same** frame slot — otherwise the arrow mutates a slot nobody reads. Rewrite both sides from one slot map; do not maintain two.

- [ ] **Step 4: Run the test**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: PASS — `before-await ran=0\nafter-await ran=1\n`.

- [ ] **Step 5: Adversarial re-mask probe**

Drop the frame write in the lifted arrow (make it assign a local instead of `__fr[slot]`). Rebuild. The test MUST go red with `after-await ran=0`. Revert; confirm empty diff.

- [ ] **Step 6: Full-workspace gate + commit**

```bash
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t6.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t6.txt"   # newly-red: MUST be empty
comm -23 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t6.txt"   # expect the 22 queue_microtask names
git add crates/kali_cli && git commit -m "feat(build): capturing arrows resolve through the async frame [stage6a]"
```

---

## Task 7: `for await` inside an async function (per-iteration suspension)

**Files:**
- Modify: `crates/kali_cli/src/build/async_split.rs`
- Test: `crates/kali_cli/tests/async_suspension.rs`

**Interfaces:**
- Consumes: Tasks 4–6.
- Produces: a `ForOfStatement` with `is_await: true` (`crates/kali_ast/src/statement.rs:136`) **inside an in-lane async function** suspends once per iteration. Outside an async function it is left **degenerate and untouched** (spec §3.2.2 — 10 green tests depend on this).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn for_await_yields_to_the_microtask_queue_each_iteration() {
    let out = assert_matches_node(
        r#"async function main() {
  let seen = 0;
  queueMicrotask(() => {
    seen = 1;
  });
  for await (const v of [10, 20]) {
    console.log("v=" + v + " seen=" + seen);
  }
}
main();
"#,
        "main.js",
    );
    // node: the first iteration's await yields, so the queued microtask runs
    // BEFORE the first body executes.
    assert_eq!(out, "v=10 seen=1\nv=20 seen=1\n");
}
```

Run this against node FIRST and pin whatever node actually prints — do not trust the expectation above without running it. If node disagrees, node wins and the assertion changes.

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL — kali runs the loop synchronously, so `seen=0` on the first iteration.

- [ ] **Step 3: Desugar `for await` into an awaited loop**

Rewrite a `ForOfStatement { is_await: true }` inside an in-lane async function into an index-driven loop whose body begins with a suspension, so each iteration goes through the same head/continuation machinery Task 4 built. The loop counter and the iterable handle are **frame slots** (they live across the suspension).

- [ ] **Step 4: Run the test, then gate + commit**

```bash
cargo test -p kali_cli --test async_suspension -- --test-threads=4     # PASS
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t7.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t7.txt"   # MUST be empty
```

**Watch the 10 top-level `for await` tests from Task 1 Step 4 specifically** — they must remain green. If any went red, the pass is transforming a `for await` outside an async function, which §3.2.2 forbids.

```bash
git add crates/kali_cli && git commit -m "feat(build): for-await suspends per iteration inside async functions [stage6a]"
```

---

## Task 8: The census and the default-deny

The soundness task. Until now the transform has been opt-in and everything out-of-lane kept its old (wrong) behavior. Now every async construct the transform did **not** rewrite becomes a hard `E5506`.

**Files:**
- Modify: `crates/kali_cli/src/build/async_split.rs`
- Test: `crates/kali_cli/tests/async_suspension.rs`

**Interfaces:**
- Consumes: Tasks 4–7's `is_in_lane`.
- Produces:
  ```rust
  /// Census EVERY AwaitExpression / async function / is_await ForOf node in the
  /// entry AST and reject any the transform did not rewrite. Exhaustive match,
  /// NO `_ =>` arm.
  fn deny_untransformed_async(statements: &[Statement], diagnostics: &mut Vec<Diagnostic>);
  ```

- [ ] **Step 1: Write the failing reject tests**

```rust
fn assert_rejects(source: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join(filename);
    fs::write(&path, source).expect("write source");
    let out = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        !out.status.success(),
        "expected a fail-closed reject, but kali exited 0\nstdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
}

#[test]
fn combinators_fail_closed_rather_than_returning_zero() {
    assert_rejects(
        r#"async function main() {
  let v = await Promise.all([Promise.resolve(1), Promise.resolve(2)]);
  console.log(v.length);
}
main();
"#,
        "main.js",
    );
}

#[test]
fn await_inside_a_try_region_fails_closed() {
    assert_rejects(
        r#"async function main() {
  try {
    await Promise.resolve();
  } catch (e) {
    console.log("caught");
  }
}
main();
"#,
        "main.js",
    );
}

#[test]
fn promise_executor_fails_closed() {
    assert_rejects(
        r#"async function main() {
  let p = new Promise((resolve) => resolve(1));
  console.log(await p);
}
main();
"#,
        "main.js",
    );
}
```

- [ ] **Step 2: Run and watch them fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL — all three currently exit 0 with a silent placeholder (`Promise.all` prints `0`).

- [ ] **Step 3: Implement the census**

Mirror `deny_import_positions_expression` (`crates/kali_cli/src/build/module_link.rs:2601`) **exactly** in shape: an exhaustive `match` over every `Expression` and `Statement` variant, **no `_ =>` arm**, every no-op arm carrying a `kali_ast` `file:line` citation proving the node cannot carry an `await`.

Reject (`E5506`) any of: an `AwaitExpression` whose operand is not on the operand allowlist (this catches all four combinators, `.then`, `new Promise`, `Promise.reject`); an `await`/`for await` inside a `try` region within an async function; an `async` arrow / method / generator; an in-lane-shaped async function the transform declined for any other reason.

**Do NOT reject** a `ForOfStatement { is_await: true }` outside an async function — spec §3.2.2, and the 10 green tests from Task 1 Step 4 prove it.

> **Why an allowlist and not a denylist.** Stage 5 spent four review rounds on a denylist of shapes; each round closed one fail-open and left a sibling, including one that silently linked the wrong module. Only "census every node, reject unless the position is on a short allowlist" stopped the leak. Same lesson, independently, in the for-in-key class. Do not re-litigate this.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: PASS — all three reject with `E5506` and a non-zero exit.

- [ ] **Step 5: Full-workspace gate — the critical one**

```bash
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t8.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t8.txt"   # newly-red: MUST be empty
```

The 128 combinator names were already red (silent `0`); they are now red honestly (`E5506`) — **same names, no newly-red**. If a *previously-green* test turns red here, the deny is over-broad: it is rejecting a shape that worked. Adjudicate against node and narrow the deny, or apply the Stage-5 decision rule and defer that sub-shape.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli
git commit -m "fix(build): default-deny every async construct outside the proven lane [stage6a]"
```

---

## Task 9: `Kali.test` async settlement + failure attribution

**Files:**
- Modify: `crates/kali_runtime/src/execute.rs:326`
- Modify: `crates/kali_runtime/src/browser/harness.rs`
- Test: `crates/kali_cli/tests/async_suspension.rs`

**Interfaces:**
- Consumes: Task 3's drain.
- Produces: a trap raised while draining a test's continuations is attributed to **that test** (`tests_failed += 1`), not turned into a whole-run abort.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn an_async_self_check_throw_fails_that_test_and_only_that_test() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(
        &path,
        r#"async function bad() {
  await Promise.resolve();
  throw new Error('self-check failed');
}
Kali.test('async self check', () => bad());
Kali.test('sync ok', () => {});
"#,
    )
    .expect("write source");

    let out = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&path)
        .output()
        .expect("run kali test");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success(), "a throwing async test must fail the run");
    // The failure must be ATTRIBUTED: 2 tests ran, exactly 1 failed.
    assert!(
        stdout.contains("FAILED 1"),
        "expected exactly one attributed failure, got: {stdout}"
    );
}
```

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test -p kali_cli --test async_suspension -- --test-threads=4`
Expected: FAIL. Baseline verified this session: with eager `await`, the throw lands inline and correctly reports `FAILED 1`. Once `await` suspends (Tasks 4–5), the throw moves into the drain and `drain_event_loop`'s error path (`execute.rs:326`) does `return Err(vec![diagnostic])` — aborting the whole run, so `FAILED 1` never appears.

- [ ] **Step 3: Attribute drain-time traps to the running test**

In `execute.rs`, inside the per-test loop, the post-callback `drain_event_loop` error must increment `tests_failed` and record the diagnostic against the current test — the same treatment the direct-callback error path already gets (`:322`) — rather than returning `Err`. Preserve the existing `take_pending_exit_code()` short-circuit (an explicit `process.exit` is not a test failure).

Mirror the same settlement in the browser harness JS: after invoking a test callback, drain the job queue, and count a trap during that drain as a failure of the test that was running. Stage 0 fixed the harness trap-swallow; do not reintroduce it through the async door.

- [ ] **Step 4: Run the test, then gate + commit**

```bash
cargo test -p kali_cli --test async_suspension -- --test-threads=4     # PASS
cargo build -p kali_cli && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-t9.txt"
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-t9.txt"   # MUST be empty
```

Watch the Stage-0 envelope tests (`browser_runtime_summary_fallback_*`, `runtime_smoke/test.rs`) — they pin the honest failed-run envelope and are the ones this change can break.

```bash
git add crates/kali_runtime && git commit -m "fix(runtime): attribute a drain-time trap to the test that was running [stage6a]"
```

---

## Task 10: Stage gate, adversarial sweep, triage doc

**Files:**
- Modify: `docs/superpowers/followups/throw-fallout-stage6a-triage.md`
- Modify: `crates/kali_cli/tests/runtime_smoke.rs:806` (`SYNTHETIC_FUNCTIONS` mirror)

- [ ] **Step 1: Sync the test-side synthetic mirror**

Add every generated-function prefix the pass emits (`__await_step`, `__await_promise`, `__settle`, and the `__async*` families) to `SYNTHETIC_FUNCTIONS` in `count_tag_boxing_ops` (`runtime_smoke.rs:806`). Stage-4 lesson: a missing entry makes the census report a **test-side desync as a product regression**, and someone will spend a day bisecting it.

- [ ] **Step 2: Two independent full-workspace enumerations + the primary gate**

```bash
cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-post-run1.txt"
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-post-run2.txt"
diff "$SCRATCH/stage6a-post-run1.txt" "$SCRATCH/stage6a-post-run2.txt"   # zero drift
sort -u "$SCRATCH/stage6a-post-run1.txt" "$SCRATCH/stage6a-post-run2.txt" > "$SCRATCH/stage6a-post.txt"

comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-post.txt"   # PRIMARY GATE: must print NOTHING
comm -23 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-post.txt"   # drain (expect ~49)
```

- [ ] **Step 3: Main-worktree cross-check**

```bash
# inside /workspace/.worktrees/kali-main (main), NOT the branch
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/main-post.txt"
comm -13 "$SCRATCH/main-post.txt" "$SCRATCH/stage6a-post.txt" | comm -13 "$SCRATCH/stage6a-pre.txt" -
```

Expected: **empty** (nothing red on-branch/green-on-main beyond what stage entry already carried).

- [ ] **Step 4: Adversarial whole-stage sweep**

Probe for silent wrongness the per-task tests would miss, on a **freshly built binary** (fix reports are not evidence — Stage-5 rule):

- an async function called but never awaited (fire-and-forget) — does its chain still drain?
- two async functions interleaved — do their frames stay separate, or does one clobber the other's slots?
- a `queueMicrotask` **inside** a continuation (a job that enqueues a job) — does the drain reach fixpoint?
- an async function with **no** `await` — still returns a promise, still settles?
- recursion: an async function awaiting itself with a base case.
- a capturing arrow **and** an `await` mutating the **same** local — one slot, or two?

Every divergence from node is either fixed or recorded in the triage doc with node-vs-kali evidence. **A doc that claims fail-closed while a fail-open is live is itself the defect.**

- [ ] **Step 5: Write the triage doc**

Record: entry 731 → exit (measured); the drain list bucketed by sub-family with per-bucket mechanism; every newly-red adjudication with its node evidence; the re-mask probe results proving non-vacuity; and the follow-up inventory — at minimum: the **four combinators** (Stage 6b), **try-region-preserving continuations** (§3.2.1), the **`for await`-outside-async residual divergence** with its 10 green test names as sizing evidence (§3.2.2), and the **`const`-bound-call double-evaluation** miscompile (measured: `uses + 1` evaluations; `let` is correct).

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/followups/throw-fallout-stage6a-triage.md crates/kali_cli/tests/runtime_smoke.rs
git commit -m "docs(soundness): stage6a checkpoint — drain, gate, adversarial sweep [stage6a]"
```

---

## Self-Review

**Spec coverage:** §2 architecture → Tasks 3–7. §2.3 ABI → Task 3. §3 scope/fail-closed → Task 8. §3.2.1 try-region deny → Task 1 Step 3 (adjudication) + Task 8. §3.2.2 `for await` residual → Task 1 Step 4 + Task 7 + Task 10 Step 5. §3.3 `Kali.test` attribution → Task 9. §3.4 module_link ordering → Task 4 Step 3. §4 blast radius → every task's full-workspace gate + Task 4 Step 6 adjudication rule. §5.1 gate → Task 10. §5.2 acceptance suite → Tasks 2/4/5/6/7. §5.3 re-mask probes → Tasks 2/4/6 + Task 10 Step 4. §5.4 fixture constraints → Global Constraints. §6 deliverables 1–8 → all covered. **No gaps.**

**Type consistency:** `assert_matches_node(&str, &str) -> String` and `assert_rejects(&str, &str)` are defined in Task 2 / Task 8 and used consistently. Frame layout `[0]=promise, [1]=landing, [2..]=locals` and promise layout `[0]=state, [1]=value, [2]=waiter_fn, [3]=waiter_frame` are fixed in Task 3 and used unchanged in 4–7. `__await_step` / `__await_promise` / `__settle` / `__queue_job` / `__fn_index` keep the same names and arities throughout. `invoke_callback` gains one `frame: i64` parameter, applied at every call site in Task 3.

**Known risk carried into execution:** Task 4's gate is the blast-radius moment — it reorders every in-lane async fixture at once. If newly-red is large, the lane in `is_in_lane` is too wide; narrow it, land the gate green, and widen in a follow-up task rather than adjudicating dozens of names under pressure.
