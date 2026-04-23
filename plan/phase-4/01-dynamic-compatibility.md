# Stage 4.1 — Dynamic Compatibility

**Phase:** 4 — Advanced Compatibility & Deep Verification  
**Spec refs:** [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/14-packages.md`](../../specs/14-packages.md), [`specs/18-schemas.md`](../../specs/18-schemas.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** Phase 3 complete (all stages 3.1–3.4)

## Goal

Enable the hardest explicitly gated dynamic features without violating Kali's hard invariants:

- executable `eval` / `Function()` behind the documented compatibility switch
- non-literal dynamic loading against the already-linked graph
- the stable public `kali package-audit` registry-analysis command

This stage does **not** own the later host/object-model breadth tracked in Phase 5.

## Workable Milestone

- `--compat eval` is the one documented gate for executable `eval` / `Function()`.
- non-literal `import(expr)` works only within the already-linked graph and fails clearly when the
  target cannot be resolved there.
- `kali package-audit <package>` is publicly available as the schema-v1 context-free,
  envelope-only registry-analysis command.

## Progress

- `kali package-audit` now has regressions that prove inherited browser/runtime manifest settings do not alter the command's context-free registry-analysis semantics, that explicit `--api browser` stays on the package-analysis-specific flag rejection path, that `--pretty --output json` keeps the envelope deterministic, that `--quiet --output json` still preserves the envelope payload, and that bare `--pretty` remains invalid usage.
- The package-audit JSON envelope now also has an error-finding regression for a native-addon package shape, pinning the `success: false` / `exitCode: 1` path and the structured `errors` array alongside the earlier warning-only case.
- Added an explicit browser-only inherited-context regression for `kali package-audit` so the context-free registry-analysis boundary stays pinned even when `compilerOptions.apiSurface = browser` is selected in the surrounding project manifest.
- The effect-analysis smoke suite now also pins `--compat eval` across both source-graph `effects` output and inherited-manifest `package-effects` output, keeping the dynamic-compatibility boundary aligned with the effect-report surface rather than only with `check` / `run` / `test`.

**Status:** Complete for the documented Phase-4 compatibility milestone.

## Historical stage tasks

### 1. `eval` / `Function()` compatibility path

Implement executable dynamic-code compatibility without introducing a language-level JIT:

- gate the path with the single documented compatibility feature name: `eval`
- keep compilation AOT-only even when guest code evaluates dynamic strings
- ensure static effect analysis, runtime enforcement, and diagnostics all recognize the same gate
- keep ordinary ungated `eval` / `Function()` usage on the canonical availability path

### 2. Non-literal dynamic loading

Open the non-literal dynamic-import compatibility lane while preserving the linked-artifact model:

- runtime lookup resolves only against the graph compiled and linked ahead of time
- missing/unlinked targets fail with the canonical dynamic-import diagnostic path
- no runtime WASM module fetching or second Kali compilation tier is introduced

### 3. `kali package-audit` — stable public release

Open the context-free registry-analysis/security-audit command in the spec-owned shape:

```bash
kali package-audit lodash
kali package-audit --output json lodash
kali package-audit --pretty --output json lodash
```

Important contract details:

- the CLI selector is the canonical **identity-only registry target**, not `pkg@version`
- the command is schema v1's **envelope-only JSON command**
- findings flow through the standard diagnostic arrays
- successful JSON output keeps `payload: null`
- inherited project analysis context does not change the command's semantics

### 4. Evidence and gating discipline

- positive `eval` / `Function()` compatibility tests under the documented gate
- positive and negative non-literal dynamic-import tests
- positive `package-audit` command-shape / JSON / deterministic-version-selection coverage
- negative tests proving later host/object/runtime breadth is still tracked elsewhere

## Out of Scope

- language-level JIT
- tracing/background GC
- weak references, finalization, `Proxy`, and other late object-model work tracked in Phase 5
- standalone browser runtime/test work tracked in Phase 5
- late host-control APIs tracked in Phase 5

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
