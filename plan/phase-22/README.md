# Phase 22 — Host/Runtime Capability Contracts

## Goal

Expand runtime and host capability only where Kali can mediate, test, and describe it honestly.

## Owning specs

- `specs/09-sandboxing.md`
- `specs/10-runtime.md`
- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 22.1 Threaded runtime semantics

- Complete guest-facing threaded behavior beyond profile and budget acceptance.
- Define valid positive thread budgets by command, API surface, target, and runtime profile.
- Specify interaction with `SharedArrayBuffer`, `Atomics`, workers, resource accounting, and deterministic failure modes.
- Runtime smoke now also covers deterministic thread-topology snapshots for spawned workers, including empty posted-message and shared-buffer sets.
- Runtime outcomes now also expose deterministic thread-topology snapshots so worker accounting stays observable after execution.
- `kali run` / `kali test` JSON payloads now surface the deterministic `threadTopology` snapshot alongside the existing provenance labels.
- Preserve AOT-only compilation, no tracing/background GC, deterministic JSON, and resource-limit honesty.

### 22.2 Browser runtime contract

- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to stable standalone browser runtime support.
- Before promotion, specify host ownership, summary JSON behavior, sandbox limitations, diagnostics, and failure modes.
- Keep browser-targeted `check` / `build --bundle`, harness execution, and post-deployment browser behavior separate.

### 22.3 Late host APIs and resources

- Add subprocess, socket/listener, worker/thread, env materialization, env mutation, cwd/process-control, and late Node/Deno module support only with policy/effect/resource contracts.
- Keep host visibility aligned with effective `apiSurface`, command family, runtime profile, and maturity gate.
- Preserve explicit gates for unavailable host members and unsupported alias spellings.

### 22.4 Late object/runtime APIs

- Triage `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and wider object helpers against no-GC/no-JIT and optimization constraints.
- Promote only with conformance, sandbox/resource, effect-report, and JSON-output evidence.

## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Support wording names exact command, API surface, profile, and artifact/runtime context.
- Unsupported host/object surfaces fail through canonical diagnostics.
