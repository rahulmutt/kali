# Stage 1.5 — Type Checker

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/04-type-system.md`](../../specs/04-type-system.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.4 — Name Resolution](04-name-resolution.md)

## Goal

Implement TypeScript-compatible type checking inside `kali_types`, including the **bounded
inference contract** for both TypeScript and first-class JavaScript source. After this stage
`kali check` provides genuine type-error feedback comparable to (and stricter than) `tsc`.

## Workable Milestone

- `kali check` reports type errors (`E3xxx`) on real TypeScript and JavaScript programs.
- The shared **bounded inference contract** is implemented: local variable types, obvious
  unannotated parameters, and analyzable return types are inferred; the
  **annotation-required inference boundary** is enforced for public/exported API surfaces.
- First-class JavaScript support: `.js` files receive conservative inference rather than
  being treated as untyped or silently widened to `any`.

## Tasks

### 1. Type representation

Define the `Ty` type in `kali_types`:

- **Primitive types**: `undefined`, `null`, `boolean`, `number`, `bigint`, `string`, `symbol`.
- **Literal types**: `true`, `false`, `42`, `"hello"`, `42n`.
- **Object types**: structural record of named properties; each property carries type, optionality,
  and readonly flag.
- **Function types**: parameter list (with labels, optionality, rest), return type, and type
  parameters.
- **Array / tuple types**: `T[]`, `readonly T[]`, `[A, B?, ...C[]]`.
- **Union / intersection types**: `A | B`, `A & B` — normalised at construction.
- **Generic types**: type parameters with optional constraints and defaults; instantiated lazily.
- **Conditional types**: `T extends U ? X : Y`.
- **Mapped types**: `{ [K in keyof T]: ... }`.
- **Template literal types**.
- **Indexed access types**: `T[K]`.
- **Special types**: `any`, `unknown`, `never`, `void`, `object`.
- **Infer type**: `infer T` placeholder inside conditional type positions.
- **Type predicates**: `x is T`.
- **Opaque error type**: `TyError` sentinel used when the type checker emits a diagnostic and
  needs to continue without propagating spurious cascading errors.

All `Ty` values are interned so structural equality is a pointer compare. Use a per-compilation
`TyInterner`.

### 2. Bounded inference contract

Implement inference following the Phase-1 **bounded inference contract** from `specs/04-type-system.md`:

| Context | Inference strategy |
|---|---|
| Local variable with initialiser | Infer from initialiser; `let x = 1` → `number` |
| Function parameter without annotation | Apply conservative inference: union of all call-site argument types within the **intra-module budget**; fall back to `unknown` when budget is exceeded |
| Function return type without annotation | Infer from all `return` statements reachable within the function; fall back to requiring annotation if cross-module |
| Exported / public API surface | Require explicit annotation; emit `E3020` if annotation is absent and inference would cross the **annotation-required inference boundary** |
| `.js` file inference | Use the same rules but fall back to `unknown` (not `any`) when no annotation is present and local inference is insufficient; never silently invent `any` |
| Budgeted local constraint solving | Allow bounded local/intra-module constraint solving under the compile-time budget; open-ended cross-module solving is Phase 3 |

### 3. Type checking algorithm

Walk the typed AST (post name resolution) and produce a `TypedProgram` side table that maps each
expression node to its type:

- **Literals**: assign their literal type (`42` → `42`, `"hi"` → `"hi"`, widened to `number` /
  `string` when assigned to a `let`).
- **Binary / unary expressions**: implement standard JS coercion rules for `+`, `-`, `*`, `/`,
  `%`, `**`, `===`, `!==`, `<`, `>`, bitwise operators, logical `&&` / `||` / `??`, nullish
  coalescing, optional chaining.
- **Assignment**: check assignability of RHS to LHS type.
- **Call expressions**: resolve overloads, check argument/parameter assignability, infer generic
  type arguments.
- **Member access**: resolve property on object type; report `E3030` for missing property.
- **Control flow narrowing**: `if` / `switch` / `while` conditions, `typeof`, `instanceof`,
  `in`, equality checks, and type predicates narrow the type of the tested binding inside the
  appropriate branch.
- **`await` / `yield`**: unwrap `Promise<T>` / `Generator<T>` as expected by the enclosing
  async/generator context.
- **TypeScript casts and assertions**: `x as T`, `<T>x`, `x!` (non-null assertion), `satisfies`.
- **Generics**: unify type arguments against constraints; produce concrete types for each
  instantiation site (lazy instantiation within the bounded budget).
- **Conditional types**: evaluate when the check type is concrete; defer when it contains
  free inference variables.

### 4. Assignability

Implement structural subtyping:

- A type `S` is assignable to type `T` if every property of `T` is present in `S` with a
  compatible type (covariant for read-only, invariant for mutable properties).
- `never` is assignable to everything; nothing is assignable to `never` except `never`.
- `unknown` is the top type; `any` is bi-directionally assignable.
- Union: `S` is assignable to `A | B` if `S` is assignable to `A` or `B`.
- Intersection: `S` is assignable to `A & B` if `S` is assignable to both.
- For `.js` source, use `unknown` rather than `any` as the fallback for unannotated positions
  so the assignability checker still catches obvious mismatches.

### 5. TypeScript declaration merging

Support the TypeScript-specific declaration merging rules:

- Interface merging (multiple `interface Foo` declarations merge their members).
- Namespace + value merging (`namespace Foo { }` alongside `function Foo() {}`).
- `enum` member access as values.
- `const enum` — inline member values at use sites.

### 6. First-class JavaScript support

Follow the shared **first-class JavaScript compilation** contract:

- `.js` files participate in the full type-checking pipeline; they are not a parse-only lane.
- Conservative fallback: when a `.js` file lacks JSDoc annotations and local inference is
  insufficient, use `unknown` (not `any`) and emit a bounded-inference note if the user has
  `--strict` enabled.
- JSDoc type annotations (`@param`, `@returns`, `@type`, `@typedef`) are parsed and used as
  type annotations for `.js` files; treat them equivalently to TypeScript annotations in the
  matching position.

### 7. Type error codes

Extend the `E3xxx` namespace with type error codes:

| Code | Meaning |
|---|---|
| `E3020` | Public/exported API surface requires explicit type annotation |
| `E3021` | Type is not assignable to expected type |
| `E3022` | Argument is not assignable to parameter type |
| `E3023` | Property does not exist on type |
| `E3024` | Object literal has excess properties |
| `E3025` | Cannot invoke non-callable type |
| `E3026` | Cannot use `await` outside async function |
| `E3027` | Cannot use `yield` outside generator function |
| `E3028` | Type parameter constraint not satisfied |
| `E3029` | Missing required property in object literal |
| `E3030` | Property access on possibly-null/undefined value |
| `E3031` | Type narrowing conflict (unreachable branch) |
| `E3032` | Circular type reference |

### 8. `--strict` flag

Implement the `--strict` / `compilerOptions.strict` flag that enables a set of stricter checks
(matching TypeScript's `--strict` defaults): `strictNullChecks`, `strictFunctionTypes`,
`strictPropertyInitialization`, `noImplicitAny` (maps to `unknown` fallback for Kali),
`noImplicitThis`.

When `strict` is off, `null` and `undefined` are assignable to every type (TypeScript lenient
mode); when on, they are narrowed from unions explicitly.

### 9. Checker baselines and regression tests

Following `specs/16-testing.md`:

- **Checker baselines**: for a set of fixture files, snapshot the full list of diagnostics
  (code, message, span). Assert against the golden baseline on every CI run.
- **Inference golden tests**: assert the inferred type of specific expression nodes matches the
  expected type string.
- **Regression tests**: one test per `E3xxx` code; assert the correct code is emitted.
- **`.js` inference tests**: fixture `.js` files with and without JSDoc; assert conservative
  `unknown` fallback and JSDoc annotation lifting.
- **Strict-mode toggle**: same fixture with and without `--strict`; assert different error sets.

## Out of Scope

- HIR lowering or code generation (Stage 1.6 onward).
- Inferred-effect-vs-policy comparison (Phase 2 target).
- Open-ended cross-module constraint solving (Phase 3 target).
- Dependent types, totality checking, or proof terms.

## Definition of Done

- [ ] `kali check` reports type errors on real TS/JS programs with accurate spans and codes.
- [ ] Bounded inference contract implemented and tested for both TS and JS source.
- [ ] Checker baseline snapshots committed and passing.
- [ ] `cargo test -p kali_types` passes including all type-error regression tests.
- [ ] No Stage 1.1–1.4 regressions.
