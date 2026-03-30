# 03 — AST

## Design Principles

- **Dedicated node types**: Each AST node is a distinct Rust type where practical, not a catch-all enum. Use enums for true categories such as `Expression`, `Statement`, and `Declaration`; keep semantic typing information in later checker side tables rather than implying the parsed AST is already type-checked.
- **Arena-allocated**: All nodes live in a per-file arena. References are indices or arena pointers.
- **Span-annotated**: Every node carries a `Span` for error reporting and source mapping.
- **Immutable after construction**: The AST is built once by the parser and never mutated.

## Core Node Types

### Top-Level
```rust
struct Module {
    body: Vec<ModuleItem>,
    span: Span,
}

enum ModuleItem {
    Statement(Statement),
    ImportDeclaration(ImportDeclaration),
    ExportDeclaration(ExportDeclaration),
}
```

### Statements
- `BlockStatement`, `VariableDeclaration`, `ExpressionStatement`
- `IfStatement`, `SwitchStatement`
- `ForStatement`, `ForInStatement`, `ForOfStatement`, `WhileStatement`, `DoWhileStatement`
- `ReturnStatement`, `ThrowStatement`, `TryStatement`
- `BreakStatement`, `ContinueStatement`, `LabeledStatement`
- `FunctionDeclaration`, `ClassDeclaration`
- `WithStatement` (flagged for sandbox/strict-mode analysis)
- `DebuggerStatement`

### Expressions
- `Identifier`, `Literal` (number, string, boolean, null, bigint, regexp)
- `BinaryExpression`, `UnaryExpression`, `UpdateExpression`, `LogicalExpression`
- `AssignmentExpression`, `ConditionalExpression`, `SequenceExpression`
- `CallExpression`, `NewExpression`, `MemberExpression`, `OptionalChainExpression`
- `ArrowFunctionExpression`, `FunctionExpression`
- `ObjectExpression`, `ArrayExpression`, `SpreadElement`
- `TemplateLiteral`, `TaggedTemplateExpression`
- `YieldExpression`, `AwaitExpression`
- `ClassExpression`
- `MetaProperty` (`new.target`, `import.meta`)
- `ImportExpression` (dynamic import syntax; semantic support is phase-gated later)

### TypeScript-Specific Nodes
- `TypeAnnotation` — wraps any type node
- `TypeReference`, `TypeLiteral`, `TupleType`, `UnionType`, `IntersectionType`
- `ConditionalType`, `MappedType`, `IndexedAccessType`, `InferType`
- `FunctionType`, `ConstructorType`
- `TypeParameter` with constraint and default
- `InterfaceDeclaration`, `TypeAliasDeclaration`, `EnumDeclaration`
- `AsExpression`, `SatisfiesExpression`, `NonNullExpression`
- `TypePredicateAnnotation` (`x is T`)

### JSX Nodes
- `JsxElement` — `<Foo>...</Foo>`, opening + closing tags
- `JsxSelfClosingElement` — `<Foo />`
- `JsxFragment` — `<>...</>`
- `JsxAttribute`, `JsxSpreadAttribute`
- `JsxExpression` — `{expr}` within JSX
- `JsxText` — literal text content

### Kali-Specific Nodes
- `EffectAnnotation` — effect summary on function signatures: `! FileSystem.Read | Network.Fetch` *(parsed early; semantically enabled from the Phase 2 target onward)*
- `PureModifier` — `pure function f() { ... }` *(parsed early; semantically enabled from the Phase 2 target onward)*
- `EffectDeclaration`, `PerformExpression`, `HandleExpression` for algebraic effects/handlers *(later experimental surface; not part of the Phase 1 MVP and not a separate canonical status label)*

These nodes may appear in the AST before the corresponding feature has semantic support. The parser records the syntax; later phases of checking/lowering decide whether the construct is currently enabled according to [specs/19-feature-maturity.md](19-feature-maturity.md).

Simplification rule:
- the AST should add dedicated node kinds only for genuinely distinct syntax/semantics, not for every later compatibility switch
- for example, `ImportExpression` needs its own node because `import()` is a distinct ECMAScript syntactic form
- by contrast, the `Function()` constructor path is represented through ordinary call/new-expression nodes and is phase-gated semantically under the shared `eval` compatibility switch rather than by inventing a second AST node family just for that compatibility path

### Patterns (Destructuring)
- `IdentifierPattern`, `ObjectPattern`, `ArrayPattern`
- `AssignmentPattern` (with default), `RestElement`

## Node ID System

Each AST node gets a unique `NodeId` (u32) assigned during parsing. This enables:
- O(1) lookup of type information by node
- Side tables for resolved types, symbols, etc.
- No need to store type info directly on AST nodes

## Symbol Table

Built during the name-resolution phase of the type checker (see [specs/04-type-system.md](04-type-system.md)):
- `Symbol` — represents a named entity (variable, function, class, type, etc.)
- `Scope` — tree of lexical scopes with parent pointers
- Each `Identifier` node is linked to its `Symbol` via `NodeId → SymbolId` map
- Handles hoisting (`var`, function declarations), TDZ (`let`, `const`), and TypeScript declaration merging
