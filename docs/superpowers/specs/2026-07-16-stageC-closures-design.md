# Stage C — Environment-pointer closures (block-arrow prereq)

> **Design spec.** Third stage (B → A → **C** → D) of the block-arrow un-flatten
> prerequisites project. Umbrella:
> `docs/superpowers/specs/2026-07-15-blockarrow-prereqs-design.md`. Stage C was
> deferred there for its own brainstorm because it is greenfield — kali has **no
> closure model** today. This spec is the output of that brainstorm.

**Branch:** `soundness-batch1-pra` · **Baseline commit:** `192984c39` ·
**Frozen failure baseline:** **731** tests · **Develop patch-free on the clean
branch** (the un-flatten WIP patch is applied only in Stage D).

---

## 1. The gap

kali codegen resolves every name through a per-function `self.locals`
(`crates/kali_codegen/src/emitter.rs:99`, `name → wasm local index`) plus
module-scope WASM globals (`module_global_slots`). A callback that references a
binding declared in an **enclosing function scope** finds it in neither, and
today that produces **two** distinct wrong behaviors:

- **Write path — hard E5506.** `count += 1` on an outer `count`:
  `crates/kali_codegen/src/emit/literal.rs:496` (and the update-expression twin
  `crates/kali_codegen/src/emit/operators.rs:22`) fall through to
  `E5506 FEATURE_UNAVAILABLE`. Sound but over-rejecting.
- **Read path — SILENT miscompile.** `return count` / `count !== 1` on an outer
  `count`: the identifier arm falls through to
  `crates/kali_codegen/src/emit/control_flow.rs:1304`
  (`push_placeholder_fallback_diagnostic`, `emitter.rs:529`) which emits
  `I64Const(0)` and a mere **warning** (`e3::UNDEFINED_IDENTIFIER`), not E5506.
  A closure that reads an outer local silently sees `0`. This is a
  reject-don't-miscompile violation and closing it is the soundness core of the
  stage.

The only cross-boundary sharing that exists is `module_global_slots`
(`emitter.rs:200`), which is hardwired to **module** scope and scalar-only — not a
closure mechanism.

`kali_mir` **already computes the capture set** — free-variable detection at
`crates/kali_mir/src/analysis/walk.rs:381` populates `MirBinding.captured_by`
(`crates/kali_mir/src/binding.rs:16`) and seeds
`LayoutDescriptor::Closure { captures }`
(`crates/kali_mir/src/layout.rs:14`). But that data dead-ends: it is consumed
only to force captured bindings to `OwnershipClass::SharedHeap`
(`analysis/mod.rs:119`) and into the **never-reset global arena**
(`analysis/arena_gate.rs:355`). **No env record is ever materialized and codegen
never reads the capture set.**

### 1.1 Reframe (load-bearing)

The gap is **not** a property of block-arrow parsing. It reproduces identically on
the clean branch with plain `function(){}` expressions and IIFEs — same write
E5506, same silent read-zero. It is a pre-existing soundness hole in how kali
compiles **any** real function body that closes over an outer binding. Stage C is
therefore developed and gated **patch-free** on the clean branch; Stage D (the
un-flatten) only multiplies how often the closure path fires.

---

## 2. Scope (decided in the brainstorm)

| Dimension | Decision |
|---|---|
| **Capture surface** | **Full: deferred + heap.** Scalar and heap captures; synchronous and deferred callbacks. |
| **Callback consumers** | **Host-invoked callbacks only:** `Kali.test`, `setTimeout`/`setInterval`, `queueMicrotask`, `addEventListener`. Array per-element callbacks (`arr.map(x => x + base)`) have **no callback ABI today** (only identity-shaped map/filter are folded — `crates/kali_codegen/src/intrinsics/array.rs:220`); they **stay E5506** and become their **own follow-up stage**. |
| **Nesting depth** | **Full env chains** — each env record carries a `parent_env_ptr`; a capture resolves by walking the chain N levels. |
| **Reclamation** | **Controlled leak into the never-reset `__alloc_global` region** (g4/g5/g6). This matches today's `arena_gate` behavior for captured bindings; it is not a new leak. A *reclaimable* escaping-capture region is a **follow-up**, not this stage. Honors the GC-less invariant (region/escape only; never a tracing GC). |

