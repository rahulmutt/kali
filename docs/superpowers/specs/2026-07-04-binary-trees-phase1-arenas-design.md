# Binary-trees Phase 1: dynamic current-arena reclamation + precursor fixes

**Date:** 2026-07-04
**Status:** Approved
**Goal:** `kali run` executes the canonical CLBG binary-trees benchmark at N=21
byte-for-byte, under plain `kali run` (no `--sandbox` policy expected).
**Builds on:** `2026-07-04-region-reclaiming-allocator-design.md` (Phase 0, shipped
PR #5). Respects the standing GC-less invariant: reclamation is region/arena-based
only — no tracing, copying, or generational collection, no shadow stack, no write
barriers.

**Supersession note:** the Phase-0 spec sketched Layer 2 as static region assignment
with the arena threaded as an implicit function parameter ("region-polymorphic
factories"). This spec supersedes that mechanism with a **dynamic current-arena**
model (rationale in "Approach"). The Layer-2 goals — per-loop/per-function arenas,
page pool, free-page list, one-directional escape bias, outlives soundness — are
unchanged.

## What actually blocks binary-trees (findings, 2026-07-04)

Empirical results on main (`abc068239`):

1. **A latent wild-pointer bug at ~17.1MB cumulative allocation.** Object workloads
   with bound identifiers and correct shapes run byte-correct until cumulative bump
   allocation crosses a boundary in **(17,087,840 .. 17,169,600] bytes**, then trap
   E4000 on a wild field access in *user* code. Pinned repro: depth-8 trees
   (511 nodes × 16B = 8,176B/tree) allocated in a loop — 2,090 iterations correct
   output, 2,100 traps. Byte-driven (depth-4 trees trap at the same cumulative
   volume), not iteration- or depth-driven. `__alloc` is absent from the trap
   backtrace and growth to 33MB+ works for arrays, so this is a pointer-path bug,
   not an allocator OOM. Root cause unknown — P0a below.
2. **Object-shape inference misses call-result arguments.**
   `itemCheck(bottomUpTree(d))` silently miscompiles (returns 1) unless some other
   call site passes a bound identifier, which seeds the callee param's shape as a
   side effect. This is the known "shape flows only from bound idents" gap; it
   also retroactively explains the Phase-0 task-3 report's "broken at depth 10"
   (nothing regressed or got fixed since). Reject-don't-miscompile violation — P0b.
3. **No reclamation.** N=21 allocates ~9.4GB cumulative against wasm32's 4GB
   ceiling, so no budget policy can save it; arenas are required — P1. With
   arenas, peak live ≈ stretch tree (134MB) + long-lived tree (67MB) + one
   iteration tree (67MB) ≈ **270MB**, comfortably under 4GB. Plain `kali run`
   has no memory ceiling today (module declares `maximum: None`, no default
   policy), so no `--sandbox` policy is expected; if that proves wrong
   empirically, the mandelbrot scoped-policy precedent is the fallback.

Also established: stdout is buffered and **lost** when a trap occurs (a leading
`console.log` before a later trap prints nothing), which disguised bug (1) as
several different bugs during investigation.

## Scope

Three work packages, in dependency order.

### P0a — root-cause and fix the ~17.1MB wild-pointer trap

- Entry: the pinned repro above. Method: systematic debugging (`.wat` dump with a
  name section, inspect the pointer path around the boundary). No fix is designed
  here because the root cause is unknown; the spec pins the acceptance instead:
  the repro family runs byte-correct to **≥64MB cumulative** allocation.
- Drive-by fixes riding along (same debugging surface):
  - flush buffered stdout before reporting a runtime trap — partial output is
    evidence and must not be discarded;
  - the E4000 diagnostic distinguishes trap kinds (out-of-bounds access vs
    `unreachable` from the allocator's OOM check).

### P0b — wire call-result arguments into object-shape inference

- In `kali_types/src/repr_infer.rs`: a call expression in argument position unions
  its **return-storage node** with the callee's **param-storage node** (the edge
  already exists for bound identifiers; this adds the arg-position call-result
  case).
- Acceptance: `itemCheck(bottomUpTree(d))` is correct with no other call site
  seeding the param.
- Backstop: any object-valued argument shape the inference still cannot classify
  is rejected at compile time with E5506 — never lowered as a scalar.
- Out of scope: the module-scope nested-object miscompile
  (`const t = { left: leafA, ... }` at top level reading `t.left === null` as
  true) — unless P0b's fix covers it incidentally; it gets the E5506 backstop
  only if free. It stays on the follow-up inventory.

### P1 — dynamic current-arena reclamation

Loop arenas land first (they alone reach N=21), function arenas second within the
same plan — the machinery is shared.

Out of scope for the whole spec: Phase 2 manual-free heap (design-noted in the
Phase-0 spec, still deferred); browser-side arena work beyond keeping existing
harnesses green.

## Approach: why dynamic current-arena (vs threaded params)

Chosen for performance and invasiveness, with the user's sign-off:

- **Allocation fast path unchanged in shape.** The current arena's cursor/limit
  live in wasm globals, so `__alloc`'s hot path stays the proven
  read-add-compare-write bump. Threaded params would put cursors behind a
  descriptor pointer in linear memory (or force rebuilding the globals mechanism
  anyway).
- **Zero per-call overhead.** No signature changes; `bottomUpTree` at N=21 is
  ~590M calls that would each pay an extra argument under threading. Closures and
  indirect calls are unaffected.
- **Region polymorphism for free.** A factory allocates into whatever arena its
  caller had active: called before the loop → the function arena (long-lived
  tree); called inside the loop → the iteration arena. Same reclamation power as
  threading for this workload.
- **Costs move to scope boundaries:** arena open/reset ~500k times at N=21 vs
  590M allocations — three orders of magnitude off the hot path.
- Kali compiles to single-threaded wasm, so dynamic scoping has no reentrancy
  hazard; recursion is safe because open/close is properly nested via wasm frames.
- Threading's theoretical edge is precision (distinct regions within one dynamic
  extent, lower peak in some programs); irrelevant here at 270MB peak vs 4GB.
- **Selectivity rule (performance guard):** an arena is opened only where the
  analysis shows reachable, non-escaping allocation — `itemCheck` (1.2B calls,
  allocates nothing) gets no arena and no prologue.

## Architecture

### Memory model: one page pool, arenas are page lists

- All memory above `heap_base` is a pool of **64KB pages** (wasm page size).
  Page header: 8 bytes, `{next_page, span_pages}`; payload 8-aligned.
- **Free-page list:** intrusive, head in a global. Page acquisition: pop free
  list, else bump the frontier (today's `__heap`, repurposed), growing memory
  with Phase-0's geometric `memory.grow` (still trapping cleanly on `-1`).
- **Spans:** allocations larger than a page payload (big arrays) take contiguous
  multi-page spans from the frontier; a span returns to the free list as one
  entry (`span_pages` in the header). Arrays keep working unchanged in arenas.
  Free-list reuse policy (exact-fit vs split-and-remainder) is `__page_get`'s
  implementation detail — either is sound because pages are uniform and every
  entry carries its length.
- **Global arena:** never reset; active at `_start`; also the target for
  escape-gated sites via `__alloc_global`.

### Current arena = three globals; open/close state in wasm locals

- `__arena_page` / `__arena_cursor` / `__arena_limit` describe the active arena;
  `__global_page` / `__global_cursor` / `__global_limit` describe the global
  arena; plus the free-list head. `__heap` stays **global 0** (the frontier),
  export name unchanged; new globals are appended after it.
- **Open** = save the three current-arena globals into locals of the enclosing
  function, install a fresh page. **Close/release** = return the arena's page
  list to the free list, restore the globals from those locals. No descriptors
  in memory, no shadow stack — nesting and recursion ride the wasm frame
  structure.
- **Per-iteration reset** keeps the arena's first page (rewind cursor, surplus
  pages to free list) — steady-state iterations do zero page-list churn.

### The escape gate (in kali_mir)

Extends the existing `OwnershipAnalyzer` (`crates/kali_mir/src/analysis/`), whose
`BindingState` already tracks `returned` / `escaped_via_flow` / `captured_by`.

- **Per-site fate lattice:** scope-local → returned → global. Propagation:
  embedding in an object-literal field joins child to parent's fate; `return`
  hands the fate to call sites; assignment into a module binding, a closure
  capture, or a field/element of a **pre-existing** object forces global.
  Returned is *not* global in the dynamic model — the obligation moves to
  whoever opened the active arena:
- **Arena-opening rule (where soundness lives):** a loop or function body gets an
  arena only if (a) it transitively reaches current-arena allocation sites, and
  (b) **no heap-typed value flows out of the scope** — not via assignment to an
  outer-declared binding, not via return/`break` paths, not via storage into
  outer objects. Scalars flow freely. Host imports on a known non-retaining
  whitelist (`console.log`, stdout writes) consume their arguments; any unknown
  or closure/indirect call in scope **vetoes** the arena.
- **One-directional bias = the soundness argument:** vetoing an arena or sending
  a site to the global heap is always sound; the only cost is memory not
  reclaimed. All ambiguity fails closed.
- **v1 granularity (deliberate coarseness):** site classification is uniform per
  function (any global site ⇒ all of that function's sites global); arena
  decisions are per loop, keyed `(function_name, loop_preorder_ordinal)`. For
  binary-trees this costs nothing: `bottomUpTree`'s literals are returned-fate,
  `itemCheck` allocates nothing, both benchmark loops pass the outflow check.

### Data flow: name-keyed ArenaTable (ReprTable precedent)

- No node ids survive lowering (MIR→LIR reallocates ids; codegen sees only LIR),
  so the analysis emits a **name-keyed `ArenaTable`** in `kali_common` beside
  `ReprTable`: `arena_eligible(function)`, `opens_arena(function)`,
  `loop_arena(function, loop_ordinal)`. Delivered as a second field on
  `CodegenCtx`, set in `compile.rs` next to `repr_table`.
- **Misses fail closed** (no arena / global alloc). This also covers
  `kali_optimize` specialization under new names: degraded reclamation, never
  unsoundness.
- MIR→LIR is a 1:1 structural copy, so loop pre-order ordinals are stable by
  construction; a dedicated test pins ordinal stability across HIR→MIR→LIR so
  any future reordering fails loudly.

### Codegen changes

- **Five synthetics**, hand-emitted via the `FunctionPlan` idiom Phase 0
  established for `__alloc` (name-map resolution, type entry, locals
  special-case, coverage exclusion — the `functionsTotal` filter becomes a
  synthetic-name set):
  `__alloc` (rewritten: bump from `__arena_*`, overflow → slow path),
  `__alloc_global` (bump from `__global_*`),
  `__page_get` (shared slow path: free list, else frontier + geometric grow),
  `__arena_reset` (rewind to first page, surplus to free list),
  `__arena_release` (all pages to free list).
- **No new host imports** — the four hand-mirrored browser-harness JS import
  lists need no changes.
- `_start` prologue installs the global arena's first page and points the
  current-arena globals at it.
- **Emission hooks:** the emitter's existing `loop_frames` stack gains a parallel
  open-arena stack (which frames opened arenas; which locals hold saved globals).
  (1) Function prologue: if `opens_arena`, save globals to three fresh locals,
  install a fresh page. (2) `emit_loop`: if the loop has an arena, open before
  the wasm `loop`; per-iteration reset at the **top** of each iteration (making
  `continue` correct by construction — every re-entry passes the reset; nothing
  live spans iterations by the outflow rule). (3) Scope exits: `emit_return`,
  `break`, and the fall-through function end unwind the open-arena stack,
  releasing every arena frame they cross (both return paths: explicit
  `Instruction::Return` in `control_flow.rs` and the implicit trailing `End` in
  `lower.rs`).
- **Call lowering untouched.** Allocation sites (`emit_object_allocation`, array
  allocation) change one line: call `__alloc` if the enclosing function is
  `arena_eligible`, else `__alloc_global`.
- **Audit item:** confirm runtime template-literal string building allocates via
  `__alloc` (or route it to the global arena) so interpolated strings cannot
  dangle across a reset.

## The fixture

- `binary-trees-benchmark-v1.ts` in the **canonical CLBG shape**, including
  direct `itemCheck(bottomUpTree(depth))` call-result arguments (legitimate after
  P0b), normalized only by established kali idioms (`depth = depth + 2`,
  `1 << k` iteration counts, template-literal output with real tabs — the
  Phase-0 escape lane).
- Mirrors the four existing CLBG fixtures: `.json` metadata (sha256, buildModes),
  e2e `clbg_binary_trees_runtime.rs`, new feature-maturity row.
- Canonical N=21 output lines:
  `stretch tree of depth 22\t check: …`, `N\t trees of depth d\t check: …`,
  `long lived tree of depth 21\t check: …`.

## Testing

- **P0a:** the 17MB repro pinned as a regression test — cumulative allocation to
  ≥64MB byte-correct (extends the `heap_grow_runtime.rs` family).
- **P0b:** `repr_infer` unit tests (call-result arg seeds callee param with no
  other seeding site); E5506 rejection case for unclassifiable object args.
- **Escape gate (kali_mir unit tests):** each lattice transition (returned,
  global-store, capture, literal-field embedding) and each opening-rule veto
  (outer-binding assignment, unknown/closure call, host whitelist).
- **Keying:** loop-ordinal stability across HIR→MIR→LIR.
- **Arena runtime (e2e):**
  - proof-of-reclamation: ~1,000 iterations × ~1MB/iteration ≈ 1GB cumulative
    with a few-MB peak — impossible before arenas;
  - nested arena'd loops; arena'd loop inside arena'd function;
  - early exits: `return` / `break` / `continue` crossing arena frames;
  - multi-page spans (large arrays inside an arena'd loop);
  - adversarial soundness (gate must fail closed to the global heap, values
    correct): the long-lived-tree pattern, store-to-outer-binding from inside a
    loop, escape via object-field chain.
- **Acceptance:** binary-trees N=21 byte-for-byte canonical output under plain
  `kali run`.
- **Test tiers:** always-on byte-exact small-N run (n=10, sub-second); the N=21
  run's CI placement decided empirically — wasm executes at native speed
  regardless of host profile, so it stays always-on if within the suite's
  wall-clock budget, else `#[ignore]` + a named mise task (small-N remains the
  gate).
- **Regression:** all four existing CLBG fixtures output-identical (wasm bytes
  will differ — `__alloc` rewritten, globals shifted; the bar is canonical
  output incl. mandelbrot's 5011-byte golden PBM). Full 5-crate gate green,
  `kali_fmt` clean, browser smoke run (no import changes expected).
- **Performance sanity:** n-body and mandelbrot wall-clock not materially
  regressed — the bump fast path is shape-identical, so any regression is a bug.

## Risks

Carried from Phase 0: per-loop arenas are mandatory against region explosion
(MLKit's lesson); the escaping-value trap (dangling pointer on a wrongly-reset
arena) is covered by the one-directional bias plus adversarial tests; page-based
arenas, not saved-pointer tricks; every growth path checks `memory.grow == -1`;
synthetic-function index bookkeeping stays name-map-resolved.

New: the 17MB bug's root cause is unknown and could reshape assumptions — P0a
runs first for exactly this reason; loop-ordinal keying fragility — pinned by the
stability test, and misses fail closed; the runtime string-building allocation
path — explicit audit item.
