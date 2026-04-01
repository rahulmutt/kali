# Kali Compiler - TODO and Task Tracking

## Current Status

### ✅ Phase 1 Foundation - COMPLETE
1. **Workspace Scaffold (Stage 1.1)** ✅
   - All core crates created
   - Workspace Cargo.toml configured
   - Proof-ready baseline established

2. **Error Handling & Common Utilities** ✅
   - `kali_common`: FileId, Span, SourceMap implementation
   - `kali_error`: Diagnostic system with full error code registry
   - **FIXED**: Serde support added to all diagnostic types

3. **Stage 1.2 - Lexer (kali_lexer)** ✅
   - Token enumeration complete
   - Lexing implementation functional
   - Unit tests passing (7 tests)
   - Span propagation working

### 🚧 Phase 1 Core Implementation - IN PROGRESS
1. **Stage 1.3 - Parser & AST (kali_parser, kali_ast)** 🚧
   - Parser skeleton exists with stub implementation (needs actual parsing)
   - AST has basic node types with serde support but needs significant expansion
   - Arena allocation not yet implemented - need to replace Vec-based approach
   - Missing: Full ECMA-262 + TypeScript parsing implementation
   - Missing: Pratt parsing for expression precedence
   - Missing: Full NodeKind variants (~80+ needed)
   - Missing: Error recovery mechanism
   - Needs: Complete integration with lexer output
   - Priority: 🔴 HIGH - Implementing now

### 📋 Backlog (Phase 1)
1. Stage 1.4 - Name Resolution
2. Stage 1.5 - Type Checker
3. Stage 1.6 - HIR & LIR Lowering
4. Stage 1.7 - WASM Code Generation
5. Stage 1.8 - Runtime & Execution
6. Stage 1.9-1.14 - Supporting infrastructure

## Next Task: Stage 1.3 - Parser & AST

**Blocked by:** ✅ All dependencies satisfied (lexer is complete)

**Dependencies:**
- `kalI_lexer` output tokens

**Key Requirements:**
1. Update ParseSource to accept Token stream
2. Expand NodeKind enum (~80 variants needed)
3. Implement arena-based AST allocation
4. Add recursive descent parser
5. Implement error recovery
6. Write comprehensive tests

## Notes

Serde fix was a foundational requirement - without it, the entire pipeline is unusable. All diagnostic types now support JSON output as required by the spec.
