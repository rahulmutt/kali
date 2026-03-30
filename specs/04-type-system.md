# 04 — Type System

## Overview

Kali's type system is a superset of TypeScript's, combining:
1. **Flow-sensitive typing** (like tsc) — narrowing through control flow
2. **Bounded inference beyond plain TypeScript** — inference for unannotated code where it improves on TypeScript without sacrificing predictable compile times
3. **Effect summaries** — tracking side effects for sandboxing
4. **Constraint solving** — for advanced generic inference

Bootstrap-normalization note:
- the bootstrap's references to Haskell, Idris, Agda, Lean, and Rust are interpreted here as **design guidance**, not as an immediate promise of dependent types, totality checking, proof terms, or theorem-prover-style user workflows in Phase 1
- in practice, early Kali should borrow principled ideas about inference, purity, effects, and constraints while still behaving like a pragmatic TypeScript superset with explicit annotation boundaries and predictable compile costs

Implementation order matters:
- **Phase 1**: preserve TypeScript compatibility, ship **first-class JavaScript compilation** from [SPEC.md](../SPEC.md), and apply the shared **bounded inference contract** to locals, unannotated parameters where the call/context makes them obvious, and function return types when the body stays within the cheap local-inference fragment.
- **Phase 2**: extend that inference more confidently across module boundaries, stabilize the built-in capability-effect model, and expose the user-facing effect-report/effect-annotation surface.
- **Phase 3 target**: expand bounded advanced constraints where compile-time cost stays predictable.
- **Later compatibility**: effect polymorphism and other higher-complexity type-system extensions land only after the built-in capability/effect contract is stable.

The type checker operates on the raw AST and produces a `TypedAST` (AST with resolved type information in side tables keyed by `NodeId`). Name resolution is the first phase of type checking — it builds the symbol table and scope tree from the AST.

## JavaScript-First Inference Rules

Kali's **first-class JavaScript compilation** contract means plain JavaScript input is a real compilation target, not a second-class compatibility mode.

Early-phase rules:
- the shared **executable/analyzable source-file class** from [SPEC.md](../SPEC.md) goes through one parser, resolver, and checker pipeline, with module-kind interpretation following the canonical resolver/runtime rules instead of ad hoc extension-specific shortcuts
- declaration-only files remain side inputs only, per the shared **canonical source-file classes** rule in [SPEC.md](../SPEC.md)
- missing annotations generate inference variables rather than immediate `any`
- inference prefers stable, local conclusions over clever whole-program guesses
- when inference cannot prove a precise representation cheaply, the compiler keeps the program valid by using `unknown`, unions, or dynamic/tagged layouts instead of inventing fragile static assumptions
- JSDoc types are treated as contextual type hints where present, but are not required for efficient compilation

This keeps JavaScript support aligned with the project goal: compile ordinary JS efficiently when the program is analyzable, and degrade conservatively when it is not.

Module-boundary simplification rule:
- within a module, Kali may use the full bounded-inference fragment to recover efficient layouts and call signatures
- at exported/public boundaries, if a precise signature would require open-ended cross-module search, prefer an explicit annotation or a conservative exported type over a clever unstable inferred API
- this keeps plain-JavaScript package compilation practical without letting inferred public APIs become phase-dependent guesswork

### Canonical JavaScript Fallback Contract

To keep `.js` compilation predictable across the checker, IR, and codegen, Kali uses the following fallback ladder when precision is insufficient:

1. **Precise static type + stable layout**
   - Use when inference can prove a concrete primitive/object/function shape cheaply.
   - This is the preferred path for optimized compilation.

2. **Union of a small known set of cases**
   - Use when control flow or local inference yields a bounded set such as `string | number` or `{x:number} | null`.
   - Lowering may still optimize via tagged unions or specialized branches.

3. **`unknown` at the type boundary**
   - Use at module/API boundaries or escape points when Kali cannot justify a stronger public type.
   - `unknown` is preferred over `any` because it preserves type safety and forces narrowing before unsafe use.

4. **Dynamic/tagged runtime value representation**
   - Use when values flow through operations that require runtime discrimination and cannot be kept in a small static union cheaply.
   - This is an IR/runtime representation choice, not a license to erase type errors.

