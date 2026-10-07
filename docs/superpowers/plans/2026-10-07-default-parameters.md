# Default Parameters Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A function declaration whose parameters carry scalar literal defaults compiles, and every direct call of it runs as node runs it; every other default parameter is refused with E5506 under `check` and `run` alike.

**Architecture:** The parser keeps a declaration's defaults on the AST (`FunctionDeclaration::defaults`) and refuses defaults on every other function form. A new AST pass, `kali_cli/src/build/default_params`, runs right after the block-scope rename: it walks the program twice through the shared `block_scope_rename::walk` walker, first collecting defaulted declarations and every use of their names, then (if nothing was refused) appending a fresh clone of each default to every call that omits it and stripping the defaults. Every later stage sees an ordinary fixed-arity function.

**Tech Stack:** Rust (workspace crates `kali_ast`, `kali_parser`, `kali_common`, `kali_cli`), the TOML `cases` runner, node v26.10.0 as the oracle.

**Spec:** `docs/superpowers/specs/2026-10-07-default-parameters-design.md`

## Global Constraints

- Every refusal is `E5506` (`kali_error::_error_codes::e5::FEATURE_UNAVAILABLE`). No new diagnostic code.
- Message texts live in `crates/kali_common/src/messages.rs` and are used verbatim as written in Task 2.
- Rust unit tests go in sibling `*_tests.rs` files wired with `#[cfg(test)] #[path = "…_tests.rs"] mod …;`, never inline test bodies.
- Black-box tests are `.toml` files under `crates/kali_cli/tests/cases/`, run by the single `cases` target. No new `tests/*.rs` targets.
- Every expected stdout in a case file is node v26.10.0's output for the same program, measured with `env -u FORCE_COLOR node FILE.js`.
- Build and test inside the worktree with `CARGO_TARGET_DIR=$PWD/target`, `-j 6`, and `--test-threads=6`. Never put a target directory under `/tmp`.
- Launch any run longer than a few minutes detached: `setsid nohup bash -c '…; echo $? > EXITFILE' > /dev/null 2>&1 < /dev/null &`, then wait for `EXITFILE`.
- `check` and `run` must give the same refusal for every program in this plan.

## Corrections to the spec found while planning

These are carried out by the tasks below and recorded in the spec's amendments section in Task 6:

- **A-1. Where defaults live.** The spec puts `default` on `FunctionParam`. `FunctionParam` is only used by arrows and function expressions; `FunctionDeclaration::params` is a `Vec<String>`. The field is therefore `FunctionDeclaration::defaults: Vec<Option<Box<Expression>>>`, index-aligned with `params`, empty when no parameter has a default.
- **A-2. Where non-declaration defaults are refused.** Because arrows, methods and function expressions never carry defaults on the AST, the parser refuses them directly, with the spec's message "default parameters are only available on function declarations in the current phase". The parser is shared by `check` and `run`, so they still agree.
- **A-3. Where exported defaulted functions are refused.** `export function f(…)` parses to a plain `Statement::FunctionDeclaration`; the export is not visible to the pass. The parser refuses it in `parse_export_declaration`. `export { f }` is visible to the pass and is refused there.
- **A-4. No spans.** AST nodes carry no source spans, so diagnostics name the function and parameter instead of pointing at a span.
- **A-5. Spread arguments.** `f(...xs)` to a defaulted function is refused: the pass cannot count the arguments. New message in Task 2.
- **A-6. `f?.()`.** It parses as `OptionalChainExpression { object: Identifier("f"), optional: true }` and the call and its arguments are dropped (pre-existing, measured 2026-10-07). The pass sees `f` as a value use and refuses it. The dropped call goes in the followups file.
- **A-7. Scalar literals only (human partner's ruling, 2026-10-07).** Kali already refuses an object or array literal passed directly as a call argument (`f({})`: "an object literal passed directly as a call argument is unavailable in the current phase; bind it to a const first"; `f([])`: "passing an array literal to function … is unavailable"). Filling `{}` or `[]` in at the call site would hit that refusal with a message about an argument the user never wrote. The pass therefore admits only number, string, boolean, `null` and BigInt literals and unary `-`/`+` on a number, and refuses an object or array default with its own message. The corpus's one `hooks = {}` (`task_queue.js`) stays blocked.

## Review Focus

1. **A defaulted function called before its declaration** (hoisting, `console.log(f()); function f(a = 1) { return a; }`): a reader expects node's `1`. Pinned in Task 4 (`a_call_before_the_declaration_is_filled`); see the note below.
2. **A defaulted function declared inside another function or a block**: the collector must see nested declarations. Pinned in Task 4 (`a_nested_declaration_is_collected_and_filled`).
3. **An explicit `undefined` in the middle** (`f(1, undefined, 3)`): only the middle argument is replaced. Pinned in Task 4 (`a_middle_literal_undefined_is_replaced`).
4. **A program with no defaults is untouched**, byte for byte, so the pass cannot regress the existing suite. Pinned in Task 4 (`a_program_without_defaults_is_untouched`).
5. **A string default spelled as a template literal without interpolation** (`` b = `x` ``): the parser produces `Literal(String)` for it, so it must be admitted. Pinned in Task 4 (`a_template_string_default_is_a_literal`).

Item 1 is pinned at the pass level only. End to end, kali refuses ANY call before the callee's declaration with `E3100 undefined identifier` at the baseline (measured 2026-10-07 with no defaults involved), so no case file can pin it; the followups file records that.

## File Structure

| file | change | responsibility |
|---|---|---|
| `crates/kali_ast/src/declaration.rs` | modify | `FunctionDeclaration::defaults` field |
| `crates/kali_ast/src/declaration_tests.rs` | modify | the field is absent from JSON when empty |
| every other `FunctionDeclaration { … }` site (33, listed in Task 1) | modify | add `defaults: Vec::new()` or `..` |
| `crates/kali_common/src/messages.rs` | modify | the nine message functions |
| `crates/kali_common/src/messages_tests.rs` | modify | message texts pinned |
| `crates/kali_parser/src/declaration.rs` | modify | scanner keeps defaults; declarations parse them; other forms refuse |
| `crates/kali_parser/src/module.rs` | modify | exported defaulted function refused |
| `crates/kali_parser/src/declaration_tests/default_params.rs` | create | parser tests for defaults |
| `crates/kali_parser/src/declaration_tests.rs` | modify | `mod default_params;` |
| `crates/kali_parser/src/declaration_tests/unsupported_params.rs` | modify | re-pin the three declaration rows |
| `crates/kali_cli/src/build/block_scope_rename/walk.rs` | modify | two new `Hooks` methods with empty defaults |
| `crates/kali_cli/src/build/default_params/mod.rs` | create | the pass: collect, check, fill, strip |
| `crates/kali_cli/src/build/default_params/literal.rs` | create | `DefaultKind`, `classify_default` |
| `crates/kali_cli/src/build/default_params/literal_tests.rs` | create | classifier tests |
| `crates/kali_cli/src/build/default_params/default_params_tests.rs` | create | pass tests |
| `crates/kali_cli/src/build/mod.rs` | modify | `pub mod default_params;` |
| `crates/kali_cli/src/build/compile.rs` | modify | call the pass after the rename |
| `crates/kali_cli/tests/cases/soundness/default_parameters.toml` | create | fills, module and in-function scope |
| `crates/kali_cli/tests/cases/soundness/default_parameters_refused.toml` | create | one refusal per message, `check` and `run` |
| `specs/19-feature-maturity.md` | modify | new row |
| `docs/superpowers/specs/2026-10-07-default-parameters-design.md` | modify | §7 amendments A-1..A-7 |
| `docs/superpowers/followups/default-parameters-discovered-defects.md` | create | residue |

---

### Task 0: Start from merged lane A

Lane A (`numeric-literal-grammar`) edits `kali_parser`. Start from its merge.

- [ ] **Step 1: Confirm lane A is merged and rebase**

```bash
cd /workspace/.worktrees/default-parameters
git fetch --all 2>/dev/null || true
git log --oneline main | grep -m1 numeric-literal-grammar   # must print the merge
git rebase main
git log --oneline -3
```

Expected: the spec commit `docs(default-parameters): design spec …` and this plan sit on top of the lane A merge. If lane A is not merged, STOP and ask the human partner.

- [ ] **Step 2: Baseline build**

```bash
cd /workspace/.worktrees/default-parameters
CARGO_TARGET_DIR=$PWD/target cargo build -j 6 -p kali_cli --bin kali
```

Expected: builds. Then confirm the baseline refusal:

```bash
printf 'function f(a, b = 2) { return a + b; }\nconsole.log(f(1));\n' > "$TMPDIR_SCRATCH/dp.js"
target/debug/kali run "$TMPDIR_SCRATCH/dp.js"
```

(`TMPDIR_SCRATCH` is the session scratchpad directory.) Expected: `error[E5506]: a default parameter is not supported — kali functions take a fixed list of plain named parameters`, exit 1.

---

### Task 1: `FunctionDeclaration::defaults`

**Files:**
- Modify: `crates/kali_ast/src/declaration.rs:9-17`
- Test: `crates/kali_ast/src/declaration_tests.rs`
- Modify (mechanical): every `FunctionDeclaration {` site the compiler reports. At the baseline they are: `crates/kali_parser/src/declaration.rs` (1), `crates/kali_cli/src/build/module_link.rs` (1), `crates/kali_hir/src/lowering/statement.rs` (1, a pattern), `crates/kali_types/src/resolve/mod.rs` (1, a pattern with `..`), `crates/kali_types/src/class_instances/translate.rs` (1), and test files `crates/kali_types/src/resolve/function_tests/generator_functions.rs` (9), `crates/kali_cli/src/build/name_anon_functions_tests.rs` (7), `crates/kali_cli/src/build_tests/collect.rs` (5), `crates/kali_types/src/resolve/expression_tests/exports.rs` (3), `crates/kali_ast/src/declaration_tests.rs` (3), `crates/kali_types/src/resolve/jsx_tests.rs` (1).

**Interfaces:**
- Produces: `kali_ast::FunctionDeclaration { name, params, defaults: Vec<Option<Box<Expression>>>, body, is_async, generator }`. Invariant: `defaults.is_empty() || defaults.len() == params.len()`.

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_ast/src/declaration_tests.rs`:

```rust
#[test]
fn a_function_declaration_without_defaults_serializes_without_the_field() {
    let declaration = FunctionDeclaration {
        name: "f".to_string(),
        params: vec!["a".to_string()],
        defaults: Vec::new(),
        body: Box::new(crate::BlockStatement { body: Vec::new() }),
        is_async: false,
        generator: false,
    };
    let json = serde_json::to_string(&declaration).expect("serializes");
    assert!(!json.contains("defaults"), "{json}");
    let back: FunctionDeclaration = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, declaration);
}

