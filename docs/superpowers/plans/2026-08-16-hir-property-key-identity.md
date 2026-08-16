# Property-Key Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make an HIR key-slot node's text be the property name itself — `String(key)` — so that R-56 and its two siblings close at the root cause instead of at a consumer.

**Architecture:** The type of a property key (`Identifier` / `Number` / `String` / BigInt) is known in the parser and destroyed by the time HIR stores it, so consumers re-derive it from quoting conventions that invert between slots. This plan restores the discriminator where the type still exists: the parser gains a BigInt variant and stops fabricating the key `0`, and `lower_property_name` computes the JavaScript property name once via `format_js_number`. The codegen machinery that guessed the type — `KeyTextSlot`, `is_hir_numeric_key_spelling` and its NaN guard — is then deleted rather than repaired.

**Tech Stack:** Rust (workspace crates `kali_ast`, `kali_parser`, `kali_hir`, `kali_types`, `kali_codegen`, `kali_optimize`, `kali_mir`, `kali_common`, `kali_blast_radius`), the declarative TOML case corpus under `crates/kali_cli/tests/cases/`, `node v26.7.0` as oracle.

**Spec:** `docs/superpowers/specs/2026-08-16-hir-property-key-identity-design.md`

## Global Constraints

- **Baseline for every "before" measurement:** commit `5c80f081d2`, binary `.cache/cargo-target/debug/kali` (`kali 0.1.0`), oracle `node v26.7.0`.
- **Every claim is a reading, never a memory.** Any number, transcript or verdict written into a doc or a case rationale must be produced by a command run at the commit it names, in that task.
- **Both scopes.** Every behavioural case exists at module scope *and* inside a function; they are different programs in kali.
- **Classify every re-pin** in the commit that makes it, as either *was wrong, now right* or *was right, now spelled differently*. An unclassified re-pin is forbidden.
- **Instrument changes commit separately** from the findings they enable (ranking spec §4.3).
- **Do not add callers to `canonical_property_key_text`.** Its two current call sites are safe only because they were written together (follow-up file §3).
- **Never `cd`;** run every command from `/workspace`.
- Build the CLI with `cargo build -p kali_cli --bin kali`; the binary lands at `.cache/cargo-target/debug/kali`.

---

### Task 1: Pin the three defects and the two controls

Tests only. No source file changes. Every case here records what kali does **today**, so the suite is green at the end of this task and each later task turns a specific, named case red.

**Files:**
- Create: `crates/kali_cli/tests/cases/object/property_key_identity.toml`

**Interfaces:**
- Consumes: nothing.
- Produces: case ids under `object/property_key_identity::` that Tasks 2, 3 and 4 re-pin by name.

- [ ] **Step 1: Re-measure every pin against the live binary before writing it**

```bash
cargo build -p kali_cli --bin kali
K=.cache/cargo-target/debug/kali
S=/tmp/claude-1000/-workspace/scratch-pin && mkdir -p $S

cat > $S/bigint_module.js <<'EOF'
const o = {42n: 1};
console.log(Object.hasOwn(o, 0));
console.log(o[0]);
console.log(o[42]);
EOF
$K run $S/bigint_module.js; echo "exit=$?"
node $S/bigint_module.js; echo "node exit=$?"

cat > $S/keys_module.js <<'EOF'
for (const k of Object.keys({1e-7: 1})) console.log(k);
for (const k of Object.keys({[-1e999]: 1, [1e21]: 2})) console.log(k);
EOF
$K run $S/keys_module.js; echo "exit=$?"
node $S/keys_module.js; echo "node exit=$?"

cat > $S/controls_module.js <<'EOF'
const o = {5: 1};
const p = {"5": 1};
console.log(Object.hasOwn(o, 5));
console.log(Object.hasOwn(p, 5));
console.log(Object.hasOwn(o, "5"));
const q = {'"5"': 1};
console.log(q['"5"']);
EOF
$K run $S/controls_module.js; echo "exit=$?"
node $S/controls_module.js; echo "node exit=$?"
```

Expected from kali at the baseline: `true / 1 / 0`, then `0.0000001 / -inf / 1000000000000000000000`, then `true / true / true / 1`. If any line differs, **stop** — the tree has moved off the spec's baseline and the spec's §2 must be re-derived before continuing.

- [ ] **Step 2: Write the case file pinning exactly those answers**

Create `crates/kali_cli/tests/cases/object/property_key_identity.toml`:

```toml
# Pins for the property-key identity project (spec
# docs/superpowers/specs/2026-08-16-hir-property-key-identity-design.md).
#
# EVERY EXPECTATION IN THIS FILE IS WRONG ON PURPOSE at the commit that adds
# it. These cases record what kali does at `5c80f081d2` so that the fix has
# something to turn red. The `rationale` on each says what node does and which
# task is expected to move it; a case that moves without its task moving is a
# surprise, which is the point.
#
# The two control cases at the end are the opposite: they are CORRECT today and
# must stay correct. `{5:1}` and `{"5":1}` are the same property in JavaScript,
# and a discriminator that separated them would fix R-56 by breaking conforming
# programs.

[source]
"bigint_module.js" = """const o = {42n: 1};
console.log(Object.hasOwn(o, 0));
console.log(o[0]);
console.log(o[42]);
"""
"bigint_function.js" = """function main() {
  const o = {42n: 1};
  console.log(Object.hasOwn(o, 0));
  console.log(o[0]);
  console.log(o[42]);
}
main();
"""
"keys_small_module.js" = """for (const k of Object.keys({1e-7: 1})) console.log(k);
"""
"keys_small_function.js" = """function main() {
  for (const k of Object.keys({1e-7: 1})) console.log(k);
}
main();
"""
"keys_extreme_module.js" = """for (const k of Object.keys({[-1e999]: 1, [1e21]: 2})) console.log(k);
"""
"keys_extreme_function.js" = """function main() {
  for (const k of Object.keys({[-1e999]: 1, [1e21]: 2})) console.log(k);
}
main();
"""
"numeric_string_same_key_module.js" = """const o = {5: 1};
const p = {"5": 1};
console.log(Object.hasOwn(o, 5));
console.log(Object.hasOwn(p, 5));
console.log(Object.hasOwn(o, "5"));
"""
"numeric_string_same_key_function.js" = """function main() {
  const o = {5: 1};
  const p = {"5": 1};
  console.log(Object.hasOwn(o, 5));
  console.log(Object.hasOwn(p, 5));
  console.log(Object.hasOwn(o, "5"));
}
main();
"""
"quoted_numeric_member_read_module.js" = """const q = {'"5"': 1};
console.log(q['"5"']);
"""
"quoted_numeric_member_read_function.js" = """function main() {
  const q = {'"5"': 1};
  console.log(q['"5"']);
}
main();
"""

[[case]]
name = "bigint_key_is_stored_under_zero_module_scope"
rationale = "WRONG ON PURPOSE. node prints `false`, `undefined`, `1`: the property is `\"42\"`. kali's parser cannot read `42n` as an f64 and falls back to `unwrap_or(0.0)` (kali_parser/src/expression/object.rs:52), so the property is stored under the key `0` and a program reads a value out of a key it never wrote. Task 2 moves this to node's answer."
args = ["run", "bigint_module.js"]
exit = "success"
stdout = "true\n1\n0\n"

