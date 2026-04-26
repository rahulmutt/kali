# 11 — Standard APIs

## Strategy

Implement APIs through one shared guest-facing capability model. Using the canonical terminology from [SPEC.md](../SPEC.md), Kali realizes that model through different **host adapters**: the native host adapter for Kali-hosted execution and the browser host adapter for browser-targeted bundle output. In the target implementation structure, each API surface is organized as a separate crate that defines the relevant bindings/registration logic for that surface.

Repository-state note: API-surface crate names in this chapter describe the intended ownership/decomposition. The repository may already contain similarly named crates, but command/API-surface availability still comes only from [19 — Feature Maturity](19-feature-maturity.md).

Using the canonical **host-support staircase** from [SPEC.md](../SPEC.md), compatibility is delivered in layers:
1. **Web baseline**: shared platform primitives needed by modern JS libraries.
2. **Primary standalone host**: Deno-style APIs, since they align well with explicit permissions and sandboxing.
3. **Browser-targeted context**: in Phase 1, the shared **Phase-1 browser-targeted command set** plus the ambient typing/build support needed to target the real browser host rather than a standalone Kali browser runtime.
4. **Node compatibility surface**: later package-driven compatibility work, added incrementally and tested against real packages.

The spec goal is broad compatibility, but the implementation should prefer a smaller, dependable surface over a shallow imitation of every host API.

A key simplification rule applies throughout this section: Phase 1 should target the smallest API set that unlocks real-world package execution, and every later API addition should be justified by package compatibility or standards pressure.

Consistency note:
- browser work in Phase 1 is limited to the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md), whose exact command boundary is owned there rather than re-expanded ad hoc in each chapter
- Node work is a later compatibility surface, not a second Phase-1 standalone host peer
- references to browser support in this chapter should therefore prefer the cross-spec **browser-targeted context** wording, and in Phase 1 usually the exact **Phase-1 browser-targeted command set**, instead of implying one broad "browser runtime" milestone

For dynamic or semantically expensive APIs (for example `Proxy`, weak references, and threaded primitives), the canonical phase/status lives in [specs/19-feature-maturity.md](19-feature-maturity.md). This section should describe API layering, not restate a conflicting maturity decision.

## API-Surface Loading Rule

For the compact cross-spec summary of early host/API behavior, see the canonical **Host/API Summary** in [SPEC.md](../SPEC.md).

To keep runtime imports, globals, and package expectations aligned:
- the **Web baseline** is the shared baseline across supported surfaces
- `--api deno`, `--api node`, and `--api browser` control which **additional** ambient globals/modules beyond that baseline are available for the selected supported command/profile
- for the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md), `--api browser` means the real browser ambient typing layer, not merely the smaller **Kali-mediated capability subset** used by schema-v1 sandbox/effect contracts; see the **Browser ambient typing vs mediated capability split** in [SPEC.md](../SPEC.md)
- browser-targeted contexts must not expose process/env/file globals just because the underlying host runtime happens to have them
- unsupported globals/modules are absent; Kali must not invent dummy shims by default
- use the canonical `E5506` availability path for **documented command/profile or feature gating** (for example `--api node` before Phase 3, or `run --api browser` in early phases)
- use ordinary unresolved-name/type diagnostics when code references a global that simply is not part of the selected ambient surface in an otherwise-supported mode (for example `document` under `--api deno`)

This prevents a common source of drift: host-runtime implementation convenience must not silently widen the language-visible API contract.

Canonical terminology simplification:
- use the cross-spec term **browser-targeted context** from [SPEC.md](../SPEC.md) for command contexts whose effective `apiSurface` is `browser`
- in Phase 1, that user-visible browser-targeted context is exactly the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md)
- later analysis commands may reuse that same ambient typing layer and **package-resolution context** once their own maturity rows allow it
- it does **not** mean a standalone embedded browser runtime, DOM emulation layer, or permission to expose non-browser globals during analysis/build

## Command/API-Surface Snapshot

This chapter describes API layering, but command availability still follows the shared **support-claim reading order**, **package-support ladder** where package compatibility is involved, and **availability context** terms from [SPEC.md](../SPEC.md), plus the maturity matrix.

