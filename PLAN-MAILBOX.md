# PLAN-MAILBOX - Stage 1.3 Notes

## Current Status

Stage 1.3 (Parser & AST) is **IN PROGRESS**. The lexer (Stage 1.2) is complete, and the AST skeleton and parser skeleton exist but need expansion.

## Findings

### What Exists
1. **AST Definitions** (`crates/kali_ast/src/lib.rs`):
   - Basic typed structs for statements (Statement enum with ~20 variants)
   - Basic expression types (Expression enum with 4 variants)
   - NodeKind-based legacy system (still present, not removed)
   - ASTBuilder and AST types with basic builder pattern
   - Import/Export declaration types
   - Some TypeScript type aliases (simplified)

2. **Parser Skeleton** (`crates/kali_parser/src/lib.rs`):
   - TokenStream wrapper
   - Parser struct with basic fields (file_id, stream, diagnostics, jsx_mode)
   - parse() stub returning empty result

3. **Lexer** (`crates/kali_lexer/src/lib.rs`):
   - **COMPLETE** - Full lexical grammar supporting ECMA-262 + TypeScript
   - Token types cover keywords, punctuators, literals
   - Error handling for unterminated strings/templates

### What Needs to Happen

1. **Expand Expression enum** from 4 variants to ~50:
   - Basic literals: Identifier, Literal, This, Super
   - Member: MemberExpr, CallExpr, NewExpr, MetaProperty
   - Unary: UnaryExpr (Delete, Void, Typeof, +, -, ~, !, ++)
   - Binary: BinaryExpr (all operators)
   - Advanced: ArrayLit, ObjectLit, SpreadExpr, TemplateExpr, TaggedTemplateExpr
   - Control: AwaitExpr, YieldExpr, ClassExpr, ArrowFunc
   - Logical: LogicalExpr (&&, ||, ??)
   - Assignment: AssignExpr (all assignment operators)
   - Update: UpdateExpr (++, --)
   - Others: SequenceExpr, ConditionalExpr, OptionalChainExpr

2. **Expand TypeScript type nodes**:
   - TsTypeRef, TsUnionType, TsIntersectionType, TsTupleType
   - TsArrayType, TsFunctionType, TsConstructorType
   - TsConditionalType, TsMappedType, TsIndexedAccessType
   - TsInferType, TsTypeQuery, TsTypePredicate, TsLiteralType

3. **Add JSX node types**:
   - JsxElement, JsxFragment, JsxOpeningElement, JsxClosingElement
   - JsxSelfClosingElement, JsxAttribute, JsxSpreadAttribute
   - JsxExpressionContainer, JsxText

4. **Add Kali-specific annotations**:
   - EffectAnnotation, PureModifier, ErrorNode (for error recovery)

5. **Refactor/remove legacy system**:
   - Consider removing NodeKind enum (not needed for typed AST approach)
   - Consolidate into single typed AST representation

6. **Implement parser core**:
   - Token iterator with proper lookahead
   - ASI (Automatic Semicolon Insertion)
   - Primary expression parsing
   - Member call expression parsing
   - Pratt/precedence climbing for operators
   - Statement parsing
   - TypeScript type annotation handling
   - JSX mode handling

7. **Error recovery**:
   - Panic-mode recovery with synchronization points
   - Error node creation for recovery

8. **Tests**:
   - Snapshot tests for various fixtures
   - Error recovery tests

## Decision: Continue with current approach

The typed AST approach (separate structs for each node type) is correct for Stage 1.3. The NodeKind enum-based system in `node.rs` is legacy and can be left for later migration or removal. The typed approach:

- Has better type safety
- Makes spans straightforward (each struct has a span field)
- Matches the spec requirements

## Next Task

**Expand the Expression enum in `crates/kali_ast/src/lib.rs` with comprehensive node types.**

This is the foundation - the parser implementation depends on all these types existing first. I'll implement:

1. All basic expression types (~15 types)
2. All statement types that are missing or simplified (~10 types)
3. All TypeScript type annotation types (~15 types)
4. JSX node types (~8 types)
5. Kali-specific annotations (3 types)

This will take several edits to the same file due to the size of changes.