[[case]]
name = "bigint_key_is_stored_under_zero_in_function"
rationale = "WRONG ON PURPOSE. Same as the module-scope case, in function scope, because top-level and in-function are different programs in kali. node prints `false`, `undefined`, `1`. Task 2 moves this."
args = ["run", "bigint_function.js"]
exit = "success"
stdout = "true\n1\n0\n"

[[case]]
name = "object_keys_renders_small_magnitude_key_with_rust_display_module_scope"
rationale = "WRONG ON PURPOSE. node prints `1e-7`. kali prints `0.0000001` because HIR stores the key with Rust's `Display for f64` and the enumeration fold renders the key node through the expression renderer. Object.keys returns STRINGS, so this text propagates into comparisons and JSON round-trips. Tasks 3 and 4 move this."
args = ["run", "keys_small_module.js"]
exit = "success"
stdout = "0.0000001\n"

[[case]]
name = "object_keys_renders_small_magnitude_key_with_rust_display_in_function"
rationale = "WRONG ON PURPOSE. Same defect in function scope; node prints `1e-7`. Tasks 3 and 4 move this."
args = ["run", "keys_small_function.js"]
exit = "success"
stdout = "0.0000001\n"

[[case]]
name = "object_keys_leaks_rust_infinity_spelling_module_scope"
rationale = "WRONG ON PURPOSE, and the worst line in this file: `-inf` is not a string any JavaScript program can produce. node prints `-Infinity` then `1e+21`. Measured for the first time by this project; the follow-up file's §2.5 recorded only the 1e-7 lane. Tasks 3 and 4 move this."
args = ["run", "keys_extreme_module.js"]
exit = "success"
stdout = "-inf\n1000000000000000000000\n"

[[case]]
name = "object_keys_leaks_rust_infinity_spelling_in_function"
rationale = "WRONG ON PURPOSE. Same defect in function scope; node prints `-Infinity` then `1e+21`. Tasks 3 and 4 move this."
args = ["run", "keys_extreme_function.js"]
exit = "success"
stdout = "-inf\n1000000000000000000000\n"

[[case]]
name = "numeric_and_quoted_numeric_keys_are_the_same_property_module_scope"
rationale = "CONTROL -- correct today and must stay correct. JavaScript gives `{5:1}` and `{\"5\":1}` the same property name, and both probes agree with node at `true true true`. This case is what a discriminator that over-separates would break, so it is the guard on Task 3 rather than a target of it."
args = ["run", "numeric_string_same_key_module.js"]
exit = "success"
stdout = "true\ntrue\ntrue\n"

[[case]]
name = "numeric_and_quoted_numeric_keys_are_the_same_property_in_function"
rationale = "CONTROL -- correct today and must stay correct, in function scope. node agrees at `true true true`."
args = ["run", "numeric_string_same_key_function.js"]
exit = "success"
stdout = "true\ntrue\ntrue\n"

[[case]]
name = "quoted_numeric_string_key_member_read_is_correct_module_scope"
rationale = "CONTROL -- correct today and must stay correct. The member read of R-56's key already agrees with node (`1`); it is the two `Object.hasOwn` probes in the oracle case `r56a_quoted_numeric_string_key_module_scope` that diverge. Keeping the read pinned here means a fix that repairs the probes by breaking the read cannot pass."
args = ["run", "quoted_numeric_member_read_module.js"]
exit = "success"
stdout = "1\n"

[[case]]
name = "quoted_numeric_string_key_member_read_is_correct_in_function"
rationale = "CONTROL -- correct today and must stay correct, in function scope. node prints `1`."
args = ["run", "quoted_numeric_member_read_function.js"]
exit = "success"
stdout = "1\n"
```

- [ ] **Step 3: Run the new cases and verify they pass**

```bash
cargo test -p kali_cli --test cases -- object/property_key_identity
```

Expected: 10 passed. They pass because they pin today's behaviour. A failure here means Step 1's transcript was not transcribed faithfully — fix the expectation to match the measurement, never the reverse.

- [ ] **Step 4: Verify the filter matched something**

The runner treats a filter matching zero trials as a hard error, so a green `0 passed` is impossible — but confirm the count is 10 and not, say, 2, which would mean the file parsed only partially.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cases/object/property_key_identity.toml
git commit -m "test(object): pin the property-key identity family before touching it

Ten cases, both scopes. Six are WRONG ON PURPOSE -- they record what kali does
at 5c80f081d2 so the fix has something to turn red: the BigInt key stored under
0, and Object.keys rendering 1e-7 as 0.0000001 and -1e999 as -inf.

Four are controls that must not move: {5:1} and {\"5\":1} are the same property
in JS, and R-56's member read is already correct."
```

---

### Task 2: `PropertyName::BigInt`, and the parser stops fabricating keys

