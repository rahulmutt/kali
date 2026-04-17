# TODO

## Completed

### Stage 4.1 - Package-audit availability
- ✅ `kali package-audit` now runs without requiring `--preview`; the preview flag remains accepted as a no-op compatibility shim.

### Stage 4.1 - Eval compatibility gating
- ✅ `--compat eval` now accepts dynamically constructed eval / Function() strings derived from constant program-state fragments.
- ✅ `check` / `run` now reject `eval` and `Function()` usage unless the shared `--compat eval` gate is enabled.

### Stage 1.3 - Parser & AST
- ✅ Parser compiles successfully
- ✅ `cargo test -p kali_parser --lib` passes
- ✅ `cargo test -p kali_parser --test parser_integration` passes
- ✅ `cargo test --workspace` passes
- ✅ Parser handles variable declarations, blocks, functions, classes, control flow, try/catch, switch, debugger, throw, break/continue
- ✅ Parser handles primary expressions, function expressions, call chains, member access, binary expressions, and `new`
- ✅ Parser now accepts import declarations and literal dynamic `import()` expressions, which keeps package-corpus analysis and later code-splitting work on the real AST path
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

### Stage 1.8 - Deno API compatibility scaffold
- ✅ `kali_api_deno` now exposes the Deno-oriented host-support layer on top of the shared Web baseline
- ✅ Read-only env/args views, deterministic filesystem helpers, and the query-only permissions facade are available for the Phase-1 standalone context

### Stage 2.1 - HIR object-literal normalization follow-up
- ✅ Object-literal properties now lower through a dedicated `ObjectProperty` HIR node
- ✅ Property keys lower as literals, so MIR escape analysis no longer mistakes them for bindings
- ✅ Stable heap-store shapes now feed the ownership analyzer for object-literal value escapes
- ✅ Array-element and member-assignment heap-store flows now have explicit MIR ownership coverage
- ✅ Aliased function-expression calls now preserve direct-callee escape precision for local function-valued bindings
- ✅ Alias chains of function expressions now resolve to the canonical lowered target, including anonymous function expressions

### Stage 4.1 - Runtime dynamic import graph lookup
- ✅ Browser bundle JS now normalizes runtime `loadDynamicImport(specifier)` requests before target lookup, so path-equivalent runtime specifiers resolve through the bundle-local map instead of requiring an exact static spelling.
- ✅ Browser bundle smoke coverage now exercises a normalized runtime specifier (`./sub/../lazy.ts`) against a discovered chunk target.

## Next Work
- [x] Browser bundle source-map companions
  - `kali build --bundle` now emits a deterministic `.js.map` companion and appends the matching `sourceMappingURL` footer.
- [x] Browser bundle chunk artifacts for literal dynamic imports
  - `kali build --bundle` now emits deterministic chunk directories for literal `import("...")` boundaries, including `.wasm`, `.js`, `.map`, and metadata companions for each discovered chunk.
- [x] Broader package-shape / host-fit diagnostics matrix coverage
  - Added host-fit coverage for `node:fs` and `require("child_process")` package entrypoints.
- [x] CLI integration coverage for install repair/prune scenarios
  - Added CLI smoke coverage for pruning stale registry layouts back to an empty install state.
- [x] Build direct-input shape enforcement
  - `kali build` now rejects multi-file invocations with the canonical `E5008` usage diagnostic and remains a single-primary-input command.
- [x] Phase-gated later-surface placeholders and smoke coverage
  - Added `effects`, `package-effects`, `package-audit`, `build --capi`, `build --component`, and `run`/`test` API-surface gating so Phase-2+ surfaces now fail with the canonical `E5006` path instead of plain unknown-command parsing.
- [x] Shared `--compat` CLI plumbing for the Phase-4 compatibility vocabulary
  - Source-graph commands now parse `--compat` / `compat.features` requests, surface them in the command context, and reject the unavailable `eval` path through the canonical `E5006` gate instead of silently dropping the request.
- [x] Function() compatibility path for simple statically-resolved bodies
  - `new Function("return 1 + 2;")()` now rewrites through the shared `--compat eval` path and executes in the runtime smoke suite.
- [x] Embedding API scaffolding
  - `kali_embed` now exposes `KaliCompiler`, `CompiledArtifact`, `LibArtifact`, and deterministic WIT sidecar generation for the statically known export surface.
- [x] Stage 3.1 optimization scaffolding
  - `kali_optimize` now performs release constant folding, constant-branch elimination, and release-advanced algebraic identities, the CLI build path wires those passes into WASM generation, and `--max-specializations` now overrides the deterministic specialization budget used by the optimizer/cache path.
- [x] Stage 3.2 Node API layer scaffold
  - Added `kali_api_node` helpers for process/path/crypto/events/buffer/util plus fs/url/os scaffolding and unit tests; the Node-targeted command path is now wired through check/build/run/test and node-only import resolution in the analysis context.
  - Expanded the helper layer with Node-style assertion helpers and a synchronous `util.promisify` bridge so the documented Node helper surface is closer to the planned phase-3 subset.
  - The helper layer now exposes `NodePath`, `NodeCrypto`, `NodeUtil`, `NodeAssert`, and `NodeRuntimeProjection` facades so future linker registration has a single Node-host surface to project through.
  - The runtime linker now consumes the Node projection facade for `fs/promises`, stream, HTTP, and process argv/env host imports when the effective API surface is `node`.
  - Install-time package host-fit validation now keys off the project `compilerOptions.apiSurface`, so Node-targeted installs can accept Node-only builtins while the default standalone context still rejects them with `E6005`.

