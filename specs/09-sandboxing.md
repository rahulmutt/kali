# 09 — Sandboxing & Effects

## Overview

Sandboxing is a first-class concern in Kali. The system combines:
1. **Static effect analysis** — maintain a conservative capability-summary model, with a stable user-facing JSON report starting in Phase 2
2. **Sandbox policies** — declarative rules for what's allowed
3. **Runtime limits** — cross-cutting resource budgets (CPU, memory, open files, processes, threads) plus selected capability-local caps such as timers and network connections

## Static Effect Analysis

The static effect system is intentionally scoped around **sandbox-relevant capabilities** first. The goal is a conservative summary of possible effects, not a full research-grade effect calculus.

Phase simplification:
- **Phase 1**: internal effect bookkeeping may exist to support diagnostics/runtime integration, but the user-facing contract is runtime sandbox enforcement, policy-schema validation, and resource limits rather than a stable effect-report command
- **Phase 2+**: `kali effects`, compile-time effect-vs-policy validation, and explicit `pure` / effect annotations become part of the supported workflow

This keeps the sandbox-first story implementable: enforcement exists from the beginning, while the stable effect-report contract lands once the type/effect infrastructure is ready.

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
- `analysisContext`
- `entryPoints`
- `effects`
- `dynamicEffects`
- `dynamicReasons`

Scope rule:
- `analysisContext` records the semantic knobs that materially affect the report (`apiSurface`, `runtimeProfiles`, `compatFeatures`)
- `entryPoints` names the analysis roots
- the summarized `effects` cover the full statically reachable program/dependency graph rooted at those entry points under that recorded analysis context
- the report is therefore a conservative whole-program summary for that rooted graph, not a file-local listing of only the syntax inside the directly named source file

Other commands that embed effect data should place the full report under the CLI envelope's `payload` field instead of redefining the structure.

CLI simplification rule:
- `kali effects` is an observational reporting command, not a second policy-validation command
- therefore `kali effects --sandbox ...` is rejected rather than inventing a second place to compare effects against policy
- policy compatibility checks belong to `kali check --sandbox ...` and `kali build --sandbox ...`, which already own the pass/fail contract

### `dynamicEffects` Flag
Set to `true` when the report has one or more canonical `dynamicReasons` from [specs/18-schemas.md](18-schemas.md). That schema file is the single source of truth for the stable machine-readable reason codes.

In schema v1, these reasons use the canonical machine-readable codes from [specs/18-schemas.md](18-schemas.md):
- `eval`
- `function-constructor`
- `dynamic-import`
- `proxy-traps`
- `computed-host-access`

When `true`, the static analysis is incomplete — the sandbox must enforce at runtime.

## Sandbox Policies

### No-Policy Default

An attached sandbox policy is optional even though sandboxing is a first-class design concern.

Canonical behavior when no policy is attached:
- if neither `--sandbox <policy>` nor top-level `kali.json#sandbox` is provided, Kali runs with **no project policy file attached**
- a CLI `--sandbox <policy>` path is resolved relative to the current working directory; a relative `kali.json#sandbox` path is resolved relative to the directory containing that config file
- in that mode, Kali still enforces intrinsic guarantees such as API-surface/feature gating, WASM/runtime safety, and any direct invocation resource caps explicitly supplied on the CLI
- `kali check` / `kali build` simply skip policy validation when no policy is attached
- `kali run` / `kali test` skip policy-file-driven capability filtering when no policy is attached
- `--max-memory`, `--max-cpu`, `--max-open-files`, and later direct invocation resource-cap flags may still be used without a policy file; without a policy they become the effective cap directly

Important distinction:
- absence of a policy is **not** modeled as an implicit synthesized allow-all `kali.policy.json`
- tooling and diagnostics should preserve the difference between “no policy attached”, “policy attached and permissive”, and “policy attached and restrictive”

### Policy Definition
Sandbox policies are **declarative data files**, not arbitrary executable TypeScript. This keeps them auditable, easy to diff, and safe to evaluate before running untrusted code.

Default format: `kali.policy.json`