**Files:**
- Modify: `crates/kali_ast/src/literal.rs:37-42`
- Modify: `crates/kali_parser/src/expression/object.rs:49-56`
- Modify: `crates/kali_hir/src/lowering/object.rs:20-43`
- Modify: `crates/kali_types/src/monomorphize.rs:963`, `crates/kali_types/src/late_host.rs:287`, `crates/kali_types/src/repr_infer.rs:1713`, `crates/kali_types/src/resolve/expression.rs:2769`, `crates/kali_types/src/static_analysis/object.rs:465`, `crates/kali_types/src/static_analysis/object.rs:533`
- Test: `crates/kali_parser/src/expression/object_tests.rs`
- Test: `crates/kali_cli/tests/cases/object/property_key_identity.toml` (re-pin two cases)

**Interfaces:**
- Consumes: Task 1's case ids.
- Produces: `PropertyName::BigInt(String)` holding decimal digits **without** the `n` suffix; HIR stores those digits verbatim as the key text.

- [ ] **Step 1: Write the failing parser test**

Append to `crates/kali_parser/src/expression/object_tests.rs`:

```rust
#[test]
fn bigint_object_property_keys_keep_their_digits() {
    let obj = parse_object_literal("({42n: 1})");
    assert_eq!(
        obj.properties[0].key,
        PropertyName::BigInt("42".to_string())
    );
}

#[test]
fn large_bigint_object_property_keys_are_exact() {
    // The whole reason the variant holds text: this value has no exact f64.
    let obj = parse_object_literal("({123456789012345678901234567890n: 1})");
    assert_eq!(
        obj.properties[0].key,
        PropertyName::BigInt("123456789012345678901234567890".to_string())
    );
}
```

Use whatever helper the neighbouring tests in that file already use to reach an `ObjectExpression` (they parse a source string and index `obj.properties`); match their style rather than introducing a new harness.

- [ ] **Step 2: Run it and watch it fail**

```bash
cargo test -p kali_parser bigint_object_property_keys_keep_their_digits
```

Expected: a compile error, `no variant named BigInt found for enum PropertyName`.

- [ ] **Step 3: Add the AST variant**

In `crates/kali_ast/src/literal.rs`:

```rust
/// Property name
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PropertyName {
    Identifier(String),
    Number(f64),
    String(String),
    /// A BigInt literal key, as its decimal digits without the `n` suffix.
    ///
    /// Text, not a parsed value: `{123456789012345678901234567890n: 1}` has no
    /// exact `f64`, and the property name JavaScript computes for it is the
    /// exact digits (`String(123456789012345678901234567890n)`).
    BigInt(String),
}
```

- [ ] **Step 4: Give the parser a BigInt branch and delete the fabricated key**

The lexer consumes a trailing `n` into the `NumericLiteral` token (`kali_lexer/src/number.rs:54-58`), so the parser sees the text `42n`. Replace the numeric-key arm in `crates/kali_parser/src/expression/object.rs`:

```rust
Some(TokenType::NumericLiteral) => {
    let text = self
        .stream
        .advance()
        .map(|token| token.value)
        .unwrap_or_default();
    let _ = self.stream.accept(TokenType::Colon);
    let Some(name) = numeric_property_name(&text) else {
        // NOT `unwrap_or(0.0)`. Fabricating the key `0` for a literal this
        // parser could not read is how `{42n: 1}` came to answer
        // `Object.hasOwn(o, 0)` with `true` -- a program reading a value out
        // of a key it never wrote. If it cannot be read, it is refused.
        self.push_feature_unavailable(
            "this numeric property key is unavailable in the current phase; use a decimal or string literal key",
        );
        let _ = self.parse_expression();
        continue;
    };
    (name, self.parse_expression())
}
```

and add the free function beside it:

```rust
/// The property name a numeric-literal key token denotes, or `None` when the
/// token is not one this phase can read.
///
/// The BigInt arm keeps DIGITS: `String(42n)` is `"42"`, exactly, for values
/// with no exact `f64`.
fn numeric_property_name(text: &str) -> Option<PropertyName> {
    if let Some(digits) = text.strip_suffix('n') {
        return (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| PropertyName::BigInt(digits.to_string()));
    }
    text.parse::<f64>().ok().map(PropertyName::Number)
}
```

Leave `computed_object_property_name` (`:136`, unary arm at `:172-191`) alone: `{[42n]: 1}` already fails closed with `E5506` at the baseline (measured), so no BigInt reaches the computed path.

- [ ] **Step 5: Add the HIR arm**

In `crates/kali_hir/src/lowering/object.rs`, beside the existing arms:

```rust
PropertyName::BigInt(digits) => {
    // `String(42n)` is "42": the digits ARE the property name.
    self.builder
        .alloc_text(HirNodeKind::Literal, None, digits.clone())
}
```

- [ ] **Step 6: Add the six `kali_types` arms the compiler now demands**

Each of these already declines numeric keys; a BigInt key gets the same treatment, sharing the arm rather than copying its body:

```rust
// crates/kali_types/src/monomorphize.rs:963
PropertyName::Number(_) | PropertyName::BigInt(_) => return None,

// crates/kali_types/src/late_host.rs:290
PropertyName::Number(_) | PropertyName::BigInt(_) => continue,

// crates/kali_types/src/repr_infer.rs:1716 -- extend the existing arm's pattern
kali_ast::PropertyName::Number(_) | kali_ast::PropertyName::BigInt(_) => {

// crates/kali_types/src/resolve/expression.rs:2769
PropertyName::Identifier(_)
| PropertyName::Number(_)
| PropertyName::String(_)
| PropertyName::BigInt(_) => {}

// crates/kali_types/src/static_analysis/object.rs:465 and :533 -- add to both matches!
PropertyName::Identifier(_)
    | PropertyName::Number(_)
    | PropertyName::String(_)
    | PropertyName::BigInt(_)
```

- [ ] **Step 7: Run the parser tests and the workspace build**

```bash
cargo test -p kali_parser expression::object
cargo build --workspace
```

Expected: the two new tests pass; the workspace compiles with no non-exhaustive-match errors left.

