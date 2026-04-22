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

- The type resolver now issues the canonical `E5006` availability diagnostic for late host-control member accesses such as `Deno.pid`, `Deno.cwd`, `Deno.chdir`, `Deno.exit`, `process.pid`, `process.cwd`, `process.chdir`, and `process.exit`, including the `globalThis.`-qualified forms that appear in browser/Node-style source. CLI smoke coverage now pins the Deno-side `check` rejection path so the availability gate is visible both in resolver tests and in the user-facing command surface.
- The object-model gate now also rejects direct and `globalThis.`-qualified `Proxy`, `WeakMap`, `WeakSet`, and `FinalizationRegistry` uses with the same canonical `E5006` shape, and the effect-analysis path marks `new Proxy(...)` as `proxy-traps` so the later compatibility boundary stays visible in both checking and machine-readable analysis output; the CLI smoke suite now pins the full object-model gate surface in both text and JSON output.
- The checker now also rejects broader `Intl` access — direct `Intl`, `globalThis.Intl`, and member-access forms such as `Intl.NumberFormat` and `globalThis.Intl.NumberFormat` — with the same canonical `E5006` later-compatibility shape so the late web/Intl boundary stays explicit in semantic analysis as well as in the plan prose, and the CLI smoke suite now pins the same gate in both text and JSON output.
- The browser support library now widens the later Web Crypto breadth slice with deterministic `crypto.subtle.digest` support for SHA-1/SHA-256/SHA-384/SHA-512, keeping the broader crypto path pure Rust while still preserving the shared randomness helpers and explicit unsupported-algorithm rejection.
- The browser package-corpus baseline now also exercises `crypto.subtle.digest`, so the later Web Crypto breadth slice has package-evidence coverage in addition to the direct API tests.

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

In progress.