**Non-goals:** first-class closure values / a WASM function table / `call_indirect`
(not needed — see §3); `arguments`, `this`-capture, `eval`, `with`, generator
capture; the array per-element callback ABI; a reclaimable capture region; the
un-flatten patch (Stage D).

---

## 3. Architecture

### 3.1 The closure-value model

A closure is the pair **`(function_index, env_ptr)`**. kali has no first-class
function values and no function table, and needs neither, because both sites a
closure is *used* know the pair statically:

- a **direct call** of a nested function (`inc()`), and
- a **callback argument** to a host-scheduled consumer
  (`addEventListener(t, cb)`, `queueMicrotask(cb)`, `setTimeout(cb, d)`,
  `Kali.test(name, cb)`).

The pair is threaded **explicitly only at the deferred boundary** — `env_ptr` is
passed to the host schedule import and the host sets `current_env` before the
existing nullary `__kali_callback_<idx>` call
(`crates/kali_runtime/src/host/enforce.rs:100-120`). For a **synchronous direct
call** no pair is threaded: `current_env` at the call site is already the callee's
lexical parent, and the callee's own prologue extends the chain if it owns
captures (§3.4). So "invoking a closure" is uniformly "run with `current_env` set
to the lexical-parent env" — set by the host for deferred, already-in-place for
direct.

Net-new machinery is small: **one WASM global** (`current_env`), **one extra
`i64 env_ptr` argument** on the four scheduling imports, and env load/store
lowering. No `call_indirect`, no function table, no first-class closures.

### 3.2 Env record layout & reclamation

Layout in linear memory:

```
env_ptr → [ parent_env_ptr : i64 ][ cell_0 ][ cell_1 ] … [ cell_{k-1} ]
```

- One record **per activation** of any function that owns ≥1 **captured** binding
  (a local some nested function closes over).
- Cells are the captured bindings in a **MIR-fixed order**. A scalar cell holds
  the i64/f64 value; a heap cell holds the object/string/array pointer.
- `parent_env_ptr` links to the enclosing activation's env — the **env chain**.
- **Allocation at function entry**, `parent = current_env` at entry; the body
  then runs with `current_env` = the new record. Uniform for sync and deferred:
  a deferred callback is entered by the host with `current_env` = the `env_ptr`
  captured at schedule time, so its own entry-allocation chains onto the
  still-live parent.
- **Region: `__alloc_global`** (never reset, g4/g5/g6 —
  `crates/kali_codegen/src/lower.rs:951`, accessor `emitter.rs:403`). Because env
  records never reset, the chain stays valid after a parent activation returns —
  which is exactly what makes deferred callbacks and recursion correct (each
  activation gets a fresh, distinct env; a single global cell would conflate
  activations). `arena_gate.rs:355` already forces captured bindings here, so
  this is not a new leak — it becomes *load-bearing* rather than merely
  conservative.

### 3.3 Variable promotion & access resolution (correctness crux)

JS closures capture **variables, not values**: `count += 1` in the listener must
mutate the same `count` the enclosing scope reads afterward. So a captured
mutable binding *is* the env cell, and **every** access — owner and closure — is
routed to that one cell.

- **Promotion (MIR-decided, not heuristic):** any binding `captured_by` a nested
  function (`walk.rs:381`) is promoted from a plain WASM local to a cell in its
  owning function's env record. Forced by semantics.
- **Access resolution** — a name in a function body resolves to exactly one of:
  1. **own uncaptured local/param** → unchanged (`LocalGet`/`LocalSet`);
  2. **own captured cell** → `current_env + offset` load/store;
  3. **captured from an outer scope** → walk `parent_env` *k* times, then
     `+ offset`; `k` (nesting distance) and `offset` are statically known from
     MIR scope analysis;
  4. **module-scope binding** → unchanged (`module_global_slots` WASM globals —
     module scope is *not* a closure capture; already works; this is the simpler
     path exercised by the non-test-mode variant of the headline test).

