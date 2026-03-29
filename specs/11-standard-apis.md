# 11 — Standard APIs

## Strategy

Implement APIs as host functions provided by the runtime. Each API surface is a separate crate that registers its host functions with the WASM runtime.

Compatibility is delivered in layers:
1. **Baseline**: Web platform primitives needed by modern JS libraries.
2. **Primary host**: Deno-style APIs, since they align well with explicit permissions and sandboxing.
3. **Compatibility layers**: Node.js shims and browser-facing glue, added incrementally and tested against real packages.

The spec goal is broad compatibility, but the implementation should prefer a smaller, dependable surface over a shallow imitation of every host API.

A key simplification rule applies throughout this section: Phase 1 should target the smallest API set that unlocks real-world package execution, and every later API addition should be justified by package compatibility or standards pressure.

For dynamic or semantically expensive APIs (for example `Proxy`, weak references, and threaded primitives), the canonical phase/status lives in [specs/19-feature-maturity.md](19-feature-maturity.md). This section should describe API layering, not restate a conflicting maturity decision.

## API-Surface Loading Rule

For the compact cross-spec summary of early host/API behavior, see the canonical host capability table in [SPEC.md](../SPEC.md).

To keep runtime imports, globals, and package expectations aligned:
- the **Web Platform baseline** is the shared baseline across supported surfaces
- `--api deno`, `--api node`, and `--api browser` control which **additional** globals/modules beyond that baseline are available
- browser-targeted profiles must not expose process/env/file globals just because the underlying host runtime happens to have them
- unsupported globals/modules are absent; Kali must not invent dummy shims by default
- use the canonical `E5006` availability path for **documented command/profile or feature gating** (for example `--api node` before Phase 3, or `run --api browser` in early phases)
- use ordinary unresolved-name/type diagnostics when code references a global that simply is not part of the selected ambient surface in an otherwise-supported mode (for example `document` under `--api deno`)

This prevents a common source of drift: host-runtime implementation convenience must not silently widen the language-visible API contract.

Canonical terminology simplification:
- **browser-targeted profile** means exactly the early supported browser paths: `kali check --api browser` and `kali build --bundle --api browser`
- it does **not** mean a standalone embedded browser runtime, DOM emulation layer, or permission to expose non-browser globals during analysis/build

## API Layers

### Web Platform APIs (Baseline)
Available across supported execution surfaces as the shared baseline.

Interpretation rule:
- in standalone execution (`run` / `test`), this baseline is present for supported runtime profiles
- in browser-targeted output (`build --bundle --api browser`), this is the baseline the emitted code targets in the real browser host
- command/profile combinations that are themselves phase-gated are still rejected according to [specs/19-feature-maturity.md](19-feature-maturity.md)

**Phase 1 MVP baseline**
- `console` (`log`, `warn`, `error`, `debug`, `info`)
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
- `Deno.permissions` as a read-only compatibility facade over Kali sandbox policy state; it reports granted/denied capabilities but does not perform interactive permission prompts, `request()`, or `revoke()`-style privilege escalation flows in Phase 1 (the canonical maturity decision for this facade lives in [specs/19-feature-maturity.md](19-feature-maturity.md))

Implementation simplification:
- this read-only `Deno.permissions` facade should normally be derived from Kali's already-resolved runtime/policy state rather than from a separate permission-prompt host API
- that keeps the Deno compatibility story aligned with the sandbox-first model: permission status is observed, not negotiated interactively at runtime

For host-capability maturity, the canonical source of truth is [specs/19-feature-maturity.md](19-feature-maturity.md). In particular:
- read-only environment access is part of the Phase 1 standalone contract
- mutable environment access, subprocess spawning, and socket/listener networking follow the Phase 3 maturity path
- process identity, termination, and working-directory APIs remain a later-compatibility path until a future schema/policy revision gives them an auditable contract

Process identity (`Deno.pid`), process termination (`Deno.exit`), and working-directory mutation/introspection (`Deno.cwd`, `Deno.chdir`) are therefore intentionally outside the Phase 1 MVP. They widen the embedding/sandbox contract but are not needed for the initial package-oriented baseline.

Rule of thumb: when Kali exposes a Deno file/metadata API in Phase 1, it should expose the sync and async forms together unless there is a strong implementation reason not to. This avoids needless package-compatibility drift between `readFile` and `readFileSync`-style code paths.

**Phase 3 target expansion**
- `Deno.open`, `Deno.create`, `Deno.mkdir`, `Deno.remove`, `Deno.rename`, `Deno.lstat`
- `Deno.env.set`
- `Deno.Command` (process spawning)
- `Deno.serve` (HTTP server / listen path)
- broader filesystem, networking, and subprocess coverage

**Later compatibility expansion**
- `Deno.pid`
- `Deno.cwd`, `Deno.chdir`
- `Deno.exit`

Cross-spec consistency note:
- subprocess, mutable-environment, and network/listener APIs fit schema-v1's policy vocabulary
- process identity, termination, and working-directory APIs do **not** yet have dedicated schema-v1 policy/effect keys
- therefore `Deno.pid`, `Deno.exit`, `Deno.cwd`, and `Deno.chdir` remain later-compatibility features until a future schema/policy revision makes their sandbox contract explicit

