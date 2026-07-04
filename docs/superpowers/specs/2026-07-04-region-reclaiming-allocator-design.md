# Reclaiming allocator: memory.grow + escape-analysis regions — with binary-trees as the proving CLBG fixture (design)

Date: 2026-07-04
Status: proposed (awaiting user review)
Topic: replace the non-reclaiming bump allocator so the next CLBG fixture (binary-trees) can run at its canonical parameter

## Context

kali has four vendored, end-to-end-executing adapted CLBG fixtures, each of which
opened one new representation lane:

- **fannkuch-redux** — integer imperative core (`i64` loops, mutable locals, calls).
- **spectral-norm** — floating-point (`f64`) arithmetic, `f64` arrays, `Math.sqrt`, `.toFixed`.
- **n-body** — fixed-shape bump-allocated heap objects (flat, non-nested).
- **mandelbrot** — bitwise-integer operators + host-only binary stdout.

The next natural CLBG program is **binary-trees**. Of the remaining canonical set it is
the only one that needs no stdin (rules out reverse-complement, k-nucleotide,
regex-redux) and no arbitrary-precision bignum (rules out pidigits — kali's `BigInt` is
i64-backed in the compiled path and silently traps on values exceeding i64). Empirically,
its core capability — **nested / self-referential heap objects with nullable reference
fields, and recursion returning object references** — *already works*: a depth-4 tree
returns the correct node count (31), and depth-9 returns the correct long-lived check
(1023 = 2¹⁰−1), even though the n-body maturity row explicitly disclaimed nested objects.

The blocker is memory. binary-trees allocates a large volume of trees and discards them;
in real JS the garbage collector reclaims them. kali's allocator **never reclaims**, so
cumulative (not live) allocation is what matters, and canonical depth N=21 (millions of
accumulating nodes) is unreachable. This design fixes that.

### The two independent limits (both confirmed in codegen)

1. **No `memory.grow` anywhere.** `crates/kali_codegen/src/lower.rs` defines linear memory
   at `minimum: 16` pages = **1 MB**, `maximum: None`, and nothing ever grows it. Every
   program has a hard 1 MB heap wall regardless of the sandbox `resources.maxMemoryMB`
   budget — the sandbox memory budget is effectively unusable by guest programs today.
   binary-trees N=9 fits (~825 KB); N=10 needs ~2.2 MB and traps (`E4000`) — *even under a
   4 GB / 600 s sandbox policy*, confirming the wall is the fixed wasm memory, not the
   sandbox.

2. **No reclamation, and headerless objects.** `emit_object_allocation`
   (`crates/kali_codegen/src/emit/object.rs`) does `base = __heap; __heap += nfields*8`
   with **no type tag, no size field, no GC metadata**. Arrays carry an `i64` length
   header at offset 0; objects carry nothing. Allocation is inlined at **5 sites** (objects
   + arrays in `emit/object.rs` and `emit/call.rs`), each poking the `__heap` i32 global
   (global index 0) directly. There is no shared `alloc(size)` helper and no internal
   (non-import) helper-function pattern in codegen today — only host imports (fixed indices
   17–20) and user functions offset by `FUNCTION_INDEX_OFFSET`.

## Goal

Give kali's runtime a **reclaiming allocator** so allocation-churning programs run at
realistic scale, and prove it by vendoring a **binary-trees** CLBG fixture that executes
end-to-end under `kali run` at the **canonical parameter N=21**, pinned byte-for-byte
against canonical CLBG output.

Two independently-valuable capabilities land underneath:

- **`memory.grow`** so the heap can exceed 1 MB up to the sandbox budget.
- **Escape-analysis-driven lexical arenas (regions)** that reclaim scope-local allocations
  by O(1) page-list reset, the primary reclamation mechanism.

Plus one small orthogonal lexer lane the fixture's canonical output requires:

- **String-escape processing** (`\t`, `\n`, `\\`, `\"`, `\'`, `\0`, `\r`, `\b`, `\f`, `\v`).
  Today the lexer passes escapes through verbatim (`"a\tb"` → literal `a\tb`,
  `"c\nd"` → literal backslash-n), so canonical binary-trees output (real TAB between
  fields) cannot currently be produced.

