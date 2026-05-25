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
- browser-targeted `check` / `build --bundle` plus browser-harness smoke paths, with browser-harness `check` coverage now also rejecting anonymous default-export generator and async-generator declarations across JS, TS, JSX, and TSX input;
- generator and async-generator class-expression rejection now also covers sequence-wrapped forms on the supported build/check/runtime smoke paths;
- public source-graph and registry effect/audit commands, with package-effects and package-audit JSON smoke now also asserting canonical CLI flag context for package-analysis-specific flag rejections and the shared flag-precedence rejection path, plus normalized requested/effective package-argument context for padded single-target JSON rejections;
- package install/materialization, registry/raw-URL flows, install JSON removal reporting for pruned registry/raw-URL entries, and package-corpus probes, including browser-harness `test` rejection coverage for published bin entrypoints when a harness command is configured;
- direct mixed-quote `Array.from` helper aliases now also flow through the supported array-iteration smoke path;
- Deno/Web/Node API slices with explicit late-compatibility gates, including direct and bracketed `globalThis.Deno.Command` / `globalThis.Deno.connect` / `globalThis.Deno.listen` / `globalThis.Deno.serve` alias coverage in the phase-three host-API rejection matrix;
- runtime object, iterator, BigInt, Math, Promise combinators (all / allSettled / race / any), console, and dynamic-import slices where evidence exists, including single-quoted `Proxy.revocable` browser-late-compat alias coverage, browser bundle console routing/assertion smoke in `.js`, `.ts`, `.jsx`, and `.tsx` input, the frozen bracketed `Object.is` callable alias on the primitive-literal slice, browser-requested `for await` helper smoke now also covering mixed/bracketed frozen callable aliases for `Object.keys` / `Object.values` / `Object.entries`, object-enumeration helper resolution now also covering parenthesized receiver-wrapped bracketed aliases for `Object.keys` / `Object.values` / `Object.entries` in js-like input, and browser-harness smoke now also exercising the parenthesized receiver-wrapped bracketed helper variants on that same slice, including `Object.freeze((globalThis["Object"]).keys)` / `Object.freeze((globalThis["Object"]).values)` / `Object.freeze((globalThis["Object"]).entries)` receiver-wrapped callable forms, with the direct resolution regression now also pinning the parenthesized receiver-wrapped bracket-root `Object.freeze((globalThis["Object"]).values)` / `Object.freeze((globalThis["Object"]).entries)` aliases in js-like input, plus the single-quoted parenthesized `Object.freeze((globalThis['Object'])["values"])` alias on the values slice, and the single-quoted parenthesized `Object.freeze((globalThis['Object'])["entries"])` alias on the entries slice, the logical-or `Object.entries` wrapper slice alongside the existing logical-and/nullish forms, the parenthesized receiver-wrapped bracketed `Object.entries` helper variant on that same browser-harness path, plus the parenthesized single-quoted bracketed `Object.entries` helper variant on that same browser-harness/browser-bundle matrix, the frozen `Object.entries` helper-call slice on the supported object-enumeration path, browser-bundle smoke now also exercises both the parenthesized receiver-wrapped bracketed and the parenthesized bracket-root `Object.entries` helper variants on that same slice, and the single-quoted parenthesized `Object.freeze((globalThis['Object'])["entries"])` alias on the same browser-runtime spread fixture, and the frozen root-object `Object.freeze((globalThis["Reflect"]))["ownKeys"]` alias on the supported `Reflect.ownKeys` slice, with the browser-harness/browser-bundle runtime-smoke matrix now also pinning that alias, and browser-requested dynamic-import smoke now also covering `Object.freeze`-wrapped literal imports with logical wrappers in `.js`, `.ts`, `.jsx`, and `.tsx` input, plus the parenthesized receiver-wrapped bracketed `Object.keys` / `Object.values` helper variants on the browser-harness/browser-bundle matrix, and the parenthesized single-quoted receiver-bracketed `Object.keys` helper variant on that same matrix, and the dotted parenthesized receiver-wrapped bracket-root `Object.freeze((globalThis["Object"]).keys)` / `Object.freeze((globalThis["Object"]).values)` forms on that same matrix, and `Array.from` over frozen `globalThis["Set"]` / `globalThis['Set']` / `globalThis['Map']` / `globalThis["Map"]` constructor results in standalone and browser harness/bundle smoke, plus parenthesized direct constructor aliases like `new (globalThis["Set"])` and `new (globalThis['Map'])`, and the helper inventory now also carries nullish/logical wrappers around the bracketed `globalThis["Set"]` / `globalThis['Set']` / `globalThis["Map"]` / `globalThis['Map']` constructor spellings, the object-has-own helper inventory now also includes single-quoted bracketed, parenthesized single-quoted bracketed, and frozen single-quoted bracketed callable aliases on the supported `Object.fromEntries` path, and the Math.round frozen-callable inventory now also includes the parenthesized receiver-wrapped mixed-bracket `Object.freeze((globalThis.Math)["round"])` alias on the supported round slice;
- iterator smoke now also covers frozen `Object.entries` helper calls on the supported object-enumeration slices, and array callback-produced iterables such as `map`/`filter`/`find`/`findIndex`/`findLast`/`findLastIndex`/`flatMap`/`some`/`every`/`reduce`/`reduceRight` remain on the canonical `E5506` gate across build/check/run/test smoke, with browser-harness coverage now also spanning JS/TS/JSX/TSX input; browser-harness/browser-bundle spread smoke now also covers frozen callable `Object.keys` / `Object.entries` aliases, including the parenthesized receiver-wrapped bracketed and single-quoted forms;
- browser harness map/set constructor iteration smoke now also covers the parenthesized bracketed `new (globalThis["Set"])` / `new (globalThis['Set'])` and `new (globalThis["Map"])` / `new (globalThis['Map'])` aliases in JS, TS, JSX, and TSX input, and the frozen-callable constructor inventories now also carry nullish/logical wrapper aliases around the root and `globalThis` constructor forms;
- browser object-values spread bundle and harness smoke now also cover the mixed double/single-quoted `globalThis["Object"]['values']` helper alias alongside the existing spread helper inventory;
- browser-harness threaded-budget smoke now also covers the canonical positive `--max-threads` path with and without `--wasm-threads` across explicit and inherited browser API-surface forms, keeping the phase-22 resource gate evidence-backed; threaded-runtime-global rejection smoke now also covers single-quoted bracketed `SharedArrayBuffer` / `Atomics` aliases on the same gate;
- browser-runtime unavailability diagnostics now also name the `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` opt-in env var so the harness-backed browser-requested path and the still-gated standalone browser runtime contract stay easier to distinguish in CLI output;
- Promise.any browser smoke now also covers mixed-bracket, transparent wrapper, and bracketed frozen-callable aliases on the shared browser body, and compiler build smoke now also exercises that shared `Promise.any` body across Deno and browser `ts` / `js` / `jsx` / `tsx` inputs; the browser harness/bundle path now also covers bracket-root frozen `globalThis["Promise"]["any"]` / `globalThis['Promise']['any']` aliases, plus the mixed-bracket frozen-callable alias family around `globalThis["Promise"]['any']`, and the parenthesized receiver-wrapped dotted `Object.freeze((globalThis["Promise"]).any)` / `Object.freeze((globalThis['Promise']).any)` aliases.
- library, WIT, C ABI, component, metadata, and binding-package artifact lanes;
- deterministic PGO input handling and benchmark fixtures, including the new `math-round-builtin` / `math-round-builtin-js` pair now does the same for `Math.round`, and the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`), and browser-targeted build smoke now also covers the `Math.hypot(3, 4)` perfect-square-sum slice in JSX and TSX input, with the `Math.hypot` frozen-callable smoke now also widening to the parenthesized receiver-wrapped bracket-root spellings around `globalThis["Math"]` and `globalThis['Math']`, and browser-targeted build smoke now also covers the `Math.floor` / `Math.trunc` / `Math.ceil` alias-chain and bracketed literal slices in JSX and TSX input on the browser API-surface path;
- Lean proof infrastructure with proof-backed claims limited to the published boundary.

## Planning boundary

Active planning starts from remaining spec gaps, not from historical Phase 1 through Phase 20 checklists. Do not re-add completed packet journals to plan files.

Open work is concentrated in:

- faithful full-language semantics for currently gated constructs;
- host/runtime capability contracts that require sandbox, effects, and resource mediation;
- package compatibility growth by exact support rung and context;
- optimization/performance claims backed by deterministic evidence;
- proof, schema, diagnostic, and CLI contract widening without claim drift.
