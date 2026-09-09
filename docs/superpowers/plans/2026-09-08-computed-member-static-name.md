# Computed Member Static Name Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A computed member index the compiler cannot read becomes a value it refuses (E5506, in both `kali check` and `kali run`) rather than a property name it invents, and a `const`-bound literal key folds to the dot spelling; this closes register entries R-59 and R-13 (both lanes) and the bracket-store family.

**Architecture:** The parser returns `Option<String>` and the AST member node carries `Option<String>`, so absence is a type. Below HIR a nameless computed member gets its own node kind (`ComputedMember`) so every existing recognizer declines it by construction. The checker and codegen each get one read gateway and one store choke point that fold a bare `const`-with-literal-initializer identifier through one shared rule and otherwise emit one shared E5506 message.

**Tech Stack:** Rust workspace (`cargo`), black-box CLI cases under `crates/kali_cli/tests/cases/` run by the single `cases` target, `node v26.8.1` as the oracle, the blast-radius instrument under `tools/blast-radius/` (node) and `crates/kali_blast_radius`.

**Spec:** `docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md`

## Global Constraints

- One tree, one cargo target dir (`.cache/cargo-target`), no worktrees, no second target dir, no `cargo clean`. The pod has died of disk exhaustion before. "Before" numbers come from `git show <commit>:<path>`, never from building an old commit.
- Rust unit tests live in sibling `*_tests.rs` files wired with `#[cfg(test)] #[path = "x_tests.rs"] mod x_tests;`, never inline `#[cfg(test)]` modules.
- Black-box CLI tests are `.toml` case files under `crates/kali_cli/tests/cases/`; no new `tests/*.rs` targets.
- The E5506 messages are defined once, in `crates/kali_common/src/messages.rs`, and used verbatim by the checker and codegen. Their exact text (spec §4.4):
  - computed member: `computed member access \`o[k]\` is unavailable in the current phase unless the index is a literal or a compile-time-constant \`const\` binding, or the receiver is a runtime array or a \`for..in\` key over the same object`
  - string receiver: `indexing a string \`s[i]\` is unavailable in the current phase; use \`charAt\`/\`at\` on a statically-known ASCII string or the later compatibility path`
- The fold rule (spec §4.4 step 2): a **bare identifier** naming a **`const`** whose initializer is a **string or number literal**. Nothing else folds anywhere.
- The parser's readable set does not grow (spec §4.1).
- Commit subjects: `<type>(<scope>): <lowercase claim>`; every commit message ends with `Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h`.
- Between Task 2 and Task 7 the `cases` target is expected red **only** on the cases Task 7 flips (listed there). Every other task ends with its own crate's unit tests green and `cargo build --workspace` green. Do not edit a case file's expectation outside Task 7.
- Baseline for every "today" figure: `dc19c3a040`, node v26.8.1.
- `<SHA>`, `<list each file>` and `<before>→<after>` in Tasks 7 and 8 are **measured values, not placeholders**: each is filled from the commit the binary was built from, the red set of the run you just did, or the diff you just took. A plan step that leaves one unfilled has not been done.

---

### Task 1: `MemberExpression.property` becomes `Option<String>` (mechanical, behaviour-neutral)

**Files:**
- Modify: `crates/kali_ast/src/expression.rs:109-123`
- Create: `crates/kali_ast/src/expression_tests.rs`
- Modify: `crates/kali_parser/src/expression/call.rs:62,75,81,258,271,341`
- Modify: `crates/kali_hir/src/lowering/expression.rs:46-57`
- Modify: `crates/kali_cli/src/build/module_link.rs:413,1685,1724,5372`, `crates/kali_cli/src/build/exports/signatures.rs:357`
- Modify: every `.property` read in `crates/kali_types/src` (117 sites; list in the checker survey below) and every `property:` in its test modules (435 struct literals, `test_support.rs:16` and the `member!` macro at `test_support.rs:69-78`)
- Modify: parser tests that assert `member.property` (`crates/kali_parser/src/expression/call_tests/*.rs`)

**Interfaces:**
- Produces: `MemberExpression { object, property: Option<String>, computed_index }`, `MemberExpression::dot_name(&self) -> Option<&str>`, `MemberExpression::static_name(&self) -> Option<&str>`.
- After this task the parser still fabricates (it wraps the fabricated name in `Some`), so no behaviour changes. Task 2 makes it decline.

- [ ] **Step 1: Write the failing AST accessor test**

Create `crates/kali_ast/src/expression_tests.rs`:

```rust
use crate::*;

fn dot(name: &str) -> MemberExpression {
    MemberExpression {
        object: Expression::Identifier("o".to_string()),
        property: Some(name.to_string()),
        computed_index: None,
    }
}

fn computed(name: Option<&str>) -> MemberExpression {
    MemberExpression {
        object: Expression::Identifier("o".to_string()),
        property: name.map(str::to_string),
        computed_index: Some(Box::new(Expression::Identifier("i".to_string()))),
    }
}

#[test]
fn dot_access_always_has_both_names() {
    let member = dot("b");
    assert_eq!(member.dot_name(), Some("b"));
    assert_eq!(member.static_name(), Some("b"));
}

#[test]
fn a_readable_computed_index_has_a_static_name_but_no_dot_name() {
    let member = computed(Some("b"));
    assert_eq!(member.dot_name(), None);
    assert_eq!(member.static_name(), Some("b"));
}

#[test]
fn an_unreadable_computed_index_has_no_name_at_all() {
    let member = computed(None);
    assert_eq!(member.dot_name(), None);
    assert_eq!(member.static_name(), None);
}

#[test]
fn an_absent_property_field_deserializes_as_none() {
    let json = r#"{"object":{"Identifier":"o"},"computed_index":{"Identifier":"i"}}"#;
    let member: MemberExpression = serde_json::from_str(json).expect("absent property is None");
    assert_eq!(member.property, None);
}
```

Wire it at the top of `crates/kali_ast/src/expression.rs`, mirroring `builder.rs:6-8`:

```rust
#[cfg(test)]
#[path = "expression_tests.rs"]
mod expression_tests;
```

If `serde_json` is not a dev-dependency of `kali_ast`, add it to `[dev-dependencies]` in `crates/kali_ast/Cargo.toml` at the version the workspace already uses (`grep serde_json Cargo.toml crates/*/Cargo.toml`).

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_ast expression_tests`
Expected: compile error, `property: Option<String>` mismatched (field is `String`), no `dot_name`.

- [ ] **Step 3: Change the AST**

Replace `crates/kali_ast/src/expression.rs:111-122` with:

```rust
pub struct MemberExpression {
    pub object: Expression,
    /// `Some(name)`: the property name JavaScript will read is statically
    /// known — always for dot access, and for a computed access only when the
    /// parser could read the index (a string or number literal, or a
    /// parenthesized, sequence-last or `+`/`-` unary form of one). `None`: a
    /// computed access whose index must be evaluated; `computed_index` is
    /// `Some`. There is no sentinel value — the parser used to fabricate one
    /// (register entry R-59), and this field is an `Option` so it cannot.
    #[serde(default)]
    pub property: Option<String>,
    /// Structured index expression for computed access `a[<expr>]`.
    ///
    /// `None` for dot access (`a.b`). `Some(expr)` for bracket access, so
    /// codegen can evaluate arithmetic/dynamic indices (e.g. `a[i + 1]`) at
    /// runtime rather than relying on `property`.
    #[serde(default)]
    pub computed_index: Option<Box<Expression>>,
}

impl MemberExpression {
    /// The property name of a DOT access, `None` for any computed access. For
    /// sites that are only correct on the dot form.
    pub fn dot_name(&self) -> Option<&str> {
        if self.computed_index.is_none() {
            self.property.as_deref()
        } else {
            None
        }
    }

    /// The statically known property name: dot access, or a computed access
    /// whose index the parser could read. `None` means the index must be
    /// evaluated, and a consumer that formats or compares the name must
    /// decline rather than substitute anything.
    pub fn static_name(&self) -> Option<&str> {
        self.property.as_deref()
    }
}
```

- [ ] **Step 4: Migrate the parser (still fabricating, wrapped in `Some`)**

In `crates/kali_parser/src/expression/call.rs`:
- line 62: `property: Some(index_str),`
- line 75: `property: Some(prop_name),`
- line 81: `property: Some("unknown".to_string()),`
- line 258: `property: Some(prop_name),`
- line 271: `property: Some(index_str),`
- line 341, `member_access_name`: `Some(format!("{object}.{}", member.static_name()?))`

- [ ] **Step 5: Migrate HIR lowering (final form)**

Replace `crates/kali_hir/src/lowering/expression.rs:46-57` with:

```rust
            Expression::MemberExpression(expr) => {
                // A member with a statically known name carries it as text. A
                // computed access the parser could not name carries NO text:
                // the index is its second child and is the only description of
                // the access (register R-59; spec §4.3).
                let id = match &expr.property {
                    Some(name) => {
                        self.builder
                            .alloc_text(HirNodeKind::MemberExpr, None, name.clone())
                    }
                    None => self.builder.alloc(HirNodeKind::MemberExpr, None),
                };
                push_child!(self, id, self.lower_expression(&expr.object));
                // Computed access `a[<expr>]` carries the structured index as a
                // second child; dot access (`a.b`) keeps a single `[object]` child.
                if let Some(index) = &expr.computed_index {
                    push_child!(self, id, self.lower_expression(index));
                }
                id
            }
```

- [ ] **Step 6: Migrate `kali_cli`**

- `crates/kali_cli/src/build/exports/signatures.rs:357`: `Some(format!("{object}.{}", member.static_name()?))`
- `crates/kali_cli/src/build/module_link.rs:413`: `if member.dot_name() == Some("freeze")`
- `module_link.rs:1685` (already behind the `computed_index.is_some()` refusal at 1675): `let Some(property) = member.dot_name() else { return None; };` then `module.exports.contains_key(property)`.
- `module_link.rs:1724`: `let Some(property) = member.dot_name().map(str::to_string) else { return; };` (this function returns `()` after pushing a diagnostic; keep that shape).
- `module_link.rs:5372` (a `MemberExpression { .. }` construction): `property: Some(<the existing expression>),`.

- [ ] **Step 7: Migrate `kali_types` sources by rule**

Run `cargo build -p kali_types 2>&1 | grep -c "^error"` to get the site count, then fix each site by exactly one of these rules. The checker survey classified all 117; the (N) sites are named here so none is guessed.

Rule A — a site already inside a `computed_index.is_none()` / `is_some()` guard (26 sites: `resolve/expression.rs:102,114,749,1053,1123,1173,1376,1479,1483`; `late_host.rs:124,218`; `static_analysis/string.rs:978`; `growable.rs:336,733`; `repr_infer.rs:54,65,2931,2984,3000,3147,6577,6614,6682`): replace `member.property.as_str()` / `member.property` with `member.dot_name()` and compare against `Some("…")`. Example, `resolve/expression.rs:102`:

```rust
        if member.computed_index.is_some() || member.dot_name() != Some("argv") {
```

Rule B — the three name builders in `resolve/member.rs:45-73` and `member_object_name` at `:187`:

```rust
    pub(crate) fn member_access_name(expr: &MemberExpression) -> Option<String> {
        let object_name = Self::member_access_root_name(&expr.object)?;
        Some(format!("{}.{}", object_name, expr.static_name()?))
    }
    pub(crate) fn member_access_name_bracketed(expr: &MemberExpression) -> Option<String> {
        let object_name = Self::member_access_bracketed_root_name(&expr.object)?;
        Some(format!("{}[\"{}\"]", object_name, expr.static_name()?))
    }
    pub(crate) fn member_access_name_single_quoted(expr: &MemberExpression) -> Option<String> {
        let object_name = Self::member_access_single_quoted_root_name(&expr.object)?;
        Some(format!("{}['{}']", object_name, expr.static_name()?))
    }
```

Rule C — the `.unwrap_or_else(|| expr.property.clone())` idiom (`late_host.rs:455,483,500,519,545,563,622,653,686,744,764`, `resolve/call.rs:457`, `resolve/expression.rs:1921`): a nameless member has no dotted name, so the fallback is the empty string, which matches no host name and therefore declines:

```rust
        let dotted = Self::member_access_name(expr)
            .unwrap_or_else(|| expr.static_name().unwrap_or_default().to_string());
```

Rule D — `let method = member.property.as_str();` at the head of a resolver (`static_analysis/string.rs:205,313,365,421,469,512,534,572,620,661,696,731,766,802,834,874,924`; `static_analysis/array.rs:216,229,242,261,546,672,721,772,813,1002,1030`; `static_analysis/promise.rs:12`; `repr_infer.rs:4143`): decline with the function's own "not this shape" value:

```rust
        let Some(method) = member.static_name() else {
            return; // or `return false;` / `return None;` — whatever the function returns for "not this shape"
        };
```

Rule E — `matches!(expr.property.as_str(), "a" | "b")` and `member.property == "x"` comparisons (`late_host.rs:315,327,334,344-415,451,478,731-750,773,777,821`; `resolve/member.rs:52`; `static_analysis/promise.rs:32`; `growable.rs:514`; `repr_infer.rs:4070,4127,6460,6513,6528`): `matches!(expr.static_name(), Some("a" | "b"))` and `member.static_name() == Some("x")`. For `repr_infer.rs:6466`, which returns `&member.property`, return `member.static_name()` (the function's return type becomes `Option<&str>` if it is not already).

Rule F — `repr_table.shape_field(shape, &member.property)` (`resolve/expression.rs:662,707,709`): `let Some(field) = member.static_name() else { return false; /* or None */ };` then `shape_field(shape, field)`.

Rule G — the two dot branches in `repr_infer.rs` that run only when `computed_index` is `None` (`:3939` store field, `:4081` read field): the invariant guarantees a name there:

```rust
                    field: member
                        .dot_name()
                        .expect("a non-computed member always carries its name (kali_ast::MemberExpression invariant)")
                        .to_string(),
```

Rule H — `growable.rs:807`:

```rust
                        if member
                            .static_name()
                            .is_some_and(|name| name == "length" || name.parse::<u64>().is_ok())
                        {
                            return;
                        }
```

Rule I — the `member!` macro and the builder in `crates/kali_types/src/test_support.rs:16,69-78`: `property: Some($prop.to_string())` / `property: Some("Math".to_string())`.

- [ ] **Step 8: Build the workspace**

Run: `cargo build --workspace`
Expected: success. If any crate other than those named above fails, apply Rule A–E by the same logic and record the file in the commit message.

- [ ] **Step 9: Commit the source migration**

```bash
git add crates/kali_ast crates/kali_parser/src crates/kali_hir crates/kali_cli/src crates/kali_types/src
git reset crates/kali_types/src/**/*_tests* crates/kali_types/src/*_tests.rs crates/kali_parser/src/**/*_tests* 2>/dev/null
git commit -m "refactor(ast): MemberExpression.property is an Option, and every consumer decides

Behaviour-neutral: the parser still fabricates a name for an unreadable index
and wraps it in Some. What changes is the type — a computed access with no
static name is now representable, and every name-reading site in kali_types,
kali_cli, kali_hir and the parser either uses dot_name()/static_name() or
declines on None. The test-literal churn lands in the next commit.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

(`cargo test` does not compile at this commit; `cargo build --workspace` does. The next commit restores it.)

- [ ] **Step 10: Migrate the test struct literals mechanically**

```bash
grep -rl 'property: ' crates/kali_types/src crates/kali_parser/src crates/kali_cli/src --include='*.rs' \
  | xargs perl -0pi -e 's/property: ("[^"]*"\.to_string\(\)|[a-z_]+\.to_string\(\)|[a-z_]+\.clone\(\))/property: Some($1)/g'
grep -rl '\.property, "' crates/kali_parser/src crates/kali_types/src --include='*.rs' \
  | xargs perl -pi -e 's/assert_eq!\(([\w.]+)\.property, "([^"]*)"\)/assert_eq!($1.property.as_deref(), Some("$2"))/g'
```

Then `cargo test --workspace --no-run 2>&1 | grep -E "^error" -A5` and fix by hand every remaining site the regexes did not match (e.g. `property: helper.to_string()` inside a loop, `&member.property` comparisons). Do not touch `rationale`/docs.

- [ ] **Step 11: Run the whole workspace**

Run: `cargo test --workspace --no-fail-fast 2>&1 | tail -30`
Expected: green. This commit is behaviour-neutral, so any red test is a migration mistake, not a flip.

- [ ] **Step 12: Commit the test migration**

```bash
git add -A crates
git commit -m "test: wrap every MemberExpression property literal in Some

Mechanical companion to the previous commit: 435 struct literals in
kali_types' test modules, the member! macro, and the parser's property
assertions. No expectation changes.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

---

### Task 2: The parser declines, and a nameless member gets its own kind below HIR

**Files:**
- Modify: `crates/kali_parser/src/literal.rs:113-138` (+ its doc comment), `crates/kali_parser/src/expression/call.rs:55-66,264-275`
- Create: `crates/kali_parser/src/literal_tests.rs`
- Modify: `crates/kali_parser/src/expression/call_tests/optional_chain.rs` (`test_parse_optional_chain_index_expression`)
- Modify: `crates/kali_hir/src/lowering/expression_tests.rs`
- Modify: `crates/kali_mir/src/node.rs:7-17`, `crates/kali_mir/src/lower.rs:46`
- Modify: `crates/kali_lir/src/node.rs:7-16`, `crates/kali_lir/src/lower.rs:55-66`
- Modify: `crates/kali_common/src/messages.rs`
- Modify: `crates/kali_codegen/src/emit/control_flow.rs:1859-1910`, `crates/kali_codegen/src/emit/call.rs:220-227,5432-5441`
- Create: `crates/kali_codegen/src/emit/computed_member.rs` (this task: the message-only deny arm; Task 3 fills it), `crates/kali_codegen/src/emit/computed_member_tests.rs`

**Interfaces:**
- Produces: `Parser::expression_to_property_name(&Expression) -> Option<String>`; `Parser::normalize_string_literal` becomes `pub` (Task 6 calls it from `kali_types`); `MirNodeKind::ComputedMember`; `LirNodeKind::ComputedMember`; `kali_common::computed_member_access_unavailable_message() -> &'static str`; `kali_common::string_index_access_unavailable_message() -> &'static str`; `FunctionEmitter::emit_computed_member(&mut self, function, id, node, want_value) -> EmittedValue` (deny-only until Task 3).

- [ ] **Step 1: Write the failing parser tests**

Create `crates/kali_parser/src/literal_tests.rs`:

```rust
use crate::test_support::lex;
use crate::Parser;
use kali_ast::{Expression, Statement};

/// Parses `<source>` (one expression statement that is a computed member
/// access) and returns the member's static property name.
fn computed_member_property(source: &str) -> Option<String> {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics for {source}: {:?}",
        output.diagnostics
    );
    let Statement::ExpressionStatement(stmt) = &output.statements[0] else {
        panic!("expected an expression statement for {source}, got {:?}", output.statements[0]);
    };
    let Expression::MemberExpression(member) = stmt.expression.as_ref() else {
        panic!("expected a member expression for {source}, got {:?}", stmt.expression);
    };
    assert!(member.computed_index.is_some(), "{source} must parse as a computed access");
    member.property.clone()
}