This closes **both** §1 fail sites: the write path (`literal.rs:496` /
`operators.rs:22`) and the silent read path (`control_flow.rs:1304`). Heap
captures use the same cells (cell holds the pointer; the pointee is already
SharedHeap-in-global-region, so it survives the deferred boundary).

### 3.4 Env-pointer threading

`current_env` is a **new WASM global**, a structural mirror of the g1/g2/g3
current-arena trio (already saved-to-locals and restored around dynamic scopes —
`crates/kali_codegen/src/emit/control_flow.rs:247-281, 565-593` — the exact
pattern reused here).

- **Function prologue/epilogue (the only place `current_env` is mutated for sync
  code):** if the function owns captures, the prologue saves the incoming
  `current_env` to a local, allocates its env (`parent = incoming`), and sets
  `current_env` to it; the epilogue restores the saved value on every exit path.
  A function that captures outer bindings but owns none of its own allocates
  nothing and leaves `current_env` untouched — it reads through the inherited
  `current_env` directly.
- **Synchronous direct call** of a nested function needs **nothing at the call
  site**: `current_env` is already the caller's env (= the callee's lexical
  parent), and the callee's prologue extends the chain. Correct under recursion
  because each activation's prologue allocates a distinct record.
- **Deferred callbacks (host boundary):** the four scheduling paths —
  `setTimeout`/`setInterval` (`crates/kali_runtime/src/host/imports_default.rs:815`),
  `queueMicrotask` (`:878`), `addEventListener`
  (`crates/kali_runtime/src/host/imports_node.rs:563`), and the `Kali.test`
  register path (`imports_default.rs:250`) — each gain a stored **`env_ptr` =
  `current_env` at schedule time**, alongside the `callback_id`.
  `invoke_callback` (`enforce.rs:87`) sets the `current_env` global (via
  `instance.get_global`) to the stored `env_ptr` immediately before the nullary
  `callback.call(&[], …)`, and restores it after. `drain_event_loop`
  (`enforce.rs:23`) is otherwise unchanged. That — one stored field + one
  `global.set` before invoke — is the entire deferred mechanism.

---

## 4. Fail-closed boundaries

Reject-don't-miscompile. Everything the analysis cannot lower soundly emits
**E5506**, never a silent wrong answer:

- a capture MIR can't lay out (unknown cell repr; a binding not proven
  scalar-or-heap-pointer) → E5506 at closure creation;
- `arguments`, `this`-capture, `eval`, `with`, generator/`yield` capture → E5506;
- array per-element callbacks with captures → E5506 (the follow-up stage);
- the **F-AB-2 exotic unseeded positions** — a fn-expr inside an object literal
  passed directly as a call arg, a spread arg, a tagged-template / `yield` /
  optional-chain operand, a bare or doubly-nested array literal (documented at the
  `repr_infer.rs` `visit_expr` `_` arm tripwire). Stage AB made seeding-or-failing
  these a **hard prerequisite** before Stage C makes any of them *invocable*. This
  design **fails them closed (E5506)** rather than seeding them — smaller and
  sound.

### 4.1 F-AB-2 lockstep assertion (required)

Add the mechanical assertion F-AB-2 demanded: assert that the `__kali_fn_N` set
discovered by `repr_infer` walks 1–3 **equals** the set walk 4 seeds, so a future
divergence trips a test rather than a silent i64. (See
`docs/superpowers/followups/stageAB-followups.md` §F-AB-2.)

---

## 5. MIR → codegen bridge

The capture analysis exists but dead-ends at ownership classification (§1).
Stage C threads it to codegen:

- **Surface** `MirBinding.captured_by` + `LayoutDescriptor::Closure { captures }`
  into a per-function **env plan**: for each function, (a) which of its own locals
  are promoted (the cells + fixed order/offsets), and (b) which outer bindings it
  captures and at what `(k, offset)`.
