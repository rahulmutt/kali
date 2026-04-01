# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)

## Goal

Implement `kali_parser` and `kali_ast` — a recursive-descent parser that covers the full latest
published ECMA-262 grammar plus TypeScript syntax extensions, producing a concrete AST with full
span coverage, JSX support, and resilient error recovery.

## Workable Milestone

**CURRENT STATUS: Parser & AST in progress - AST skeleton exists, needs expansion**

✅ Lexer (Stage 1.2) complete  
⏳ AST node definitions - skeleton exists, needs comprehensive expansion  
⏳ AST Builder pattern exists, needs refinement  
⏳ Parser skeleton exists, needs full implementation  
⏳ Snapshot tests pending  
⏳ Error handling and recovery pending  

**NEXT DEVELOPMENT TASK:**
1. Expand AST Expression enum (~50 variants: literals, identifiers, binary/unary/logical operators, member/call expressions, array/object literals, template expressions, function expressions, arrow functions, class expressions, etc.)
2. Expand AST Statement enum (~20 variants: do-while, switch cases, labeled statements, empty statements, debugger statements)
3. Add all TypeScript type annotation nodes (~15 variants: union types, intersection types, tuple types, array types, function types, conditional types, mapped types, etc.)
4. Add JSX node types (~8 variants: elements, fragments, attributes, expressions, text)
5. Add Kali-specific annotations (EffectAnnotation, PureModifier, ErrorNode)
6. Implement parser core infrastructure (token iteration, lookahead, ASI, strict mode tracking)
7. Implement expression parsing via Pratt parsing (precedence climbing)
8. Implement statement parsing (declarations, control flow, try-catch, etc.)
9. Implement TypeScript type annotation handling
10. Implement JSX mode parsing with ambiguity resolution
11. Implement error recovery (panic-mode, synchronization points, ErrorNode)
12. Write snapshot tests for JS, TS, TSX fixtures

## Tasks

### AST node definitions (`kali_ast`)

Define the full node tree with proper Span annotations. The current AST module has two separate systems that need to be consolidated:

1. **Typed AST structs** (defined in separate types files) - Each AST node has its own type with a Span field
2. **NodeKind-based system** (in `ast/src/lib.rs`) - Uses a generic Node enum

**Consolidation Plan:**
- Keep the typed struct approach for Stage 1.3 since:
  - Each AST node already has Span via serialization/derivation
  - Better type safety for downstream stages
  - Matches spec requirements: "Nodes use typed enums rather than stringly-typed `kind` fields"
  - The NodeKind-based system in `ast/src/lib.rs` can remain but typed structs should be the primary interface

Primary node families (expand existing definitions in `kali_ast`):

| Family | Key Types |
|---|---|
| **Declarations** | `VarDecl`, `FunctionDecl`, `ClassDecl`, `InterfaceDecl`, `TypeAliasDecl`, `EnumDecl`, `NamespaceDecl`, `ImportDecl`, `ExportDecl` |
| **Statements** | `BlockStmt`, `IfStmt`, `ForStmt`, `ForInStmt`, `ForOfStmt`, `WhileStmt`, `DoWhileStmt`, `ReturnStmt`, `ThrowStmt`, `TryStmt`, `SwitchStmt`, `BreakStmt`, `ContinueStmt`, `LabeledStmt`, `ExprStmt`, `EmptyStmt`, `DebuggerStmt` |
| **Expressions** | `BinExpr`, `UnaryExpr`, `UpdateExpr`, `AssignExpr`, `CondExpr`, `CallExpr`, `NewExpr`, `MemberExpr`, `IndexExpr`, `SpreadExpr`, `TemplateExpr`, `TaggedTemplateExpr`, `AwaitExpr`, `YieldExpr`, `ArrowFunc`, `FuncExpr`, `ClassExpr`, `SequenceExpr`, `OptionalChainExpr`, `IdentifierExpr`, `LiteralExpr`, `ArrayLit`, `ObjectLit` |
| **TypeScript types** | `TsTypeRef`, `TsUnionType`, `TsIntersectionType`, `TsTupleType`, `TsArrayType`, `TsFunctionType`, `TsConstructorType`, `TsConditionalType`, `TsMappedType`, `TsIndexedAccessType`, `TsTemplateLiteralType`, `TsInferType`, `TsTypeQuery`, `TsTypePredicate`, `TsTypeAssertion`, `TsAsExpr`, `TsSatisfiesExpr`, `TsNonNullExpr`, `TsTypeAnnotation` |
| **JSX** | `JsxElement`, `JsxFragment`, `JsxOpeningElement`, `JsxClosingElement`, `JsxSelfClosingElement`, `JsxAttribute`, `JsxSpreadAttribute`, `JsxExpressionContainer`, `JsxText` |
| **Module** | `Module` (root node), `Script` (for script-mode files), `ImportSpecifier`, `ExportSpecifier`, `ExportDefault` |
| **Kali extensions** | `EffectAnnotation` (function effect summaries), `PureModifier` (pure function modifier) |
| **Pattern matching** | `ObjectPat`, `ArrayPat`, `RestPat`, `AssignPat`, `BindingIdent` |

### 2. Recursive-descent parser (`kali_parser`)

Implement a hand-written recursive-descent parser driven by the token stream from `kali_lexer`.

Key parsing concerns:

- **Operator precedence** — use Pratt parsing for expression precedence; avoids deep call stacks
  for operator-heavy code.
