# Interprocedural Escape Flow — Design

**Date:** 2026-07-05
**Status:** Approved (brainstorm session, sections approved individually)
**Prerequisite for:** binary-trees Phase 1 Tasks 5–8 (arena codegen, N=21) — human ruling
2026-07-05: this round runs BEFORE any codegen reads the `ArenaTable`.
**Root-cause analysis:** `.superpowers/sdd/task-4-report.md` rounds 4–5b.
**Acceptance tests (pre-written):** the two `#[ignore]` xfail pins in
`crates/kali_mir/src/analysis/arena_gate_tests.rs`
(`ineligible_on_hoisted_function_launder`, `ineligible_on_param_mediated_escape`).

## Problem

Five review rounds against the Phase-1 arena escape gate kept finding fail-open launder
shapes with one shared root cause: **plain-ident dataflow (`a = b` assignments and param
binding at call sites) is unmodeled** in both the gate's store notes and the ownership
engine's escape machinery. Two members of the family survived intra-procedural patching and
are pinned as xfails:

1. **Hoisted-function-order launder** — `cache = x` is classified before the hoisted
   `helper` body (`x = mk()`) is walked, so the store is judged against a stale scalar
   layout. Any walk-order-sensitive judgment has this hole.
2. **Param-mediated escape** — `function retain(p) { sink = p; }` leaves
   `p.escapes == false` (a pre-existing engine bug: `is_heap_store_target` covers only
   member/chain LHS, so a plain-ident store's RHS is walked with `UseContext::Normal`).
   A caller passing a heap value to `retain` keeps `arena_eligible`, and the retained
   tree would dangle after the arena's exit reset.

Both shapes corrupt `arena_eligible` itself, so every consumer inherits the hole —
`opens_arena` AND `loop_arena` via a driver loop (round 5b: **no axis containment, only
pattern containment**). Cheap intra-function variants are provably wrong: all-idents dep
extraction vetoes `sum = sum + itemCheck(tree)`, the core loop-arena grant binary-trees
requires (round-5 Check 1).

## Decisions made during brainstorm

- **Scope: engine + gate.** The engine's own param-escape blindness is fixed in the same
  round, not just papered over for the gate. Blast radius is contained: `escapes` and
  `OwnershipClass` are consumed only inside `kali_mir` (the flag's one consumer is the
  walk itself, choosing `UseContext::Escape` vs `Normal` for call arguments).
- **Generality: full dataflow model.** Tri-state value judgment + flow edges + call-graph
  fixpoint, per the round-5 sketch. Per-shape patching demonstrably does not converge.
- **Approach A: one shared pass, two consumers.** A single new analysis module answers
  every heap-value and escape question; the engine and the gate both consume it. The
  gate keeps its site/loop/fate bookkeeping (which works and is pinned). Rejected:
  (B) parallel in-place fixes — keeps two heap judgments that can drift again, the exact
  structure that produced rounds 1–5; (C) collapsing the gate onto the engine — rewrites
  the five-times-reviewed 713-line `arena_gate.rs` wholesale for no acceptance-test gain.

## Architecture

New module: **`crates/kali_mir/src/analysis/escape_flow.rs`**, running after the ownership
walk collects raw facts and before the two judgment points: the
`BindingState → MirBinding` conversion (where `escapes = returned || escaped_via_flow ||
capture_escapes` is computed, `analysis/mod.rs`) and `ArenaCollector::into_facts` /
`compute_arena_table` (`analysis/arena_gate.rs`).

### Data structures

1. **`ValueClass`** — tri-state RHS judgment: `Heap`, `Scalar`, or `DependsOn(flow
   sources)`, where a flow source is a binding reference or a callee's return slot.
   Operator-aware: arithmetic/comparison/logical-on-scalars produce `Scalar` regardless of
   operand classes (`sum + itemCheck(tree)` is scalar no matter what `tree` is — this
   preserves the binary-trees grant); object/array literals are `Heap`; a bare ident is
   `DependsOn(binding)`; a call is `DependsOn(return-of-callee)`; ternary / logical-value /
   sequence positions join their branches.
2. **`FlowGraph`** — edges the walk records instead of judging in place: plain-ident
   assignment (`a = b` ⇒ edge `b → a`), declarator init, param binding at call sites
   (arg *k* → param *k* of the resolved callee), and return edges (returned value →
   callee's return slot). Store sites keep the gate's existing site distinction
   (Global vs ScopeLocal vs loop outflow) but carry a `ValueClass` instead of a resolved
   boolean.
3. **`FunctionSummary`** — per function: `param_escapes: Vec<bool>` ("param *i* is stored
   beyond the function's dynamic extent"), `stores_capture_outward` (set of outer binding
   names), `returns_heap` (tri-state until the fixpoint resolves it), and the existing
   `has_unknown_call`. "Stored beyond the dynamic extent" means any of: stored into a
   module-level binding, stored into a heap object's field/element, returned, or passed
   as an argument to a callee whose summary does one of these to that param (the
   transitive case is what the fixpoint resolves).

### Fixpoint

One worklist fixpoint over the call graph + intra-scope flow edges. Each binding /
return slot holds a may-heap bit; each param / capture holds a stored-outward bit. Edges
re-enqueue their targets when a source flips. Bits move only `false → true` (finite
monotone lattice), so termination is structural and the solution is **walk-order-independent
by construction** — the hoisted-function pin dies because there is no "before helper's body
was seen" state left to exploit.

### Consumers (no other behavior change)

- **Engine:** the fixpoint summary feeds the existing `escapes` disjunction as a fourth
  term (param/capture stored outward), fixing `sink = p`.
- **Gate:** `arena_is_heap_value` and the `fresh_heap_bindings` / `maybe_heap_bindings`
  sets are replaced by `ValueClass` + fixpoint results. The gate's fate axes, loop
  ordinals, and name-collision poisoning stay as-is. A call to a function whose summary
  says "param *k* escapes" taints the argument's source binding in the caller — the exact
  mechanism that flips `f`'s eligibility in the `retain(x)` pin.

## Pipeline order

All inside `kali_mir`'s `analyze`; no API change visible outside the crate.

1. The ownership walk runs exactly once, as today, but its arena hooks and `resolve_use`
   **record** flow edges and `ValueClass`-tagged store sites instead of resolving heap-ness
   against possibly-stale layouts.
2. `escape_flow::solve(graph) → Solution` runs the worklist fixpoint.
3. Engine consumer: `Solution` is consulted during `BindingState → MirBinding` conversion.
4. Gate consumer: `into_facts` / `compute_arena_table` consult `Solution` for every
   "is this value heap / does this callee leak my argument" question.

## Conservatism policy

Every unknown resolves toward veto:

- **Unknown callee** (indirect, imported, not module-level): assumed to store every
  argument outward and return heap — same stance as the existing `has_unknown_call`.
- **Recursion / mutual recursion:** no special casing; the fixpoint handles cycles.
  Bits start at ⊥ and monotonically rise, which is sound because every real store site
  injects taint independent of cycle order.
- **Same-name collisions:** the existing poisoning is kept and applied **before** the
  fixpoint (a poisoned function's summary is worst-case), so collision conservatism
  propagates interprocedurally too.
- **Unresolved `DependsOn`** (e.g. depends on an unknown import): treated as `Heap` at
  consumption time. Unresolved never means safe.
- **Higher-order flow** (functions as values, calls through bindings): out of precision
  scope — such a call is an unknown call (vetoes). Matches the existing gate stance;
  costs binary-trees nothing (its call graph is direct).

### Preserved round-3 asymmetry

Call results entering may-heap must NOT feed the gate's fate classification as fresh
allocation sites: `x = mk()` makes `x` heap-tainted (veto side) but is not a `ScopeLocal`
allocation of the caller (grant side — classifying it so would wrongly grant `opens_arena`
and dangle returned values after the exit reset). The `Solution` exposes these as distinct
queries (`may_heap(binding)` vs the walk's own `allocates`/site facts) so the round-3 bug
cannot be reintroduced by construction.

## Testing & acceptance

**Acceptance (pre-written):**

- Both xfail pins have `#[ignore]` removed and pass ALL assertions:
  `!loop_arena("g", 0)`, `!opens_arena("f")`, `!arena_eligible("f")` × both shapes.
- The 24 existing gate pins stay green **unchanged** — most critically the positive
  grants (`loop_arena_when_no_outflow` with `sum = sum + itemCheck(tree)`), the proof the
  tri-state judgment does not over-veto binary-trees.

**New tests** (unit tests beside `escape_flow.rs` plus gate-level pins):

- Tri-state judgment table: operator shapes → `Scalar`; literals → `Heap`; ident/call →
  `DependsOn`; ternary/logical joins.
- Transitive plain-ident chain (`x → keep → cache` across hoisted helpers) — the round-4
  shape the fixpoint claims to close.
- Launder-through-return (`function id(p) { return p; }` — heap-ness survives the round
  trip).
- Mutual recursion terminates and stays sound; unknown callee taints all args; poisoned
  collision propagates interprocedurally.
- Worklist-order independence: a test shuffles insertion order and asserts an identical
  solution.

**Engine-side verdict changes:** `sink = p` flipping `p.escapes` to `true` is a bug fix.
Existing ownership tests pinning the wrong verdict are corrected (each with a comment
citing this round) and a new engine-level pin locks the fixed behavior. Expected to be
few: `escapes` currently only feeds call-argument walk context.

**Regression bar:**

- `cargo test -p kali_mir` fully green (no ignored tests remaining in the gate suite).
- Standing 5-crate verification gate (lexer/common/types/codegen/cli) exit 0.
- `cargo fmt --check` clean.
- The four CLBG fixtures (nbody, spectral-norm, mandelbrot, fannkuch) byte-identical —
  the pass must be behavior-neutral for all shipped programs, since nothing reads
  `ArenaTable` in codegen yet.

**Determinism:** all new containers are `BTreeMap`/`BTreeSet`, matching the existing
collector.

## Delivery

Own branch + PR (push and merge after review, per the standing integration convention).
Then binary-trees Phase 1 Tasks 5–8 (page pool, loop arenas, function arenas, N=21
fixture) resume on a fresh branch with the gate now trustworthy. Respects the GC-less
invariant: this is escape analysis for region reclamation, not garbage collection.
