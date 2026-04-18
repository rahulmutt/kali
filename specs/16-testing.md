# 16 — Testing

Current repository-state note:
- this repository is still spec-first; the crate names, test directories, and CI lanes below define the target implementation/testing contract, not a claim that every Rust crate, fixture tree, or hosted CI job already exists today
- current repo obligations are therefore narrower: keep the spec/docs internally consistent, keep phase-gated workflows honestly marked as unavailable until their maturity rows open, and follow the shared **proof-ready vs proof-backed split** from [SPEC.md](../SPEC.md) plus the published proof-boundary policy in `proofs/BOUNDARY.md`
- when this chapter needs a one-line statement about the repository's current verification posture, reuse the manifest's canonical short summary verbatim: **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**
- the current published boundary also keeps the RC snapshot helper slice explicit, including the pure release helper's ownership/allocation, heap-characterisation, origin/ownership, and disjointness corollaries (`releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefHeapCharacterisation`, `releaseRefHeapCellOriginAndOwnership`, `releaseRefReleasedNotLiveRef`) alongside the existing release-recording, zero-count collection (`KaliCore.Safety.releaseAndCollectDropsZeroCountCells`), zero-count removal (`KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`), positive-count preservation and post-collection positivity (`KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells`), release-and-decrement positive-count preservation (`KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementKeepsOriginalPositiveCountCells`), release-and-decrement target-cell positive-count preservation (`KaliCore.Safety.releaseAndDecrementKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`), release-and-decrement provenance-and-ownership (`KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, the release-and-decrement origin-and-positive-count theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, the release-and-decrement origin/ownership/positivity theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`), heap-characterisation theorems (`KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation`), target-cell retention (`KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`), exact released-reference cons-shape via `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons`, heap-characterisation (`KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`), bundled ownership-and-origin (`KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`), bundled origin-plus-positive-count (`KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount`), final-heap positive-count (`KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`), and helper-level no-dangling-reference corollaries `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`, while still stopping short of the fuller ownership/freeing story

## Test Strategy

### Unit Tests
Each crate has its own unit tests (Rust `#[cfg(test)]` modules):
- `kali_lexer`: Token output for input strings
- `kali_parser`: AST output for input programs
- `kali_types`: Type inference and checking results
- `kali_hir/mir/lir`: IR correctness after transformations
- `kali_codegen`: WASM binary output validation
- `kali_sandbox`: Policy parsing/validation, runtime-enforcement helpers, and internal effect-analysis correctness without implying the Phase 1 CLI already exposes stable effect-report commands

### Integration Tests
End-to-end tests in `tests/`:
- Source file → compile → execute → check output
- Source file → compile → check errors
- Plain `.js` source → check/build/run across representative inference tiers → confirm the shared **first-class JavaScript compilation** contract: precise local inference when cheap, conservative `unknown`/union/dynamic fallbacks when needed, and no silent invention of fresh `any`
- Source file → effects analysis → check effect-report JSON output and the one-root explicit-input contract for `kali effects <file>` *(Phase 2 target; this belongs to the shared **public effect-report surface** from [SPEC.md](../SPEC.md), so earlier phases should assert that the command is unavailable even if internal effect bookkeeping tests already exist)*
- Source file + policy → `kali run --sandbox` / `kali test --sandbox` → check runtime enforcement result *(Phase 1 MVP runtime-enforcement owner)*
- Source graph + policy → the shared **Phase-1 static policy-validation surface** → check static policy-schema/config validation result *(Phase 1 MVP static-policy-validation owner; Phase 2 target adds inferred effect-vs-policy rejection on those same command paths rather than a second dry-run workflow)*
- Install workflow → `kali install`, `kali install <pkg>`, `kali install --dev <pkg>`, and opt-in `kali install --allow-scripts ...` → deterministic manifest/lock/materialization behavior, correct lifecycle-hook gating, clean/no-op rejection when **effective npm-scriptable install work** is empty, and explicit rejection for raw-URL / JSR combinations that do not participate in npm lifecycle hooks
- Test source set → `kali test [files...]` / `kali test --filter ...` → correct discovery-vs-explicit-file selection behavior, stable post-selection filtering, and expected invalid-entrypoint rejection for declaration-only test inputs
- Library source with a **statically known export surface** → `kali build --lib` → export-oriented **base library artifact** + deterministic artifact metadata for **exact-version consumers** *(Phase 1 MVP for the base library artifact; the stable public embedding surface remains a Phase 2 target)*
- Browser-targeted source → the shared **Phase-1 browser-targeted command set** → expected diagnostics/type success for `check` and emitted artifact + smoke execution in a real browser harness for `build --bundle`, including equivalent inherited-config forms and supported `--sandbox` variants where applicable
- Repeated build of the same pinned input/context → byte-stable artifacts and stable machine-readable metadata by default