- **Automatic Semicolon Insertion (ASI)** — implement the exact rules from ECMA-262 §12.10 using
  the newline markers from the lexer.
- **Ambiguity resolution** — handle the classic TypeScript ambiguities:
  - `<T>` cast vs JSX opening tag (resolve by file extension / `--jsx` flag).
  - Arrow function `(params) =>` vs parenthesised expression.
  - `async` as identifier vs `async function` / `async arrow`.
  - `type` as identifier vs TypeScript type keyword.
- **JSX mode** — toggle JSX parsing when `.tsx` / `.jsx` extensions are active; the lexer emits
  basic bracket tokens and the parser drives token re-interpretation.
- **Strict mode** — track `"use strict"` directives and module-mode strictness; propagate to
  relevant parse rules.
- **Declaration-file mode** — `.d.ts` / `.d.mts` / `.d.cts` files are parsed in ambient context;
  implementations bodies are not expected.

### 3. Module system parsing

Parse all static module constructs defined for Phase 1:

- `import x from "mod"`, `import { x, y as z } from "mod"`, `import * as ns from "mod"`,
  `import type { T } from "mod"`, `import "mod"` (side-effect import).
- `export { x }`, `export default expr`, `export { x } from "mod"`,
  `export * from "mod"`, `export * as ns from "mod"`, `export type { T }`.
- `import.meta` (as an expression node).
- Dynamic `import(expr)` — parse to an `ImportCallExpr` node; semantic gating (rejected by
  default for non-literal in Phase 1) is handled by the type checker, not the parser.

CommonJS `require("literal")` is parsed as a call expression; the type checker / lowering phase
handles rewriting.

### 4. TypeScript extensions

Parse the full TypeScript surface beyond type annotations:

- Decorators (`@decorator` on class / method / parameter).
- Parameter properties (`constructor(private x: T)`).
- `abstract` classes and methods.
- `override` modifier.
- `satisfies` operator.
- `namespace` / `module` blocks (ambient and implementation).
- `enum` and `const enum`.
- Triple-slash reference directives (`/ // <reference ...>`).
- `declare` ambient context.

### 5. Error recovery

On a syntax error:

- Emit the appropriate `E2xxx` diagnostic.
- Attempt *panic-mode* recovery: skip tokens until a synchronization point (statement boundary,
  closing delimiter, or end of file) and resume parsing.
- Produce an `ErrorNode` placeholder so downstream stages see a complete tree.

Initial `E2xxx` error codes:

| Code | Meaning |
|---|---|
| `E2001` | Unexpected token |
| `E2002` | Expected expression |
| `E2003` | Expected identifier |
| `E2004` | Expected `)` / `]` / `}` (mismatched delimiter) |
| `E2005` | Invalid destructuring pattern |
| `E2006` | Invalid assignment target |
| `E2007` | Duplicate export |
| `E2008` | Import/export outside module context |
| `E2009` | TypeScript syntax in `.js` file (when strict-JS mode applies) |

### 6. Per-file arena allocation

The parser writes AST nodes into a per-file `Arena<T>`. Node references are typed arena indices
rather than raw pointers so lifetimes are explicit and bulk deallocation is O(1).

### 7. Unit and snapshot tests

- **Snapshot tests** (using `insta` or equivalent): parse a representative fixture file and assert
  the full AST matches a stored golden snapshot. Cover:
  - Pure JavaScript: declarations, control flow, closures, generators, async/await.
  - TypeScript: interfaces, generics, conditional types, decorators, enums.
  - JSX / TSX: element nesting, fragments, spread attributes, expression containers.
  - CommonJS `require` patterns.
  - Declaration files (`.d.ts`).
- **Error recovery tests**: files with deliberate syntax errors produce specific `E2xxx` codes and
  still yield a (partial) AST.

## Out of Scope

- Name resolution or type binding (Stage 1.4).
- Semantic validation beyond syntax (e.g. duplicate `let` in same scope — Stage 1.4/1.5).
- Stage-3+ TC39 proposals beyond the latest published ECMA-262 edition (rejected by default).

## Definition of Done

**Completed:**

- [x] Lexer (Stage 1.2) complete
- [x] Parser skeleton with TokenStream wrapper created
- [x] AST struct with nodes/root split from builder created
- [x] Unified Statement enum with basic statement variants
- [x] Basic Expression enum with 4 variants (Identifier, Literal, BinaryExpr, UnaryExpr)
- [x] Import/Export declaration types created
- [ ] AST Builder with arena-like Vec storage completed

**Pending:**

- [ ] Expand Expression enum to ~50 variants (all expression forms)
- [ ] Expand AST Statement enum to ~20 variants (all statement forms)
- [ ] Add all TypeScript type annotation nodes (~15 variants)
- [ ] Add JSX node types (~8 variants)
- [ ] Add Kali-specific annotations (EffectAnnotation, PureModifier, ErrorNode)
- [ ] Implement comprehensive parser (expressions, statements, types, JSX)
- [ ] Implement operator precedence parsing (Pratt parsing)
- [ ] Implement ASI (Automatic Semicolon Insertion)
- [ ] Implement error recovery with panic-mode
- [ ] Add E2xxx error codes (E2001-E2009 minimum)
- [ ] Create snapshot tests (JS, TS, TSX, D.TS fixtures)
- [ ] Error recovery tests with deliberate syntax errors
- [ ] `cargo test -p kali_parser -p kali_ast` passes
- [ ] `cargo clippy` passes with no regressions
