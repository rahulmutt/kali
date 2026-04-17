# 19 — Feature Maturity

This document is the canonical matrix for features that are easy to describe inconsistently across architecture, runtime, package, and CLI specs.

If another spec needs to mention one of these features, it should link here for phase/status rather than restating a different maturity decision. When another chapter talks about whether a request is supported yet, it should prefer the shared **support-claim reading order** and **availability context** terms from [SPEC.md](../SPEC.md) instead of re-explaining the whole command/rung/API-surface/runtime-profile/compatibility combination each time.

Status-label spelling rule:
- docs should use the canonical labels from this chapter verbatim (`Phase 1 MVP`, `Phase 1 MVP (opt-in only)`, `Phase 2 target`, `Phase 3 target`, `Phase 4 compatibility`, `Later compatibility`, `Opt-in only`, `Later compatibility (opt-in only)`, `Rejected by default`) rather than near-duplicates such as `later-compatibility`
- `experimental` is not a canonical public maturity label in this spec set; if a chapter mentions implementation-only experiments or preview plumbing, that wording must not be used as a substitute for a matrix-owned availability/status claim

Phase-label reading rule:
- these labels name the **earliest support contract**, not the recommended implementation sequence
- if a feature is documented early for vocabulary/schema stability, that does not promote it into the current phase by itself
- use [SPEC.md](../SPEC.md)'s **Phase Contracts vs Implementation Order** guidance whenever roadmap sequencing and maturity labels might otherwise get conflated

Bootstrap-triage note:
- this matrix classifies **phase contracts** and **phase-gated breadth targets** after the normalization rules in [SPEC.md](../SPEC.md)
- it does **not** downgrade the top-level bootstrap **hard invariants** such as AOT-only compilation, the **Pure-Rust implementation contract**, no tracing/background GC, sandbox honesty, or deterministic machine contracts into optional toggles
- it should also be read together with the top-level **Phase-1 Explicit Non-Goals** guardrail in [SPEC.md](../SPEC.md), so broad bootstrap aspirations do not get mistaken for shipped Phase-1 breadth
- when a row about compatibility breadth appears to conflict with one of those hard invariants, the invariant wins and the breadth feature must be redesigned or remain gated
- some rows in this chapter refer to a command/flag/artifact family whose stable shape is already documented elsewhere; follow the shared **defined command family** rule from [SPEC.md](../SPEC.md): documented shape and actual availability are separate, and this matrix is the availability owner

## Status Labels

- **Phase 1 MVP** — required for the first practically useful implementation
- **Phase 1 MVP (opt-in only)** — required in the first practically useful implementation, but intentionally disabled by default and available only behind an explicit flag/config
- **Phase 2 target** — planned once ownership/effects infrastructure lands
- **Phase 3 target** — planned once specialization/ecosystem work lands
- **Phase 4 compatibility** — supported only in the advanced compatibility phase
- **Later compatibility** — intentionally deferred until semantics and cost are justified
- **Opt-in only** — supported only behind an explicit flag or config; when the earliest phase matters, prefer a phase-qualified label instead of leaving the phase implicit
- **Later compatibility (opt-in only)** — deferred until a later phase and, even then, enabled only behind an explicit runtime/profile switch
- **Rejected by default** — Kali may still recognize the surface (for example syntax, config, policy, flag, or command shape), but normal compile/run/command handling should fail unless a documented compatibility or availability path is enabled in a phase that actually implements it

## Status Interpretation Table

| Status label | Meaning in practice |
|---|---|
| Phase 1 MVP | Must ship in the first dependable end-to-end release |
| Phase 1 MVP (opt-in only) | Must exist in the first dependable release, but only behind an explicit flag/config and with a default-off posture |
| Phase 2 target / Phase 3 target / Phase 4 compatibility | Planned work for that phase; before then, reject with the canonical gating path rather than partially emulating it |
| Later compatibility | Intentionally deferred with no near-phase promise |
| Opt-in only | Implemented only behind an explicit flag/config once it exists; use this only when the surrounding row/table already makes the earliest phase obvious |
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
  - package compatibility → curated package corpus results for the claimed source-graph command/context combinations **and the claimed rung of the shared package-support ladder** from [SPEC.md](../SPEC.md) (including browser-targeted `check` / `build --bundle` when those package claims are made)
  - install workflow / opt-in npm lifecycle hooks → install-command integration tests for manifest/lock/materialization updates, explicit npm-target hook execution, clean/no-op rejection when **effective npm-scriptable install work** is empty, and invalid-combination coverage for raw-URL / JSR targets
  - registry-analysis commands (`package-effects`, `package-audit`) → command-shape/arity negatives, deterministic single-package version-selection tests, context-participation assertions, and JSON-contract coverage for native-JSON vs envelope-only output
  - host/runtime APIs → integration + sandbox/resource-limit coverage
  - the shared **Phase-1 browser-targeted command set** → browser-targeted `check` tests + browser-targeted `build --bundle` tests + emitted-bundle smoke runs in a real browser harness
  - CLI/JSON contracts → golden/snapshot/schema tests
- isolated demos or one package anecdote do **not** by themselves justify raising a feature's maturity wording
- likewise, later `package-effects` / `package-audit` coverage does **not** by itself justify broader package-compatibility wording for ordinary source-graph commands such as `check`, `build`, `run`, or `test`

Verification-baseline clarification:
- the Phase-1 **proof-ready** baseline is intentionally a repository/process claim first, not automatically a claim that hosted proof automation already exists
- before concrete proof workflow files land, the minimum evidence for that **proof-ready** row is the published `proofs/BOUNDARY.md` manifest plus its explicit proof-CI trigger policy
- the current repository proof state should be read from `proofs/BOUNDARY.md`, not from duplicated chapter prose
- a mechanized non-empty proof boundary counts as **proof-backed for the published boundary** while still leaving any later target it does not name outside the claim
- stronger **proof-backed** release/support claims for a wider boundary still require that the newly named claims be mechanized plus the corresponding proof jobs/evidence for the covered subset

This keeps “Phase 1 MVP” and later status labels tied to measurable behavior rather than intent alone.

## Phase-1 Shipped Surface Summary

This section is a compact reading aid for the most common Phase-1 question: **what is actually shipped end to end?**

It is intentionally narrower than the full command/profile matrix below:
- it lists only the core command/context families that should be treated as shipped in Phase 1,
- it keeps later documented command families visible as explicitly **not yet shipped**,
- the full matrix later in this chapter remains the normative owner for exact arity, inherited-context equivalence, and diagnostic precedence.

| Area | Phase 1 shipped surface | Not yet shipped in Phase 1 |
|---|---|---|
| Project workflow | `kali init`, `kali init --lib`, `kali install`, the opt-in schema-v1 **install-time npm-package hook path** via `kali install --allow-scripts` when the invocation has non-empty **effective npm-scriptable install work**, `kali fmt` *(including `--check`)*, `kali lint` *(including `--fix`)*, `kali check [files...]` *(including the project-discovery no-file form and explicit file sets)*, plus first-class `.js` source support under the bounded-inference contract | no automatic dependency repair outside `kali install` |
| Execution | `kali run <file>` and `kali test [files...]` in the shared **Default standalone context (schema v1)**, with supported `--sandbox` runtime enforcement on the Deno-oriented standalone surface | no standalone browser runtime/test contract; no Node execution path yet |
| Executable build | `kali build <file>` in the shared **Deno-oriented build context (schema v1)**, with shipped static policy validation on the supported `kali build --sandbox <policy> <file>` path | no non-bundle browser build mode; no Node executable build path yet |
| Export-oriented build / embedding | Phase-1 **base library artifact** via `kali build --lib <file>` in the shared **Deno-oriented build context (schema v1)** for **exact-version consumers**, only when Kali can determine a **statically known export surface**; shipped static policy validation also covers `kali build --lib --sandbox <policy> <file>`. Here, the Deno-oriented build context is the build/analysis default, not a claim that Phase-1 library outputs expose a Deno-specific public ABI. | no stable public Rust embedding API; no stable public WIT sidecar for plain `--lib`; no stable public C ABI or Component Model flow; no cross-version host-loading guarantee yet |
| Browser-targeted support | exactly the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md) — concretely, browser-targeted `check [files...]` plus `build --bundle <file>`, with the supported `--sandbox` variants and equivalent inherited-config forms — while keeping that canonical term as the cross-spec owner | no `run --api browser`; no `test --api browser`; no browser library/embed artifact modes |
| Effects/sandbox | internal sandbox-oriented effect bookkeeping; policy-schema/config validation for the shared **Phase-1 static policy-validation surface**; runtime enforcement for `run --sandbox` / `test --sandbox` | no stable public `kali effects`; no stable public `kali package-effects`; no inferred-effect-vs-policy rejection yet; no dry-run `run` / `test` effect-report workflow |
| Registry analysis | none shipped in Phase 1 | under the shared **registry-analysis command split**, no stable public single-package registry-analysis commands ship in Phase 1: `kali package-effects` remains the Phase-2 registry-analysis/effect-report command and `kali package-audit` opens in Phase 4 as the context-free registry-analysis/security-audit command |
| Verification | published **proof-boundary manifest**, proof-CI trigger policy, and **proof-ready** repository state | no **proof-backed** release/support claims while the boundary is still empty or unmechanized |

Interpretation shortcut:
- `kali package-effects` appears in both the **Effects/sandbox** and **Registry analysis** stories on purpose; follow the shared **`package-effects` dual classification** from [SPEC.md](../SPEC.md): by command/input shape it is a registry-analysis command, and by report/output contract it is also part of the later **public effect-report surface**.
- the shared **Phase-1 static policy-validation surface** likewise appears here on purpose: it is the whole static side in Phase 1, while `run/test --sandbox` remain the runtime-enforcement side. Attaching `--sandbox` to some other invalid or phase-gated build shape does not widen that surface.
- reading rule: use the effect row to answer **what kind of reporting surface it belongs to**, and the registry-analysis row to answer **what kind of command/input workflow it is**.

Use this summary to avoid broad bootstrap overreads, then drop to the canonical matrix below for exact command/context rows.

Release-note/support-claim shortcut:
- prefer naming the exact command/context/rung instead of saying a subsystem is simply “supported”
- the table below is a writing aid for summaries and release notes; the canonical matrix still owns exact status

