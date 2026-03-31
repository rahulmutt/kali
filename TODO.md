# Next Tasks

## Current State Analysis

### Complete:
1. ✅ **Phase 1 Core Compilation Foundation** - Workspace scaffold, error handling, common utilities
2. ✅ **Stage 1.2 - Lexer (kali_lexer)** - Fully implemented, tests pass, clippy clean (except dependencies)

### Next Task: Stage 1.3 - Parser & AST (kali_parser, kali_ast)

**Dependencies**: lexer (1.2)

**Goal**: Implement a complete TypeScript/JavaScript parser that produces a full AST from tokenized source.

**Key Implementation Points**:

1. **Full ECMA-262 Parser** - Complete recursive descent parser handling:
   - All statement types (variable, function, class, if, for, while, switch, try, etc.)
   - All expression types (arithmetic, logical, relational, conditional, etc.)
   - Destructuring (patterns with defaults)
   - Generators and async generators
   - Optional chaining (`?.`), nullish coalescing (`??`)
   - Dynamic `import()`
   - Template literals with expressions

2. **TypeScript Extensions** - Parse TS-specific syntax:
   - Type annotations on variables, parameters, return types
   - Interfaces, type aliases, enums, mapped types
   - Generic type parameters
   - `as`, `satisfies`, non-null assertion (`!`)
   - JSX/TSX grammar

3. **ASI Handling** - Automatic semicolon insertion per ECMA-262 spec
4. **Error Recovery** - Synchronization tokens to continue parsing after errors
5. **Span Propagation** - All AST nodes carry source spans from lexer

**Testing Strategy**:
- Comprehensive unit tests for each statement/expression type
- Fixture tests with real TypeScript/JavaScript source files
- Error case tests for various parse failures
- Round-trip: tokenize → parse → verify structure

**Files to Update**:
- `/workspace/specs/19-feature-maturity.md` - Mark parser support as implemented for Phase 1
- `/workspace/specs/02-lexer-parser.md` - Update to reflect actual implementation status

**Phase 1 Completion Tracking**:
- Stage 1.2 is complete - can mark lexer feature as implemented in maturity matrix
- Next focus: Stage 1.3 parser implementation
- After parser: Stage 1.4 name resolution, Stage 1.5 type checker
