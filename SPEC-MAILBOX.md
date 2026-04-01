# Spec Mailbox - Stage 1.3 Changes Required

## Observations and Required Spec Updates

### 1. AST Node Type Simplification

**Observation:** The current AST implementation has two systems that need merging:
1. Typed AST structs (in separate type files) with Span fields
2. NodeKind-based system (ast/src/lib.rs) with generic Node enum

**Recommendation:** The spec says to use typed enums as primary interface for better type safety. This matches what we've started but need to confirm in Spec.md or specs/03-ast.md.

### 2. E2xxx Error Code Registry

**Observation:** Stage 1.3 needs error codes E2001-E2009 as documented. Need to verify these are added to specs/15-errors.md.

**Action:** Add E2xxx error code registry:
- E2001: Unexpected token  
- E2002: Expected expression
- E2003: Expected identifier
- E2004: Expected delimiter (mismatched bracket/brace/paren)
- E2005: Invalid destructuring pattern
- E2006: Invalid assignment target
- E2007: Duplicate export
- E2008: Import/export outside module context
- E2009: TypeScript syntax in .js file (strict mode)

### 3. Node Span Annotation

**Observation:** Spec 03-ast.md requires ALL nodes to have span annotations, not just some. The current ASTBuilder uses Option<Span> for optional spans but this should be consistent.

**Recommendation:** All AST nodes should have required Span fields. The Option<Span> pattern should be removed for consistency with spec requirements.

### 4. Arena Allocation

**Observation:** Spec requires arena-based allocation with per-file Arena. Current implementation uses Vec<Node>. The TODO mentions implementing arena allocation.

**Action:** Need to implement proper arena allocation system where nodes live in Arena<ASTNode> references rather than direct Vec storage with NodeIds.

### 5. JSX/TSX Handling

**Observation:** The lexer needs to support JSX mode toggling. Currently lexer doesn't have JSX-specific token types or mode tracking. JSX disambiguation between `<T>` generics and JSX tags is a parser concern based on file extension.

**Action:** Update lexer to expose file extension information for JSX mode detection by the parser.

### 6. ASI (Automatic Semicolon Insertion)

**Observation:** Spec 02-lexer-parser.md says parser should handle ASI. The lexer should emit newline tokens as trivia for this purpose.

**Action:** Ensure lexer's newline tracking provides enough information for parser ASI logic.

## Priority Items for Stage 1.3 Before Implementation

1. **Add E2xxx error codes to specs/15-errors.md** - Required for parser errors
2. **Define arena allocation interface** - Should be part of common crate
3. **Clarify NodeKind enum approach** - Confirm typed enums as primary interface
4. **Update spec section on JSX handling** - File extension driven mode switching

## Non-Blocking Observations

- Current AST structure is good foundation - should keep it
- Typed structs approach is better than enum-based approach for type safety
- Can implement incrementally - start with basic types, add incrementally