5. **Dynamic object layout**
   - Use when property sets, key spaces, prototype interactions, or mutation patterns are not statically stable enough for a fixed layout.
   - Objects in this state remain valid but lose fixed-layout optimizations until later analysis can recover them.

6. **Reject rather than guess**
   - If the program uses a feature whose semantics are outside the current phase/profile contract, Kali must emit the canonical feature-maturity diagnostic instead of silently widening to `any` or pretending the feature is supported.

Additional rules:
- `any` is preserved when it comes from TypeScript semantics or explicit user intent, but Kali should not invent fresh `any` merely to avoid analysis work.
- Falling back from a fixed object layout to a dynamic one is a representational downgrade, not a type-system escape hatch.
- Fallback decisions should be monotonic: later passes may refine `unknown`/dynamic representations into more precise ones, but they must not assume precision that earlier analysis failed to establish.

## Type Representation

```rust
enum Type {
    // Primitives
    Never, Unknown, Any, Void, Undefined, Null,
    Boolean, Number, String, BigInt, Symbol,

    // Literal types
    BooleanLiteral(bool),
    NumberLiteral(f64),
    StringLiteral(InternedString),

    // Compound types
    Union(Vec<TypeId>),
    Intersection(Vec<TypeId>),
    Tuple(Vec<TupleElement>),
    Array(TypeId),
    Object(ObjectType),
    Function(FunctionType),
    Class(ClassType),

    // TypeScript advanced
    Conditional { check: TypeId, extends: TypeId, true_type: TypeId, false_type: TypeId },
    Mapped { key_type: TypeId, value_type: TypeId, modifiers: MappedModifiers },
    IndexedAccess { object: TypeId, index: TypeId },
    Infer(InferTypeId),
    TemplateLiteral(Vec<TemplateLiteralPart>),
    Keyof(TypeId),
    Typeof(SymbolId),

    // Kali extensions
    Effect(EffectType),
    TypeVariable(TypeVarId),       // HM unification variable
    Constrained(TypeId, Vec<Constraint>),
}
```

Types are interned via `TypeId` (u32 index into a type arena) for cheap comparison and storage.

## Bounded Inference Contract

Follow the shared **bounded inference contract** and **annotation-required inference boundary** from [SPEC.md](../SPEC.md).

For the type checker, that means:
- Phase 1 inference should go beyond plain `tsc`-style local contextual typing, but stay inside that predictable bounded fragment
- when the checker would need unbounded search, repeated speculative instantiations, wide cross-module backtracking, or unstable public-API inference, it should stop the advanced path early
- the fallback remains the canonical JavaScript/TypeScript-safe set: explicit annotation requirement, `unknown`, unions, or a dynamic/layout-conservative representation
- the checker must not invent fresh `any` merely to keep inference moving

This is the main simplification that keeps stronger-than-`tsc` inference compatible with blazing-fast compilation: Kali supports a strong bounded fragment early and grows outward only where cost remains measurable and testable.

## Hindley-Milner Integration

### Unification
- Unification variables (`TypeVariable`) are generated for unannotated parameters and locals
- In practice, Kali uses a **hybrid inference engine**: TypeScript-style contextual typing and flow analysis first, then bounded unification where it is unambiguous and cheap.
- Unification is extended for:
  - Row polymorphism (object types with unknown fields)
  - Structural subtyping where it does not break principal inference
  - Effect polymorphism in later phases

### Let-Generalization
- Functions and let-bindings are generalized (polymorphic) when their type variables don't escape **and** the binding stays inside the shared **bounded inference contract**
- Public/module-boundary generalization should follow the shared **annotation-required inference boundary**: when principality or exported API stability is unclear, prefer an explicit annotation or a conservative boundary type over a clever inferred signature
- Monomorphization happens later (see [specs/07-specialization.md](07-specialization.md))

### Constraint Solving
```rust
enum Constraint {
    Equality(TypeId, TypeId),          // T = U
    Subtype(TypeId, TypeId),           // T <: U
    HasProperty(TypeId, String, TypeId), // T has property 'k' of type U
    Callable(TypeId, Vec<TypeId>, TypeId), // T is callable with args → return
    EffectSubset(EffectSet, EffectSet),    // effects(T) ⊆ effects(U)
}
```

