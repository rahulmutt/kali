# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for public support status.

## Language semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Implement or deliberately keep gated high-cost language semantics that are still unavailable despite parser/diagnostic coverage: generator lowering, broader `for...of` / iterator lowering beyond the supported literal-array slice (literal elements, const numeric alias elements, const string alias elements, simple variable bindings, and parenthesized identifier binding wrappers, plus the supported transparent parenthesized/type-assertion/satisfies/chain wrappers and TS `as const` wrapper cases, now including browser-harness/browser-bundle coverage for the `for...of` `as const` wrapper slice, browser-harness `run` / `test` JS-input coverage for the `for...of` `as const` and `satisfies` wrapper slices, and the matching `for await...of` parenthesized binding wrappers now also covered in browser-harness smoke, with the previously missing TS-run / JS-test parity now closed for that wrapper slice in the browser-harness matrix), browser-harness JSON-output coverage now also pins the parenthesized-binding wrapper slice for both loops in `.js` input, and the browser-harness JSON-output matrix now also covers that same parenthesized-binding slice in TSX input, broader `for await...of` / async-iterator semantics beyond that same literal-array slice, the remaining gated `Math` members (with `Math.floor` now also constant-folding statically-known numeric literals and resolving const numeric alias chains, `Math.trunc` / `Math.ceil` now also constant-folding statically-known numeric literals, the statically-known perfect-square integer literal path for `Math.sqrt` plus the statically-known perfect-cube integer literal path for `Math.cbrt` staying covered on the supported subset, `Math.exp` / `Math.log` now also having exact zero/one identity folds, `Math.expm1` / `Math.log1p` now also having exact zero identity folds, including const numeric alias chains, `Math.atan2` now also supporting the zero-numerator / non-negative-denominator literal slice, including const numeric alias chains, on the supported checker/codegen/build path while broader calls remain gated, browser-requested run/test smoke now also covering the exact zero-identity `Math.sinh` / `Math.cosh` / `Math.tanh` slice in TS and `.js` input, browser-harness/browser-bundle smoke for that slice now also covering the `globalThis.Math.sinh` / `globalThis.Math.cosh` / `globalThis.Math.tanh` root forms, browser-harness smoke for that slice now also covering the positive-integer-exponent `Math.pow` alias-chain case and the exponent-one identity slice in TS and `.js` input, browser-harness/browser-bundle smoke now also covers the base-one `Math.pow` identity slice in JS input, and browser-requested run/test JSON-output coverage now also covers the `Math.asinh` / `Math.acosh` / `Math.atanh` identity slice and the `Math.sinh` / `Math.cosh` / `Math.tanh` zero-identity slice in TS and `.js` input, browser-harness/browser-bundle root smoke for `Math.expm1` / `Math.log1p` now also covers JSX and TSX input, and the mixed-bracket / fully bracketed root smoke for those same `Math.expm1` / `Math.log1p` spellings now also covers JSX and TSX input, compound assignment on non-local targets and immutable bindings (now pinned as explicit E5506 gates on the CLI smoke paths), plain `build` rejection on the same non-literal slice in `.ts` input with matching JSON-output coverage, and non-literal `import(expr)`.
- Progress note: browser-harness and browser-bundle smoke now also cover the `globalThis.Math.sqrt` / `globalThis.Math.cbrt` root slice in TS and `.js` input, keeping the supported root-access matrix aligned with the bare `Math.sqrt` / `Math.cbrt` slice.

