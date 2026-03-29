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
- Source file → effects analysis → check JSON output
- Source file + policy → sandbox validation → check result

### Conformance Test Suites

#### ECMAScript (test262)
Run against the [test262](https://github.com/nicelhc13/test262) conformance suite:
- Track pass/fail/skip counts
- Known failures documented and triaged
- CI blocks regressions (new failures)
- Target: >95% pass rate for non-annex-B tests

#### TypeScript (tsc tests)
Inspired by TypeScript's test suite:
- **Type check tests**: `.ts` file + expected diagnostics
- **Inference tests**: Check inferred types match expectations
- **Emit tests**: Check compiled output for specific patterns

Format:
```typescript
// @filename: test.ts
// @errors: E1001

let x: number = "hello";
//              ~~~~~~~ E1001: Type 'string' is not assignable to type 'number'
```

#### Kali-Specific Tests
- **Effect inference tests**: Source → expected effects JSON
- **Sandbox tests**: Source + policy → expected pass/fail
- **Memory tests**: Source → expected allocation strategy (stack/heap/Rc)
- **Specialization tests**: Generic source → expected number of specializations
- **Optimization tests**: Source → check specific optimization was applied

### Snapshot Tests
For IR representations:
- Source → HIR snapshot
- Source → MIR snapshot (including memory layout decisions)
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
kali test --coverage                        # With coverage report
```

Kali's own test runner for `.test.ts` / `_test.ts` files, supporting:
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
9. Lean proof verification
```

### Test Data
- `tests/fixtures/` — source files for integration tests
- `tests/snapshots/` — IR/output snapshots
- `tests/conformance/` — test262 and tsc-derived tests
- `tests/sandbox/` — sandbox policy + program pairs
- `tests/effects/` — effect inference test cases
- `tests/memory/` — ownership and allocation decision test cases

### Runtime Execution Tests
In addition to compiler tests, run compiled WASM programs and verify:
- stdout/stderr output matches expectations
- Exit codes are correct
- Resource limits are enforced (sandbox tests)
- Async operations complete correctly
- API compatibility with Deno/Node behavior
