# 11 — Standard APIs

## Strategy

Implement APIs as host functions provided by the runtime. Each API surface is a separate crate that registers its host functions with the WASM runtime.

## API Layers

### Web Platform APIs (Baseline)
Always available regardless of runtime mode:
- `console` (log, warn, error, debug, info, table, time/timeEnd)
- `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`
- `queueMicrotask`
- `fetch` (WHATWG Fetch API)
- `URL`, `URLSearchParams`
- `TextEncoder`, `TextDecoder`
- `crypto` (Web Crypto API subset: getRandomValues, subtle)
- `AbortController`, `AbortSignal`
- `Blob`, `File`, `FormData`
- `Headers`, `Request`, `Response`
- `ReadableStream`, `WritableStream`, `TransformStream`
- `structuredClone`
- `atob`, `btoa`
- `performance.now()`
- `WebSocket`
- `EventTarget`, `Event`, `CustomEvent`

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
- `Deno.permissions` (maps to Kali sandbox policies)
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
- DOM APIs are **not** natively supported (Kali does not embed a browser engine)
- Provide a minimal DOM shim for testing (`kali_api_web`): `window`, `document`, `navigator` stubs
- Only Web Platform APIs are available (no Deno or Node.js APIs)
- Primarily for running browser-targeted library code in server/CLI context
- For actual browser deployment, use `kali build --bundle` to emit WASM + JS glue that runs in real browsers

**Note**: Web Platform APIs (fetch, crypto, streams, etc.) are always available regardless of `--api` mode. The `--api` flag controls which *additional* platform-specific APIs are loaded.

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
Implemented in `kali_runtime` (compiled to WASM):
- `Object` prototype methods
- `Array` and all typed arrays (`Uint8Array`, etc.)
- `Map`, `Set`, `WeakMap`, `WeakSet`
- `Promise`
- `RegExp` (using a pure-Rust regex engine)
- `Date`
- `JSON`
- `Math`
- `String` prototype methods
- `Number`, `Boolean`, `Symbol`
- `Error` and subtypes
- `Proxy`, `Reflect`
- `Intl` (basic — full ICU is large, provide subset)
- `ArrayBuffer`, `SharedArrayBuffer`, `DataView`
- Iterators, Generators, AsyncGenerators

## Global Scope

The global object provides:
- All Web Platform APIs
- Runtime-specific APIs (Deno/Node based on mode)
- `globalThis` reference
- TypeScript-aware — all globals are typed in Kali's standard lib `.d.ts` files
