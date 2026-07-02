# Baseline Green + Closure-Return Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Take `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` from 138 kali_cli failures to fully green by fixing three real compiler bugs (closure-return escape, BigInt `/`, keyword property names), re-pinning stale test expectations, and rewriting E3200-stale fixtures — plus the `keys.length !== 2`→`4` fixture typo.

**Architecture:** The dominant fix (79 tests) routes const-bound arrow functions through the codegen lane that named functions and unnamed function expressions already ride correctly: `function_plan` in kali_codegen learns to recognize the arrow LIR shape (last child = `Branch("return")`) so arrows compile as standalone wasm functions, and the parser learns to parse block-bodied arrows in declarator-init position as unnamed `FunctionExpression`s. Call dispatch needs no work — `resolve_bound_member_callable_node` already follows const bindings to the function node. The remaining fixes are two small compiler patches, mechanical test-expectation flips, and fixture rewrites (all pre-validated against the built binary during planning).

**Tech Stack:** Rust workspace (kali_parser, kali_codegen, kali_cli integration tests), `wasm_encoder`, node for ground truth.

**Spec:** `docs/superpowers/specs/2026-07-02-baseline-green-and-closure-return-fix-design.md` (approved). Investigation: `/workspace/.superpowers/sdd/baseline-fix-investigation.md`.

## Global Constraints