#[test]
fn a_readable_index_keeps_its_name() {
    for (source, name) in [
        ("o[\"b\"];", "b"),
        ("o['b'];", "b"),
        ("o[1];", "1"),
        ("o[1.5];", "1.5"),
        ("o[(1)];", "1"),
        ("o[(0, 1)];", "1"),
        ("o[+1];", "1"),
        ("o[-1];", "-1"),
        ("o[\"\"];", ""),
    ] {
        assert_eq!(computed_member_property(source).as_deref(), Some(name), "{source}");
    }
}

#[test]
fn an_unreadable_index_has_no_name_and_no_fallback() {
    for source in [
        "o[i];",
        "o[i + 0];",
        "o[(i)];",
        "o[(0, i)];",
        "o[+i];",
        "o[-i];",
        "o[true];",
        "o[null];",
        "o[1n];",
        "o[f()];",
        "o[\"a\" + \"b\"];",
    ] {
        assert_eq!(computed_member_property(source), None, "{source} must decline, not fabricate");
    }
}
```

Wire it at the bottom of `crates/kali_parser/src/literal.rs`:

```rust
#[cfg(test)]
#[path = "literal_tests.rs"]
mod literal_tests;
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_parser literal_tests`
Expected: `an_unreadable_index_has_no_name_and_no_fallback` fails with `Some("i")` / `Some("index")`.

- [ ] **Step 3: Make the parser decline**

Replace `crates/kali_parser/src/literal.rs:113-138` (the function body) with:

```rust
    pub(crate) fn expression_to_property_name(expr: &Expression) -> Option<String> {
        match expr {
            Expression::ParenthesizedExpression(parenthesized) => {
                Self::expression_to_property_name(&parenthesized.expression)
            }
            Expression::SequenceExpression(sequence) => {
                Self::expression_to_property_name(sequence.expressions.last()?)
            }
            Expression::UnaryExpression(unary)
                if unary.operator == "+" || unary.operator == "-" =>
            {
                let value = Self::expression_to_property_name(&unary.argument)?
                    .parse::<f64>()
                    .ok()?;
                let value = if unary.operator == "+" { value } else { -value };
                // `format_js_number` renders both zeros as "0", so the signed
                // zero a `-0` index folds to needs no separate branch.
                Some(format_js_number(value))
            }
            Expression::Literal(LiteralValue::String(s)) => Some(Self::normalize_string_literal(s)),
            Expression::Literal(LiteralValue::Number(n)) => Some(format_js_number(*n)),
            _ => None,
        }
    }

    pub fn normalize_string_literal(value: &str) -> String {
```

Rewrite the doc comment above it (lines 45-112) to this, dropping the R-59 mechanism narrative and the "RECOMMENDATION, not done here" paragraph, but keeping the R-57 escape note and the one-formatter warning:

```rust
    /// The property name a computed member INDEX denotes, when this parser can
    /// read it: a string literal (delimiters stripped — escape sequences are
    /// NOT decoded, which is register entry R-57), a number literal rendered
    /// by `format_js_number`, and the parenthesized, sequence-last and `+`/`-`
    /// unary forms that recurse into one of those.
    ///
    /// `None` for every other shape — a bare identifier, a binary expression,
    /// a boolean/`null`/BigInt/regex literal, an empty sequence, a unary whose
    /// argument does not read as a number. There is NO fallback string: this
    /// function used to fabricate one (the identifier's own text, or the
    /// literal `"index"`), and the static lanes downstream read it as a real
    /// property name — register entry R-59, closed by the
    /// computed-member-static-name project
    /// (`docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md`).
    /// A `None` here is the whole of what the caller records; the structured
    /// index child is then the only description of the access, and the
    /// checker and codegen either fold it (a bare `const`-with-literal
    /// identifier) or refuse it (E5506).
    ///
    /// A NUMBER is rendered with `format_js_number`, the same function
    /// `kali_hir`'s `lower_property_name` stores a numeric KEY with, so a probe
    /// and the key it probes land on one spelling BY CONSTRUCTION. Do not
    /// reintroduce a second number formatter here, and do not translate
    /// between them at the comparison site.
```

Update both parse sites in `crates/kali_parser/src/expression/call.rs`: replace the R-59 comment block and line at 55-59 with

```rust
                    // The static name of the access, or None when the index is
                    // not one this parser reads (spec §4.1). The structured
                    // index below is then the only description of the access.
                    let property = Self::expression_to_property_name(&index);
                    expr = Expression::MemberExpression(Box::new(MemberExpression {
                        object: expr,
                        property,
                        computed_index: Some(Box::new(index)),
                    }));
```

and the same shape at 264-275 for `o?.[e]` (drop the "fabricates identically" comment).

- [ ] **Step 4: Run the parser tests**

Run: `cargo test -p kali_parser`
Expected: `literal_tests` green. `test_parse_optional_chain_index_expression` may now fail on its `property` assertion: change that assertion to `assert!(member.property.is_none(), "an identifier index has no static name");` (it previously recorded the fabricated `"expr"`). Any other red test in the parser crate is one that asserted a fabricated name; change its assertion to `None` and say so in the commit.

- [ ] **Step 5: Write the failing HIR test**

Append to `crates/kali_hir/src/lowering/expression_tests.rs`:

```rust
#[test]
fn a_computed_member_without_a_static_name_lowers_without_text() {
    let statements = crate::test_support::parse("o[i]; o[\"b\"]; o.c;");
    let expressions: Vec<&Expression> = statements
        .iter()
        .map(|statement| match statement {
            Statement::ExpressionStatement(stmt) => stmt.expression.as_ref(),
            other => panic!("expected an expression statement, got {other:?}"),
        })
        .collect();
    let mut lowerer = HirLowerer::new();

    let nameless = lowerer.lower_expression(expressions[0]);
    let node = &lowerer.builder.nodes[nameless.0 as usize];
    assert_eq!(node.kind, HirNodeKind::MemberExpr);
    assert_eq!(node.text, None, "o[i] has no static name and must carry no text");
    assert_eq!(node.children.len(), 2, "the index is the second child");

    let named = lowerer.lower_expression(expressions[1]);
    let node = &lowerer.builder.nodes[named.0 as usize];
    assert_eq!(node.text.as_deref(), Some("b"));
    assert_eq!(node.children.len(), 2);

    let dot = lowerer.lower_expression(expressions[2]);
    let node = &lowerer.builder.nodes[dot.0 as usize];
    assert_eq!(node.text.as_deref(), Some("c"));
    assert_eq!(node.children.len(), 1);
}
```

Add `use kali_ast::Statement;` to the file's imports if absent.

- [ ] **Step 6: Run the HIR tests**

Run: `cargo test -p kali_hir a_computed_member_without_a_static_name_lowers_without_text`
Expected: PASS already (Task 1 Step 5 landed the lowering). If it fails, the lowering is wrong; fix it, do not weaken the test.

- [ ] **Step 7: Add the shared messages**

Append to `crates/kali_common/src/messages.rs`:

```rust
/// Canonical feature-unavailable wording for a computed member access whose
/// index neither the parser nor the `const` fold can name, and which no
/// runtime lane admits. Used verbatim by `kali_types` (so `kali check`
/// refuses) and `kali_codegen` (so `kali build`/`run` refuse the same way).
/// Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.
pub const fn computed_member_access_unavailable_message() -> &'static str {
    "computed member access `o[k]` is unavailable in the current phase unless the index is a literal or a compile-time-constant `const` binding, or the receiver is a runtime array or a `for..in` key over the same object"
}

/// Canonical feature-unavailable wording for indexing a statically-known
/// string, in both the literal (`s[1]`) and the folded (`const k = 1; s[k]`)
/// spelling. The literal spelling was a silent `0` before; there is no
/// character fold here on purpose (spec §4.4, "String receivers").
pub const fn string_index_access_unavailable_message() -> &'static str {
    "indexing a string `s[i]` is unavailable in the current phase; use `charAt`/`at` on a statically-known ASCII string or the later compatibility path"
}
```

- [ ] **Step 8: Add the MIR and LIR kinds**

`crates/kali_mir/src/node.rs`, in `pub enum MirNodeKind` after `Expr,`:

```rust
    /// A computed member access whose index has no static name (a HIR
    /// `MemberExpr` with no text). Kept distinct from `Expr` because kinds are
    /// erased below HIR and a text-less two-child `Expr` would be
    /// indistinguishable from a two-element array literal (spec §4.3).
    ComputedMember,
```

`crates/kali_mir/src/lower.rs:46`:

```rust
        let kind = if node.kind == HirNodeKind::MemberExpr && node.text.is_none() {
            MirNodeKind::ComputedMember
        } else {
            map_kind(&node.kind)
        };
```

`crates/kali_lir/src/node.rs`, in `pub enum LirNodeKind` after `Value,`:

```rust
    /// A computed member access `[object, index]` with no static property
    /// name. Every recognizer that matches `Value` declines this kind by
    /// construction; codegen's `emit_computed_member` is its only consumer.
    ComputedMember,
```

`crates/kali_lir/src/lower.rs` `map_kind`: add `MirNodeKind::ComputedMember => LirNodeKind::ComputedMember,`.

- [ ] **Step 9: Add the codegen arms (deny-only) and the exhaustive-match arms**

Create `crates/kali_codegen/src/emit/computed_member.rs`:

```rust
//! A computed member access with no static property name
//! (`LirNodeKind::ComputedMember`): the one place such a node is emitted.
//!
//! Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.

use super::*;
use kali_common::computed_member_access_unavailable_message;

impl FunctionEmitter<'_> {
    /// Read gateway for a nameless computed member. Filled in by Task 3 of the
    /// plan; until then every such access refuses, which is already strictly
    /// more honest than the fabricated name it replaces.
    pub(crate) fn emit_computed_member(
        &mut self,
        function: &mut Function,
        _id: LirNodeId,
        _node: &LirNode,
        _want_value: bool,
    ) -> EmittedValue {
        self.deny_e5506(function, computed_member_access_unavailable_message())
    }
}

