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

- `kali_optimize` is now wired into the build pipeline and `release` / `release-advanced` run real,
  deterministic optimization passes instead of placeholders.
- The current optimizer performs constant folding, branch elimination, small-function inlining,
  aggressive dead top-level-function pruning, and MIR-aware call-site specialization for
  layout-stable callees.
- Layout-aware specialization now fingerprints struct and array descriptors precisely and keeps
  closure capture identities in the layout signature, so identical higher-order call-site layouts
  reuse one specialized clone while distinct closure capture sets no longer collapse onto the same
  specialization.
- Array-valued MIR bindings now preserve their element/length fingerprints through call-site
  specialization, so the array-layout widening can split into separate clones while same-shaped
  callers still share one clone instead of collapsing onto a single shared body.
- Direct array-literal call-site arguments now also carry explicit `Value:array:len=...` shape
  signatures, so the direct array-literal shape widening can split inline arrays with different
  lengths into separate clones even when the callee only sees a tagged parameter.
- Quoted string-literal call-site arguments now carry distinct specialization signatures, so
  different string literals can split into separate clones instead of collapsing onto the generic
  tagged fallback while still respecting the deterministic specialization budget.
- No-substitution template-literal call-site arguments now keep a template-specific signature, so
  backtick-delimited string literals stay distinct from quoted strings instead of collapsing onto
  the generic tagged fallback while still respecting the deterministic specialization budget.
- RegExp-literal call-site arguments now carry distinct canonical signatures, so `/foo/i` and
  `/bar/i` stay separate in the specialization path instead of collapsing onto the generic tagged
  fallback while still respecting the deterministic specialization budget, and the MIR-aware
  specialization path preserves that same regex-literal split when it has to run without layout
  metadata.
- `null` and `undefined` call-site arguments now also carry distinct literal signatures instead of
  collapsing onto the old zero-valued fallback, so the specialization path stays honest about
  nullish arguments when the MIR plan can see them as constants.
- Boolean call-site arguments now likewise preserve their `true` / `false` identity in the
  specialization signature, so tagged-parameter and MIR-backed hot paths do not collapse distinct
  control-flow constants onto one shared clone.
- Numeric-literal call-site arguments now also carry value-specific signatures, so repeated `1`
  calls still share a clone while `1` and `2` no longer collapse onto the same specialized body.
- Signed-zero numeric literals now preserve `-0` as a distinct specialization signature from `0`,
  so the literal-signature path stays honest about the JavaScript signed-zero edge case without
  changing the deterministic budget story.
- BigInt-literal call-site arguments now carry distinct `1n` / `2n` signatures as well, so the
  specialization path keeps BigInt constants separate from the old numeric fallback without
  changing the deterministic budget story.
- `Infinity`, `-Infinity`, and `NaN` call-site arguments now also carry distinct literal
  signatures, so the remaining special-number edge cases stay visible to the deterministic
  specialization path instead of collapsing onto the generic tagged fallback.
- Nested MIR-bound bindings inside object-literal call sites now also participate in the MIR-aware
  specialization signature, so composite arguments can split into distinct clones when the same
  surface shape is fed by different scoped binding layouts.
- Object-literal property order is now canonicalized in the specialization signature, so
  semantically identical object shapes with reordered fields reuse one specialized clone instead
  of splitting on insertion order.
- Const-bound object property reads and constant-index array reads now fold before codegen, and
  optimized numeric hot paths are regression-tested to stay free of tag-check / untag boxing.
- Incremental compilation now reuses `.kali-cache/incremental/` for unchanged modules, and the
  specialization budget is enforced per function owner so separate hot paths keep independent caps.
- Newly created MIR-specialized clones are recursively revisited under their own owner key, so
  clone-specific optimization can expose deeper specializable call sites without collapsing the
  parent function's deterministic budget accounting.
- A nested-call regression now proves a specialized clone can recursively surface a second
  specializable call site inside its own body, so the current depth story reaches past the first
  cloned layer without changing the deterministic budget model.
- Tagged-parameter call sites now specialize when the actual arguments have a concrete literal or
  MIR-backed layout, even when the callee is too large to inline but still small enough to stay
  within the deterministic specialization budget, so the monomorphisation path reaches one level
  deeper than the previous non-tagged-layout gate.
- MIR-backed binding layout lookups are now scoped by function owner, so identically named bindings
  in different functions can specialize independently instead of collapsing to one shared fallback
  layout.
- The pure-LIR optimizer path now also performs a concrete-argument fallback specialization for
  literal-shaped call sites when MIR layout metadata is unavailable, so generic/function
  specialization can still clone deterministic helpers even before the MIR-aware pass runs.
- The MIR-aware specialization path now also clones literal-shaped concrete-argument calls when the
  MIR plan has no parameter layout metadata, so the MIR pass can keep deterministic monomorphized
  helpers alive instead of dropping back to a generic no-specialization result.
- Identical generic specializations are now reused across owners when the callee and argument
  signatures already match, so the specialization path avoids cloning duplicate helpers just
  because the same generic call appears in more than one function scope.
- MIR-aware specialization now checks the specialization cache before spending the current
  owner's remaining budget, so an already-materialized clone can still be reused even after that
  owner has exhausted its specialization slots.
- A representative benchmark suite now records compile time, WASM size, instruction count, and
  add-op deltas across `fast`, `release`, and `release-advanced`.
- The release-advanced MIR-specialized clone path now keeps the generic-instantiation path available
  inside its specialized bodies, so a layout-specialized function can still clone and fold a large
  generic callee after the MIR pass narrows the arguments.
- MIR-specialized clones now keep generic specialization enabled inside their bodies in `release`
  too, so a layout-specialized wrapper can still clone and fold a large generic callee after the
  MIR pass narrows its arguments.
- `release_advanced` now has a regression that proves the same layout-specialized wrapper path still
  materializes a nested generic clone for a large callee, so the deeper monomorphisation path
  remains covered in the advanced mode as well.
- Nested generic specializations are now also reused across layout-specialized wrapper owners, so
  identical imported helpers stay deduplicated even when different caller shapes force separate
  wrapper clones.
- The new re-export-chain regression now also proves the same generic helper clone is reused once
  when the call flow runs through an explicit `public` / `bridge` / helper chain in release mode,
  and the bridge wrapper itself still specializes once, so the cross-module-style reuse story stays
  concrete without changing the deterministic budget model.

## Tasks

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

## Out of Scope

- LTO across Kali + user-provided native libraries (later compatibility).
- Profile-guided optimisation (later compatibility).
- `eval` / dynamic code generation (Phase 4 target).

## Definition of Done

- [x] `kali build --release` produces measurably faster or smaller WASM than `--fast` on the
  CI benchmark suite.
- [x] `kali build --release-advanced` produces a further improvement over `--release`.
- [x] Monomorphisation tests confirm no `TagCheck` / `Untag` instructions in specialised
  hot paths.
- [x] `compilerOptions.maxSpecializations` enforced; code-size explosion test passes, with the
  specialization budget scoped per function so separate hot paths keep independent caps.
- [x] Incremental compilation: a second build of an unchanged module is a cache hit (no
  recompile).
- [x] All Phase-1 and Phase-2 tests continue to pass without regression.
