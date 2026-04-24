# Phase 6 — Semantic Conformance and Frontend Depth

## Goal

Move from broad working coverage to measurable language conformance for the surfaces Kali already claims or intends to claim next.

## Owning specs

- `specs/02-lexer-parser.md`
- `specs/03-ast.md`
- `specs/04-type-system.md`
- `specs/05-ir.md`
- `specs/10-runtime.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

## Work packets

### 6.1 Conformance inventory

- Status: delivered in the current repository snapshot as [`conformance-dashboard.md`](./conformance-dashboard.md).
- The dashboard groups supported, gated, and rejected ECMA/TS features and ties each row to tests or canonical diagnostic gates.
- Keep draft TC39 proposals rejected by default unless a specific opt-in is specified.

### 6.2 Checker and inference hardening

- Expand JS inference fixtures within the bounded inference contract.
- Add negative tests for annotation-required boundaries.
- Avoid open-ended cross-module/public-API solver claims until a later evidence-backed packet exists.
- Current progress: added `.js` fixture coverage for directory-index and parenthesized const-bound dynamic import resolution in `crates/kali_types/src/tests.rs`, plus matching build-discovery coverage for `.js` directory-index and parenthesized const-bound chunks in `crates/kali_cli/src/build_tests.rs`, and a `.js` base-library artifact smoke test in `crates/kali_cli/tests/runtime_smoke.rs`.
- Current progress: unresolved named exports at a module public boundary now fail with the canonical undefined-identifier diagnostic in `crates/kali_types/src/tests.rs`, so the annotation-required/public-boundary negative coverage called out by the phase plan is no longer just prose. The same public-boundary rejection is now also mirrored in `.js` input coverage.
- Current progress: the phase-3-budgeted cross-module inference fixtures now include mirrored `.js` cases for both the direct call-chain and the explicit specialization-cap variants in `crates/kali_cli/tests/runtime_smoke.rs`, keeping first-class JavaScript compilation covered by the same bounded-inference expectations as the TypeScript lanes.
- Current progress: parser now accepts async function declarations/expressions and async generator syntax as AST forms, plus generator function syntax and `yield` / `yield*` expressions, while the checker and CLI smoke tests still gate generator lowering with the canonical `E5506` availability path instead of letting it reach codegen; that gate is now covered on `check`, `build`, `run`, and `test` smoke paths, including delegated yield syntax, async generator syntax, async generator function-expression syntax, and minimized `.js` input coverage across the `check`/`build` lanes plus matching `run`/`test` `.js` generator-gate fixtures, now including mirrored async-generator `.js` fixtures too, and the browser-targeted `check` / `build --bundle` smoke lane now also pins the same generator-lowering rejection in the browser analysis/build context, including mirrored `.js` input fixtures on both browser lanes.
- Current progress: read-only `Deno.permissions.query(...)` now accepts the documented descriptor subset (`read`, `write`, `env`, `net`) while rejecting unsupported descriptor kinds with the canonical `E5506` availability path, and runtime-smoke coverage now exercises the full documented subset in both direct and computed JS input forms plus a `check`-lane JS-input regression for the same subset, keeping the Deno permission facade aligned with the Phase-1 schema-v1 contract instead of silently treating unknown descriptors as denied.
- Current progress: the phase-6 conformance dashboard now splits parser-supported generator syntax from the later generator-lowering gate so the supported/gated buckets mirror the actual current snapshot more precisely, and the console-routing regressions now include mirrored `.js` input coverage alongside the existing assertion checks.

### 6.3 Module and dynamic-loading semantics

- Literal-string `import()` over the linked graph is implemented in the current snapshot; keep the one-linked-payload rule and preserve the linked-graph lowering path.
- Directory-style linked targets now resolve through `index.*` entries so `import("./dir")` can lower the same way as `import("./dir/index.ts")` when the directory is part of the linked graph.
- Current progress: added minimized package-corpus regressions for mixed CommonJS/ESM default-import interop on the default standalone surface, including an exports-map variant that keeps the interop path honest under the package resolver, and the same utility corpus now also exercises `build` alongside `run`/`test` for the TS and `.js` interop fixtures.
- Current progress: browser-targeted package-corpus regressions now also cover `.js` entrypoints for browser replacement-map packages, including a scoped package case, and now add a minimized mixed CommonJS/ESM browser interop fixture too, so first-class JavaScript compilation stays exercised in the browser analysis/build lane without dropping the mixed-format resolver path.
- Current progress: browser bundle chunk smoke coverage now also exercises literal dynamic imports from `.js` input, matching the TypeScript bundle chunk regression and keeping the linked-graph lowering path honest for first-class JavaScript compilation.
- Current progress: browser bundle runtime smoke coverage now also exercises the dynamic-import loader for `.js` input, including directory-index targets, so the browser bundle path keeps its first-class JavaScript dynamic-loading behavior mirrored across the JS and TS lanes.
- Current progress: added `.js` negative coverage for directory dynamic imports without an `index.*` target in `crates/kali_types/src/tests.rs`, and mirrored the non-literal dynamic import rejection onto the `check` / `build` JS input lane in `crates/kali_cli/tests/runtime_smoke.rs`.
- Preserve the one-linked-payload rule.
- Keep non-literal `import(expr)` on the later compatibility path and reject it with the canonical `E5506` gate.

### 6.4 Runtime semantic regressions

- Add minimized tests for exceptions, async/await, iterators/generators, built-ins, CJS/ESM interop, and object semantics.
- Current progress: `try/finally` sequencing now has a dedicated runtime-smoke regression alongside the existing try/catch case.
- Ensure unsupported dynamic features produce canonical availability diagnostics rather than silent placeholders.
- Current progress: added regression coverage for arithmetic precedence and array literal length handling, including mirrored `.js` input coverage for both semantics, plus function call return semantics, async/await sequencing, including mirrored `.js` input coverage on both the `run` and `test` paths, relational comparison semantics, try/catch exception semantics, try/finally sequencing, BigInt addition semantics, including mirrored `.js` run/test coverage, a gated `for...of` array-iteration rejection fixture plus a mirrored `.js` input-class rejection fixture, generator and async generator lowering gates, including mirrored `.js` generator-gate fixtures for `check`/`build` and `run`/`test`, `Object.keys()` / `Object.entries()` / `Object.values()` object-enumeration semantics including overwrite ordering and integer-like key ordering, plus mirrored `.js` coverage for both overwrite-ordering and integer-like-key-ordering object-enumeration cases on both the `run` and `test` paths, `Math.max()` / `Math.min()` / `Math.abs()` / `Math.sign()` built-in lowering with mirrored `.js` input coverage, and `console.error()` / `console.warn()` / `console.info()` / `console.debug()` routing plus `console.assert()` false-branch reporting, including a mirrored `.js` console-level routing regression, in `crates/kali_cli/tests/runtime_smoke.rs`.
- Current progress: the default standalone package corpus now also has a minimized mixed-format interop fixture for `test`, complementing the existing `run` coverage for the same package shape, and the `.js` interop fixture now also covers `build`.
- Follow-up: generator lowering still needs a dedicated implementation packet for real lowering; the current snapshot keeps that surface gated at resolution/check time rather than miscompiling it, and the remaining coverage work is about keeping the minimized gate fixtures aligned across input classes.
- Follow-up: object-enumeration coverage should keep the overwrite-ordering and integer-like-ordering regressions mirrored across TS and JS input classes and across the `run`/`test` paths while later object-model gaps are pursued.

## Exit gate

- Conformance dashboard exists and is deterministic.
- New supported semantics have tests and schema/diagnostic coverage where relevant.
- Maturity rows are updated only for surfaces with evidence.
