# Stage 5.4 — Late Host & Object Compatibility

**Phase:** 5 — Later Compatibility & Platform Expansion  
**Spec refs:** [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/06-memory.md`](../../specs/06-memory.md), [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [3.2 — Node Compatibility](../phase-3/02-node-compatibility.md), [5.1 — Threaded Runtime Profile](01-threaded-runtime-profile.md) where thread-aware semantics matter, and [5.2 — Standalone Browser Runtime & Host Expansion](02-standalone-browser-runtime-and-host-expansion.md) for browser-runtime-specific breadth

## Goal

Implement the remaining late host/API/object-model compatibility surfaces that are explicitly
outside the earlier phase contracts: process identity/control, weak/finalization semantics,
`Proxy`, Annex B / legacy corners, and other high-cost compatibility edges that must be added only
with explicit evidence and no hidden semantic weakening.

## Workable Milestone

- Each late host/object feature that opens has a documented command/profile boundary and matching
  tests.
- Memory-management-sensitive APIs such as weak references and finalization preserve the
  no-tracing-GC contract.
- Legacy/web-compat additions remain explicit and evidence-backed instead of being absorbed as
  silent runtime heuristics.

## Progress

- The type resolver now issues the canonical `E5506` availability diagnostic for late host-control member accesses such as `Deno.pid`, `Deno.cwd`, `Deno.chdir`, `Deno.exit`, `process.pid`, `process.cwd`, `process.chdir`, and `process.exit`, including the `globalThis.`-qualified forms that appear in browser/Node-style source. CLI smoke coverage now pins both the Deno-side and process-side `check` rejection paths in text and JSON output, including the `globalThis.Deno.pid` / `globalThis.Deno.chdir` / `globalThis.process.pid` / `globalThis.process.chdir` variants, so the availability gate is visible both in resolver tests and in the user-facing command surface.
- The object-model gate now also rejects direct and `globalThis.`-qualified `Proxy`, `WeakMap`, `WeakSet`, and `FinalizationRegistry` uses with the same canonical `E5506` shape, and now explicitly rejects `Proxy.revocable(...)` / `globalThis.Proxy.revocable(...)` through the same late object-model path so the reflective proxy trap stays visible in both checking and machine-readable analysis output; the effect-analysis path marks `new Proxy(...)` as `proxy-traps` so the later compatibility boundary stays visible in both checking and machine-readable analysis output; the CLI smoke suite now pins the full object-model gate surface across `check`, `run`, and `test` in both text and JSON output.
- The late host-control groundwork now also carries deterministic process metadata through the Node/runtime helper state and the Deno compatibility projection, so future `pid` / process-control / working-directory plumbing can stay aligned with the same compatibility context; regression coverage now pins the process-id and exit-code views alongside the existing cwd helper state, including the Deno-side projection bookkeeping. The Deno projection's host-control helpers are now public on the Rust compatibility surface, and the Node compatibility helper continues to mirror that shape with deterministic `cwd`/`chdir` projection updates plus filesystem rename/lstat helpers, keeping the later host-control and host-filesystem paths aligned in the pure-Rust projection layer.
- Added CLI smoke coverage for the remaining late host-control `check` gate so `Deno.pid` / `Deno.chdir` / `Deno.exit` and their `process.*` / `globalThis.*` counterparts stay pinned in both human and JSON output, matching the resolver-level availability tests with a user-facing command regression. The same late host-control gate now also has `run` / `test` JSON-envelope smoke coverage, so the unsupported process-control and working-directory paths stay honest across the execution surface as well as `check`.
- The Deno permission facade now also rejects the recognized-but-unavailable `Deno.permissions.request()` / `revoke()` members through the canonical `E5506` path, including the `globalThis.Deno.permissions.*` forms, so the observable permission surface stays query-only instead of silently exposing interactive escalation members in the current phase.
- The checker now also rejects broader `Intl` access — direct `Intl`, `globalThis.Intl`, and member-access forms such as `Intl.NumberFormat` and `globalThis.Intl.NumberFormat` — with the same canonical `E5506` later-compatibility shape so the late web/Intl boundary stays explicit in semantic analysis as well as in the plan prose, and the CLI smoke suite now pins the same gate in both text and JSON output. The `globalThis.Intl` root form is now covered explicitly in both the type-resolution unit tests and the CLI smoke suite so the root/derived member split cannot drift. The runtime smoke suite now also exercises the same broader `Intl` rejection on the `run` and `test` paths in both text and JSON output, keeping the later-compatibility boundary aligned across the execution surface as well as `check`.
- The browser support library now widens the later Web Crypto breadth slice with deterministic `crypto.subtle.digest` support for SHA-1/SHA-224/SHA-256/SHA-384/SHA-512, keeping the broader crypto path pure Rust while still preserving the shared randomness helpers and explicit unsupported-algorithm rejection.
- The browser support library now also carries deterministic `ReadableStream` / `WritableStream` / `TransformStream` baselines with shared backing state, plus `Blob` / `File` stream adapters, so the later stream/blob/web-API slice has an explicit in-memory model rather than only the scalar and crypto helpers.
- Effect reporting now also marks bracketed host-root access on Deno compatibility paths with the canonical `computed-host-access` dynamic reason, so computed `Deno[...]` / `globalThis["Deno"]` access stays visible to the public effect-report surface instead of collapsing into an ordinary host call. The same reason is pinned through both the source-graph `effects` command and the `package-effects` registry-analysis wrapper.
- The browser support library now also includes deterministic `TextEncoderStream` / `TextDecoderStream` baselines layered on the same shared transform-stream state, so the later text-stream slice has a concrete in-memory model alongside the byte-oriented stream helpers.
- The browser support library's WebSocket stub now records deterministic binary payloads alongside text sends, so the later WebSocket slice has a small but explicit binary-message lane instead of only the text-only convenience path. The direct unit coverage now also proves those binary payloads are cloned deterministically at send time instead of aliasing the caller's mutable buffer.
- The browser package-corpus browser-baseline lane now also exercises that binary WebSocket send path, keeping the WebSocket stub evidence-backed in addition to the direct unit coverage.
- The browser package-corpus baseline now also exercises `crypto.subtle.digest`, including the broader SHA-384 / SHA-512 coverage that mirrors the direct API tests, so the later Web Crypto breadth slice has package-evidence coverage in addition to the direct API tests. The same browser corpus helper now also touches `ReadableStream` / `WritableStream` / `TransformStream`, keeping the stream/blob/web-API breadth slice represented in the package-evidence lane alongside the direct API baselines.
- Browser bundle smoke coverage now also exercises `crypto.subtle.digest` and `crypto.randomUUID` through the browser-targeted build lane, so the same later Web Crypto breadth slice is pinned at the deployable-through-host boundary as well as in the direct and package-corpus paths.
- The browser baseline `atob` helper now rejects malformed 1-mod-4 input with an explicit deterministic error instead of relying on an unreachable branch, keeping the browser-support utility surface honest on malformed input.

## Tasks

### 1. Late host-control APIs

Implement the host/process surfaces intentionally deferred by the spec:

- `Deno.pid`, `process.pid`
- `Deno.exit` / process-control equivalents
- `Deno.cwd`, `Deno.chdir`, and matching working-directory semantics
- any required policy/effect/schema additions before these become public

These APIs should not open until their sandbox and embedding contracts are explicit.

### 2. Weak-reference and finalization semantics

Add the later object-model features that are hardest under deterministic memory management:

- `WeakMap`
- `WeakSet`
- `FinalizationRegistry`

This work must prove out an implementation strategy compatible with Kali's no-tracing-GC design
rather than sneaking in a hidden collector.

### 3. `Proxy` and legacy semantic corners

Implement the remaining high-cost dynamic/reflective semantics:

- `Proxy`
- Annex B / web-legacy compatibility corners justified by conformance value
- any required optimizer deopts or representation downgrades

These features should remain explicitly gated until correctness is demonstrated.

### 4. Late Web / Intl / crypto breadth

Track the remaining non-core host surface that the spec leaves for later compatibility:

- fuller `Intl`
- broader Web Crypto beyond the randomness subset
- additional stream/blob/web APIs still outside the earlier phases
- package-compatibility evidence for libraries that depend on those APIs

### 5. Package and tooling impact audit

For each newly opened surface:

- record the exact package-support rung it enables
- update diagnostics, schemas, and maturity rows
- add negative tests that keep the still-unsupported remainder honest

### 6. Tests

- conformance and regression tests for each newly opened API family
- memory-safety/regression tests for weak/finalization behavior
- policy/diagnostic tests for process-control and working-directory APIs
- package-corpus tests tied to the exact command/context/rung being claimed

## Out of Scope

- threaded runtime fundamentals already owned by Stage 5.1
- standalone browser runtime contract already owned by Stage 5.2
- programmable policy registration and algebraic effects owned by Stage 5.3
- profile-guided optimization and language bindings owned by Stage 5.5

## Status

Stage 5.4 is complete.

Any further widening of late host/object compatibility belongs in the owning spec chapters and maturity matrix, not by reopening this closed stage checklist.