Phase-1 reading aid:

| Command family / build lane | `deno` | `browser` | `node` |
|---|---|---|---|
| `check` | the shared **default source-graph analysis context (schema v1)**; in support-ladder terms this is the ordinary **checkable** path, including the supported static `--sandbox` validation variants and equivalent inherited-config forms | browser-targeted analysis context; in support-ladder terms this is the browser **checkable** path, including the supported static `--sandbox` validation variants and equivalent inherited-config forms | gated |
| `build` (default executable lane) | the shared **Deno-oriented build context (schema v1)**, including the supported static `--sandbox` validation variants | invalid command shape in early phases; browser executable builds require `--bundle` and therefore stay on the canonical `E5508` contradiction path until that selector is present | gated |
| `build --bundle` (browser-targeted executable/deploy lane) | invalid command shape in schema v1; `--bundle` is browser-only and stays on the canonical `E5508` contradiction path under a non-browser effective API surface | browser-targeted bundle only; in support-ladder terms this is the browser **deployable-through-host** path, including supported static `--sandbox` variants and equivalent inherited-config forms, and it remains distinct from any standalone browser **executable** contract | invalid command shape in schema v1 too; explicit `--api node` does not create a second maturity-gated bundle lane because `--bundle` is already reserved for browser-targeted output |
| `build --lib` (export-oriented build lane) | the Phase-1 **base library artifact** path for **exact-version consumers** in the shared **Deno-oriented build context (schema v1)** — but only when Kali can determine a **statically known export surface** — including `build --lib --sandbox`; here, that Deno-oriented build context is the build/analysis default rather than a claim that Phase-1 library outputs expose a Deno-specific public ABI | invalid command shape in early phases; browser/library combinations stay on the canonical `E5508` contradiction path rather than becoming a hidden browser embedding mode | gated |
| `run`, `test` | the shared **Default standalone context (schema v1)**; in support-ladder terms this is the standalone **executable** path | not yet a standalone browser-runtime/test **executable** contract | gated |
| later semantic-analysis/reporting reuse (`effects`; inherited-context-only `package-effects`; and later browser-context analysis reuse) | later Deno-oriented analysis/reporting reuse once those commands exist | the same browser-targeted analysis context is reused later without widening the shared **Phase-1 browser-targeted command set** | gated |

Interpretation rules:
- this is a command/API-surface snapshot only; the owning CLI shape still lives in [12 — CLI](12-cli.md), and exact phase labels still live in [19 — Feature Maturity](19-feature-maturity.md)
- explicit CLI spellings and equivalent inherited-config forms mean the same effective request once `apiSurface` resolves to `deno`, `browser`, or `node`; inherited config must not silently widen or narrow support
- practical Phase-1 simplification: browser-targeted support here means only the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md); it does **not** mean standalone `run` / `test`, and it does **not** open browser library/embed artifact lanes
- `browser` in this table means **checkable** / **deployable-through-host** support where noted, not a hidden standalone DOM runtime promise
- for browser-only artifact selectors such as `build --bundle`, both non-browser columns describe the same schema-v1 contradiction path (`E5508`) rather than two different maturity stories
- `node` stays gated across these command families until the documented Node subset exists; package-compatibility work must not imply an undocumented partial `--api node` mode earlier
- the last row is a context-reuse reading aid, not a Phase-1 command-availability claim; exact command maturity still comes from [19 — Feature Maturity](19-feature-maturity.md)
- registry-analysis commands remain distinct even when they reuse analysis context: later `package-effects` may inherit the same browser/Node analysis context and shared browser package-resolution rule once its own maturity row opens, while `package-audit` remains context-free in schema v1 and is intentionally excluded from the table's last row
- simplification rule: read that last row in two halves — `effects` may use explicit `--api ...` when its command exists, while `package-effects` keeps semantic context **inherited-only** in schema v1. That reuse story is about shared analysis semantics, not about inventing a second package-analysis flag family.