| Vague claim | Preferred precise phrasing |
|---|---|
| “JavaScript support ships in Phase 1” | “Phase 1 ships **first-class JavaScript compilation with bounded inference**: `.js` is a real check/build/run input class with conservative fallback rules, not a parse-only compatibility lane.” |
| “browser support ships in Phase 1” | “Phase 1 ships the shared **Phase-1 browser-targeted command set** from `SPEC.md`; read that canonical term for the exact command boundary, supported `--sandbox` variants, and inherited-config equivalence rules.” |
| “packages work in browser mode” | “For the shared **Phase-1 browser-targeted command set**, package claims are usually **checkable** or **deployable-through-host**, not standalone-browser **executable** support.” |
| “embedding ships in Phase 1” | “Phase 1 ships only the export-oriented **base library artifact** via `kali build --lib` for **exact-version consumers**; the stable public embedding surface remains Phase 2.” |
| “effects support ships in Phase 1” | “Phase 1 may use internal sandbox-oriented effect bookkeeping and ships policy validation/runtime enforcement where documented, but the stable public effect-report surface is still gated; when it opens, reporting is through explicit commands (`kali effects <file>`, `kali package-effects <package>`), not a hidden project-discovery or `run --dry` workflow.” |
| “registry analysis ships in Phase 1” | “No stable public registry-analysis command ships in Phase 1; the documented next steps stay split on purpose: `kali package-effects <package>` is the Phase-2 registry-analysis/effect-report command, while `kali package-audit <package>` opens in Phase 4 as the context-free registry-analysis/security-audit command.” |
| “JSON output means the command already ships” | “JSON selectors do not create availability: once shipped, `kali effects` / `kali package-effects` are the later **native-JSON commands**, while `kali package-audit` is the Phase 4 schema-v1 **envelope-only JSON command**; before then, the ordinary command gate still wins.” |
| “npm lifecycle scripts ship in Phase 1” | “Phase 1 ships only the schema-v1 **install-time npm-package hook path**: `kali install --allow-scripts`, and only when the invocation has non-empty **effective npm-scriptable install work**. This does not widen ordinary package/runtime support or imply early `--api node` compatibility.” |
| “formal verification ships in Phase 1” | “Kali is **proof-backed for the published boundary**; read `proofs/BOUNDARY.md` for the current claim boundary.” |

Browser-build simplification note:
- `--bundle` is the browser-only executable packaging path in schema v1, not a generic multi-artifact build switch
- therefore `kali build --bundle --api node ...` stays an `E5008` command-shape contradiction rather than an early Node build lane, and `kali build --lib --api browser ...` stays an `E5008` browser-library contradiction rather than a hidden browser embedding path

Phase-1 sandbox-behavior reading aid:

| Command family | Phase 1 `--sandbox` meaning |
|---|---|
| `kali run <file>` / `kali test [files...]` | runtime sandbox enforcement |
| the shared **Phase-1 static policy-validation surface** | policy-schema/config validation only; for its browser-targeted `build --bundle` member, this remains browser-targeted static validation only and does not imply post-deployment runtime enforcement |

Phase 2 later extends the `check` / `build` rows with inferred-effect-vs-policy validation; this table is only the compact Phase-1 reading aid.

Maintenance note:
- when a row below says **Phase-1 browser-targeted command set**, reuse the exact command-boundary definition from [SPEC.md](../SPEC.md) instead of re-expanding the same command list inline
- that keeps the availability matrix aligned with the one canonical cross-spec term and reduces drift when that boundary is clarified later

## Canonical Matrix

