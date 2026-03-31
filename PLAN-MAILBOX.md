# Plan Mailbox - Observations and Suggestions

## Fix Commit Summary

During implementation work on the parser and AST task (Stage 1.3), I discovered and resolved a critical blocker:

### Issue: Missing Serde Support in Error Handling Crate

**Problem:**
- The `kali_codegen` crate was trying to derive Serialize/Deserialize for `CodegenResult`
- This required `Diagnostic` type to also support serialization
- The `kali_error` crate had conditional serde support that wasn't properly configured with feature flags

**Resolution:**
1. Fixed `kali_error` Cargo.toml to define proper `serde` feature
2. Updated `diagnostic.rs` and `severity.rs` to conditionally derive Serialize/Deserialize
3. Manually implemented custom Serialize/Deserialize for `Severity` enum (deserializing from string)
4. Updated workspace Cargo.toml and dependencies

**Files Modified:**
- `/workspace/crates/kali_error/Cargo.toml` - Added proper serde feature
- `/workspace/crates/kali_error/src/diagnostic.rs` - Added conditional serde derives for Diagnostic, Span, FileId
- `/workspace/crates/kali_error/src/severity.rs` - Added custom Serialize/Deserialize impl for Severity enum
- `/workspace/crates/kali_codegen/src/lib.rs` - Fixed diagnostics type imports
- `/workspace/Cargo.toml` - Updated kali_error dependency

**Status:** ⚠️ BUILD BLOCKER RESOLVED

This was a foundational fix that needed to be in place before Stage 1.3 implementation because:
1. All diagnostics types need serializability for JSON output requirements
2. The codegen stage depends on working Diagnostic types
3. Without this fix, the entire compilation pipeline is unusable

## Observations for Stage 1.3 (Parser & AST)

After reviewing the current state:

### kali_parser Status:
- Skeleton exists but only stub implementation (`parse()` returns input unchanged)
- ParseSource struct needs to be updated to consume Token stream from lexer instead of plain text
- ParseResult type is defined but full AST output type is missing

### kali_ast Status:
- Basic Node and NodeKind types exist but are incomplete
- Missing: Arena-based allocation for efficient memory management
- NodeKind enum only has ~30 variants, needs expansion to cover full ECMA-262 + TypeScript

### Recommendations:

1. **Update Parser to use Lexer output**: The TODO.md mentions the parser should accept tokenized input, not raw text. This needs to be addressed.

2. **Expand NodeKind enum**: Need to add all required variants from SPEC.md Stage 1.3 requirements:
   - Full statement types (Try, Catch, Finally, Throw, Debugger, Labeled, Return, With, etc.)
   - All expression types (Arithmetic, Logical, Relational, Assignment, Conditional, etc.)
   - TypeScript-specific types
   - JSX types (JSXElement, JSXFragment, etc.)

3. **Implement Arena Allocation**: Use rust-arena or similar for efficient AST node management

4. **Add comprehensive tokenizer integration**: Ensure Token spans are correctly propagated to AST nodes

### Next Steps:

- [ ] Update `ParseSource` to accept `Vec<Token>` from lexer
- [ ] Expand `NodeKind` enum to full coverage
- [ ] Implement arena-based node allocation
- - [ ] Implement recursive descent parser for basic constructs
- [ ] Add proper error recovery mechanism
- [ ] Add snapshot tests with fixture files