Constraints are collected during inference and solved iteratively. Unsolved constraints become type errors.

## Flow-Sensitive Narrowing

Maintain a `TypeNarrowingMap` at each control-flow point:
- `typeof x === "string"` narrows `x` to `string`
- `x instanceof Foo` narrows `x` to `Foo`
- `x != null` narrows out `null | undefined`
- `if (x)` narrows out falsy types
- Type guards (`x is T`) create user-defined narrowing
- Narrowing through destructuring, switch/case, and pattern matching

Narrowing state is forked at branches and merged at join points (union of narrowed types).

## Effect System

Effect tracking follows the shared **effect-surface split** from [SPEC.md](../SPEC.md):
- **Phase 1** may maintain conservative built-in effect facts internally for sandbox-first implementation, lowering, and diagnostics
- the stable user-facing effect-report commands, explicit effect annotations, and policy-comparison workflow start in the Phase 2 target window
- later experimental/user-defined effect syntax must not accidentally leak into the stable Phase-1/2 machine contract just because the compiler has an internal representation for it

### Effect Types

Effect tracking is primarily a **capability summary system** for sandboxing. It is not required to expose full algebraic effects syntax in the initial implementation.
```rust
struct EffectType {
    /// Set of effects this function may perform
    effects: EffectSet,
}

enum Effect {
    FileSystem(FsAccess),        // e.g. FileSystem.Read, FileSystem.Write
    Network(NetAccess),          // e.g. Network.Fetch, Network.Listen
    Process(ProcAccess),         // e.g. Process.Spawn, Process.EnvRead
    Timer(TimerAccess),          // e.g. Timer.Schedule
    Random(RandomAccess),        // e.g. Random.GetBytes
    Eval,                        // eval, Function constructor
    Console(ConsoleAccess),      // e.g. Console.Write
    Custom(InternedString),      // User-defined effects (later/experimental `effect` declarations)
}

// Note: There is no `IO` super-effect. Each effect is tracked individually.
// Sandbox policies in specs/09-sandboxing.md map directly only to the built-in variants above.
// FsAccess, NetAccess, ProcAccess, TimerAccess, RandomAccess, and ConsoleAccess
// are sub-enums for finer-grained control and stable JSON names.
// `Custom(...)` is reserved for the later experimental algebraic/user-defined effect surface;
// it must not become part of the Phase 1-2 stable report/policy contract by accident.
```

Normalization rule:
- built-in effects are the only stable Phase-1/2 sandbox/report vocabulary
- internal compiler data structures may still carry placeholders such as `Custom(...)` so later effect-system work does not require a second representation
- machine-readable reports, schema-owned strings, and policy-checking semantics must continue to project that richer internal representation down to the documented built-in vocabulary until a later chapter explicitly broadens the contract

### Effect Inference
- Effects are inferred bottom-up: leaf functions determine effects, callers accumulate
- **Phase 2 target**: explicit effect annotations (`! Effect`) and `pure` modifiers are checked against inferred effects for the built-in sandbox-relevant capability set
- **Later compatibility**: effect polymorphism and user-defined/custom effect syntax may be added once the built-in capability model is stable
- Until that later phase lands, policy validation/comparison, diagnostics, and machine-readable effect reports are defined only for the built-in sandbox-relevant effect family
- `pure` functions have an empty effect set — enforced by the checker when the explicit effect-annotation surface is enabled

### Relationship to Sandboxing
The effect system feeds directly into the sandbox analyzer (see [specs/09-sandboxing.md](09-sandboxing.md)):
- `kali effects` is the Phase 2 user-facing command for emitting effect reports as JSON (see [specs/12-cli.md](12-cli.md))
- **Phase 1**: runtime sandbox enforcement works even when full static effect-policy validation is not yet exposed as a stable user-facing feature
- **Phase 2 target**: inferred effects are validated against sandbox policies at compile/check time

## Canonical Strictness Bundle

Kali exposes one top-level **strictness bundle** in schema v1: `compilerOptions.strict`.

