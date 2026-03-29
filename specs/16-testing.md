# 16 — Testing

## Test Strategy

### Unit Tests
Each crate has its own unit tests (Rust `#[cfg(test)]` modules):
- `kali_lexer`: Token output for input strings
- `kali_parser`: AST output for input programs
- `kali_types`: Type inference and checking results
- `kali_hir/mir/lir`: IR correctness after transformations
- `kali_codegen`: WASM binary output validation
- `kali_sandbox`: Effect analysis correctness

### Integration Tests
End-to-end tests in `tests/`:
- Source file → compile → execute → check output
- Source file → compile → check errors
- Source file → effects analysis → check JSON output *(Phase 2 target; earlier phases should assert that the command is unavailable or explicitly experimental)*
- Source file + policy → sandbox validation → check result *(Phase 1: runtime enforcement + policy-file validation, Phase 2+: inferred-effect-vs-policy validation too)*
- Browser-targeted source → `kali check --api browser` → expected diagnostics/type success
- Browser-targeted source → `kali build --bundle --api browser` → emitted artifact + smoke execution in a real browser harness

### Conformance Test Suites

The conformance strategy is intentionally split by concern area so language support, typing, packages, and sandbox behavior can advance at different rates without muddying pass/fail claims.

### Canonical Evidence Matrix for Maturity Claims

To keep phase labels and compatibility claims honest, each concern area needs its own evidence track before the project can call that area “supported” in a given command/profile/surface:

| Concern area | Minimum evidence before claiming support |
|---|---|
| Language syntax/semantics | parser tests + integration coverage + the applicable test262/conformance subset |
| Type checking / inference | checker baselines + inference golden tests + targeted regression cases |
| Host APIs / runtime behavior | integration tests that execute the API path + sandbox/resource-limit tests where relevant |
| Browser-targeted analysis/build support | browser-targeted check/build tests + emitted-bundle smoke runs in a real browser harness |
| Package compatibility | curated package corpus results recorded per command/profile (`check`, `build`, `test`, `run`) |
| CLI behavior / JSON schemas | golden CLI snapshots + schema validation tests + exit-code assertions |
| Proof-backed claims | passing Lean proof jobs for the currently modeled subset |

Interpretation rule:
- a feature can stay listed as a future phase target before these tests exist
- but the spec set should only describe the feature as **supported** once the matching evidence track is in place and runs in CI
- one passing demo or anecdotal package success is useful for exploration, but it is not enough to upgrade the canonical maturity wording

