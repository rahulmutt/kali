# throw-fallout Stage 6a — honest `await` suspension (the async ordering core)

**Program:** throw-fallout (prerequisite for Soundness Batch 1 PR-A / PR #16).
**Branch:** `soundness-batch1-pra`. **Stage base:** `e16ed0a5d` (Stage 5 CERTIFIED).
**Stage-entry denominator:** **731** (measured this session, two enumerations, `sort -u`).
**Target drain:** the `async_await_sequencing` (27) + `queue_microtask` (22) names ≈ **49** → ~682.
**Bucket:** #1 async/Promise value lane. Stage 6a is the *ordering core*; **Stage 6b** (a separate
spec) implements the four combinators.

---

## 1. Why this stage exists — the measured baseline

Bucket #1 is not "partially implemented". `await` does not suspend at all, and every combinator
returns the placeholder `0`. All four probes below were run this session on a freshly built branch
binary against real `node` (v26.5.0). **Every kali cell exits 0 with no diagnostic** — these are
silent wrong answers, not unimplemented-feature rejects.

| probe | node | kali (branch, fresh binary) |
|---|---|---|
| `async function main(){ console.log("a"); await Promise.resolve(); console.log("c"); } main(); console.log("b");` | `a` `b` `c` | **`a` `c` `b`** — wrong order |
| `queueMicrotask(cb)` then read the flag before/after `await` | `false` then `true` | **`1` then `1`** — callback ran eagerly |
| `await Promise.all([Promise.resolve(1n), Promise.resolve(2n)])` | `len=2 v0=1 v1=2` | **`len=0 v0=0 v1=0`** |
| `await Promise.race([...])` / `Promise.any([...])` | `1` / `3` | **`0` / `0`** |

Mechanism, confirmed by inspecting the emitted module: `kali build p2.js` produces a wasm that
imports **no** `queue_microtask` at all. The host-side microtask FIFO exists and is drained after
`_start`, but the guest never enqueues onto it in this lane — the `queueMicrotask` arrow is
*inlined* and its body runs eagerly. `Expression::AwaitExpression` resolves straight through to its
argument (`kali_types/src/resolve/expression.rs:1984`), so `await` is a no-op.

### Current failing-set composition (731)

The async bucket is 177 names in exactly six sub-families (no unmatched remainder):

| sub-family | names | stage |
|---|---|---|
| `promise_all` | 48 | 6b |
| `promise_race` | 28 | 6b |
| `promise_any` | 28 | 6b |
| `promise_all_settled` | 24 | 6b |
| `async_await_sequencing` (`await_*`) | 27 | **6a** |
| `queue_microtask` | 22 | **6a** |

### Fixture surface census (what kali must actually support)

Across all of `crates/kali_cli/tests/`: `await` 579 · `for await` 462 · `async function` 283 ·
`Promise.resolve` 69 · `queueMicrotask` 50 · `Promise.allSettled` 27 · `setTimeout` 14 · `.then(` 9 ·
`Promise.race`/`any`/`all` 4 each · `Promise.reject` 2 · async arrow 1 · **`new Promise` — zero**.

No executor anywhere is the load-bearing fact of this design: **every promise in the proven lane is
either settled on creation (`Promise.resolve`) or owned by an in-lane async function.** That makes
the promise object trivial and removes the entire pending-external-resolution problem from scope.
Timers (`setTimeout`/`setInterval`) already work host-side and are untouched.

---

## 2. Architecture — the heap frame is the whole trick

A new **AST-level pass**, `crates/kali_cli/src/build/async_split.rs`, wired in `compile.rs` in the
same slot as `module_link` and `monomorphize_statements` — i.e. **before the resolver**. Downstream
(`kali_types`, `kali_codegen`) then sees only ordinary functions plus a few synthetic runtime calls.
This is the Stage-5 primitive applied verbatim: zero repr/codegen edits *by construction*, because
the resolver never learns that async existed.

### The transform

`async function f` becomes an ordinary function plus one continuation per `await`:

```
async function f() { A; const x = await E; B(x); return R; }
```

lowers to (schematically — real names are mangled, see §2.4):

```
function f() {                               // the head — returns a promise handle
  const __fr = __frame_alloc(N);             // heap frame: this call's live state
  __frame_set(__fr, 0, __promise_new());     // slot 0 = f's own promise
  A;                                         // (locals live across the await → frame slots)
  __promise_await(E, f__k1, __fr);           // enqueue-or-park; slot 1 = landing slot
  return __frame_get(__fr, 0);
}
function f__k1(__fr) {                       // continuation: ordinary fn, one i32 param
  const x = __frame_get(__fr, 1);            // the awaited value
  B(x);
  __promise_settle(__frame_get(__fr, 0), R); // settle f's promise → enqueue its waiter
}
```

