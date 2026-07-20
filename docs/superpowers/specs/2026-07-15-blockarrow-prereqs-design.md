# Block-arrow un-flatten prerequisites — function-body soundness + closures

> **Umbrella design.** Covers the project shape, methodology, stage sequence, and
> each stage's scope + success criteria. Each stage (B, A, C, D) gets its own
> spec → plan → subagent-driven execution → review cycle. Stage C additionally
> gets its own dedicated design brainstorm before its plan (it is greenfield).

**Branch:** `soundness-batch1-pra` · **Baseline commit:** `192984c39` (Stage 6
Tasks 1–4 landed) · **Frozen failure baseline:** **731** tests.

**Origin:** Stage 6 (block-arrow un-flatten) landed its repr-tracking foundation
(Tasks 1–4) but the un-flatten itself could not land — applying it produced 22
newly-red, 20 of which trace to three structural gaps repr-tracking does not
touch. User decision (2026-07-15): build the missing capabilities, then land the
un-flatten. Full outcome: `docs/superpowers/followups/throw-fallout-stage6-triage.md`
§10. Preserved un-flatten work: `docs/superpowers/followups/task5-block-arrows-WIP.patch`
(605 lines, verified to re-apply onto `192984c39`).

---

## 1. The reframe (load-bearing)

The three gaps are **not** properties of block-arrow parsing. Verified on the
clean branch (no patch applied): all three reproduce **identically** using plain
`function(){}` expressions (as `queueMicrotask(function(){…})` and as bare IIFEs)
— same E5506s for A and C, same silent wrong answer for B. They are pre-existing
soundness holes in how kali compiles **any real function body**. The un-flatten
(Stage D) is blocked behind them only because un-flattening turns thousands of
callback bodies into real function bodies, multiplying how often the gaps fire.

**Consequences:**
- A/B/C are **independent** soundness fixes, each valuable on its own regardless
  of whether the un-flatten ever lands. Different subsystems: A = `repr_infer`,
  B = codegen loop/arena keying, C = closure lowering.
- A/B/C are **developed and gated on the clean branch** using `function(){}`
  reproducers — **no un-flatten patch and no feature flag** are needed until
  Stage D. "Harden the sink before widening what feeds it."

## 2. Methodology and gating (all stages)

- **Develop on the clean branch** with `function(){}` fixtures. Do not apply the
  WIP un-flatten patch except in Stage D.
- **Primary gate: zero newly-red.** `comm -13 <baseline> <post>` over a full
  `cargo test --workspace --no-fail-fast` enumeration must be empty. Cross-check
  against a `main` worktree.
- **Enumeration uses `sort -u`, never plain `sort`** — 18 test names exist in two
  harness binaries each; raw `sort` fabricates newly-red.
- A full workspace run exceeds one command timeout — run it detached with a
  `.done` marker and poll a bounded blocking loop; do not rely on background
  notifications.
- **No drain is claimed** by any stage. A stage that closes a fail-closed
  over-rejection may turn some currently-red tests green — measure it, do not
  forecast it.
- **Reject-don't-miscompile** is the invariant. Where a stage cannot lower a
  construct correctly, it must fail closed (E5506) with a clear diagnostic —
  never emit a silent wrong answer. This is GC-less by design (region/escape
  reclamation only; never a tracing GC).
- Each stage: its own spec → plan → subagent-driven execution → task reviews →
  whole-stage review. The **whole-branch review of Stage 6 Tasks 1–4 is deferred
  to Stage D**, where the un-flatten exercises that foundation for real.

## 3. Stage sequence: B → A → C → D

Ordered by soundness urgency. B is the only **silent** miscompile (worst class);
A and C both fail **closed** today (sound, merely over-rejecting), so they are
less urgent. D lands last on the hardened foundation.

### Stage B — array-callback nested-function SILENT miscompile *(first)*

**Symptom:** a named `FunctionDeclaration` nested one level inside a real function
body, containing **2+ array-callback for-of loops** (`for (const x of [1,2].map(v=>v)) …`),
produces a silently wrong result — kali prints `0` (or otherwise wrong), exit 0,
no diagnostic; node prints `1,2,3,4`. Bisected (in the Task-5 investigation) to
require both (a) the enclosing function body being a real (non-module) scope and
(b) a nested named `FunctionDeclaration`; a single such loop works, and the same
loops at module scope or in a non-nested function work.

**Root cause:** unknown. Suspected a per-function keying collision in the codegen
loop-ordinal / arena machinery (`kali_codegen` loop/arena tables, possibly
interacting with `kali_mir` escape flow).

**Stage B is investigation-first.** Its plan opens with a systematic root-cause
(bisect to the specific table/line), then a fix. This is a
`systematic-debugging` stage, not a design stage.

**Success criteria:**
- The bisected repro computes the correct result **or** fails closed E5506 —
  never a silent wrong answer.
- A regression fixture pinning the previously-silent case (asserting the correct
  output, cross-checked vs node) that goes red if the fix is reverted.
