# Phase 6 Conformance Dashboard

This dashboard is a deterministic implementation snapshot for the language features called out in the phase-6 roadmap.

- Normative availability still comes from [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
- Evidence links point to tests or canonical diagnostic gates.
- Rows are grouped by the current maturity bucket so the supported / gated / rejected split stays obvious.

## Supported today

| Feature | Status | Evidence |
|---|---|---|
| Latest published ECMA-262 lexical grammar (tokenization) | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| Current-edition non-Annex-B semantics for features Kali marks as supported in a given command/profile | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_runtime/src/tests.rs` |
| Static ESM `import` / `export` | Phase 1 MVP | `crates/kali_parser/src/tests.rs`, `crates/kali_types/src/tests.rs` |
| Async function declarations / expressions and async generator syntax | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| Arrow function expressions with concise bodies | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| Generator function declarations / expressions and `yield` / `yield*` expressions | Phase 1 MVP | `crates/kali_parser/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (including browser-targeted `.js` generator-gate fixtures plus JSON-output coverage for the browser-analysis and browser-bundle JS-input `yield*` gates, along with JSON-output coverage on the run/test JS-input generator gate and the JS-input generator-function lowering gate on the `build` / `run` / `test` lanes, plus the matching browser-bundle TS-input JSON `yield*` gate) |
| First-class JavaScript compilation with bounded inference | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` |
| Object literal expressions with identifier, string, and numeric property names, including shorthand identifier properties | Phase 1 MVP | `crates/kali_parser/src/tests.rs`, `crates/kali_hir/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` |
| Read-only `Deno.permissions.query(...)` over the shared descriptor subset (`read`, `write`, `env`, `net`) | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_sandbox/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_api_deno/src/tests.rs` |
| Budgeted local/intra-module constraint solving inside the shared bounded inference contract | Phase 1 MVP | `crates/kali_types/src/tests.rs` |
| Basic arithmetic precedence and array length semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic arithmetic precedence and array length semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic function call return semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input coverage) |
| Basic relational comparison semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input coverage) |
| Basic strict equality / inequality semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage) |
| Basic boolean conjunction / disjunction semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including mirrored `.js` input coverage) |
| Basic async/await sequencing | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage on the `run` and `test` paths) |
| Basic queueMicrotask ordering semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Browser timing baseline / `performance.now()` monotonic ordering in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (browser-requested `run` / `test` and browser bundle smoke coverage, including inherited-browser-api-surface coverage and JSON-output coverage for the browser-requested `.js` `test` lane) |
| Basic optional chaining member and element access | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage on the `run` and `test` paths, plus browser-requested `.js` harness coverage) |
| Basic try/catch exception handling and try/finally sequencing | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage) |
| Browser bundle try/catch exception semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Basic BigInt addition semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, plus browser bundle smoke coverage in both TS and `.js` inputs) |
| Basic BigInt addition semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, plus browser bundle smoke coverage in `.js` input) |
| Basic `Object.keys()` enumeration semantics, including overwrite ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.keys()` enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.entries()` enumeration semantics, including overwrite ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.entries()` enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.values()` enumeration semantics, including overwrite ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.values()` enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.keys()` / `Object.entries()` / `Object.values()` string-primitive enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.keys()` / `Object.entries()` / `Object.values()` enumeration semantics in `.js` input, including overwrite ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Browser-requested `run` / `test` Web Crypto `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `crypto.getRandomValues()` plus `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` / `Math.clz32()` semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `try/catch` exception semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `try/finally` sequencing in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `async/await` sequencing in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `queueMicrotask` ordering in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.ts` and `.js` input) |
| Browser bundle basic strict equality / inequality semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle basic boolean conjunction / disjunction semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage) |
| Browser bundle `Object.keys()` / `Object.entries()` / `Object.values()` enumeration semantics in `.ts` and `.js` input, including string-primitive enumeration and overwrite ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle integer-like key ordering semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (browser execution smoke plus JSON-output coverage) |
| Browser-requested `run` / `test` object-enumeration semantics in `.ts` and `.js` input, including string-primitive enumeration in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` integer-like key ordering semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.ts` and `.js` input) |
| Browser-requested `run` / `test` boolean conjunction / disjunction semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage for the browser-requested JS-input lane) |
| Browser-requested `run` / `test` basic strict equality / inequality semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.ts` and `.js` input) |
| Browser-requested `run` / `test` queueMicrotask ordering in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` basic try/catch exception handling and try/finally sequencing in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` Web Crypto randomness subset via `crypto.getRandomValues()` in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage) |
| Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `test` `Math.clz32()` semantics in `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics in `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Basic `Math.max()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.min()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.abs()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.sign()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.imul()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.clz32()` built-in semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including browser-requested `.js` harness JSON coverage) |
| Console error / warn / info / debug routing plus `console.assert()` false-branch reporting | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input console-level routing and assertion coverage) |
| Browser-requested `run` / `test` console error / warn / info / debug routing plus `console.assert()` false-branch reporting in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage) |
| Browser bundle console error / warn / info / debug routing plus `console.assert()` false-branch reporting | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (browser bundle TS and `.js` input, including JSON-output coverage) |
| Web Crypto randomness subset (`crypto.getRandomValues`, mapping to the canonical `Random.GetBytes` effect / `effects.random` policy key) | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including browser-requested `run` / `test` `.js` harness coverage and JSON-output coverage for that slice), `crates/kali_api_web/src/tests.rs`, `crates/kali_api_deno/src/tests.rs` |
| Browser-requested Web Crypto `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| CommonJS module lowering | Phase 1 MVP | `crates/kali_codegen/src/tests.rs` |
| Mixed CommonJS/ESM package default-import interop | Phase 1 MVP | `crates/kali_cli/tests/package_corpus.rs` |
| Canonical `semver` package corpus probe on `.js` input for the default standalone surface | Phase 1 MVP | `crates/kali_cli/tests/package_corpus.rs` |
| `require("literal")` | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_codegen/src/tests.rs` |
| Literal-string `import()` over the linked graph, including directory-index targets | Phase 3 target | `crates/kali_types/src/tests.rs`, `crates/kali_cli/src/build_tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (standalone run/test TS and `.js` input plus browser-targeted bundle smoke) |
| Generator lowering | Later compatibility | `crates/kali_parser/src/tests.rs`, `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (`check` / `build` / `run` / `test` gate coverage, including async generator syntax, async generator function-expression syntax, minimized `.js` input fixtures, and browser-targeted `.js` mirrors) |
| Integer-like key ordering in object enumeration | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including browser-requested `run` / `test` harness coverage and standalone run/test JSON-output coverage in `.js` input) and `crates/kali_cli/tests/runtime_smoke.rs` browser-bundle coverage now exercise the 4-key ordering path across TS and `.js` inputs |

