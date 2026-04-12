# Stage 2.4 — Lean Model Foundation

**Phase:** 2 — Ownership, Effects & Public Embedding  
**Spec refs:** [`specs/17-verification.md`](../../specs/17-verification.md), [`specs/04-type-system.md`](../../specs/04-type-system.md), [`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.5 — Type Checker](../phase-1/05-type-checker.md) (type system design must be stable before formalising); [1.1 — Workspace & Crate Scaffold](../phase-1/01-workspace-scaffold.md) (proof-ready baseline must exist); [2.1 — MIR & Ownership Analysis](01-mir-and-ownership.md) for the memory-safety proof task only — the type-calculus model and type-soundness proof may begin as soon as Phase 1 is complete

> **Parallelism note (see [`PLAN.md`](../../PLAN.md)):** the type-calculus modelling and type-soundness proof tasks (§1–§4) can proceed in parallel with Stages 2.1–2.3.  The memory-safety proof tasks (§5) require Stage 2.1 to be complete first.

## Goal

Lay the Lean 4 groundwork for formal verification: set up the `proofs/` workspace, model the core
Kali type calculus in Lean 4, and prove the foundational type-soundness properties (progress +
preservation) for a well-defined bounded subset of the language. Wire real (non-trivial) Lean
proof jobs into CI.

This stage advances Kali from "the Lean toolchain is available and trivially-passing stubs exist"
(Stage 1.1) to "real mechanised theorems are running in CI for the core type calculus". It is the
direct prerequisite for the **proof-backed** claims in Stage 4.2, which closes the formal-
verification programme by naming a non-empty published proof boundary and opening proof-backed
release claims.

## Workable Milestone

- The `proofs/` Lean 4 workspace contains real (non-trivial) Lean source files.
- The core type calculus is modelled in `KaliCore/Types.lean`.
- Progress and preservation are stated and proved for the bounded core calculus.
- CI runs the Lean jobs on every commit touching `proofs/`; a failing proof blocks merge.
- The published proof boundary in `proofs/BOUNDARY.md` is updated to reflect the provisional
  scope; the repository remains **proof-ready**, not yet **proof-backed** (that claim is
  reserved for Stage 4.2 when the boundary is declared non-provisional and non-empty).

## Tasks

### 1. Lean 4 workspace setup

Create the Lean 4 workspace under `proofs/`:

```
proofs/
├── BOUNDARY.md                     — provisional non-empty scope described
├── lakefile.lean                   — Lean 4 build file; all proof targets declared here
├── KaliCore/
│   ├── Types.lean                  — core type calculus model
│   ├── Semantics.lean              — small-step operational semantics
│   ├── Soundness.lean              — progress + preservation
│   └── Safety.lean                 — memory-safety stubs (filled in after Stage 2.1)
└── KaliIR/
    └── HIRModel.lean               — HIR model stubs (filled in during / after Stage 4.2 depth work)
```

The `lakefile.lean` must declare every `.lean` target so CI can validate completeness.

### 2. Core type calculus model (`KaliCore/Types.lean`)

Define the core Kali type calculus as a Lean 4 inductive type family. Scope the model to the
**bounded core calculus**: the subset of Kali that does not require `eval`, dynamic `import()`,
threaded execution, or browser/OS host-specific APIs.

Minimum definitions:

```lean
-- Value types
inductive Ty : Type where
  | TNever    : Ty
  | TUnknown  : Ty
  | TAny      : Ty
  | TVoid     : Ty
  | TUndef    : Ty
  | TNull     : Ty
  | TBool     : Ty
  | TNumber   : Ty
  | TBigInt   : Ty
  | TString   : Ty
  | TSymbol   : Ty
  | TLit      : LitVal → Ty          -- literal type
  | TFun      : List Ty → Ty → Ty   -- (params) → return
  | TObj      : List (String × Ty) → Ty
  | TUnion    : Ty → Ty → Ty
  | TInter    : Ty → Ty → Ty

-- Terms (core calculus)
inductive Expr : Type where
  | ELit   : LitVal → Expr
  | EVar   : String → Expr
  | EFun   : String → Expr → Expr   -- λ-abstraction
  | EApp   : Expr → Expr → Expr     -- application
  | ESeq   : Expr → Expr → Expr     -- sequence (stmt semicolon)
  | EIf    : Expr → Expr → Expr → Expr
  | EAssign: String → Expr → Expr
  | ETry   : Expr → String → Expr → Expr
  | EThrow : Expr → Expr
```

Keep the model small and self-consistent. The goal is a model that can be reasoned about, not a
full-fidelity encoding of every surface-language feature.

### 3. Small-step operational semantics (`KaliCore/Semantics.lean`)

Define a small-step evaluation relation `step : Expr → Expr → Prop` (or a big-step variant if
simpler for initial proofs). Include:

- Beta reduction for function application.
- Sequencing: the first expression steps to its value, then evaluation continues with the
  second expression.
- `if` dispatch based on the evaluated condition.
- Variable lookup from an environment / store.
- Throw propagation and `try`/`catch` handler invocation.

Define a value predicate `Value : Expr → Prop` identifying the fully-evaluated forms (literals,
closures).

### 4. Type-soundness proof (`KaliCore/Soundness.lean`)

Prove the standard type-soundness pair for the core calculus:

**Progress:** If `⊢ e : T` (expression `e` is well-typed with type `T`) and `e` is not a value,
then there exists an expression `e'` such that `step e e'`.

```lean
theorem progress : ∀ (e : Expr) (T : Ty),
    TypingJudgement [] e T → Value e ∨ ∃ e', step e e' := by
  ...
```

**Preservation:** If `⊢ e : T` and `step e e'`, then `⊢ e' : T`.

```lean
theorem preservation : ∀ (e e' : Expr) (T : Ty),
    TypingJudgement [] e T → step e e' → TypingJudgement [] e' T := by
  ...
```

Both proofs are restricted to the bounded core calculus. Exclusions must be documented in the
theorem statements or in a companion lemma list.

### 5. Memory-safety model stubs (`KaliCore/Safety.lean`)

*(Depends on Stage 2.1 — begin after MIR & Ownership Analysis is complete)*

Once the `OwnershipClass` model from Stage 2.1 is stable, populate `Safety.lean` with:

- A model of the ownership class annotations (`Stack`, `OwnedHeap`, `SharedHeap`, `Borrowed`).
- A statement of the no-dangling-reference property: a well-typed, well-ownership-annotated
  program never holds a reference to freed memory.
- Initial proof sketches or sorry-placeholders with a documented proof strategy.

Full mechanised proofs of memory safety are a Stage 4.2 goal; this task establishes the model
and proof structure so Stage 4.2 can fill them in without rework.

### 6. CI integration

Update the CI pipeline's `proof-check` job (introduced as a stub in Stage 1.1) to run real
Lean proof checks:

```yaml
proof-check:
  if: paths changed under proofs/
  runs-on: ubuntu-latest
  steps:
    - uses: leanprover/lean4-action@v1
    - run: cd proofs && lake build
```

A failing `lake build` must block the PR merge. The trivially-passing stub from Stage 1.1 is
replaced by this real job.

### 7. Update `proofs/BOUNDARY.md`

Update the proof-boundary manifest to describe the provisional scope:

```markdown
## Current status: proof-ready (provisional Lean model in progress)

## Modelled subsystems (provisional — not yet proof-backed)

### Core type calculus (KaliCore/Types.lean, KaliCore/Soundness.lean)
- Type soundness: progress + preservation for the bounded core Kali calculus
- Excludes: eval, dynamic import, browser/OS host interactions, threaded execution

### Memory safety (KaliCore/Safety.lean) — stubs only
- Model of ownership classes from Stage 2.1 MIR analysis
- No-dangling-reference property stated; proof to be completed in Stage 4.2

## What is NOT claimed
- Proof-backed release/support status (reserved for Stage 4.2)
- Proof coverage of the full surface language
- Proof coverage of eval / dynamic features
- Proof coverage of the WASM host runtime (wasmtime)
```

The repository remains **proof-ready**, not **proof-backed**, until Stage 4.2 publishes a
non-empty, non-provisional boundary and the CI proof jobs have been passing continuously.

### 8. Tests

- **CI proof job**: `lake build` in `proofs/` passes on every commit touching `proofs/`.
- **Completeness guard**: a CI script asserts that every `*.lean` file under `proofs/` is
  declared in `lakefile.lean`; adding an undeclared proof file fails CI.
- **Boundary consistency test**: assert that every modelled subsystem named in
  `proofs/BOUNDARY.md` corresponds to at least one `*.lean` file; removing a Lean file without
  updating the boundary fails CI.
- **Sorry-free gate (Phase 4 target)**: the presence of `sorry` in the type-soundness proofs is
  permitted in this stage as a placeholder strategy marker, but any `sorry` must be documented
  and tracked as a Stage 4.2 obligation; a CI warning (not block) is emitted for each `sorry`.

## Out of Scope

- Full proof-backed release claims (Stage 4.2 target — requires non-provisional, non-empty
  published boundary).
- Lowering-correctness proofs for HIR → LIR / MIR → LIR (Stage 4.2 depth).
- Proof coverage of `eval` / dynamic compatibility features (Stage 4.2, after Phase 4.1).
- Automated proof generation or LLM-assisted proof search.
- Proof coverage of Node or browser API semantics.

## Definition of Done

- [ ] `proofs/lakefile.lean` exists and declares all `.lean` targets.
- [ ] `KaliCore/Types.lean` defines the core type calculus including `Ty` and `Expr`.
- [ ] `KaliCore/Semantics.lean` defines the small-step evaluation relation and value predicate.
- [ ] `KaliCore/Soundness.lean` contains proved (or documented-sorry) progress and preservation
  theorems for the bounded core calculus.
- [ ] `KaliCore/Safety.lean` contains the ownership-class model (after Stage 2.1) and the
  no-dangling-reference property statement.
- [ ] CI `proof-check` job runs `lake build` and blocks on failure.
- [ ] Completeness guard and boundary consistency CI tests pass.
- [ ] `proofs/BOUNDARY.md` updated with provisional scope description.
- [ ] All Phase-1 and Phase-2 (1–3) tests continue to pass.
