# 09 — Sandboxing & Effects

## Overview

Sandboxing is a first-class concern in Kali. The system combines:
1. **Static effect analysis** — maintain a conservative capability-summary model, with a stable user-facing JSON report starting in Phase 2
2. **Sandbox policies** — declarative rules for what's allowed
3. **Runtime limits** — cross-cutting resource budgets (CPU, memory, open files, processes, threads) plus selected capability-local caps such as the `timer` family and network-connection limits

Cross-spec workflow rule:
- follow the shared **workflow-owner split** from [SPEC.md](../SPEC.md)
- this chapter therefore treats `effects` / `package-effects` as reporting-only surfaces, `check/build --sandbox` as the static policy-validation path, and `run/test --sandbox` as the runtime-enforcement path instead of letting those workflows blur together

Command-behavior simplification:

| Command family | `--sandbox` meaning in schema v1 | Runtime enforcement after command returns? |
|---|---|---|
| `run`, `test` | Attach policy, validate schema/ranges, and enforce it during **Kali-hosted execution** | Yes, for the documented Kali-hosted capability/resource contract |
| `check`, `build` | Static validation only: Phase 1 validates policy/schema/config; Phase 2+ also checks inferred effects against policy | No |
| browser-targeted `check` / `build --bundle` under an effective `apiSurface = browser` (including inherited-config forms) | Same static validation only, but narrowed by the shared **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md) | No |
| `effects`, `package-effects`, `package-audit` | No sandbox-comparison mode; `--sandbox` is invalid usage (`E5008`) | N/A |

This table is a reading aid only. The normative command-shape and phase-gating rules still live in [specs/12-cli.md](12-cli.md), [specs/19-feature-maturity.md](19-feature-maturity.md), and the shared terminology in [SPEC.md](../SPEC.md).

## Static Effect Analysis

The static effect system is intentionally scoped around **sandbox-relevant capabilities** first. The goal is a conservative summary of possible effects, not a full research-grade effect calculus.

Phase simplification:
- follow the shared **effect-surface split** from [SPEC.md](../SPEC.md)
- **Phase 1**: **internal effect bookkeeping** may exist to support diagnostics/runtime integration, but the user-facing contract is runtime sandbox enforcement, policy-schema validation, and resource limits rather than the stable **public effect-report surface**
- **Phase 2+**: that **public effect-report surface** arrives — `kali effects`, `kali package-effects`, compile/check-time effect-vs-policy validation, and explicit `pure` / effect annotations become part of the supported workflow

This keeps the sandbox-first story implementable: enforcement exists from the beginning, while the stable effect-report contract lands once the type/effect infrastructure is ready.

### Conceptual Effect Inference Model
The checker/runtime pipeline reasons about effects per function (see [specs/04-type-system.md](04-type-system.md)). In Phase 1 this may still be internal-only bookkeeping rather than a guaranteed user-visible command/report surface.

Illustrative internal summary:
```typescript
// Conceptual summary: ! FileSystem.Read | Console.Write
function processFile(path: string) {
    const data = Deno.readTextFileSync(path);
    console.log(data.length);
}
```

This example is intentionally descriptive, not a promise that Phase 1 already exposes stable per-function effect syntax or JSON output.

### Public JSON Effect Report (Phase 2+)
```bash
kali effects program.ts
```

`kali effects` is part of the Phase-2 **public effect-report surface**. `kali package-effects` joins that same surface for registry-package analysis and reuses the shared effect vocabulary/report contract with package-specific metadata layered on top. Before then, equivalent internal analysis may exist only as **internal effect bookkeeping** and does not need to be exposed as a stable user-facing command.

The canonical effect-report schema lives in [specs/18-schemas.md](18-schemas.md). This chapter treats that schema as the single source of truth for field names and payload shape.