- [ ] **Step 8: Re-measure the BigInt program and re-pin its two cases**

```bash
cargo build -p kali_cli --bin kali
.cache/cargo-target/debug/kali run /tmp/claude-1000/-workspace/scratch-pin/bigint_module.js
node /tmp/claude-1000/-workspace/scratch-pin/bigint_module.js
```

Expected from both: `false`, `undefined`, `1`.

If kali prints `undefined` for `o[0]` and `1` for `o[42]`, update both BigInt cases in `property_key_identity.toml` to `stdout = "false\nundefined\n1\n"` and rewrite each `rationale` to record the classification:

> **RE-PINNED, was wrong now right** (Task 2, at `<commit>`): kali and node both print `false`, `undefined`, `1`. The parser now reads `42n` as `PropertyName::BigInt("42")` instead of falling back to the key `0`.

If kali prints something else — in particular if `o[0]` prints `0` rather than `undefined` — that is a *different* defect (absent-property reads), not this one. Pin what it actually prints, classify it as *was wrong, now differently wrong*, and record the residual in the task's commit message; do not adjust the fix to chase it.

- [ ] **Step 9: Run the full case corpus and classify anything else that moved**

```bash
cargo test -p kali_cli --test cases 2>&1 | tail -30
cargo test --workspace 2>&1 | tail -30
```

Expected: only the two re-pinned cases moved. Any other red case is either a fabricated-key site nobody knew about (classify it, pin it, keep it) or a regression (stop and diagnose before continuing).

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "fix(parser): a BigInt property key keeps its digits, and an unreadable key is refused

PropertyName gains a BigInt variant holding decimal digits as text -- text
because {123456789012345678901234567890n: 1} has no exact f64 and the property
name is the exact digits.

The unwrap_or(0.0) fallback is gone. It stored {42n: 1} under the key 0, so
Object.hasOwn(o, 0) answered true and o[0] read 1 -- a program reading a value
out of a key it never wrote. An unreadable numeric key is now refused through
the same push_feature_unavailable path the computed-key arm already takes.

Re-pinned, was wrong now right: the two bigint cases in
object/property_key_identity.toml."
```

---

### Task 3: HIR stores the property name, and the key slot collapses

This is the task that closes R-56. The HIR change and the codegen collapse must land **together**: with HIR still writing `"5"` for a numeric key, the key-slot arm's renumbering is what makes `{'"5"': 1}` collide, and with the arm still renumbering, HIR's new bare text would be re-derived by a predicate that no longer has anything to derive.

**Files:**
- Modify: `crates/kali_hir/src/lowering/object.rs:26-37`
- Modify: `crates/kali_hir/Cargo.toml` (only if `kali_common`'s `js_number` module is not already reachable — it is a dependency already)
- Modify: `crates/kali_codegen/src/intrinsics/object.rs:5-197`
- Test: `crates/kali_hir/src/lowering/object_tests.rs:35-102` (three tests re-pinned)
- Test: `crates/kali_codegen/src/intrinsics/object_tests/has_own.rs:228-560` (four tests rewritten or deleted)

**Interfaces:**
- Consumes: `PropertyName::BigInt` from Task 2.
- Produces: the invariant *an HIR key-slot node's text is `String(key)`*, on which Tasks 4 and 5 depend. `canonical_property_key_text(text: &str) -> String` survives with the slot parameter **removed**, and means "the property name this EXPRESSION denotes".

- [ ] **Step 1: Write the failing HIR test**

Replace the three quoting tests in `crates/kali_hir/src/lowering/object_tests.rs` (`test_numeric_object_property_names_lower_as_string_literals` at `:35`, `..._from_parsed_source_...` at `:58`, `..._negative_zero_...` at `:84`) with one table-driven test in their place:

```rust
#[test]
fn numeric_object_property_names_lower_to_their_javascript_property_name() {
    // The invariant: a key-slot node's text IS `String(key)`. No quoting
    // marker, because a marker is what made `{'"5"': 1}` and `{5: 1}` the same
    // text (register R-56), and Rust's `Display` is what made `{1e-7: 1}`'s key
    // `0.0000001` instead of `1e-7`.
    for (source, expected) in [
        ("({3: 1})", "3"),
        ("({5: 1})", "5"),
        ("({1e-7: 1})", "1e-7"),
        ("({1e21: 1})", "1e+21"),
        ("({[-0]: 1})", "0"),
        ("({[-1e999]: 1})", "-Infinity"),
    ] {
        let (program, key) = lower_first_property_key(source);
        assert_eq!(
            program.node(key).text.as_deref(),
            Some(expected),
            "source {source}"
        );
    }
}
```

Write `lower_first_property_key` as a small helper in the same test file if one does not already exist, following the lowering setup the existing tests at `:6-34` use.

- [ ] **Step 2: Run it and watch it fail**

```bash
cargo test -p kali_hir numeric_object_property_names_lower_to_their_javascript_property_name
```

Expected: FAIL, first mismatch `Some("\"3\"")` vs `Some("3")`.

- [ ] **Step 3: Make `lower_property_name` compute the name**

Replace the body in `crates/kali_hir/src/lowering/object.rs`:

```rust
    /// A property key lowers to its JAVASCRIPT PROPERTY NAME -- `String(key)`
    /// -- and to nothing else. No quoting marker, in either direction.
    ///
    /// The marker this replaces encoded "was a number" as a leading `"`, which
    /// a string key's own content can also carry: `{'"5"': 1}` and `{5: 1}`
    /// reached codegen as the same text, so `Object.hasOwn` answered both
    /// wrongly at exit 0 while the member read still worked (register R-56).
    /// No downstream predicate could recover the difference, which is why the
    /// discriminator is spent HERE, where the type is still known, rather than
    /// re-derived there.
    ///
    /// `format_js_number` is the same function every other lane renders a
    /// number with, so a key's name and a number's rendering cannot drift.
    pub(crate) fn lower_property_name(&mut self, name: &PropertyName) -> HirNodeId {
        let text = match name {
            PropertyName::Identifier(value) | PropertyName::String(value) => value.clone(),
            PropertyName::Number(value) => format_js_number(*value),
            PropertyName::BigInt(digits) => digits.clone(),
        };
        self.builder.alloc_text(HirNodeKind::Literal, None, text)
    }
