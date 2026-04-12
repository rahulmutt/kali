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
- [x] Wire basic console host imports into the wasmtime linker (`console_log`, `console_error`, `console_warn`)
- [x] Add Phase-1 `kali test --filter` narrowing and phase-gate `--coverage` rejection
- [x] Add the Deno-oriented host surface subset (`fetch`, filesystem, env, args)
- [x] Add the timer / microtask scheduler surface (`setTimeout`, `setInterval`, `queueMicrotask`)
- [x] Implement the guest-side `Kali.test(...)` registration protocol
- [x] Add repo-backed fixture smoke coverage for `hello.ts`, test-suite discovery, and declaration-only rejection
- [ ] Expand runtime fixture coverage for the remaining Stage 1.8 edge cases (`async.ts`, `fetch.ts`, invalid-trap source fixture)
