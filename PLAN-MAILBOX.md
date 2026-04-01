# Plan Mailbox - Stage 1.3 Fixes Complete

## Issues Found

While implementing Stage 1.3 (Parser & AST), I discovered critical code issues blocking compilation:

### Issue 1: Parser lib.rs was truncated/incomplete
**File:** `crates/kali_parser/src/lib.rs` (line 98)
- The `Parser::new()` method had incomplete implementation ending with `lexer.`

### Issue 2: AST `Statement` enum was missing
**File:** `crates/kali_ast/src/lib.rs`
- No `enum Statement` definition despite many individual statement structs being defined
- Recursive type issues between `Statement` and statement structs (e.g., `IfStatement.body: Statement`)

### Issue 3: Duplicate NodeKind definitions
- Two `pub enum NodeKind` definitions in the AST file
- One with `Eq`/`PartialEq` derive, another with manual impl
- `NodeKind::Module` had `body: Vec<ModuleItem>` where `ModuleItem` didn't exist

### Issue 4: Missing dependencies
- `crates/kali_parser/Cargo.toml` missing `kali_ast` dependency
- Duplicate import aliases (`FileId` from both `kali_common` and `kali_lexer`)

## Fixes Applied

### 1. Fixed Parser
- Completed `Parser::new()` method implementation
- Added `parse()` stub method (returns empty parse - placeholder for Stage 1.3 full implementation)
- Fixed imports (removed unused `Lexer`, corrected `FileId` usage)

### 2. Fixed AST
- Added complete `Statement` enum with all statement variants
- Added `ModuleItem = Node` type alias
- Added `Box` indirection to fix recursive types:
  - `WithStatement.body: Box<Statement>`
  - `LabeledStatement.body: Box<Statement>`
  - `IfStatement.consequent: Box<Statement>`, `alternate: Option<Box<Statement>>`
  - `ForStatement.body: Box<Statement>`
  - `ForInStatement.body: Box<Statement>`
  - `ForOfStatement.body: Box<Statement>`
  - `WhileStatement.body: Box<Statement>`
  - `DoWhileStatement.body: Box<Statement>`
- Added `PartialEq` derive to `Node` struct (needed for `NodeKind::Module.body: Vec<Node>` PartialEq impl)
- Removed duplicate `NodeId` impl blocks
- Removed duplicate `NodeKind` enum (kept one with manual `PartialEq`/`Eq` impls)
  - Manually implemented `PartialEq` for `NodeKind` (can't use derive due to nested Vec inside Module)
  - Implemented `Eq` for `NodeKind`

### 3. Fixed Dependencies
- Added `kali_ast = { workspace = true }` to `crates/kali_parser/Cargo.toml`
- Fixed unused imports removed

## Build Status

```
cargo build -p kali_ast ✓ Compiles successfully
cargo build -p kali_parser ✓ Compiles successfully  
cargo test --workspace ✓ Passes (no test failures)
```

## Summary

The foundational AST and parser infrastructure for Stage 1.3 is now in place:
- ✅ AST definitions complete with typed structs and unified Statement enum
- ✅ Recursive type issues resolved with Box indirection
- ✅ Parser skeleton in place with basic parsing interface
- ⏳ Full recursive-descent parsing still needs implementation
- ✅ Module graph foundation in place (AST nodes can form program trees)

## Next Steps

The foundation is now ready for actual recursive-descent parsing implementation:
1. Implement `parse()` method to process token stream
2. Add expression parsing methods (binop precedence handling)
3. Add statement parsing methods
4. Add declaration parsing methods
5. Implement error recovery
6. Add snapshot tests with test fixtures
7. Implement full parse pipeline (lexer → parser → AST)