## Non-goals

- **No full Tofte–Talpin region inference.** Its region polymorphism / arrow-effect
  machinery is overkill for a statically-typed, single-threaded imperative subset, and
  (per 15 years of MLKit experience) is prone to region-explosion space leaks. We build the
  lighter escape-analysis form. The *one* Tofte–Talpin idea we keep is region-polymorphic
  factory functions.
- **No tracing garbage collector in this cycle.** We design *toward* a copying/generational
  GC (forward-compat decisions listed) but do not build it. The fallback heap remains the
  existing non-reclaimed bump behavior for escaping/ambiguous objects this cycle.
- **No size-classed free-list fallback heap in this cycle** (Phase 2, deferred). Escaping
  objects go to the existing global bump heap; they are not individually freed yet.
- **No throughput / performance claim** (consistent with the other CLBG maturity rows).
  This records execution-correctness coverage for a reclamation lane, not a speed claim.
- **No shadow stack / GC roots build-out in this cycle** — only the cheap, non-locking-in
  discipline choices that keep the GC endgame from becoming a rewrite (see Phase 2).
- **No `\xNN` / `\uNNNN` / `\u{…}` numeric escapes** — recognized escapes are processed;
  unknown/numeric escapes are **rejected with a diagnostic**, never silently passed through
  (reject-don't-miscompile).

## Strategy synthesis (grounded in the research pass)

Three research writeups (fast manual allocators; region inference & escape analysis; GC in
AOT/wasm-compiled languages) converge on the following. Full citations in References.

- **`memory.grow`:** grow **geometrically/batched** (double, or the requested amount,
  whichever is larger — never one page per allocation, because some engines memcpy all of
  linear memory on grow). 64 KiB page granularity. `memory.grow` returns **−1 on failure
  and never traps** — the result must be checked and treated as clean OOM. wasm memory
  **never shrinks**; there is no `munmap`/decay analogue to port. (walloc; Wellons 2025;
  wasm core spec.)

- **Primary reclamation = escape-analysis-driven lexical arenas** (Choi et al., OOPSLA'99),
  not full region inference. Single-threaded kali collapses the escape lattice to
  essentially "escapes this scope / doesn't" (no thread-escape, no lock elision). Per-
  **function** and per-**loop** arenas; the **per-loop arena is the whole win** for
  binary-trees (transient inner-loop trees become O(1) live memory per iteration). This is
  the **Reaps** design — regions for the common case + a general heap for escapers
  (Berger/Zorn/McKinley, OOPSLA'02), and matches how Cyclone pairs LIFO regions with a
  GC'd heap region.

- **Region-polymorphic factories** (the one kept Tofte–Talpin idea): a factory that returns
  a fresh object (`bottomUpTree`, spectral-norm's `makeVector`) must build into the
  **caller's** arena, so an escaping result lands in the right-lifetime region. Same code
  serves the transient and escaping call sites; the caller passes the target arena.

- **Page-based arenas, not a single saved bump pointer.** Two arenas are live
  simultaneously in binary-trees (the escaping long-lived tree in the function/global arena
  is allocated *interleaved* with transient inner-loop nodes in the loop arena). A single
  bump pointer cannot host two interleaved live arenas; each arena must be a **linked list
  of 64 KiB pages** drawn from a global free-page list, and reset returns its pages to that
  list (MLKit region representation). The saved-bump-pointer scheme is kept only as a fast
  path for an escape-free leaf scope.

- **Soundness is one-directional — over-approximate escape.** Wrong toward "escapes" costs
  retained memory (safe). Wrong toward "local" causes a dangling pointer and heap
  corruption (catastrophic). Every imprecision resolves toward *escapes → outer arena →
  fallback heap*. The arena optimization sits on top of a correct-by-default "everything on
  the fallback heap" baseline, so only over-eager promotion-to-local can break safety — and
  that direction gets the proofs/tests. This is Go's stance ("ambiguous ⇒ heap").

## Architecture

Three layers over wasm linear memory, all reached through one centralized helper.

### Centralized `__alloc` helper (foundation)

Refactor the 5 inline allocation sites to call a single synthetic wasm helper function
`__alloc(size, region) -> ptr` (a new emitted-function pattern — the first internal,
non-import helper; requires extending the function-index bookkeeping in `lower.rs`). Object
and array emitters compute `size` and the target `region` and call it. This is the seam
both `memory.grow` and arenas hang off, and it removes the duplicated bump logic.

### Layer 1 — page allocator (`memory.grow`)

Owns 64 KiB pages. `alloc_page()` pops from a free-page list, else bumps a global page
cursor, else `memory.grow`s (geometric/batched; checks −1 → OOM trap with a clean
diagnostic). `free_page(p)` pushes to the free-page list. Pages are 64 KiB-aligned so
`addr >> 16` indexes a flat side table. **Phase 0 ships this** and immediately lifts the
1 MB wall to the sandbox `maxMemoryMB` budget.

### Layer 2 — arenas (primary reclamation, Phase 1)

An arena = `{ page_list, cur_page, cursor, limit }`. Allocation is a pure bump within the
current page; overflow pulls a fresh page via `alloc_page()` and relinks. **Reset** returns
the arena's pages to the free-page list — O(#pages), metadata-free. Arenas are opened at:

- **function body** → per-call arena, reset on return;
- **loop body** → per-iteration arena, reset at the bottom of each iteration;
- **module/program top** → the **global arena** = today's `__heap`, never reset; doubles as
  the **fallback heap**.

A new **MIR escape-analysis pass** (natural home: `crates/kali_mir/src/analysis/`, beside
the existing ownership/binding analyses) computes, per allocation site, the innermost scope
it does not escape, and rewrites the site to allocate in that scope's arena; factories are
made region-polymorphic (the arena is threaded as an implicit parameter). Anything global
or ambiguous → fallback heap.

### Layer 3 — size-classed fallback heap (Phase 2, deferred)

For escaping/global/ambiguous objects that should be individually freed. Single-threaded,
size-classed (exact 8-byte classes: every kali object is a multiple of 8, so
`class = size >> 3` → zero internal fragmentation on 16-byte tree nodes), bump-in-page then
intrusive-free-list hybrid, all mimalloc/jemalloc concurrency machinery deleted. **Not
built this cycle** — escaping objects use the non-reclaimed global bump heap for now.

### Forward-compatibility for the eventual GC (design-noted; only the cheap discipline now)

The copying/generational GC endgame keeps bump allocation and suits the binary-trees
"mostly garbage" pattern. To avoid a later rewrite, this cycle adopts only the cheap,
non-locking-in disciplines and *defers* the expensive build-out:

1. **Centralize the object-base → field-address computation and every pointer-field store**
   through one codegen path (write-barrier / spill hook later). *(Do now — cheap discipline.)*
2. **Emit the static `ShapeId → {size, pointer-slot bitmap}` descriptor table** in a
   read-only data section. Near-free (ShapeId is known at every site) and it is the exact
   trace-time metadata. *(Do now if cheap; else Phase 2.)*
3. **Keep arenas shape-homogeneous where practical** so layout is recoverable from the page
   header (tag-free), avoiding an 8-byte header on 16-byte nodes. *(Prefer now.)*
4. **Shadow stack of pointer-typed locals at safepoints** — the highest-priority and least-
   retrofittable GC prerequisite, but **deferred**: not needed for arenas (which free en
   masse and never trace). Codegen should merely *know* which locals are pointer-typed so
   the spill is a later addition, not a re-architecture.

## Escape analysis — worked examples

Notation: `@iter` per-iteration loop arena, `@fn` function arena, `@global` program/fallback
heap. `@r` = region-polymorphic parameter.

### binary-trees (flagship)

```
function bottomUpTree(depth) @r {                 // region-polymorphic factory
  if (depth <= 0) return { left: null, right: null } at r;
  return { left: bottomUpTree(depth-1) @r,
           right: bottomUpTree(depth-1) @r } at r;
}
// stretch tree: built, checked, dropped — never stored outward
{ let st = bottomUpTree(stretchDepth) @stretch;   // NoEscape
  check(itemCheck(st)); }                          // reset @stretch  O(1)
let longLived = bottomUpTree(maxDepth) @fn;        // ESCAPES block -> @fn
for (let d = minDepth; d <= maxDepth; d += 2) {
  for (let i = 1; i <= iterations; i++) {
    let t = bottomUpTree(d) @iter;                 // NoEscape w.r.t. iteration
    sum += itemCheck(t);
  }                                                // reset @iter  O(1)/iteration
}
check(itemCheck(longLived));                        // longLived still live -> @fn
```

- The inner-loop `t` is read only within its iteration ⇒ `NoEscape` ⇒ `@iter`, reset every
  iteration. **This turns O(work) live memory into O(1)/iteration — the reason N=21 fits.**
- `longLived` is read after the loop ⇒ escapes ⇒ `@fn`.
- A node points at its children ⇒ children must outlive the parent ⇒ same arena as the
  parent; a whole tree is one arena's pages, freed as a unit.
- Canonical N=21 live set ≈ one long-lived depth-21 tree (~4M nodes × 16 B = 64 MB) + one
  transient tree + brief stretch tree — fits the 256 MB default budget *with* memory.grow
  and per-iteration reset.

### n-body

Five body objects in a top-level array, mutated in place for thousands of steps ⇒ all
`@global`, never reset. The analysis must **not** mis-assign them to a loop arena. The
integration loop allocates nothing, so arenas add zero overhead here — the guarantee is
"no regression."

### spectral-norm

`makeVector` is region-polymorphic; a returned vector is allocated in the caller's arena
(escapes), while function-local scratch vectors are `NoEscape` and reset on return. Canonical
"some vectors escape, some don't."

## Soundness invariants (the hard gate)

For every stored pointer `a.f = b`, array-element write, or captured variable, the region
of the pointee `b` must **outlive** the region of the holder `a`. An arena may be reset only
if no value reachable from a surviving root (the scope's result, any outer-arena object, any
global) points into it. Enforced by:

- may-escape over-approximation (bias every imprecision to "escapes");
- an outlives check on every store/return, emitted as a hard compile error when violated
  (reject-don't-miscompile — never silently reset a region that something outer points into);
- closures/captured locals modeled in the transfer functions (a captured local that outlives
  its scope escapes) — if the fixture subset avoids escaping closures, that path may be
  rejected rather than analyzed this cycle, but must not be silently mis-assigned;
- adversarial unit tests on return / store-into-outer / capture patterns.

## Phased delivery plan

- **Phase 0 — `memory.grow` + centralized `__alloc` helper** *(+ string-escape lexer lane)*.
  Independently valuable: lifts the 1 MB wall, makes the sandbox memory budget real, lifts
  binary-trees to ~N=15, and lands the `__alloc` seam. String escapes are orthogonal and
  small. Ships as its own PR(s).
- **Phase 1 — escape-analysis arenas.** MIR escape-analysis pass, page-based arenas +
  free-page list, per-loop/per-function arena open/reset, region-polymorphic factory
  lowering, outlives soundness gate, fallback-to-global for escapers. **Reaches canonical
  binary-trees N=21.** The proving fixture lands here.
- **Phase 2 — deferred (design-noted):** size-classed fallback heap; shadow stack + full GC
  forward-compat; eventual copying/generational GC. Built when profiling or real programs
  demand individual-object reclamation.

## The binary-trees fixture

- Kali-normalized TS port (`for (i = i + 1)` idiom, no `+=` reliance beyond proven surface,
  `1 << k` shift for iteration counts — a proven bitwise-lane op).
- Canonical output at N=21 with **real TAB** separators (needs the Phase 0 escape lane):
  `stretch tree of depth 22\t check: …`, `N\t trees of depth d\t check: …`, `long lived
  tree of depth 21\t check: …`.
- Runs under plain `kali run` if N=21 fits the default budget after reclamation; otherwise
  via a scoped `--sandbox` policy raising `maxMemoryMB` (mirroring mandelbrot's canonical-
  via-policy precedent). Which one is determined empirically once Phase 1 lands.
- Fixture set mirrors the others: `binary-trees-benchmark-v1.ts`, `.json` metadata
  (sha256, buildModes), an end-to-end runtime test `clbg_binary_trees_runtime.rs`, and a
  new feature-maturity row.

## Testing strategy

- **Phase 0:** `memory.grow` unit tests (allocation past 1 MB succeeds; OOM past the sandbox
  cap traps cleanly with a diagnostic, not a wild pointer); `__alloc` refactor keeps all
  existing object/array/n-body/mandelbrot fixtures byte-identical; string-escape lexer tests
  (each recognized escape; unknown/`\x`/`\u` escapes rejected with a diagnostic).
- **Phase 1:** escape-analysis unit tests (NoEscape vs escapes-to-caller vs escapes-global on
  return/store-into-outer/capture); arena reset correctness (no live pointer into a reset
  arena — adversarial); n-body/spectral-norm remain byte-identical (no mis-assignment
  regression); binary-trees runs end-to-end and matches canonical N=21 output byte-for-byte.
- **Regression:** the full existing 5-crate green gate stays green; `kali_fmt` clean.

## Risks / pitfalls

- **Region explosion** (MLKit's central lesson): without the per-loop arena, loop allocations
  pile into the function arena and grow monotonically. The per-loop arena is mandatory, not
  optional. Consider lightweight arena-size profiling if retention is suspected.
- **The escaping-value trap:** failing to promote `longLived` (or a returned vector) and
  resetting its arena = dangling pointer + silent corruption. Mitigated by the one-directional
  escape bias, the outlives gate, and adversarial tests.
- **Single bump pointer can't host two live arenas** — page-based arenas are required, not a
  saved-pointer shortcut.
- **`memory.grow` returns −1, never traps** — every growth path must check it; a missed check
  turns OOM into a later hard trap.
- **`__alloc` as the first internal helper** perturbs function-index bookkeeping — the
  refactor must keep host-import indices (17–20) and `FUNCTION_INDEX_OFFSET` consistent.
- **A complementary GC is eventually needed** for graph/cache/indefinite-lifetime data no LIFO
  region can reclaim; the fallback heap is architected to *become* GC'd without disturbing the
  arena fast path.

## References

- Tofte & Talpin, *Region-Based Memory Management*, Information and Computation 132(2), 1997.
- Tofte & Birkedal, *A Region Inference Algorithm*, ACM TOPLAS 20(4), 1998.
- Hallenberg, Elsman & Tofte, *Combining Region Inference and Garbage Collection*, PLDI 2002.
- Grossman et al., *Region-Based Memory Management in Cyclone*, PLDI 2002.
- Choi et al., *Escape Analysis for Java*, OOPSLA 1999.
- Berger, Zorn & McKinley, *Reconsidering Custom Memory Allocation* (Reaps), OOPSLA 2002.
- Gay & Aiken, *Memory Management with Explicit Regions*, PLDI 1998.
- Leijen, Zorn & de Moura, *Mimalloc: Free List Sharding in Action*, MSR-TR-2019-18, 2019.
- Evans, *A Scalable Concurrent malloc(3) Implementation for FreeBSD* (jemalloc), BSDCan 2006.
- Google, *TCMalloc design*. Blackburn & McKinley, *Immix*, PLDI 2008.
- Cheney, *A Nonrecursive List Compacting Algorithm*, CACM 13(11), 1970.
- Appel, *Simple Generational Garbage Collection and Fast Allocation*, SP&E 19(2), 1989.
- Elsman & Hallenberg, *Integrating region memory management and tag-free generational GC*, JFP 2020.
- Wingo, *walloc*; Wellons, *WebAssembly: How to allocate your allocator*, 2025.
- V8, *Bringing GC languages to WebAssembly (WasmGC)*, 2023; WebAssembly core spec (memory).