- **`__promise_await(p, k, fr)`**: if `p` is settled, store its value into the frame's landing slot
  and **enqueue** `(k, fr)` on the host microtask FIFO; if pending, record `(k, fr)` as `p`'s waiter.
  **A settled promise still enqueues rather than calls** — that one-tick deferral is exactly what
  produces node's `a b c` ordering. Calling inline here reproduces the bug this stage exists to fix.
- **`__promise_settle(p, v)`**: mark fulfilled, store `v`, and enqueue `p`'s waiter if one is parked.
- Reaching `return` in any continuation settles the head's promise.

### 2.1 The frame collapses three problems into one

The frame is a heap struct (existing heap-object/arena lane): slot 0 = the call's own promise,
slot 1 = the awaited-value landing slot, slots 2.. = locals live across an `await`.

1. **Resumption.** A continuation is an ordinary nullary-plus-frame function; its locals survive the
   suspension because they live in the frame, not in wasm locals.
2. **Promise state.** `state` / `value` / `waiter_fn` / `waiter_frame` — a 4-field heap object.
   Trivial precisely because no fixture uses `new Promise`.
3. **Closure capture — the one that isn't obvious.** `queueMicrotask(() => { microtaskRan = true; })`
   *captures a local of the enclosing async function* and the enclosing function then reads it after
   the await. Once that local lives in the frame, the arrow compiles to an ordinary function taking
   **the same frame pointer** and writing **the same slot**. The capturing arrow and the continuation
   are the same mechanism. This is also why the long-deferred block-arrow item
   (`docs/superpowers/followups/task8-block-arrows-DEFERRED.md`, blocked on "untracked function
   scopes") is *not* a prerequisite here: inside an async function, the frame **is** the tracked scope.

### 2.2 Machinery that already exists (why this is tractable)

- **The host already runs a microtask FIFO and drains it** (`kali_runtime/src/state.rs`
  `pending_microtasks`, `host/enforce.rs::drain_event_loop`, called after `_start` in `execute.rs`).
- **Every non-entry function is already exported as `__kali_callback_<wasm_index>`**
  (`kali_codegen/src/lower.rs:774`) and the host dispatches it **by export name**
  (`host/enforce.rs::invoke_callback`). `Kali.test` already passes a callback's raw wasm index to
  `test_register`. **No `call_indirect`, no funcref table, no new dispatch mechanism is needed** —
  continuations are just functions, enqueued by index.

### 2.3 The one ABI change

Queue entries widen from `i32 callback_id` to **`(i32 fn_index, i32 frame_ptr)`**, exposed as a new
`kali:rt` host import `queue_job(fn_index, frame_ptr)`. `invoke_callback` already inspects the
target's type to shape its result vector; it gains the symmetric params handling — pass the stored
frame when the target's arity is 1, nothing when it is 0 (plain `queueMicrotask` callbacks and timer
callbacks stay nullary, so **timers are unaffected**).

> **Import-sync rule (memory: `kali-browser-harness-import-sync`).** `queue_job` must be mirrored in
> **all four** hand-written `kali:rt` import lists (`kali_runtime/src/browser/harness.rs` ×2,
> `kali_cli/src/bin/cmd_build.rs` bundle glue ×2) or the browser lane dies with a LinkError.

### 2.4 Name mangling and index resolution

Continuations are emitted as top-level functions named `__async{N}_{fn}__k{M}`. The AST pass cannot
know wasm indices, so it emits the continuation **by name**; codegen resolves name → index at emit
time using the *existing* `kali_test_callback_index` mechanism in `emit/call.rs` (the same path
`Kali.test(name, arrow)` already uses). This is the only codegen-side addition: a recognizer for the
synthetic `__promise_await` / `queue_job` calls' function-name argument.

Mangled names must not collide with source names, and the **binding-shadow default-deny from Stage 5
applies**: a cloned/emitted body whose local shadows an emitted function name must be denied, not
silently renamed (Stage 5 shipped a live wrong-call-target miscompile through exactly this hole).

---

## 3. Scope boundary and the fail-closed contract

### In lane (Stage 6a)

- `async function` **declarations**.
- `await` whose operand is on the **operand allowlist**: `Promise.resolve(v)`; a call to another
  in-lane async function; a plain non-promise value; a Stage-5-linked `await import(...)`.
