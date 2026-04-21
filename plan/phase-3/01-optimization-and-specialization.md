# Stage 3.1 — Optimization & Specialization

**Phase:** 3 — Specialization, Optimization & Ecosystem Breadth  
**Spec refs:** [`specs/07-specialization.md`](../../specs/07-specialization.md), [`specs/05-ir.md`](../../specs/05-ir.md), [`specs/01-architecture.md`](../../specs/01-architecture.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [2.1 — MIR & Ownership Analysis](../phase-2/01-mir-and-ownership.md) (MIR layout descriptors are the foundation for layout specialization)

## Goal

Implement `kali_optimize` — the specialization and optimization passes that make `--release` and
`--release-advanced` meaningfully faster than `--fast`. Introduce generic/function/layout
specialization so the `TaggedVal` uniform representation is replaced with unboxed scalars wherever
types are statically known.

## Workable Milestone

- `kali build --release` produces measurably faster/smaller WASM than `--fast` on representative
  compute benchmarks.
- Monomorphization eliminates `TaggedVal` tagging/untagging in statically-typed hot paths.
- Incremental compilation reduces re-compile time for projects with unchanged modules.
- Open-ended cross-module constraint solving is available for the **bounded inference contract**
  within the Phase-3 budget.

## Progress

- `kali_optimize` is wired into the build pipeline, and both `release` and
  `release-advanced` now run real deterministic optimization passes.
- The delivered optimization set includes constant folding, branch elimination, small-function
  inlining, aggressive dead-code pruning, MIR-aware call-site specialization, and incremental
  compilation reuse via `.kali-cache/incremental/`.
- Specialization keys are now materially richer and more stable: layout-aware fingerprints cover
  object/array descriptors, closure captures, literal distinctions (including string/template,
  regex, nullish, boolean, numeric, signed-zero, BigInt, and special-number cases), and owner-
  scoped MIR binding layouts.
- Clone reuse and budget enforcement are now deterministic across owners and nested specialized
  bodies, including cache-before-budget reuse and deeper specialization inside already-specialized
  wrappers.
- Regression and benchmark coverage now tracks `fast` vs `release` vs `release-advanced`, nested
  specialization depth, cross-owner reuse, and re-export-chain specialization behavior.

## Status

Stage 3.1 is complete.

Any further widening in this area is owned by the normative optimization/spec maturity docs rather
than by reopening this stage checklist. See:
- [`specs/07-specialization.md`](../../specs/07-specialization.md) for the optimization and specialization contract, and
- [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md) for maturity boundaries.

## Remaining Work

This stage's closed follow-up lane stays intentionally narrow:
- later optimization families such as LTO/profile-guided work remain tracked in [`specs/07-specialization.md`](../../specs/07-specialization.md)
- maturity wording stays controlled by [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)
- benchmark/regression evidence continues to live in the test suite rather than reopening the stage checklist

## Historical Stage Tasks

### 1. Generic / function specialisation (monomorphisation)

For each generic function instantiated with concrete type arguments, emit a specialised WASM
function that uses unboxed scalars instead of `TaggedVal`:

- `function add<T extends number>(a: T, b: T): T` instantiated with `T = number` → emits an
  `i64.add` (or `f64.add`) specialisation with no tag operations.
- Track specialisation depth under `compilerOptions.maxSpecializations` to prevent code-size
  explosion.

### 2. Layout specialisation

Use MIR layout descriptors (from Stage 2.1) to emit struct/array accesses using fixed-offset
`Load` / `Store` instead of generic `TaggedVal` property lookup:

- Objects with a statically known shape get a fixed layout in linear memory; field accesses are
  a single `i64.load offset=N` instruction.
- Arrays with a homogeneous element type get typed element loads (`f64.load offset=N*8`).

### 3. Optimisation passes (`kali_optimize`)

Implement optimisation passes that run after MIR lowering and before LIR emission:

**`fast` mode**: no passes (unchanged from Phase 1).

**`release` mode**:
- Constant folding and propagation.
- Dead code elimination (DCE).
- Inlining of small functions (configurable size threshold).
- Common subexpression elimination (CSE) within basic blocks.
- Branch simplification (known-dead branches removed).

**`release-advanced` mode** (in addition to `release`):
- Whole-program DCE across the full linked module.
- Interprocedural constant propagation.
- Loop invariant code motion.
- Optional user-provided WASM post-pass integration (e.g. `wasm-opt` as an external tool;
  must follow the **Pure-Rust implementation contract** — `wasm-opt` is a user add-on, not
  part of Kali's core pipeline).

### 4. Incremental compilation

Implement incremental compilation at the module level:

- Cache the compiled artifact for each module (keyed by content hash + flags).
- On re-build, only recompile modules whose source or transitive dependencies changed.
- The full-program link step still runs when any module changes.
- Cache stored in `.kali-cache/incremental/`.

### 5. Open-ended cross-module constraint solving (Phase 3)

Lift the **annotation-required inference boundary** for well-typed public APIs:

- Allow higher-cost solver work across module boundaries when the inference budget allows.
- Gate behind `compilerOptions.maxSpecializations` and the solver's compile-time budget.

### 6. Tests

- Benchmark suite: measure compile time and WASM binary size/execution speed for representative
  programs under `fast`, `release`, and `release-advanced`. Assert measurable improvement.
- Monomorphisation tests: assert that specialised functions for concrete type arguments contain
  no `TagCheck` / `Untag` instructions.
- Incremental compilation tests: assert that a second build of an unchanged module does not
  recompile (cache hit).
- All Phase-1 and Phase-2 tests continue to pass.

## Follow-up Tracking

This stage's remaining forward-looking work is already tracked normatively in:
- [`specs/07-specialization.md`](../../specs/07-specialization.md) for later optimization families such as LTO/profile-guided work,
- [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md) for what is publicly available, and
- benchmark/regression evidence in the test suite rather than a reopened stage checklist.
