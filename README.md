# Kali
An ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, designed for sandboxed execution, strong static analysis, and AI-friendly tooling.

## Read this repo in the right order
- [`BOOTSTRAP.md`](./BOOTSTRAP.md) — original input brief and background, not the post-normalization source of truth
- [`SPEC.md`](./SPEC.md) — normalized cross-spec rules and shared terminology
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) — what is actually shipped in each phase
- owning chapter in [`specs/`](./specs) — subsystem details
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) — current verification claim boundary

Shortcuts:
- **Is this supported yet?** → `SPEC.md` → `specs/19-feature-maturity.md` → owning chapter
- **How does it work?** → owning chapter first, then `SPEC.md`
- **What proof coverage is claimed today?** → `proofs/BOUNDARY.md`
- **How did the bootstrap brief get normalized?** → `SPEC.md`'s **Bootstrap Acceptance Snapshot**, **Bootstrap Normalization Rule**, and **Bootstrap Traceability Matrix**

README scope note:
- this file is overview-first and intentionally non-normative
- `SPEC.md` owns shared terminology/normalization, `specs/19-feature-maturity.md` owns shipped availability, owning chapters own subsystem details, and `proofs/BOUNDARY.md` owns the repository's current verification-claim boundary
- `BOOTSTRAP.md` remains the original brief, not the post-normalization source of truth
- when README summary bullets and the detailed specs differ, prefer the specs

## Hard invariants
These are fixed unless the top-level spec changes:
- **Guest-language AOT only** — no language-level JIT; host-engine WASM translation remains an execution detail rather than a second Kali compilation tier
- **Pure-Rust implementation contract** — no embedded C/C++ implementation dependencies
- **No tracing/background GC** — ownership/reference-counted strategies only where the owning chapters allow them
- **Sandbox-first honesty** — no overclaiming what Kali can actually mediate
- **Deterministic machine-readable contracts** — CLI output, diagnostics, and artifacts stay explicit and tool-friendly

## Phase 1 at a glance
Phase 1 is intentionally narrow. For exact boundaries, read the **Phase-1 Shipped Surface Summary** in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

- **Language/frontend**: `.ts` and `.js` are first-class inputs; Phase 1 JavaScript support is the bounded-inference path, not a downgraded parse-only compatibility mode.
- **Project workflow**: `kali init`, `kali init --lib`, `kali install`, `kali fmt`, `kali lint`, and `kali check [files...]` *(including the no-file project-discovery form)* are the main authoring loop.
- **Execution**: `kali run <file>` and `kali test [files...]` *(including the no-file project-discovery form for `test`)* ship only in the shared **Default standalone context (schema v1)**, using wasmtime for Kali-hosted execution on the Deno-oriented standalone surface. Production/embedding flows should prefer engine precompilation where avoiding launch-time translation matters.
- **Builds**: `kali build <file>` ships as the default executable build in the shared **Deno-oriented build context (schema v1)**; `kali build --lib <file>` ships as the Phase-1 **base library artifact** for **exact-version consumers** when Kali can determine a **statically known export surface**. In support-ladder terms, that export-oriented path is buildable for exact-version consumers, not yet the stable public embedding surface. Here, the Deno-oriented build context is the build/analysis default, not a claim that Phase-1 library outputs expose a Deno-specific public ABI. Browser-targeted `build --bundle` support is intentionally described by the separate **Browser support** bullet so the non-browser build context and the shared **Phase-1 browser-targeted command set** do not blur together; that browser-bundle path ships with a source-map companion and supports `--format esm|cjs` wrapper selection.
- **Browser support**: Phase 1 browser support is exactly the shared **Phase-1 browser-targeted command set** from [`SPEC.md`](./SPEC.md). Read that canonical term for the exact command boundary, supported `--sandbox` variants, and inherited-config equivalence rules. In support-ladder terms, browser APIs are **checkable** there and browser bundles are **deployable-through-host** there; this is not standalone browser-runtime **executable** support.
- **Sandboxing/effects**: `run/test --sandbox` remain the runtime-enforcement path. The shared **Phase-1 static policy-validation surface** from [`SPEC.md`](./SPEC.md) still owns policy-schema/config validation, and the later public effect-report surface is now the explicit `kali effects <file>` / `kali package-effects <package>` pair rather than a `run/test --dry` side path.
- **Packages**: early support is broad only inside the shared **pure JS/TS package contract** plus the documented raw-URL workflow, and every package claim should still be read through the same order: package shape → host/API fit → command maturity, all against the published artifact Kali actually installs, and only then the exact support rung. In practice, Phase-1 Deno-oriented claims may be **installable/materializable**, **checkable**, **buildable**, or **executable**, while browser-targeted claims are usually **checkable** or **deployable-through-host**. npm lifecycle hooks stay the narrow schema-v1 **install-time npm-package hook path** only: `kali install --allow-scripts` is valid only when the invocation has non-empty **effective npm-scriptable install work**. That opt-in never implies early `--api node` runtime/package compatibility.
- **Registry analysis**: follow the shared **registry-analysis command split** from [`SPEC.md`](./SPEC.md): `kali package-effects <package>` is the single-package registry-analysis/effect-report command, while `kali package-audit <package>` is the Phase 4 context-free registry-analysis/security-audit command. Output shape stays split too: `package-effects` is a native-JSON command, while `package-audit` remains the schema-v1 **envelope-only JSON command**.
- **Verification**: reuse the canonical repository summary from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md): **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.** The published boundary currently includes the widened closed fragment plus a small RC snapshot safety slice with live-reference ownership/allocation projection, release-update preservation, explicit release-recording, pure release-helper ownership/allocation and disjointness corollaries, ownership-envelope preservation on the release-only, decrement, and collection helpers, target-cell decrement bookkeeping, last-ref zeroing, zero-count collection, zero-count removal from the decrement pass via `releaseAndCollectDropsZeroCountCells`, zero-count removal from the collected heap via `releaseAndCollectRemovesZeroCountCells`, positive-count preservation on the local collection helper, the helper-level theorem that positive-count cells from the original heap survive when they are not the released target, unrelated-heap preservation, the local `releaseAndCollect` release-recording/disjointness theorems, the local `releaseAndCollect` other-live-preservation theorem, the local `releaseAndCollect` heap-characterisation theorem, the helper-level theorem that the local collection helper's final heap contains only positive-count cells, the helper-level theorem that original zero-count cells are dropped from the final heap, the helper-level ownership/allocation preservation corollaries on the decrement and collection paths, and a refcount-decrement update helper, plus a widened HIR lowering-correctness slice that now also includes bare throw.

