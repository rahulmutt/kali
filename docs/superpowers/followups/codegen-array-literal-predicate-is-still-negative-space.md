# The negative-space array-literal predicate is a TRIPLET, not a single mistake — two copies are still live

> **CORRECTED 2026-09-11** by the **length-fails-closed** project
> (`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`). **§4's
> ranking is withdrawn, and the prescription in §3 and §4 is falsified by
> measurement.** That project picked this document as its item and spiked exactly
> the move §4 calls "small and well-scoped": both codegen copies narrowed to
> decline the one-child shape a `new` wrapper shares. The full workspace suite
> went from 0 to 32 failures, **three of them silent** (`const arr = [mk(0)]`
> printed `0` where its pin expects `201`, and two `E5506` fail-closed pins started
> exiting 0), and 5 of 12 one-element probe programs went from correct to silently
> wrong, with none improving (spec §2.2). In `kali_codegen`, "not an array literal"
> falls through to fallbacks that fabricate a number, so **narrowing this predicate
> is fail-OPEN until the consumer's floor refuses.** Two facts this document did
> not have: `new C(x)` and `[C(x)]` lower to the **same** LIR node (spec §2.1), so
> no shape check can tell them apart; and the predicate does produce one measured
> wrong value, `(new Array(3)).length` → `1` (node `3`), through the array-literal
> arms of `render_length` and `emit_unary`. That value is still open, and is filed
> in `length-fails-closed-discovered-defects.md` §3. The
> `docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/` directory and
> the `task-5-report.md`, `task-7-report.md` and `task-8-report.md` files cited
> below were never committed and do not exist in this repository.

> **Notice, 2026-09-12, by the inline-allocation-value-position project**
> (`docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md`,
> Task 6, retiring register entry **R-66**): the LIR collision this document
> describes — `new C(x)` and `[C(x)]` lowering to the same node — is now
> broken, **but only for allocation shapes, and only at the AST, not the
> LIR**. `crates/kali_types/src/resolve/expression.rs`'s
> `expression_is_array_allocation` now refuses a one-element array literal
> whose sole element is an `Array`/`Uint8Array` allocation (including through
> `new`, a bare call, `await`, `as`/`satisfies`, or a `globalThis`-qualified
> callee) *before* it ever reaches codegen, at type-resolution time — so the
> LIR-level collision this document's §1 and §2 describe is never actually
> exercised for an allocation on that path; the AST-level check catches it
> first and fails closed (`E5506`, both scopes; register R-66). **The
> underlying LIR node sharing this document is about is completely
> unchanged**: `[f()]` and `new f()` — an ordinary function call, not a
> recognized allocation — still lower to the identical LIR node described in
> §1, because `expression_is_array_allocation`'s refusal is scoped to the
> allocation-constructor shapes named above, not to "any call wrapped in a
> one-element array literal or a bare `new`." §1's claim of ~45 call sites
> still answering "is this an array literal?" the negative-space way, and §2's
> reach/consolidation argument, are both unaffected by this narrower,
> AST-level fix. See this repository's
> `inline-allocation-value-position-discovered-defects.md` for what that
> project measured and left open.

**Filed** 2026-09-10, by the **release-tier-allocation-identity** project
(`docs/superpowers/specs/2026-09-10-release-tier-allocation-identity-design.md`,
`docs/superpowers/sdd/2026-09-10-release-tier-allocation-identity/`), Task 8,
Part B1. **This is the highest-priority finding this project is filing that
it did not fix**, ranked above every other Part B item — see §4 for why.

## 1. The claim

Stage 1 of this project (Task 2) fixed exactly one instance of a
negative-space `is_array_literal` predicate — "a text-less `Value` node that
is not an object literal" — in `crates/kali_optimize/src/layout.rs`, replacing
it with a positive element check. **Two more copies of the identical
predicate exist, character-for-character the same mistake, and neither was
touched:**

```rust
// crates/kali_codegen/src/intrinsics/array.rs:5, FunctionEmitter::is_array_literal
pub(crate) fn is_array_literal(&self, node: &LirNode) -> bool {
    node.kind == LirNodeKind::Value && node.text.is_none() && !self.is_object_literal(node)
}
```

```rust
// crates/kali_codegen/src/lower.rs:7189, declarator_init_is_array_literal
// "Mirrors `FunctionEmitter::is_array_literal`" — its own doc comment says so.
pub(crate) fn declarator_init_is_array_literal(nodes: &[LirNode], init_id: LirNodeId) -> bool {
    // ... (unwraps sequence wrappers, then:)
    if node.kind != LirNodeKind::Value || node.text.is_some() { return false; }
    return !declarator_init_is_object_literal(nodes, id);
}
```

Both are in `kali_codegen`, a different crate from the one Stage 1 touched.
**The second is on the declarator-initializer path — the exact path the
Stage 1 defect lived on** (its own doc comment names
`FunctionEmitter::is_array_literal` as what it mirrors, and it is called from
two declarator-lowering sites, `lower.rs:5975` and `:6049`).

## 2. Reach

- `crates/kali_codegen/src/intrinsics/array.rs`'s `is_array_literal` (and the
  two functions built directly on it, `array_literal_contains_spread` and,
  transitively through the same file, `is_truthy_array_literal`) has **~43
  call sites across 9 files** in `kali_codegen` (`task-7-report.md`, §1c(ii)).