| Feature | Status | Rationale |
|---|---|---|
| Latest published ECMA-262 lexical grammar (tokenization) | Phase 1 MVP; parser pending | Lexer fully implemented and tested; tokenization of all ECMA-262 forms supported; parser implementation in progress |
| Stage-3+/draft TC39 proposals beyond the latest published ECMA-262 edition | Rejected by default | Keep the “latest ECMA-262” promise scoped to published editions; proposal support needs an explicit proposal opt-in or its own maturity row instead of being implied by grammar tracking |
| Current-edition non-Annex-B semantics for features Kali marks as supported in a given command/profile | Phase 1 MVP | "Latest standard support" is not parser-only: once Kali claims a feature is supported for a command/profile, the supported path should aim at faithful current-edition semantics and be backed by the matching evidence track rather than by syntax acceptance alone |
| Static ESM `import` / `export` | Phase 1 MVP | Core module system |
| First-class JavaScript compilation with bounded inference | Phase 1 MVP | Required so `.js` projects are not forced to migrate to TypeScript before benefiting from Kali; this uses the shared **first-class JavaScript compilation** contract plus the shared **bounded inference contract** rather than open-ended whole-program search |
| Budgeted local/intra-module constraint solving inside the shared bounded inference contract | Phase 1 MVP | Early stronger-than-`tsc` inference may use deterministic bounded constraint solving where compile-time budgets stay predictable |
| Open-ended or unstable cross-module/public-API constraint solving | Phase 3 target | Exported/public boundaries should keep the shared **annotation-required inference boundary** until higher-cost solver work has an explicit later-phase contract |
| CommonJS module lowering | Phase 1 MVP | Needed for early npm package compatibility within the linked-artifact model |
| `require("literal")` | Phase 1 MVP | Rewritten during compilation when statically resolvable |
| Dynamic `require()` | Rejected by default | Conflicts with the early **linked-artifact model** |
| Broad npm compatibility for packages that expect more Node built-ins | Phase 3 target | Depends on broader `--api node` support beyond the Phase 1 package baseline |
| Literal-string `import()` | Phase 3 target | Can be lowered to the already-linked graph without runtime WASM module linking |
| Non-literal `import(expr)` | Later compatibility | Requires a dynamic host-mediated path and conservative effect handling |
| `eval` | Phase 4 compatibility | Parsed and effect-tracked earlier, but full runtime support is deferred; compatibility path is the schema-v1 `--compat eval` switch when implemented, and that path must still preserve the top-level no-language-level-JIT invariant |
| `Function()` constructor | Phase 4 compatibility | Same status as `eval`; schema v1 intentionally reuses the same `--compat eval` switch instead of introducing a second compatibility-feature name, and it inherits the same no-language-level-JIT constraint |
| AOT-only compilation model (no language-level JIT) | Phase 1 MVP | Foundational product constraint from the bootstrap brief; optimization work must preserve ahead-of-time compilation rather than introducing a language-level JIT path |
| No tracing/background GC in the execution model | Phase 1 MVP | Deterministic ownership/reference-counted strategies may exist where other chapters allow them, but tracing/background GC is outside the supported design |
| Pure-Rust implementation with no embedded C/C++ libraries | Phase 1 MVP | Follows the shared **Pure-Rust implementation contract** from [SPEC.md](../SPEC.md): Kali stays Rust-only from the project/toolchain point of view without treating ordinary platform runtime/system libraries reached through Rust bindings as spec violations |
| Stable build-mode vocabulary (`fast`, `release`, `release-advanced`) | Phase 1 MVP | The mode names and CLI/config surface ship in Phase 1 as compile-budget contracts; later phases deepen what `release` and `release-advanced` actually do without inventing a second optimization-mode vocabulary or implying that every later MIR/LTO/post-pass optimization already exists in Phase 1 |
| Standardized Kali-hosted execution engine: `wasmtime` | Phase 1 MVP | Early standalone execution and embedding target one documented pure-Rust engine so runtime behavior and testing have a single baseline |
| Alternative Kali-hosted execution engines beyond `wasmtime` | Later compatibility | Engine plurality is an implementation extension after the first documented runtime contract is stable; it must not weaken the language/runtime guarantees or the **Pure-Rust implementation contract** |
| Invocation arguments (`Deno.args`; later Node `process.argv`) | Phase 1 MVP | Treated as caller-supplied execution context rather than a separately policy-gated host capability in schema v1 |
| Read-only environment access (`Deno.env.get`, `Deno.env.toObject`, policy-filtered host env view) | Phase 1 MVP | Needed for practical standalone compatibility while still fitting the sandbox model |
| Read-only `Deno.permissions` facade (query-only over the shared **Deno-compatible permission descriptor subset (schema v1)**, returning only the shared **stable permission status subset (schema v1)**; no `request()` / `revoke()` escalation path) | Phase 1 MVP | Exposes Kali sandbox state for compatibility without interactive permission escalation, while avoiding synthetic Kali-only permission names inside the Deno facade; descriptor observations stay scoped to the currently modeled capability slice, so Phase 1 `env` is effectively env-read-only and Phase 1 `net` is effectively fetch-only rather than implied mutation/socket/listener promises |
| Interactive permission escalation APIs (`Deno.permissions.request()` / `revoke()` and similar prompt-style flows) | Rejected by default | Kali's sandbox model is resolved before execution; permission observation is supported in Phase 1, but runtime privilege negotiation/prompting is not. In schema v1 these remain **recognized-but-unavailable compatibility members** so checker/runtime behavior stays aligned on the canonical `E5006` path instead of drifting into ordinary missing-member behavior, and they are not implied roadmap targets unless a future sandbox model explicitly reopens them. |
| Web Crypto randomness subset (`crypto.getRandomValues`, mapping to the canonical `Random.GetBytes` effect / `effects.random` policy key) | Phase 1 MVP | Keeps the Phase 1 Web baseline aligned with the effect and sandbox schemas without overpromising the full Web Crypto surface |
| Mutable environment access (`Deno.env.set`, `process.env = ...`-style host mutation) | Phase 3 target | Widens the host contract and must remain policy-controlled |
| Subprocess spawning (`Deno.Command`, host `process_spawn`) | Phase 3 target | Requires explicit sandbox/process-budget integration |
| Socket/listener networking (`Network.Connect`, `Network.Listen`, `Deno.serve`) | Phase 3 target | Requires explicit network policy and concurrency controls |
| Process identity and process-control/working-directory APIs (`Deno.pid`, `process.pid`, `Deno.exit`, `Deno.cwd`, `Deno.chdir`) | Later compatibility | Deferred until a future schema/policy contract makes their sandbox and embedding behavior explicit |
| Internal sandbox-oriented effect bookkeeping | Phase 1 MVP | Follows the shared **effect-surface split** from [SPEC.md](../SPEC.md): Phase 1 may already maintain conservative built-in effect facts internally for sandbox-first implementation and later integration work without claiming a stable CLI/JSON reporting surface |
| Stable public built-in effect reporting / `kali effects` / `kali package-effects` | Phase 2 target | This is the reporting half of the Phase-2 **public effect-report surface** side of that same split: stable user-facing effect JSON as a conservative upper-bound report plus the reporting commands that expose it |
| Compile/check-time inferred-effect-vs-policy validation on `kali check --sandbox` / `kali build --sandbox` | Phase 2 target | This is the pass/fail half of the same Phase-2 **public effect-report surface**: it extends the existing Phase-1 policy-schema/config validation workflow instead of creating a second sandbox command family |
| Explicit effect annotations / `pure` | Phase 2 target | Initially scoped to the built-in sandbox capability model |
| User-defined/custom effect kinds in stable reports or policy validation/comparison | Later compatibility | Keep Phase 1-2 machine contracts limited to built-in sandbox-relevant effects |
| Algebraic effect declarations / handlers | Later compatibility | Reserved later surface; must not block delivery of the core capability/effect system |
| Executable project-local sandbox policy code (`kali.policy.json` hooks / inline predicates) | Rejected by default | Project policy files stay declarative data; Kali should not execute project code just to decide whether a capability is allowed |
| Host-registered sandbox policy predicates | Later compatibility | This is the long-term programmable-policy path: initial policies stay declarative, and a later embedding-only extension may add pure host-registered predicates without turning policy files into executable project code. These predicates are narrowing-only: they may reject operations the declarative policy would otherwise allow, but they must not widen declarative denies or bypass command/profile maturity gates. |
| Published **proof-boundary manifest** + proof-CI trigger policy for the current published proof boundary | Phase 1 MVP | This is the Phase-1 **proof-ready** baseline: Kali needs an explicit proof boundary at `proofs/BOUNDARY.md` plus the proof-CI trigger policy for whatever boundary is currently published there. During pre-proof iteration that boundary may still be empty; once the Lean tree exists, it may instead be a provisional non-empty boundary, and in both cases the practical requirement is the published boundary plus its honest trigger policy rather than already-wired hosted proof automation |
| Proof-backed release/support claims while `proofs/BOUNDARY.md` is still the shared **placeholder proof-boundary manifest** | Rejected by default | The proof-ready baseline alone is not enough for proof-backed marketing. Until `proofs/BOUNDARY.md` names at least one concrete modeled subsystem plus theorem/property inventory, release notes, README summaries, and support claims must not present formal verification as a shipped Kali capability |
| Proof-backed release/support claims while `proofs/BOUNDARY.md` is still a provisional non-empty proof boundary | Rejected by default | The published boundary may already name concrete modeled subsystems, but while those claims remain provisional or documented-sorry placeholders they are still only proof-ready; release notes, README summaries, and support claims must not present them as shipped mechanized evidence |
| Broader proof coverage for ownership/effects/lowering beyond the initial published proof boundary | Phase 2 target | Align verification expansion with the ownership/effects phase instead of letting proof claims drift ahead of the modeled semantics |
| Full-language or full-host formal verification claims | Later compatibility | The project should not imply end-to-end proof coverage for the whole JS/TS surface, dynamic compatibility paths, or concrete host integrations until those semantics are modeled honestly |
| Annex B / web-legacy compatibility corners | Later compatibility | Keep the MVP focused on dependable core semantics; add legacy web behaviors only when conformance value justifies the cost |
| `Proxy` | Later compatibility | High semantic cost and optimization barriers |
| `WeakMap` / `WeakSet` | Later compatibility | Deferred until weak-reference semantics fit the no-tracing-GC design |
| `FinalizationRegistry` | Later compatibility | Same reason as weak collections |
| `SharedArrayBuffer` / `Atomics` | Later compatibility (opt-in only) | Requires a separate threaded runtime profile and should not be implied by the Phase 1 single-threaded runtime |
| `--wasm-threads` | Later compatibility (opt-in only) | Enables the threaded runtime profile once that profile exists; must fail explicitly before then and on unsupported targets/engines |
| Real browser ambient surface for the **Phase-1 browser-targeted command set**, including ambient DOM typings | Phase 1 MVP | Phase 1 exposes the real browser ambient surface only through the shared **Phase-1 browser-targeted command set**, including the DOM typings normally expected in browser programs, without claiming DOM support in Kali's standalone runtime; this status requires its own browser-targeted evidence track rather than inference from standalone runtime tests, and later analysis commands may reuse that same browser context only once their own maturity rows allow it |
| `package.json#exports` condition `deno` for `--api deno` resolution | Phase 1 MVP | Aligns package resolution with the Deno-oriented standalone surface |
| `package.json#browser` replacement maps and `exports` condition `browser` for the **Phase-1 browser-targeted command set** | Phase 1 MVP | Needed for practical browser-targeted npm compatibility without widening standalone runtime claims; the shared **Phase-1 browser-targeted command set** should use one browser **package-resolution context** (browser `exports` condition order plus any applicable `package.json#browser` rewrites) rather than inventing per-command ladders |
| `run --api browser` | Later compatibility | Early standalone runtime does not emulate a browser host; reject with `E5006` until a real browser-execution contract exists |
| npm lifecycle scripts (`kali install --allow-scripts`) | Phase 1 MVP (opt-in only) | Disabled by default for sandbox-first behavior; this uses the shared **install-time npm-package hook path** from [SPEC.md](../SPEC.md), not evidence of `--api node` support or participation in the normal sandbox/effect-report contract |
| Automatic dependency installation or lockfile/materialization repair during `check` / `effects` / `build` / `run` / `test` | Rejected by default | Keeps dependency state deterministic and makes `kali install` the single mutating dependency-management command; missing/stale state should fail with `E5004` instead of being repaired implicitly |
| Packages whose normal install/runtime path falls into the shared **native/binary/bootstrap-heavy package contract** | Rejected by default | Use the shared **published-artifact-first package reading** from [SPEC.md](../SPEC.md): this row applies when the published package Kali installs still depends on native/binary/bootstrap steps for normal install/runtime behavior. That falls outside the shared **pure JS/TS package contract**, weakens deterministic install expectations, and must not be implied by `--allow-scripts` |
| npm packages that require unsupported Node core modules | Phase 3 target | Depends on broader `--api node` compatibility work |
| Phase-1 **base library artifact** (`kali build --lib`) | Phase 1 MVP | This is the Phase-1 side of the shared **embedding-stability split**: projects can build non-executable exported modules early without waiting for the later **public embedding surface** to freeze, but only when Kali can determine a **statically known export surface** after frontend lowering. That early artifact is for **exact-version consumers** only and is still not a stable public ABI/WIT or cross-version host-loading promise until the Phase-2 contract lands |
| Stable public Rust embedding API | Phase 2 target | Part of the Phase-2 **public embedding surface** side of that same split |
| Stable public library/WIT contract for `kali build --lib` | Phase 2 target | The same `--lib` selector is promoted from the Phase-1 **base library artifact** into the stable public **WIT-first** library contract and emits WIT by default once that interface surface is frozen |
| Stable public C ABI / `kali build --capi` flow | Phase 2 target | Part of the same Phase-2 **public embedding surface** stabilization work |
| WIT emission for public library/embedding interfaces | Phase 2 target | Gives the Phase-2 **public embedding surface** one canonical exported interface description instead of parallel ad hoc metadata |
| WebAssembly Component Model packaging (`kali build --component`) | Phase 2 target | Part of the same Phase-2 **public embedding surface** set, layered on top of the linked core WASM payload for host interop; executable builds still center on the core module path |
| Implicit Component Model packaging for every public library build | Rejected by default | Keep plain public `--lib` + WIT as the stable default library contract once Phase 2 lands; `--component` stays an explicit packaging choice so hosts do not silently opt into a different loading model |
| Host ABI versioning for `kali_capi` | Phase 2 target | Stable embedding requires explicit load-time compatibility checks |
| DOM APIs in standalone runtime | Rejected by default | Kali does not embed a browser engine |

## Interpretation Rules

1. **Single-payload rule**: builds in Phases 1-3 target one linked WASM payload for the resolved static graph. Artifact modes may still add companion artifacts such as JS glue, WIT files, component wrappers, or C headers, but they must not reintroduce runtime WASM module linking.
2. **Parse vs support**: accepted syntax does not imply full runtime support; unsupported dynamic features should be diagnosed explicitly.
3. **Effect boundaries**: features marked as dynamic compatibility paths should be reflected in static effect analysis.
4. **No silent fallback**: if a feature cannot be implemented faithfully under the current phase constraints, Kali should reject or gate it rather than emulate it loosely.
5. **Policy alignment**: sandbox policy validation may always deny a capability, but it must reject policies that try to enable capabilities unavailable in the selected command/profile/phase.
6. **Canonical gating diagnostic**: use the shared feature-maturity diagnostic contract (`E5006`) so CLI, checker, runtime, and package tooling report availability gating consistently.
7. **Do not overuse `E5006`**: selecting an unavailable command/profile/feature uses `E5006`, but ordinary references to names/globals that are simply absent from the selected supported ambient surface should use the normal name/type diagnostics instead.
8. **Sandbox-domain honesty**: build-time policy compatibility for browser-targeted artifacts must not be described as equivalent to Kali-hosted runtime sandbox enforcement.

## Canonical Command/Profile Matrix

This table exists to stop drift between CLI examples, runtime behavior, package tooling, and error reporting.