Support-claim phrasing shortcut:
- when this chapter says a surface is available for a command family, read that through the shared **support-claim reading order** and **availability context** terms from [SPEC.md](../SPEC.md) rather than as one broad yes/no promise
- for early browser-facing claims, prefer wording such as **checkable** or **deployable-through-host** over vague “browser support” prose
- reserve **executable** for Kali-hosted runtime/test contracts only
- for Phase-1 `build --lib`, prefer wording such as **buildable for exact-version consumers** over generic “embedding support” prose so the Phase-1 **base library artifact** does not get overread as the later **public embedding surface**

## API Layers

### Web baseline
Available across supported execution surfaces as the shared baseline.

Interpretation rule:
- in standalone execution (`run` / `test`), this baseline is present for supported runtime profiles
- in browser-targeted output (`build --bundle --api browser`), this is the baseline the emitted code targets in the real browser host through the browser host adapter
- in browser-targeted analysis-only commands such as `check --api browser`, this section describes the ambient typing surface being checked, not runtime provisioning by Kali
- command/profile combinations that are themselves phase-gated are still rejected according to [specs/19-feature-maturity.md](19-feature-maturity.md)
- this baseline list describes the JS-visible API contract, not a one-host-import-per-item requirement: some entries are expected to be implemented in Kali's guest/runtime support library rather than as dedicated host imports (for example `queueMicrotask`, `URL`, `TextEncoder`, `TextDecoder`, `AbortController`, `structuredClone`, and event primitives)

**Phase 1 MVP baseline**
- `console` (`log`, `warn`, `error`, `debug`, `info`, `assert`)
- `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`
- `queueMicrotask`
- `fetch`, `Headers`, `Request`, `Response`
- `URL`, `URLSearchParams`
- `TextEncoder`, `TextDecoder`
- `AbortController`, `AbortSignal`
- `structuredClone`
- `performance.now()`
- `EventTarget`, `Event`, `CustomEvent`
- `crypto.getRandomValues` as the narrow randomness-only Web Crypto subset needed for the canonical `Random.GetBytes` effect/policy path

Interpretation rule:
- Phase 1 exposes only the minimal randomness subset (`crypto.getRandomValues`) rather than the full Web Crypto API surface
- this keeps the API/effect/policy model aligned with the schema-v1 `effects.random` capability and the `Random.GetBytes` effect name without overpromising broader cryptography support

**Later compatibility expansion**
- `Blob`, `File`, `FormData`
- `ReadableStream`, `WritableStream`, `TransformStream`
- broader `crypto` / Web Crypto surface beyond the MVP randomness subset (for example `crypto.subtle`)
- `atob`, `btoa`
- `WebSocket`

### Deno API (`--api deno`, default)
Deno is the primary standalone-runtime API surface because it fits Kali's explicit sandbox model.

**Phase 1 MVP subset**
- File APIs: `Deno.readTextFile`, `Deno.readTextFileSync`, `Deno.writeTextFile`, `Deno.writeTextFileSync`, `Deno.readFile`, `Deno.readFileSync`, `Deno.writeFile`, `Deno.writeFileSync`
- Metadata APIs: `Deno.stat`, `Deno.statSync`, `Deno.readDir`, `Deno.readDirSync`
- Invocation arguments: `Deno.args`
- Environment access: `Deno.env.get`, `Deno.env.toObject` *(both expose only the sandbox-permitted environment view rather than the raw host environment)*
- `Deno.permissions` as the canonical **observation-only compatibility facade** over Kali sandbox policy state; in Phase 1 this is a **query-only** surface that reports granted/denied capability state and does not provide interactive permission prompts or privilege-escalation flows (the canonical maturity decision lives in [specs/19-feature-maturity.md](19-feature-maturity.md), and the cross-spec terminology lives in [SPEC.md](../SPEC.md))

Effect/sandbox mapping simplification:
- `Deno.stat*` and `Deno.readDir*` stay under the existing `effects.fileSystem.read` capability rather than introducing separate metadata-directory effect keys in schema v1
- `Deno.env.get` and `Deno.env.toObject` stay under `effects.process.envRead`
- as an **observation-only compatibility facade**, query-only `Deno.permissions` observation is derived from already-resolved Kali sandbox/runtime state and is therefore **effect-free** in schema v1; it does not add a second `permissions.query` effect/policy key