```

with `use kali_common::js_number::format_js_number;` at the top of the file.

- [ ] **Step 4: Run the HIR test and watch it pass**

```bash
cargo test -p kali_hir
```

Expected: the new test passes. Other `kali_hir` tests that spell a numeric key as `"3"` may now fail — re-pin each, classified *was right, now spelled differently* (the node text changed; no behaviour claim moved).

- [ ] **Step 5: Collapse the key slot in codegen**

In `crates/kali_codegen/src/intrinsics/object.rs`:

1. Delete `KeyTextSlot` (`:5-25`) and `is_hir_numeric_key_spelling` (`:42-93`) entirely, including their doc comments — they document a convention that no longer exists.
2. Reduce `canonical_property_key_text` to the former `Expression` arm and drop the `slot` parameter:

```rust
/// The property key an EXPRESSION denotes, computed the way JS does
/// (`String(key)`).
///
/// Only one currency exists now: a key-slot node's text is already the
/// property name (`kali_hir`'s `lower_property_name`), so this function is for
/// the PROBE side alone -- the key an expression evaluates to. Its convention
/// is invertible and was never the defect: a quote character means a string
/// literal whose content is the key, an `n` suffix means BigInt digits, and a
/// bare text is a number's spelling to be rendered.
pub(crate) fn canonical_property_key_text(text: &str) -> String {
    let long_enough = text.len() >= 2;
    let quoted = long_enough
        && matches!(
            (text.chars().next(), text.chars().last()),
            (Some('"'), Some('"')) | (Some('\''), Some('\'')) | (Some('`'), Some('`'))
        );
    if quoted {
        return text[1..text.len() - 1].to_string();
    }
    // `String(42n)` is "42": exact, and textual, so the digits of a BigInt too
    // large for an `f64` survive.
    if is_bigint_literal_text(text) {
        return text[..text.len() - 1].to_string();
    }
    parse_numeric_literal_value(text)
        .map(format_js_number)
        .unwrap_or_else(|| text.to_string())
}
```

3. Split `static_property_key_text` into the two things it was doing:

```rust
    /// The property name a KEY-SLOT node holds.
    ///
    /// Its text is already `String(key)`, so this reads it. Key-slot nodes are
    /// never resolved as bindings (`{a: 1}`'s key is the name `a`, not the
    /// value of a variable `a`), which is why this does not defer to
    /// `render_static_value`.
    pub(crate) fn static_object_key_text(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        (node.kind == LirNodeKind::Literal)
            .then(|| node.text.clone())
            .flatten()
    }

    /// The property name a PROBE expression denotes.
    pub(crate) fn static_probe_key_text(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Literal {
            return Some(canonical_property_key_text(node.text.as_deref()?));
        }
        self.render_static_value(id)
    }
```

4. Update the three call sites: `:553` and `:637` take `static_probe_key_text`; `:568` (the stored-key side of `static_object_has_own`) takes `static_object_key_text`. Keep the comment at `:549-552` explaining why both sides go through one currency — it is still true, and now it is true by construction.

- [ ] **Step 6: Rewrite the four convention tests**

In `crates/kali_codegen/src/intrinsics/object_tests/has_own.rs`, `canonical_property_key_text_reads_the_two_slots_opposite_ways` (`:229`), `..._never_renames_a_string_key` (`:276`), `..._renumbers_only_hir_double_quoted_keys` (`:366`) and `object_literal_key_renumbers_exactly_the_spellings_hir_can_write` (`:487`) all test the deleted convention. Replace all four with one:

```rust
#[test]
fn probe_key_text_is_the_property_name_the_expression_denotes() {
    use super::super::canonical_property_key_text;

    // Numbers render as JS renders them.
    assert_eq!(canonical_property_key_text("1000000000000000000000"), "1e+21");
    assert_eq!(canonical_property_key_text("0.0000001"), "1e-7");
    assert_eq!(canonical_property_key_text("5"), "5");

    // A quote means a string literal, whose content is the key however spelled.
    assert_eq!(canonical_property_key_text("\"1000000000000000000000\""), "1000000000000000000000");
    assert_eq!(canonical_property_key_text("\"a\""), "a");
    assert_eq!(canonical_property_key_text("'b'"), "b");

    // BigInt digits survive exactly.
    assert_eq!(canonical_property_key_text("42n"), "42");
    assert_eq!(
        canonical_property_key_text("123456789012345678901234567890n"),
        "123456789012345678901234567890"
    );
}
```

- [ ] **Step 7: Run the codegen tests**

```bash
cargo test -p kali_codegen 2>&1 | tail -30
```

Expected: green, or red only on tests that spell a stored key with quotes — re-pin those, classified.

- [ ] **Step 8: Measure R-56 against node**

```bash
cargo build -p kali_cli --bin kali
S=/tmp/claude-1000/-workspace/scratch-pin
cat > $S/r56.js <<'EOF'
const o = {'"5"': 1};
console.log(o['"5"']);
console.log(Object.hasOwn(o, '"5"'));
console.log(Object.hasOwn(o, 5));
EOF
.cache/cargo-target/debug/kali run $S/r56.js
node $S/r56.js
```

Expected from both: `1`, `true`, `false`. **This is the fix's whole claim**; if the second and third lines do not match node, do not proceed to Task 4.

- [ ] **Step 9: Confirm the oracle case goes red**

```bash
cargo test -p kali_cli --test cases -- oracle/tier2 2>&1 | grep -i r56
```

Expected: `r56a_quoted_numeric_string_key_module_scope` and `..._in_function` FAIL, because they assert `verdict = "silent"` and the run now classifies FIXED. That failure is the signal, and Task 6 acts on it. The case's own rationale predicted this: *"If the entry is genuinely CLOSED … this case turns FIXED and goes red against its `silent` verdict."*

- [ ] **Step 10: Confirm the controls did not move**

```bash
cargo test -p kali_cli --test cases -- object/property_key_identity 2>&1 | tail -20
```

Expected: the four control cases still pass. If `numeric_and_quoted_numeric_keys_are_the_same_property_*` failed, the discriminator over-separated `{5:1}` from `{"5":1}` and the fix is wrong — stop.

- [ ] **Step 11: Run the whole workspace and classify every re-pin**

```bash
cargo test --workspace 2>&1 | tail -40
```

The `Object.keys` cases from Task 1 are expected to still be red-or-moved; Task 4 owns them, so pin whatever they print now with an explicit *moved by Task 3, finished by Task 4* note rather than leaving them stale.

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "fix(hir): a property key lowers to its JavaScript property name, closing R-56

lower_property_name now stores String(key) -- format_js_number for numbers,
digits for BigInts, the name verbatim otherwise -- with no quoting marker.

The marker was the defect. It encoded 'was a number' as a leading double quote,
which a string key's own content can also carry, so {'\"5\"': 1} and {5: 1}
reached codegen as the same text and Object.hasOwn answered both wrongly at
exit 0 (R-56). No consumer-side predicate could recover the difference; five
rounds of trying is on the record in the follow-up file.

Retired with it: KeyTextSlot, is_hir_numeric_key_spelling, its NaN guard, and
the four tests that pinned the convention. canonical_property_key_text survives
as the probe-side function only.

Measured: {'\"5\"': 1} now answers 1/true/false, matching node."
```