This keeps the Phase 1 host surface small and auditable while still establishing Deno as the default API model.

### Node.js API (`--api node`)
Node compatibility is a **Phase 3 ecosystem target**, not a Phase 1 promise. The goal is package compatibility first, not full Node parity.

Canonical gating rule:
- `kali check --api node`, `kali effects --api node`, `kali build --api node`, `kali run --api node`, and `kali test --api node` are all phase-gated until the documented Node subset exists
- early phases must reject these modes with the canonical `E5006` diagnostic instead of exposing a partial ambient `process`/built-ins surface

**Phase 3 target subset**
- `fs`, `fs/promises` — file system operations
- `path` — path manipulation (pure, no host calls needed)
- `buffer` — Buffer class
- `events` — EventEmitter
- `util` — utilities (promisify, inspect, etc.)
- `url` — URL parsing
- `assert` — assertions
- `process` — process global subset needed by real packages first (`env`, `argv`, selected control/query helpers); `pid`, `exit`, and `cwd`-style process-introspection/control APIs stay on the later-compatibility path until the policy and embedding contract for them is specified

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
Browser mode is primarily a **build/check profile** in early phases, not a promise that the standalone runtime behaves like a browser. See the canonical host/profile summary in [SPEC.md](../SPEC.md) and the phase-gating matrix in [19 — Feature Maturity](19-feature-maturity.md) when deciding whether a given command/profile combination is supported.

Two layers matter here and should not be conflated:
- **browser ambient typing surface** — the globals/types visible to `check` and browser-targeted builds (for example `Window`, `Document`, `HTMLElement`, `fetch`, `URL`)
- **standalone runtime host surface** — the APIs provided by Kali's own runtime when it executes code directly

Canonical rule:
- browser-targeted `check` and `build --bundle --api browser` should type-check against the real browser ambient surface, including DOM typings that are normally present in browser-focused TypeScript programs
- this does **not** mean Kali's standalone runtime implements or emulates those DOM APIs
- when Kali emits browser-targeted artifacts, DOM/Web APIs are expected to come from the real browser host at deployment time
- `--sandbox` on a browser-targeted build therefore constrains static analysis/build-time compatibility, not automatic post-deployment browser-permission enforcement by Kali itself
- no Deno or Node globals are exposed in browser mode unless a later compatibility spec explicitly says so
- any lightweight DOM test shim is a separate testing utility, not part of the core browser compatibility contract

This resolves a common ambiguity: browser-targeted analysis may know about `document`/`window`, while standalone execution still rejects browser-runtime assumptions because Kali does not embed a browser engine.

**Canonical early-phase rule**:
- `kali check --api browser ...` is allowed for browser-targeted analysis
- `kali build --bundle --api browser ...` is allowed for browser-targeted artifacts
- `kali build --api browser ...` without `--bundle` is rejected by default in early phases to keep browser mode tied to a real browser-host deployment path
- `kali run --api browser ...` is rejected by default until a later runtime profile explicitly supports it
- `kali test --api browser ...` is also rejected by default in early phases for the same reason; browser support is not yet a standalone execution/test-runtime contract

**Note**: For supported command/profile combinations, the Phase 1 baseline Web Platform APIs are available regardless of `--api` mode. The `--api` flag controls which *additional* platform-specific APIs are loaded, and unsupported command/surface combinations in early phases should produce the canonical feature-maturity diagnostic described in [specs/15-errors.md](15-errors.md) rather than silently falling back.

## Phase 1 Host API Exit Criteria

This section turns the broad API story into a small implementation checklist so runtime, CLI, and testing do not drift:

- **Web baseline must work end-to-end**: `console`, timers, `queueMicrotask`, `fetch`, `URL`, `TextEncoder`/`TextDecoder`, `AbortController`, `structuredClone`, `performance.now()`, the MVP randomness subset (`crypto.getRandomValues`), and event primitives are available in `run` and covered by integration tests.
- **Deno baseline must work end-to-end**: file read/write, metadata/read-dir, invocation arguments, and read-only env access all execute through the host ABI and obey the documented sandbox/execution contract.
- **Every Phase 1 host call is policy-aware**: the runtime may not expose an unchecked host backdoor just because the API itself is part of the MVP.
- **Node mode is not partially implied**: `--api node` remains phase-gated across `check` / `effects` / `build` / `run` / `test` until its documented subset is implemented; package compatibility must not depend on undocumented fallback behavior.
- **Browser mode stays profile-oriented**: browser-targeted analysis/build can expose browser ambient typings, but standalone runtime does not pretend to provide DOM APIs; browser-specific behavior comes from bundle/glue output and the real browser host.

This intentionally keeps the Phase 1 promise small: one dependable Web baseline plus one dependable Deno baseline.

## Implementation Architecture

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

### Pure vs Host APIs
- **Pure**: Implemented in Rust, compiled to WASM, runs inside the sandbox (Math, String, Array, JSON, RegExp)
- **Host**: Implemented as wasmtime host functions, run outside WASM (I/O, network, process, crypto)

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
- All Web Platform APIs in the shared baseline
- API-surface-specific additions selected by mode (`deno` in early standalone phases; broader `node` later when that surface is implemented)
- `globalThis` reference
- TypeScript-aware — all globals are typed in Kali's standard lib `.d.ts` files