- `queueMicrotask(cb)` where `cb` is an arrow or a named function (capturing or not).
- `throw` inside an async body (keeps today's honest print-then-trap; see §3.2).
- `for await` over a sync iterable — suspends **once per iteration**.
- `Kali.test` callbacks that return a promise (see §3.3 — mandatory).

### Fail-closed — hard `E5506` (Stage 6a)

`new Promise(executor)` · `.then` / `.catch` / `.finally` · `Promise.reject` · async arrows · async
methods · async generators · **the four combinators** · **`await` / `for await` inside a `try` region
within an async function** (§3.2.1) · `await` of any operand not on the allowlist.

**Not denied, by decision:** `for await` outside an async function — left degenerate, a known residual
divergence (§3.2.2).

**Combinators failing closed for one stage is deliberate.** Those 128 names are red *today* for the
wrong reason (`await Promise.all([...])` silently returns `0`). Under 6a they are red for an honest
reason (the build rejects). Same denominator, **no newly-red**, and it *proves the default-deny is
live* instead of asserting it. Stage 6b adds the combinators to the operand allowlist and greens them.

### 3.1 How the deny is built — allowlist at the choke point, NOT a denylist of shapes

The pass censuses **every** `Expression::AwaitExpression` and every `async`-marked function node in
the entry AST, at any depth, via an **exhaustive match with no `_ =>` arm** (catch-alls are permitted
only in name *collectors*, where widening merely adds rejects). Any node the transform did not
provably rewrite is **rejected**, not ignored.

This is not stylistic. Stage 5 spent **four** review rounds on a denylist of shapes — each round
closed one fail-open and left a sibling, including one that silently linked the **wrong module** —
and only a census-every-node-and-allowlist-the-position rule stopped the leak. The identical lesson
landed independently in the for-in-key value-escape class. Two headline lessons, same shape:

> **A no-op walk arm is a fail-open.** Every no-op arm must cite `kali_parser` / `kali_ast`
> `file:line` proving the node cannot carry an `await` or an async function. An arm whose no-op rests
> on an *unverified* parser claim is how Stage 5 shipped a live miscompile.

> **"No provenance" is the ABSENCE of a signal, not a rejection.** Only an explicit deny is
> fail-closed.

### 3.2 `throw` inside an async body

Today `throw` is print-then-trap (Stage 0). Stage 6a **keeps that**: a throw inside an async body
traps immediately rather than rejecting a promise as a value. A trap raised inside a microtask drain
must still surface as a non-zero exit — `drain_event_loop` returns `Result`, so this should already
hold, but it is a **triage item to verify empirically, not assume** (a swallowed post-suspension trap
would be a silent fail-open). Rejection-as-a-value is Stage 6b's problem (`Promise.reject`,
`allSettled`).

### 3.2.1 Suspension inside a `try` region — E5506, but ONLY inside an async function

Measured, not assumed: **25 `try` blocks in the fixture corpus contain an `await`**, all of them
`for await` loops inside `try`/`catch`/`finally` with `throw` / `return` / `continue` in the loop body.

Splitting a body at a suspension point inside a **`try` region** means the continuation runs as a
*separate function* and the handler context is lost — a `throw` node catches would instead trap, and
a `finally` would not run. A silent divergence, i.e. exactly what this program refuses.

**Rule for 6a: `await` / `for await` lexically inside a `try` block, *within an `async function`*, is
`E5506` fail-closed.** Try-region-preserving continuations (handler re-establishment across a
suspension) are deferred and recorded in the follow-up inventory. The affected tests
(`object_enumeration_finalization`, 2 native + browser variants) are **already red**, so this costs no
newly-red — verify per name at triage.

### 3.2.2 `for await` OUTSIDE an async function — a KNOWN residual divergence, deliberately NOT denied

This one falsified an earlier draft of this spec and is recorded loudly rather than buried.

`ForOfStatement` carries an **`is_await: bool`** field (`crates/kali_ast/src/statement.rs:136`), so a
top-level `for await` **is** visible to an AST census — the pass cannot pretend it isn't there.
And **10 currently-GREEN tests** in `crates/kali_cli/tests/for_of_array_iteration_spread.rs`
(`array_from_iteration_body`, `browser_harness_array_from_source`,
`array_from_set_map_break_continue_body`) use `for await (const value of Array.from(values))` **at top
level**, outside any async function, where it currently degenerates to a plain synchronous `for..of`.

Three options, all bad in different ways:

1. **Reject (E5506)** → those 10 green tests turn **newly-red**. That is a **capability regression**
   (kali could no longer build a program it builds today) and it **violates the primary gate**.