- Branch: `fix/stale-browser-bundle-expectations` (already checked out). Commit on it. **Never `git push`.**
- Gate for every task: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` — **NEVER `--workspace`** (~900 pre-existing chromium-sandbox failures elsewhere are not yours).
- End-state gate: that command fully green (0 failures) — this project exists to make the baseline green.
- The 1696 currently-passing runtime_smoke tests and all currently-green test binaries must stay green. A changed expected output of any currently-PASSING test is a regression to investigate, never a re-baseline.
- fannkuch-redux / spectral-norm pinned benchmark outputs must not change.
- `cargo fmt --all` before every commit; conventional commit messages (`fix(codegen): …`, `test(cli): …`).
- No new dependencies.
- Every re-pinned expectation must be justified against node ground truth in a test comment or the commit message (style of commit b5c085401). Ground truth used in this plan: `node -e 'console.log(Math.sqrt(1.6))'` → `1.2649110640673518`; `node -e 'console.log((3n/2n).toString())'` → `1`; `((name) => name)(1n, 2n) === 1n`.
- OUT OF SCOPE (do not fix, do not "improve"): `ThrowStmt` codegen (a no-op today — many fixture self-checks are vacuous, leave them), block-bodied arrows outside declarator-init position (incl. the E3100 param-scoping defect and the `Kali.test('…', () => {…})` callback lane), BigInt `/` on non-literal operands.

## File Structure

| File | Responsibility |
|---|---|
| `crates/kali_codegen/src/lower.rs` (modify) | `function_plan`: accept arrow `Branch("return")` bodies |
| `crates/kali_codegen/src/emit/call.rs` (modify) | pad non-produced call arguments with `I64Const(0)` |
| `crates/kali_codegen/src/emit/operators.rs` (modify) | BigInt-literal `/` → `I64DivS`; `is_float_valued` mirror |
| `crates/kali_codegen/src/emit/operators_tests.rs` (modify) | unit tests for BigInt division lowering |
| `crates/kali_parser/src/statement.rs` (modify) | declarator-init hook for block-bodied arrows |
| `crates/kali_parser/src/declaration.rs` (modify) | `try_parse_block_arrow_function_expression` |
| `crates/kali_parser/src/declaration_tests.rs` (modify) | parser unit tests for block-arrow declarators |
| `crates/kali_parser/src/expression/call.rs` (modify) | keyword tokens as `.` property names |
| `crates/kali_parser/src/expression/call_tests/member.rs` (modify) | parser unit test for keyword properties |
| `crates/kali_cli/tests/closure_return_isolation.rs` (create) | integration regression tests for the closure fix |
| `crates/kali_cli/tests/runtime_smoke.rs` (modify) | typo helper fix; `assert_browser_bundle_executes_with_result` |
| `crates/kali_cli/tests/runtime_smoke/{build,check,run,test}.rs` (modify) | class-1 flips, class-3 fixture rewrites, class-4a typo fixtures, class-6 greet callers |
| `crates/kali_cli/tests/browser_math_unsupported_member_calls_harness_jsx_tsx.rs` (modify) | 3 sqrt harness tests flip to success |
| `crates/kali_cli/tests/schema_docs/misc.rs` (modify) | add fannkuch-redux / spectral-norm to slug lists |
| `crates/kali_cli/tests/fixtures/benchmarks/string-concatenation-benchmark-v1.{ts,json}` (modify) | literal-rooted rewrite + new sha256 |
| `crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1.{ts,json}` (modify) | literal-rooted rewrite + new sha256 |
| `crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1-js.{js,json}` (modify) | byte-identical js twin + new sha256 |

Notes for implementers with zero context:

- **Pipeline:** source → `kali_lexer` → `kali_parser` (AST in `kali_ast`) → `kali_types` (resolution/diagnostics, e.g. E3100/E3200/E5506) → `kali_hir` → `kali_mir` → `kali_lir` → `kali_codegen` (wasm via `wasm_encoder`). HIR/MIR/LIR are generic `{kind, text, children}` node trees; codegen re-derives structure from shapes, so many codegen helpers pattern-match exact child layouts — do not change LIR shapes, only how codegen interprets them.
- **How arrows reach codegen today:** an expression-bodied arrow `(x) => x + 1` lowers (HIR `lowering/function.rs:42-67`) to a `FunctionExpr` node = `[Ident("x"), ReturnStmt[x + 1]]` with a synthetic name `__kali_fn_N`; in LIR that is `Instruction(text = "__kali_fn_N", children = [Value("x"), Branch("return")])`. Named functions instead end in a `Block` child. `function_plan` (`crates/kali_codegen/src/lower.rs:729`) requires the `Block`, so arrows are NOT collected as functions, NOT skipped by `is_function_like` at their declaration site (`emit/control_flow.rs:350`), and their statements — including the synthesized real `Instruction::Return` — are emitted inline into the ENCLOSING function, truncating it. Verified: `console.log("A"); const f = (x) => x; console.log("B");` prints only `A`, exit 0.
- **Block-bodied arrows** (`() => { … }`) are worse: both arrow parse paths bail on a `{` body (`declaration.rs:254-257`, `expression/primary.rs:26-27`), the `{…}` becomes a stray top-level block that executes inline at the declaration point. This is also why `Kali.test('t', () => { … })` callbacks execute inline today — that call-argument lane is deliberately NOT touched by this plan (hundreds of passing tests pin its behavior).
- **Call dispatch is already solved:** `emit_call` resolves the callee through `resolve_bound_member_callable_node` (`emit/call.rs:2802`), which follows the per-function `bindings` map (`const` name → init LIR node). Once an arrow is in the `functions` map (name → wasm index) under its synthetic name, `f(...)` dispatches a real `Call`. Control verified on the built binary: `const f = function () { console.log("X"); return 7; }; console.log(f()); console.log("after");` prints `X`,`7`,`after`.
- **`throw` is a codegen no-op** (no `ThrowStmt` codegen exists), so fixture self-checks like `if (…) { throw … }` never abort — several tests only pin `console.log` output. Do not "fix" this; it is a recorded follow-up.
- **Running tests:** integration tests live in `crates/kali_cli/tests/`; `runtime_smoke.rs` is one test binary with `build/check/run/test/misc` modules. Filters: `cargo test -p kali_cli --test runtime_smoke -- <substring>`. The `kali` binary is rebuilt automatically (tests use `CARGO_BIN_EXE_kali`). `node` is installed for ground-truth probes. Line numbers in this plan are from main @ 81496c912 and will drift as you edit — always locate by function name / quoted text.
- Fixture strings inside tests are sometimes raw strings (`r#"…"#`) and sometimes escaped double-quoted strings — the JS content shown in this plan is authoritative; preserve the surrounding Rust quoting style of each site.

---

### Task 1: Closure-return isolation (classes 2, 4, 5 — fixes 79 of 138)

Const-bound arrows must compile as standalone wasm functions instead of leaking their `return` into the enclosing function. Three changes: `function_plan` accepts the arrow body shape; the parser handles block-bodied arrows in declarator-init position; the call-arg loop pads non-produced arguments.

**Files:**
- Modify: `crates/kali_codegen/src/lower.rs` (fn `function_plan`, currently lines 729-763)
- Modify: `crates/kali_codegen/src/emit/call.rs` (arg loop in `emit_call`, currently lines 2187-2199)
- Modify: `crates/kali_parser/src/statement.rs` (fn `parse_variable_declaration`, lines 80-106)
- Modify: `crates/kali_parser/src/declaration.rs` (new fn after `try_parse_arrow_function_expression_from`)
- Test: Create `crates/kali_cli/tests/closure_return_isolation.rs`; append to `crates/kali_parser/src/declaration_tests.rs`

**Interfaces:**
- Consumes: existing `FunctionPlan`, `is_function_like`, `resolve_bound_member_callable_node`, `parse_block_statement` (returns `Option<Statement>`), `kali_ast::FunctionExpression { id: Option<String>, params: Vec<FunctionParam>, body: Option<Box<BlockStatement>>, is_async: bool, generator: bool }`.
- Produces: `Parser::try_parse_block_arrow_function_expression(&mut self) -> Option<Expression>` (pub(crate), in `declaration.rs`); arrows appear in codegen's `functions: BTreeMap<String, u32>` under their `__kali_fn_N` names. Later tasks rely on nothing else from this task.

- [ ] **Step 1: Write the failing integration tests**

Create `crates/kali_cli/tests/closure_return_isolation.rs` with exactly:

```rust
//! Regression tests for the const-bound-arrow return-escape miscompile
//! (baseline classes 2/4/5): declaring an arrow whose body contains a return
//! (explicit, or the implicit return synthesized for an expression body) used
//! to emit a real wasm `return` into the ENCLOSING function, silently
//! truncating execution with exit 0. Arrows must instead compile as
//! standalone wasm functions — the lane named functions and unnamed function
//! expressions already ride.

use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(filename: &str, source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali")
}

fn assert_run_stdout(filename: &str, source: &str, expected_stdout: &str) {
    let output = run_source(filename, source);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected_stdout,
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_executes_statements_after_const_expression_bodied_arrow_declaration() {
    assert_run_stdout(
        "decl-only.js",
        "console.log(\"A\");\nconst f = (x) => x;\nconsole.log(\"B\");\n",
        "A\nB\n",
    );
}

#[test]
fn run_calls_const_expression_bodied_arrow_via_binding() {
    assert_run_stdout(
        "call-arrow.js",
        "const h = (x) => x + 1;\nconsole.log(h(41));\nconsole.log(\"after\");\n",
        "42\nafter\n",
    );
}

#[test]
fn run_executes_block_bodied_arrow_body_at_call_time_not_declaration() {
    // Class-2 shape: the trailing argument must be evaluated (printing "bump")
    // at the Math.atan2 call, then the fold result 0 prints, then execution
    // continues. node ground truth: bump, 0, after.
    assert_run_stdout(
        "block-arrow.js",
        "const bump = () => { console.log(\"bump\"); return 2; };\nconsole.log(Math.atan2(0, 1, bump()));\nconsole.log(\"after\");\n",
        "bump\n0\nafter\n",
    );
}

#[test]
fn run_object_enumeration_survives_const_arrow_preamble() {
    // Class-4/5 shape: the consumeArray declaration/calls must not truncate
    // the top-level enumeration logs. node ground truth: 2.
    assert_run_stdout(
        "enum-preamble.js",
        r#"const obj = { "a": 1, "b": 2 };
const keys = Object.keys(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
console.log(keys.length);
"#,
        "2\n",
    );
}
```

- [ ] **Step 2: Run the new file to verify it fails**

Run: `cargo test -p kali_cli --test closure_return_isolation`
Expected: FAIL — 4 failed. Typical failures: `run_executes_statements_after_const_expression_bodied_arrow_declaration` sees stdout `"A\n"` (missing `B`); the block-arrow test sees `"bump\n"` printed at declaration with `0`/`after` missing; the enumeration test sees empty stdout.

- [ ] **Step 3: Write the failing parser unit tests**

Append to `crates/kali_parser/src/declaration_tests.rs`:

```rust
#[test]
fn test_parse_block_bodied_arrow_declarator_init_as_function_expression() {
    let tokens = lex("const bump = () => { console.log(\"bump\"); return 2; };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1, "{:?}", output.statements);
    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected variable declaration, got {:?}",
            output.statements[0]
        );
    };
    assert_eq!(decl.kind, "const");
    assert_eq!(decl.declarations[0].id, "bump");
    let init = decl.declarations[0].init.as_ref().expect("declarator init");
    let Expression::FunctionExpression(function) = init else {
        panic!("expected function-expression init, got {init:?}");
    };
    assert_eq!(function.id, None);
    assert!(function.params.is_empty());
    assert!(!function.is_async);
    assert!(!function.generator);
    let body = function.body.as_ref().expect("function body");
    assert_eq!(body.body.len(), 2, "{:?}", body.body);
    assert!(matches!(body.body[1], Statement::ReturnStatement(_)));
}

#[test]
fn test_parse_block_bodied_arrow_declarator_init_with_params() {
    let tokens = lex("const add = (a, b) => { return a + b; };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected variable declaration, got {:?}",
            output.statements[0]
        );
    };
    let init = decl.declarations[0].init.as_ref().expect("declarator init");
    let Expression::FunctionExpression(function) = init else {
        panic!("expected function-expression init, got {init:?}");
    };
    let names: Vec<&str> = function
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn test_parse_expression_bodied_arrow_declarator_init_stays_arrow() {
    let tokens = lex("const f = (x) => x + 1;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let Statement::VariableDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected variable declaration, got {:?}",
            output.statements[0]
        );
    };
    let init = decl.declarations[0].init.as_ref().expect("declarator init");
    assert!(
        matches!(init, Expression::ArrowFunctionExpression(_)),
        "expression-bodied arrows must keep their existing AST shape: {init:?}"
    );
}
```

(`declaration_tests.rs` already has `use crate::test_support::lex; use crate::*; use kali_ast::{Expression, Statement};` at the top — no new imports needed. `FunctionParam` is reached through the `function.params` field, no direct import required.)

- [ ] **Step 4: Run parser tests to verify the two block-arrow tests fail**

Run: `cargo test -p kali_parser declaration_tests`
Expected: FAIL — `test_parse_block_bodied_arrow_declarator_init_as_function_expression` and `..._with_params` fail (today the mangled parse yields multiple statements / a non-FunctionExpression init); `..._stays_arrow` passes.

- [ ] **Step 5: Implement the parser change**

Edit 5a — `crates/kali_parser/src/statement.rs`, in `parse_variable_declaration` (line 93), replace:

```rust
        let init = if self.stream.accept(TokenType::Eq) {
            Some(self.parse_expression())
        } else {
            None
        };
```

with:

```rust
        let init = if self.stream.accept(TokenType::Eq) {
            // Statement-bodied arrows (`(a, b) => { ... }`) are not representable
            // in the expression grammar (`ArrowFunctionExpression.body` is an
            // `Expression`, and `return` inside `{}` is a statement), so the
            // general arrow parser bails on `{` bodies. In declarator-init
            // position parse them as an unnamed `FunctionExpression` — the exact
            // AST shape `const f = function () { ... }` produces, which the whole
            // pipeline (resolver scoping, HIR synthetic naming, codegen
            // standalone-function collection, const-binding call dispatch)
            // already compiles correctly.
            if let Some(arrow) = self.try_parse_block_arrow_function_expression() {
                Some(arrow)
            } else {
                Some(self.parse_expression())
            }
        } else {
            None
        };
```

Edit 5b — `crates/kali_parser/src/declaration.rs`, insert this new method immediately after the closing brace of `try_parse_arrow_function_expression_from` (after line 272; all needed names — `Statement`, `BlockStatement`, `Expression`, `FunctionExpression`, `FunctionParam`, `TokenType` — are already imported at the top of the file):

```rust
    /// Parses `(params) => { statements }` — a block-bodied arrow — into an
    /// unnamed `FunctionExpression`. Only invoked from variable-declarator init
    /// position (`parse_variable_declaration`); every other position keeps the
    /// legacy behavior so the `Kali.test('…', () => { … })` callback lane is
    /// untouched. Returns `None` (with the stream position unchanged) unless
    /// the tokens ahead are exactly a paren parameter list, `=>`, then `{`.
    pub(crate) fn try_parse_block_arrow_function_expression(&mut self) -> Option<Expression> {
        let start = self.stream.position;
        let mut scan = start;
        let mut params = Vec::new();
        match self.stream.tokens.get(scan).map(|token| &token.kind) {
            Some(TokenType::LeftParen) => {
                scan += 1;
                match self.stream.tokens.get(scan).map(|token| &token.kind) {
                    Some(TokenType::RightParen) => {
                        scan += 1;
                    }
                    Some(TokenType::Identifier) => loop {
                        let token = self.stream.tokens.get(scan)?;
                        params.push(token.value.clone());
                        scan += 1;

                        match self.stream.tokens.get(scan).map(|token| &token.kind) {
                            Some(TokenType::Comma) => {
                                scan += 1;
                            }
                            Some(TokenType::RightParen) => {
                                scan += 1;
                                break;
                            }
                            _ => return None,
                        }
                    },
                    _ => return None,
                }
            }
            _ => return None,
        }

        if self.stream.tokens.get(scan).map(|token| &token.kind) != Some(&TokenType::Arrow) {
            return None;
        }
        if self.stream.tokens.get(scan + 1).map(|token| &token.kind)
            != Some(&TokenType::LeftBrace)
        {
            return None;
        }

        self.stream.position = scan + 1;
        let Some(Statement::BlockStatement(block)) = self.parse_block_statement() else {
            self.stream.position = start;
            return None;
        };
        Some(Expression::FunctionExpression(Box::new(FunctionExpression {
            id: None,
            params: params
                .into_iter()
                .map(|name| FunctionParam { name })
                .collect(),
            body: Some(Box::new(block)),
            is_async: false,
            generator: false,
        })))
    }
```

- [ ] **Step 6: Run parser tests to verify they pass**

Run: `cargo test -p kali_parser`
Expected: PASS — all kali_parser tests green (including the three new ones).

- [ ] **Step 7: Implement the codegen changes**

Edit 7a — `crates/kali_codegen/src/lower.rs`, in `function_plan` (line 739), replace:

```rust
    let body_id = *node.children.last()?;
    if nodes.get(body_id.0 as usize)?.kind != LirNodeKind::Block {
        return None;
    }
```

with:

```rust
    let body_id = *node.children.last()?;
    let body_node = nodes.get(body_id.0 as usize)?;
    // A function body is either a real `Block` (function declarations,
    // function expressions, block-bodied arrows) or the single synthesized
    // `Branch("return")` statement an expression-bodied arrow lowers to
    // (`(x, y) => x + y`). Recognizing the latter compiles const-bound arrows
    // as standalone wasm functions: inside their own function the emitted
    // `Instruction::Return` is correct, whereas inlining it at the declaration
    // site terminated the ENCLOSING function (silently truncating execution
    // with exit 0). Call sites already dispatch through the const `bindings`
    // resolution in `resolve_bound_member_callable_node`.
    let is_block_body = body_node.kind == LirNodeKind::Block;
    let is_arrow_return_body =
        body_node.kind == LirNodeKind::Branch && body_node.text.as_deref() == Some("return");
    if !is_block_body && !is_arrow_return_body {
        return None;
    }
```

Edit 7b — `crates/kali_codegen/src/emit/call.rs`, in `emit_call`'s generic argument loop (line 2187), replace:

```rust
        for (arg_index, arg) in node.children.iter().skip(1).enumerate() {
            let _ = self.emit_node(function, *arg, true);
```

with:

```rust
        for (arg_index, arg) in node.children.iter().skip(1).enumerate() {
            let produced = self.emit_node(function, *arg, true);
            // A function-valued argument (e.g. an arrow, compiled as a
            // standalone function and skipped by `is_function_like` here)
            // produces no stack value; pad with a zero placeholder so the call
            // arity — and the fallback per-argument `Drop` loop — stay valid.
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
```

(The rest of the loop body — the F64 promotion `if` — is unchanged.)

- [ ] **Step 8: Run the integration tests to verify they pass**

Run: `cargo test -p kali_cli --test closure_return_isolation`
Expected: PASS — 4 passed, 0 failed.

- [ ] **Step 9: Run the failing class lanes**

Run: `cargo test -p kali_cli --test browser_math_atan2_trailing_argument_evaluation_bundle --test browser_math_atan2_trailing_argument_evaluation_harness`
Expected: PASS — 14 tests, 0 failed (class 2 except the 2 runtime_smoke members).

Run: `cargo test -p kali_cli --test runtime_smoke -- atan2_trailing`
Expected: PASS — includes `run_supports_math_atan2_trailing_argument_evaluation_in_js_input` and `run_and_test_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_js_input`.

Run: `cargo test -p kali_cli --test runtime_smoke -- object_enumeration`
Expected: PASS for every previously-failing enumeration test EXCEPT none — all green (classes 4/5; the 10 typo-fixture tests pass because `throw` is a no-op — the typo is corrected in Task 2).

Run: `cargo test -p kali_cli --test runtime_smoke -- string_primitive_enumeration`
and: `cargo test -p kali_cli --test runtime_smoke -- object_values`
and: `cargo test -p kali_cli --test runtime_smoke -- integer_like`
Expected: PASS, 0 failed each.

- [ ] **Step 10: Full gate — verify exactly 59 failures remain, all in the untouched classes**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli --no-fail-fast 2>&1 | tail -40`
Expected: kali_lexer/kali_common/kali_types/kali_codegen fully green; kali_cli down from 138 to **59** failures, all matching classes 1 (29), 3 (6), 6 (13), 7 (2), 8 (6), 9 (1), 10 (1), 11 (1). **If any test that passed on the baseline now fails, STOP and investigate before committing** — prime suspects are tests that pin bundle metadata/exports (arrows now add `__kali_fn_N` wasm exports) or tests relying on inline execution of arrow bodies. None were found during planning (the only const-bound block-arrow fixtures in the tree are the failing class-2 `bump` fixtures), but this is the checkpoint that proves it.

- [ ] **Step 11: Format and commit**

```bash
cargo fmt --all
git add crates/kali_codegen/src/lower.rs crates/kali_codegen/src/emit/call.rs crates/kali_parser/src/statement.rs crates/kali_parser/src/declaration.rs crates/kali_parser/src/declaration_tests.rs crates/kali_cli/tests/closure_return_isolation.rs
git commit -m "fix(codegen,parser): isolate const-bound arrow closures as standalone wasm functions

Declaring a const-bound arrow whose body contains a return (explicit or the
synthesized implicit return of an expression body) inlined the arrow's
statements at the declaration site, so the real wasm Return introduced by
104ef4de1 escaped into and terminated the ENCLOSING function (exit 0, silent
truncation). Two defects, one lane fix:

- function_plan now accepts the arrow LIR body shape (last child
  Branch(\"return\")) alongside Block, so arrows are collected as standalone
  wasm functions, skipped at their declaration site by is_function_like, and
  dispatched through the existing const-binding call resolution — the same
  lane named functions and unnamed function expressions already ride
  correctly.
- Block-bodied arrows ((…) => { … }) were not parseable as arrows at all
  (both arrow paths bail on '{'); in variable-declarator init position they
  now parse as an unnamed FunctionExpression with a real BlockStatement body.
  Call-argument positions (the Kali.test callback lane) are deliberately
  unchanged.
- emit_call pads non-produced arguments with an i64 zero so function-valued
  arguments cannot underflow the wasm stack.

Fixes baseline classes 2/4/5 (79 tests): Math.atan2 trailing-argument
evaluation and the object-enumeration families, whose pins were already
node-correct.

Spec: docs/superpowers/specs/2026-07-02-baseline-green-and-closure-return-fix-design.md"
```

---

### Task 2: `keys.length !== 2` → 4-key checks in 10 enumeration fixtures (fixes 0 additional — fixture integrity within the 38 already fixed)

The 4-key object `{ "b": 1, "2": 2, "a": 3, "1": 4 }` is asserted with 2-key checks in 10 fixtures (6 inline + 1 shared helper used by 4 tests). Since `throw` is a no-op the tests pass either way; the typo fix restores the fixtures' intended self-checks so they become meaningful the day `ThrowStmt` codegen lands.

**Files:**
- Modify: `crates/kali_cli/tests/runtime_smoke.rs` (fn `browser_runtime_object_enumeration_test_source`, currently line 933)
- Modify: `crates/kali_cli/tests/runtime_smoke/run.rs` (tests `json_run_supports_integer_like_key_ordering_semantics_in_js_input` ~18183, `..._in_ts_input` ~18247)
- Modify: `crates/kali_cli/tests/runtime_smoke/test.rs` (tests `json_test_supports_integer_like_key_ordering_semantics_in_js_input` ~13680, `..._in_ts_input` ~13746, `test_supports_object_enumeration_integer_like_key_ordering` ~11100, `..._in_js_input` ~11155)

**Interfaces:** none — fixture-string edits only; no Rust signatures change.

- [ ] **Step 1: Apply the JS-level replacement in all 7 sites**

In each of the 7 locations above, the JS fixture contains this defective check block (the shared helper's version has a trailing `||` because more clauses follow; the 6 inline fixtures end at `values[1] !== 2` before the closing `)`):

```js
  keys.length !== 2 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  entries.length !== 2 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  values.length !== 2 ||
  values[0] !== 4 ||
  values[1] !== 2
```

Replace it with the correct 4-key block, matching the verified sibling `browser_runtime_integer_like_object_enumeration_test_source` (runtime_smoke.rs:1295; node ground truth `Object.keys({"b":1,"2":2,"a":3,"1":4})` → `["1","2","b","a"]`, values `[4,2,1,3]`):

```js
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
```

Preserve each site's trailing `||` (the runtime_smoke.rs:933 helper) or lack of it (the 6 inline fixtures), and each site's Rust string quoting (raw string vs `\"`-escaped). Touch ONLY the `keys`/`entries`/`values` clauses of the 4-key `obj` — the `fromEntries*`/`frozen*` clauses in the :933 helper legitimately use 2 and stay.

- [ ] **Step 2: Run the 10 affected tests**

Run: `cargo test -p kali_cli --test runtime_smoke -- integer_like_key_ordering`
Expected: PASS — 4 tests (the json_run/json_test pairs).

Run: `cargo test -p kali_cli --test runtime_smoke -- object_enumeration_integer_like_key_ordering`
Expected: PASS — 2 tests.

Run: `cargo test -p kali_cli --test runtime_smoke -- object_enumeration_in`
Expected: PASS — includes the 4 test.rs users of the :933 helper (`test_accepts_browser_api_surface_with_object_enumeration_in_{js,ts}_input_when_a_browser_harness_command_is_configured` and the two `inherited` variants).

- [ ] **Step 3: Format and commit**

```bash
cargo fmt --all
git add crates/kali_cli/tests/runtime_smoke.rs crates/kali_cli/tests/runtime_smoke/run.rs crates/kali_cli/tests/runtime_smoke/test.rs
git commit -m "test(cli): fix keys.length 2->4 typo in 10 object-enumeration fixtures

The 4-key fixture object { b:1, \"2\":2, a:3, \"1\":4 } was self-checked with
keys/entries/values.length !== 2 and only index 0..1 comparisons in 6 inline
fixtures and browser_runtime_object_enumeration_test_source. Align them with
the correct 4-key sibling (node: keys [\"1\",\"2\",\"b\",\"a\"], values
[4,2,1,3]). Behavior-neutral today because throw lowers to a no-op (recorded
follow-up), but the self-checks become meaningful once ThrowStmt codegen
lands."
```

---

### Task 3: BigInt literal division lowers to `I64DivS` (class 7 — fixes 2 of 138)

`console.log(3n / 2n)` prints `1.5` because `emit_binary` unconditionally treats `/` as float. JS BigInt division truncates toward zero — exactly `i64.div_s`.

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs` (new helper; `is_float_valued` `/` arm ~line 495; `emit_binary` `float_op` ~line 614 and `/` arm ~line 672)
- Test: `crates/kali_codegen/src/emit/operators_tests.rs`

**Interfaces:**
- Produces: `FunctionEmitter::is_bigint_literal_valued(&self, id: LirNodeId) -> bool` (private to the emitter impl). No other task depends on it.

- [ ] **Step 1: Write the failing unit tests**

Append to `crates/kali_codegen/src/emit/operators_tests.rs`:

```rust
#[test]
fn bigint_literal_division_lowers_to_truncating_integer_division() {
    // node: (3n / 2n).toString() === "1" — BigInt `/` truncates toward zero.
    let program = parse_and_lower_lir("console.log(3n / 2n);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.div_s"), "{printed}");
    assert!(!printed.contains("f64.div"), "{printed}");
}

#[test]
fn number_division_still_lowers_to_float_division() {
    let program = parse_and_lower_lir("console.log(3 / 2);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("f64.div"), "{printed}");
}
```

- [ ] **Step 2: Run to verify red**

Run: `cargo test -p kali_codegen bigint_literal_division`
Expected: FAIL — `i64.div_s` absent, `f64.div` present. (`number_division_still_lowers_to_float_division` already passes.)

- [ ] **Step 3: Implement**

Edit 3a — `crates/kali_codegen/src/emit/operators.rs`: insert this helper immediately after `is_float_literal_text` (after line 419), inside the same `impl<'a> FunctionEmitter<'a>` block:

```rust
    /// True when `id` (after unwrapping transparent wrappers and resolving
    /// const bindings) is a BigInt literal such as `3n`. BigInt arithmetic
    /// stays on the i64 lane; in particular JS BigInt `/` truncates toward
    /// zero — `i64.div_s` — never `f64.div`. Scope is deliberately literal /
    /// const-bound-literal operands: the repr machinery has no BigInt axis
    /// yet, so BigInt-typed mutable locals keep the (wrong) float path — a
    /// recorded follow-up.
    fn is_bigint_literal_valued(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        node.kind == LirNodeKind::Literal
            && node
                .text
                .as_deref()
                .and_then(|text| text.strip_suffix('n'))
                .is_some_and(|digits| digits.parse::<i64>().is_ok())
    }
```

Edit 3b — in `is_float_valued`, the two-child binary arm (line 494-495), replace:

```rust
                        match text {
                            "/" => true,
```

with:

```rust
                        match text {
                            "/" => !(self.is_bigint_literal_valued(node.children[0])
                                && self.is_bigint_literal_valued(node.children[1])),
```

Edit 3c — in `emit_binary`, replace (lines 613-619):

```rust
        let operand_float = self.is_float_valued(left) || self.is_float_valued(right);
        let float_op = match op {
            "/" => true,
            "+" | "-" | "*" => operand_float,
            "<" | "<=" | ">" | ">=" | "==" | "===" | "!=" | "!==" => operand_float,
            _ => false,
        };
```

with:

```rust
        let operand_float = self.is_float_valued(left) || self.is_float_valued(right);
        let float_op = match op {
            // `/` is float (JS division yields a double in this model) UNLESS
            // both operands are BigInt literals: BigInt division truncates
            // toward zero and must stay on the i64 lane (`i64.div_s`).
            "/" => {
                !(self.is_bigint_literal_valued(left) && self.is_bigint_literal_valued(right))
            }
            "+" | "-" | "*" => operand_float,
            "<" | "<=" | ">" | ">=" | "==" | "===" | "!=" | "!==" => operand_float,
            _ => false,
        };
```

Edit 3d — in `emit_binary`, replace the `/` arm (lines 672-679):

```rust
            "/" => {
                // `float_op` is always true here.
                function.instruction(&Instruction::F64Div);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Float,
                }
            }
```

with:

```rust
            "/" => {
                if float_op {
                    function.instruction(&Instruction::F64Div);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Float,
                    }
                } else {
                    // BigInt `/`: truncation toward zero is exactly `i64.div_s`.
                    function.instruction(&Instruction::I64DivS);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            }
```

- [ ] **Step 4: Run unit tests to verify green**

Run: `cargo test -p kali_codegen`
Expected: PASS — all kali_codegen tests including both new ones.

- [ ] **Step 5: Run the two failing integration tests**

Run: `cargo test -p kali_cli --test runtime_smoke -- bigint_division`
Expected: PASS — `run::run_supports_bigint_division_semantics` and `..._in_js_input` (their pin `"1"` was already node-correct).

Also run: `cargo test -p kali_cli --test runtime_smoke -- bigint`
Expected: PASS, 0 failed (no sibling BigInt lane regressed).

- [ ] **Step 6: Format and commit**

```bash
cargo fmt --all
git add crates/kali_codegen/src/emit/operators.rs crates/kali_codegen/src/emit/operators_tests.rs
git commit -m "fix(codegen): lower BigInt literal division to i64.div_s (truncate toward zero)

aa9c6765e0 made '/' unconditionally F64Div (\"JS division yields a double\"),
which is wrong for BigInt operands: node (3n / 2n).toString() === '1', kali
printed 1.5. When both '/' operands are BigInt literals (n-suffixed LIR
literals, resolved through transparent wrappers and const bindings) the
operation now stays on the i64 lane in both emit_binary and its
is_float_valued mirror. BigInt-typed mutable locals are a recorded follow-up
(no BigInt repr axis yet)."
```

---

### Task 4: Keyword tokens as property names after `.` (class 8 — fixes 6 of 138)

`event.type` fails to parse because the lexer reserves `type` and the member parser only accepts `Identifier | Delete | From` after `.`; inside the strict `${…}` sub-parse this surfaces as E2004.

**Files:**
- Modify: `crates/kali_parser/src/expression/call.rs` (Dot arm, lines 44-70; new associated fn)
- Test: `crates/kali_parser/src/expression/call_tests/member.rs`

**Interfaces:**
- Produces: `Parser::is_property_name_token(kind: &TokenType) -> bool` (pub(crate) associated fn in `call.rs`). No other task depends on it.

- [ ] **Step 1: Write the failing parser unit test**

Append to `crates/kali_parser/src/expression/call_tests/member.rs`:

```rust
#[test]
fn test_parse_keyword_property_names_after_dot() {
    // Reserved words are valid property names in JS/TS (`event.type`,
    // `config.default`, ...). The lexer reserves these words, so the member
    // parser must accept keyword-shaped tokens after `.`.
    let tokens = lex(
        "event.type; config.default; chain.from; task.async; list.of; state.case; item.new; box.in;",
    );
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 8, "{:?}", output.statements);

    let expected = [
        ("event", "type"),
        ("config", "default"),
        ("chain", "from"),
        ("task", "async"),
        ("list", "of"),
        ("state", "case"),
        ("item", "new"),
        ("box", "in"),
    ];
    for (statement, (object, property)) in output.statements.iter().zip(expected) {
        let Statement::ExpressionStatement(stmt) = statement else {
            panic!("expected expression statement, got {statement:?}");
        };
        let Expression::MemberExpression(member) = stmt.expression.as_ref() else {
            panic!("expected member expression, got {:?}", stmt.expression);
        };
        assert_eq!(member.property, property);
        assert!(
            matches!(&member.object, Expression::Identifier(name) if name == object),
            "object of .{property}: {:?}",
            member.object
        );
    }
}
```

- [ ] **Step 2: Run to verify red**

Run: `cargo test -p kali_parser test_parse_keyword_property_names_after_dot`
Expected: FAIL (today `event.type` stops after `event`, so the statement is not a MemberExpression / statement count differs).

- [ ] **Step 3: Implement**

Edit 3a — `crates/kali_parser/src/expression/call.rs`, in the `Some(TokenType::Dot)` arm, replace (lines 46-49):

```rust
                    match self.stream.current_kind() {
                        Some(TokenType::Identifier)
                        | Some(TokenType::Delete)
                        | Some(TokenType::From) => {
```

with:

```rust
                    match self.stream.current_kind().copied() {
                        Some(kind) if Self::is_property_name_token(&kind) => {
```

(`.copied()` matters: binding `kind` by reference from `current_kind()` would keep `self.stream` borrowed across the arm body, conflicting with the `self.stream.advance()` call inside it — `TokenType` is `Copy`, and the file's other call sites already use the `.copied()` idiom. The arm body and the `_ => { break; }` arm are unchanged.)

Edit 3b — add this associated fn inside `impl Parser` in the same file, immediately before `parse_optional_chain_expression` (line 136):

```rust
    /// True for every token `lex_identifier` can produce: a plain identifier
    /// or any reserved word. Reserved words are valid property names after `.`
    /// in JS/TS (`event.type`, `config.default`, `list.of`, ...); the token's
    /// `value` field always carries the word text, so the member-access parser
    /// can consume it like an identifier.
    pub(crate) fn is_property_name_token(kind: &TokenType) -> bool {
        matches!(
            kind,
            TokenType::Identifier
                | TokenType::If
                | TokenType::Else
                | TokenType::For
                | TokenType::While
                | TokenType::Do
                | TokenType::Switch
                | TokenType::Case
                | TokenType::Default
                | TokenType::Break
                | TokenType::Continue
                | TokenType::Return
                | TokenType::Throw
                | TokenType::Try
                | TokenType::Catch
                | TokenType::Finally
                | TokenType::Debugger
                | TokenType::New
                | TokenType::Function
                | TokenType::Var
                | TokenType::Let
                | TokenType::Const
                | TokenType::Class
                | TokenType::Interface
                | TokenType::Type
                | TokenType::Enum
                | TokenType::Import
                | TokenType::Export
                | TokenType::From
                | TokenType::As
                | TokenType::This
                | TokenType::Super
                | TokenType::Extends
                | TokenType::Implements
                | TokenType::Async
                | TokenType::Await
                | TokenType::Yield
                | TokenType::InstanceOf
                | TokenType::In
                | TokenType::Of
                | TokenType::True
                | TokenType::False
                | TokenType::Null
                | TokenType::Undefined
                | TokenType::Void
                | TokenType::Delete
                | TokenType::Typeof
        )
    }
```

- [ ] **Step 4: Run parser tests to verify green**

Run: `cargo test -p kali_parser`
Expected: PASS — all tests, including the new one and the existing `.delete`/`.from` coverage (the new predicate is a strict superset of the old arm).

- [ ] **Step 5: Run the 6 failing integration tests**

Run: `cargo test -p kali_cli --test runtime_smoke -- structured_clone_and_event_primitives`
Expected: PASS — 6 tests (3 run.rs + 3 test.rs), stdout containing `web baseline ok` / Kali-test pass. If any still fails with a NEW diagnostic (e.g. E3200 on the desugared `'unexpected Event behavior ' + event.type`), stop and report — that would mean `event.type` is being classified as a string-typed variable operand, which planning probes did not indicate.

- [ ] **Step 6: Format and commit**

```bash
cargo fmt --all
git add crates/kali_parser/src/expression/call.rs crates/kali_parser/src/expression/call_tests/member.rs
git commit -m "fix(parser): accept keyword tokens as property names after '.'

The member parser only accepted Identifier | Delete | From after '.', so
'event.type' stopped after 'event' ('type' lexes as the reserved
TokenType::Type) and the strict \${...} sub-parser introduced by 55c1c37b5
hard-errored E2004. Accept every keyword-shaped token lex_identifier can
produce — reserved words are valid JS/TS property names and the token value
already carries the word text. Fixes the 6 web-baseline
structured-clone/event tests."
```

---

### Task 5: Flip 29 Math.sqrt(1.6) rejection tests to supported-output pins (class 1 — fixes 29 of 138)

`Math.sqrt(1.6)` is supported since e5d776d93. Ground truth, node and the built kali binary bit-for-bit: `1.2649110640673518` (verified during planning for the plain, bracket, and `globalThis` forms; the six-form harness source prints it six times; `kali test` appends `ok 1`; `--output json` puts program output in top-level `json["stdout"]` for run/test and `null` for build/check).

26 tests live in `crates/kali_cli/tests/runtime_smoke/{build,check,run,test}.rs`, 3 in `crates/kali_cli/tests/browser_math_unsupported_member_calls_harness_jsx_tsx.rs`. Rename every flipped test by substituting `rejects_unsupported_math_member_calls` → `supports_math_sqrt_member_calls` in the fn name (prefixes/suffixes unchanged). **Do NOT touch** the shared rejection helpers (`assert_unsupported_math_member_calls_rejection_{text,json}` — still used by Math.exp/Math.log/atan2/pow tests) and do NOT touch any test whose name contains `additional` or `broader` or `atan2` or `pow`.

**Files:**
- Modify: `crates/kali_cli/tests/runtime_smoke/build.rs` (6 tests, at ~8631, 8653, 8681, 8721, 8751, 8787)
- Modify: `crates/kali_cli/tests/runtime_smoke/check.rs` (7 tests, at ~2866, 2958, 3382, 3403, 3430, 3470, 3499)
- Modify: `crates/kali_cli/tests/runtime_smoke/run.rs` (8 tests, at ~15788, 16037, 16059, 16087, 16118, 16155, 16208, 16271)
- Modify: `crates/kali_cli/tests/runtime_smoke/test.rs` (5 tests, at ~12251, 12274, 12303, 12336, 12371)
- Modify: `crates/kali_cli/tests/browser_math_unsupported_member_calls_harness_jsx_tsx.rs` (new success helper + 3 flipped tests)

**Interfaces:** none consumed from earlier tasks. Produces a file-local helper `assert_browser_harness_math_sqrt_success` in the harness file only.

- [ ] **Step 1: Confirm the red state**

Run: `cargo test -p kali_cli --test runtime_smoke -- rejects_unsupported_math_member_calls 2>&1 | tail -5`
Expected: 26 failed (plus the passing `additional` siblings filtered in — those 2 pass).
Run: `cargo test -p kali_cli --test browser_math_unsupported_member_calls_harness_jsx_tsx 2>&1 | tail -5`
Expected: 3 failed (`*_unsupported_math_member_calls_*`), 3 passed (`*_broader_math_atan2_*`).

- [ ] **Step 2: Flip the 10 simple text-mode tests (groups A + D)**

Every one of these keeps its body up to and including `.output().expect("run kali");` unchanged (fixture, args, env, manifest) — only the fn name and the assertion tail change.

Members and lanes:

| file | old fn (rename per the mapping above) | lane |
|---|---|---|
| build.rs:8631 | `build_rejects_unsupported_math_member_calls_in_browser_api_surface_in_js_input` | build |
| build.rs:8721 | `build_rejects_..._in_inherited_browser_api_surface_in_js_input` | build |
| check.rs:2866 | `check_rejects_..._in_js_input` (inline asserts) | check |
| check.rs:3382 | `check_rejects_..._in_browser_api_surface_in_js_input` | check |
| check.rs:3470 | `check_rejects_..._in_inherited_browser_api_surface_in_js_input` | check |
| run.rs:15788 | `run_rejects_..._in_js_input` (inline asserts) | run |
| run.rs:16037 | `run_rejects_..._in_browser_api_surface_with_harness_js_input` | run |
| run.rs:16087 | `run_rejects_..._in_inherited_browser_api_surface_with_harness_js_input` | run |
| test.rs:12251 | `test_rejects_..._in_js_input` | test |
| test.rs:12336 | `test_rejects_..._in_inherited_browser_api_surface_with_harness_js_input` | test |

Old assertion tail (helper form; the two inline-assert tests at check.rs:2866 and run.rs:15788 end with three `assert!(stderr.contains(…))` lines instead of the helper call — delete those too):

```rust
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_unsupported_math_member_calls_rejection_text(&stderr);
```

New tail by lane — **build**:

```rust
    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
```

**check**:

```rust
    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checked 1 file(s)"), "stdout: {stdout}");
```

**run**:

```rust
    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "1.2649110640673518\n", "stdout: {stdout}");
```

**test**:

```rust
    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
```

- [ ] **Step 3: Flip the 10 simple json-mode tests (group B)**

Members and lanes:

| file | old fn | lane |
|---|---|---|
| build.rs:8653 | `json_build_rejects_..._in_browser_api_surface_in_js_input` | build |
| build.rs:8751 | `json_build_rejects_..._in_inherited_browser_api_surface_in_js_input` | build |
| check.rs:2958 | `json_check_rejects_..._in_js_input` | check |
| check.rs:3403 | `json_check_rejects_..._in_browser_api_surface_in_js_input` | check |
| check.rs:3499 | `json_check_rejects_..._in_inherited_browser_api_surface_in_js_input` | check |
| run.rs:16059 | `json_run_rejects_..._in_browser_api_surface_with_harness_js_input` | run |
| run.rs:16118 | `json_run_rejects_..._in_inherited_browser_api_surface_with_harness_js_input` | run |
| test.rs:12274 | `test_rejects_..._in_js_input_in_json` | test |
| test.rs:12303 | `test_rejects_..._in_browser_api_surface_with_harness_js_input_in_json` | test |
| test.rs:12371 | `test_rejects_..._in_inherited_browser_api_surface_with_harness_js_input_in_json` | test |

Old tail (command literal is `"build"`, `"check"`, `"run"`, or `"test"` per file):

```rust
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "<cmd>");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_unsupported_math_member_calls_rejection_json(errors);
```

New tail — shared head for all lanes (keep each test's own `"<cmd>"` literal):

```rust
    // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
    // 1.2649110640673518 (bit-for-bit match with `kali run`).
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "<cmd>");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
```

then append per lane — **run**:

```rust
    assert_eq!(json["stdout"], "1.2649110640673518\n");
```

**test**:

```rust
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("json stdout")
            .contains("1.2649110640673518"),
        "json: {json}"
    );
```

**check**:

```rust
    assert_eq!(json["payload"]["filesChecked"], 1);
```

**build**: nothing further (build JSON reports `stdout: null`; success + empty errors is the pin).

- [ ] **Step 4: Rewrite the 6 loop tests (group C) as complete functions**

Replace the following 6 loop tests in full (bodies below; fn names already renamed):

build.rs:8681 →

```rust
#[test]
fn build_supports_math_sqrt_member_calls_in_browser_api_surface_in_jsx_and_tsx_input() {
    let dir = tempdir().expect("tempdir");

    for extension in ["tsx", "jsx"] {
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg("build")
                .arg("--bundle")
                .arg("--api")
                .arg("browser")
                .arg(&source_path)
                .output()
                .expect("run kali");

            // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
            // 1.2649110640673518 (bit-for-bit match with `kali run`).
            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], "build");
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
        }
    }
}
```

build.rs:8787 → identical shape; keep its own fixture filename `app.{extension}` and its `kali.json` manifest `fs::write` block exactly as they are today, drop `--api browser` from the args (it is inherited from the manifest in the original), i.e.:

```rust
#[test]
fn build_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_in_jsx_and_tsx_input() {
    let dir = tempdir().expect("tempdir");

    for extension in ["tsx", "jsx"] {
        let source_path = dir.path().join(format!("app.{extension}"));
        fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");
        fs::write(
            dir.path().join("kali.json"),
            r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
        )
        .expect("write manifest");

        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output.current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            let output = output
                .arg("build")
                .arg("--bundle")
                .arg(&source_path)
                .output()
                .expect("run kali");

            // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
            // 1.2649110640673518 (bit-for-bit match with `kali run`).
            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], "build");
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
            }
        }
    }
}
```

check.rs:3430 →

```rust
#[test]
fn check_supports_math_sqrt_member_calls_in_browser_api_surface_in_jsx_and_tsx_input() {
    let dir = tempdir().expect("tempdir");

    for extension in ["tsx", "jsx"] {
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "console.log(Math.sqrt(1.6));\n").expect("write source");

        for command in ["check", "build"] {
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output.current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command);
                if command == "build" {
                    output.arg("--bundle");
                }
                output.arg("--api").arg("browser").arg(&source_path);
                let output = output.output().expect("run kali");

                // Math.sqrt(1.6) is supported since e5d776d93; node ground
                // truth 1.2649110640673518 (bit-for-bit match with `kali run`).
                assert!(
                    output.status.success(),
                    "stdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if output_json {
                    let json = parse_json_stdout(&output);
                    assert_eq!(json["schemaVersion"], 1);
                    assert_eq!(json["command"], command);
                    assert_eq!(json["success"], true);
                    assert!(json["errors"].as_array().expect("errors array").is_empty());
                }
            }
        }
    }
}
```

run.rs:16155 → (fixture uses the bracket form `globalThis["Math"]["sqrt"](1.6)` — keep it; both source `fs::write` blocks unchanged)

```rust
#[test]
fn run_and_test_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let run_source_path = dir.path().join(format!("main.{extension}"));
        let test_source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &run_source_path,
            "console.log(globalThis[\"Math\"][\"sqrt\"](1.6));\n",
        )
        .expect("write run source");
        fs::write(
            &test_source_path,
            "Kali.test('supported math', () => { console.log(globalThis[\"Math\"][\"sqrt\"](1.6)); });\n",
        )
        .expect("write test source");

        for command in ["run", "test"] {
            let source_path = if command == "run" {
                &run_source_path
            } else {
                &test_source_path
            };
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output
                    .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                    .current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command).arg(source_path);
                let output = output.output().expect("run kali");

                // Math.sqrt(1.6) is supported since e5d776d93; node ground
                // truth 1.2649110640673518 (bit-for-bit match with `kali run`,
                // verified for the bracket/globalThis access forms too).
                assert!(
                    output.status.success(),
                    "stdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if output_json {
                    let json = parse_json_stdout(&output);
                    assert_eq!(json["schemaVersion"], 1);
                    assert_eq!(json["command"], command);
                    assert_eq!(json["success"], true);
                    assert!(json["errors"].as_array().expect("errors array").is_empty());
                    assert!(
                        json["stdout"]
                            .as_str()
                            .expect("json stdout")
                            .contains("1.2649110640673518"),
                        "json: {json}"
                    );
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
                    if command == "test" {
                        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
                    }
                }
            }
        }
    }
}
```

run.rs:16208 →

```rust
#[test]
fn run_and_test_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let run_source_path = dir.path().join(format!("main.{extension}"));
        let test_source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(
            &run_source_path,
            "console.log(globalThis[\"Math\"][\"sqrt\"](1.6));\n",
        )
        .expect("write run source");
        fs::write(
            &test_source_path,
            "Kali.test('supported math', () => { console.log(globalThis[\"Math\"][\"sqrt\"](1.6)); });\n",
        )
        .expect("write test source");
        fs::write(
            dir.path().join("kali.json"),
            r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
        )
        .expect("write manifest");

        for command in ["run", "test"] {
            let source_path = if command == "run" {
                &run_source_path
            } else {
                &test_source_path
            };
            for output_json in [false, true] {
                let mut output = Command::new(kali_bin());
                output
                    .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                    .current_dir(dir.path());
                if output_json {
                    output.arg("--output").arg("json");
                }
                output.arg(command).arg(source_path);
                let output = output.output().expect("run kali");

                // Math.sqrt(1.6) is supported since e5d776d93; node ground
                // truth 1.2649110640673518 (bit-for-bit match with `kali run`,
                // verified for the bracket/globalThis access forms too).
                assert!(
                    output.status.success(),
                    "stdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if output_json {
                    let json = parse_json_stdout(&output);
                    assert_eq!(json["schemaVersion"], 1);
                    assert_eq!(json["command"], command);
                    assert_eq!(json["success"], true);
                    assert!(json["errors"].as_array().expect("errors array").is_empty());
                    assert!(
                        json["stdout"]
                            .as_str()
                            .expect("json stdout")
                            .contains("1.2649110640673518"),
                        "json: {json}"
                    );
                } else {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
                    if command == "test" {
                        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
                    }
                }
            }
        }
    }
}
```

run.rs:16271 → (plain `Math.sqrt`, ts-only, with manifest)

```rust
#[test]
fn run_and_test_supports_math_sqrt_member_calls_in_inherited_browser_api_surface_with_harness_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let run_source_path = dir.path().join("main.ts");
    let test_source_path = dir.path().join("smoke.test.ts");
    fs::write(&run_source_path, "console.log(Math.sqrt(1.6));\n").expect("write run source");
    fs::write(
        &test_source_path,
        "Kali.test('supported math', () => { console.log(Math.sqrt(1.6)); });\n",
    )
    .expect("write test source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    for command in ["run", "test"] {
        let source_path = if command == "run" {
            &run_source_path
        } else {
            &test_source_path
        };
        for output_json in [false, true] {
            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
                output.arg("--output").arg("json");
            }
            output.arg(command).arg(source_path);
            let output = output.output().expect("run kali");

            // Math.sqrt(1.6) is supported since e5d776d93; node ground truth
            // 1.2649110640673518 (bit-for-bit match with `kali run`).
            assert!(
                output.status.success(),
                "stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output_json {
                let json = parse_json_stdout(&output);
                assert_eq!(json["schemaVersion"], 1);
                assert_eq!(json["command"], command);
                assert_eq!(json["success"], true);
                assert!(json["errors"].as_array().expect("errors array").is_empty());
                assert!(
                    json["stdout"]
                        .as_str()
                        .expect("json stdout")
                        .contains("1.2649110640673518"),
                    "json: {json}"
                );
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("1.2649110640673518"), "stdout: {stdout}");
                if command == "test" {
                    assert!(stdout.contains("ok 1"), "stdout: {stdout}");
                }
            }
        }
    }
}
```

(Note: the two `Kali.test('supported math', …)` strings rename the old `'unsupported math'` test label to match the new intent — cosmetic, nothing asserts on the label.)

- [ ] **Step 5: Flip the 3 harness-file tests**

In `crates/kali_cli/tests/browser_math_unsupported_member_calls_harness_jsx_tsx.rs`:

5a. Add this helper directly below `assert_browser_harness_unsupported_math_rejection` (which stays — the 3 atan2 tests still use it):

```rust
/// Math.sqrt on a non-perfect-square literal is SUPPORTED since e5d776d93
/// (runtime F64Sqrt). node ground truth, bit-for-bit with `kali run`:
/// Math.sqrt(1.6) -> 1.2649110640673518. The fixtures call sqrt through six
/// access forms, so run/test stdout carries the value exactly six times.
fn assert_browser_harness_math_sqrt_success(
    command: &str,
    filename: &str,
    source: &str,
    bundle: bool,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if bundle {
        cli.arg("--bundle");
    }
    cli.arg("--api").arg("browser").arg(&source_path);

    let output = cli.output().expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        if command != "build" {
            let stdout = json["stdout"].as_str().expect("json stdout");
            assert_eq!(
                stdout.matches("1.2649110640673518").count(),
                6,
                "stdout: {stdout}"
            );
        }
    } else if command != "build" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.matches("1.2649110640673518").count(),
            6,
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}
```

5b. Rewrite the 3 sqrt tests (lines 96-115, 138-157, 180-199) to use the new helper and names — the loop shapes and source factories (`browser_harness_run_source`, `browser_harness_test_source`) stay:

```rust
#[test]
fn run_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_math_sqrt_success(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_source(),
            false,
            false,
        );
        assert_browser_harness_math_sqrt_success(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_source(),
            false,
            true,
        );
    }
}
```

```rust
#[test]
fn test_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_math_sqrt_success(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_source(),
            false,
            false,
        );
        assert_browser_harness_math_sqrt_success(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_source(),
            false,
            true,
        );
    }
}
```

```rust
#[test]
fn build_supports_math_sqrt_member_calls_in_browser_api_surface_with_harness_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        assert_browser_harness_math_sqrt_success(
            "build",
            &format!("main.{extension}"),
            browser_harness_run_source(),
            true,
            false,
        );
        assert_browser_harness_math_sqrt_success(
            "build",
            &format!("main.{extension}"),
            browser_harness_run_source(),
            true,
            true,
        );
    }
}
```

The 3 `*_broader_math_atan2_*` tests and both atan2 source factories remain byte-identical.

- [ ] **Step 6: Run the flipped tests**

Run: `cargo test -p kali_cli --test runtime_smoke -- supports_math_sqrt_member_calls`
Expected: PASS — 26 tests, 0 failed.
Run: `cargo test -p kali_cli --test browser_math_unsupported_member_calls_harness_jsx_tsx`
Expected: PASS — 6 tests (3 flipped + 3 atan2), 0 failed.

- [ ] **Step 7: Verify the rejection siblings did not drift**

Run: `cargo test -p kali_cli --test runtime_smoke -- additional_unsupported_math_member_calls`
and: `cargo test -p kali_cli --test runtime_smoke -- math_pow`
Expected: PASS each, 0 failed (Math.exp/Math.log/pow rejection pins untouched).

- [ ] **Step 8: Format and commit**

```bash
cargo fmt --all
git add crates/kali_cli/tests/runtime_smoke/build.rs crates/kali_cli/tests/runtime_smoke/check.rs crates/kali_cli/tests/runtime_smoke/run.rs crates/kali_cli/tests/runtime_smoke/test.rs crates/kali_cli/tests/browser_math_unsupported_member_calls_harness_jsx_tsx.rs
git commit -m "test(cli): flip 29 Math.sqrt(1.6) rejection pins to supported-output pins

