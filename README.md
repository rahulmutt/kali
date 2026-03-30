# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Bootstrap-normalized headline assumptions:
- `BOOTSTRAP.md` is the input brief; [SPEC.md](./SPEC.md) plus [specs/19-feature-maturity.md](./specs/19-feature-maturity.md) are the normative source of truth after normalization
- hard invariants stay fixed across phases: **AOT only**, the **Pure-Rust implementation contract**, **no tracing/background GC**, **sandbox-first honesty**, and deterministic machine-readable contracts
- Phase 1 is intentionally narrow: **Deno-first** standalone execution plus the shared **Phase-1 browser-targeted command set**; broader Node support comes later. See [SPEC.md#phase-1-browser-targeted-command-set](./SPEC.md#phase-1-browser-targeted-command-set) for the exact browser-targeted command boundary.
- stronger-than-`tsc` inference is still bounded: Kali improves local/obvious inference, but keeps an explicit annotation-required boundary instead of open-ended whole-program search
- latest ECMA-262 means the **latest published edition**; accepted grammar does not by itself imply same-phase runtime support for every feature
- optimization vocabulary is intentionally small: `fast` is the bounded-cost default, while `release` and `release-advanced` are the canonical compile-budget expansion modes; any optional external post-pass in `release-advanced` stays a user-provided add-on rather than part of Kali's required core toolchain
- the CLI is Deno-inspired at the workflow level (`init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`), but that does **not** imply flag-for-flag Deno parity or same-phase availability for every documented command family
- documented command/artifact shapes and actual availability are intentionally separate: CLI/package/embedding chapters may define stable spellings or artifact layouts before they are phase-enabled, and [specs/19-feature-maturity.md](./specs/19-feature-maturity.md) remains the availability owner
- the upstream project list in `BOOTSTRAP.md` is a **design-reference list**, not an architecture-copy or dependency promise
- the language-inspiration list in `BOOTSTRAP.md` is also normalized: Haskell/Idris/Agda/Lean inform purity/effects/constraint design, but do not imply Phase-1 dependent types, totality checking, or proof-term workflows in ordinary Kali code
- early runtime standardization is **wasmtime first**; alternative engines are later extensions
- embedding is phased: Phase 1 ships a useful but unstable `kali build --lib` **base library artifact** for exact-version/internal consumers in the default/inherited Deno-oriented build context (that is, the effective `apiSurface` still resolves to `deno`); Phase 2 adds the stable **public embedding surface**: stable Rust embedding API plus the stable public **WIT-first** `--lib` contract, with `--capi` and `--component` as explicit projections/packaging flows over that same export surface
- effects are phased too: Phase 1 may use internal effect bookkeeping for sandboxing, while the stable Phase-2 **public effect-report surface** is split into a reporting half (`kali effects`, `kali package-effects`) and a policy-comparison half (compile/check-time inferred-effect-vs-policy validation on `check/build --sandbox`)
- the bootstrap's “statically run a command and get JSON output of all potential effects” ask is normalized to that analysis/reporting split, not to a second `run --dry` / `test --dry` command family
- `kali package-audit` is intentionally separate from effect reporting: it is a later, context-free registry-analysis workflow rather than part of the sandbox/effect-report surface
- verification follows the shared **proof state split** from [SPEC.md](./SPEC.md): Phase 1 must be **proof-ready** (published proof boundary + proof-CI trigger policy), while **proof-backed** is only required for release/support wording that wants to market formal verification as shipped evidence. See [proofs/BOUNDARY.md](./proofs/BOUNDARY.md) for the current repository state and current proof-status summary
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](./SPEC.md): target crate/workspace/proof-tree layouts, illustrative cargo commands, and names such as `kali_fmt`, `kali_lint`, `kali_embed`, and `kali_capi` describe intended structure and stable shapes, not present-repo facts unless the corresponding files/artifacts actually exist
- package installation stays context-agnostic in Phase 1, while package support claims use the shared **package-support decision order**: package shape first, then host/API fit for the active context, then command maturity, all under the **published-artifact-first package reading**
- the bootstrap's “support non node-gyp packages from npm” goal is normalized to the broader **pure JS/TS package contract**, not to a brittle “anything without `node-gyp` must work” rule; packages that still depend on N-API/native bindings, prebuilt binaries, or bootstrap-heavy install/runtime paths remain outside the early supported set even if they never invoke `node-gyp`
- practical package-reading shortcut: use the shared **package-support ladder** from [SPEC.md](./SPEC.md) — a package may be **installable/materializable** without yet being **analyzable/checkable**, **buildable**, **executable**, or **deployable-through-host** in the selected command/context
- package-evidence shortcut: early package-compatibility claims are primarily about ordinary **source-graph commands** (`check`, `build`, `run`, `test`, plus the shared browser-targeted `check` / `build --bundle` paths), not about the later single-package registry-analysis workflows `package-effects` / `package-audit`
- dependency mutability is intentionally simple: `kali install` owns manifest/lock/materialized dependency state, while non-install commands fail with the canonical `E5004` path instead of auto-installing or silently repairing dependency state
- npm lifecycle hooks are an install-only opt-in: `kali install --allow-scripts` is meaningful only when the current invocation has non-empty npm install work, and it still does not broaden support beyond the normal pure-JS/TS package contract

