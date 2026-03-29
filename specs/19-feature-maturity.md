# 19 — Feature Maturity

This document is the canonical matrix for features that are easy to describe inconsistently across architecture, runtime, package, and CLI specs.

If another spec needs to mention one of these features, it should link here for phase/status rather than restating a different maturity decision.

## Status Labels

- **Phase 1 MVP** — required for the first practically useful implementation
- **Phase 2 target** — planned once ownership/effects infrastructure lands
- **Phase 3 target** — planned once specialization/ecosystem work lands
- **Phase 4 compatibility** — supported only in the advanced compatibility phase
- **Later compatibility** — intentionally deferred until semantics and cost are justified
- **Opt-in only** — supported only behind an explicit flag or config
- **Later compatibility (opt-in only)** — deferred until a later phase and, even then, enabled only behind an explicit runtime/profile switch
- **Rejected by default** — parser may accept the syntax, but compile/run should fail unless the documented compatibility switch is enabled in a phase that actually implements the feature

## Status Interpretation Table

| Status label | Meaning in practice |
|---|---|
| Phase 1 MVP | Must ship in the first dependable end-to-end release |
| Phase 2 target / Phase 3 target / Phase 4 compatibility | Planned work for that phase; before then, reject with the canonical gating path rather than partially emulating it |
| Later compatibility | Intentionally deferred with no near-phase promise |
| Opt-in only | Implemented only behind an explicit flag/config even after support exists |
| Later compatibility (opt-in only) | Both deferred and explicitly gated when it eventually lands |
| Rejected by default | Parse support may exist, but normal compile/run still rejects it unless a documented compatibility path exists and is implemented |

This table exists to keep the status labels operational: a label implies whether Kali should ship, gate, reject, or require an explicit opt-in.

## Canonical Matrix

