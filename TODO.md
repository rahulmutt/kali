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

### Stage 1.6 - HIR/LIR Lowering
- ✅ Deterministic AST/statement → HIR lowering implemented
- ✅ HIR → MIR lowering implemented
- ✅ MIR → LIR lowering implemented
- ✅ Representative parser-backed lowering tests pass

## Next Work

### Stage 1.8 - Runtime Execution
- [x] Wire wasmtime execution to the emitted WASM modules
- [x] Add smoke tests for `kali run` and `kali test`
- [x] Keep the compiler pipeline deterministic and runnable end-to-end
- [ ] Add the Deno-oriented host surface (`console`, `fetch`, timers, filesystem)
- [ ] Implement the guest-side `Kali.test(...)` registration protocol
- [ ] Expand runtime coverage to the Stage 1.8 fixture set (`hello.ts`, async/timer, mocked fetch, invalid traps)