Implementation simplification:
- this read-only `Deno.permissions` facade should normally be derived from Kali's already-resolved runtime/policy state rather than from a separate permission-prompt host API
- that keeps the Deno compatibility story aligned with the sandbox-first model: permission status is observed, not negotiated interactively at runtime
- Phase 1 therefore keeps one compact split:
  - `Deno.permissions.query(...)` is the only **stable callable path** in the facade
  - `Deno.permissions.request(...)` / `revoke(...)` remain **recognized-but-unavailable compatibility members** and therefore fail with the canonical availability path (`E5506`) instead of disappearing as ordinary missing properties; the same applies to statically-known string-literal bracketed property forms such as `Deno.permissions["request"](...)` and `globalThis["Deno"]["permissions"]["revoke"](...)`
  - this compatibility-visible rejection is intentional simplification, not a hidden Phase-2/3 promise: unless a future sandbox model explicitly reopens interactive escalation, these members stay documented as unavailable rather than silently graduating into a roadmap lane
- accepted `query(...)` descriptor names follow the shared **Deno-compatible permission descriptor subset (schema v1)** from [SPEC.md](../SPEC.md); in Phase 1 that effectively means the `read` / `write` / `net` / `env` subset, but each descriptor still projects only the capability slice that actually exists for the active phase/API surface

Phase-1 descriptor projection shorthand:

| `Deno.permissions.query(...)` descriptor | What it observes in Phase 1 | Must **not** imply |
|---|---|---|
| `read` | the documented file/metadata read slice (`effects.fileSystem.read`) | a second metadata-only permission family |
| `write` | the documented file-write slice (`effects.fileSystem.write`) | unrelated host-mutation capabilities |
| `env` | the read-only environment slice (`effects.process.envRead`) | Phase-3 environment mutation support |
| `net` | the modeled fetch-only network slice (`effects.network.fetch`) | future socket/listener permissions |

- practical consequence: in the Phase 1 standalone contract, `Deno.permissions.query({ name: "env" })` reports only the read-only environment capability state, and `Deno.permissions.query({ name: "net" })` observes the modeled `fetch` capability state only; neither descriptor implies that the broader later-phase mutation/socket/listener surfaces already exist just because the descriptor name is broad
- Kali's broader schema-v1 capability/effect vocabulary still includes the `timer` family, random, console, and later `eval`, but those are **not** surfaced as synthetic `Deno.permissions.query({ name: ... })` descriptor kinds in schema v1. This keeps the Deno-compat API smaller and avoids implying non-standard Deno permission names.
- returned states follow the shared **stable permission status subset (schema v1)** from [SPEC.md](../SPEC.md)
- to keep checker and runtime behavior aligned, unsupported `query(...)` descriptor kinds should also fail with `E5506` rather than degrading into silent `denied`, fake `prompt`, or missing-surface drift
- type checking should model that same split: Kali's Deno-compat typing for this facade should expose the shared descriptor subset and stable status subset `"granted" | "denied"` for `query(...)`, while keeping `request()` / `revoke()` and the corresponding statically-known string-literal bracketed property forms in the documented **recognized-but-unavailable compatibility member** lane rather than advertising an implemented interactive permission flow

For host-capability maturity, the canonical source of truth is [specs/19-feature-maturity.md](19-feature-maturity.md). In particular:
- read-only environment access is part of the Phase 1 standalone contract
- mutable environment access, subprocess spawning, and socket/listener networking follow the Phase 3 maturity path; this is why the Phase 1 `net` permission observation remains fetch-only rather than implying `connect`/`listen`
- process identity query `Deno.pid` / `globalThis.Deno.pid` is already available on the default standalone surface in the current repository snapshot, while termination and working-directory APIs remain a later compatibility path until a future schema/policy revision gives them an auditable contract

Process identity `Deno.pid` / `globalThis.Deno.pid` is a read-only query on the default standalone surface in the current repository snapshot, while process termination (`Deno.exit`) and working-directory mutation/introspection (`Deno.cwd`, `Deno.chdir`) remain intentionally outside the Phase 1 MVP. Those latter APIs widen the embedding/sandbox contract but are not needed for the initial package-oriented baseline.

Rule of thumb: when Kali exposes a Deno file/metadata API in Phase 1, it should expose the sync and async forms together unless there is a strong implementation reason not to. This avoids needless package-compatibility drift between `readFile` and `readFileSync`-style code paths.