e5d776d93 narrowed the sqrt rejection and added real F64Sqrt codegen, but
left 29 integration tests pinning the old E5506 rejection. Flip them to
success and pin the real output: node -e 'console.log(Math.sqrt(1.6))' ->
1.2649110640673518, bit-for-bit identical to kali run/test/json stdout for
the dot, bracket, and globalThis access forms (x6 for the six-form harness
fixture). Tests renamed rejects_unsupported_math_member_calls ->
supports_math_sqrt_member_calls; the Math.exp/Math.log/atan2/pow rejection
siblings and shared rejection helpers are untouched."
```

---

### Task 6: Pin the real `greet(1n, 2n)` result in the browser-bundle smoke helper (class 6 — fixes 13 of 138)

**Files:**
- Modify: `crates/kali_cli/tests/runtime_smoke.rs` (fn `assert_browser_bundle_executes`, currently line 1638)
- Modify: `crates/kali_cli/tests/runtime_smoke/build.rs` (13 `"greet"` call sites)

**Interfaces:**
- Produces: `assert_browser_bundle_executes_with_result(bundle_root: &Path, export_name: &str, expected: &str)` in `runtime_smoke.rs`; `assert_browser_bundle_executes(bundle_root, export_name)` keeps its signature and delegates with `"0"` — the other 94 call sites are untouched.

- [ ] **Step 1: Confirm the red state**

Run: `cargo test -p kali_cli --test runtime_smoke -- browser_bundle 2>&1 | tail -5`
Expected: 13 failures, all in `build::`, each with node stderr `Error: unexpected result 1`.

- [ ] **Step 2: Refactor the helper**

In `crates/kali_cli/tests/runtime_smoke.rs`, replace the whole `assert_browser_bundle_executes` fn (line 1638-1680) with:

```rust
/// Runs the bundle's `export_name(1n, 2n)` through the node harness and pins
/// the result. `expected` is the decimal digits of the expected BigInt (i64
/// exports surface as BigInt through the raw wasm-export ABI). Pinning the
/// real source value proves the exported body actually executed; a blanket
/// `0n` pin is indistinguishable from a dead stub (b5c085401 precedent).
fn assert_browser_bundle_executes_with_result(
    bundle_root: &Path,
    export_name: &str,
    expected: &str,
) {
    let bundle_dir = bundle_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("bundle directory name");
    let harness_path = bundle_root
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        bundle_dir,
        false,
        &format!(
            r#"const mod = await import(bundleJs.href);
const result = await mod.{export_name}(1n, 2n);
if (result !== {expected}n) {{
  throw new Error(`unexpected result ${{result}}`);
}}
console.log(String(result));
"#,
            export_name = export_name,
            expected = expected,
        ),
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_bundle_harness_command_parts();
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(bundle_root)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(expected), "stdout: {stdout}");
}

