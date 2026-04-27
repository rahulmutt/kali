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
| Literal-string `import()` lowering over the linked graph, including directory-index targets | Phase 3 target | `crates/kali_types/src/tests.rs`, `crates/kali_cli/src/build_tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (standalone run/test TS and `.js` input, including JSON-output coverage for the `test` lane, plus browser-targeted bundle smoke) |
| Directory-index dynamic-import targets in `.tsx` and `.jsx` input | Phase 3 target | `crates/kali_types/src/tests.rs`, `crates/kali_cli/src/build_tests.rs` |
| Async function declarations / expressions and async generator syntax | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| Arrow function expressions with concise bodies | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| Async `await` expressions in async functions | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| `for await...of` statements in async functions | Phase 1 MVP | `crates/kali_parser/src/tests.rs` |
| Generator function declarations / expressions and `yield` / `yield*` expressions | Phase 1 MVP | `crates/kali_parser/src/tests.rs`, `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (including browser-targeted `.js` generator-gate fixtures plus JSON-output coverage for the browser-analysis and browser-bundle JS-input `yield*` gates, along with JSON-output coverage on the JS-input generator gate across `check` / `build` / `run` / `test` and the browser-analysis `check` lane, plus the JS-input generator-function lowering gate on the `build` / `run` / `test` lanes, plus the matching browser-bundle TS-input JSON `yield*` gate) |
| First-class JavaScript compilation with bounded inference | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` |
| Object literal expressions with identifier, string, and numeric property names, including shorthand identifier properties | Phase 1 MVP | `crates/kali_parser/src/tests.rs`, `crates/kali_hir/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` |
| Read-only `Deno.permissions.query(...)` over the shared descriptor subset (`read`, `write`, `env`, `net`), including bracketed `query` access forms | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_sandbox/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_api_deno/src/tests.rs` |
| Budgeted local/intra-module constraint solving inside the shared bounded inference contract | Phase 1 MVP | `crates/kali_types/src/tests.rs` |
| Basic arithmetic precedence and array length semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic arithmetic precedence and array length semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic function call return semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input coverage) |
| Basic relational comparison semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input coverage) |
| Basic strict equality / inequality semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage) |
| Basic boolean conjunction / disjunction semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including mirrored `.js` input coverage) |
| Standalone `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` suite in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Basic async/await sequencing | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage on the `run` and `test` paths) |
| Shared web-baseline primitives (`structuredClone`, `AbortController`, `AbortSignal`, `EventTarget`, `CustomEvent`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`) in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including JSON-output coverage) |
| Browser-requested `run` / `test` shared web-baseline primitives (`structuredClone`, `AbortController`, `AbortSignal`, `EventTarget`, `CustomEvent`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`) in `.ts` and `.js` input, including inherited-browser-api-surface coverage and JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle shared web-baseline primitives (`structuredClone`, `AbortController`, `AbortSignal`, `EventTarget`, `CustomEvent`, `URL`, `URLSearchParams`, `TextEncoder`, `TextDecoder`) in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage) |
| Basic queueMicrotask ordering semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Browser timing baseline / `performance.now()` monotonic ordering in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (browser-requested `run` / `test` and browser bundle smoke coverage, including inherited-browser-api-surface coverage, JSON-output coverage for the browser-requested `.js` `run` lane, the inherited browser-api-surface `.js` run lane, the browser-requested `.js` `test` lane and the browser-requested `.ts` `test` lane, the inherited browser-api-surface `.js` and `.ts` `test` lanes, plus JSON-output coverage for the browser bundle `.js` lane) |
| Read-only `Deno.pid` / `globalThis.Deno.pid` handling in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including JSON-output coverage on the `test` lane) |
| Basic optional chaining member and element access | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage on the `run` and `test` paths, standalone JSON-output coverage on the `.js` `run` / `test` lanes, plus browser-requested `.js` harness coverage) |
| Basic try/catch exception handling and try/finally sequencing | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including mirrored `.js` input coverage) |
| Browser bundle try/catch exception semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Basic BigInt addition semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, plus browser bundle smoke coverage in both TS and `.js` inputs) |
| Basic BigInt addition semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, plus browser bundle smoke coverage in `.js` input) |
| Basic `Object.keys()` enumeration semantics, including overwrite/delete-reinsert ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic object property deletion / `in`-operator semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including JSON-output coverage in `.ts` and `.js` input) |
| Runtime object type / constructor semantics (`typeof`, `instanceof`) in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including JSON-output coverage in `.ts` and `.js` input) |
| Browser-requested `run` / `test` runtime object type / constructor semantics (`typeof`, `instanceof`) in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle runtime object type / constructor semantics (`typeof`, `instanceof`) in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Basic `Object.keys()` enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including JSON-output coverage in `.js` input) |
| Basic `Object.entries()` enumeration semantics, including overwrite/delete-reinsert ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.entries()` enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including JSON-output coverage in `.js` input) |
| Basic `Object.values()` enumeration semantics, including overwrite/delete-reinsert ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.values()` enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including JSON-output coverage in `.js` input) |
| Basic unary prefix semantics (`!`, unary `-`, unary `+`, and `void`) in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.keys()` / `Object.entries()` / `Object.values()` string-primitive enumeration semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Object.keys()` / `Object.entries()` / `Object.values()` enumeration semantics in `.js` input, including overwrite/delete-reinsert ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test plus standalone JSON-output coverage in TS and `.js` input) |
| Browser-requested `run` / `test` Web Crypto `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `crypto.getRandomValues()` plus `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` / `Math.clz32()` semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage for the `.ts` and `.js` `Math.max()` slices, the `.ts` and `.js` `Math.min()` / `Math.abs()` / `Math.sign()` slices, and the `.js` `Math.imul()` / `Math.clz32()` slices) |
| Browser bundle `Math.trunc()` semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.ts` and `.js` input) |
| Browser bundle `try/catch` exception semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.js` input) |
| Browser bundle `try/finally` sequencing in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.js` input) |
| Browser bundle `async/await` sequencing in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle `queueMicrotask` ordering in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.ts` and `.js` input) |
| Browser bundle basic strict equality / inequality semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle basic boolean conjunction / disjunction semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage) |
| Browser bundle `Object.keys()` / `Object.entries()` / `Object.values()` enumeration semantics in `.ts` and `.js` input, including string-primitive enumeration and overwrite/delete-reinsert ordering, plus JSON-output coverage for string-primitive enumeration and overwrite/delete-reinsert ordering | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser bundle integer-like key ordering semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (browser execution smoke plus JSON-output coverage) |
| Browser-requested `run` / `test` object-enumeration semantics in `.ts` and `.js` input, including overwrite/delete-reinsert ordering plus string-primitive enumeration in `.js` input and JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` basic object property deletion / `in`-operator semantics in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` integer-like key ordering semantics in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` boolean conjunction / disjunction semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage for the browser-requested JS-input lane) |
| Browser-requested `run` / `test` basic strict equality / inequality semantics in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.ts` and `.js` input) |
| Browser-requested `run` / `test` queueMicrotask ordering in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input under the configured browser harness | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` basic async/await sequencing in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` basic try/catch exception handling and try/finally sequencing in `.ts` and `.js` input, including JSON-output coverage, plus inherited browser-api-surface coverage in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` Web Crypto randomness subset via `crypto.getRandomValues()` in `.ts` and `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage) |
| Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage in `.ts` and `.js` input on the `run` and `test` lanes) |
| Browser-requested `run` / `test` `Math.clz32()` semantics in `.ts` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` `Math.clz32()` semantics in `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `test` `Math.clz32()` semantics in `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` `Math.clz32()` semantics when the browser API surface is inherited in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics in `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` `Math.trunc()` semantics in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` `Math.trunc()` semantics when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` / `Math.imul()` semantics when the browser API surface is inherited in `.ts` and `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` `Math.trunc()` semantics in `.js` input, including JSON-output coverage | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Basic `Math.max()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.min()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.abs()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.sign()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.imul()` built-in semantics | Phase 1 MVP | `crates/kali_codegen/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (run/test) |
| Basic `Math.clz32()` built-in semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, including standalone `.js` JSON-output coverage and browser-requested `.js` harness JSON coverage, plus inherited browser-api-surface `.js` coverage on `run` / `test`) |
| Basic `Math.trunc()` built-in semantics | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (run/test, `.js` input, including standalone JSON-output coverage) |
| Console error / warn / info / debug routing plus `console.assert()` false-branch reporting | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input console-level routing and assertion coverage) |
| Browser-requested `run` / `test` console error / warn / info / debug routing plus `console.assert()` false-branch reporting in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including JSON-output coverage) |
| Browser bundle console error / warn / info / debug routing plus `console.assert()` false-branch reporting | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (browser bundle TS and `.js` input, including JSON-output coverage) |
| Web Crypto randomness subset (`crypto.getRandomValues`, mapping to the canonical `Random.GetBytes` effect / `effects.random` policy key) | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` (including browser-requested `run` / `test` `.ts` and `.js` harness coverage and JSON-output coverage for that slice), `crates/kali_api_web/src/tests.rs`, `crates/kali_api_deno/src/tests.rs` |
| Browser-requested Web Crypto `crypto.subtle.digest()` / `crypto.randomUUID()` semantics in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-requested `run` / `test` zero-capable budget pair (`--max-threads 0` / `--max-spawned-processes 0`) in `.js` input | Phase 1 MVP | `crates/kali_cli/tests/runtime_smoke.rs` |
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
| `for await...of` array iteration | Rejected by default | Canonical iterator-lowering gate (`E5506`); `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` (TS and `.js` `run` / `test` coverage, plus browser-targeted `check` / `build --bundle` and browser-requested `run` / `test` `.js` coverage, including JSON-output coverage) |
| Unsupported `Deno.permissions.query(...)` descriptor kinds (for example `ffi` / `sys`-style names) | Rejected by default | Canonical Phase-1 Deno permission-facade gate (`E5506`); `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/permission_query_js_input.rs` |
| Environment snapshot materialization (`Deno.env.toObject`) | Later compatibility | Canonical later object-materialization gate; `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_js_input.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `eval` | Phase 4 compatibility | Canonical `--compat eval` gate and `E5506` until enabled; `crates/kali_cli/tests/late_compat_browser_js_input.rs` (explicit and inherited browser API-surface rejection coverage) |
| `Function()` constructor | Phase 4 compatibility | Canonical `--compat eval` gate and `E5506` until enabled; `crates/kali_cli/tests/late_compat_browser_js_input.rs` (explicit and inherited browser API-surface rejection coverage) |
| `Proxy` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_js_input.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `WeakMap` / `WeakSet` / `WeakRef` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_js_input.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `FinalizationRegistry` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| `SharedArrayBuffer` / `Atomics` | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |
| Broader `Intl` surface | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_js_input.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` (including bracketed `globalThis["Intl"]` / `globalThis["Intl"]["NumberFormat"]` / `globalThis["Intl"]["DateTimeFormat"]` source coverage) |
| Process identity and process-control/working-directory APIs (`process.pid`, `Deno.exit`, `Deno.cwd`, `Deno.chdir`, `process.chdir`, `process.exit`; read-only `Deno.pid` / `globalThis.Deno.pid` / bracketed `globalThis["Deno"]["pid"]` stays the default standalone exception) | Later compatibility | Canonical availability gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_js_input.rs`, `crates/kali_cli/tests/late_compat_browser_js_input.rs` |