Scope rule:
- `analysisContext` records the semantic knobs that materially affect the report: `apiSurface`, `runtimeProfiles`, and emitted JSON field `compatFeatures` (the flattened report form of config key `compat.features`; see [SPEC.md](../SPEC.md))
- schema v1 keeps the shared field name `entryPoints`, but it names the report's logical roots rather than promising runtime entrypoints in every producer
- for `kali effects`, those `entryPoints` are the analysis-root labels
- the summarized `effects` cover the full statically reachable program/dependency graph rooted at those logical roots under that recorded analysis context
- the report is therefore a conservative whole-program summary for that rooted graph, not a file-local listing of only the syntax inside the directly named source file

Commands that directly emit the shared effect-report payload (for example `kali effects --output json`) should place that report under the CLI envelope's `payload` field instead of redefining it. Commands that wrap the same effect data with extra package metadata (for example `kali package-effects`) should still reuse the canonical nested report shape from [specs/18-schemas.md](18-schemas.md) rather than inventing a second effect vocabulary.

CLI simplification rule:
- following the shared **workflow-owner split** from [SPEC.md](../SPEC.md), `kali effects` is an observational reporting command, not a second policy-validation command
- therefore `kali effects --sandbox ...` is rejected rather than inventing a second place to compare effects against policy
- policy compatibility checks belong to `kali check --sandbox ...` and `kali build --sandbox ...`, which already own the pass/fail contract

### `dynamicEffects` Flag
Set to `true` when the report has one or more canonical `dynamicReasons` from [specs/18-schemas.md](18-schemas.md). That schema file is the single source of truth for the stable machine-readable reason codes.

Interpretation rule:
- distinct `eval` and `function-constructor` report reasons help tooling explain *which* dynamic path was seen, but they still map to the single schema-v1 compatibility feature name `eval`
- this chapter intentionally does not restate the full reason-code list so schema-v1 machine strings stay owned in one place

When `true`, the static analysis is incomplete.
- for **Kali-hosted execution** (`run`, `test`, embedding), runtime sandbox enforcement remains the authoritative backstop for any operations the static report could not fully classify
- for browser-targeted analysis/build contexts, this flag is still valuable as a static warning signal, but it does **not** imply that deployed browser bundles automatically inherit Kali runtime enforcement after deployment

## Sandbox Policies

### No-Policy Default

An attached sandbox policy is optional even though sandboxing is a first-class design concern.

Canonical behavior when no policy is attached:
- if neither `--sandbox <policy>` nor top-level `kali.json#sandbox` is provided, Kali runs with **no project policy file attached**
- a CLI `--sandbox <policy>` path is resolved relative to the current working directory; a relative `kali.json#sandbox` path is resolved relative to the directory containing that config file
- in that mode, Kali still enforces intrinsic guarantees such as API-surface/feature gating, WASM/runtime safety, and any direct invocation resource caps explicitly supplied on the CLI
- `kali check` / `kali build` simply skip policy validation when no policy is attached
- `kali run` / `kali test` skip policy-file-driven capability filtering when no policy is attached
- `--max-memory`, `--max-cpu`, `--max-open-files`, and later profile-specific caps such as `--max-spawned-processes` and `--max-threads` may still be used without a policy file; without a policy they become the effective cap directly
- for later-gated capability-specific caps, `0` remains a valid explicit deny/tightening value even before the underlying capability exists, while non-zero values are still rejected until that capability/profile is actually supported

Important distinction:
- absence of a policy is **not** modeled as an implicit synthesized allow-all `kali.policy.json`
- tooling and diagnostics should preserve the difference between “no policy attached”, “policy attached and permissive”, and “policy attached and restrictive”

### Policy Definition
Sandbox policies are **declarative data files**, not arbitrary executable TypeScript. This keeps them auditable, easy to diff, and safe to evaluate before running untrusted code.

Canonical default filename: `kali.policy.json`.

