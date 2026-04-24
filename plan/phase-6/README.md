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
- Current progress: added `.js` fixture coverage for directory-index and parenthesized const-bound dynamic import resolution in `crates/kali_types/src/tests.rs`, plus matching build-discovery coverage for `.js` directory-index and parenthesized const-bound chunks in `crates/kali_cli/src/build_tests.rs`.
- Current progress: parser now accepts async function declarations/expressions and async generator syntax as AST forms, plus generator function syntax and `yield` / `yield*` expressions, while the checker and CLI smoke tests still gate generator lowering with the canonical `E5506` availability path instead of letting it reach codegen; that gate is now covered on `check`, `build`, `run`, and `test` smoke paths, including delegated yield syntax and async generator syntax.

### 6.3 Module and dynamic-loading semantics

- Literal-string `import()` over the linked graph is implemented in the current snapshot; keep the one-linked-payload rule and preserve the linked-graph lowering path.
- Directory-style linked targets now resolve through `index.*` entries so `import("./dir")` can lower the same way as `import("./dir/index.ts")` when the directory is part of the linked graph.
- Current progress: added minimized package-corpus regressions for mixed CommonJS/ESM default-import interop on the default standalone surface, including an exports-map variant that keeps the interop path honest under the package resolver.
- Current progress: browser-targeted package-corpus regressions now also cover `.js` entrypoints for browser replacement-map packages, so first-class JavaScript compilation stays exercised in the browser analysis/build lane too.
- Current progress: added `.js` negative coverage for directory dynamic imports without an `index.*` target in `crates/kali_types/src/tests.rs`.
- Preserve the one-linked-payload rule.
- Keep non-literal `import(expr)` on the later compatibility path and reject it with the canonical `E5506` gate.

### 6.4 Runtime semantic regressions

- Add minimized tests for exceptions, async/await, iterators/generators, built-ins, CJS/ESM interop, and object semantics.
- Current progress: `try/finally` sequencing now has a dedicated runtime-smoke regression alongside the existing try/catch case.
- Ensure unsupported dynamic features produce canonical availability diagnostics rather than silent placeholders.
- Current progress: added regression coverage for arithmetic precedence, array literal length handling, function call return semantics, async/await sequencing, relational comparison semantics, try/catch exception semantics, try/finally sequencing, BigInt addition semantics, a gated `for...of` array-iteration rejection fixture, generator and async generator lowering gates, `Object.keys()` / `Object.entries()` / `Object.values()` object-enumeration semantics, `Math.max()` / `Math.min()` built-in lowering, and `console.error()` / `console.warn()` / `console.info()` / `console.debug()` routing in `crates/kali_cli/tests/runtime_smoke.rs`.
- Follow-up: generator lowering still needs a dedicated implementation packet for real lowering; the current snapshot keeps that surface gated at resolution/check time rather than miscompiling it.

## Exit gate

- Conformance dashboard exists and is deterministic.
- New supported semantics have tests and schema/diagnostic coverage where relevant.
- Maturity rows are updated only for surfaces with evidence.
