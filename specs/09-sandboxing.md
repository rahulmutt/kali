# 09 — Sandboxing & Effects

## Overview

Sandboxing is a first-class concern in Kali. The system combines:
1. **Static effect analysis** — know all possible effects before running
2. **Sandbox policies** — declarative rules for what's allowed
3. **Runtime resource limits** — CPU, memory, processes, network

## Static Effect Analysis

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

Outputs:
```json
{
    "effects": [
        {
            "kind": "FileSystem.Read",
            "locations": [
                {"file": "program.ts", "line": 2, "col": 18, "function": "processFile"}
            ]
        },
        {
            "kind": "Console.Write",
            "locations": [
                {"file": "program.ts", "line": 3, "col": 5, "function": "processFile"}
            ]
        }
    ],
    "entryPoints": ["main"],
    "dynamicEffects": false,
    "usesEval": false
}
```

### `dynamicEffects` Flag
Set to `true` when:
- `eval` or `Function()` is used
- Dynamic `import()` with non-literal specifier
- `Proxy` with handler traps that could perform any effect
- Computed property access on host API objects

When `true`, the static analysis is incomplete — the sandbox must enforce at runtime.

## Sandbox Policies

### Policy Definition
```typescript
// sandbox.policy.ts
export const policy: SandboxPolicy = {
    effects: {
        fileSystem: { read: ["/data/**"], write: false },
        network: { fetch: ["https://api.example.com/*"], listen: false },
        process: { spawn: false, env: ["PATH", "HOME"] },
        timer: { maxTimeout: 5000 },
        eval: false,
        random: true,
    },
    resources: {
        maxMemoryMB: 256,
        maxCpuTimeMs: 10_000,
        maxOpenFiles: 10,
        maxSpawnedProcesses: 0,
        maxThreads: 0,
    },
};
```

### Policy Validation (Compile-Time)
When a policy is provided at build time:
1. Inferred effects are checked against allowed effects
2. Violations are **compile errors** (not warnings)
3. Unused permissions are reported as **warnings**

```bash
kali build --sandbox sandbox.policy.ts program.ts
```

```
error[E4001]: sandbox violation: FileSystem.Write not allowed
  --> program.ts:5:5
  |
5 |     Deno.writeTextFileSync("out.txt", result);
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = policy: fileSystem.write is disabled in sandbox.policy.ts:4
```

### Policy Validation (Runtime)
For dynamic effects that can't be checked at compile time:
- Host function imports are wrapped with policy-checking middleware
- Violations throw a `SandboxViolationError` (non-catchable by default)
- All API calls check path patterns, URL patterns, etc. at runtime

## Runtime Resource Limits

Enforced by the WASM host (wasmtime/wasmer):

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

## Sandbox Validator Functions

Users can define custom validation functions in the policy:

```typescript
export const policy: SandboxPolicy = {
    validators: {
        // Custom validator for file system access
        fileSystemRead(path: string): boolean {
            return path.startsWith("/safe/") && !path.includes("..");
        },
        // Custom validator for network access
        networkFetch(url: string): boolean {
            const u = new URL(url);
            return u.hostname === "api.example.com" && u.protocol === "https:";
        },
    },
};
```

Validator functions:
- Must be `pure` (no effects) — enforced by the compiler
- Run synchronously before the guarded operation
- Return `false` → `SandboxViolationError`
- Are themselves compiled by Kali and run in the host

## Algebraic Effect Handlers (Advanced)

Kali supports algebraic effects for advanced control over side effects:

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

This enables:
- **Testing**: Mock all I/O without dependency injection
- **Sandboxing**: Intercept and validate every effect occurrence
- **Composition**: Layer effect handlers for logging, caching, etc.

## Integration with CLI

```bash
# Show all effects (JSON)
kali effects program.ts

# Check program against a policy (no execution)
kali check --sandbox policy.ts program.ts

# Run with sandbox enforcement
kali run --sandbox policy.ts program.ts

# Run with resource limits only (no effect policy)
kali run --max-memory 256mb --max-cpu 10s program.ts
```
