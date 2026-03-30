# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

Bootstrap-normalized headline assumptions:
- use the triage from [SPEC.md#bootstrap-triage-rule](./SPEC.md#bootstrap-triage-rule): first separate **hard invariants** from **phase contracts** and **phase-gated breadth targets**, then read the owning chapter plus the maturity matrix
- hard invariants from the bootstrap brief stay fixed across the early plan: **AOT only**, **pure Rust**, **no tracing/background GC**, **sandbox-first honesty**, and deterministic machine-readable contracts
- stronger-than-`tsc` inference in Phase 1 follows one shared **bounded inference contract**: Kali improves local/obvious inference materially, but keeps an explicit annotation-required boundary instead of drifting into open-ended whole-program search
- host support follows one small staircase: shared **Web baseline**, **Deno-first** standalone execution, the shared **Phase-1 browser-targeted command set** (`check --api browser`, `build --bundle --api browser`, or equivalent inherited-config forms), and broader **Node compatibility** as a later ecosystem phase
- later analysis/reporting commands may reuse that same browser-targeted context once their own maturity rows open, but that later reuse does **not** broaden the exact **Phase-1 browser-targeted command set**
- read every broad support claim through the shared **support-claim reading order** in [SPEC.md](./SPEC.md): first check command shape, then the intended **compatibility delivery ladder** rung (`accepted`, `checkable`, `buildable`, `executable`, `deployable-through-host`, `policy/effect-modeled`), then phase/context availability
- latest ECMA-262 tracking means the **latest published edition**; grammar support does **not** imply blanket same-phase runtime support for every accepted feature, and draft / Stage-3+ proposal support is explicit and experimental rather than implied
- the upstream project list in `BOOTSTRAP.md` (Boa, V8, JavaScriptCore, SpiderMonkey, Deno, `tsc`, Porffor, Hermes, Bun) is normalized as a **design-reference list**, not as a promise to copy those architectures or inherit their dependency stacks
- dynamic compatibility paths such as `eval` and `Function()` are part of the long-term contract, but remain explicitly phase-gated behind the single schema-v1 compatibility switch `eval`
- runtime/embedding behavior is standardized on **wasmtime first**; alternative WASM engines are a later extension, not an equal Phase-1 contract
- build artifact modes follow one canonical matrix: default executable compile intent, browser-bundle executable compile intent, a Phase-1 **base library artifact** for library compile intent, and later Phase-2 **public embedding surface** milestones layered on that same exported-library contract
- the Phase-1 plain `kali build --lib` output is intentionally useful but **not yet a stable public ABI/WIT promise**; stable public embedding starts in Phase 2
- follow the shared **effect-surface split** from [SPEC.md](./SPEC.md): public static effect-report commands (`kali effects`, `kali package-effects`) are a **Phase-2** surface, while Phase 1 may already rely on internal effect bookkeeping for sandboxing without implying a stable user-facing JSON report yet; `kali package-audit` is even later and should not be read back into the Phase-1 sandbox-first story
- Lean-backed verification is also phase-scoped: support claims should read through the published **proof-boundary manifest** at `proofs/BOUNDARY.md` rather than assuming whole-language or whole-host verification from the start
- until the first concrete proofs land, that manifest may truthfully remain empty; treat that as an anti-overclaiming guardrail, not as evidence that any subsystem is already mechanically verified
- package installation stays **context-agnostic** in early phases: one lock/install state serves the default Deno path and the shared **Phase-1 browser-targeted command set**, while final `exports`/`browser` edge selection happens at command time
- Phase-1 package compatibility is still **context-sensitive**: pure JS/TS packages may target the Deno-first standalone baseline or the supported browser-targeted `check`/`build --bundle` contexts, while Node-host-heavy assumptions remain phase-gated
- package compatibility in Phase 1 stays inside the shared **pure JS/TS package contract** under the shared **published-artifact-first package reading**: what matters is the published package Kali installs, not the upstream repository's prepublish toolchain; `--allow-scripts` still does not widen support to the excluded **native/binary/bootstrap-heavy package contract**

Quick Phase-1 non-goals:
- no general `--api node` command support yet across `check` / `effects` / `build` / `run` / `test`
- no standalone browser runtime or browser-hosted `run` / `test`
- no stable user-facing `kali effects`, `kali package-effects`, or `kali package-audit` workflow yet
- no `eval` / `Function()` support yet
- no threaded runtime profile yet
- no Phase-2 **public embedding surface** yet: no stable public Rust embedding API, no `--capi`, no `--component`, and no default WIT sidecars for plain `--lib`

Recommended Phase-1 implementation order:
1. frontend + checking foundation
2. deterministic install/package foundation
3. Deno-first Kali-hosted run/test foundation with sandbox enforcement
4. build outputs (`build`, browser bundle, Phase-1 `--lib`)
5. developer workflow polish (`check`, `fmt`, `lint`, diagnostics, JSON contracts)
6. evidence hardening (conformance, package corpus, browser smoke tests, determinism)

See the normative cross-spec version in [SPEC.md#recommended-phase-1-implementation-order](./SPEC.md#recommended-phase-1-implementation-order).

Quick support-reading checklist:
1. **What command shape is being asked for?** For example, `build --bundle --api browser` and `run --api browser` are different requests with different early-phase outcomes.
2. **What rung of support is meant?** Use the shared **compatibility delivery ladder** in [SPEC.md](./SPEC.md): a feature can be parser-accepted, checkable, buildable, executable, deployable-through-host, or policy/effect-modeled without all higher rungs being true yet.
3. **What effective context is selected?** Read `apiSurface`, `runtimeProfiles`, `compat.features`, and any attached sandbox policy together rather than in isolation.
4. **Which chapter owns the answer?** Command shape lives in `12-cli`, availability in `19-feature-maturity`, JSON shape in `18-schemas`, diagnostics in `15-errors`.

This four-step reading order is the shortest safe way to answer “does Kali support X yet?” without over-reading a broad bootstrap aspiration.

## Specification
- Top-level overview, implementation strata, cross-spec simplification rules, canonical terminology, chapter ownership, chapter guide, artifact-mode matrix, bootstrap traceability, and bootstrap-resolution notes: [SPEC.md](./SPEC.md)
- Bootstrap-brief normalization rule: [SPEC.md#bootstrap-normalization-rule](./SPEC.md#bootstrap-normalization-rule)
- Bootstrap triage rule for hard invariants vs phase-gated breadth: [SPEC.md#bootstrap-triage-rule](./SPEC.md#bootstrap-triage-rule)
- Cross-spec simplification rules: [SPEC.md#cross-spec-simplification-rules](./SPEC.md#cross-spec-simplification-rules)
- Bootstrap traceability table: [SPEC.md#bootstrap-traceability-matrix](./SPEC.md#bootstrap-traceability-matrix) *(includes triage bucket + earliest explicit phase promise for each bootstrap ask)*
- Detailed chapter set: [`specs/`](./specs)
- Single source of truth for gated command/profile availability: [specs/19-feature-maturity.md](./specs/19-feature-maturity.md)

Reading rule:
- treat `BOOTSTRAP.md` as the input brief and the spec set as the normative source of truth after normalization
- when a bootstrap aspiration and a phase-specific promise seem to differ, prefer `SPEC.md` plus the owning chapter and the feature-maturity matrix
- when a support claim still feels ambiguous, use the shared **support-claim reading order** plus the **compatibility delivery ladder** in `SPEC.md` before assuming Kali means one undifferentiated notion of “support”
- remember the main naming splits used across the specs: config stores compatibility switches under `compat.features` while emitted reports use `compatFeatures`; cross-spec semantic axes use leaf names such as `apiSurface` / `buildMode` / `runtimeProfiles` while concrete `kali.json` storage uses paths such as `compilerOptions.apiSurface` / `compilerOptions.buildMode` / `compilerOptions.runtimeProfiles`; semantic effect kinds such as `FileSystem.Read` map onto policy/schema keys such as `effects.fileSystem.read`; and registry-package CLI/manifests/logical-root labels use the identifier spelling (`lodash`, `jsr:@std/path`) while structured JSON metadata uses the decomposed package-coordinate form (`registry`, `name`, `version`)
- for maintenance, keep the ownership split tight: command shape/flags live in `12-cli`, diagnostic semantics in `15-errors`, JSON field names in `18-schemas`, and phase availability in `19-feature-maturity`
- registry-analysis commands intentionally stay simpler than the canonical **source-graph commands** from [SPEC.md](./SPEC.md): `package-effects` inherits `apiSurface` / `runtimeProfiles` / `compat.features` from config/defaults rather than taking its own `--api` / `--wasm-threads` / `--compat` flag family, while `package-audit` is context-free in schema v1
- if that inherited `package-effects` context later resolves to `apiSurface = browser`, it reuses the same browser-targeted analysis context as other browser analysis commands, but only once `package-effects` itself exists; this still does **not** make it part of the exact Phase-1 browser-targeted command set

Quick navigation:
- frontend and language design: [01 — Architecture](./specs/01-architecture.md), [02 — Lexer & Parser](./specs/02-lexer-parser.md), [03 — AST](./specs/03-ast.md), [04 — Type System](./specs/04-type-system.md)
- lowering, memory, optimization, and code generation: [05 — IR](./specs/05-ir.md), [06 — Memory Management](./specs/06-memory.md), [07 — Optimization & Specialization](./specs/07-specialization.md), [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- sandboxing, runtime, APIs, and embedding: [09 — Sandboxing & Effects](./specs/09-sandboxing.md), [10 — Runtime](./specs/10-runtime.md), [11 — Standard APIs](./specs/11-standard-apis.md), [13 — Embedding](./specs/13-embedding.md)
- CLI, packages, diagnostics, schemas, testing, verification, and maturity: [12 — CLI](./specs/12-cli.md), [14 — Package Management](./specs/14-packages.md), [15 — Errors](./specs/15-errors.md), [16 — Testing](./specs/16-testing.md), [17 — Formal Verification](./specs/17-verification.md), [18 — Schemas](./specs/18-schemas.md), [19 — Feature Maturity](./specs/19-feature-maturity.md)

## Project posture
This repository is currently spec-first: the top-level spec and chapter set are the source of truth for scope, staging, and machine-readable contracts.

## Related project
- [Kai](https://github.com/rahulmutt/kai), an AI-based coding assistant
