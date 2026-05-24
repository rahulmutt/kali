# Current Implementation Baseline

This file records the high-level planning baseline for the active continuation roadmap. It is not an availability matrix, implementation journal, or exhaustive evidence log.

## Evidence used for this cleanup

- `cargo run -q -p kali_cli --bin kali -- --help` exposes the live command set.
- The workspace contains compiler, CLI, runtime, package, sandbox/effects, optimization, embedding, schema, and proof infrastructure.
- Public availability remains owned by [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- Proof-backed scope remains owned by [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).

## Live surface at a glance

The checked-in repository is past the original Phase 1 MVP and includes broad later-surface implementation. The live CLI exposes:

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

- schema-v1 JSON envelopes and deterministic payload validation;
- source-file checking, building, running, and testing for TS/JS input classes;
- browser-targeted `check` / `build --bundle` plus browser-harness smoke paths;
- public source-graph and registry effect/audit commands;
- package install/materialization, registry/raw-URL flows, install JSON removal reporting for pruned registry/raw-URL entries, and package-corpus probes;
- Deno/Web/Node API slices with explicit late-compatibility gates;
- runtime object, iterator, BigInt, Math, Promise, console, and dynamic-import slices where evidence exists, including single-quoted `Proxy.revocable` browser-late-compat alias coverage, browser bundle console routing/assertion smoke in `.js`, `.ts`, `.jsx`, and `.tsx` input, the frozen bracketed `Object.is` callable alias on the primitive-literal slice, browser-requested `for await` helper smoke now also covering mixed/bracketed frozen callable aliases for `Object.keys` / `Object.values` / `Object.entries`, object-enumeration helper resolution now also covering parenthesized receiver-wrapped bracketed aliases for `Object.keys` / `Object.values` / `Object.entries` in js-like input, including `Object.freeze((globalThis["Object"]).keys)` / `Object.freeze((globalThis["Object"]).values)` / `Object.freeze((globalThis["Object"]).entries)` receiver-wrapped callable forms, plus the single-quoted parenthesized `Object.freeze((globalThis['Object'])["values"])` alias on the values slice, and browser-requested dynamic-import smoke now also covering `Object.freeze`-wrapped literal imports with logical wrappers in `.js`, `.ts`, `.jsx`, and `.tsx` input;
- library, WIT, C ABI, component, metadata, and binding-package artifact lanes;
- deterministic PGO input handling and benchmark fixtures, including the new `math-round-builtin` / `math-round-builtin-js` pair now does the same for `Math.round`, and the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`);
- Lean proof infrastructure with proof-backed claims limited to the published boundary.

## Planning boundary

Active planning starts from remaining spec gaps, not from historical Phase 1 through Phase 20 checklists. Do not re-add completed packet journals to plan files.

Open work is concentrated in:

- faithful full-language semantics for currently gated constructs;
- host/runtime capability contracts that require sandbox, effects, and resource mediation;
- package compatibility growth by exact support rung and context;
- optimization/performance claims backed by deterministic evidence;
- proof, schema, diagnostic, and CLI contract widening without claim drift.