/// Callers whose fixtures genuinely `return 0n;`.
fn assert_browser_bundle_executes(bundle_root: &Path, export_name: &str) {
    assert_browser_bundle_executes_with_result(bundle_root, export_name, "0");
}
```

- [ ] **Step 3: Update the 13 greet call sites**

In `crates/kali_cli/tests/runtime_smoke/build.rs`, replace **every** occurrence of:

```rust
    assert_browser_bundle_executes(&bundle_dir, "greet");
```

with:

```rust
    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&bundle_dir, "greet", "1");
```

and every occurrence of:

```rust
    assert_browser_bundle_executes(&dir.path().join("app"), "greet");
```

with:

```rust
    // greet is `function greet(name) { return name; }` — greet(1n, 2n) === 1n.
    assert_browser_bundle_executes_with_result(&dir.path().join("app"), "greet", "1");
```

(That is 5 + 8 = 13 sites, at build.rs lines ~3369, 3438, 3515, 3605, 3656, 11325, 11367, 11421, 11455, 11497, 11531, 11590, 11645. Verify with `grep -n '"greet")' crates/kali_cli/tests/runtime_smoke/build.rs` — zero hits must remain for the old helper with `"greet"`.)

- [ ] **Step 4: Run the affected lanes**

Run: `cargo test -p kali_cli --test runtime_smoke -- browser_bundle`
Expected: PASS, 0 failed — the 13 greet callers now pin `1n`; the ~94 `return 0n;` callers still pass through the delegating wrapper.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt --all
git add crates/kali_cli/tests/runtime_smoke.rs crates/kali_cli/tests/runtime_smoke/build.rs
git commit -m "test(cli): pin real greet(1n,2n) result 1n in browser bundle smoke harness

assert_browser_bundle_executes pinned 0n from the stub era. Its 13 greet
callers ship 'function greet(name) { return name; }', so greet(1n, 2n)
correctly returns its first argument 1n (JS: ((name) => name)(1n, 2n) === 1n)
now that bodies execute for real (104ef4de1). Parameterize the helper with
the expected BigInt digits; greet callers pin 1, the remaining callers keep 0
via a delegating wrapper because their fixtures genuinely return 0n. Mirrors
the b5c085401 dynamic-import chunk re-pin."
```

