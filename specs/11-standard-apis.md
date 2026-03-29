# 11 — Standard APIs

## Strategy

Implement APIs as host functions provided by the runtime. Each API surface is a separate crate that registers its host functions with the WASM runtime.

Compatibility is delivered in layers:
1. **Baseline**: Web platform primitives needed by modern JS libraries.
2. **Primary host**: Deno-style APIs, since they align well with explicit permissions and sandboxing.
3. **Compatibility layers**: Node.js shims and browser-facing glue, added incrementally and tested against real packages.

The spec goal is broad compatibility, but the implementation should prefer a smaller, dependable surface over a shallow imitation of every host API.

A key simplification rule applies throughout this section: Phase 1 should target the smallest API set that unlocks real-world package execution, and every later API addition should be justified by package compatibility or standards pressure.

For dynamic or semantically expensive APIs (for example `Proxy`, weak references, and threaded primitives), the canonical phase/status lives in [specs/19-feature-maturity.md](19-feature-maturity.md). This section should describe API layering, not restate a conflicting maturity decision.

## API Layers

### Web Platform APIs (Baseline)
Always available regardless of runtime mode.

**Phase 1 MVP baseline**
- `console` (`log`, `warn`, `error`, `debug`, `info`)
- `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`
- `queueMicrotask`
- `fetch`, `Headers`, `Request`, `Response`
- `URL`, `URLSearchParams`
- `TextEncoder`, `TextDecoder`
- `AbortController`, `AbortSignal`
- `structuredClone`
- `performance.now()`
- `EventTarget`, `Event`, `CustomEvent`

**Later compatibility expansion**
- `Blob`, `File`, `FormData`
- `ReadableStream`, `WritableStream`, `TransformStream`
- `crypto` (broader Web Crypto surface beyond the MVP subset)
- `atob`, `btoa`
- `WebSocket`

### Deno API (`--api deno`, default)
Primary API surface, following Deno's design:
- `Deno.readTextFile`, `Deno.writeTextFile`, `Deno.readFile`, `Deno.writeFile`
- `Deno.open`, `Deno.create`, `Deno.mkdir`, `Deno.remove`, `Deno.rename`
- `Deno.stat`, `Deno.lstat`, `Deno.readDir`
- `Deno.env.get`, `Deno.env.set`, `Deno.env.toObject`
- `Deno.args`, `Deno.exit`, `Deno.pid`
- `Deno.Command` (process spawning)
- `Deno.serve` (HTTP server)
- `Deno.cwd`, `Deno.chdir`
- `Deno.permissions` as a read-only compatibility facade over Kali sandbox policy state; it reports granted/denied capabilities but does not perform interactive permission prompts
- All sync and async variants

### Node.js API (`--api node`)
Compatibility layer for Node.js ecosystem:
- `fs`, `fs/promises` — file system operations
- `path` — path manipulation (pure, no host calls needed)
- `os` — operating system info
- `child_process` — process spawning
- `http`, `https` — HTTP server/client
- `crypto` — cryptographic operations
- `buffer` — Buffer class
- `stream` — Node streams
- `events` — EventEmitter
- `util` — utilities (promisify, inspect, etc.)
- `url` — URL parsing
- `querystring` — query string parsing
- `assert` — assertions
- `process` — process global (env, argv, exit, cwd, etc.)

**Strategy**: Implement Node APIs as wrappers around Deno-style host functions. Use `deno_std/node` as a reference for compatibility.

### Browser API (`--api browser`)
For code targeting browser environments:
- DOM APIs are **not** natively supported by the standalone Kali runtime (it does not embed a browser engine)
- Only Web Platform APIs are available (no Deno or Node.js APIs)
- Primarily for compiling browser-targeted libraries, shared modules, and non-DOM code paths
- For actual browser deployment, use `kali build --bundle` to emit WASM + JS glue that runs in a real browser host
- Any lightweight DOM test shim is a separate testing utility, not part of the core browser compatibility contract

**Note**: The Phase 1 baseline Web Platform APIs are always available regardless of `--api` mode. The `--api` flag controls which *additional* platform-specific APIs are loaded. Later Web API expansions remain subject to phase gating and should not be assumed to exist merely because a different `--api` mode is selected.

## Implementation Architecture

```
User Code (WASM)
    │
    ├── Direct calls → Host Functions (Rust)
    │                      │
    │                      ├── Sandbox policy check
    │                      ├── Actual I/O (via tokio)
    │                      └── Return result to WASM
    │
    └── Pure APIs (in WASM runtime)
        ├── Math operations
        ├── String operations
        ├── Array methods
        ├── JSON parse/stringify
        └── RegExp engine
```

### Pure vs Host APIs
- **Pure**: Implemented in Rust, compiled to WASM, runs inside the sandbox (Math, String, Array, JSON, RegExp)
- **Host**: Implemented as wasmtime host functions, run outside WASM (I/O, network, process, crypto)

### Built-in Objects (WASM Runtime)
Implemented in `kali_runtime` (compiled to WASM), in phases:

**Phase 1 core runtime**
- `Object` prototype methods
- `Array` and typed arrays needed by common libraries
- `Promise`
- `RegExp` (using a pure-Rust implementation of ECMAScript RegExp semantics; a generic Rust regex crate is only acceptable as an internal building block, not as the semantic contract)
- `Date`
- `JSON`
- `Math`
- `String` prototype methods
- `Number`, `Boolean`, `Symbol`
- `Error` and common subtypes
- `ArrayBuffer`, `DataView`
- Iterators, Generators, AsyncGenerators
- `Map`, `Set`
- `Reflect` subset required by transpiled/bundled code

**Later compatibility phases**
- `SharedArrayBuffer` (when WASM threads are enabled)
- `WeakMap`, `WeakSet` (only once weak-reference semantics are specified well enough to preserve behavior)
- `FinalizationRegistry` (only once weak/finalization semantics can be preserved without undermining the no-tracing-GC design)
- `Proxy`
- fuller `Intl` support

The runtime should prioritize the subset needed by real-world packages and conformance tests before implementing hard edge-case APIs with large semantic cost.

## Global Scope

The global object provides:
- All Web Platform APIs
- Runtime-specific APIs (Deno/Node based on mode)
- `globalThis` reference
- TypeScript-aware — all globals are typed in Kali's standard lib `.d.ts` files
