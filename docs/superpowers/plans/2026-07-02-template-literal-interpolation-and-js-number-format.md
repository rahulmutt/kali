# Template-Literal Interpolation + Exact JS Number Formatting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close both followups from the float-console work: (1) interpolated template literals currently print their raw source at runtime (`` console.log(`v: ${7 / 2}`) `` prints `v: ${7 / 2}`); (2) the wasmtime host's `format_js_number` diverges from JS `String(number)` for small magnitudes (`1 / 10000000` prints `0.0000001` on the host but `1e-7` in a browser).

**Architecture:** (1) The lexer emits an entire backtick span as one `TokenType::Template` token and the parser collapses it into a plain string `Literal` carrying raw text (`kali_parser/src/expression/primary.rs:67-71`); nothing downstream ever interpolates. Fix: desugar in the parser — when a template token contains `${`, split it into quasi/expression segments (new public splitter in `kali_common`, shared with the existing `resolve_interpolated_template_literal`) and build a left-associated string-`+` chain. Quasis are carried as backtick-delimited string literals (a quasi can never contain a backtick — the lexer would have ended the token). Everything downstream already works: codegen's string-`+` path stringifies via `emit_as_string` → `float_to_string`/`int_to_string` → `string_concat`. No new host imports; the four hand-mirrored JS import lists are untouched. (2) Replace `format!("{value}")` in `format_js_number` with the `ryu-js` crate (exact ECMA-262 Number→String), keeping the NaN/Infinity/±0 guards.

**Tech Stack:** Rust (kali_common, kali_error, kali_parser, kali_runtime, kali_cli tests), `ryu-js` 1.0, node (differential + bundle harness lanes).

## Global Constraints