Clarification:
- this is a filename convention, not a requirement that every policy path use that exact basename
- `--sandbox <policy>` may point to any explicit policy-file path
- JSON is still the canonical schema/interchange format for CLI tooling and AI agents in schema v1

The canonical policy schema is defined in [specs/18-schemas.md](18-schemas.md). An equivalent TOML format may be supported later, but it would be a convenience syntax layered on top of the JSON data model rather than a separate policy contract.

Cross-spec consistency rule:
- schema v1 string allowlists use the canonical matching rules from [specs/18-schemas.md](18-schemas.md)
- validation, compile-time effect-vs-policy checks, and runtime enforcement must all apply those same normalization/matching rules rather than inventing subsystem-specific pattern semantics
- schema v1 covers only the **Kali-mediated capability subset** from [SPEC.md](../SPEC.md), not every ambient browser/DOM API that may be visible during browser-targeted analysis/build

For process environment access, the policy model distinguishes `effects.process.envRead` from `effects.process.envWrite` so read-only inspection and mutation can be granted independently.

Compatibility-switch boundary:
- sandbox policy answers **"may this capability be used if the command/profile exposes it?"**
- CLI/config compatibility switches answer **"is this optional compatibility path enabled at all for this invocation?"**
- therefore a permissive policy entry such as `effects.eval: true` is only an authorization ceiling; it must not implicitly enable the separate `--compat eval` / `compat.features = ["eval"]` switch

Policy-structure simplification rule:
- `effects.*` controls whether a capability exists and, where needed, capability-local allowlists/caps (for example URL patterns, timer-family caps, or network-connection caps)
- `resources.*` is reserved for cross-cutting runtime budgets that apply regardless of which specific API triggered them (for example total memory, CPU time, open files, spawned processes, threads)
- schema v1 intentionally has **no** executable predicate/hook fields inside `kali.policy.json`; later programmable checks, if added, belong only to the embedding-oriented host-predicate extension described below
- specs should not duplicate the same numeric limit in both places under different names

Policy-decision layering rule:
- declarative policy data is always the first gate
- a later host-registered predicate may only make an already-allowed operation **more restrictive**, never widen a declarative deny into an allow
- this keeps project-visible policy review simple: `kali.policy.json` remains the portable baseline contract, while host predicates are a trusted embedding-specific narrowing layer
- CLI/config diagnostics should therefore continue to explain denials in terms of the declarative capability/resource model first, with predicate-specific detail as optional host-side context rather than as a replacement for the base policy model

### Policy Validation (Compile-Time)
Compile-time policy handling is intentionally split to keep Phase 1 smaller and less ambiguous:

- **Phase 1**: `--sandbox` validates the policy file itself (schema, patterns, resource-limit ranges, unsupported fields) and attaches it to the build/run configuration, but does **not** promise a complete static proof that all effects fit the policy.
- **Phase 2+**: inferred effects are checked against the allowed policy capabilities.
- For the hybrid `kali check` command, `kali check --sandbox <policy>` without explicit file arguments still uses the canonical project-discovery result; `--sandbox` adds policy validation, not a new input-selection mode.
- With explicit `check` file arguments, `--sandbox` keeps the same **set-oriented explicit-file command** behavior as plain `kali check`: it validates the supplied file set, and it does not collapse `check` into a one-entrypoint command just because a policy was attached.