#[test]
fn a_function_declaration_with_a_default_round_trips() {
    let declaration = FunctionDeclaration {
        name: "f".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        defaults: vec![
            None,
            Some(Box::new(crate::Expression::Literal(crate::LiteralValue::Number(2.0)))),
        ],
        body: Box::new(crate::BlockStatement { body: Vec::new() }),
        is_async: false,
        generator: false,
    };
    let json = serde_json::to_string(&declaration).expect("serializes");
    let back: FunctionDeclaration = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, declaration);
}
```

If `serde_json` is not already a dev-dependency of `kali_ast`, check `crates/kali_ast/Cargo.toml`; add `serde_json = { workspace = true }` under `[dev-dependencies]` only if it is missing.

- [ ] **Step 2: Run it to verify it fails**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_ast a_function_declaration_`
Expected: compile error, `struct FunctionDeclaration has no field named defaults`.

- [ ] **Step 3: Add the field**

In `crates/kali_ast/src/declaration.rs`, replace the struct with:

```rust
/// Function declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    /// Default values, index-aligned with `params` (default-parameters spec
    /// §3.1, A-1). Empty when no parameter has a default; otherwise exactly
    /// `params.len()` long. Only the parser fills it, and `kali_cli`'s
    /// `default_params` pass empties it before any later stage runs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub defaults: Vec<Option<Box<Expression>>>,
    pub body: Box<BlockStatement>,
    pub is_async: bool,
    pub generator: bool,
}
```

- [ ] **Step 4: Fix every construction and pattern site**

Run: `CARGO_TARGET_DIR=$PWD/target cargo build -j 6 --workspace --tests 2>&1 | grep -E '^error' -A6`

For each `missing field defaults` error in a construction, add `defaults: Vec::new(),` after `params`. For each `pattern does not mention field defaults` error (e.g. `crates/kali_hir/src/lowering/statement.rs:280`), add `..` at the end of the pattern. In `crates/kali_parser/src/declaration.rs` (`parse_function_declaration_with_async`) add `defaults: Vec::new(),` for now; Task 3 fills it. Repeat until the workspace builds with its tests.

- [ ] **Step 5: Run the tests**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_ast`
Expected: PASS, including the two new tests.

- [ ] **Step 6: Commit**

```bash
git add -A crates
git commit -m "feat(default-parameters): FunctionDeclaration carries index-aligned defaults (spec A-1)"
```

---

### Task 2: Refusal messages

**Files:**
- Modify: `crates/kali_common/src/messages.rs` (append at the end)
- Test: `crates/kali_common/src/messages_tests.rs`

**Interfaces:**
- Produces (all `pub`, re-exported from `kali_common` by `pub use messages::*`):
  - `default_param_non_declaration_message() -> &'static str`
  - `default_param_exported_message(function: &str) -> String`
  - `default_param_not_literal_message(function: &str, param: &str) -> String`
  - `default_param_composite_message(function: &str, param: &str) -> String`
  - `default_param_async_or_generator_message(function: &str) -> String`
  - `default_param_value_use_message(function: &str) -> String`
  - `default_param_spread_call_message(function: &str) -> String`
  - `default_param_omitted_argument_message(function: &str, param: &str) -> String`
  - `default_param_eval_refused_message() -> &'static str`

- [ ] **Step 1: Write the failing test**

Append to `crates/kali_common/src/messages_tests.rs`:

```rust
#[test]
fn default_parameter_messages_name_the_function_and_parameter() {
    assert_eq!(
        default_param_non_declaration_message(),
        "default parameters are only available on function declarations in the current phase"
    );
    assert_eq!(
        default_param_exported_message("f"),
        "a function with default parameters cannot be exported in the current phase: `f`"
    );
    assert_eq!(
        default_param_not_literal_message("f", "b"),
        "a default parameter value must be a number, string, boolean, null or BigInt literal in the current phase: `b` in `f`"
    );
    assert_eq!(
        default_param_composite_message("f", "o"),
        "an object or array default parameter is unavailable in the current phase: kali cannot pass an object or array literal directly as a call argument; `o` in `f`"
    );
    assert_eq!(
        default_param_async_or_generator_message("f"),
        "default parameters are only available on function declarations in the current phase: `f` is a generator or `async` function"
    );
    assert_eq!(
        default_param_value_use_message("f"),
        "a function with default parameters can only be called directly by name in the current phase; `f` is used as a value here"
    );
    assert_eq!(
        default_param_spread_call_message("f"),
        "a function with default parameters cannot be called with a spread argument in the current phase: `f(...)`"
    );
    assert_eq!(
        default_param_omitted_argument_message("f", "b"),
        "`f(…)` omits an argument for `b`, which has no default"
    );
    assert_eq!(
        default_param_eval_refused_message(),
        "default parameters are unavailable under --compat eval"
    );
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_common default_parameter_messages`
Expected: compile error, `cannot find function default_param_non_declaration_message`.

- [ ] **Step 3: Add the messages**

Append to `crates/kali_common/src/messages.rs`:

```rust
/// Default-parameters spec §3.3. A default on an arrow, method or function
/// expression (spec A-2: refused by the parser).
pub const fn default_param_non_declaration_message() -> &'static str {
    "default parameters are only available on function declarations in the current phase"
}

/// Default-parameters spec §3.3, A-3. An exported function has call sites the
/// pass cannot see.
pub fn default_param_exported_message(function: &str) -> String {
    format!("a function with default parameters cannot be exported in the current phase: `{function}`")
}

/// Default-parameters spec §3.2 step 1, A-7.
pub fn default_param_not_literal_message(function: &str, param: &str) -> String {
    format!(
        "a default parameter value must be a number, string, boolean, null or BigInt literal in the current phase: `{param}` in `{function}`"
    )
}

/// Default-parameters spec A-7: kali refuses an object or array literal
/// passed directly as a call argument, which is what the call-site fill would
/// produce.
pub fn default_param_composite_message(function: &str, param: &str) -> String {
    format!(
        "an object or array default parameter is unavailable in the current phase: kali cannot pass an object or array literal directly as a call argument; `{param}` in `{function}`"
    )
}

/// Default-parameters spec §3.2 step 1.
pub fn default_param_async_or_generator_message(function: &str) -> String {
    format!(
        "default parameters are only available on function declarations in the current phase: `{function}` is a generator or `async` function"
    )
}

/// Default-parameters spec §3.2 step 2.
pub fn default_param_value_use_message(function: &str) -> String {
    format!(
        "a function with default parameters can only be called directly by name in the current phase; `{function}` is used as a value here"
    )
}

/// Default-parameters spec A-5.
pub fn default_param_spread_call_message(function: &str) -> String {
    format!(
        "a function with default parameters cannot be called with a spread argument in the current phase: `{function}(...)`"
    )
}

/// Default-parameters spec §3.2 step 3 / §3.3.
pub fn default_param_omitted_argument_message(function: &str, param: &str) -> String {
    format!("`{function}(…)` omits an argument for `{param}`, which has no default")
}

/// Default-parameters spec §3.2: `eval` can call a function by a name the
/// pass never sees.
pub const fn default_param_eval_refused_message() -> &'static str {
    "default parameters are unavailable under --compat eval"
}
```

- [ ] **Step 4: Run the test**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_common default_parameter_messages`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/messages.rs crates/kali_common/src/messages_tests.rs
git commit -m "feat(default-parameters): refusal messages (spec §3.3, A-4, A-5, A-7)"
```

---

### Task 3: The parser keeps a declaration's defaults and refuses every other default

**Files:**
- Modify: `crates/kali_parser/src/declaration.rs` (`ParamListScan`, `classify_param_segment` at :83-104, `scan_param_list` at :109-183, `parse_parameter_list` at :199-218, the declaration call at :274, the arrow path at :605-616)
- Modify: `crates/kali_parser/src/module.rs:88-140` (`parse_export_declaration`)
- Create: `crates/kali_parser/src/declaration_tests/default_params.rs`
- Modify: `crates/kali_parser/src/declaration_tests.rs` (add `mod default_params;` beside `mod unsupported_params;` at :50)
- Modify: `crates/kali_parser/src/declaration_tests/unsupported_params.rs:70-92`

**Interfaces:**
- Consumes: `FunctionDeclaration::defaults` (Task 1); `kali_common::default_param_non_declaration_message`, `kali_common::default_param_exported_message` (Task 2).
- Produces: a `Statement::FunctionDeclaration` whose `defaults` holds each parsed default (`Some`) or `None`, or is empty when the list has no default. The default expression is exactly what `parse_assignment_expression` returns for the tokens after `=`.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_parser/src/declaration_tests/default_params.rs`:

```rust
//! Default parameters (default-parameters spec §3.1, A-2, A-3).

