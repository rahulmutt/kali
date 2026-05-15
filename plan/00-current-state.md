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
- Build/artifact lanes: executable builds, browser bundles, IR validation, library artifacts, C ABI artifacts, component artifacts, deterministic metadata, and sidecar manifests.
- Runtime/reporting lanes: `run`, `test`, `test --coverage`, source-graph effects, package effects, package audit, deterministic JSON envelopes, schema-v1 payload validation, bracketed `Object["keys"]` / `globalThis["Object"]["entries"]` object-enumeration lowering coverage, browser-harness sequence-wrapper coverage over the supported object-enumeration calls including the single-quoted bracketed `Object` spellings, direct `Reflect.ownKeys(...)` iterator lowering over static object-literal slices and its sequence-expression wrapper slice in direct/boundary smoke, including `Object.freeze`-wrapped frozen-object variants and the single-quoted bracketed `Reflect` alias in the browser-harness smoke path, plus browser-requested run/test smoke for the frozen-object `Reflect.ownKeys(...)` slice in JS, TS, JSX, and TSX input with JSON-output coverage, the current string-concatenation iterable slice for `for...of` / `for await...of` over statically known string operands, plus dedicated browser-bundle smoke for that same string-concatenation slice in JS, TS, JSX, and TSX input and browser-requested run/test browser-harness smoke for the same string-concatenation slice in JS, TS, JSX, and TSX input with JSON-output coverage, plus browser-bundle/browser-harness smoke for the `for...of` template-literal slice over statically known string operands in JS and TS input, matching standalone TS `for await...of` string-concatenation smoke, and the existing `for await...of` string-concatenation smoke, the `Object.is` slice for static primitive values, same-reference object/array aliases, `Object.freeze`-wrapped same-reference aliases, distinct fresh object/array literal comparisons, and the BigInt literal comparison slice in standalone JS and browser-harness TSX runtime smoke, and the static `Number.isFinite` / `Number.isNaN` / `Number.isInteger` / `Number.isSafeInteger` primitive-value slice. Type-resolution regression coverage now also explicitly exercises the shared `Object.is` and `Number.is*` alias spellings that feed those slices, including the parser-backed bracketed `Object.is` and `Number.is*` forms used by the browser/runtime smoke paths, and browser-requested run/test browser-harness smoke now also covers the same-reference alias-chain and frozen same-reference alias-chain `Object.is` slices in JS, TS, JSX, and TSX input with JSON-output coverage, plus dedicated browser-harness alias-chain smoke for the same `Object.is` object/array and frozen-object/frozen-array variants in JS, TS, JSX, and TSX input, plus standalone `run` / `test` smoke for the Deno filesystem slice now also covering JSX and TSX input.
- Browser-requested sequence-wrapped template-literal dynamic import smoke now also covers TS, JSX, and TSX input with JSON-output coverage.
- Browser iterator rejection coverage now also includes non-literal `Object.entries(...)` sources in the browser-targeted `check` / `build --bundle` lanes and their inherited browser-config forms.
- Try/finally sequencing smoke now also covers the supported JS-input `for...of` / `for await...of` array-iteration slices, including the browser-harness JSON-output path.
- The number-predicate smoke now also exercises bracketed and mixed `globalThis["Number"]["isFinite"]` / `globalThis["Number"]["isInteger"]` / `globalThis.Number["isNaN"]` / `globalThis["Number"].isNaN` / `globalThis["Number"].isSafeInteger` spellings in the same supported slice, the static `Number.isSafeInteger` slice now joins the existing `Number.isFinite` / `Number.isNaN` / `Number.isInteger` coverage, standalone `run` / `test` smoke now also covers that same primitive-value slice in JS and TS input with JSON-output coverage, browser-harness smoke now also covers that same primitive-value slice in JSX and TSX input, and browser-bundle smoke now also covers that same primitive-value slice in JS, TS, JSX, and TSX input.
- Promise.allSettled smoke now also covers standalone build/runtime and browser-requested browser-harness paths across JS, TS, JSX, and TSX input.
- Math bundle/harness smoke now also covers the fully bracketed `globalThis["Math"]["sqrt"]` / `globalThis["Math"]["cbrt"]` spellings in JS, TS, JSX, and TSX input.
- Frontend metadata lanes: class generator methods now preserve async/generator flags in the parser, AST, HIR, MIR, LIR, and codegen export-analysis prechecks, with direct MIR/LIR regression coverage now pinning that propagation while later phases still gate lowering with the canonical generator diagnostic. Async class methods now resolve through the shared async-function lowering path; generator class methods remain gated, build smoke now also exercises async class methods across TS, JS, JSX, and TSX input on both the Deno and browser artifact paths, and direct `run` / `test` now carry an explicit E5506 gate for the async-class-method runtime slice pending the runtime follow-up. Generator codegen now also emits distinct generator vs async-generator gate wording while preserving the same canonical E5506 path. The direct runtime-entrypoint rejection helper now also catches async class expressions, generator class expressions, and parenthesized export-default class expressions in JS/TS input, plus generator class expressions in JSX/TSX input and async-generator default-export class expressions in JS input, and the library-export collection path now also rejects generator and async-generator declarations/expressions through the same canonical E5506 gate instead of treating them as statically known exports. The parser now also canonicalizes transparent wrapper forms around computed object-property keys when they still resolve to static string/number names.
- Host/API slices: default standalone Deno-oriented APIs, browser-targeted `check` / `build --bundle`, configured browser-harness execution, and documented Node-compatible analysis/build/runtime slices.
- Package evidence: registry/raw-URL/install flows, package-corpus probes by context/rung, registry-analysis commands, and negative evidence for unsupported native/binary/bootstrap-heavy or published-bin-entrypoint cases. The corpus now also rejects native-addon entrypoints on the default standalone source-graph commands in JS input, keeping the excluded package contract explicit.
- Filesystem host APIs: `Deno.open` / `create` / `mkdir` / `remove` / `rename` / `lstat` now also have standalone `check` / `build` / `run` / `test` smoke coverage in JS, TS, JSX, and TSX input with JSON-output coverage on the executable lanes.
- Embedding/evidence surfaces: public embedding artifacts and bindings metadata, optimization modes, deterministic PGO profile validation, benchmark fixtures, and Lean proof infrastructure for the published boundary.
- CLI/config diagnostics: malformed `compat.features` and `compilerOptions.runtimeProfiles` manifest entries now carry structured config-context metadata in JSON diagnostics.

## Planning consequence

The active plan starts from remaining spec gaps only. It does not preserve line-by-line progress notes for implemented slices; those belong in tests, specs, maturity current-state notes, and proof-boundary documents.

## Non-negotiable guardrails

Future phases must preserve:

- AOT-only guest-language compilation.
- Pure Rust implementation contract.
- No tracing/background GC.
- Sandbox-first honesty.
- Deterministic machine contracts.
- Public availability discipline from `specs/19-feature-maturity.md`.
- Proof-backed claim discipline from `proofs/BOUNDARY.md`.