**Phase 3 target expansion**
- `Deno.open`, `Deno.create`, `Deno.mkdir`, `Deno.remove`, `Deno.rename`, `Deno.lstat`
- `Deno.env.set`
- `Deno.Command` (process spawning)
- `Deno.serve` (HTTP server / listen path)
- broader filesystem, networking, and subprocess coverage

**Later compatibility expansion**
- `Deno.pid` / `globalThis.Deno.pid` *(current repository snapshot: read-only query on the default standalone surface)*
- `Deno.cwd`, `Deno.chdir`
- `Deno.exit`

Cross-spec consistency note:
- subprocess, mutable-environment, and network/listener APIs fit schema-v1's policy vocabulary
- process identity, termination, and working-directory APIs do **not** yet have dedicated schema-v1 policy/effect keys
- therefore `Deno.exit`, `Deno.cwd`, and `Deno.chdir` remain **Later compatibility** features until a future schema/policy revision makes their sandbox contract explicit; `Deno.pid` / `globalThis.Deno.pid` stay noted as current-repository read-only queries in the snapshot above

This keeps the Phase 1 host surface small and auditable while still establishing Deno as the default API model.

### Node.js API (`--api node`)
Node compatibility is a **Phase 3 ecosystem target**, not a Phase 1 promise. The goal is package compatibility first, not full Node parity.

Canonical gating rule:
- `kali check --api node`, `kali effects --api node`, `kali build --api node`, `kali run --api node`, and `kali test --api node` are all phase-gated until the documented Node subset exists
- early phases must reject these modes with the canonical `E5506` diagnostic instead of exposing a partial ambient `process`/built-ins surface

**Phase 3 target subset**
- `fs`, `fs/promises` — file system operations
- `path` — path manipulation (pure, no host calls needed)
- `buffer` — Buffer class
- `events` — EventEmitter
- `util` — utilities (promisify, inspect, etc.)
- `url` — URL parsing
- `assert` — assertions
- `process` — process global subset needed by real packages first (`env`, `argv`, selected control/query helpers); `pid`, `exit`, and `cwd`-style process-introspection/control APIs stay on the later compatibility path until the policy and embedding contract for them is specified

**Later compatibility expansion**
- `os`
- `child_process`
- `http`, `https`
- `crypto`
- `stream`
- `querystring`
- remaining Node core modules justified by real package demand

**Strategy**: Implement Node APIs as wrappers around Deno-style host functions where possible. Use `deno_std/node` as a compatibility reference, not as a hard dependency.

### Browser API (`--api browser`)
Browser mode is primarily a **browser-targeted context** in early phases, not a promise that the standalone runtime behaves like a browser. See the canonical host/profile summary in [SPEC.md](../SPEC.md) and the phase-gating matrix in [19 — Feature Maturity](19-feature-maturity.md) when deciding whether a given command/profile combination is supported.

Three browser meanings matter here and should not be conflated:

| Layer | Answers | Phase-1 contract | Does **not** mean |
|---|---|---|---|
| **browser ambient typing surface** | what may type-check in a browser-targeted context | the shared **Phase-1 browser-targeted command set** type-checks against the real browser ambient surface, including ordinary DOM typings | Kali's standalone runtime implements or emulates the DOM |
| **browser mediated capability subset** | what Kali can model in schema-v1 sandbox/effect terms | browser-targeted policy/effect reasoning uses only the documented **canonical browser-applicable mediated subset (schema v1)** | one stable policy/effect key per DOM API, or automatic access to every global capability key |
| **standalone runtime host surface** | what Kali can execute itself without handing control to a real browser host | still later compatibility for `run --api browser` / `test --api browser` | a hidden embedded browser engine or DOM-runtime parity in Phase 1 |