To keep CLI/config simple in early phases, this is a bundle, not a menu of many near-duplicate booleans. The canonical intent is:
- `strict: true` *(default)* enables Kali's TypeScript-inspired strict checker behavior
- `strict: false` relaxes only the subset of diagnostics where Kali has a documented conservative fallback
- strictness changes type-checking diagnostics only; it must **not** change runtime semantics, sandbox enforcement, feature-maturity gating, or dependency-resolution behavior

Phase-1 contents of the strict bundle:
- strict nullability and flow-sensitive narrowing remain enabled as the default type model
- definite-assignment / use-before-safe-initialization checks stay enabled where Kali can prove them cheaply
- unsafe implicit-top-type behavior should be diagnosed instead of silently inventing fresh `any`
- exported/package-boundary inference should prefer explicit annotations or conservative boundary types over unstable clever guesses

Rules for `strict: false`:
- Kali may downgrade selected strictness diagnostics to warnings or accept conservative fallback types such as `unknown` where the program can still be compiled faithfully
- Kali must not use `strict: false` as permission to silently enable unsupported language/runtime features
- Kali must not weaken sandbox/effect diagnostics, ownership safety checks, or canonical `E5006` feature-availability failures
- the JavaScript fallback ladder from this chapter remains the same; `strict` is not a separate JS-vs-TS mode switch

Future schema revisions may split this bundle into named sub-options only when there is clear ecosystem value and the behavior can be specified without reintroducing ambiguous checker modes.

## TypeScript Compatibility

### Long-Term Compatibility Targets
These are the intended TypeScript-language compatibility targets for Kali overall, **not** a promise that every advanced checker feature is equally mature in Phase 1.

The canonical phase/maturity rules still live in [specs/19-feature-maturity.md](19-feature-maturity.md). For implementation planning, Phase 1 should prioritize the subset that unlocks real JS/TS projects and package compatibility before pursuing every advanced edge-case of the TypeScript type system.

Target feature families include:
- utility types (`Partial`, `Required`, `Pick`, `Omit`, `Record`, etc.)
- conditional types with `infer`
- template literal types
- mapped types with modifiers (`readonly`, `?`)
- declaration merging (interfaces, namespaces, enums)
- module augmentation
- `keyof`, `typeof`, indexed access types
- generic defaults and constraints
- overload signatures
- `this` types in classes
- discriminated unions
- `satisfies` operator
- `const` type parameters

### Extensions Beyond tsc
- **First-class JavaScript inference**: infer useful types for unannotated `.js` programs and module boundaries without forcing TypeScript migration first
- **Fuller program inference**: infer more types across module boundaries without annotations, but prefer predictable behavior over maximal cleverness
- **Boundary fidelity over guesswork**: cross-module/package types must follow the exact resolved import subpath rather than a package-wide shortcut; see the canonical declaration-resolution rules in [specs/14-packages.md](14-packages.md)
- **Effect annotations** *(Phase 2 target)*: `function read(path: string): string ! FileSystem.Read`
- **Purity checking** *(Phase 2 target)*: `pure function add(a: number, b: number): number`
- **User-defined/algebraic effects** *(later experimental surface)*: kept out of the MVP and introduced only after the sandbox capability model is stable
- **Refined types** (future): `type PositiveInt = number & { __brand: "positive" }`
- **Constraint propagation**: more aggressive generic resolution than tsc where compile-time cost stays bounded

## Type Checking Phases

1. **Name resolution**: Build symbol table, resolve imports/exports
2. **Declaration processing**: Process type aliases, interfaces, class declarations
3. **Inference**: Walk function bodies, generate constraints, run unification
4. **Narrowing**: Apply flow-sensitive narrowing at each CFG node
5. **Effect analysis**: Infer capability effects and, once Phase 2 support is enabled, check explicit effect annotations / `pure`
6. **Validation**: Check all constraints are satisfied, report errors

Each phase operates per-module with cross-module dependencies resolved lazily (query-based).

Cross-spec rule:
- package-boundary type information must be attached to the same resolved package/subpath edge chosen by the resolver/runtime pipeline
- when package declarations are incomplete or ambiguous for that exact edge, Kali should fall back conservatively (`unknown`, warning, or canonical availability failure) instead of borrowing unrelated package-root declarations
