# Phase 22 — Host/Runtime Capability Contracts

## Goal

Expand runtime and host capability only where Kali can mediate, test, and describe it honestly.

Keep browser-targeted compilation, browser harness execution, and standalone browser runtime claims separate.

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
- Current smoke now also rejects single-quoted bracketed `SharedArrayBuffer` / `Atomics` aliases in the threaded-runtime gate, keeping the late-compat spellings honest alongside the positive budget path; the browser JS/JSX/TSX source inventories now also carry those single-quoted bracketed aliases, the shared threaded-runtime alias inventory is now canonicalized in `kali_common`, the canonical helper set now also includes the matching single-quoted `true &&` / `null ??` wrapper variants, the double-quoted `false ||` wrapper variants, and the direct dot-root `true &&` / `false ||` wrapper variants for both globals, and the CLI build rejection lane reuses the same canonical late-threaded-runtime source inventory.
- The shared late-object-model inventory now also carries the parenthesized single-quoted receiver-wrapped `Proxy.revocable` alias family, including the bracket-access form and frozen wrappers, alongside the other late-object/runtime spellings, keeping the browser and runtime smoke aligned on that wrapper family too; the same family now also includes the parenthesized receiver-wrapped bracket-access frozen aliases.
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

- Triage `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and wider object helpers against no-GC/no-JIT and optimization constraints; the shared late-object-model inventory now also carries frozen bracket-root spellings for `WeakRef` and `FinalizationRegistry`, including the single-quoted variants, plus single-quoted bracketed `Proxy` constructor aliases, so the browser and runtime smoke stay aligned on those aliases across the browser JS/JSX/TSX fixture variants. The shared broader-Intl browser JS helper now also carries single-quoted bracketed member spellings for `DateTimeFormat`, `RelativeTimeFormat`, `PluralRules`, `Collator`, `DisplayNames`, `Segmenter`, and `Locale` on the same rejection path, now including the matching single-quoted-root `globalThis['Intl'].<member>` and `globalThis['Intl']["<member>"]` forms for the same member set, and the browser JSX/TSX late-compat inventories now mirror that same single-quoted bracketed `Intl` member set on the browser rejection path; that helper inventory is now also canonicalized and unit-tested in `kali_common`, and the standalone Deno `build` rejection smoke now mirrors the same single-quoted-bracketed Intl member coverage across the TS/JS/JSX/TSX matrix. The same late-object helper family now also includes bracket-root frozen property-access aliases for `Object.hasOwn`, keeping the object-helper smoke aligned with the existing bracket-root frozen `keys` / `values` / `entries` coverage.
- Promote only with conformance, sandbox/resource, effect-report, and JSON-output evidence.
- Keep unsupported object/runtime APIs on canonical diagnostics rather than partial emulation.

## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Support wording names exact command, API surface, profile, and artifact/runtime context.
- Unsupported host/object surfaces fail through canonical diagnostics.
