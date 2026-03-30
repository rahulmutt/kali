# 19 — Feature Maturity

This document is the canonical matrix for features that are easy to describe inconsistently across architecture, runtime, package, and CLI specs.

If another spec needs to mention one of these features, it should link here for phase/status rather than restating a different maturity decision.

Status-label spelling rule:
- docs should use the canonical labels from this chapter verbatim (`Phase 1 MVP`, `Phase 2 target`, `Phase 3 target`, `Phase 4 compatibility`, `Later compatibility`, `Opt-in only`, `Later compatibility (opt-in only)`, `Rejected by default`) rather than near-duplicates such as `later-compatibility`

## Status Labels

- **Phase 1 MVP** — required for the first practically useful implementation
- **Phase 2 target** — planned once ownership/effects infrastructure lands
- **Phase 3 target** — planned once specialization/ecosystem work lands
- **Phase 4 compatibility** — supported only in the advanced compatibility phase
- **Later compatibility** — intentionally deferred until semantics and cost are justified
- **Opt-in only** — supported only behind an explicit flag or config
- **Later compatibility (opt-in only)** — deferred until a later phase and, even then, enabled only behind an explicit runtime/profile switch
- **Rejected by default** — Kali may still recognize the surface (for example syntax, config, policy, flag, or command shape), but normal compile/run/command handling should fail unless a documented compatibility or availability path is enabled in a phase that actually implements it

## Status Interpretation Table

| Status label | Meaning in practice |
|---|---|
| Phase 1 MVP | Must ship in the first dependable end-to-end release |
| Phase 2 target / Phase 3 target / Phase 4 compatibility | Planned work for that phase; before then, reject with the canonical gating path rather than partially emulating it |
| Later compatibility | Intentionally deferred with no near-phase promise |
| Opt-in only | Implemented only behind an explicit flag/config even after support exists |
| Later compatibility (opt-in only) | Both deferred and explicitly gated when it eventually lands |
| Rejected by default | The surface may still be recognized or parsed, but normal compile/run/command handling rejects it unless a documented compatibility or availability path exists and is implemented |

This table exists to keep the status labels operational: a label implies whether Kali should ship, gate, reject, or require an explicit opt-in.

## Evidence-Backed Promotion Rule

A maturity label is not only a design intention; it also constrains how Kali should talk about support publicly and across the spec set.

Promotion rule:
- a feature may be planned for a given phase before implementation exists
- a feature should only be treated as **supported** for a command/profile/surface once the corresponding evidence exists in the canonical testing tracks from [specs/16-testing.md](16-testing.md)
- the required evidence should match the claim being made:
  - language/runtime semantics → conformance + integration coverage
  - type-system behavior → checker/inference baselines
  - package compatibility → curated package corpus results
  - host/runtime APIs → integration + sandbox/resource-limit coverage
  - browser-targeted analysis/build support → browser-targeted check/build tests + emitted-bundle smoke runs in a real browser harness
  - CLI/JSON contracts → golden/snapshot/schema tests
- isolated demos or one package anecdote do **not** by themselves justify raising a feature's maturity wording

This keeps “Phase 1 MVP” and later status labels tied to measurable behavior rather than intent alone.

## Canonical Matrix

