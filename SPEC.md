# Kali Specification

This document is the top-level guide and normalization layer for the Kali spec set.

It exists for three reasons:
1. turn `BOOTSTRAP.md` into one coherent phased plan,
2. define cross-spec terminology/rules that should not drift between chapters,
3. point readers to the owning chapter for each detailed subsystem.

The detailed subsystem requirements live in [`specs/`](./specs). When this document and a detailed chapter both speak about the same topic, prefer this file for cross-cutting normalization and the owning chapter for the concrete subsystem contract.

## Overview

Kali is an ahead-of-time TypeScript/JavaScript compiler and runtime targeting WebAssembly, implemented in Rust, designed around:
- strong static analysis,
- sandbox-first execution,
- deterministic machine-readable tooling,
- explicit memory/ownership decisions rather than tracing/background GC,
- aggressive but auditable specialization,
- embeddability through a Phase-1 base library artifact that is useful immediately for exact-version/internal consumers, with a Phase-2 public embedding surface: stable Rust embedding API plus the stable public **WIT-first** `--lib` contract, with `--capi` and `--component` as explicit projections/packaging flows over that same export surface.

Kali aims for broad JavaScript/TypeScript compatibility over time, but the spec deliberately phases hard features instead of implying that every aspiration is part of the MVP.

## MVP Cut at a Glance

To keep the rest of the spec readable, the normalized Phase 1 MVP can be summarized in one page:

| Axis | Phase 1 MVP contract |
|---|---|
| Language/frontend | Latest published ECMA-262 grammar, TypeScript compatibility where implemented, and first-class `.js` compilation with bounded conservative inference |
| Runtime model | AOT-only, one linked WASM payload, no tracing/background GC, Rust implementation, standardized on wasmtime for Kali-hosted execution |
| Host support | `--api deno` for Kali-hosted execution; `--api browser` only for the shared **Phase-1 browser-targeted command set** (`kali check [files...]` plus its supported `--sandbox` variants, and `kali build --bundle <file>` plus its supported `--sandbox` variants, in both explicit-flag and equivalent inherited-config forms when the effective `apiSurface` is `browser`); `--api node` remains gated |
| Sandboxing | Declarative policy files, runtime enforcement for Kali-hosted execution, policy-schema validation for `check`/`build`, no project-executed policy code |
| Effects | Internal effect bookkeeping may exist in Phase 1; the Phase-2 stable **public effect-report surface** is intentionally split into a reporting half (`kali effects`, `kali package-effects`) and a policy-comparison half (compile/check-time inferred-effect-vs-policy validation on `check/build --sandbox`) |
| Registry audit | `kali package-audit` is a separate context-free registry-analysis workflow and remains later compatibility |
| Packaging | One lock/install state, Phase-1 registry support for the **pure JS/TS package contract**, Phase-1 raw-URL lock/cache support, coverage across the Deno-first standalone path and the shared **Phase-1 browser-targeted command set** (including inherited-config equivalents), and rejection by default for the **native/binary/bootstrap-heavy package contract** |
| Embedding | Phase-1 **base library artifact** via `kali build --lib` for exact-version/internal consumers in the default/inherited Deno-oriented build context (that is, the effective `apiSurface` still resolves to `deno`); the Phase-2 **public embedding surface** adds the stable Rust API plus the stable public **WIT-first** `--lib` contract, with `--capi` and `--component` as explicit projections/packaging flows over that same export surface |
| Formal verification | Phase-1 **proof-ready** repository baseline: published **proof-boundary manifest** plus the proof-CI trigger policy for the currently modeled subset; the modeled subset may still be empty while Kali is only **proof-ready**, and no proof-backed release/support claims may extend beyond the published boundary |
| Tooling | Deno-inspired CLI workflow, concise AI-friendly diagnostics, versioned JSON outputs, deterministic artifacts/reports |

Use this table as a reading aid only. Detailed behavior still belongs to the owning chapters and the maturity matrix.

For the compact answer to “what is actually shipped in Phase 1?”, use the **Phase-1 Shipped Surface Summary** in [specs/19-feature-maturity.md](./specs/19-feature-maturity.md) before dropping into the full command/profile matrix.

## Implementation Strata

To keep the bootstrap brief implementable, the chapter set is intentionally grouped into a small number of logical delivery units:

| Stratum | Purpose | Primary chapters |
|---|---|---|
| bootstrap normalization + cross-spec rules | Turn `BOOTSTRAP.md` into one phase-correct contract and define shared terminology/gating rules | `SPEC.md`, `19-feature-maturity` |
| frontend + semantics | Parse TS/JS, build typed program meaning, and define what Kali is allowed to infer | `01`-`04` |
| lowering + runtime core | Choose representations, lower to WASM, enforce runtime/sandbox boundaries, and define host/runtime behavior | `05`-`11` |
| product/tooling surface | Define command behavior, packages, diagnostics, schemas, tests, embedding, and proof claims | `12`-`18` |

Strata note:
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) is intentionally a cross-cutting availability overlay rather than a fourth product-only chapter; read it alongside whichever owning chapter defines the command, artifact, or subsystem contract.

### Chapter Map

To make the bootstrap brief easier to navigate, each detailed spec chapter owns one primary slice of the design:

| Chapter | Owns | Bootstrap themes primarily normalized there |
|---|---|---|
| [`01 — Architecture`](./specs/01-architecture.md) | core architecture, crate boundaries, AOT-only pipeline, pure-Rust implementation contract | AOT-only compilation, no embedded C/C++, overall engine/runtime structure |
| [`02 — Lexer & Parser`](./specs/02-lexer-parser.md) | ECMAScript/TypeScript grammar acceptance and parser behavior | latest published ECMA-262 grammar, parse-vs-support boundary |
| [`03 — AST`](./specs/03-ast.md) | source-level program representation and node ownership | frontend representation and analysis ergonomics |
| [`04 — Type System`](./specs/04-type-system.md) | TS-superset typing, first-class JavaScript inference, effects, constraints | stronger-than-`tsc` typing, bounded HM-style inference, pragmatic Rust-like ergonomics |
| [`05 — IR`](./specs/05-ir.md) | lowering stages and optimization-facing IR contracts | layout-aware IR design, explicit dynamic/deoptimized paths |
| [`06 — Memory`](./specs/06-memory.md) | ownership classes, escape analysis, no-tracing-GC memory model | compile-time stack/heap/shared decisions, no GC |
| [`07 — Specialization`](./specs/07-specialization.md) | generic/function/layout specialization rules and build-mode cost budgets | aggressive specialization with auditable compile-time cost |
| [`08 — WASM Codegen`](./specs/08-wasm-codegen.md) | artifact shapes, code generation, host adapter outputs | fast AOT WebAssembly generation, bundle/lib/component artifact boundaries |
| [`09 — Sandboxing & Effects`](./specs/09-sandboxing.md) | sandbox policy model, runtime enforcement, effect/policy workflow split | sandbox-first execution, static effect reporting roadmap, resource limits |
| [`10 — Runtime`](./specs/10-runtime.md) | Kali-hosted runtime behavior and dynamic-compatibility execution boundaries | runtime model, engine choice, no language-level JIT, later `eval` execution path |
| [`11 — Standard APIs`](./specs/11-standard-apis.md) | Deno/Web/Node API layering and host-surface delivery | Deno-first execution, browser-targeted analysis/build, later Node support |
| [`12 — CLI`](./specs/12-cli.md) | command shapes, flags, arity, output behavior, exit-code ownership | Deno-inspired workflow, concise AI-friendly CLI behavior |
| [`13 — Embedding, WIT & C ABI`](./specs/13-embedding.md) | embedding surface, WIT-first library contract, C ABI, component packaging | embeddability, Rust API, C API, WIT, Component Model |
| [`14 — Package Management`](./specs/14-packages.md) | dependency resolution, install/lock rules, package-shape support, raw URLs | npm/JSR/raw-URL support, pure-JS/TS package contract |
| [`15 — Error Reporting`](./specs/15-errors.md) | diagnostic meanings, human-readable conventions, canonical error boundaries | AI-friendly errors, stable codes, compact feedback loops |
| [`16 — Testing`](./specs/16-testing.md) | evidence lanes, conformance strategy, package/browser test expectations | `tsc`-inspired test breadth, conformance and determinism evidence |
| [`17 — Formal Verification`](./specs/17-verification.md) | Lean verification program, proof-ready/proof-backed split | formal verification roadmap and proof-boundary discipline |
| [`18 — Schemas`](./specs/18-schemas.md) | machine-readable JSON/config/policy/artifact schemas | AI-consumable JSON outputs and stable machine contracts |
| [`19 — Feature Maturity`](./specs/19-feature-maturity.md) | canonical phase/status matrix for all cross-cutting support claims | exact MVP cut, later compatibility gates, availability truth source |

Reading shortcut:
- if you are deciding **whether Kali supports something yet**, read `SPEC.md` → `19-feature-maturity.md` → the owning chapter
- if you are deciding **how a supported thing works**, read the owning chapter first, then fall back to `SPEC.md` only for shared terminology or cross-spec conflict resolution

## Phase-1 Explicit Non-Goals

To keep the bootstrap brief ambitious without making the MVP blurry, Phase 1 should say these non-goals out loud:
- no standalone `run --api browser` or `test --api browser` runtime contract yet;
- no supported `--api node` command path yet;
- no stable public `kali effects` / `kali package-effects` workflow yet;
- no compile/check-time inferred-effect-vs-policy validation yet on `kali check --sandbox` / `kali build --sandbox` beyond policy schema/config validation;
- no stable public `kali package-audit` workflow yet;
- no stable public embedding ABI/WIT/C-ABI contract yet beyond the Phase-1 **base library artifact**;
- no executable project-local sandbox policy code;
- no runtime `eval` / `Function()` compatibility path yet;
- no threaded runtime profile yet.

Rule:
- Phase-1 examples may still mention these later command/profile shapes to define stable CLI/schema vocabulary,
- but Phase-1 support summaries, release notes, and tests must not imply they are already shipped.

## Phase-1 Guardrail Splits

Several later chapters reuse the same six distinctions because they are the easiest places for broad bootstrap goals to blur into accidental Phase-1 overclaims:
- **browser-targeted context** ≠ **standalone browser runtime/test contract**
- **browser ambient typing surface** ≠ **browser mediated sandbox/effect subset**
- **base library artifact** ≠ **public embedding surface**
- **internal effect bookkeeping** ≠ **public effect-report surface**
- **public effect-report surface** ≠ **context-free registry-audit surface**
- **proof-ready state** ≠ **proof-backed support state**

Reading rule:
- when a support claim feels ambiguous, check whether it accidentally crossed one of those six boundaries before assuming the broader reading
- later chapters should prefer reusing these canonical split names instead of re-explaining them in new prose each time

## Recommended Phase-1 Implementation Order

To keep the bootstrap brief actionable and avoid trying to build every aspiration at once, Phase 1 should be implemented in this order:

1. **Frontend + checking foundation** — lexer, parser, AST, name resolution, TypeScript-compatible checking, first-class JavaScript handling, and the bounded conservative inference promised for Phase 1.
2. **Deterministic package/install foundation** — `kali install`, shared lock/materialization rules, package resolution, and strict non-mutating behavior for non-install commands.
3. **Kali-hosted execution foundation** — one AOT pipeline to one linked WASM payload, `run`/`test` on the Deno-oriented standalone surface, and the Phase-1 runtime/resource sandbox contract.
4. **Build/artifact foundation** — default executable builds, the browser-bundle half of the shared **Phase-1 browser-targeted command set**, and the Phase-1 `build --lib` base library artifact.
5. **Developer workflow foundation** — `init`, `check`, `fmt`, `lint`, AI-friendly diagnostics, and stable schema-v1 JSON envelopes/artifact metadata.
6. **Phase-1 evidence hardening** — conformance tests, package corpus coverage, browser-bundle smoke tests, determinism checks required by the maturity matrix, and maintenance/hardening of the Phase-1 **proof-ready** verification baseline.

Sequencing rule:
- later Phase-1 work may deepen earlier layers, but should not bypass them with feature-specific shortcuts
- the published **proof-boundary manifest** and its proof-CI trigger policy should exist from the start of the spec-first repository state so Kali is already **proof-ready**; step 6 is about hardening and maintaining that baseline alongside the rest of the evidence lanes, not deferring proof-readiness until the end
- in particular, Phase-2/3 breadth work such as stable effect-report commands, public embedding flows, broader Node compatibility, or dynamic compatibility paths must not land by weakening the earlier hard invariants

## Phase Contracts vs Implementation Order

Kali uses two different orderings on purpose:
- **phase contracts** describe the earliest user-visible support promise for a feature,
- **implementation order** describes the recommended engineering sequence for getting there.

Rules:
- a later-phase feature may still appear in docs, schemas, CLI vocabulary, or internal crate boundaries earlier if doing so prevents naming drift,
- that early documentation does **not** promote the feature into the current phase,
- release notes, support summaries, tests, and examples must still read availability from [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md), not from the mere fact that a command/flag/artifact family already has a documented shape.

Practical consequence:
- it is valid for schema-v1 to define the stable shape of later commands such as `kali effects`, `kali package-effects`, `kali package-audit`, `kali build --capi`, or `kali build --component` before those surfaces are actually available,
- `kali package-audit` should still be read as a later context-free registry-analysis workflow, not as part of the Phase-1/2 sandbox-first effect-report surface,
- but Phase-1 support claims must still treat them as reserved or phase-gated until their maturity rows open.

Compact reading aid:

| Defined early in docs/schemas | Why define it before it ships? | Availability owner |
|---|---|---|
| `kali effects` / `kali package-effects` | stabilize the shared effect vocabulary, JSON shape, and CLI/output-mode rules before the public effect-report surface opens | [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| `kali package-audit` | reserve the separate context-free registry-analysis workflow and its envelope-only JSON contract without accidentally folding it into the effect-report surface | [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| `kali build --capi` / `kali build --component` | reserve the public embedding artifact vocabulary before the Phase-2 public embedding surface is actually shipped | [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| stable public plain `--lib` + default WIT | keep the final WIT-first library contract visible while Phase 1 still ships only the unstable **base library artifact** | [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |

Use this table when a command or artifact family is already documented but you still need to know whether Kali actually ships it yet.

## Bootstrap Normalization Rule

`BOOTSTRAP.md` is the input brief. This spec set is the normative source of truth after normalization.

Normalization rules:
- treat broad product goals in `BOOTSTRAP.md` as **directional requirements**, then map them onto explicit phase promises in the spec chapters;
- when the bootstrap says Kali “should support” something large or expensive, do **not** infer same-phase MVP support unless a chapter and the maturity matrix say so;
- when the bootstrap lists competing goals, preserve the stronger safety/determinism constraint first.

Canonical examples of that normalization:
- **“Support Node, Deno, and browser APIs”** → Phase 1 is Deno-first plus the shared **Phase-1 browser-targeted command set**; broad Node compatibility is Phase 3.
- **“Support all features including eval”** → `eval`/`Function()` are part of the long-term compatibility contract, but Phase 4-gated behind the single schema-v1 compatibility switch `eval`, and that later compatibility path must still preserve Kali's no-language-level-JIT invariant.
- **“Statically run a command and get JSON output of all potential effects”** → Phase 1 may keep internal conservative effect bookkeeping for sandboxing/runtime integration, but the stable **public effect-report surface** is Phase 2 and is intentionally split in one place: the reporting half (`kali effects`, `kali package-effects`) and the policy-comparison half (compile/check-time inferred-effect-vs-policy validation on `check/build --sandbox`). This is intentionally an analysis/reporting workflow, not a second `run --dry` / `test --dry` command family.
- **“Latest ECMA-262”** → latest **published** ECMA-262 grammar is Phase 1; draft/Stage-3+ proposal support is experimental rather than implied.
- **“Programmable sandbox policy conditions”** → project policy files stay declarative in early phases; later programmable narrowing is via host-registered predicates, not executable project policy code.
- **“Use wasmtime or wasmer”** → standardize on `wasmtime` first; alternative engines are later implementation extensions.
- **“CLI usage should be clean and similar to deno - formatting, linting, typechecking, running, etc.”** → keep one Deno-inspired workflow vocabulary (`init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`) and concise defaults, but do **not** imply flag-for-flag Deno parity or that every Deno command shape automatically exists in the same phase.
- **“Lexing/parsing/typechecking/codegen should be blazing fast, with stronger optimization modes available when users want them”** → keep one explicit build-mode vocabulary: `fast` is the bounded-cost default, while `release` and `release-advanced` are the only canonical compile-budget expansion modes; deeper optimizations should strengthen those modes instead of spawning new near-duplicate optimization tiers.
- **“Take inspiration from Haskell / Idris / Agda / Lean while staying pragmatic like Rust”** → use those languages as design references for principled typing, purity, effects, and constraint solving, but do **not** imply Phase-1 dependent types, totality checking, proof terms, or theorem-prover ergonomics in ordinary Kali programs.
- **“Support WIT / Component Model”** → Phase 1 keeps a base exported-library artifact; Phase 2 promotes plain public `--lib` into the stable **WIT-first** library contract, while Component Model packaging remains the explicit `--component` projection over that same export surface instead of becoming an implicit default for every library build.
- **“Must be embeddable / expose a C API / be easy to use as a Rust library”** → Phase 1 is library-first internally and already includes the base `kali build --lib` artifact, but the stable public Rust embedding API, stable WIT contract, host-side C ABI, and component/C-embedding packaging are Phase 2 targets.
- **“Take inspiration from Boa / V8 / JavaScriptCore / SpiderMonkey / Deno / tsc / Porffor / Hermes / Bun”** → treat these as design references and benchmarking/comparison inputs, not as promises to copy their architecture wholesale, match their extension surfaces, or inherit their implementation dependencies; Kali still resolves trade-offs through its own AOT-only, sandbox-first, and pure-Rust constraints.
- **“No GC”** → no tracing/background GC is allowed; deterministic ownership/reference-counted strategies are acceptable where the owning chapters permit them.

### Bootstrap wording shortcuts

A few bootstrap phrases are easy to overread as one broad yes/no promise when Kali actually needs a smaller cross-spec split:
- **“supports browser APIs”** should be read through three separate questions: browser ambient typing, browser-targeted bundle/deploy path, and standalone browser execution. Phase 1 ships only the first two inside the shared **Phase-1 browser-targeted command set**.
- **“supports npm packages”** should be read through the shared **package-support decision order** plus the **package-support ladder**: package shape first, then host/API fit, then command maturity, then the exact rung being claimed (`installable/materializable`, `checkable`, `buildable`, `executable`, or `deployable-through-host`).
- **“sandbox policy passed in when running”** should be read through the shared **workflow-owner split**: `run/test --sandbox` enforce at runtime, `check/build --sandbox` validate statically, and `effects` / `package-effects` report only.
- **“latest ECMA-262 support”** should be read through the shared **compatibility delivery ladder**: parser breadth, checker support, executable support, and deployable-host support are related but intentionally distinct claims.

Use these shortcuts before treating any remaining broad bootstrap sentence as an undifferentiated Phase-1 support promise.

## Bootstrap Traceability Matrix

This table is the compact “where did each bootstrap ask land?” view.

It intentionally merges three questions in one place so readers do not have to bounce between the bootstrap brief, the triage rule, and the maturity matrix just to answer “is this a hard invariant, a Phase-1 promise, or a later breadth target?”.

| Bootstrap theme | Triage bucket | Earliest explicit promise | Normalized contract | Primary owner(s) |
|---|---|---|---|---|
| TypeScript + first-class JavaScript compilation | Phase contract | Phase 1 MVP | TS compatibility stays broad; `.js` is a first-class input with stronger bounded inference rather than a downgraded mode | [`specs/04-type-system.md`](./specs/04-type-system.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Sandbox-first design + static effect reporting | Mixed: hard invariant + phase-gated reporting breadth | Phase 1 for enforcement/policy validation; Phase 2 for the stable **public effect-report surface** | Phase 1 ships runtime enforcement plus policy validation; Phase 2 then opens the reporting half (`kali effects`, `kali package-effects`) and the policy-comparison half (compile/check-time inferred-effect-vs-policy validation) as two explicit maturity rows under one shared effect-surface split | [`specs/09-sandboxing.md`](./specs/09-sandboxing.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| AOT only / no JIT | Hard invariant | Phase 1 MVP | Kali is language-level AOT only; runtime engine internals must not become part of the language contract | [`specs/01-architecture.md`](./specs/01-architecture.md), [`specs/10-runtime.md`](./specs/10-runtime.md) |
| No tracing GC / explicit memory decisions | Hard invariant | Phase 1 MVP | No tracing/background GC; deterministic ownership, escape analysis, and layout decisions are the core memory story, including compile-time selection between the canonical ownership classes (`stack`, `owned heap`, `shared heap`, `borrowed`) instead of deferring shared-reference strategy to an opaque runtime policy | [`specs/06-memory.md`](./specs/06-memory.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Aggressive specialization + layout-aware IR | Phase contract that deepens later | Phase 1 for the staged pipeline vocabulary plus a valid Phase-1 `HIR → LIR` path; Phases 2-3 for canonical MIR-backed layout work and deeper optimization | Optimization is staged: Phase 1 fixes the target IR pipeline shape without requiring every build to route through MIR yet, and Phases 2-3 then make MIR/layout work canonical and deepen specialization without weakening auditability | [`specs/05-ir.md`](./specs/05-ir.md), [`specs/07-specialization.md`](./specs/07-specialization.md) |
| Fast frontend/checking/codegen + explicit optimization modes | Phase contract | Phase 1 MVP | `fast` is the bounded-cost default; `release` and `release-advanced` are the canonical compile-budget expansion modes, and later optimizations should deepen those existing modes rather than inventing a second vocabulary | [`specs/01-architecture.md`](./specs/01-architecture.md), [`specs/07-specialization.md`](./specs/07-specialization.md), [`specs/12-cli.md`](./specs/12-cli.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Comprehensive conformance/testing strategy inspired by upstream `tsc` | Phase contract that deepens with compatibility breadth | Phase 1 MVP for the core evidence lanes; later phases expand corpus breadth and proof-backed claims | Phase 1 ships the core evidence tracks: unit/integration coverage, TypeScript/JavaScript checker baselines, package-corpus checks, browser-targeted smoke coverage for the shared **Phase-1 browser-targeted command set**, determinism checks, and the proof-ready verification baseline. Later phases deepen those same lanes rather than inventing a second testing vocabulary. | [`specs/16-testing.md`](./specs/16-testing.md), [`specs/17-verification.md`](./specs/17-verification.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Language/type-system inspiration from Haskell, Idris, Agda, Lean balanced with Rust pragmatism | Mixed: hard direction + phase-gated depth | Phase 1 for a pragmatic TypeScript superset with bounded inference; later for deeper purity/effect/constraint features | Treat those languages as design references for principled typing, purity, effects, and constraints, not as a Phase-1 promise of dependent types, totality checking, proof terms, or theorem-prover UX; Kali stays pragmatic and ergonomic like Rust | [`specs/04-type-system.md`](./specs/04-type-system.md), [`specs/17-verification.md`](./specs/17-verification.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Deno-inspired CLI workflow (`fmt`, `lint`, `check`, `build`, `run`, etc.) | Hard direction with phase-owned command availability | Phase 1 MVP for the core workflow vocabulary | Kali follows a Deno-inspired command/workflow shape and concise defaults, but that is a workflow/UX reference point rather than a promise of flag-for-flag Deno parity or same-phase availability for every documented command family | [`specs/12-cli.md`](./specs/12-cli.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Deno, Node, and browser support | Phase-gated breadth target | Phase 1 for Deno-first + the exact **Phase-1 browser-targeted command set**; Phase 3 for broader Node compatibility | Phase 1 is Deno-first with the shared **Phase-1 browser-targeted command set**; Node is phase-gated until Phase 3 | [`specs/11-standard-apis.md`](./specs/11-standard-apis.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Support all features including `eval` | Phase-gated compatibility breadth target | Phase 4 compatibility for the executable path | Syntax may be accepted and effects may be modeled earlier, but executable `eval`/`Function()` support is gated behind the single schema-v1 compatibility feature `eval` and must still preserve Kali's no-language-level-JIT invariant | [`specs/10-runtime.md`](./specs/10-runtime.md), [`specs/12-cli.md`](./specs/12-cli.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| npm / JSR / raw-URL package access | Phase contract with bounded package-shape scope | Phase 1 MVP inside the **pure JS/TS package contract** | Early package support is broad for packages inside the **pure JS/TS package contract** that fit the **linked-artifact model** and whose host assumptions match either the Deno-first standalone surface or the shared **Phase-1 browser-targeted command set**, but narrow for the excluded **native/binary/bootstrap-heavy package contract** | [`specs/14-packages.md`](./specs/14-packages.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Embeddability, C ABI, WIT, Component Model | Mixed: hard direction + phase-gated public surface | Phase 1 for the base `--lib` artifact; Phase 2 for the stable public embedding surface | Phase 1 ships the base `--lib` artifact; Phase 2 then makes plain public `--lib` the stable **WIT-first** library contract, while `--capi` and `--component` stay explicit projections/packaging flows over that same export surface instead of becoming implicit defaults | [`specs/13-embedding.md`](./specs/13-embedding.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Latest published ECMA-262 boundary | Phase contract with explicit exclusion boundary | Phase 1 MVP for latest published grammar; later/explicit for draft or proposal semantics | Kali tracks the latest **published** ECMA-262 edition; draft or proposal semantics stay explicitly experimental rather than implied | [`specs/02-lexer-parser.md`](./specs/02-lexer-parser.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |
| Pure Rust implementation / no embedded C or C++ | Hard invariant | Phase 1 MVP | Implementation choices must preserve the pure-Rust host/runtime/toolchain contract rather than smuggling in embedded C/C++ dependencies | [`specs/01-architecture.md`](./specs/01-architecture.md), [`specs/10-runtime.md`](./specs/10-runtime.md) |
| AI-friendly CLI and diagnostics | Hard invariant | Phase 1 MVP | Human output stays concise; JSON contracts, stable codes, and AI-friendly machine payloads are explicit product requirements | [`specs/12-cli.md`](./specs/12-cli.md), [`specs/15-errors.md`](./specs/15-errors.md), [`specs/18-schemas.md`](./specs/18-schemas.md) |
| Lean-backed verification | Phase contract with an explicit proof-boundary rule | Phase 1 for the **proof-ready** baseline (published proof boundary + proof-CI trigger policy); deeper proof coverage later | Formal verification is phased and model-based rather than implied for the full implementation on day one, and proof-backed release/support claims require a non-empty published boundary | [`specs/17-verification.md`](./specs/17-verification.md), [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |

Use this table as a navigation aid only. The owning chapters and the maturity matrix remain normative.

If a bootstrap aspiration and a detailed chapter seem in tension, prefer:
1. this normalization rule,
2. the owning chapter,
3. the feature-maturity matrix in [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

## Goal Precedence

When goals compete, Kali resolves them in this order:
1. semantic correctness,
2. sandbox honesty and auditability,
3. determinism and explicitness,
4. predictable compilation cost,
5. performance and compatibility breadth.

This ordering is intentional. Kali should reject or deopt before it silently guesses.

## Bootstrap Triage Rule

To keep `BOOTSTRAP.md` actionable without turning every aspiration into an MVP promise, classify each bootstrap ask into one of three buckets before editing any chapter:

1. **hard invariant** — must remain true across all phases unless the top-level spec is intentionally changed;
2. **phase contract** — explicitly promised for a named phase by the owning chapter and the maturity matrix;
3. **phase-gated breadth target** — important long-term direction, but not yet part of the guaranteed user-visible contract.

Canonical **hard invariants** from the bootstrap brief:
- **AOT only** — no language-level JIT path;
- **pure Rust implementation contract** — no embedded C/C++ implementation dependencies;
- **no tracing/background GC** — ownership/reference-counted strategies may exist only where the owning chapters permit them;
- **sandbox-first honesty** — policy/enforcement claims must never overpromise what Kali can actually mediate;
- **deterministic machine contracts** — JSON output, artifact/report structure, and command behavior should stay explicit and tool-friendly.

Triage heuristics:
- if a feature widens the host/runtime contract, requires dynamic code loading/reflection, or introduces a second near-duplicate workflow vocabulary, treat it as a **phase-gated breadth target** unless a chapter and [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) explicitly promote it;
- optimization, inference, or compatibility work may deepen within the hard invariants, but must not silently weaken them;
- when in doubt, preserve the hard invariant and phase-gate the broader compatibility request.

This rule is what keeps bootstrap goals such as broad Node/browser support, `eval`, programmable policy logic, and Component Model packaging aligned with the rest of the spec without letting them erase the project's safety and determinism constraints.

## Bootstrap Editing Loop

When translating a new bootstrap ask into the spec set, use this short loop before editing chapter prose:
1. **Classify the ask** with the **Bootstrap Triage Rule**.
2. **Find the owning chapter** and update phase availability there plus [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) if the user-visible contract changed.
3. **Normalize shared vocabulary here first** when the ask introduces a cross-spec term, command family, or command/context split.
4. **Prefer one canonical rule over repetition**: if two chapters would need the same long explanation, add or reuse a canonical term in this file and cross-reference it.
5. **Check the release-claim surface**: README summaries, phase summaries, and examples must still read availability from the maturity matrix rather than from aspirational wording or internal implementation scaffolding.

This loop is intentionally shorter than the full anti-drift checklist. Use it first for scoping; use the longer checklist later for wording cleanup.

## Support-Claim Checklist

To keep broad bootstrap asks from turning back into fuzzy “support” wording, any new support claim should answer these five questions explicitly before it lands in chapter prose, README summaries, or release notes:
1. **Which command or artifact shape?**
2. **Which effective context?** (the participating axes of the **effective command context** — typically `apiSurface`, command-relevant `buildMode`, relevant `runtimeProfiles`, `compat.features`, and whether `--sandbox` participates)
3. **Which delivery rung?** Reuse the shared **compatibility delivery ladder** instead of saying only “supported”.
4. **Which earliest phase/status?** The answer must line up with [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).
5. **Which evidence track owns the claim?** Reuse the testing/proof tracks instead of relying on one-off examples.

Practical rule:
- if a sentence cannot answer those five questions cheaply, it is probably too vague to be normative and should be rewritten or moved behind a cross-reference.

## Cross-Spec Simplification Rules

To keep the spec set implementable and reduce drift between chapters, Kali intentionally standardizes on a few cross-cutting simplifications:
- **one guest-facing host ABI** realized through different host adapters, rather than separate guest contracts for standalone execution, browser bundles, and embedding;
- **one linked-artifact model**, with one linked core payload per build/analysis root and any companion artifacts such as JS glue, WIT, headers, or component wrappers layered on top rather than becoming separate runtime-linked guest graphs;
- **one browser-targeted context model** reused across the shared **Phase-1 browser-targeted command set** and later browser-context analysis commands that explicitly opt into it, with browser-context `package-effects` inheriting that context from config/defaults instead of growing a package-analysis-specific `--api` flag family;
- **one install/lock state** shared across the default Deno-oriented standalone path and the shared **Phase-1 browser-targeted command set** in schema v1;
- **one package-support decision order**: decide package shape first, then host/API fit for the active context, then command/profile maturity, all under the same published-artifact reading;
- **one static-analysis workflow split**: the bootstrap's “statically run a command and get JSON output of all potential effects” request maps to `effects` / `package-effects` for reporting and `check/build --sandbox` for policy comparison, rather than adding dry-run variants of `run` / `test`;
- **one compatibility-feature name** (`eval`) for both direct `eval` and `Function()`;
- **one sandbox/effect vocabulary** for the Kali-mediated capability subset, rather than per-DOM/per-host-API policy keys;
- **one current-repository-state vs target-contract reading**: illustrative crate trees, workspace layouts, proof trees, cargo commands, and target artifact examples may define the intended implementation/package shape before the repository actually contains every listed file or crate; current-repository claims must therefore point to existing files/artifacts instead of being inferred from those target examples;
- **one published-standard boundary**: latest **published** ECMA-262 grammar in Phase 1, current-edition non-Annex-B semantics for the features Kali marks as supported, and explicit gating for Annex B corners or draft/proposal features instead of letting “latest ECMA-262” mean “everything now”.
- **one pure-Rust implementation contract**: Kali itself and its shipped dependencies remain Rust-only from the project/toolchain point of view; ordinary platform runtime/system libraries reached through Rust toolchains or OS bindings do not count as smuggling in embedded C/C++ libraries, but bundling or requiring project-specific C/C++ implementation dependencies still violates the contract.
- **one specialization key model** based on observable layout/representation fingerprints plus the small set of semantic distinctions that still affect correctness, rather than blindly keying every specialization on the full inferred source-level type.
- **one build-mode vocabulary** (`fast`, `release`, `release-advanced`) for compile-budget/performance trade-offs, rather than per-subsystem optimization tiers that would drift between CLI, optimizer, and maturity docs.

These are deliberate simplifications, not accidental omissions. Later phases may add capability, but should not fork the core vocabulary or workflow without a clear need.

## Spec-Maintenance Anti-Drift Checklist

When editing or extending the spec set, prefer referencing the owning chapter/term instead of re-explaining it with slightly different wording.

Use this checklist:
- command shape, flags, arity, `--output json`, and exit behavior belong to [`specs/12-cli.md`](./specs/12-cli.md)
- diagnostic-code meaning and error-boundary rules belong to [`specs/15-errors.md`](./specs/15-errors.md)
- JSON field names, payload schemas, artifact kinds/roles, and generated metadata-file shapes such as C ABI embedding metadata belong to [`specs/18-schemas.md`](./specs/18-schemas.md)
- phase availability belongs to [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md)
- bootstrap inspiration wording should keep the “**design references, not compatibility targets**” reading from this file instead of turning the upstream project list into an implied architecture-copy or dependency requirement
- illustrative crate/workspace/proof-tree layouts and example host/build commands should reuse the shared **current-repository-state vs target-contract reading** instead of scattering one-off “target only / not in repo yet” caveats with slightly different wording
- shared cross-spec tables/rules such as the **Command-context axis participation table**, the **canonical browser-targeted budget compatibility rule**, and the artifact-mode matrix should have exactly one normative copy in this file; other chapters may include short **reading-aid summaries** only when they label them as non-normative and point back here instead of restating a second near-duplicate normative table
- install/lock/materialization rules and command-time package selection belong to [`specs/14-packages.md`](./specs/14-packages.md)
- host/API-layering wording should reuse the **host-support staircase**
- broad “support” wording across syntax/check/build/run/bundle/policy claims should reuse the **compatibility delivery ladder** and the **support-claim reading order** instead of implying one undifferentiated notion of support
- early browser-command availability wording should reuse the **Phase-1 browser-targeted command set** when a chapter means `kali check [files...]` *(including the project-discovery no-file form and explicit-file-set forms)* or `kali build --bundle <file>` under an effective browser API surface, including their supported `--sandbox` variants
- browser ambient-typing versus sandbox/effect wording should reuse the **Browser ambient typing vs mediated capability split**
- browser command-shape versus browser-runtime availability wording should reuse the **canonical browser-surface rejection split**
- browser-targeted `--sandbox` wording should reuse the **browser-targeted static sandbox contract**
- zero-versus-positive wording for `resources.maxSpawnedProcesses` / `resources.maxThreads` and their matching CLI caps should reuse the **feature-gated zero-capable execution budgets** term instead of restating the same `0`-is-valid / positive-is-gated rule in each chapter
- compatibility-surface wording for query-only permission observation should reuse the **observation-only compatibility facade** and **recognized-but-unavailable compatibility member** terms
- library/export-oriented build wording should reuse the **compile intent**, **embedding-stability split**, **library-oriented instantiation rule**, **statically known export surface**, and **host ABI header vs program-specific exports header** terms
- source-command versus package-command wording should reuse the **source-graph command**, **dependency-graph command**, **discovery-driven command**, and **registry-analysis command** terms instead of re-listing the same command families ad hoc
- single-package registry-analysis wording should reuse the **single-package registry-analysis command**, the bundled **registry-analysis target contract (schema v1)**, **registry-analysis availability boundary**, **registry-analysis context split**, and **registry-analysis command split** instead of re-listing command shape, version selection, and project-independence details ad hoc
- shared-flag wording should reuse the **shared flag buckets**, **semantic/context flag surface**, and **JSON-mode selectors** terms instead of letting command-local prose accidentally treat output-format controls as semantic context or vice versa
- JSON machine-output wording should reuse the canonical **native-JSON command**, **envelope-only JSON command**, **JSON-producing mode**, **JSON-mode selectors**, and **registry-analysis command split** terms instead of restating near-duplicate output-mode rules
- schema-v1 `package-audit` machine-output wording should point to [specs/18-schemas.md](./specs/18-schemas.md)'s **Package Audit JSON Output (schema v1)** section instead of restating a near-duplicate envelope-only rule
- project-install/discovery interactions for raw URL dependency state should reuse the **install-time declaration graph** term
- config-discovery/install interactions without a discovered `kali.json` should reuse the **configless install split** term
- config-field wording should reuse the **config leaf key vs full config path** split: use leaf names such as `apiSurface`, `buildMode`, and `runtimeProfiles` for cross-spec semantic axes, but use concrete schema paths such as `compilerOptions.apiSurface`, `compilerOptions.buildMode`, `compilerOptions.runtimeProfiles`, and `compat.features` when a chapter means actual `kali.json` storage or diagnostic `configPath` values
- registry-package CLI/manifest spelling versus structured JSON package metadata should reuse the **registry package identifier vs package coordinate** term instead of re-explaining the `jsr:` prefix split in slightly different ways
- schema-v1 registry dependency value wording should reuse the **exact-version-first registry manifest rule (schema v1)** instead of restating the exact-version requirement in slightly different prose
- package-audit semantics that intentionally ignore inherited host-analysis/runtime config should reuse **context-free registry analysis (schema v1)** instead of restating the ignored-axis list
- package-effects configless/default-context wording should reuse **default inherited analysis context (schema v1)** instead of repeating a partial default tuple and risking drift about which axes actually participate
- package-effects inherited-context maturity wording should reuse **axis-aligned inherited analysis gating** instead of re-listing the browser/node/runtime-profile/compatibility examples in each chapter
- Phase-1 internal effect machinery versus Phase-2 stable effect-report-command wording should reuse the **effect-surface split** instead of creating new near-duplicate “effects exist internally but not publicly yet” prose in each chapter
- command-purpose wording that distinguishes reporting, policy validation, runtime enforcement, install-time hooks, and registry audit should reuse the **workflow-owner split** instead of creating overlapping “analysis”, “sandbox”, or “inspection” narratives for the same command families
- `--sandbox` behavior across `check` / `build` / `run` / `test` should reuse the **sandbox-attachment orthogonality** rule instead of re-explaining in each chapter that sandbox attachment does not change command family, file arity, compile intent, artifact mode, or API-surface gating
- verification-boundary wording should reuse the **proof-boundary manifest** term instead of scattering slightly different “modeled subset”, “proof kernel”, or “published proof scope” prose across verification, testing, and maturity chapters
- `kali init` scaffold wording should reuse the **minimal canonical scaffold contract**, the **canonical scaffold filename convention**, and the **template selection vs build artifact mode split** instead of reintroducing duplicate `main.ts` / `lib.ts` defaults or near-duplicate prose about the library template not implicitly switching later `kali build` invocations into library mode
- install-lifecycle-script wording should reuse **install-time npm-package hook path** and **effective npm-scriptable install work** instead of re-explaining the `--allow-scripts` boundary in each chapter
- explicit raw-URL install wording should reuse the **raw-URL install staging/pin workflow** term instead of re-explaining “lock/cache yes, durable declaration no” in each package/install section
- package-loading and whole-graph-linking wording should reuse the **linked-artifact model** term instead of restating slightly different “single linked payload”, “already-linked graph”, or “no runtime-linked WASM modules” prose
- package-compatibility wording should reuse the **package-support decision order**, the **package-support ladder**, the **published-artifact-first package reading**, **pure JS/TS package contract**, and **native/binary/bootstrap-heavy package contract** terms instead of repeating slightly different repo-build-pipeline caveats or native-addon / downloaded-binary exclusion lists
- graph-scope wording for analysis, effect reporting, and static sandbox validation should reuse the **resolved source graph** term instead of alternating between near-duplicate phrases such as “full statically reachable graph”, “discovered project graph”, or “linked graph rooted at the primary source input” when the same source-graph scope is meant
- source-file-kind wording should reuse **canonical source-file classes**, **executable/analyzable source-file class**, and **canonical project file set** instead of repeating long extension lists in every command chapter
- first-class `.js` support wording should reuse the **first-class JavaScript compilation** term instead of alternating between near-duplicate phrases such as “plain JavaScript mode”, “JS-first compilation”, or “JavaScript compatibility lane”
- early stronger-than-`tsc` inference wording should reuse the **bounded inference contract** and the **annotation-required inference boundary** instead of creating near-duplicate “HM-like but still fast” descriptions in architecture, checker, and maturity chapters
- checker-config wording for `compilerOptions.strict` should reuse the **strictness bundle** term instead of restating slightly different “strict mode but not many booleans” prose in each chapter

Practical rule:
- if a chapter needs more than a short paragraph to restate one of those shared rules, add or reuse a canonical term here instead of creating another near-duplicate explanation.

## Canonical Terminology

### API surface
The selected host-facing ambient/runtime family:
- `deno`
- `node`
- `browser`

`browser` is a **browser-targeted context** in early phases, not a promise of a standalone browser runtime.

Rule:
- public APIs should preserve this term explicitly: use `apiSurface` as the canonical JSON/report field name and config leaf name (in schema-v1 `kali.json`, the concrete path is `compilerOptions.apiSurface`), `ApiSurface` in typed APIs, and `api_surface` / `apiSurface`-equivalent spellings in FFI surfaces rather than collapsing the concept to a generic `api` name that could be confused with a concrete host API namespace

### Effective API surface
The final `apiSurface` value after merging built-in defaults, discovered `kali.json`, and explicit CLI flags.

Rule:
- this is just the `apiSurface` slice of the broader **effective command context**
- chapters may use “effective API surface” as shorthand when only that one axis matters
- docs should not invent alternate names such as “resolved API mode” or “active host flavor” for the same concept

### Config leaf key vs full config path
Kali uses one deliberate naming split so cross-spec terminology can stay short without making the on-disk schema ambiguous.

Canonical examples:
- semantic/config **leaf keys**: `apiSurface`, `buildMode`, `runtimeProfiles`, `strict`, `maxSpecializations`
- concrete schema-v1 `kali.json` **paths**: `compilerOptions.apiSurface`, `compilerOptions.buildMode`, `compilerOptions.runtimeProfiles`, `compilerOptions.strict`, `compilerOptions.maxSpecializations`
- compatibility config is already stored at its canonical full path: `compat.features`

Rules:
- use the short leaf-key names when a chapter is talking about semantic axes, effective-context merging, report fields, or CLI/config vocabulary alignment
- use the full schema path when a chapter is talking about actual `kali.json` layout, defaults for a stored field, or diagnostic metadata such as `Diagnostic.context.configPath`
- docs should not blur these into competing vocabularies; `apiSurface` and `compilerOptions.apiSurface` are the same concept at different specificity levels, not two different settings

### Host-support staircase
Kali's host/API story is intentionally staged as one small staircase rather than three equally mature runtimes:
1. **Web baseline** — shared JS-visible baseline APIs used across supported surfaces
2. **Deno-oriented standalone surface** — the Phase-1 primary runtime/API surface for Kali-hosted execution
3. **Browser-targeted context** — Phase-1 ambient typing + bundle/build support that targets the real browser host rather than a standalone Kali browser runtime
4. **Node compatibility surface** — later package-driven compatibility work, not a second Phase-1 primary host

Rule:
- chapters should prefer this staircase when explaining how Web baseline, Deno, browser-targeted support, and later Node compatibility relate
- docs should avoid phrasing Node and browser support as though they were simply two more Phase-1 peers of the Deno standalone runtime
- browser-targeted support and Node compatibility may both expand later, but they start from different contracts and should not be described as one generic "compatibility layer"

### Build mode
The compilation-cost/performance dial:
- `fast`
- `release`
- `release-advanced`

### First-class JavaScript compilation
The shared cross-spec meaning of Kali treating `.js` as a real compiled input, not a downgraded compatibility lane.

Rules:
- `.js` sources go through the same parser, resolver, checker, lowering pipeline, artifact modes, and optimization vocabulary as TypeScript sources
- the difference is primarily the amount of explicit type information available to the checker, not a separate "transpile-only" product mode
- early precision for `.js` follows the shared **bounded inference contract** and the **annotation-required inference boundary** rather than open-ended whole-program guessing
- when the checker cannot cheaply prove a precise `.js` type/layout fact, it must fall back conservatively (`unknown`, unions, dynamic/tagged layouts, or explicit annotation requirements) instead of inventing implicit `any` or a speculative public API
- chapters should reuse this term instead of alternating between near-duplicate phrases such as "plain JavaScript support", "JS-first compilation", or "JavaScript compatibility mode" when they mean the same product promise

### Bounded inference contract
The shared cross-spec meaning of Kali's Phase-1 “stronger than plain `tsc`, but still predictably fast” inference promise.

In scope for the early bounded contract:
- local let-bindings and small expression trees,
- analyzable return types from cheap local function-body analysis,
- destructuring from known tuple/object shapes,
- straightforward call-site generic inference,
- sometimes unannotated parameters when one obvious contextual/call-site type exists and using it does not require wide search.

Outside the early bounded contract:
- open-ended whole-program inference,
- wide cross-module/package backtracking,
- public API reshaping based on non-principal or expensive inference,
- mutually recursive inference problems that require broad iterative/global search,
- repeated speculative instantiation whose compile-time cost is hard to predict.

Rules:
- when the checker hits one of those outer cases, it should stop the advanced inference path early and fall back to an explicit annotation requirement or a conservative boundary type such as `unknown`, unions, or a dynamic/layout-conservative representation
- Kali must not invent fresh `any` merely to keep inference moving
- chapters should reuse this term instead of creating new near-duplicate “HM-like inference, but bounded” prose every time the same early contract is meant

### Annotation-required inference boundary
The canonical point where Kali intentionally stops the **bounded inference contract** and requires user help or a conservative boundary type.

Canonical early examples:
- exported/public declarations whose inferred signature would become part of a stable module/package surface but is not cheaply principal,
- mutually recursive function/value SCCs that would require broad iterative solving,
- generic or constraint cycles that trigger repeated speculative instantiation/backtracking,
- cross-module/package inference cases whose cost or API consequences are hard to bound.

Rules:
- prefer an explicit annotation or a conservative boundary type over unstable clever inference
- this boundary is about predictability and API stability, not a checker failure to understand local code
- `compilerOptions.strict = false` does not remove this boundary or license fallback to implicit `any`

### Strictness bundle
The cross-spec name for schema-v1 `compilerOptions.strict`.

Rules:
- it is one top-level checker-behavior bundle, not a menu of many early-phase sub-booleans
- it changes checker diagnostics and accepted conservative fallbacks only; it must not change runtime semantics, sandbox/effect enforcement, feature-maturity gates, or dependency-resolution behavior
- schema-v1 default is `true`
- chapters should reuse this term instead of rephrasing it as separate “strict mode”, “strict defaults”, or “TS-strict-like bundle” concepts when they mean the same config switch

### Runtime profile
An execution-capability profile orthogonal to API surface, for example:
- baseline single-threaded runtime (default)
- later `wasm-threads`

API surface and runtime profile must not be conflated.

### Default standalone context (schema v1)
The canonical no-overrides command context for early standalone-style commands:
- `apiSurface = deno`
- `buildMode = fast`
- `runtimeProfiles = []`
- `compat.features = []`

Rules:
- when docs show plain standalone-oriented examples such as `kali run main.ts`, `kali build main.ts`, or `kali test` without extra context, this is the implied starting point
- only participating axes matter for a given command: for example `check` and Phase-2 `effects` still default their analysis context from this same baseline, but they do not suddenly become build-mode-sensitive just because `buildMode = fast` exists in the shared default tuple
- inherited config or explicit CLI flags may still replace any participating axis; this term exists to avoid repeating the same four-field default tuple in multiple chapters

### Default inherited analysis context (schema v1)
The configless/default semantic analysis context reused by schema-v1 inherited-analysis workflows such as `package-effects`.

It is the analysis-axis projection of the **Default standalone context (schema v1)**:
- `apiSurface = deno`
- `runtimeProfiles = []`
- `compat.features = []`

Rules:
- use this term when a chapter means the configless/default inherited analysis knobs specifically, rather than the full command context for `run`/`build`/`test`
- this term intentionally excludes `buildMode` and `sandbox`, because they do not participate in schema-v1 inherited package analysis
- its purpose is to prevent drift between chapters that would otherwise restate only part of the broader default tuple in slightly different words

### Compile intent
The host-visible meaning of one compilation request, orthogonal to API surface, build mode, and runtime profile:
- **executable intent** — the compiled module/artifact is expected to have an executable entry contract
- **library intent** — the compiled module/artifact is expected to expose a **statically known export surface** for host calls or export-oriented artifact flows

Rules:
- CLI artifact modes and embedding compile APIs should select compile intent explicitly rather than forcing hosts to infer it later from whether they try `run` versus `instantiate` / `call`
- executable-style helpers operate only on executable-intent modules and must fail explicitly on library-intent modules
- library-intent flows reuse the shared **library-oriented instantiation rule** and the **statically known export surface** requirement

### Compat feature
An explicitly gated compatibility switch for semantics that are intentionally off by default. In schema v1, the canonical stable compat feature name is:
- `eval`

That single name covers both direct `eval` and the `Function()` constructor path.

### Config compat selection vs emitted `compatFeatures`
Kali intentionally uses two closely related spellings for the same semantic set:
- config stores compatibility switches under `compat.features`
- emitted self-contained JSON reports flatten that same set to `compatFeatures`

Rule:
- this is a shape normalization only, not a second vocabulary
- docs should not invent alternatives such as `compatFlags` or `compatMode`

### Browser-targeted context
A command context whose effective `apiSurface` is `browser`.

In Phase 1, this context is user-visible only through the shared **Phase-1 browser-targeted command set** (including equivalent inherited-config forms once the effective `apiSurface` resolves to `browser`):
- `kali check [files...]` when the effective `apiSurface` is `browser` *(including both the project-discovery no-file form and explicit-file-set forms)*
- `kali build --bundle <file>` when the effective `apiSurface` is `browser`

Later commands may reuse the same browser-targeted context only when their own maturity rows explicitly open that path.

It does **not** mean:
- a standalone Kali-hosted browser runtime,
- DOM emulation inside `kali run`/`kali test`,
- permission to expose Deno/Node globals during browser-targeted analysis/build.

### Phase-1 browser-targeted command set
The exact Phase-1 command families that expose the browser-targeted context after effective-context resolution:
- `kali check [files...]` when the effective `apiSurface` is `browser` *(including both the project-discovery no-file form and explicit-file-set forms)*
- `kali build --bundle <file>` when the effective `apiSurface` is `browser`

Included variants inside this same set:
- explicit `--api browser` spellings and equivalent inherited-config forms
- the supported browser-targeted `--sandbox` attachments for those same command families

Quick reading-aid examples:

| Effective request | Phase 1 meaning |
|---|---|
| `kali check --api browser main.ts` | supported browser-targeted analysis |
| plain `kali check main.ts` under inherited `compilerOptions.apiSurface = browser` | same supported browser-targeted analysis |
| `kali build --bundle --api browser main.ts` | supported browser-targeted bundle build |
| plain `kali build --bundle main.ts` under inherited `compilerOptions.apiSurface = browser` | same supported browser-targeted bundle build |
| `kali build --bundle --api node main.ts` | contradictory non-browser bundle shape (`E5008`); `--bundle` is the browser-only executable packaging path, not an early Node build mode |
| `kali build --api browser main.ts` | wrong browser build shape (`E5008`) until a non-bundle browser build mode exists |
| `kali build --lib --api browser lib.ts` | contradictory browser-library build shape (`E5008`); browser mode is Phase-1 `check` + `build --bundle`, not a library artifact mode |
| `kali run --api browser main.ts` / `kali test --api browser` | unavailable browser runtime/test contract (`E5006`) |

Rules:
- this term exists to stop a common ambiguity: Phase-1 browser-targeted support does **not** mean every command that performs analysis or build-like work is browser-enabled in Phase 1
- explicit CLI spellings, inherited-config forms, and the supported `--sandbox` attachments above all count as the same command set once effective-context resolution chooses `apiSurface = browser`; for example, discovered `compilerOptions.apiSurface = browser` makes plain `kali check --sandbox kali.policy.json` and `kali build --bundle main.ts` part of this same Phase-1 set rather than creating extra browser modes
- later commands such as `kali effects --api browser` or inherited browser-context `kali package-effects` reuse the same browser-targeted context only when their own maturity rows explicitly say so
- chapters should prefer this term when they mean the exact early browser-enabled command set, instead of saying only "supported browser analysis/build commands" or similar loose phrases and forcing readers to infer which commands are already in scope

### Browser ambient typing vs mediated capability split
Kali keeps one explicit boundary between two browser-related layers that are easy to blur together:
- **browser ambient typing surface** — the globals/types visible during the supported browser-targeted command set (`Window`, `Document`, DOM types, `fetch`, `URL`, and similar browser-host types)
- **Kali-mediated capability subset** — the smaller stable sandbox/effect vocabulary used by policy validation and effect reporting

Rules:
- the shared **Phase-1 browser-targeted command set** and later browser-context analysis commands that explicitly reuse it may expose the broader browser ambient typing surface without implying that every such ambient API is individually modeled by Kali's sandbox/effect system
- schema-v1 sandbox/effect contracts remain scoped to the documented browser-applicable part of the **Kali-mediated capability subset**, not one policy/effect key per DOM API
- docs should reuse this term when explaining why browser-targeted `check`/`build --bundle` can understand DOM/browser programs while browser-targeted `--sandbox` still validates only the documented mediated subset

### Canonical browser-surface rejection split
Kali uses one shared rejection boundary for browser-related command shapes in early phases:
- use **`E5008` invalid command usage** when the user selected a contradictory browser build shape for a mode that otherwise exists (for example `kali build --api browser main.ts` without `--bundle`, or pairing `--api browser` with `--lib` / `--capi` / `--component`)
- use **`E5006` unavailable feature** when the user requested a browser runtime/test contract that Kali does not yet define (for example `kali run --api browser main.ts` or `kali test --api browser`)

Rule:
- chapters should reuse this split instead of restating near-duplicate prose about browser bundle/build availability versus missing browser runtime/test support

### Kali-hosted execution
Execution where Kali or an embedding host owns the runtime/import boundary, including:
- `kali run`
- `kali test`
- embedding hosts using Kali-controlled imports

### Host adapter
The implementation layer that satisfies Kali's one guest-facing host ABI/capability model for a concrete deployment mode.

Canonical early adapters:
- **native host adapter** — used for Kali-hosted execution (`run`, `test`, embedding)
- **browser host adapter** — generated JS glue used by `build --bundle --api browser`

Rule:
- Kali keeps one guest-facing host ABI and capability vocabulary across adapters
- adapters may differ in implementation technique, but they must not silently widen the documented command/profile contract
- browser-targeted analysis/build exposing browser ambient typings does not imply one adapter entry or one sandbox key per DOM API

### Pure-Rust implementation contract
The cross-spec interpretation of “implemented in Rust” / “no embedded C or C++ libraries”.

Rules:
- Kali's implementation crates and shipped dependency stack must remain Rust-only from the project/toolchain point of view; bundling or requiring project-specific C/C++ libraries violates the contract.
- ordinary platform runtime/system libraries reached through the normal Rust toolchain, system call bindings, or OS-provided interfaces do **not** by themselves violate the contract.
- exposing a C ABI for embedding does **not** weaken this rule; a Rust implementation may publish C-callable boundaries without embedding a C/C++ implementation.
- optional user-provided external tools may be invoked only as additive post-processing helpers and must stay non-required: Kali's documented core compile/runtime pipeline, tests, and feature claims must remain valid without them, and the project must not quietly turn such tools into hidden required dependencies.
- docs should reuse this term instead of re-explaining the distinction as “pure Rust except libc”, “no C/C++ in-tree”, or “C ABI is okay because only the boundary is C”.

### Linked-artifact model
Kali's early package/build assumption that one resolved static graph lowers into one linked core guest artifact per build or analysis root.

In practice this means:
- ordinary ESM edges, lowered CommonJS edges, and other statically resolvable dependencies are folded into one already-linked graph before execution or artifact emission,
- companion outputs such as browser JS glue, WIT, headers, or component wrappers may be emitted, but they layer around that same linked core payload rather than turning it into runtime guest-module linking,
- dynamic loading forms such as non-literal `require()` or late host-driven guest module linking sit outside this model unless a later maturity row explicitly opens them.

Rules:
- package-compatibility claims that use this term mean the package's normal code graph can be resolved and lowered under this static whole-graph model; they do **not** by themselves promise that the package's selected host APIs are already supported for the active `apiSurface`
- docs should reuse this term instead of alternating between near-duplicate phrases such as “single linked payload”, “already-linked graph”, or “no runtime-linked WASM modules” when they mean the same boundary

### Published-artifact-first package reading
The shared rule for judging package compatibility by the published package/version Kali actually installs.

Rules:
- package triage is based on the installed tarball/version plus the selected entry files/conditions for the active context, not on the upstream repository's development toolchain
- repository-time code generation, bundling, or native build steps do **not** by themselves make a package unsupported if the published artifact already contains the ordinary JS/TS files Kali consumes
- install-time lifecycle-script metadata matters only when Kali must actually rely on that script path for the selected published artifact to work

### Pure JS/TS package contract
The shared early-phase package-compatibility boundary for registry packages Kali can treat as ordinary source packages.

A package stays inside this contract when:
- its shipped code is JavaScript/TypeScript rather than a native host module,
- it uses ordinary JS module systems that Kali models (`import`/ESM and supported CommonJS lowering),
- its normal install/runtime path does **not** require the **native/binary/bootstrap-heavy package contract**.

Rules:
- this term describes package-shape compatibility, not whether the package's chosen host APIs are already supported for the active `apiSurface`.
- compatibility is judged through the shared **published-artifact-first package reading**: what matters is the package tarball/version Kali actually installs plus the selected entry files/conditions for the active context.
- therefore a package can still stay inside this contract when its published artifact already contains the JS/TS files Kali consumes, even if the source repository used a heavier build pipeline to produce that published artifact.
- staying inside this contract is necessary but not sufficient for support: packages may still be phase-gated by unavailable Node/browser/runtime features.
- docs should reuse this term instead of inventing near-duplicate phrases such as “pure JS packages”, “no native addons”, or “ordinary source-only packages” when the same boundary is meant.

### Native/binary/bootstrap-heavy package contract
The shared cross-spec name for package behaviors that fall outside Kali's early ordinary-source package model.

A package is in this contract when its normal install/runtime path depends on one or more of:
- native addons or `node-gyp`,
- N-API bindings or other compiled native code,
- prebuilt native modules,
- postinstall-downloaded executables,
- other platform-specific binary/bootstrap artifacts or selection steps.

Rules:
- use the shared **published-artifact-first package reading** here: a package falls into this contract only when the published package/version Kali installs still depends on those native/binary/bootstrap steps for its normal install/runtime path, not merely because the upstream repository used such tools during development before publishing ready-to-run JS artifacts.
- this contract is rejected by default in early phases unless an owning chapter and the maturity matrix explicitly say otherwise.
- opting into npm lifecycle hooks through the **install-time npm-package hook path** does **not** promote these packages into the supported set.
- the mere presence of optional or unused lifecycle-script metadata does not by itself move a package into this contract; what matters is whether Kali must rely on that script/binary/bootstrap path for the selected published artifact to work.
- docs should reuse this term instead of repeating slightly different lists such as “native/N-API/prebuilt modules”, “binary/bootstrap-heavy packages”, or “native addon / downloaded executable packages” when the same exclusion boundary is meant.

### Package-support decision order
The shared reading order for broad package-support claims such as “Kali supports this package”.

Canonical order:
1. **package shape** — does the published package stay inside the **pure JS/TS package contract**, or does it fall into the **native/binary/bootstrap-heavy package contract**?
2. **host/API fit** — if the package shape is acceptable, do its host/API assumptions fit the active context (`deno`, the browser-targeted context, or later `node`)?
3. **command/profile maturity** — even if the package and host fit, is the selected command/profile actually available in the current phase?
4. **published artifact first** — evaluate all three steps against the published package/version Kali actually installs, not the repository's development pipeline.

Rules:
- package-shape compatibility alone does **not** imply the package is runnable in every command or API surface.
- browser-targeted package support in Phase 1 still means support inside the shared **Phase-1 browser-targeted command set**, not standalone browser execution in Kali itself.
- docs should reuse this term instead of collapsing package-shape support, host/API support, and command availability into one ambiguous “package support” claim.

### Package-support ladder
The shared shorthand for the question “supported in what sense?” once a package has already been read through the **package-support decision order**.

Canonical rungs:
1. **installable/materializable** — Kali can deterministically resolve, lock, fetch, and materialize the dependency under the documented install rules.
2. **analyzable/checkable** — Kali can parse, resolve, and type-check the published JS/TS package shape under the selected context.
3. **buildable** — Kali can lower that package successfully through the selected build path and artifact mode.
4. **executable** — Kali can execute it inside a Kali-hosted runtime for the selected command/context.
5. **deployable-through-host** — Kali can produce the documented non-Kali-hosted deployment artifact for it, such as the browser-targeted bundle path in the shared **Phase-1 browser-targeted command set**.

Rules:
- later rungs imply the earlier ones for the same package/context, but not the reverse.
- being **installable/materializable** does **not** by itself imply analyzable, buildable, executable, or deployable-through-host.
- package discussions should name the rung they mean instead of using one broad word such as “supported” for all of them at once.
- package corpus claims, release notes, and roadmap prose should record support per rung and per command/context rather than promoting a package from “installs” to “works everywhere”.

### Kali-mediated capability subset
The stable schema-v1 capability vocabulary shared across effects and sandbox policy:
- filesystem
- network
- timer
- random
- console
- process
- eval

This is the stable capability vocabulary, **not** a claim that every command/profile/API surface enables every capability.

### Built-in effect kind vs policy/schema key
Kali intentionally uses two related naming layers for effects:
- semantic built-in effect kinds such as `FileSystem.Read`, `Network.Fetch`, `Process.EnvRead`, `Timer.Schedule`, `Random.GetBytes`, `Console.Write`, and `Eval`
- schema/policy keys such as `effects.fileSystem.read`, `effects.network.fetch`, `effects.process.envRead`, `effects.timer.schedule`, `effects.random`, `effects.console`, and `effects.eval`

Rule:
- built-in effect kinds are the semantic names used by the type/effect system and effect reports
- `effects.*` keys are the policy/schema paths used for configuration and authorization
- the mapping between those two layers is centralized in [`specs/18-schemas.md`](./specs/18-schemas.md) and should not be re-invented per chapter

### Effect-surface split
Kali keeps one explicit split between internal effect machinery and the later stable user-facing effect surface:
- **internal effect bookkeeping** — conservative compiler/runtime effect facts that may exist in Phase 1 to support sandbox-first implementation, diagnostics, lowering decisions, or later-proofed integration work
- **public effect-report surface** — the stable Phase-2 user-facing effect surface, intentionally treated as one umbrella with two halves:
  - the **reporting half** — `kali effects` and `kali package-effects`
  - the **policy-comparison half** — compile/check-time inferred-effect-vs-policy validation on `kali check --sandbox ...` and `kali build --sandbox ...`

Rules:
- Phase 1 may rely on **internal effect bookkeeping** without implying that effect JSON, command availability, or machine-readable report fields are already stable
- docs should use this split when they need to explain why sandbox-first implementation can start before the stable report commands land
- when a chapter means only one Phase-2 half, it should say **reporting half** or **policy-comparison half** rather than naming the whole umbrella and making readers infer the distinction
- chapters should avoid phrasing that makes the absence of the **public effect-report surface** sound like the total absence of effect infrastructure

### Workflow-owner split
Kali keeps one canonical owner for each of the easy-to-confuse analysis/policy/install/audit workflows instead of letting multiple commands grow near-duplicate semantics.

In schema v1 this means:
- `kali effects` and `kali package-effects` are **observational reporting** commands only: they report inferred effects and do not accept `--sandbox`
- `kali check --sandbox ...` and `kali build --sandbox ...` are the **static sandbox-policy** path: Phase 1 validates policy/schema/config, and Phase 2 extends that same path with inferred-effect-vs-policy validation
- `kali run --sandbox ...` and `kali test --sandbox ...` are the **runtime enforcement** path for **Kali-hosted execution**
- `kali install --allow-scripts` is the **install-time npm-package hook path** only and stays outside the normal source-program sandbox/effect-report contract
- `kali package-audit` is the **context-free registry-analysis/security-audit** path and does not become a second host-context-aware effect/policy command

Rules:
- adding JSON output, pretty-printing, or inherited analysis context must not change which workflow owner a command belongs to
- docs should reuse this split instead of describing the same command family as partly “reporting”, partly “validation”, and partly “runtime sandboxing” depending on chapter prose
- later phases may deepen a workflow owner's capabilities, but should not create a second near-duplicate command path unless the maturity matrix opens it explicitly

### Sandbox-attachment orthogonality
Attaching `--sandbox <policy>` never changes the base command family or its existing input/artifact semantics.

In schema v1 this means:
- on `run` / `test`, `--sandbox` adds runtime policy enforcement for the same executable command/profile request,
- on `check` / `build`, `--sandbox` adds the static sandbox-policy workflow step owned by those commands: Phase 1 policy/schema/config validation first, then Phase 2 inferred-effect-vs-policy validation on that same path,
- it does **not** change `check` from a hybrid/set-oriented command into a single-entry command,
- it does **not** change `build` compile intent, artifact selection, or browser-vs-non-browser build-shape rules,
- it does **not** bypass API-surface or feature-maturity gates, whether the participating context came from CLI flags or inherited config.

Rules:
- if the underlying command/context combination is contradictory or unavailable, attaching `--sandbox` keeps the same contradiction/availability outcome and merely adds the sandbox-validation layer when that owner is otherwise valid
- chapters should reuse this term instead of re-explaining in slightly different prose that sandbox attachment is “orthogonal to artifact mode”, “does not change file arity”, or “does not create a second availability path”

### Proof-boundary manifest
The checked-in declaration of what Kali's current formal-verification claims actually cover.

Canonical location:
- `proofs/BOUNDARY.md` at the repository root

Canonical contents:
- the modeled subsystem/calculus boundary,
- the named theorem/property inventory currently claimed,
- the trusted assumptions and explicitly unmodeled features,
- the implementation/spec areas that are expected to stay aligned with that model,
- the CI rule for when proof jobs must run.

Rules:
- Phase-1 proof-backed support claims should point to one published **proof-boundary manifest** instead of scattering slightly different proof-scope descriptions across chapters
- `proofs/BOUNDARY.md` is the canonical file for that manifest in this repository; other docs may summarize it, but should not replace it with ad hoc proof-scope prose
- this manifest scopes confidence claims; it does **not** by itself promote maturity or replace command/profile evidence from [`specs/16-testing.md`](./specs/16-testing.md)
- when the implementation grows beyond the currently published proof boundary, the unsupported remainder must stay outside the manifest rather than being described as informally covered

### Proof state split
Kali keeps one explicit split between **being proof-ready** and **advertising proof-backed support**:
- **proof-ready state** — `proofs/BOUNDARY.md` exists and truthfully declares the currently modeled proof boundary
- **proof-backed support state** — release notes or support claims actively rely on formal verification as shipped evidence for some Kali behavior
- **placeholder proof-boundary manifest** — a published **proof-boundary manifest** whose modeled boundary is still empty; this is acceptable for the **proof-ready state**, but not for **proof-backed support state** claims

Rules:
- the repo should reach the **proof-ready state** early so it has one honest place to say “no mechanized coverage yet”
- the **placeholder proof-boundary manifest** is acceptable only while Kali is still avoiding proof-backed support claims
- before a release or support summary advertises formal verification as a shipped capability, the boundary must move beyond the **placeholder proof-boundary manifest**, name at least one concrete modeled subsystem, and list the claimed theorem/property inventory so the claim is genuinely proof-backed rather than merely proof-ready
- proof CI follows the proof-CI trigger policy declared by the published boundary; an empty modeled boundary requires proof jobs only for `proofs/`, and once covered implementation/spec areas are named they also become proof-CI triggers
- that proof-CI trigger policy is normative even before concrete CI workflow files land; until automation is wired up, docs must describe it as policy rather than implying that proof jobs already run in hosted CI
- chapters should reuse this term instead of re-explaining the same empty-boundary-versus-proof-backed distinction in slightly different prose

Current repository note:
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) is the single source of truth for the repository's current verification state
- README summaries, release notes, and support tables should link to that manifest instead of paraphrasing formal-verification status from memory

### Canonical browser-applicable mediated subset (schema v1)
When a chapter says browser-targeted policy/effect reasoning uses the browser-applicable part of the **Kali-mediated capability subset**, it means:
- `effects.network.fetch`, plus the capability-local cap `effects.network.maxConnections`
- `effects.timer.schedule`, `effects.timer.maxTimeoutMs`, `effects.timer.maxActiveTimers`
- `effects.random`
- `effects.console`
- later `effects.eval` only when the separate `eval` compatibility path itself exists and is enabled

It does **not** include early schema-v1 Deno/Node-oriented capability keys such as:
- `effects.fileSystem.*`
- `effects.process.*`
- `effects.network.connect`
- `effects.network.listen`

This browser-applicable subset is a **static compatibility/build-time vocabulary** for browser-targeted contexts in early phases. It is not a promise that deployed browser bundles inherit Kali-hosted runtime enforcement, and it does not create one policy/effect key per DOM or browser API.

### Observation-only compatibility facade
A host/API surface that lets programs inspect already-resolved runtime or policy state without negotiating new permissions or widening authority.

Canonical schema-v1 example:
- the read-only `Deno.permissions` facade, in its query-only compatibility form

Rules:
- these facades report state that Kali already resolved elsewhere; they are not interactive permission-prompt channels
- they are effect-free in schema v1 unless an owning chapter explicitly adds a new effect family later
- they must not imply a second sandbox-policy namespace just for observation APIs

### Deno-compatible permission descriptor subset (schema v1)
The only stable `Deno.permissions.query({ name })` descriptor names that Kali models in schema v1:
- `read`
- `write`
- `net`
- `env`
- later `run` once subprocess support exists

Rules:
- this subset exists so Kali can expose a useful Deno-compatible observation facade without inventing Kali-only permission names for unrelated capabilities such as timers, randomness, console, or `eval`
- descriptor names observe the **currently modeled capability slice**, not some future superset; in particular, `net` reflects only the network capabilities that actually exist for the active phase/API surface
- in Phase 1's standalone surface, that means `net` effectively reports the status of the modeled `fetch` path only, not future socket/listener powers
- unsupported descriptor names (for example `ffi`, `sys`, or any other non-modeled name in the current phase) follow the canonical availability failure path (`E5006`) instead of returning a misleading synthetic status
- in Phase 1, this effectively means the `read` / `write` / `net` / `env` subset only

### Stable permission status subset (schema v1)
The only stable status values for Kali's query-only `Deno.permissions` compatibility facade in schema v1:
- `granted`
- `denied`

Rules:
- Kali must not report a synthetic `prompt` state in schema v1, because the compatibility surface is observation-only and does not provide interactive escalation
- chapters should reuse this term instead of restating the same two-status rule in slightly different prose

### Recognized-but-unavailable compatibility member
An API member that Kali intentionally recognizes as part of a broader compatibility surface, but that is unavailable in the current phase/availability context and therefore fails through the canonical `E5006` path instead of behaving like an ordinary missing/unknown member.

Canonical schema-v1 examples:
- `Deno.permissions.request(...)`
- `Deno.permissions.revoke(...)`

Rules:
- use this term when the compatibility surface should remain visible/documented, but the specific member is still phase-gated
- these members must not degrade into silent no-ops, fake prompts, hidden policy mutation, or ordinary missing-member drift between checker and runtime
- ordinary absent globals/properties that are simply not part of the selected ambient surface are still handled by the usual name/type diagnostics rather than by this term

### Kali-hosted execution budgets
The schema-v1 cross-cutting `resources.*` limits used for Kali-controlled execution environments, such as:
- memory
- CPU time
- open files
- spawned processes
- threads

These budgets are part of the Kali-hosted runtime/embedding contract. They are not, by themselves, a promise that the same enforcement exists for deployed browser bundles.

### Effective execution envelope
The final runtime capability/resource ceiling for one Kali-hosted execution after all applicable limits are merged.

It is derived from:
1. intrinsic command/profile/phase/API-surface gating,
2. any attached declarative sandbox policy,
3. per-invocation tightening flags such as `--max-memory`, `--max-cpu`, `--max-open-files`, and later supported tightening caps.

Rules:
- CLI/runtime overrides may only tighten this envelope; they must not widen a stricter attached policy.
- when no sandbox policy is attached, direct invocation caps still contribute to the envelope without implying a synthesized allow-all policy file.
- this term applies to Kali-hosted execution (`run`, `test`, embedding), not to deployed browser bundles.

### Browser-targeted static sandbox contract
The canonical early-phase meaning of `--sandbox` in a browser-targeted context.

It consists of:
- static compatibility checking only,
- validation against the documented browser-applicable portion of the **Kali-mediated capability subset**,
- no promise of Kali-controlled post-deployment runtime enforcement inside a real browser host,
- no carry-over of cross-cutting **Kali-hosted execution budgets** into deployed browser bundles.

Rule:
- chapters should reference this term instead of restating near-duplicate prose about “build-time-only browser sandboxing”, “static browser policy validation”, or “no automatic browser runtime enforcement”.

### Feature-gated zero-capable execution budgets
The schema-v1 rule for execution-budget fields whose domain naturally allows an explicit zero-concurrency deny/tightening value, while any positive value still assumes the underlying capability/profile actually exists.

Canonical early examples:
- policy fields `resources.maxSpawnedProcesses` and `resources.maxThreads`
- matching CLI tightening caps such as `--max-spawned-processes` and `--max-threads`

Rules:
- omission means “no extra tightening from this source”, not an implicit zero
- `0` is a valid explicit deny/tightening value even before subprocess or threaded-profile support exists
- positive values remain availability-gated and must fail with `E5006` until the selected command/profile/API surface actually supports the corresponding capability/profile
- practical reading: positive `maxSpawnedProcesses` follows the subprocess-support phase path, while positive `maxThreads` follows the opt-in threaded-profile phase path; the cap does not mature earlier than the capability it is trying to budget
- this rule is intentionally narrower than generic numeric-cap validation: it does **not** apply to positive-only capability-local caps such as `effects.timer.maxActiveTimers` or `effects.network.maxConnections`
- browser-targeted policy validation still follows the **canonical browser-targeted budget compatibility rule**, so in browser-targeted contexts these fields may be omitted or set to `0`, but positive values remain invalid

Rule:
- use this term instead of re-explaining the same `0`-is-valid / positive-is-gated split for these fields and flags in each chapter.

### Canonical browser-targeted budget compatibility rule
Because schema-v1 `resources.*` fields are **Kali-hosted execution budgets**, browser-targeted contexts treat them as a narrow validation boundary rather than as deployed-browser guarantees.

In schema v1 this means:
- `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` are invalid whenever present in a browser-targeted policy/context,
- `resources.maxSpawnedProcesses` and `resources.maxThreads` may be omitted or set to `0`, but positive values are invalid,
- capability-local browser-applicable caps such as `effects.network.maxConnections`, `effects.timer.maxTimeoutMs`, and `effects.timer.maxActiveTimers` remain the right place for static browser-targeted limits inside the documented mediated subset.

Rule:
- use this term when a chapter means “browser-targeted validation may talk about browser-applicable capability caps, but not about Kali-hosted `resources.*` runtime budgets as though they carried over into deployed browser bundles”.

### Analysis context
The semantic context that materially affects static analysis results:
- `apiSurface`
- `runtimeProfiles`
- compatibility-feature selection (`compat.features` in config, `compatFeatures` in emitted JSON)

### Inherited analysis context
The final analysis context used by schema-v1 inherited-analysis workflows that do not take their own package-analysis-specific context flags.

Canonical early example:
- `kali package-effects <package>`

It is derived from the participating analysis axes after defaults and discovered config are merged:
- `apiSurface`
- `runtimeProfiles`
- compatibility-feature selection (`compat.features` in config, `compatFeatures` in emitted JSON)

Rules:
- in configless mode, this resolves to the **default inherited analysis context (schema v1)**
- this term exists so package-analysis docs can talk about inherited browser/node/profile/compat selection once, without restating the merge story in each chapter
- in schema v1 this inherited context is already the effective one, because there is no package-analysis-specific CLI override layer
- it affects analysis semantics and availability gating, but it does not alter package identity/version selection or the project-independence rules for registry analysis

### Command-context axis participation table
To keep effective-context validation consistent across commands, schema v1 uses one shared participation table for the main semantic axes:

| Command family | `apiSurface` | `buildMode` | `runtimeProfiles` | `compat.features` | top-level `sandbox` |
|---|---|---|---|---|---|
| `run`, `test` | participates | participates | participates | participates | participates |
| `build` | participates | participates | participates | participates | participates |
| `check` | participates | ignored | participates | participates | participates |
| `effects` | participates | ignored | participates | participates | ignored |
| `package-effects` | inherited/participates | ignored | inherited/participates | inherited/participates | ignored |
| `package-audit` | ignored | ignored | ignored | ignored | ignored |
| `fmt`, `lint` | ignored | ignored | ignored | ignored | ignored |
| `install` | ignored | ignored | ignored | ignored | ignored |
| `init` | ignored | ignored | ignored | ignored | ignored |

Rules:
- “participates” means the effective value is part of validation and semantics for that command.
- “ignored” means the command does not validate or semantically use that axis in schema v1.
- “inherited/participates” means the command has no package-analysis-specific CLI flag for that axis in schema v1, but the effective inherited value from defaults/discovered config still materially affects semantics and gating.
- this table is about command semantics only; project root/config discovery, explicit path rules, and output-format flags are separate concerns.

### Layout/representation fingerprint
A canonical specialization key fragment describing the parts of a value that materially affect generated code shape.

It is based on things like:
- concrete scalar representation (`f64`, `i32` fast path, tagged)
- object/aggregate layout class and field-offset shape
- ownership/indirection facts only when they change calling convention, lifetime handling, or runtime operations
- dynamic/boxed fallbacks when layout is not statically stable

It is intentionally **not** the full source-level type identity. Distinct source types may share one fingerprint when they lower to the same observable code shape.

Rule:
- layout-driven specialization should key primarily on these fingerprints plus any remaining semantic distinctions that still affect correctness
- chapters should not require a separate codegen instantiation merely because two source-level types have different names while lowering to the same layout/behavioral contract
- the owning details live in [`specs/05-ir.md`](./specs/05-ir.md) and [`specs/07-specialization.md`](./specs/07-specialization.md)

Build mode affects compile effort and optimization behavior, but for early effect/package-analysis contracts the main semantic analysis context is the trio above unless an owning chapter says otherwise.

### Package-resolution context
The normalized context used when selecting package entry files/conditions:
- `apiSurface`
- module edge kind (`import` vs `require`)

Rule:
- supported browser-targeted commands share one browser package-resolution rule rather than inventing per-command ladders
- in schema v1, that browser rule means the browser `exports` condition order plus any applicable `package.json#browser` rewrites, as owned by [`specs/14-packages.md`](./specs/14-packages.md)
- later browser-targeted analysis commands should reuse that same package-resolution context once their own maturity rows allow them

### Effective command context
The fully merged invocation context that a command validates and executes against:
1. built-in defaults,
2. discovered `kali.json`,
3. explicit CLI flags.

Rules:
- validation runs against this merged result rather than against only the literal CLI spelling,
- config-derived values trigger the same gating and contradiction checks as explicit flags,
- commands must not silently fall back from an unsupported effective value just because the user omitted the matching flag.

Canonical examples:
- discovered `compilerOptions.apiSurface = browser` makes `kali build --bundle main.ts` the same supported Phase-1 browser-bundle request as explicit `kali build --bundle --api browser main.ts`
- that same inherited browser value makes plain `kali build main.ts` an `E5008` command-shape contradiction until a non-bundle browser build mode exists
- discovered `compilerOptions.apiSurface = node` keeps plain `kali run main.ts` and plain `kali test` on the same `E5006` Node-availability gate as their explicit `--api node` forms

### Availability context
The normalized context used for maturity and availability checks **after** command-shape validation succeeds.

It consists of:
- the selected command,
- any command-shape/artifact-mode choice that survived contradiction checks,
- `apiSurface`,
- `runtimeProfiles`,
- compatibility-feature selection (`compat.features` in config, `compatFeatures` in emitted JSON),
- the current implementation phase/maturity table.

Rules:
- use this term when a chapter means “the combination that determines whether Kali supports this request yet” rather than only the literal CLI spelling,
- command-shape contradictions still fail first and therefore stay outside this term's responsibility,
- docs should prefer this shared term over repetitive phrases such as “phase/profile/API-surface/compatibility gating” when the same idea is meant.

### Command-shape taxonomy vs availability
The command-shape terms in this section classify how a command behaves **when that command exists** in schema v1.

Rules:
- these terms describe arity, discovery, context inheritance, and output shape, not whether the command is already Phase 1 available,
- phase availability still comes from the owning chapter plus [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md),
- docs should avoid rephrasing a command-shape term as an availability promise.

### Defined command family
A command, flag family, or artifact family whose stable shape is documented before its maturity row is open.

Canonical schema-v1 examples:
- `kali effects`
- `kali package-effects`
- `kali package-audit`
- `kali build --capi`
- `kali build --component`

Rules:
- documenting one of these families early is a vocabulary-stability move, not an availability promise,
- command-shape rules for a **defined command family** still apply once the command exists, but the command may remain phase-gated until its maturity row opens,
- docs should prefer this term over repeating looser prose such as “defined here but maybe unavailable”, “reserved future command”, or “documented in advance” when the same boundary is meant.

### JSON-producing mode
A command invocation whose primary success output is JSON.

In schema v1 this happens in exactly two ways:
- the invocation is a **native-JSON command** in its default success mode, or
- `--output json` selects the standard command envelope.

Rules:
- `--pretty` is meaningful only in **JSON-producing mode**
- output-format flags do not create a second availability path or separate semantic context
- docs should reuse this term instead of spelling out slightly different “already JSON vs wrapped JSON” rules in each chapter

### Native-JSON command
A command whose default successful output is its command-specific JSON payload rather than the standard command envelope.

Schema-v1 examples once those commands are available:
- `kali effects`
- `kali package-effects`

Rules:
- default success output is the native payload on stdout with no interleaved status/progress text
- `--output json` wraps that same payload in the standard command envelope instead of changing the payload schema
- failures without `--output json` follow the ordinary human-diagnostic path; machine-readable failure output still requires the envelope request path

### Envelope-only JSON command
A command that may support `--output json` through the standard command envelope even though schema v1 defines no dedicated success-payload schema for it yet.

Canonical schema-v1 example once that later command exists:
- `kali package-audit`

Rules:
- the stable machine-readable contract is the standard command envelope itself
- `payload` should be omitted or `null` rather than populated with ad hoc command-specific objects
- `stdout` / `stderr` remain captured text-stream fields only, not hidden structured result channels
- docs should reuse this term instead of restating a near-duplicate “envelope but no payload schema” rule per command

### Shared flag buckets
The canonical split between broad CLI flag categories.

In schema v1:
- **presentation/control flags** affect output formatting, verbosity, color, or other non-semantic command presentation/control behavior (for example `--verbose`, `--quiet`, `--color`, `--output json`, and mode-appropriate `--pretty`)
- **semantic/context flags** affect command meaning, selected analysis/runtime context, sandbox attachment, resource envelopes, or other semantic command behavior (for example `--api`, `--compat`, `--wasm-threads`, `--sandbox`, resource-cap flags, and command-specific semantic selectors)

Rules:
- docs should reuse this split instead of informally alternating between “global flags”, “shared flags”, and “semantic flags” when they mean the same two buckets
- output-format controls stay in the presentation/control bucket even when some commands place extra command-shape constraints on them
- a command chapter may still define a smaller accepted subset of either bucket for one command family, but it should say which bucket is being narrowed

### Semantic/context flag surface
The subset of **semantic/context flags** that a given command family accepts in schema v1.

Rules:
- this term is about semantic/context participation only; it does **not** include ordinary shared presentation/control flags
- use it when a chapter wants to say a command keeps a deliberately small semantic surface without implying that normal presentation controls such as `--quiet`, `--verbose`, or `--color` suddenly stop working
- command-local JSON/output controls belong to the separate **JSON-mode selectors** term below rather than being folded into semantic/context vocabulary

### JSON-mode selectors
The command-local presentation/output selectors that participate in schema-v1 JSON behavior.

In schema v1 this means:
- `--output json`
- mode-appropriate `--pretty`

Rules:
- these selectors affect presentation/envelope behavior, not semantic analysis context
- they may still be command-shape constrained: for example an **envelope-only JSON command** may require `--output json` before `--pretty` is meaningful
- use this term when a chapter needs to talk about command-local JSON/output acceptance without accidentally treating those flags as semantic/context inputs

### Direct-input command
A command whose schema-v1 shape requires exactly one explicit primary source input once that command is available:
- `run`
- `build`
- `effects`

Rule:
- this is a command-shape property, not an availability promise; for example `effects` keeps this one-input shape even while the command itself remains Phase 2-gated

### Hybrid analysis command
A command that accepts explicit files or falls back to project discovery:
- `check`

### Project-oriented command
A command that defaults to project discovery when no explicit files are given:
- `fmt`
- `lint`
- `test`

### Dependency-graph command
A command whose primary semantic input is the project's declared/discovered dependency graph rather than one explicit primary source file or one explicit registry package target.

Canonical early example:
- `install`

Rules:
- in schema v1, plain `install` without an explicit target reconciles the discovered project's dependency graph, including raw URL imports found through the canonical project-discovery result
- explicit install targets (`kali install <pkg>` or `kali install https://...`) keep `install` in the same command family; they narrow what new dependency state is requested, but do not turn the command into a source-graph command or a registry-analysis command
- this term is intentionally separate from **project-oriented command** so docs do not blur “discovers project files” with “mutates/reconciles project dependency state”

### Discovery-driven command
A command behavior that consults the **canonical project-discovery result** as part of input selection in schema v1.

Canonical early cases:
- no-argument **hybrid analysis command** `check`
- no-argument **project-oriented commands** `fmt`, `lint`, and `test`
- the source-discovery portion of the **dependency-graph command** `install`

Rules:
- this is an umbrella term for discovery behavior only; it does **not** replace the more specific command-family terms above
- `include` / `exclude`, default project-root walking, nested-project stopping, and default managed/generated-directory skipping should be described against this term when the same discovery rule applies to multiple command families
- explicit CLI file arguments still bypass discovery for input selection, except where another command-specific rule says discovery contributes additional non-primary inputs (for example the source-discovery portion of `install`)

### Set-oriented explicit-file command
A command whose explicit file arguments, when present, are interpreted as a file set rather than as one primary source input:
- `check`
- `fmt`
- `lint`
- `test`

This term is orthogonal to discovery mode:
- `check` is still the canonical **hybrid analysis command**
- `fmt`, `lint`, and `test` are still **project-oriented commands** when no explicit files are supplied

### Current-directory-scoped scaffold command
A command whose target root is always the current working directory rather than the nearest discovered ancestor project:
- `init`

In schema v1, `init` is the canonical exception to ordinary ancestor-based config discovery. It may create a nested child project inside an existing ancestor project as long as the current working directory itself does not already contain `kali.json`.

### Source-graph command
A command whose primary semantic input is a local source/import graph rooted in explicit source files or canonical project discovery.

Canonical early examples:
- `check`
- `effects`
- `build`
- `run`
- `test`

Rules:
- these commands own the explicit host/runtime-analysis flag family in schema v1 (`--api`, `--compat`, `--wasm-threads`),
- they validate against the full **effective command context** for the axes that participate in that command,
- they follow the project-root, explicit-path, and source-file-kind rules for ordinary source commands,
- they are intentionally distinct from the package-oriented **registry-analysis commands**.

### Resolved source graph
The full statically reachable source/import/dependency graph selected by one **source-graph command** after input selection, config merging, and context-sensitive resolution.

Canonical early cases:
- for the direct-input commands `build`, `run`, and `effects`, the graph is rooted at the one explicit primary source input
- for the hybrid command `check`, the graph is rooted at either the canonical project-discovery result or the explicit file set
- browser-targeted and later Node-targeted commands still use this same term; only the effective resolution context changes

Rules:
- use this term when the intended scope is “the whole graph this command actually analyzes/builds/reports on”, not only the root file textually named on the CLI
- transitive imports and resolved package entry files are part of the **resolved source graph** once reached from the selected roots
- discovery filters such as `include` / `exclude` choose or narrow roots, but they do not prune already-reached transitive dependencies inside the **resolved source graph**
- when prose needs to distinguish project-discovery from explicit-input cases, prefer saying the command validates or reports on the **resolved source graph selected by its roots** rather than reintroducing longer variants such as “rooted at the discovered project or explicit file set”
- static sandbox validation and effect reporting should reuse this same scope rather than inventing narrower root-file-only readings per command

### Registry-analysis command
A command that analyzes exactly one explicit registry package identity rather than a project graph in early phases:
- `package-effects`
- `package-audit`

These commands do not invent a no-argument whole-project analysis mode in schema v1.

### Single-package registry-analysis command
The shared schema-v1 command-shape rule for registry-analysis commands.

In schema v1 this means:
- the command takes **exactly one** explicit **identity-only registry target**,
- that target must use the canonical registry package identifier spelling,
- omitting the target, passing more than one target, or supplying a raw URL/local path is invalid command usage (`E5008`),
- the command shape is package-oriented only; it does not imply a whole-project dependency analysis mode.

Canonical early examples:
- `package-effects`
- `package-audit`

Validation-order note:
- command-shape validation still wins before base command availability for these commands,
- therefore malformed invocations such as `kali package-effects`, `kali package-effects lodash react`, or `kali package-audit ./local.ts` stay `E5008` even before those commands themselves are available in the current phase,
- the corresponding well-formed base invocations (`kali package-effects lodash`, `kali package-audit lodash`) then fall through to their own maturity gates.

Rule:
- use this term instead of restating “exactly one explicit registry package identifier, no raw URLs/local paths, no implicit whole-project mode” in each chapter.
- package version selection, inherited analysis context, and project-independence are separate rules owned by the neighboring registry-analysis terms.

### Registry-analysis target contract (schema v1)
The bundled schema-v1 target-selection contract shared by `package-effects` and `package-audit`.

It deliberately packages together the three registry-analysis rules that often drift apart when chapters paraphrase them from memory:
1. **command shape** — follow the **single-package registry-analysis command** rule: exactly one explicit canonical registry package identifier, with raw URLs, local paths, omitted targets, and multiple targets rejected as `E5008`
2. **version selection** — follow the **stable-release selection rule (schema v1)** unless a later owning chapter adds an explicit version-aware or lock-aware mode
3. **project independence** — follow the **registry-analysis project-independence rule**: current-project `kali.json`, `kali.lock`, `node_modules/`, and `.kali/cache/urls/` do not pick a different version, and the commands do not mutate project-managed dependency state

Canonical consequence:
- `package-effects` and `package-audit` are both registry-package workflows over one explicit package identity, not whole-project dependency analyzers, raw-URL analyzers, or “whatever this repo currently has installed” commands.

Rule:
- use this bundled term when a chapter needs the full early registry-analysis target-selection story instead of restating command shape, stable-release selection, and project-independence as three separate mini-lists.
- when a chapter only needs one slice, it may still reference the narrower owned term directly.

### Registry-analysis availability boundary
The shared validation-order rule for well-formed versus malformed registry-analysis invocations.

In schema v1 this means:
- malformed registry-analysis invocations still fail first with `E5008` under the **registry-analysis target contract (schema v1)** (and, when relevant, the shared **JSON-producing mode** rules),
- the corresponding well-formed base invocations then fall through to the command's own availability gate (`E5006`) until that command exists in the current phase,
- once the base command exists, narrower inherited-context/profile gates apply after that base-command gate rather than replacing it,
- output-format selectors such as `--output json` or `--pretty` never create a second availability path for the command itself.

Canonical early consequences:
- `kali package-effects` → `E5008`
- `kali package-effects lodash` before Phase 2 → `E5006`
- `kali package-audit --pretty lodash` → `E5008`
- `kali package-audit --output json lodash` before the command exists → `E5006`

Rule:
- use this term when a chapter needs the shared `E5008`-before-`E5006` boundary for registry-analysis commands instead of re-explaining the same well-formed/malformed examples.

### Registry-analysis context split
To keep single-package tooling predictable and avoid a second near-duplicate flag family:
- `package-effects`, once that command exists, is **analysis-context-aware**: it inherits `apiSurface`, `runtimeProfiles`, and the effective compatibility-feature selection from config/defaults, then records that context in JSON using the emitted field name `compatFeatures` instead of taking package-analysis-specific `--api` / runtime-profile / `--compat` flags.
- because schema v1 intentionally omits package-analysis-specific context flags, non-default `package-effects` contexts come only from defaults or discovered config; in configless mode the command therefore uses the **default inherited analysis context (schema v1)** unless/until a later spec adds explicit package-analysis context flags.
- `package-effects` follows the maturity of the inherited analysis axis instead of inventing its own separate gate table: inherited browser context lines up with browser-targeted effect analysis, inherited Node context lines up with the Node analysis gate, inherited `wasm-threads` lines up with the threaded-profile gate, and inherited compat features such as `eval` line up with their own compatibility-phase gates.
- `package-audit`, once that command exists, follows **context-free registry analysis (schema v1)**.
- both commands still continue to follow the shared **registry-analysis target contract (schema v1)** while differing only in context participation and JSON/output behavior.

### Registry-analysis command split
Kali intentionally keeps the two schema-v1 single-package registry-analysis commands small and non-overlapping instead of growing one fuzzy “package inspection” surface.

In schema v1 this means:
- `package-effects` is the **Phase 2 target** analysis-context-aware effect-report command: it inherits the shared **inherited analysis context**, follows **axis-aligned inherited analysis gating**, and is a **native-JSON command** once available.
- `package-audit` is the **Later compatibility** context-free security-audit command: it follows **context-free registry analysis (schema v1)** and is an **envelope-only JSON command** in schema v1.
- both commands still share the same **registry-analysis target contract (schema v1)**.

Rule:
- use this term when a chapter needs the top-level contrast between `package-effects` and `package-audit` instead of restating the same availability/context/JSON split in slightly different prose.
- command spelling and arity remain owned by [`specs/12-cli.md`](./specs/12-cli.md), payload schemas remain owned by [`specs/18-schemas.md`](./specs/18-schemas.md), and availability remains owned by [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).

### Registry-analysis reminder: inherited analysis context
The semantic analysis context that `package-effects` uses in schema v1.

It is already defined above in the general terminology section.

Naming simplification:
- use **inherited analysis context** as the canonical term
- in schema v1 this context is already the effective one, because `package-effects` has no package-analysis-specific CLI `--api` / runtime-profile / `--compat` override layer
- chapters should therefore not alternate between “inherited analysis context” and “effective inherited analysis context” for the same concept

Registry-analysis-specific reminder:
- for `package-effects`, it consists of built-in defaults plus discovered `kali.json` values for `apiSurface`, `runtimeProfiles`, and `compat.features`
- in configless mode, it therefore resolves to the **default inherited analysis context (schema v1)**
- top-level `sandbox` and `buildMode` stay outside this term in early phases
- if a later spec adds package-analysis-specific CLI context flags, that later spec should extend the one canonical definition above instead of creating a second near-duplicate definition here

### Axis-aligned inherited analysis gating
The schema-v1 rule for how `package-effects` availability interacts with its **inherited analysis context**.

In schema v1 this means:
- `package-effects` first follows its own base maturity row,
- once the command exists, each inherited analysis axis reuses the same maturity gate as the corresponding ordinary analysis/effect command path,
- Kali must not invent a package-analysis-specific shadow gate table or silently fall back to a smaller context.

Canonical consequences:
- inherited `apiSurface = browser` reuses the browser-targeted analysis gate,
- inherited `apiSurface = node` reuses the Node analysis gate,
- inherited `runtimeProfiles = ["wasm-threads"]` reuses the threaded-profile gate,
- inherited `compat.features = ["eval"]` reuses the compatibility-feature gate.

Rule:
- use this term instead of re-listing those axis-by-axis examples when a chapter means this exact package-effects maturity behavior.

### Context-free registry analysis (schema v1)
The early schema-v1 rule for registry-analysis commands whose semantics intentionally do not depend on inherited host-analysis/runtime configuration.

In schema v1 this means:
- inherited `apiSurface`, `buildMode`, `runtimeProfiles`, `compat.features`, and top-level `sandbox` do not change the command's semantics,
- the command still follows its own maturity row and command-shape rules,
- output-format selectors such as `--output json` still change only formatting/envelope behavior, not semantic analysis context.

Canonical early example:
- `package-audit`

Rule:
- use this term instead of restating the full ignored-axis list each time a chapter means this exact schema-v1 behavior
- this term is about semantic context participation only; it does not by itself imply anything about package version selection, cache identity, or project mutability

### Registry-analysis project-independence rule
Single-package registry-analysis commands intentionally analyze a registry package as a standalone target, not as "whatever version this project currently has installed."

Rules:
- version selection follows the shared **stable-release selection rule (schema v1)** unless an owning chapter later adds an explicit version-aware or lock-aware mode,
- the current project's `kali.json`, `kali.lock`, `node_modules/`, and `.kali/cache/urls/` must not change which package version is analyzed,
- these commands must not mutate project-managed dependency state as a side effect,
- `package-effects` may still inherit its **inherited analysis context**, but that inherited context affects analysis semantics only and must not change project-independence for package identity/version selection,
- any fetched metadata/tarballs belong to the separate **registry-analysis cache**, not to project installation state.

### Registry-analysis cache
A non-project-managed cache that registry-analysis commands may use for fetched package metadata/tarballs.

Rules:
- it is outside project-managed dependency state (`kali.json`, `kali.lock`, `node_modules/`, and `.kali/cache/urls/`),
- it may be discarded between invocations and must not be treated as an installed project dependency snapshot,
- cache identity is keyed by at least the canonical registry identifier plus the resolved concrete version,
- for analysis-context-aware registry analysis (`package-effects`), the **inherited analysis context** is also part of the cache identity so browser/deno/profile/compat analyses cannot collide accidentally.

### Configless install split
The canonical schema-v1 install behavior when config discovery finds no `kali.json` and the command therefore runs in **configless project mode**.

It has exactly three branches:
- **plain `kali install`** → succeed as a no-op when there are no dependency inputs; do not create a placeholder manifest just because the command ran
- **explicit registry-package add** (`kali install <pkg>` / `kali install --dev <pkg>`) → first create the minimal canonical manifest `{ "schemaVersion": 1 }`, then record the dependency there and continue with normal install work
- **explicit raw-URL install** (`kali install https://...`) → may create lock/cache state for that exact URL, but must not create a placeholder manifest by itself

Rule:
- chapters should reference this term instead of re-explaining the three-way configless install behavior in slightly different prose.

### Library-oriented artifact modes
Non-browser, export-oriented build modes:
- `--lib`
- `--capi`
- `--component`

### Embedding-stability split
Kali uses one shared stability split for library-oriented outputs:
- **base library artifact** — the Phase-1 `kali build --lib` output shape: export-oriented and useful immediately for exact-version/internal consumers, but still the pre-stable Phase-1 half of the public embedding surface
- **public embedding surface** — the Phase-2 stabilized public embedding story built on that same exported-library contract: the stable Rust embedding API plus the stable public **WIT-first** library contract (`--lib`), stable C ABI, and explicit Component Model packaging path
- **public embedding artifact flows** — the artifact-producing part of that Phase-2 public embedding surface: stable public `--lib` + WIT, `--capi`, and `--component`

Rule:
- docs should reference this split instead of rephrasing it as “usable but not yet stable”, “public embedding contract”, “stable public library contract”, “library-first internally”, or “WIT/C ABI/component packaging lands later” in slightly different ways
- Phase 1 shipping the **base library artifact** does **not** by itself imply the Phase-2 **public embedding surface**: no stable public Rust API, stable public **WIT-first** library contract, stable C ABI, cross-version host-loading guarantee, or component packaging yet
- once Phase 2 promotes that path, plain public `--lib` is the canonical stable **WIT-first** library contract and emits WIT by default
- `--capi` and `--component` are explicit projections/wrappers over that same **statically known export surface** rather than alternate export semantics or implicit defaults for every library build

### Host ABI header vs program-specific exports header
Kali intentionally distinguishes the stable host-side C ABI header from build-emitted program-specific export declarations.

Canonical terms:
- **host ABI header** — the stable `kali.h` header shipped by `kali_capi` and versioned with the host C ABI
- **program-specific exports header** — the generated `<entry>.exports.h` header emitted by `kali build --capi` for one compiled library's **statically known export surface**

Rules:
- docs should not use `kali.h` as a loose synonym for both headers
- `kali build --capi` emits the **program-specific exports header**, not a second copy of the **host ABI header**
- ABI/version-compatibility wording should keep the host-side `kali_capi` contract separate from the generated exported-function declarations for one library build

### Library-oriented instantiation rule
For library-oriented artifact modes:
- Kali omits any **synthetic executable entry invocation**,
- normal ECMAScript module-instantiation semantics still apply,
- therefore top-level module initialization still runs when the host instantiates the artifact,
- and the host-callable surface is the build's **statically known export surface** rather than a synthesized executable entry.

Rule:
- `--lib`, `--capi`, and `--component` all share this same instantiation rule unless an owning chapter explicitly says otherwise.
- Docs should prefer referencing this shared term instead of restating slightly different versions of the same behavior.

### Statically known export surface
The export set for a library-oriented build that Kali can determine statically after frontend lowering without relying on runtime reflection or host-side discovery.

Rules:
- ESM entry modules satisfy this directly from their explicit exports.
- CommonJS entry modules participate only when static CJS lowering can determine one fixed export set.
- If Kali cannot determine one stable export surface, library-oriented build modes fail rather than synthesizing reflective exports.

This term exists so `--lib`, `--capi`, `--component`, embedding docs, and the maturity matrix can all refer to the same export-surface requirement without restating slightly different versions.

Naming rule:
- prefer **statically known export surface** over phrases such as "proved exports" or "proved export surface" so embedding/build terminology does not blur into the separate Lean verification vocabulary.

### Logical roots
The normalized “what this report/build/test run is about” identifiers carried in schemas as `entryPoints`. Examples:
- `src/main.ts`
- a discovered test label
- `lodash`

This is a naming bridge only: schema field `entryPoints` is the canonical JSON field name.

## Phase-1 Non-Goals Snapshot

To keep the normalized bootstrap scope easy to scan, Phase 1 does **not** imply:
- general `--api node` command support across `check` / `effects` / `build` / `run` / `test`,
- standalone browser runtime or browser-hosted `run` / `test`,
- `eval` / `Function()` support,
- interactive permission-prompt / privilege-escalation flows such as `Deno.permissions.request()` / `revoke()`,
- threaded runtime profiles / `SharedArrayBuffer` / `Atomics`,
- the Phase-2 **public embedding surface**: stable public Rust embedding, the stable public **WIT-first** `--lib` contract, `--capi`, or `--component`.

These are all tracked elsewhere in the owning chapters and the maturity matrix; this snapshot exists only to make the early boundary obvious in one place.

Two additional bootstrap-driven scope clarifications belong here because they are easy to overread from the broad product brief:
- Phase 1 does **not** imply stable user-facing `kali effects`, `kali package-effects`, or `kali package-audit` workflows just because Kali is sandbox-first and internally tracks effects.
- Phase 1 does **not** imply automatic dependency installation/repair during `check` / `effects` / `build` / `run` / `test`; `kali install` remains the one project dependency mutator.

## Host/API Summary

Using the canonical **host-support staircase**:
- **standalone execution** is Deno-first,
- **browser support** is analysis/build-first,
- **Node compatibility** is a later ecosystem phase,
- **wasmtime** is the standardized early runtime engine,
- **AOT only**; no language-level JIT,
- **pure Rust only**; no embedded C/C++ libraries,
- **no tracing/background GC**,
- one guest-facing host ABI is realized through different **host adapters** rather than through unrelated per-deployment guest contracts.

Shared API-loading rule:
- Web baseline APIs are the shared baseline across supported surfaces,
- `--api deno|node|browser` selects the additional API surface beyond that baseline,
- for analysis-oriented commands (`check`, later `effects`) and build-time selection work, that means ambient typing, package-resolution, and policy/effect-modeling context rather than a promise that a runtime host is already instantiated,
- for executable commands (`run`, `test`, embedding execution), that same selection also chooses the runtime host surface mediated by the relevant host adapter,
- for browser-targeted bundle output, it selects deployment-host assumptions and browser package-resolution behavior; execution still comes from the real browser host after deployment rather than from a hidden Kali browser runtime,
- unsupported globals/modules are absent rather than shimmed by default.

## Compatibility Delivery Ladder

To keep broad bootstrap asks such as “support Node”, “support browser APIs”, or “support latest ECMA-262” from turning into accidental overclaims, Kali uses one shared compatibility delivery ladder across the spec set.

A feature may sit on different rungs at the same time depending on command/profile:

| Ladder rung | Meaning |
|---|---|
| **accepted** | Kali parses/recognizes the syntax or surface name |
| **checkable** | Kali can type-check/analyze code that uses it in the selected analysis context |
| **buildable** | Kali can produce the documented artifact shape for it |
| **executable** | Kali can execute it inside a Kali-hosted runtime/embedding context |
| **deployable-through-host** | Kali can emit an artifact that expects the real host to provide the runtime surface (for example browser bundles) |
| **policy/effect-modeled** | Kali's stable sandbox/effect contract can reason about the documented mediated subset of that surface |

Interpretation rules:
- higher rungs do not automatically imply lower or sibling rungs for every command/profile combination; the owning chapter plus the maturity matrix still decide availability
- browser support is the clearest example: many browser APIs are **checkable** and browser bundles are **deployable-through-host** in Phase 1, while the same APIs are not yet **executable** in a standalone Kali-hosted browser runtime
- syntax acceptance does not by itself imply runtime support; `eval`, `Function()`, and other dynamic surfaces may be **accepted** long before they are executable
- policy/effect modeling is intentionally narrower than ambient API visibility; DOM/browser ambient typing may be broader than the stable schema-v1 mediated capability vocabulary

Use this ladder when reading or editing any “support” claim in the spec set.

## Support-Claim Reading Order

To keep feature claims short without making them ambiguous, interpret every “Kali supports X” statement in this order:

1. **Command shape** — is the requested invocation/selector combination valid at all for the command?
   - If not, this is the `E5008` / `E5007` side of the boundary.
   - Examples: `kali build --api browser main.ts` without `--bundle`, conflicting artifact selectors, or a declaration-only file passed as a runtime/build/effect primary input.
2. **Compatibility delivery rung** — what kind of support is actually being claimed?
   - Use the shared **compatibility delivery ladder** above instead of collapsing parsing, checking, building, execution, browser deployment, and policy/effect modeling into one overloaded word.
3. **Availability context** — is that rung available for the selected command/profile/API surface/runtime-profile/compat set in the current phase?
   - If not, this is the canonical `E5006` boundary.

Practical reading examples:
- “browser support in Phase 1” means browser APIs are **checkable** and browser bundles are **deployable-through-host** for the shared **Phase-1 browser-targeted command set**; it does **not** imply standalone browser-runtime **executable** support.
- “`eval` is supported later” means the syntax is **accepted** early, may be partially **policy/effect-modeled** earlier, but does not become **executable** until the Phase-4 compatibility path.
- “Node support is phase-gated” means a well-formed `--api node` request reaches the **availability context** gate (`E5006`) rather than becoming a browser-shape contradiction (`E5008`).

When editing other chapters, prefer linking back to this reading order instead of re-explaining the same parse-vs-check-vs-build-vs-run distinction in new words.

## Browser Support Reading Aid

This section is a non-normative summary that points back to the earlier canonical browser terms instead of redefining them with duplicate headings.

### Reading aid: browser ambient typing vs mediated capability split

The earlier canonical term **Browser ambient typing vs mediated capability split** remains normative.

Practical reminder:
- browser-targeted analysis/build may expose the real browser ambient typing layer (`window`, `document`, DOM types, browser globals),
- while schema-v1 effects and sandbox policy still reason only about the documented **Kali-mediated capability subset**,
- and deployed browser bundles still run through a real browser host rather than through a standalone Kali-hosted browser runtime.

Consequences:
- the shared **Phase-1 browser-targeted command set** type-checks against browser ambient types,
- later browser-targeted analysis commands such as `effects --api browser` and inherited browser-context `package-effects` should reuse that same split instead of defining a second browser-analysis model,
- browser-targeted `--sandbox` remains a static compatibility/build-time validation contract,
- deployed browser bundles do not automatically inherit Kali-hosted runtime enforcement.

### Reading aid: browser rejection split

The earlier canonical term **Canonical browser-surface rejection split** remains normative.

Quick mnemonic:
- wrong browser **build shape** → `E5008`
- unavailable browser **runtime/test contract** → `E5006`

Representative examples:
- `kali build --api browser main.ts` → `E5008` (wrong build shape; browser builds are bundle-only early)
- `kali build --lib --api browser lib.ts` → `E5008`
- `kali build --capi --api browser lib.ts` → `E5008`
- `kali build --component --api browser lib.ts` → `E5008`
- `kali build --bundle --api node main.ts` → `E5008`
- `kali run --api browser main.ts` → `E5006`
- `kali test --api browser` → `E5006`

### Reading aid: browser-targeted static sandbox contract

The earlier canonical term **Browser-targeted static sandbox contract** remains normative.

Practical reminder:
- `--sandbox` in browser-targeted contexts validates static compatibility against the documented **Kali-mediated capability subset**,
- it applies equally to explicit `--api browser` invocations and equivalent inherited-config browser contexts,
- it does not promise Kali-controlled post-deployment sandbox enforcement inside an arbitrary real browser host,
- and cross-cutting `resources.*` budgets are still interpreted as **Kali-hosted execution budgets**.

#### Reading aid: browser-targeted budget compatibility

The normative browser-targeted budget rule is the earlier canonical term **Canonical browser-targeted budget compatibility rule** in this file.

Keep only these consequences in mind when reading other chapters:
- browser-targeted `--sandbox` remains a static compatibility/build-time contract over the documented browser-applicable mediated subset
- cross-cutting `resources.*` fields are still **Kali-hosted execution budgets**, so they do not become post-deployment browser guarantees
- if the browser-targeted budget rule changes in a future schema revision, update that one canonical definition and let the rest of the spec inherit it by reference

## Artifact-Mode Matrix

Early documented build artifact modes form one small canonical matrix:

| Build invocation shape | Meaning |
|---|---|
| `kali build foo.ts` | default executable-oriented artifact flow |
| `kali build --bundle foo.ts` when the effective `apiSurface` is `browser` | browser-targeted bundle output |
| `kali build --lib lib.ts` | Phase-1 **base library artifact**; in Phase 2 the same selector becomes part of the stable public **WIT-first** library contract and adds a default WIT sidecar |
| `kali build --capi lib.ts` | Phase-2 **public embedding artifact flow** for C embedding |
| `kali build --component lib.ts` | Phase-2 **public embedding artifact flow** for Component Model packaging |

Rules:
- `--bundle`, `--lib`, `--capi`, and `--component` are mutually exclusive unless a later chapter explicitly says otherwise,
- omitting all four selects the default **executable compile intent**,
- `--bundle` is browser-only, requires effective `apiSurface = browser`, and keeps that same executable compile intent while swapping in the browser host adapter/output shape,
- the browser-bundle row is selected by the fully merged effective context, so explicit `--api browser` and equivalent inherited-config browser forms are the same artifact-mode request,
- `--lib`, `--capi`, and `--component` are the explicit **library compile-intent** selectors in early phases,
- attaching `--sandbox <policy>` to `build` is orthogonal to artifact mode: it adds the build workflow's static sandbox-policy step (Phase 1 validation, Phase 2 later comparison) but does **not** change compile intent, artifact selection, or the command's ordinary API-surface/maturity gates,
- library-oriented artifact modes are non-browser in early phases,
- Phase 1 plain `--lib` is the **base library artifact**, and in Phase 2 that same selector becomes part of the stable **public embedding surface** rather than introducing a second plain-library mode,
- companion artifacts such as JS glue, WIT, C headers, or component wrappers do not weaken the single linked core payload rule.

## Chapter Ownership

| Topic | Owning file |
|---|---|
| architecture and phases | [`specs/01-architecture.md`](./specs/01-architecture.md) |
| lexer/parser | [`specs/02-lexer-parser.md`](./specs/02-lexer-parser.md) |
| AST | [`specs/03-ast.md`](./specs/03-ast.md) |
| type system and inference | [`specs/04-type-system.md`](./specs/04-type-system.md) |
| IR pipeline | [`specs/05-ir.md`](./specs/05-ir.md) |
| memory/ownership | [`specs/06-memory.md`](./specs/06-memory.md) |
| specialization/optimization | [`specs/07-specialization.md`](./specs/07-specialization.md) |
| WASM/code emission/artifacts | [`specs/08-wasm-codegen.md`](./specs/08-wasm-codegen.md) |
| sandbox/effects/policy | [`specs/09-sandboxing.md`](./specs/09-sandboxing.md) |
| runtime/host ABI | [`specs/10-runtime.md`](./specs/10-runtime.md) |
| standard APIs | [`specs/11-standard-apis.md`](./specs/11-standard-apis.md) |
| CLI shape and exit behavior | [`specs/12-cli.md`](./specs/12-cli.md) |
| embedding/C ABI/WIT | [`specs/13-embedding.md`](./specs/13-embedding.md) |
| packages/install/lock behavior | [`specs/14-packages.md`](./specs/14-packages.md) |
| diagnostics semantics | [`specs/15-errors.md`](./specs/15-errors.md) |
| testing/conformance evidence | [`specs/16-testing.md`](./specs/16-testing.md) |
| Lean verification | [`specs/17-verification.md`](./specs/17-verification.md) |
| JSON/config/policy schemas | [`specs/18-schemas.md`](./specs/18-schemas.md) |
| phase gating and maturity | [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) |

## Chapter Guide

Read in this order for a clean mental model:
1. this file,
2. `01` architecture,
3. `19` feature maturity,
4. language/frontend chapters `02`-`04`,
5. lowering/runtime chapters `05`-`11`,
6. toolchain/product chapters `12`-`18`.

## JSON Output Modes

To keep CLI, schemas, and command docs aligned, schema v1 uses the shared **JSON-producing mode**, **native-JSON command**, and **envelope-only JSON command** terms defined earlier in this file.

Schema-v1 command assignment:
- **native-JSON commands** once available: `effects`, `package-effects`
- canonical **envelope-only JSON command** once available: `package-audit`

Rules:
- `--pretty` is meaningful only in **JSON-producing mode**
- `--pretty` does **not** by itself switch a command into JSON-producing mode; for an **envelope-only JSON command**, `--output json` is still required
- **native-JSON commands** reserve stdout for the success payload in their default success mode, and `--output json` wraps that same payload in the standard command envelope rather than inventing a second payload shape
- these are output-format classifications only; they must not be treated as separate command surfaces, second context models, or alternate availability paths

## Command/Context Axis Participation Table

The normative schema-v1 participation table is the earlier **Command-context axis participation table** in the canonical-terminology section of this file.

This section exists only to make the reuse rule explicit:
- CLI, schemas, package-analysis docs, diagnostics, and examples should all reuse that one table instead of maintaining a second copy here or in an owning chapter
- when prose needs the table, prefer saying that an axis **participates**, is **ignored**, or is **inherited/participates** using the canonical meanings already defined earlier in this file
- if a future schema revision changes command-axis participation, update the canonical table once and have the rest of the spec set inherit that change by reference rather than by parallel edits

## Canonical Sandbox-Aware vs Sandbox-Agnostic Commands

### Sandbox-aware commands
- `run`
- `test`
- `check`
- `build`

### Effect-reporting commands
- `effects`
- `package-effects`

These are reporting commands, not alternate policy-validation entrypoints.

### Sandbox-agnostic commands
- `fmt`
- `lint`
- `install`
- `init`
- early `package-audit`

Top-level `kali.json#sandbox` is ignored by effect-reporting and sandbox-agnostic commands.

## Canonical Validation-Order Rule

Report the outermost failing gate first:
1. command shape / arity / contradictory flag combination,
2. base command availability,
3. narrower inherited-context or profile gating,
4. source-code diagnostics within the selected valid context.

In other words:
- `E5008` owns contradictory command shape,
- `E5006` owns unavailable-but-real requests inside the chosen **availability context**.

Consequences:
- contradictory browser build shapes fail before any narrower feature gate,
- a command that is itself unavailable reports that fact before reporting a narrower inherited profile problem,
- config-derived invalid effective values trigger the same checks as explicit CLI values.

## Project Discovery

### Canonical source-file classes

Kali uses one cross-spec split for source-file kinds:
- **executable/analyzable source-file class**: `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`
- declaration-only side inputs: `.d.ts`, `.d.mts`, `.d.cts`

Command-facing rule:
- runtime-bearing entrypoints and other primary program inputs use only the **executable/analyzable source-file class**,
- declaration-only files may still participate as type-loading side inputs,
- `check`, `fmt`, and `lint` may accept declaration-only files explicitly,
- passing a declaration-only file where a command requires an executable/analyzable primary input is the canonical input-kind mismatch path (`E5007`), not general CLI misuse.

### Canonical project file set

Project discovery starts from the union of those two source-file classes.

Runtime-bearing entrypoints and direct executable inputs still use only the **executable/analyzable source-file class**.

### Default project-discovery rule

This is the canonical **default project-root walk**.

If a command needs discovery and no explicit files are supplied:
- start at the effective project root,
- include the canonical project file set,
- honor `include` / `exclude` from `kali.json` when present,
- otherwise skip default managed/generated directories.

### Canonical project-discovery result

The ordered file set produced by applying the **default project-discovery rule** (plus any command-specific narrowing layered on top of it, such as test-file pattern filtering).

Rules:
- this term names the shared discovered file set before a command applies its own later semantic filters
- **discovery-driven commands** start from this result whenever their schema-v1 command shape uses discovery
- plain `install` uses this result only for the source-discovery portion of the **install-time declaration graph**; package manifest/import-map declarations remain separate inputs
- explicit CLI file arguments bypass this discovery result entirely for input selection, except that the **explicit path boundary rule** still applies

### Default excluded managed/generated directories
- `node_modules/`
- `.kali/`
- `dist/`
- `build/`
- `coverage/`
- `.git/`

### Nested project boundary rule

Discovery stops at nested child directories containing their own `kali.json`. Those are separate projects in schema v1.

### Explicit path boundary rule

For file-accepting source commands (`run`, `build`, `check`, `effects`, `fmt`, `lint`, `test`):
- explicit file/path targets must stay inside the effective project root,
- explicit file/path targets must not point into a nested child project that has its own `kali.json`,
- crossing into another project root is invalid command usage (`E5008`),
- once a target is explicit, `include` / `exclude` no longer filter it out.

This keeps explicit inputs from silently redefining project boundaries while still letting users name concrete files directly.

## Config Discovery and Configless Project Mode

Config discovery:
- commands search the current working directory and ancestors for the nearest `kali.json`,
- if found, that directory is the effective project root,
- if none is found, commands run in **configless project mode** with the current working directory as the effective project root,
- the schema-v1 exception is the **current-directory-scoped scaffold command** `init`, which always targets the current working directory instead of retargeting to a discovered ancestor project.

Configless project mode rules follow one shared **configless install split**:
- plain `kali install` is a no-op success when there are no dependency inputs,
- explicit registry-package add (`kali install <pkg>` / `kali install --dev <pkg>`) creates the minimal manifest `{ "schemaVersion": 1 }` first,
- explicit raw-URL install may create lock/cache state but must not create a placeholder manifest by itself.

## Canonical scaffold filename convention
In schema v1, `kali init` uses one minimal filename convention for the built-in templates:
- default app template → `main.ts`
- library template (`kali init --lib`) → `lib.ts`

Rules:
- these filenames are part of the default scaffold contract, not guesses made later by `run` or `build`
- later template specs may introduce other filenames only explicitly; they should not silently redefine the schema-v1 defaults
- docs should reference this convention instead of repeating the two filenames ad hoc in multiple chapters

## Minimal canonical scaffold contract
In schema v1, `kali init` should emit the smallest project scaffold that still establishes one valid project root and one obvious starter source file.

Default scaffold shape:
- create `kali.json` containing only `{ "schemaVersion": 1 }` unless the selected built-in template explicitly needs more
- use the **canonical scaffold filename convention** for the starter source file (`main.ts` for the default app template, `lib.ts` for the library template)
- do **not** create `kali.lock`, `node_modules/`, `.kali/cache/`, or placeholder dependency/sandbox/compat sections

Schema-v1 built-in scaffold outputs:

| Command | Files created by default | Files/directories intentionally not created by default |
|---|---|---|
| `kali init` | `kali.json`, `main.ts` | no `src/`, no `test/`, no `kali.lock`, no dependency state |
| `kali init --lib` | `kali.json`, `lib.ts` | no `src/`, no `test/`, no `kali.lock`, no dependency state |

Rules:
- the scaffold contract is about file presence/minimality first; the exact starter-file contents may evolve, but they should stay minimal and valid for the selected built-in template rather than growing extra boilerplate by default
- `kali init` is current-directory-scoped: it scaffolds the current working directory rather than retargeting itself to an ancestor-discovered project root
- if the current working directory already contains `kali.json`, `kali init` fails instead of overwriting the existing project root
- if only an ancestor contains `kali.json`, `kali init` may still create a nested child project rooted at the current working directory
- scaffolding is intentionally separate from dependency reconciliation; `kali install` remains the command that creates or refreshes managed dependency state
- docs should prefer this term when they mean the whole schema-v1 `init` default, instead of restating file names, minimal config contents, and “no lockfile/dependency state” rules separately

## Template selection vs build artifact mode split
Kali keeps one explicit separation between **project scaffolding** and **later artifact selection**:
- `kali init --lib` selects the library-oriented scaffold template only
- later `kali build --lib` selects the library-oriented build artifact mode / compile intent

Rules:
- choosing the library template does **not** implicitly change the default artifact mode for later `kali build` invocations or auto-select `--lib`
- docs should reuse this term instead of repeating near-duplicate prose such as “library template only”, “library scaffold does not switch later builds into library mode”, or “template choice and artifact mode remain separate knobs”
- this split exists so scaffolding can stay minimal while build commands keep their own explicit artifact selection contract

## Canonical Dependency-Management Mutability Rule

In early phases, `kali install` is the only command that mutates project-managed dependency state.

Non-install commands must not silently:
- rewrite manifests,
- repair lockfiles,
- fetch and materialize missing dependency state as a hidden side effect.

They should fail with the canonical dependency-state diagnostic path instead.

## Shared Install State vs Command-Time Package Selection

Kali uses one deliberate simplification for early package management:
- `install` locks versions and materializes package contents,
- later commands choose the final package edge at command time from that already-installed metadata using the effective analysis/runtime context.

Consequences:
- one `kali.lock` plus one materialized package tree serves both the default Deno-oriented standalone path and the shared **Phase-1 browser-targeted command set** in Phase 1,
- changing `apiSurface` between `deno` and that shared browser-targeted context changes package entry selection, not whether the project is considered installed,
- separate per-surface installs/lockfiles must not be implied unless a later lockfile revision explicitly introduces that complexity.

## Raw-URL install staging/pin workflow
The canonical meaning of an explicit raw-URL install such as `kali install https://example.com/mod.ts`.

It means:
- pin/materialize that exact raw URL into the shared lock/cache state,
- do **not** create a new manifest dependency section or durable manifest entry,
- keep durable raw-URL ownership in source imports or `kali.json#imports`,
- allow a later plain `kali install` to prune that staged URL again if the project's current declaration graph does not actually reference it.

Rules:
- docs should reuse this term instead of re-explaining “staging/pin convenience”, “lock/cache only”, or “not a second declaration channel” in each install/package chapter
- this workflow is intentionally narrower than registry-package install: explicit registry-package adds mutate the manifest, while explicit raw-URL installs only stage shared lock/cache state

## Install-Time Declaration Graph

The dependency-owning declaration set that `kali install` reconciles for one effective project root.

It includes:
- registry dependencies declared in `kali.json` (`dependencies` / `devDependencies`),
- import-map declarations from `kali.json#imports`, with only raw-URL rewrites contributing external materialization state,
- source-level raw URL imports discovered from the project's canonical discovery result for that root.

Rules:
- plain `kali install` reconciles this graph into the shared project-managed dependency state (`kali.lock`, `node_modules/`, and `.kali/cache/urls/` as applicable),
- explicit file targets passed to non-install commands do **not** retroactively widen this graph,
- if explicit non-install command targets reach additional raw URL dependency state outside the currently installed graph, the command fails with the canonical dependency-state path (`E5004`) until the project's discoverable declaration set is updated and `kali install` is rerun.

This term exists so CLI, package-management, config-discovery, and dependency-state diagnostics can all refer to the same install-owned boundary without re-explaining it differently.

## Identity-Only Registry Target

Several early package workflows intentionally take only a registry **identity**, not an inline version selector. Canonical examples:
- `kali install lodash`
- `kali install --dev jsr:@std/path`
- `kali package-effects lodash`
- `kali package-audit jsr:@std/path`

The command then applies the shared **stable-release selection rule (schema v1)**. This keeps early CLI/package flows deterministic and simple.

## Registry Package Identifier

The canonical schema-v1 spelling for a registry package target. Examples:
- npm: `lodash`, `@types/node`
- JSR: `jsr:@std/path`

This term is used consistently across:
- `kali install`
- `kali package-effects`
- `kali package-audit`
- manifest keys under `dependencies` / `devDependencies`
- logical-root labels such as effect-report `entryPoints`

## Registry package identifier vs package coordinate

Kali intentionally uses two related representations for registry packages:
- **registry package identifier** — the user-facing string spelling used by CLI arguments, manifest keys, diagnostics, and logical-root labels such as effect-report `entryPoints`; examples: `lodash`, `jsr:@std/path`
- **package coordinate** — the structured JSON form used when a schema needs decomposed metadata, typically `{ registry, name, version }`

Rules:
- npm package coordinates keep `registry: "npm"` and `name` as the bare npm package name
- JSR package coordinates keep `registry: "jsr"` and `name` as the registry-native package name **without** the `jsr:` identity marker; the prefix stays represented by `registry`, not duplicated inside `name`
- when a schema needs a stable user-facing root label or diagnostic spelling, prefer the **registry package identifier** form rather than reconstructing an ad hoc string from a package coordinate
- docs should not invent a third spelling such as embedding the `jsr:` prefix into JSON `name` fields while also carrying `registry: "jsr"`

## Stable-Release Selection Rule (schema v1)

When a schema-v1 workflow accepts an **identity-only registry target**, Kali resolves exactly one concrete version using this rule:
- select the latest non-yanked stable published release for that registry package identifier,
- do not silently choose a prerelease,
- do not infer a different version from ambient project install state unless an owning chapter later adds an explicit lock-aware/version-aware mode,
- if no acceptable stable release exists, fail with the canonical `E5001` path.

This rule keeps early install and single-package analysis flows deterministic and project-independent.

## Exact-Version-First Registry Manifest Rule (schema v1)

When schema-v1 writes a registry dependency into `kali.json`, the recorded value is the exact resolved version string, not a SemVer range.

Rules:
- this applies to registry dependency values under `dependencies` and `devDependencies`,
- schema-v1 registry manifests that use broad version-range syntax for those fields are invalid config (`E5009`) rather than a second supported dependency-policy mode,
- explicit registry adds via `kali install <pkg>` and `kali install --dev <pkg>` therefore use the **stable-release selection rule (schema v1)** first, then write that exact resolved version into the manifest,
- lockfile state and manifest intent should stay tightly aligned in schema v1,
- wider range syntax may be added later only as a separately documented manifest/CLI contract rather than being implied by identity-only install flows.

This keeps manifest edits deterministic and AI-friendly while avoiding a second hidden version-selection policy between `kali.json` and `kali.lock`.

## Registry-Analysis Mutability/Version-Selection Reading Aid

This is a non-normative reminder that points back to the earlier canonical **Registry-analysis project-independence rule** instead of redefining it with a second identical heading.

For `package-effects` and `package-audit` in schema v1:
- version selection follows the **stable-release selection rule (schema v1)**,
- current-project manifest/lock/install state does not pick a different version,
- commands may use the shared **registry-analysis cache**,
- commands must not mutate `kali.json`, `kali.lock`, `node_modules/`, or `.kali/cache/urls/`.

`package-effects` may still inherit its **inherited analysis context**; this reminder is about dependency state and version selection, not about ambient analysis semantics.

## Effective npm-Scriptable Install Work

`--allow-scripts` is meaningful only when the current `install` invocation includes npm package work that could actually run lifecycle scripts.

Rules:
- this is **invocation-scoped**, not a project-wide switch;
- it includes directly requested npm package targets and any transitively touched npm dependencies that the current install must newly materialize, relink, or otherwise reconcile in a way that could run lifecycle hooks;
- an explicit npm target such as `kali install lodash` therefore counts as non-empty effective npm-scriptable install work whenever resolution reaches the normal npm install path for that target;
- a clean no-op install on an already-synchronized graph has **empty** effective npm-scriptable install work, even if the project depends on npm packages;
- if that set is empty, `kali install --allow-scripts` is invalid usage rather than permission to silently behave like plain `install`.

## Install-Time npm-Package Hook Path

The `--allow-scripts` escape hatch is the schema-v1 **install-time npm-package hook path**.

Rules:
- it is limited to the invocation's **effective npm-scriptable install work**;
- it is not meaningful for explicit `jsr:` targets, raw URL targets, or non-install commands;
- it does **not** imply Node runtime support, project sandbox participation for install hooks, or participation in normal `kali effects` / sandbox-policy contracts;
- it does **not** make the excluded **native/binary/bootstrap-heavy package contract** supported.

## Canonical Numeric-Limit Semantics

Kali uses one cross-spec numeric-limit rule:
- positive-budget dimensions use omission as the “unspecified” state and reject `0`,
- zero-capable concurrency counters may use `0` as an explicit deny/tightening value.

Examples:
- policy fields `resources.maxMemoryMB`, `resources.maxCpuTimeMs`, and `resources.maxOpenFiles` must be positive when present,
- CLI overrides `--max-memory`, `--max-cpu`, and `--max-open-files` follow the same positive-only rule after unit normalization,
- `resources.maxSpawnedProcesses`, `resources.maxThreads`, `--max-spawned-processes`, and `--max-threads` may use `0` as an explicit deny/tightening value,
- non-zero values for later-gated capabilities/profiles remain unavailable until those capabilities/profiles exist.

## Design References, Not Compatibility Targets or Immediate Feature Promises

The bootstrap brief names projects such as Boa, V8, JavaScriptCore, SpiderMonkey, Deno, `tsc`, Porffor, Hermes, and Bun as implementation inspiration sources, and languages such as Haskell, Idris, Agda, Lean, and Rust as language-design inspiration sources.

Normalization rules:
- treat the engine/tooling list as reference points for implementation techniques, test strategy, performance investigation, packaging ergonomics, and compatibility prioritization;
- treat the language-design list as reference points for principled typing, purity/effect design, constraint solving, and ergonomics trade-offs;
- do **not** read either list as a promise to mirror any one engine's architecture, extension surface, embedding story, dependency stack, type-theory depth, theorem-prover UX, or release cadence;
- when two inspiration sources suggest different designs, Kali still follows its own goal precedence and hard invariants first: semantic correctness, sandbox honesty, determinism, predictable compilation cost, AOT-only execution, and the **Pure-Rust implementation contract**.

Practical reading rule:
- “inspired by Haskell / Idris / Agda / Lean” does **not** by itself mean Phase 1 includes dependent types, totality checking, proof terms, or interactive theorem-prover workflows in user code;
- “pragmatic and ergonomic like Rust” means the early language should prefer explicit boundaries, predictable compilation cost, and comprehensible tooling over maximal type-theory ambition.

This keeps the inspiration lists useful without letting them silently override the rest of the spec.

## Published-Standard Boundary

“Latest ECMA-262” means the latest **published** ECMA-262 edition.

It does not implicitly include:
- draft spec text,
- Stage-3+ proposals,
- proposal semantics not yet in the published standard.

Proposal support, if any, must be explicit and experimental.

## Canonical Dynamic-Loading Boundary

To preserve the single linked core payload model:
- static `import` / `export` are core,
- literal `require()` is supported when statically resolvable,
- dynamic `require()` is rejected by default early,
- literal-string `import()` is a later lowering path over the already-linked graph,
- non-literal `import(expr)` is later compatibility work.

Kali should prefer explicit gating over bundler-style guesswork.

## Representation-Downgrade Ladder

When optimization assumptions break, downgrade the representation as little as necessary:
1. keep static layout + stack ownership when possible,
2. move to static layout + owned/shared heap if lifetime/aliasing requires it,
3. use partially dynamic layout only when closed-shape reasoning is no longer sound,
4. use fully dynamic/hash-map representation only when semantics require it.

Dynamic layout is a semantic fallback, not a synonym for heap allocation.

## Reproducibility Goal

Build outputs and machine-readable reports should be deterministic by default for the same pinned inputs, config, and toolchain.

This applies to:
- emitted WASM artifacts,
- generated metadata sidecars,
- JSON envelopes/reports,
- diagnostics ordering where the producer naturally owns that order.

## Chapter Navigation

- [01 — Architecture](./specs/01-architecture.md)
- [02 — Lexer & Parser](./specs/02-lexer-parser.md)
- [03 — AST](./specs/03-ast.md)
- [04 — Type System](./specs/04-type-system.md)
- [05 — IR](./specs/05-ir.md)
- [06 — Memory Management](./specs/06-memory.md)
- [07 — Optimization & Specialization](./specs/07-specialization.md)
- [08 — WASM Codegen](./specs/08-wasm-codegen.md)
- [09 — Sandboxing & Effects](./specs/09-sandboxing.md)
- [10 — Runtime](./specs/10-runtime.md)
- [11 — Standard APIs](./specs/11-standard-apis.md)
- [12 — CLI](./specs/12-cli.md)
- [13 — Embedding](./specs/13-embedding.md)
- [14 — Package Management](./specs/14-packages.md)
- [15 — Errors](./specs/15-errors.md)
- [16 — Testing](./specs/16-testing.md)
- [17 — Formal Verification](./specs/17-verification.md)
- [18 — Schemas](./specs/18-schemas.md)
- [19 — Feature Maturity](./specs/19-feature-maturity.md)