Interpretation rule:
- matrix rows are evaluated against the fully merged **effective command context** (built-in defaults, then discovered config, then CLI flags); public support claims should describe the resulting **availability context** from [SPEC.md](../SPEC.md) rather than re-expanding every participating axis inline
- some rows refer to **defined command families** from [SPEC.md](../SPEC.md): their command shape may already be documented in [12 — CLI](12-cli.md) or other owning chapters even when this matrix still marks them with a later canonical status label such as `Phase 2 target` or `Later compatibility`
- examples written with explicit flags also apply when the same value was inherited from `kali.json`
- for the shared source-graph command axes `--wasm-threads` and `--compat eval`, one representative explicit-flag row may stand in for the same maturity gate across `check` / `effects` / `build` / `run` / `test` unless a later row carves out a command-specific exception; inherited `compilerOptions.runtimeProfiles` / `compat.features` must hit that same gate rather than being silently dropped
- unless a row explicitly says otherwise, a plain command spelling without an inherited-context qualifier is read under the command's canonical default **availability context** (derived from its canonical default effective context); inherited non-default contexts that change the outcome should get their own explicit rows instead of being hidden inside a rationale sentence
- canonical reading example: discovered `compilerOptions.apiSurface = browser` makes plain `kali check main.ts` map to the same supported browser-targeted analysis row as explicit `kali check --api browser main.ts`, and makes `kali build --bundle main.ts` map to the same supported row as explicit `kali build --bundle --api browser main.ts`; that same inherited browser value still makes plain `kali build main.ts` fail on the browser build-shape contradiction path (`E5008`) until a non-bundle browser build mode exists
- only the axes that participate for the selected command are maturity-relevant; non-participating inherited axes are ignored rather than becoming hidden gates or contradictions
- Kali must not silently fall back from an inherited unsupported/contradictory participating context to a different API surface/profile just because the user omitted the matching CLI flag
- browser rows follow the top-level **canonical browser-surface rejection split** from [SPEC.md](../SPEC.md): wrong browser build shapes are `E5008`, while unsupported browser execution/test/runtime contracts are `E5006`
- `--sandbox` rows follow the shared **sandbox-attachment orthogonality** rule from [SPEC.md](../SPEC.md): sandbox attachment never creates a second command family, browser build shape, library mode, or inherited-context fallback path
- follow the canonical validation-order rule from [SPEC.md](../SPEC.md): **command shape/arity first**, then the command's own phase availability, then finer-grained inherited-context/profile gates inside that command
- output-format selectors such as `--output json` and `--pretty` never create a second availability path: after any earlier command-shape error, they inherit the same base maturity row as the underlying command/context combination and change presentation only
- matrix-row status names the **earliest phase where the full command/context combination can be supported**, not necessarily the first diagnostic a pre-support implementation should report when more than one independent gate is still closed
- for `build` rows, the selected artifact mode also fixes the shared **compile intent** from [SPEC.md](../SPEC.md): default/no selector and `--bundle` are executable compile-intent paths, while `--lib` / `--capi` / `--component` are library compile-intent paths
- in this command/profile matrix, the status label is a planning/maturity summary, not by itself the diagnostic choice: rows marked **Rejected by default** may still fail as `E5008` invalid usage or `E5006` unavailable-feature gating depending on the canonical handling column and the shared validation-order rules
- for example, `kali build --capi --api node lib.ts` is listed as a **Phase 3 target** because that full combination cannot work before both the Phase-2 **public embedding surface** and the Node surface exist, but an early implementation should still report the outermost failing gate first (`--capi` itself in Phase 1, then `--api node` once `--capi` exists but Node remains gated)
- this keeps diagnostics stable for commands such as `package-effects`: before Phase 2, plain `kali package-effects lodash` should fail on the command's base maturity row; once the command exists, inherited-context maturity follows the shared **axis-aligned inherited analysis gating** rule from [SPEC.md](../SPEC.md) instead of a package-analysis-specific shadow matrix
- simplification rule for later analysis/reporting rows: `effects` may have explicit `--api ...` / `--compat ...` / `--wasm-threads` forms once its command exists, but schema-v1 `package-effects` keeps those semantics **inherited-only**. So browser/Node/threaded/`eval` package-analysis rows below are inherited-context examples, not permission to add a second package-analysis flag family.
- apply that same reading to later **defined command families** such as `effects`: before the command itself ships, the base command gate wins even if an inherited context would eventually be supported (for example browser analysis); once the command exists, supported inherited contexts should behave exactly like their explicit-flag forms instead of being reinterpreted as hidden fallback requests