| Feature | Status | Rationale |
|---|---|---|
| Latest published ECMA-262 lexical/parser grammar (current standard edition) | Phase 1 MVP | Front-end coverage should track the current standard grammar even when some semantics remain phase-gated |
| Stage-3+/draft TC39 proposals beyond the latest published ECMA-262 edition | Rejected by default | Keep the “latest ECMA-262” promise scoped to published editions; proposal support needs an explicit experimental flag or its own maturity row instead of being implied by grammar tracking |
| Current-edition non-Annex-B semantics for features Kali marks as supported in a given command/profile | Phase 1 MVP | "Latest standard support" is not parser-only: once Kali claims a feature is supported for a command/profile, the supported path should aim at faithful current-edition semantics and be backed by the matching evidence track rather than by syntax acceptance alone |
| Static ESM `import` / `export` | Phase 1 MVP | Core module system |
| First-class JavaScript compilation with inference | Phase 1 MVP | Required so `.js` projects are not forced to migrate to TypeScript before benefiting from Kali |
| CommonJS module lowering | Phase 1 MVP | Needed for early npm package compatibility within the linked-artifact model |
| `require("literal")` | Phase 1 MVP | Rewritten during compilation when statically resolvable |
| Dynamic `require()` | Rejected by default | Conflicts with the early single-linked-artifact model |
| Broad npm compatibility for packages that expect more Node built-ins | Phase 3 target | Depends on broader `--api node` support beyond the Phase 1 package baseline |
| Literal-string `import()` | Phase 3 target | Can be lowered to the already-linked graph without runtime WASM module linking |
| Non-literal `import(expr)` | Later compatibility | Requires a dynamic host-mediated path and conservative effect handling |
| `eval` | Phase 4 compatibility | Parsed and effect-tracked earlier, but full runtime support is deferred; compatibility path is the schema-v1 `--compat eval` switch when implemented |
| `Function()` constructor | Phase 4 compatibility | Same status as `eval`; schema v1 intentionally reuses the same `--compat eval` switch instead of introducing a second compatibility-feature name |
| AOT-only compilation model (no language-level JIT) | Phase 1 MVP | Foundational product constraint from the bootstrap brief; optimization work must preserve ahead-of-time compilation rather than introducing a language-level JIT path |
| No tracing/background GC in the execution model | Phase 1 MVP | Deterministic ownership/reference-counted strategies may exist where other chapters allow them, but tracing/background GC is outside the supported design |
| Pure-Rust implementation with no embedded C/C++ libraries | Phase 1 MVP | Keeps the toolchain/runtime/embedding contract aligned with the bootstrap constraint and avoids drifting into native dependency exceptions later |
| Standardized Kali-hosted execution engine: `wasmtime` | Phase 1 MVP | Early standalone execution and embedding target one documented pure-Rust engine so runtime behavior and testing have a single baseline |
| Alternative Kali-hosted execution engines beyond `wasmtime` | Later compatibility | Engine plurality is an implementation extension after the first documented runtime contract is stable; it must not weaken the language/runtime guarantees or the pure-Rust constraint |
| Invocation arguments (`Deno.args`; later Node `process.argv`) | Phase 1 MVP | Treated as caller-supplied execution context rather than a separately policy-gated host capability in schema v1 |
| Read-only environment access (`Deno.env.get`, `Deno.env.toObject`, policy-filtered host env view) | Phase 1 MVP | Needed for practical standalone compatibility while still fitting the sandbox model |
| Read-only `Deno.permissions` facade (`query`-style granted/denied view only; no `request()` / `revoke()` escalation path) | Phase 1 MVP | Exposes Kali sandbox state for compatibility without interactive permission escalation |
| Interactive permission escalation APIs (`Deno.permissions.request()` / `revoke()` and similar prompt-style flows) | Rejected by default | Kali's sandbox model is resolved before execution; permission observation is supported in Phase 1, but runtime privilege negotiation/prompting is not |
| Web Crypto randomness subset (`crypto.getRandomValues`, mapping to the canonical `Random.GetBytes` effect / `effects.random` policy key) | Phase 1 MVP | Keeps the Phase 1 Web baseline aligned with the effect and sandbox schemas without overpromising the full Web Crypto surface |
| Mutable environment access (`Deno.env.set`, `process.env = ...`-style host mutation) | Phase 3 target | Widens the host contract and must remain policy-controlled |
| Subprocess spawning (`Deno.Command`, host `process_spawn`) | Phase 3 target | Requires explicit sandbox/process-budget integration |
| Socket/listener networking (`Network.Connect`, `Network.Listen`, `Deno.serve`) | Phase 3 target | Requires explicit network policy and concurrency controls |
| Process identity and process-control/working-directory APIs (`Deno.pid`, `process.pid`, `Deno.exit`, `Deno.cwd`, `Deno.chdir`) | Later compatibility | Deferred until a future schema/policy contract makes their sandbox and embedding behavior explicit |
| Built-in effect inference / `kali effects` | Phase 2 target | Required for sandbox-first analysis and policy checking |
| Explicit effect annotations / `pure` | Phase 2 target | Initially scoped to the built-in sandbox capability model |
| User-defined/custom effect kinds in stable reports or policy checking | Later compatibility | Keep Phase 1-2 machine contracts limited to built-in sandbox-relevant effects |
| Algebraic effect declarations / handlers | Later compatibility | Experimental and must not block delivery of the core capability/effect system |
| Executable project-local sandbox policy code (`kali.policy.json` hooks / inline predicates) | Rejected by default | Project policy files stay declarative data; Kali should not execute project code just to decide whether a capability is allowed |
| Host-registered sandbox policy predicates | Later compatibility | This is the long-term programmable-policy path: initial policies stay declarative, and a later embedding-only extension may add pure host-registered predicates without turning policy files into executable project code. These predicates are narrowing-only: they may reject operations the declarative policy would otherwise allow, but they must not widen declarative denies or bypass command/profile maturity gates. |
| Annex B / web-legacy compatibility corners | Later compatibility | Keep the MVP focused on dependable core semantics; add legacy web behaviors only when conformance value justifies the cost |
| `Proxy` | Later compatibility | High semantic cost and optimization barriers |
| `WeakMap` / `WeakSet` | Later compatibility | Deferred until weak-reference semantics fit the no-tracing-GC design |
| `FinalizationRegistry` | Later compatibility | Same reason as weak collections |
| `SharedArrayBuffer` / `Atomics` | Later compatibility (opt-in only) | Requires a separate threaded runtime profile and should not be implied by the Phase 1 single-threaded runtime |
| `--wasm-threads` | Later compatibility (opt-in only) | Enables the threaded runtime profile once that profile exists; must fail explicitly before then and on unsupported targets/engines |
| Browser API surface for supported analysis/build commands (`--api browser`), including ambient DOM typings for those commands | Phase 1 MVP | Phase 1 enables browser-targeted analysis/build against the real browser ambient surface for `check` and `build --bundle`, including the DOM typings normally expected in browser programs, without claiming DOM support in Kali's standalone runtime; this status requires its own browser-targeted evidence track rather than inference from standalone runtime tests, and later analysis commands may reuse that same browser context once their own maturity rows allow it |
| `package.json#exports` condition `deno` for `--api deno` resolution | Phase 1 MVP | Aligns package resolution with the default Deno-oriented standalone API surface |
| `package.json#browser` replacement maps and `exports` condition `browser` in browser-targeted analysis/build contexts | Phase 1 MVP | Needed for practical browser-targeted npm compatibility without widening standalone runtime claims; supported browser-targeted commands should share one browser **package-resolution context** (browser `exports` condition order plus any applicable `package.json#browser` rewrites) rather than inventing per-command ladders |
| `run --api browser` | Rejected by default | Early standalone runtime does not emulate a browser host |
| npm lifecycle scripts (`kali install --allow-scripts`) | Opt-in only | Disabled by default for sandbox-first behavior; this is an install-time package-hook escape hatch, not evidence of `--api node` support or participation in the normal sandbox/effect-report contract |
| Automatic dependency installation or lockfile/materialization repair during `check` / `effects` / `build` / `run` / `test` | Rejected by default | Keeps dependency state deterministic and makes `kali install` the single mutating dependency-management command; missing/stale state should fail with `E5004` instead of being repaired implicitly |
| Packages whose normal install/runtime path depends on native addons, compiled native code, postinstall-downloaded executables, or other platform-specific binary/bootstrap artifacts | Rejected by default | Violates the pure-Rust/no-native-addon goal, weakens deterministic install expectations, and should not be implied by `--allow-scripts` |
| Native addons / `node-gyp` packages | Rejected by default | Violates the pure-Rust/no-native-addon constraints |
| npm packages that require unsupported Node core modules | Phase 3 target | Depends on broader `--api node` compatibility work |
| Phase-1 **base library artifact** (`kali build --lib`) | Phase 1 MVP | This is the Phase-1 side of the shared **embedding-stability split**: projects can build non-executable exported modules early without waiting for the later public embedding contract to freeze |
| Stable public Rust embedding API | Phase 2 target | Part of the Phase-2 **public embedding outputs** side of that same split |
| Stable public library/WIT contract for `kali build --lib` | Phase 2 target | The same `--lib` selector is promoted from the Phase-1 **base library artifact** into the stable public library contract and emits WIT by default once that interface surface is frozen |
| Stable public C ABI / `kali build --capi` flow | Phase 2 target | Part of the same Phase-2 **public embedding outputs** stabilization work |
| WIT emission for public library/embedding interfaces | Phase 2 target | Gives the Phase-2 **public embedding outputs** one canonical exported interface description instead of parallel ad hoc metadata |
| WebAssembly Component Model packaging (`kali build --component`) | Phase 2 target | Part of the same Phase-2 **public embedding outputs** set, layered on top of the linked core WASM payload for host interop; executable builds still center on the core module path |
| Host ABI versioning for `kali_capi` | Phase 2 target | Stable embedding requires explicit load-time compatibility checks |
| DOM APIs in standalone runtime | Rejected by default | Kali does not embed a browser engine |

