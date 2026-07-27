# R-35 Switch Lowering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop `switch` from moving code out of its enclosing function, then lower an allowlisted subset of `switch` correctly and fail closed with `E5506` on everything else.

**Architecture:** Two stages, each its own PR. Stage 1 is a parser fix: `parse_switch_statement` never consumes the switch's closing `}`, so every statement after a `switch` inside a function body is silently reparented to module scope. Stage 2 gives the HIR `SwitchStmt` node a text tag (today it has none, so it reaches codegen as a `Branch` with `text: None` and falls into the generic arm — which is the `if` lowering), then adds an `emit_switch` that starts by denying **every** switch and admits proven shapes one at a time. At no point between tasks does a silent miscompile exist: the switch is either correct or honestly refused.

**Tech Stack:** Rust (workspace of ~23 crates), `wasm-encoder` for codegen, `cargo test` for the gate, `node v26.5.0` as the differential oracle, `tempfile` for end-to-end test fixtures.

## Global Constraints

Copied from `docs/superpowers/specs/2026-07-27-r35-switch-lowering-design.md`. Every task's requirements implicitly include this section.

- **Oracle**: `node v26.5.0`. Every admitted shape must match node **byte-for-byte**.
- **Gate**: `cargo test --workspace --no-fail-fast`, diffed against a `main` worktree built from the same commit. The bar is **zero newly-red**. A red that is also red on `main` is baseline, not a regression.
- **Never quote a count without its baseline commit.** A number without a named baseline is not a measurement.
- **Probe instrument rules** (violating any one of these silently invalidates the evidence):
  1. Exactly **one argument per `console.log`**, built by literal-rooted concatenation (`"x=" + v`). Multi-argument logging is itself a defect (R-04).
  2. Capture kali's exit status **without a pipe**, or via `PIPESTATUS`. `cmd | tail` makes `$?` the status of `tail`, erasing the fail-closed/silent distinction.
  3. **No default parameters anywhere in a fixture** (R-01 truncates the module).
  4. **No probe may depend on a statement placed after the `switch`** until Task 2 has landed. That is the corrupted position.
- **Anti-spot-check**: for every admitted cell, vary the discriminant and assert the answer varies with it. Never use a discriminant for which the buggy lowering happens to be right — today `s(10)` is exactly such a value.
- **Allowlist, never denylist.** Deny by failing to construct positive evidence, at one choke point. Do not enumerate bad shapes.
- **Scope is a required test axis.** Module scope and function scope are different programs in kali.
- **Register edits land in the same commit as the status change they describe.** §0 of the register is a precedence section: a stale §0 row outranks correct per-entry text.

---

## File Structure

**Stage 1 — parser**

| file | responsibility | change |
|---|---|---|
| `crates/kali_parser/src/parser.rs` | parser struct + shared helpers | **Modify** — add `expect(kind) -> bool` next to the existing `push_feature_unavailable` |
| `crates/kali_parser/src/statement.rs:503-575` | `parse_switch_statement` | **Modify** — consume the closer; route six required-token positions through `expect` |
| `crates/kali_parser/tests/parser_integration.rs:508` | parser-level assertions | **Modify** — extend `mod switch` |
| `crates/kali_cli/tests/switch_parser_containment.rs` | end-to-end leak pins | **Create** |
| `docs/superpowers/followups/r35-switch-boundary-rederived.md` | the re-derived boundary matrix | **Create** |
| `docs/superpowers/followups/kali-silent-miscompile-register.md` | canonical register | **Modify** — §0.2, §0.3, new G1 entry, blind-`advance()` inventory |

**Stage 2 — lowering**

| file | responsibility | change |
|---|---|---|
| `crates/kali_hir/src/lowering/statement.rs:84-100` | AST→HIR for `SwitchStatement` | **Modify** — tag the switch and each case block |
| `crates/kali_codegen/src/lower.rs` | LIR helpers, ordinals, local reservation | **Modify** — add `switch_preorder_ordinals`, `switch_disc_local_name`, reserve in `collect_function_locals` |
| `crates/kali_codegen/src/emitter.rs` | `FunctionEmitter` state | **Modify** — add `switch_ordinals` field, resolve in `new` |
| `crates/kali_codegen/src/emit/switch.rs` | `SwitchPlan`, admittance, `emit_switch` | **Create** — the whole feature lives in one focused file |
| `crates/kali_codegen/src/emit/mod.rs` | emit module list | **Modify** — declare `switch` |
| `crates/kali_codegen/src/emit/control_flow.rs:1760-1799` | `Branch` text dispatch | **Modify** — add the `Some("switch")` arm |
| `crates/kali_cli/tests/switch_runtime.rs` | admitted-shape acceptance matrix | **Create** |
| `crates/kali_cli/tests/switch_fail_closed.rs` | denied-shape `E5506` matrix | **Create** |

`emit/switch.rs` is a new file rather than more lines in `control_flow.rs` (already 3215 lines) because the admittance predicate and the lowering must be readable together — that is the whole review surface for "does the allowlist actually close the class".

---

# STAGE 1 — Parser containment

### Task 1: Establish the gate baseline

No code changes. This exists because the spec forbids quoting a count without a baseline, and every later task's "zero newly-red" claim is meaningless without this number.

**Files:**
- Create: `docs/superpowers/followups/r35-gate-baseline.txt`

- [ ] **Step 1: Create a `main` worktree at the current commit**

```bash
cd /workspace
git rev-parse HEAD          # record this SHA; it is the named baseline
git worktree add .worktrees/r35-base main
```

- [ ] **Step 2: Run the full workspace gate on the baseline worktree**

```bash
cd /workspace/.worktrees/r35-base
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-baseline.txt
grep -E "^(test result|error)" /tmp/r35-baseline.txt | tail -40
```

Expected: this takes a long time. Do not use `--fail-fast`; partial enumeration has produced false drain counts in this repository before.

- [ ] **Step 3: Extract the failing-test list**

```bash
cd /workspace/.worktrees/r35-base
grep -E "^test .* FAILED$" /tmp/r35-baseline.txt | sort > /workspace/docs/superpowers/followups/r35-gate-baseline.txt
wc -l /workspace/docs/superpowers/followups/r35-gate-baseline.txt
```

- [ ] **Step 4: Write the baseline header**

Prepend to `docs/superpowers/followups/r35-gate-baseline.txt`:

```
# Workspace gate baseline for the R-35 switch project
# Baseline commit: <SHA from Step 1>
# Command: cargo test --workspace --no-fail-fast
# Measured: <date>
# Failing test count: <N from Step 3>
#
# Every later "zero newly-red" claim in this project is diffed against THIS list.
# A test failing here and failing on the branch is baseline, not a regression.
```

- [ ] **Step 5: Commit**

```bash
cd /workspace
git add docs/superpowers/followups/r35-gate-baseline.txt
git commit -m "test(gate): record the R-35 workspace gate baseline with its named commit"
```

---

### Task 2: Close the parser leak

**Files:**
- Modify: `crates/kali_parser/src/statement.rs:511-517`
- Test: `crates/kali_cli/tests/switch_parser_containment.rs` (create)
- Test: `crates/kali_parser/tests/parser_integration.rs:508-527`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: a parser where a statement following a `switch` inside a function body is a sibling of the switch **within that function**. Every later task depends on this; probes written before it silently test a different program.

- [ ] **Step 1: Write the failing end-to-end leak test**

Create `crates/kali_cli/tests/switch_parser_containment.rs`:

```rust
use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

// `s` is NEVER called. If the statement after the switch stays inside `s`,
// nothing mutates `g` and the program prints `g=0` (node's answer). Before the
// fix, `g = 99` was reparented to module scope and ran at module load, so kali
// printed `g=99`.
#[test]
fn statement_after_switch_does_not_escape_the_function() {
    let src = "var g = 0;\n\
               function s(x) {\n\
                 switch (x) {\n\
                   case 1: g = 1;\n\
                 }\n\
                 g = 99;\n\
               }\n\
               console.log(\"g=\" + g);\n";
    assert_eq!(run_js(src), "g=0\n");
}

// A whole function declared AFTER a switch-containing function used to vanish,
// because the leaked `return` terminated the module before it was reached.
#[test]
fn function_declared_after_a_switch_function_survives() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 1: return \"A\";\n\
                 }\n\
                 return \"Z\";\n\
               }\n\
               function t() { return \"T\"; }\n\
               console.log(\"t=\" + t());\n";
    assert_eq!(run_js(src), "t=T\n");
}

// The callee's own output used to disappear entirely: the leaked `return 0;`
// terminated the module, so the module-scope console.log never ran.
#[test]
fn a_call_whose_callee_contains_a_switch_still_prints() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case 10: r = 1;\n\
                 }\n\
                 return 0;\n\
               }\n\
               console.log(\"v=\" + s(10));\n";
    assert_eq!(run_js(src), "v=0\n");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --test switch_parser_containment`

Expected: FAIL. `statement_after_switch_does_not_escape_the_function` asserts `"g=0\n"` and receives `"g=99\n"`. The other two fail on empty stdout.

- [ ] **Step 3: Write the parser-level assertion**

In `crates/kali_parser/tests/parser_integration.rs`, inside `mod switch`, add:

```rust
    // The switch's closing brace must be CONSUMED. If it is not, the enclosing
    // function-body block parser sees it as its own terminator and every
    // statement after the switch is reparented to module scope.
    #[test]
    fn test_switch_does_not_leak_following_statements_out_of_a_function() {
        let output = parse("function s(x) { switch (x) { case 1: x = 1; } return x; }");

        // The whole program is ONE statement: the function declaration.
        assert_eq!(
            output.statements.len(),
            1,
            "statements leaked out of the function body: {:?}",
            output.statements
        );

        match &output.statements[0] {
            kali_ast::Statement::FunctionDeclaration(fd) => {
                // switch, then return — both inside the function.
                assert_eq!(fd.body.body.len(), 2, "function body: {:?}", fd.body.body);
                assert!(matches!(
                    fd.body.body[0],
                    kali_ast::Statement::SwitchStatement(_)
                ));
                assert!(matches!(
                    fd.body.body[1],
                    kali_ast::Statement::ReturnStatement(_)
                ));
            }
            other => panic!("Expected FunctionDeclaration, got {other:?}"),
        }
    }
```

- [ ] **Step 4: Run it to verify it fails**

Run: `cargo test -p kali_parser --test parser_integration switch`