---

### Task 4: `Object.keys` yields strings

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/object.rs:759-815` (`collect_object_enumeration_iteration_items`)
- Test: `crates/kali_cli/tests/cases/object/property_key_identity.toml` (re-pin four cases)

**Interfaces:**
- Consumes: Task 3's invariant — a key node's text is the property name.
- Produces: enumeration items for `Keys` / `ReflectOwnKeys` that are string-valued nodes, not key nodes.

- [ ] **Step 1: Measure what Task 3 left**

```bash
.cache/cargo-target/debug/kali run /tmp/claude-1000/-workspace/scratch-pin/keys_module.js
node /tmp/claude-1000/-workspace/scratch-pin/keys_module.js
```

Record the output verbatim. The printed text may already look right — the key node's text is now canonical — while `typeof k` is still `number`, which is the defect this task closes.

- [ ] **Step 2: Write the failing case for the type, not just the text**

Add to `crates/kali_cli/tests/cases/object/property_key_identity.toml`, in `[source]`:

```toml
"keys_typeof_module.js" = """for (const k of Object.keys({5: 1})) console.log(typeof k);
"""
"keys_typeof_function.js" = """function main() {
  for (const k of Object.keys({5: 1})) console.log(typeof k);
}
main();
"""
```

and the cases:

```toml
[[case]]
name = "object_keys_yields_strings_not_numbers_module_scope"
rationale = "Object.keys returns STRINGS. Printed output alone cannot tell the two apart -- console.log(\"5\") and console.log(5) are the same line -- so this case probes the type directly. node prints `string`."
args = ["run", "keys_typeof_module.js"]
exit = "success"
stdout = "string\n"

[[case]]
name = "object_keys_yields_strings_not_numbers_in_function"
rationale = "Same probe in function scope; node prints `string`."
args = ["run", "keys_typeof_function.js"]
exit = "success"
stdout = "string\n"
```

- [ ] **Step 3: Run and watch it fail**

```bash
cargo test -p kali_cli --test cases -- object/property_key_identity::object_keys_yields_strings
```

Expected: FAIL, printing `number` (or a `typeof` result that is not `string`). If it already prints `string`, record that in the rationale and skip Steps 4-5 — the fold was already string-valued and only the text was wrong.

- [ ] **Step 4: Push a string-valued node, not the key node**

In `collect_object_enumeration_iteration_items`, the `Keys` / `ReflectOwnKeys` arm currently pushes the key node id (`items.push(key)`). Push a scratch literal carrying the key's text in string form instead, using the same `alloc_scratch_node` the `Entries` arm already uses for its pairs:

```rust
ObjectEnumerationMode::Keys | ObjectEnumerationMode::ReflectOwnKeys => {
    // `Object.keys` yields STRINGS. The key node's text is the property
    // name (`lower_property_name`), so the name is already right -- but
    // handing the key node itself to the expression renderer makes a
    // numeric-looking name render as a NUMBER, and `typeof k` then answers
    // `number` for a value the language says is a string.
    let key_text = self.node(key).text.clone()?;
    items.push(self.alloc_scratch_string_literal(&key_text));
}
```

If no `alloc_scratch_string_literal` helper exists, add one beside `alloc_scratch_node` that allocates a `Literal` whose text is the source-quoted form of the given content, and give it a doc comment naming this call site as its reason to exist. Read how the `Entries` arm builds its pair nodes first and follow that construction rather than inventing a second one.

- [ ] **Step 5: Run the typeof cases and watch them pass**

```bash
cargo test -p kali_cli --test cases -- object/property_key_identity::object_keys_yields_strings
```

Expected: PASS, `string`.

- [ ] **Step 6: Re-measure and re-pin the four enumeration cases**

```bash
S=/tmp/claude-1000/-workspace/scratch-pin
for f in keys_module keys_extreme_module; do
  .cache/cargo-target/debug/kali run $S/$f.js; node $S/$f.js
done
```

Expected from both: `1e-7`, and `-Infinity` then `1e+21`. Update the four `object_keys_*` cases' `stdout` and rewrite each rationale:

> **RE-PINNED, was wrong now right** (Task 4, at `<commit>`): kali and node both print `1e-7`. The key text is `String(key)` as of Task 3, and the enumeration fold now yields it as a string rather than handing the key node to the expression renderer.

- [ ] **Step 7: Run the whole corpus**

```bash
cargo test -p kali_cli --test cases 2>&1 | tail -30
cargo test --workspace 2>&1 | tail -30
```

Expected: green except the two R-56 oracle cases, which Task 6 closes.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "fix(codegen): Object.keys yields strings, not renderings of the key node

The enumeration fold pushed the key NODE into its iteration items, so the
expression renderer read a numeric-looking property name as a number:
Object.keys({1e-7:1}) printed 0.0000001 and, worse, {[-1e999]:1} printed -inf,
a string no JS program can produce.

Printed output could not have caught the other half of this -- console.log(\"5\")
and console.log(5) are the same line -- so the new cases probe typeof directly.

Re-pinned, was wrong now right: four object_keys_* cases, now matching node at
1e-7, -Infinity and 1e+21."
```

