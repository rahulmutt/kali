# 10 — Runtime

## WASM Execution Engine

### Engine Choice: wasmtime
**Early-phase default:** standardize the runtime on `wasmtime` first.
- Pure Rust implementation
- Fuel-based metering for CPU limits
- Configurable memory limits
- Mature, well-maintained, WASI support
- Supports serialized/precompiled artifacts for production embedding

This is a deliberate simplification for Phase 1-3, not a forever-exclusive backend promise. The rest of the spec assumes wasmtime semantics first so the runtime, sandboxing, and embedding contracts stay coherent while the product is still maturing.

**Important consistency rule**: Kali itself is AOT-only and performs no language-level JIT compilation. A host runtime may still validate, translate, or precompile the emitted WASM as an execution detail, but Kali must not depend on speculative/adaptive JIT behavior for correctness or performance.

Preferred execution modes:
- **Development**: instantiate emitted WASM directly in wasmtime for fast iteration
- **Production/embedding**: use wasmtime's precompiled/serialized module support where available to avoid per-launch recompilation costs

### Optional Alternative Backend (Later Phase)
An engine abstraction may be added later to support backends such as `wasmer` when there is a demonstrated embedding or platform need. This must not complicate the initial runtime design, and any added backend must preserve the same externally visible sandbox/resource/diagnostic contracts rather than introducing backend-specific semantics into user-facing behavior.

Engine-choice simplification rule:
- subsystem implementations may compare `wasmtime` and `wasmer` internally, but the public early-phase spec contract is standardized on `wasmtime`
- no chapter should phrase engine choice as though Phase 1 leaves both backends equally normative
- adding another backend later is an implementation-extension decision, not a license to fork the CLI, sandbox, or embedding contracts by engine

## Host-Guest Interface

### Host Adapter Modes
Kali keeps one guest-facing host ABI, but early phases allow more than one **host adapter** to implement it:
- **Kali-hosted execution** (`kali run`, `kali test`, embedding) uses native Rust/wasmtime host functions.
- **Browser bundle output** (`kali build --bundle --api browser`) uses generated JS glue to adapt the same guest-facing capability model onto the real browser host.

Cross-spec consistency rule:
- the guest module should target one coherent Kali host ABI/capability model rather than a totally different imported-API shape per deployment mode
- browser glue may implement that ABI differently from the native Rust runtime, but it must not silently widen the documented browser-targeted contract
- unsupported capabilities for the selected artifact/profile are still rejected during analysis/build rather than left to fail later through missing imports

### Host Functions
The WASM module imports host functions for operations that can't be done in pure WASM.

Important loading rule: the runtime registers only the host imports required by the selected **API surface** and **runtime profile**. The list below is the union of early-phase import categories, not a promise that every program always gets every import.

