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
- Current progress: parser now accepts async function declarations/expressions and async generator syntax as AST forms, plus generator function syntax and `yield` / `yield*` expressions, while the checker and CLI smoke tests still gate generator lowering with the canonical `E5506` availability path instead of letting it reach codegen; that gate is now covered on `check`, `build`, `run`, and `test` smoke paths, including delegated yield syntax, async generator syntax, async generator function-expression syntax, and minimized `.js` input coverage across the `check`/`build` lanes plus matching `run`/`test` `.js` generator-gate fixtures, now including mirrored async-generator `.js` fixtures too, and the browser-targeted `check` / `build --bundle` smoke lane now also pins the same generator-lowering rejection in the browser analysis/build context.
- Current progress: read-only `Deno.permissions.query(...)` now accepts the documented descriptor subset (`read`, `write`, `env`, `net`) while rejecting unsupported descriptor kinds with the canonical `E5506` availability path, keeping the Deno permission facade aligned with the Phase-1 schema-v1 contract instead of silently treating unknown descriptors as denied.
- Current progress: the phase-6 conformance dashboard now splits parser-supported generator syntax from the later generator-lowering gate so the supported/gated buckets mirror the actual current snapshot more precisely.

### 6.3 Module and dynamic-loading semantics

- Literal-string `import()` over the linked graph is implemented in the current snapshot; keep the one-linked-payload rule and preserve the linked-graph lowering path.
- Directory-style linked targets now resolve through `index.*` entries so `import("./dir")` can lower the same way as `import("./dir/index.ts")` when the directory is part of the linked graph.
- Current progress: added minimized package-corpus regressions for mixed CommonJS/ESM default-import interop on the default standalone surface, including an exports-map variant that keeps the interop path honest under the package resolver.
- Current progress: browser-targeted package-corpus regressions now also cover `.js` entrypoints for browser replacement-map packages, including a scoped package case, and now add a minimized mixed CommonJS/ESM browser interop fixture too, so first-class JavaScript compilation stays exercised in the browser analysis/build lane without dropping the mixed-format resolver path.
- Current progress: added `.js` negative coverage for directory dynamic imports without an `index.*` target in `crates/kali_types/src/tests.rs`.
- Preserve the one-linked-payload rule.
- Keep non-literal `import(expr)` on the later compatibility path and reject it with the canonical `E5506` gate.

### 6.4 Runtime semantic regressions

- Add minimized tests for exceptions, async/await, iterators/generators, built-ins, CJS/ESM interop, and object semantics.
- Current progress: `try/finally` sequencing now has a dedicated runtime-smoke regression alongside the existing try/catch case.
- Ensure unsupported dynamic features produce canonical availability diagnostics rather than silent placeholders.
- Current progress: added regression coverage for arithmetic precedence, array literal length handling, function call return semantics, async/await sequencing, relational comparison semantics, try/catch exception semantics, try/finally sequencing, BigInt addition semantics, a gated `for...of` array-iteration rejection fixture, generator and async generator lowering gates, including mirrored `.js` generator-gate fixtures for `check`/`build` and `run`/`test`, `Object.keys()` / `Object.entries()` / `Object.values()` object-enumeration semantics including overwrite ordering, `Math.max()` / `Math.min()` / `Math.abs()` built-in lowering, and `console.error()` / `console.warn()` / `console.info()` / `console.debug()` routing in `crates/kali_cli/tests/runtime_smoke.rs`.
- Current progress: the default standalone package corpus now also has a minimized mixed-format interop fixture for `test`, complementing the existing `run` coverage for the same package shape.
- Follow-up: generator lowering still needs a dedicated implementation packet for real lowering; the current snapshot keeps that surface gated at resolution/check time rather than miscompiling it, and the remaining coverage work is about keeping the minimized gate fixtures aligned across input classes.

## Exit gate

- Conformance dashboard exists and is deterministic.
- New supported semantics have tests and schema/diagnostic coverage where relevant.
- Maturity rows are updated only for surfaces with evidence.