- NO new host imports and NO edits to the four hand-mirrored JS import lists (`cmd_build.rs` ESM + CJS, `harness.rs` ×2) — nothing in this plan changes the wasm import surface. If you find yourself editing those files, stop: you are off-plan.
- Desugar rule (from the spec): the leading quasi is ALWAYS emitted as a string literal, even when empty, so the chain is string-valued from its first operand. Later empty quasis are skipped. Templates without `${` keep today's plain-literal path byte-for-byte.
- Quasi literals are backtick-delimited (`` `...` ``) `LiteralValue::String` values — string literal token values in this codebase carry their delimiters (see `lex_string` pushing the quote into `value`), and `strip_string_delimiters`/`is_string_valued` already accept backticks.
- New parser diagnostic code: `e2::MALFORMED_TEMPLATE_INTERPOLATION = 2004` (next free in the E2000-2099 syntax-error block).
- `ryu-js = "1.0"` goes in `[workspace.dependencies]` of the root `Cargo.toml`; `kali_runtime` references it with `{ workspace = true }`. Commit the `Cargo.lock` change with it.
- Known pre-existing lexer limitation, OUT OF SCOPE: `lex_template` ends the token at the first closing backtick, so a nested template inside `${...}` truncates the outer token. Do not add nested-template tests; do not fix the lexer.
- `cargo test --workspace` is NOT a usable gate (pre-existing chromium-sandbox failures). Gates are the named per-task lanes.
- Repo hygiene before every commit: `cargo fmt` produces no diff; `cargo clippy -p <touched crates> --tests -- -D warnings` clean.
- Regression pins that MUST stay green (they exercise the static template paths the desugar changes the AST shape for): `for_of_template_literal_string_iteration_ts_input`, `..._js_input`, `browser_template_literal_string_iteration_harness`, `..._bundle`, `browser_template_literal_dynamic_import_harness`, the `runtime_smoke` dynamic-import template-specifier tests, and all `kali_fmt`/`kali_types` tests. If any regresses, fix the desugar (or the static resolver's handling of the desugared shape) — do NOT edit those tests.

---

### Task 1: `split_template_literal` in kali_common + refactor `resolve_interpolated_template_literal`

**Files:**
- Modify: `crates/kali_common/src/template.rs`
- Create: `crates/kali_common/src/template_tests.rs`

**Interfaces:**
- Consumes: existing private `find_template_expression_end(text: &str, start: usize) -> Option<usize>` (already in `template.rs`; delimiter-and-nesting-aware scan that returns the index of the closing `}`).
- Produces: `pub struct TemplateLiteralSegments { pub quasis: Vec<String>, pub expressions: Vec<String> }` with invariant `quasis.len() == expressions.len() + 1`, and `pub fn split_template_literal(text: &str) -> Option<TemplateLiteralSegments>` — `None` when `text` is not backtick-delimited or an interpolation is unterminated. Task 2's parser desugar calls `split_template_literal` via `kali_common::template::split_template_literal`.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_common/src/template_tests.rs`:

```rust
use super::*;

#[test]
fn split_returns_none_for_non_backtick_text() {
    assert!(split_template_literal("\"v: ${x}\"").is_none());
    assert!(split_template_literal("plain").is_none());
}

#[test]
fn split_handles_template_without_interpolation() {
    let segments = split_template_literal("`hello`").expect("split");
    assert_eq!(segments.quasis, vec!["hello".to_string()]);
    assert!(segments.expressions.is_empty());
}

#[test]
fn split_extracts_quasis_and_expressions() {
    let segments = split_template_literal("`v: ${7 / 2} end`").expect("split");
    assert_eq!(segments.quasis, vec!["v: ".to_string(), " end".to_string()]);
    assert_eq!(segments.expressions, vec!["7 / 2".to_string()]);
}

#[test]
fn split_handles_adjacent_interpolations_and_edges() {
    let segments = split_template_literal("`${a}${b}`").expect("split");
    assert_eq!(
        segments.quasis,
        vec!["".to_string(), "".to_string(), "".to_string()]
    );
    assert_eq!(segments.expressions, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn split_respects_nested_braces_and_strings_in_expressions() {
    let segments = split_template_literal("`v: ${fn({ a: '}' })}`").expect("split");
    assert_eq!(segments.quasis, vec!["v: ".to_string(), "".to_string()]);
    assert_eq!(segments.expressions, vec!["fn({ a: '}' })".to_string()]);
}

#[test]
fn split_returns_none_for_unterminated_interpolation() {
    assert!(split_template_literal("`v: ${7`").is_none());
}

#[test]
fn resolve_still_renders_via_segments() {
    let rendered =
        resolve_interpolated_template_literal("`v: ${x} end`", |segment| {
            (segment == "x").then(|| "3.5".to_string())
        })
        .expect("render");
    assert_eq!(rendered, "v: 3.5 end");
}

#[test]
fn resolve_still_passes_through_plain_templates() {
    let rendered = resolve_interpolated_template_literal("`hello`", |_| None).expect("render");
    assert_eq!(rendered, "hello");
}
```

Hook the module at the end of `crates/kali_common/src/template.rs`:

```rust
#[cfg(test)]
#[path = "template_tests.rs"]
mod template_tests;
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p kali_common template`
Expected: compile FAIL — `split_template_literal` and `TemplateLiteralSegments` not found.

- [ ] **Step 3: Implement the splitter and refactor the resolver on top of it**

In `crates/kali_common/src/template.rs`, replace the body of `resolve_interpolated_template_literal` and add the new items ABOVE `find_template_expression_end` (which stays exactly as it is):

```rust
/// An interpolated template literal split into its literal chunks and the raw
/// source of each `${...}` expression. Invariant: `quasis.len() ==
/// expressions.len() + 1` (leading and trailing quasis may be empty).
pub struct TemplateLiteralSegments {
    pub quasis: Vec<String>,
    pub expressions: Vec<String>,
}

/// Splits a backtick-delimited template literal (delimiters included in
/// `text`) into quasis and raw `${...}` expression sources. Returns `None`
/// when `text` is not backtick-delimited or an interpolation has no closing
/// `}`.
pub fn split_template_literal(text: &str) -> Option<TemplateLiteralSegments> {
    let trimmed = text.trim();
    let inner = trimmed
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))?;

    let mut quasis = Vec::new();
    let mut expressions = Vec::new();
    let mut quasi = String::new();
    let mut index = 0usize;
    while index < inner.len() {
        let Some(relative) = inner[index..].find("${") else {
            quasi.push_str(&inner[index..]);
            break;
        };

        let chunk_start = index + relative;
        quasi.push_str(&inner[index..chunk_start]);
        quasis.push(std::mem::take(&mut quasi));

        let expression_start = chunk_start + 2;
        let expression_end = find_template_expression_end(inner, expression_start)?;
        expressions.push(inner[expression_start..expression_end].to_string());
        index = expression_end + 1;
    }
    quasis.push(quasi);

    Some(TemplateLiteralSegments {
        quasis,
        expressions,
    })
}

pub fn resolve_interpolated_template_literal(
    text: &str,
    mut resolve_expression: impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    let segments = split_template_literal(text)?;
    let mut rendered = segments.quasis[0].clone();
    for (expression, quasi) in segments.expressions.iter().zip(&segments.quasis[1..]) {
        rendered.push_str(&resolve_expression(expression)?);
        rendered.push_str(quasi);
    }
    Some(rendered)
}
```

Delete the old body of `resolve_interpolated_template_literal` (the manual `while index < inner.len()` render loop) — the function signature is unchanged, so no caller changes.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p kali_common`
Expected: PASS (new template tests + all existing kali_common tests).