## Interpretation Rules

1. **Single-payload rule**: builds in Phases 1-3 target one linked WASM payload for the resolved static graph. Artifact modes may still add companion artifacts such as JS glue, WIT files, component wrappers, or C headers, but they must not reintroduce runtime WASM module linking.
2. **Parse vs support**: accepted syntax does not imply full runtime support; unsupported dynamic features should be diagnosed explicitly.
3. **Effect boundaries**: features marked as dynamic compatibility paths should be reflected in static effect analysis.
4. **No silent fallback**: if a feature cannot be implemented faithfully under the current phase constraints, Kali should reject or gate it rather than emulate it loosely.
5. **Policy alignment**: sandbox policy validation may always deny a capability, but it must reject policies that try to enable capabilities unavailable in the selected command/profile/phase.
6. **Canonical gating diagnostic**: use the shared feature-maturity diagnostic contract (`E5006`) so CLI, checker, runtime, and package tooling report phase/profile gating consistently.
7. **Do not overuse `E5006`**: selecting an unavailable command/profile/feature uses `E5006`, but ordinary references to names/globals that are simply absent from the selected supported ambient surface should use the normal name/type diagnostics instead.
8. **Sandbox-domain honesty**: build-time policy compatibility for browser-targeted artifacts must not be described as equivalent to Kali-hosted runtime sandbox enforcement.

## Canonical Command/Profile Matrix

This table exists to stop drift between CLI examples, runtime behavior, package tooling, and error reporting.

Interpretation rule:
- matrix rows are evaluated against the fully merged **effective command context** (built-in defaults, then discovered config, then CLI flags)
- examples written with explicit flags also apply when the same value was inherited from `kali.json`
- only the axes that participate for the selected command are maturity-relevant; non-participating inherited axes are ignored rather than becoming hidden gates or contradictions
- Kali must not silently fall back from an inherited unsupported/contradictory participating context to a different API surface/profile just because the user omitted the matching CLI flag
- browser rows follow the top-level **canonical browser-surface rejection split** from [SPEC.md](../SPEC.md): wrong browser build shapes are `E5008`, while unsupported browser execution/test/runtime contracts are `E5006`
- follow the canonical validation-order rule from [SPEC.md](../SPEC.md): **command shape/arity first**, then the command's own phase availability, then finer-grained inherited-context/profile gates inside that command
- matrix-row status names the **earliest phase where the full command/context combination can be supported**, not necessarily the first diagnostic a pre-support implementation should report when more than one independent gate is still closed
- in this command/profile matrix, the status label is a planning/maturity summary, not by itself the diagnostic choice: rows marked **Rejected by default** may still fail as `E5008` invalid usage or `E5006` unavailable-feature gating depending on the canonical handling column and the shared validation-order rules
- for example, `kali build --capi --api node lib.ts` is listed as a **Phase 3 target** because that full combination cannot work before both public embedding artifacts and the Node surface exist, but an early implementation should still report the outermost failing gate first (`--capi` itself in Phase 1, then `--api node` once `--capi` exists but Node remains gated)
- this keeps diagnostics stable for commands such as `package-effects`: before Phase 2, plain `kali package-effects lodash` should fail on the command's base maturity row; once the command exists, inherited analysis context follows the maturity of the inherited axis instead of a package-analysis-specific shadow matrix (`apiSurface = browser` follows the browser analysis row, `apiSurface = node` follows the Node analysis row, `runtimeProfiles = ["wasm-threads"]` follows the threaded-profile row, and `compat.features = ["eval"]` follows the compatibility-feature row)