| Feature | Status | Rationale |
|---|---|---|
| Latest published ECMA-262 lexical/parser grammar (current standard edition) | Phase 1 MVP | Front-end coverage should track the current standard grammar even when some semantics remain phase-gated |
| Static ESM `import` / `export` | Phase 1 MVP | Core module system |
| First-class JavaScript compilation with inference | Phase 1 MVP | Required so `.js` projects are not forced to migrate to TypeScript before benefiting from Kali |
| CommonJS module lowering | Phase 1 MVP | Needed for early npm package compatibility within the linked-artifact model |
| `require("literal")` | Phase 1 MVP | Rewritten during compilation when statically resolvable |
| Dynamic `require()` | Rejected by default | Conflicts with the early single-linked-artifact model |
| Broad npm compatibility for packages that expect more Node built-ins | Phase 3 target | Depends on broader `--api node` support beyond the Phase 1 package baseline |
| Literal-string `import()` | Phase 3 target | Can be lowered to the already-linked graph without runtime WASM module linking |
| Non-literal `import(expr)` | Later compatibility | Requires a dynamic host-mediated path and conservative effect handling |
| `eval` | Phase 4 compatibility | Parsed and effect-tracked earlier, but full runtime support is deferred; compatibility path is `--compat eval` when implemented |
| `Function()` constructor | Phase 4 compatibility | Same status as `eval` and uses the same compatibility switch |
| Invocation arguments (`Deno.args`; later Node `process.argv`) | Phase 1 MVP | Treated as caller-supplied execution context rather than a separately policy-gated host capability in schema v1 |
| Read-only environment access (`Deno.env.get`, `Deno.env.toObject`, policy-filtered host env view) | Phase 1 MVP | Needed for practical standalone compatibility while still fitting the sandbox model |
| Read-only `Deno.permissions` facade (`query`-style granted/denied view only) | Phase 1 MVP | Exposes Kali sandbox state for compatibility without interactive permission escalation |
| Mutable environment access (`Deno.env.set`, `process.env = ...`-style host mutation) | Phase 3 target | Widens the host contract and must remain policy-controlled |
| Subprocess spawning (`Deno.Command`, host `process_spawn`) | Phase 3 target | Requires explicit sandbox/process-budget integration |
| Socket/listener networking (`Network.Connect`, `Network.Listen`, `Deno.serve`) | Phase 3 target | Requires explicit network policy and concurrency controls |
| Process identity and process-control/working-directory APIs (`Deno.pid`, `process.pid`, `Deno.exit`, `Deno.cwd`, `Deno.chdir`) | Later compatibility | Deferred until a future schema/policy contract makes their sandbox and embedding behavior explicit |
| Built-in effect inference / `kali effects` | Phase 2 target | Required for sandbox-first analysis and policy checking |
| Explicit effect annotations / `pure` | Phase 2 target | Initially scoped to the built-in sandbox capability model |
| User-defined/custom effect kinds in stable reports or policy checking | Later compatibility | Keep Phase 1-2 machine contracts limited to built-in sandbox-relevant effects |
| Algebraic effect declarations / handlers | Later compatibility | Experimental and must not block delivery of the core capability/effect system |
| Executable project-local sandbox policy code (`kali.policy.json` hooks / inline predicates) | Rejected by default | Project policy files stay declarative data; Kali should not execute project code just to decide whether a capability is allowed |
| Host-registered sandbox policy predicates | Later compatibility | Initial policies stay declarative; a later embedding-only extension may add pure host-registered predicates without turning policy files into executable code |
| Annex B / web-legacy compatibility corners | Later compatibility | Keep the MVP focused on dependable core semantics; add legacy web behaviors only when conformance value justifies the cost |
| `Proxy` | Later compatibility | High semantic cost and optimization barriers |
| `WeakMap` / `WeakSet` | Later compatibility | Deferred until weak-reference semantics fit the no-tracing-GC design |
| `FinalizationRegistry` | Later compatibility | Same reason as weak collections |
| `SharedArrayBuffer` / `Atomics` | Later compatibility (opt-in only) | Requires a separate threaded runtime profile and should not be implied by the Phase 1 single-threaded runtime |
| `--wasm-threads` | Later compatibility (opt-in only) | Enables the threaded runtime profile once that profile exists; must fail explicitly before then and on unsupported targets/engines |
| `--api browser` for `check` / `build --bundle` | Phase 1 MVP | Browser-targeted analysis/build against the real browser ambient surface, without claiming DOM support in Kali's standalone runtime |
| `package.json#exports` condition `deno` for `--api deno` resolution | Phase 1 MVP | Aligns package resolution with the default Deno-oriented standalone API surface |
| `package.json#browser` / `exports` condition `browser` in browser bundle mode | Phase 1 MVP | Needed for practical browser-targeted npm compatibility without widening standalone runtime claims |
| `run --api browser` | Rejected by default | Early standalone runtime does not emulate a browser host |
| npm lifecycle scripts (`kali install --allow-scripts`) | Opt-in only | Disabled by default for sandbox-first behavior; enabling scripts does not make native addons supported |
| Native addons / `node-gyp` packages | Rejected by default | Violates the pure-Rust/no-native-addon constraints |
| npm packages that require unsupported Node core modules | Phase 3 target | Depends on broader `--api node` compatibility work |
| Stable public Rust embedding API | Phase 2 target | Phase 1 stays library-first internally, but the public embedding contract is stabilized later |
| Stable public C ABI / `kali build --capi` flow | Phase 2 target | Depends on the same public embedding stabilization work |
| Host ABI versioning for `kali_capi` | Phase 2 target | Stable embedding requires explicit load-time compatibility checks |
| Browser ambient DOM typings for `check --api browser` / `build --bundle --api browser` | Phase 1 MVP | Type-check against the real browser host surface for browser-targeted programs; this is not a standalone runtime promise |
| DOM APIs in standalone runtime | Rejected by default | Kali does not embed a browser engine |

## Interpretation Rules

