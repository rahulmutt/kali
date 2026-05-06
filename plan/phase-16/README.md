# Phase 16 — Semantic Closure and Conformance Promotion

## Goal

Close remaining language gaps by either implementing faithful semantics with evidence or preserving explicit, tested availability gates.

## Owning specs

- `specs/02-lexer-parser.md`
- `specs/03-ast.md`
- `specs/04-type-system.md`
- `specs/05-ir.md`
- `specs/10-runtime.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

## Work packets

### 16.1 Generators and iterator protocol

- Generators need a dedicated resumable-iterator representation before faithful lowering can land; the current HIR path does not yet preserve enough function-kind metadata for that work.
- Split the generator packet into prerequisite plumbing, then state-machine lowering, then async-generator/finalization coverage.
- Expand `for...of` and `for await...of` toward full iterator/async-iterator protocol semantics beyond bounded static slices, including spread of supported `Object.keys(...)` / `Object.values(...)` / `Object.entries(...)` slices where that remains shape-safe.
- Keep unavailable forms on canonical `E5506` gates with TS/JS/JSX/TSX and JSON-output regressions where applicable.

#### 16.1a Generator prerequisites

- Preserve generator/async-generator kind metadata through the lowering pipeline.
- Add resumable iterator/state-machine plumbing needed for `yield`, `yield*`, async interaction, and finalization.

#### 16.1b Generator lowering

- Implement generator and async-generator lowering only after the prerequisite plumbing lands, with state-machine, `yield` / `yield*`, async interaction, error, and finalization coverage.

### 16.2 Expression, built-in, and object semantics

- Promote additional operators, compound assignment targets, dynamic language forms, and built-ins only with checker/codegen/runtime evidence.
- Keep unsupported `Math`, object-model, dynamic import, `eval`-adjacent, and reflective forms explicitly gated.
- Verify observable JavaScript semantics before exposing optimization-sensitive folds.

### 16.3 TypeScript and JavaScript inference

- Grow inference inside the bounded-inference contract only.
- Require annotations or conservative `unknown` fallbacks at public/exported or cross-module boundaries when budgets are exceeded.
- Mirror accepted and rejected cases in checker baselines and CLI JSON diagnostics.

### 16.4 Conformance evidence hygiene

- Maintain minimized conformance fixtures for supported and gated behavior.
- Keep dashboards and snapshots concise; do not use plan files as progress logs.

## Exit gate

- Each promoted semantic slice has parser/checker/lowering/runtime evidence.
- Unsupported but recognized forms fail through the canonical diagnostic path.
- Maturity and current-state docs are updated only for evidence-backed availability changes.
