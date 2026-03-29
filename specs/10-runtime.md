# 10 — Runtime

## WASM Execution Engine

### Engine Choice: wasmtime
**Phase 1-3 mandate:** use `wasmtime` as the execution engine.
- Pure Rust implementation
- Fuel-based metering for CPU limits
- Configurable memory limits
- Mature, well-maintained, WASI support
- Supports serialized/precompiled artifacts for production embedding

**Important consistency rule**: Kali itself is AOT-only and performs no language-level JIT compilation. A host runtime may still validate, translate, or precompile the emitted WASM as an execution detail, but Kali must not depend on speculative/adaptive JIT behavior for correctness or performance.

Preferred execution modes:
- **Development**: instantiate emitted WASM directly in wasmtime for fast iteration
- **Production/embedding**: use wasmtime's precompiled/serialized module support where available to avoid per-launch recompilation costs

### Optional Alternative Backend (Later Phase)
An engine abstraction may be added later to support backends such as `wasmer` when there is a demonstrated embedding or platform need. This must not complicate the initial runtime design; all core specs assume wasmtime semantics first.

## Host-Guest Interface

### Host Functions
The WASM module imports host functions for operations that can't be done in pure WASM.

Important loading rule: the runtime registers only the host imports required by the selected **API surface** and **runtime profile**. The list below is the union of early-phase import categories, not a promise that every program always gets every import.

```rust
// Union of early-phase host-import categories; actual registration is profile-dependent.
mod host {
    // Web baseline / I/O
    fn fs_read(path_ptr: i32, path_len: i32) -> i32;
    fn fs_write(path_ptr: i32, path_len: i32, data_ptr: i32, data_len: i32) -> i32;
    fn net_fetch(url_ptr: i32, url_len: i32, opts_ptr: i32) -> i32;
    fn console_log(msg_ptr: i32, msg_len: i32);

    // Deno/Node-style environment or process metadata
    fn env_get(key_ptr: i32, key_len: i32, val_ptr: i32) -> i32;
    fn process_args(buf_ptr: i32) -> i32;
    fn process_pid() -> i32;

    // Timers
    fn timer_set(callback_id: i32, delay_ms: i32, repeat: i32) -> i32;
    fn timer_clear(timer_id: i32);

    // System
    fn clock_now() -> f64;
    fn random_bytes(buf_ptr: i32, len: i32);
}
```

Interpretation rules:
- `console`, timers, `fetch`, time, and randomness belong to the Phase 1 Web baseline and may exist across supported API surfaces.
- `env_get`, `process_args`, and `process_pid` are registered only for profiles that expose the corresponding Deno/Node process APIs; browser-targeted builds must not assume they exist.
- Every registered host import is policy-aware; enabling an API surface does not bypass sandbox checks.
- Unsupported imports for the current command/profile are not stubbed silently; code that requires them should fail with the canonical feature-maturity diagnostic.

Later compatibility/embedding imports extend this set when the corresponding API surface is enabled:
- `process_spawn(...)` for subprocess support
- `process_exit(code)` for explicit termination control / embedding once the process-control contract is specified; this does **not** imply that `Deno.exit` is part of the Phase 1 API surface
- `eval_compile(...)` only for the Phase 4 `--compat eval` path

### Data Passing
- Strings/buffers: passed as (pointer, length) pairs referencing WASM linear memory
- Complex objects: serialized to a shared buffer in linear memory
- Return values: via return value or writing to a caller-provided buffer
- Error handling: return error codes + error detail in a dedicated error buffer

## Event Loop

For async operations, Kali implements a single-threaded event loop:

```
┌─────────────────────────────┐
│         Event Loop          │
│                             │
│  1. Run microtask queue     │
│  2. Run one macrotask       │
│  3. Check timer queue       │
│  4. Poll I/O completions    │
│  5. Repeat until idle       │
└─────────────────────────────┘
```

- **Microtasks**: Promise callbacks, queueMicrotask
- **Macrotasks**: setTimeout/setInterval callbacks, I/O callbacks
- Async I/O backed by `tokio` on the host side
- WASM execution is synchronous — async operations yield to the host

## `eval` Support

`eval` and `Function()` are **Phase 4 compatibility features**.

Implementation strategy:
1. **Phases 1-3**: parse them, report the `Eval` effect, and reject them by default.
2. **Phase 4**: support runtime compilation through a host callback (`eval_compile`) with conservative deoptimization of the surrounding scope, enabled via `--compat eval`.

Requirements for the Phase 4 path:
- Treat all directly reachable locals as boxed/shared values
- Disable layout-sensitive optimizations in the affected region
- Preserve JavaScript-visible semantics before recovering performance
- Cache repeated eval compilations where safe

This is intentionally conservative:
- **Expensive** — full compilation may occur at runtime
- **Blocked by default** in sandbox policies (effect: `Eval`)
- **Flagged** in static effect analysis (see [specs/09-sandboxing.md](09-sandboxing.md))
- **Optimization barrier** for surrounding code

The exact mechanism for scope capture and memory sharing is an implementation detail and is deliberately left unspecified here to avoid overspecifying a fragile design too early.

## Async/Await Runtime

Async functions are compiled to state machines (in HIR lowering):

```
async function fetch(url) → StateMachine {
    state 0: call host_fetch(url), suspend → state 1
    state 1: receive result, resume, return
}
```

- Each await point is a state transition
- The event loop drives state machine progression
- Promise resolution triggers the next state
- Implemented without OS threads — single-threaded cooperative scheduling

## Threading Model

Kali's primary execution model is single-threaded (one event loop per runtime instance).

### SharedArrayBuffer & Atomics
`SharedArrayBuffer`, `Atomics`, and the `--wasm-threads` runtime profile are later compatibility features, not part of the Phase 1 single-threaded baseline.

Once the threaded profile exists and `--wasm-threads` is enabled:
- Each worker/thread runs its own Kali runtime instance with a shared `SharedArrayBuffer`
- `Atomics` operations map to WASM atomic instructions
- Workers communicate via message passing (structured clone) or shared memory
- Each thread has its own stack and allocator; the shared heap region is explicitly managed via `SharedArrayBuffer`
- Thread count is constrained by sandbox policy (`resources.maxThreads`)

Until then, the CLI/runtime must reject `--wasm-threads` with the canonical feature-maturity diagnostic rather than silently degrading to single-threaded execution. Even after the threaded profile lands, unsupported targets/engines must still reject the flag explicitly.

## Module System

### ES Modules
Initial implementation strategy:
- Parse the full static module graph up front
- Resolve `import`/`export` in the compiler
- Lower the whole program/package graph into a **single linked WASM module** per build artifact
- Static imports become direct internal calls or data references after linking
- Literal-string `import()` may later be lowered to an async lookup over the already-linked graph
- Non-literal dynamic `import()` remains a host-mediated compatibility path and is treated as a dynamic effect boundary

This deliberately avoids depending on the WebAssembly module-linking proposal in early phases.

### Module Instantiation Order
1. Parse and compile all statically imported modules in the graph
2. Resolve imports/exports in the compiler linker
3. Emit one linked WASM artifact for the graph
4. Execute module top-level code in ECMAScript dependency order
5. Run entry point

## Error Handling

### JavaScript Exceptions
- `throw` → set error state + unwind
- `try/catch/finally` → WASM exception handling proposal (when available) or manual unwind via return codes
- Unhandled exceptions → host catches and formats error

### Stack Traces
- Maintain a shadow call stack in linear memory
- Each function entry pushes (function name, source location)
- On error, serialize the shadow stack for the error message
- Minimal overhead (one i32 store per function call)