Canonical rule:
- the shared **Phase-1 browser-targeted command set** uses the real browser ambient surface and the shared browser **package-resolution context** from [SPEC.md](../SPEC.md) and [14 — Packages](14-packages.md), so ambient typing and package entry selection do not drift apart
- this follows the top-level **Browser ambient typing vs mediated capability split** in [SPEC.md](../SPEC.md): browser ambient typing is broader than the stable sandbox/effect model
- when Kali emits browser-targeted artifacts, DOM/Web APIs come from the real browser host at deployment time; the browser host adapter is for runtime bootstrap plus Kali-mediated capability wiring, not a promise that every ambient browser API is individually wrapped by Kali
- schema-v1 sandbox policies and stable effect reports still cover only the **Kali-mediated capability subset** from [SPEC.md](../SPEC.md): browser-targeted contexts keep only the documented **canonical browser-applicable mediated subset (schema v1)** — notably `effects.network.fetch` plus `effects.network.maxConnections`, `effects.timer.*`, `effects.random`, `effects.console`, and later `effects.eval` when enabled — while Deno/Node-only keys remain unavailable there
- `--sandbox` on a browser-targeted build therefore follows the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md), not automatic post-deployment browser-permission enforcement by Kali itself
- no Deno or Node globals are exposed in the browser-targeted context unless a later compatibility spec explicitly says so
- any lightweight DOM test shim is a separate testing utility, not part of the core browser compatibility contract

This resolves a common ambiguity: browser-targeted analysis may know about `document`/`window`, while standalone execution still rejects browser-runtime assumptions because Kali does not embed a browser engine.

**Canonical early-phase rule**:
- follow the **canonical browser-surface rejection split** from [SPEC.md](../SPEC.md)
- the only Phase-1 supported browser-targeted paths are the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md)
- browser-targeted build shapes requested outside that canonical boundary use `E5508` rather than `E5506`; examples include `kali build --api browser ...`, `kali build --lib --api browser ...`, `kali build --capi --api browser ...`, and `kali build --component --api browser ...`
- `kali run --api browser ...` and `kali test --api browser ...` use `E5506` in early phases because Kali does not yet define a standalone browser runtime/test contract

**Note**: For supported command/profile combinations, the Phase 1 **Web baseline** APIs are available regardless of `--api` mode. Follow the shared **API-loading rule** from [SPEC.md](../SPEC.md): on analysis/build commands, `--api` chooses ambient typing, package-resolution, and policy/effect-modeling context; on executable commands, it also chooses the runtime host surface. Browser-targeted bundle output is the deployment-host path for the real browser rather than evidence of a hidden Kali browser runtime. Early unsupported command/surface combinations should follow that same split rather than silently falling back.

## Phase 1 Host API Exit Criteria

This section turns the broad API story into a small implementation checklist so runtime, CLI, and testing do not drift:

- **Kali-hosted standalone Web baseline must work end to end**: `console`, timers, `queueMicrotask`, `fetch`, `URL`, `TextEncoder`/`TextDecoder`, `AbortController`, `structuredClone`, `performance.now()`, the MVP randomness subset (`crypto.getRandomValues`), and event primitives are available in Kali-hosted `run` / `test` on the supported standalone surface and covered by integration tests.
- **Deno baseline must work end to end**: file read/write, metadata/read-dir, invocation arguments, and read-only env access all execute through the host ABI and obey the documented sandbox/execution contract.
- **Every Phase 1 host call is sandbox-contract-aware**: the runtime may not expose an unchecked host backdoor just because the API itself is part of the MVP. When a policy file is attached, host calls must consult it; when no policy file is attached, the same host-call path must still honor intrinsic phase/API gating plus any direct invocation resource caps instead of bypassing the sandbox machinery entirely.
- **Node mode is not partially implied**: `--api node` remains phase-gated across `check` / `effects` / `build` / `run` / `test` until its documented subset is implemented; package compatibility must not depend on undocumented fallback behavior.
- **Browser mode stays API-surface-oriented**: the shared **Phase-1 browser-targeted command set** can expose browser ambient typings, but standalone runtime does not pretend to provide DOM APIs; browser-specific behavior comes from bundle/glue output and the real browser host.
- **Browser-targeted Web-baseline overlap is evidenced through the browser host, not through a hidden browser runtime**: the subset of the Web baseline that intersects browser-targeted bundle behavior must survive emitted-bundle smoke tests in a real browser harness, while broader ambient DOM coverage is evidenced through browser-targeted type-checking plus bundle/deploy smoke rather than through Kali-hosted runtime tests.
- **Browser-targeted support is evidenced separately**: Phase 1 browser claims require dedicated coverage for the shared **Phase-1 browser-targeted command set**, including smoke execution of emitted bundles in a real browser harness rather than only DOM mocks/unit shims.