Clarification:
- not every Phase 1 Web-baseline API needs its own host import. Some baseline functionality is expected to live in the guest/runtime support library itself (for example `queueMicrotask`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`, `AbortController`, `AbortSignal`, `structuredClone`, `EventTarget`, `Event`, and `CustomEvent`) and therefore does not have to appear as a dedicated host import in this table.

```rust
// Union of early-phase host-import categories; actual registration is profile-dependent.
mod host {
    // Shared Web-platform baseline
    fn net_fetch(url_ptr: i32, url_len: i32, opts_ptr: i32) -> i32;
    fn console_write(level: i32, msg_ptr: i32, msg_len: i32);

    // Deno-oriented standalone filesystem/process surface
    // (the same host-layer abstractions may be reused by later Node compatibility work,
    // but their presence here does not imply early `--api node` support)
    fn fs_read(path_ptr: i32, path_len: i32) -> i32;
    fn fs_write(path_ptr: i32, path_len: i32, data_ptr: i32, data_len: i32) -> i32;
    fn fs_stat(path_ptr: i32, path_len: i32, out_ptr: i32) -> i32;
    fn fs_read_dir(path_ptr: i32, path_len: i32, out_ptr: i32) -> i32;
    fn env_get(key_ptr: i32, key_len: i32, val_ptr: i32) -> i32;
    fn env_list(out_ptr: i32) -> i32; // policy-filtered snapshot, not raw host environment
    fn process_args(buf_ptr: i32) -> i32;

    // Shared timers
    fn timer_set(callback_id: i32, delay_ms: i32, repeat: i32) -> i32;
    fn timer_clear(timer_id: i32);

    // Shared system primitives
    fn clock_now() -> f64;
    fn random_bytes(buf_ptr: i32, len: i32);
}
```

Interpretation rules:
- `console`, timers, `fetch`, time, and randomness belong to the Phase 1 Web baseline and may exist across supported API surfaces.
- the host-import table is therefore a capability/host-boundary summary, not an exhaustive inventory of every JS-visible global provided by the baseline library layer.
- `console_write` is the canonical host-import shape for the Phase 1 console family; guest-visible `console.log` / `warn` / `error` / `debug` / `info` all lower through this one `Console.Write` capability family with a level discriminator rather than through separate ad hoc imports per method.
- in Kali-hosted standalone/embedded execution, these are normally satisfied by native Rust host functions; in browser-targeted bundle output, the generated JS glue is responsible for wiring the equivalent behavior onto the real browser host
- `fs_read`, `fs_write`, `fs_stat`, `fs_read_dir`, `env_get`, `env_list`, and `process_args` belong to the Deno-oriented standalone host surface in Phase 1, not to the shared Web baseline; later Node compatibility may reuse similar host abstractions, but browser-targeted builds must not assume these imports exist.
- `process_args` exposes only the invocation's caller-supplied argument vector; in schema v1 this is treated as execution-context input rather than a separately policy-gated host capability.
- `env_get` / `env_list` expose only the sandbox-permitted environment view; they must not leak the raw host environment and then rely on guest-side filtering.
- The read-only `Deno.permissions` facade is derived from already-resolved runtime/policy state and normally does not need a dedicated host import; Kali should not model it as an interactive permission-prompt channel.
- In Phase 1 this is a query-only compatibility surface: the runtime may expose the minimal status-query behavior, but `request()` / `revoke()`-style escalation methods are absent or rejected rather than being implemented as no-op prompts.
- The Phase 1 runtime does not provide interactive permission-prompt imports; permission state is an already-resolved sandbox contract, not a request-at-runtime workflow.
- Every registered host import is policy-aware; enabling an API surface does not bypass sandbox checks.
- This native host-import enforcement model applies directly only when code executes inside a Kali-controlled runtime or embedding host. Browser-targeted emitted artifacts instead rely on the generated JS glue/browser host adapter, which must preserve the documented browser-targeted capability contract without being described as Kali-controlled post-deployment sandbox enforcement unless a later browser-specific host contract says otherwise.
- That browser glue is responsible for runtime bootstrap plus Kali-mediated capability calls that still go through the guest ABI; it is not a promise that every ambient browser API (for example arbitrary DOM methods) is lowered through one Kali host import per browser primitive.
- Unsupported imports for the current command/profile are not stubbed silently.
- If lowering/runtime setup requires a capability that is phase-gated or profile-gated, fail with the canonical feature-maturity diagnostic.
- If source code merely references a global that is absent from the selected ambient surface in an otherwise-supported mode, that should normally already have been reported as an ordinary name/type error before runtime setup.

Later compatibility/embedding imports extend this set when the corresponding API surface is enabled:
- `process_spawn(...)` for the Phase 3 subprocess-support path
- `env_set(...)` for the Phase 3 mutable-environment path once `effects.process.envWrite` is part of the enabled host surface
- socket/listener networking imports for the Phase 3 `Network.Connect` / `Network.Listen` / `Deno.serve` path
- `process_pid()` only on the later-compatibility process-identity path once a schema/policy revision defines its sandbox contract
- `process_exit(code)` only on the later-compatibility process-control path once a schema/policy revision defines its sandbox contract; this does **not** imply that `Deno.exit` is part of the Phase 1 API surface
- `cwd_get(...)` / `cwd_set(...)` only on the later-compatibility working-directory path once a documented policy/effect contract exists
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
- Async I/O is backed by the host's Rust async/runtime primitives; the specific executor is an implementation detail unless a later embedding contract makes it observable
- WASM execution is synchronous — async operations yield to the host

## `eval` Support

`eval` and `Function()` are **Phase 4 compatibility features**.

Implementation strategy:
1. **Phases 1-3**: parse them, report the `Eval` effect, and reject them by default.
2. **Phase 4**: support runtime compilation through a host callback (`eval_compile`) with conservative deoptimization of the surrounding scope, enabled via `--compat eval`.

Compatibility-switch rule:
- schema v1 keeps one stable compatibility-feature name, `eval`, for both direct `eval` and the `Function()` constructor path
- runtime docs and diagnostics should therefore describe `Function()` as covered by `--compat eval` rather than implying a second independent switch

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
`SharedArrayBuffer`, `Atomics`, and the threaded runtime profile `wasm-threads` are later compatibility features, not part of the Phase 1 single-threaded baseline.

Terminology rule:
- CLI uses `--wasm-threads`
- config/embedding use the runtime-profile name `wasm-threads`
- both refer to the same runtime-profile switch rather than two separate features

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
- Lower the whole program/package graph into one linked WASM module as the core payload of each build artifact *(for browser builds, this may be accompanied by JS glue, but not by runtime WASM module linking)*
- Static imports become direct internal calls or data references after linking
- Literal-string `import()` may later be lowered to an async lookup over the already-linked graph
- Non-literal dynamic `import()` remains a host-mediated compatibility path and is treated as a dynamic effect boundary

This deliberately avoids depending on the WebAssembly module-linking proposal in early phases.

### Module Instantiation Order
1. Parse and compile all statically imported modules in the graph
2. Resolve imports/exports in the compiler linker
3. Emit one linked WASM payload for the graph
4. Execute module top-level code in ECMAScript dependency order
5. Run entry point

## Error Handling

### JavaScript Exceptions
- `throw` → set error state + unwind
- **Phase 1 baseline**: `try/catch/finally` is implemented with explicit runtime-managed unwind/state machinery rather than depending on the WASM exception-handling proposal
- **Later optimization path**: if WASM exception handling is enabled for a supported target/runtime profile, Kali may lower compatible regions to native WASM exceptions without changing language-visible behavior
- Unhandled exceptions → host catches and formats error

### Stack Traces
- Maintain a shadow call stack in linear memory
- Each function entry pushes (function name, source location)
- On error, serialize the shadow stack for the error message
- Minimal overhead (one i32 store per function call)