| Command / profile | Early-phase status | Canonical handling |
|---|---|---|
| `kali init` | Phase 1 MVP | Follow the shared **minimal canonical scaffold contract** from [SPEC.md](../SPEC.md): create the smallest valid schema-v1 project scaffold in the current working directory, using the canonical built-in starter filenames (`main.ts` by default, `lib.ts` with `--lib`), without adding dependencies, writing `kali.lock`, or materializing packages. |
| `kali init` when the current working directory already contains `kali.json` | Rejected by default | Fail with `E5008` instead of silently overwriting the existing project config |
| `kali init` in a subdirectory whose ancestor already contains `kali.json` | Phase 1 MVP | Create a nested child project rooted at the current working directory when that directory itself does not already contain `kali.json`; later discovery treats that child root as a separate project boundary |
| `kali init --sandbox kali.policy.json` | Rejected by default | `init` is sandbox-agnostic in early phases; scaffolding does not accept the runtime/build policy-attachment flag, so this is invalid usage (`E5008`) |
| `kali init --lib` | Phase 1 MVP | Follows the shared **minimal canonical scaffold contract** plus the **template selection vs build artifact mode split** from [SPEC.md](../SPEC.md): selects the library-oriented scaffold template only and does not switch later plain `kali build` invocations into library mode |
| `kali fmt` | Phase 1 MVP | Stable formatting command over the canonical project file set relevant to formatting, including declaration-only files |
| `kali fmt --check` | Phase 1 MVP | Same file/discovery contract as `kali fmt`, but report formatting drift without rewriting files; this is the canonical CI-friendly/read-only formatting path rather than a later add-on mode |
| `kali fmt --sandbox kali.policy.json` | Rejected by default | `fmt` is sandbox-agnostic in early phases; top-level config sandbox is ignored for it, and the CLI `--sandbox` flag is invalid usage (`E5008`) |
| `kali lint` | Phase 1 MVP | Stable lint command with conservative autofix support over the canonical project file set, including declaration-only files |
| `kali lint --fix` | Phase 1 MVP | Apply only structured non-speculative lint fixes; overlapping fixes stay unapplied rather than being guessed into one rewrite in schema v1 |
| `kali lint --sandbox kali.policy.json` | Rejected by default | `lint` is sandbox-agnostic in early phases; top-level config sandbox is ignored for it, and the CLI `--sandbox` flag is invalid usage (`E5008`) |
| `kali install` | Phase 1 MVP | Reconcile the project's managed dependency state: update dependency-owning manifest fields when an explicit registry target is added, resolve/materialize dependency state for the declared dependency source kinds, and write `kali.lock`; install is profile-agnostic in early phases and does not require separate per-`--api` installs |
| `kali install <registry-package>` | Phase 1 MVP | Add one explicit registry package identifier (for example npm or `jsr:`) to `dependencies`, then refresh lock/materialized state. In the shared **configless install split** from [SPEC.md](../SPEC.md), first create the minimal canonical manifest `{ "schemaVersion": 1 }` at the effective project root, then record the dependency and materialize/install it. |
| `kali install --dev <registry-package>` | Phase 1 MVP | Same as `kali install <registry-package>`, but record the dependency under `devDependencies` instead of `dependencies`. In canonical configless project mode, first create the minimal manifest `{ "schemaVersion": 1 }` at the effective project root before recording the dependency. |
| plain `kali install` in canonical configless project mode with no dependency inputs | Phase 1 MVP | Follows the shared **configless install split** from [SPEC.md](../SPEC.md): succeed as a no-op and do not create a placeholder `kali.json`; running the command alone is not a request to scaffold a project |
| `kali install foo bar` | Rejected by default | Early phases accept at most one explicit install target for `install`; batch package adds or multi-target installs require a later explicit mode, so this is invalid command usage (`E5008`) |
| `kali install --dev` | Rejected by default | `--dev` modifies an explicit registry package target in early phases; using it without one is invalid command usage (`E5008`) |
| `kali install --api ...` | Rejected by default | `install` is profile-agnostic in early phases, so `--api` is invalid command usage (`E5008`) rather than a second install mode |
| `kali install --sandbox kali.policy.json` | Rejected by default | `install` is sandbox-agnostic in early phases; top-level config sandbox is ignored for it, but the CLI `--sandbox` flag is not accepted here and should fail with `E5008` |
| `kali install https://...` | Phase 1 MVP | Explicitly pin/materialize a raw URL dependency into the shared lock/materialization model; in configless mode this follows the shared **configless install split** from [SPEC.md](../SPEC.md): it may still create `kali.lock` and `.kali/cache/urls/` state, but it does not scaffold a placeholder manifest |
| `kali install --dev https://...` | Rejected by default | `--dev` applies only to explicit registry-package targets in early phases; pairing it with a raw URL is invalid command usage (`E5008`) rather than a second raw-URL manifest mode |
| `kali install --allow-scripts` | Phase 1 MVP (opt-in only) | Valid when the invocation has non-empty **effective npm-scriptable install work** from [SPEC.md](../SPEC.md); that invocation-scoped npm install work is the only part of the command allowed onto the shared **install-time npm-package hook path**, even when the same install also touches JSR/raw-URL inputs |
| `kali install --allow-scripts lodash` (or another explicit npm package target) | Phase 1 MVP (opt-in only) | Canonical explicit npm-target shape for the shared **install-time npm-package hook path**; still does not broaden support beyond otherwise eligible npm install work |
| `kali install --allow-scripts` when effective npm-scriptable install work is empty | Rejected by default | If the invocation has no effective npm-scriptable install work for the flag to affect, fail with `E5008` instead of silently behaving like plain `install` |
| `kali install --allow-scripts https://...` | Rejected by default | Raw URLs do not have npm lifecycle hooks, so pairing `--allow-scripts` with a raw URL is invalid command usage (`E5008`) rather than a second install mode |
| `kali install --allow-scripts jsr:@std/path` | Rejected by default | JSR packages do not participate in npm lifecycle-script execution in schema v1, so this flag/target combination is invalid command usage (`E5008`) |
| non-install command auto-repair of missing/stale dependency state | Rejected by default | `check` / `effects` / `build` / `run` / `test` must fail with `E5004` and point users to `kali install` instead of mutating dependency state opportunistically |
| `kali run` with no explicit entrypoint | Rejected by default | `run` is a direct-input command in early phases; omitting the entrypoint should fail with `E5008` rather than guessing `main.ts` or scanning the project |
| `kali run a.ts b.ts` | Rejected by default | Early phases accept exactly one primary runtime entrypoint; multi-entry execution requires a later explicit mode, so this should fail with `E5008` |
| `kali run main.ts` | Phase 1 MVP | Compile and execute with the shared **Default standalone context (schema v1)** from [SPEC.md](../SPEC.md) |
| `kali run --sandbox kali.policy.json main.ts` | Phase 1 MVP | Runtime sandbox enforcement path; policy schema/ranges must validate before execution starts |
| `kali run --api deno main.ts` | Phase 1 MVP | Supported standalone runtime path |
| `kali run --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset lands |
| plain `kali run main.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali run --api node main.ts`; inherited config must not silently fall back to `deno` for execution |
| plain `kali run --sandbox kali.policy.json main.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali run --api node --sandbox kali.policy.json main.ts`; attaching `--sandbox` does not bypass the Node execution gate |
| `kali run --api browser main.ts` | Later compatibility | Reject with `E5006` until a real browser-execution contract exists; Phase 1 browser support is limited to the shared **Phase-1 browser-targeted command set** first |
| plain `kali run main.ts` under an inherited browser API surface | Later compatibility | Same effective request as explicit `kali run --api browser main.ts`; inherited config must not silently fall back to `deno` for execution |
| plain `kali run --sandbox kali.policy.json main.ts` under an inherited browser API surface | Later compatibility | Same browser-execution gate as explicit `kali run --api browser --sandbox kali.policy.json main.ts`; attaching `--sandbox` does not turn browser into an early standalone runtime |
| `kali check` | Phase 1 MVP | Type-check the canonical project-discovery result under the shared **default source-graph analysis context (schema v1)** from [SPEC.md](../SPEC.md) |
| `kali check main.ts` | Phase 1 MVP | Type-check an explicit file input under the shared **default source-graph analysis context (schema v1)** from [SPEC.md](../SPEC.md) |
| `kali check a.ts b.ts` | Phase 1 MVP | `check` follows the shared **set-oriented explicit-file command** rule in early phases: multiple explicit files are allowed and should be checked as one explicit file set rather than rejected as though `check` were a single-entry direct command |
| `kali check types.d.ts` | Phase 1 MVP | Declaration-only files are valid explicit file inputs for `check`, even though they are not valid runtime entrypoints, build/effect primary inputs, or test entrypoints |
| `kali check --sandbox kali.policy.json` | Phase 1 MVP | Reuse the same project-discovery behavior as plain `kali check`; Phase 1 validates policy-schema/config over the same **resolved source graph** from [SPEC.md](../SPEC.md), and from the Phase 2 target onward the same path also checks inferred effects against the policy |
| `kali check --sandbox kali.policy.json main.ts` | Phase 1 MVP | Same validation path, but over the **resolved source graph** selected by that explicit file set rather than the project-discovery case |
| `kali check --api node` | Phase 3 target | Reject with `E5006` until the documented Node typing/global subset exists; `check` keeps its project-discovery no-file form here too rather than inventing a node-specific direct-input requirement |
| plain `kali check` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali check --api node`; inherited config must not silently fall back to `deno` for checking |
| `kali check --api node main.ts` | Phase 3 target | Same Node analysis gate for an explicit file set |
| plain `kali check main.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali check --api node main.ts`; inherited config must not silently fall back to `deno` for explicit-file checking |
| `kali check --api node --sandbox kali.policy.json` | Phase 3 target | Same Node availability gate for the project-discovery policy-validation form; attaching `--sandbox` does not bypass the gated API surface |
| plain `kali check --sandbox kali.policy.json` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali check --api node --sandbox kali.policy.json`; inherited config must not silently fall back to `deno` when `--sandbox` is present |
| `kali check --api browser` | Phase 1 MVP | Supported browser-targeted analysis context over the canonical project-discovery result; browser targeting changes the analysis context, not `check`'s shared **hybrid analysis command** behavior |
| plain `kali check` under an inherited browser API surface | Phase 1 MVP | Same supported browser-targeted project-discovery request as explicit `kali check --api browser`; this row makes the inherited-config equivalence explicit instead of leaving it only in matrix prose |
| `kali check --api browser main.ts` | Phase 1 MVP | Supported browser-targeted analysis context for an explicit file set |
| plain `kali check main.ts` under an inherited browser API surface | Phase 1 MVP | Same supported browser-targeted explicit-file analysis request as explicit `kali check --api browser main.ts`; effective-context inheritance must not silently fall back to `deno` |
| `kali check --api browser a.ts b.ts` | Phase 1 MVP | The shared **Phase-1 browser-targeted command set** keeps `check`'s ordinary **set-oriented explicit-file command** behavior; browser targeting changes the analysis context, not the file-arity model |
| plain `kali check a.ts b.ts` under an inherited browser API surface | Phase 1 MVP | Same browser-targeted multi-file analysis request as explicit `kali check --api browser a.ts b.ts`; inherited browser context must not collapse `check` back to a single-file or `deno` interpretation |
| `kali check --wasm-threads main.ts` | Later compatibility (opt-in only) | `check` participates in runtime-profile-sensitive analysis when relevant; reject with `E5006` until the threaded profile exists and the selected analysis mode supports it |
| `kali check --api browser --sandbox kali.policy.json` | Phase 1 MVP | Browser-targeted static policy validation over the same **resolved source graph** from [SPEC.md](../SPEC.md), following the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md) |
| plain `kali check --sandbox kali.policy.json` under an inherited browser API surface | Phase 1 MVP | Same supported browser-targeted project-discovery validation request as explicit `kali check --api browser --sandbox kali.policy.json`; effective-context inheritance must not silently fall back to `deno` when `--sandbox` is present |
| `kali check --api browser --sandbox kali.policy.json main.ts` | Phase 1 MVP | Same browser-targeted static policy validation path, but over the **resolved source graph** selected by that explicit file set rather than the project-discovery case and still following the **browser-targeted static sandbox contract** |
| plain `kali check --sandbox kali.policy.json main.ts` under an inherited browser API surface | Phase 1 MVP | Same supported browser-targeted explicit-file validation request as explicit `kali check --api browser --sandbox kali.policy.json main.ts`; effective-context inheritance must not silently fall back to `deno` when `--sandbox` is present |
| `kali check --api browser --sandbox kali.policy.json a.ts b.ts` | Phase 1 MVP | Browser-targeted static policy validation keeps the same **set-oriented explicit-file command** behavior for explicit multi-file sets; `--sandbox` does not create a browser-specific single-entry mode |
| plain `kali check --sandbox kali.policy.json a.ts b.ts` under an inherited browser API surface | Phase 1 MVP | Same browser-targeted multi-file static policy-validation request as explicit `kali check --api browser --sandbox kali.policy.json a.ts b.ts`; inherited browser context must not silently fall back when `--sandbox` is present |
| `kali check --sandbox kali.policy.json a.ts b.ts` | Phase 1 MVP | `check` keeps that shared **set-oriented explicit-file command** behavior under `--sandbox`; this validates the supplied file set rather than inventing a single-entry mode |
| `kali check --fix` | Later compatibility | The checker may emit structured fix metadata earlier, but schema v1 keeps CLI autofix lint-only to avoid unstable multi-diagnostic rewrite semantics |
| `kali build` with no explicit primary source input | Rejected by default | `build` is a direct-input command in early phases; omitting the primary source input should fail with `E5008` rather than guessing `main.ts` or scanning the project |
| `kali build a.ts b.ts` | Rejected by default | Early phases accept exactly one primary build source input; multi-entry artifact modes require a later explicit spec, so this should fail with `E5008` |
| `kali build main.ts` | Phase 1 MVP | Produce one linked WASM payload with the shared **Deno-oriented build context (schema v1)** from [SPEC.md](../SPEC.md) and the default executable artifact mode / executable compile intent |
| plain `kali build main.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --api node main.ts`; inherited config must not silently fall back to `deno` for builds |
| plain `kali build main.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --api browser main.ts`, so this remains the browser build-shape contradiction path (`E5008`) until a non-bundle browser build mode exists |
| `kali build --sandbox kali.policy.json main.ts` | Phase 1 MVP | Phase 1 validates policy-schema/config for the build; from the Phase 2 target onward the same path also performs effect-vs-policy validation |
| plain `kali build --sandbox kali.policy.json main.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --api node --sandbox kali.policy.json main.ts`; attaching `--sandbox` does not bypass the Node build gate |
| plain `kali build --sandbox kali.policy.json main.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --api browser --sandbox kali.policy.json main.ts`, so this remains the browser build-shape contradiction path (`E5008`) until a non-bundle browser build mode exists |
| `kali build --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset lands for builds too |
| `kali build --bundle --api browser main.ts` | Phase 1 MVP | Supported browser artifact path; this keeps executable compile intent while switching to the browser-targeted output/host-adapter shape (`kind: wasm-module`, `role: primary-executable` + `kind: js-glue`, `role: browser-glue`) |
| `kali build --bundle --api browser --sandbox kali.policy.json main.ts` | Phase 1 MVP | Browser-targeted static policy validation path only, following the **browser-targeted static sandbox contract** from [SPEC.md](../SPEC.md) |
| `kali build --bundle main.ts` under an effective API surface other than `browser` | Rejected by default | Invalid command usage (`E5008`): `--bundle` is reserved for browser-targeted output and therefore requires the effective API surface to be `browser` |
| plain `kali build --bundle main.ts` under an inherited browser API surface | Phase 1 MVP | The same plain command spelling becomes the supported browser-bundle shortcut once the effective API surface already resolves to `browser`, matching explicit `kali build --bundle --api browser main.ts` |
| `kali build --bundle --sandbox kali.policy.json main.ts` under an effective API surface other than `browser` | Rejected by default | Invalid command usage (`E5008`): attaching `--sandbox` does not change the browser-only meaning of `--bundle`, so the effective API surface must still be `browser` |
| plain `kali build --bundle --sandbox kali.policy.json main.ts` under an inherited browser API surface | Phase 1 MVP | The same plain command spelling becomes the supported browser-targeted static policy-validation request once the effective API surface already resolves to `browser`, matching explicit `kali build --bundle --api browser --sandbox kali.policy.json main.ts` |
| `kali build --bundle --api node main.ts` | Rejected by default | Invalid command usage (`E5008`): browser bundle mode exists, but pairing `--bundle` with an explicit non-browser API surface is a contradictory command shape rather than a separate maturity-gated runtime mode |
| `kali build --api browser main.ts` | Rejected by default | Invalid command usage (`E5008`) in early phases: the browser-targeted context is available for `check` and `build --bundle`, but a non-bundled browser build mode is not defined yet |
| `kali build --api browser --sandbox kali.policy.json main.ts` | Rejected by default | Same browser build-shape contradiction as `kali build --api browser main.ts`; attaching `--sandbox` adds only static policy validation and does not create a non-bundle browser build mode. |
| `kali build --lib lib.ts` | Phase 1 MVP | Produce one linked export-oriented **base library artifact** and therefore explicit library compile intent, following the shared **library-oriented instantiation rule** from [SPEC.md](../SPEC.md). This row applies only when Kali can determine the required **statically known export surface** after frontend lowering; otherwise the build falls to the separate `E5011` rejection row below. Phase 1 emits the base `wasm-module` (`role: primary-library`) only, and support claims for that artifact stop at **exact-version consumers**; the stable public library/WIT and cross-version host-loading surface is still Phase 2 work and then adds the default `wit` sidecar (`role: interface-wit`). The default `apiSurface = deno` here is the build/analysis context, not a Deno-specific ABI brand on the emitted library artifact. |
| `kali build --lib --sandbox kali.policy.json lib.ts` | Phase 1 MVP | Same Phase-1 base library artifact path plus the ordinary build-time static policy-validation path. On `build`, `--sandbox` is orthogonal to artifact mode: it does not change compile intent, export-surface rules, or API-surface gating. |
| library-oriented build with a **statically known export surface** after frontend lowering | Phase 1 MVP | Uses the shared definition from [SPEC.md](../SPEC.md): ESM exports are direct, while CommonJS participates only when static lowering can determine one fixed export set |
| library-oriented build without a statically known export surface | Rejected by default | Fail with `E5011` instead of inventing reflection-based exports for `--lib` / `--capi` / `--component` |
| `kali build --lib --api node lib.ts` | Phase 3 target | Library-oriented build modes still obey the same Node build gate as ordinary `kali build --api node ...`; they do not create a separate early Node surface |
| `kali build --lib --api node --sandbox kali.policy.json lib.ts` | Phase 3 target | Same Node build gate as `kali build --lib --api node lib.ts`; attaching `--sandbox` adds static policy validation only and does not bypass that gate. |
| `kali build --lib --api browser lib.ts` | Rejected by default | Early browser support is limited to the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md); browser-library artifact modes remain outside that canonical boundary |
| `kali build --lib --api browser --sandbox kali.policy.json lib.ts` | Rejected by default | Same browser-library contradiction as `kali build --lib --api browser lib.ts`; attaching `--sandbox` adds static policy validation only and does not create a browser library build mode. |
| plain `kali build --lib lib.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --lib --api node lib.ts`; inherited config must not silently fall back to a non-Node library build |
| plain `kali build --lib --sandbox kali.policy.json lib.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --lib --api node --sandbox kali.policy.json lib.ts`; inherited config must not silently fall back to a non-Node library build when `--sandbox` is present. |
| plain `kali build --lib lib.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --lib --api browser lib.ts`; inherited browser context must not silently fall back to a non-browser library build |
| plain `kali build --lib --sandbox kali.policy.json lib.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --lib --api browser --sandbox kali.policy.json lib.ts`; inherited browser context must not silently fall back to a non-browser library build when `--sandbox` is present. |
| `kali build --capi lib.ts` | Phase 2 target | Public embedding artifact generation should stay gated until the embedding contract is stable; when enabled it emits `kind: wasm-module` (`role: primary-library`) + `kind: wit` (`role: interface-wit`) + `kind: c-header` (`role: embedding-header`, the generated program-specific exports header rather than the host ABI header `kali.h`) + `kind: cabi-metadata` (`role: embedding-metadata`) |
| `kali build --capi --sandbox kali.policy.json lib.ts` | Phase 2 target | Same public embedding artifact flow plus the ordinary build-time static policy-validation path; on `build`, `--sandbox` stays orthogonal to artifact mode and does not change compile intent or API-surface gating. |
| `kali build --capi --api node lib.ts` | Phase 3 target | Public embedding artifact flows remain subject to the ordinary Node build gate; both the embedding flow and the selected API surface must be implemented |
| `kali build --capi --api node --sandbox kali.policy.json lib.ts` | Phase 3 target | Same Node build gate as `kali build --capi --api node lib.ts`; attaching `--sandbox` adds static policy validation only and does not bypass that gate. |
| plain `kali build --capi lib.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --capi --api node lib.ts`; once `--capi` exists, inherited config must not silently fall back to a non-Node embedding build |
| plain `kali build --capi --sandbox kali.policy.json lib.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --capi --api node --sandbox kali.policy.json lib.ts`; attaching `--sandbox` does not bypass the Node build gate for the C-embedding artifact flow. |
| plain `kali build --capi lib.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --capi --api browser lib.ts`; the browser build-shape contradiction wins over inherited-context omission too |
| plain `kali build --capi --sandbox kali.policy.json lib.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --capi --api browser --sandbox kali.policy.json lib.ts`; inherited browser context does not create a browser C-embedding artifact mode. |
| `kali build --component lib.ts` | Phase 2 target | Component-oriented library packaging path; when enabled it emits `kind: wasm-module` (`role: primary-library`) + `kind: wit` (`role: interface-wit`) + `kind: wasm-component` (`role: primary-component`) |
| `kali build --component --sandbox kali.policy.json lib.ts` | Phase 2 target | Same component-oriented library packaging path plus the ordinary build-time static policy-validation path; `--sandbox` remains orthogonal to artifact mode and does not change compile intent or API-surface gating. |
| `kali build --component --api node lib.ts` | Phase 3 target | Component packaging remains subject to the ordinary Node build gate; it does not create a separate early Node component profile |
| `kali build --component --api node --sandbox kali.policy.json lib.ts` | Phase 3 target | Same Node build gate as `kali build --component --api node lib.ts`; attaching `--sandbox` adds static policy validation only and does not bypass that gate. |
| plain `kali build --component lib.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --component --api node lib.ts`; once `--component` exists, inherited config must not silently fall back to a non-Node component build |
| plain `kali build --component --sandbox kali.policy.json lib.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali build --component --api node --sandbox kali.policy.json lib.ts`; attaching `--sandbox` does not bypass the Node build gate for component packaging. |
| plain `kali build --component lib.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --component --api browser lib.ts`; inherited browser context does not create a browser-component artifact mode |
| plain `kali build --component --sandbox kali.policy.json lib.ts` under an inherited browser API surface | Rejected by default | Same effective request as explicit `kali build --component --api browser --sandbox kali.policy.json lib.ts`; inherited browser context does not create a browser-component artifact mode even when `--sandbox` is present. |
| `kali build` with more than one explicit artifact-mode selector from `--bundle` / `--lib` / `--capi` / `--component` | Rejected by default | Artifact mode is a one-of selector in early phases; conflicting combinations such as `--bundle --lib`, `--bundle --capi`, `--bundle --component`, `--lib --capi`, `--lib --component`, or `--capi --component` should fail with `E5008` rather than a feature-maturity diagnostic |
| `kali build --capi --api browser lib.ts` | Rejected by default | Early browser support is limited to the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md); browser-embedding artifact modes remain outside that canonical boundary |
| `kali build --capi --api browser --sandbox kali.policy.json lib.ts` | Rejected by default | Same browser-embedding contradiction as `kali build --capi --api browser lib.ts`; attaching `--sandbox` adds only static policy validation and does not create a browser C-embedding artifact mode. |
| `kali build --component --api browser lib.ts` | Rejected by default | Early browser support is limited to the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md); browser-component artifact modes remain outside that canonical boundary |
| `kali build --component --api browser --sandbox kali.policy.json lib.ts` | Rejected by default | Same browser-component contradiction as `kali build --component --api browser lib.ts`; attaching `--sandbox` adds only static policy validation and does not create a browser component artifact mode. |
| `kali test` / `kali test --api deno` | Phase 1 MVP | Compile and run tests with the shared **Default standalone context (schema v1)** from [SPEC.md](../SPEC.md) unless overridden |
| `kali test a.test.ts b.test.ts` | Phase 1 MVP | Explicit test files bypass naming-pattern discovery and are treated as one explicit test-module set, provided every file is from the executable/analyzable source set |
| `kali test --filter "math"` | Phase 1 MVP | `--filter` narrows the selected test cases after discovery or explicit-file-set selection; it does not create a second discovery mode or change API-surface/runtime gating |
| declaration-only file passed to `run` / `effects` / `build` / `test` as a primary input | Rejected by default | Declaration files are analysis/type inputs, not executable entrypoints or build/effect primary inputs; use the canonical invalid-entrypoint diagnostic (`E5007`) rather than treating this as general CLI misuse |
| `kali test --sandbox kali.policy.json` | Phase 1 MVP | Runtime sandbox enforcement path for tests; policy schema/ranges must validate before execution starts |
| `kali test --api node` | Phase 3 target | Reject with `E5006` until the documented Node subset lands for test runs too |
| plain `kali test` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali test --api node`; inherited config must not silently fall back to `deno` for test execution |
| plain `kali test --sandbox kali.policy.json` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali test --api node --sandbox kali.policy.json`; attaching `--sandbox` does not bypass the Node test-runtime gate |
| `kali test --api browser` | Later compatibility | Reject with `E5006` until a real browser-test contract exists; early browser support is limited to the shared **Phase-1 browser-targeted command set**, not a standalone test-runtime profile |
| plain `kali test` under an inherited browser API surface | Later compatibility | Same effective request as explicit `kali test --api browser`; inherited config must not silently fall back to `deno` for test execution |
| plain `kali test --sandbox kali.policy.json` under an inherited browser API surface | Later compatibility | Same browser-test gate as explicit `kali test --api browser --sandbox kali.policy.json`; attaching `--sandbox` does not turn browser into an early test-runtime profile |
| `kali test --coverage` | Phase 2 target | Coverage needs a stable machine-readable report contract instead of ad hoc runner output |
| `kali effects` with no explicit primary source input | Rejected by default | Schema v1 keeps `effects` as a direct-input command once it exists; omitting the analysis root should fail with `E5008` rather than implicitly scanning the project |
| `kali effects a.ts b.ts` | Rejected by default | Schema v1 accepts exactly one primary analysis root for `effects`; multi-entry reporting requires a later explicit mode, so this should fail with `E5008` |
| `kali effects main.ts` (and JSON-formatting forms such as `--pretty`, `--output json`, or both) | Phase 2 target | Base row for the reporting half of the shared **public effect-report surface**: before then the command stays unavailable rather than exposing a partial bespoke report. JSON-formatting selectors do not create an earlier path; once the command exists they change presentation only while analysis defaults come from the shared **default source-graph analysis context (schema v1)** unless overridden. |
| `kali effects --sandbox kali.policy.json main.ts` | Rejected by default | Keep `effects` as a pure reporting command; policy validation belongs to `check/build --sandbox` so the CLI has one canonical policy-validation path. This rejection should use `E5008`, not the `E5006` maturity gate. |
| `kali effects --api browser main.ts` | Phase 2 target | Reuses the same browser API-surface analysis context as `kali check --api browser` once the Phase 2 command exists, without implying standalone browser execution |
| plain `kali effects main.ts` under an inherited browser API surface | Phase 2 target | Same effective request as explicit `kali effects --api browser main.ts`; inherited config must not silently fall back to `deno` for effect analysis |
| `kali effects --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset exists for effect analysis too |
| plain `kali effects main.ts` under an inherited Node API surface | Phase 3 target | Same effective request as explicit `kali effects --api node main.ts`; inherited config must not silently fall back to `deno` for effect analysis |
| `kali effects --wasm-threads main.ts` | Later compatibility (opt-in only) | `effects` records runtime-profile-sensitive analysis context; reject with `E5006` until the threaded profile exists and effect analysis supports it |
| plain `kali effects main.ts` under an inherited threaded analysis context (`runtimeProfiles = ["wasm-threads"]`) | Later compatibility (opt-in only) | Same effective request as explicit `kali effects --wasm-threads main.ts`; inherited config must not silently drop the threaded profile for effect analysis |
| `kali effects --compat eval main.ts` | Phase 4 compatibility | Effect analysis follows the same compatibility gate as other source-graph commands; once implemented it reports the `Eval` effect / dynamic reasons under the enabled compatibility path |
| plain `kali effects main.ts` under an inherited `eval` compatibility context (`compat.features = ["eval"]`) | Phase 4 compatibility | Same effective request as explicit `kali effects --compat eval main.ts`; inherited config must not silently remove `eval` to make effect analysis succeed earlier |
| `kali package-effects` with no explicit package | Rejected by default | Violates the shared **registry-analysis target contract (schema v1)** from [SPEC.md](../SPEC.md): omitting the explicit registry target is invalid command usage (`E5008`) |
| `kali package-effects lodash` (and JSON-formatting forms such as `--pretty`, `--output json`, or both) | Phase 2 target | Base row for the analysis-context-aware half of the shared **registry-analysis command split** from [SPEC.md](../SPEC.md): follow the shared **registry-analysis availability boundary** until the effect-report pipeline exists, and once it exists inherited-context maturity follows the shared **axis-aligned inherited analysis gating** rule rather than a package-analysis-specific shadow matrix. Schema v1 keeps that semantic context inherited-only here: explicit package-analysis flags such as `--api ...`, `--compat ...`, `--wasm-threads`, and `--sandbox ...` remain invalid usage instead of creating a second CLI vocabulary. |
| `kali package-effects lodash react` | Rejected by default | Early phases do not define a multi-package effect-analysis batch mode, so passing more than one package is invalid command usage (`E5008`) |
| `kali package-effects lodash` under an inherited browser analysis context | Phase 2 target | Representative inherited browser case under the shared **axis-aligned inherited analysis gating** rule: once the base command exists, browser reuse follows the same browser-targeted analysis context/package-resolution rule as other browser analysis commands without widening the exact **Phase-1 browser-targeted command set**. There is intentionally no separate explicit `kali package-effects --api browser ...` form in schema v1; the explicit per-command semantic-flag spellings stay on the rejected row below. |
| `kali package-effects lodash` under an inherited Node analysis context | Phase 3 target | Representative inherited Node case under the shared **axis-aligned inherited analysis gating** rule: inherited config must hit the same Node analysis gate as other source-graph analysis/effect commands rather than silently falling back to `deno`. |
| `kali package-effects lodash` under an inherited threaded analysis context (`runtimeProfiles = ["wasm-threads"]`) | Later compatibility (opt-in only) | Representative inherited threaded-profile case under the shared **axis-aligned inherited analysis gating** rule: inherited config must hit the same threaded-profile gate as other effect-analysis paths rather than silently dropping the profile. |
| `kali package-effects lodash` under an inherited `eval` compatibility context (`compat.features = ["eval"]`) | Phase 4 compatibility | Representative inherited compatibility-feature case under the shared **axis-aligned inherited analysis gating** rule: inherited config must hit the same `eval` gate as other analysis/effect paths rather than silently removing `eval` to make package analysis succeed earlier. |
| `kali package-effects` with package-analysis-specific flags (`--api ...`, `--compat ...`, `--wasm-threads`, or `--sandbox ...`) | Rejected by default | Early package analysis inherits semantic context from config/defaults instead of taking its own package-analysis flag family. Using the **shared flag buckets** / **semantic/context flag surface** split from [SPEC.md](../SPEC.md), these rejected forms are the package-analysis-specific semantic/context flags only; ordinary shared presentation/control flags and the command's documented **JSON-mode selectors** remain governed by the normal CLI rules. `package-effects` is also a reporting command rather than a second policy-validation entrypoint, so these combinations are invalid command usage (`E5008`) unless a later spec adds them |
| `kali package-effects` with a non-registry target (for example `https://...` or `./local.ts`) | Rejected by default | `package-effects` analyzes registry packages only; raw URLs and local paths belong to the project/import-graph workflow instead |
| `kali package-audit` with no explicit package | Rejected by default | Violates the shared **registry-analysis target contract (schema v1)** from [SPEC.md](../SPEC.md): omitting the explicit registry target is invalid command usage (`E5008`) rather than an implicit whole-project audit mode |
| `kali package-audit lodash` (and envelope-formatting forms such as `--output json` or `--pretty --output json`) | Phase 4 compatibility | This is the context-free half of the shared **registry-analysis command split** from [SPEC.md](../SPEC.md): the command is a one-package registry-analysis workflow, and JSON-formatting follows the schema-owned **Package Audit JSON Output (schema v1)** contract from [18 — Schemas](18-schemas.md) without creating a separate availability path; findings are reported through standard diagnostics and the envelope keeps canonical `payload: null` rather than a dedicated success payload. |
| `kali package-audit --pretty lodash` without `--output json` | Rejected by default | Invalid command usage (`E5008`): `--pretty` without `--output json` is not meaningful for schema v1's envelope-only `package-audit` JSON mode, and this command-shape error wins before the command-availability gate. |
| `kali package-audit lodash react` | Rejected by default | The command accepts exactly one explicit package argument; multi-package audit requires a later explicit mode, so this is invalid command usage (`E5008`) |
| `kali package-audit` with a non-registry target (for example `https://...` or `./local.ts`) | Rejected by default | `package-audit` is registry-package-oriented rather than a second raw-URL/local-path analysis path |
| `kali package-audit` with package-analysis-specific flags (`--api ...`, `--compat ...`, `--wasm-threads`, or `--sandbox ...`) | Rejected by default | `package-audit` follows **context-free registry analysis (schema v1)** from [SPEC.md](../SPEC.md) and intentionally keeps the package selector as its semantic/context flag surface rather than growing host-analysis/runtime/sandbox flags. Under the **shared flag buckets** split, that rejection is about package-analysis-specific semantic/context flags only; ordinary shared presentation/control flags and the command's documented **JSON-mode selectors** still follow the normal CLI/output rules. These combinations are therefore invalid command usage (`E5008`) unless a later spec adds them |
| `kali run/test --max-spawned-processes 0 ...` | Phase 1 MVP | Follows the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value even before subprocess support exists |
| `kali run/test --max-spawned-processes N` with `N > 0` before subprocess support exists | Phase 3 target | Positive spawned-process caps track the same maturity path as subprocess support itself: before that Phase-3 capability exists, reject with `E5006`; once it exists, the cap becomes an ordinary tightening control within the selected command/profile/API surface |
| `kali run/test --max-threads 0 ...` | Phase 1 MVP | Follows the shared **feature-gated zero-capable execution budgets** rule from [SPEC.md](../SPEC.md): `0` is a valid explicit deny/tightening value even before the threaded runtime profile exists |
| `kali run/test --max-threads N` with `N > 0` before thread support exists | Later compatibility (opt-in only) | Positive thread caps track the same maturity path as the opt-in threaded runtime profile itself: before that profile exists, reject with `E5006`; once it exists, the cap becomes an ordinary tightening control for the selected threaded command/profile |
| `--compat eval` | Phase 4 compatibility | Before the documented `eval` compatibility path exists for the effective command context, reject with `E5006` rather than parsing and silently ignoring the flag; once implemented, sandbox policy permission for `effects.eval` still does not implicitly enable this compatibility switch |
| `--wasm-threads` | Later compatibility (opt-in only) | Reject with `E5006` until the threaded runtime profile exists; after that, still reject explicitly when unavailable on the selected target/engine |