1. **Single-payload rule**: Phase 1-3 builds target one linked WASM payload for the resolved static graph. Output modes may still add companion artifacts such as JS glue or C headers, but they must not reintroduce runtime WASM module linking.
2. **Parse vs support**: accepted syntax does not imply full runtime support; unsupported dynamic features should be diagnosed explicitly.
3. **Effect boundaries**: features marked as dynamic compatibility paths should be reflected in static effect analysis.
4. **No silent fallback**: if a feature cannot be implemented faithfully under the current phase constraints, Kali should reject or gate it rather than emulate it loosely.
5. **Policy alignment**: sandbox policy validation may always deny a capability, but it must reject policies that try to enable capabilities unavailable in the selected command/profile/phase.
6. **Canonical gating diagnostic**: use the shared feature-maturity diagnostic contract (`E5006`) so CLI, checker, runtime, and package tooling report phase/profile gating consistently.
7. **Do not overuse `E5006`**: selecting an unavailable command/profile/feature uses `E5006`, but ordinary references to names/globals that are simply absent from the selected supported ambient surface should use the normal name/type diagnostics instead.
8. **Sandbox-domain honesty**: build-time policy compatibility for browser-targeted artifacts must not be described as equivalent to Kali-hosted runtime sandbox enforcement.

## Canonical Command/Profile Matrix

This table exists to stop drift between CLI examples, runtime behavior, package tooling, and error reporting.

| Command / profile | Early-phase status | Canonical handling |
|---|---|---|
| `kali init` | Phase 1 MVP | Create the minimal canonical `kali.json` scaffold; for the default app template this should normally be just `{ "schemaVersion": 1 }` unless the chosen template needs more |
| `kali init --lib` | Phase 1 MVP | Select a library-oriented project template only; it does not implicitly change the later `kali build` artifact mode |
| `kali fmt` | Phase 1 MVP | Stable formatting command for JS/TS sources |
| `kali lint` | Phase 1 MVP | Stable lint command with conservative autofix support |
| `kali install` | Phase 1 MVP | Resolve/materialize dependency state and write `kali.lock` for the project's declared dependency source kinds; install is profile-agnostic in early phases and does not require separate per-`--api` installs |
| `kali install https://...` | Phase 1 MVP | Explicitly pin/materialize a raw URL dependency into the shared lock/materialization model |
| `kali run main.ts` | Phase 1 MVP | Compile and execute with the canonical default tuple: `apiSurface=deno`, `buildMode=fast`, `runtimeProfiles=[]`, `compat.features=[]` |
| `kali run --sandbox kali.policy.json main.ts` | Phase 1 MVP | Runtime sandbox enforcement path; policy schema/ranges must validate before execution starts |
| `kali run --api deno main.ts` | Phase 1 MVP | Supported standalone runtime path |
| `kali run --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset lands |
| `kali run --api browser main.ts` | Rejected by default | Reject with `E5006`; browser is a check/build profile first |
| `kali check main.ts` | Phase 1 MVP | Type-check with the canonical default API surface (`apiSurface=deno`) |
| `kali check types.d.ts` | Phase 1 MVP | Declaration-only files are valid direct inputs for `check`, even though they are not valid runtime/build/test entrypoints |
| `kali check --sandbox kali.policy.json main.ts` | Phase 1 MVP | Phase 1 validates policy schema/config; Phase 2+ also checks inferred effects against the policy |
| `kali check --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node typing/global subset exists |
| `kali check --api browser main.ts` | Phase 1 MVP | Supported browser-targeted analysis/profile |
| `kali build main.ts` | Phase 1 MVP | Produce one linked WASM payload with the canonical default tuple (`apiSurface=deno`, `buildMode=fast`, `runtimeProfiles=[]`, `compat.features=[]`) and the default executable artifact mode |
| `kali build --sandbox kali.policy.json main.ts` | Phase 1 MVP | Phase 1 validates policy schema/config for the build; Phase 2+ also performs effect-vs-policy validation |
| `kali build --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset lands for builds too |
| `kali build --bundle --api browser main.ts` | Phase 1 MVP | Supported browser artifact path (`kind: wasm-module` + `kind: js-glue`) |
| `kali build --bundle main.ts` | Rejected by default | In early phases `--bundle` is reserved for browser-targeted output and therefore requires `--api browser` |
| `kali build --api browser main.ts` | Rejected by default | In early phases browser mode is a bundle/check profile, not a standalone non-bundled artifact mode |
| `kali build --lib lib.ts` | Phase 1 MVP | Produce one linked library-style WASM artifact without automatic program start |
| `kali build --lib --api browser lib.ts` | Rejected by default | Early browser support is a bundle/check profile, not a browser-library artifact mode |
| `kali build --capi lib.ts` | Phase 2 target | Public embedding artifact generation should stay gated until the embedding contract is stable; when enabled it emits `kind: wasm-module` + `kind: c-header` + `kind: cabi-metadata` |
| `kali build --capi --api browser lib.ts` | Rejected by default | Early browser support is a bundle/check profile, not a browser-embedding artifact mode |
| `kali test` / `kali test --api deno` | Phase 1 MVP | Compile and run tests with the default standalone tuple (`apiSurface=deno`, `buildMode=fast`, `runtimeProfiles=[]`, `compat.features=[]`) unless overridden |
| declaration-only file passed to `run` / `effects` / `build` / `test` as an entrypoint | Rejected by default | Declaration files are analysis/type inputs, not executable/effect-report entrypoints |
| `kali test --sandbox kali.policy.json` | Phase 1 MVP | Runtime sandbox enforcement path for tests; policy schema/ranges must validate before execution starts |
| `kali test --api node` | Phase 3 target | Reject with `E5006` until the documented Node subset lands for test runs too |
| `kali test --api browser` | Rejected by default | Early browser support is a check/build profile, not a standalone test-runtime profile |
| `kali test --coverage` | Phase 2 target | Coverage needs a stable machine-readable report contract instead of ad hoc runner output |
| `kali effects main.ts` | Phase 2 target | Before then: unavailable or explicitly experimental, never a partial bespoke report; when available it uses the same default API-surface selection as `check` (`apiSurface=deno`) unless overridden |
| `kali effects --api browser main.ts` | Phase 2 target | Browser-targeted effect analysis follows the same browser-analysis intent as `kali check --api browser` once the Phase 2 command exists |
| `kali effects --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset exists for effect analysis too |
| `kali package-effects lodash` | Phase 2 target | Depends on effect-report pipeline; reject/mark experimental before then |
| `kali package-audit [pkg]` | Later compatibility | Tooling feature, not a Phase 1-2 compiler/runtime milestone |
| `kali install --allow-scripts <pkg>` | Opt-in only | Explicit one-shot escape hatch for packages that need lifecycle scripts; still reject native addons / `node-gyp` |
| `--compat eval` | Phase 4 compatibility | Before runtime support exists, reject with `E5006` rather than parsing and silently ignoring the flag |
| `--wasm-threads` | Later compatibility (opt-in only) | Reject with `E5006` until the threaded runtime profile exists; after that, still reject explicitly when unavailable on the selected target/engine |