Expected: FAIL with `statements leaked out of the function body` — the parse yields 2 top-level statements (the function, plus the leaked `return`).

- [ ] **Step 5: Make the fix**

In `crates/kali_parser/src/statement.rs`, in `parse_switch_statement`, replace the clause-loop's non-consuming break:

```rust
            if self.stream.current_kind() == Some(&TokenType::RightBrace) {
                break;
            }
```

with the consuming form its three siblings already use (`parse_block_statement:179`, `parse_class_body:286`, `parse_arrow_function_body_expression:547`):

```rust
            // Consume the switch's closing brace. Inspecting it without
            // consuming left it for the ENCLOSING block parser, which then
            // treated it as its own terminator — silently reparenting every
            // statement after the switch to module scope (a function that was
            // never called still ran its post-switch assignment at module
            // load). Every other block-closing site in this parser consumes
            // its closer; this was the only one that did not.
            if self.stream.accept(TokenType::RightBrace) {
                break;
            }
```

- [ ] **Step 6: Run both test suites to verify they pass**

Run: `cargo test -p kali_parser --test parser_integration switch && cargo test -p kali_cli --test switch_parser_containment`

Expected: PASS, all four tests.

- [ ] **Step 7: Verify no newly-red against the baseline**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task2.txt
grep -E "^test .* FAILED$" /tmp/r35-task2.txt | sort > /tmp/r35-task2-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task2-failed.txt
```

Expected: the `comm` output is **empty** (no newly-red). If a test appears, it is a regression and must be resolved before committing — do not re-pin it.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_parser/src/statement.rs \
        crates/kali_parser/tests/parser_integration.rs \
        crates/kali_cli/tests/switch_parser_containment.rs
git commit -m "fix(parser): consume the switch closing brace — stop reparenting post-switch statements to module scope

parse_switch_statement broke its clause loop on RightBrace by inspecting it
without consuming it, so the enclosing block parser treated that brace as its
own terminator. Every statement after a switch inside a function body was
silently moved to module scope: a function that was never called still ran its
post-switch assignment at module load (g=99 where node prints g=0), and a
function declared after a switch-containing function disappeared entirely.

Unique site in the parser; parse_block_statement, parse_class_body and
parse_arrow_function_body_expression all already accept() their closer."
```

---

### Task 3: Report missing required tokens instead of silently accepting them

Task 2 fixed the symptom. This task fixes the mechanism: all six required-token positions in `parse_switch_statement` are blind `advance()` calls or a discarded `accept` bool, so each silently accepts whatever token is present. The switch's opening `{` is consumed only by the clause loop's "unknown token, skip it" fallthrough arm.

**Files:**
- Modify: `crates/kali_parser/src/parser.rs` (add `expect`)
- Modify: `crates/kali_parser/src/statement.rs:503-575`
- Test: `crates/kali_parser/tests/parser_integration.rs`

**Interfaces:**
- Consumes: Task 2's consumed closer.
- Produces: `Parser::expect(&mut self, kind: TokenType) -> bool` — consumes and returns `true` on match; pushes `E2000` and returns `false` on mismatch. Available to the whole parser crate; only `parse_switch_statement` calls it in this project.

- [ ] **Step 1: Write the failing test**

In `crates/kali_parser/tests/parser_integration.rs`, inside `mod switch`, add:

```rust
    // E2000 (`e2::EXPECTED_TOKEN`) is declared in kali_error but was emitted
    // nowhere in the compiler: the parser had never once reported a required
    // token as missing. A malformed switch header was silently accepted.
    #[test]
    fn test_switch_missing_paren_reports_expected_token() {
        let output = parse("switch x { case 1: break; }");
        assert!(
            output.diagnostics.iter().any(|d| d.code == 2000),
            "expected an E2000 diagnostic, got {:?}",
            output.diagnostics
        );
    }

    #[test]
    fn test_switch_missing_case_colon_reports_expected_token() {
        let output = parse("switch (x) { case 1 break; }");
        assert!(
            output.diagnostics.iter().any(|d| d.code == 2000),
            "expected an E2000 diagnostic, got {:?}",
            output.diagnostics
        );
    }

    // A well-formed switch must stay clean — the helper must not fire on the
    // shapes that already parse.
    #[test]
    fn test_well_formed_switch_reports_no_expected_token() {
        let output = parse("switch (x) { case 1: break; default: break; }");
        assert!(
            !output.diagnostics.iter().any(|d| d.code == 2000),
            "well-formed switch produced E2000: {:?}",
            output.diagnostics
        );
    }
```

- [ ] **Step 2: Run to verify the first two fail**

Run: `cargo test -p kali_parser --test parser_integration switch`

Expected: `test_switch_missing_paren_reports_expected_token` and `test_switch_missing_case_colon_reports_expected_token` FAIL (`expected an E2000 diagnostic, got []`). `test_well_formed_switch_reports_no_expected_token` already passes — that is correct; it is the guard against over-firing.

- [ ] **Step 3: Add the `expect` helper**

In `crates/kali_parser/src/parser.rs`, add to `impl Parser` next to `push_feature_unavailable`, and extend the existing `use kali_error::{_error_codes::e5, diagnostic::Diagnostic};` to `use kali_error::{_error_codes::{e2, e5}, diagnostic::Diagnostic};`:

```rust
    /// Consume `kind` if present; otherwise report `E2000` and consume nothing.
    ///
    /// The parser had only `accept -> bool`, so every REQUIRED-token position
    /// was a blind `advance()` or a discarded bool — each silently accepting
    /// whatever token happened to be there. `e2::EXPECTED_TOKEN` was declared
    /// in `kali_error` and emitted nowhere in the compiler.
    ///
    /// Returns whether the token was consumed, so a caller can decide between
    /// continuing and bailing. It deliberately does NOT skip the offending
    /// token: recovery stays the caller's decision.
    pub(crate) fn expect(&mut self, kind: TokenType) -> bool {
        if self.stream.accept(kind) {
            return true;
        }
        let found = match self.stream.current_kind() {
            Some(k) => format!("{k:?}"),
            None => "end of input".to_string(),
        };
        self.diagnostics.push(Diagnostic::error(
            e2::EXPECTED_TOKEN as u32,
            format!("expected {kind:?} but found {found}"),
        ));
        false
    }
```

- [ ] **Step 4: Route the six required-token positions through it**

In `crates/kali_parser/src/statement.rs`, `parse_switch_statement`. Replace the header's three blind advances:

```rust
        let _ = self.stream.advance();
        let _ = self.stream.advance();

        let discriminant = self.parse_expression();
        let _ = self.stream.advance();
```

with:

```rust
        let _ = self.stream.advance(); // `switch`, established by the dispatch
        self.expect(TokenType::LeftParen);

        let discriminant = self.parse_expression();
        self.expect(TokenType::RightParen);
        self.expect(TokenType::LeftBrace);
```

The `{` is now consumed explicitly. It was previously consumed only by the clause loop's "unknown token, skip it" fallthrough arm, which is why a malformed switch body vanished silently.

Replace the `case` clause's colon skip:

```rust
                let _ = self.stream.advance();
                let test = self.parse_expression();
                let _ = self.stream.advance();
```

with:

```rust
                let _ = self.stream.advance(); // `case`, established by the check above
                let test = self.parse_expression();
                self.expect(TokenType::Colon);
```

Replace the `default` clause's colon skip:

```rust
                let _ = self.stream.advance();
                let _ = self.stream.advance();
```

with:

```rust
                let _ = self.stream.advance(); // `default`, established by the check above
                self.expect(TokenType::Colon);
```

Finally, report the unterminated case. Replace Task 2's loop guard:

```rust
            if self.stream.eof() {
                break;
            }
```

with:

```rust
            if self.stream.eof() {
                // Ran out of tokens without ever seeing the closing brace.
                self.expect(TokenType::RightBrace);
                break;
            }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p kali_parser --test parser_integration switch`

Expected: PASS, all six tests in `mod switch`.

Note: if `TokenType::Colon` does not exist under that name, run `grep -n "Colon" crates/kali_lexer/src/token.rs` and use the actual variant. Do not skip the colon positions.

- [ ] **Step 6: Confirm the end-to-end pins still pass**

Run: `cargo test -p kali_cli --test switch_parser_containment`

Expected: PASS, all three.

- [ ] **Step 7: Verify no newly-red against the baseline**

```bash
cd /workspace
cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task3.txt
grep -E "^test .* FAILED$" /tmp/r35-task3.txt | sort > /tmp/r35-task3-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task3-failed.txt
```

Expected: empty. `expect` reports where the old code silently accepted, so a newly-red here means a real program in the suite had a malformed switch — investigate before proceeding; do not weaken `expect`.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_parser/src/parser.rs \
        crates/kali_parser/src/statement.rs \
        crates/kali_parser/tests/parser_integration.rs
git commit -m "fix(parser): report missing required tokens in switch headers via E2000