### Phase 1 explicit non-goals
These are the most common bootstrap overreads to reject up front:
- no standalone browser `run` / `test` contract yet
- no supported `--api node` command path yet
- no stable public `kali package-audit` workflow in Phase 1
- no compile/check-time inferred-effect-vs-policy rejection yet on `check/build --sandbox`
- no stable public embedding surface yet beyond the Phase-1 **base library artifact** (so no stable public Rust embedding API or stable public WIT/C ABI/component flow yet)
- no executable project-local sandbox policy code
- no executable `eval` / `Function()` compatibility path yet
- no threaded runtime profile yet

## Important normalization highlights
A few bootstrap asks are intentionally narrower after normalization:
- **“supports browser APIs”** means the shared **Phase-1 browser-targeted command set** first, not standalone browser `run`/`test`
- **“supports npm packages”** means support is bounded by package shape, host/API fit, and command maturity, all against the published artifact Kali actually installs, and only then by the exact support rung being claimed (`installable/materializable`, `checkable`, `buildable`, `executable`, or `deployable-through-host`) — not “everything without node-gyp works”
- **“npm lifecycle scripts ship in Phase 1”** means only the schema-v1 **install-time npm-package hook path**: `kali install --allow-scripts`, and only when the invocation has non-empty **effective npm-scriptable install work**. It does not widen ordinary runtime/package compatibility or imply early `--api node` support
- **“supports all features including eval”** means parser acceptance and later compatibility planning now, but executable `eval`/`Function()` only in the later gated compatibility path
- **“static JSON effect reporting”** becomes two later public surfaces: reporting (`kali effects <file>`, `kali package-effects <package>`) and policy comparison (`check/build --sandbox`); when reporting ships, those reports are conservative upper bounds rather than exact execution traces, and `kali effects` stays a one-root reporting command rather than a hidden project-discovery or `run --dry` mode
- **“embeddable / C API / WIT / Component Model”** means a Phase-1 base `--lib` artifact for **exact-version consumers** first, then the stable public embedding surface later, with plain public `--lib` + WIT as the canonical contract and `--capi` / `--component` as explicit projections over that same export surface
- **“formal verification”** means Kali is expected to be **proof-ready** in Phase 1, but the published proof boundary may be described as **proof-backed** once its named theorem/property claims are mechanized; the current boundary in `proofs/BOUNDARY.md` is proof-backed for the widened closed fragment — now including assignment and try/catch in addition to literals, variables, closed functions, application, sequencing, and conditionals — plus a small RC snapshot safety slice with live-reference ownership/allocation projection, release-update preservation, explicit release-recording, pure release-helper ownership/allocation and disjointness corollaries, ownership-envelope preservation on the release-only, decrement, and collection helpers, target-cell decrement bookkeeping, last-ref zeroing, zero-count collection, zero-count removal from the decrement pass, zero-count removal from the collected heap, positive-count preservation on the local collection helper, the helper-level theorem that original zero-count cells are dropped from the final heap, unrelated-heap preservation, the local `releaseAndCollect` release-recording/disjointness theorems, the local `releaseAndCollect` other-live-preservation theorem, the local `releaseAndCollect` heap-characterisation theorem, the helper-level ownership/allocation preservation corollaries on the decrement and collection paths, and a refcount-decrement update helper, and a widened HIR lowering-correctness slice that now also includes bare throw, but it does not claim the later Stage 4.2 ownership/memory-safety or lowering-correctness target
- **“take inspiration from Boa / V8 / JavaScriptCore / SpiderMonkey / Deno / tsc / Porffor / Hermes / Bun”** means design references and benchmarking inputs, not architecture-copy promises, compatibility targets, or permission to pull in non-Rust implementation dependencies
- **“latest ECMA-262 support”** means the latest **published** ECMA-262 edition for shipped parser/semantic support claims; draft or Stage-3+ proposal semantics stay explicitly gated instead of being implied by that headline

