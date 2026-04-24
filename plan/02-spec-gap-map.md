# Spec Gap Map

This map lists active implementation goals implied by the specs after the current repository baseline. It is not an availability matrix; use `specs/19-feature-maturity.md` for public support status.

## Semantics and frontend depth

Owners: `specs/02-lexer-parser.md`, `specs/03-ast.md`, `specs/04-type-system.md`, `specs/05-ir.md`, `specs/10-runtime.md`, `specs/16-testing.md`.

Remaining goals:

- Expand ECMA-262 conformance for supported syntax and runtime semantics; async/await sequencing now has a dedicated runtime-smoke regression, array iteration now has a minimized gated `for...of` rejection fixture plus a mirrored `.js` input-class rejection fixture, minimized mixed CommonJS/ESM package interop fixtures now cover the default standalone surface — including an exports-map variant — and `.js` base-library artifact coverage now exercises the first-class JavaScript build lane; the default standalone `test` corpus now also has a minimized mixed-format interop fixture, browser-targeted `.js` entrypoints now have a minimized mixed CommonJS/ESM interop fixture too, and generator-lowering gate coverage now also includes minimized `.js` input fixtures on the `check`/`build` smoke lanes plus mirrored `run`/`test` `.js` generator-gate fixtures, including mirrored async-generator `.js` fixtures, while non-literal dynamic import rejection now also has mirrored `.js` coverage on the `check` / `build` lanes. The read-only `Deno.permissions.query(...)` subset now has runtime-smoke coverage for all four documented descriptor names (`read`, `write`, `env`, `net`) across both direct and computed JS input forms, and the `check` lane now also carries a JS-input regression for the same supported subset. Browser bundle runtime smoke now also exercises the dynamic-import loader for `.js` input, including directory-index targets, and the runtime-smoke regression set now also mirrors BigInt addition into `.js` run/test coverage, keeping the linked-graph browser bundle path mirrored across JS and TS lanes.
- Strengthen TypeScript/JavaScript inference within the bounded inference contract. Current progress: unresolved named exports at a module public boundary now fail with the canonical undefined-identifier diagnostic in `crates/kali_types/src/tests.rs`, including mirrored `.js` input coverage for the same public-boundary rejection, and the phase-3-budgeted cross-module inference fixtures now have mirrored `.js` coverage for both the direct call-chain and the explicit specialization-cap variants in `crates/kali_cli/tests/runtime_smoke.rs`.
- Keep open-ended cross-module/public-API inference gated until a later solver/evidence lane exists.
- Implement literal-string `import()` over the already-linked graph when ready; keep non-literal dynamic import gated.
- Improve minimized regression fixtures for supported dynamic/object/runtime behavior. Current progress: console error/warn/info/debug routing now also has mirrored `console.assert()` false-branch reporting coverage in `crates/kali_cli/tests/runtime_smoke.rs`, and the runtime-smoke regression set now also mirrors BigInt addition into `.js` run/test coverage, keeping the supported console baseline aligned with first-class JavaScript compilation.
- Generator lowering still needs a dedicated implementation packet; the parser now accepts async function declarations/expressions, async generator syntax, and generator syntax, including delegated `yield*`, but lowering remains gated at resolution/check time. Current runtime smoke coverage now includes both sync and async generator gate coverage across `check`, `build`, `run`, and `test`, including async generator function-expression coverage plus mirrored `.js` generator-gate fixtures on the `check`/`build` and `run`/`test` lanes, while other regression cases remain limited to simpler examples such as arithmetic, exceptions (including try/finally sequencing), object enumeration, and built-ins. The browser-targeted `check` / `build --bundle` smoke lane now also pins the same generator-lowering rejection in the browser analysis/build context, including mirrored `.js` input fixtures on both browser lanes. The remaining gap is mostly about keeping the minimized gate fixtures mirrored across TS and JS input classes rather than widening the supported surface.
- `Object.values()` is now covered alongside the already-covered `Object.keys()` / `Object.entries()` pair, and the runtime-smoke regression set now also checks overwrite ordering and integer-like key ordering for that trio, including mirrored `.js` input coverage; keep the object-enumeration smoke regressions in sync while other object-model gaps are pursued.
- `Math.sign()` now rounds out the existing `Math.max()` / `Math.min()` / `Math.abs()` built-in math coverage on the runtime-smoke path; keep the four-function built-in regression set in sync with the corresponding codegen import tests, including mirrored `.js` input coverage.

## Runtime, host, and platform expansion

