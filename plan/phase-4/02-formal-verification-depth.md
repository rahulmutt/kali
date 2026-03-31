# Stage 4.2 — Formal Verification Depth

**Phase:** 4 — Advanced Compatibility & Deep Verification  
**Spec refs:** [`specs/17-verification.md`](../../specs/17-verification.md), [`specs/16-testing.md`](../../specs/16-testing.md), [`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.1 — Workspace & Crate Scaffold](../phase-1/01-workspace-scaffold.md) (proof-ready baseline must exist from Stage 1.1; Lean modelling can begin once the type system design is stable, i.e. after Stage 1.5); proof-*backed* claims require a non-empty published boundary, which this stage delivers

## Goal

Advance from **proof-ready** (the repository baseline from Stage 1.1) to **proof-backed**: publish
a non-empty Lean 4 proof boundary that names at least one concrete modelled subsystem with actual
mechanized theorems. Enable **proof-backed** release/support claims once the published boundary
is non-empty.

## Workable Milestone

- `proofs/BOUNDARY.md` names at least one modelled subsystem (e.g. the type checker's
  core soundness properties) with a concrete theorem inventory.
- CI runs Lean proof jobs that verify those theorems on every commit touching `proofs/`.
- Release notes and documentation may cite formal verification for the published boundary.

## Tasks

### 1. Lean 4 proof tree setup

Create the `proofs/` Lean 4 workspace following the target structure from `specs/17-verification.md`:

```
proofs/
├── BOUNDARY.md               — updated to non-empty boundary
├── lakefile.lean             — Lean 4 build file
├── KaliCore/
│   ├── Types.lean            — core type calculus model
│   ├── Semantics.lean        — operational semantics
│   ├── Soundness.lean        — type soundness proof
│   └── Safety.lean           — memory safety properties
└── KaliIR/
    ├── HIRModel.lean         — HIR model
    └── LoweringCorrectness.lean — HIR → LIR lowering correctness
```

The Lean proofs target a **core Kali calculus** — not the full surface language. Exclude
late-compatibility features (`eval`, dynamic loading, browser/OS host details) which are handled
by phase gates in the implementation.

### 2. First proof-backed milestone: type soundness

Following `specs/17-verification.md`'s **First proof-backed milestone** guidance:

Model the core type system in Lean 4:

- Define the value set, type set, and typing judgements.
- Define small-step operational semantics for the core calculus.
- Prove **progress**: a well-typed program either is a value or can take a step.
- Prove **preservation**: if a well-typed term takes a step, the result is well-typed.

This is the standard type-soundness proof ("well-typed programs don't get stuck").

### 3. Memory safety properties

Model the ownership / reference-counting memory model from Phase 2:

- Define the memory model (linear memory + RC heap).
- Prove that well-typed programs with correct ownership annotations never produce dangling
  references (no use-after-free).
- Prove that the RC decrement path correctly frees all reachable objects (no leaks within the
  modelled subset).

### 4. HIR → LIR lowering correctness (within the modelled subset)

Prove that the HIR → LIR lowering preserves the semantics of the core calculus:

- For each HIR term in the modelled subset, the emitted LIR evaluates to the same value under
  the LIR operational semantics.

This is limited to the modelled subset (no `eval`, no dynamic dispatch beyond what the model
covers).

### 5. Update `proofs/BOUNDARY.md`

Replace the placeholder proof-boundary manifest with a concrete non-empty one:

```markdown
# Proof Boundary

## Current status: proof-backed

## Modelled subsystems

### Core type system (KaliCore/Types.lean, KaliCore/Soundness.lean)
- Type soundness: progress + preservation for the core Kali calculus
- Excludes: eval, dynamic import, browser/OS host interactions

### Memory safety (KaliCore/Safety.lean)
- No dangling references in well-typed programs with correct ownership annotations
- No leaks within the modelled ownership subset
- Excludes: cross-FFI pointers, native addons

### HIR → LIR lowering (KaliIR/LoweringCorrectness.lean)
- Semantic preservation for the core calculus subset
- Excludes: eval, dynamic dispatch beyond the model

## What is NOT claimed
- Proof coverage of the full surface language
- Proof coverage of the WASM host runtime (wasmtime)
- Proof coverage of browser/OS host API interactions
- Proof coverage of eval / dynamic features
```

### 6. CI proof jobs

Update the CI pipeline to run Lean proof jobs on every commit touching `proofs/`:

```yaml
proof-check:
  if: paths changed under proofs/
  runs-on: ubuntu-latest
  steps:
    - uses: leanprover/lean4-action@v1
    - run: cd proofs && lake build
```

A failing proof job blocks merges, ensuring the published boundary stays honest.

### 7. Update release claims

Update `README.md`, `specs/19-feature-maturity.md`, and any affected summaries to replace the
proof-ready canonical summary with the proof-backed boundary statement, quoting the updated
`proofs/BOUNDARY.md` verbatim for the claimed subsystems.

### 8. Tests

- CI proof job: `lake build` in `proofs/` succeeds on every commit.
- Anti-drift test: assert that `proofs/BOUNDARY.md` content matches the actual set of `*.lean`
  files in the repository (CI fails if a proof file is deleted without updating the boundary).
- Regression: adding a new proof file without updating `proofs/BOUNDARY.md` triggers a CI
  warning (not a block; the update is required but may follow).

## Out of Scope

- Proof coverage of the full Kali surface language (aspirational long-term goal).
- Proof coverage of wasmtime internals (wasmtime has its own verification program).
- Proof automation for dynamically-typed code paths (excluded from the modelled subset).

## Definition of Done

- [ ] `proofs/BOUNDARY.md` is non-empty and names the modelled subsystems.
- [ ] Lean proofs compile and pass: type soundness, memory safety, lowering correctness.
- [ ] CI proof job runs and blocks on proof failures.
- [ ] README and maturity matrix updated with proof-backed claims for the published boundary.
- [ ] All Phase-1 through Phase-4.1 tests continue to pass.