## Phase Exit Criteria

These checklists keep the phase labels operational rather than purely descriptive.

### Phase 1 exit criteria
- One linked-WASM-payload compile/run pipeline works end-to-end for TS and JS inputs, with companion artifacts only where an output mode explicitly requires them.
- `kali run`, `build`, `check`, `fmt`, `lint`, `test`, and `install` exist with stable core behavior.
- The checker ships the bounded HM-style local/return inference fragment promised for Phase 1, while still falling back conservatively instead of doing open-ended whole-program search.
- Browser-targeted `check --api browser` and `build --bundle --api browser` work against the real browser ambient surface without implying DOM runtime support in Kali itself.
- `kali check` / `build` / `run` / `test` all use the same early-phase API-surface maturity rules: Deno-supported, Node phase-gated, browser supported only for the documented browser-targeted check/bundle paths.
- once `kali effects` lands in Phase 2, it follows the same API-surface split as analysis commands: default `deno`, browser-targeted analysis supported, and Node phase-gated until the documented subset exists.
- Runtime sandbox enforcement and resource limits work for the documented Phase 1 host APIs.
- Unsupported dynamic features fail with the canonical feature-maturity diagnostic instead of silently degrading.
- Package support works for the documented pure JS/TS, statically linkable subset.

### Phase 2 exit criteria
- MIR is the canonical ownership/layout IR.
- `kali effects` emits the documented stable JSON report.
- Explicit effect annotations and `pure` checking are enabled for the built-in capability model.
- Compile/check-time effect-vs-policy validation works against the declarative policy schema.
- Stable public Rust embedding and C ABI surfaces are documented and shipped.