use super::*;
use kali_ast::{Expression, LiteralValue};

fn parse_ok(source: &str) -> Vec<Statement> {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(output.diagnostics.is_empty(), "{source}: {:?}", output.diagnostics);
    output.statements
}

fn diagnostics(source: &str) -> Vec<String> {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    parser
        .parse(None)
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect()
}

fn declaration(statements: &[Statement]) -> &kali_ast::FunctionDeclaration {
    match &statements[0] {
        Statement::FunctionDeclaration(f) => f,
        other => panic!("expected a FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn a_declaration_keeps_its_defaults_index_aligned() {
    let statements = parse_ok("function f(a, b = 2, c = \"x\") { return a; }");
    let f = declaration(&statements);
    assert_eq!(f.params, vec!["a", "b", "c"]);
    assert_eq!(f.defaults.len(), 3);
    assert!(f.defaults[0].is_none());
    assert_eq!(
        f.defaults[1].as_deref(),
        Some(&Expression::Literal(LiteralValue::Number(2.0)))
    );
    assert_eq!(
        f.defaults[2].as_deref(),
        Some(&Expression::Literal(LiteralValue::String("\"x\"".to_string())))
    );
}

#[test]
fn a_declaration_without_defaults_has_an_empty_vector() {
    let statements = parse_ok("function f(a, b) { return a; }");
    assert!(declaration(&statements).defaults.is_empty());
}

#[test]
fn a_typed_parameter_keeps_its_default() {
    let statements = parse_ok("function f(a: number, b: number = 3) { return a; }");
    let f = declaration(&statements);
    assert_eq!(f.params, vec!["a", "b"]);
    assert_eq!(
        f.defaults[1].as_deref(),
        Some(&Expression::Literal(LiteralValue::Number(3.0)))
    );
}

#[test]
fn a_plain_parameter_may_follow_a_defaulted_one() {
    let statements = parse_ok("function f(a = 1, b) { return a; }");
    let f = declaration(&statements);
    assert!(f.defaults[0].is_some());
    assert!(f.defaults[1].is_none());
}

#[test]
fn a_non_literal_default_is_parsed_and_left_to_the_pass() {
    let statements = parse_ok("function f(a, b = a * 2) { return b; }");
    assert!(matches!(
        declaration(&statements).defaults[1].as_deref(),
        Some(Expression::BinaryExpression(_))
    ));
}

#[test]
fn an_object_default_is_parsed() {
    let statements = parse_ok("function f(hooks = {}) { return 1; }");
    assert!(matches!(
        declaration(&statements).defaults[0].as_deref(),
        Some(Expression::ObjectExpression(_))
    ));
}

#[test]
fn defaults_on_other_function_forms_are_refused() {
    for source in [
        "const g = function (b = 5) { return b; };",
        "class C { m(b = 5) { return b; } }",
        "const g = (b = 5) => b;",
    ] {
        let messages = diagnostics(source);
        assert!(
            messages
                .iter()
                .any(|m| m.contains(kali_common::default_param_non_declaration_message())),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn an_exported_defaulted_declaration_is_refused() {
    for source in [
        "export function f(a = 1) { return a; }",
        "export async function f(a = 1) { return a; }",
        "export default function f(a = 1) { return a; }",
    ] {
        let messages = diagnostics(source);
        assert!(
            messages
                .iter()
                .any(|m| m == &kali_common::default_param_exported_message("f")),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn an_exported_declaration_without_defaults_is_still_accepted() {
    assert!(diagnostics("export function f(a) { return a; }").is_empty());
}
```

In `crates/kali_parser/src/declaration_tests/unsupported_params.rs`, replace the three declaration rows (`default_param_in_function_declaration_fails_closed`, `default_param_after_plain_param_fails_closed`, `multiple_default_params_fail_closed`) with:

```rust
/// Default parameters on a declaration are accepted now (default-parameters
/// spec §3.1); the module must still not be truncated.
fn assert_accepted_without_truncating(source: &str) {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:?}", output.diagnostics);
    assert_module_not_truncated(&output, source);
}

#[test]
fn default_param_in_function_declaration_is_accepted() {
    assert_accepted_without_truncating("function g(b = 5) { return b; }\nconsole.log(\"after\");");
}

#[test]
fn default_param_after_plain_param_is_accepted() {
    assert_accepted_without_truncating("function g(a, b = 5) { return a; }\nconsole.log(\"after\");");
}

#[test]
fn multiple_default_params_are_accepted() {
    assert_accepted_without_truncating("function g(a = 1, b = 2) { return a; }\nconsole.log(\"after\");");
}
```

The function-expression, class-method and arrow rows stay as they are: they look for `"default parameter"`, and the new refusal message contains that text.

Add `mod default_params;` to `crates/kali_parser/src/declaration_tests.rs`, next to `mod unsupported_params;`, following the file's existing `mod` lines.

- [ ] **Step 2: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_parser default_param`
Expected: FAIL. The declaration tests report the old diagnostic `a default parameter is not supported …`.

- [ ] **Step 3: Teach the scanner about defaults**

In `crates/kali_parser/src/declaration.rs`:

1. Add, above `enum ParamListScan`:

```rust
/// One scanned parameter: its name and, when it has a default, the absolute
/// token range of the default's expression (the tokens after `=`).
struct ScannedParam {
    name: String,
    default: Option<std::ops::Range<usize>>,
}
```

2. Change `ParamListScan::Simple` to `Simple { after: usize, params: Vec<ScannedParam> }`.

3. Change `classify_param_segment` to return the offset of a top-level `=` inside the segment:

```rust
    /// Classifies one comma-separated parameter-list segment.
    ///
    /// `Ok((name, None))` for `ident` and `ident: Type` (the annotation is
    /// erased). `Ok((name, Some(offset)))` for `ident = expr` and
    /// `ident: Type = expr`, where `offset` indexes the `=` within the
    /// segment (default-parameters spec §3.1). `Err(construct)` for
    /// everything else.
    fn classify_param_segment(segment: &[Token]) -> Result<(String, Option<usize>), &'static str> {
        let Some(first) = segment.first() else {
            return Err("an empty parameter");
        };
        match first.kind {
            TokenType::DotDotDot => Err("a rest parameter"),
            TokenType::LeftBrace | TokenType::LeftBracket => Err("a destructured parameter"),
            TokenType::Identifier => match segment.get(1).map(|token| &token.kind) {
                None => Ok((first.value.clone(), None)),
                Some(TokenType::Eq) => Ok((first.value.clone(), Some(1))),
                Some(TokenType::Colon) => {
                    Ok((first.value.clone(), Self::top_level_eq(segment, 2)))
                }
                // `ident?: Type` — an optional parameter has no default to
                // fill in, so the call-site rewrite cannot adapt its arity.
                Some(TokenType::Question) => Err("an optional parameter"),
                _ => Err("this parameter form"),
            },
            _ => Err("this parameter form"),
        }
    }

    /// The index of the first `=` at bracket depth 0 in `segment[from..]`.
    fn top_level_eq(segment: &[Token], from: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in segment.iter().enumerate().skip(from) {
            match token.kind {
                TokenType::LeftParen | TokenType::LeftBrace | TokenType::LeftBracket => depth += 1,
                TokenType::RightParen | TokenType::RightBrace | TokenType::RightBracket => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Eq if depth == 0 => return Some(index),
                _ => {}
            }
        }
        None
    }
```

4. In `scan_param_list`, segments are sub-slices of `body`, which starts at `start + 1`. Track each segment's absolute start so the default range can be absolute. Replace the segment-splitting and classification loop with:

```rust
        // Split on top-level commas, keeping each segment's absolute start.
        let mut segments: Vec<(usize, &[Token])> = Vec::new();
        let mut depth = 0usize;
        let mut segment_start = 0usize;
        for (offset, token) in body.iter().enumerate() {
            match token.kind {
                TokenType::LeftParen | TokenType::LeftBrace | TokenType::LeftBracket => depth += 1,
                TokenType::RightParen | TokenType::RightBrace | TokenType::RightBracket => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Comma if depth == 0 => {
                    segments.push((start + 1 + segment_start, &body[segment_start..offset]));
                    segment_start = offset + 1;
                }
                _ => {}
            }
        }
        segments.push((start + 1 + segment_start, &body[segment_start..]));

        // A single trailing comma is legal and produces one empty final
        // segment; drop it. An empty segment anywhere else is a syntax error
        // and falls through to `classify_param_segment`'s rejection.
        if segments.len() > 1 && segments.last().is_some_and(|(_, last)| last.is_empty()) {
            segments.pop();
        }

        let mut params = Vec::with_capacity(segments.len());
        for (absolute_start, segment) in segments {
            match Self::classify_param_segment(segment) {
                Ok((name, eq)) => {
                    let default = match eq {
                        Some(eq) if eq + 1 < segment.len() => {
                            Some(absolute_start + eq + 1..absolute_start + segment.len())
                        }
                        // `ident =` with nothing after it.
                        Some(_) => return ParamListScan::Unsupported {
                            after,
                            construct: "an empty default parameter",
                        },
                        None => None,
                    };
                    params.push(ScannedParam { name, default });
                }
                Err(construct) => return ParamListScan::Unsupported { after, construct },
            }
        }
        ParamListScan::Simple { after, params }
```

The empty-body early return becomes `return ParamListScan::Simple { after, params: Vec::new() };` (unchanged text; the type now holds `ScannedParam`).

- [ ] **Step 4: Parse defaults for declarations, refuse them elsewhere**

Still in `crates/kali_parser/src/declaration.rs`:

1. Replace `parse_parameter_list` with two functions:

```rust
    /// Parses a parameter list with the stream positioned AT the opening `(`,
    /// for a function form that cannot take defaults (methods and function
    /// expressions; default-parameters spec A-2). A default is refused with
    /// E5506; the names are still returned so the body parses normally.
    ///
    /// Always leaves the cursor just past the matching `)` when one exists, so
    /// the caller can parse the body without risk of absorbing the rest of the
    /// module.
    pub(crate) fn parse_parameter_list(&mut self) -> Vec<String> {
        let (params, defaults) = self.parse_declaration_parameter_list();
        if defaults.iter().any(Option::is_some) {
            self.push_feature_unavailable(kali_common::default_param_non_declaration_message());
        }
        params
    }

    /// Parses a function declaration's parameter list, keeping each default
    /// (default-parameters spec §3.1). Returns the names and the defaults,
    /// index-aligned; the defaults vector is empty when no parameter has one.
    pub(crate) fn parse_declaration_parameter_list(
        &mut self,
    ) -> (Vec<String>, Vec<Option<Box<Expression>>>) {
        match self.scan_param_list(self.stream.position) {
            ParamListScan::Simple { after, params } => {
                let mut names = Vec::with_capacity(params.len());
                let mut defaults = Vec::with_capacity(params.len());
                for param in params {
                    names.push(param.name);
                    defaults.push(param.default.map(|range| Box::new(self.parse_default(range))));
                }
                if defaults.iter().all(Option::is_none) {
                    defaults.clear();
                }
                self.stream.position = after;
                (names, defaults)
            }
            ParamListScan::Unsupported { after, construct } => {
                self.reject_unsupported_param(construct);
                self.stream.position = after;
                (Vec::new(), Vec::new())
            }
            ParamListScan::NotAParamList => {
                self.push_feature_unavailable(
                    "unterminated parameter list — expected a closing `)`".to_string(),
                );
                self.stream.position = self.stream.tokens.len();
                (Vec::new(), Vec::new())
            }
        }
    }

    /// Parses one default's tokens as an assignment expression with a
    /// sub-parser, so the main stream never moves into the parameter list.
    /// Tokens left over after the expression are refused.
    fn parse_default(&mut self, range: std::ops::Range<usize>) -> Expression {
        let mut tokens: Vec<Token> = self.stream.tokens[range.clone()].to_vec();
        let eof_span = tokens.last().map(|token| token.span).unwrap_or_default();
        tokens.push(Token::new(TokenType::Eof, String::new(), eof_span));
        let mut sub = Parser::new(self.file_id, tokens);
        let expression = sub.parse_assignment_expression();
        if !matches!(sub.stream.current_kind(), Some(TokenType::Eof) | None) {
            sub.push_feature_unavailable(
                "this default parameter value is unavailable in the current phase",
            );
        }
        self.diagnostics.extend(sub.diagnostics);
        expression
    }
```

If `Span` has no `Default` impl or is not `Copy`, use `tokens.last().map(|token| token.span.clone())` and fall back to `self.stream.tokens[range.start].span.clone()` (the range is non-empty by construction in Step 3). Check `crates/kali_common/src/span.rs` (or wherever `Span` is defined) before choosing.

2. In `parse_function_declaration_with_async`, replace `let params = self.parse_parameter_list();` with `let (params, defaults) = self.parse_declaration_parameter_list();`, and construct the declaration with `defaults,` instead of `defaults: Vec::new(),`.

3. In the arrow path (around :605), the `ParamListScan::Simple { after, params }` arm becomes:

```rust
            ParamListScan::Simple { after, params } => {
                if params.iter().any(|param| param.default.is_some()) {
                    self.push_feature_unavailable(
                        kali_common::default_param_non_declaration_message(),
                    );
                }
                ArrowParams::Ok {
                    after,
                    params: params.into_iter().map(|param| param.name).collect(),
                }
            }
```

`ArrowParams::Ok` keeps `params: Vec<String>`.

4. Add `use kali_lexer::{Token, TokenType};` is already present; confirm `Expression` is imported (it is, at the top of the file).

- [ ] **Step 5: Refuse exported defaulted declarations**

In `crates/kali_parser/src/module.rs`, `parse_export_declaration`: every place that parses a function declaration for an export (`export default async function`, `export default function`, `export async function`, `export function`) passes the result through a new helper on `Parser`:

```rust
    /// Default-parameters spec A-3: an exported function has call sites the
    /// call-site rewrite cannot see, so its defaults are refused here, where
    /// the export is still visible.
    fn refuse_exported_defaults(&mut self, declaration: &FunctionDeclaration) {
        if !declaration.defaults.is_empty() {
            self.push_feature_unavailable(kali_common::default_param_exported_message(
                &declaration.name,
            ));
        }
    }
```

Apply it in the two `export default` arms inside their `and_then` closures before wrapping, and for the two plain-export returns:

```rust
        if self.stream.current_kind() == Some(&TokenType::Async)
            && self.stream.peek_next_kind() == Some(&TokenType::Function)
        {
            let statement = self.parse_function_declaration_with_async(true, false);
            if let Some(Statement::FunctionDeclaration(declaration)) = &statement {
                self.refuse_exported_defaults(declaration);
            }
            return statement;
        }

        if self.stream.current_kind() == Some(&TokenType::Function) {
            let statement = self.parse_function_declaration();
            if let Some(Statement::FunctionDeclaration(declaration)) = &statement {
                self.refuse_exported_defaults(declaration);
            }
            return statement;
        }
```

The `export default` closures borrow `self` immutably through `self.parse_function_declaration_with_async(...)` and then map; rewrite each as a `match` so `self.refuse_exported_defaults(&function)` can be called before building `ExportDefaultDeclaration::FunctionDeclaration(function)`. Import `FunctionDeclaration` from `kali_ast` in `module.rs` if it is not already imported.

- [ ] **Step 6: Run the parser tests**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_parser`
Expected: PASS, including every `default_params` test and the re-pinned `unsupported_params` rows.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_parser
git commit -m "feat(default-parameters): parser keeps declaration defaults; other forms and exports refuse (spec §3.1, A-2, A-3)"
```

At this commit a program with a literal default parses and is NOT yet rewritten. It must not be run end to end until Task 4 lands; Task 4 is the next task.

---

### Task 4: The `default_params` pass

**Files:**
- Modify: `crates/kali_cli/src/build/block_scope_rename/walk.rs` (the `Hooks` trait at :47-68; `Statement::FunctionDeclaration` arm at :257; `ExportDefaultDeclaration::FunctionDeclaration` arm at :300; `Expression::CallExpression` arm at :358)
- Create: `crates/kali_cli/src/build/default_params/mod.rs`
- Create: `crates/kali_cli/src/build/default_params/literal.rs`
- Create: `crates/kali_cli/src/build/default_params/literal_tests.rs`
- Create: `crates/kali_cli/src/build/default_params/default_params_tests.rs`
- Modify: `crates/kali_cli/src/build/mod.rs` (add `pub mod default_params;` in alphabetical position after `pub mod capture_param_rewrite;`)
- Modify: `crates/kali_cli/src/build/compile.rs:819-826` (call the pass after the rename)

**Interfaces:**
- Consumes: `FunctionDeclaration::defaults` (Task 1); the message functions (Task 2); parser output (Task 3); `block_scope_rename::walk::{walk_program, Hooks, BindKind, ScopeKind}`.
- Produces:
  - `pub fn apply_default_params(statements: &mut [Statement], compat_eval: bool) -> Vec<String>` in `crate::build::default_params`. Returns the refusal messages; when it is non-empty the statements are unchanged. When it is empty every `FunctionDeclaration::defaults` in the program is empty.
  - `pub(crate) enum DefaultKind { Scalar, Composite, Other }` and `pub(crate) fn classify_default(expression: &Expression) -> DefaultKind` in `crate::build::default_params::literal`.
  - Two new `Hooks` methods with empty default bodies: `fn function_declaration(&mut self, _decl: &mut FunctionDeclaration) {}` and `fn call_expression(&mut self, _call: &mut CallExpression) {}`.

- [ ] **Step 1: Write the failing classifier test**

Create `crates/kali_cli/src/build/default_params/literal_tests.rs`:

```rust
use super::*;
use crate::build::block_scope_rename::test_support::parse;
use kali_ast::Statement;

/// The default of `b` in `function f(b = <default>) {}`.
fn default_of(default: &str) -> Expression {
    let statements = parse(&format!("function f(b = {default}) {{ return 1; }}"));
    let Statement::FunctionDeclaration(f) = &statements[0] else {
        panic!("{statements:?}")
    };
    *f.defaults[0].clone().expect("a default")
}

#[test]
fn scalar_literal_defaults_are_admitted() {
    for default in [
        "0", "2.5", "-1", "+3", "\"x\"", "'y'", "`z`", "true", "false", "null", "0n",
    ] {
        assert_eq!(classify_default(&default_of(default)), DefaultKind::Scalar, "{default}");
    }
}

#[test]
fn object_and_array_defaults_are_composite() {
    for default in ["[]", "[1, 2]", "{}", "{ a: 1 }", "[[1], { c: {} }]"] {
        assert_eq!(classify_default(&default_of(default)), DefaultKind::Composite, "{default}");
    }
}

#[test]
fn every_other_default_is_not_a_literal() {
    for default in [
        "x", "1 + 2", "-x", "!true", "f()", "`a${1}`", "() => 1", "/re/", "undefined", "void 0",
    ] {
        assert_eq!(classify_default(&default_of(default)), DefaultKind::Other, "{default}");
    }
}
```

`parse` asserts there are no parser diagnostics; every default above parses without one at Task 3. If one of these shapes is itself refused by the parser, drop that row and note it in the followups file rather than weakening `parse`.

- [ ] **Step 2: Write the classifier**

Create `crates/kali_cli/src/build/default_params/literal.rs`:

```rust
//! What kind of value a default is (default-parameters spec §3.2 step 1, A-7).

use kali_ast::{Expression, LiteralValue};

#[cfg(test)]
#[path = "literal_tests.rs"]
mod literal_tests;

/// The three kinds of default the pass distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DefaultKind {
    /// A number, string, boolean, `null` or BigInt literal, or unary `-`/`+`
    /// on a number. It has no side effects and reads no binding, so
    /// evaluating a clone of it at the call site is not observable (spec
    /// §3.2, "Evaluation order").
    Scalar,
    /// An object or array literal. Refused (spec A-7): kali refuses an object
    /// or array literal passed directly as a call argument, which is what the
    /// fill would produce.
    Composite,
    /// Anything else.
    Other,
}

pub(crate) fn classify_default(expression: &Expression) -> DefaultKind {
    match expression {
        Expression::Literal(LiteralValue::Regex { .. }) => DefaultKind::Other,
        Expression::Literal(_) | Expression::BigIntLiteral(_) => DefaultKind::Scalar,
        Expression::UnaryExpression(unary)
            if (unary.operator == "-" || unary.operator == "+")
                && matches!(unary.argument, Expression::Literal(LiteralValue::Number(_))) =>
        {
            DefaultKind::Scalar
        }
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => DefaultKind::Composite,
        _ => DefaultKind::Other,
    }
}
```

- [ ] **Step 3: Add the walker hooks**

In `crates/kali_cli/src/build/block_scope_rename/walk.rs`, add to the `Hooks` trait (after `assignment_statement`):

```rust
    /// A function declaration, seen before its name is bound and its body
    /// walked, with mutable access (default-parameters spec §3.2).
    fn function_declaration(&mut self, _decl: &mut FunctionDeclaration) {}
    /// A call expression, seen with mutable access before `call` and before
    /// its callee and arguments are walked (default-parameters spec §3.2).
    fn call_expression(&mut self, _call: &mut CallExpression) {}
```

Call `hooks.function_declaration(f);` as the first line of the `Statement::FunctionDeclaration(f)` arm and of the `ExportDefaultDeclaration::FunctionDeclaration(f)` arm. In the `Expression::CallExpression(e)` arm, call `hooks.call_expression(e);` before `hooks.call(&e.callee);`. `e` is a `Box<CallExpression>`, so pass `e` (auto-deref to `&mut CallExpression`) or `&mut **e`.

- [ ] **Step 4: Write the failing pass tests**

Create `crates/kali_cli/src/build/default_params/default_params_tests.rs`:

```rust
use super::*;
use crate::build::block_scope_rename::test_support::parse;
use kali_ast::{Expression, LiteralValue, Statement};

fn applied(source: &str) -> (Vec<String>, Vec<Statement>) {
    let mut statements = parse(source);
    let refusals = apply_default_params(&mut statements, false);
    (refusals, statements)
}

/// The arguments of the `n`-th call to `name`, in source order, found by a
/// walk over the rewritten program.
fn call_args(statements: &mut [Statement], name: &str) -> Vec<Vec<Expression>> {
    #[derive(Default)]
    struct Calls {
        name: String,
        found: Vec<Vec<Expression>>,
    }
    impl Hooks for Calls {
        fn enter(&mut self, _: ScopeKind, _: Option<&str>) {}
        fn exit(&mut self) {}
        fn bind(&mut self, _: &mut String, _: BindKind) {}
        fn reference(&mut self, _: &mut String) {}
        fn call_expression(&mut self, call: &mut kali_ast::CallExpression) {
            if matches!(&call.callee, Expression::Identifier(n) if *n == self.name) {
                self.found.push(call.args.clone());
            }
        }
    }
    let mut calls = Calls { name: name.to_string(), ..Calls::default() };
    walk::walk_program(statements, &mut calls);
    calls.found
}

fn number(value: f64) -> Expression {
    Expression::Literal(LiteralValue::Number(value))
}

fn declaration<'s>(statements: &'s [Statement], name: &str) -> &'s kali_ast::FunctionDeclaration {
    statements
        .iter()
        .find_map(|statement| match statement {
            Statement::FunctionDeclaration(f) if f.name == name => Some(f),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no declaration `{name}` in {statements:?}"))
}

#[test]
fn an_omitted_trailing_argument_gets_its_default() {
    let (refusals, mut got) = applied("function f(a, b = 2) { return a + b; } f(1);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "f"), vec![vec![number(1.0), number(2.0)]]);
    assert!(declaration(&got, "f").defaults.is_empty());
}

#[test]
fn every_omitted_default_is_appended_in_order() {
    let (refusals, mut got) = applied("function f(a = 1, b = 2) { return a + b; } f(); f(5);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![vec![number(1.0), number(2.0)], vec![number(5.0), number(2.0)]]
    );
}

#[test]
fn a_literal_undefined_is_replaced() {
    let (refusals, mut got) = applied("function f(a, b = 2) { return a + b; } f(1, undefined);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "f"), vec![vec![number(1.0), number(2.0)]]);
}

#[test]
fn a_middle_literal_undefined_is_replaced() {
    let (refusals, mut got) =
        applied("function f(a, b = 2, c = 3) { return a + b + c; } f(1, undefined, 9);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![vec![number(1.0), number(2.0), number(9.0)]]
    );
}

#[test]
fn void_of_a_literal_is_replaced() {
    let (refusals, mut got) = applied("function f(a = 7) { return a; } f(void 0);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "f"), vec![vec![number(7.0)]]);
}

#[test]
fn a_non_literal_argument_passes_through() {
    let (refusals, mut got) = applied("function f(a = 7) { return a; } let x = 3; f(x);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![vec![Expression::Identifier("x".to_string())]]
    );
}


#[test]
fn a_recursive_call_is_filled() {
    let (refusals, mut got) =
        applied("function f(n, acc = 0) { if (n === 0) return acc; return f(n - 1, acc + n); } f(3); f(2);");
    assert!(refusals.is_empty(), "{refusals:?}");
    let calls = call_args(&mut got, "f");
    assert_eq!(calls[1], vec![number(3.0), number(0.0)]);
    assert_eq!(calls[2], vec![number(2.0), number(0.0)]);
}

#[test]
fn a_call_before_the_declaration_is_filled() {
    let (refusals, mut got) = applied("f(); function f(a = 1) { return a; }");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "f"), vec![vec![number(1.0)]]);
}

#[test]
fn a_nested_declaration_is_collected_and_filled() {
    let (refusals, mut got) =
        applied("function outer() { function inner(a = 4) { return a; } return inner(); } outer();");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "inner"), vec![vec![number(4.0)]]);
}

#[test]
fn a_template_string_default_is_a_literal() {
    let (refusals, _) = applied("function f(a = `x`) { return a; } f();");
    assert!(refusals.is_empty(), "{refusals:?}");
}

#[test]
fn a_program_without_defaults_is_untouched() {
    let source = "function f(a, b) { return a + b; } const g = (x) => f(x, 1); g(2);";
    let (refusals, got) = applied(source);
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(got, parse(source));
}

fn refused(source: &str) -> Vec<String> {
    let mut statements = parse(source);
    let before = statements.clone();
    let refusals = apply_default_params(&mut statements, false);
    assert!(!refusals.is_empty(), "{source}: expected a refusal");
    assert_eq!(statements, before, "{source}: a refused program must be left unchanged");
    refusals
}

#[test]
fn a_non_literal_default_is_refused() {
    assert_eq!(
        refused("function f(a, b = a * 2) { return b; } f(1);"),
        vec![kali_common::default_param_not_literal_message("f", "b")]
    );
}

#[test]
fn an_object_or_array_default_is_refused() {
    assert_eq!(
        refused("function f(o = {}) { return 1; } f();"),
        vec![kali_common::default_param_composite_message("f", "o")]
    );
    assert_eq!(
        refused("function f(a, xs = []) { return a; } f(1);"),
        vec![kali_common::default_param_composite_message("f", "xs")]
    );
}

#[test]
fn an_async_or_generator_defaulted_function_is_refused() {
    assert_eq!(
        refused("async function f(a = 1) { return a; } f();"),
        vec![kali_common::default_param_async_or_generator_message("f")]
    );
    assert_eq!(
        refused("function* f(a = 1) { yield a; } f();"),
        vec![kali_common::default_param_async_or_generator_message("f")]
    );
}

#[test]
fn a_value_use_is_refused() {
    for source in [
        "function f(a = 1) { return a; } const g = f; g();",
        "function f(a = 1) { return a; } [1].map(f);",
        "function f(a = 1) { return a; } f.call(null);",
        "function f(a = 1) { return a; } console.log(typeof f);",
        "function f(a = 1) { return a; } f?.();",
    ] {
        assert_eq!(
            refused(source),
            vec![kali_common::default_param_value_use_message("f")],
            "{source}"
        );
    }
}

#[test]
fn an_export_specifier_is_refused() {
    assert_eq!(
        refused("function f(a = 1) { return a; } export { f };"),
        vec![kali_common::default_param_exported_message("f")]
    );
}

#[test]
fn a_spread_call_is_refused() {
    assert_eq!(
        refused("function f(a = 1) { return a; } const xs = [2]; f(...xs);"),
        vec![kali_common::default_param_spread_call_message("f")]
    );
}

#[test]
fn an_omitted_argument_without_a_default_is_refused() {
    assert_eq!(
        refused("function f(a = 1, b) { return a; } f();"),
        vec![kali_common::default_param_omitted_argument_message("f", "b")]
    );
}

#[test]
fn eval_compat_refuses_any_defaulted_function() {
    let mut statements = parse("function f(a = 1) { return a; } f();");
    let before = statements.clone();
    assert_eq!(
        apply_default_params(&mut statements, true),
        vec![kali_common::default_param_eval_refused_message().to_string()]
    );
    assert_eq!(statements, before);
}

#[test]
fn eval_compat_without_defaults_is_untouched() {
    let mut statements = parse("function f(a) { return a; } f(1);");
    assert!(apply_default_params(&mut statements, true).is_empty());
}
```

Note: `f?.()` parses at the baseline as `OptionalChainExpression { object: Identifier("f"), .. }` (spec A-6), so the walker reports `f` as a reference that is not a callee.

- [ ] **Step 5: Run them to verify they fail**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_cli --lib default_params`
Expected: compile error, `cannot find function apply_default_params` (the module does not exist yet).

- [ ] **Step 6: Write the pass**

Create `crates/kali_cli/src/build/default_params/mod.rs`:

```rust
//! Default parameters (default-parameters spec §3.2). See `apply_default_params`.
//!
//! Kali emits every call at its callee's exact arity and has no runtime
//! `undefined` distinct from `0` (register cluster G4), so a default cannot be
//! applied inside the callee. This pass applies it at the call site instead:
//! every direct call `f(…)` that omits a defaulted argument, or passes a
//! literal `undefined` for it, gets a fresh clone of the default. Any other
//! argument passes through unchanged (the human partner's ruling). Afterwards
//! the declaration's defaults are removed, so every later stage sees an
//! ordinary fixed-arity function.
//!
//! The pass runs after `block_scope_rename`, so every binding has a unique
//! spelling and a call can be matched to its declaration by name.

mod literal;

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{CallExpression, Expression, FunctionDeclaration, Statement};

use super::block_scope_rename::walk::{self, BindKind, Hooks, ScopeKind};
use literal::{classify_default, DefaultKind};

#[cfg(test)]
#[path = "default_params_tests.rs"]
mod default_params_tests;

/// Rewrites every direct call of a defaulted function declaration and strips
/// the defaults. Returns the refusal messages (E5506 texts); when any is
/// returned the program is left exactly as it was.
pub fn apply_default_params(statements: &mut [Statement], compat_eval: bool) -> Vec<String> {
    let mut collector = Collector::default();
    walk::walk_program(statements, &mut collector);
    if collector.defaulted.is_empty() {
        return Vec::new();
    }
    if compat_eval {
        return vec![kali_common::default_param_eval_refused_message().to_string()];
    }

    let exported = exported_names(statements);
    let refusals = collector.refusals(&exported);
    if !refusals.is_empty() {
        return refusals;
    }

    let mut filler = Filler {
        defaulted: collector.defaulted,
    };
    walk::walk_program(statements, &mut filler);
    Vec::new()
}

/// A defaulted declaration: its parameter names and index-aligned defaults.
struct Defaulted {
    params: Vec<String>,
    defaults: Vec<Option<Box<Expression>>>,
    /// Set when the declaration itself is out of scope (§3.2 step 1).
    refusal: Option<String>,
}

/// One direct call `name(…)`.
struct Call {
    name: String,
    argument_count: usize,
    has_spread: bool,
}

#[derive(Default)]
struct Collector {
    defaulted: BTreeMap<String, Defaulted>,
    calls: Vec<Call>,
    /// Every name referenced other than as the callee of a direct call.
    value_uses: BTreeSet<String>,
    /// The callee name of the call whose callee is about to be walked.
    pending_callee: Option<String>,
}

impl Collector {
    /// One message per refused function, in name order, then one per refused
    /// call, in source order.
    fn refusals(&self, exported: &BTreeSet<String>) -> Vec<String> {
        let mut refusals = Vec::new();
        for (name, defaulted) in &self.defaulted {
            if let Some(refusal) = &defaulted.refusal {
                refusals.push(refusal.clone());
            } else if exported.contains(name) {
                refusals.push(kali_common::default_param_exported_message(name));
            } else if self.value_uses.contains(name) {
                refusals.push(kali_common::default_param_value_use_message(name));
            }
        }
        for call in &self.calls {
            let Some(defaulted) = self.defaulted.get(&call.name) else {
                continue;
            };
            if call.has_spread {
                refusals.push(kali_common::default_param_spread_call_message(&call.name));
                continue;
            }
            // A literal `undefined` for a parameter with no default is passed
            // through, like any other argument; only an omission is refused.
            for index in call.argument_count..defaulted.params.len() {
                if defaulted.defaults[index].is_none() {
                    refusals.push(kali_common::default_param_omitted_argument_message(
                        &call.name,
                        &defaulted.params[index],
                    ));
                    break;
                }
            }
        }
        refusals
    }
}

impl Hooks for Collector {
    fn enter(&mut self, _kind: ScopeKind, _label: Option<&str>) {}
    fn exit(&mut self) {}
    fn bind(&mut self, _name: &mut String, _kind: BindKind) {}

    fn reference(&mut self, name: &mut String) {
        if self.pending_callee.as_deref() == Some(name.as_str()) {
            self.pending_callee = None;
        } else {
            self.value_uses.insert(name.clone());
        }
    }

    fn call(&mut self, callee: &Expression) {
        self.pending_callee = match callee {
            Expression::Identifier(name) => Some(name.clone()),
            _ => None,
        };
    }

    fn call_expression(&mut self, call: &mut CallExpression) {
        let Expression::Identifier(name) = &call.callee else {
            return;
        };
        self.calls.push(Call {
            name: name.clone(),
            argument_count: call.args.len(),
            has_spread: call
                .args
                .iter()
                .any(|argument| matches!(argument, Expression::SpreadElement(_))),
        });
    }

    fn function_declaration(&mut self, decl: &mut FunctionDeclaration) {
        if decl.defaults.is_empty() {
            return;
        }
        let refusal = if decl.is_async || decl.generator {
            Some(kali_common::default_param_async_or_generator_message(&decl.name))
        } else {
            decl.params
                .iter()
                .zip(&decl.defaults)
                .find_map(|(param, default)| match default.as_deref().map(classify_default) {
                    None | Some(DefaultKind::Scalar) => None,
                    Some(DefaultKind::Composite) => {
                        Some(kali_common::default_param_composite_message(&decl.name, param))
                    }
                    Some(DefaultKind::Other) => {
                        Some(kali_common::default_param_not_literal_message(&decl.name, param))
                    }
                })
        };
        self.defaulted.insert(
            decl.name.clone(),
            Defaulted {
                params: decl.params.clone(),
                defaults: decl.defaults.clone(),
                refusal,
            },
        );
    }
}

/// Fills every direct call of a defaulted function and strips the defaults.
/// Runs only when the collector found nothing to refuse.
struct Filler {
    defaulted: BTreeMap<String, Defaulted>,
}

impl Hooks for Filler {
    fn enter(&mut self, _kind: ScopeKind, _label: Option<&str>) {}
    fn exit(&mut self) {}
    fn bind(&mut self, _name: &mut String, _kind: BindKind) {}
    fn reference(&mut self, _name: &mut String) {}

    fn call_expression(&mut self, call: &mut CallExpression) {
        let Expression::Identifier(name) = &call.callee else {
            return;
        };
        let Some(defaulted) = self.defaulted.get(name) else {
            return;
        };
        for (index, default) in defaulted.defaults.iter().enumerate() {
            let Some(default) = default else {
                continue;
            };
            match call.args.get_mut(index) {
                Some(argument) if is_literal_undefined(argument) => {
                    *argument = (**default).clone();
                }
                Some(_) => {}
                None => call.args.push((**default).clone()),
            }
        }
    }

    fn function_declaration(&mut self, decl: &mut FunctionDeclaration) {
        decl.defaults.clear();
    }
}

/// A literal `undefined`, or `void` applied to a literal.
fn is_literal_undefined(argument: &Expression) -> bool {
    match argument {
        Expression::Identifier(name) => name == "undefined",
        Expression::UnaryExpression(unary) => {
            unary.operator == "void" && matches!(unary.argument, Expression::Literal(_))
        }
        _ => false,
    }
}

/// Names exported by a top-level `export { … }` with no `from`.
fn exported_names(statements: &[Statement]) -> BTreeSet<String> {
    statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::ExportNamed(export) if export.source.is_none() => Some(export),
            _ => None,
        })
        .flat_map(|export| export.specifiers.iter().map(|specifier| specifier.local.clone()))
        .collect()
}
```

The `refusals` order is deterministic: declarations by name (a `BTreeMap`), then calls in source order. Each refused function is reported once; for a function both exported and used as a value, the export message wins, because an `export { f }` specifier is itself reported to `reference` and would otherwise produce both.

- [ ] **Step 7: Wire the module and the compile step**

In `crates/kali_cli/src/build/mod.rs`, add `pub mod default_params;` after `pub mod capture_param_rewrite;`.

In `crates/kali_cli/src/build/compile.rs`, directly after the block-scope rename's `compat_eval` check (the block ending at `:825`) and before the captured-parameter rewrite comment, add:

```rust
    // Default-parameters spec §3.2: fill each omitted literal default in at
    // its call site and strip the defaults, AFTER the rename (names are
    // unique) and BEFORE every stage that keys a function by its arity.
    let default_refusals =
        crate::build::default_params::apply_default_params(&mut parsed.statements, compat_eval);
    if !default_refusals.is_empty() {
        return Err(default_refusals
            .into_iter()
            .map(|message| Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message))
            .collect());
    }
```

- [ ] **Step 8: Run the unit tests**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_cli --lib default_params`
Expected: PASS, every test in `literal_tests.rs` and `default_params_tests.rs`.

Then the rename's own tests, which share the walker:

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_cli --lib block_scope_rename capture_param_rewrite name_anon`
Expected: PASS (the new hooks have empty defaults).

- [ ] **Step 9: Smoke-test end to end**

```bash
CARGO_TARGET_DIR=$PWD/target cargo build -j 6 -p kali_cli --bin kali
printf 'function f(a, b = 2) { return a + b; }\nconsole.log(f(1));\n' > "$TMPDIR_SCRATCH/dp.js"
target/debug/kali check "$TMPDIR_SCRATCH/dp.js"; echo "check $?"
target/debug/kali run "$TMPDIR_SCRATCH/dp.js"; echo "run $?"
node "$TMPDIR_SCRATCH/dp.js"
```

Expected: `check 0`; `run` prints `3` and exits 0; node prints `3`.

- [ ] **Step 10: Commit**

```bash
git add crates/kali_cli/src/build
git commit -m "feat(default-parameters): call-site fill pass for literal defaults (spec §3.2, A-5, A-6)"
```

---

### Task 5: Black-box cases

**Files:**
- Create: `crates/kali_cli/tests/cases/soundness/default_parameters.toml`
- Create: `crates/kali_cli/tests/cases/soundness/default_parameters_refused.toml`

**Interfaces:**
- Consumes: the built `kali` binary with Tasks 1-4.

- [ ] **Step 1: Write the fill cases**

Every program below was checked on 2026-10-07 with its defaults written out by hand at each call site: `kali run` at the lane A merge printed exactly node's output. So a failure here is about default parameters, not about another construct. Programs that use an object or array argument, `=== null`, a call before the callee's declaration, or `Math.round` of a division were measured to fail at the baseline for unrelated reasons (Task 6 records them) and are deliberately absent.

Create `crates/kali_cli/tests/cases/soundness/default_parameters.toml`. `[source]` is file-wide, so each program has its own filename:

```toml
# Default parameters (default-parameters spec §3, §5.2, A-7). Every expected
# stdout is node v26.10.0's for the same file, measured 2026-10-07 with
# `env -u FORCE_COLOR node FILE.js`.

[source]
"kinds_module.js" = """
function num(a, b = 2) { return a + b; }
function str(s = "x") { return s + "!"; }
function flag(b = true) { return b ? "yes" : "no"; }
function neg(n = -1) { return n * 2; }
console.log(num(1), num(1, 5), str(), str("y"), flag(), flag(false), neg(), neg(4));
"""
"kinds_function.js" = """
function main() {
  function num(a, b = 2) { return a + b; }
  function str(s = "x") { return s + "!"; }
  console.log(num(1), num(1, 5), str(), str("y"));
}
main();
"""
"omissions.js" = """
function f(a = 1, b = 2) { return a * 10 + b; }
console.log(f(), f(5), f(5, 6), f(undefined, 6), f(5, undefined));
"""
"greet.js" = """
function greet(name, greeting = "Hello", punct = "!") { return greeting + ", " + name + punct; }
console.log(greet("Ada"));
console.log(greet("Bob", "Hi"));
console.log(greet("Cy", "Yo", "?"));
"""
"recursion.js" = """
function sum(n, acc = 0) { if (n === 0) return acc; return sum(n - 1, acc + n); }
console.log(sum(4), sum(3, 10));
"""

[[case]]
name = "scalar_defaults_fill_omitted_arguments_module_scope"
rationale = "A number, string, boolean and negative-number default, each called with and without its argument."
args = ["run", "kinds_module.js"]
exit = "success"
stdout = "3 6 x! y! yes no -2 8\n"

[[case]]
name = "scalar_defaults_fill_omitted_arguments_in_function"
rationale = "The same fill for declarations nested in a function."
args = ["run", "kinds_function.js"]
exit = "success"
stdout = "3 6 x! y!\n"

[[case]]
name = "omitted_and_literal_undefined_arguments_take_the_default"
rationale = "`f(undefined, 6)` and `f(5, undefined)` take the default at the `undefined` position only."
args = ["run", "omissions.js"]
exit = "success"
stdout = "12 52 56 16 52\n"

[[case]]
name = "string_defaults_fill_trailing_arguments"
rationale = "The shape of the corpus's `wrap(text, width = 72, indent = \"\")`: two trailing defaults, omitted, one given, both given."
args = ["run", "greet.js"]
exit = "success"
stdout = "Hello, Ada!\nHi, Bob!\nYo, Cy?\n"

[[case]]
name = "a_recursive_call_omitting_the_default_is_filled"
args = ["run", "recursion.js"]
exit = "success"
stdout = "10 16\n"

[[case]]
name = "check_accepts_a_program_with_scalar_defaults"
args = ["check", "greet.js"]
exit = "success"
```

- [ ] **Step 2: Confirm node agrees**

```bash
cd "$TMPDIR_SCRATCH" && python3 - /workspace/.worktrees/default-parameters/crates/kali_cli/tests/cases/soundness/default_parameters.toml <<'PY'
import sys, tomllib, subprocess
doc = tomllib.load(open(sys.argv[1], "rb"))
for case in doc["case"]:
    if case["args"][0] != "run":
        continue
    program = case["args"][-1]
    open(program, "w").write(doc["source"][program])
    out = subprocess.run(["node", program], capture_output=True, text=True).stdout
    print("OK " if out == case["stdout"] else "DIFF", case["name"], repr(out))
PY
```

Expected: every line starts with `OK`. A `DIFF` means the expected string in the file is wrong; replace it with node's output shown on that line.

- [ ] **Step 3: Write the refusal cases**

Create `crates/kali_cli/tests/cases/soundness/default_parameters_refused.toml`:

```toml
# Every default-parameter refusal (default-parameters spec §3.3, A-2..A-6),
# under both `check` and `run`, so the two commands are shown to agree.

[matrix]
cmd = ["check", "run"]

[source]
"not_literal.js" = "function f(a, b = a * 2) { return b; }\nconsole.log(f(1));\n"
"composite.js" = "function f(a, o = {}) { return a; }\nconsole.log(f(1));\n"
"async.js" = "async function f(a = 1) { return a; }\nf();\n"
"value_use.js" = "function f(a = 1) { return a; }\nconst g = f;\nconsole.log(g());\n"
"export_specifier.js" = "function f(a = 1) { return a; }\nexport { f };\n"
"export_declaration.js" = "export function f(a = 1) { return a; }\n"
"spread.js" = "function f(a = 1) { return a; }\nconst xs = [2];\nconsole.log(f(...xs));\n"
"omitted.js" = "function f(a = 1, b) { return a; }\nconsole.log(f());\n"
"arrow.js" = "const f = (a = 1) => a;\nconsole.log(f());\n"
"method.js" = "class C { m(a = 1) { return a; } }\nconsole.log(new C().m());\n"
"expression.js" = "const f = function (a = 1) { return a; };\nconsole.log(f());\n"
"eval.js" = "function f(a = 1) { return a; }\nconsole.log(f());\n"

[[case]]
name = "a_non_literal_default_is_refused"
args = ["${cmd}", "not_literal.js"]
exit = "failure"
stderr_contains = ["E5506", "must be a number, string, boolean, null or BigInt literal", "`b` in `f`"]

[[case]]
name = "an_object_default_is_refused"
rationale = "Spec A-7: kali refuses an object literal passed directly as a call argument, so an object default is refused with its own message rather than that one."
args = ["${cmd}", "composite.js"]
exit = "failure"
stderr_contains = ["E5506", "object or array default parameter", "`o` in `f`"]

[[case]]
name = "an_async_defaulted_function_is_refused"
args = ["${cmd}", "async.js"]
exit = "failure"
stderr_contains = ["E5506", "generator or `async` function"]

[[case]]
name = "a_defaulted_function_used_as_a_value_is_refused"
args = ["${cmd}", "value_use.js"]
exit = "failure"
stderr_contains = ["E5506", "can only be called directly by name", "`f` is used as a value"]

[[case]]
name = "an_export_specifier_of_a_defaulted_function_is_refused"
args = ["${cmd}", "export_specifier.js"]
exit = "failure"
stderr_contains = ["E5506", "cannot be exported", "`f`"]

[[case]]
name = "an_exported_defaulted_declaration_is_refused"
args = ["${cmd}", "export_declaration.js"]
exit = "failure"
stderr_contains = ["E5506", "cannot be exported", "`f`"]

[[case]]
name = "a_spread_call_to_a_defaulted_function_is_refused"
args = ["${cmd}", "spread.js"]
exit = "failure"
stderr_contains = ["E5506", "spread argument"]

[[case]]
name = "an_omitted_argument_without_a_default_is_refused"
args = ["${cmd}", "omitted.js"]
exit = "failure"
stderr_contains = ["E5506", "omits an argument for `b`"]

[[case]]
name = "an_arrow_default_is_refused"
args = ["${cmd}", "arrow.js"]
exit = "failure"
stderr_contains = ["E5506", "only available on function declarations"]

[[case]]
name = "a_method_default_is_refused"
args = ["${cmd}", "method.js"]
exit = "failure"
stderr_contains = ["E5506", "only available on function declarations"]

[[case]]
name = "a_function_expression_default_is_refused"
args = ["${cmd}", "expression.js"]
exit = "failure"
stderr_contains = ["E5506", "only available on function declarations"]

[[case]]
name = "a_defaulted_function_under_compat_eval_is_refused"
args = ["${cmd}", "--compat", "eval", "eval.js"]
exit = "failure"
stderr_contains = ["E5506", "unavailable under --compat eval"]
```

Before relying on `kali check --compat eval`, confirm `check` accepts `--compat`: `target/debug/kali check --help | grep -- --compat`. If it does not, move the eval case into its own file with `args = ["run", "--compat", "eval", "eval.js"]` and no matrix.

- [ ] **Step 4: Run the new cases**

Run: `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_cli --test cases -- soundness/default_parameters`
Expected: PASS, every trial in both files.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cases/soundness/default_parameters.toml crates/kali_cli/tests/cases/soundness/default_parameters_refused.toml
git commit -m "test(default-parameters): node-checked fill cases and one refusal per message under check and run"
```

---

### Task 6: Full suite, corpus measurement and bookkeeping

**Files:**
- Modify: `specs/19-feature-maturity.md` (new row after the "Hexadecimal, binary, octal, separated and legacy-octal numeric literals" row that lane A added)
- Modify: `docs/superpowers/specs/2026-10-07-default-parameters-design.md` (append `## 7. Amendments`)
- Create: `docs/superpowers/followups/default-parameters-discovered-defects.md`
- Possibly modify: `tools/blast-radius/accepts.json`, `tools/blast-radius/counts.json`, `docs/superpowers/followups/blast-radius-ranking.md`, register rows and oracle cases (only if Step 3 shows a move)

- [ ] **Step 1: Run the whole workspace, detached, under the watchdog**

```bash
cd /workspace/.worktrees/default-parameters
S="$TMPDIR_SCRATCH"; rm -f "$S/dp-workspace.exit"
setsid nohup bash -c "CARGO_TARGET_DIR=$PWD/target cargo test -j 6 --workspace --no-fail-fast -- --test-threads=6 > $S/dp-workspace.log 2>&1; echo \$? > $S/dp-workspace.exit" > /dev/null 2>&1 < /dev/null &
```

Wait for `$S/dp-workspace.exit`. Then:

```bash
grep -E '^test result' "$S/dp-workspace.log" | grep -v ' 0 failed'
grep -E '^test .* FAILED$' "$S/dp-workspace.log"
```

Expected: no failures. A failure that pins the old "a default parameter is not supported" refusal, or kali's old output for a program that now runs, is a moved pin: compare it with node, re-pin to node's output with a dated `RE-PINNED 2026-10-07 by the default-parameters project` note in its rationale (the convention in `crates/kali_cli/tests/cases/object/property_key_identity.toml`), and list it in the followups file. Any other failure: STOP and debug with superpowers:systematic-debugging.

- [ ] **Step 2: Re-check the 14 extension programs**

```bash
cd /workspace/.worktrees/default-parameters/tools/blast-radius/corpus/extension
for f in argv_stats build_report_json heat_diffusion_1d hex_dump histogram_bars matrix_ops moving_average paginate_results pivot_sales task_queue template_render turnstile_fsm word_frequency wrap_paragraph; do
  out=$(/workspace/.worktrees/default-parameters/target/debug/kali check $f.js 2>&1)
  echo "== $f: $(echo "$out" | grep -c 'default parameter') default-parameter errors"
  echo "$out" | grep '^error' | sed -E 's/`[^`]*`/`X`/g' | cut -c1-100 | sort -u
done
```

Expected: every program reports `0 default-parameter errors`. Copy the remaining error lists into the followups file (§2 there).

- [ ] **Step 3: Re-measure the accept set**

```bash
cd /workspace/.worktrees/default-parameters/tools/blast-radius
ln -s /workspace/tools/blast-radius/node_modules node_modules
node --test && node accepts.mjs && node count.mjs
cd ../.. && git diff --stat tools/
```

If only `kaliBinary` changed in `accepts.json`, run `git checkout tools/blast-radius/accepts.json tools/blast-radius/counts.json` and record "accept set unchanged" in the followups file. If the accept set moved, keep both files, run `CARGO_TARGET_DIR=$PWD/target cargo run -j 6 -p kali_blast_radius --example rank`, splice its stdout between the `GENERATED-PROVENANCE` and `GENERATED` markers of `docs/superpowers/followups/blast-radius-ranking.md` (provenance is the part before `## 2. The bands`), add a dated §6 amendment in the form of the TWELFTH one, and run `CARGO_TARGET_DIR=$PWD/target cargo test -j 6 -p kali_blast_radius` (expected PASS). In both cases, `rm tools/blast-radius/node_modules` afterwards; it must not be committed.

- [ ] **Step 4: Maturity row**

Add after the numeric-literal row in `specs/19-feature-maturity.md`:

```markdown
| Default parameters on function declarations (`function f(a, b = 2)`) | Phase 1 MVP | A function declaration may give any parameter a scalar literal default: a number, string, boolean, `null` or BigInt literal, or unary `-`/`+` on a number. Each direct call `f(…)` that omits a defaulted argument, or passes a literal `undefined` or `void <literal>` for it, gets a copy of the default at the call site; any other argument is passed through unchanged. Refused (E5506, `check` and `run` alike): an object or array default (kali cannot pass an object or array literal directly as a call argument); any other non-literal default; a default on an arrow, method or function expression; a defaulted generator or `async` function; a defaulted function used as a value (stored, passed, `typeof`, `.call`, `f?.()`), exported, or called with a spread argument; a call that omits an argument whose parameter has no default; any defaulted function under `--compat eval`. Not claimed: an argument that holds `undefined` at runtime gets `0` semantics rather than the default (kali has no runtime `undefined`, register cluster G4); defaults that read other parameters or bindings. Evidence: `default_params_tests.rs`, `literal_tests.rs`, parser `declaration_tests/default_params.rs`, `cases/soundness/default_parameters.toml`, `cases/soundness/default_parameters_refused.toml` |
```

- [ ] **Step 5: Spec amendments**

Append to `docs/superpowers/specs/2026-10-07-default-parameters-design.md`:

```markdown
---

## 7. Amendments

Found while planning (`docs/superpowers/plans/2026-10-07-default-parameters.md`), before any code was written:

* **A-1.** Defaults live on `FunctionDeclaration::defaults` (index-aligned with `params`), not on `FunctionParam`. `FunctionDeclaration::params` is a `Vec<String>`; `FunctionParam` is only used by arrows and function expressions. §3.1's `FunctionParam::plain` constructor is not needed.
* **A-2.** A default on an arrow, method or function expression is refused by the parser, with §3.3's message, rather than by the pass. Those forms never carry a default on the AST.
* **A-3.** `export function f(…)` parses to a plain function declaration, so the pass cannot see the export. The parser refuses an exported defaulted declaration; the pass refuses `export { f }`.
* **A-4.** AST nodes carry no spans. Diagnostics name the function and parameter instead (§3.3's "spans" sentence is withdrawn).
* **A-5.** A spread argument to a defaulted function (`f(...xs)`) is refused: the pass cannot count the arguments. New message `default_param_spread_call_message`.
* **A-6.** `f?.(…)` parses as `OptionalChainExpression { object: f }` and the call is dropped (pre-existing). The pass sees a value use and refuses it with the value-use message.
* **A-7 (human partner's ruling, 2026-10-07).** Scalar literal defaults only. Kali refuses an object or array literal passed directly as a call argument ("an object literal passed directly as a call argument is unavailable in the current phase; bind it to a const first"), so filling `{}` or `[]` at the call site would be refused with a message about an argument the user never wrote. An object or array default is refused by the pass with its own message. §1's "17 numbers or strings and one `hooks = {}`" now reads: 20 of the corpus's 21 defaults are in scope, and `task_queue.js`'s `hooks = {}` is refused. The §5.2 `{}`-freshness case is withdrawn.
```

Add any further amendment found during execution as A-7 onward, with what was measured.

- [ ] **Step 6: Followups file**

Create `docs/superpowers/followups/default-parameters-discovered-defects.md` with these sections, filled from Steps 1-3 and from the measurements already taken while planning:

```markdown
# Defects the default-parameters project measured and did NOT fix

**Filed** <date> by the **default-parameters** project
(`docs/superpowers/specs/2026-10-07-default-parameters-design.md`). **Oracle:**
`node v26.10.0`. **Baseline:** the lane A merge on `main`. **Branch:**
`default-parameters`.

## §1. Pre-existing, found while planning

| program | node | kali at the baseline |
|---|---|---|
| `function f(a, b) { return a + b; } console.log(f(1));` | `NaN` | `check` exit 0; `run` `error[E4201]: failed to load WASM module`, exit 1 |
| `f?.(1);` (any `f`) | calls `f` | parsed as `OptionalChainExpression { object: f }`; the call and its arguments are dropped |
| `console.log(late(1)); function late(a) { return a; }` | `1` | `error[E3100]: undefined identifier 'late'` (a call before the callee's declaration) |
| `function bar(c, p, w) { return Math.round((c / p) * w); } console.log(bar(3, 6, 40));` | `20` | `run`: `error[E4201]: failed to load WASM module`, exit 1 |
| `function tag(o) { return 1; } console.log(tag({}));` | `1` | `error[E5506]: an object literal passed directly as a call argument is unavailable in the current phase; bind it to a const first` (the reason for spec A-7) |
| `function arr(xs) { return xs.length; } console.log(arr([]));` | `0` | `error[E5506]: passing an array literal to function 'arr' is unavailable …` |

## §2. What the 14 extension programs need next

<the per-program error lists from Task 6 Step 2>

## §3. Accept set

<"unchanged at anchor 125/137, extension 0/40" or the new figures from Step 3>

## §4. Pins moved by this project

<each re-pinned case from Step 1, or "none">
```

Replace each `<…>` with the measured content before committing; do not commit the angle-bracket text.

- [ ] **Step 7: Format, lint, commit**

```bash
cd /workspace/.worktrees/default-parameters
cargo fmt --all --check
CARGO_TARGET_DIR=$PWD/target cargo clippy -q -j 6 -p kali_ast -p kali_common -p kali_parser -p kali_cli 2>&1 | grep -E '^(warning|error)' -A4 | head -40
git status --short
git add specs docs tools crates
git commit -m "docs(default-parameters): maturity row, spec amendments A-1..A-7, followups; corpus re-measured"
```

Expected: `fmt` clean, no new clippy warnings in the touched crates, and `git status` clean afterwards (no `node_modules`).