Owners: `specs/09-sandboxing.md`, `specs/10-runtime.md`, `specs/11-standard-apis.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Extend the threaded runtime profile beyond the supported `run`/`test` execution paths into the remaining analysis/build contexts; guest-facing thread-spawn host import plumbing is now in place, browser-targeted `effects` and inherited-browser-context `package-effects` now have regression coverage for the remaining `--wasm-threads` gates, including a human-output regression for the browser-context `package-effects` rejection, and browser-build smoke coverage now pins the remaining browser-targeted rejection paths in both text and JSON output modes, while fuller lowering / multi-worker execution semantics still need follow-up work.
- Standalone `run --api browser` / `test --api browser` only after a real browser-runtime contract exists; the current browser-harness evidence now also covers inherited browser `apiSurface` configs for both commands when that harness override is present, including browser package-resolution coverage on the inherited path and mirrored `.js` package-resolution smoke for the browser harness path.
- Late process/working-directory APIs after policy/effect/schema contracts are explicit.
- Weak references, `Proxy`, `FinalizationRegistry`, broader `Intl`, and other late object-model surfaces only with conformance evidence.
- Alternative runtime backends only if they preserve visible contracts.

## Ecosystem and packages

Owners: `specs/11-standard-apis.md`, `specs/12-cli.md`, `specs/14-packages.md`, `specs/16-testing.md`, `specs/18-schemas.md`, `specs/19-feature-maturity.md`.

Remaining goals:

- Broader pure JS/TS npm/JSR package compatibility by support rung.
- Larger package corpus covering Deno, browser-targeted, and Node contexts separately.
- More Node built-ins and package-resolution cases when evidence justifies them.
- Current progress: browser-targeted package corpus coverage now also exercises `.js` entrypoints for browser replacement-map packages, including a scoped package case and a `vue/runtime-dom` browser replacement-map JS-entrypoint case, closing one of the first-class-JavaScript evidence gaps in the browser surface.
- Current progress: the browser package corpus now also exercises the canonical pure-JS `semver` probe on `.js` input, so the browser package-support evidence now spans the browser-targeted first-class-JavaScript path instead of only the TypeScript lane.
- Current progress: the default standalone utility corpus now also mirrors its minimized mixed-format CommonJS/ESM interop fixture on `.js` input for `run`, `test`, and `build`, keeping first-class JavaScript package evidence aligned with the existing TypeScript lane instead of only covering the TS form.
- Current progress: browser bundle chunk smoke coverage now also exercises literal dynamic imports from `.js` input, keeping the linked-graph lowering path aligned with first-class JavaScript compilation on the browser build lane.
- Current progress: the runtime-smoke regression set now also mirrors arithmetic precedence and array literal length handling into `.js` input, alongside async/await sequencing on both the `run` and `test` paths, keeping those core supported semantics aligned with first-class JavaScript compilation evidence.
- Current progress: `effects` JSON output now also exercises combined inherited `compat.features` + `compilerOptions.runtimeProfiles` normalization in one regression, keeping the mixed-axis analysis-context contract deterministic across schema-v1 payloads.
- Current progress: Node package corpus coverage now also exercises the documented Node build surface for the same `node:`-based package set that already has Node `check` / `run` evidence, keeping package compatibility evidence aligned across analysis, build, and execution lanes.
- Current progress: the Node package corpus now also exercises the canonical pure-JS `semver` probe on `.js` input across the Node `check` / `build` / `run` lanes, closing a first-class-JavaScript evidence gap on the Node surface instead of only the TypeScript form.
- Current progress: Deno package corpus coverage now also exercises the documented Deno build surface for the same Deno-host package set that already has Deno `check` / `run` evidence, keeping package compatibility evidence aligned across analysis, build, and execution lanes.
- Current progress: Deno package corpus coverage now also exercises a canonical `jsr:@std/path` package fixture materialized at `node_modules/@std/path` on the Deno surface, keeping the `jsr:` registry prefix and on-disk path mapping honest in the package-resolution evidence.
- Current progress: registry-analysis command-shape coverage now also rejects explicit package-version targets for `package-effects` and `package-audit` in both text and JSON output modes, keeping the single-package target contract aligned with the documented `E5508` path.
- Current progress: `package-audit` now also has pretty-JSON envelope coverage while inheriting a browser analysis context, keeping the envelope-only JSON command honest across formatting mode and inherited context at the same time.
- Batch or richer registry-analysis workflows only after command/schema revisions.
- Keep native/binary/bootstrap-heavy packages rejected by default unless the specs deliberately change.

## Optimization and PGO

Owners: `specs/07-specialization.md`, `specs/08-wasm-codegen.md`, `specs/16-testing.md`, `specs/18-schemas.md`.

Remaining goals:

- Deterministic build-only PGO input (`--profile`) with strict schema validation if not fully promoted; CLI integration now covers version and unknown-field rejection in text and JSON build modes.
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
