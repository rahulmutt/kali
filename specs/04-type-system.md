# 04 — Type System

## Overview

Kali's type system is a superset of TypeScript's, combining:
1. **Flow-sensitive typing** (like tsc) — narrowing through control flow
2. **Hindley-Milner inference** — global type inference for unannotated code
3. **Effect types** — tracking side effects for sandboxing
4. **Constraint solving** — for advanced generic inference

The type checker operates on the AST + symbol table and produces a `TypedAST` (AST with resolved type information in side tables keyed by `NodeId`).

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
- Unification follows standard Algorithm W, extended for:
  - Row polymorphism (object types with unknown fields)
  - Subtyping (TypeScript's structural subtyping)
  - Effect polymorphism

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
```rust
struct EffectType {
    /// Set of effects this function may perform
    effects: EffectSet,
}

enum Effect {
    IO,                     // Any I/O operation
    FileSystem(FsAccess),   // File system access (read, write, delete)
    Network(NetAccess),     // Network access (listen, connect, fetch)
    Process(ProcAccess),    // Process operations (spawn, exit, env)
    Timer,                  // setTimeout, setInterval
    Random,                 // Math.random, crypto
    Eval,                   // eval, Function constructor
    Dom,                    // DOM manipulation
    Custom(InternedString), // User-defined effects
}
```

### Effect Inference
- Effects are inferred bottom-up: leaf functions determine effects, callers accumulate
- Annotated effects (`! Effect`) are checked against inferred effects
- Effect polymorphism: generic functions can be polymorphic over effects
- `pure` functions have an empty effect set — enforced by the checker

### Relationship to Sandboxing
The effect system feeds directly into the sandbox analyzer (see [specs/09-sandboxing.md](09-sandboxing.md)):
- Static `--effects` flag outputs all effects as JSON
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
- **Full program inference**: infer types across module boundaries without annotations
- **Effect annotations**: `function read(path: string): string ! FileSystem`
- **Purity checking**: `pure function add(a: number, b: number): number`
- **Refined types** (future): `type PositiveInt = number & { __brand: "positive" }`
- **Constraint propagation**: more aggressive generic resolution than tsc

## Type Checking Phases

1. **Name resolution**: Build symbol table, resolve imports/exports
2. **Declaration processing**: Process type aliases, interfaces, class declarations
3. **Inference**: Walk function bodies, generate constraints, run unification
4. **Narrowing**: Apply flow-sensitive narrowing at each CFG node
5. **Effect analysis**: Infer and check effect annotations
6. **Validation**: Check all constraints are satisfied, report errors

Each phase operates per-module with cross-module dependencies resolved lazily (query-based).