Quick Phase-1 non-goals:
- no general `--api node` command support yet across `check` / `effects` / `build` / `run` / `test`
- no standalone browser runtime or browser-hosted `run` / `test`
- no non-bundle browser build lane: browser-targeted support in Phase 1 is limited to `check` plus `build --bundle`; under an effective browser API surface there is no plain `build` / `build --lib` browser path yet
- no stable public effect workflow yet: neither the reporting half (`kali effects`, `kali package-effects`) nor the policy-comparison half (compile/check-time inferred-effect-vs-policy validation) is shipped in Phase 1; `kali check/build --sandbox ...` still perform only policy schema/config validation in Phase 1, and there is no dry-run `run` / `test` replacement for that workflow
- no stable user-facing `kali package-audit` workflow yet; that later command is intentionally separate from the effect-report surface
- no `eval` / `Function()` support yet
- no threaded runtime profile yet
- no Phase-2 **public embedding surface** yet: no stable public Rust embedding API, no `--capi`, no `--component`, no default WIT sidecars for plain `--lib`, and no cross-version host-loading guarantee for the Phase-1 base library artifact

Recommended Phase-1 implementation order:
1. frontend + checking foundation
2. deterministic install/package foundation
3. Deno-first Kali-hosted run/test foundation with sandbox enforcement
4. build outputs (`build`, browser bundle, Phase-1 `--lib`)
5. developer workflow polish (`init`, `check`, `fmt`, `lint`, diagnostics, JSON contracts)
6. evidence hardening (conformance, package corpus, browser smoke tests, determinism, and ongoing maintenance of the already-published Phase-1 proof-ready boundary/trigger-policy baseline)