Adds the parser's missing expect(kind) helper and routes all six required-token
positions in parse_switch_statement through it: switch, (, ), {, each clause's
colon, and the closing brace at EOF.

e2::EXPECTED_TOKEN was declared in kali_error and emitted nowhere in the
compiler — the parser had never reported a required token as missing. The
switch's opening brace in particular was consumed only by the clause loop's
'unknown token, skip it' arm, so a malformed switch body vanished silently."
```

---

### Task 4: Re-derive the R-35 boundary and correct the register

The register's recorded R-35 boundary ("`break` in a case → E5506; a local read in a case → E3100 — so the silent window is exactly all-return/no-break/no-local") was the parser leak's shadow: that `break` was a *leaked* break evaluated at module scope with no loop frame, and that E3100 was a *leaked* identifier read resolved against module scope. Both artifacts are now gone, so the true boundary is unknown and must be measured.

**Files:**
- Create: `docs/superpowers/followups/r35-switch-boundary-rederived.md`
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md`

**Interfaces:**
- Consumes: Tasks 2 and 3.
- Produces: the measured boundary matrix that finalizes Stage 2's allowlist rules 4 (clause terminators) and 5 (`let`/`const` in clause bodies). Task 7 onward reads this file.

- [ ] **Step 1: Build a fresh binary**

```bash
cd /workspace && cargo build --bin kali
```

Fix reports are unreliable in this repository — always re-run reproducers on a freshly built binary. A prior stage lost a round to a fix report that falsely claimed a lane was closed.

- [ ] **Step 2: Run the boundary matrix**

Create `/tmp/r35-rederive/` and run each probe below under both `./target/debug/kali run <f>` and `node <f>`, recording stdout and the **unpiped** exit status for each. One `console.log` argument per call, literal-rooted concatenation, no default parameters.

The cells to measure, all in **both** module scope and in-function scope:

| # | shape |
|---|---|
| 1 | all clauses `return`, numeric discriminant, `default` last |
| 2 | all clauses `return`, **string** discriminant |
| 3 | all clauses `r = n; break;` over a `var` declared before the switch |
| 4 | a clause declaring **and** reading its own `var` |
| 5 | a clause declaring **and** reading its own `let` |
| 6 | a clause declaring **and** reading its own `const` |
| 7 | a clause ending in `throw` |
| 8 | true fallthrough (non-empty clause with no terminator) |
| 9 | empty-clause grouping (`case 1:` immediately followed by `case 2: return x;`) |
| 10 | `default` in a **non-final** position |
| 11 | duplicate case tests |
| 12 | switch nested inside a `for` loop, clause containing `break` |
| 13 | switch nested inside a `for` loop, clause containing `continue` |
| 14 | non-literal case test (`case y:` where `y` is a binding) |
| 15 | float discriminant |
| 16 | boolean discriminant |

- [ ] **Step 3: Record the matrix**

Create `docs/superpowers/followups/r35-switch-boundary-rederived.md` with a header naming the commit, the binary, the oracle version, and the date, then one row per cell:

```markdown
# R-35 boundary, re-derived after the parser-containment fix

Baseline commit: <SHA>
Binary: ./target/debug/kali, rebuilt at that commit
Oracle: node v26.5.0
Measured: <date>

Supersedes the boundary recorded in the register's §0.3 R-35 bullet, which was
measured THROUGH the parser leak and is therefore void.

| # | shape | scope | kali | node | exit | verdict |
|---|---|---|---|---|---|---|
```

Verdict vocabulary, from the register: `SILENT` (exit 0, no diagnostic, wrong), `FAIL-CLOSED` (honest nonzero Enn), `FL-INTERNAL` (nonzero but wrong kind — E4201/E4003), `CORRECT`.

- [ ] **Step 4: Rewrite the register's R-35 bullet**

In `docs/superpowers/followups/kali-silent-miscompile-register.md` §0.3, replace the R-35 bullet's boundary sentence. It must now state: clauses beyond the second are **never emitted at all** (so R-35 is Tier 1, silently drops code, not only Tier 2); the wrong clause's **side effects** run, not merely its value (`console.log` in clauses prints `ten` where node prints `twenty`); **string discriminants** are affected; and the previously-recorded boundary was the parser leak's shadow, now superseded by `r35-switch-boundary-rederived.md`.

- [ ] **Step 5: Add the parser-leak entry**

Add a new numbered entry in cluster **G1** (parser fail-open recovery) for the leak. It is not R-35 — different layer, different blast radius, higher severity. Use the next free `R-nn` (the register currently holds R-01..R-48, so **R-49**; confirm with `grep -c "^### R-" docs/superpowers/followups/kali-silent-miscompile-register.md` before assigning). Record: the mechanism (`parse_switch_statement` inspected `RightBrace` without consuming it), the decisive repro (`g=99` where node prints `g=0`, with the function never called), that it was the **unique** such site in the parser, and that it is **CLOSED** by this stage with the closing commit named.

- [ ] **Step 6: Update §0.2's status rows in the same edit**

Add or update the §0.2 rows for R-35 and the new R-49 entry. §0 is a precedence section — a stale §0 row outranks correct per-entry text, which is the exact trap PRs #28 and #29 existed to clean up. Do not leave this to a later commit.

- [ ] **Step 7: File the blind-`advance()` inventory**

`crates/kali_parser/src/statement.rs` alone holds 28 `let _ = self.stream.advance();` sites. Record the count and a `file:line` list as explicit follow-up work in the register, noting that the parser-wide sweep is its own project with its own test-census cost and was deliberately not attempted here.

```bash
grep -rn "let _ = self.stream.advance();" crates/kali_parser/src/ | wc -l
grep -rn "let _ = self.stream.advance();" crates/kali_parser/src/
```

- [ ] **Step 8: Record that E2000/E2001 are now emitted**

Add a line to the register noting that `e2::EXPECTED_TOKEN` was declared-but-never-emitted until this stage. "The parser has never reported a missing required token" was a standing fact about this repository's evidence base and is no longer true.

- [ ] **Step 9: Commit**

```bash
git add docs/superpowers/followups/
git commit -m "docs(register): re-derive the R-35 boundary after parser containment; add the G1 leak entry

The recorded R-35 boundary was the parser leak's shadow and is void. R-35 is
Tier 1 (clauses beyond the second are never emitted), flips side effects, and
covers string discriminants. Adds the parser leak as its own G1 entry, updates
the §0.2 precedence rows in this same commit, and files the blind-advance()
inventory as follow-up work."
```

- [ ] **Step 10: Open the Stage 1 PR**

```bash
gh pr create --title "R-35 Stage 1 — switch parser containment" --body "$(cat <<'EOF'
Closes the parser half of R-35.

`parse_switch_statement` never consumed the switch's closing brace, so every
statement after a `switch` inside a function body was silently reparented to
module scope. Proven with a function that is never called yet whose post-switch
assignment executes at module load (`g=99` where node prints `g=0`).

- Consume the closer, matching the parser's three other block-closing sites.
- Add the parser's missing `expect(kind)` helper; route all six required-token
  positions in `parse_switch_statement` through it. `e2::EXPECTED_TOKEN` was
  declared and emitted nowhere in the compiler until now.
- Re-derive the R-35 boundary on the fixed parser; the recorded one was the
  leak's shadow.
- Register: rewrite the R-35 bullet, add the leak as its own G1 entry, update
  the §0.2 precedence rows in the same commit, file the blind-advance inventory.

Gate: zero newly-red against the baseline recorded in
`docs/superpowers/followups/r35-gate-baseline.txt`.

Note: `switch` still selects the wrong clause. That is Stage 2.
EOF
)"
```

**Whole-stage review gate:** before merging, run an adversarial review of the entire Stage 1 diff, not just per-task review. Seven consecutive stages in this repository have had a CRITICAL that only the whole-stage pass caught.

---

# STAGE 2 — Allowlisted switch lowering

### Task 5: Tag the switch and its case blocks in HIR

Behavior-neutral on its own: `Some("switch")` matches no arm in the `Branch` dispatch yet, so it still falls through `_` to `emit_branch`. That is deliberate — this task is separable and reviewable without any behavior change.

**Files:**
- Modify: `crates/kali_hir/src/lowering/statement.rs:84-100`
- Test: `crates/kali_hir/src/lowering/` (follow the crate's existing test placement; `grep -rn "mod .*tests" crates/kali_hir/src/` to find it)

**Interfaces:**
- Produces: an LIR `Branch` node with `text == Some("switch")` whose `children[0]` is the discriminant and whose `children[1..]` are case blocks, each a `Block` with `text == Some("case")` or `Some("default")`. A `"case"` block's `children[0]` is its test expression; a `"default"` block has no test child. Tasks 6-10 rely on exactly this shape.

- [ ] **Step 1: Write the failing test**

Assert that lowering `switch (x) { case 1: return 1; default: return 2; }` produces a `SwitchStmt` whose text is `"switch"`, whose first case block's text is `"case"`, and whose second case block's text is `"default"`.

```rust
#[test]
fn switch_and_its_case_blocks_are_text_tagged() {
    let hir = lower_source("function f(x) { switch (x) { case 1: return 1; default: return 2; } }");
    let switch = find_node(&hir, HirNodeKind::SwitchStmt).expect("a SwitchStmt");
    assert_eq!(switch.text.as_deref(), Some("switch"));

    // children[0] is the discriminant; children[1..] are the case blocks.
    let cases: Vec<_> = switch.children[1..]
        .iter()
        .map(|id| hir.node(*id).text.as_deref())
        .collect();
    assert_eq!(cases, vec![Some("case"), Some("default")]);
}
```

Adapt `lower_source` / `find_node` / `hir.node` to the helpers the crate's existing HIR tests use — read them first rather than inventing new ones.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_hir switch`

Expected: FAIL — `switch.text` is `None`, and both case blocks' texts are `None`.

- [ ] **Step 3: Tag the nodes**

In `crates/kali_hir/src/lowering/statement.rs`, replace the `Statement::SwitchStatement` arm's two `alloc` calls with `alloc_text`:

```rust
            Statement::SwitchStatement(SwitchStatement {
                discriminant,
                cases,
            }) => {
                // Text "switch" survives MIR ControlFlow -> LIR Branch so
                // codegen's text-keyed Branch dispatch can lower it. A
                // None-text Branch falls into the generic arm, which IS the
                // `if` lowering — the discriminant got truthiness-tested,
                // clauses 0/1 became then/else and clauses 2+ were never
                // emitted. Exactly the hole recorded for `throw` below.
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::SwitchStmt, None, "switch".to_string());
                push_child!(self, id, self.lower_expression(discriminant));
                for case in cases {
                    // Tag "case" vs "default": a case block's children are
                    // [testExpr, stmts...] and a default's are [stmts...], so
                    // without the tag a `default` is positionally
                    // indistinguishable from a `case` whose first statement is
                    // an expression statement.
                    let tag = if case.test.is_some() { "case" } else { "default" };
                    let case_id =
                        self.builder
                            .alloc_text(HirNodeKind::Block, None, tag.to_string());
                    if let Some(test) = &case.test {
                        push_child!(self, case_id, self.lower_expression(test));
                    }
                    for stmt in &case.consequent {
                        push_child!(self, case_id, self.lower_statement(stmt));
                    }
                    push_child!(self, id, case_id);
                }
                id
            }
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p kali_hir switch`

Expected: PASS.

- [ ] **Step 5: Verify behavior is unchanged**

```bash
cargo build --bin kali
cd /tmp && cat > t.js <<'EOF'
function s(x) {
  switch (x) {
    case 10: return "A";
    case 20: return "B";
    default: return "D";
  }
}
console.log("s20=" + s(20));
EOF
/workspace/target/debug/kali run t.js
```

Expected: still `s20=A` (wrong, matching the pre-task behavior). This task must not change behavior — `Some("switch")` matches no dispatch arm yet, so it still reaches `_ => emit_branch`. If the output changed, a `Block` text tag has been consumed somewhere unexpected; find that consumer before proceeding.

- [ ] **Step 6: Verify no newly-red against the baseline**

```bash
cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task5.txt
grep -E "^test .* FAILED$" /tmp/r35-task5.txt | sort > /tmp/r35-task5-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task5-failed.txt
```

Expected: empty. A newly-red here means something keys on a `Block` having no text.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_hir/
git commit -m "refactor(hir): text-tag SwitchStmt and its case blocks (behavior-neutral)

SwitchStmt allocated with no text, so it reached codegen as a Branch with
text: None and fell into the generic arm — the `if` lowering. Tagging it
'switch' makes the text-keyed dispatch able to route it; no arm matches yet, so
behavior is unchanged. Case blocks are tagged 'case'/'default' because a
default was otherwise positionally indistinguishable from a case whose first
statement is an expression statement."
```

---

### Task 6: Fail closed on every switch

Introduces `emit_switch` with an **empty allowlist**: it denies every switch with `E5506`. This is a real milestone, not scaffolding — it converts an everyday silent miscompile into an honest refusal. Tasks 7-10 then admit proven shapes one at a time, so no intermediate state of this branch ever miscompiles a switch silently.

**Files:**
- Create: `crates/kali_codegen/src/emit/switch.rs`
- Modify: `crates/kali_codegen/src/emit/mod.rs`
- Modify: `crates/kali_codegen/src/emit/control_flow.rs:1760-1799`
- Test: `crates/kali_cli/tests/switch_fail_closed.rs` (create)

**Interfaces:**
- Consumes: Task 5's tagged nodes.
- Produces: `FunctionEmitter::emit_switch(&mut self, function: &mut Function, id: LirNodeId, node: &LirNode) -> EmittedValue`, and `SwitchPlan` / `SwitchClause` / `ClauseTerminator` in `emit/switch.rs`. Tasks 7-10 extend `SwitchPlan::build` only.

- [ ] **Step 1: Write the failing test**

Create `crates/kali_cli/tests/switch_fail_closed.rs` with the `run_js_expect_failure` helper copied from `crates/kali_cli/tests/bitwise_operators_runtime.rs:27-46`, plus:

```rust
// Until a shape is explicitly admitted, every switch must fail closed with an
// honest E5506 naming the limit — never silently select the wrong clause.
#[test]
fn switch_is_fail_closed_not_silently_wrong() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 10: return \"A\";\n\
             case 20: return \"B\";\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(20));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
    assert!(
        out.contains("switch"),
        "the diagnostic must name switch as the limit, got: {out}"
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test switch_fail_closed`

Expected: FAIL with `expected rejection but it ran` — kali currently exits 0 printing `v=A`.

- [ ] **Step 3: Create the emit module**

Create `crates/kali_codegen/src/emit/switch.rs`:

```rust
//! `switch` lowering: admittance plan + emit.
//!
//! The plan is built from POSITIVE evidence only. `SwitchPlan::build` returns
//! `Err(reason)` unless it can prove every part of the switch is in the
//! admitted set, and `emit_switch` denies on `Err`. There is deliberately no
//! denylist of bad shapes anywhere in this file: this repository's most
//! repeated lesson is that a denylist of shapes leaks forever and only an
//! allowlist at the choke point closes a class (Spec 4a needed six rounds
//! before a default-deny at the single read site closed the for-in-key class
//! by construction).
//!
//! Extending the admitted set therefore means adding a proof to `build`, never
//! removing a rejection.

use crate::*;

/// How a clause body ends. Only terminators in this enum are admitted; a
/// clause that ends any other way is true fallthrough and is denied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClauseTerminator {
    /// The clause's last statement is `return`.
    Return,
    /// The clause's last statement is an unlabeled `break`.
    Break,
    /// The clause has no statements at all and groups onto the next clause.
    EmptyGroup,
}

/// One admitted clause.
pub(crate) struct SwitchClause {
    /// `None` for the `default` clause.
    pub(crate) test: Option<LirNodeId>,
    pub(crate) body: LirNodeId,
    pub(crate) terminator: ClauseTerminator,
}

/// A switch this emitter has proven it can lower correctly.
pub(crate) struct SwitchPlan {
    pub(crate) discriminant: LirNodeId,
    pub(crate) clauses: Vec<SwitchClause>,
}

impl<'a> FunctionEmitter<'a> {
    /// Build a plan, or explain why this switch is not admitted.
    ///
    /// The allowlist is currently EMPTY: nothing is admitted. Tasks that widen
    /// the admitted set replace the unconditional rejection with a proof.
    pub(crate) fn switch_plan(&self, _node: &LirNode) -> Result<SwitchPlan, String> {
        Err("no switch shape is admitted in the current phase".to_string())
    }

    pub(crate) fn emit_switch(
        &mut self,
        function: &mut Function,
        _id: LirNodeId,
        node: &LirNode,
    ) -> EmittedValue {
        match self.switch_plan(node) {
            Ok(plan) => self.emit_switch_plan(function, plan),
            Err(reason) => {
                let message = format!(
                    "this `switch` is not in the supported lowering set ({reason}); \
                     rewrite it as `if`/`else if` or use a supported switch shape \
                     (fail-closed)"
                );
                self.deny_e5506(function, &message)
            }
        }
    }

    /// Emit an admitted plan. Unreachable until a task admits a shape.
    fn emit_switch_plan(&mut self, function: &mut Function, _plan: SwitchPlan) -> EmittedValue {
        self.deny_e5506(
            function,
            "internal: a switch plan was admitted but no lowering exists for it (fail-closed)",
        )
    }
}
```

- [ ] **Step 4: Declare the module**

In `crates/kali_codegen/src/emit/mod.rs`, add `pub(crate) mod switch;` alongside the existing module declarations (match the surrounding declaration style exactly — read the file first).

- [ ] **Step 5: Add the dispatch arm**

In `crates/kali_codegen/src/emit/control_flow.rs`, in the `LirNodeKind::Branch => match node.text.as_deref()` block, add above the `_ =>` fallthrough at line 1798:

```rust
                Some("switch") => self.emit_switch(function, id, &node),
```

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p kali_cli --test switch_fail_closed`

Expected: PASS.

- [ ] **Step 7: Confirm the silent miscompile is gone**

```bash
cargo build --bin kali
cd /tmp && /workspace/target/debug/kali run t.js; echo "exit=$?"
```

Expected: nonzero exit with an `E5506` naming `switch`. It must **not** print `s20=A`.

- [ ] **Step 8: Verify no newly-red against the baseline**

```bash
cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task6.txt
grep -E "^test .* FAILED$" /tmp/r35-task6.txt | sort > /tmp/r35-task6-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task6-failed.txt
```

Expected: empty, or a small set of tests that used `switch` and relied on the accidental lowering. Only two files in the tree contain `switch(`, so a large diff here means something unintended — investigate rather than re-pin.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_codegen/
git commit -m "fix(codegen): fail closed on every switch instead of silently selecting the wrong clause

Adds emit_switch behind a text-keyed dispatch arm, with an EMPTY allowlist:
SwitchPlan::build admits nothing yet, so every switch now exits nonzero with an
honest E5506 naming the limit. Replaces a Tier-1 silent miscompile (the
discriminant was truthiness-tested, clauses 0/1 became then/else, clauses 2+
were never emitted) with an honest refusal.

Subsequent tasks admit proven shapes one at a time by adding proofs to build(),
never by removing rejections."
```

---

### Task 7: Admit numeric discriminants with all-`return` clauses

The first shape admitted, and the exact shape R-35 documents.

**Files:**
- Modify: `crates/kali_codegen/src/emit/switch.rs`
- Modify: `crates/kali_codegen/src/lower.rs` (ordinals + local reservation)
- Modify: `crates/kali_codegen/src/emitter.rs` (`switch_ordinals` field)
- Test: `crates/kali_cli/tests/switch_runtime.rs` (create)

**Interfaces:**
- Consumes: `SwitchPlan`, `SwitchClause`, `ClauseTerminator` from Task 6.
- Produces: `crate::lower::switch_preorder_ordinals(nodes, body) -> HashMap<LirNodeId, u32>` and `crate::lower::switch_disc_local_name(ordinal) -> String`; `FunctionEmitter::switch_ordinals: HashMap<LirNodeId, u32>`. Tasks 8-10 reuse all three.

**Why a reserved local:** codegen locals are reserved by name during `collect_function_locals` and resolved via `self.locals[&name]`. The discriminant must be evaluated **once** — `switch (f(x))` must call `f` once, and a chain that re-emits the discriminant per test would call it once per clause. This mirrors the existing `for_in_preorder_ordinals` / `for_in_ord_local_name` pattern exactly (`lower.rs:3013` and `:3160`), which exists for the same reason: a dedicated per-construct slot that nested emission cannot clobber.

- [ ] **Step 1: Write the failing test**

Create `crates/kali_cli/tests/switch_runtime.rs` with the `run_js` helper copied from `crates/kali_cli/tests/bitwise_operators_runtime.rs:8-25`, plus:

```rust
const S: &str = "function s(x) {\n\
                   switch (x) {\n\
                     case 10: return \"A\";\n\
                     case 20: return \"B\";\n\
                     default: return \"D\";\n\
                   }\n\
                 }\n";

// Anti-spot-check: s(10) is EXCLUDED on purpose. The pre-fix lowering returned
// "A" for every truthy discriminant, so s(10) agreed with node by coincidence
// and proves nothing. Every assertion below uses a discriminant the broken
// lowering got wrong, and the answers must vary with the input.
#[test]
fn numeric_switch_selects_the_matching_clause() {
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(20));")), "v=B\n");
}
#[test]
fn numeric_switch_falls_to_default_on_no_match() {
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(40));")), "v=D\n");
}
#[test]
fn numeric_switch_handles_a_zero_discriminant() {
    // The pre-fix lowering truthiness-tested the discriminant, so 0 took the
    // else branch and returned clause 1's "B". This is the cell that proves
    // the discriminant is compared, not tested for truth.
    assert_eq!(run_js(&format!("{S}console.log(\"v=\" + s(0));")), "v=D\n");
}
#[test]
fn numeric_switch_reaches_the_third_clause() {
    // Clauses beyond the second were never emitted at all (the Tier-1 half).
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 1: return \"one\";\n\
                   case 2: return \"two\";\n\
                   case 3: return \"three\";\n\
                   case 4: return \"four\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(3));")), "v=three\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(4));")), "v=four\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(9));")), "v=other\n");
}
#[test]
fn numeric_switch_evaluates_the_discriminant_exactly_once() {
    // If the chain re-emitted the discriminant per clause test, `hits` would
    // count once per comparison instead of once per call.
    let src = "var hits = 0;\n\
               function d(x) { hits = hits + 1; return x; }\n\
               function s(x) {\n\
                 switch (d(x)) {\n\
                   case 1: return \"one\";\n\
                   case 2: return \"two\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n\
               s(2);\n\
               console.log(\"hits=\" + hits);\n";
    assert_eq!(run_js(src), "hits=1\n");
}
#[test]
fn numeric_switch_selects_correctly_with_a_negative_case_test() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case -1: return \"neg\";\n\
                   case 1: return \"pos\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(-1));")), "v=neg\n");
}
```

Scope is a required axis, but a module-scope switch cannot use `return`, so its admitted twin arrives with Task 9's `break`. What this task can pin at module scope is the *denial*. Add to `crates/kali_cli/tests/switch_fail_closed.rs`:

```rust
#[test]
fn true_fallthrough_at_module_scope_is_fail_closed() {
    let out = run_js_expect_failure(
        "var v = \"?\";\n\
         var x = 20;\n\
         switch (x) {\n\
           case 10: v = \"A\";\n\
           case 20: v = \"B\";\n\
           default: v = \"D\";\n\
         }\n\
         console.log(\"v=\" + v);\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p kali_cli --test switch_runtime`

Expected: FAIL — every admitted-shape test fails with kali rejecting via `E5506` (Task 6's empty allowlist).

- [ ] **Step 3: Add the ordinal bookkeeping**

In `crates/kali_codegen/src/lower.rs`, add next to `for_in_preorder_ordinals` (`:3013`) and its walker (`:3261` region):

```rust
/// Pre-order, function-scoped ordinal for each `switch`-text `Branch` node.
///
/// Exists ONLY to name a dedicated per-switch i64 local
/// (`switch_disc_local_name`) holding the evaluated discriminant, so nested
/// emission inside a clause body cannot clobber it. Wholly independent of
/// `loop_preorder_ordinals` (which must never learn about switch — see its doc
/// comment on the arena-ordinal desync danger) and never threaded into
/// `ArenaTable`/`loop_arena` lookups. Consulted from exactly two call sites,
/// both inside `kali_codegen`: `collect_function_locals`, which reserves the
/// local, and `FunctionEmitter::new`, which resolves it back for `emit_switch`.
pub(crate) fn switch_preorder_ordinals(
    nodes: &[LirNode],
    body: LirNodeId,
) -> HashMap<LirNodeId, u32> {
    let mut ordinals = HashMap::new();
    let mut next = 0u32;
    switch_preorder_ordinals_walk(nodes, body, &mut next, &mut ordinals);
    ordinals
}

fn switch_preorder_ordinals_walk(
    nodes: &[LirNode],
    id: LirNodeId,
    next: &mut u32,
    ordinals: &mut HashMap<LirNodeId, u32>,
) {
    let Some(node) = nodes.get(id.0 as usize) else {
        return;
    };
    if node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("switch") {
        ordinals.insert(id, *next);
        *next += 1;
    }
    for child in &node.children {
        switch_preorder_ordinals_walk(nodes, *child, next, ordinals);
    }
}

/// Name of the dedicated i64 local holding switch `ordinal`'s evaluated
/// discriminant. The discriminant is evaluated exactly once into this slot;
/// re-emitting it per clause test would call `f` once per clause in
/// `switch (f(x))`.
pub(crate) fn switch_disc_local_name(ordinal: u32) -> String {
    format!("__switch_disc#{ordinal}")
}
```

Mirror the `for_in` walker's exact traversal shape — read `for_in_preorder_ordinals_walk` first and match it, including whether it descends into nested function bodies.

- [ ] **Step 4: Reserve the local**

In `collect_function_locals` (`lower.rs:3459-3472` region), alongside the existing `for_in_ord_local_name` reservation, add:

```rust
    let mut switch_ordinals: Vec<u32> = switch_preorder_ordinals(nodes, body_id)
        .into_values()
        .collect();
    switch_ordinals.sort_unstable();
    for ordinal in switch_ordinals {
        locals.push(switch_disc_local_name(ordinal));
    }
```

Match the surrounding code's exact variable names and iteration style — read lines 3455-3475 first.

- [ ] **Step 5: Thread the ordinals into the emitter**

In `crates/kali_codegen/src/emitter.rs`, mirroring `for_in_ordinals` at `:362`, `:549` and `:668`:

```rust
    /// Pre-order switch ordinals (see `crate::lower::switch_preorder_ordinals`),
    /// resolved once at construction so `emit_switch` can find each switch's
    /// dedicated discriminant local (`crate::lower::switch_disc_local_name`).
    pub(crate) switch_ordinals: HashMap<LirNodeId, u32>,
```

with `let switch_ordinals = crate::lower::switch_preorder_ordinals(&program.nodes, body);` next to the `for_in_ordinals` binding, and `switch_ordinals,` in the struct literal.

- [ ] **Step 6: Build the plan**

In `crates/kali_codegen/src/emit/switch.rs`, replace `switch_plan`'s unconditional rejection:

```rust
    pub(crate) fn switch_plan(&self, node: &LirNode) -> Result<SwitchPlan, String> {
        let mut children = node.children.iter().copied();
        let discriminant = children
            .next()
            .ok_or_else(|| "a switch with no discriminant".to_string())?;

        // Rule 1: the discriminant must be a PROVEN i64 scalar. Anything not
        // proven — float, boolean, object, array, unknown — is denied. Task 8
        // widens this to proven strings.
        if !self.is_provable_i64_scalar(discriminant) {
            return Err("the discriminant is not a proven integer".to_string());
        }

        let mut clauses = Vec::new();
        let mut default_seen = false;
        for case_id in children {
            let case = self.node(case_id);
            let is_default = match case.text.as_deref() {
                Some("case") => false,
                Some("default") => true,
                // Rule 3 of the allowlist is enforced by construction: an
                // untagged clause block cannot be classified, so it is denied.
                _ => return Err("an unclassifiable switch clause".to_string()),
            };
            if is_default {
                if default_seen {
                    return Err("more than one `default` clause".to_string());
                }
                default_seen = true;
            }

            // A "case" block's children are [test, stmts...]; a "default"'s
            // are [stmts...].
            let (test, stmts) = if is_default {
                (None, &case.children[..])
            } else {
                let test = *case
                    .children
                    .first()
                    .ok_or_else(|| "a `case` clause with no test".to_string())?;
                // Rule 2: the test must be a literal in the discriminant's
                // domain, including unary +/- on a numeric literal.
                if !self.is_numeric_literal_case_test(test) {
                    return Err("a `case` test that is not a numeric literal".to_string());
                }
                (Some(test), &case.children[1..])
            };

            // Rule 4: this task admits ONLY `return`-terminated clauses.
            // Empty grouping arrives in Task 10, `break` in Task 9. Anything
            // else is true fallthrough and stays denied.
            let terminator = match stmts.last() {
                Some(last) if self.is_return_statement(*last) => ClauseTerminator::Return,
                _ => {
                    return Err(
                        "a clause that does not end in `return` (true fallthrough is not \
                         in the supported lowering set)"
                            .to_string(),
                    )
                }
            };

            // Rule 5: `let`/`const` in a clause body is denied — block
            // shadowing is unmodeled (register R-10), so a case-scoped binding
            // would build on a known-broken foundation. `var` is
            // function-scoped and admitted.
            if stmts.iter().any(|s| self.declares_block_scoped_binding(*s)) {
                return Err("a `let`/`const` declaration in a clause body".to_string());
            }

            clauses.push(SwitchClause {
                test,
                body: case_id,
                terminator,
            });
        }

        if clauses.is_empty() {
            return Err("a switch with no clauses".to_string());
        }
        Ok(SwitchPlan {
            discriminant,
            clauses,
        })
    }
```

The four predicates (`is_provable_i64_scalar`, `is_numeric_literal_case_test`, `is_return_statement`, `declares_block_scoped_binding`) must be **derived from the existing repr and node queries, not hand-written duplicates**. Before writing any of them, find the existing equivalents:

```bash
grep -rn "fn is_provable\|fn scalar_repr\|Repr::I64" crates/kali_codegen/src/ | head -20
grep -rn "fn is_return\|Some(\"return\")" crates/kali_codegen/src/ | head -10
grep -rn "Some(\"let\" | \"const\")\|\"let\" | \"var\" | \"const\"" crates/kali_codegen/src/ | head -10
```

Codegen oracles and `kali_types` predicates in this repository are hand-mirrored, and the Spec 2 lesson is that a new expression kind needs arms on **both** sides or it fails open. One query, one caller.

- [ ] **Step 7: Emit the plan**

Replace `emit_switch_plan` in `emit/switch.rs`. Take `id` through so the discriminant local can be resolved:

```rust
    fn emit_switch_plan(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        plan: SwitchPlan,
    ) -> EmittedValue {
        // Evaluate the discriminant EXACTLY ONCE into this switch's dedicated
        // local. A chain that re-emitted it per clause test would call `f`
        // once per clause in `switch (f(x))`.
        let ordinal = self.switch_ordinals[&id];
        let disc_local = self.locals[&crate::lower::switch_disc_local_name(ordinal)];
        let produced = self.emit_node(function, plan.discriminant, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(disc_local));

        self.emit_clause_chain(function, disc_local, &plan.clauses, 0);

        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// Nested if/else chain over the clauses from `index` onward. The `default`
    /// clause becomes the innermost `else`.
    ///
    /// A duplicate case test needs no rule: an if/else chain is first-match-
    /// wins by construction, which is the correct JS semantics. A `default` in
    /// a non-final position needs no rule either: once true fallthrough is
    /// denied, `default`'s position carries no semantics.
    fn emit_clause_chain(
        &mut self,
        function: &mut Function,
        disc_local: u32,
        clauses: &[SwitchClause],
        index: usize,
    ) {
        let Some(clause) = clauses.get(index) else {
            return;
        };
        let Some(test) = clause.test else {
            // The default clause: run it unconditionally at this depth.
            self.emit_clause_body(function, clause);
            return;
        };

        function.instruction(&Instruction::LocalGet(disc_local));
        let produced = self.emit_node(function, test, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I64Eq);

        let frame = self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_clause_body(function, clause);
        function.instruction(&Instruction::Else);
        self.emit_clause_chain(function, disc_local, clauses, index + 1);
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::If);
        let _ = frame;
    }

    /// Emit a clause's statements, skipping a `case` clause's leading test
    /// child (a `default` clause has no test child).
    fn emit_clause_body(&mut self, function: &mut Function, clause: &SwitchClause) {
        let body = self.node(clause.body);
        let skip = usize::from(clause.test.is_some());
        let stmts: Vec<LirNodeId> = body.children.iter().copied().skip(skip).collect();
        for stmt in stmts {
            let produced = self.emit_node(function, stmt, false);
            if produced.produced {
                function.instruction(&Instruction::Drop);
            }
        }
    }
```

Update `emit_switch`'s `Ok` arm to `self.emit_switch_plan(function, id, plan)` and drop the leading underscore from its `id` parameter.

Check `push_control_frame` / `pop_control_frame`'s exact signatures at `emitter.rs:1164-1176` and match them; `emit_branch` at `control_flow.rs:3069` is the reference for how an `If`/`Else`/`End` triple is framed.

- [ ] **Step 8: Run to verify the tests pass**

Run: `cargo test -p kali_cli --test switch_runtime`

Expected: PASS, all seven tests.

- [ ] **Step 9: Confirm the denied shapes are still denied**

Run: `cargo test -p kali_cli --test switch_fail_closed`

Expected: the Task 6 test now FAILS, because that switch is admitted and correct. Update it to assert the **correct** answer, and replace its fail-closed role with a shape this task does not admit:

```rust
#[test]
fn true_fallthrough_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           var r = 0;\n\
           switch (x) {\n\
             case 1: r = 1;\n\
             case 2: r = 2;\n\
           }\n\
           return r;\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}

#[test]
fn a_non_literal_case_test_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x, y) {\n\
           switch (x) {\n\
             case y: return \"A\";\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1, 1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}

#[test]
fn a_let_declaration_in_a_clause_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case 1: { let a = 1; return \"A\" + a; }\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}
```

- [ ] **Step 10: Verify no newly-red against the baseline**

```bash
cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task7.txt
grep -E "^test .* FAILED$" /tmp/r35-task7.txt | sort > /tmp/r35-task7-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task7-failed.txt
```

Expected: empty.

- [ ] **Step 11: Commit**

```bash
git add crates/kali_codegen/ crates/kali_cli/tests/switch_runtime.rs crates/kali_cli/tests/switch_fail_closed.rs
git commit -m "feat(codegen): admit numeric switch with all-return clauses

First shape admitted by SwitchPlan: proven-i64 discriminant, numeric-literal
case tests (including unary +/-), every clause ending in return, at most one
default. Lowered as a nested if/else chain with the discriminant evaluated
exactly once into a dedicated per-switch local, mirroring the existing
for_in_preorder_ordinals / for_in_ord_local_name pattern.

Duplicate case tests and a non-final default need no rules: an if/else chain is
first-match-wins by construction, and once true fallthrough is denied a
default's position carries no semantics.

Tests exclude s(10) deliberately — the broken lowering returned \"A\" for every
truthy discriminant, so that cell agreed with node by coincidence."
```

---

### Task 8: Admit string discriminants

**Files:**
- Modify: `crates/kali_codegen/src/emit/switch.rs`
- Test: `crates/kali_cli/tests/switch_runtime.rs`

**Interfaces:**
- Consumes: Task 7's `switch_plan` and `emit_clause_chain`.
- Produces: `SwitchPlan` gains a `disc_is_string: bool` field that `emit_clause_chain` reads to pick the comparison.

- [ ] **Step 1: Write the failing test**

Add to `crates/kali_cli/tests/switch_runtime.rs`:

```rust
const SS: &str = "function s(x) {\n\
                    switch (x) {\n\
                      case \"a\": return 1;\n\
                      case \"b\": return 2;\n\
                      default: return 3;\n\
                    }\n\
                  }\n";

#[test]
fn string_switch_selects_the_matching_clause() {
    assert_eq!(run_js(&format!("{SS}console.log(\"v=\" + s(\"b\"));")), "v=2\n");
}
#[test]
fn string_switch_falls_to_default() {
    assert_eq!(run_js(&format!("{SS}console.log(\"v=\" + s(\"z\"));")), "v=3\n");
}
#[test]
fn string_switch_compares_by_content_not_handle() {
    // Two equal strings built differently must select the same clause. If the
    // comparison were handle identity rather than __streq content equality,
    // the runtime-built string would miss every case and fall to default.
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"ab\": return 1;\n\
                   default: return 3;\n\
                 }\n\
               }\n\
               var built = \"a\" + \"b\";\n\
               console.log(\"v=\" + s(built));\n";
    assert_eq!(run_js(src), "v=1\n");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test switch_runtime string`

Expected: FAIL — `E5506` "the discriminant is not a proven integer".

- [ ] **Step 3: Widen rule 1 and rule 2**

In `switch_plan`, replace the i64-only discriminant proof with a two-domain one, and carry the domain into the plan:

```rust
        // Rule 1: the discriminant must be PROVEN to be an i64 scalar or a
        // string. Float, boolean, object, array and unknown stay denied.
        let disc_is_string = if self.is_provable_i64_scalar(discriminant) {
            false
        } else if self.is_provable_string(discriminant) {
            true
        } else {
            return Err("the discriminant is not a proven integer or string".to_string());
        };
```

and make rule 2 domain-matched:

```rust
                // Rule 2: the test must be a literal in the DISCRIMINANT's
                // domain. A string case against an integer discriminant (or
                // vice versa) is denied rather than silently never matching.
                let test_ok = if disc_is_string {
                    self.is_string_literal_case_test(test)
                } else {
                    self.is_numeric_literal_case_test(test)
                };
                if !test_ok {
                    return Err(
                        "a `case` test that is not a literal in the discriminant's domain"
                            .to_string(),
                    );
                }
```

Add `disc_is_string` to the `SwitchPlan` struct and to its construction.

Find the existing string proof rather than writing one — `grep -rn "fn is_string_valued\|Repr::String" crates/kali_codegen/src/ | head -20`. `is_string_valued` is referenced in `control_flow.rs`'s `dynamic_array_read_base` doc comment as one of the string oracles; use that family.

- [ ] **Step 4: Route the comparison through the existing `===` emit**

In `emit_clause_chain`, replace the unconditional `I64Eq` with a domain-dispatched comparison, and thread `disc_is_string` through from the plan:

```rust
        if disc_is_string {
            // Reuse the existing strict-equality emit so string comparison goes
            // through __streq CONTENT equality rather than handle identity.
            // R-08's `===` half is FIXED, so switch inherits correct strict
            // equality by construction and cannot drift from it later.
            self.emit_string_content_equality(function);
        } else {
            function.instruction(&Instruction::I64Eq);
        }
```

Find the real entry point before writing this — `grep -n "streq" crates/kali_codegen/src/emit/equality.rs crates/kali_codegen/src/emit/*.rs | head`. Do **not** hand-roll a string comparison; that is precisely the hand-mirrored-oracle drift the Spec 2 lesson warns about.

- [ ] **Step 5: Run to verify the tests pass**

Run: `cargo test -p kali_cli --test switch_runtime`

Expected: PASS, all ten tests.

- [ ] **Step 6: Add the mismatched-domain denial test**

In `crates/kali_cli/tests/switch_fail_closed.rs`:

```rust
#[test]
fn a_string_case_against_a_numeric_discriminant_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           switch (x) {\n\
             case \"1\": return \"A\";\n\
             default: return \"D\";\n\
           }\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(out.contains("E5506"), "expected E5506, got: {out}");
}
```

Run: `cargo test -p kali_cli --test switch_fail_closed`

Expected: PASS.

- [ ] **Step 7: Verify no newly-red against the baseline**

```bash
cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task8.txt
grep -E "^test .* FAILED$" /tmp/r35-task8.txt | sort > /tmp/r35-task8-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task8-failed.txt
```

Expected: empty.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_codegen/ crates/kali_cli/tests/
git commit -m "feat(codegen): admit string discriminants in switch

Widens rule 1 to a proven string discriminant and makes rule 2 domain-matched,
so a string case against an integer discriminant is denied rather than silently
never matching. The comparison reuses the existing strict-equality emit, so
string cases go through __streq content equality — switch inherits R-08's fixed
=== semantics by construction instead of hand-rolling a comparison."
```

---

### Task 9: Admit `break`-terminated clauses

**Files:**
- Modify: `crates/kali_codegen/src/emit/switch.rs`
- Modify: `crates/kali_codegen/src/emitter.rs` (`LoopFrame.continue_index` becomes `Option<usize>`)
- Modify: `crates/kali_codegen/src/emit/control_flow.rs` (`emit_break_or_continue`, `emit_loop`)
- Test: `crates/kali_cli/tests/switch_runtime.rs`, `crates/kali_cli/tests/switch_fail_closed.rs`

**Interfaces:**
- Consumes: Tasks 7-8.
- Produces: `LoopFrame { break_index: usize, continue_index: Option<usize> }`. `emit_loop` passes `Some(idx)`; `emit_switch_plan` passes the enclosing frame's `continue_index` verbatim (or `None`).

**The by-construction property:** `emit_break_or_continue` (`control_flow.rs:4`) already resolves an unlabeled `break` to `loop_frames.last().break_index`, an unlabeled `continue` to `.continue_index`, and already rejects labels. So a switch frame whose `break_index` is the switch's own end block and whose `continue_index` is **inherited from the enclosing loop frame** makes "`break` binds to the switch, `continue` reaches past it to the loop" true without any precedence rule a later edit could get wrong.

- [ ] **Step 1: Write the failing tests**

Add to `crates/kali_cli/tests/switch_runtime.rs`:

```rust
#[test]
fn break_terminated_clauses_select_correctly() {
    let src = "function s(x) {\n\
                 var r = 0;\n\
                 switch (x) {\n\
                   case 10: r = 1; break;\n\
                   case 20: r = 2; break;\n\
                   default: r = 9; break;\n\
                 }\n\
                 return r;\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(20));")), "v=2\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(40));")), "v=9\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(0));")), "v=9\n");
}