---

### Task 7: Rewrite compat-eval fixtures to literal-rooted concatenation (class 3 — fixes 6 of 138)

The fixtures build their eval/Function source strings with `prefix + suffix` on string-typed const variables, which the sound E3200 guard now rejects. Pure-literal concatenation preserves the dynamic-eval intent and was probe-verified green (`run`/`build`/`check --compat eval`) during planning.

**Files:**
- Modify: `crates/kali_cli/tests/runtime_smoke/run.rs` (tests at ~8443 and ~8470)
- Modify: `crates/kali_cli/tests/runtime_smoke/build.rs` (tests at ~600 and ~631)
- Modify: `crates/kali_cli/tests/runtime_smoke/check.rs` (tests at ~2410 and ~2439)

**Interfaces:** none — fixture-string edits only.

- [ ] **Step 1: Rewrite the eval fixture (run.rs only)**

In `run_evaluates_dynamic_eval_sources_when_compat_eval_is_enabled` (run.rs:8443), replace the `fs::write` source string:

```rust
        "const prefix = \"1\"; const suffix = \" + 2\"; const source = prefix + suffix; if (eval(source) !== 3) { throw new Error('bad eval result'); }",
```

with:

```rust
        "const source = \"1\" + \" + 2\"; if (eval(source) !== 3) { throw new Error('bad eval result'); }",
```

