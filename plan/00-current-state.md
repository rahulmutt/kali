# Current Implementation Baseline

This file records the planning baseline for the active continuation roadmap. It is a current-state summary, not an active checklist and not an availability matrix.

## Evidence used for this cleanup

- `cargo run -q -p kali_cli --bin kali -- --help` exposes the current public command set.
- The workspace contains Rust crates for CLI, lexer/parser, AST/HIR/MIR/LIR, type checking, codegen, runtime, sandbox/effects, package management, optimization, Deno/Web/Node API projections, embedding/C ABI, formatting, linting, and bindings.
- The spec-owned availability and current-state nuance remain in [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- The proof-backed boundary remains owned by [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).

## Live surface at a glance

The checked-in repository already includes:

- CLI commands: `doctor`, `init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`, `effects`, `package-effects`, and `package-audit`.
- Static-literal analysis now also unwraps await-wrapped numeric literals in the bounded inference path, keeping cheap wrapper handling consistent across the checker.
- The benchmark inventory notes now record that the new `math-round-builtin` / `math-round-builtin-js` pair now does the same for `Math.round`, and that the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`).
- Build/artifact lanes: executable builds, browser bundles, IR validation, library artifacts, C ABI artifacts, component artifacts, deterministic metadata, and sidecar manifests.
- Runtime/reporting lanes: `run`, `test`, `test --coverage`, source-graph effects, package effects, package audit, deterministic JSON envelopes, and schema-v1 payload validation.
- Host/API lanes: default standalone, browser-targeted check/build/bundle flows, browser harness execution paths, documented Deno and Node API slices, resource budget validation, and late-host/object-model gating.
- Package lanes: install/lock/materialization, registry and raw-URL workflows, package-shape rejection, package-corpus probes, and single-package registry-analysis commands.
- Verification lanes: Lean proof project and published proof-backed boundary limited by `proofs/BOUNDARY.md`.

## Planning consequence

The active plan should not list already-implemented command families, artifact families, smoke matrices, alias inventories, or diagnostic regressions as open work. Future tasks should be stated as remaining capability goals and should point to specs/tests for details.
