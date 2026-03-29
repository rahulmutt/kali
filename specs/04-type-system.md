# 04 — Type System

## Overview

Kali's type system is a superset of TypeScript's, combining:
1. **Flow-sensitive typing** (like tsc) — narrowing through control flow
2. **Hindley-Milner-style inference** — inference for unannotated code where it improves on TypeScript without sacrificing predictable compile times
3. **Effect summaries** — tracking side effects for sandboxing
4. **Constraint solving** — for advanced generic inference

Implementation order matters:
- **Phase 1**: preserve TypeScript compatibility and flow-sensitive narrowing.
- **Phase 2**: add broader inference for locals, returns, and module boundaries.
- **Phase 3+**: expand effect polymorphism and more advanced constraints.

The type checker operates on the raw AST and produces a `TypedAST` (AST with resolved type information in side tables keyed by `NodeId`). Name resolution is the first phase of type checking — it builds the symbol table and scope tree from the AST.

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

## Hindley-Milner Integration

### Unification
- Unification variables (`TypeVariable`) are generated for unannotated parameters and locals
- In practice, Kali uses a **hybrid inference engine**: TypeScript-style contextual typing and flow analysis first, HM-style unification second where it is unambiguous and cheap.
- Unification is extended for:
  - Row polymorphism (object types with unknown fields)
  - Structural subtyping where it does not break principal inference
  - Effect polymorphism in later phases

### Let-Generalization
- Functions and let-bindings are generalized (polymorphic) when their type variables don't escape
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
    Custom(InternedString),      // User-defined effects (via `effect` declarations)
}

// Note: There is no `IO` super-effect. Each effect is tracked individually.
// Sandbox policies in specs/09-sandboxing.md map directly to these variants.
// FsAccess, NetAccess, ProcAccess, TimerAccess, RandomAccess, and ConsoleAccess
// are sub-enums for finer-grained control and stable JSON names.
```

### Effect Inference
- Effects are inferred bottom-up: leaf functions determine effects, callers accumulate
- **Phase 2 target**: explicit effect annotations (`! Effect`) and `pure` modifiers are checked against inferred effects for the built-in sandbox-relevant capability set
- **Later phase**: effect polymorphism and user-defined/custom effect syntax may be added once the built-in capability model is stable
- `pure` functions have an empty effect set — enforced by the checker when the explicit effect-annotation surface is enabled

### Relationship to Sandboxing
The effect system feeds directly into the sandbox analyzer (see [specs/09-sandboxing.md](09-sandboxing.md)):
- `kali effects` subcommand outputs all effects as JSON (see [specs/12-cli.md](12-cli.md))
- Sandbox policies are validated against inferred effects at compile time

## TypeScript Compatibility

### Supported Features
- All utility types (`Partial`, `Required`, `Pick`, `Omit`, `Record`, etc.)
- Conditional types with `infer`
- Template literal types
- Mapped types with modifiers (`readonly`, `?`)
- Declaration merging (interfaces, namespaces, enums)
- Module augmentation
- `keyof`, `typeof`, indexed access types
- Generic defaults and constraints
- Overload signatures
- `this` types in classes
- Discriminated unions
- `satisfies` operator
- `const` type parameters

### Extensions Beyond tsc
- **Fuller program inference**: infer more types across module boundaries without annotations, but prefer predictable behavior over maximal cleverness
- **Effect annotations** *(Phase 2 target)*: `function read(path: string): string ! FileSystem.Read`
- **Purity checking** *(Phase 2 target)*: `pure function add(a: number, b: number): number`
- **User-defined/algebraic effects** *(later, experimental)*: kept out of the MVP and introduced only after the sandbox capability model is stable
- **Refined types** (future): `type PositiveInt = number & { __brand: "positive" }`
- **Constraint propagation**: more aggressive generic resolution than tsc where compile-time cost stays bounded

## Type Checking Phases

1. **Name resolution**: Build symbol table, resolve imports/exports
2. **Declaration processing**: Process type aliases, interfaces, class declarations
3. **Inference**: Walk function bodies, generate constraints, run unification
4. **Narrowing**: Apply flow-sensitive narrowing at each CFG node
5. **Effect analysis**: Infer and check effect annotations
6. **Validation**: Check all constraints are satisfied, report errors

Each phase operates per-module with cross-module dependencies resolved lazily (query-based).
