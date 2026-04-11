# Stage 1.3 — Parser & AST

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/02-lexer-parser.md`](../../specs/02-lexer-parser.md), [`specs/03-ast.md`](../../specs/03-ast.md)  
**Depends on:** [1.2 — Lexer](02-lexer.md)  
**Status:** ✅ Complete — parser and AST integration is working

### Completed Work

- TokenStream-based parser foundation
- Statement parsing for variable declarations, blocks, functions, classes, control flow, try/catch, switch, debugger, throw, break/continue
- Expression parsing for identifiers, literals, parenthesized expressions, function expressions, call chains, member access, binary expressions, and `new`
- AST support aligned with the parser for block-bodied control-flow nodes and class methods
- Lexer fixes for punctuation, `debugger`, and division tokens so parser tests terminate correctly

### Evidence

- ✅ `cargo build` succeeds
- ✅ `cargo test -p kali_parser --lib` passes
- ✅ `cargo test -p kali_parser --test parser_integration` passes
- ✅ `cargo test --workspace` passes
- ✅ 38/38 parser integration tests pass

### Workable Milestone

Stage 1.3 now provides a usable parser/AST layer for downstream compiler stages:
- token iteration and expression parsing
- control-flow and declaration coverage
- stable AST types for the currently supported statement and expression shapes

### Next Stage

Proceed to [1.4 — Name Resolution](04-name-resolution.md).
