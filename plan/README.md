# Active Plan Navigation

This directory contains only active continuation planning. Completed historical phase checklists were removed so implemented work is not treated as open.

## Files

- [`00-current-state.md`](./00-current-state.md) — checked-in implementation baseline used by the active roadmap.
- [`01-roadmap.md`](./01-roadmap.md) — continuation phase order, dependencies, and promotion gates.
- [`02-spec-gap-map.md`](./02-spec-gap-map.md) — remaining implementation goals mapped to owning specs.
- [`03-evidence-and-release-gates.md`](./03-evidence-and-release-gates.md) — evidence required before support claims widen.
- [`04-risk-register.md`](./04-risk-register.md) — active risks for future work.

## Active phases

- [`phase-11/`](./phase-11/README.md) — language semantics and conformance closure.
- [`phase-12/`](./phase-12/README.md) — runtime, host, and capability expansion.
- [`phase-13/`](./phase-13/README.md) — ecosystem compatibility expansion.
- [`phase-14/`](./phase-14/README.md) — optimization and performance promotion.
- [`phase-15/`](./phase-15/README.md) — verification and machine-contract widening.

## Rules

- Do not reopen removed Phase-1 through Phase-10 checklists as active work.
- Do not infer availability from this plan; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- Do not infer proof-backed scope from this plan; use [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).
- Any public CLI/schema/diagnostic behavior change must update the owning specs before the work is considered complete.
- Progress note: `Math.pow` now has browser-harness TS/JS-input smoke coverage for the positive-integer-exponent alias-chain slice plus representative negative-exponent E5506 rejection coverage, and the documented `globalThis.Math.pow` root now mirrors that slice in the browser harness and browser-bundle smoke paths, alongside the existing checker/codegen/runtime coverage.
- Progress note: browser bundle smoke now also covers the const string-alias slice for `for...of` and `for await...of` on the browser API surface in both TS and `.js` input.
- Progress note: browser build smoke now also covers the `for await...of` `as const` and `satisfies` wrapper slices in TS input on both the Deno and browser surfaces.
- Progress note: browser-harness smoke now also covers the supported `globalThis.Math.exp` / `globalThis.Math.log` exact-identity slice in TS and `.js` input, and browser build smoke mirrors that root-access slice on the browser API surface.
- Progress note: browser-harness and browser-bundle smoke now also cover the bracketed `globalThis["Math"]` root for the `Math.exp` / `Math.log` identity slice in TS and `.js` input.
- Progress note: browser-harness and browser-bundle smoke now also cover the exact `Math.asinh` / `Math.acosh` / `Math.atanh` identity slice in TS and `.js` input.
- Progress note: browser-harness and browser-bundle smoke now also cover the bracketed `globalThis["Math"]` root for the `Math.sin` / `Math.cos` / `Math.tan` zero-identity slice in TS and `.js` input.
- Progress note: browser bundle smoke now also covers the `Math.floor` / `Math.trunc` / `Math.ceil` const-alias-chain slice in TS and `.js` input.
- Progress note: the static numeric-literal `Object.is` slice now folds deterministically on the supported standalone/browser smoke paths, including const alias chains and bracketed `globalThis["Object"]["is"]` spellings.