The canonical policy schema is defined in [specs/18-schemas.md](18-schemas.md). JSON is the canonical interchange format for CLI tooling and AI agents. An equivalent TOML format may be supported later, but it would be a convenience syntax layered on top of the JSON data model rather than a separate policy contract.

Cross-spec consistency rule:
- schema v1 string allowlists use the canonical matching rules from [specs/18-schemas.md](18-schemas.md)
- validation, compile-time effect-vs-policy checks, and runtime enforcement must all apply those same normalization/matching rules rather than inventing subsystem-specific pattern semantics
- schema v1 covers the built-in **Kali-mediated capability surface**, not every ambient browser/DOM API that may be visible during browser-targeted analysis/build

For process environment access, the policy model distinguishes `effects.process.envRead` from `effects.process.envWrite` so read-only inspection and mutation can be granted independently.

Policy-structure simplification rule:
- `effects.*` controls whether a capability exists and, where needed, capability-local allowlists/caps (for example URL patterns, timer counts, or network connection counts)
- `resources.*` is reserved for cross-cutting runtime budgets that apply regardless of which specific API triggered them (for example total memory, CPU time, open files, spawned processes, threads)
- schema v1 intentionally has **no** executable predicate/hook fields inside `kali.policy.json`; later programmable checks, if added, belong only to the embedding-oriented host-predicate extension described below
- specs should not duplicate the same numeric limit in both places under different names

### Policy Validation (Compile-Time)
Compile-time policy handling is intentionally split to keep Phase 1 smaller and less ambiguous:

- **Phase 1**: `--sandbox` validates the policy file itself (schema, patterns, resource-limit ranges, unsupported fields) and attaches it to the build/run configuration, but does **not** promise a complete static proof that all effects fit the policy.
- **Phase 2+**: inferred effects are checked against the allowed policy capabilities.
- For the hybrid `kali check` command, `kali check --sandbox <policy>` without explicit file arguments still uses the canonical project-discovery result; `--sandbox` adds policy validation, not a new input-selection mode.
- With explicit `check` file arguments, `--sandbox` keeps the same set-oriented semantics as plain `kali check`: it validates the supplied file set, and it does not collapse `check` into a one-entrypoint command just because a policy was attached.

Availability rule for policy validation:
- a policy may always **deny** a capability, even if that capability's corresponding API/feature is later-phase
- in schema v1, the canonical deny values for capability fields are `false` for boolean capabilities and `[]` for allowlist-shaped capabilities
- numeric limit/budget fields are **not** one generic "deny" channel across the whole schema: they remain numeric constraints with field-specific semantics
- omission is the canonical "no explicit budget provided" state for resource-budget fields such as `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles`
- `0` is meaningful only for the resource counters whose domain naturally allows zero concurrent uses (`resources.maxSpawnedProcesses`, `resources.maxThreads`); it is not the generic schema-wide deny value for every numeric field
- a policy must **not claim to allow** a capability that the selected command/profile/API surface/phase cannot actually provide
- therefore validation should reject any unavailable capability being enabled through a non-deny value, not just `true`; non-empty arrays/allowlists are equally invalid when the capability itself is unavailable, and unavailable numeric-budget fields such as `resources.maxSpawnedProcesses` / `resources.maxThreads` must also reject positive values
- capability-local numeric limit fields are **constraints only**, not implicit enable switches; for example `effects.network.maxConnections` does not by itself allow network use, and `effects.timer.maxActiveTimers` does not by itself permit timers when `effects.timer.schedule` is `false`
- examples include `effects.fileSystem.read: true` or `effects.fileSystem.read: ["/tmp/**"]` under an effective API surface of `browser`, `effects.eval: true` before Phase 4, `effects.process.spawn: true` before subprocess support exists, `effects.process.envWrite: true` before mutable env APIs exist, `resources.maxSpawnedProcesses > 0` before subprocess support exists, or `resources.maxThreads > 0` before the threaded runtime profile exists
- under an effective API surface of `browser`, this rejection applies to capabilities outside the browser-targeted Phase 1 surface, and it also applies to cross-cutting `resources.*` runtime budgets with any non-deny value because those budgets are a Kali-hosted execution contract rather than a post-deployment browser-bundle guarantee
- the shared Web-baseline capability keys (`effects.network.fetch`, `effects.timer.*`, `effects.random`, `effects.console`) remain valid browser-targeted policy targets at the capability-model level, but that does **not** upgrade browser bundles into a full cross-cutting runtime-budget-enforcement environment
- browser ambient DOM APIs are still outside the schema-v1 capability model even when browser typings are visible during analysis/build; policy validation must not imply there is a per-DOM-call sandbox key just because `Window`/`Document` types are available
- this avoids a misleading policy that appears more permissive than the runtime/compiler can really honor

