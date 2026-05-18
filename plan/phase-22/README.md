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
- Runtime outcomes now also expose deterministic thread-topology snapshots so worker accounting stays observable after execution, and the browser-requested run/test harness path now also preserves those snapshots when guest code uses the threaded host import.
- `kali run` / `kali test` JSON payloads now surface the deterministic `threadTopology` snapshot alongside the existing provenance labels.
- The thread-topology regression set now also pins monotonic instance-id assignment after termination/re-spawn, keeping worker accounting deterministic when a released slot is followed by a new worker.
- Browser-harness max-thread smoke now also asserts the empty `threadTopology` snapshot in JSON output so the browser-requested threaded path stays aligned with the standalone contract.
- Browser-harness max-thread smoke now also accepts the zero-capable `--max-threads 0` budget with the same empty `threadTopology` snapshot, keeping the browser-threaded budget path aligned with the standalone zero-deny contract.
- Browser-requested browser-harness smoke now also accepts inherited browser `runtimeProfiles=["wasm-threads"]` configs in JS, TS, JSX, and TSX input alongside positive `--max-threads` requests.
- Browser-requested run/test JS, TS, JSX, and TSX smoke now also accepts the zero-capable `--max-threads 0` / `--max-spawned-processes 0` budget with matching human-output and JSON-output evidence, including the dedicated zero-spawned-process regression added in the current repository, while positive `--max-spawned-processes` requests now also reject with the canonical `E5506` budget gate on the browser-requested and explicit browser-API paths.
- Preserve AOT-only compilation, no tracing/background GC, deterministic JSON, and resource-limit honesty.

### 22.2 Browser runtime contract

- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to stable standalone browser runtime support.
- Before promotion, specify host ownership, summary JSON behavior, sandbox limitations, diagnostics, and failure modes.
- Keep browser-targeted `check` / `build --bundle`, harness execution, and post-deployment browser behavior separate.

### 22.3 Late host APIs and resources

- Add subprocess, socket/listener, worker/thread, env materialization, env mutation, cwd/process-control, and late Node/Deno module support only with policy/effect/resource contracts.
- Keep host visibility aligned with effective `apiSurface`, command family, runtime profile, and maturity gate.
- Preserve explicit gates for unavailable host members and unsupported alias spellings.
- Browser late-compat process-control coverage now also rejects the alias-target zero-probe spelling `globalThis["process"]["kill"](+0)` across JS, TS, JSX, and TSX input, plus the receiver-freeze aliases and their +0 siblings `Object.freeze(process)["kill"](0)` / `Object.freeze(globalThis.process)["kill"](0)` / `Object.freeze(globalThis["process"])["kill"](0)`, so the negative matrix stays aligned with the documented Node zero-probe aliases. Browser-targeted build smoke now also carries the bracketed `Object.freeze(globalThis["process"]["kill"])(+0)` alias on that same slice, and the browser late-compat smoke corpus now mirrors that bracketed frozen `+0` alias across the JS, TS, JSX, and TSX rejection fixtures.
- The direct call-target binding helper now also centralizes the `process.kill` / `globalThis.process["kill"]` / `globalThis["process"]["kill"]` aliases used by the Node API-surface smoke, so the callable-target aliases stay single-sourced alongside the existing sequence-callable-target helper.
- The Node API-surface positive regression now also covers the `Object.freeze(...)`-wrapped zero-probe aliases on the explicit and inherited Node surfaces.
- The browser late-compat TSX zero-probe inventory now also asserts the parenthesized freeze-wrapper aliases around `globalThis.process["kill"]` so it stays aligned with the shared inventory helper, and the shared zero-probe matrix now also includes the parenthesized callable-freeze `Object.freeze((process["kill"]))(0/+0)` spelling in the same Node/browser alias family.
- The Node API-surface process-control regression now also covers the bracketed `process["cwd"]` / `process["chdir"]` / `process["exit"]` spellings and their `globalThis.process[...]` / `globalThis["process"][...]` aliases on the documented Node surface in JS, JSX, and TSX input, including the explicit `globalThis["process"]["cwd"]` / `globalThis["process"]["chdir"]` / `globalThis["process"]["exit"]` spellings in the current corpus, and the shared zero-probe helper tests now pin those bracketed process-control aliases as part of the canonical late-process-control prefix.
- The explicit and inherited Node API-surface late-module rejection matrices now also pin `node:worker_threads` on the same canonical `E5506` path across `.js`, `.jsx`, and `.tsx` input as the explicit Node surface.

### 22.4 Late object/runtime APIs

- Triage `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and wider object helpers against no-GC/no-JIT and optimization constraints. Frozen `Proxy.revocable` aliases now also reject through the same canonical `E5506` path in runtime, browser, and Node smoke, including the Node API-surface frozen-alias regression coverage, and the shared late-object-model helper source now carries that frozen alias slice in `kali_common`. Browser-harness/browser-bundle `Math.pow` smoke now also covers parenthesized freeze-wrapper spellings around `globalThis.Math["pow"]` and `globalThis["Math"]["pow"]` on the supported alias-chain slice.
- Promote only with conformance, sandbox/resource, effect-report, and JSON-output evidence.

## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Support wording names exact command, API surface, profile, and artifact/runtime context.
- Unsupported host/object surfaces fail through canonical diagnostics.