---

### Task 5: Classify the fourteen `trim_matches('"')` sites

The design's §4.6 does **not** promise all fourteen fall. This task is a classification pass with a written outcome for each, not a sweep.

**Files:**
- Modify (as classification dictates): `crates/kali_optimize/src/object_fold.rs` (8 sites: `:156`, `:165`, `:174`, `:195`, `:748`, `:756`, `:762`, `:920`), `crates/kali_codegen/src/intrinsics/object.rs` (`:220`, `:230`), `crates/kali_codegen/src/lower.rs:4895`, `crates/kali_codegen/src/emit/call.rs:4893`, `crates/kali_mir/src/analysis/infer.rs:134`, `crates/kali_common/src/object.rs:9`
- Create: `docs/superpowers/followups/property-key-trim-site-classification.md`

**Interfaces:**
- Consumes: Task 3's invariant.
- Produces: a table naming, for each site, the currency of its input and the action taken.

- [ ] **Step 1: Enumerate the sites at HEAD rather than trusting this plan's line numbers**

```bash
grep -rn "trim_matches('\"')" --include="*.rs" crates/ | grep -v "_tests\|/tests"
```

Expected: 15 lines, of which `kali_fmt/src/formatter.rs` is **out of scope** (it trims source literal text in the formatter, not key text). The other 14 are this task's subjects.

- [ ] **Step 2: Classify each site by reading its callers**

For each site, answer in one sentence: *does this input arrive as key-slot text, as probe/rendered text, or as both?* `crates/kali_codegen/src/intrinsics/object.rs:559` already documents one deliberate asymmetry (`object_literal_field` is fed raw HIR text by member-access property names and inferred shape field names) — read it before classifying `:220` and `:230`.

- [ ] **Step 3: Write the classification document**

Create `docs/superpowers/followups/property-key-trim-site-classification.md` with a row per site:

```markdown
| file:line | input currency | action | why |
|---|---|---|---|
| `kali_optimize/src/object_fold.rs:156` | … | deleted / kept | … |
```

A **kept** row must say what non-key-slot text reaches it. A kept trim is not a defect; an unexamined one is.

- [ ] **Step 4: Delete the trims classified as key-slot-only**

For each such site, remove the `.trim_matches('"')` and run the owning crate's tests:

```bash
cargo test -p kali_optimize 2>&1 | tail -20
cargo test -p kali_mir 2>&1 | tail -20
cargo test -p kali_codegen 2>&1 | tail -20
cargo test -p kali_common 2>&1 | tail -20
```

- [ ] **Step 5: Rewrite `property_order_key`'s doc comment**

`crates/kali_common/src/object.rs:3-7` asserts the old cross-layer contract in terms: *"LIR literal text keeps source quoting, while AST/repr key text is unquoted; both layers must classify identically."* Replace it with what is true after Task 3, whichever way its own trim was classified — an ES-order classifier reading a stale contract is exactly the drift this project exists to end.

- [ ] **Step 6: Run the whole workspace**

```bash
cargo test --workspace 2>&1 | tail -40
```

Expected: green except the two R-56 oracle cases. Classify any re-pin.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(keys): classify the fourteen key-text trim sites, delete the dead ones

After lower_property_name stores the property name itself, a key-slot text
never carries quotes, so trimming one is either dead or harmful -- it conflates
the string key '\"5\"' with the numeric key 5, which is R-56's collision at a
second address.

Classified rather than swept: each site's input currency is recorded in
docs/superpowers/followups/property-key-trim-site-classification.md, with the
kept ones naming the non-key-slot text that reaches them."
```

---

### Task 6: Close R-56 in the ledger, and regenerate the ranking

**Files:**
- Modify: `crates/kali_cli/tests/cases/oracle/tier2.toml` (R-56's two cases)
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 row, §2 entry body, §1 severity table if the retirement precedent touches it)
- Modify: `tools/blast-radius/clusters.json`
- Modify: `docs/superpowers/followups/blast-radius-ranking.md` (generated region + a §6 amendment)

**Interfaces:**
- Consumes: Task 3's measured closure of R-56.
- Produces: a register and ranking that say what the oracle now measures.

- [ ] **Step 1: Read the retirement precedent before inventing one**

```bash
git show 3a636f62fb --stat
git show 5aebc5ec3d --stat
```

R-33 retired on this branch's parent. Whatever those commits did about `clusters.json`, the catalogue record, `§1`'s severity table and the numbering-note re-count series is what this task does for R-56. Do not devise a different convention.

- [ ] **Step 2: Re-measure R-56 in both scopes and record the transcript**

```bash
S=/tmp/claude-1000/-workspace/scratch-pin
.cache/cargo-target/debug/kali run $S/r56.js; echo "exit=$?"
node $S/r56.js; echo "node exit=$?"
git rev-parse --short HEAD
```

- [ ] **Step 3: Flip both oracle cases to `fixed`**

In `crates/kali_cli/tests/cases/oracle/tier2.toml`, change `verdict = "silent"` to `verdict = "fixed"` on `r56a_quoted_numeric_string_key_module_scope` and `..._in_function`, and append to each rationale:

> MEASURED `<date>` at `<commit>` against node v26.7.0: FIXED. kali prints `1`, `true`, `false` at exit 0; node prints the same three lines at exit 0. The entry's own prediction — that restoring the Number/String discriminator upstream is the only place it could be closed — is what happened; the discriminator was restored in `kali_hir`'s `lower_property_name` by storing `String(key)` rather than a quoted marker.

- [ ] **Step 4: Re-derive §0.2's row and record the closure in §2**

```bash
cargo test -p kali_blast_radius 2>&1 | tail -30
```

The gate compares §0.2's status column against the case verdicts and names the mismatch. Update R-56's §0.2 row to FIXED, then record the closure in R-56's §2 entry body against the commit that closed it, following R-33's wording from Step 1.

- [ ] **Step 5: Update `clusters.json` and regenerate the ranking**

```bash
cargo run -p kali_blast_radius --example rank > /tmp/claude-1000/-workspace/scratch-pin/rank.out
```

Splice the generator's stdout into `blast-radius-ranking.md`'s `GENERATED:BEGIN`/`END` region verbatim, and update the provenance table's HEAD cell.

- [ ] **Step 6: Verify the splice gate**

```bash
cargo test -p kali_blast_radius spliced_document_matches_the_generator
```

Expected: PASS. This test re-renders both regions and asserts they equal the committed text, so a hand-edited figure inside the markers turns it red.

- [ ] **Step 7: Write the §6 amendment from the regeneration, not from prediction**

Add an amendment to `blast-radius-ranking.md` §6 recording what the generator **printed**: the ranked-entry count, the cluster count, and whether any band moved. The spec predicted 28 → 27, 17 → 16 clusters and no band movement, since R-56 measured 0 reachable / 0 raw — report the actual figures and say so explicitly if they differ from that prediction. §6.6 item 4's instruction is *re-run, do not re-read*.

- [ ] **Step 8: Run the whole workspace**

```bash
cargo test --workspace 2>&1 | tail -40
```

Expected: fully green for the first time since Task 3.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "docs(register): R-56 closes, and the ranking regenerates

Both oracle cases re-measured at HEAD against node v26.7.0: FIXED. kali and
node both print 1/true/false. §0.2's row is re-derived from the cases, not
edited to agree with them.

R-56 leaves the SILENT filter, so the ranking regenerated: <figures read out of
the generator's stdout>. The §6 amendment records what it printed rather than
what the spec predicted."
```