- [ ] **Step 2: Rewrite the Function-constructor fixture (5 sites)**

In all five of: `run_evaluates_dynamic_function_constructor_sources_when_compat_eval_is_enabled` (run.rs:8470), `build_accepts_dynamic_function_constructor_sources_when_compat_eval_is_enabled` (build.rs:600) and `..._in_json` (build.rs:631), `check_accepts_dynamic_function_constructor_sources_when_compat_eval_is_enabled` (check.rs:2410) and `..._in_json` (check.rs:2439), replace the identical source string:

```rust
        "const bodyPrefix = \"return \"; const body = bodyPrefix + \"1 + 2;\"; const value = new Function(body)(); if (value !== 3) { throw new Error('bad function result'); }",
```

with:

```rust
        "const body = \"return \" + \"1 + 2;\"; const value = new Function(body)(); if (value !== 3) { throw new Error('bad function result'); }",
```

- [ ] **Step 3: Run the six tests**

Run: `cargo test -p kali_cli --test runtime_smoke -- compat_eval_is_enabled`
Expected: PASS — includes all 6 (2 run, 2 build, 2 check), 0 failed.

- [ ] **Step 4: Format and commit**

```bash
cargo fmt --all
git add crates/kali_cli/tests/runtime_smoke/run.rs crates/kali_cli/tests/runtime_smoke/build.rs crates/kali_cli/tests/runtime_smoke/check.rs
git commit -m "test(cli): rewrite compat-eval fixtures to literal-rooted concatenation (E3200)

The db1463d51/2e28aaa3a/0e9c430e8 guard soundly rejects '+' with a
string-typed variable operand (the old lowering miscompiled it). The
compat-eval fixtures' prefix + suffix source-string construction was
incidental to what they test (dynamic eval/Function under --compat eval);
build the strings from string literals instead. Expected values unchanged
(eval('1 + 2') === 3; new Function('return 1 + 2;')() === 3, node-verified)."
```