### Phase-Correct Testing Rule
To keep tests from accidentally widening support claims, the repository should treat each workflow family according to its phase owner:
- **Phase 1-shipped workflows** must have positive integration coverage for their supported command/context combinations.
- **Later documented workflows** may already have schemas, CLI spellings, fixtures, or internal plumbing, but CI should assert unavailability/gating until their maturity rows open.
- **Internal-only machinery** (for example Phase-1 effect bookkeeping) should be tested through unit/integration helpers without being mislabeled as the stable public CLI/API surface.

Practical shortcut:
- `run` / `test` sandbox enforcement, the shared **Phase-1 static policy-validation surface**, and the **Phase-1 browser-targeted command set** need positive Phase-1 coverage.
- later command families and later proof claims stay negative/gated until their owning phase opens: `kali effects`, `kali package-effects`, `kali package-audit`, inferred-effect-vs-policy rejection on `check/build --sandbox`, stable public embedding flows (`--capi`, `--component`, stable public `--lib` + WIT), and proof-backed release claims.

Support-claim maintenance rule:
- if a test/evidence change would justify promoting or narrowing a public support claim, update the owning chapter, [`specs/19-feature-maturity.md`](./19-feature-maturity.md), and any affected root-level summaries such as [`README.md`](../README.md) in the same change.
- negative/gating tests are part of the support contract too: removing them without opening the matching maturity row is drift, not simplification.

### Conformance Test Suites

The conformance strategy is intentionally split by concern area so language support, typing, packages, and sandbox behavior can advance at different rates without muddying pass/fail claims.

### Canonical Evidence Matrix for Maturity Claims

To keep phase labels and compatibility claims honest, each concern area needs its own evidence track before the project can call that area “supported” in a given command/profile/surface:

| Concern area | Minimum evidence before claiming support |
|---|---|
| Language syntax/semantics | parser tests + integration coverage + the applicable test262/conformance subset |
| Latest-ECMA grammar claim | parser fixtures for the current edition + tracked unsupported-semantics list where relevant |
| Runtime semantic support claim | command/profile-specific integration tests + the applicable conformance subset for the claimed feature family |
| Type checking / inference | checker baselines + inference golden tests + targeted regression cases |
| First-class JavaScript compilation | dedicated `.js` fixtures across `check` / `build` / `run`, JSDoc-hint coverage, and golden cases for the canonical fallback ladder (`precise` → `small union` → `unknown` / dynamic layout) so `.js` support does not regress into parse-only compatibility or implicit-`any` drift |
| Host APIs / runtime behavior | integration tests that execute the API path + sandbox/resource-limit tests where relevant |
| The **Phase-1 browser-targeted command set** | browser-targeted `check` tests + browser-targeted `build --bundle` tests + emitted-bundle smoke runs in a real browser harness |
| Base library/export artifact support (`kali build --lib`) | library-build integration tests + artifact-manifest/schema assertions + deterministic rebuild checks for fixtures with a **statically known export surface**, all scoped to the Phase-1 **base library artifact** consumption story for **exact-version consumers** rather than the later stable public embedding surface |
| Package compatibility | curated package corpus results recorded per shipped source-graph command/context **and per claimed rung of the shared package-support ladder** from [SPEC.md](../SPEC.md) (for example standalone `check` / `build` / `run` / `test`, plus browser-targeted `check` / `build --bundle` when those package claims are made) |
| Install workflow / opt-in npm lifecycle hooks | install-command integration tests for manifest/lock/materialization updates, explicit npm-target hook execution behind `kali install --allow-scripts`, clean/no-op rejection when **effective npm-scriptable install work** is empty, and invalid-combination coverage for raw-URL / JSR targets |
| Registry-analysis commands (`package-effects`, `package-audit`) | command-shape/arity negatives, deterministic single-package version-selection tests, context-participation tests (`package-effects` inherited analysis context vs `package-audit` context-free behavior), and JSON-contract assertions for native-JSON vs envelope-only output |
| CLI behavior / JSON schemas | golden CLI snapshots + schema validation tests + exit-code assertions |
| Artifact reproducibility | repeated-build tests over pinned inputs/toolchains + normalized artifact-byte comparisons + stable emitted-metadata assertions |
| Proof-backed claims | passing Lean proof jobs for the current published proof boundary, scoped by the published **proof-boundary manifest** and the shared **proof-ready vs proof-backed split**; Phase 1 may be merely **proof-ready** earlier, but proof-backed release claims require a non-empty published boundary with named theorem/property claims, including the current RC snapshot helper slice's `KaliCore.Safety.releaseAndCollectDropsZeroCountCells`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementKeepsOriginalPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisation`, `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, the release-and-decrement origin-and-positive-count theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, the release-and-decrement origin/ownership/positivity theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`, `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, `KaliCore.Safety.releaseAndCollectReleasedRefsCons`, `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`, `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, and the helper-level no-dangling-reference corollaries `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`, plus the surrounding release-helper ownership/allocation and disjointness corollaries |