### Phase 3 exit criteria
- Specialization materially improves generated code for common generic/layout-heavy programs.
- Incremental compilation exists for realistic multi-module projects.
- Node compatibility covers a meaningful documented subset rather than isolated package anecdotes.
- Browser packaging/interoperability improves beyond the Phase 1 bundle baseline.

### Phase 4 exit criteria
- Dynamic-compatibility paths such as `eval` are implemented behind explicit compatibility switches.
- Advanced compatibility features preserve the sandbox/effect model instead of bypassing it.
- Proof coverage expands for the most security- and correctness-critical subsystems.

## Compatibility Appendix by Concern Area

This appendix separates the broad compatibility story into smaller tables so language support, type-system support, host/runtime support, and package support do not get conflated.

### Language Semantics

| Concern | Early canonical status | Notes |
|---|---|---|
| Core ECMAScript syntax and static ESM graph | Phase 1 MVP | Parser stays broad and should track the latest published standard grammar; unsupported semantics are gated separately |
| Annex B / web-legacy semantics | Later compatibility | Broad syntax support does not imply immediate support for every legacy browser semantic corner |
| Plain JavaScript compilation with inference | Phase 1 MVP | `.js` is a first-class input, not a degraded compatibility mode |
| CommonJS lowering with statically resolvable `require("...")` | Phase 1 MVP | Compile-time transform inside the linked-artifact model |
| Literal-string `import()` | Phase 3 target | Lower to the already-linked graph rather than runtime WASM module linking |
| Non-literal dynamic loading | Later compatibility | Host-mediated path with dynamic effect boundary |
| `eval` / `Function()` | Phase 4 compatibility | Explicit `--compat eval` path only |
| Weak refs / finalization / proxy-heavy semantics | Later compatibility | Deferred until faithful semantics fit the no-GC, AOT-first design |

### Type System and Analysis

| Concern | Early canonical status | Notes |
|---|---|---|
| TypeScript-compatible checking and flow narrowing | Phase 1 MVP | Compatibility first |
| Stronger JS inference and conservative fallback to `unknown` / dynamic representations | Phase 1 MVP | Needed for plain JS compilation |
| Bounded HM-style local/return inference | Phase 1 MVP | Early inference should improve materially on plain `tsc` local inference without requiring open-ended whole-program search |
| Stable built-in capability-effect reporting | Phase 2 target | `kali effects` and policy checking |
| Explicit `pure` / effect annotations | Phase 2 target | Built-in sandbox capability model first |
| Stable user-defined/custom effects in machine contracts | Later compatibility | Keep early schemas/policies simple |

### Host and Runtime Profiles

| Concern | Early canonical status | Notes |
|---|---|---|
| Deno-oriented standalone API surface (`--api deno`) | Phase 1 MVP | Default API surface for standalone execution; typically paired with the baseline single-threaded runtime profile |
| Invocation arguments in the standalone surface (`Deno.args`) | Phase 1 MVP | Part of the execution context rather than a separately policy-gated capability in schema v1 |
| Read-only environment access in the Deno standalone surface | Phase 1 MVP | Exposes only the sandbox-permitted environment view |
| Mutable environment access / process-environment mutation | Phase 3 target | Policy-controlled host mutation, not part of the Phase 1 baseline |
| Subprocess spawning and socket/listener networking | Phase 3 target | Shares the same sandbox/process/network maturity path as the corresponding capability rows above |
| Browser-targeted `check` and `build --bundle` | Phase 1 MVP | Real browser host via emitted glue, with browser ambient typings available during analysis/build but no standalone browser emulation |
| Standalone `run --api browser` | Rejected by default | No embedded browser engine |
| Node API surface across `check` / `effects` / `build` / `run` / `test` | Phase 3 target | Package-driven subset first; early phases reject `--api node` consistently rather than exposing a partial surface |
| Threaded runtime profile / `--wasm-threads` | Later compatibility (opt-in only) | Runtime-profile switch, independent from API-surface selection |

### Packages and Ecosystem