## Defined shape vs shipped availability
Some command families and artifact flows are documented before they ship so names and JSON schemas do not drift.

Examples:
- `kali effects` and `kali package-effects` are native-JSON commands in schema v1; `kali package-audit` is the Phase 4 compatibility command and uses the envelope-only JSON path
- `kali package-audit` is defined early but remains Phase 4 compatibility rather than a Phase-1 command; its schema-v1 JSON mode is the **envelope-only JSON command** path rather than a second native payload shape
- plain `--lib` is documented early as the future stable public WIT-first path, but in Phase 1 it is still only the export-oriented **base library artifact**
- `kali build --capi` and `kali build --component` are defined early but remain later public embedding artifact flows
- artifact/schema vocabulary such as `kind: wasm-component`, `role: interface-wit`, or `role: embedding-metadata` may be reserved before those flows ship; stable names do not imply earlier availability
- browser-targeted `check` / `build --bundle` availability does not imply standalone browser `run` / `test`

Rule of thumb:
- read **shape** from the owning chapter
- read **availability** from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)

Quick support-answer formula:
- **command/artifact** → what exact thing is being asked for?
- **rung** → is the claim about `installable/materializable`, `checkable`, `buildable`, `executable`, or `deployable-through-host`?
- **availability context** → which resulting command/profile/API-surface/compatibility combination applies for that rung?
- **phase** → confirm it in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) before treating it as shipped
- **evidence** → which testing/proof track owns the claim?

Preferred one-line answer:
- **`<thing>` is `<rung>` for `<command/artifact>` in `<availability context>` starting in `<phase/status>` because `<fit/gating reason>`.**

## Repository posture
This repository is currently spec-first:
- the checked-in source of truth today is the spec set plus [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md)
- example crate trees, CI layouts, Lean project layouts, and command examples describe the intended target shape, not necessarily files that already exist in the repo
- current verification claims must be read from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md), not inferred from roadmap prose

## Spec map
To keep the README aligned with `SPEC.md`, use the same implementation strata:
- Bootstrap normalization + cross-spec rules: [`SPEC.md`](./SPEC.md), [`19 — Feature Maturity`](./specs/19-feature-maturity.md)
- Frontend + semantics: [`01 — Architecture`](./specs/01-architecture.md), [`02 — Lexer & Parser`](./specs/02-lexer-parser.md), [`03 — AST`](./specs/03-ast.md), [`04 — Type System`](./specs/04-type-system.md)
- Lowering + runtime core: [`05 — Intermediate Representations`](./specs/05-ir.md), [`06 — Memory Management`](./specs/06-memory.md), [`07 — Optimization & Specialization`](./specs/07-specialization.md), [`08 — WebAssembly Code Generation`](./specs/08-wasm-codegen.md), [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md), [`10 — Runtime`](./specs/10-runtime.md), [`11 — Standard APIs`](./specs/11-standard-apis.md)
- Product/tooling surface: [`12 — CLI`](./specs/12-cli.md), [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md), [`14 — Package Management`](./specs/14-packages.md), [`15 — Error Reporting`](./specs/15-errors.md), [`16 — Testing`](./specs/16-testing.md), [`17 — Formal Verification`](./specs/17-verification.md), [`18 — Schemas`](./specs/18-schemas.md)

Reading simplification:
- `13 — Embedding, WIT & C ABI` cross-cuts runtime and tooling concerns, but the public artifact/embedding contract is grouped with the product/tooling surface to match the top-level spec's delivery planning.
