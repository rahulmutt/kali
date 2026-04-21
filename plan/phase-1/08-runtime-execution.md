# Stage 1.8 — Runtime & Execution

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** [1.7 — WASM Code Generation](07-wasm-codegen.md)

**Status:** ✅ Complete — runtime execution is wired through wasmtime, the default Deno surface is exercised explicitly and implicitly, and the stage's evidence coverage is closed

## Goal

Implement the Kali-hosted execution environment: integrate `wasmtime` as the execution engine,
wire the **Default standalone context (schema v1)** host APIs (Deno-oriented standalone surface
plus the **shared Web baseline**), and make `kali run` and `kali test` functional for real
TypeScript/JavaScript programs.

## Workable Milestone

- `kali run <file>` compiles a TS/JS source file to WASM and executes it inside the
  Default standalone context.
- `kali test [files...]` discovers and runs test files, reporting pass/fail counts.
- Deno-oriented standard APIs (`Deno.*`, `fetch`, `console`, `setTimeout`) are available to guest programs, alongside the Stage 1.8 Web primitives exercised here (`performance.now()`, `crypto.getRandomValues()`).
- Exit codes are correct (0 = success, non-zero = runtime error or test failure).

## Progress

- `kali_runtime` is now wired through wasmtime for simple emitted WASM modules.
- `kali run` and `kali test` are no longer stubs; both drive the compiler output end to end.
- Declaration-only entrypoints are rejected for runtime-bearing commands with `E5007`.
- Smoke tests cover a repo-backed successful run fixture, declaration-only rejection, explicit-file test reporting, checked-in test discovery, and guest-registered `Kali.test(...)` callbacks.
- The basic console host imports are now wired into the runtime linker, and a first Deno-oriented host-surface subset is now available (filesystem read/write, environment lookup, arguments, fetch, and timer/microtask scheduling); the guest-side `Kali.test(...)` registration protocol now registers callbacks with the host test runner.
- Web-baseline host primitives for `performance.now()` and `crypto.getRandomValues()` are now wired into the runtime linker so the Stage 1.8 baseline has concrete time/random coverage.
- The guest-side Web baseline support-library follow-up is now implemented in `kali_api_web` with URL parsing/resolution, UTF-8 text encoding/decoding, `structuredClone`, `AbortController`/`AbortSignal`, and event primitives.
- The Deno-oriented support-library scaffold is now checked in `kali_api_deno`, providing read-only env/args/filesystem helpers plus the query-only permissions projection on top of the shared Web baseline.
- Runtime fixture coverage now includes timer/interval clearing, mocked fetch failure, and entrypoint trap diagnostics for the remaining Stage 1.8 edge cases.
- The default API surface is now locked in by explicit `--api deno` smoke coverage for both `run` and `test`, confirming the spelled-out default behaves the same as the implicit path.

## Tasks

### 1. wasmtime integration (`kali_runtime`)

Add `wasmtime` as a dependency (pure-Rust WASM engine, consistent with the
**Pure-Rust implementation contract**).

Implement the `KaliRuntime` struct:

- Creates a `wasmtime::Engine` with Kali's standard compile/execution configuration.
- Creates a `wasmtime::Store<KaliHostState>` where `KaliHostState` holds the host-side state
  (file handles, pending timers, permission policy, etc.).