2. **Transform** → requires suspending `_start` itself (splitting top-level code into continuations),
   which is a materially larger change than 6a and is not justified by any failing test.
3. **Leave it degenerate** → keeps the 10 tests green, costs nothing, and preserves a **residual
   divergence**: the loop does not yield to the microtask queue per iteration, so ordering differs
   from node *only if a microtask is queued concurrently at top level* — which no fixture does.

**Decision: option 3.** This is precisely the Stage-5 `typeof` bucket-C situation, and the house
decision rule from that stage applies verbatim: *measure the census; if the only landable flip trades
a passing capability for a failing one, do not flip — record the sizing evidence and defer.*

**This is a live, known fail-open, and it is stated here as such.** `for await` outside an async
function is allowlisted as "left degenerate", is **not** transformed, and is **not** denied. It goes in
the follow-up inventory with these 10 test names attached as its sizing evidence. A doc that claimed
"fail-closed by construction" while this was live would itself be the defect (Stage-5 corollary).

### 3.3 `Kali.test` must settle async callbacks — or 6a is unsound

**Corrected against the code — an earlier draft of this spec overstated the failure mode.** The
native test lane **already drains the event loop after each test callback**
(`crates/kali_runtime/src/execute.rs:326`, inside the per-test loop), so once `await` suspends, an
async body still *runs* during that drain. The failure mode is therefore **not** a silent `passed`.