Interpretation rule:
- a feature can stay listed as a future phase target before these tests exist
- but the spec set should only describe the feature as **supported** once the matching evidence track is in place and runs in CI
- grammar-coverage claims and semantic-support claims are intentionally separate: accepting syntax from the latest ECMA-262 edition is not, by itself, evidence that every such construct already executes in every Kali mode
- one passing demo or anecdotal package success is useful for exploration, but it is not enough to upgrade the canonical maturity wording
- evidence for later single-package registry-analysis commands is separate from package-corpus evidence for ordinary project/source-graph command support; a good `package-effects` or `package-audit` test lane does **not** by itself prove that the same package is runnable in Kali-hosted `run` / `test`

#### ECMAScript (test262)
Run against the [test262](https://github.com/tc39/test262) conformance suite:
- Track pass/fail/skip counts
- Known failures documented and triaged
- CI blocks regressions within the currently supported feature set
- Keep parser-breadth tracking separate from execution-semantic support claims so "latest ECMA-262 grammar" does not silently become "every current-edition semantic edge already works"
- Conformance targets are phased:
  - Phase 1: parser/runtime smoke coverage on a curated subset
  - Phase 2-3: expanding automated coverage with feature-based gating
  - Later compatibility: >95% pass rate for supported non-Annex-B tests

#### TypeScript/JavaScript (`tsc`-style baselines)
Inspired by TypeScript's test suite:
- **Type check tests**: source file from the shared executable/analyzable source-file class + expected diagnostics
- **JavaScript-first inference tests**: `.js` fixtures that exercise local inference, exported-boundary conservatism, JSDoc hints, and the fallback ladder from [04 — Type System](04-type-system.md)
- **Inference tests**: Check inferred types match expectations across both annotated TypeScript and first-class JavaScript inputs
- **Emit tests**: Check compiled output for specific patterns
- **Baseline stability**: diagnostics and machine-readable outputs should use stable golden files where possible so spec and implementation drift are easy to spot

Format:
```typescript
// @filename: test.ts
// @errors: E1001

let x: number = "hello";
//              ~~~~~~~ E1001: Type 'string' is not assignable to type 'number'
```

#### Package Compatibility Suites
Because Kali aims to support real npm/JS ecosystems, package compatibility needs its own evidence track rather than anecdotal one-off testing:
- maintain a curated corpus of representative packages (validators, parsers, utility libraries, browser-targeted libs, selected Node-host-heavy packages once Phase 3 begins)
- record whether each package is expected to work for each shipped **source-graph command/context** claim and for each claimed rung of the shared **package-support ladder** from [SPEC.md](../SPEC.md), rather than only by broad package label — for example standalone `check` / `build` / `run` / `test`, plus browser-targeted `check` / `build --bundle` when those package claims are made
- keep this corpus aligned with the shared package-workflow split from [14 — Package Management](14-packages.md): Phase-1 package compatibility evidence is primarily about ordinary source-graph commands, not about the later registry-analysis commands `package-effects` / `package-audit`
- treat package suites as phase-scoped contracts: Phase 1 corpus targets packages inside the shared **pure JS/TS package contract** that fit the linked-artifact model; later corpora can add harder Node/browser packages
- keep packages in the excluded **native/binary/bootstrap-heavy package contract** in a clearly separate exclusion/negative track so `--allow-scripts` evidence does not get misreported as general support for that contract
- failures should distinguish resolution/type-check/runtime/sandbox causes so roadmap gaps are visible

#### Browser-Targeted Evidence Track
Because Phase 1 already promises the shared **Phase-1 browser-targeted command set**, those paths need an explicit evidence lane instead of being treated as a side effect of standalone runtime tests:
- run browser-targeted type-check fixtures that exercise DOM/browser ambient typings without implying standalone DOM runtime support
- run bundle smoke tests in at least one real browser automation harness so emitted JS glue + WASM bootstrap are tested together
- include negative tests that confirm unsupported standalone browser commands (`run --api browser`, `test --api browser`) still fail with the canonical gating diagnostic
- keep this track separate from any lightweight DOM/unit-test shim so Kali does not accidentally overclaim browser-runtime support from mock-only tests

#### Base-Library Artifact Evidence Track
Because Phase 1 already promises the export-oriented base `kali build --lib` mode **when Kali can determine a statically known export surface**, that artifact needs its own explicit evidence lane too.

Preferred reading:
- this lane proves that plain `kali build --lib` is **buildable for exact-version consumers** in Phase 1 on fixtures where Kali can determine the required **statically known export surface**
- it does **not** prove the later stable public embedding/WIT/C-ABI/Component-Model surface

Required checks:
- run library-build fixtures that verify the expected exported-library artifact shape without implying the later stable public embedding/WIT contract yet
- treat any host-consumption smoke coverage for this lane as an **exact-version consumer** test only: pin the producing Kali toolchain/runtime version and do not present those fixtures as evidence of cross-version/public ABI stability
- include negative tests for library inputs that do **not** have a statically known export surface so the build fails with `E5011` instead of synthesizing reflective exports
- assert deterministic artifact metadata/output ordering across repeated `--lib` builds of the same pinned input
- keep this lane separate from the Phase 2 stable embedding/C ABI/Component Model evidence so Phase 1 does not accidentally overclaim public ABI stability

#### Init-Scaffold Evidence Track
Because schema v1 now treats the built-in `kali init` templates as exact minimal scaffolds rather than vague starter layouts, that contract needs direct tests too:
- assert that `kali init` creates exactly `kali.json` + `main.ts` by default, and `kali init --lib` creates exactly `kali.json` + `lib.ts`
- assert that neither scaffold writes `kali.lock`, `node_modules/`, `.kali/cache/`, `src/`, or `test/` by default
- include negative tests for `kali init` in a directory that already contains `kali.json` so the command fails with `E5008` instead of partially overwriting an existing project root
- keep this lane separate from later richer template work so Phase 1 does not accidentally drift from the shared **minimal canonical scaffold contract**

#### Kali-Specific Tests
- **Effect inference tests**: Source → expected effects JSON for the command's **resolved source graph** from [SPEC.md](../SPEC.md), including rejection cases for omitted roots or accidental batch/project-discovery shortcuts *(Phase 2 target; under the shared **effect-surface split**, Phase 1 may instead test internal effect-bookkeeping units without claiming the stable CLI/JSON surface)*
- **JSON-mode coverage for registry/effect reporting commands**: once `kali effects` / `kali package-effects` exist, assert both the native bare-payload mode and the `--output json` envelope mode; once `kali package-audit` exists, assert its envelope-only `--output json` behavior, including canonical `payload: null`, and the invalid `--pretty`-without-`--output json` path so the CLI/output-model split from [SPEC.md](../SPEC.md) and [specs/18-schemas.md](18-schemas.md) cannot drift
- **Sandbox tests**: Source + policy → expected pass/fail, including explicit checks for Phase 1 runtime enforcement vs Phase 2 compile/check-time effect-vs-policy rejection, and for the rule that policy checks cover the command's **resolved source graph** from [SPEC.md](../SPEC.md), not just the root file
- **Memory tests**: Source → expected allocation strategy (stack/owned-heap/shared-heap)
- **Specialization tests**: Generic source → expected number of specializations
- **Optimization tests**: Source → check specific optimization was applied
- **AI-facing diagnostics tests**: ensure concise human output and stable JSON diagnostics remain aligned

### Snapshot Tests
For IR representations:
- Source → HIR snapshot
- Source → MIR snapshot (including memory layout decisions) *(from the Phase 2 target onward, once MIR is the canonical ownership/layout IR)*
- Source → WASM text format snapshot
- Snapshots reviewed on change, committed to repo

### Fuzz Testing
- Fuzz the lexer with arbitrary byte sequences
- Fuzz the parser with grammar-aware fuzzing (using `cargo-fuzz` / `libfuzzer`)
- Fuzz the type checker with randomly generated type programs
- Fuzz the WASM codegen (verify output passes `wasm-validate`)
- Property: compiler never panics on any input

### Performance Benchmarks
Track compile-time performance:
- Lexing throughput (MB/s)
- Parsing throughput (MB/s)
- Type checking time for large programs
- WASM codegen time
- End-to-end compile time for real-world projects

Implementation-phase rule:
- once benchmark infrastructure exists, the benchmark suite should run in CI (or another documented automated regression lane) and flag statistically meaningful regressions automatically
- until then, this section remains a target evidence lane rather than a claim that the current spec-first repository already has hosted benchmark automation

## Test Infrastructure

### Test Runner
```bash
kali test                                   # Run all test files
kali test --filter "type"                   # Filter by name
kali test --coverage                        # Phase 2 target: with coverage report once the stable contract lands
```

Kali's own test runner for discovered test files, supporting:
- default discovery starts from the canonical project-discovery result from [SPEC.md](../SPEC.md), then matches `*.test.*` / `*_test.*` across the shared **executable/analyzable source-file class**
- declaration-only files are excluded from test discovery even if they match the naming pattern
- explicit file arguments to `kali test` bypass the naming-pattern discovery filter and are treated as one explicit test-module set, but they must still belong to that same shared source-file class; passing a declaration-only file is the canonical invalid-entrypoint error (`E5007`) rather than a silent skip
- `--filter` should be tested as a post-selection narrowing step over both discovered tests and explicit test-module sets so it cannot drift into a second discovery mode
- coverage reporting is a **Phase 2 target** so Phase 1 should reject `--coverage` until the report contract is stabilized
- `describe`, `it`, `test` blocks
- `expect` assertions
- `beforeEach`, `afterEach`, `beforeAll`, `afterAll`
- Async test support
- Sandbox-aware (tests can run in sandbox mode)

### CI Pipeline
Target implementation-phase CI pipeline:
```
1. cargo fmt --check
2. cargo clippy -- -D warnings
3. cargo test (unit tests)
4. Integration tests (compile + run)
5. test262 conformance
6. TypeScript test suite
7. Fuzz testing (time-limited)
8. Benchmarks (compare to baseline)
9. Lean proof verification when required by the `proofs/BOUNDARY.md` proof-CI trigger policy
```

Current spec-first repo baseline:
- until the Rust implementation/test tree exists, the practical CI minimum is spec/docs consistency plus the proof-boundary-policy checks described in `proofs/BOUNDARY.md`
- the repository's proof-ready baseline should already exist before that full CI pipeline does; the proof job simply runs only when the published proof boundary says it must
- once implementation crates, fixtures, and hosted automation land, this target pipeline becomes the expected default CI shape for supported surfaces

Proof-job consistency rule:
- Lean verification is **not** an all-or-nothing claim that the whole language/runtime is already modeled.
- CI should follow the published proof state in `proofs/BOUNDARY.md`: while the boundary is empty, proof CI is required for changes under `proofs/`; once the manifest names covered Rust/spec subsystems, the proof job also runs for changes to those covered areas.
- Changes outside that published boundary do not need to block on unrelated proof jobs, but they still must not weaken the documented proof boundary accidentally.
- A release should not treat a still-empty published proof boundary as evidence of shipped proof coverage; proof-backed release claims require a concrete published boundary with named theorem/property claims.

### Test Data
Current-state clarification:
- the directory names below are the **target test/proof layout** once implementation fixtures land; in the current spec-first repo they should be read as intended locations, not guaranteed present directories
- the one verification artifact that is required today is `proofs/BOUNDARY.md`

- `tests/fixtures/` — source files for integration tests
- `tests/snapshots/` — IR/output snapshots
- `tests/conformance/` — test262 and tsc-derived tests
- `tests/sandbox/` — sandbox policy + program pairs
- `tests/effects/` — effect-analysis cases; in Phase 1 these may target internal bookkeeping/helpers, while from the Phase 2 target onward they additionally cover the stable public effect-report surface
- `tests/memory/` — ownership and allocation decision test cases
- `proofs/` — today: the published proof-boundary manifest; later: Lean models and proofs for the currently verified core subset once the proof tree exists

### Runtime Execution Tests
In addition to compiler tests, run compiled WASM programs and verify:
- stdout/stderr output matches expectations
- Exit codes are correct
- Resource limits are enforced (sandbox tests)
- Async operations complete correctly
- API compatibility with the documented **Default standalone context (schema v1)**: the shared **Web baseline** plus the **Deno-oriented standalone surface**
- browser-targeted compatibility is evidenced separately through emitted-bundle smoke tests in a real browser host via the browser host adapter rather than by treating browser APIs as part of the standalone runtime baseline
- phase-gated features produce the canonical `E5006` diagnostic instead of silent fallback
