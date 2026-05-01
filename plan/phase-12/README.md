# Phase 12 — Runtime, Host, and Capability Expansion

## Goal

Expand host/runtime capability only where Kali can mediate, test, and describe it honestly.

## Owning specs

- `specs/09-sandboxing.md`
- `specs/10-runtime.md`
- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 12.1 Threaded runtime semantics

- Complete guest-facing threaded execution beyond profile acceptance and host-import plumbing.
- Keep positive thread budgets valid only under the supported threaded profile and target.
- Preserve no tracing/background GC, AOT-only compilation, deterministic JSON, and resource-budget enforcement.

### 12.2 Browser runtime contract

- Decide whether harness-assisted `run --api browser` / `test --api browser` should become a stable standalone browser runtime contract.
- If promoted, specify host ownership, sandbox limitations, summary-file fallback rules, supported commands, and diagnostics before changing support wording.
- Keep browser-targeted `check` / `build --bundle` distinct from standalone browser execution and post-deployment sandbox enforcement.

### 12.3 Late host APIs

- Add environment materialization/mutation, process identity/control, cwd/chdir, subprocess, and socket/listener APIs only with explicit effect keys, policy behavior, and resource limits.
- Keep host visibility aligned with `apiSurface` and command context.
- Maintain canonical gates for unavailable Node/Deno/browser host members.
- Progress note: the documented Node surface now exposes the read-only `process.pid` query across the explicit and inherited `check` / `build` / `run` / `test` paths; `process.cwd` / `process.chdir` / `process.exit` remain gated.

### 12.4 Late object/runtime APIs

- Triage `Proxy`, own-property helpers, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, and `Atomics` against no-GC/no-JIT and optimization constraints.
- Promote only with conformance evidence and sandbox/resource implications documented.

## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Browser/runtime support wording names exact command/context/profile.
- Unsupported host/object surfaces fail through the canonical diagnostic path.