Then run the resolver's downstream users to prove the refactor is behavior-preserving:

Run: `cargo test -p kali_types && cargo test -p kali_cli --test for_of_template_literal_string_iteration_ts_input --test for_of_template_literal_string_iteration_js_input`
Expected: PASS, no changes.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/kali_common/src/template.rs crates/kali_common/src/template_tests.rs
git commit -m "feat(common): public split_template_literal; resolve_interpolated_template_literal reuses it"
```

---

### Task 2: Parser desugars interpolated templates to string-`+` chains (+ new e2 diagnostic)

**Files:**
- Modify: `crates/kali_error/src/_error_codes.rs` (e2 block, currently ends at `DUPLICATE_ITEM: u16 = 2003`)
- Modify: `crates/kali_parser/src/expression/primary.rs:67-71` (the `StringLiteral | Template | Backtick` arm) + new helper methods
- Modify: `crates/kali_parser/src/expression/primary_tests.rs`

**Interfaces:**
- Consumes: `kali_common::template::split_template_literal(text: &str) -> Option<TemplateLiteralSegments>` from Task 1 (`kali_parser` already depends on `kali_common`); `kali_lexer::Lexer::new(FileId, String)` + `lex_all() -> LexerResult { tokens, diagnostics }`; `Parser::new(FileId, Vec<Token>)` + `pub(crate) fn parse_expression(&mut self) -> Expression` (`expression/mod.rs:16`); `kali_ast::BinaryExpression { operator: String, left: Expression, right: Expression }` wrapped as `Expression::BinaryExpression(Box<...>)`.
- Produces: interpolated `Template`/`Backtick` tokens parse to a left-associated `+` `BinaryExpression` chain whose string operands are backtick-delimited `LiteralValue::String` literals. Diagnostic `e2::MALFORMED_TEMPLATE_INTERPOLATION` (2004) on unterminated `${` or empty `${}`. Tasks 3+ rely only on this observable AST shape.

- [ ] **Step 1: Add the error code**

In `crates/kali_error/src/_error_codes.rs`, in the `pub mod e2` block, directly under `pub const DUPLICATE_ITEM: u16 = 2003;`:

```rust
    pub const MALFORMED_TEMPLATE_INTERPOLATION: u16 = 2004;
```

- [ ] **Step 2: Write the failing parser tests**

Append to `crates/kali_parser/src/expression/primary_tests.rs` (test style matches the file's existing `lex` + `Parser::new` pattern):

```rust
fn parse_single_init_expression(source: &str) -> (Expression, Vec<kali_error::diagnostic::Diagnostic>) {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1, "statements: {:?}", output.statements);
    let Statement::VariableDeclaration(vd) = &output.statements[0] else {
        panic!("Expected VariableDeclaration, got {:?}", output.statements[0]);
    };
    (
        vd.declarations[0].init.clone().expect("initializer"),
        output.diagnostics,
    )
}

fn expect_string_literal(expression: &Expression, expected: &str) {
    match expression {
        Expression::Literal(kali_ast::LiteralValue::String(value)) => {
            assert_eq!(value, expected)
        }
        other => panic!("Expected string literal {expected:?}, got {other:?}"),
    }
}

fn expect_plus(expression: &Expression) -> (&Expression, &Expression) {
    match expression {
        Expression::BinaryExpression(expr) if expr.operator == "+" => (&expr.left, &expr.right),
        other => panic!("Expected `+` chain, got {other:?}"),
    }
}

#[test]
fn test_interpolated_template_desugars_to_string_plus_chain() {
    let (init, diagnostics) = parse_single_init_expression("const m = `v: ${7 / 2} end`;");
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
    // ((`v: ` + (7 / 2)) + ` end`)
    let (left, right) = expect_plus(&init);
    expect_string_literal(right, "` end`");
    let (quasi, division) = expect_plus(left);
    expect_string_literal(quasi, "`v: `");
    match division {
        Expression::BinaryExpression(expr) => {
            assert_eq!(expr.operator, "/");
            assert_eq!(
                expr.left,
                Expression::Literal(kali_ast::LiteralValue::Number(7.0))
            );
            assert_eq!(
                expr.right,
                Expression::Literal(kali_ast::LiteralValue::Number(2.0))
            );
        }
        other => panic!("Expected division, got {other:?}"),
    }
}

#[test]
fn test_adjacent_interpolations_get_leading_empty_quasi_and_skip_empty_rest() {
    let (init, diagnostics) = parse_single_init_expression("const m = `${a}${b}`;");
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
    // ((`` + a) + b) — leading empty quasi kept, later empty quasis skipped
    let (left, right) = expect_plus(&init);
    assert_eq!(*right, Expression::Identifier("b".to_string()));
    let (quasi, a) = expect_plus(left);
    expect_string_literal(quasi, "``");
    assert_eq!(*a, Expression::Identifier("a".to_string()));
}