Phase-1 capability snapshot for supported surfaces:

| Policy capability | Early availability | Notes |
|---|---|---|
| `effects.fileSystem.read` / `write` | Available with `--api deno` | Enforced for the documented Deno file APIs |
| `effects.process.envRead` | Available with `--api deno` | Read-only environment view only |
| `effects.network.fetch` | Available in the Web baseline | Shared across supported surfaces |
| `effects.timer.*` | Available in the Web baseline | Covers timers, not CPU-limit enforcement itself |
| `effects.random` | Available in the Web baseline | Maps to the documented random-byte capability family |
| `effects.console` | Available in the Web baseline | Console writes are policy-controlled |
| `effects.network.connect` / `listen` | Phase 3 target | Policy may deny them now; enabling them is rejected until the APIs exist |
| `effects.process.spawn` | Phase 3 target | Same rule as above |
| `effects.process.envWrite` | Phase 3 target | Same rule as above |
| `effects.eval` | Phase 4 compatibility | Reserved for the `--compat eval` path |
| `resources.maxSpawnedProcesses` | Phase 3 target | Becomes meaningful only once subprocess support exists |
| `resources.maxThreads` | Later compatibility (opt-in only) | Reserved for the later threaded runtime profile |

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
- All API calls check the same canonical path/URL/address/env matching rules described in [specs/18-schemas.md](18-schemas.md)
- Runtime enforcement only applies to capabilities that are actually registered for the selected API surface/profile; sandbox policy does not conjure unavailable APIs into existence

### Enforcement Domains
To keep the sandbox story precise across commands and deployment targets:
- **Kali-hosted runtime enforcement** applies to `kali run`, `kali test`, and embedding hosts that instantiate Kali-controlled host imports.
- **`check` / `build` with `--sandbox`** provide static validation only: policy-schema/config validation in Phase 1, plus effect-vs-policy validation in Phase 2+.
- **Browser-targeted builds** (`kali build --bundle --api browser`) may be checked against a policy at build time, but the emitted artifact running inside a real browser does not automatically inherit Kali runtime enforcement after deployment.

Interpretation rule:
- a successful browser-targeted build under `--sandbox` means the source graph is compatible with the supplied policy under Kali's static model
- it does **not** mean Kali can mediate every later browser-host capability once the bundle is deployed outside a Kali-controlled runtime
- browser ambient APIs that are outside the schema-v1 capability model (for example most DOM object operations) are therefore analysis/build concerns, not individually policy-governed runtime calls in early phases
- cross-cutting `resources.*` budgets are also outside the early browser-deployment guarantee; browser-targeted `check` / `build --bundle` may validate policy shape, but they must not imply post-deployment enforcement of CPU, memory, file-handle, process, or thread budgets in the real browser host
- specs and diagnostics should therefore avoid wording that suggests browser deployment has the same runtime-enforcement guarantee as `kali run` / `kali test`

## Runtime Resource Limits

For **Kali-hosted execution** (`kali run`, `kali test`, and embedding), runtime resource limits are enforced by the execution host (wasmtime in early phases).

Browser-targeted emitted artifacts do **not** automatically inherit Kali-hosted runtime resource enforcement after deployment into a real browser. Any browser-side budgeting beyond Kali's build-time checks would require a separate later host contract.

Cross-contract simplification:
- the schema-v1 `resources.*` block is a **Kali-hosted execution budget contract**
- therefore browser-targeted `check` / `build --bundle` may validate that the policy file is well-formed, but non-deny `resources.*` values must be rejected for that profile instead of implying post-deployment browser enforcement that Kali does not currently promise
- capability-local policy keys under `effects.*` remain the place where browser-targeted static compatibility can still be described for the documented Kali-mediated built-ins

