# TODO

## Completed

### Stage 1.3 - Parser & AST
- ✅ Parser compiles successfully
- ✅ `cargo test -p kali_parser --lib` passes
- ✅ `cargo test -p kali_parser --test parser_integration` passes
- ✅ `cargo test --workspace` passes
- ✅ Parser handles variable declarations, blocks, functions, classes, control flow, try/catch, switch, debugger, throw, break/continue
- ✅ Parser handles primary expressions, function expressions, call chains, member access, binary expressions, and `new`
- ✅ Lexer fixes landed for punctuation advancement, `debugger`, and division tokens

## Next Work

### Stage 1.6 - HIR/LIR Lowering
- [ ] Implement TypedAST → HIR → LIR pipeline
- [ ] Add lowering tests for representative JS/TS snippets
- [ ] Keep the compiler pipeline deterministic and runnable end-to-end
