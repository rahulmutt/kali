# Soundness Batch 1 PR-A Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the nine known miscompile/fail-open items from the Soundness Batch 1 spec (`docs/superpowers/specs/2026-07-10-soundness-batch-design.md`) — everything except optional chaining, which is PR-B.

**Architecture:** kali is a JS→WASM compiler: parser (AST) → HIR → MIR → LIR → codegen, with kali_types running resolve/repr passes **on the AST** and codegen keeping hand-mirrored twin predicates. Each task converts one silent wrong-output path into either correct JS semantics (verified against node) or a clean diagnostic, plus a pinned test.

**Tech Stack:** Rust workspace, `cargo test` per crate, integration tests drive the built `kali` binary via `CARGO_BIN_EXE_kali`.

## Global Constraints

- **Reject-don't-miscompile:** every closed path must either match node's observable behavior or fail with a diagnostic. Never trade one silent wrong output for another.
- **Twins in the same commit:** any change to what types admits must land with the matching codegen recognizer change in the same commit (and vice versa).
- **Suite gate:** `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli` must be green at every task's commit. Full run takes several minutes; per-task steps name narrower filters, the final task runs the whole gate.
- **CLBG goldens:** the six benchmark fixtures (nbody, fannkuch, spectral-norm, mandelbrot, binary-trees, fasta) are byte-for-byte pins inside the kali_cli suite — if any fails, your change broke it; do not touch the goldens.
- **Stale cache footgun:** run `rm -rf crates/kali_cli/tests/fixtures/.kali-cache` before believing any fixture result.
- **Fresh binary footgun:** re-run every reproducer through a freshly built binary (`cargo build -p kali_cli` first) before claiming a fix.
- **Work on branch `soundness-batch1-pra`** off `main`. One commit per task minimum.
- Formatting/lint: `cargo fmt --all` and `cargo clippy --workspace --all-targets` clean at the end (Task 11).
- Integration tests live in `crates/kali_cli/tests/`, one new file per task, each self-contained with its own `run_source` helper copied from the house pattern (see Task 1 Step 1 — the helper uses a `static AtomicU64` counter + pid in the temp dir slug; this is deliberate, a past flake fix; keep it).

---

### Task 1: `throw` lowers to print-then-trap

**Files:**
- Modify: `crates/kali_hir/src/lowering/statement.rs:102-106`
- Modify: `crates/kali_codegen/src/emit/control_flow.rs:951-967` (Branch dispatch)
- Create: `crates/kali_codegen/src/emit/throw.rs` (new: `emit_throw` + `throw_message_text`)
- Modify: `crates/kali_codegen/src/emit/mod.rs` (declare the new module — match how sibling emit modules are declared)
- Test: `crates/kali_cli/tests/soundness_throw.rs`

**Interfaces:**
- Produces: `emit_throw(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue` — prints a node-shaped message via `CONSOLE_ERROR_IMPORT_INDEX` when recoverable, then emits `Instruction::Unreachable`. Task 2 reuses the *pattern* (string print → `Unreachable`), not the function.
- Consumes: `crate::CONSOLE_ERROR_IMPORT_INDEX` (exists, see `intrinsics/host.rs:35`), `emit_literal` (exists, `control_flow.rs:948` shows the call shape `emit_literal(function, node.text.as_deref(), self.strings)`).

**Background you need:** Today `ThrowStmt` is allocated with no text (`alloc(HirNodeKind::ThrowStmt, None)`), flows HIR→MIR `ControlFlow` (`kali_mir/src/lower.rs:95`) →LIR `Branch` (`kali_lir/src/lower.rs:64`), and a `None`-text Branch falls into the generic `_ => self.emit_branch(...)` arm — a no-op. Execution silently continues past `throw`. MIR/LIR lowering copies node text generically; giving the HIR node text `"throw"` makes it dispatchable in codegen (Step 2 verifies this assumption before you build on it).

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_throw.rs`:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-throw-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.ts");
    std::fs::write(&path, src).unwrap();
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

// node: `throw new Error("boom")` prints "Uncaught Error: boom" on stderr and
// exits nonzero; statements after the throw must NOT run. Today kali silently
// falls through and prints "after" with exit 0 — the headline no-op miscompile.
#[test]
fn throw_error_literal_aborts_with_message() {
    let out = run_source("throw new Error(\"boom\");\nconsole.log(\"after\");\n");
    assert!(!out.status.success(), "throw must abort: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Uncaught Error: boom"), "stderr: {stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("after"), "must not execute past throw: {stdout}");
}

#[test]
fn throw_string_literal_aborts_with_message() {
    let out = run_source("throw \"plain\";\nconsole.log(\"after\");\n");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Uncaught plain"), "stderr: {stderr}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("after"));
}

// Non-literal argument: no message proof — generic message, still aborts.
#[test]
fn throw_computed_value_still_aborts() {
    let out = run_source("let x = 3;\nthrow x + 1;\nconsole.log(\"after\");\n");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Uncaught exception"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("after"));
}

// A throw on a never-taken path must not affect the taken path — the whole
// suite's defensive self-check throws depend on this staying compilable.
#[test]
fn unreached_throw_is_harmless() {
    let out = run_source(
        "function check(n) { if (n > 10) { throw new Error(\"nope\"); } return n; }\nconsole.log(check(5));\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}
```

- [ ] **Step 2: Run tests to verify they fail for the right reason**

Run: `cargo test -p kali_cli --test soundness_throw`
Expected: the first three FAIL (exit 0 / "after" printed — the no-op); `unreached_throw_is_harmless` PASSES already. If `unreached_throw_is_harmless` fails, stop — your baseline is broken; investigate before proceeding.

- [ ] **Step 3: Give ThrowStmt its dispatch text in HIR**

In `crates/kali_hir/src/lowering/statement.rs`, replace:

```rust
Statement::ThrowStatement(ThrowStatement { argument }) => {
    let id = self.builder.alloc(HirNodeKind::ThrowStmt, None);
    push_child!(self, id, self.lower_expression(argument));
    id
}
```

with:

```rust
Statement::ThrowStatement(ThrowStatement { argument }) => {
    // Text "throw" survives MIR ControlFlow → LIR Branch so codegen's
    // text-keyed Branch dispatch can lower it (a None-text Branch falls
    // into the generic arm, which is how throw was a silent no-op).
    let id = self
        .builder
        .alloc_text(HirNodeKind::ThrowStmt, None, "throw".to_string());
    push_child!(self, id, self.lower_expression(argument));
    id
}
```

