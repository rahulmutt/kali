# 16 — Testing

This chapter defines Kali's evidence lanes and the minimum testing discipline required before a feature may be described as supported.

Planning ownership:
- this chapter defines **what evidence is required** for a claim
- [`PLAN.md`](../PLAN.md) and [`plan/`](../plan) own **when** test infrastructure is built, expanded, or promoted in CI
- [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) owns the current proof-backed boundary

## Core rule

A feature may be documented before it ships, but it may be described as **supported** only when:
1. its maturity row is open in [19 — Feature Maturity](./19-feature-maturity.md), and
2. the matching evidence lane in this chapter exists and passes.

One demo, one fixture, or one anecdotal package success is not enough to widen a public support claim.

## Phase-correct testing rule

Treat workflow families according to their maturity owner:
- **Phase-1 shipped workflows** require positive integration coverage for every supported command/context combination.
- **Later documented workflows** may already have command shapes, schemas, or internal plumbing, but tests must assert unavailability until the matching maturity row opens.
- **Internal-only machinery** may be tested without being presented as a stable public CLI/API surface.

Examples:
- `run/test --sandbox`, the **Phase-1 static policy-validation surface**, and the **Phase-1 browser-targeted command set** need positive Phase-1 coverage.
- Historical negative examples before their rows opened were `kali effects`, `kali package-effects`, `kali package-audit`, inferred-effect-vs-policy rejection on `check/build --sandbox`, stable public embedding flows (`--capi`, `--component`, stable public `--lib` + WIT), and wider proof-backed claims.
- Once those rows are open, the same surfaces must move to positive evidence in their matching lanes instead of lingering as legacy negative tests.
- Current-repository-state shortcut: the public effect-reporting surface, the built-in effect-vs-policy validation path, the stable public embedding flows (`--lib` + WIT, `--capi`, `--component`), `kali package-audit`, and proof-backed claims for the published boundary should now be treated as **positive-evidence** lanes.

## Evidence matrix

| Concern area | Minimum evidence before claiming support |
|---|---|
| Language syntax/semantics | parser tests, integration coverage, and the applicable test262/conformance subset |
| Type checking / inference | checker baselines, inference goldens, and targeted regressions |
| First-class JavaScript compilation | dedicated `.js` fixtures across `check` / `build` / `run`, JSDoc-hint coverage, and fallback-ladder cases |
| Host APIs / runtime behavior | integration tests that execute the API path plus sandbox/resource-limit coverage where relevant |
| Phase-1 browser-targeted command set | browser-targeted `check` coverage, browser-targeted `build --bundle` coverage, and emitted-bundle smoke runs in a real browser harness |
| Base library artifact (`kali build --lib`) | library-build integration tests, artifact/schema assertions, `E5511` negatives for unknown export surfaces, and deterministic rebuild checks |
| Package compatibility | curated package-corpus results recorded per shipped source-graph command/context and per claimed rung of the shared package-support ladder |
| Install workflow / opt-in npm lifecycle hooks | install-command integration tests for manifest/lock/materialization updates, hook gating, and invalid raw-URL / JSR combinations |
| Registry-analysis commands (`package-effects`, `package-audit`) | command-shape negatives, deterministic single-package version-selection tests, context-participation tests, and JSON-contract assertions |
| Optimization/performance claims | version-pinned benchmark suite coverage, reproducible benchmark harness runs, and like-for-like build-mode comparisons against the claimed baseline; the canonical benchmark lane should include adapted Computer Language Benchmarks Game workloads derived from the Node.js / JavaScript submissions and normalized to Kali's TS/JS pipeline |
| CLI behavior / JSON schemas | golden CLI snapshots, schema validation, exit-code assertions, and the `kali test --coverage` payload contract |
| Artifact reproducibility | repeated-build tests over pinned inputs plus stable artifact-byte and metadata assertions |
| Proof-backed claims | passing Lean proof jobs for the currently published proof boundary |

## Bootstrap-evidence normalization

The bootstrap brief names a few evidence asks that are easy to lose once the spec set is split across phases. Read them through this one normalized checklist:

| Bootstrap ask | Normalized evidence lane |
|---|---|
| Comprehensive test suite inspired by upstream `tsc` | parser/integration coverage plus `test262` and `tsc`-style checker baselines |
| Fast compiler + optional advanced optimizations | build-mode comparisons and reproducible benchmark runs, not vague throughput claims |
| Benchmark against Rust / Benchmarks Game style workloads | the canonical optimization/performance lane includes adapted Computer Language Benchmarks Game workloads derived from Node.js / JavaScript submissions |
| Real-package validation such as `semver` and `@mariozechner/pi-coding-agent` | package-corpus fixtures with phase-correct expected rung/outcome, plus explicit npm-binary entrypoint probes where the bootstrap brief asks for runnable binaries; do not turn those probes into blanket executable-package promises |

This section is intentionally a reading aid: it does not create new support claims by itself. Availability still comes from [`specs/19-feature-maturity.md`](./19-feature-maturity.md).

Interpretation rules:
- grammar coverage and execution-semantic support are separate claims
- package-corpus evidence for ordinary source-graph commands is separate from evidence for later registry-analysis commands
- proof evidence strengthens confidence only for the published proof boundary; it does not replace command/profile-specific implementation tests

## Test families

### Unit tests
Each implementation subsystem should have focused unit coverage for its own invariants, including at least:
- lexer tokenization and recovery
- parser / AST construction
- type inference and checking
- IR transformation correctness
- codegen validation
- sandbox policy parsing and enforcement helpers