#[cfg(test)]
#[path = "computed_member_tests.rs"]
mod computed_member_tests;
```

Declare the module beside its siblings (`grep -n "^mod \|^pub(crate) mod " crates/kali_codegen/src/emit/mod.rs` shows the list; add `mod computed_member;` in alphabetical position). Check the `use super::*;` line matches what `emit/object.rs` imports; copy that file's import block if `super::*` does not bring `Function`, `LirNode`, `LirNodeId`, `EmittedValue` into scope.

In `crates/kali_codegen/src/emit/control_flow.rs` `emit_node`'s kind match (line ~1859), add after the `LirNodeKind::Value` arm:

```rust
            LirNodeKind::ComputedMember => {
                self.emit_computed_member(function, id, &node, want_value)
            }
```

In `crates/kali_codegen/src/emit/call.rs:220-227` and `:5432-5441`, add `| LirNodeKind::ComputedMember` to the `=> false` arms (a nameless member is not a callback shape and has no callback provenance).

Run `cargo build --workspace 2>&1 | grep -B2 -A8 "non-exhaustive"` and add a decline arm to every other exhaustive match the compiler names (the survey found none beyond these in codegen; `kali_optimize` and `kali_lir` may have some).

- [ ] **Step 10: Write the failing LIR-shape and refusal tests**

Create `crates/kali_codegen/src/emit/computed_member_tests.rs`:

```rust
use crate::test_support::parse_and_lower_lir;
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_error::diagnostic::Diagnostic;
use kali_lir::LirNodeKind;

pub(crate) fn diagnostics_for(source: &str) -> Vec<Diagnostic> {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    lower_lir_to_wasm(&mut ctx, &program).diagnostics
}

pub(crate) fn assert_e5506(diagnostics: &[Diagnostic], needle: &str, context: &str) {
    assert!(
        diagnostics.iter().any(|d| d.is_error() && d.code == Some(5506) && d.message.contains(needle)),
        "{context}: expected an E5506 containing {needle:?}, got {diagnostics:?}"
    );
}

#[test]
fn a_nameless_computed_member_lowers_to_its_own_kind() {
    let program = parse_and_lower_lir("const o = {index: 9, i: 7}; let i = 1; o[i]; o[\"b\"];");
    let nameless: Vec<_> = program
        .nodes
        .iter()
        .filter(|node| node.kind == LirNodeKind::ComputedMember)
        .collect();
    assert_eq!(nameless.len(), 1, "exactly one nameless computed member: o[i]");
    assert_eq!(nameless[0].text, None);
    assert_eq!(nameless[0].children.len(), 2);
    assert!(
        program.nodes.iter().any(|node| node.kind == LirNodeKind::Value
            && node.text.as_deref() == Some("b")
            && node.children.len() == 2),
        "o[\"b\"] stays a named two-child Value"
    );
}

#[test]
fn a_nameless_computed_member_read_refuses_instead_of_reading_a_fabricated_name() {
    let diagnostics =
        diagnostics_for("const o = {index: 9, i: 7}; let i = 1; console.log(o[i]); console.log(o[i + 0]);");
    assert_e5506(
        &diagnostics,
        "computed member access `o[k]` is unavailable",
        "R-59's repro",
    );
}
```

Check the exact import paths against `crates/kali_codegen/src/emit/object_tests.rs:85-109` (`CodegenCtx`, `TargetConfig`, `lower_lir_to_wasm`, `Diagnostic`) and adjust to match that file.

- [ ] **Step 11: Run the tests**

Run: `cargo test -p kali_codegen computed_member_tests && cargo test -p kali_mir && cargo test -p kali_lir && cargo test -p kali_hir && cargo test -p kali_parser`
Expected: all green.

- [ ] **Step 12: Commit**

```bash
git add crates/kali_parser crates/kali_hir crates/kali_mir crates/kali_lir crates/kali_common crates/kali_codegen
git commit -m "fix(parser): an unreadable computed index is declined, and gets its own kind below HIR

expression_to_property_name returns None for every shape it cannot read;
there is no fallback string. A nameless MemberExpr lowers to
MirNodeKind::ComputedMember / LirNodeKind::ComputedMember so it cannot be
mistaken for a two-element array literal. Codegen refuses it with the
shared E5506 for now; the fold and the runtime lanes come next.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

---

### Task 3: The codegen read gateway: runtime lanes, fold, refuse

**Files:**
- Modify: `crates/kali_codegen/src/emit/computed_member.rs`, `computed_member_tests.rs`
- Modify: `crates/kali_codegen/src/emit/control_flow.rs:3019-3021` (string-receiver refusal before `emit_unary`)
- Modify: `crates/kali_codegen/src/intrinsics/host.rs:718` (`render_static_value`), `:1306` (`render_length`)

**Interfaces:**
- Consumes: `resolve_static_object_identity_value(&self, id) -> Option<StaticObjectIdentityValue>` (`intrinsics/object.rs:255`), `computed_forin_object_access(&self, &LirNode)` (`emit/object.rs:204`), `growable_array_read_base`, `growable_field_read_base`, `emit_growable_index_read`, `dynamic_array_read_base(&self, &LirNode) -> Option<String>` (`control_flow.rs:1926`), `emit_dynamic_array_read_node`, `emit_object_field_read_dynamic`, `deny_e5506` (`intrinsics/host.rs:1624`), `format_js_number`.
- Produces: `FunctionEmitter::static_member_name(&self, node: &LirNode) -> Option<String>` (Task 4 uses it), `FunctionEmitter::named_twin(node: &LirNode, name: String) -> LirNode`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/kali_codegen/src/emit/computed_member_tests.rs`:

```rust
use kali_lir::LirNode;