See the normative cross-spec version in [SPEC.md#recommended-phase-1-implementation-order](./SPEC.md#recommended-phase-1-implementation-order).
For the compact “what is actually shipped in Phase 1?” answer, see the **Phase-1 Shipped Surface Summary** in [specs/19-feature-maturity.md](./specs/19-feature-maturity.md).

Defined-now vs shipped-now reminder:

| Surface family | Why it is already documented | Still shipped in Phase 1? |
|---|---|---|
| `kali effects` / `kali package-effects` | reserve the public effect-report vocabulary and JSON contract early | no — Phase 2 target |
| `kali package-audit` | reserve the separate context-free registry-analysis workflow early | no — Later compatibility |
| `kali build --capi` / `kali build --component` | reserve the public embedding artifact vocabulary early | no — Phase 2 target |
| plain public `--lib` + default WIT | keep the final WIT-first library contract visible while Phase 1 still ships only the unstable base artifact | no — Phase 1 ships only the **base library artifact** |

Quick support-reading checklist:
1. **What command shape is being asked for?** `build --bundle --api browser` and `run --api browser` are different requests.
2. **What rung of support is meant?** Use the shared **compatibility delivery ladder** in [SPEC.md](./SPEC.md): parser-accepted, checkable, buildable, executable, deployable-through-host, or policy/effect-modeled.
3. **If this is about packages, which layer is being asked about?** Use the shared **package-support decision order** plus the **package-support ladder** in [SPEC.md](./SPEC.md): package shape first, then host/API fit, then command maturity, and finally the exact support rung being claimed.
4. **What effective context is selected?** Read the participating axes together: `apiSurface`, command-relevant `buildMode`, `runtimeProfiles`, `compat.features`, and any attached sandbox policy.
5. **Which chapter owns the answer?** Command shape lives in `12-cli`, package semantics in `14-packages`, availability in `19-feature-maturity`, JSON shape in `18-schemas`, diagnostics in `15-errors`.

Use that order before treating any broad bootstrap aspiration as shipped support.

Common early-phase misreads worth rejecting quickly:
- the whole **Phase-1 browser-targeted command set** is supported in Phase 1 — including explicit `--api browser` spellings, equivalent inherited-config forms, and the supported `--sandbox` variants — but `kali run --api browser main.ts` and `kali test --api browser` are still later compatibility.
- `kali build --lib lib.ts` is a supported Phase-1 **base library artifact** for exact-version/internal consumers in the default/inherited Deno-oriented build context (effective `apiSurface = deno`); `kali build --lib --sandbox kali.policy.json lib.ts` is the same Phase-1 base library build plus static policy validation, while `kali build --capi lib.ts` and `kali build --component lib.ts` are still Phase-2 embedding flows.
- `kali check --sandbox ...` and `kali build --sandbox ...` are Phase-1 policy-schema/config validation paths only; under the shared **sandbox-attachment orthogonality** rule from [SPEC.md](./SPEC.md), that sandbox attachment does **not** yet imply the Phase-2 compile/check-time inferred-effect-vs-policy validation workflow, does not change `check` file arity, and on `build` does not change artifact mode or compile intent.
- Phase-1 verification wording is about repository/process hygiene first: one published boundary, one proof-CI trigger policy, and no proof-backed marketing beyond that boundary. The repo should already be **proof-ready** from the start; later evidence work merely hardens and maintains that baseline.

Practical Phase-1 command/context cheat sheet:

| Request | Phase 1 reading |
|---|---|
| `kali check --api browser main.ts` | supported browser-targeted analysis |
| `kali check --api browser --sandbox kali.policy.json main.ts` | supported browser-targeted static policy validation |
| plain `kali check main.ts` under inherited `compilerOptions.apiSurface = browser` | same supported browser-targeted analysis |
| plain `kali check --sandbox kali.policy.json main.ts` under inherited `compilerOptions.apiSurface = browser` | same supported browser-targeted static policy validation |
| `kali build --bundle --api browser main.ts` | supported browser-targeted bundle build |
| `kali build --bundle --api browser --sandbox kali.policy.json main.ts` | supported browser-targeted bundle build plus static policy validation |
| plain `kali build --bundle main.ts` under inherited `compilerOptions.apiSurface = browser` | same supported browser-targeted bundle build |
| plain `kali build --bundle --sandbox kali.policy.json main.ts` under inherited `compilerOptions.apiSurface = browser` | same supported browser-targeted bundle build plus static policy validation |
| `kali build --bundle --api node main.ts` | contradictory non-browser bundle shape (`E5008`); `--bundle` is the browser-only executable packaging path |
| `kali build --api browser main.ts` | invalid browser build shape (`E5008`) until a non-bundle browser build mode exists |
| `kali build --lib --api browser lib.ts` | contradictory browser-library build shape (`E5008`); browser mode is Phase-1 `check` + `build --bundle`, not a library artifact mode |
| `kali run --api browser main.ts` / `kali test --api browser` | unavailable browser runtime/test contract (`E5006`) |

## Specification
- Top-level overview, implementation strata, cross-spec simplification rules, canonical terminology, chapter ownership, chapter guide, artifact-mode matrix, bootstrap traceability, and bootstrap-resolution notes: [SPEC.md](./SPEC.md)
- Bootstrap-brief normalization rule: [SPEC.md#bootstrap-normalization-rule](./SPEC.md#bootstrap-normalization-rule)
- Bootstrap triage rule for hard invariants vs phase-gated breadth: [SPEC.md#bootstrap-triage-rule](./SPEC.md#bootstrap-triage-rule)
- Cross-spec simplification rules: [SPEC.md#cross-spec-simplification-rules](./SPEC.md#cross-spec-simplification-rules)
- Bootstrap traceability table: [SPEC.md#bootstrap-traceability-matrix](./SPEC.md#bootstrap-traceability-matrix) *(includes triage bucket + earliest explicit phase promise for each bootstrap ask)*
- Detailed chapter set: [`specs/`](./specs)
- Single source of truth for gated command/profile availability: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)
- Reading shortcut: treat `19-feature-maturity` as the cross-cutting availability overlay and read it alongside the owning command/subsystem chapter rather than as a detached appendix

Reading rule:
- treat `BOOTSTRAP.md` as the input brief and the spec set as the normative source of truth after normalization
- when a bootstrap aspiration and a phase-specific promise seem to differ, prefer `SPEC.md` plus the owning chapter and the feature-maturity matrix
- when a support claim still feels ambiguous, use the shared **support-claim reading order** plus the **compatibility delivery ladder** in `SPEC.md` before assuming Kali means one undifferentiated notion of “support”
- remember the main naming splits used across the specs: config stores compatibility switches under `compat.features` while emitted reports use `compatFeatures`; cross-spec semantic axes use leaf names such as `apiSurface` / `buildMode` / `runtimeProfiles` while concrete `kali.json` storage uses paths such as `compilerOptions.apiSurface` / `compilerOptions.buildMode` / `compilerOptions.runtimeProfiles`; semantic effect kinds such as `FileSystem.Read` map onto policy/schema keys such as `effects.fileSystem.read`; and registry-package CLI/manifests/logical-root labels use the identifier spelling (`lodash`, `jsr:@std/path`) while structured JSON metadata uses the decomposed package-coordinate form (`registry`, `name`, `version`)
- for maintenance, keep the ownership split tight: command shape/flags live in `12-cli`, diagnostic semantics in `15-errors`, JSON field names in `18-schemas`, and phase availability in `19-feature-maturity`

Registry-analysis shortcut:
- `package-effects` is the later **analysis-context-aware** registry workflow: it stays a Phase 2 target, inherits the shared **inherited analysis context** from config/defaults instead of taking package-analysis-specific `--api` / `--wasm-threads` / `--compat` flags, and is a schema-v1 **native-JSON command** once it exists.
- `package-audit` is the later **context-free** registry workflow: it stays **Later compatibility**, ignores host-analysis/runtime context in schema v1, and remains a schema-v1 **envelope-only JSON command** (`--pretty` still requires `--output json`).
- JSON-formatting flags do not create second command variants: `--pretty` / `--output json` change presentation only after ordinary command-shape validation, so they inherit the same availability gate as the underlying `effects` / `package-effects` / `package-audit` request.
- The full command/context/error split lives in [SPEC.md](./SPEC.md)'s registry-analysis terminology plus [12 — CLI](./specs/12-cli.md), [14 — Package Management](./specs/14-packages.md), [18 — Schemas](./specs/18-schemas.md), and [19 — Feature Maturity](./specs/19-feature-maturity.md).

Quick navigation:
- frontend and language design: [01 — Architecture](./specs/01-architecture.md), [02 — Lexer & Parser](./specs/02-lexer-parser.md), [03 — AST](./specs/03-ast.md), [04 — Type System](./specs/04-type-system.md)
- lowering, memory, optimization, and code generation: [05 — IR](./specs/05-ir.md), [06 — Memory Management](./specs/06-memory.md), [07 — Optimization & Specialization](./specs/07-specialization.md), [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- sandboxing, runtime, APIs, and embedding: [09 — Sandboxing & Effects](./specs/09-sandboxing.md), [10 — Runtime](./specs/10-runtime.md), [11 — Standard APIs](./specs/11-standard-apis.md), [13 — Embedding, WIT & C ABI](./specs/13-embedding.md)
- CLI, packages, diagnostics, schemas, testing, verification, and maturity: [12 — CLI](./specs/12-cli.md), [14 — Package Management](./specs/14-packages.md), [15 — Errors](./specs/15-errors.md), [16 — Testing](./specs/16-testing.md), [17 — Formal Verification](./specs/17-verification.md), [18 — Schemas](./specs/18-schemas.md), [19 — Feature Maturity](./specs/19-feature-maturity.md)

## Project posture
This repository is currently spec-first: the top-level spec and chapter set are the source of truth for scope, staging, and machine-readable contracts.

Current repository baseline:
- the checked-in source of truth today is the spec set plus the published proof-boundary manifest at [proofs/BOUNDARY.md](./proofs/BOUNDARY.md)
- target crate trees, test directories, CI lanes, and Lean project layouts shown in the spec chapters describe the intended implementation shape, not a claim that every such file or directory already exists in this repository
- when reading architecture, testing, embedding, or verification examples, prefer the shared **current-repository-state vs target-contract reading** from [SPEC.md](./SPEC.md) before inferring present implementation status from an illustrative tree or command example

## Related project
- [Kai](https://github.com/rahulmutt/kai), an AI-based coding assistant
