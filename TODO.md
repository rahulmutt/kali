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
- ✅ Package-shape and host-fit coverage now rejects Node-only host APIs surfaced through direct imports/requires with `E6005`
- ✅ Registry metadata lookups now use a process-local cache, avoiding redundant refetches during repeated resolution within one install run

### Stage 1.11 - Build Artifacts
- ✅ `kali build` now emits deterministic `kali:metadata` custom sections in the executable `.wasm` artifact
- ✅ `kali build --lib` now emits `.lib.wasm` plus a deterministic `.lib.meta.json` export inventory
- ✅ `kali build --bundle` now emits a browser bundle directory with `.wasm`, `.js`, and `.meta.json` outputs
- ✅ CLI/runtime smoke coverage exercises the new library and bundle artifact flows

### Stage 2.2 - Public Effect Reporting
- ✅ `kali effects` emits native JSON effect reports for source roots
- ✅ `kali package-effects` emits native JSON package effect reports for installed packages
- ✅ `check/build --sandbox` reject inferred effects that exceed the active policy
- ✅ Positive CLI/runtime smoke coverage replaces the old unavailable-command gates

### Stage 2.1 - HIR object-literal normalization follow-up
- ✅ Object-literal properties now lower through a dedicated `ObjectProperty` HIR node
- ✅ Property keys lower as literals, so MIR escape analysis no longer mistakes them for bindings
- ✅ Stable heap-store shapes now feed the ownership analyzer for object-literal value escapes
- ✅ Array-element and member-assignment heap-store flows now have explicit MIR ownership coverage
- ✅ Aliased function-expression calls now preserve direct-callee escape precision for local function-valued bindings

## Next Work
- [x] Broader package-shape / host-fit diagnostics matrix coverage
  - Added host-fit coverage for `node:fs` and `require("child_process")` package entrypoints.
- [x] CLI integration coverage for install repair/prune scenarios
  - Added CLI smoke coverage for pruning stale registry layouts back to an empty install state.
- [x] Build direct-input shape enforcement
  - `kali build` now rejects multi-file invocations with the canonical `E5008` usage diagnostic and remains a single-primary-input command.
- [x] Phase-gated later-surface placeholders and smoke coverage
  - Added `effects`, `package-effects`, `package-audit`, `build --capi`, `build --component`, and `run`/`test` API-surface gating so Phase-2+ surfaces now fail with the canonical `E5006` path instead of plain unknown-command parsing.
- [x] Embedding API scaffolding
  - `kali_embed` now exposes `KaliCompiler`, `CompiledArtifact`, `LibArtifact`, and deterministic WIT sidecar generation for the statically known export surface.