fn program_prints(source: &str) -> (Vec<Diagnostic>, String) {
    let program = parse_and_lower_lir(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("printable wasm");
    (result.diagnostics, printed)
}

#[test]
fn a_const_string_key_folds_to_the_literal_spelling() {
    // `o[k]` with `const k = "b"` must compile exactly as `o["b"]` does: no
    // diagnostic, and the folded read reaches the object-literal field fold.
    let (folded, _) = program_prints("const o = {a: 1, b: 2}; const k = \"b\"; console.log(o[k] + 1);");
    let (literal, _) = program_prints("const o = {a: 1, b: 2}; console.log(o[\"b\"] + 1);");
    assert!(folded.iter().all(|d| !d.is_error()), "folded read must not error: {folded:?}");
    assert_eq!(
        folded.iter().filter(|d| d.is_error()).count(),
        literal.iter().filter(|d| d.is_error()).count()
    );
}

#[test]
fn a_const_number_key_folds_into_the_static_element_fold() {
    let (diagnostics, _) = program_prints("const a = [5, 6]; const i = 1; console.log(a[i]);");
    assert!(diagnostics.iter().all(|d| !d.is_error()), "{diagnostics:?}");
}

#[test]
fn a_let_key_refuses_even_when_never_reassigned() {
    let diagnostics = diagnostics_for("const o = {a: 1, b: 2}; let k = \"b\"; console.log(o[k]);");
    assert_e5506(&diagnostics, "computed member access `o[k]` is unavailable", "let key");
}

#[test]
fn a_mutable_index_over_an_array_literal_refuses() {
    let diagnostics = diagnostics_for("const a = [5, 6, 7]; for (let j = 0; j < 3; j++) console.log(a[j]);");
    assert_e5506(&diagnostics, "computed member access `o[k]` is unavailable", "array literal, mutable index");
}

#[test]
fn a_runtime_array_keeps_its_dynamic_index_lane() {
    let diagnostics = diagnostics_for(
        "const a = new Array(3); for (let i = 0; i < 3; i++) { a[i] = i * 2; } let j = 1; console.log(a[j]);",
    );
    assert!(
        !diagnostics.iter().any(|d| d.message.contains("computed member access")),
        "the linear-memory lane must still admit a runtime array: {diagnostics:?}"
    );
}

#[test]
fn a_string_receiver_refuses_in_both_spellings() {
    let literal = diagnostics_for("const s = \"abc\"; console.log(s[1]);");
    assert_e5506(&literal, "indexing a string `s[i]` is unavailable", "literal index on a string");
    let folded = diagnostics_for("const s = \"abc\"; const k = 1; console.log(s[k]);");
    assert_e5506(&folded, "indexing a string `s[i]` is unavailable", "folded index on a string");
}

#[test]
fn a_chained_access_off_a_nameless_member_refuses_rather_than_rendering_the_child_count() {
    // Measured at dc19c3a040: `o[k].length` prints `2` for EVERY string
    // ("xyz" and "xyzwv" both print 2), because `render_length`'s text-less
    // arm returns the member node's CHILD COUNT. It is not a working lane
    // being regressed — it is a silent miscompile, and refusing is strictly
    // better. (The dot spelling `o.a.length` prints `1` by the same arm and
    // is NOT fixed here — see the follow-up recorded in Task 8.)
    for source in [
        "const o = {a: \"xyz\"}; const k = \"a\"; console.log(o[k].length);",
        "const o = {a: \"xyzwv\"}; const k = \"a\"; console.log(o[k].length);",
    ] {
        let diagnostics = diagnostics_for(source);
        assert!(
            diagnostics.iter().any(|d| d.is_error() && d.code == Some(5506)),
            "{source}: by-id consumers see the nameless kind and must decline: {diagnostics:?}"
        );
    }
}

#[test]
fn the_static_renderers_decline_a_nameless_member() {
    // `render_static_value` and `render_length` render a text-less two-child
    // node as its CHILD COUNT — the string "2". Measured at dc19c3a040:
    // `Object[k](o).length` prints 2 for a four-key object, and
    // `o[k].length` prints 2 for every string. A ComputedMember must never
    // reach that arm; it declines, and the access then refuses.
    let program = parse_and_lower_lir("const o = {a: 1}; let k = \"a\"; Object.hasOwn(o, o[k]);");
    let node: &LirNode = program
        .nodes
        .iter()
        .find(|node| node.kind == LirNodeKind::ComputedMember)
        .expect("o[k] with a let key is nameless");
    assert_eq!(node.text, None);

    let diagnostics =
        diagnostics_for("const o = {a: 1, b: 2, c: 3, d: 4}; const k = \"keys\"; console.log(Object[k](o).length);");
    assert!(
        diagnostics.iter().any(|d| d.is_error() && d.code == Some(5506)),
        "Object[k] must refuse, not render the child count: {diagnostics:?}"
    );
}
```

(`wasmprinter` is already a dev-dependency of `kali_codegen`; see `intrinsics/number_tests.rs:24`.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_codegen computed_member_tests`
Expected: the fold tests fail (they refuse today), the string-receiver literal test fails (silent `0` today).

- [ ] **Step 3: Implement the resolver, the named twin, and the gateway**

Replace the body of `crates/kali_codegen/src/emit/computed_member.rs` with:

```rust
//! A computed member access with no static property name
//! (`LirNodeKind::ComputedMember`): the one place such a node is emitted.
//!
//! Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.

use super::*;
use crate::ctx::StaticObjectIdentityValue;
use kali_common::js_number::format_js_number;
use kali_common::{computed_member_access_unavailable_message, string_index_access_unavailable_message};

impl FunctionEmitter<'_> {
    /// The property name a member node denotes statically: its text when it
    /// has one, else the fold of its index child. The fold admits exactly a
    /// childless `Value` (a bare identifier) whose `const` binding resolves
    /// to a static string or number — `bindings` is `const`-only by
    /// construction, and a `let`/`var` lives in `locals` and never resolves
    /// here. A number is rendered by `format_js_number`, the same formatter
    /// HIR stores a numeric key with, so probe and key stay one currency.
    pub(crate) fn static_member_name(&self, node: &LirNode) -> Option<String> {
        if let Some(text) = node.text.as_deref() {
            return Some(text.to_string());
        }
        let index = *node.children.get(1)?;
        let index_node = self.node(index);
        if index_node.kind != LirNodeKind::Value || !index_node.children.is_empty() {
            return None;
        }
        match self.resolve_static_object_identity_value(index)? {
            StaticObjectIdentityValue::String(value) => Some(value),
            StaticObjectIdentityValue::Number(value) => Some(format_js_number(value)),
            _ => None,
        }
    }

    /// `node` as the ordinary `Value`-shaped member the rest of codegen
    /// understands, carrying `name` as its text. Used to re-dispatch a folded
    /// access through the lanes the literal spelling takes; the original node
    /// is not changed (the emitter borrows the program immutably), which is
    /// why by-id consumers keep seeing the nameless kind and decline.
    pub(crate) fn named_twin(node: &LirNode, name: String) -> LirNode {
        let mut twin = node.clone();
        twin.kind = LirNodeKind::Value;
        twin.text = Some(name);
        twin
    }

    /// True when `id` resolves to a statically-known string, the receiver
    /// shape that has no admitting index lane in either spelling.
    pub(crate) fn is_static_string_receiver(&self, id: LirNodeId) -> bool {
        matches!(
            self.resolve_static_object_identity_value(id),
            Some(StaticObjectIdentityValue::String(_))
        )
    }

    /// Read gateway for a nameless computed member (spec §4.4, in order):
    /// 1. the runtime lanes that never read the text for a name, offered a
    ///    `Value`-shaped text-less probe;
    /// 2. the fold;
    /// 3. dot semantics — the named twin re-dispatched through `emit_value`;
    /// 4. refuse.
    pub(crate) fn emit_computed_member(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        let mut probe = node.clone();
        probe.kind = LirNodeKind::Value;
        probe.text = None;

        if let Some((base, index, elem)) = self.computed_forin_object_access(&probe) {
            return self.emit_object_field_read_dynamic(function, base, index, elem);
        }
        if self.growable_array_read_base(&probe).is_some() || self.growable_field_read_base(&probe) {
            return self.emit_growable_index_read(function, node.children[0], node.children[1]);
        }
        if let Some(base_name) = self.dynamic_array_read_base(&probe) {
            return self.emit_dynamic_array_read_node(
                function,
                node.children[0],
                node.children[1],
                &base_name,
            );
        }
        if self.is_static_string_receiver(node.children[0]) {
            return self.deny_e5506(function, string_index_access_unavailable_message());
        }
        let Some(name) = self.static_member_name(node) else {
            return self.deny_e5506(function, computed_member_access_unavailable_message());
        };
        let twin = Self::named_twin(node, name);
        self.emit_value(function, id, &twin, want_value)
    }
}

#[cfg(test)]
#[path = "computed_member_tests.rs"]
mod computed_member_tests;
```

Check the `StaticObjectIdentityValue` path (`crates/kali_codegen/src/ctx.rs:6-13`) and the exact signatures of `growable_array_read_base`, `growable_field_read_base`, `emit_growable_index_read`, `emit_dynamic_array_read_node`, `emit_object_field_read_dynamic` as they are called in `control_flow.rs:2984-3019`, and match them.

- [ ] **Step 4: Refuse a string receiver in the literal spelling, guard the two renderers, and deny a nameless base**

In `crates/kali_codegen/src/emit/control_flow.rs`, immediately before `self.emit_unary(function, node)` at the end of the `2 =>` arm (line ~3021), add:

```rust
                // A statically-known string has no index lane in either
                // spelling (`s[1]` was a silent `0`; `s[k]` folds onto the
                // same lane). Refuse here so the literal spelling and the
                // folded twin agree (spec §4.4, "String receivers").
                if self.is_static_string_receiver(node.children[0]) {
                    return self.deny_e5506(function, string_index_access_unavailable_message());
                }
```

(add `use kali_common::string_index_access_unavailable_message;` to the file's imports.)

In `crates/kali_codegen/src/intrinsics/host.rs`, at the top of `render_static_value` (line 718, right after `let node = self.node(id);` or equivalent) and at the top of `render_length` (line 1306):

```rust
        // A nameless computed member has no static value and no static
        // length; the text-less arm below would otherwise render its CHILD
        // COUNT. Measured at dc19c3a040: `o[k].length` printed `2` for every
        // string and `Object[k](o).length` printed `2` for a four-key object.
        // Spec §4.3: by-id consumers decline the kind.
        if node.kind == LirNodeKind::ComputedMember {
            return None;
        }
```

And in `crates/kali_codegen/src/emit/operators.rs`, at the top of `emit_unary` (line 124, right after `let arg = node.children[0];`):

```rust
        // A member read whose BASE is a nameless computed member cannot be
        // resolved: the base's own fold is visible only through the read
        // gateway's re-dispatch, and this node reaches its base by id. Deny
        // rather than fall through to the renderers' child-count arm or the
        // placeholder tail (spec §4.5, chained access).
        if self.node(arg).kind == LirNodeKind::ComputedMember {
            return self.deny_e5506(function, computed_member_access_unavailable_message());
        }
```

(add `use kali_common::computed_member_access_unavailable_message;` to `operators.rs`'s imports.)

- [ ] **Step 5: Pin that the binding-free passes decline**

Spec §4.3 says the passes with no binding knowledge decline on a nameless
member. The survey verified each declines by construction (they compare
`text.as_deref()` against `Some("join")`/`Some("substring")`/a field name, and
`None` matches none of them), so this step pins that rather than changing it.
Append to `crates/kali_optimize/src/object_fold_tests/object_has_own.rs`:

```rust
#[test]
fn a_nameless_computed_member_folds_as_no_static_method_name() {
    // `Object[k]` used to build the name "Object.<fabricated>"; a nameless
    // member has no name at all, so the fold declines. Built by hand because
    // this crate's tests do not lower from source.
    let mut builder = LirBuilder::new();
    let object = builder.alloc(LirNodeKind::Value);
    builder.node_mut(object).unwrap().text = Some("Object".to_string());
    let index = builder.alloc(LirNodeKind::Value);
    builder.node_mut(index).unwrap().text = Some("k".to_string());
    let member = builder.alloc(LirNodeKind::ComputedMember);
    builder.node_mut(member).unwrap().children = vec![object, index];
    let root = builder.alloc(LirNodeKind::Program);
    builder.node_mut(root).unwrap().children = vec![member];
    let program = LirProgram { root, nodes: builder.into_nodes() };
    let optimizer = Optimizer::new(OptimizationLevel::Release);
    assert_eq!(
        optimizer.member_access_name(&program, &program.nodes[member.0 as usize]),
        None,
        "a nameless computed member has no dotted name"
    );
}
```

Match the imports and the `Optimizer` construction to the file's existing tests
(`object_fold_tests/object_has_own.rs:3-20`); if `member_access_name` is not
reachable from that module, put the test beside it in
`crates/kali_optimize/src/helpers_tests.rs` instead, creating and wiring that
file the same way.

Run: `cargo test -p kali_optimize`
Expected: PASS.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p kali_codegen && cargo test -p kali_optimize && cargo test -p kali_mir`
Expected: `computed_member_tests` green; the rest of the crates green. A red elsewhere in these crates means a fixture asserted a silent `0` for `s[1]`, a fabricated read, or a `.length` child count; re-pin it in this task only if it is a unit test in one of these crates, and record it in the commit message.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen
git commit -m "feat(codegen): the computed-member gateway folds a const key and refuses the rest

emit_computed_member offers the runtime lanes a text-less probe, folds a
bare const-with-literal identifier through resolve_static_object_identity_value,
re-dispatches the named twin, and otherwise denies with the shared E5506.
A statically-known string receiver refuses in both spellings, and the two
static renderers decline the nameless kind instead of rendering \"2\".

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

---

### Task 4: The codegen store choke point: the dot arm for a named bracket store, E5506 otherwise

**Files:**
- Modify: `crates/kali_codegen/src/emit/literal.rs:344-423` (extract), `:599-609`, `:613-635`, `:744-768` (insert)
- Modify: `crates/kali_codegen/src/emit/computed_member.rs` (`store_target_node`), `computed_member_tests.rs`

**Interfaces:**
- Consumes: `static_member_name`, `named_twin` (Task 3), `object_shape_of_node`, `repr_table.shape_field`, `is_binary_operator_text`.
- Produces: `FunctionEmitter::store_target_node(&self, left: LirNodeId) -> LirNode`; `FunctionEmitter::try_emit_shaped_field_store(&mut self, function, target: &LirNode, right: LirNodeId) -> bool`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/kali_codegen/src/emit/computed_member_tests.rs`:

```rust
#[test]
fn a_bracket_store_on_an_array_literal_refuses_instead_of_vanishing() {
    let diagnostics = diagnostics_for("const a = [5, 6]; a[1] = 9; console.log(a[1]);");
    assert!(
        diagnostics.iter().any(|d| d.is_error() && d.code == Some(5506)),
        "a[1] = 9 on an array literal has no store lane and must refuse: {diagnostics:?}"
    );
}

#[test]
fn a_nameless_bracket_store_refuses_with_the_shared_message() {
    let diagnostics = diagnostics_for("const o = {a: 1, b: 2}; let k = \"b\"; o[k] = 8; console.log(o.b);");
    assert_e5506(&diagnostics, "computed member access `o[k]` is unavailable", "let key store");
}

#[test]
fn a_named_bracket_store_without_a_shape_refuses_rather_than_dropping() {
    // parse_and_lower_lir runs no type inference, so `o` has no materialized
    // shape here; the point is that the store is REFUSED, never silently
    // dropped. The end-to-end FIXED reading (o.b prints 8) is a case in
    // crates/kali_cli/tests/cases/object/computed_member_static_name.toml.
    let diagnostics = diagnostics_for("const o = {a: 1, b: 2}; o[\"b\"] = 8; console.log(o.b);");
    assert!(
        diagnostics.iter().any(|d| d.is_error() && d.code == Some(5506)),
        "an unshaped bracket store must refuse, not vanish: {diagnostics:?}"
    );
}

#[test]
fn a_runtime_array_element_store_keeps_its_lane() {
    let diagnostics = diagnostics_for("const a = new Array(2); let i = 1; a[i] = 9; console.log(a[i]);");
    assert!(
        !diagnostics.iter().any(|d| d.is_error() && d.message.contains("computed member access")),
        "{diagnostics:?}"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_codegen computed_member_tests`
Expected: the first three fail (today the store is dropped with no diagnostic; the `let k` store hits the read gateway's E5506 only if the target is emitted as a read — check the message).

- [ ] **Step 3: Add `store_target_node` and use it in the three existing arms**

In `computed_member.rs`, inside the `impl`:

```rust
    /// The store target as the `Value`-shaped node the store arms match on.
    /// A nameless computed member (`LirNodeKind::ComputedMember`) is presented
    /// as a text-less two-child `Value` so the for-in ordinal and runtime
    /// array arms — which never read the text for a name — keep admitting it.
    pub(crate) fn store_target_node(&self, left: LirNodeId) -> LirNode {
        let mut target = self.node(left).clone();
        if target.kind == LirNodeKind::ComputedMember {
            target.kind = LirNodeKind::Value;
        }
        target
    }
```

In `emit/literal.rs`, replace `let left_node = self.node(left).clone();` with `let left_node = self.store_target_node(left);` at the three computed arms: the for-in store (`:600`), the dynamic array element write (`:615`), and the for-in compound store (`:745`).

- [ ] **Step 4: Extract the dot-store arm**

Move the body of the arm at `emit/literal.rs:344-423` — the `if op == "=" { let left_node = ...; if left_node.kind == LirNodeKind::Value && left_node.children.len() == 1 { if let Some(field) = left_node.text.clone().filter(...) { let base_id = left_node.children[0]; if let Some(shape) = self.object_shape_of_node(base_id) { ... } } } }` block — into a new method in `literal.rs`, replacing `left_node` with `target` and every `return true;` inside with `return true;` (unchanged), and returning `false` at the end when no branch fired:

```rust
    /// `<base>.field = v` on a base with a materialized fixed shape: the typed
    /// store at the field's static offset, unknown fields gated E5506. Called
    /// for the dot spelling, and (Task 4 of the computed-member plan) for a
    /// bracket store whose name folded — the named, one-child twin of the
    /// bracket target is exactly this arm's shape.
    pub(crate) fn try_emit_shaped_field_store(
        &mut self,
        function: &mut Function,
        target: &LirNode,
        right: LirNodeId,
    ) -> bool {
        if target.kind != LirNodeKind::Value || target.children.len() != 1 {
            return false;
        }
        let Some(field) = target.text.clone().filter(|text| !text.is_empty()) else {
            return false;
        };
        let base_id = target.children[0];
        let Some(shape) = self.object_shape_of_node(base_id) else {
            return false;
        };
        // ... the existing body from `let Some((index, repr)) = self.repr_table.shape_field(shape, &field) else { … }` through the reload, verbatim ...
        true
    }
```

and make the original site:

```rust
        if op == "=" {
            let left_node = self.node(left).clone();
            if self.try_emit_shaped_field_store(function, &left_node, right) {
                return true;
            }
        }
```

Run `cargo test -p kali_codegen` here: this is a pure extraction and must be green before the next step.

- [ ] **Step 5: Insert the computed-store arm**

In `emit/literal.rs`, after the for-in compound arm (ends `:751`) and before `let Some(name) = self.assignment_target_name(node, left) else {` (`:753`):

```rust
        // A bracket store no runtime lane admitted (spec §4.4, store choke
        // point): fold the index to a name and take the dot spelling's arm,
        // or refuse. Never `return false` from here — the caller turns
        // `false` into a bare read of the target, which is the silently
        // dropped store the console-render follow-up's item 2.3 and the
        // register's R-13 write lane record.
        let target = self.store_target_node(left);
        if target.kind == LirNodeKind::Value
            && target.children.len() == 2
            && !is_binary_operator_text(target.text.as_deref().unwrap_or_default())
        {
            if op == "=" {
                if let Some(name) = self.static_member_name(&target) {
                    let mut dot = Self::named_twin(&target, name);
                    dot.children.truncate(1);
                    if self.try_emit_shaped_field_store(function, &dot, right) {
                        return true;
                    }
                }
            }
            let message = if op == "=" {
                computed_member_access_unavailable_message().to_string()
            } else {
                "compound assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
            };
            self.diagnostics.push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            function.instruction(&Instruction::I64Const(0));
            return true;
        }
```

(add `use kali_common::computed_member_access_unavailable_message;` to `literal.rs`'s imports; the compound message is the one already at `:765`, kept byte-identical.)

- [ ] **Step 6: Run the tests**

Run: `cargo test -p kali_codegen`
Expected: green.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen
git commit -m "feat(codegen): a bracket store with a static name takes the dot-store arm, and one without refuses

The dot-store arm is extracted as try_emit_shaped_field_store and reused
for the named one-child twin of a bracket target; a bracket store nothing
admits is an E5506, never a dropped store. Closes the codegen half of the
console-render follow-up's item 2.3 and R-13's write lane.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

---

### Task 5: The checker twin: the `const` set, the one fold rule, and the read/store gates

**Files:**
- Create: `crates/kali_types/src/static_analysis/computed_member.rs`, `computed_member_tests.rs`
- Modify: `crates/kali_types/src/static_analysis/mod.rs` (module declaration; `ls crates/kali_types/src/static_analysis/` and mirror how `string.rs` is declared)
- Modify: `crates/kali_types/src/scope.rs:21-112` (field + init), `crates/kali_types/src/resolve/mod.rs:877-886`
- Modify: `crates/kali_types/src/resolve/member.rs:5-42`
- Modify: `crates/kali_types/src/resolve/expression.rs:1583-1608` (`reject_literal_array_unfoldable_mutation`), `:98` (`is_process_argv_member` → `pub(crate)`)
- Modify: `crates/kali_types/src/resolve/member_tests.rs`

**Interfaces:**
- Consumes: `kali_parser::Parser::normalize_string_literal` (made `pub` in Task 2), `format_js_number`, `for_in_key_shape`, `is_for_in_key_value`, `is_structural_runtime_array`, `is_growable_array_binding`, `string_element_array_binding`, `growable_i64_field_member_parts`, `current_function_name`, `repr_table.is_array_binding(func, name)`, `resolve_static_string_expression`.
- Produces: `static_analysis::computed_member::fold_const_initializer(&Expression) -> Option<String>`; `fold_nameless_computed_index(&Expression, impl Fn(&str) -> Option<String>) -> Option<String>`; `Scope::const_index_names: IndexMap<String, String>`; `TypeContext::const_index_name(&self, name) -> Option<String>`; `TypeContext::gate_nameless_computed_member`, `gate_static_string_receiver_index`.

- [ ] **Step 1: Write the failing rule tests**

Create `crates/kali_types/src/static_analysis/computed_member_tests.rs`:

```rust
use super::computed_member::{fold_const_initializer, fold_nameless_computed_index};
use kali_ast::{Expression, LiteralValue};

fn ident(name: &str) -> Expression {
    Expression::Identifier(name.to_string())
}

#[test]
fn a_string_or_number_literal_initializer_folds_to_the_property_name() {
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::String("\"b\"".to_string()))).as_deref(),
        Some("b")
    );
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::Number(1.0))).as_deref(),
        Some("1")
    );
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::Number(1e21))).as_deref(),
        Some("1e+21")
    );
}

#[test]
fn any_other_initializer_does_not_fold() {
    assert_eq!(fold_const_initializer(&Expression::Literal(LiteralValue::Boolean(true))), None);
    assert_eq!(fold_const_initializer(&Expression::Literal(LiteralValue::Null)), None);
    assert_eq!(fold_const_initializer(&ident("other")), None);
}

#[test]
fn only_a_bare_identifier_with_a_recorded_name_folds() {
    let lookup = |name: &str| (name == "k").then(|| "b".to_string());
    assert_eq!(fold_nameless_computed_index(&ident("k"), lookup).as_deref(), Some("b"));
    assert_eq!(fold_nameless_computed_index(&ident("j"), lookup), None);
    let parenthesized = Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
        expression: ident("k"),
    }));
    assert_eq!(
        fold_nameless_computed_index(&parenthesized, lookup),
        None,
        "a parenthesized identifier is not the admitted shape"
    );
}
```

(Check `ParenthesizedExpression`'s field name and boxing in `crates/kali_ast/src/expression.rs`; adjust the constructor.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_types computed_member_tests`
Expected: compile error, module missing.

- [ ] **Step 3: Write the shared rule**

Create `crates/kali_types/src/static_analysis/computed_member.rs`:

```rust
//! The one fold rule for a computed member index the parser could not name.
//!
//! Three passes must agree on what folds — the resolver (this crate), the
//! materialization pass (`repr_infer`, this crate) and codegen — so the rule
//! is deliberately narrow and defined once: a BARE identifier naming a
//! `const` whose initializer is a string or number LITERAL. The two checker
//! passes call these functions with their own lookup; codegen mirrors the rule
//! on LIR through its `const`-only `bindings` map.
//!
//! Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.

use kali_ast::{Expression, LiteralValue};
use kali_common::js_number::format_js_number;
use kali_parser::Parser;

/// The property name a `const` declarator's initializer denotes when it is a
/// string or number literal; `None` for every other initializer.
pub(crate) fn fold_const_initializer(init: &Expression) -> Option<String> {
    match init {
        Expression::Literal(LiteralValue::String(raw)) => Some(Parser::normalize_string_literal(raw)),
        Expression::Literal(LiteralValue::Number(value)) => Some(format_js_number(*value)),
        _ => None,
    }
}

/// The property name a nameless computed index denotes: only a bare identifier
/// whose recorded `const` literal name `lookup` returns. Parenthesized, unary
/// and sequence spellings of the identifier are declined on purpose — the
/// parser already reads those forms when they wrap a literal, and widening
/// them here would have to be mirrored in three places.
pub(crate) fn fold_nameless_computed_index(
    index: &Expression,
    lookup: impl Fn(&str) -> Option<String>,
) -> Option<String> {
    let Expression::Identifier(name) = index else {
        return None;
    };
    lookup(name)
}

#[cfg(test)]
#[path = "computed_member_tests.rs"]
mod computed_member_tests;
```

Declare `pub(crate) mod computed_member;` in `crates/kali_types/src/static_analysis/mod.rs` beside the other modules. Confirm `kali_parser` is a normal dependency of `kali_types` (`grep kali_parser crates/kali_types/Cargo.toml`); it is used by `static_analysis/string.rs:232` already.

- [ ] **Step 4: Run the rule tests**

Run: `cargo test -p kali_types computed_member_tests`
Expected: PASS.

- [ ] **Step 5: Record `const` literal names in the resolver's scope**

`crates/kali_types/src/scope.rs`: add a field to `Scope` after `static_numeric_values` (line ~34):

```rust
    /// `const` declarators whose initializer is a string or number literal,
    /// mapped to the property name that literal denotes — the only bindings
    /// the computed-member fold admits (`static_analysis::computed_member`).
    /// NOT invalidated on reassignment: a `const` cannot be reassigned, and a
    /// `let`/`var` never enters this map.
    pub const_index_names: IndexMap<String, String>,
```

initialize it in `Scope::new` (`const_index_names: IndexMap::new(),`). In `crates/kali_types/src/resolve/mod.rs`, immediately after the `mutable_bindings` insert block at `:877-886`, add:

```rust
                if declaration.kind == "const" {
                    if let Some(init) = declarator.init.as_ref() {
                        if let Some(name) = crate::static_analysis::computed_member::fold_const_initializer(init) {
                            if let Some(scope) = self.scopes.get_mut(&target_scope) {
                                scope.const_index_names.insert(declarator.id.clone(), name);
                            } else if self.global_scope.contains(&declarator.id) {
                                self.global_scope
                                    .const_index_names
                                    .insert(declarator.id.clone(), name);
                            }
                        }
                    }
                }
```

(Check that `declarator.init` is in scope at that point; the survey shows `init` bound at `:966`, so if the declarator loop binds it earlier use that binding.)

- [ ] **Step 6: Write the failing gate tests**

Append to `crates/kali_types/src/resolve/member_tests.rs`:

```rust
use crate::test_support::parse_statements;
use kali_error::_error_codes::e5;

fn e5506_messages(source: &str) -> Vec<String> {
    let statements = parse_statements(source);
    let mut ctx = TypeContext::new();
    ctx.resolve_statements(&statements)
        .diagnostics
        .into_iter()
        .filter(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32))
        .map(|d| d.message)
        .collect()
}

const COMPUTED: &str = "computed member access `o[k]` is unavailable";
const STRING: &str = "indexing a string `s[i]` is unavailable";

#[test]
fn a_const_literal_key_folds_and_the_checker_admits_the_read_and_the_store() {
    assert!(e5506_messages("const o = {a:1, b:2}; const k = \"b\"; console.log(o[k]);").is_empty());
    assert!(e5506_messages("const o = {a:1, b:2}; const k = \"b\"; o[k] = 8; console.log(o.b);").is_empty());
    assert!(e5506_messages("const o = {1: \"one\"}; const k = 1; console.log(o[k]);").is_empty());
}

#[test]
fn a_let_var_or_parameter_key_refuses_once_with_the_shared_message() {
    for source in [
        "const o = {a:1, b:2}; let k = \"b\"; console.log(o[k]);",
        "const o = {a:1, b:2}; var k = \"b\"; console.log(o[k]);",
        "const o = {index: 9, i: 7}; let i = 1; console.log(o[i]);",
        "const o = {index: 9, i: 7}; let i = 1; console.log(o[i + 0]);",
        "const o = {a:1, b:2}; let k = \"b\"; o[k] = 8;",
        "function f(o, k) { return o[k]; }",
    ] {
        let messages = e5506_messages(source);
        assert_eq!(messages.len(), 1, "{source}: exactly one refusal, got {messages:?}");
        assert!(messages[0].contains(COMPUTED), "{source}: {messages:?}");
    }
}

#[test]
fn a_string_receiver_refuses_in_both_spellings() {
    let literal = e5506_messages("const s = \"abc\"; console.log(s[1]);");
    assert!(literal.iter().any(|m| m.contains(STRING)), "{literal:?}");
    let folded = e5506_messages("const s = \"abc\"; const k = 1; console.log(s[k]);");
    assert!(folded.iter().any(|m| m.contains(STRING)), "{folded:?}");
}

#[test]
fn an_array_literal_element_store_refuses_even_with_a_literal_index() {
    let messages = e5506_messages("const a = [5, 6]; a[1] = 9; console.log(a[1]);");
    assert!(messages.iter().any(|m| m.contains("mutating a literal array")), "{messages:?}");
    assert!(!messages.iter().any(|m| m.contains(COMPUTED)), "one owner per refusal: {messages:?}");
}

#[test]
fn the_runtime_lanes_are_still_admitted() {
    for source in [
        "const a = new Array(3); let i = 1; console.log(a[i]);",
        "const a = new Array(3); let i = 1; a[i] = 4;",
        "const o = {a: 1, b: 2}; for (const k in o) { console.log(o[k]); }",
    ] {
        let messages = e5506_messages(source);
        assert!(!messages.iter().any(|m| m.contains(COMPUTED)), "{source}: {messages:?}");
    }
}
```

(`TypeContext` is what the existing tests in this file construct; keep their import style.)

- [ ] **Step 7: Run the gate tests to verify they fail**

Run: `cargo test -p kali_types member_tests`
Expected: the refusal tests fail (no diagnostic today).

- [ ] **Step 8: Implement the gates**

In `crates/kali_types/src/resolve/member.rs`, add to the `impl TypeContext`:

```rust
    /// The recorded literal name of a `const` binding, walking the scope
    /// chain like `resolve_static_string_binding`.
    pub(crate) fn const_index_name(&self, name: &str) -> Option<String> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.const_index_names.get(name) {
                return Some(value.clone());
            }
            current = scope.parent;
        }
        self.global_scope.const_index_names.get(name).cloned()
    }

    /// Step 1 of spec §4.4: the runtime lanes codegen admits without a name.
    /// Every entry here mirrors a lane in codegen's two-child member arm; a
    /// currently-green fixture that starts refusing at `check` but not at
    /// `run` means a lane is missing HERE — add it, do not loosen the gate.
    pub(crate) fn nameless_computed_member_is_admitted_by_a_runtime_lane(
        &self,
        member: &MemberExpression,
    ) -> bool {
        let Some(index) = member.computed_index.as_deref() else {
            return false;
        };
        if let Expression::Identifier(key) = index {
            if self.for_in_key_shape(key).is_some() || self.is_for_in_key_value(key) {
                return true;
            }
        }
        if Self::is_process_argv_member(&member.object) {
            return true;
        }
        match &member.object {
            Expression::Identifier(base) => {
                // `current_function_name`'s return type decides the borrow
                // here — check it (`grep -n "fn current_function_name"
                // crates/kali_types/src`) and pass `&str` however that
                // function hands it over.
                self.is_structural_runtime_array(base)
                    || self.is_growable_array_binding(base)
                    || self.string_element_array_binding(base)
                    || self.repr_table.is_array_binding(&self.current_function_name(), base)
            }
            Expression::MemberExpression(inner) => self.growable_i64_field_member_parts(inner).is_some(),
            _ => false,
        }
    }

    /// Steps 2–4 of spec §4.4 for a computed member with no static name:
    /// admitted by a runtime lane, folded, or refused with the shared E5506.
    /// Runs for reads AND assignment targets (targets are resolved through
    /// `resolve_member_expression`), so a store gets exactly one diagnostic.
    pub(crate) fn gate_nameless_computed_member(&mut self, member: &MemberExpression) {
        if member.property.is_some() || member.computed_index.is_none() {
            return;
        }
        if self.nameless_computed_member_is_admitted_by_a_runtime_lane(member) {
            return;
        }
        let index = member.computed_index.as_deref().expect("checked above");
        let folded = crate::static_analysis::computed_member::fold_nameless_computed_index(
            index,
            |name| self.const_index_name(name),
        );
        if folded.is_some() {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::computed_member_access_unavailable_message().to_string(),
        ));
    }

    /// A statically-known string receiver has no index lane in either
    /// spelling (spec §4.4, "String receivers"); codegen refuses the same way.
    /// Returns whether it claimed the access, so the nameless gate does not
    /// also fire and report the same defect twice.
    pub(crate) fn gate_static_string_receiver_index(&mut self, member: &MemberExpression) -> bool {
        if member.computed_index.is_none() {
            return false;
        }
        if self.resolve_static_string_expression(&member.object).is_none() {
            return false;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::string_index_access_unavailable_message().to_string(),
        ));
        true
    }
```

and in `resolve_member_expression` (`member.rs:5-8`), after `self.reject_nonuniform_forin_key_object_access(expr);`:

```rust
        if !self.gate_static_string_receiver_index(expr) {
            self.gate_nameless_computed_member(expr);
        }
```

Make `is_process_argv_member` (`resolve/expression.rs:98`) `pub(crate)`.

In `reject_literal_array_unfoldable_mutation` (`resolve/expression.rs:1583-1608`): the nameless gate owns a nameless index, and a named literal index on an array literal has no store lane in codegen either (Task 4), so the store refuses unconditionally for a named index:

```rust
        let Some(index) = member.computed_index.as_deref() else {
            return;
        };
        if member.property.is_none() {
            return; // owned by `gate_nameless_computed_member`
        }
        let Expression::Identifier(base_name) = &member.object else {
            return;
        };
        if !self.resolve_array_literal_binding_name(base_name) {
            return;
        }
        let _ = index;
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "mutating a literal array is unavailable in the current direct-runtime path; use new Array(n) for runtime mutation".to_string(),
        ));
