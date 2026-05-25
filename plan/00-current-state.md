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
- runtime object, iterator, BigInt, Math, Promise combinators (all / allSettled / race / any), console, and dynamic-import slices where evidence exists, including single-quoted `Proxy.revocable` browser-late-compat alias coverage, browser bundle console routing/assertion smoke in `.js`, `.ts`, `.jsx`, and `.tsx` input, the frozen bracketed `Object.is` callable alias on the primitive-literal slice, browser-requested `for await` helper smoke now also covering mixed/bracketed frozen callable aliases for `Object.keys` / `Object.values` / `Object.entries`, object-enumeration helper resolution now also covering parenthesized receiver-wrapped bracketed aliases for `Object.keys` / `Object.values` / `Object.entries` in js-like input, and browser-harness smoke now also exercising the parenthesized receiver-wrapped bracketed helper variants on that same slice, including `Object.freeze((globalThis["Object"]).keys)` / `Object.freeze((globalThis["Object"]).values)` / `Object.freeze((globalThis["Object"]).entries)` receiver-wrapped callable forms, with the direct resolution regression now also pinning the parenthesized receiver-wrapped bracket-root `Object.freeze((globalThis["Object"]).values)` / `Object.freeze((globalThis["Object"]).entries)` aliases in js-like input, plus the single-quoted parenthesized `Object.freeze((globalThis['Object'])["values"])` alias on the values slice, and the single-quoted parenthesized `Object.freeze((globalThis['Object'])["entries"])` alias on the entries slice, the logical-or `Object.entries` wrapper slice alongside the existing logical-and/nullish forms, the parenthesized receiver-wrapped bracketed `Object.entries` helper variant on that same browser-harness path, plus the parenthesized single-quoted bracketed `Object.entries` helper variant on that same browser-harness/browser-bundle matrix, the frozen `Object.entries` helper-call slice on the supported object-enumeration path, browser-bundle smoke now also exercises both the parenthesized receiver-wrapped bracketed and the parenthesized bracket-root `Object.entries` helper variants on that same slice, and the single-quoted parenthesized `Object.freeze((globalThis['Object'])["entries"])` alias on the same browser-runtime spread fixture, and the frozen root-object `Object.freeze((globalThis["Reflect"]))["ownKeys"]` alias on the supported `Reflect.ownKeys` slice, with the browser-harness/browser-bundle runtime-smoke matrix now also pinning that alias, and browser-requested dynamic-import smoke now also covering `Object.freeze`-wrapped literal imports with logical wrappers in `.js`, `.ts`, `.jsx`, and `.tsx` input, plus the parenthesized receiver-wrapped bracketed `Object.keys` / `Object.values` helper variants on the browser-harness/browser-bundle matrix, and the dotted parenthesized receiver-wrapped bracket-root `Object.freeze((globalThis["Object"]).keys)` / `Object.freeze((globalThis["Object"]).values)` forms on that same matrix, and `Array.from` over frozen `globalThis["Set"]` / `globalThis['Set']` / `globalThis['Map']` / `globalThis["Map"]` constructor results in standalone and browser harness/bundle smoke;
- iterator smoke now also covers frozen `Object.entries` helper calls on the supported object-enumeration slices;
- Promise.any browser smoke now also covers mixed-bracket and transparent wrapper aliases on the shared browser body;
- library, WIT, C ABI, component, metadata, and binding-package artifact lanes;
- deterministic PGO input handling and benchmark fixtures, including the new `math-round-builtin` / `math-round-builtin-js` pair now does the same for `Math.round`, and the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`), and browser-targeted build smoke now also covers the `Math.hypot(3, 4)` perfect-square-sum slice in JSX and TSX input, with the `Math.hypot` frozen-callable smoke now also widening to the parenthesized receiver-wrapped bracket-root spellings around `globalThis["Math"]` and `globalThis['Math']`;
- Lean proof infrastructure with proof-backed claims limited to the published boundary.

## Planning boundary

Active planning starts from remaining spec gaps, not from historical Phase 1 through Phase 20 checklists. Do not re-add completed packet journals to plan files.

Open work is concentrated in:

- faithful full-language semantics for currently gated constructs;
- host/runtime capability contracts that require sandbox, effects, and resource mediation;
- package compatibility growth by exact support rung and context;
- optimization/performance claims backed by deterministic evidence;
- proof, schema, diagnostic, and CLI contract widening without claim drift.