| Command / profile | Early-phase status | Canonical handling |
|---|---|---|
| `kali init` | Phase 1 MVP | Create the minimal canonical project scaffold **in the current working directory**; `init` is current-directory-scoped and does not reuse an ancestor project's discovered root. For the default app template this normally means `kali.json` containing just `{ "schemaVersion": 1 }` plus `main.ts`; for the library template, the same minimal config plus `lib.ts`. Scaffolding does not count as dependency-state mutation: `init` must not add dependencies, write `kali.lock`, or materialize packages. |
| `kali init` when the current working directory already contains `kali.json` | Rejected by default | Fail with `E5008` instead of silently overwriting the existing project config |
| `kali init` in a subdirectory whose ancestor already contains `kali.json` | Phase 1 MVP | Create a nested child project rooted at the current working directory when that directory itself does not already contain `kali.json`; later discovery treats that child root as a separate project boundary |
| `kali init --sandbox kali.policy.json` | Rejected by default | `init` is sandbox-agnostic in early phases; scaffolding does not accept the runtime/build policy-attachment flag, so this is invalid usage (`E5008`) |
| `kali init --lib` | Phase 1 MVP | Select a library-oriented project template only; it does not implicitly change the later `kali build` artifact mode |
| `kali fmt` | Phase 1 MVP | Stable formatting command over the canonical project file set relevant to formatting, including declaration-only files |
| `kali fmt --sandbox kali.policy.json` | Rejected by default | `fmt` is sandbox-agnostic in early phases; top-level config sandbox is ignored for it, and the CLI `--sandbox` flag is invalid usage (`E5008`) |
| `kali lint` | Phase 1 MVP | Stable lint command with conservative autofix support over the canonical lintable project file set, including declaration-only files |
| `kali lint --fix` | Phase 1 MVP | Apply only structured non-speculative lint fixes; overlapping fixes stay unapplied rather than being guessed into one rewrite in schema v1 |
| `kali lint --sandbox kali.policy.json` | Rejected by default | `lint` is sandbox-agnostic in early phases; top-level config sandbox is ignored for it, and the CLI `--sandbox` flag is invalid usage (`E5008`) |
| `kali install` | Phase 1 MVP | Reconcile the project's managed dependency state: update dependency-owning manifest fields when an explicit registry target is added, resolve/materialize dependency state for the declared dependency source kinds, and write `kali.lock`; install is profile-agnostic in early phases and does not require separate per-`--api` installs |
| `kali install lodash` in canonical configless project mode | Phase 1 MVP | Create the minimal canonical manifest `{ "schemaVersion": 1 }` at the effective project root, then record the dependency and materialize/install it; explicit registry-package adds stay on the manifest path rather than inventing a configless side channel |
| plain `kali install` in canonical configless project mode with no dependency inputs | Phase 1 MVP | Succeed as a no-op and do not create a placeholder `kali.json`; running the command alone is not a request to scaffold a project |
| `kali install foo bar` | Rejected by default | Early phases accept at most one explicit install target for `install`; batch package adds or multi-target installs require a later explicit mode, so this is invalid command usage (`E5008`) |
| `kali install --dev` | Rejected by default | `--dev` modifies an explicit registry package target in early phases; using it without one is invalid command usage (`E5008`) |
| `kali install --api ...` | Rejected by default | `install` is profile-agnostic in early phases, so `--api` is invalid command usage (`E5008`) rather than a second install mode |
| `kali install --sandbox kali.policy.json` | Rejected by default | `install` is sandbox-agnostic in early phases; top-level config sandbox is ignored for it, but the CLI `--sandbox` flag is not accepted here and should fail with `E5008` |
| `kali install https://...` | Phase 1 MVP | Explicitly pin/materialize a raw URL dependency into the shared lock/materialization model; in configless mode this may still create `kali.lock` and `.kali/cache/urls/` state, but it does not scaffold a placeholder manifest |
| `kali install --dev https://...` | Rejected by default | `--dev` applies only to explicit registry-package targets in early phases; pairing it with a raw URL is invalid command usage (`E5008`) rather than a second raw-URL manifest mode |
| `kali install --allow-scripts` | Opt-in only | Valid when the invocation has non-empty **effective npm-scriptable install work**; that subset is invocation-scoped and covers only the npm package work the install actually reconciles in a lifecycle-hook-relevant way, including directly requested npm targets and transitively touched npm dependencies, while JSR/raw-URL work stays on the normal script-free path |
| `kali install --allow-scripts` on a URL-only / JSR-only / clean already-synchronized / otherwise no-npm graph | Rejected by default | If the invocation has no effective npm-scriptable install work for the flag to affect, fail with `E5008` instead of silently behaving like plain `install` |
| `kali install --allow-scripts https://...` | Rejected by default | Raw URLs do not have npm lifecycle hooks, so pairing `--allow-scripts` with a raw URL is invalid command usage (`E5008`) rather than a second install mode |
| `kali install --allow-scripts jsr:@std/path` | Rejected by default | JSR packages do not participate in npm lifecycle-script execution in schema v1, so this flag/target combination is invalid command usage (`E5008`) |
| non-install command auto-repair of missing/stale dependency state | Rejected by default | `check` / `effects` / `build` / `run` / `test` must fail with `E5004` and point users to `kali install` instead of mutating dependency state opportunistically |
| `kali run` with no explicit entrypoint | Rejected by default | `run` is a direct-input command in early phases; omitting the entrypoint should fail with `E5008` rather than guessing `main.ts` or scanning the project |
| `kali run a.ts b.ts` | Rejected by default | Early phases accept exactly one primary runtime entrypoint; multi-entry execution requires a later explicit mode, so this should fail with `E5008` |
| `kali run main.ts` | Phase 1 MVP | Compile and execute with the canonical default tuple: `apiSurface=deno`, `buildMode=fast`, `runtimeProfiles=[]`, `compat.features=[]` |
| `kali run --sandbox kali.policy.json main.ts` | Phase 1 MVP | Runtime sandbox enforcement path; policy schema/ranges must validate before execution starts |
| `kali run --api deno main.ts` | Phase 1 MVP | Supported standalone runtime path |
| `kali run --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset lands |
| `kali run --api browser main.ts` | Rejected by default | Reject with `E5006`; browser is an analysis/build context first |
| `kali check` | Phase 1 MVP | Type-check the canonical project-discovery result with the default API surface (`apiSurface=deno`) |
| `kali check main.ts` | Phase 1 MVP | Type-check with the canonical default API surface (`apiSurface=deno`) |
| `kali check a.ts b.ts` | Phase 1 MVP | `check` follows the shared **set-oriented explicit-file** rule in early phases: multiple explicit files are allowed and should be checked as one explicit file set rather than rejected as though `check` were a single-entry direct command |
| `kali check types.d.ts` | Phase 1 MVP | Declaration-only files are valid explicit file inputs for `check`, even though they are not valid runtime entrypoints, build/effect primary inputs, or test entrypoints |
| `kali check --sandbox kali.policy.json` | Phase 1 MVP | Reuse the same project-discovery behavior as plain `kali check`; Phase 1 validates policy schema/config for the discovered project graph, and Phase 2+ also checks inferred effects against the policy |
| `kali check --sandbox kali.policy.json main.ts` | Phase 1 MVP | Same validation path, but scoped to the explicit file set rather than the discovered project graph |
| `kali check --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node typing/global subset exists |
| `kali check --api browser` | Phase 1 MVP | Supported browser-targeted analysis context over the canonical project-discovery result; browser targeting changes the analysis context, not the hybrid-input nature of `check` |
| `kali check --api browser main.ts` | Phase 1 MVP | Supported browser-targeted analysis context for an explicit file set |
| `kali check --wasm-threads main.ts` | Later compatibility (opt-in only) | `check` participates in runtime-profile-sensitive analysis when relevant; reject with `E5006` until the threaded profile exists and the selected analysis mode supports it |
| `kali check --api browser --sandbox kali.policy.json` | Phase 1 MVP | Browser-targeted static policy validation over the discovered project graph, following the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md) |
| `kali check --api browser --sandbox kali.policy.json main.ts` | Phase 1 MVP | Same browser-targeted static policy validation path, but scoped to the explicit file set rather than the discovered project graph and still following the **browser-targeted static sandbox contract** |
| `kali check --sandbox kali.policy.json a.ts b.ts` | Phase 1 MVP | `check` keeps that shared **set-oriented explicit-file** behavior under `--sandbox`; this validates the supplied file set rather than inventing a single-entry mode |
| `kali check --fix` | Later compatibility | The checker may emit structured fix metadata earlier, but schema v1 keeps CLI autofix lint-only to avoid unstable multi-diagnostic rewrite semantics |
| `kali build` with no explicit primary source input | Rejected by default | `build` is a direct-input command in early phases; omitting the primary source input should fail with `E5008` rather than guessing `main.ts` or scanning the project |
| `kali build a.ts b.ts` | Rejected by default | Early phases accept exactly one primary build source input; multi-entry artifact modes require a later explicit spec, so this should fail with `E5008` |
| `kali build main.ts` | Phase 1 MVP | Produce one linked WASM payload with the canonical default tuple (`apiSurface=deno`, `buildMode=fast`, `runtimeProfiles=[]`, `compat.features=[]`) and the default executable artifact mode |
| `kali build --sandbox kali.policy.json main.ts` | Phase 1 MVP | Phase 1 validates policy schema/config for the build; Phase 2+ also performs effect-vs-policy validation |
| `kali build --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset lands for builds too |
| `kali build --bundle --api browser main.ts` | Phase 1 MVP | Supported browser artifact path (`kind: wasm-module`, `role: primary-executable` + `kind: js-glue`, `role: browser-glue`) |
| `kali build --bundle --api browser --sandbox kali.policy.json main.ts` | Phase 1 MVP | Browser-targeted static policy validation path only, following the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md) |
| `kali build --bundle main.ts` | Rejected by default | Under the default tuple this is invalid command usage (`E5008`) because `--bundle` is reserved for browser-targeted output and therefore requires the effective API surface to be `browser`; with browser selected via CLI/config, the browser-bundle path is the supported Phase 1 mode |
| `kali build --bundle --api node main.ts` | Rejected by default | Invalid command usage (`E5008`): browser bundle mode exists, but pairing `--bundle` with an explicit non-browser API surface is a contradictory command shape rather than a separate maturity-gated runtime mode |
| `kali build --api browser main.ts` | Rejected by default | Invalid command usage (`E5008`) in early phases: browser mode is available for `check` and `build --bundle`, but a non-bundled browser build mode is not defined yet |
| `kali build --lib lib.ts` | Phase 1 MVP | Produce one linked export-oriented **base library** WASM artifact following the shared **library-oriented instantiation rule** from [SPEC.md](../SPEC.md). Phase 1 emits the base `wasm-module` (`role: primary-library`) only, while the stable public library/WIT surface is still Phase 2 work and then adds the default `wit` sidecar (`role: interface-wit`) |
| library-oriented build with a **statically known export surface** after frontend lowering | Phase 1 MVP | Uses the shared definition from [SPEC.md](../SPEC.md): ESM exports are direct, while CommonJS participates only when static lowering can prove one fixed export set |
| library-oriented build without a statically known export surface | Rejected by default | Fail with `E5011` instead of inventing reflection-based exports for `--lib` / `--capi` / `--component` |
| `kali build --lib --api node lib.ts` | Phase 3 target | Library-oriented build modes still obey the same Node build gate as ordinary `kali build --api node ...`; they do not create a separate early Node surface |
| `kali build --lib --api browser lib.ts` | Rejected by default | Early browser support is an analysis/build context tied to `check` and `build --bundle`, not a browser-library artifact mode |
| `kali build --capi lib.ts` | Phase 2 target | Public embedding artifact generation should stay gated until the embedding contract is stable; when enabled it emits `kind: wasm-module` (`role: primary-library`) + `kind: wit` (`role: interface-wit`) + `kind: c-header` (`role: embedding-header`) + `kind: cabi-metadata` (`role: embedding-metadata`) |
| `kali build --capi --api node lib.ts` | Phase 3 target | Public embedding artifacts remain subject to the ordinary Node build gate; both the embedding flow and the selected API surface must be implemented |
| `kali build --component lib.ts` | Phase 2 target | Component-oriented library packaging path; when enabled it emits `kind: wasm-module` (`role: primary-library`) + `kind: wit` (`role: interface-wit`) + `kind: wasm-component` (`role: primary-component`) |
| `kali build --component --api node lib.ts` | Phase 3 target | Component packaging remains subject to the ordinary Node build gate; it does not create a separate early Node component profile |
| `kali build` with more than one explicit artifact-mode selector from `--bundle` / `--lib` / `--capi` / `--component` | Rejected by default | Artifact mode is a one-of selector in early phases; conflicting combinations such as `--bundle --lib`, `--bundle --capi`, `--bundle --component`, `--lib --capi`, `--lib --component`, or `--capi --component` should fail with `E5008` rather than a feature-maturity diagnostic |
| `kali build --capi --api browser lib.ts` | Rejected by default | Early browser support is an analysis/build context tied to `check` and `build --bundle`, not a browser-embedding artifact mode |
| `kali build --component --api browser lib.ts` | Rejected by default | Early browser support is an analysis/build context tied to `check` and `build --bundle`, not a browser-component artifact mode |
| `kali test` / `kali test --api deno` | Phase 1 MVP | Compile and run tests with the default standalone tuple (`apiSurface=deno`, `buildMode=fast`, `runtimeProfiles=[]`, `compat.features=[]`) unless overridden |
| `kali test a.test.ts b.test.ts` | Phase 1 MVP | Explicit test files bypass naming-pattern discovery and are treated as one explicit test-module set, provided every file is from the executable/analyzable source set |
| declaration-only file passed to `run` / `effects` / `build` / `test` as a primary input | Rejected by default | Declaration files are analysis/type inputs, not executable entrypoints or build/effect primary inputs; use the canonical invalid-entrypoint diagnostic (`E5007`) rather than treating this as general CLI misuse |
| `kali test --sandbox kali.policy.json` | Phase 1 MVP | Runtime sandbox enforcement path for tests; policy schema/ranges must validate before execution starts |
| `kali test --api node` | Phase 3 target | Reject with `E5006` until the documented Node subset lands for test runs too |
| `kali test --api browser` | Rejected by default | Early browser support is an analysis/build context, not a standalone test-runtime profile |
| `kali test --coverage` | Phase 2 target | Coverage needs a stable machine-readable report contract instead of ad hoc runner output |
| `kali effects` with no explicit primary source input | Rejected by default | `effects` is a direct-input command in early phases; omitting the analysis root should fail with `E5008` rather than permission to scan the project |
| `kali effects a.ts b.ts` | Rejected by default | Early phases accept exactly one primary analysis root for `effects`; multi-entry reporting requires a later explicit mode, so this should fail with `E5008` |
| `kali effects main.ts` | Phase 2 target | Before then: unavailable or explicitly experimental, never a partial bespoke report; when available it uses the same default API-surface selection as `check` (`apiSurface=deno`) unless overridden |
| `kali effects --sandbox kali.policy.json main.ts` | Rejected by default | Keep `effects` as a pure reporting command; policy validation belongs to `check/build --sandbox` so the CLI has one canonical policy-validation path. This rejection should use `E5008`, not the `E5006` maturity gate. |
| `kali effects --api browser main.ts` | Phase 2 target | Reuses the same browser API-surface analysis context as `kali check --api browser` once the Phase 2 command exists, without implying standalone browser execution |
| `kali effects --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset exists for effect analysis too |
| `kali effects --wasm-threads main.ts` | Later compatibility (opt-in only) | `effects` records runtime-profile-sensitive analysis context; reject with `E5006` until the threaded profile exists and effect analysis supports it |
| `kali package-effects` with no explicit package | Rejected by default | `package-effects` takes exactly one explicit registry-package argument in early phases; omitting it is invalid command usage (`E5008`) |
| `kali package-effects lodash` | Phase 2 target | Depends on effect-report pipeline; reject/mark experimental before then. Once it exists, it still uses the effective inherited analysis context and must fail with `E5006` instead of silently falling back when that inherited context selects an unavailable analysis mode. |
| `kali package-effects lodash react` | Rejected by default | Early phases do not define a multi-package effect-analysis batch mode, so passing more than one package is invalid command usage (`E5008`) |
| `kali package-effects lodash` under inherited `apiSurface=browser` | Phase 2 target | Reuses the same browser-targeted analysis context and browser **package-resolution context** as `kali check --api browser` once package-effect analysis exists, without introducing package-analysis-specific analysis-context flags |
| `kali package-effects lodash` under inherited `apiSurface=node` | Phase 3 target | Reuses the same Node analysis gate as other analysis commands; before that gate opens, fail with `E5006` rather than silently falling back to `deno` |
| `kali package-effects lodash` under inherited `runtimeProfiles=["wasm-threads"]` | Later compatibility (opt-in only) | Reuses the same threaded-profile gate as other analysis commands; before that profile exists for package-effect analysis, fail with `E5006` |
| `kali package-effects lodash` under inherited `compat.features=["eval"]` | Phase 4 compatibility | Reuses the same compatibility-feature gate as the rest of effect analysis; before then, fail with `E5006` rather than dropping the inherited compatibility selection |
| `kali package-effects --api ... lodash` | Rejected by default | Early package analysis inherits context from config/defaults instead of taking its own `--api` / runtime-profile / `--compat` flag family, so this is invalid command usage (`E5008`) unless a later spec adds those flags |
| `kali package-effects --compat eval lodash` | Rejected by default | Early package analysis inherits context from config/defaults instead of taking its own `--api` / runtime-profile / `--compat` flag family, so this is invalid command usage (`E5008`) unless a later spec adds those flags |
| `kali package-effects --wasm-threads lodash` | Rejected by default | Early package analysis inherits runtime profiles from config/defaults instead of taking package-analysis-specific runtime-profile flags, so this is invalid command usage (`E5008`) unless a later spec adds that mode |
| `kali package-effects --sandbox kali.policy.json lodash` | Rejected by default | `package-effects` is a reporting command, not a second policy-validation entrypoint; top-level config sandbox is ignored for it, and the CLI `--sandbox` flag is invalid usage (`E5008`) |
| `kali package-effects` with a non-registry target (for example `https://...` or `./local.ts`) | Rejected by default | `package-effects` analyzes registry packages only; raw URLs and local paths belong to the project/import-graph workflow instead |
| `kali package-audit` with no explicit package | Rejected by default | `package-audit` is a single-package registry-analysis command in early phases; omitting the package is invalid command usage (`E5008`) rather than an implicit whole-project audit mode |
| `kali package-audit lodash` | Later compatibility | Tooling feature, not a Phase 1-2 compiler/runtime milestone; early `package-audit` is context-free, so inherited `apiSurface` / `buildMode` / `runtimeProfiles` / `compat.features` / top-level `sandbox` do not change its semantics. If it supports `--output json` before a dedicated audit payload schema exists, it uses the canonical **envelope-only JSON support** model: the stable schema-v1 contract is the standard command envelope alone (`payload` omitted or `null`), not an ad hoc audit JSON object, undocumented package/version metadata fields, or `stdout` / `stderr` repurposed as hidden result channels. |
| `kali package-audit lodash react` | Rejected by default | Early phases accept exactly one explicit package argument for `package-audit`; multi-package audit requires a later explicit mode, so this is invalid command usage (`E5008`) |
| `kali package-audit` with a non-registry target (for example `https://...` or `./local.ts`) | Rejected by default | `package-audit` is registry-package-oriented rather than a second raw-URL/local-path analysis path |
| `kali package-audit --api ... lodash` | Rejected by default | Early `package-audit` intentionally stays a single-package registry tool and does not grow package-analysis-specific `--api` / runtime-profile / `--compat` flags; using them is invalid command usage (`E5008`) unless a later spec adds them |
| `kali package-audit --compat eval lodash` | Rejected by default | Early `package-audit` does not take package-analysis-specific `--api` / runtime-profile / `--compat` flags; it stays a context-free registry tool, so this is invalid command usage (`E5008`) unless a later spec adds those flags |
| `kali package-audit --wasm-threads lodash` | Rejected by default | Early `package-audit` is context-free and does not take package-analysis-specific runtime-profile flags, so this is invalid command usage (`E5008`) unless a later spec adds that mode |
| `kali package-audit --sandbox kali.policy.json lodash` | Rejected by default | Early `package-audit` is a context-free registry tool; top-level config sandbox is ignored for it, and the CLI `--sandbox` flag is invalid usage (`E5008`) |
| `kali install --allow-scripts <npm-pkg>` | Opt-in only | Explicit-package example of the same `--allow-scripts` contract above: lifecycle hooks are permitted only for that npm-targeted install work, while native addons, binary/bootstrap-heavy packages, and `node-gyp` remain rejected and this still does not imply Node-runtime or project-sandbox support |
| `kali run/test --max-spawned-processes 0 ...` | Phase 1 MVP | `0` is a valid explicit deny/tightening value even before subprocess support exists; only non-zero values are phase-gated |
| `kali run/test --max-spawned-processes N` with `N > 0` before subprocess support exists | Rejected by default | Reject with `E5006` until the selected command/profile/API surface actually supports subprocesses |
| `kali run/test --max-threads 0 ...` | Phase 1 MVP | `0` is a valid explicit deny/tightening value even before the threaded runtime profile exists; only non-zero values are phase-gated |
| `kali run/test --max-threads N` with `N > 0` before thread support exists | Rejected by default | Reject with `E5006` until the threaded runtime profile exists and the selected command/profile supports it |
| `--compat eval` | Phase 4 compatibility | Before runtime support exists, reject with `E5006` rather than parsing and silently ignoring the flag; once implemented, sandbox policy permission for `effects.eval` still does not implicitly enable this compatibility switch |
| `--wasm-threads` | Later compatibility (opt-in only) | Reject with `E5006` until the threaded runtime profile exists; after that, still reject explicitly when unavailable on the selected target/engine |