This intentionally keeps the Phase 1 promise small: one dependable Kali-hosted standalone baseline plus one dependable browser-targeted deploy/build lane, without implying a standalone browser runtime.

## Implementation Architecture

Kali uses one guest-facing capability model, but the host adapter depends on the deployment mode.

### Kali-hosted execution / embedding

```
User Code (WASM)
    │
    ├── Direct calls → Host Functions (Rust)
    │                      │
    │                      ├── Sandbox policy check
    │                      ├── Actual I/O (via Rust async/runtime primitives)
    │                      └── Return result to WASM
    │
    └── Pure APIs (in WASM runtime)
        ├── Math operations
        ├── String operations
        ├── Array methods
        ├── JSON parse/stringify
        └── RegExp engine
```

### Browser-targeted bundle output

```
User Code (WASM)
    │
    ├── Direct calls → Browser host adapter (generated JS glue)
    │                      │
    │                      ├── Maps guest ABI calls onto real browser APIs
    │                      ├── Preserves the documented browser-targeted contract
    │                      └── Does not imply Kali-controlled post-deployment sandbox enforcement
    │
    └── Pure APIs (in WASM runtime)
        ├── Math operations
        ├── String operations
        ├── Array methods
        ├── JSON parse/stringify
        └── RegExp engine
```

Boundary clarification:
- this browser host adapter covers guest-ABI bootstrap and the documented **Kali-mediated capability subset** paths
- ordinary DOM/global browser operations still resolve against the real browser ambient environment rather than being re-modeled as one Kali host-adapter entry per browser API

### Pure vs Host APIs
- **Pure**: Implemented in Rust, compiled to WASM, runs inside the guest/runtime artifact (Math, String, Array, JSON, RegExp)
- **Host**: Implemented outside the guest WASM artifact through the selected host adapter — native Rust/wasmtime host functions for Kali-hosted execution, or generated JS glue for browser-targeted bundles for the browser-applicable portion of the documented **Kali-mediated capability subset** (in schema v1: fetch with its `effects.network.maxConnections` cap, the `timer` family, random, console, and later eval only when that separate compatibility path exists)

Clarification:
- host-adapter examples are surface-specific, not one global bag of APIs
- mentioning browser-targeted JS glue here must not be read as implying that browser bundles expose Deno/Node-only capabilities such as `process`

### Built-in Objects (WASM Runtime)
Implemented in `kali_runtime` (compiled to WASM), in phases:

**Phase 1 core runtime**
- `Object` prototype methods
- `Array` and typed arrays needed by common libraries
- `Promise`
- `RegExp` (using a pure-Rust implementation of ECMAScript RegExp semantics; a generic Rust regex crate is only acceptable as an internal building block, not as the semantic contract)
- `Date`
- `JSON`
- `Math`
- `String` prototype methods
- `Number`, `Boolean`, `Symbol`
- `Error` and common subtypes
- `ArrayBuffer`, `DataView`
- Iterators, Generators, AsyncGenerators
- `Map`, `Set`
- `Reflect` subset required by transpiled/bundled code

**Later compatibility phases**
- `SharedArrayBuffer` (later compatibility, only when the separate WASM-threaded runtime profile is implemented and enabled)
- `WeakMap`, `WeakSet` (only once weak-reference semantics are specified well enough to preserve behavior)
- `FinalizationRegistry` (only once weak/finalization semantics can be preserved without undermining the no-tracing-GC design)
- `Proxy`
- fuller `Intl` support

The runtime should prioritize the subset needed by real-world packages and conformance tests before implementing hard edge-case APIs with large semantic cost.

## Global Scope

The global object provides:
- All APIs in the shared Web baseline
- API-surface-specific additions selected by mode (`deno` in early standalone phases; broader `node` later when that surface is implemented)
- `globalThis` reference
- TypeScript-aware — all globals are typed in Kali's standard lib `.d.ts` files