## Rejected by default

| Feature | Status | Evidence |
|---|---|---|
| Stage-3+/draft TC39 proposals beyond the latest published ECMA-262 edition | Rejected by default | Canonical rejected-by-default row in `specs/19-feature-maturity.md` |
| Dynamic `require()` | Rejected by default | Canonical rejected-by-default row and `E5506`/unsupported-dynamic-loading diagnostics |
| Non-literal `import(expr)` | Rejected by default | Canonical non-literal dynamic-loading gate (`E5506`); `crates/kali_types/src/tests.rs`; `crates/kali_cli/tests/runtime_smoke.rs` (including `.js` input coverage on `check` / `build`) |
| Unsupported `Deno.permissions.query(...)` descriptor kinds such as `ffi` / `sys` | Rejected by default | Canonical Phase-1 Deno permission-facade gate (`E5506`); `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/permission_query_js_input.rs`, `crates/kali_cli/tests/permission_query_ts_input.rs` |
| Interactive permission escalation APIs (`Deno.permissions.request()` / `revoke()` and similar prompt-style flows) | Rejected by default | Canonical Phase-1 Deno permission-facade gate (`E5506`); `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs` |
| Browser-targeted `for...of` array iteration | Rejected by default | Canonical iterator-lowering gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs` (`check` / `build --bundle` on `.js` input, including JSON-output coverage) |
| Browser-requested `run` / `test` `for...of` array iteration in TS and `.js` input | Rejected by default | Canonical iterator-lowering gate (`E5506`); `crates/kali_cli/tests/runtime_smoke.rs` (browser-requested `run` / `test` on TS and `.js` input, including JSON-output coverage) |
| Nullish coalescing `??` | Rejected by default | Parser acceptance plus canonical `E5506` availability gate in `crates/kali_parser/src/tests.rs`, `crates/kali_types/src/tests.rs`, `crates/kali_cli/tests/runtime_smoke.rs`, `crates/kali_cli/tests/late_compat_js_input.rs`, and `crates/kali_cli/tests/late_compat_browser_js_input.rs` |

## Reading note

This dashboard is intentionally smaller than the full maturity matrix. It only tracks the ECMA/TS slice that the Phase 6 roadmap cares about today, and it stays sorted by maturity bucket so new rows can be compared without guessing at the intended status.