## Phase Exit Criteria

These checklists keep the phase labels operational rather than purely descriptive.

### Phase 1 exit criteria
- One linked-WASM-payload compile/run pipeline works end-to-end for TS and JS inputs, with companion artifacts only where an artifact mode explicitly requires them.
- Repeated builds with the same pinned inputs and toolchain produce stable artifact bytes and stable machine-readable output ordering by default.
- `kali run`, `build`, `check`, `fmt`, `lint`, `test`, and `install` exist with stable core behavior.
- The Phase-1 **base library artifact** (`kali build --lib`) works end-to-end as the Phase-1 half of the shared **embedding-stability split**.
- The checker ships the bounded HM-style inference fragment promised for Phase 1 for locals, obvious unannotated parameters, and analyzable return types, while still falling back conservatively instead of doing open-ended whole-program search.
- Browser-targeted `check --api browser` and `build --bundle --api browser` work against the real browser ambient surface without implying DOM runtime support in Kali itself.
- That browser-targeted claim is backed by dedicated browser-targeted tests, including emitted-bundle smoke runs in a real browser harness rather than only mock DOM/unit tests.
- `kali check` / `build` / `run` / `test` all use the same early-phase API-surface maturity rules: Deno-supported, Node phase-gated, browser supported only for the documented browser-targeted check/bundle paths.
- Runtime sandbox enforcement and resource limits work for the documented Phase 1 host APIs.
- `check/build --sandbox` perform the documented Phase-1 policy-schema/config validation without overclaiming full inferred-effect-vs-policy checking yet.
- Unsupported dynamic features fail with the canonical feature-maturity diagnostic instead of silently degrading.
- Package support works for the documented pure JS/TS, statically linkable subset.

