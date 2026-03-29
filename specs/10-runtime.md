# 10 — Runtime

## WASM Execution Engine

### Engine Choice: wasmtime
Use `wasmtime` as the primary WASM execution engine:
- Pure Rust implementation
- Fuel-based metering for CPU limits
- Configurable memory limits
- Mature, well-maintained, WASI support
- Uses Cranelift internally to compile WASM to native machine code at load time

Note: Kali's AOT pipeline compiles TypeScript/JavaScript → WASM. The WASM runtime (wasmtime) then compiles WASM → native code for execution. These are two separate compilation stages.

### Alternative: wasmer
Provide wasmer as an optional backend for cases where:
- Different platform support is needed
- Users want the Singlepass compiler for faster cold start
- Specific embedding requirements

Selectable via `--runtime wasmer` flag.

## Host-Guest Interface

### Host Functions
The WASM module imports host functions for operations that can't be done in pure WASM:

```rust
// Categories of host imports
mod host {
    // I/O
    fn fs_read(path_ptr: i32, path_len: i32) -> i32;
    fn fs_write(path_ptr: i32, path_len: i32, data_ptr: i32, data_len: i32) -> i32;
    fn net_fetch(url_ptr: i32, url_len: i32, opts_ptr: i32) -> i32;
    fn console_log(msg_ptr: i32, msg_len: i32);
    
    // Process
    fn process_exit(code: i32);
    fn process_spawn(cmd_ptr: i32, cmd_len: i32) -> i32;
    fn env_get(key_ptr: i32, key_len: i32, val_ptr: i32) -> i32;
    
    // Timers
    fn timer_set(callback_id: i32, delay_ms: i32, repeat: i32) -> i32;
    fn timer_clear(timer_id: i32);
    
    // Eval (when permitted)
    fn eval_compile(src_ptr: i32, src_len: i32) -> i32;
    
    // System
    fn clock_now() -> f64;
    fn random_bytes(buf_ptr: i32, len: i32);
}
```

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

`eval` and `Function()` constructor are supported via a host callback:

1. WASM calls `eval_compile(source)` host function
2. Host invokes the Kali compiler on the source string (full pipeline: lex → parse → typecheck → codegen)
3. New WASM module is produced and instantiated
4. New module is linked to share the parent module's linear memory and function table
5. Eval'd code can access variables in scope via a shared scope descriptor passed to the host
6. Result is returned to the calling WASM module

Memory sharing details:
- The parent module exports its `Memory` and `Table` objects
- The eval'd module imports them, enabling direct access to the same linear memory
- The eval'd module uses the same heap allocator (shared allocator state in linear memory)
- Scope variables are serialized to a known memory region before eval and deserialized after
- Reference counts are shared — the eval'd code can inc/dec refs on parent objects correctly since both use the same `RcHeader` layout in shared linear memory

This is:
- **Expensive** (full compilation per eval call)
- **Blocked by default** in sandbox policies (effect: `Eval`)
- **Flagged** in static effect analysis (see [specs/09-sandboxing.md](09-sandboxing.md))
- **Correct** — maintains full language semantics
- **Cacheable** — repeated eval of the same source string reuses compiled module

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
When WASM threads are enabled (via `--wasm-threads` flag):
- Each worker/thread runs its own Kali runtime instance with a shared `SharedArrayBuffer`
- `Atomics` operations map to WASM atomic instructions
- Workers communicate via message passing (structured clone) or shared memory
- Each thread has its own stack and allocator; the shared heap region is explicitly managed via `SharedArrayBuffer`
- Thread count is constrained by sandbox policy (`resources.maxThreads`)

This is an advanced feature. Most programs use the single-threaded event loop.

## Module System

### ES Modules
- `import`/`export` → WASM module linking
- Static imports resolved at compile time → direct function calls
- Dynamic `import()` → host function that compiles and loads module at runtime

### Module Instantiation Order
1. Parse and compile all statically imported modules
2. Link modules (resolve imports/exports)
3. Execute module top-level code in dependency order
4. Run entry point

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
