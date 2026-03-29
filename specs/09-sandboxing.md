# 09 — Sandboxing & Effects

## Overview

Sandboxing is a first-class concern in Kali. The system combines:
1. **Static effect analysis** — produce a conservative summary of possible effects before running, marking dynamic/incomplete cases explicitly
2. **Sandbox policies** — declarative rules for what's allowed
3. **Runtime resource limits** — CPU, memory, processes, network

## Static Effect Analysis

The static effect system is intentionally scoped around **sandbox-relevant capabilities** first. The initial goal is a conservative JSON summary of possible effects, not a full research-grade effect calculus.

Phase simplification:
- **Phase 1**: runtime sandbox enforcement, policy-schema validation, and resource limits work without requiring full static effect reports.
- **Phase 2+**: `kali effects`, compile-time effect-vs-policy validation, and explicit `pure` / effect annotations become part of the supported workflow.

This keeps the sandbox-first story implementable: enforcement exists from the beginning, while richer static analysis lands once the type/effect infrastructure is ready.

### Effect Inference
The type checker infers effects for every function (see [specs/04-type-system.md](04-type-system.md)):
```typescript
// Inferred: ! FileSystem.Read | Console.Write
function processFile(path: string) {
    const data = Deno.readTextFileSync(path);
    console.log(data.length);
}
```

### JSON Effect Report
```bash
kali effects program.ts
```

`kali effects` is a Phase 2 target feature. Before then, equivalent internal analysis may exist only as compiler infrastructure and does not need to be exposed as a stable user-facing command.

The canonical effect-report schema lives in [specs/18-schemas.md](18-schemas.md). The report contains:
- `schemaVersion`
- `entryPoints`
- `effects`
- `dynamicEffects`
- `dynamicReasons`

Other commands that embed effect data should place the full report under the CLI envelope's `payload` field instead of redefining the structure.

### `dynamicEffects` Flag
Set to `true` when the report has one or more canonical `dynamicReasons` from [specs/18-schemas.md](18-schemas.md). That schema file is the single source of truth for the stable machine-readable reason codes.

In schema v1, these reasons cover cases such as:
- `eval` or `Function()`
- Dynamic `import()` with non-literal specifier
- `Proxy` with handler traps that could perform any effect
- Computed property access on host API objects

When `true`, the static analysis is incomplete — the sandbox must enforce at runtime.

## Sandbox Policies

### Policy Definition
Sandbox policies are **declarative data files**, not arbitrary executable TypeScript. This keeps them auditable, easy to diff, and safe to evaluate before running untrusted code.

Default format: `kali.policy.json`

The canonical policy schema is defined in [specs/18-schemas.md](18-schemas.md). JSON is the canonical interchange format for CLI tooling and AI agents. An equivalent TOML format may be supported later, but it would be a convenience syntax layered on top of the JSON data model rather than a separate policy contract.

For process environment access, the policy model distinguishes `effects.process.envRead` from `effects.process.envWrite` so read-only inspection and mutation can be granted independently.

Policy-structure simplification rule:
- `effects.*` controls whether a capability exists and, where needed, capability-local allowlists/caps (for example URL patterns or timer counts)
- `resources.*` is reserved for cross-cutting runtime budgets that apply regardless of which specific API triggered them (for example total memory, CPU time, open files, spawned processes, threads)
- specs should not duplicate the same numeric limit in both places under different names

### Policy Validation (Compile-Time)
Compile-time policy handling is intentionally split to keep Phase 1 smaller and less ambiguous:

- **Phase 1**: `--sandbox` validates the policy file itself (schema, patterns, resource-limit ranges, unsupported fields) and attaches it to the build/run configuration, but does **not** promise a complete static proof that all effects fit the policy.
- **Phase 2+**: inferred effects are checked against the allowed policy capabilities.

In Phase 2+ when a policy is provided at build or check time:
1. Inferred effects are checked against allowed effects
2. Violations are **compile errors** (not warnings)
3. Unused permissions are reported as **warnings**

```bash
kali build --sandbox kali.policy.json program.ts
```

```
error[E4001]: sandbox violation: FileSystem.Write not allowed
  --> program.ts:5:5
  |
5 |     Deno.writeTextFileSync("out.txt", result);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = policy: fileSystem.write is disabled in kali.policy.json
```

### Policy Validation (Runtime)
For dynamic effects that can't be checked at compile time:
- Host function imports are wrapped with policy-checking middleware
- Violations terminate the current operation with `SandboxViolationError`
- By default, sandbox violations are treated as fatal runtime errors for the top-level execution unless the embedding host explicitly opts into catchable host exceptions
- All API calls check path patterns, URL patterns, etc. at runtime

