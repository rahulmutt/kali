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
- Current progress: added `.js` fixture coverage for directory-index dynamic import resolution in `crates/kali_types/src/tests.rs`, plus matching build-discovery coverage for `.js` directory-index chunks in `crates/kali_cli/src/build_tests.rs`.

### 6.3 Module and dynamic-loading semantics

- Literal-string `import()` over the linked graph is implemented in the current snapshot; keep the one-linked-payload rule and preserve the linked-graph lowering path.
- Directory-style linked targets now resolve through `index.*` entries so `import("./dir")` can lower the same way as `import("./dir/index.ts")` when the directory is part of the linked graph.
- Preserve the one-linked-payload rule.
- Keep non-literal `import(expr)` on the later compatibility path and reject it with the canonical `E5506` gate.

### 6.4 Runtime semantic regressions

- Add minimized tests for exceptions, async/await, iterators/generators, built-ins, CJS/ESM interop, and object semantics.
- Ensure unsupported dynamic features produce canonical availability diagnostics rather than silent placeholders.
- Current progress: added regression coverage for arithmetic precedence, array literal length handling, function call return semantics, relational comparison semantics, try/catch exception semantics, BigInt addition semantics, `Object.keys()` object-enumeration semantics, and `Math.max()` built-in lowering in `crates/kali_cli/tests/runtime_smoke.rs`.
- Follow-up: async/await and iterator/generator lowering still need a dedicated implementation packet; the current snapshot does not yet cover those semantics.

## Exit gate

- Conformance dashboard exists and is deterministic.
- New supported semantics have tests and schema/diagnostic coverage where relevant.
- Maturity rows are updated only for surfaces with evidence.
