# The negative-space array-literal predicate is a TRIPLET, not a single mistake — two copies are still live

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