| Concern | Early canonical status | Notes |
|---|---|---|
| Pure JS/TS npm packages within the linked-artifact model | Phase 1 MVP | No native addons |
| Pure JS/TS JSR packages within the linked-artifact model | Phase 1 MVP | Registry-style install/lock/materialization path just like npm in early phases |
| Raw URL imports in the shared lock/materialization model | Phase 1 MVP | Pin in `kali.lock`, materialize under `.kali/cache/urls/`, and keep ordinary commands deterministic |
| Deno-condition package resolution in the default standalone surface | Phase 1 MVP | Honor `exports` condition `deno` when `--api deno` is selected |
| Browser-condition package resolution in browser bundle mode | Phase 1 MVP | `browser` field / `exports` browser condition |
| npm lifecycle scripts | Opt-in only | `kali install --allow-scripts` |
| Native addons / `node-gyp` | Rejected by default | Violates pure-Rust/no-native-addon constraints |
| Broader Node-host-heavy npm compatibility | Phase 3 target | Depends on meaningful Node API support |

## Hard-Feature Implementation Stage Matrix

This matrix is a compact cross-check for the features most likely to drift between parser, checker, effects, lowering, and runtime docs.

| Feature | Parse | Type/effect analysis | Lowering / codegen | Execution |
|---|---|---|---|---|
| dynamic `require()` | Yes | Recognize as unsupported dynamic loading | No early-phase lowering | Rejected by default |
| literal-string `import()` | Yes | Analyze as statically known target once implemented | Phase 3 target lowering to the already-linked graph | Phase 3 target |
| non-literal `import(expr)` | Yes | Mark as dynamic effect boundary when analyzed | No early-phase lowering | Rejected by default |
| `eval` / `Function()` | Yes | Report `Eval` effect; type-check conservatively around the boundary | Phase 4 compatibility path only | Rejected by default unless `--compat eval` is implemented and enabled |
| explicit `pure` / effect annotations | Yes | Phase 2 target validation | N/A beyond analysis metadata | N/A |
| `Proxy` | Yes | Analyze conservatively where possible; may trigger `dynamicReasons: ["proxy-traps"]` | Lower only once faithful runtime support exists | Later compatibility |
| `WeakMap` / `WeakSet` / `FinalizationRegistry` | Yes | Parse and basic type-checking may exist ahead of lowering | Lower only once faithful semantics are specified | Later compatibility |

## Features Most Likely to Appear in Diagnostics

The compiler should produce clear, stable diagnostics for these cases, using the canonical `E5006` shape from [specs/15-errors.md](15-errors.md) unless a stricter subsystem-specific error is more informative:
- dynamic `require()` in early phases
- non-literal `import(expr)` in early phases
- `eval` / `Function()` without `--compat eval`
- host-registered sandbox policy predicates before the documented embedding-only compatibility path exists
- `Proxy` usage in unsupported runtime modes
- weak-reference APIs before their semantics are implemented
- `--api node` or browser-only assumptions outside the documented profile
- `--wasm-threads` before the threaded runtime profile exists, or on targets/profiles that do not support it

## Canonical Early-Phase Handling

To reduce drift between CLI, runtime, package, and error-reporting specs, unsupported or gated features should follow this table unless a later spec explicitly tightens the behavior.

| Feature | Parse support | Early-phase semantic handling |
|---|---|---|
| dynamic `require()` | Yes | Reject by default with a feature-maturity diagnostic |
| non-literal `import(expr)` | Yes | Reject by default; mark as a dynamic effect boundary when analyzed |
| literal-string `import()` | Yes | Parse early; enable only once lowered to the already-linked graph |
| `eval` / `Function()` | Yes | Report `Eval` effect; reject by default unless `--compat eval` is enabled and the runtime phase supports it |
| `pure` / explicit effect annotations | Yes | Parse early; checker enables and validates in Phase 2+ |
| `Proxy` | Yes | Type-check where possible, but reject unsupported runtime lowering paths |
| `WeakMap` / `WeakSet` / `FinalizationRegistry` | Yes | Reject or gate until faithful semantics are implemented |

This table intentionally separates syntax acceptance from semantic support so the parser and AST can stay broad without forcing premature runtime commitments.

See also:
- [SPEC.md](../SPEC.md)
- [01 — Architecture](01-architecture.md)
- [10 — Runtime](10-runtime.md)
- [14 — Package Management](14-packages.md)