## Gated for later phases

| Feature | Status | Evidence |
|---|---|---|
| Open-ended or unstable cross-module/public-API constraint solving | Phase 3 target | Canonical `E5506`/annotation-required boundary in `specs/04-type-system.md` and `specs/19-feature-maturity.md`; public export-boundary negative coverage in `crates/kali_types/src/tests.rs` |
| Non-literal `import(expr)` | Rejected by default | Canonical non-literal dynamic-loading gate (`E5506`); `crates/kali_types/src/tests.rs`; `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input coverage on `check` / `build`) |
| `for...of` array iteration | Rejected by default | Canonical iterator-lowering gate (`E5506`); `crates/kali_parser/src/tests.rs`, `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (TS and `.js` `check` / `build` / `run` / `test` coverage) |
| Unsupported `Deno.permissions.query(...)` descriptor kinds (for example `ffi` / `sys`-style names) | Rejected by default | Canonical Phase-1 Deno permission-facade gate (`E5506`); `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/permission_query_js_input.rs` |
| `eval` | Phase 4 compatibility | Canonical `--compat eval` gate and `E5506` until enabled |
| `Function()` constructor | Phase 4 compatibility | Canonical `--compat eval` gate and `E5506` until enabled |
| `Proxy` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `WeakMap` / `WeakSet` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `FinalizationRegistry` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `SharedArrayBuffer` / `Atomics` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| Broader `Intl` surface | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |

## Rejected by default

| Feature | Status | Evidence |
|---|---|---|
| Stage-3+/draft TC39 proposals beyond the latest published ECMA-262 edition | Rejected by default | Canonical rejected-by-default row in `specs/19-feature-maturity.md` |
| Dynamic `require()` | Rejected by default | Canonical rejected-by-default row and `E5506`/unsupported-dynamic-loading diagnostics |
| Non-literal `import(expr)` | Rejected by default | Canonical non-literal dynamic-loading gate (`E5506`); `crates/kali_types/src/tests.rs`; `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input coverage on `check` / `build`) |
| Unsupported `Deno.permissions.query(...)` descriptor kinds such as `ffi` / `sys` | Rejected by default | Canonical Phase-1 Deno permission-facade gate (`E5506`); `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/permission_query_js_input.rs`, `crates/kali_cli/tests/permission_query_ts_input.rs` |
| Browser-targeted `for...of` array iteration | Rejected by default | Canonical iterator-lowering gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs` (`check` / `build --bundle` on `.js` input, including JSON-output coverage) |
| Nullish coalescing `??` | Rejected by default | Parser acceptance plus canonical `E5506` availability gate in `crates/kali_parser/src/tests.rs`, `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_js_input.rs`, and `crates/kali_cli/tests/late_compat_browser_js_input.rs` |

## Reading note

This dashboard is intentionally smaller than the full maturity matrix. It only tracks the ECMA/TS slice that the Phase 6 roadmap cares about today, and it stays sorted by maturity bucket so new rows can be compared without guessing at the intended status.
