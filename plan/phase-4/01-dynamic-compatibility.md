# Stage 4.1 — Dynamic Compatibility

**Phase:** 4 — Advanced Compatibility & Deep Verification  
**Spec refs:** [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** Phase 3 complete (all stages 3.1–3.3)

## Goal

Enable the hardest dynamic JavaScript features that were explicitly gated in earlier phases:
executable `eval` / `Function()`, non-literal dynamic `import()`, and deeper API coverage.
All of these must be enabled only via the documented compatibility gate and must not weaken
Kali's hard invariants (AOT-only, no language-level JIT, no tracing GC).

## Workable Milestone

- `eval("...")` and `new Function(...)` execute correctly when `compat.features.eval = true`
  is set in `kali.json`.
- Non-literal `import(expr)` resolves against the already-linked module graph at runtime.
- `kali package-audit` opens as a stable public command.

## Progress

- `kali package-audit` is now publicly available without the old preview shim and performs
  deterministic registry-metadata findings over the selected package version, including the
  documented `KALI_REGISTRY` override path for npm metadata fetches.
- Browser-bundle chunk discovery now follows simple statically resolvable dynamic-import targets,
  including concatenated, parenthesized, and const-bound static fragments, and the emitted bundle
  now carries a runtime `loadDynamicImport(specifier)` helper with path-normalized lookup.
- Unknown dynamic-import targets now take the dedicated `E4008` path instead of blending into
  generic import-resolution failures.
- The shared compatibility-feature plumbing now accepts `--compat eval` and inherited
  `compat.features = ["eval"]`, rewrites simple statically resolvable `eval(...)` strings before
  compilation, and applies the same gate to simple `Function()` constructor bodies.
- `eval` / `Function()` usage without the compatibility gate continues to fail with the canonical
  diagnostic path.

## Tasks

### 1. `eval` / `Function()` compatibility path

The executable `eval` path must preserve Kali's no-language-level-JIT invariant. The chosen
approach: at build time, when `compat.features.eval` is enabled, include an interpreter-backed
fallback for `eval`-evaluated code strings. The interpreter runs inside the same WASM module
(not a second JIT tier) and is subject to all sandbox and effect constraints.

Implementation options (choose one, document the decision):
- **WASM interpreter in WASM**: embed a minimal JS interpreter compiled to WASM. This preserves
  the AOT-only outer boundary.
- **Pre-compiled eval stubs**: analyse the source for `eval(literal)` patterns at compile time
  and AOT-compile the literal strings. Non-literal `eval` calls that cannot be resolved
  statically trap with a clear error unless the interpreter path is also enabled.

The `eval` compatibility switch (`compat.features.eval`) is the **sole gate**. Programs that
use `eval` without this switch continue to fail with a clear diagnostic.

### 2. Non-literal `import()` compatibility path

Phase 1 rejected non-literal dynamic `import()`. Phase 4 allows it within the already-linked
module graph:

- All modules in the project are still compiled and linked at build time (AOT guarantee preserved).
- At runtime, `import(expr)` resolves against the linked graph's module table.
- If `expr` resolves to a module that was not linked (i.e. a specifier not known at build time),
  trap with `E4008` (dynamic import target not in linked graph).

This preserves the single-linked-WASM-payload rule: no new modules are loaded at runtime.

### 3. `kali package-audit` — stable public release

Open `kali package-audit <pkg>` as a stable public command (from its **Later compatibility**
status):

```
kali package-audit <pkg>@<version>
kali package-audit --output json <pkg>@<version>
```

This is the **envelope-only JSON command** in schema v1 — unlike `kali effects` and
`kali package-effects` (native-JSON commands), `kali package-audit` always wraps its output in
the standard JSON envelope even when invoked without `--output json`.

Package audit covers: known vulnerability advisories (from the npm advisory database),
license compatibility, dependency count, and known malicious package signals.

### 4. Deeper API coverage

- Full `Intl` object (internationalisation APIs).
- `WeakRef` and `FinalizationRegistry` (compatible with reference-counting memory model).
- `SharedArrayBuffer` and `Atomics` (gated by the threaded runtime profile flag; not default).
- Broader `fetch` API (`Request`, `Response` body streaming).
- `WebAssembly` global object (for programs that embed their own WASM).

### 5. Tests

- `eval("1 + 2")` with `compat.features.eval = true` → returns `3`.
- `eval(dynamicString)` with `compat.features.eval = true` → executes correctly for dynamic
  strings derived from program state.
- `eval("...")` without the compat switch → diagnostic error.
- Non-literal `import(expr)` resolves to a linked module → works.
- Non-literal `import(expr)` with unknown specifier → `E4008`.
- `kali package-audit lodash@4.17.21` → JSON audit report with zero vulnerabilities.
- All Phase-1, Phase-2, and Phase-3 tests continue to pass.

## Out of Scope

- Language-level JIT (hard invariant; never allowed).
- Tracing GC (hard invariant; never allowed).
- Full POSIX process model (`fork`, `exec` beyond the sandboxed `child_process` stub).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