## Phase Exit Criteria

These checklists keep the phase labels operational rather than purely descriptive.

### Phase 1 exit criteria
- One linked-WASM-payload compile/run pipeline works end-to-end for TS and JS inputs, with companion artifacts only where an artifact mode explicitly requires them.
- Repeated builds with the same pinned inputs and toolchain produce stable artifact bytes and stable machine-readable output ordering by default.
- `kali init`, `install`, `run`, `build`, `check`, `fmt`, `lint`, and `test` exist with stable core behavior.
- `kali init` keeps the shared **minimal canonical scaffold contract**: the scaffold stays intentionally small, does not materialize dependencies, and does not blur into `install` or later artifact-mode promises.
- The Phase-1 **base library artifact** (`kali build --lib`) works end-to-end as the Phase-1 half of the shared **embedding-stability split**.
- The checker ships the shared **bounded inference contract** promised for Phase 1 for locals, obvious unannotated parameters, and analyzable return types, while still falling back conservatively instead of doing open-ended whole-program search.
- The shared **Phase-1 browser-targeted command set** works against the real browser ambient surface without implying DOM runtime support in Kali itself.
- That browser-targeted claim is backed by dedicated browser-targeted tests, including emitted-bundle smoke runs in a real browser harness rather than only mock DOM/unit tests.
- `kali check` / `build` / `run` / `test` all use the same early-phase API-surface maturity rules: Deno-supported, Node phase-gated, browser supported only for the documented browser-targeted check/bundle paths.
- Runtime sandbox enforcement and resource limits work for the documented Phase 1 host APIs.
- `check/build --sandbox` perform the documented Phase-1 policy-schema/config validation without overclaiming full inferred-effect-vs-policy validation/comparison yet.
- The shared **effect-surface split** remains intact in Phase 1: internal effect bookkeeping may exist, but both Phase-2 halves of the stable **public effect-report surface** — the reporting half (`kali effects` / `kali package-effects`) and the policy-comparison half (inferred-effect-vs-policy validation) — are still correctly absent from the shipped Phase-1 surface.
- The Lean-backed verification story is phase-correct: the repository reaches the Phase-1 **proof-ready** baseline with one published **proof-boundary manifest** plus the proof-CI trigger policy, and the published boundary is proof-backed for the widened closed fragment — now including assignment and try/catch in addition to literals, variables, closed functions, application, sequencing, and conditionals — plus a small RC snapshot safety slice with live-reference ownership/allocation projection, release-update preservation, explicit release-recording, target-cell decrement bookkeeping, last-ref zeroing, zero-count collection, unrelated-heap preservation, disjointness on the decrement path, the local `releaseAndCollect` disjointness theorem, and a refcount-decrement update helper, and a widened HIR lowering-correctness slice while remaining intentionally narrower than the later Stage 4.2 ownership/memory-safety and lowering-correctness target. The shared **placeholder proof-boundary manifest** remains acceptable only while Kali stays proof-ready without advertising formal verification as a shipped capability.
- Unsupported dynamic features fail with the canonical feature-maturity diagnostic instead of silently degrading.
- Package support works for the documented pure JS/TS, statically linkable subset, and the same Phase-1 lock/materialization story also covers supported raw-URL dependency graphs.
- Non-install commands still fail with `E5004` on missing/stale dependency state instead of auto-installing or auto-repairing project-managed dependency state.
- Phase-1 support summaries and release notes stay honest about the top-level **Phase-1 Explicit Non-Goals** from [SPEC.md](../SPEC.md): no standalone browser runtime/test contract, no `--api node` support, no stable public effect-report commands yet, no stable public embedding surface beyond the **base library artifact** (so no stable public WIT/C-ABI/component flow yet), no runtime `eval` path, and no threaded runtime profile.