---

### Task 8: Benchmark corpus — literal-rooted concat fixtures, sha256, and slug lists (classes 9, 10, 11 — fixes 3 of 138)

Three benchmark fixtures trip E3200 (via the string-typed `folded` const), aborting the optimization suite (class 9) and the fixture-tree `check` walk (class 10); separately the schema_docs slug lists lack the two new CLBG fixtures (class 11). The rewrites below were probe-verified green under `kali check`, and their sha256 values were computed from the exact file bytes given here.

**Files:**
- Modify: `crates/kali_cli/tests/fixtures/benchmarks/string-concatenation-benchmark-v1.ts` and `.json`
- Modify: `crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1.ts` and `.json`
- Modify: `crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1-js.js` and `.json`
- Modify: `crates/kali_cli/tests/schema_docs/misc.rs` (the two `expected_benchmark_*` lists, ~lines 2073-2206)

**Interfaces:** none.

- [ ] **Step 1: Confirm the red state**

Run: `cargo test -p kali_cli --test runtime_smoke -- optimization_benchmark_suite`
Expected: FAIL at `string-concatenation-benchmark-v1` with E3200 in the captured stderr.
Run: `cargo test -p kali_cli --test runtime_smoke -- check_discovers_fixture_tree`
Expected: FAIL (non-zero exit, 4× E3200 across the three fixtures).
Run: `cargo test -p kali_cli --test schema_docs -- benchmark_fixture_metadata_schema`
Expected: FAIL (`benchmark slugs should match the checked-in benchmark corpus`).