```

Update that function's doc comment: the old text said the mutation was admitted when "the whole access folds statically"; it never did anything (codegen dropped the store, follow-up item 2.3), so now it refuses always.

- [ ] **Step 9: Run the checker tests**

Run: `cargo test -p kali_types`
Expected: the new tests green. Other tests in the crate that go red pinned either a fabricated read or the old "literal array mutation admitted when foldable" message; re-pin them to the refusal and list each in the commit message. If `the_runtime_lanes_are_still_admitted` fails on the `for..in` program, the for-in gate's own diagnostic is firing (not this project's); change that assertion to the message check only, as written.

- [ ] **Step 10: Commit**

```bash
git add crates/kali_types
git commit -m "feat(types): kali check refuses a nameless computed member and folds a const literal key

One rule in static_analysis::computed_member, a const_index_names table on
Scope, and two gates in resolve_member_expression. The admit list mirrors
codegen's runtime lanes; a string receiver refuses in both spellings; a
literal-array element store refuses unconditionally, since it never landed.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

---

### Task 6: The checker's materialization pass records a folded read and store as the dot form

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (struct fields near `:623`, `visit_declarator_init` at `:2904`, `visit_member` at `:4007-4060`, `visit_assignment` at `:3904-3931`)
- Modify: `crates/kali_types/src/repr_infer_tests.rs`

**Interfaces:**
- Consumes: `fold_const_initializer`, `fold_nameless_computed_index` (Task 5), `ObjAccess`, `ObjSlot::Binding`, `member_base_slot`, `array_elem_node_for`.
- Produces: `ReprInfer::const_index_names: BTreeMap<(String, String), String>`; `ReprInfer::static_member_field(&self, func, member) -> Option<String>`.

- [ ] **Step 1: Write the failing materialization test**

Append to `crates/kali_types/src/repr_infer_tests.rs`:

```rust
#[test]
fn a_bracket_store_with_a_static_name_materializes_the_object_like_the_dot_store() {
    let dot = reprs("const q = {a: 1}; q.a = 7; console.log(q.a);\n");
    assert!(
        matches!(dot.scalar("_start", "q"), Repr::Object(_)),
        "control: the dot store must materialize q (got {:?})",
        dot.scalar("_start", "q")
    );
    let literal = reprs("const p = {a: 1}; p[\"a\"] = 7; console.log(p.a);\n");
    assert!(
        matches!(literal.scalar("_start", "p"), Repr::Object(_)),
        "p[\"a\"] = 7 must materialize p as the dot store does (got {:?})",
        literal.scalar("_start", "p")
    );
    let folded = reprs("const o = {a: 1}; const k = \"a\"; o[k] = 7; console.log(o.a);\n");
    assert!(
        matches!(folded.scalar("_start", "o"), Repr::Object(_)),
        "o[k] = 7 with const k must materialize o (got {:?})",
        folded.scalar("_start", "o")
    );
    let unfolded = reprs("const o = {a: 1}; let k = \"a\"; o[k] = 7; console.log(o.a);\n");
    assert!(
        !matches!(unfolded.scalar("_start", "o"), Repr::Object(_)),
        "a let key does not fold and records no field write"
    );
}

#[test]
fn a_folded_read_carries_the_field_repr() {
    let t = reprs("const o = {a: 1.5}; const k = \"a\"; let v = o[k];\n");
    assert_eq!(t.scalar("_start", "v"), Repr::F64, "v reads the F64 field through the folded name");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_types repr_infer_tests::a_bracket_store`
Expected: the literal and folded assertions fail (the control passes; if the control fails, the materialization mechanism is not `Repr::Object` on the scalar — read `repr_infer_tests.rs:146` to find the accessor the crate uses for a materialized object and use that accessor in all four assertions).

- [ ] **Step 3: Record `const` literal names and add the field resolver**

In `struct ReprInfer` (near `repr_infer.rs:623`), add:

```rust
    /// `(func, binding)` → the property name a `const`'s string/number
    /// literal initializer denotes; the computed-member fold's lookup table
    /// for this pass (`static_analysis::computed_member`).
    const_index_names: BTreeMap<(String, String), String>,
```

initialize it where the other `BTreeMap` fields are (`BTreeMap::new()`). At the top of `visit_declarator_init(&mut self, func: &str, kind: &str, id: &str, init: &Expression)` (`:2904`):

```rust
        if kind == "const" {
            if let Some(name) = crate::static_analysis::computed_member::fold_const_initializer(init) {
                self.const_index_names
                    .insert((func.to_string(), id.to_string()), name);
            }
        }
```

Add the resolver next to `member_base_slot` (`:1971`):

```rust
    /// The static property name of a member — the parser's (`o.b`, `o["b"]`)
    /// or the `const` fold's (`const k = "b"; o[k]`) — or `None` for an index
    /// that must be evaluated.
    fn static_member_field(&self, func: &str, member: &kali_ast::MemberExpression) -> Option<String> {
        if let Some(name) = member.static_name() {
            return Some(name.to_string());
        }
        let index = member.computed_index.as_deref()?;
        crate::static_analysis::computed_member::fold_nameless_computed_index(index, |name| {
            self.const_index_names
                .get(&(func.to_string(), name.to_string()))
                .cloned()
        })
    }
```

- [ ] **Step 4: Record the deferred object access beside the array-element edge**

In `visit_member`'s computed branch (`:4043-4057`), replace the `if let Expression::Identifier(name) = &member.object { … return result; }` block with:

```rust
            let static_field = self.static_member_field(func, member);
            if let Expression::Identifier(name) = &member.object {
                let elem = self.array_elem_node_for(func, name);
                let result = self.new_node();
                // Read is directed: element -> read result (see the note on
                // the string axis that used to sit here — unchanged).
                self.add_edge(elem, result);
                // A computed read with a static name is ALSO the dot read
                // `o.<name>`: record the same deferred object access, so an
                // object receiver wires the field's repr into the result and
                // materializes on the same evidence. For an array receiver
                // `resolve_objects` finds no fields and skips it, which is
                // why both records can coexist (spec §4.4, checker).
                if let Some(field) = static_field {
                    self.obj_accesses.push(ObjAccess {
                        base: ObjSlot::Binding(func.to_string(), name.clone()),
                        field,
                        other: result,
                        is_write: false,
                    });
                }
                return result;
            }
```

(keep the original long comment about the string axis in place above `add_edge`.)

In `visit_assignment`'s computed branch (`:3904-3931`), inside `if let Expression::Identifier(name) = &member.object { … }` after `self.seed_persisted_for_in_key_string_use(func, &assign.right);`, add:

```rust
                    // Same as the read: a static-name bracket store is the
                    // dot store `o.<name> = v` and records the write access
                    // that materializes the object (follow-up item 2.3,
                    // R-13's write lane).
                    if let Some(field) = self.static_member_field(func, member) {
                        self.obj_accesses.push(ObjAccess {
                            base: ObjSlot::Binding(func.to_string(), name.clone()),
                            field,
                            other: rn,
                            is_write: true,
                        });
                    }
```

- [ ] **Step 5: Run the tests**

Run: `cargo test -p kali_types`
Expected: green, including the two new tests.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types
git commit -m "feat(types): a static-name bracket access records the dot form's deferred object access

repr_infer records const literal names at the declarator and, for a computed
read or store whose name is the parser's or the fold's, pushes the same
ObjAccess the dot spelling pushes beside the array-element edge, so the
object materializes and codegen's dot-store arm has a shape to store into.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

---

### Task 7: The cases: pin, flip, and re-pin, in both scopes

**Files:**
- Create: `crates/kali_cli/tests/cases/object/computed_member_static_name.toml`
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml` (`r13r_*`, `r13w_*`, `r59a_*`)
- Modify: `crates/kali_cli/tests/cases/oracle/classifier_ground_truth.toml` (header table `:20-31`, `silent.js` at `:122`, the case at `:185-193`)
- Modify: `crates/kali_cli/tests/cases/object/property_key_identity.toml` (`:631-692`, and the R-59 paragraph in its header, `:40-95`)
- Modify: any other case file whose expectation pinned a fabricated read, a silent `s[1]`, or a dropped bracket store (found by the full run in Step 6)

Every measurement below is taken on a binary built from this branch (`cargo build -p kali_cli --bin kali`, `.cache/cargo-target/debug/kali`) against `node --version` (v26.8.1), in **both** scopes, before a verdict or an expectation is written. Write the commit SHA the binary was built from into every rationale that cites a measurement.

- [ ] **Step 1: Write the new case file**

Create `crates/kali_cli/tests/cases/object/computed_member_static_name.toml`:

```toml
# Cases for the computed-member-static-name project (spec
# docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md).
#
# The admitted matrix (dot, literal, const-folded; read and store), the
# refused matrix (let/var/parameter key, binary index, boolean/null/BigInt
# literal, string receiver, array-literal store and mutable-index read), and
# the controls of spec §2.5. Every refusal is asserted under BOTH `check` and
# `run` with the one shared message, because `kali check` runs the checker
# only and the refusal must live in both twins.
#
# Every expectation here was measured at the commit named in its rationale
# against node v26.8.1, in both scopes where the program has a function twin.

[constants]
COMPUTED = "computed member access `o[k]` is unavailable in the current phase unless the index is a literal or a compile-time-constant `const` binding, or the receiver is a runtime array or a `for..in` key over the same object"
STRING = "indexing a string `s[i]` is unavailable in the current phase; use `charAt`/`at` on a statically-known ASCII string or the later compatibility path"

[source]
"const_key_read_module.js" = """const o = {a: 1, b: 2}; const k = "b"; console.log(o[k]); console.log(o[k] + 1);
"""
"const_key_read_function.js" = """function main() {
  const o = {a: 1, b: 2}; const k = "b"; console.log(o[k]); console.log(o[k] + 1);
}
main();
"""
"const_number_key_read.js" = """const o = {1: "one", 2: "two"}; const k = 1; console.log(o[k]);
"""
"const_key_store_module.js" = """const o = {a: 1, b: 2}; const k = "b"; o[k] = 8; console.log(o.b);
"""
"const_key_store_function.js" = """function main() {
  const o = {a: 1, b: 2}; const k = "b"; o[k] = 8; console.log(o.b);
}
main();
"""
"literal_key_store_string.js" = """const p = {a: 1}; p["a"] = 7; console.log(p["a"]);
"""
"literal_key_store_number.js" = """const o = {5: 1}; o[5] = 7; console.log(o[5]);
"""
"dot_store_control.js" = """const q = {a: 1}; q.a = 7; console.log(q.a);
"""
"const_key_optional_chain.js" = """const o = {a: 1, b: 2}; const k = "b"; console.log(o?.[k]);
"""
"absent_property_after_fold.js" = """const o = {a: 1, b: 2}; const k = "c"; console.log(o[k]); console.log(o["c"]); console.log(o.c);
"""
"let_key_read.js" = """const o = {a: 1, b: 2}; let k = "b"; console.log(o[k]);
"""
"var_key_read.js" = """const o = {a: 1, b: 2}; var k = "b"; console.log(o[k]);
"""
"parameter_key_read.js" = """function get(o, k) { return o[k]; }
console.log(get({a: 1}, "a"));
"""
"binary_index_read.js" = """const o = {index: 9, i: 7}; let i = 1; console.log(o[i + 0]);
"""
"boolean_null_bigint_index.js" = """const o = {index: 5, true: 7}; console.log(o[true]); console.log(o[null]); console.log(o[1n]);
"""
"let_key_store.js" = """const o = {a: 1, b: 2}; let k = "b"; o[k] = 8; console.log(o.b);
"""
"string_receiver_literal.js" = """const s = "abc"; console.log(s[1]);
"""
"string_receiver_folded.js" = """const s = "abc"; const k = 1; console.log(s[k]);
"""
"array_literal_store.js" = """const a = [5, 6]; a[1] = 9; console.log(a[1]);
"""
"array_literal_mutable_index_read.js" = """const a = [5, 6, 7]; for (let j = 0; j < 3; j++) { console.log(a[j]); }
"""
"array_literal_const_index_read.js" = """const a = [5, 6]; const i = 1; console.log(a[i]);
"""
"runtime_array_control.js" = """const a = new Array(3); for (let i = 0; i < 3; i++) { a[i] = i * 2; } let j = 2; console.log(a[j]);
"""
"readable_string_key_control.js" = """const o = {a: 1, b: 2}; console.log(o["b"]);
"""
"readable_numeric_spellings_control.js" = """const o = {1: "one", 2: "two"}; console.log(o[(1)]); console.log(o[(0, 1)]); console.log(o[+1]);
"""
"chained_access_off_folded_member.js" = """const o = {a: "xyz"}; const k = "a"; console.log(o[k].length);
"""
"computed_callee_fold.js" = """const o = {a: 1, b: 2, c: 3, d: 4}; const k = "keys"; console.log(Object[k](o).length);
"""

# ---- admitted: the fold is the dot spelling ---------------------------------

[[case]]
name = "a_const_string_key_reads_the_property_module_scope"
rationale = """R-13's read lane, spec §2.2. node prints 2 then 3. Before this project kali printed 0 then 1 at exit 0: the parser fabricated the name `k`, the fold-lane object had no such field, and the miss was the placeholder 0. Now `k` folds to "b" through the const rule and the read is `o.b`. Measured at <SHA> against node v26.8.1."""
args = ["run", "const_key_read_module.js"]
exit = "success"
stdout = "2\n3\n"

[[case]]
name = "a_const_string_key_reads_the_property_in_function"
rationale = "The module-scope program inside `function main() { ... }`; the fold is scope-independent (a const declarator in the same scope). Measured at <SHA>."
args = ["run", "const_key_read_function.js"]
exit = "success"
stdout = "2\n3\n"

[[case]]
name = "a_const_number_key_folds_through_the_shared_number_formatter"
rationale = "`const k = 1; o[k]` on `{1: \"one\"}`: the fold renders 1 with format_js_number, the same formatter HIR stored the key with, so probe and key are one spelling. node prints one. Measured at <SHA>."
args = ["run", "const_number_key_read.js"]
exit = "success"
stdout = "one\n"

[[case]]
name = "a_const_string_key_store_lands_module_scope"
rationale = """R-13's write lane, spec §2.3. node prints 8. Before this project kali printed 2: the bracket store fell out of every store arm and the caller emitted a bare read. Now the folded target takes the dot-store arm, and the checker recorded the write access that materializes `o`. Measured at <SHA>."""
args = ["run", "const_key_store_module.js"]
exit = "success"
stdout = "8\n"

[[case]]
name = "a_const_string_key_store_lands_in_function"
rationale = "The write lane inside `function main() { ... }`. Measured at <SHA>."
args = ["run", "const_key_store_function.js"]
exit = "success"
stdout = "8\n"

[[case]]
name = "a_literal_string_key_bracket_store_lands"
rationale = "console-render follow-up item 2.3, string key: `p[\"a\"] = 7` then `p[\"a\"]`. node prints 7; kali printed 1 before this project. The named bracket store takes the dot-store arm. Measured at <SHA>."
args = ["run", "literal_key_store_string.js"]
exit = "success"
stdout = "7\n"

[[case]]
name = "a_literal_number_key_bracket_store_lands"
rationale = "console-render follow-up item 2.3, numeric key: `o[5] = 7` then `o[5]` on `{5: 1}`. node prints 7; kali printed 1 before. Measured at <SHA>."
args = ["run", "literal_key_store_number.js"]
exit = "success"
stdout = "7\n"

[[case]]
name = "an_absent_property_after_a_successful_fold_is_r21s_lane_not_this_projects"
rationale = """WRONG ON PURPOSE, and NO TASK IN THIS PLAN OWNS CLOSING IT. node prints undefined three times; kali prints 0 three times, at exit 0.

WHAT IT PINS, AND WHY IT IS A CASE AT ALL. Spec section 5 carves this lane out: a `const` key that folds to a name the object does NOT have reads the same placeholder `0` the dot spelling `o.c` reads today. All three spellings are pinned together precisely so they must move together: the fold's whole claim is that a folded bracket access IS the dot access, and if a later change made `o[k]` and `o.c` disagree here, that claim would have quietly broken. The `0` itself is R-21 (there is no value distinct from the scalar 0), which this project does not touch.

Measured at `dc19c3a040` and re-measured at <SHA>: all three lines print 0 before and after this project."""
args = ["run", "absent_property_after_fold.js"]
exit = "success"
stdout = "0\n0\n0\n"