Availability rule for policy validation:
- a policy may always **deny** a capability, even if that capability's corresponding API/feature is later-phase
- in schema v1, the canonical deny values for capability fields are `false` for boolean capabilities and `[]` for allowlist-shaped capabilities
- numeric limit/budget fields are **not** one generic "deny" channel across the whole schema: they remain numeric constraints with field-specific semantics
- omission is the canonical "no explicit budget provided" state for resource-budget fields such as `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles`
- `0` is meaningful only for the **feature-gated zero-capable execution budgets** from [SPEC.md](../SPEC.md) (`resources.maxSpawnedProcesses`, `resources.maxThreads`); it is not the generic schema-wide deny value for every numeric field
- a policy must **not claim to allow** a capability that the selected command/profile/API surface/phase cannot actually provide
- therefore validation should reject any unavailable capability being enabled through a non-deny value, not just `true`; non-empty arrays/allowlists are equally invalid when the capability itself is unavailable, and those **feature-gated zero-capable execution budgets** must also reject positive values until their corresponding capability/profile exists
- capability-local numeric limit fields are **constraints only**, not implicit enable switches; for example `effects.network.maxConnections` does not by itself allow network use, and `effects.timer.maxActiveTimers` does not by itself allow timer creation when `effects.timer.schedule` is `false`
- in schema v1, `effects.timer.maxTimeoutMs`, `effects.timer.maxActiveTimers`, and `effects.network.maxConnections` must be positive integers when present; `0` is invalid for those fields rather than a second disable/deny form
- examples include `effects.fileSystem.read: true` or `effects.fileSystem.read: ["/tmp/**"]` under an effective API surface of `browser`, `effects.eval: true` before Phase 4, `effects.eval: true` when the effective command context did not enable `--compat eval`, `effects.process.spawn: true` before subprocess support exists, `effects.process.envWrite: true` before mutable env APIs exist, or positive values for the **feature-gated zero-capable execution budgets** before subprocess/thread support exists
- under an effective API surface of `browser`, follow the **browser-targeted static sandbox contract**, the **canonical browser-targeted budget compatibility rule**, and the **canonical browser-applicable mediated subset (schema v1)** from [SPEC.md](../SPEC.md): browser-targeted `--sandbox` validation stays inside that documented browser-applicable subset, and schema-v1 `resources.*` fields are treated as Kali-hosted execution budgets rather than as post-deployment browser guarantees
- keep the exact browser-targeted `resources.*` accept/reject matrix and capability-family membership canonical in [SPEC.md](../SPEC.md) rather than maintaining a second near-duplicate table here
- browser-targeted validation therefore admits only the documented browser-applicable members of the global schema-v1 capability vocabulary; Deno/Node-only keys remain unavailable and must stay denied or omitted there
- browser ambient DOM APIs are still outside that schema-v1 subset even when browser typings are visible during analysis/build; policy validation must not imply there is a per-DOM-call sandbox key just because `Window`/`Document` types are available
- this avoids a misleading policy that appears more permissive than the runtime/compiler can really honor

Diagnostic boundary:
- use `E5010` when the policy file itself is malformed (unknown keys, wrong types, invalid matcher shapes, invalid numeric ranges)
- use `E5006` when the policy is well-formed but tries to enable a real capability/profile that is unavailable in the effective command/profile/API-surface context
- this keeps policy validation aligned with [specs/15-errors.md](15-errors.md) and the CLI exit-code rules in [specs/12-cli.md](12-cli.md)

Phase-1 capability snapshot for supported surfaces:

| Policy capability | Early availability | Notes |
|---|---|---|
| `effects.fileSystem.read` / `write` | Available with `--api deno` | Enforced for the documented Deno file APIs; schema v1 also treats metadata/read-dir APIs such as `Deno.stat*` and `Deno.readDir*` as part of `fileSystem.read` rather than separate metadata keys |
| `effects.process.envRead` | Available with `--api deno` | Read-only environment view only; covers both `Deno.env.get` and `Deno.env.toObject` |
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

Interpretation note:
- this snapshot focuses on capability families and the resource fields whose availability is phase-gated
- always-valid positive-budget fields for **Kali-hosted execution** — `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` — are intentionally omitted from the table because they are already part of the Phase 1 runtime-budget contract rather than separate later-phase capability gates

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
  = policy: effects.fileSystem.write is disabled in kali.policy.json