### Integration tests
End-to-end coverage should include:
- source → compile → execute → expected output
- source → compile/check → expected diagnostics
- `.js` source across representative inference tiers
- source + policy → `run/test --sandbox` runtime enforcement
- source graph + policy → the **Phase-1 static policy-validation surface**
- install workflow: `kali install`, `kali install <pkg>`, `kali install --dev <pkg>`, and opt-in `kali install --allow-scripts ...`
- test discovery / explicit-file selection / `--filter` / `--coverage`
- `kali build --lib` for fixtures with a **statically known export surface**
- browser-targeted `check` / `build --bundle`
- repeated builds of identical pinned inputs for determinism

### Snapshot tests
Snapshot tests are appropriate for:
- HIR
- MIR once MIR is the canonical ownership/layout IR
- generated WASM text or other stable internal representations

Snapshots must stay deterministic and reviewable.

### Fuzzing
Fuzz the lexer, parser, checker, and codegen. The minimum invariant is: **the compiler must not panic on arbitrary input**.

### Conformance suites
- Use test262 for ECMAScript conformance.
- Use `tsc`-style baseline fixtures for typing and inference behavior.
- Keep parser-breadth tracking separate from execution-semantic support claims.

### Package corpus
Maintain a curated package corpus that records expected outcomes per shipped command/context and per claimed support rung. Keep excluded native/binary/bootstrap-heavy packages in a separate negative track so installer-hook evidence is not misreported as general compatibility.

Bootstrap-normalization note:
- representative real-package probes from the bootstrap brief belong here as evidence fixtures, not as blanket support promises
- at minimum, keep a pure-JS utility probe such as `semver` and a broader host-heavy probe such as `@mariozechner/pi-coding-agent` in the corpus with phase-correct expected outcomes
- when the bootstrap brief names runnable npm binaries, record those as **binary-entrypoint probes** in the same corpus lane: the fixture should name the package, the invoked published bin entrypoint, the command/context under test, and the exact claimed support rung/outcome
- `semver` is the canonical early positive probe for the pure-JS/TS package contract and should be asserted at the exact claimed rung/context (for example installable/materializable first, then checkable/buildable, and only later executable where the host/API fit is actually satisfied)
- `@mariozechner/pi-coding-agent` is the canonical breadth/negative probe and should stay explicitly phase-correct rather than being treated as an implied Phase-1 executable-package promise; until the required Node/browser/runtime maturity rows open, the corpus should record the expected rejection or narrower rung honestly
- those probes should assert the exact rung/context being claimed at the time: for example, a package may be installable/materializable before it is executable, or a bin may be checkable/buildable as package content before its published CLI is executable in the selected host/API surface
- simplification rule: keep the corpus table phrased in support-ladder terms (`installable/materializable`, `checkable`, `buildable`, `executable`, `deployable-through-host`, or explicit rejection) so package evidence does not silently overclaim broader compatibility than the maturity matrix allows

Minimum bootstrap probe table:

| Bootstrap-named probe | Why it exists | Minimum evidence shape |
|---|---|---|
| `semver` | canonical early positive pure-JS package probe | assert the exact rung currently claimed (`installable/materializable`, then `checkable` / `buildable`, and only later `executable` when the host/API fit is truly supported) |
| `@mariozechner/pi-coding-agent` package content | canonical breadth probe for a host-heavier real package | assert the honest current rung or rejection reason instead of treating it as an implied Phase-1 success |
| `@mariozechner/pi-coding-agent` published bin entrypoint | canonical npm-binary-entrypoint probe from the bootstrap brief | record the exact bin name, invoked command/context, and expected outcome; this stays a separate probe from package install/materialization so the corpus does not blur “package present” into “published CLI executable” |

This table is intentionally minimal: it keeps the bootstrap brief's named package expectations visible without turning them into broader compatibility promises than the maturity matrix and package-support ladder justify.

## Determinism requirements

All machine-facing outputs used in support claims must be deterministic for pinned inputs, including:
- CLI JSON outputs
- build artifacts
- artifact metadata
- lockfiles
- report ordering

Equivalent dependency graphs should converge on byte-stable lockfile and artifact output rather than fetch order or hash-map iteration order.

## Browser-targeted evidence lane

Because Phase 1 explicitly ships the **Phase-1 browser-targeted command set**, it needs a dedicated evidence lane:
- browser-targeted type-check fixtures exercising browser ambient typings
- real-browser smoke tests for emitted bundles
- negative tests for unsupported standalone browser commands (`run --api browser`, `test --api browser`)

Mock-only DOM tests are not enough to justify browser-runtime support wording.

## Base-library artifact evidence lane

Because Phase 1 explicitly ships `kali build --lib` for **exact-version consumers** when the export surface is statically known, it needs a dedicated evidence lane:
- positive library-build fixtures
- deterministic artifact assertions
- negative `E5511` cases for inputs without a statically known export surface
- any host-consumption smoke test in this lane must be described as an **exact-version consumer** test, not as cross-version public ABI evidence

## Proof claim discipline

Proof-related testing follows the shared **proof-ready vs proof-backed** split:
- proof-ready is a repository/process baseline
- proof-backed claims require a non-empty published boundary in [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md)
- the current proof claim is always read from that manifest, not from duplicated prose here
- the canonical short summary is: **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**
- theorem/property inventory, covered paths, and trusted assumptions stay owned by [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md); this chapter intentionally does not mirror that list

If a release/support claim changes, update:
- this chapter,
- [17 — Formal Verification](./17-verification.md),
- [19 — Feature Maturity](./19-feature-maturity.md),
- [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), and
- any affected summaries such as [`README.md`](../README.md)

## Practical implementation note

Concrete CI layout, directory structure, benchmark automation, and staged evidence expansion belong to the implementation plan, primarily:
- [`PLAN.md`](../PLAN.md)
- [`plan/phase-1/14-evidence-hardening.md`](../plan/phase-1/14-evidence-hardening.md)
- later phase plan files when new evidence lanes open