#[test]
fn break_terminated_switch_works_at_module_scope() {
    let src = "var v = \"?\";\n\
               var x = 20;\n\
               switch (x) {\n\
                 case 10: v = \"A\"; break;\n\
                 case 20: v = \"B\"; break;\n\
                 default: v = \"D\"; break;\n\
               }\n\
               console.log(\"v=\" + v);\n";
    assert_eq!(run_js(src), "v=B\n");
}

#[test]
fn break_inside_a_switch_inside_a_loop_exits_the_switch_not_the_loop() {
    // If `break` bound to the LOOP instead of the switch, the loop would stop
    // after one iteration and the sum would be 1 instead of 3.
    let src = "var sum = 0;\n\
               for (var i = 0; i < 3; i = i + 1) {\n\
                 switch (i) {\n\
                   case 0: sum = sum + 1; break;\n\
                   case 1: sum = sum + 1; break;\n\
                   default: sum = sum + 1; break;\n\
                 }\n\
               }\n\
               console.log(\"sum=\" + sum);\n";
    assert_eq!(run_js(src), "sum=3\n");
}

#[test]
fn continue_inside_a_switch_inside_a_loop_continues_the_loop() {
    // `continue` must reach PAST the switch frame to the loop. If it bound to
    // the switch's break target it would merely leave the switch and `hits`
    // would count 3 instead of 1.
    let src = "var hits = 0;\n\
               for (var i = 0; i < 3; i = i + 1) {\n\
                 switch (i) {\n\
                   case 0: continue;\n\
                   case 1: continue;\n\
                   default: hits = hits + 1; break;\n\
                 }\n\
               }\n\
               console.log(\"hits=\" + hits);\n";
    assert_eq!(run_js(src), "hits=1\n");
}
```

And to `crates/kali_cli/tests/switch_fail_closed.rs`:

```rust
#[test]
fn continue_in_a_switch_with_no_enclosing_loop_is_fail_closed() {
    let out = run_js_expect_failure(
        "function s(x) {\n\
           var r = 0;\n\
           switch (x) {\n\
             case 1: continue;\n\
             default: r = 9; break;\n\
           }\n\
           return r;\n\
         }\n\
         console.log(\"v=\" + s(1));\n",
    );
    assert!(!out.is_empty(), "expected a diagnostic, got nothing");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p kali_cli --test switch_runtime break && cargo test -p kali_cli --test switch_runtime continue`

Expected: FAIL — `E5506` "a clause that does not end in `return`".

- [ ] **Step 3: Make `continue_index` optional**

In `crates/kali_codegen/src/emitter.rs:82-85`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LoopFrame {
    pub(crate) break_index: usize,
    /// `None` for a switch frame with no enclosing loop: `continue` then has
    /// no target and must fail closed rather than branch to the switch's exit.
    pub(crate) continue_index: Option<usize>,
}
```

In `emit_loop` (`control_flow.rs:265-269`), wrap its push: `continue_index: Some(continue_index)`.

In `emit_break_or_continue` (`control_flow.rs:48-53`), replace the target selection:

```rust
        let target_index = if is_continue {
            match loop_frame.continue_index {
                Some(index) => index,
                None => {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "`continue` inside a `switch` requires an enclosing loop; there is \
                         none here (fail-closed)",
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
            }
        } else {
            loop_frame.break_index
        };
```

- [ ] **Step 4: Admit `break` in the plan**

In `switch_plan`, widen rule 4:

```rust
            let terminator = match stmts.last() {
                Some(last) if self.is_return_statement(*last) => ClauseTerminator::Return,
                Some(last) if self.is_unlabeled_break_statement(*last) => ClauseTerminator::Break,
                _ => {
                    return Err(
                        "a clause that does not end in `return` or `break` (true fallthrough \
                         is not in the supported lowering set)"
                            .to_string(),
                    )
                }
            };
```

`is_unlabeled_break_statement` must accept only text exactly `"break"`, never a `"break:<label>"` prefix — labels are already rejected globally by `emit_break_or_continue` and must not be admitted here. Read `emit_break_or_continue:23-31` for the exact text encoding.

- [ ] **Step 5: Push the switch frame**

In `emit_switch_plan`, wrap the chain in a `block` and push the frame:

```rust
        // The switch's own break target. `continue_index` is INHERITED from
        // the enclosing loop frame, so an unlabeled `break` binds to this
        // switch while an unlabeled `continue` reaches past it to the loop —
        // by construction, not by a precedence rule a later edit could get
        // wrong. `None` (no enclosing loop) makes `continue` fail closed.
        let inherited_continue = self.loop_frames.last().and_then(|f| f.continue_index);
        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index: inherited_continue,
        });

        self.emit_clause_chain(function, disc_local, plan.disc_is_string, &plan.clauses, 0);

        self.loop_frames.pop();
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);
```

**A switch opens no arena frame.** Do not push an `ArenaFrame`, and do not emit an arena release here. `emit_break_or_continue`'s comment (`control_flow.rs:57-75`) records that an earlier version which released inline double-released, splicing an enclosing arena's still-live pages onto the free list. A `break` out of a switch nested in a loop must still fall through to that loop's single release.

- [ ] **Step 6: Run to verify the tests pass**

Run: `cargo test -p kali_cli --test switch_runtime && cargo test -p kali_cli --test switch_fail_closed`

Expected: PASS.

- [ ] **Step 7: Verify the arena property explicitly**

```bash
cargo build --bin kali
cd /tmp && cat > arena.js <<'EOF'
var total = 0;
for (var i = 0; i < 200; i = i + 1) {
  var o = { a: i, b: i + 1 };
  switch (i % 3) {
    case 0: total = total + o.a; break;
    case 1: total = total + o.b; break;
    default: break;
  }
}
console.log("total=" + total);
EOF
/workspace/target/debug/kali run arena.js; echo "exit=$?"
node arena.js
```

Expected: identical output, exit 0. This exercises a `break` out of a switch inside an allocating loop across 200 iterations — the shape that would surface a double-release or a corrupted free list. If the numbers diverge or the run traps, the switch is disturbing `arena_frames`; do not proceed.

- [ ] **Step 8: Verify no newly-red against the baseline**

```bash
cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task9.txt
grep -E "^test .* FAILED$" /tmp/r35-task9.txt | sort > /tmp/r35-task9-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task9-failed.txt
```

Expected: empty. `LoopFrame.continue_index` changed type, so every construction site must have been updated — a compile error here is expected during development, not a newly-red at commit time.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_codegen/ crates/kali_cli/tests/
git commit -m "feat(codegen): admit break-terminated switch clauses

Wraps the clause chain in a wasm block and pushes a LoopFrame whose break_index
is that block and whose continue_index is INHERITED from the enclosing loop
frame. An unlabeled break therefore binds to the switch and an unlabeled
continue reaches past it to the loop, by construction rather than by a
precedence rule. continue_index becomes Option; None (a switch with no
enclosing loop) makes continue fail closed.

A switch opens no arena frame, so a break out of a switch inside a loop still
falls through that loop's single release — verified across 200 allocating
iterations, not assumed."
```

---

### Task 10: Admit empty-clause grouping

**Files:**
- Modify: `crates/kali_codegen/src/emit/switch.rs`
- Test: `crates/kali_cli/tests/switch_runtime.rs`

**Interfaces:**
- Consumes: Tasks 7-9.
- Produces: `SwitchClause.tests: Vec<LirNodeId>` replaces `test: Option<LirNodeId>` — a grouped run of empty clauses collapses into one clause carrying several tests. `emit_clause_chain` ORs them.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn empty_clauses_group_onto_the_next() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 1:\n\
                   case 2: return \"low\";\n\
                   case 3: return \"mid\";\n\
                   default: return \"high\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(1));")), "v=low\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(2));")), "v=low\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(3));")), "v=mid\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(9));")), "v=high\n");
}

#[test]
fn a_grouped_body_is_emitted_once_not_per_test() {
    // If the body were duplicated per grouped test, `hits` would be 2.
    let src = "var hits = 0;\n\
               function s(x) {\n\
                 switch (x) {\n\
                   case 1:\n\
                   case 2: hits = hits + 1; break;\n\
                   default: break;\n\
                 }\n\
                 return 0;\n\
               }\n\
               s(1);\n\
               console.log(\"hits=\" + hits);\n";
    assert_eq!(run_js(src), "hits=1\n");
}

#[test]
fn empty_clauses_group_with_a_string_discriminant() {
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case \"a\":\n\
                   case \"b\": return \"ab\";\n\
                   default: return \"other\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(\"b\"));")), "v=ab\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(\"z\"));")), "v=other\n");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p kali_cli --test switch_runtime group`

Expected: FAIL — `E5506` "a clause that does not end in `return` or `break`" (an empty clause has no last statement).

- [ ] **Step 3: Change the clause shape**

In `emit/switch.rs`:

```rust
pub(crate) struct SwitchClause {
    /// The case tests that select this clause. More than one means a run of
    /// EMPTY clauses grouped onto this body (`case 1: case 2: return x;`).
    /// Empty for the `default` clause.
    pub(crate) tests: Vec<LirNodeId>,
    pub(crate) body: LirNodeId,
    pub(crate) terminator: ClauseTerminator,
}
```

In `switch_plan`, accumulate pending tests instead of rejecting an empty clause:

```rust
        let mut pending_tests: Vec<LirNodeId> = Vec::new();
        // ... inside the per-clause loop, after rule 2 has validated `test`:

            // An EMPTY non-default clause groups onto the next clause. It
            // contributes its test and no body, so no body is ever emitted
            // twice. A default clause may not be empty-grouped: it has no test
            // to contribute.
            if stmts.is_empty() && !is_default {
                pending_tests.push(test.expect("a non-default clause has a test"));
                continue;
            }

        // ... when a terminated clause is pushed:
            let mut tests = std::mem::take(&mut pending_tests);
            if let Some(test) = test {
                tests.push(test);
            }
            clauses.push(SwitchClause {
                tests,
                body: case_id,
                terminator,
            });
```

After the loop, reject a trailing group with nothing to fall onto:

```rust
        if !pending_tests.is_empty() {
            return Err("an empty trailing clause with no body to group onto".to_string());
        }
```

- [ ] **Step 4: OR the tests in the chain**

In `emit_clause_chain`, replace the single comparison with a disjunction over `clause.tests`:

```rust
        // `disc === t1 || disc === t2 || ...` — ONE body guarded by all the
        // grouped tests, so no clause body is ever emitted twice.
        for (i, test) in clause.tests.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(disc_local));
            let produced = self.emit_node(function, *test, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            if disc_is_string {
                self.emit_string_content_equality(function);
            } else {
                function.instruction(&Instruction::I64Eq);
            }
            if i > 0 {
                function.instruction(&Instruction::I32Or);
            }
        }
```

Confirm the comparison's result type before writing the `I32Or` — if `emit_string_content_equality` leaves an i64, the disjunction needs `I64Or` plus a wrap, or a `I32WrapI64` per operand. Check what `emit_branch` (`control_flow.rs:3054-3067`) does with each `ValueShape` and match it; a type mismatch here is a wasm validation error (`E4201`), not a silent bug, so it will surface immediately.

Change the default-clause detection from `clause.test.is_none()` to `clause.tests.is_empty()`, and `emit_clause_body`'s skip from `usize::from(clause.test.is_some())` to `usize::from(!clause.tests.is_empty())`.

- [ ] **Step 5: Run to verify the tests pass**

Run: `cargo test -p kali_cli --test switch_runtime`

Expected: PASS, all shapes.

- [ ] **Step 6: Verify no newly-red against the baseline**

```bash
cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-task10.txt
grep -E "^test .* FAILED$" /tmp/r35-task10.txt | sort > /tmp/r35-task10-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-task10-failed.txt
```

Expected: empty.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_codegen/ crates/kali_cli/tests/switch_runtime.rs
git commit -m "feat(codegen): admit empty-clause grouping in switch

A run of empty clauses collapses into the next terminated clause, carrying its
tests: SwitchClause.test becomes SwitchClause.tests and the chain emits
disc === t1 || disc === t2 || ... guarding ONE body. No clause body is ever
emitted twice. A trailing empty clause with nothing to group onto is denied."
```

---

### Task 11: Close out — re-masking probe, acceptance matrix, register

**Files:**
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md`
- Modify: `crates/kali_cli/tests/switch_runtime.rs` (matrix completion)

**Interfaces:**
- Consumes: Tasks 5-10.
- Produces: the merged Stage 2 record.

- [ ] **Step 1: Run the re-masking probe**

Deliberately break the lowering and confirm the suite notices. In `emit_clause_chain`, temporarily invert the comparison (e.g. emit `I64Ne` instead of `I64Eq`), rebuild, and run:

```bash
cargo build --bin kali && cargo test -p kali_cli --test switch_runtime
```

Expected: **the suite goes red.** A suite that stays green when the feature is broken is measuring nothing. Record which tests caught it. Then revert the deliberate break and confirm green again:

```bash
git checkout crates/kali_codegen/src/emit/switch.rs
cargo build --bin kali && cargo test -p kali_cli --test switch_runtime
```

- [ ] **Step 2: Complete the acceptance matrix**

Confirm every axis in the spec's §6.2 has at least one admitted cell and one denied cell in the test files, and add whichever are missing:

discriminant repr (I64 × String) × clause terminator (return × break × empty-grouping) × `default` (absent × last × mid) × **scope (module × in-function)** × nesting (bare × inside a loop, with `break`, and with `continue`).

The cells most likely still missing after Tasks 7-10: `default` absent entirely with no match (control must fall past the whole switch), `default` in a non-final position, and a `switch` with a single clause.

```rust
#[test]
fn a_switch_with_no_default_and_no_match_falls_through() {
    let src = "function s(x) {\n\
                 var r = \"none\";\n\
                 switch (x) {\n\
                   case 1: r = \"one\"; break;\n\
                   case 2: r = \"two\"; break;\n\
                 }\n\
                 return r;\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(9));")), "v=none\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(2));")), "v=two\n");
}

#[test]
fn a_default_in_a_non_final_position_still_selects_correctly() {
    // Once true fallthrough is denied, default's POSITION carries no
    // semantics — this is admitted for free by the if/else chain.
    let src = "function s(x) {\n\
                 switch (x) {\n\
                   case 1: return \"one\";\n\
                   default: return \"other\";\n\
                   case 2: return \"two\";\n\
                 }\n\
               }\n";
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(2));")), "v=two\n");
    assert_eq!(run_js(&format!("{src}console.log(\"v=\" + s(9));")), "v=other\n");
}
```

Run: `cargo test -p kali_cli --test switch_runtime && cargo test -p kali_cli --test switch_fail_closed`

Expected: PASS.

- [ ] **Step 3: Update the register**

- Mark R-35's §0.2 row and its §0.3 bullet with the resolved status and the closing commit. State precisely what is now **correct** and what remains **FAIL-CLOSED**: true fallthrough, `let`/`const` in a clause body, non-literal case tests, float/boolean discriminants, and `continue` with no enclosing loop.
- Update the §0.2 row **in the same commit** — §0 is a precedence section and a stale row outranks correct per-entry text.
- Record `throw`-as-clause-terminator as an open follow-up (deferred in the spec, §5.2).
- Record the residual denied set as named follow-up entries so a later stage can pick them up without re-deriving them.

- [ ] **Step 4: Final gate**

```bash
cd /workspace && cargo test --workspace --no-fail-fast 2>&1 | tee /tmp/r35-final.txt
grep -E "^test .* FAILED$" /tmp/r35-final.txt | sort > /tmp/r35-final-failed.txt
comm -13 docs/superpowers/followups/r35-gate-baseline.txt /tmp/r35-final-failed.txt
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: `comm` output empty; fmt clean; clippy clean (CI runs with `-D warnings`).

- [ ] **Step 5: Commit and open the Stage 2 PR**

```bash
git add docs/superpowers/followups/ crates/kali_cli/tests/
git commit -m "docs(register): close R-35's codegen half; record the residual denied set

Records what switch now lowers correctly and what remains fail-closed (true
fallthrough, let/const in a clause body, non-literal case tests, float/boolean
discriminants, continue with no enclosing loop), plus throw-as-terminator as a
deferred follow-up. §0.2 precedence rows updated in this same commit."

gh pr create --title "R-35 Stage 2 — allowlisted switch lowering" --body "$(cat <<'EOF'
Closes the codegen half of R-35.

`SwitchStmt` allocated with no text, so it reached codegen as a `Branch` with
`text: None` and fell into the generic arm — which is the `if` lowering. The
discriminant was truthiness-tested, clauses 0/1 became then/else, and clauses
2+ were never emitted at all.

- Tag `SwitchStmt` and its case blocks (behavior-neutral on its own).
- Add `emit_switch` with an EMPTY allowlist first, so every intermediate commit
  on this branch fails closed rather than miscompiling silently.
- Admit, one task at a time: numeric discriminant + all-return clauses; string
  discriminants via the existing `===`/`__streq` emit; `break`-terminated
  clauses; empty-clause grouping.
- Denied and honestly `E5506`'d: true fallthrough, `let`/`const` in a clause
  body, non-literal case tests, float/boolean discriminants, `continue` with no
  enclosing loop.

`break` binds to the switch and `continue` reaches past it to the enclosing
loop by construction: the switch frame's `continue_index` IS the enclosing
loop's. The discriminant is evaluated exactly once into a dedicated per-switch
local. A switch opens no arena frame, verified across 200 allocating iterations.

Gate: zero newly-red against `docs/superpowers/followups/r35-gate-baseline.txt`.
Re-masking probe run: inverting the clause comparison turns the suite red.
EOF
)"
```

**Whole-stage review gate:** before merging, run an adversarial review of the entire Stage 2 diff. The recurring shape this catches is a **store site or value sink** the per-task view never enumerated — so the reviewer's brief is explicitly: enumerate every position a clause body can write to or escape from, not only the clause-selection logic.

---

## Self-Review

**Spec coverage.** Every section of the design maps to a task: §4.1 items 1-3 → Tasks 2-3; §4.1 item 4 → Task 4; §4.2 inventory → Task 4 Step 7; §4.3 tests → Task 2 Step 1; §5.1 tagging → Task 5; §5.2 rules 1-5 → Tasks 6-10 (rule 1 in 7/8, rule 2 in 7/8, rule 3 in 7, rule 4 in 7/9/10, rule 5 in 7); §5.3 lowering → Tasks 7-10; §5.4 break frame → Task 9; §5.5 arena verification → Task 9 Step 7; §5.6 types side → Task 7 Step 6 and Task 8 Step 3; §6.1 gate → Task 1 and every task's penultimate step; §6.2 matrix → Task 11 Step 2; §6.3 instrument rules → Global Constraints; §6.4 anti-spot-check → Task 7 Step 1; §6.5 re-masking → Task 11 Step 1; §6.6 review → the two whole-stage review gates; §7 docs debt items 1-6 → Task 4 Steps 4-8 and Task 11 Step 3.

**Known gap, deliberate:** the spec's §5.2 note that rules 4 and 5 are *provisional* pending Task 4's re-derivation. Tasks 7-10 encode them as written. If Task 4's matrix contradicts either — for example if `let` in a clause body turns out to fail closed on its own, or if `throw` is a clean terminator — the executing agent must revise Tasks 7 and 9 before implementing them rather than following this plan verbatim. This is called out here because it is the one place the plan cannot be fully determined in advance.

**Placeholder scan:** no TBD/TODO; every code step carries actual code. Three steps direct the implementer to `grep` for an existing predicate rather than naming it (Task 7 Step 6, Task 8 Steps 3-4, Task 10 Step 4). That is deliberate and is the *opposite* of a placeholder: naming a guessed symbol would invite a hand-mirrored duplicate of an existing oracle, which is the specific failure mode the Spec 2 lesson records. Each of those steps states what to search for and what to do with the result.

**Type consistency:** `SwitchPlan` gains `disc_is_string` in Task 8 and `SwitchClause.test: Option<LirNodeId>` becomes `tests: Vec<LirNodeId>` in Task 10 — both are called out in the affected tasks' Interfaces blocks, and Task 10 Step 4 lists the two call sites that must change with it (`clause.test.is_none()` and the `emit_clause_body` skip). `LoopFrame.continue_index` becomes `Option<usize>` in Task 9, with `emit_loop`'s construction site named. `emit_switch_plan` gains an `id` parameter in Task 7 Step 7, called out in that step.