### Phase 2 exit criteria
- MIR is the canonical ownership/layout IR.
- `kali effects` emits the documented stable JSON report.
- Explicit effect annotations and `pure` checking are enabled for the built-in capability model.
- Compile/check-time effect-vs-policy validation works against the declarative policy schema, extending the existing Phase-1 policy-file/config validation path rather than replacing it.
- Stable public Rust embedding and C ABI surfaces are documented and shipped.
- The Phase-1 base `kali build --lib` artifact is promoted into the stable public library contract, including default WIT emission.
- Public library/component outputs emit the documented WIT interface contract, and the initial Component Model packaging path works end-to-end.

### Phase 3 exit criteria
- Specialization materially improves generated code for common generic/layout-heavy programs.
- Incremental compilation exists for realistic multi-module projects.
- Node compatibility covers a meaningful documented subset rather than isolated package anecdotes.
- Browser packaging/interoperability improves beyond the Phase 1 bundle baseline.

### Phase 4 exit criteria
- Dynamic-compatibility paths such as `eval` are implemented behind explicit compatibility switches.
- Advanced compatibility features preserve the sandbox/effect model instead of bypassing it.
- Proof coverage expands for the most security- and correctness-critical subsystems.

## Cross-Phase Command Continuity Rule

When a later-phase command or profile becomes user-visible, it inherits the same already-established axis splits unless this matrix explicitly overrides them:
- analysis-oriented commands default to `apiSurface = deno`
- browser support remains browser-targeted analysis/build first unless a standalone browser-runtime contract is added explicitly
- Node support remains phase-gated uniformly across `check` / `effects` / `build` / `run` / `test` until the documented Node subset exists
- adding a new command should reuse existing artifact/effect/policy contracts instead of inventing a near-duplicate workflow

