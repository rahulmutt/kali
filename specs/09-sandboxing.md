# 09 — Sandboxing & Effects

## Overview

Sandboxing is a first-class concern in Kali. The system combines:
1. **Static effect analysis** — know all possible effects before running
2. **Sandbox policies** — declarative rules for what's allowed
3. **Runtime resource limits** — CPU, memory, processes, network

## Static Effect Analysis

The static effect system is intentionally scoped around **sandbox-relevant capabilities** first. The initial goal is a conservative JSON summary of possible effects, not a full research-grade effect calculus.

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

The canonical effect-report schema lives in [specs/18-schemas.md](18-schemas.md). The report contains:
- `schemaVersion`
- `entryPoints`
- `effects`
- `dynamicEffects`
- `usesEval`

Other commands that embed effect data should place the full report under the CLI envelope's `payload` field instead of redefining the structure.

### `dynamicEffects` Flag
Set to `true` when:
- `eval` or `Function()` is used
- Dynamic `import()` with non-literal specifier
- `Proxy` with handler traps that could perform any effect
- Computed property access on host API objects

When `true`, the static analysis is incomplete — the sandbox must enforce at runtime.

## Sandbox Policies

### Policy Definition
Sandbox policies are **declarative data files**, not arbitrary executable TypeScript. This keeps them auditable, easy to diff, and safe to evaluate before running untrusted code.

Default format: `kali.policy.json`

The canonical policy schema is defined in [specs/18-schemas.md](18-schemas.md). JSON is the canonical interchange format for CLI tooling and AI agents. An equivalent TOML format may be supported later, but it would be a convenience syntax layered on top of the JSON data model rather than a separate policy contract.

### Policy Validation (Compile-Time)
When a policy is provided at build time:
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

Enforced by the WASM host (wasmtime in initial phases):

### CPU Limits
- **Fuel-based**: wasmtime's fuel mechanism — each WASM instruction consumes fuel
- Configurable fuel budget maps to approximate CPU time
- When fuel runs out → `ResourceLimitError`

### Memory Limits
- WASM linear memory max pages configured per policy
- Host tracks total allocation via custom allocator callbacks
- OOM → `ResourceLimitError`

### Process Limits
- Process spawning goes through host functions → policy-checked
- Count of active child processes tracked and limited

### Timer Limits
- `setTimeout`/`setInterval` delays capped by policy
- Maximum number of active timers enforced
- Infinite loop detection via fuel

### Network Limits
- URL pattern matching on fetch/connect
- Port/address restrictions on listen
- Connection count limits

## Sandbox Validator Functions (Later Phase)

The initial sandbox model is intentionally **declarative**: path globs, URL patterns, booleans, and numeric resource limits. This keeps policy evaluation simple, auditable, and portable.

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
kali check --sandbox kali.policy.json program.ts

# Run with sandbox enforcement
kali run --sandbox kali.policy.json program.ts

# Run with resource limits only (no effect policy)
kali run --max-memory 256mb --max-cpu 10s program.ts
```