#[test]
fn test_template_without_interpolation_stays_plain_literal() {
    let (init, diagnostics) = parse_single_init_expression("const m = `hello`;");
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
    expect_string_literal(&init, "`hello`");
}

#[test]
fn test_unterminated_interpolation_reports_e2004_and_falls_back_to_raw() {
    let (init, diagnostics) = parse_single_init_expression("const m = `v: ${7`;");
    assert!(
        diagnostics.iter().any(|d| d.code
            == Some(kali_error::_error_codes::e2::MALFORMED_TEMPLATE_INTERPOLATION as u32)),
        "diagnostics: {diagnostics:?}"
    );
    expect_string_literal(&init, "`v: ${7`");
}

#[test]
fn test_empty_interpolation_reports_e2004() {
    let (init, diagnostics) = parse_single_init_expression("const m = `v: ${}`;");
    assert!(
        diagnostics.iter().any(|d| d.code
            == Some(kali_error::_error_codes::e2::MALFORMED_TEMPLATE_INTERPOLATION as u32)),
        "diagnostics: {diagnostics:?}"
    );
    // Desugared shape still string-valued: (`v: ` + ``)
    let (quasi, empty) = expect_plus(&init);
    expect_string_literal(quasi, "`v: `");
    expect_string_literal(empty, "``");
}
```

(`Diagnostic.code` is `Option<u32>` — `kali_error/src/diagnostic.rs:32` — hence the `Some(...)` comparisons. `Expression` derives `PartialEq`, so the `assert_eq!` comparisons on subexpressions compile as written.)

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p kali_parser primary_tests`
Expected: FAIL — the first, second, fourth, and fifth new tests fail (template currently parses to a raw string literal); `test_template_without_interpolation_stays_plain_literal` may already pass.

- [ ] **Step 4: Implement the desugar**

In `crates/kali_parser/src/expression/primary.rs`, add imports at the top:

```rust
use kali_common::template::split_template_literal;
use kali_error::{_error_codes::e2, diagnostic::Diagnostic};
```

Replace the arm at lines 67-71:

```rust
            TokenType::StringLiteral | TokenType::Template | TokenType::Backtick => {
                let token = self.stream.advance();
                let value = token.map(|t| t.value).unwrap_or_default();
                if matches!(kind, TokenType::Template | TokenType::Backtick)
                    && value.contains("${")
                {
                    return self.desugar_template_literal(&value);
                }
                Expression::Literal(kali_ast::LiteralValue::String(value))
            }
```

Add the helpers to the same `impl Parser` block (below `parse_primary_expression`):