- `crates/kali_codegen/src/lower.rs:7189`'s `declarator_init_is_array_literal`
  has **2 callers**, both declarator-lowering call sites (`:5975`, `:6049`).

Every one of those ~45 call sites currently answers "is this an array
literal?" the same wrong way `kali_optimize`'s copy did before Stage 1: any
text-less `Value` node that is not an object literal qualifies, including a
node wrapping a `Call` (an allocation, `new Array(n)`, or a function call
whose result flows through) — the exact shape that produced this project's
headline defect (R-61 in the silent-miscompile register).

## 3. Why this project did not fix it

Two independent reasons, both already on record:

1. **`task-5-report.md`'s Ruling 2 investigation** traced whether
   `nbody-benchmark-v1`'s unfixed `E5506` refusal comes from
   `intrinsics/array.rs`'s copy (its own `array_bindings` registration,
   gated by `dynamic_array_read_base`, is a plausible but **not fully
   traced** candidate — the investigation stopped at its 20-minute budget
   once the core question, "does `nbody`'s binding still enter
   `kali_optimize`'s spec env?", was answered "no"). Whether either codegen
   copy is load-bearing for that specific refusal is undetermined.
2. **`task-7-report.md`'s Stage 2 sizing** measured the true behavioural
   blast radius of representing array-literal-ness positively in the LIR at
   **133 core sites across three crates** (`kali_codegen`, `kali_optimize`,
   `kali_lir`, plus a required `kali_mir` change the original plan never
   scoped) against a 15-site sizing gate — and correctly stopped rather than
   attempt a partial migration. That is a much larger undertaking than fixing
   these two predicate copies alone; see B12 in `task-8-report.md` (the
   report accompanying this filing) for the sizing-gate lesson that decision
   produced.

Fixing these two copies is **not** the same task as Stage 2. It is the
Stage-1-shaped move — replace negative space with a positive element check,
exactly as `kali_optimize`'s copy now does — applied twice more, in a crate
this project never touched. `task-7-report.md`'s own site list (§4, item 1)
names it as the first thing a follow-on project should do, because doing it
**shrinks** Stage 2: consolidating three copies into one before migrating the
representation means there is one predicate to migrate, not three to keep in
lockstep.

## 4. Why this ranks above every other Part B finding

Two reasons, stated plainly rather than left to be inferred from placement:

1. **It is the same defect class this whole project exists to close,
   unfixed, in the exact shape (the declarator-initializer path) that caused
   the headline defect.** Every other Part B finding (B2 through B13) is
   either a status report on fixtures this project did not restore, an
   instrument gap, a pre-existing unrelated defect, or a process lesson.
   This one is a live, traced, characterized instance of *this project's own
   root cause*, sitting one crate away from where it was fixed.
2. **It is small and well-scoped, unlike Stage 2.** Consolidating two
   predicate copies into a shared positive check (or into calls to the now-
   correct `kali_optimize` version, if the crate boundary allows it) is a
   bounded change with a clear TDD shape: write the same four unit tests
   Stage 1 used, watch them fail against the current negative-space bodies,
   fix the bodies, watch them pass. It does not require the `kali_mir`/
   `kali_lir` representation work Stage 2 needs, and it does not carry
   Stage 2's ~133-site blast radius — the fix is *narrowing what two
   functions accept*, not changing what kind of node an array literal is.

**In the judgment of the engineer who found it** (recorded in
`task-7-report.md`'s own Concerns §6, item 3): this "deserves its own
near-term project rather than waiting behind a full Stage 2." This document
agrees and elevates that judgment to Task 8's own ranking (see
`task-8-report.md`'s Part B grouping section) rather than leaving it as a
concern buried in a sizing report nobody reads once the sizing decision is
made.

## 5. What a follow-on project should NOT assume

- **This is not proven to be the cause of `nbody-benchmark-v1`'s refusal.**
  It is a plausible, named, *unconfirmed* candidate (§3, item 1). A follow-on
  project should trace it fully before claiming credit for fixing nbody —
  the refusal could equally be `intrinsics/array.rs`'s own separate
  `array_bindings` registration machinery (`emitter.rs:380`, populated at
  `emit/control_flow.rs:1634-1683`), which this project did not audit.
- **Fixing these two copies does not, by itself, close R-61's root-cause
  bullet's implication that a future allocating construct could reach the
  same hole through a different recognizer.** It closes the specific
  negative-space definition named here. The broader claim — that
  array-literal-ness should be represented, not inferred — is Stage 2's, and
  remains open (`task-7-report.md`).

## 6. Suggested home

Not the silent-miscompile register: this is a live code-shape finding about
an unfixed predicate, not a measured wrong value with a repro (no program has
been shown to hit `intrinsics/array.rs`'s or `lower.rs:7189`'s copy and
produce a silent divergence — the investigation that would establish one is
exactly what §3, item 1 says was not completed). It is filed here, as its own
document, per the instruction that this finding "deserves its own near-term
project" — a scope smaller than Stage 2 and worth starting before Stage 2 is
re-attempted, since it shrinks Stage 2's own blast radius by removing two of
the three copies a full migration would otherwise have to keep in lockstep.