---

### Task 7: Correct the document that sent us here

**Files:**
- Modify: `docs/superpowers/followups/console-render-unification-discovered-defects.md` (§2.5, §2.8, §3)

**Interfaces:**
- Consumes: the measured outcomes of Tasks 2, 3 and 4.
- Produces: a follow-up file whose remaining rows are still true.

- [ ] **Step 1: Re-measure every claim being changed, at HEAD**

```bash
S=/tmp/claude-1000/-workspace/scratch-pin
for f in bigint_module keys_module keys_extreme_module; do
  echo "--- $f"; .cache/cargo-target/debug/kali run $S/$f.js; node $S/$f.js
done
git rev-parse --short HEAD
```

- [ ] **Step 2: Rewrite §2.8**

Mark it **fixed at `<commit>`**, with the new transcript, and correct its attribution: the key was not merely destroyed by `lower_property_name`, it was replaced with `0` by `kali_parser/src/expression/object.rs`'s `unwrap_or(0.0)`, one crate above the function §2.8 named. Name the case ids that now pin it.

- [ ] **Step 3: Rewrite §2.5**

Mark it **fixed at `<commit>`**, and add the `-1e999` and `1e21` lanes this project measured — §2.5 recorded only the `1e-7` lane, and `-inf` is the worse of the three because no JavaScript program can produce that string. Name the case ids, including the `typeof` probe, and note why printed output alone could not have caught it.

- [ ] **Step 4: Rewrite §3**

§3 proposed this fix. Record that it happened, at which commit, and what it retired: `KeyTextSlot`, `is_hir_numeric_key_spelling`, its NaN guard, and the parser's fabricated-key fallback. Keep §3's warning about `canonical_property_key_text`'s call sites, updated to the function's new, probe-only meaning — the reason not to add a third caller has not expired.

- [ ] **Step 5: Check the rest of the file for rows this project falsified**

Read §2.1 through §2.13 and §4 with the change in mind. §2.6's note that R-56's SameValueZero rationale is theoretical, and §2.4's boolean-repr row, both mention key handling; correct anything that is now false, and leave anything still true alone rather than restating it.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "docs(followups): §2.5 and §2.8 close, and §3's upstream fix is recorded as done

Both re-measured at HEAD against node v26.7.0 rather than assumed from the
change that fixed them.

Two corrections to this file's own claims: §2.8's BigInt key was replaced with
the key 0, not merely lost, and the loss happens in kali_parser rather than the
HIR function §2.8 named; §2.5's defect was not confined to 1e-7 -- Object.keys
also yielded -inf, a string no JS program can produce."
```

---

## Self-Review

**Spec coverage.** §4.1 → Task 2 Step 3. §4.2 → Task 2 Steps 4-5. §4.3 → Task 3 Step 3. §4.4 → Task 3 Steps 5-6. §4.5 → Task 4. §4.6 → Task 5. §4.7's edge cases → Task 3 Step 1's table (`1e-7`, `1e+21`, `-0`, `-Infinity`) and Task 2 Step 1's large-BigInt test; `{NaN:1}` and `{[0/0]:1}` need no task because they are identifier and fail-closed paths respectively, unchanged by any step here. §5's non-goals are enforced by Task 5's classify-don't-unify rule and by nothing in any task touching `Repr`. §6.1-6.3 → Tasks 1, 3 Step 10, and the global re-pin constraint. §7 → Task 6. §8's risk 1 is the explicit stop condition in Task 5 Step 2; risk 2 is why Task 1 exists; risk 3 → Task 2 Step 6; risk 4 → Task 5 Step 5.

**Placeholders.** The `<commit>`, `<date>` and `<figures …>` markers in Tasks 2, 4, 6 and 7 are values that can only be produced by running the step that precedes them; each is preceded by the command that produces it. No step defers a decision.

**Type consistency.** `PropertyName::BigInt(String)` holds digits without the `n` suffix in Task 2 and is read as digits in Task 3's HIR arm. `canonical_property_key_text` loses its `slot` parameter in Task 3 Step 5 and is called with one argument in Task 3 Step 6's test. `static_property_key_text` is replaced by `static_object_key_text` and `static_probe_key_text` in the same step that updates its three call sites.