```rust
    /// Desugars an interpolated template literal token into a left-associated
    /// string `+` chain: `` `v: ${7 / 2}` `` becomes `"v: " + (7 / 2)`.
    /// Quasis are carried as backtick-delimited string literals — a quasi can
    /// never contain a backtick, so the delimiter is unambiguous and the
    /// literal takes the plain-template path everywhere downstream. The
    /// leading quasi is always emitted (even empty) so the chain is
    /// string-valued from its first operand; later empty quasis are skipped.
    fn desugar_template_literal(&mut self, raw: &str) -> Expression {
        let Some(segments) = split_template_literal(raw) else {
            self.diagnostics.push(Diagnostic::error(
                e2::MALFORMED_TEMPLATE_INTERPOLATION as u32,
                "Unterminated `${` interpolation in template literal",
            ));
            return Expression::Literal(kali_ast::LiteralValue::String(raw.to_string()));
        };

        fn quasi_literal(quasi: &str) -> Expression {
            Expression::Literal(kali_ast::LiteralValue::String(format!("`{quasi}`")))
        }
        fn concat(left: Expression, right: Expression) -> Expression {
            Expression::BinaryExpression(Box::new(kali_ast::BinaryExpression {
                operator: "+".to_string(),
                left,
                right,
            }))
        }

        let mut chain = quasi_literal(&segments.quasis[0]);
        for (expression_source, quasi) in
            segments.expressions.iter().zip(&segments.quasis[1..])
        {
            let expression = self.parse_template_expression_segment(expression_source);
            chain = concat(chain, expression);
            if !quasi.is_empty() {
                chain = concat(chain, quasi_literal(quasi));
            }
        }
        chain
    }

    /// Parses the raw source of one `${...}` segment with a sub-lexer and
    /// sub-parser, merging their diagnostics into this parser. Spans inside
    /// the segment are relative to the segment text, matching the precedent
    /// set by `resolve_static_string_from_source` in kali_types.
    fn parse_template_expression_segment(&mut self, source: &str) -> Expression {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                e2::MALFORMED_TEMPLATE_INTERPOLATION as u32,
                "Empty `${}` interpolation in template literal",
            ));
            return Expression::Literal(kali_ast::LiteralValue::String("``".to_string()));
        }
        let lexed = kali_lexer::Lexer::new(self.file_id, trimmed.to_string()).lex_all();
        self.diagnostics.extend(lexed.diagnostics);
        let mut sub_parser = Parser::new(self.file_id, lexed.tokens);
        let expression = sub_parser.parse_expression();
        self.diagnostics.extend(sub_parser.diagnostics);
        expression
    }
```

(`kali_lexer::Lexer` may need adding to the `use kali_lexer::...` import line if not already imported in this file — currently it imports only `TokenType`.)

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_parser`
Expected: PASS (all, including the 5 new tests and the existing parser integration suite).

- [ ] **Step 6: Run the downstream regression pins**

```bash
cargo test -p kali_types
cargo test -p kali_fmt
cargo test -p kali_hir
cargo test -p kali_cli --test for_of_template_literal_string_iteration_ts_input
cargo test -p kali_cli --test for_of_template_literal_string_iteration_js_input
cargo test -p kali_cli --test browser_template_literal_string_iteration_harness
cargo test -p kali_cli --test browser_template_literal_string_iteration_bundle
cargo test -p kali_cli --test browser_template_literal_dynamic_import_harness
cargo test -p kali_cli --test runtime_smoke -- template
cargo test -p kali_cli --test imperative_core_runtime
```

Expected: ALL PASS. The static resolvers already handle the desugared shape (`resolve_static_string_iterable_expression` has a `BinaryExpression "+"` arm at `kali_types/src/static_analysis/string.rs:12-16`; `resolve_static_string_expression` strips a backtick quasi via its `resolve_interpolated_template_literal` arm). If a pin regresses, the desugared AST shape is not resolving through one of those arms — fix the resolver arm or the desugar, never the test, and record what changed in the commit message.

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add crates/kali_error/src/_error_codes.rs crates/kali_parser/src/expression/primary.rs crates/kali_parser/src/expression/primary_tests.rs
git commit -m "feat(parser): desugar interpolated template literals to string-+ chains (E2004 on malformed \${})"
```

---

### Task 3: End-to-end interpolation tests (host runtime + browser bundle) + close the old spec's out-of-scope note

**Files:**
- Create: `crates/kali_cli/tests/template_literal_interpolation_runtime.rs`
- Create: `crates/kali_cli/tests/browser_bundle_template_literal_interpolation.rs`
- Modify: `crates/kali_fmt/src/tests.rs` (one idempotence test)
- Modify: `docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md` (final paragraph)

**Interfaces:**
- Consumes: the desugared parse from Task 2; existing codegen string-`+`/`float_to_string` machinery (unchanged); `kali_runtime::browser_bundle_harness_script` and `kali_runtime::browser_harness_command_parts_for` (same helpers `browser_bundle_float_console.rs` uses).
- Produces: pinned runtime behavior — interpolated templates print interpolated values on both lanes.

- [ ] **Step 1: Write the host runtime test**

Create `crates/kali_cli/tests/template_literal_interpolation_runtime.rs`:

```rust
//! Interpolated template literals evaluate their `${...}` expressions at
//! runtime with string-`+` semantics (floats via `float_to_string`).
//! Regression for templates printing their raw source text.
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_fixture(source: &str) -> (bool, String, String) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn run_interpolates_float_expressions() {
    let (ok, stdout, stderr) =
        run_fixture("console.log(`v: ${7 / 2}`);\nconst x = 7 / 2;\nconsole.log(`x=${x}`);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "v: 3.5\nx=3.5\n");
}

#[test]
fn run_interpolates_ints_strings_and_adjacent_segments() {
    let (ok, stdout, stderr) = run_fixture(
        "console.log(`${1}${2}`);\nconsole.log(`sum: ${1 + 2}`);\nconst name = \"kali\";\nconsole.log(`hi ${name}!`);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "12\nsum: 3\nhi kali!\n");
}

#[test]
fn run_keeps_plain_templates_unchanged() {
    let (ok, stdout, stderr) = run_fixture("console.log(`hello`);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "hello\n");
}

#[test]
fn run_interpolates_inside_functions() {
    let (ok, stdout, stderr) = run_fixture(
        "function show(v) {\n  console.log(`p: ${v}`);\n}\nshow(9 / 2);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "p: 4.5\n");
}
```

- [ ] **Step 2: Run to verify (should pass already — Tasks 1-2 delivered the behavior)**

Run: `cargo test -p kali_cli --test template_literal_interpolation_runtime`
Expected: PASS. If any case FAILS, stop and fix the emit path before proceeding (most likely a `render_static_value` fast path swallowing the chain) — do not weaken the assertions.

- [ ] **Step 3: Write the browser bundle test**

Create `crates/kali_cli/tests/browser_bundle_template_literal_interpolation.rs` (pattern copied from `browser_bundle_float_console.rs`):

```rust
//! Interpolated template literals print interpolated values through the
//! browser bundle glue under the node harness.
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn browser_bundle_interpolates_template_literals_via_start() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "console.log(`v: ${7 / 2}`);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let harness_path = dir.path().join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.start();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
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
    assert!(stdout.contains("v: 3.5\n"), "stdout: {stdout:?}");
}
```

- [ ] **Step 4: Run the bundle test**

Run: `cargo test -p kali_cli --test browser_bundle_template_literal_interpolation`
Expected: PASS (node harness lane).

- [ ] **Step 5: Pin formatter idempotence on an interpolated template**

The formatter is token-based (`formatter.rs:102` emits `Template` tokens raw), so the desugar cannot affect it — pin that. Append to `crates/kali_fmt/src/tests.rs`:

```rust
#[test]
fn interpolated_template_literals_format_idempotently() {
    let source = "console.log(`v: ${7 / 2} end`);\n";
    let once = format_source(source);
    let twice = format_source(&once);
    assert_eq!(once, twice);
    assert!(once.contains("`v: ${7 / 2} end`"), "formatted: {once:?}");
}
```

Run: `cargo test -p kali_fmt`
Expected: PASS.

- [ ] **Step 6: Close the old spec's out-of-scope note**

In `docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md`, replace the final paragraph:

```markdown
**Recorded, out of scope:** template literals currently print their raw source
(`` console.log(`v: ${7 / 2}`) `` prints `v: ${7 / 2}`) — discovered during
this investigation; separate issue.
```

with:

```markdown
**Recorded, out of scope (since closed):** template literals printed their raw
source (`` console.log(`v: ${7 / 2}`) `` printed `v: ${7 / 2}`) — fixed by the
parser desugar in `2026-07-02-template-literal-interpolation-and-js-number-format-design.md`.
```

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add crates/kali_cli/tests/template_literal_interpolation_runtime.rs crates/kali_cli/tests/browser_bundle_template_literal_interpolation.rs crates/kali_fmt/src/tests.rs docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md
git commit -m "test(cli,fmt): pin runtime template-literal interpolation on host and browser-bundle lanes"
```

---

### Task 4: `format_js_number` delegates to ryu-js (exact JS `String(number)`)

**Files:**
- Modify: `/workspace/Cargo.toml` (`[workspace.dependencies]`, external block)
- Modify: `crates/kali_runtime/Cargo.toml`
- Modify: `crates/kali_runtime/src/host/imports_default.rs:820-838` (rustdoc + `format_js_number`)
- Create: `crates/kali_runtime/src/host/imports_default_tests.rs`
- Modify (generated): `Cargo.lock`

**Interfaces:**
- Consumes: `ryu_js::Buffer::new().format_finite(value: f64) -> &str` (crate `ryu-js` 1.0.2, imported as `ryu_js`; `format_finite` requires a finite value — the existing guards guarantee that).
- Produces: `format_js_number(f64) -> String` byte-identical to JS `String(value)` for all doubles. No signature change; the `float_to_string` host import at `imports_default.rs:663` picks it up unchanged.

- [ ] **Step 1: Add the dependency**

In `/workspace/Cargo.toml`, `# External dependencies` block (alphabetical placement is not enforced; append near `once_cell`):

```toml
ryu-js = "1.0"
```

In `crates/kali_runtime/Cargo.toml` `[dependencies]`, after `url = { workspace = true }`:

```toml
ryu-js = { workspace = true }
```

Run: `cargo build -p kali_runtime`
Expected: compiles; `Cargo.lock` gains `ryu-js` v1.0.x.

- [ ] **Step 2: Write the failing tests**

Create `crates/kali_runtime/src/host/imports_default_tests.rs`:

```rust
use super::format_js_number;

#[test]
fn formats_specials_and_zero() {
    assert_eq!(format_js_number(f64::NAN), "NaN");
    assert_eq!(format_js_number(f64::INFINITY), "Infinity");
    assert_eq!(format_js_number(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(format_js_number(0.0), "0");
    assert_eq!(format_js_number(-0.0), "0");
}

#[test]
fn formats_ordinary_magnitudes_with_shortest_round_trip() {
    assert_eq!(format_js_number(3.5), "3.5");
    assert_eq!(format_js_number(0.1), "0.1");
    assert_eq!(format_js_number(0.30000000000000004), "0.30000000000000004");
    assert_eq!(format_js_number(-42.0), "-42");
}

#[test]
fn formats_small_magnitudes_at_the_js_exponent_threshold() {
    // JS keeps decimal notation down to 1e-6 and switches at 1e-7.
    assert_eq!(format_js_number(1e-6), "0.000001");
    assert_eq!(format_js_number(1e-7), "1e-7");
    assert_eq!(format_js_number(1.0 / 10000000.0), "1e-7");
    assert_eq!(format_js_number(-1e-7), "-1e-7");
}

#[test]
fn formats_large_magnitudes_at_the_js_exponent_threshold() {
    // JS keeps decimal notation up to (excluding) 1e21, with a '+' sign above.
    assert_eq!(format_js_number(1e20), "100000000000000000000");
    assert_eq!(format_js_number(1e21), "1e+21");
    assert_eq!(format_js_number(-1e21), "-1e+21");
    assert_eq!(format_js_number(5e-324), "5e-324");
    assert_eq!(format_js_number(f64::MAX), "1.7976931348623157e+308");
}

/// Differential pin against node's native `String(value)`. Values cross the
/// wire as exact f64 bit patterns so no text round-trip can mask a mismatch.
/// Node is a hard requirement of the harness lanes, so no skip-if-missing.
#[test]
fn matches_node_string_conversion() {
    let values: &[f64] = &[
        3.5,
        0.1,
        0.30000000000000004,
        1e-6,
        1e-7,
        1.0 / 10000000.0,
        -1e-7,
        0.000001234,
        1e20,
        1e21,
        -1e21,
        123456789.123,
        1.5e300,
        5e-324,
        f64::MAX,
        f64::MIN_POSITIVE,
    ];
    let bits: Vec<String> = values
        .iter()
        .map(|value| format!("{:#x}n", value.to_bits()))
        .collect();
    let script = format!(
        "for (const bits of [{}]) {{ const v = new Float64Array(new BigUint64Array([bits]).buffer)[0]; process.stdout.write(String(v) + \"\\n\"); }}",
        bits.join(",")
    );
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(&script)
        .output()
        .expect("node available (required by the harness lanes)");
    assert!(
        output.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let node_lines: Vec<&str> = stdout.trim_end().split('\n').collect();
    assert_eq!(node_lines.len(), values.len());
    for (value, expected) in values.iter().zip(node_lines) {
        assert_eq!(
            format_js_number(*value),
            expected,
            "bits {:#x}",
            value.to_bits()
        );
    }
}
```

Hook the module at the end of `crates/kali_runtime/src/host/imports_default.rs`:

```rust
#[cfg(test)]
#[path = "imports_default_tests.rs"]
mod imports_default_tests;
```

- [ ] **Step 3: Run tests to verify the new expectations fail on the current implementation**

Run: `cargo test -p kali_runtime imports_default_tests`
Expected: FAIL — `formats_small_magnitudes_at_the_js_exponent_threshold` (current code prints `0.0000001`), `formats_large_magnitudes_at_the_js_exponent_threshold` (current prints Rust notation for `1e21`/`f64::MAX`), and `matches_node_string_conversion`. The specials/ordinary tests pass.

- [ ] **Step 4: Implement**

In `crates/kali_runtime/src/host/imports_default.rs`, replace the function and its rustdoc (currently lines 820-838, doc mentioning the known divergence):

```rust
/// JS `String(number)` semantics: `NaN`, `Infinity`, `-Infinity`, `0` for
/// ±0, and the ECMA-262 Number-to-String algorithm (via `ryu-js`) for every
/// other double — byte-identical to the JS glue mirrors' native
/// `String(value)`, including exponent notation for |x| >= 1e21 and
/// magnitudes below 1e-6.
fn format_js_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    ryu_js::Buffer::new().format_finite(value).to_owned()
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p kali_runtime`
Expected: PASS (all five new tests + entire existing kali_runtime suite — the `float_to_fixed` and existing float-console behaviors must be untouched).

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add Cargo.toml Cargo.lock crates/kali_runtime/Cargo.toml crates/kali_runtime/src/host/imports_default.rs crates/kali_runtime/src/host/imports_default_tests.rs
git commit -m "feat(runtime): format_js_number delegates finite doubles to ryu-js for exact JS String(number)"
```

---

### Task 5: Divergence-closure e2e + docs + whole-plan verification

**Files:**
- Modify: `crates/kali_cli/tests/float_console_runtime.rs` (add one test)
- Modify: `crates/kali_cli/tests/browser_bundle_float_console.rs` (extend fixture + assertions)
- Modify: `docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md` (divergence paragraph)
- Modify: `docs/superpowers/specs/2026-07-02-template-literal-interpolation-and-js-number-format-design.md` (verification note)

**Interfaces:**
- Consumes: Task 4's `format_js_number`; the existing test files' `run_fixture`/harness patterns.
- Produces: pinned proof that host and browser stdout agree on the previously-divergent value.

- [ ] **Step 1: Add the host-lane divergence test**

Append to `crates/kali_cli/tests/float_console_runtime.rs`:

```rust
#[test]
fn run_prints_small_magnitudes_with_js_exponent_notation() {
    // Was the recorded reachable divergence: host printed 0.0000001 while the
    // browser mirrors printed 1e-7.
    let (ok, stdout, stderr) = run_fixture("console.log(1 / 10000000);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "1e-7\n");
}
```

- [ ] **Step 2: Extend the browser-bundle float test to cover the same value**

In `crates/kali_cli/tests/browser_bundle_float_console.rs`, change the fixture write from:

```rust
        "console.log(7 / 2);\nconsole.log(\"v: \" + (0 / 0));\nconsole.log(7 / 0);\n",
```

to:

```rust
        "console.log(7 / 2);\nconsole.log(\"v: \" + (0 / 0));\nconsole.log(7 / 0);\nconsole.log(1 / 10000000);\n",
```

and add alongside the existing stdout assertions:

```rust
    assert!(stdout.contains("1e-7\n"), "stdout: {stdout:?}");
```

- [ ] **Step 3: Run both lanes**

```bash
cargo test -p kali_cli --test float_console_runtime
cargo test -p kali_cli --test browser_bundle_float_console
```

Expected: PASS — identical `1e-7` on host (wasmtime + ryu-js) and browser glue (native `String`).

- [ ] **Step 4: Update the docs**

In `docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md`, replace the sentence beginning `Known divergences (documented on `format_js_number`):` through `...stays in i64).` (spec lines 19-25) with:

```markdown
The formatting divergences originally recorded here (JS exponent notation for
very small magnitudes and for |x| >= 1e21) were closed on 2026-07-02: the host
now delegates finite doubles to `ryu-js` (exact ECMA-262 Number-to-String), so
host and browser stdout agree — see
`2026-07-02-template-literal-interpolation-and-js-number-format-design.md`.
```

In `docs/superpowers/specs/2026-07-02-template-literal-interpolation-and-js-number-format-design.md`, no content change is required unless earlier tasks deviated; if they did, record the deviation in a short **Implementation notes** section appended to the file.

- [ ] **Step 5: Whole-plan verification**

```bash
cargo test -p kali_common
cargo test -p kali_parser
cargo test -p kali_types
cargo test -p kali_fmt
cargo test -p kali_hir
cargo test -p kali_runtime
cargo test -p kali_codegen
cargo test -p kali_cli --test template_literal_interpolation_runtime
cargo test -p kali_cli --test browser_bundle_template_literal_interpolation
cargo test -p kali_cli --test float_console_runtime
cargo test -p kali_cli --test browser_bundle_float_console
cargo test -p kali_cli --test for_of_template_literal_string_iteration_ts_input
cargo test -p kali_cli --test for_of_template_literal_string_iteration_js_input
cargo test -p kali_cli --test browser_template_literal_string_iteration_harness
cargo test -p kali_cli --test browser_template_literal_string_iteration_bundle
cargo test -p kali_cli --test browser_template_literal_dynamic_import_harness
cargo test -p kali_cli --test runtime_smoke -- template
cargo test -p kali_cli --test imperative_core_runtime
cargo test -p kali_cli --test browser_cdp_smoke
cargo clippy -p kali_common -p kali_error -p kali_parser -p kali_runtime -p kali_cli --tests -- -D warnings
cargo fmt
```

Expected: every test lane green; clippy clean; `cargo fmt` produces no diff. If ANY gate is red, STOP and report — do not commit.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/float_console_runtime.rs crates/kali_cli/tests/browser_bundle_float_console.rs docs/superpowers/specs/2026-07-02-float-console-and-reserved-names.md docs/superpowers/specs/2026-07-02-template-literal-interpolation-and-js-number-format-design.md
git commit -m "test(cli),docs(spec): pin 1e-7 host/browser agreement; record divergence closure"
```

---

## Verification (whole plan)

- All Task 5 Step 5 gates green.
- `git status` clean; five commits, each `cargo fmt`-clean.
- Both followups recorded by the float-console spec are closed and the spec text reflects it.
