# Current Implementation Baseline

This file records the planning baseline for the active continuation roadmap. It is not an availability matrix, implementation journal, or exhaustive evidence log.

## Evidence used for this cleanup

- `cargo run -q -p kali_cli --bin kali -- --help` exposes the live command set.
- The workspace contains compiler, CLI, runtime, package, sandbox/effects, optimization, embedding, schema, API-surface, and proof infrastructure.
- Public availability remains owned by [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- Proof-backed scope remains owned by [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).

## Live surface at a glance

The checked-in repository is past the original Phase 1 MVP. The live CLI exposes:

- `doctor`
- `init`
- `install`
- `fmt`
- `lint`
- `check`
- `build`
- `run`
- `test`
- `effects`
- `package-effects`
- `package-audit`

Implemented areas include:

- schema-v1 JSON envelopes, artifact manifests, diagnostics, and deterministic validation paths;
- source checking, building, running, testing, formatting, and linting for the supported TS/JS source classes;
- browser-targeted `check` / `build --bundle` and harness-backed browser smoke paths;
- public source-graph and registry effect/audit commands;
- package install/materialization, registry/raw-URL flows, lifecycle-hook gating, and package-corpus probes;
- Deno/Web/Node API slices with explicit late-compatibility gates;
- runtime slices for supported object, iterator, BigInt, Math, Promise, console, dynamic-import, and reflection behavior; the literal-array identity `values.map((value) => value)` iterator slice now also has direct `for...of` and `for await...of` regression coverage, and browser-targeted `check` / `build --bundle` smoke now also covers that identity-map slice in JS, TS, JSX, and TSX input; the truthy-literal identity `values.filter((value) => value)` iterator slice now also has direct `for...of` regression coverage, and its resolved array slice now also flows through `Array.from(...)` and spread consumers; the static identity `values.some((value) => value)` / `values.every((value) => value)` slices on literal arrays now also lower to deterministic boolean results in the type checker and codegen smoke lanes; browser-requested run/test browser-harness coverage now also accepts the supported identity `map`/`filter`/`flatMap` literal-array slices, while the remaining callback-bearing methods stay gated with the canonical E5506 path;
- library, WIT, C ABI, component, metadata, and binding-package artifact lanes;
- deterministic PGO input handling and benchmark fixtures; the new `math-round-builtin` / `math-round-builtin-js` pair now does the same for `Math.round`, and the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`);
- Lean proof infrastructure with proof-backed claims limited to the published boundary.

## Planning boundary

Active planning starts from remaining spec gaps, not from historical Phase 1 through Phase 20 checklists. Do not re-add completed packet journals to plan files.

Open work is concentrated in:

- faithful full-language semantics for currently gated constructs;
- host/runtime capability contracts that require sandbox, effects, and resource mediation;
- package compatibility growth by exact support rung and context;
- optimization/performance claims backed by deterministic evidence;
- proof, schema, diagnostic, and CLI contract widening without claim drift.