```

### Policy Validation (Runtime)
For dynamic effects that can't be checked at compile time:
- Host function imports are wrapped with policy-checking middleware
- Violations terminate the current operation with `SandboxViolationError`
- By default, sandbox violations are treated as fatal runtime errors for the top-level execution unless the embedding host explicitly opts into catchable host exceptions
- All API calls check the same canonical path/URL/address/env matching rules described in [specs/18-schemas.md](18-schemas.md)
- Runtime enforcement only applies to capabilities that are actually registered for the selected API surface/profile; sandbox policy does not conjure unavailable APIs into existence
- query-only **observation-only compatibility facades** over already-resolved sandbox/runtime state (for example Phase-1 `Deno.permissions.query`; see the canonical term in [SPEC.md](../SPEC.md)) are effect-free in schema v1 and therefore do not require a separate policy key

### Enforcement Domains
To keep the sandbox story precise across commands and deployment targets:
- **Kali-hosted runtime enforcement** applies to `kali run`, `kali test`, and embedding hosts that instantiate Kali-controlled host imports.
- For embedding, that same rule covers both executable-style helpers and library-oriented instantiation/calls: creating an instance from a `--lib`-style module and invoking its proved exports is still **Kali-hosted execution**, not a second unsandboxed host path.
- **`check` / `build` with `--sandbox`** provide static validation only: policy-schema/config validation in Phase 1, plus effect-vs-policy validation in Phase 2+.
- **Browser-targeted builds** in the shared **Phase-1 browser-targeted command set** (that is, `kali build --bundle <file>` when the effective `apiSurface` is `browser`, including inherited-config forms) follow the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md): they may be checked against a policy at build time for the documented mediated subset, but the emitted artifact running inside a real browser does not automatically inherit Kali runtime enforcement after deployment.

Interpretation rule:
- a successful browser-targeted build under `--sandbox` means the source graph is compatible with the supplied policy under Kali's static model for the **Kali-mediated capability subset**
- it does **not** mean Kali can mediate every later browser-host capability once the bundle is deployed outside a Kali-controlled runtime
- browser ambient APIs that are outside that schema-v1 subset (for example most DOM object operations) are therefore analysis/build concerns, not individually policy-governed runtime calls in early phases
- cross-cutting `resources.*` budgets are also outside the early browser-deployment guarantee; browser-targeted `check` / `build --bundle` should therefore reuse the shared **canonical browser-targeted budget compatibility rule** from [SPEC.md](../SPEC.md) instead of restating a second per-chapter budget list
- specs and diagnostics should therefore avoid wording that suggests browser deployment has the same runtime-enforcement guarantee as `kali run` / `kali test`

Quick browser-targeted examples:
- `kali check --sandbox web.policy.json` under an effective browser API surface is a static compatibility verdict only
- `kali build --bundle --sandbox web.policy.json app.ts` under an effective browser API surface is a static compatibility verdict plus bundle generation only
- explicit `--api browser` spellings and equivalent inherited-config forms are the same browser-targeted static-policy-validation path once the effective `apiSurface` resolves to `browser`
- a browser-targeted policy may constrain the documented browser-applicable capability-local keys such as `effects.network.maxConnections` or `effects.timer.maxActiveTimers`
- the same browser-targeted policy must still satisfy the shared **canonical browser-targeted budget compatibility rule** from [SPEC.md](../SPEC.md)

## Runtime Resource Limits

For **Kali-hosted execution** (`kali run`, `kali test`, and embedding), runtime resource limits are enforced by the execution host (wasmtime in early phases).

Embedding clarification:
- this includes library-oriented instantiation too: top-level module initialization performed at instantiate time and later export calls both run inside the same **effective execution envelope** as other Kali-hosted execution paths
- building a `--lib` artifact is therefore not itself a runtime event, but instantiating or calling that artifact through Kali-controlled imports is

Browser-targeted emitted artifacts do **not** automatically inherit Kali-hosted runtime resource enforcement after deployment into a real browser. Any browser-side budgeting beyond Kali's build-time checks would require a separate later host contract.

Cross-contract simplification:
- the schema-v1 `resources.*` block is a **Kali-hosted execution budget contract**
- for browser-targeted `check` / `build --bundle`, follow the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md) instead of restating a second browser-budget rule here
- capability-local policy keys under `effects.*` remain the place where browser-targeted static compatibility can still be described for the documented Kali-mediated built-ins

Effective-limit rule:
- for Kali-hosted execution, the final runtime ceiling is the **effective execution envelope** from [SPEC.md](../SPEC.md)
- when a sandbox policy is attached, its values are the maximum capability/resource envelope for the run
- per-invocation CLI overrides such as `--max-memory`, `--max-cpu`, `--max-open-files`, and later profile-specific caps such as `--max-spawned-processes` and `--max-threads` may further tighten that envelope
- `--max-memory` literals normalize to bytes internally, while schema-v1 policy values are stored as `resources.maxMemoryMB`; comparison therefore happens after canonical unit conversion rather than by string matching
- `--max-cpu` literals normalize to milliseconds internally, while schema-v1 policy values are stored as `resources.maxCpuTimeMs`
- `--max-open-files` normalizes to an integer handle count and compares against `resources.maxOpenFiles`
- `--max-spawned-processes` normalizes to an integer child-process count and compares against `resources.maxSpawnedProcesses`
- `--max-threads` normalizes to an integer thread count and compares against `resources.maxThreads`
- when no sandbox policy is attached, direct invocation caps still contribute to that effective envelope for the resource dimensions they cover
- for later-gated capability-specific caps, `0` remains a valid explicit deny/tightening value, while non-zero values still require that the underlying capability/profile already be supported
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
- this field follows the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value, while positive values must still be rejected until subprocess support exists

### Timer Limits
- Timer creation can be disabled entirely via `effects.timer.schedule: false`
- `setTimeout`/`setInterval` delays are capped by policy (`effects.timer.maxTimeoutMs`)
- Maximum number of active timers is enforced (`effects.timer.maxActiveTimers`)
- `effects.timer.maxTimeoutMs` and `effects.timer.maxActiveTimers` are positive-integer constraints when present; `0` is rejected instead of being treated as an alternate disable channel
- Infinite loop detection still relies on fuel metering

### Network Limits
- URL pattern matching applies to `fetch` allowlists (`effects.network.fetch`)
- Outbound socket-style connections can be disabled or gated separately (`effects.network.connect`)
- Port/address listeners can be disabled or gated separately (`effects.network.listen`)
- Concurrent network usage is capped by the capability-local field `effects.network.maxConnections`, not by `resources.*`; this keeps network-specific concurrency policy attached to the network capability instead of duplicating it as a second global resource knob
- in browser-targeted contexts, that cap applies only to the modeled browser-targeted `effects.network.fetch` path from the **canonical browser-applicable mediated subset (schema v1)**, not to arbitrary ambient browser networking APIs outside the schema-v1 model
- `effects.network.maxConnections` is a positive-integer constraint when present; `0` is rejected instead of being treated as an alternate deny form

### Thread Limits (Later Threaded Profile)
- `resources.maxThreads` matters only for the later `--wasm-threads` runtime profile
- this field follows the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value, while positive values must still be rejected until the threaded profile exists
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
- Be a **narrowing layer only**: they may reject an operation that the declarative policy would otherwise allow, but they must not authorize an operation that the declarative policy, command profile, or feature-maturity gate already rejected
- Return `false` → `SandboxViolationError`
- Receive a small canonical operation-context object rather than raw host handles, so policy checks stay auditable and portable
- Use one canonical operation vocabulary aligned with the schema-v1 capability model (`effects.*` / `resources.*`) instead of inventing a second unrelated predicate namespace

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
# Phase 2 target: show inferred effects only (JSON; no policy comparison here)
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
kali run --max-spawned-processes 0 program.ts
```