- [ ] **Step 2: Rewrite `string-concatenation-benchmark-v1.ts`**

Replace the file's entire contents with exactly (the only change vs. today: `hot` inlines the literal-folded chain instead of routing it through the string-typed `folded` const — the measured fold shape is preserved, the E3200-tripping variable operand is gone):

```typescript
function dead0(value) { return ("ka" + "li") + value; }
function dead1(value) { return ("ka" + "li") + value; }
function dead2(value) { return ("ka" + "li") + value; }
function dead3(value) { return ("ka" + "li") + value; }
function dead4(value) { return ("ka" + "li") + value; }

function hot(prefix, suffix) {
  return prefix + (("a" + "head") + ("-" + "of") + ("-" + "time")) + suffix;
}

hot("start-", "-end");
```

- [ ] **Step 3: Rewrite both template-literal fixtures (byte-identical .ts and .js)**

Replace the entire contents of BOTH `template-literal-concatenation-benchmark-v1.ts` and `template-literal-concatenation-benchmark-v1-js.js` with exactly (only `hot` changes — the folded template is inlined into the return template):

```typescript
function dead0(value) {
  if (false) {
    return `ka${"li"}${value}`;
  }
  return value;
}
function dead1(value) {
  if (false) {
    return `ka${"li"}${value}`;
  }
  return value;
}
function dead2(value) {
  if (false) {
    return `ka${"li"}${value}`;
  }
  return value;
}
function dead3(value) {
  if (false) {
    return `ka${"li"}${value}`;
  }
  return value;
}
function dead4(value) {
  if (false) {
    return `ka${"li"}${value}`;
  }
  return value;
}

function hot(prefix, suffix) {
  return `${prefix}${"a"}${"head"}-${"of"}-${"time"}${suffix}`;
}

hot("start-", "-end");
```

- [ ] **Step 4: Update the three metadata sha256 values**

The metadata schema pins `sourceSha256` = `sha256-` + lowercase hex SHA-256 of the exact source file bytes (verified in `assert_optimization_benchmark_fixture` and schema_docs). Recompute from disk — do not trust transcription:

```bash
sha256sum crates/kali_cli/tests/fixtures/benchmarks/string-concatenation-benchmark-v1.ts crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1.ts crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1-js.js
```

Expected (if your files match this plan byte-for-byte; trailing newline included):
- `string-concatenation-benchmark-v1.ts` → `03c716967dd876bda179b3dfeff8974a7a78f78cccc0b3c2347561b1963f16bb`
- both template files → `3b9e6cd18be429b419668ce4fd3bec22d04cd4314e8073196d178affe12b3058` (identical bytes → identical hash, as before)

Then in each `.json`, replace only the `sourceSha256` line:
- `string-concatenation-benchmark-v1.json`: `"sourceSha256": "sha256-827e1e2cdc791e0e32b02216d49314faa0420aa0115a3deaff749a9727659081",` → `"sourceSha256": "sha256-03c716967dd876bda179b3dfeff8974a7a78f78cccc0b3c2347561b1963f16bb",`
- `template-literal-concatenation-benchmark-v1.json` and `template-literal-concatenation-benchmark-v1-js.json`: `"sourceSha256": "sha256-f1fb47871cbef634d9685539a76071b7f836220a31747d9c46659f026a39d6e0",` → `"sourceSha256": "sha256-3b9e6cd18be429b419668ce4fd3bec22d04cd4314e8073196d178affe12b3058",`

All other metadata fields (`benchmark`, `version`, `sourceFile`, `buildModes`) stay.

- [ ] **Step 5: Add the two CLBG slugs to the schema_docs lists**

In `crates/kali_cli/tests/schema_docs/misc.rs` (both lists are `BTreeSet`s, so position is cosmetic — append at the end):

In `expected_benchmark_names` (~line 2073), replace the final entry:

```rust
        "nullish-specialization",
    ]
```

with:

```rust
        "nullish-specialization",
        "fannkuch-redux",
        "spectral-norm",
    ]
```

In `expected_benchmark_sources` (~line 2140), replace the final entry:

```rust
        "nullish-benchmark-v1.ts",
    ]
```

with:

```rust
        "nullish-benchmark-v1.ts",
        "fannkuch-redux-benchmark-v1.ts",
        "spectral-norm-benchmark-v1.ts",
    ]
```

(The slugs/sources come verbatim from the checked-in `fannkuch-redux-benchmark-v1.json` / `spectral-norm-benchmark-v1.json` added by c15f29f91 / ba19d224b.)

- [ ] **Step 6: Run the three tests**

Run: `cargo test -p kali_cli --test runtime_smoke -- optimization_benchmark_suite`
Expected: PASS (all ~61 fixtures build in `--fast`/`--release`/`--release-advanced` again; the sha256 asserts pass).
Run: `cargo test -p kali_cli --test runtime_smoke -- check_discovers_fixture_tree`
Expected: PASS — stdout still `Checked 65 file(s)` (no files added or removed).
Run: `cargo test -p kali_cli --test schema_docs`
Expected: PASS, 0 failed (both list asserts and the per-fixture sha256 recomputation).

- [ ] **Step 7: Verify the pinned CLBG outputs did not change**

Run: `cargo test -p kali_cli --test clbg_fannkuch_runtime --test clbg_spectral_norm_runtime`
Expected: PASS — their pinned canonical outputs are untouched by this task (only test-list bookkeeping changed).

- [ ] **Step 8: Format and commit**

```bash
cargo fmt --all
git add crates/kali_cli/tests/fixtures/benchmarks/string-concatenation-benchmark-v1.ts crates/kali_cli/tests/fixtures/benchmarks/string-concatenation-benchmark-v1.json crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1.ts crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1.json crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1-js.js crates/kali_cli/tests/fixtures/benchmarks/template-literal-concatenation-benchmark-v1-js.json crates/kali_cli/tests/schema_docs/misc.rs
git commit -m "test(cli): literal-root benchmark concat fixtures; enroll fannkuch/spectral slugs

The string/template concatenation benchmark fixtures routed their folded
literal chain through a string-typed 'folded' const, which the sound E3200
guard (db1463d51 trio) now rejects — aborting the optimization benchmark
suite at fixture 54 and the fixture-tree check walk. Inline the fold into
the hot return (same measured fold shape, no variable operand) and update
the three sourceSha256 pins to the new file bytes. Also add the
fannkuch-redux / spectral-norm slugs and source files vendored by
c15f29f91/ba19d224b to the hardcoded schema_docs corpus lists. Fixture-tree
count unchanged (Checked 65 file(s)); CLBG pinned outputs unchanged."
```

---

### Task 9: Final verification — baseline fully green (fixes the remaining 0; proves all 138)

**Files:** none (verification only; fix-forward only if a failure appears).

- [ ] **Step 1: Full gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli --no-fail-fast 2>&1 | grep -E "^test result|Running"`
Expected: **every** test binary reports `0 failed`. In particular the five kali_cli suites that were failing (`runtime_smoke`, `schema_docs`, `browser_math_atan2_trailing_argument_evaluation_{bundle,harness}`, `browser_math_unsupported_member_calls_harness_jsx_tsx`) and every previously-green binary. runtime_smoke total should be its former 1696 passing + the re-shaped 138, minus nothing.

- [ ] **Step 2: Parser crate spot-check (outside the mandated gate but touched by Tasks 1 and 4)**

Run: `cargo test -p kali_parser -p kali_hir -p kali_mir -p kali_lir`
Expected: PASS, 0 failed.

- [ ] **Step 3: Format check and clean tree**

Run: `cargo fmt --all -- --check`
Expected: no output, exit 0.
Run: `git status --porcelain`
Expected: empty (all work committed on `fix/stale-browser-bundle-expectations`; nothing pushed).

- [ ] **Step 4: If anything failed** — do NOT re-pin the failing expectation. Identify which task introduced the change (`git log --oneline`), reproduce the failing test in isolation, and fix the code (or escalate) so the previously-passing pin holds. The only acceptable end state is Step 1 fully green.
