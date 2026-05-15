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
- Browser-harness max-thread smoke now also asserts the empty `threadTopology` snapshot in JSON output so the browser-requested threaded path stays aligned with the standalone contract.
- Browser-harness max-thread smoke now also accepts the zero-capable `--max-threads 0` budget with the same empty `threadTopology` snapshot, keeping the browser-threaded budget path aligned with the standalone zero-deny contract.
- Browser-requested browser-harness smoke now also accepts inherited browser `runtimeProfiles=["wasm-threads"]` configs in JS, TS, JSX, and TSX input alongside positive `--max-threads` requests.
- Browser-requested run/test JS smoke now also accepts the zero-capable `--max-threads 0` / `--max-spawned-processes 0` budget with matching human-output and JSON-output evidence.
- Preserve AOT-only compilation, no tracing/background GC, deterministic JSON, and resource-limit honesty.

### 22.2 Browser runtime contract

- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to stable standalone browser runtime support.
- Before promotion, specify host ownership, summary JSON behavior, sandbox limitations, diagnostics, and failure modes.
- Keep browser-targeted `check` / `build --bundle`, harness execution, and post-deployment browser behavior separate.

### 22.3 Late host APIs and resources

- Add subprocess, socket/listener, worker/thread, env materialization, env mutation, cwd/process-control, and late Node/Deno module support only with policy/effect/resource contracts.
- Keep host visibility aligned with effective `apiSurface`, command family, runtime profile, and maturity gate.
- Preserve explicit gates for unavailable host members and unsupported alias spellings.
- Browser late-compat process-control coverage now also rejects the alias-target zero-probe spelling `globalThis["process"]["kill"](+0)` across JS, TS, JSX, and TSX input so the negative matrix stays aligned with the documented Node zero-probe aliases.
- The Node API-surface positive regression now also covers the `Object.freeze(...)`-wrapped zero-probe aliases on the explicit and inherited Node surfaces.

### 22.4 Late object/runtime APIs

- Triage `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and wider object helpers against no-GC/no-JIT and optimization constraints.
- Promote only with conformance, sandbox/resource, effect-report, and JSON-output evidence.

## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Support wording names exact command, API surface, profile, and artifact/runtime context.
- Unsupported host/object surfaces fail through canonical diagnostics.