### Phase 2 exit criteria
- MIR is the canonical ownership/layout IR.
- The Phase-2 **public effect-report surface** is live: `kali effects` emits the documented stable JSON report, `kali package-effects` reuses that shared contract, and explicit effect annotations / `pure` checking are enabled for the built-in capability model.
- Compile/check-time effect-vs-policy validation works against the declarative policy schema, extending the existing Phase-1 policy-file/config validation path rather than replacing it.
- Proof coverage expands alongside the modeled ownership/effect/lowering core rather than remaining frozen at the initial Phase-1 published proof boundary, and the published **proof-boundary manifest** grows with those claims instead of leaving proof scope implicit.
- Stable public Rust embedding and C ABI surfaces are documented and shipped.
- The Phase-1 base `kali build --lib` artifact is promoted into the stable public **WIT-first** library contract, including default WIT emission.
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
- browser support remains anchored to the shared **Phase-1 browser-targeted command set** first unless a standalone browser-runtime contract is added explicitly
- Node support remains phase-gated uniformly across `check` / `effects` / `build` / `run` / `test` until the documented Node subset exists
- adding a new command should reuse existing artifact/effect/policy contracts instead of inventing a near-duplicate workflow

This keeps Phase 1 exit criteria phase-correct while still making later command behavior predictable.

