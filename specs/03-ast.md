# 03 — AST

## Design Principles

- **Typed nodes**: Each AST node is a distinct Rust type, not a catch-all enum where possible. Use enums for categories (Expression, Statement, Declaration).
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
- `ImportExpression` (dynamic import)

### TypeScript-Specific Nodes
- `TypeAnnotation` — wraps any type node
- `TypeReference`, `TypeLiteral`, `TupleType`, `UnionType`, `IntersectionType`
- `ConditionalType`, `MappedType`, `IndexedAccessType`, `InferType`
- `FunctionType`, `ConstructorType`
- `TypeParameter` with constraint and default
- `InterfaceDeclaration`, `TypeAliasDeclaration`, `EnumDeclaration`
- `AsExpression`, `SatisfiesExpression`, `NonNullExpression`
- `TypePredicateAnnotation` (`x is T`)

### Kali-Specific Nodes
- `EffectAnnotation` — effect type on function signatures: `! Effect1 | Effect2`
- `SandboxDirective` — decorator-like policy annotation
- `PerformExpression` — `perform effectName(args)` for algebraic effects

### Patterns (Destructuring)
- `IdentifierPattern`, `ObjectPattern`, `ArrayPattern`
- `AssignmentPattern` (with default), `RestElement`

## Node ID System

Each AST node gets a unique `NodeId` (u32) assigned during parsing. This enables:
- O(1) lookup of type information by node
- Side tables for resolved types, symbols, etc.
- No need to store type info directly on AST nodes

## Symbol Table

Built during a post-parse name-resolution pass (or integrated into the type checker):
- `Symbol` — represents a named entity (variable, function, class, type, etc.)
- `Scope` — tree of lexical scopes with parent pointers
- Each `Identifier` node is linked to its `Symbol` via `NodeId → SymbolId` map
- Handles hoisting (`var`, function declarations), TDZ (`let`, `const`), and TypeScript declaration merging
