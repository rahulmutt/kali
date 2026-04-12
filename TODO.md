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

### Stage 1.4 - Name Resolution
- ✅ Resolver reports unresolved identifiers, duplicate bindings, and missing import targets
- ✅ `kali check` is wired to the resolver and passes CLI smoke coverage
- ✅ `cargo test -p kali_types --lib` passes
- ✅ `cargo test -p kali_cli --test runtime_smoke` passes
- ✅ `cargo test --workspace` passes

### Stage 1.6 - HIR/LIR Lowering
- ✅ Deterministic AST/statement → HIR lowering implemented
- ✅ HIR → MIR lowering implemented
- ✅ MIR → LIR lowering implemented
- ✅ Representative parser-backed lowering tests pass

### Stage 1.9 - Sandbox & Policy
- ✅ Declarative policy files parse and validate against schema v1
- ✅ `kali run --sandbox` enforces policy at runtime and reports `E4001` on violations
- ✅ `kali check --sandbox` validates policy schema/config without executing the program
- ✅ `kali build --sandbox` embeds the validated policy as `kali:policy` in the emitted WASM artifact
- ✅ Runtime policy enforcement and build embedding are covered by CLI/runtime tests

### Stage 1.10 - Package Management
- ✅ Manifest collision preflight now rejects registry identities that would materialize to the same `node_modules/` path
- ✅ Semver ranges now resolve deterministically to the highest matching published version
- ✅ Transitive install-path conflicts are rejected with `E6002`, and stale registry lock entries are pruned during `kali install`
- ✅ `kali install` now reconciles package cache and `node_modules/` state when the lock graph is already present
- ✅ Raw URL reconciliation now follows project-discovery/import-map declarations and prunes stale
  URL cache entries when the declaration graph changes

## Next Work
- [ ] Broader package-shape / host-fit diagnostics matrix coverage
- [ ] CLI integration coverage for install repair/prune scenarios
