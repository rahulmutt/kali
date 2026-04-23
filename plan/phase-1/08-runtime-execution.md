# Stage 1.8 — Runtime & Execution

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** [1.7 — WASM Code Generation](07-wasm-codegen.md)

**Status:** ✅ Complete — runtime execution is wired through wasmtime, the default standalone
context is exercised end to end, and the Stage-1.8 evidence lane is closed.

## Goal

Implement Kali-hosted execution: integrate `wasmtime` as the execution engine, wire the
**Default standalone context (schema v1)**, and make `kali run` / `kali test` functional for real
TypeScript/JavaScript programs.

## Workable Milestone

- `kali run <file> [-- args...]` compiles and executes a TS/JS source file in the default standalone context.
- `kali test [files...]` discovers or accepts explicit test files and reports pass/fail results.
- The Phase-1 standalone host surface is available end to end:
  - Deno-oriented file/metadata access in the documented Phase-1 subset
  - `Deno.args`
  - read-only `Deno.env.get` / `Deno.env.toObject`
  - query-only `Deno.permissions`
  - the shared Web baseline used by the standalone surface (`console`, timers,
    `queueMicrotask`, `fetch`, `URL`, `TextEncoder`/`TextDecoder`, `AbortController`,
    `structuredClone`, `performance.now()`, `crypto.getRandomValues`, and event primitives)
- Exit codes and runtime diagnostics are stable.

## Progress

- `kali_runtime` is wired through wasmtime for emitted WASM modules.
- `kali run` and `kali test` drive the compiler output end to end.
- Declaration-only primary inputs are rejected for runtime-bearing commands with `E5007`.
- Smoke tests cover successful runs, explicit-file test reporting, discovery-driven test runs,
  declaration-only rejection, guest-registered test callbacks, and `run` guest-argument
  passthrough after `--`.
- The default standalone host surface now includes the documented file/metadata subset,
  read-only env/args, query-only permissions projection, fetch, timers/microtasks, and the Phase-1
  Web-baseline primitives.
- Explicit `--api deno` smoke coverage proves the spelled-out default matches the implicit path,
  and the semver-style Node regression now exercises the `process.argv` help/argument flow through
  the documented `--` split.
- Added a semver consumer/runtime regression on the default standalone surface so the common
  `valid` / `satisfies` / `minVersion` package calls now stay observable with exact stdout instead
  of collapsing to placeholder zeros.
- The unresolved imported bindings/call-target fallback diagnostics now also carry structured
  source-context metadata, and the regression suite now asserts the source-path note alongside the
  requested/effective context so the compatibility escape hatch stays machine-readable without
  changing the existing fallback behavior.
- The semver probe follow-up now tracks that structured source-context coverage explicitly so the
  remaining compatibility gap stays visible in the stage notes instead of being implied by the
  codegen path alone.

## Historical stage tasks

### 1. wasmtime integration (`kali_runtime`)

Use one documented pure-Rust execution engine for early Kali-hosted execution:

- instantiate emitted WASM modules with a stable host-import table
- execute through one store/host-state model
- map traps and unhandled runtime failures onto Kali diagnostics
- preserve the AOT-only rule: engine-level WASM compilation/caching is not a second Kali language
  compilation tier

### 2. Default standalone context (schema v1)

Implement the Phase-1 standalone host surface.

#### Console / stdio

- `console.log`
- `console.warn`
- `console.error`

#### File system / metadata (Deno-oriented Phase-1 subset)

- read/write helpers in the documented Phase-1 subset
- metadata / directory-read helpers in the documented Phase-1 subset
- all host calls routed through the same runtime/sandbox mediation path

#### Environment and invocation context

- `Deno.args`
- `Deno.env.get`
- `Deno.env.toObject`
- query-only `Deno.permissions` over Kali's already-resolved policy/runtime state

#### Network / timers / Web baseline

- `fetch`
- timers and microtasks
- `URL`, `TextEncoder`, `TextDecoder`, `AbortController`, `structuredClone`
- `performance.now()`
- `crypto.getRandomValues()`
- core event primitives

### 3. `kali run` subcommand

```bash
kali run <file> [-- args...]
kali run --sandbox <policy> <file> [-- args...]
kali run --api deno <file> [-- args...]
```

The runtime path compiles, instantiates, executes, and reports runtime failures through the
canonical diagnostics and exit-code contract.

### 4. `kali test` subcommand

```bash
kali test [files...]
kali test --filter <pattern> [files...]
kali test --sandbox <policy> [files...]
```

The test runner supports project discovery and explicit-file-set execution while keeping the same
runtime host/context rules as `kali run`.

### 5. Declaration-only file rejection

Declaration-only files (`.d.ts`, `.d.mts`, `.d.cts`) are not valid runtime-bearing primary inputs.
Reject them with the canonical invalid-entrypoint diagnostic before execution begins.

### 6. Evidence

- positive `run` fixtures
- positive `test` fixtures
- declaration-only negative fixtures
- host-API integration tests for the Phase-1 standalone surface

## Follow-up work uncovered by the semver probe

A real `semver` execution attempt exposed runtime-path gaps that should be tracked explicitly.
One of them — the `--` guest-argument split for `kali run` — is now fixed and regression-covered;
package-execution semantics now also have a dedicated Node-path semver package-json/version smoke,
and the unresolved imported bindings/call-target placeholder fallback remains tracked explicitly,
while broader CommonJS lowering gaps remain tracked explicitly.

### Semver-specific regression surfaces

- `kali run --api node node_modules/semver/bin/semver.js -- 1.2.3` previously parsed `1.2.3` as
  another primary source input; the CLI now treats everything after `--` as guest arguments, and
  the regression suite covers both the no-args help path and the argument-flow path.
- A small consumer program importing `semver` built and ran, but produced incorrect runtime output,
  showing that the package-execution path once allowed unresolved or mis-lowered imported
  functionality to reach execution instead of preserving real package semantics or failing earlier;
  the current regression suite now pins the correct stdout for that consumer.

### Systematic fix plan

1. Tighten the compile→run handoff so unresolved imported bindings/call targets are no longer
   silently lowered into executable placeholder values; they must either lower faithfully or stop
   the run with a hard diagnostic before WASM emission.
2. Add an end-to-end package execution regression using the real `semver` consumer/library case so
   runtime output is compared against known-good behavior rather than only checking exit success.
3. Extend the package-bin regression for `semver/bin/semver.js` to cover the remaining Node-path
   slices from the original probe, including `require('../package.json')`, the exact help-path
   output shape, and guest-argument counting on the Node path.

## Out of Scope

- sandbox enforcement details owned by Stage 1.9
- mutable environment access (`Deno.env.set`) and subprocess/socket/listener work tracked later
- process identity/control and working-directory APIs tracked later
- `--api node` runtime support (Phase 3)
- standalone browser runtime/test support (later compatibility)
- executable `eval` / `Function()` compatibility (Phase 4)
- threaded runtime profile (later compatibility)

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