## Compatibility Appendix by Concern Area

This appendix separates the broad compatibility story into smaller tables so language support, type-system support, host/runtime support, and package support do not get conflated.

### Language Semantics

| Concern | Early canonical status | Notes |
|---|---|---|
| Core ECMAScript syntax and static ESM graph | Phase 1 MVP | Parser stays broad and should track the latest published standard grammar; unsupported semantics are gated separately |
| **E1xxx lexing diagnostics** | Phase 1 MVP (opt-in only) | Lexer produces stable E1xxx error codes and recovers from lexing errors; see [specs/15-errors.md](15-errors.md) for the full E1xxx namespace
| Annex B / web-legacy semantics | Later compatibility | Broad syntax support does not imply immediate support for every legacy browser semantic corner |
| First-class JavaScript compilation with bounded inference | Phase 1 MVP | `.js` is a first-class input under the shared **first-class JavaScript compilation** contract, with early precision bounded by the shared **bounded inference contract** |
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
| Shared bounded inference contract for locals, obvious parameters, and analyzable returns | Phase 1 MVP | Early inference should improve materially on plain `tsc` local inference while staying inside the shared **bounded inference contract** and using the shared **annotation-required inference boundary** when cost or API stability would otherwise become unpredictable |
| Stable built-in capability-effect reporting | Phase 2 target | Reuses the reporting half of the canonical **public effect-report surface** row above: stable `kali effects` / `kali package-effects` output |
| Compile/check-time effect-vs-policy validation | Phase 2 target | Reuses the policy-comparison half of that same canonical **public effect-report surface** row above: pass/fail validation on `check/build --sandbox` |
| Explicit `pure` / effect annotations | Phase 2 target | Built-in sandbox capability model first |
| Stable user-defined/custom effects in machine contracts | Later compatibility | Keep Phase 1-2 schemas, reports, and policy validation/comparison scoped to the built-in sandbox-relevant effect family |

### Host and Runtime Profiles

| Concern | Early canonical status | Notes |
|---|---|---|
| Deno-oriented standalone surface (`--api deno`) | Phase 1 MVP | Default API surface for standalone execution; typically paired with the baseline single-threaded runtime profile |
| Invocation arguments in the standalone surface (`Deno.args`) | Phase 1 MVP | Part of the execution context rather than a separately policy-gated capability in schema v1 |
| Read-only `Deno.permissions` facade over resolved policy state | Phase 1 MVP | Canonical **observation-only compatibility facade**: query-only over the shared **Deno-compatible permission descriptor subset (schema v1)**, returning only the shared **stable permission status subset (schema v1)**, with no interactive `request()` / `revoke()` escalation flows; Phase 1 `net` remains fetch-only because socket/listener networking is still later-phase |
| Interactive permission escalation / revocation APIs | Rejected by default | The Phase 1 Deno-compatibility story is query-only; runtime prompt/escalation flows are outside the sandbox model |
| Read-only environment access on the Deno-oriented standalone surface | Phase 1 MVP | Exposes only the sandbox-permitted environment view |
| Web-baseline randomness subset (`crypto.getRandomValues`) | Phase 1 MVP | Covers the schema-v1 `effects.random` / `Random.GetBytes` capability without implying full Web Crypto support |
| Mutable environment access / process-environment mutation | Phase 3 target | Policy-controlled host mutation, not part of the Phase 1 baseline |
| Subprocess spawning and socket/listener networking | Phase 3 target | Shares the same sandbox/process/network maturity path as the corresponding capability rows above |
| The **Phase-1 browser-targeted command set** | Phase 1 MVP | These commands use the real browser ambient surface, and emitted browser bundles execute against the real browser host via the browser host adapter, with no standalone browser emulation; support claims require dedicated browser-targeted tests and real-browser bundle smoke coverage, and later browser-targeted analysis commands should reuse the same ambient typing layer and browser package-resolution context only once their own rows become available |
| Standalone `run --api browser` | Later compatibility | No embedded browser engine yet; reject with `E5006` until a real browser-execution contract exists |
| Node API surface across `check` / `effects` / `build` / `run` / `test` | Phase 3 target | Package-driven subset first; early phases reject `--api node` consistently rather than exposing a partial surface |
| Threaded runtime profile / `--wasm-threads` | Later compatibility (opt-in only) | Runtime-profile switch, independent from API-surface selection |

### Packages and Ecosystem

| Concern | Early canonical status | Notes |
|---|---|---|
| Pure JS/TS npm packages within the linked-artifact model | Phase 1 MVP | Restricted to the shared **pure JS/TS package contract** and still context-sensitive: early support covers the Deno-oriented standalone surface plus the shared **Phase-1 browser-targeted command set** from [SPEC.md](../SPEC.md), not broad Node-host-heavy assumptions. Read claims through the support ladder: early Deno-oriented rows may be **installable/materializable**, **checkable**, **buildable**, or **executable**, while early browser-targeted rows are usually **checkable** or **deployable-through-host** rather than standalone-browser **executable** support. |
| Pure JS/TS JSR packages within the linked-artifact model | Phase 1 MVP | Registry-style install/lock/materialization path just like npm in early phases, with the same Deno-oriented standalone surface or **Phase-1 browser-targeted command set** boundary and the same support-rung reading |
| Raw URL imports in the shared lock/materialization model | Phase 1 MVP | Pin in `kali.lock`, materialize under `.kali/cache/urls/`, and keep ordinary commands deterministic |
| Deno-condition package resolution in the default standalone surface | Phase 1 MVP | Honor `exports` condition `deno` when `--api deno` is selected |
| Browser-condition package resolution for the **Phase-1 browser-targeted command set** | Phase 1 MVP | Reuse one shared browser package-resolution context for the **Phase-1 browser-targeted command set**: honor `exports` condition `browser` plus applicable `package.json#browser` replacement maps consistently |
| npm lifecycle scripts | Phase 1 MVP (opt-in only) | `kali install --allow-scripts`; install-time package hooks stay outside the normal runtime API-surface and project-policy contracts, and in mixed install graphs they still apply only to the npm subset being reconciled |
| Packages in the native/binary/bootstrap-heavy package contract | Rejected by default | The excluded shared package contract stays unsupported; `--allow-scripts` must not silently broaden support to it |
| Broader Node-host-heavy npm compatibility | Phase 3 target | Depends on meaningful Node API support |

Package-support reading shortcut:
- package compatibility claims should name both the selected command/context and the claimed rung from the shared support ladder (`installable/materializable`, `checkable`, `buildable`, `executable`, or `deployable-through-host`)
- in particular, Phase-1 browser-targeted package claims should usually be read as **checkable** or **deployable-through-host** claims, not as standalone browser-runtime **executable** claims
- registry-analysis commands (`package-effects`, `package-audit`) remain separate maturity questions from ordinary project-command package support; `package-audit` opens in Phase 4 as the context-free registry-analysis/security-audit command

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

To reduce drift between CLI, runtime, package, and error-reporting specs, reuse the **Hard-Feature Implementation Stage Matrix** above as the normative parse-vs-analysis-vs-lowering-vs-execution source of truth instead of maintaining a second near-duplicate feature table here.

Operational reading rule:
- broad syntax acceptance does **not** imply early lowering or execution support;
- early phases should reject unavailable runtime/lowering paths with the canonical feature-maturity diagnostic (`E5006`) unless a stricter subsystem-specific error is more informative;
- when a feature is parsed early for compatibility planning (`import()`, `eval`, `Proxy`, weak-reference APIs, effect syntax), the checker/runtime docs should point back to the matrix above rather than re-stating a parallel mini-table with slightly different wording.

This keeps syntax acceptance clearly separated from semantic support without duplicating the same feature list in two places.

See also:
- [SPEC.md](../SPEC.md)
- [01 — Architecture](01-architecture.md)
- [10 — Runtime](10-runtime.md)
- [14 — Package Management](14-packages.md)
