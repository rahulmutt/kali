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
- Progress note: `Math.pow` now has browser-harness TS/JS-input smoke coverage for the positive-integer-exponent alias-chain slice plus representative negative-exponent E5506 rejection coverage, and the documented `globalThis.Math.pow` root now mirrors that slice in the browser harness and browser-bundle smoke paths; browser bundle smoke now also covers that alias-chain slice in JSX and TSX input, alongside the existing checker/codegen/runtime coverage.
- Progress note: the browser-bundle `globalThis["Math"]` root smoke for the `Math.pow` zero-exponent non-integer-base slice now also covers JSX and TSX input, keeping the source-class matrix aligned with the browser-harness coverage for that supported slice.
- Progress note: the default standalone surface now also accepts the `SharedArrayBuffer` / `Atomics` threaded-globals slice on `.js` input under `--wasm-threads`, including inherited runtime-profile coverage alongside the existing Node-surface acceptance tests.
- Progress note: the `effects` command now also accepts that same `--wasm-threads` profile slice on `.js` input, including inherited runtime-profile coverage and the JSON effects envelope, keeping the threaded-profile evidence aligned across analysis and execution-style host surfaces.
- Progress note: bracketed and mixed dot/bracket `Deno.env.set` / `Deno.env.delete` spellings now also have direct type/codegen smoke coverage on the default standalone and browser-late-compat paths, keeping the mutable-env slice aligned across the documented alias families.
- Progress note: the browser JS-input permission-escalation smoke now also includes the mixed-bracket `globalThis.Deno["permissions"]["request/revoke"]` spellings in source coverage, while the rejection diagnostics still canonicalize through the existing permission-escalation API gate.
- Progress note: browser and standalone late-process-control smoke now also pin the bracketed `globalThis["process"].{pid,cwd,chdir,exit}` and `globalThis.process["pid"|"cwd"|"chdir"|"exit"]` spellings in JS-input regression helpers, keeping the process-control alias matrix aligned with the existing dotted forms.
- Progress note: `Math.exp2` now also has the exact zero-identity fold on the supported codegen/runtime smoke path, and browser-harness/browser-bundle smoke now also covers the `globalThis.Math.exp2` zero-identity slice in JSX and TSX input alongside the existing TS and `.js` coverage.
- Progress note: browser-harness and browser-bundle smoke now also cover the `Math.sqrt` / `Math.cbrt` literal slices in JSX and TSX input, extending the existing TS and `.js` coverage on the supported browser-requested and browser-bundle paths.
- Progress note: browser-harness and browser-bundle smoke now also cover the `globalThis.Math.sqrt` / `globalThis.Math.cbrt` root slice in TS and `.js` input, keeping the supported root-access matrix aligned with the bare `Math.sqrt` / `Math.cbrt` slice.
- Progress note: browser-harness and browser-bundle smoke now also cover the mixed-bracket `globalThis.Math["expm1"]` / `globalThis.Math["log1p"]` spelling in TS and `.js` input, alongside the existing `globalThis.Math.expm1` / `globalThis.Math.log1p`, bracketed-root, and fully bracketed-root coverage.
- Progress note: type-resolution and codegen smoke now also pin the `Math.expm1` / `Math.log1p` const numeric alias-chain slice, keeping the zero-identity evidence aligned with the documented current repository state.
- Progress note: browser bundle smoke now also covers the const string-alias slice for `for...of` and `for await...of` on the browser API surface in both TS and `.js` input.
- Progress note: browser build smoke now also covers the `for await...of` `as const` and `satisfies` wrapper slices in TS input on both the Deno and browser surfaces, and browser bundle smoke now also covers those wrapper slices in `.js` input.
- Progress note: direct build smoke now also covers the `for await...of` parenthesized-binding wrapper slice in Deno and browser TS and `.js` inputs, matching the already-covered codegen/runtime slice.
- Progress note: browser-harness smoke now also covers the supported `globalThis.Math.exp` / `globalThis.Math.log` exact-identity slice in TS and `.js` input, and browser build smoke mirrors that root-access slice on the browser API surface.
- Progress note: browser-harness and browser-bundle smoke now also cover the bracketed `globalThis["Math"]` root for the `Math.exp` / `Math.log` identity slice in TS and `.js` input.
- Progress note: browser-harness and browser-bundle smoke now also cover the direct `globalThis.Math.log2` / `globalThis.Math.log10` root spelling alongside the bracketed-root slice in TS and `.js` input.
- Progress note: browser-harness/browser-bundle JSON success paths for the `globalThis.Math.atan2` / `globalThis["Math"].atan2` and mixed `globalThis.Math["log2"]` / `globalThis.Math["log10"]` slices now also assert empty `errors` arrays, keeping the deterministic JSON envelope explicit on those smoke paths.
- Progress note: browser-harness and browser-bundle smoke now also cover the fully bracketed `globalThis["Math"]["exp"]` / `globalThis["Math"]["log"]` identity slice in TS and `.js` input.
- Progress note: browser-harness and browser-bundle smoke now also cover the exact `Math.asinh` / `Math.acosh` / `Math.atanh` identity slice in TS and `.js` input.
- Progress note: browser-harness and browser-bundle smoke now also cover the bracketed `globalThis["Math"]` root for the `Math.sin` / `Math.cos` / `Math.tan` zero-identity slice in TS and `.js` input.
- Progress note: browser-harness and browser-bundle smoke now also cover the bracketed `globalThis["Math"]` root for the `Math.pow` zero-exponent non-integer-base slice in TS and `.js` input.
- Progress note: browser-harness and browser-bundle smoke now also cover the bracketed `globalThis["Math"]` / fully bracketed `globalThis["Math"]["floor"]`, `globalThis["Math"]["trunc"]`, and `globalThis["Math"]["ceil"]` slices in TS and `.js` input.
- Progress note: browser bundle smoke now also covers the `Math.floor` / `Math.trunc` / `Math.ceil` const-alias-chain slice in TS and `.js` input.
- Progress note: the static numeric-literal `Object.is` slice now folds deterministically on the supported standalone/browser smoke paths, including const alias chains and bracketed `globalThis["Object"]["is"]` spellings, and browser-harness run/test smoke now also mirrors that slice in TS and `.js` input.
- Progress note: `Object.fromEntries` now also folds the `globalThis.Object.fromEntries` and bracketed `globalThis["Object"]["fromEntries"]` alias forms on the derived-shape object-enumeration path, and the browser/runtime smoke now pins those aliases alongside the existing bare-call coverage.
- Progress note: browser bundle smoke now also covers the mixed `globalThis.Object.prototype["hasOwnProperty"]["call"]` spelling for the supported `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` slice in TS and `.js` input, and now also mirrors the `globalThis.Object["prototype"]["hasOwnProperty"]["call"]`, `globalThis["Object"].prototype["hasOwnProperty"]["call"]`, and fully bracketed `globalThis["Object"]["prototype"].hasOwnProperty["call"]` spellings in that same browser bundle slice.
- Progress note: bounded library-export coverage now also includes async `const` function-expression bindings through transparent conditional wrappers and export aliases, and default-export function expressions now also keep their inferred signatures through transparent decorated wrappers.
- Progress note: browser-requested run/test JSON-output coverage now also covers the `Math.asinh` / `Math.acosh` / `Math.atanh` identity slice and the `Math.sinh` / `Math.cosh` / `Math.tanh` zero-identity slice in TS and `.js` input.
- Progress note: browser-requested run/test coverage on the browser API surface now also rejects generator and async-generator spellings in `.js` input with canonical E5506 errors, keeping the runtime smoke aligned with the existing generator gate.
- Progress note: browser-harness run/test coverage now also mirrors the `for await...of` `as const` and `satisfies` wrapper slices in `.js` input, closing the remaining JS-input parity gap for that wrapper slice on the supported browser-requested run/test path.
- Progress note: browser-harness run/test and JSON-output coverage now also exercises the `for...of` and `for await...of` `as const` / `satisfies` wrapper slices in TSX input, keeping the browser-requested iterator harness matrix aligned across the supported wrapper slice.
- Progress note: executable build smoke now also covers the default standalone `Deno.env.set` / `Deno.env.delete` slice across TS, JS, JSX, and TSX input, including the fully bracketed `globalThis["Deno"]["env"]["set/delete"]` spellings, and the browser bundle/browser-harness JSON-output paths for the supported iterator and object-helper smoke slices now also pin empty `errors` arrays to keep the success envelope explicit.
- Progress note: browser-harness JSON-output coverage now also exercises the `for...of` and `for await...of` `as const` / `satisfies` wrapper slices in `.js` input, keeping the browser-requested JSON envelope aligned with the existing smoke coverage for those iterator wrappers.
- Progress note: browser-harness `run` / `test` smoke now also covers the parenthesized const-alias wrapper slice for both `for...of` and `for await...of` in `.js` input, keeping the JS-input browser-requested iterator harness matrix aligned with the existing build/runtime slice.
- Progress note: bounded library-export smoke now also preserves the conservative `unknown` signature fallback for mixed conditional public-boundary exports whose function-shaped branches disagree.
- Progress note: the browser-requested unreadable-summary fallback smoke now also covers `.ts` input on the supported JSON `test` and human `run` / `test` paths, keeping the stdout-authoritative rule aligned across source classes.
- Progress note: the browser-requested whitespace-only summary fallback smoke now also covers `.ts` and `.tsx` input on the supported JSON `test` path, keeping the blank-summary fallback aligned across the browser-capable source classes.