[[case]]
name = "the_dot_store_control_still_lands"
rationale = "Spec §2.5 control: `q.a = 7` was correct before and must stay correct. Measured at <SHA>."
args = ["run", "dot_store_control.js"]
exit = "success"
stdout = "7\n"

[[case]]
name = "the_optional_chain_spelling_folds_the_same_way"
rationale = "`o?.[k]` goes through the second parse site, which used to fabricate identically; it now declines identically and the const key folds. node prints 2. Measured at <SHA>."
args = ["run", "const_key_optional_chain.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "the_readable_string_key_spelling_stays_correct"
rationale = "Spec §2.5: `o[\"b\"]` on `{a: 1, b: 2}` was correct before and must stay correct. node prints 2. Measured at <SHA>."
args = ["run", "readable_string_key_control.js"]
exit = "success"
stdout = "2\n"

[[case]]
name = "the_readable_numeric_spellings_stay_correct"
rationale = """Spec §2.5: the parser's readable set is unchanged — the parenthesized, sequence-last and folded-unary spellings of a numeric literal all read the right property on `{1: \"one\", 2: \"two\"}`. node prints one three times. The receiver is a numeric-key object on purpose: `o[\"b\"]` on THIS object would be an absent read, which prints 0 against node's undefined (R-21's lane, not this project's), so the two controls are separate programs. Measured at <SHA>."""
args = ["run", "readable_numeric_spellings_control.js"]
exit = "success"
stdout = "one\none\none\n"

[[case]]
name = "a_const_index_on_an_array_literal_folds_into_the_element_fold"
rationale = "`const i = 1; a[i]` on `[5, 6]`: the fold renders \"1\", which the static element fold reads. node prints 6; kali printed 0 before. Measured at <SHA>."
args = ["run", "array_literal_const_index_read.js"]
exit = "success"
stdout = "6\n"

[[case]]
name = "the_runtime_array_lane_is_untouched"
rationale = "Spec §2.5 control: a `new Array(n)` receiver reads its index child through the linear-memory lane in both the checker and codegen. node prints 4. Measured at <SHA>."
args = ["run", "runtime_array_control.js"]
exit = "success"
stdout = "4\n"

# ---- refused: one shared message, in both check and run ---------------------

[[case]]
name = "check_refuses_a_let_key"
rationale = "Spec §5: a `let` key does not fold — the checker could, codegen cannot, and refusing is the only answer the twins share. The refusal must live in `check`, which runs the checker only. Measured at <SHA>."
args = ["check", "let_key_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_a_let_key"
rationale = "The same program under `run`, the same message: codegen's gateway is the checker's twin. node prints 2; before this project kali printed 0 at exit 0. Measured at <SHA>."
args = ["run", "let_key_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "json_check_refuses_a_let_key_with_the_canonical_message"
rationale = "The JSON envelope carries the same code and message byte for byte."
args = ["--output", "json", "check", "let_key_read.js"]
exit = "failure"
json.errors.0.code = "E5506"
json.errors.0.message = "${COMPUTED}"

[[case]]
name = "check_refuses_a_var_key"
rationale = "The ground-truth classifier's old SILENT fixture spelling (var k; o[k]). node prints 2; kali printed 0 before. Measured at <SHA>."
args = ["check", "var_key_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_a_var_key"
rationale = "Twin of the check case."
args = ["run", "var_key_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "check_refuses_a_parameter_key"
rationale = "A parameter is not a const declarator and folds nowhere. Measured at <SHA>."
args = ["check", "parameter_key_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_a_parameter_key"
rationale = "Twin of the check case."
args = ["run", "parameter_key_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "check_refuses_a_binary_index"
rationale = "R-59's second line: `o[i + 0]` used to read the property literally named `index` (9). node prints undefined. Measured at <SHA>."
args = ["check", "binary_index_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_a_binary_index"
rationale = "Twin of the check case."
args = ["run", "binary_index_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "check_refuses_boolean_null_and_bigint_indexes"
rationale = "Spec §5: the parser's readable set does not grow; `o[true]`, `o[null]`, `o[1n]` used to fabricate `index` (5, 5, 5 against node's 7, undefined, undefined). Measured at <SHA>."
args = ["check", "boolean_null_bigint_index.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_boolean_null_and_bigint_indexes"
rationale = "Twin of the check case."
args = ["run", "boolean_null_bigint_index.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "check_refuses_a_let_key_store"
rationale = "The store choke point's twin: a nameless bracket store refuses instead of vanishing. node prints 8; kali printed 2 before. Measured at <SHA>."
args = ["check", "let_key_store.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_a_let_key_store"
rationale = "Twin of the check case."
args = ["run", "let_key_store.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "check_refuses_a_string_receiver_literal_index"
rationale = "Spec §4.4 string receivers: `s[1]` was a silent 0 (node: b). No character fold is added; both spellings refuse with the string message. Measured at <SHA>."
args = ["check", "string_receiver_literal.js"]
exit = "failure"
stderr_contains = ["E5506", "${STRING}"]

[[case]]
name = "run_refuses_a_string_receiver_literal_index"
rationale = "Twin of the check case."
args = ["run", "string_receiver_literal.js"]
exit = "failure"
stderr_contains = ["E5506", "${STRING}"]

[[case]]
name = "check_refuses_a_string_receiver_folded_index"
rationale = "`const k = 1; s[k]` folds onto the same lane and refuses the same way."
args = ["check", "string_receiver_folded.js"]
exit = "failure"
stderr_contains = ["E5506", "${STRING}"]

[[case]]
name = "run_refuses_a_string_receiver_folded_index"
rationale = "Twin of the check case."
args = ["run", "string_receiver_folded.js"]
exit = "failure"
stderr_contains = ["E5506", "${STRING}"]

[[case]]
name = "check_refuses_an_array_literal_element_store"
rationale = "Spec §2.3: `a[1] = 9` on an array literal vanished (kali 6, node 9). There is no store lane for an array literal; the checker's literal-array mutation gate now refuses unconditionally. Measured at <SHA>."
args = ["check", "array_literal_store.js"]
exit = "failure"
stderr_contains = ["E5506", "mutating a literal array"]

[[case]]
name = "run_refuses_an_array_literal_element_store"
rationale = "Twin of the check case; `run` runs the checker first, so the same message appears."
args = ["run", "array_literal_store.js"]
exit = "failure"
stderr_contains = ["E5506", "mutating a literal array"]

[[case]]
name = "check_refuses_a_mutable_index_read_over_an_array_literal"
rationale = "Spec §5 non-goal: `for (let j…) a[j]` over an array literal printed 0 0 0 (node 5 6 7). The linear-memory lane exists for `new Array(n)` receivers; extending it to literals is its own design. Measured at <SHA>."
args = ["check", "array_literal_mutable_index_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_a_mutable_index_read_over_an_array_literal"
rationale = "Twin of the check case."
args = ["run", "array_literal_mutable_index_read.js"]
exit = "failure"
stderr_contains = ["E5506", "${COMPUTED}"]

[[case]]
name = "run_refuses_a_chained_access_off_a_folded_member"
rationale = """Spec §4.5: `o[k].length` with a const key — the outer access sees the inner node by id, and by id it is the nameless kind, so it refuses. node prints 3.

THIS IS NOT A WORKING LANE BEING REGRESSED. Measured at `dc19c3a040`: this program printed `2`, and so did the same program with `"xyzwv"` — the value is `render_length`'s text-less arm returning the member node's CHILD COUNT, not the string's length. Refusing replaces a silent wrong answer with an honest one. The dot spelling `o.a.length` prints `1` by the same arm and is NOT fixed here; it is recorded as a separate divergence in the follow-up this project files. Measured at <SHA>."""
args = ["run", "chained_access_off_folded_member.js"]
exit = "failure"
stderr_contains = ["E5506"]

[[case]]
name = "run_refuses_a_computed_callee"
rationale = """`Object[k](o)` with `const k = "keys"`: the optimizer's name-string fold declines a nameless member, so the call has no resolvable callee and refuses.

ALSO NOT A WORKING LANE. Measured at `dc19c3a040`, this printed `2` for a FOUR-key object — the child count again, through the same renderer. node prints 4. Measured at <SHA>."""
args = ["run", "computed_callee_fold.js"]
exit = "failure"
stderr_contains = ["E5506"]
```

Replace every `<SHA>` with the short SHA of the commit the binary was built from. If any expectation above does not match the measured output, the **implementation** is wrong or the spec is; do not edit the expectation to match — stop and re-read spec §4.4.

- [ ] **Step 2: Run the new file**

Run: `cargo build -p kali_cli --bin kali && cargo test -p kali_cli --test cases -- object/computed_member_static_name`
Expected: green. (A `check`-green/`run`-red pair on the same program is a twin disagreement: fix the checker's admit list in `nameless_computed_member_is_admitted_by_a_runtime_lane` or codegen, per spec §8.)

- [ ] **Step 3: Flip the oracle verdicts, re-measured first**

For each program, run it by hand first:

```bash
K=.cache/cargo-target/debug/kali
for p in r13r_module r13r_function r13w_module r13w_function r59a_module r59a_function; do
  # extract the [source] body into /tmp/claude-1000/-workspace/*/scratchpad/$p.js by hand or with a 5-line python snippet
  echo "== $p"; node scratch/$p.js; echo "exit=$?"; $K run scratch/$p.js; echo "exit=$?"
done
```

Expected readings: `r13r_*` → kali `v=2`, node `v=2`, both exit 0 → **fixed**. `r13w_*` → kali `dot=8`, node `dot=8` → **fixed**. `r59a_*` → kali `error[E5506]: computed member access …` exit 1, node `undefined\nundefined` exit 0 → **fail_closed**.

In `crates/kali_cli/tests/cases/oracle/tier2.toml` change `verdict = "silent"` to the measured class on those six cases, and **append** (never replace) to each rationale a paragraph of this shape:

```
RE-MEASURED 2026-09-08 at `<SHA>` against node v26.8.1: FIXED (was SILENT). kali prints `v=2` at exit 0 with empty stderr; node prints `v=2` at exit 0. Closed by the computed-member-static-name project (docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md): `k` is a `const` with a string-literal initializer, so the index folds to "b" and the read is `o.b`. The verdict was flipped only after this reading.
```

For `r59a_*`, the paragraph says FAIL_CLOSED and quotes the E5506 message, and notes that FAIL_CLOSED is one of the two outcomes the case's own "WHAT A REGRESSION HERE WOULD MEAN" paragraph names as closing R-59 (the other being FIXED), and that a fix which produced `0`, `0` would NOT have closed it.

- [ ] **Step 4: Replace the ground-truth SILENT fixture with R-10**

In `crates/kali_cli/tests/cases/oracle/classifier_ground_truth.toml`:
- header table line 24: `#   silent            R-10           silent.js             ALSO re-measures R-10`
- `[source]` `silent.js` body: `let x = 1; { let x = 2; } console.log("r=" + x);` (the register's and `tier2.toml`'s spelling of R-10's repro).
- the case: `name = "a_block_scoped_shadow_read_classifies_as_silent"`, `register_entry = "R-10"`, rationale:

```
R-10: a block-scoped `let` shadow aliases the outer binding, so the read after the block prints the inner value. Both engines exit 0 and their stdout differs, which is the definition of SILENT -- the class this whole project exists to rank, and the one a broken classifier would most damagingly mislabel as FIXED.

This is the only case that exercises the stdout-inequality branch of `classify`'s `(false, false)` arm; the two `fixed` cases above exercise the equality branch, so they pin both sides of that comparison between them.

WHY R-10 AND NOT R-13. Until 2026-09-08 this row was R-13's `var k; o[k]` read. The computed-member-static-name project closed R-13 (both lanes FIXED with a `const` key) and made the `var`-key spelling FAIL_CLOSED, so R-13 can no longer hold the SILENT class. R-10 is chosen because its fix is architectural (register §6, Group 4: a resolver scope frame per block) and no project is near it, so this fixture will not need replacing again soon; its oracle pair in tier2.toml records SILENT in both scopes.

Measured 2026-09-08 at `<SHA>` against node v26.8.1: kali `r=2` exit 0, node `r=1` exit 0. Matches the register: §0.2's R-10 row records SILENT (both scopes).
```

Run: `cargo test -p kali_cli --test cases -- oracle/` — expected green; then `cargo test -p kali_blast_radius` — expected **red** on `every_zero_two_row_is_the_class_set_its_live_cases_assert` for R-13 and R-59 only (the register rows still say SILENT; Task 8 re-derives them). Record the exact failure text; it is the evidence Task 8 cites.

- [ ] **Step 5: Flip the wrong-on-purpose pair**

In `crates/kali_cli/tests/cases/object/property_key_identity.toml`, the two cases at `:631-692`: change to

```toml
args = ["run", "computed_member_index_module.js"]
exit = "failure"
stderr_contains = ["E5506", "computed member access `o[k]` is unavailable"]
```

(and the `_function.js` twin), and **prepend** to each rationale:

```
CLOSED 2026-09-08 at `<SHA>` by the computed-member-static-name project: both lines now refuse with the shared E5506 (`kali check` and `kali run` alike) instead of reading `7` and `9`. The paragraphs below are kept as the record of what this program did while it was wrong on purpose; they are no longer a description of the binary.
```

In the file header's R-59 paragraph (`:40-95`), replace "The cases below are UNCHANGED and stay WRONG ON PURPOSE" with a sentence saying the R-59 pair flipped to a refusal at `<SHA>`.

- [ ] **Step 6: Run the whole workspace and re-pin what moved**

Run: `bash scripts/test-gate.sh 2>&1 | tail -60`
Expected: red only on cases that pinned a fabricated read, a silent `s[1]`, a dropped bracket store, the old "mutating a literal array … unless the whole access folds" message, or a `let`-key read. For each red case: measure its program by hand on this binary and on node, then re-pin to the refusal (`exit = "failure"`, `stderr_contains = ["E5506", "<message>"]`) or the now-correct value, and append a dated paragraph to its rationale stating the old and new readings. If a red case is **not** in one of those classes, stop: it is a regression, and the task is not done until it is understood (spec §8).

Run again: `bash scripts/test-gate.sh 2>&1 | tail -5`
Expected: `0 failed`.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_cli/tests/cases
git commit -m "test(cases): computed-member static name — the admitted and refused matrices, and the verdict flips

New object/computed_member_static_name.toml (both twins on every refusal);
r13r/r13w flip to fixed and r59a to fail_closed after re-measurement; the
classifier's SILENT fixture moves from R-13 to R-10; the property-key
wrong-on-purpose pair flips to a refusal. Re-pinned: <list each file>.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

---

### Task 8: The ledger: register, outputs, ranking, clusters, follow-up, plan, doc comments

**Files:**
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 rows `:269`, `:301`; §2 entries `### R-13` at `:2182`, `### R-59` at `:3407`)
- Modify: `tools/blast-radius/clusters.json` (`:232-236`, `:274-280`, `:442-449`, the note paragraph at `:31-43`)
- Modify (outputs, generated): `tools/blast-radius/accepts.json`, `tools/blast-radius/counts.json`
- Modify (generated region + authored amendment): `docs/superpowers/followups/blast-radius-ranking.md`
- Modify: `docs/superpowers/followups/console-render-unification-discovered-defects.md` (§2.3 at `:117`)
- Create: `docs/superpowers/followups/member-length-renders-the-child-count.md`
- Modify: `plan/phase-21/README.md:43`
- Modify: doc comments at `crates/kali_codegen/src/intrinsics/object.rs:86-92`, `crates/kali_optimize/src/object_fold.rs:767-775`, `crates/kali_optimize/src/helpers.rs:172`

Order matters: the ranking generator reads §0.2's verdicts and `clusters.json`, and `counts.json` reads `accepts.json`.

- [ ] **Step 1: Re-derive the two §0.2 rows**

`kali-silent-miscompile-register.md:269` (R-13) becomes:

```
| R-13 computed var-key get/set | **FIXED** (read `r13r`; write `r13w`), both scopes | **RETIRED 2026-09-08 at `<SHA>` by the computed-member-static-name project — every lane of this entry moved, which is the rule §3.4 of the ranking states.** Re-derived from the four `r13r`/`r13w` cases, which now assert `fixed`; the gate `every_zero_two_row_is_the_class_set_its_live_cases_assert` named the mismatch first and the row followed it. ~~SILENT (read `r13r`) / SILENT (write `r13w`), both scopes: read →`v=0` where node reads `2`; write vanishes (kali `dot=2`, node `dot=8`).~~ WHAT IS PINNED BY A LIVE CASE: the `const` key, read and store, both scopes. WHAT IS NOT PINNED HERE AND IS PINNED ELSEWHERE: the `var`-key spelling this entry's ground-truth fixture used is now FAIL_CLOSED (`object/computed_member_static_name.toml`, `check_refuses_a_var_key`), so the classifier's SILENT fixture moved to R-10. |
```

`:301` (R-59) becomes:

```
| R-59 a computed member index that is not a literal is fabricated into a property name | **FAIL_CLOSED** (both scopes) | **RETIRED 2026-09-08 at `<SHA>` by the computed-member-static-name project — its one lane moved.** Re-derived from the two `r59a` cases, which now assert `fail_closed`: kali refuses both lines with `E5506: computed member access …` at exit 1; node prints `undefined` twice at exit 0. FAIL_CLOSED, not FIXED, on purpose: `let i = 1; o[i]` needs a runtime lookup this project does not build (spec §5), and a fix that printed `0`, `0` would have been R-13's shape, not a closure — the case's own regression note says so. ~~SILENT (both scopes) — added 2026-09-08 at `02297ca6c2` …~~ |
```

(Keep the struck original text verbatim after `~~`; struck text in the status column is refused by the parser, so the strike goes in the note column only, as R-56's row does.)

Run: `cargo test -p kali_blast_radius oracle_tests` — expected green again.

- [ ] **Step 2: Write the close-out into both §2 entries**

Append to `### R-13` (after its `Confidence` bullet, `:2219`) and to `### R-59` (after its `Confidence` bullet, `:3737`) a bullet block on R-56's template (`register.md:2879-2966`): **RETIRED 2026-09-08, at `<SHA>`, by the computed-member-static-name project (spec path)** / **The fix, at the address this entry named** (R-59: `expression_to_property_name` returns `Option<String>`, the two parse sites store it, and a nameless member is `LirNodeKind::ComputedMember`; R-13: the fold in `static_analysis::computed_member`, the checker gates, the codegen gateway and store choke point, and the materialization record) / **What was measured, at `<SHA>` against node v26.8.1, both scopes, byte-identical** (the six oracle readings from Task 7 Step 3, verbatim) / **Coverage, stated rather than assumed** (pinned: the oracle pairs and `object/computed_member_static_name.toml`; measured by hand, not pinned: nothing — say so) / **What is NOT this entry** (R-13: the absent read `o[k]` with `k = "c"` still prints `0`, R-21's lane; the `let` key now refuses, spec §5. R-59: `o[true]`/`o[null]`/`o[1n]` refuse rather than fold, spec §5) / **Consequence for the ranking** (both leave the SILENT filter and `clusters.json`; R-59's singleton cluster definition is deleted with it, or `ranking.rs:326` refuses to run; R-13 leaves G3, which keeps R-12; the ranking regenerates and §6 records what the generator printed).

Also in R-13's entry, replace the struck **Mechanism hypothesis** bullet's trailing "the hypothesis is disproved, not replaced" with the traced mechanism: read lane = the fabricated name missing (R-59's chain); write lane = the bracket-store family (follow-up item 2.3), which a literal key showed vanishes too — measured `o["b"] = 8` → `o.b` still `2` at `dc19c3a040`. Set **Confidence** to "high on behaviour; high on mechanism (traced and closed)".

- [ ] **Step 3: Remove both entries from `clusters.json`**

Delete the R-13 assignment (`:274-280`), the R-59 assignment (`:442-449`) and the `R-59 (unclustered)` cluster definition (`:232-236`), and extend the note paragraph at `:31-43` with one sentence per entry on the R-56 sentence's pattern: `REMOVED 2026-09-08 at \`<SHA>\` by the computed-member-static-name project: R-13 (a G3 member; G3 survives on R-12) and R-59 (a singleton, so its cluster definition goes with it).`

Run: `cargo test -p kali_blast_radius` — expected red only on `spliced_document_matches_the_generator` (the ranking is stale), green on the clusters-vs-§0.2 agreement gate.

- [ ] **Step 4: Refresh the outputs and regenerate the ranking**

```bash
cargo build -p kali_cli --bin kali
cd tools/blast-radius && npm ci && node --test && node accepts.mjs && node count.mjs && cd ../..
git diff --stat tools/blast-radius/accepts.json tools/blast-radius/counts.json
cargo run -p kali_blast_radius --example rank > /tmp/claude-1000/-workspace/*/scratchpad/rank.md
```

Then splice: replace everything between `<!-- GENERATED:BEGIN … -->` (`blast-radius-ranking.md:121`) and `<!-- GENERATED:END -->` (`:495`) with the generator's stdout from `## 2. The bands` onward, and the provenance table between `<!-- GENERATED-PROVENANCE:BEGIN -->` and `<!-- GENERATED-PROVENANCE:END -->` with the stdout's provenance table (the `this document generated at` cell will read the current HEAD; that is the one permitted mismatch).

Commit the outputs refresh **alone** first:

```bash
git add tools/blast-radius/accepts.json tools/blast-radius/counts.json
git commit -m "measure(blast-radius): re-measure the accept set against the binary that refuses nameless computed members

Outputs only: predicates.json, matchers.mjs and both frozen SHAs are
untouched. Accept rate anchor <before>→<after>, extension <before>→<after>;
reachable counts moved on <N> entries (read from the counts.json diff).

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

Run: `cargo test -p kali_blast_radius` — expected green after the splice.

- [ ] **Step 5: Write the ranking amendment from what the generator printed**

Append to `blast-radius-ranking.md` §6 (before `### 6.1`) an amendment block on the pattern of the three 2026-08-16 amendments (`:503-…`), headed `**AMENDMENT 2026-09-08, at \`<SHA>\` — the first regeneration in which the ACCEPT SET moved.**` It must state, each read from the regenerated §2–§5 and the `counts.json` diff, not predicted: ranked entries before → after; clusters before → after; whether any band moved on either axis (diff the band blocks of the pre- and post-splice documents with `git diff`, and say "checked by differencing, not by eye"); G3's frequency before → after; the accept rates before → after (§5); which entries' reachable counts changed and why (programs with a computed member now refuse); whether §2.4's contested assignments changed verdict. Then a paragraph **Whether the figures matched what was predicted**, quoting spec §7's predictions (G3 keeps R-12 alone; R-59's singleton removed; the amendment records the movement) and saying which held. Close with **One divergence was found while measuring and is NOT filed**, naming `docs/superpowers/followups/member-length-renders-the-child-count.md` and stating that this project moved its computed half and left its dot half silent, which is the same disclosure R-59's own entry makes about the leads it did not file.

- [ ] **Step 6: Record the divergence this project found and did NOT fix**

Measuring the chained-access lane turned up a defect that is **not** R-13,
**not** R-59, and not caused by computed access at all: `render_length`'s
text-less arm returns a node's CHILD COUNT, so `.length` on any
member-expression receiver reads the receiver node's arity. Measured at
`dc19c3a040` against node v26.8.1, module scope:

| program | kali | node |
|---|---|---|
| `const o = {a: "xyz"}; o.a.length` | `1` | `3` |
| `const o = {a: "xyzwv"}; o.a.length` | `1` | `5` |
| `const o = {a: "xyz"}; o["a"].length` | `2` | `3` |
| `const o = {a: "xyzwv"}; const k = "a"; o[k].length` | `2` | `5` |
| `const o = {a:1,b:2,c:3,d:4}; const k = "keys"; Object[k](o).length` | `2` | `4` |
| `const s = "xyz"; s.length` | `3` | `3` (control, the static string lane is correct) |

The value tracks the node's child count (1 for a dot member, 2 for a bracket
member), never the string or the array. Exit 0, no diagnostic, both scopes.

Create `docs/superpowers/followups/member-length-renders-the-child-count.md`
with that table, the mechanism (`crates/kali_codegen/src/intrinsics/host.rs`'s
`render_length` text-less arm, and the sibling arm in `render_static_value`),
and this disclosure, which is the point of writing it down:

> **This project moves only half of it.** The computed-member half
> (`o["a"].length`, `o[k].length`, `Object[k](...)`) now refuses, because a
> nameless member declines both renderers and a member read on a nameless base
> is denied. **The DOT half is untouched and still silently prints `1`.**
> Nothing in the computed-member-static-name project's test set would notice if
> it got worse, and no register entry covers it: R-16/R-17 (cluster G5) are
> about a string HANDLE leaking as an integer, and this is a renderer returning
> an arity. Filing it as a section 2 entry is a dozen coordinated edits (a
> predicate, a matcher, a counts re-freeze, an oracle pair, a clusters row),
> which is a task of its own.

Add a **wrong-on-purpose** case for the half that did not move, in
`crates/kali_cli/tests/cases/object/computed_member_static_name.toml`, with
`"dot_member_length.js" = ` + Q + `const o = {a: "xyz"}; console.log(o.a.length);` + Q +
` in that file's `[source]`:

```toml
[[case]]
name = "a_dot_member_length_still_renders_the_child_count"
rationale = """WRONG ON PURPOSE, and NO TASK IN THIS PLAN OWNS CLOSING IT. node prints 3; kali prints 1, and prints 1 for every string length (measured at `dc19c3a040` with "xyz" and "xyzwv"). The value is the member node's child count, returned by `render_length`'s text-less arm. See docs/superpowers/followups/member-length-renders-the-child-count.md.

WHY IT IS HERE. The computed half of this defect (`o[k].length`) refuses as of this project, and the case above pins that. Without this case the dot half would be silently unpinned, and a reader would reasonably infer from the refusal that the whole family was handled. It was not."""
args = ["run", "dot_member_length.js"]
exit = "success"
stdout = "1\n"
```

Re-measure both halves before writing either down.

- [ ] **Step 7: The follow-up file, the plan, and the doc comments**

- `console-render-unification-discovered-defects.md:117`: rename the heading to `### 2.3 Bracket assignment to an existing key does not take — FIXED at <SHA>` and append a paragraph: the write was dropped, not the read stale (codegen's assignment emitter returned `false` for a two-child target and the caller emitted a bare read); fixed by the computed-member-static-name project's store choke point; re-measured at `<SHA>` against node v26.8.1: all three lines print `7`; pinned by `a_literal_string_key_bracket_store_lands` and `a_literal_number_key_bracket_store_lands`.
- `plan/phase-21/README.md:43`: append to the §21.3 progress line: `The computed member access slice now folds a bare \`const\` key with a string or number literal initializer to the dot spelling for reads and stores on object-literal receivers, with standalone run, JSON check, and check/run parity evidence, while a \`let\`/\`var\`/parameter key, a non-literal index, a boolean/\`null\`/BigInt literal index, a string receiver, and any element access on an array literal remain on the canonical \`E5506\` gate in both \`check\` and \`run\`.`
- Doc comments: `crates/kali_codegen/src/intrinsics/object.rs:86-92`, `crates/kali_optimize/src/object_fold.rs:767-775`, `crates/kali_optimize/src/helpers.rs:172` — replace each "true only for the shapes this function reads" qualification with: the one-currency claim holds for every computed access that HAS a name, because since the computed-member-static-name project a computed access whose index the parser cannot read has none (`property: None`, `LirNodeKind::ComputedMember`) and never reaches a name-reading consumer.

- [ ] **Step 8: Full verification and commit**

Run: `cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && bash scripts/test-gate.sh 2>&1 | tail -5`
Expected: clean, `0 failed`.

```bash
git add docs plan tools/blast-radius/clusters.json crates/kali_cli/tests/cases crates/kali_codegen/src/intrinsics/object.rs crates/kali_optimize/src
git commit -m "docs(register): R-13 and R-59 close, and the ranking regenerates

Both §0.2 rows re-derived from their cases (fixed / fail_closed), both §2
entries carry the close-out, both leave clusters.json (R-59 with its
singleton definition), the ranking is re-spliced from the generator with
the accept set re-measured, follow-up item 2.3 is marked fixed, and the
phase-21 progress line names the admitted and refused sets. Also records the
.length-renders-the-child-count divergence found while measuring, whose dot
half this project leaves silent and pins wrong-on-purpose.

Claude-Session: https://claude.ai/code/session_01GjDW13LKxprVAKyvhTvg3h"
```

Then run `cargo test -p kali_blast_radius` once more: the `this document generated at` cell is the only permitted mismatch, and `spliced_document_matches_the_generator` must be green.
