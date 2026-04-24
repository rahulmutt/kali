# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use `specs/19-feature-maturity.md` for public support status.

## Semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Expand ECMA-262 conformance for supported syntax and runtime semantics.
- Strengthen TypeScript/JavaScript inference within the bounded inference contract.
- Keep open-ended cross-module/public-API inference gated until a later solver/evidence lane exists.
- Implement literal-string `import()` over the already-linked graph when ready; keep non-literal dynamic import gated.
- Improve minimized regression fixtures for supported dynamic/object/runtime behavior.
- Generator lowering still needs a dedicated implementation packet; current runtime smoke coverage is limited to simpler regression cases such as arithmetic, exceptions, object enumeration, and built-ins.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Extend the threaded runtime profile beyond the supported `run`/`test` execution paths into the remaining analysis/build contexts and actual guest thread-spawn host imports.
- Standalone `run --api browser` / `test --api browser` only after a real browser-runtime contract exists.
- Late process/working-directory APIs after policy/effect/schema contracts are explicit.
- Weak references, `Proxy`, `FinalizationRegistry`, broader `Intl`, and other late object-model surfaces only with conformance evidence.
- Alternative runtime backends only if they preserve visible contracts.

## Ecosystem and packages

Owners: `specs/11-standard-apis.md`, `specs/12-cli.md`, `specs/14-packages.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Broader pure JS/TS npm/JSR package compatibility by support rung.
- Larger package corpus covering Deno, browser-targeted, and Node contexts separately.
- More Node built-ins and package-resolution cases when evidence justifies them.
- Batch or richer registry-analysis workflows only after command/schema revisions.
- Keep native/binary/bootstrap-heavy packages rejected by default unless the specs deliberately change.

## Optimization and PGO

Owners: `specs/07-specialization.md`, `specs/08-wasm-codegen.md`, `specs/16-testing.md`, `specs/18-schemas.md`.

Remaining goals:

- Deterministic build-only PGO input (`--profile`) with strict schema validation if not fully promoted.
- Deeper release and release-advanced passes: specialization, inlining, layout-aware lowering, peepholes, and LTO-like whole-graph cleanup.
- Version-pinned benchmarks before public performance claims.
- No optimization that weakens sandbox, diagnostics, or AOT-only constraints.

## Verification and contracts

Owners: `specs/16-testing.md`, `specs/17-verification.md`, `specs/18-schemas.md`, `proofs/BOUNDARY.md`.

Remaining goals:

- Widen Lean models beyond the current published boundary only with mechanized theorem inventory.
- Connect proof-trigger policy to any implementation/spec paths newly claimed as covered.
- Add schema conformance checks for every machine-readable success and failure output.
- Keep proof theorem/property inventories out of plan files; `proofs/BOUNDARY.md` remains the sole theorem/property inventory owner.