This keeps Phase 1 exit criteria phase-correct while still making later command behavior predictable.

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
| Bounded HM-style inference for locals, obvious parameters, and analyzable returns | Phase 1 MVP | Early inference should improve materially on plain `tsc` local inference without requiring open-ended whole-program search |
| Stable built-in capability-effect reporting | Phase 2 target | `kali effects` and policy checking |
| Explicit `pure` / effect annotations | Phase 2 target | Built-in sandbox capability model first |
| Stable user-defined/custom effects in machine contracts | Later compatibility | Keep early schemas/policies simple |

### Host and Runtime Profiles

| Concern | Early canonical status | Notes |
|---|---|---|
| Deno-oriented standalone API surface (`--api deno`) | Phase 1 MVP | Default API surface for standalone execution; typically paired with the baseline single-threaded runtime profile |
| Invocation arguments in the standalone surface (`Deno.args`) | Phase 1 MVP | Part of the execution context rather than a separately policy-gated capability in schema v1 |
| Read-only `Deno.permissions` facade over resolved policy state | Phase 1 MVP | Canonical **observation-only compatibility facade**: report granted/denied capability state without interactive `request()` / `revoke()` escalation flows |
| Interactive permission escalation / revocation APIs | Rejected by default | The Phase 1 Deno-compatibility story is query-only; runtime prompt/escalation flows are outside the sandbox model |
| Read-only environment access in the Deno standalone surface | Phase 1 MVP | Exposes only the sandbox-permitted environment view |
| Web-baseline randomness subset (`crypto.getRandomValues`) | Phase 1 MVP | Covers the schema-v1 `effects.random` / `Random.GetBytes` capability without implying full Web Crypto support |
| Mutable environment access / process-environment mutation | Phase 3 target | Policy-controlled host mutation, not part of the Phase 1 baseline |
| Subprocess spawning and socket/listener networking | Phase 3 target | Shares the same sandbox/process/network maturity path as the corresponding capability rows above |
| Browser-targeted `check` and `build --bundle` | Phase 1 MVP | Browser-targeted builds execute against the real browser host via the browser host adapter, with browser ambient typings available during analysis/build but no standalone browser emulation; support claims require dedicated browser-targeted tests and real-browser bundle smoke coverage, and later browser-targeted analysis commands should reuse the same ambient typing layer and browser package-resolution context once their own rows become available |
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
| Browser-condition package resolution in supported browser-targeted analysis/build contexts | Phase 1 MVP | Shared browser package-resolution context for `check --api browser` and `build --bundle --api browser`: honor `exports` condition `browser` plus applicable `package.json#browser` replacement maps consistently |
| npm lifecycle scripts | Opt-in only | `kali install --allow-scripts`; install-time package hooks stay outside the normal runtime API-surface and project-policy contracts |
| Native/binary/bootstrap-dependent packages | Rejected by default | `--allow-scripts` must not silently broaden support to packages that need native code or platform-specific downloaded executables |
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