The real defect is **misattribution**. Verified baseline (fresh binary, today's eager semantics): a
`Kali.test` whose async callback throws after an `await` reports `FAILED 1`, exit 1 — correctly
attributed to the named test. Once `await` suspends, that throw moves **into the drain**, and
`drain_event_loop`'s error path does `return Err(vec![diagnostic])` — aborting the whole run rather
than incrementing `tests_failed` for the named test. Exit stays non-zero, but `tests_run` /
`tests_failed` and the JSON envelope no longer describe what happened — and that envelope's honesty is
exactly what Stage 0 fixed and pinned across 43 re-pinned tests.

So the harness must **attribute a drain-time trap to the test that was running**: settle the returned
promise, and on rejection/trap increment `tests_failed` for that named test rather than aborting. In
**both** the native runner (`kali_runtime`) and the browser harness JS. This lands in 6a or 6a
reports dishonest test envelopes; it is not deferrable.

### 3.4 Interaction with Stage 5 — ordering is load-bearing

`async_split` **must run after `module_link`**. Stage 5's pass owns `await import(...)`, and **39
currently-green tests** depend on statement-form `await import("./x.js")` remaining untouched. A
linked import's `await` is an **in-lane no-op** for the operand allowlist. Getting this ordering wrong
breaks the bucket Stage 5 just drained.

---

## 4. Blast radius — the central risk

Making `await` honest **reorders every currently-green async fixture**: 283 `async function` and 462
`for await` occurrences, the great majority in tests that pass *today* because eager evaluation
happens to produce the output their expectations were written against.

The saving grace is directional — we move **toward** node, so a fixture whose expectation encodes
node-truth stays green or goes green. What breaks is a fixture whose expectation encodes **kali's
wrong order**, and those are **re-pins to node truth, not regressions**.

**That distinction is established per name against a real `node` run — never asserted.** House rule
from Stage 4: a newly-red name is guilty until proven a test-census artifact. Two known census traps
apply verbatim:

- `count_tag_boxing_ops`'s hand-mirrored `SYNTHETIC_FUNCTIONS` allowlist (`runtime_smoke.rs`) **must
  be synced with every new synthetic** (`__promise_new`, `__promise_await`, `__promise_settle`,
  `__frame_alloc`, `__frame_get`, `__frame_set`, …) or the census reports a *test-side* desync as a
  product regression.
- `for await`'s 462 uses are the largest single unknown. Per-iteration suspension must not degenerate
  (silently sync) **nor** over-suspend (reordering a currently-correct loop).

---

## 5. Verification

### 5.1 Gate discipline (unchanged from five prior stages)

- **Entry snapshot:** two independent `cargo test --workspace --no-fail-fast` enumerations on a
  freshly built binary, **`sort -u`** (18 test names legitimately exist in two harness binaries each;
  raw `sort` makes `comm` report false newly-red), diffed for interleaving noise, then unioned.
  Entry = **731**, already measured.
- **PRIMARY GATE:** `comm -13 pre post` (newly-red) must be **EMPTY**. Any newly-red name is
  adjudicated against a real `node` run and either fixed or **honestly re-pinned with the
  node-vs-kali evidence recorded** — never waved through.
- **Cross-check against a `main` worktree**, never a mid-branch red baseline
  (memory: `ci-gate-vs-poisoned-baseline`).
- **Drain target:** 49 (27 `async_await_sequencing` + 22 `queue_microtask`) → ~682.
- Full enumeration is the *only* gate. Per-task runs on named binaries are necessary but **not
  sufficient** — Stage 2 shipped three regressions that every per-task gate missed because their
  owning binaries were outside the targeted run set.

```bash
cargo build -p kali_cli
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6a-post-run1.txt"     # NOTE: sort -u, not sort
# (repeat for run2, diff the two, union)
comm -13 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-post.txt"   # PRIMARY GATE: must print NOTHING
comm -23 "$SCRATCH/stage6a-pre.txt" "$SCRATCH/stage6a-post.txt"   # drain
```

### 5.2 The legacy fixtures are the wrong oracle

Several bucket fixtures **cannot distinguish a real suspension from eager evaluation** — the same
mirage Stage 5 hit (chunks returning `0n` asserted as `contains("0")`, which the fail-open also
printed). Passing them proves nothing on its own.

The load-bearing evidence is a **new exact-stdout suite, byte-compared against a real `node`**
(`crates/kali_cli/tests/async_suspension.rs`), built from probes that **cannot pass under eager
semantics**:

1. the `a` `b` `c` interleave (async call followed by top-level sync code);
2. `queueMicrotask` flag observed **false** before the await and **true** after;
3. a **capturing arrow** mutating a frame slot across a suspension;
4. an async function awaiting **another async function** (chained continuation resumption);
5. `for await` over a sync iterable, interleaved with a queued microtask.

### 5.3 Adversarial re-mask probes (mandatory — a fix report is not evidence)

Stage-5 rule: **fix reports are unreliable; re-run the reproducer on a freshly built binary.** For
each probe below, sabotage the product, rebuild, and prove the suite goes **red**; a suite that stays
green under sabotage is measuring nothing.

- **Sabotage 1:** make `__promise_await` call the continuation inline instead of enqueuing → probes
  1, 2, 4 must fail (this is precisely the pre-stage bug; if they pass, the suite is vacuous).
- **Sabotage 2:** drop the frame write in a capturing arrow → probe 3 must fail.
- **Sabotage 3:** make `Kali.test` report `passed` without settling → an async self-check `throw` must
  be caught by an explicit test asserting the harness fails (§3.3's fail-open, pinned).

### 5.4 Fixture-authoring constraints (carried from Stage 5 — live pre-existing miscompiles)

- **Never `String(<bigint>)`** — `const v = 7n; console.log(String(v))` prints `0` (node: `7`).
- **Never bind a call to a `const`** — a call-bound `const` **re-evaluates the call at every use**
  (duplicated side effects). Call directly at the `console.log` site.
- Return a plain `Number`, not a BigInt.

Ignoring these corrupts expected output for reasons that have **nothing to do with async**.

---

## 6. Deliverables

| # | deliverable |
|---|---|
| 1 | `crates/kali_cli/src/build/async_split.rs` — the AST pass (transform + census + default-deny), wired into `compile.rs` **after** `module_link` |
| 2 | Promise + frame heap objects and the `__promise_new` / `__promise_await` / `__promise_settle` / `__frame_*` synthetics |
| 3 | `queue_job(fn_index, frame_ptr)` host import + widened queue entry + `invoke_callback` params handling — **mirrored in all four `kali:rt` import lists** |
| 4 | `queueMicrotask` lowered to a real enqueue (stop inlining the arrow) |
| 5 | `Kali.test` async-callback settlement + failure propagation — native runner **and** browser harness JS |
| 6 | `for await` per-iteration suspension |
| 7 | `crates/kali_cli/tests/async_suspension.rs` — exact-stdout, node-byte-compared, non-vacuity proven by the §5.3 sabotage probes |
| 8 | Stage-6a triage doc (`docs/superpowers/followups/`) — entry snapshot, per-name newly-red adjudication with node evidence, drain table, follow-up inventory |

## 7. Explicitly out of scope (Stage 6b or later)

`Promise.all` / `allSettled` / `any` / `race` (the 128) · `Promise.reject` and rejection-as-a-value ·
`new Promise(executor)` · `.then`/`.catch`/`.finally` · async generators · async arrows/methods ·
**try-region-preserving continuations** (§3.2.1) · **suspending top-level code, which is what a
faithful top-level `for await` would require** (§3.2.2 — carries a live residual divergence, with the
10 green test names as sizing evidence) · real async I/O.

Bucket #8 (`&&`/`||` short-circuit, 13 names) remains earmarked for PR-B and is untouched.