- Instantiates the compiled WASM module with the host import table defined in Stage 1.7.
- Calls the WASM `_start` export (or the module's designated entrypoint function).
- Propagates WASM traps to Kali diagnostics as `E4xxx` runtime errors.

**No language-level JIT:** `wasmtime` may internally validate, compile, and cache the emitted WASM
(that is normal engine behaviour), but this is not a second Kali compilation tier. Kali's
compilation is complete before execution begins; there are no speculative or adaptive re-compilation
passes at the language level.

For deployments that care about eliminating launch-time WASM translation, document the option to
use engine precompilation (e.g. `wasmtime compile`) to produce an AOT-cached `.cwasm` file.

### 2. Default standalone context (schema v1)

Implement the host import functions that constitute the **Default standalone context**. These are
Rust functions registered with the `wasmtime::Linker`:

#### Console / stdio

| Host import | Behaviour |
|---|---|
| `kali:rt/console_log(val: i64)` | format `TaggedVal`, print to stdout with newline |
| `kali:rt/console_error(val: i64)` | format `TaggedVal`, print to stderr with newline |
| `kali:rt/console_warn(val: i64)` | format `TaggedVal`, print to stderr with `[warn]` prefix |

#### File system (Deno-oriented surface)

Implement the minimal async-FS subset via synchronous WASI-style host calls in Phase 1:

- `Deno.readTextFile(path)` → `Promise<string>`
- `Deno.writeTextFile(path, data)` → `Promise<void>`
- `Deno.readFile(path)` → `Promise<Uint8Array>`
- `Deno.stat(path)` → `Promise<Deno.FileInfo>`
- `Deno.mkdir(path, options?)` → `Promise<void>`
- `Deno.remove(path, options?)` → `Promise<void>`

Each is gated by the sandbox policy (Stage 1.9); in Phase 1 without a policy file, use a
default-open policy for standalone execution (not browser mode).

#### Network

- `fetch(url, options?)` → `Promise<Response>` — implement via `reqwest` (pure Rust HTTP client).
- Also gated by sandbox policy.

#### Environment

- `Deno.env.get(key)` → `string | undefined`
- `Deno.env.set(key, value)` (policy-gated)
- `Deno.args` → `string[]` (passed via CLI arguments after `--`)
- `Deno.exit(code)` → never

#### Timers

- `setTimeout(fn, ms)` / `clearTimeout(id)`
- `setInterval(fn, ms)` / `clearInterval(id)`
- `queueMicrotask(fn)`

Implement a minimal async event loop in Rust using `tokio` (or a simpler custom loop) that drives
pending timers and microtask queues between WASM call frames.

#### Web baseline

- `crypto.getRandomValues(buffer)` — via OS entropy.
- `URL`, `URLSearchParams` — implement in WASM guest code (lowered from a bundled TS polyfill)
  or as host imports.
- `TextEncoder` / `TextDecoder` — host imports.
- `AbortController` / `AbortSignal` — host imports.

### 3. `kali run` subcommand

```
kali run <file> [-- args...]
kali run --sandbox <policy> <file> [-- args...]
kali run --api deno <file> [-- args...]   # explicit; deno is the default
```

Pipeline:

1. Lex → Parse → Check → HIR → LIR → WASM codegen (same pipeline as `kali build`).
2. Instantiate WASM with the Default standalone context.
3. Run event loop until completion or unhandled rejection.
4. Exit with code from `Deno.exit()` or 0 on clean completion, 1 on unhandled exception.

Error handling: runtime panics and unhandled Promise rejections are caught, formatted as `E4xxx`
diagnostics, printed to stderr, and result in exit code 1.

`E4xxx` error codes:

| Code | Meaning |
|---|---|
| `E4001` | Unhandled runtime exception |
| `E4002` | Stack overflow |
| `E4003` | Out of memory |
| `E4004` | Sandbox policy violation |
| `E4005` | Import not available in current context |
| `E4006` | Integer divide by zero |
| `E4007` | Unreachable instruction executed |

### 4. `kali test` subcommand

```
kali test [files...]
kali test --filter <pattern> [files...]
kali test --sandbox <policy> [files...]
```

Test discovery (when no explicit files are given): walk the project tree for files matching
`**/*.test.ts`, `**/*.spec.ts`, `**/*.test.js`, `**/*.spec.js`.

Test runner protocol:

- Guest programs call `Kali.test(name: string, fn: () => void | Promise<void>)` to register tests.
- The host collects registrations, then runs each test function, catching errors.
- Output: `ok <N>`, `FAILED <N>`, with failure messages and stack traces.
- Exit code: 0 if all tests pass, 1 if any fail.

Provide a minimal `@kali/test` type declaration stub (`.d.ts`) so tests type-check correctly.

### 5. Declaration-only file rejection

As specified: declaration-only files (`.d.ts`, `.d.mts`, `.d.cts`) must be rejected as entrypoints
for `run` and `test` with the canonical invalid-entrypoint diagnostic. Wire this check before the
compilation pipeline starts.

### 6. Integration tests

- `kali run fixtures/hello.ts` → exits 0.
- `kali test fixtures/tests/` → runs test suite, reports correct pass/fail counts.
- `kali run fixtures/decl.d.ts` → exits 1 with invalid-entrypoint diagnostic.

## Out of Scope

- Sandbox policy enforcement (Stage 1.9 adds this on top of the runtime).
- `--api node` (Phase 3 target; `kali_api_node` stub only).
- `--api browser` runtime/test contract (Phase 1 non-goal).
- `eval` / `Function()` execution (Phase 4 compatibility).
- Threaded runtime profile (later compatibility).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