(Match `alloc_text`'s real signature — see the `UnaryExpr` arm at `lowering/expression.rs:31-36` for the exact call shape.)

- [ ] **Step 4: Add the codegen throw arm**

In `crates/kali_codegen/src/emit/control_flow.rs`, in the `LirNodeKind::Branch` match (line ~951), add an arm before the catch-all:

```rust
Some("throw") => self.emit_throw(function, &node),
```

Create `crates/kali_codegen/src/emit/throw.rs`:

```rust
//! `throw` lowering: print a node-shaped message when the argument is
//! provable, then trap. kali has no exception machinery (try/catch stays
//! rejected), and an uncaught JS throw's observable behavior is exactly
//! "message on stderr, abort nonzero" — which is what unreachable + the
//! CLI's E4000 trap envelope produce.

use crate::*;

impl<'a> FunctionEmitter<'a> {
    /// Node-shaped message for a throw argument, when statically provable:
    /// `throw new Error("m")` → `Uncaught Error: m`; `throw "m"` → `Uncaught m`;
    /// anything else → None (caller prints the generic message).
    fn throw_message_text(&self, arg: LirNodeId) -> Option<String> {
        let arg = self.unwrap_transparent(arg);
        let node = self.node(arg);
        // `throw "m"`: string literal.
        if node.kind == LirNodeKind::Literal {
            if let Some(text) = node.text.as_deref() {
                let unquoted = text.trim_matches(|c| c == '"' || c == '\'');
                if unquoted.len() != text.len() {
                    return Some(format!("Uncaught {unquoted}"));
                }
            }
            return None;
        }
        // `throw new Error("m")`: a call-shaped node whose callee text is
        // "Error" and whose sole argument is a string literal. Inspect the
        // LIR shape with a debugger/--emit-lir dump if the child layout
        // differs; the recognizer must stay fail-closed (None on any
        // unexpected shape).
        if node.text.as_deref() == Some("Error") || {
            node.children
                .first()
                .map(|&c| self.node(self.unwrap_transparent(c)).text.as_deref() == Some("Error"))
                .unwrap_or(false)
        } {
            for &child in &node.children {
                let child_node = self.node(self.unwrap_transparent(child));
                if child_node.kind == LirNodeKind::Literal {
                    if let Some(text) = child_node.text.as_deref() {
                        let unquoted = text.trim_matches(|c| c == '"' || c == '\'');
                        if unquoted.len() != text.len() {
                            return Some(format!("Uncaught Error: {unquoted}"));
                        }
                    }
                }
            }
        }
        None
    }

    pub(crate) fn emit_throw(
        &mut self,
        function: &mut Function,
        node: &LirNode,
    ) -> EmittedValue {
        let message = node
            .children
            .first()
            .and_then(|&arg| self.throw_message_text(arg))
            .unwrap_or_else(|| "Uncaught exception".to_string());
        // Print via console.error's host import: emit the message as a
        // string literal (same lane as `console.error("...")` — a quoted
        // Literal through emit_literal yields the handle console_error
        // consumes), then call the import.
        emit_literal(function, Some(&format!("\"{message}\"")), self.strings);
        function.instruction(&Instruction::Call(crate::CONSOLE_ERROR_IMPORT_INDEX));
        function.instruction(&Instruction::Unreachable);
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }
}
```

**Implementation notes (read before coding):** (a) the `use crate::*;` + `impl<'a> FunctionEmitter<'a>` header matches the sibling emit modules (verified against `emit/literal.rs`); wire the `mod throw;` declaration wherever the siblings are declared; (b) if `console.error("lit")` lowers through `emit_call` with extra arity/handshake instructions (check `emit_call`'s console lane in `emit/call.rs` around the `console_import_index` use at line ~62), replicate THAT exact sequence — the test in Step 5 is the arbiter; (c) verify the `new Error("m")` LIR shape empirically with one of the Step 1 reproducers rather than trusting the sketch — the recognizer returning None (generic message) is acceptable for round 1 on the `Error` form ONLY if the string-literal form works; tighten in the same task.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_throw`
Expected: all 4 PASS.

- [ ] **Step 6: Check no regression in the throw-adjacent suite**

Run: `cargo test -p kali_codegen && cargo test -p kali_cli --test runtime_smoke`
Expected: green. (Suite fixtures contain defensive self-check throws on never-taken paths; they must all still compile and pass.)

- [ ] **Step 7: Commit**

```bash
git add crates/kali_hir/src/lowering/statement.rs crates/kali_codegen/src/emit/ crates/kali_cli/tests/soundness_throw.rs
git commit -m "fix(codegen): throw lowers to print-then-trap, not a silent no-op (soundness batch 1 item 1)"
```

---

### Task 2: `1n/0n` traps with a RangeError message

**Files:**
- Modify: `crates/kali_codegen/src/emit/operators.rs:1376-1390` (`/` lowering)
- Test: `crates/kali_cli/tests/soundness_bigint_div_zero.rs`

**Interfaces:**
- Consumes: the Task 1 pattern — quoted-literal `emit_literal` + `Call(CONSOLE_ERROR_IMPORT_INDEX)` + `Unreachable`. Copy the three-instruction sequence; do not try to share `emit_throw` (its message logic is throw-specific).

- [ ] **Step 1: Write the failing test**

Create `crates/kali_cli/tests/soundness_bigint_div_zero.rs` with the same `run_source` helper as Task 1 Step 1 (slug `kali-soundness-bigintdiv`), plus:

```rust
// node: `1n/0n` throws "RangeError: Division by zero". kali already traps
// (correct abort) but with the generic unreachable envelope; pin the
// node-shaped message. Also pin that a nonzero literal divide still works.
#[test]
fn bigint_division_by_zero_traps_with_range_error() {
    let out = run_source("console.log(7n / 0n);\n");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("RangeError: Division by zero"));
}

#[test]
fn bigint_division_nonzero_still_truncates() {
    let out = run_source("console.log(7n / 2n);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3n");
}
```

(If the second test's expected output format differs — run `node -e 'console.log(7n / 2n)'` prints `3n` — defer to what the EXISTING green bigint tests pin for kali's format; do not change output formatting in this task.)

- [ ] **Step 2: Run to verify the message test fails**

Run: `cargo test -p kali_cli --test soundness_bigint_div_zero`
Expected: first FAILS (trap message is the generic "runtime trap (unreachable …)" envelope), second PASSES.

- [ ] **Step 3: Implement the zero-check**

In `crates/kali_codegen/src/emit/operators.rs`, the non-float `/` arm currently reads:

```rust
// BigInt `/`: truncation toward zero is exactly `i64.div_s`.
function.instruction(&Instruction::I64DivS);
```

Replace with a divisor zero-check (divisor is on top of stack; use a scratch local via the same local-allocation idiom the `??=` arm uses at `literal.rs:572-575`):

```rust
// BigInt `/`: truncation toward zero is exactly `i64.div_s`. A zero
// divisor traps in wasm anyway; test it explicitly first so the abort
// carries node's message (RangeError) instead of the generic
// unreachable envelope.
let divisor_local = self.locals.len() as u32;
function.instruction(&Instruction::LocalSet(divisor_local));
function.instruction(&Instruction::LocalGet(divisor_local));
function.instruction(&Instruction::I64Eqz);
function.instruction(&Instruction::If(BlockType::Empty));
emit_literal(function, Some("\"RangeError: Division by zero\""), self.strings);
function.instruction(&Instruction::Call(crate::CONSOLE_ERROR_IMPORT_INDEX));
function.instruction(&Instruction::Unreachable);
function.instruction(&Instruction::End);
function.instruction(&Instruction::LocalGet(divisor_local));
function.instruction(&Instruction::I64DivS);
```

**Note:** `self.locals.len() as u32` as a scratch slot is the house idiom (see `??=` at `literal.rs:573`) but verify the function's local declaration section actually reserves scratch slots — find how that existing arm's `temp_local` gets declared (search for where locals are counted when the wasm function header is written) and follow the same mechanism. If scratch locals need explicit registration, register yours the same way. The Step 4 test catches a mismatch (wasm validation error).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_bigint_div_zero`
Expected: both PASS.

- [ ] **Step 5: Regression check bigint lanes**

Run: `cargo test -p kali_codegen bigint; cargo test -p kali_cli bigint`
Expected: green.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_codegen/src/emit/operators.rs crates/kali_cli/tests/soundness_bigint_div_zero.rs
git commit -m "fix(codegen): BigInt division-by-zero aborts with node-shaped RangeError message (soundness batch 1 item 8)"
```

---

### Task 3: Keyword object-literal keys parse; unknown property forms reject

**Files:**
- Modify: `crates/kali_parser/src/expression/object.rs:23-73`
- Test: `crates/kali_parser/src/expression/object_tests.rs` if it exists, else `crates/kali_cli/tests/soundness_object_keys.rs` (use the latter; end-to-end proves the whole lane)

**Interfaces:**
- Consumes: `Parser::is_property_name_token` (`expression/call.rs:139`) and `Parser::push_feature_unavailable` (`parser.rs:37`).

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_object_keys.rs` (same `run_source` helper, slug `kali-soundness-objkeys`):

```rust
// `{type: 3}` then `obj.type`: the object-literal parser only accepted
// Identifier/String/Numeric/computed keys; a keyword key hit the `_ =>`
// arm which silently DISCARDED the whole property, so the read yielded 0.
// Member access `obj.type` was fixed earlier (is_property_name_token);
// this closes the literal side with the same key set.
#[test]
fn keyword_key_in_object_literal_round_trips() {
    let out = run_source("const o = { type: 3, if: 4 };\nconsole.log(o.type + o.if);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "7");
}

// The old `_ =>` arm silently dropped ANY unrecognized property form.
// Fail-closed now: unknown forms are a hard reject, not a silent drop.
#[test]
fn unrecognized_property_form_rejects() {
    // `...spread` in an object literal is unsupported surface.
    let out = run_source("const a = { x: 1 };\nconst o = { ...a };\nconsole.log(o.x);\n");
    assert!(!out.status.success(), "spread property must reject, got: {out:?}");
}
```

- [ ] **Step 2: Run to verify both fail**

Run: `cargo test -p kali_cli --test soundness_object_keys`
Expected: first FAILS printing `0` (or NaN-ish) instead of `7`; second FAILS because the spread is silently dropped and `o.x` "works" (prints 0/undefined-ish with exit 0). If the second already rejects, keep it as a pin and note it in the commit message.

- [ ] **Step 3: Implement**

In `crates/kali_parser/src/expression/object.rs`, replace the `Some(TokenType::Identifier) =>` arm's guard and the `_ =>` arm:

```rust
// was: Some(TokenType::Identifier) => { ... }
Some(kind) if Self::is_property_name_token(&kind) => {
    // Keyword property keys (`type`, `if`, …) are plain names in JS
    // object literals — same key set as `.name` member access
    // (is_property_name_token), so literal writes and member reads
    // can never disagree again.
    let name = self
        .stream
        .advance()
        .map(|token| token.value)
        .unwrap_or_default();

    if self.stream.accept(TokenType::Colon) {
        (PropertyName::Identifier(name), self.parse_expression())
    } else {
        let expr = Expression::Identifier(name.clone());
        (PropertyName::Identifier(name), expr)
    }
}
```

(The shorthand `{ type }` branch: `type` as a VALUE identifier only makes sense for tokens that can be expression identifiers; keyword shorthand like `{ if }` produces `Expression::Identifier("if")` which will fail resolve with E3100 — acceptable fail-closed behavior, do not special-case.)

And the catch-all:

```rust
_ => {
    // Fail closed: the old arm advanced-and-continued, silently
    // DISCARDING the whole property (keyword keys, spreads, methods —
    // anything unrecognized). A property the parser cannot represent
    // must reject, never vanish.
    self.push_feature_unavailable(
        "this object-literal property form is unavailable in the current phase; use `key: value` with an identifier, string, or numeric key",
    );
    let _ = self.stream.advance();
    continue;
}
```

(`push_feature_unavailable` records an error-severity diagnostic — the compile fails; the `advance(); continue;` afterward is only parser error recovery so later diagnostics still surface.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_object_keys`
Expected: both PASS.

- [ ] **Step 5: Regression check parser + object lanes**

Run: `cargo test -p kali_parser && cargo test -p kali_cli object`
Expected: green. **Watch for:** any existing test that FED an unsupported property form and relied on the silent drop — if one turns red, inspect it: if it pinned wrong-output behavior, update the pin to expect the reject and say so in the commit message; if it's a legitimate form, your allowlist is too narrow.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_parser/src/expression/object.rs crates/kali_cli/tests/soundness_object_keys.rs
git commit -m "fix(parser): keyword object-literal keys parse via is_property_name_token; unknown property forms reject instead of silently dropping (soundness batch 1 item 2)"
```

---

### Task 4: Reserved words reject as binding names

**Files:**
- Modify: `crates/kali_parser/src/statement.rs:114-116` (`parse_variable_declarator`)
- Modify: `crates/kali_parser/src/expression/call.rs` (add `is_binding_name_token` next to `is_property_name_token`)
- Test: `crates/kali_cli/tests/soundness_reserved_bindings.rs`

**Interfaces:**
- Produces: `Parser::is_binding_name_token(kind: &TokenType) -> bool` — the small allowlist of tokens legal as binding names. PR-B does not consume it; nothing else in PR-A does either.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_reserved_bindings.rs` (helper slug `kali-soundness-reserved`):

```rust
// `parse_variable_declarator` took ANY next token's value as the binding
// name, so `const if = 1` bound a variable literally named "if".
#[test]
fn reserved_word_binding_rejects() {
    for src in ["const if = 1;\n", "let for = 2;\n", "var function = 3;\n"] {
        let out = run_source(src);
        assert!(!out.status.success(), "{src:?} must reject, got: {out:?}");
    }
}

// Contextual keywords that are legal JS binding names must keep working —
// the lexer keywordizes them, but `const type = 1` is valid JS.
#[test]
fn contextual_keyword_bindings_still_work() {
    let out = run_source("const type = 1;\nconst of = 2;\nconst from = 3;\nconsole.log(type + of + from);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "6");
}
```

- [ ] **Step 2: Run to verify the reject test fails**

Run: `cargo test -p kali_cli --test soundness_reserved_bindings`
Expected: `reserved_word_binding_rejects` FAILS (compiles today). `contextual_keyword_bindings_still_work` should PASS already; if it fails today, note which token breaks and make sure it's in the Step 3 allowlist.

- [ ] **Step 3: Implement**

In `crates/kali_parser/src/expression/call.rs`, next to `is_property_name_token`:

```rust
/// Tokens legal as a BINDING name. Deliberately a small default-deny
/// allowlist (NOT is_property_name_token minus a denylist): property
/// names admit every keyword, binding names admit only identifiers and
/// the contextual keywords that are legal JS binding identifiers.
pub(crate) fn is_binding_name_token(kind: &TokenType) -> bool {
    matches!(
        kind,
        TokenType::Identifier
            | TokenType::Type
            | TokenType::Interface
            | TokenType::Enum
            | TokenType::From
            | TokenType::As
            | TokenType::Of
            | TokenType::Async
    )
}
```

In `crates/kali_parser/src/statement.rs`, `parse_variable_declarator`:

```rust
fn parse_variable_declarator(&mut self) -> Option<VariableDeclarator> {
    if !self
        .stream
        .current_kind()
        .is_some_and(Self::is_binding_name_token)
    {
        self.push_feature_unavailable(
            "a reserved word cannot be used as a binding name",
        );
        let _ = self.stream.advance();
        return None;
    }
    let name_token = self.stream.advance()?;
    let name = name_token.value;
    // ... rest unchanged (block-arrow init special case, init parse) ...
}
```

Check the caller (`parse_variable_declaration`) handles a `None` declarator without infinite-looping on the unconsumed tokens — the `advance()` above consumes the offending token; if the caller's loop still spins (e.g. on the `=` that follows), extend the recovery to skip to the next `Semicolon`/`Comma` with a bounded scan, mirroring how other parse errors in that loop recover.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_reserved_bindings`
Expected: both PASS.

- [ ] **Step 5: Regression check**

Run: `cargo test -p kali_parser && cargo test -p kali_lexer`
Expected: green. If a fixture legitimately binds a keyword-ish name your allowlist missed (e.g. `let async = …`), add that token to `is_binding_name_token` — allowlist growth is the intended maintenance mode.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_parser/src/statement.rs crates/kali_parser/src/expression/call.rs crates/kali_cli/tests/soundness_reserved_bindings.rs
git commit -m "fix(parser): reserved words reject as binding names via is_binding_name_token allowlist (soundness batch 1 item 7)"
```

---

### Task 5: Unified nullish recognizer — `= undefined` stores the −1 sentinel; `??= undefined` admit

**Files:**
- Modify: `crates/kali_codegen/src/emit/literal.rs:163-167` (recognizer) and its four use sites (`literal.rs:500-505`, `literal.rs:595-598`, `crates/kali_codegen/src/emit/control_flow.rs:855-864`)
- Modify: `crates/kali_types/src/resolve/expression.rs:1779` region (the `??=` null-literal-RHS narrow admit)
- Test: `crates/kali_cli/tests/soundness_undefined_sentinel.rs`
- Modify: `crates/kali_cli/tests/nullish_assign_reject.rs` (flip the `??= undefined` reject pin to a behavior pin)

**Interfaces:**
- Produces: `is_null_or_undefined_expr(&self, id: LirNodeId) -> bool` on the codegen emitter (replaces `is_null_or_undefined_literal` at all four sites; keep the old fn deleted, not deprecated — grep proves no other callers).
- Types twin: whatever predicate the `??=` admit at `resolve/expression.rs:1779` uses to recognize a `null` literal RHS must widen to accept identifier-`undefined` in the same commit — the Spec 7 reject existed *only* because the two sides disagreed on the identifier form.

**Background:** `undefined` parses as `Expression::Identifier("undefined")` (`primary.rs:57`), lowering to a `Value` node — `is_null_or_undefined_literal` requires `LirNodeKind::Literal`, so `last = undefined` misses the −1 sentinel stores and falls into the generic emit, which maps `"undefined"` to `I64Const(0)` (`control_flow.rs:1163`) — 0 is a VALID key ordinal, so the alias's truthiness is wrong. The code comment at `literal.rs:591` already documents the gap.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_undefined_sentinel.rs` (helper slug `kali-soundness-undef`). The for-in fixture shape below mirrors the existing for-in-key alias tests (see `crates/kali_cli/tests/` files matching `forin`; keep the object shape within the admitted fixed-shape surface):

```rust
// `last = undefined` on a for-in key alias must store the -1 null sentinel
// (matching `last = null`), NOT 0 — ordinal 0 is the FIRST KEY, so a 0
// store flips `if (last)` from false to true. node prints "none".
#[test]
fn forin_alias_undefined_reassign_reads_falsy() {
    let out = run_source(
        "const o = { a: 1, b: 2 };\nlet last = null;\nfor (const k in o) { last = k; }\nlast = undefined;\nif (last) { console.log(\"some\"); } else { console.log(\"none\"); }\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "none");
}

// Declarator form: `let last = undefined` (identifier, not null literal).
#[test]
fn forin_alias_undefined_init_reads_falsy() {
    let out = run_source(
        "const o = { a: 1 };\nlet last = undefined;\nfor (const k in o) { if (false) { last = k; } }\nif (last) { console.log(\"some\"); } else { console.log(\"none\"); }\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "none");
}

// With the recognizer twins unified, `??= undefined` no longer needs its
// disagreement reject: it stores the -1 sentinel exactly like `??= null`.
// node: last is "b" (non-nullish), so ??= does not fire; prints b.
#[test]
fn forin_alias_nullish_assign_undefined_rhs_admits() {
    let out = run_source(
        "const o = { a: 1, b: 2 };\nlet last = null;\nfor (const k in o) { last = k; }\nlast ??= undefined;\nconsole.log(last);\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "b");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p kali_cli --test soundness_undefined_sentinel`
Expected: first two FAIL printing `some` (the 0-store bug); third FAILS with the E5506 reject from Spec 7. Also verify the reproducers against node: `node -e '<same source>'` prints `none`/`none`/`b`.

- [ ] **Step 3: Implement the unified recognizer**

In `crates/kali_codegen/src/emit/literal.rs`, replace `is_null_or_undefined_literal`:

```rust
/// True when `id` resolves (through transparent wrappers) to a nullish
/// expression: the `null`/`undefined` LITERAL forms, or the bare
/// identifier `undefined` (which parses as an Identifier → Value node —
/// the form the old literal-only recognizer missed, storing ordinal 0
/// instead of the -1 sentinel: wrong truthiness). Single recognizer for
/// BOTH the types-side admit and the codegen stores, so the twins cannot
/// disagree on nullish-ness again (the `??= undefined` reject existed
/// only because of that disagreement).
pub(crate) fn is_null_or_undefined_expr(&self, id: LirNodeId) -> bool {
    let node = self.node(self.unwrap_transparent(id));
    match node.kind {
        LirNodeKind::Literal => {
            matches!(node.text.as_deref(), Some("null") | Some("undefined"))
        }
        LirNodeKind::Value => {
            node.children.is_empty() && node.text.as_deref() == Some("undefined")
        }
        _ => false,
    }
}
```

Then `grep -rn "is_null_or_undefined_literal" crates/` and replace every call site (the four listed in **Files**) with `is_null_or_undefined_expr`; delete the old function. Update the stale comment at `literal.rs:586-594` (the "bare `undefined` … rejects in resolve" sentence is no longer true — the RHS is now admitted and handled by the sentinel arm).

**Verify the `Value` shape empirically:** the bare-identifier `undefined` node may carry a child or empty text depending on lowering (compare `bare_identifier_name` at its definition — grep it — which is the house idiom for "bare identifier"). If `bare_identifier_name(id) == Some("undefined")` is cleaner, use that instead of matching kind/children by hand; the Step 5 tests arbitrate.

- [ ] **Step 4: Widen the types-side `??=` admit**

At `crates/kali_types/src/resolve/expression.rs:1779` the admit currently requires a `null` LITERAL RHS (message: "supports only a `null` literal right-hand side"). Read the surrounding predicate; widen it to also accept `Expression::Identifier(name) if name == "undefined"`, and update the message:

```
"nullish assignment on binding '{}' is unavailable: a for-in-key alias `??=` supports only a nullish right-hand side (`null` or `undefined`; any other value has no ordinal representation)"
```

This is the twin move of Step 3 — same commit, per the global constraint.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_undefined_sentinel`
Expected: all 3 PASS.

- [ ] **Step 6: Flip the stale reject pin**

Run: `cargo test -p kali_cli --test nullish_assign_reject`
Expected: exactly one failure — the Spec 7 pin asserting `??= undefined` REJECTS (added in `61dfb75c9`). Rewrite that test to pin the new behavior (compiles; sentinel semantics; use the same source it already has but assert success + node-matching output). Keep every OTHER pin in that file untouched — scalar `??=` still rejects.

Run again: `cargo test -p kali_cli --test nullish_assign_reject`
Expected: green.

- [ ] **Step 7: Regression check the for-in lane**

Run: `cargo test -p kali_cli forin; cargo test -p kali_codegen; cargo test -p kali_types`
Expected: green.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen/src/emit/ crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/soundness_undefined_sentinel.rs crates/kali_cli/tests/nullish_assign_reject.rs
git commit -m "fix(codegen,types): unified nullish recognizer — identifier undefined stores the -1 sentinel; ??= undefined admitted (soundness batch 1 item 4)"
```

---

### Task 6: Multi-arg `Array(1,2,3)` desugars to an array literal

**Files:**
- Modify: `crates/kali_parser/src/expression/call.rs:18-32` (call building) — desugar site
- Modify: the `new`-expression parse site (grep `NewExpression` in `crates/kali_parser/src` for where `new Array(...)` is built) — same desugar
- Modify: `crates/kali_types/src/resolve/expression.rs:604-630` (`declarator_registers_runtime_array` — narrow to arity ≤ 1)
- Test: `crates/kali_cli/tests/soundness_multiarg_array.rs`

**Interfaces:**
- Consumes: `Expression::ArrayExpression(ArrayExpression { elements })` with `ExpressionOrSpread::Expression` elements (see `primary.rs:101-127` for the construction shape).

**Background:** types (AST-side) registers ANY `Array(...)` call as a runtime array; codegen's `resolve_array_alloc_call` (`emit/call.rs:2331`) bails at >1 argument → the binding is "an array" that was never allocated → scalar 0. JS semantics: `Array(a, b, c)` with n ≥ 2 args IS `[a, b, c]`. Desugaring **in the parser** lands before types (which runs on the AST) and before codegen — both twins see a plain array literal and every existing array-literal gate applies fail-closed by construction.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_multiarg_array.rs` (helper slug `kali-soundness-multiarr`):

```rust
// `Array(1,2,3)` is exactly `[1,2,3]` in JS (n>=2 args). Today types
// registers the binding as an array but codegen can't allocate it → 0.
#[test]
fn multiarg_array_call_is_array_literal() {
    let out = run_source("const a = Array(1, 2, 3);\nconsole.log(a.length);\nconsole.log(a[1]);\n");
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim().split('\n').collect::<Vec<_>>(), vec!["3", "2"]);
}

#[test]
fn multiarg_new_array_call_is_array_literal() {
    let out = run_source("const a = new Array(4, 5);\nconsole.log(a.length + a[0]);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "6");
}

// Single-arg `new Array(n)` is a LENGTH, not an element — must stay on the
// existing allocation lane, NOT desugar.
#[test]
fn single_arg_array_still_allocates_by_length() {
    let out = run_source("const a = new Array(5);\nconsole.log(a.length);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}
```

- [ ] **Step 2: Run to verify the first two fail**

Run: `cargo test -p kali_cli --test soundness_multiarg_array`
Expected: first two FAIL (0/garbage output); third PASSES (existing lane).

- [ ] **Step 3: Implement the parser desugar**

In `crates/kali_parser/src/expression/call.rs`, in `parse_call_expression`'s `LeftParen` arm, after `args` is collected and before the `CallExpression` is built:

```rust
// `Array(e1, …, en)` with n >= 2 IS the array literal `[e1, …, en]`
// (JS semantics; single-arg Array(n) is a length). Desugar at parse
// time so BOTH twins (types on the AST, codegen downstream) see a
// plain ArrayExpression and every array-literal gate applies
// fail-closed by construction — no new recognizer surface.
if args.len() >= 2 {
    if let Expression::Identifier(name) = &expr {
        if name == "Array" {
            expr = Expression::ArrayExpression(ArrayExpression {
                elements: args
                    .drain(..)
                    .map(|arg| Some(ExpressionOrSpread::Expression(arg)))
                    .collect(),
            });
            continue;
        }
    }
}
expr = Expression::CallExpression(Box::new(CallExpression { callee: expr, args }));
```

(Add the `ArrayExpression`/`ExpressionOrSpread` imports to the file's `use kali_ast::{...}` list.) Then find the `new`-expression parse site (`grep -n "NewExpression" crates/kali_parser/src`): since `new Array(1,2)`'s callee is parsed through the call path, check what shape it takes AFTER the desugar — if the NewExpression wraps what is now an `ArrayExpression`, unwrap it at the new-expression build site:

```rust
// `new Array(a, b, …)` desugars identically to `Array(a, b, …)` —
// the call-path desugar already turned the callee into the array
// literal; `new` adds nothing for the Array constructor.
if matches!(&callee, Expression::ArrayExpression(_)) {
    return callee;
}
```

- [ ] **Step 4: Narrow the types twin**

In `crates/kali_types/src/resolve/expression.rs`, `declarator_registers_runtime_array`, the bare call arm currently accepts any arity:

```rust
Expression::CallExpression(call) => {
    if matches!(&call.callee, Expression::Identifier(name) if name == "Array") {
        return true;
    }
```

Narrow it (defense in depth — the parser desugar means n ≥ 2 never reaches here, but the twins must agree on what an `Array(...)` CALL is):

```rust
Expression::CallExpression(call) => {
    // Arity <= 1 only: `Array(n)` is a length allocation. n >= 2 is
    // desugared to an ArrayExpression at parse time and must never
    // register through THIS arm (codegen's resolve_array_alloc_call
    // twin also bails at >1 arg — keep the pair in lockstep).
    if matches!(&call.callee, Expression::Identifier(name) if name == "Array")
        && call.args.len() <= 1
    {
        return true;
    }
```

Apply the same arity guard to the `NewExpression` arm above it if that arm can see a multi-arg inner call (`new_expr.callee` as `CallExpression` — same `call.args.len() <= 1` check).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_multiarg_array`
Expected: all 3 PASS.

- [ ] **Step 6: Regression check array lanes**

Run: `cargo test -p kali_cli array; cargo test -p kali_types; cargo test -p kali_parser`
Expected: green.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_parser/src/expression/call.rs crates/kali_parser/src -A
git add crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/soundness_multiarg_array.rs
git commit -m "fix(parser,types): Array(a,b,...) desugars to an array literal at parse; types arity twin narrowed to lockstep (soundness batch 1 item 5)"
```

---

### Task 7: Mixed BigInt arithmetic rejects with E3202

**Files:**
- Modify: `crates/kali_error/src/_error_codes.rs` (e3 module — add the constant)
- Modify: `crates/kali_types/src/resolve/expression.rs` (binary-expression resolve — add the gate; find the arm that resolves `Expression::BinaryExpression`)
- Test: `crates/kali_cli/tests/soundness_mixed_bigint.rs`

**Interfaces:**
- Produces: `e3::MIXED_BIGINT_ARITHMETIC: u16 = 3202`.

**Background:** codegen's `/` lowering floats unless BOTH operands are BigInt literals (`operators.rs:1319`), and `+`/`-`/`*` have the same hole — `3n * 2` silently floats. Node throws `TypeError: Cannot mix BigInt and other types`. The gate is types-side (compile-time reject), syntactic-literal in scope: an operand is "BigInt-valued" here iff it is a `BigIntLiteral` (optionally under unary minus or parenthesis) or a const binding initialized to one — mirror the shape of codegen's `is_bigint_literal_valued` (`operators.rs:904-920`) but on the AST. The repr machinery has no BigInt axis; a broader proof is out of scope (the unmixed non-literal path already has its own recorded follow-up).

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_mixed_bigint.rs` (helper slug `kali-soundness-mixbig`):

```rust
// node: `3n / 2` throws TypeError (cannot mix BigInt and Number). kali
// silently floated it (F64Div → prints 1.5). Reject at compile time.
#[test]
fn mixed_bigint_arithmetic_rejects() {
    for src in [
        "console.log(3n / 2);\n",
        "console.log(3n * 2);\n",
        "console.log(3n + 2);\n",
        "console.log(2 - 3n);\n",
    ] {
        let out = run_source(src);
        assert!(!out.status.success(), "{src:?} must reject, got: {out:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("E3202"),
            "expected E3202 for {src:?}: {out:?}"
        );
    }
}

// All-BigInt stays green (existing lane).
#[test]
fn all_bigint_arithmetic_still_works() {
    let out = run_source("console.log(6n / 2n);\nconsole.log(2n * 3n);\n");
    assert!(out.status.success(), "{out:?}");
}
```

- [ ] **Step 2: Run to verify the reject test fails**

Run: `cargo test -p kali_cli --test soundness_mixed_bigint`
Expected: `mixed_bigint_arithmetic_rejects` FAILS (everything compiles and floats today); second PASSES.

- [ ] **Step 3: Add the E-code**

In `crates/kali_error/src/_error_codes.rs`, e3 module, after `MISSING_PARAMETER_TYPE`:

```rust
    // E3200-3299: Type errors (basic)
    pub const TYPE_MISMATCH: u16 = 3200;
    pub const MISSING_PARAMETER_TYPE: u16 = 3201;
    /// Binary arithmetic mixing a BigInt operand with a non-BigInt operand —
    /// a TypeError in JS, rejected at compile time (soundness batch 1).
    pub const MIXED_BIGINT_ARITHMETIC: u16 = 3202;
```

- [ ] **Step 4: Add the types-side gate**

Find where `Expression::BinaryExpression` is resolved in `crates/kali_types/src/resolve/expression.rs` (grep `BinaryExpression`). Add, for operators in `["+", "-", "*", "/", "%", "**"]`:

```rust
// Mixed BigInt arithmetic is a JS TypeError — reject at compile time
// (E3202) instead of silently floating one operand (`3n / 2` printed
// 1.5; node throws). Literal-scope recognizer mirroring codegen's
// is_bigint_literal_valued: BigIntLiteral, optionally under unary
// minus / parens / a const binding to one.
let left_bigint = self.is_bigint_literal_expr(&expr.left);
let right_bigint = self.is_bigint_literal_expr(&expr.right);
if left_bigint != right_bigint {
    self.diagnostics.push(Diagnostic::error(
        e3::MIXED_BIGINT_ARITHMETIC as u32,
        format!(
            "cannot mix BigInt and non-BigInt operands in '{}' — convert one side explicitly",
            expr.operator
        ),
    ));
}
```

with the helper (same file):

```rust
fn is_bigint_literal_expr(&self, expr: &Expression) -> bool {
    match expr {
        Expression::BigIntLiteral(_) => true,
        Expression::UnaryExpression(unary) if unary.operator == "-" => {
            self.is_bigint_literal_expr(&unary.argument)
        }
        Expression::ParenthesizedExpression(paren) => {
            self.is_bigint_literal_expr(&paren.expression)
        }
        Expression::Identifier(name) => self
            .const_binding_init(name)
            .is_some_and(|init| self.is_bigint_literal_expr(init)),
        _ => false,
    }
}
```

**Adapt to the file's real idiom:** the diagnostics-push mechanism and const-binding lookup (`const_binding_init` is a sketch — find the existing helper the file uses to chase const inits; grep `const` around `resolve/expression.rs:600` and in the `??=` gates at 1699-1785 for the established pattern). If no const-chasing helper exists, drop the `Identifier` arm — literal-only scope still closes the reported miscompile; note the narrowing in the commit message.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_mixed_bigint`
Expected: both PASS.

- [ ] **Step 6: Regression check**

Run: `cargo test -p kali_types && cargo test -p kali_cli bigint`
Expected: green. Any newly-red test that mixed BigInt with numbers was pinning the miscompile — flip it to expect E3202 and say so in the commit message.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_error/src/_error_codes.rs crates/kali_types/src/resolve/expression.rs crates/kali_cli/tests/soundness_mixed_bigint.rs
git commit -m "fix(types): mixed BigInt arithmetic rejects with E3202 instead of silently floating (soundness batch 1 item 6)"
```

---

### Task 8: Block-bodied arrows parse everywhere; anonymous function args fail closed

**Files:**
- Modify: `crates/kali_parser/src/expression/primary.rs:78-81` (LeftParen arm) and `:21-40` (Identifier arm)
- Modify: `crates/kali_types/src/resolve/call.rs:5` (`resolve_call_expression` — anonymous-fn-arg gate)
- Test: `crates/kali_cli/tests/soundness_block_arrows.rs`

**Interfaces:**
- Consumes: `Parser::try_parse_block_arrow_function_expression` (`declaration.rs:298` — already returns an unnamed `FunctionExpression`; today only the declarator init calls it).

**Background (all verified live on the current binary):** block-bodied arrows outside declarator-init position do not parse as functions. Consequences: (a) `(a) => {…}` as a call argument reparses its params against the outer scope → baffling E3100s; (b) zero-param `() => {…}` bodies FLATTEN into module scope and execute once — `foo("x", () => { console.log(42) })` prints 42 even though `foo` never invokes its callback (silent wrong execution); (c) `Kali.test("t", () => {…})` "works" only via that flatten. Meanwhile the inline `function () {…}` form ALREADY has correct semantics in both positions (real callback registered via `test_register`/`__kali_callback_<index>`; ordinary-call args correctly inert) — so parsing block arrows into the same unnamed-`FunctionExpression` shape inherits a proven lane. Remaining gap (also verified): *invoking* a function-valued param that received an ANONYMOUS function silently no-ops (`cb(5)` does nothing — named functions passed as params work via monomorphized dispatch, anonymous ones have no name to key on). Hence the companion gate: an anonymous function argument to anything other than `Kali.test` rejects.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_block_arrows.rs` (helper slug `kali-soundness-blkarrow`):

```rust
// Zero-param block arrow as a call argument: the body must NOT flatten
// into module scope. Pre-fix this printed "42\n7"; node prints only "7"
// (foo never invokes its callback)… but with the companion gate below the
// anonymous callback arg REJECTS instead — fail-closed until real
// indirect calls exist. Either way "42" must never print.
#[test]
fn block_arrow_arg_body_does_not_flatten() {
    let out = run_source(
        "function foo(a, b) { return 0; }\nfoo(\"x\", () => { console.log(42); });\nconsole.log(7);\n",
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("42"),
        "callback body must not execute at module scope: {out:?}"
    );
    assert!(!out.status.success(), "anonymous fn arg must reject (E5506): {out:?}");
}

// Param'd block arrow as a call argument: pre-fix this died with baffling
// E3100 "undefined identifier 'a'" (params reparsed against outer scope).
// Post-fix it parses as a function and the anonymous-arg gate rejects it
// CLEANLY (E5506 naming the limitation, no E3100 noise).
#[test]
fn parameterized_block_arrow_arg_rejects_cleanly() {
    let out = run_source(
        "function foo(cb) { return 0; }\nfoo((a) => { console.log(a); });\n",
    );
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "want the feature gate, got: {stderr}");
    assert!(!stderr.contains("E3100"), "param misparse must be gone: {stderr}");
}

// Kali.test's inline block-arrow callback is THE allowlisted consumer —
// it must keep compiling (real callback registration, not the flatten).
// Under `kali run` a real callback is registered but not invoked, so
// stdout must be EMPTY (the flatten used to print 42 here).
#[test]
fn kali_test_block_arrow_callback_still_compiles() {
    let out = run_source("Kali.test(\"t\", () => { console.log(42); });\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "");
}

// Declarator lane untouched.
#[test]
fn declarator_block_arrow_still_works() {
    let out = run_source("const f = (x) => { return x + 1; };\nconsole.log(f(4));\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}
```

**IMPORTANT — behavior change to flag at review:** `kali_test_block_arrow_callback_still_compiles` changes observable `kali run` behavior (the flatten used to print 42; a real callback prints nothing under `run`). The real assertion that matters is the harness suite in Step 5 — Kali.test callbacks must still EXECUTE under the test harness, via the `__kali_callback_<index>` exports.

- [ ] **Step 2: Run to verify current behavior**

Run: `cargo test -p kali_cli --test soundness_block_arrows`
Expected: first three FAIL (flatten prints 42 / E3100 noise / flatten prints 42), `declarator_block_arrow_still_works` PASSES.

- [ ] **Step 3: Parser — route block arrows through the FunctionExpression desugar everywhere**

In `crates/kali_parser/src/expression/primary.rs`, LeftParen arm, try the block form FIRST:

```rust
TokenType::LeftParen => {
    // Block-bodied arrows in ANY expression position parse as an
    // unnamed FunctionExpression — the same desugar the declarator
    // init uses. Before this, non-declarator positions fell through:
    // params reparsed against the outer scope (E3100 noise) and a
    // zero-param body FLATTENED into module scope and executed once
    // (silent wrong execution — Kali.test only "worked" via that).
    if let Some(expr) = self.try_parse_block_arrow_function_expression() {
        return expr;
    }
    if let Some(expr) = self.try_parse_arrow_function_expression() {
        return expr;
    }
    // ... existing parenthesized/sequence fallthrough unchanged ...
```

Identifier arm — the single-param form `x => { … }` currently bails via the `peek_next_kind() != LeftBrace` guard; extend it:

```rust
if self.stream.current_kind() == Some(&TokenType::Arrow) {
    if self.stream.peek_next_kind() == Some(&TokenType::LeftBrace) {
        // Single-param block arrow `x => { … }`: same desugar.
        let _ = self.stream.advance(); // consume `=>`
        if let Some(Statement::BlockStatement(block)) = self.parse_block_statement() {
            return Expression::FunctionExpression(Box::new(FunctionExpression {
                id: None,
                params: vec![FunctionParam { name }],
                body: block,
                // ... remaining FunctionExpression fields: copy the
                // construction at declaration.rs:318-328 verbatim ...
            }));
        }
        // Unparseable block: fall through to the identifier return —
        // resolve will produce its normal diagnostics.
    } else {
        // ... existing expression-bodied single-param arrow unchanged ...
    }
}
```

(Copy the exact `FunctionExpression` field set from `try_parse_block_arrow_function_expression` at `declaration.rs:318-328` — the sketch above elides fields that struct requires. Add needed imports.)

- [ ] **Step 4: Types — the anonymous-function-argument gate**

In `crates/kali_types/src/resolve/call.rs`, inside `resolve_call_expression`, before/alongside argument resolution:

```rust
// An ANONYMOUS function expression as a call argument compiles to a
// real standalone function, but nothing can invoke it: indirect calls
// through a param key on a function NAME (monomorphized dispatch), and
// an unnamed function has none — `cb(5)` silently no-ops (verified).
// Fail closed: reject anonymous function args everywhere EXCEPT
// Kali.test, whose callback is invoked BY THE HOST via the
// __kali_callback_<index> export, never through an in-wasm call.
let callee_is_kali_test = matches!(&expr.callee, Expression::MemberExpression(member)
    if member.property == "test"
        && matches!(&member.object, Expression::Identifier(obj) if obj == "Kali"));
if !callee_is_kali_test {
    for arg in &expr.args {
        let anonymous_fn = match arg {
            Expression::FunctionExpression(func) => func.id.is_none(),
            Expression::ArrowFunctionExpression(_) => true,
            _ => false,
        };
        if anonymous_fn {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "an anonymous function as a call argument is unavailable in the current phase (nothing can invoke it); declare a named function and pass its name".to_string(),
            ));
        }
    }
}
```

(Match the file's real diagnostics idiom — same push mechanism as the gates in `resolve/expression.rs:1699-1785`. If `resolve_call_expression` lacks direct diagnostics access, follow however those sibling gates report.)

**Blast-radius check before running:** `grep -rn "=> {" crates/kali_cli/tests/fixtures --include=*.js --include=*.ts | grep -v "const \|let \|var \|function"` — inline block-arrow args in fixtures OTHER than Kali.test callbacks will flip from flatten-miscompile to E5506. Each such fixture test was pinning wrong behavior; update those pins deliberately (list them in the commit message).

- [ ] **Step 5: Run the full affected suites**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_block_arrows`
Expected: all 4 PASS.

Run: `rm -rf crates/kali_cli/tests/fixtures/.kali-cache && cargo test -p kali_cli && cargo test -p kali_parser && cargo test -p kali_types`
Expected: green EXCEPT tests that pinned the flatten (fix per Step 4's blast-radius note). The Kali.test harness suites (browser_*/runtime_smoke) are the critical gate: callbacks must still register and execute under the harness. If harness tests fail because the callback lane can't resolve the now-real FunctionExpression callback (`kali_test_callback_index` resolves by node TEXT against the named-function table — an unnamed inline callback may need its HIR synthetic name to land there), the fix belongs in HIR synthetic naming/codegen function collection: the same lane that already handles `Kali.test("t", function () { … })` (verified working) — diff the two paths and align.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_parser/src/expression/primary.rs crates/kali_types/src/resolve/call.rs crates/kali_cli/tests/soundness_block_arrows.rs
git commit -m "fix(parser,types): block arrows parse as functions in all positions — closes the module-scope body-flatten miscompile; anonymous fn args fail closed except Kali.test (soundness batch 1 item 9)"
```

---

### Task 9: Object-taint seed covers no-initializer bindings

**Files:**
- Modify: `crates/kali_types/src/repr_infer.rs` (assignment visitor — find where `Expression::AssignmentExpression`/assignment statements are visited; the declarator seed at `repr_infer.rs:926-932` is the model)
- Test: `crates/kali_cli/tests/soundness_object_taint.rs`

**Interfaces:**
- Consumes: `self.object_initialized_bindings.insert((func, name))` — the existing taint set (`kali_common/src/repr.rs:97`), already consumed by the compound/update gate at `resolve/expression.rs:2026-2028`.

**Background:** the taint is seeded ONLY from declarator RHS shapes (`var o = {x:1}`). The no-initializer form — `var o; o = {x:1}; o += 1` — escapes it (verified by code inspection; the Spec 7 memory note claiming this form "verified rejecting" was wrong). Node prints `[object Object]1`; kali does integer arithmetic on a never-materialized value.

- [ ] **Step 1: Write the failing tests**

Create `crates/kali_cli/tests/soundness_object_taint.rs` (helper slug `kali-soundness-objtaint`):

```rust
// Declarator form — already rejected (Spec 7 Task 2); pin it.
#[test]
fn declarator_object_compound_rejects() {
    let out = run_source("var o = { x: 1 };\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "{out:?}");
}

// No-initializer form — the gap: the taint was seeded only from
// declarator RHS shapes, so `var o; o = {x:1}; o += 1` slipped through.
#[test]
fn late_assigned_object_compound_rejects() {
    let out = run_source("var o;\no = { x: 1 };\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "{out:?}");
}

// Reassignment-after-scalar form: same seed, via a later assignment.
#[test]
fn reassigned_to_object_compound_rejects() {
    let out = run_source("var o = 0;\no = { x: 1 };\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "{out:?}");
}
```

- [ ] **Step 2: Run to verify the gap**

Run: `cargo test -p kali_cli --test soundness_object_taint`
Expected: first PASSES (existing gate), second and third FAIL (compile and print a wrong integer). If either unexpectedly rejects already, keep it as a pin; the failing one(s) drive Step 3.

- [ ] **Step 3: Widen the seed**

In `crates/kali_types/src/repr_infer.rs`, find the assignment visitor (where `o = <expr>` statements/expressions get visited — grep `AssignmentExpression` in the file; the visitor that flows RHS into the binding's scalar node). Add, mirroring the declarator seed at 926-932:

```rust
// Same syntactic taint as the declarator seed above: an object-literal
// RHS taints the TARGET binding wherever the assignment appears, not
// just in declarator position — `var o; o = {x:1}; o += 1` must hit
// the compound/update gate exactly like `var o = {x:1}` does.
if let Expression::ObjectExpression(_) = rhs {
    if let Some(name) = assignment_target_simple_name(target) {
        self.object_initialized_bindings
            .insert((func.to_string(), name.to_string()));
    }
}
```

(`assignment_target_simple_name` is a sketch — use however the visitor already extracts the target binding name for the scalar-flow bookkeeping in the same match arm; only seed for plain identifier targets, never member/index targets.)

- [ ] **Step 4: Run tests to verify all three pass**

Run: `cargo build -p kali_cli && cargo test -p kali_cli --test soundness_object_taint`
Expected: all 3 PASS.

- [ ] **Step 5: Regression check**

Run: `cargo test -p kali_types && cargo test -p kali_common && cargo test -p kali_cli object`
Expected: green. (The taint only *widens* rejects; a newly-red test means a fixture legitimately compound-assigns an object-assigned binding — inspect whether it was silently miscompiling before.)

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/repr_infer.rs crates/kali_cli/tests/soundness_object_taint.rs
git commit -m "fix(types): object-literal taint seeds from later assignments too, closing the no-initializer compound gap (soundness batch 1 item 10a)"
```

---

### Task 10: `string_arena_loops` poisoning twin pin

**Files:**
- Modify: `crates/kali_mir/src/analysis/arena_gate_tests.rs` (append after the `string_arena_loop_*` family at lines 938-1030)

**Interfaces:**
- Consumes: `arena_table_for(source) -> ArenaTable` (existing test helper in the file), `table.string_arena_loop(func, ordinal) -> bool`.

**Background:** `poisoned_function_retains_no_arena_string_sites` (line 916) pins that name-collision poisoning clears `arena_string_site`. The twin fact — that a poisoned function also opens NO `string_arena_loop` (the Spec 7 Task 4f channel) — is unpinned. Pure test task; if the pin FAILS, that is a real fail-open (a poisoned function opening a per-iteration string arena over facts merged from two bodies) and becomes a fix task — stop and report rather than adjusting the assertion.

- [ ] **Step 1: Write the pin**

Append to `crates/kali_mir/src/analysis/arena_gate_tests.rs`, after the `string_arena_loop_*` family:

```rust
#[test]
fn poisoned_function_opens_no_string_arena_loops() {
    // Twin of `poisoned_function_retains_no_arena_string_sites` for the
    // Spec 7 Task 4f loop channel: two function expressions sharing the
    // name `h` collide in the name-keyed facts; the poisoned merged entry
    // must open NO per-iteration string arenas either — a loop arena
    // granted from facts merged across two distinct bodies could reset
    // memory another body still holds.
    let table = arena_table_for(
        "const a = function h(){ while (1 > 0) { console.log([1].join(\"\")); } };
         const b = function h(){ while (1 > 0) { console.log([2].join(\"\")); } };",
    );
    assert!(!table.string_arena_loop("h", 0));
    assert!(!table.string_arena_loop("h", 1));
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p kali_mir poisoned_function_opens_no_string_arena_loops`
Expected: PASS (the poisoning should already clear the loop channel). **If it FAILS: do not weaken the assertion — you found a live fail-open; report it to your coordinator as a new fix task with this reproducer.**

- [ ] **Step 3: Commit**

```bash
git add crates/kali_mir/src/analysis/arena_gate_tests.rs
git commit -m "test(mir): pin that name-collision poisoning also clears string_arena_loops (soundness batch 1 item 10b)"
```

---

### Task 11: Full-gate verification and PR

**Files:** none (verification + integration only)

- [ ] **Step 1: Full verification battery**

```bash
rm -rf crates/kali_cli/tests/fixtures/.kali-cache
cargo build -p kali_cli
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli
```

Expected: everything green, zero clippy warnings, fmt clean. The kali_cli suite includes the six CLBG byte-for-byte goldens — any golden failure is a regression from this branch; bisect the task commits.

- [ ] **Step 2: Fresh-binary reproducer sweep**

Re-run every headline reproducer from the task tests through the freshly built binary by rerunning all nine new test files:

```bash
cargo test -p kali_cli --test soundness_throw --test soundness_bigint_div_zero --test soundness_object_keys --test soundness_reserved_bindings --test soundness_undefined_sentinel --test soundness_multiarg_array --test soundness_mixed_bigint --test soundness_block_arrows --test soundness_object_taint
```

Expected: all green.

- [ ] **Step 3: Push and open the PR**

```bash
git push -u origin soundness-batch1-pra
gh pr create --title "Soundness Batch 1 PR-A: nine miscompile/fail-open closures" --body "Implements PR-A of docs/superpowers/specs/2026-07-10-soundness-batch-design.md: throw print-then-trap, keyword object-literal keys, unified nullish recognizer (+ ??= undefined admit), Array(a,b,c) desugar, E3202 mixed-BigInt reject, reserved-word binding reject, BigInt div-zero message, block-arrow flatten closure + anonymous-fn-arg gate, object-taint seed widening, poisoning twin pin."
```

Do NOT merge yet — the standing convention requires the whole-branch adversarial review (with live reproducers, re-reviewed after every fix wave) before self-merging. That review is the next phase after this plan completes.

---

## Verification sweep (for the final reviewer)

Every item maps spec §→task: §3.1→Task 1, §3.7→Task 2, §3.2→Task 3, §3.6→Task 4, §3.3→Task 5, §3.4→Task 6, §3.5→Task 7, §3.8+flatten closure→Task 8, §3.9 seed→Task 9, §3.9 pin→Task 10. Known deliberate deviations from the spec text (approved during planning, live-verified):
1. §3.8 originally called for a targeted parse error; live probing falsified its premise (zero-param block-arrow bodies FLATTEN into module scope and execute — a wrong-execution miscompile Kali.test depended on). Task 8 fixes the flatten properly instead, with the anonymous-fn-arg fail-closed gate as companion. Update the spec doc's §3.8 with a short revision note in the PR (same convention as Spec 7's §7/§8 falsification notes).
2. §3.9's "verified rejecting" claim for the no-initializer form was wrong — Task 9 closes it as a real gap, not a pin.