- Progress note: browser-requested `run` / `test` harness coverage now also rejects non-literal `import(expr)` in `.js` input with matching JSON-output coverage, keeping the browser-requested dynamic-loading gate aligned with the existing browser-targeted `check` / `build --bundle` and standalone `run` / `test` rejection lanes.
- Expand TypeScript/JavaScript inference only inside the bounded inference contract; keep open-ended public-API and cross-module solving gated until deterministic budgets and evidence exist. Progress note: transparent decorated wrappers now also peel through the static string/numeric/object-identity/object-target helpers so the bounded literal slice stays aligned with HIR-transparent wrappers.
- Progress note: the late-host/object-model member-access naming helpers now also peel transparent parenthesized/type-assertion/satisfies/chain/decorated wrappers around member roots, keeping wrapped alias spellings aligned for env-materialization and late-object-model gates.
- Progress note: the shared literal-array iterator lowering helper now also peels transparent `DecoratedExpression` wrappers around supported array iterables, array elements, and simple binding targets.
- Progress note: browser-harness run/test and JSON-output coverage now also exercises the `for...of` and `for await...of` `as const` / `satisfies` wrapper slices in TS and JSX input, keeping the iterator smoke aligned with the browser-bundle source-class matrix.
- Progress note: browser build smoke now also covers the `for await...of` `as const` and `satisfies` wrapper slices on the browser API surface in `.js`, `.jsx`, and `.tsx` input.
- Progress note: browser-harness run/test and JSON-output coverage now also pins the `for...of` and `for await...of` const-alias-chain slice in TS input, keeping the aliased iterator smoke aligned with the existing bundle evidence.
- Progress note: browser bundle smoke now also covers the `for...of` const-alias-chain slice in JSX and TSX input, and the `for await...of` const-alias-chain slice now also extends to JSX and TSX input across the browser-bundle and browser-harness smoke paths.
- Progress note: browser bundle smoke now also covers the `for...of` `as const` wrapper slice in JSX and TSX input, extending the browser-bundle source-class matrix for that bounded iterator slice.
- Progress note: browser-harness `run` / `test` smoke now also covers the parenthesized const-alias wrapper slice for both `for...of` and `for await...of` in `.js` input, keeping the JS-input browser-requested iterator harness matrix aligned with the existing build/runtime slice.
- Progress note: browser-harness `run` / `test` smoke now also covers the const-alias-chain slice for both `for...of` and `for await...of` in `.js` input, and browser-harness JSON-output coverage now also pins that same JS-input const-alias-chain slice for both loops, closing the remaining browser-requested harness parity gap for that bounded iterator slice.
- Progress note: codegen/type smoke now also pins the exact `Math.sin` / `Math.cos` zero-identity slice, plus the matching non-identity rejection lane, keeping that supported trig pair aligned with the existing `Math.tan` gate.
- Continue turning parser-only acceptance into either faithful runtime/checker support or explicit canonical gates (`E5506`) with mirrored TS, JS, JSX, TSX, browser, Node, and JSON-output regressions where applicable.
- Progress note: the mutable-local compound-assignment/update-expression slice now also peels transparent `DecoratedExpression` wrappers around the target, keeping the transparent-wrapper handling aligned with the existing parenthesized/type-assertion/satisfies update-target support.
- Maintain and simplify conformance dashboards as snapshots of supported/gated semantics rather than progress logs.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Complete threaded-runtime semantics beyond opt-in profile acceptance and host-import plumbing, including guest-facing multi-worker/thread behavior where the spec permits it.
- Decide whether `run --api browser` / `test --api browser` should graduate from harness-assisted later compatibility to a stable standalone browser runtime contract; if yes, specify host ownership, sandbox limits, summary JSON behavior, and failure modes first.
- Add late host APIs only with explicit policy/effect/resource contracts: environment materialization/mutation, process chdir/exit, subprocess spawning, socket/listener APIs, and late Node built-ins. Progress note: bare `check` / `build` / `run` / `test` now explicitly gate the phase-three Deno subprocess and socket/listener APIs (`Deno.Command`, `Deno.connect`, `Deno.listen`, `Deno.serve`) with canonical `E5506` diagnostics until the explicit sandbox/process-budget/network contracts exist. Progress note: the browser late-compatibility network matrix now also rejects the socket/listener slice on the browser API surface in JS input, including the bracketed `globalThis["Deno"]` alias spellings, keeping the remaining phase-three network gap explicit there too. Progress note: the browser late-compatibility env-mutation matrix now also rejects the fully bracketed `globalThis["Deno"]["env"]["set/delete"]` spellings, keeping that rejection lane aligned with the already-covered mixed and partially bracketed forms. Progress note: standalone runtime smoke now also covers the `Deno.chdir` alias family in `test` and JSON `test` paths on JS input.
- Progress note: the browser-requested `run` / `test` smoke for the supported `Object.is` numeric-literal slice now also covers TSX input on the browser API surface, including the JSON `run` path, keeping that object-helper slice aligned across the browser harness source-class matrix.
- Triage late object/runtime APIs (`Proxy`, own-property helpers, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`) with conformance and no-GC/no-JIT compatibility evidence before promotion. Progress note: the own-property-helper rejection matrix now also pins the mixed-bracket `Object.prototype["hasOwnProperty"].call` / `globalThis["Object"].prototype["hasOwnProperty"].call` spellings.
- Keep browser-targeted build/check support, browser harness execution, and Kali-hosted sandbox enforcement distinct.

## Ecosystem and packages

Owners: `specs/11-standard-apis.md`, `specs/14-packages.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Expand package-corpus coverage by host/API context and support rung without turning individual package successes into blanket npm claims.
- Grow Node package support only where the required Node built-ins and process APIs are explicitly supported or deliberately gated.
- Grow browser deployability and browser-harness package evidence while keeping published binary entrypoints and native/bootstrap-heavy packages rejected by default.
- Progress note: the browser package-corpus published-bin-entrypoint probes now also reject the `@mariozechner/pi-coding-agent` browser runtime entrypoint on both the explicit browser surface and the inherited browser surface when a harness command is configured, keeping that negative evidence separate from ordinary browser-package content support.
- Keep registry-analysis commands (`package-effects`, `package-audit`) single-package and registry-identifier based unless a future spec revision defines batch/local/raw-URL behavior.

## Optimization, PGO, and performance

Owners: `specs/07-specialization.md`, `specs/08-wasm-codegen.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Deepen `release` and `release-advanced` optimizations while preserving JavaScript-visible semantics, sandbox effects, and deterministic artifacts.
- Treat `--profile` as a deterministic build-only additive input; do not create a hidden fourth build mode.
- Promote performance wording only when benchmark evidence names workload, build mode, baseline, and reproducibility constraints.
- Keep optimization inventories as concise current-evidence snapshots, not implementation journals.

## Verification and contracts

Owners: `specs/16-testing.md`, `specs/17-verification.md`, `specs/18-schemas.md`, `proofs/BOUNDARY.md`.

Remaining goals:

- Widen Lean models for ownership/effects/lowering in small named slices, and update `proofs/BOUNDARY.md` before any proof-backed wording changes.
- Expand proof CI triggers only when the published proof boundary claims implementation or spec paths outside the proof tree.
- Continue hardening JSON payload, artifact-manifest, schema-drift, and CLI-doc contracts while respecting schema extension posture.
- Avoid duplicating theorem inventories in plan files; `proofs/BOUNDARY.md` remains the sole proof-boundary inventory.