#### ECMAScript (test262)
Run against the [test262](https://github.com/tc39/test262) conformance suite:
- Track pass/fail/skip counts
- Known failures documented and triaged
- CI blocks regressions within the currently supported feature set
- Conformance targets are phased:
  - Phase 1: parser/runtime smoke coverage on a curated subset
  - Phase 2-3: expanding automated coverage with feature-based gating
  - Later compatibility goal: >95% pass rate for supported non-annex-B tests

#### TypeScript (`tsc`-style baselines)
Inspired by TypeScript's test suite:
- **Type check tests**: `.ts` file + expected diagnostics
- **Inference tests**: Check inferred types match expectations
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
- record whether each package is expected to `check`, `build`, `test`, or `run` under each supported profile
- treat package suites as phase-scoped contracts: Phase 1 corpus targets pure JS/TS packages that fit the linked-artifact model; later corpora can add harder Node/browser packages
- failures should distinguish resolution/type-check/runtime/sandbox causes so roadmap gaps are visible

#### Browser-Targeted Evidence Track
Because Phase 1 already promises `check --api browser` and `build --bundle --api browser`, those paths need an explicit evidence lane instead of being treated as a side effect of standalone runtime tests:
- run browser-targeted type-check fixtures that exercise DOM/browser ambient typings without implying standalone DOM runtime support
- run bundle smoke tests in at least one real browser automation harness so emitted JS glue + WASM bootstrap are tested together
- include negative tests that confirm unsupported standalone browser commands (`run --api browser`, `test --api browser`) still fail with the canonical gating diagnostic
- keep this track separate from any lightweight DOM/unit-test shim so Kali does not accidentally overclaim browser-runtime support from mock-only tests

#### Kali-Specific Tests
- **Effect inference tests**: Source → expected effects JSON for the full statically reachable graph from the chosen entrypoint *(Phase 2 target; Phase 1 may instead test internal analysis units without a stable CLI surface)*
- **Sandbox tests**: Source + policy → expected pass/fail, including explicit checks for Phase 1 runtime enforcement vs Phase 2 compile-time effect-policy rejection, and for policy checks over transitive imports/dependencies rather than just the root file
- **Memory tests**: Source → expected allocation strategy (stack/owned-heap/shared-heap)
- **Specialization tests**: Generic source → expected number of specializations
- **Optimization tests**: Source → check specific optimization was applied
- **AI-facing diagnostics tests**: ensure concise human output and stable JSON diagnostics remain aligned

### Snapshot Tests
For IR representations:
- Source → HIR snapshot
- Source → MIR snapshot (including memory layout decisions) *(Phase 2+, once MIR is the canonical ownership/layout IR)*
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

Benchmark suite run on CI, regressions detected automatically.

## Test Infrastructure

### Test Runner
```bash
kali test                                   # Run all test files
kali test --filter "type"                   # Filter by name
kali test --coverage                        # Phase 2 target: with coverage report once the stable contract lands
```

Kali's own test runner for discovered test files, supporting:
- default discovery starts from the canonical project-discovery result from [SPEC.md](../SPEC.md), then matches `*.test.*` / `*_test.*` across the executable/analyzable source set (`.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs`, `.cjs`)
- declaration-only files (`.d.ts`, `.d.mts`, `.d.cts`) are excluded from test discovery even if they match the naming pattern
- explicit file arguments to `kali test` must also belong to the executable/analyzable source set; passing a declaration-only file is an invalid-entrypoint error rather than a silent skip
- coverage reporting is a **Phase 2 target** so Phase 1 may reject `--coverage` or mark it experimental until the report contract is stabilized
- `describe`, `it`, `test` blocks
- `expect` assertions
- `beforeEach`, `afterEach`, `beforeAll`, `afterAll`
- Async test support
- Sandbox-aware (tests can run in sandbox mode)

### CI Pipeline
```
1. cargo fmt --check
2. cargo clippy -- -D warnings
3. cargo test (unit tests)
4. Integration tests (compile + run)
5. test262 conformance
6. TypeScript test suite
7. Fuzz testing (time-limited)
8. Benchmarks (compare to baseline)
9. Conditional Lean proof verification for the currently modeled subset
```

Proof-job consistency rule:
- Lean verification is **not** an all-or-nothing claim that the whole language/runtime is already modeled.
- CI should run the proof job whenever changes touch the proof tree itself or a Rust/spec subsystem that the current Lean model claims to cover (for example the modeled type, effect, memory, or sandbox core).
- Changes outside that modeled subset do not need to block on unrelated proof jobs, but they still must not weaken the documented proof boundary accidentally.

### Test Data
- `tests/fixtures/` — source files for integration tests
- `tests/snapshots/` — IR/output snapshots
- `tests/conformance/` — test262 and tsc-derived tests
- `tests/sandbox/` — sandbox policy + program pairs
- `tests/effects/` — effect inference test cases
- `tests/memory/` — ownership and allocation decision test cases
- `proofs/` — Lean models and proofs for the currently verified core subset

### Runtime Execution Tests
In addition to compiler tests, run compiled WASM programs and verify:
- stdout/stderr output matches expectations
- Exit codes are correct
- Resource limits are enforced (sandbox tests)
- Async operations complete correctly
- API compatibility with the documented Phase 1 Web + Deno baseline
- browser-targeted bundle smoke tests execute through the real browser host + generated glue path rather than only through mocked DOM/unit harnesses
- phase-gated features produce the canonical `E5006` diagnostic instead of silent fallback