- Confirmed independence: the fix does not depend on A or C, and A/C do not mask
  or resolve B (checked by re-running B's repro after establishing B alone).
- Zero newly-red vs 731.

### Stage A — `repr_infer` walks function-shaped bodies

**Gap:** `repr_infer.rs` (the whole-program repr inference pass, run once at
`resolve/mod.rs:350`) has **zero expression recursion** — its three statement-only
collection walkers never descend into a `FunctionExpression` / `ArrowFunctionExpression`
/ class-method body. So no binding declared inside such a body is seeded into the
object-shape or String-repr inference, and:
- `for..in` over an object literal declared in a callback body → E5506
  ("fixed-shape" proof unavailable).
- a String-typed local computed in a callback body, `+`-concatenated → E5506 when
  checked directly, or (worse, behind an already-proven-string `+`) a silent raw
  i64. *(This silent sub-case must be closed by this stage or it becomes a Stage-D
  regression.)*

**Approach:** extend `repr_infer` to recurse into fn-expr/arrow/class-method
bodies, registering each nested body under its **Task-2 synthetic name**
(`__kali_fn_{N}`, already assigned before the resolver) and seeding/flowing its
locals under **its own key** rather than leaking into or being invisible to the
enclosing function's graph. Reuse Task 2's exhaustive-walk pattern
(`name_anon_functions.rs`) as the recursion checklist. Closes both the
object-shape and String-seed symptoms as one architectural fix (they share the
same root gap).

**Success criteria:**
- `for..in` over an object literal declared in a function body compiles and runs
  (cross-checked vs node).
- A String local computed in a function body `+`-concatenates correctly (no raw
  i64), both directly and behind a proven-string `+`.
- Whole-program BFS/reachability still converges (no regressions to existing
  `repr_table` consumers: String proofs, object-shape proofs, growable-array
  detection).
- Zero newly-red vs 731; measure any drain (over-rejections this closes).

### Stage C — environment-pointer closures *(own design brainstorm before planning)*

**Gap:** kali codegen has **no closure model**. A callback that reads or mutates
an enclosing **function-scope** local fails closed E5506 (`emit/literal.rs:496` —
`self.locals` holds only the callback's own params + declared locals; an outer
local is absent). The only cross-boundary sharing that exists is
`module_global_slots`, hardwired to **module** scope. kali_mir already computes
the capture set per function (`captured_by`, `LayoutDescriptor::Closure`,
free-variable detection at `walk.rs:391`) — but that analysis never reaches
codegen.

**Chosen approach (user decision): environment-pointer model (full closures),
not the minimal single-cell slot-promotion.** A real per-activation environment
record in linear memory; an env pointer threaded via a global for synchronous
calls (map/filter/forEach/`Kali.test`) and carried through the host schedule/drain
for deferred callbacks (`queueMicrotask`/`setTimeout`/`addEventListener`).
Correct under recursion/re-entrancy, where a single global cell would conflate
distinct activations.

**Known hard sub-problems (to be resolved in Stage C's own design pass):**
1. **Nullary callback ABI.** The host invokes callbacks by index with zero
   arguments (`enforce.rs:120`), so the env pointer cannot arrive as a parameter
   — it must be reachable from a global "current env" set before each invocation
   (and restored by the host for deferred callbacks).
2. **GC-less reclamation of the env record.** A synchronous callback's env can
   live in the enclosing call's arena (freed on return). A **deferred** callback's
   env must outlive the enclosing arena reset — needing either a reclaimable
   escaping-capture region or promotion to the never-reset `__alloc_global` region
   (a controlled leak). Heap values captured across a deferred boundary have no
   reclaimable home under region-only reclamation and **fail closed** unless this
   design introduces such a region.
3. **Which captures, which timing.** Scope the minimum that unblocks the failing
   tests (scalar capture in a deferred event-listener, `count += 1`) while keeping
   everything unprovable fail-closed.

**Because this is greenfield, Stage C is NOT specified here.** It gets a dedicated
`brainstorming` → spec pass before its implementation plan.

**Success criteria (project-level; refined in C's own spec):**
- A callback reading and mutating an enclosing function-scope scalar local
  compiles and runs correctly (cross-checked vs node), including the
  `web_baseline_primitives` event-listener `count += 1` shape.
- Cases that cannot be lowered soundly (heap capture across a deferred boundary;
  anything unprovable) fail closed E5506, never silently wrong.
- Zero newly-red vs 731.

### Stage D — land the un-flatten

**Approach:** rebase the preserved WIP patch (`task5-block-arrows-WIP.patch`) onto
A+B+C. The patch already contains the sound parser un-flatten, the
`reject_anonymous_function_argument` fix (keyed on the `__kali_fn_` synthetic
marker, since Task 2's pre-pass fills every `id`), the verified `queueMicrotask`
wiring (callback confirmed to run during `drain_event_loop` byte-for-byte vs
node), and the `setTimeout`/`setInterval` fail-closed carve-out. With A/B/C
landed, the 20 residual newly-reds should be closed; the 2 user-approved
category-1 re-pins (`runtime_smoke.rs` object-type-and-constructor-semantics
test-mode path → the trap now attributes to `__kali_callback_` in `stderr` +
`payload.failed`) are applied here.

**Success criteria:**
- Zero newly-red vs 731 (the real proof of A/B/C).
- The exemption list in `reject_anonymous_function_argument` contains **exactly**
  the consumers that are actually wired-and-invoked; every exemption has a real
  wire (an exemption without a wire silently drops the callback — the original
  bug).
- Block arrows parse as functions in all expression positions; callbacks are
  actually invoked (deferred-vs-dropped probe), not run inline.
- The deferred whole-branch review of Stage 6 Tasks 1–4 runs here.

## 4. Open item carried into Stage D (not this project's scope)

Class-method `return` still silently returns `0` (Stage 6 Task 4 verdict —
separate root cause in return-value lowering, distinct from repr-tracking). Its
test stays `#[ignore]`'d with assertions intact. Not a prerequisite for the
un-flatten; filed as an independent follow-up.

## 5. Non-goals

- No tracing/copying/generational GC (GC-less invariant — region/escape only).
- No feature flag for the un-flatten (A/B/C are developed patch-free; D lands the
  un-flatten unconditionally).
- No hand-mirrored oracles: one name assigned in the AST (Task 2's pre-pass), read
  by every consumer (`kali_types`, `kali_hir`, `repr_infer` after Stage A).