## Runtime Resource Limits

Enforced by the WASM host (wasmtime in initial phases).

Effective-limit rule:
- sandbox policy values are the maximum capability/resource envelope for the run
- per-invocation CLI overrides such as `--max-memory` and `--max-cpu` may further tighten that envelope
- CLI/config must not silently widen a stricter sandbox policy at runtime


### CPU Limits
- **Fuel-based**: wasmtime's fuel mechanism — each WASM instruction consumes fuel
- Configurable fuel budget maps to approximate CPU time
- When fuel runs out → `ResourceLimitError`

### Memory Limits
- WASM linear memory max pages configured per policy
- Host tracks total allocation via custom allocator callbacks
- OOM → `ResourceLimitError`

### File Handle Limits
- Concurrent host file handles are capped by `resources.maxOpenFiles`
- The limit applies to explicit file APIs and to internal file opens performed on behalf of higher-level read/write helpers
- Exceeding the cap fails the operation with `ResourceLimitError`

### Process Limits
- Process spawning goes through host functions → policy-checked
- Count of active child processes is capped by `resources.maxSpawnedProcesses`

### Timer Limits
- Timer creation can be disabled entirely via `effects.timer.schedule: false`
- `setTimeout`/`setInterval` delays are capped by policy (`effects.timer.maxTimeoutMs`)
- Maximum number of active timers is enforced (`effects.timer.maxActiveTimers`)
- Infinite loop detection still relies on fuel metering

### Network Limits
- URL pattern matching applies to `fetch` allowlists (`effects.network.fetch`)
- Outbound socket-style connections can be disabled or gated separately (`effects.network.connect`)
- Port/address listeners can be disabled or gated separately (`effects.network.listen`)
- Concurrent network usage is capped by `effects.network.maxConnections`

### Thread Limits (Later Threaded Profile)
- `resources.maxThreads` matters only for the later `--wasm-threads` runtime profile
- Before that profile exists, policy validation should reject `maxThreads > 0` rather than silently accepting a non-functional limit
- Once threading exists, the runtime must enforce the cap across worker/thread creation
- A per-invocation thread-limit override may only reduce the effective cap; it must never increase a stricter policy limit

## Sandbox Validator Functions (Later Phase)

The canonical maturity decision for this feature lives in [specs/19-feature-maturity.md](19-feature-maturity.md): the initial sandbox model is intentionally **declarative**.

Phase 1-2 policies are limited to path globs, URL patterns, booleans, and numeric resource limits. This keeps policy evaluation simple, auditable, portable, and easy to validate before any untrusted code runs.

Custom validator functions are a later-phase extension for embedding scenarios. If added, they must:
- Be explicitly opt-in
- Be `pure` (no effects) — enforced by the compiler
- Run synchronously before the guarded operation
- Return `false` → `SandboxViolationError`
- Integrate through the embedding API as host-registered validators, rather than requiring the runtime to self-host arbitrary policy code by default

## Algebraic Effect Handlers (Advanced, Experimental)

Algebraic effects are a later-phase feature. They are explicitly optional for the initial implementation and should not block delivery of capability summaries, policy checking, or runtime enforcement.

Illustrative syntax:

```typescript
effect FileSystem {
    read(path: string): string;
    write(path: string, content: string): void;
}

function processFile(path: string): string ! FileSystem {
    const content = perform FileSystem.read(path);
    return content.toUpperCase();
}

// Handle the effect — intercept and provide implementation
handle processFile("/data/input.txt") {
    FileSystem.read(path) => {
        // Could redirect to in-memory FS, mock, log, etc.
        return inMemoryFS.get(path) ?? "";
    }
}
```

If implemented, this enables:
- **Testing**: Mock all I/O without dependency injection
- **Sandboxing**: Intercept and validate every effect occurrence
- **Composition**: Layer effect handlers for logging, caching, etc.

## Integration with CLI

```bash
# Show all effects (JSON)
kali effects program.ts

# Check program against a policy (no execution)
# Phase 1: validates the policy file/config only
# Phase 2+: also validates inferred effects against the policy
kali check --sandbox kali.policy.json program.ts

# Run with sandbox enforcement
kali run --sandbox kali.policy.json program.ts

# Run with resource limits only (no effect policy)
kali run --max-memory 256mb --max-cpu 10s program.ts
```