Effective-limit rule:
- when a sandbox policy is attached, its values are the maximum capability/resource envelope for the run
- per-invocation CLI overrides such as `--max-memory`, `--max-cpu`, and `--max-open-files` may further tighten that envelope
- `--max-memory` literals normalize to bytes internally, while schema-v1 policy values are stored as `resources.maxMemoryMB`; comparison therefore happens after canonical unit conversion rather than by string matching
- `--max-cpu` literals normalize to milliseconds internally, while schema-v1 policy values are stored as `resources.maxCpuTimeMs`
- `--max-open-files` normalizes to an integer handle count and compares against `resources.maxOpenFiles`
- when no sandbox policy is attached, direct invocation caps become the effective envelope for the resource dimensions they cover
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
- `resources.maxSpawnedProcesses` is the cross-cutting cap for concurrently active child processes once subprocess APIs exist
- before subprocess support lands, policy validation should reject values greater than `0` here for the same reason it rejects `effects.process.spawn: true`: the policy must not appear to enable or budget for an unavailable capability

### Timer Limits
- Timer creation can be disabled entirely via `effects.timer.schedule: false`
- `setTimeout`/`setInterval` delays are capped by policy (`effects.timer.maxTimeoutMs`)
- Maximum number of active timers is enforced (`effects.timer.maxActiveTimers`)
- Infinite loop detection still relies on fuel metering

### Network Limits
- URL pattern matching applies to `fetch` allowlists (`effects.network.fetch`)
- Outbound socket-style connections can be disabled or gated separately (`effects.network.connect`)
- Port/address listeners can be disabled or gated separately (`effects.network.listen`)
- Concurrent network usage is capped by the capability-local field `effects.network.maxConnections`, not by `resources.*`; this keeps network-specific concurrency policy attached to the network capability instead of duplicating it as a second global resource knob

### Thread Limits (Later Threaded Profile)
- `resources.maxThreads` matters only for the later `--wasm-threads` runtime profile
- before that profile exists, policy validation should reject `resources.maxThreads > 0` rather than silently accepting a non-functional limit
- Once threading exists, the runtime must enforce the cap across worker/thread creation
- A per-invocation thread-limit override may only reduce the effective cap; it must never increase a stricter policy limit

## Sandbox Policy Predicates (Later Embedding-Only Extension)

The canonical maturity decision for this feature lives in [specs/19-feature-maturity.md](19-feature-maturity.md): the initial sandbox model is intentionally **declarative**.

Phase 1-2 policies are limited to path globs, URL patterns, booleans, and numeric resource limits. This keeps policy evaluation simple, auditable, portable, and easy to validate before any untrusted code runs.

Longer-term, Kali may support **host-registered sandbox policy predicates** for embedding scenarios where declarative allowlists are not expressive enough. This is the canonical interpretation of the bootstrap's programmable-policy idea: trusted hosts may register pure predicates, but `kali.policy.json` itself stays declarative data rather than becoming executable project code.

If policy predicates are added, they must:
- Be explicitly opt-in
- Be registered by the embedding host rather than loaded from arbitrary project code by default
- Be `pure` (no effects) and deterministic under the documented capability model
- Run synchronously before the guarded operation
- Return `false` → `SandboxViolationError`
- Receive a small canonical operation-context object rather than raw host handles, so policy checks stay auditable and portable

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
# Show inferred effects only (JSON; no policy comparison here)
kali effects program.ts

# Check the discovered project against a policy (no execution)
# Phase 1: validates the policy file/config only
# Phase 2+: also validates inferred effects against the policy
kali check --sandbox kali.policy.json

# Check one or more explicit files against a policy instead
kali check --sandbox kali.policy.json program.ts
kali check --sandbox kali.policy.json src/a.ts src/b.ts

# Run with sandbox enforcement
kali run --sandbox kali.policy.json program.ts

# Test with sandbox enforcement
kali test --sandbox kali.policy.json

# Run with resource limits only (no effect policy)
kali run --max-memory 256mb --max-cpu 10s --max-open-files 32 program.ts
```
