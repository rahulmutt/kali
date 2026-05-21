# Current Implementation Baseline

This file records the planning baseline for the active continuation roadmap. It is a current-state summary, not an active checklist and not an availability matrix.

## Evidence used for this cleanup

- `cargo run -q -p kali_cli --bin kali -- --help` exposes the current public command set.
- The workspace contains Rust crates for CLI, lexer/parser, AST/HIR/MIR/LIR, type checking, codegen, runtime, sandbox/effects, package management, optimization, Deno/Web/Node API projections, embedding/C ABI, formatting, linting, and bindings.
- The spec-owned availability and current-state nuance remain in [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- The proof-backed boundary remains owned by [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).

## Live surface at a glance

The checked-in repository already includes:

- CLI commands: `doctor`, `init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`, `effects`, `package-effects`, and `package-audit`.
- Static-literal analysis now also unwraps await-wrapped numeric literals in the bounded inference path, keeping cheap wrapper handling consistent across the checker.
- Static reference-root resolution now also tolerates optional-chain-wrapped roots, await-wrapped static member roots, and direct same-reference member-root comparisons, keeping `Object.is` same-reference comparisons aligned with the other transparent wrapper slices.
- Transparent object-root resolution now also unwraps `Object.freeze(...)` around static `Math` and `Number` roots, keeping the cheap math/number identity slices aligned with the existing callable-wrapper handling.
- The static `Object.is` / `Number.is*` primitive-value slices now also treat `Object.freeze(...)` around statically-known primitive arguments as a transparent wrapper in the checker and smoke coverage.
- Late object-model gating now also unwraps await-wrapped and `Object.freeze`-wrapped `Proxy.revocable` aliases before emitting the canonical E5506 rejection, so transparent wrappers do not bypass the existing late-compatibility boundary; the browser JSX/TSX late-compat smoke now also mirrors that await-wrapped `Proxy.revocable` rejection slice.
- The late-object-model test matrix now also pins frozen `Proxy.revocable` aliases in the type checker, and the shared browser late-compat helper inventory now also asserts the frozen bracketed `Proxy.revocable` aliases, keeping the rejection path explicit for wrapper-preserving diagnostics.
- The supported numeric-fallthrough path now also keeps await-wrapped `Math.atan2` numeric literals cheap in build smoke, matching the existing transparent wrapper handling on related static-literal slices, and browser-harness/browser-bundle smoke now also covers the same await-wrapped `Math.atan2` zero slice across JS, TS, JSX, and TSX input.
- The supported Number predicate slice now also accepts frozen callable aliases such as `Object.freeze(Number.isFinite)`, `Object.freeze(Number.isInteger)`, and `Object.freeze(Number.isSafeInteger)` in the checker plus runtime and browser harness/bundle smoke coverage.
- The supported `Math.round` literal slice now also treats transparent `Object.freeze(...)` wrappers around statically-known numeric literals as part of the same cheap folding path across the checker, runtime, and browser harness smoke coverage.
- Default-export async-generator function declarations now preserve function-flavor metadata through the parser, AST, HIR, MIR, and LIR layers, while executable lowering and library-export collection still reject the unsupported lowering path with the canonical gate.
- The benchmark inventory notes now record that the new `math-round-builtin` / `math-round-builtin-js` pair now does the same for `Math.round`, that browser-harness/browser-bundle smoke now also covers the dot-root `Object.freeze(globalThis.Math.round)` callable alias, the bracketed `Object.freeze(globalThis.Math["round"])` / `Object.freeze(globalThis["Math"]["round"])` aliases, and the direct `Object.freeze(Math.round)` / `Object.freeze((Math.round))` root aliases on that supported slice, that the frozen-callable `Math.abs` / `Math.sign` slice now also has direct `run` / JSON `run` smoke coverage in JS input plus browser bundle/harness coverage for `Object.freeze(Math.abs)` / `Object.freeze(Math.sign)`, and that the `folded-arithmetic-variant` slice now also has a JS workload form (`folded-arithmetic-variant-js`).
- Build/artifact lanes: executable builds, browser bundles, IR validation, library artifacts, C ABI artifacts, component artifacts, deterministic metadata, and sidecar manifests.
- Codegen function-plan evidence now also covers default-export async-generator declarations, keeping generator flavor metadata visible through the lowered function-plan path even while executable lowering remains gated.
- Runtime/reporting lanes: `run`, `test`, `test --coverage`, source-graph effects, package effects, package audit, deterministic JSON envelopes, schema-v1 payload validation, and deterministic thread-topology schema drift checks for the run/test JSON paths.
- The runtime BigInt arithmetic path now also accepts `%` remainder on the same supported smoke lanes as addition, subtraction, multiplication, division, and exponentiation.
- Object-enumeration finalization coverage now also includes async `Object.entries(...)` alongside the existing `Object.keys(...)` / `Object.values(...)` slices in the runtime and browser harness smoke lanes.
- The static object-helper wrapper slice now also accepts sequence-expression wrappers around `Object.hasOwn(...)`, `Object.keys(...)`, and `Reflect.ownKeys(...)` targets in the checker and browser/standalone smoke coverage.
- The `Object.is` same-reference smoke now also covers await-wrapped and optional-chain-wrapped static member roots in standalone and browser-requested harness/bundle paths, keeping the transparent wrapper slice aligned with the checker's current static-reference resolution.
- The array-iteration smoke now also accepts `Array.from(new Set(...))` and `Array.from(new Map(...))` wrappers over the same supported iterable path in JS and TS input, and the browser harness/bundle smoke lanes now mirror that same Array.from set/map slice too, keeping the existing Set/Map collectors visible through the Array.from alias path too. The same supported path now also accepts frozen constructor-result wrappers such as `Object.freeze(new Set(...))` and `Object.freeze(new Map(...))`, and browser-harness/browser-bundle smoke now also covers that frozen constructor-result slice in JS, TS, JSX, and TSX input, matching the browser-requested run/test harness coverage; the supported Array.from set/map slice now also has explicit break/continue smoke on the standalone and browser-requested harness paths, keeping the abrupt-completion coverage aligned with the rest of the iterator smoke. Frozen object-helper iterable slices such as `Object.freeze(Object.keys(...))` / `Object.freeze(Object.entries(...))` now also carry through the checker and smoke lanes; the build smoke matrix now also exercises the frozen `Set`/`Map` constructor-result wrappers in Deno and browser JS/TS input.
- Default-export async-generator function declarations now have explicit parser, AST, HIR, MIR, LIR, codegen, and library-export gate coverage in the current test matrix, keeping the declaration-specific generator rejection path pinned while resumable lowering remains gated.
- Host/API lanes: default standalone, browser-targeted check/build/bundle flows, browser harness execution paths, documented Deno and Node API slices, resource budget validation, and late-host/object-model gating.
- Package lanes: install/lock/materialization, registry and raw-URL workflows, package-shape rejection, package-corpus probes, and single-package registry-analysis commands.
- Verification lanes: Lean proof project and published proof-backed boundary limited by `proofs/BOUNDARY.md`.

## Planning consequence

The active plan should not list already-implemented command families, artifact families, smoke matrices, alias inventories, or diagnostic regressions as open work. Future tasks should be stated as remaining capability goals and should point to specs/tests for details.