- **Keying:** everything keys on the **Task-2 synthetic name `__kali_fn_N`**
  already assigned before the resolver — the same one name read by `kali_types`,
  `kali_hir`, and (post-Stage-A) `repr_infer`. **No hand-mirrored oracle** — this
  repo has shipped two mirrored oracles and both failed *open*.
- **`arena_gate.rs:355` interaction:** the "captured ⇒ never-reset global arena"
  rule stays and becomes load-bearing (env records depend on that region never
  resetting) rather than merely conservative.

---

## 6. Success criteria

- A callback reading **and** mutating an enclosing function-scope **scalar** local
  compiles and runs correctly vs node — the `web_baseline_primitives`
  `count += 1` shape
  (`crates/kali_cli/tests/runtime_smoke.rs:444-449`, test-mode `:426`).
- A callback capturing an enclosing **heap** object runs correctly — the same
  test's `controller.abort()`.
- An **env chain** works — a closure-in-closure reading a grandparent local.
- **Recursion/re-entrancy** — a function creating a closure per activation proves
  distinct envs (a single global cell would fail this).
- The **read-path silent miscompile is closed** — `return count` from a closure
  returns the value, not `0`.
- Cases that cannot be lowered soundly **fail closed E5506**, never silently
  wrong (§4).
- **Zero newly-red vs 731**; measure (do not forecast) any drain.

---

## 7. Testing & gating

- **Develop on the clean branch** with `function(){}` / arrow fixtures; do **not**
  apply the un-flatten WIP patch (Stage D). "Harden the sink before widening what
  feeds it."
- **Headline regression fixtures** (assert correct output vs node; go red if
  reverted): (1) function-scope scalar capture + mutation across
  `addEventListener`+`dispatchEvent`; (2) heap capture through the same closure;
  (3) an env chain (grandparent local); (4) recursion → distinct envs; (5) the
  read-path fix (`return count` → value, not 0).
- **Fail-closed fixtures:** array per-element capture, and one F-AB-2 exotic
  position — both assert E5506.
- **Adversarial re-mask probes:** null the host env `global.set` → the deferred
  fixture must break; comment out promotion → the `count` fixture must go red.
  Assert the **post-drain** value, so a probe can distinguish "captured" from
  "coincidentally zero" (the mirage this program keeps catching).
- **Primary gate: zero newly-red vs 731.** Full
  `cargo test --workspace --no-fail-fast`; enumerate with **`sort -u`** (never
  plain `sort` — 18 dual-harness dupes fabricate newly-red); `comm -13 pre post`
  must be empty; cross-check against a `main` worktree. A full run exceeds one
  command timeout — run detached with a `.done` marker and poll a bounded loop.
- **No drain claimed.** A stage that closes a fail-closed over-rejection may turn
  some red tests green — measure it, do not forecast it.

### 7.1 Implementation phasing

Each increment is sound and fails closed beyond its frontier:

- **C1** — env plan (MIR bridge) + promotion + single-level **synchronous**
  capture, **scalar only**, closing both the write and read sites.
- **C2** — heap-cell captures + **env chains** (`parent_env_ptr` walk).
- **C3** — **deferred** host-threading: `env_ptr` through the four scheduling
  imports + `invoke_callback` + `current_env` `global.set`.
- **C4** — F-AB-2 lockstep assertion, fail-closed hardening (§4), full-workspace
  gate + main-worktree cross-check.

---

## 8. Follow-ups filed by this design

- **Array per-element callback ABI** (`arr.map`/`filter`/`forEach` with a
  capturing body) — needs `call_indirect` + a WASM function table, or body
  inlining. Its own spec → plan → stage. E5506 until then.
- **Reclaimable escaping-capture region** — replace the `__alloc_global`
  controlled leak for env records with a reclaimable region. GC-less
  (region/escape only). Not required for correctness.
- **F-AB-1** (pre-existing expression-bodied-arrow return-value silent
  miscompile) remains open — orthogonal to closures; see
  `docs/superpowers/followups/stageAB-followups.md`.
- Class-method `return` silently returns `0` (Stage 6 Task 4 verdict) — carried
  into Stage D, not this stage.
