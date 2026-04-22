# Repository Layout Adoption Guide

This document expands the directory-structure sketch from [`../PLAN.md`](../PLAN.md) into an adoption plan.

The goal is not to force one exact tree immediately. The goal is to give each major subsystem a clear long-lived home so implementation stages do not fight the repository structure.

## Target top-level structure

```text
.
├── Cargo.toml
├── mise.toml
├── devenv.nix
├── devenv.yaml
├── SPEC.md
├── PLAN.md
├── README.md
├── specs/
├── plan/
├── proofs/
├── crates/
│   ├── kali/         # user-facing CLI binary
│   ├── cli/          # parsing, config discovery, command shaping, help text
│   ├── core/         # lexer, parser, AST, typing, lowering, codegen front-half
│   ├── runtime/      # wasmtime execution, host adapters, async runtime, enforcement hooks
│   ├── packages/     # install, lock, registry access, cache layout, package resolution
│   ├── sandbox/      # policy schemas, validation, effect/policy comparison helpers
│   ├── embed/        # stable embedding APIs, WIT/C ABI/component packaging
│   └── optimize/     # specialization, optimization pipelines, PGO plumbing
├── tests/
│   ├── integration/
│   ├── conformance/
│   ├── package-corpus/
│   ├── browser-smoke/
│   ├── determinism/
│   └── cli-snapshots/
├── fixtures/
│   ├── compiler/
│   ├── runtime/
│   ├── packages/
│   ├── browser/
│   └── cli/
└── tools/
    └── scripts/      # repeatable dev/CI helpers when a mise task needs helper scripts
```

## Ownership by repository area

| Area | Owns | Opens in plan |
|---|---|---|
| `specs/` | normative subsystem contracts | already present before implementation stages start |
| `plan/` | implementation sequencing and completion gates | already present before implementation stages start |
| `proofs/` | proof boundary, Lean project, proof artifacts | Stage 1.1 baseline; expands in 2.4 and 4.2 |
| `crates/kali` | binary entrypoint, command dispatch | Stage 1.1 |
| `crates/cli` | CLI parsing, output envelopes, config discovery | Stage 1.1; deepens in 1.12 and 1.13 |
| `crates/core` | frontend, checker, lowering, codegen | Stages 1.2-1.7, then 2.1 and 3.1 |
| `crates/runtime` | execution engine, host adapters, runtime scheduling | Stage 1.8; broadens in 3.2, 3.4, 4.1, 5.x |
| `crates/packages` | install/lock/materialization/resolution | Stage 1.10; broadens in 3.3 and 4.1 |
| `crates/sandbox` | policy schema, validation, enforcement helpers | Stage 1.9; broadens in 2.2 and 5.3 |
| `crates/embed` | public embedding surface and host-facing metadata | Stage 1.11 base artifact, Phase 2 public surface |
| `crates/optimize` | optional optimization and PGO-specific plumbing | minimal in 3.1, expands in 5.5 |
| `tests/` | evidence tracks that justify support claims | begins in 1.1, becomes critical in 1.14 |
| `fixtures/` | reviewable input corpora shared by tests | begins in 1.2, grows every phase |
| `tools/scripts/` | helper automation invoked by `mise` or CI | as needed; should not replace the canonical `mise` entrypoints |

## Adoption by phase

### Phase 1
Create only the areas needed for the MVP critical path and evidence:
- `crates/kali`
- `crates/cli`
- `crates/core`
- `crates/runtime`
- `crates/packages`
- `crates/sandbox`
- `tests/integration`
- `tests/conformance`
- `tests/browser-smoke`
- `tests/determinism`
- `fixtures/compiler`
- `fixtures/runtime`
- `fixtures/packages`

### Phase 2
Add or promote the areas needed for semantic depth and public surfaces:
- `crates/embed`
- expanded `proofs/` tree
- coverage-reporting fixtures/tests
- stable effect-report fixtures and schema checks

### Phase 3
Deepen performance and compatibility areas:
- `crates/optimize`
- larger package corpus
- Node-specific and host-expansion fixtures
- benchmark harness support through `tests/` and `tools/scripts/`

### Phase 4
Add only the explicitly gated late-compatibility and proof-depth support files required by that phase.
Do not scatter Phase-5 deferred work into Phase-4 directories just because the area exists.

### Phase 5
Extend runtime, browser, policy, and binding areas only when the earlier public surfaces are already stable enough to support them.

## Layout rules

### Keep docs and implementation separate
Do not mix normative spec prose into crate-local README files as a substitute for updating `specs/` or `plan/`.

### Prefer subsystem ownership over feature scattering
If a change is mostly package-related, it should primarily land under `crates/packages` even if it touches CLI wiring.

### Tests should mirror evidence lanes
Use top-level test directories that align with the evidence matrix in [`../specs/16-testing.md`](../specs/16-testing.md), so a maturity claim can point to a concrete lane.

### Fixtures should stay reviewable
Prefer small, purpose-specific fixtures over giant sample projects with many moving parts.

### Script helpers are secondary
If a helper lives in `tools/scripts/`, it should still be invoked through a canonical `mise` task for day-to-day use and CI.

## Suggested growth sequence inside `crates/`

```text
stage 1.1  -> kali, cli
stages 1.2-1.7 -> core
stage 1.8  -> runtime
stage 1.9  -> sandbox
stage 1.10 -> packages
stage 1.11 -> embed (base artifact plumbing may start here, even before the stable public API)
stage 3.1  -> optimize
phase 2+   -> embed becomes public/stable
```

This sequencing keeps the repo simple early, while still reserving stable homes for later work.

## Review checklist for layout changes

Before adding a new top-level directory or crate, ask:
1. Is this a real long-lived ownership boundary?
2. Could the work fit an existing subsystem more cleanly?
3. Does the new area align with a spec owner and a plan stage?
4. Will tests/fixtures for this change land in the evidence lane that justifies the claim?

If the answer to any of these is no, keep the structure simpler.
