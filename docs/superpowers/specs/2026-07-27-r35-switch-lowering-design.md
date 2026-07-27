# R-35 — `switch` clause selection: parser containment, then allowlisted lowering

Date: 2026-07-27
Baseline: `main` at `2b6a51908` (clean tree)
Oracle: `node v26.5.0`
Binary: `./target/debug/kali`, rebuilt at `2b6a51908` before every measurement below

Register entry: **R-35** (`docs/superpowers/followups/kali-silent-miscompile-register.md` §0.3),
named there as the current headline silent miscompile — the highest-blast-radius silent defect
on the binary now that the Group-1 evidence-corrupting entries (R-01, R-04, R-07) are resolved.

---

## 1. What was measured

Everything in this section was run in this session on a binary built at `2b6a51908`, one
`console.log` argument per call built by literal-rooted concatenation, exit status captured
without a pipe. Probe files: `scratchpad/r35/`.

### 1.1 The recorded defect reproduces exactly

```js
function s(x) {
  switch (x) {
    case 10: return "A";
    case 20: return "B";
    default: return "D";
  }
}
```

| call | kali | node |
|---|---|---|
| `s(10)` | `A` | `A` (coincidence) |
| `s(20)` | **`A`** | `B` |
| `s(40)` | **`A`** | `D` |
| `s(0)`  | **`B`** | `D` |

Exit 0, no diagnostic, in every row.

### 1.2 The recorded boundary is wrong

§0.3 states the silent window is "exactly all-return/no-break/no-local", on the strength of
"a `break` in a case → E5506; a local read in a case → E3100". Measured:

| shape | §0.3 says | measured at `2b6a51908` |
|---|---|---|
| `break` in a clause | E5506 | E5506 — but see §1.3 and §2.1: this is the parser leak's shadow, not a switch rule |
| assignment to an **enclosing** binding from a clause | "local read → E3100" | E3100 — same shadow |
| `var` declared **and** read **inside** one clause | (implied fail-loud) | **SILENT** — `A111`, node `D999` |
| string discriminant | not recorded | **SILENT** — `1`, node `2` |
| `console.log` inside clauses | not recorded | **SILENT, wrong side effect** — prints `ten`, node `twenty` |
| third clause and `default` | not recorded | **never emitted at all** |
| 2 clauses, no `default`, `return` after the switch | not recorded | **no output whatsoever**, exit 0 |

Two consequences. R-35 is **Tier 1** (silently drops code), not only Tier 2 (silently wrong
value) — clauses beyond the second are dropped wholesale. And it is not value-only: it selects
the wrong clause's *side effects*, which is a silent control-flow flip.

### 1.3 There are two defects here, not one

The E3100 and E5506 results above are not a boundary of the clause-selection defect. They are
symptoms of a separate, more severe parser defect. Decisive probe — `s` is **never called**:

```js
var g = 0;
function s(x) { switch (x) { case 1: g = 1; } g = 99; }
console.log("g=" + g);
```

kali prints `g=99`; node prints `g=0`. The statement after the switch **escaped the function
body and executed at module load**. A second probe: a whole `function t() { return "T"; }`
declared after a switch-containing function disappears — kali prints nothing where node prints
`t=T`, because the leaked `return` terminated the module.

So the recorded `break` → E5506 was a *leaked* `break` evaluated at module scope with no loop
frame, and the recorded E3100 was a *leaked* identifier read resolved against module scope. Both
are artifacts. The measured silent window is materially wider than recorded, and the whole
boundary must be re-derived once the leak is closed (§4.1, item 4).

---

## 2. Mechanism — both traced in source, neither inferred

### 2.1 Defect A — the parser never consumes the switch's closing brace (severity: Tier 1)

`crates/kali_parser/src/statement.rs:503` `parse_switch_statement`. Its clause loop breaks on
`RightBrace` by *inspecting* it:

```rust
if self.stream.current_kind() == Some(&TokenType::RightBrace) {
    break;
}
```

and then returns the `SwitchStatement` without advancing past it. The enclosing block parser
sees that `}` as its own terminator, so **every statement after a `switch` inside a function
body is silently reparented to module scope.**

Every other block-closing site in the parser consumes its closer:

| site | closes via |
|---|---|
| `statement.rs:179` `parse_block_statement` | `self.stream.accept(TokenType::RightBrace)` |
| `declaration.rs:286` `parse_class_body` | `self.stream.accept(TokenType::RightBrace)` |
| `declaration.rs:547` `parse_arrow_function_body_expression` | `self.stream.accept(TokenType::RightBrace)` |

`parse_switch_statement` is the **unique** site in the parser with this bug. The sibling sweep
found no second instance.

The mechanism underneath it is broader: all six required-token positions in that function —
`switch`, `(`, `)`, `{`, each clause's `:`, and the closing `}` — are blind
`let _ = self.stream.advance()` calls or a discarded `accept` bool, so each silently accepts
whatever token is present. The opening `{` in particular is consumed only by the loop's
"unknown token, skip it" fallthrough arm. This is cluster **G1** (parser fail-open recovery),
the same family as R-01.

Supporting fact about the evidence base: `e2::EXPECTED_TOKEN` (E2000) and `e2::UNEXPECTED_TOKEN`
(E2001) are declared at `crates/kali_error/src/_error_codes.rs:23-24` and **emitted nowhere in
the compiler**. The parser has never once reported a required token as missing.

### 2.2 Defect B — `SwitchStmt` carries no text, so it lowers as an `if`

1. `crates/kali_hir/src/lowering/statement.rs:84` allocs `HirNodeKind::SwitchStmt` via
   `alloc(..., None)` — **no text** — with children `[discriminant, caseBlock0, caseBlock1, …]`.
2. `crates/kali_mir/src/lower.rs:92` maps `SwitchStmt` to `MirNodeKind::ControlFlow`, which
   reaches codegen as `LirNodeKind::Branch` with `text: None`.
3. `crates/kali_codegen/src/emit/control_flow.rs:1760` dispatches `Branch` on `node.text`.
   With no text it falls to `:1798`, `_ => self.emit_branch(...)` — **which is the `if`
   lowering** (`:2998`).

`emit_branch` reads `cond = children[0]`, `then = children[1]`, `else = children[2]`. For a
switch that is: truthiness-test the *discriminant*, run clause 0 if truthy, clause 1 if falsy,
and never emit `children[3..]` at all. That predicts §1.1's four rows exactly, including
`s(0)` → `B` and the dropped `default`.

This is verbatim the class the codebase names for itself at
`crates/kali_hir/src/lowering/statement.rs:104`: *"a None-text Branch falls into the generic
arm, which is how throw was a silent no-op."* The same hole, still open, for a second statement
kind.

A related structural gap: a case block's HIR children are `[testExpr?, stmts…]`, so a `default`
clause is positionally indistinguishable from a `case` clause whose first statement happens to
be an expression statement. Nothing downstream can tell them apart today.

### 2.3 Blast radius in the existing suite

Two files in the whole tree contain `switch(`: `crates/kali_parser/tests/parser_integration.rs`
(one parser test) and `crates/kali_types/src/repr_infer_tests.rs` (one repr test). There is
essentially no fixture churn to budget for, and a large census diff is itself a signal that
something unintended happened.

---

## 3. Decisions

1. **Deliverable**: real lowering for an allowlisted subset, with every shape outside the
   allowlist routed to honest `E5506`. Not a blanket refusal, and not full JS `switch`
   semantics.
2. **Sequencing**: two stages, parser first, each landing its own PR with its own whole-stage
   adversarial review.
3. **v1 clause forms**: `return`-terminated clauses, `break`-terminated clauses, and empty
   clauses that group onto the next. True fallthrough is denied.

### 3.1 Why parser-first is not negotiable

Every fixture written for defect B today is silently a *different program* past the switch's
closing brace. A green `emit_switch` test authored before Stage 1 proves nothing about the shape
its author believed they were testing. This is exactly §4 warning 6 (the R-01 mechanism): the
observed behavior is the behavior of a prefix — or here, of a program whose remainder ran
somewhere else entirely. Defect A is evidence-corrupting; it is fixed and its consequences
re-measured before defect B is touched.

### 3.2 Rejected alternatives

- **Codegen-first.** Rejected for §3.1.
- **One combined change.** Rejected: it merges a small, high-confidence, one-site parser fix
  with a new capability needing its own allowlist review. Seven consecutive stages in this
  repository have had a CRITICAL that only a whole-stage review caught; separable units review
  better.
- **Fail closed on `switch` entirely.** Cheapest honest option and genuinely defensible, but it
  takes kali from "sometimes accidentally right" to "compiles zero switch statements", and the
  allowlisted lowering is not much larger once the parser stage has landed.
- **Full JS semantics (fallthrough, case-scoped `let`).** Fallthrough requires a labelled block
  ladder rather than an if-else chain — a different lowering — and case-scoped `let`/`const`
  would build on the block-scope model R-10 shows is unmodeled.

---

## 4. Stage 1 — parser containment

**Goal**: a `switch` may still compile to the wrong clause, but it may not move code out of its
function.

### 4.1 Work

1. **Consume the closer.** Change `parse_switch_statement`'s clause-loop break to consume the
   `RightBrace`, matching its three siblings. This alone closes the reparenting.
2. **Add the parser's missing `expect` helper.** `expect(kind) -> bool`: consume and return
   true on match; push `E2000 "expected <token>"` and return false on mismatch. The parser has
   only `accept -> bool` today, which is why every required-token position is a blind advance.
3. **Route `parse_switch_statement`'s six required-token positions through `expect`** —
   `switch`, `(`, `)`, `{`, each clause's `:`, and `}`. The clause loop keeps its existing
   skip-arm for genuinely unknown tokens, so recovery behavior is unchanged.
4. **Re-derive the R-35 boundary** on the fixed parser and record it. §1.2's table is void as a
   statement about `switch`; the Stage 2 allowlist is designed against the re-derived matrix,
   not against the numbers in this document.

### 4.2 Explicitly out of scope

`crates/kali_parser/src/statement.rs` alone holds 28 blind `let _ = self.stream.advance();`
sites. Converting them all is a parser-wide fail-open sweep with an unbounded test-census cost
and is its own project. Stage 1 introduces the helper and applies it **inside
`parse_switch_statement` only**, and files a counted `file:line` inventory of the remaining
sites in the register as follow-up work. An honest inventory is preferable to a parser-wide
regression surface smuggled into a switch fix.

### 4.3 Tests

Behavioral pins that assert the program *runs and prints*, not AST-shape assertions — per the
Stage 5 lesson that AST unit tests give false confidence:

- the `g=99` leak (a statement after a switch executing though the function is never called),
- a function declaration after a switch-containing function surviving,
- a call's output not vanishing (`return` after a switch inside the callee),
- one parser-level assertion that the statement following a switch is a sibling of the switch
  within the function body.

---

## 5. Stage 2 — allowlisted switch lowering

### 5.1 Tagging

- `statement.rs:84`: alloc `SwitchStmt` with text `"switch"` so the LIR `Branch` stops falling
  into the generic `if` arm — the identical fix already applied to `throw`.
- Tag each case block `"case"` / `"default"`, closing the positional ambiguity in §2.2.
- Add `Some("switch") => self.emit_switch(function, id, &node)` to the Branch dispatch at
  `control_flow.rs:1760`.

### 5.2 Admittance — positive evidence only

`emit_switch` builds a `SwitchPlan` and denies with `E5506` if it cannot construct one. There is
no denylist of shapes anywhere in the design. This mirrors R-11's `resolve_identifier_kind`
close-by-construction and G6's allowlist-at-resolve, and it is the direct application of this
repository's most-repeated lesson: a denylist of shapes leaks forever; only an allowlist at the
choke point closes a class.

| # | admitted | denied |
|---|---|---|
| 1 | discriminant proven `Repr::I64` or `Repr::String` | float, boolean, object, array, unknown |
| 2 | every case test a literal in the discriminant's domain, including unary `+`/`-` on a numeric literal (R-06's precedent) | identifiers, calls, any computed test |
| 3 | zero or one `default` | two or more |
| 4 | every non-empty clause ends in an unlabeled `return` or `break`; empty clauses group onto the next | any non-empty clause with no terminator — i.e. true fallthrough |
| 5 | `var` declarations in a clause body | `let` / `const` in a clause body |

Notes on what is deliberately *not* a rule:

- **Duplicate case tests are admitted.** An if-else chain yields first-match-wins by
  construction, which is the correct JS semantics. No rule needed.
- **A `default` in a non-final position is admitted.** Once fallthrough is denied, `default`'s
  position carries no semantics. No rule needed.

Notes on deliberate conservatism:

- Rule 4 is **syntactic**: a clause ending in an `if` whose branches both return is denied. The
  allowlist proves a terminator, it does not prove reachability.
- Rule 5 exists because R-10 shows block shadowing is unmodeled. Case-scoped `let` would build
  on a known-broken foundation.
- `throw` as a clause terminator is **deferred**, not denied on principle: it terminates in
  principle, but kali's `throw` lowering is its own lane and admitting it needs its own
  measurement. Inventoried as a follow-up.

**Rules 4 and 5 are provisional.** They are written against a boundary measured *through* the
parser leak. They are finalized against Stage 1's re-derived matrix (§4.1, item 4).

### 5.3 Lowering — an if-else chain, not `br_table`

- The discriminant is evaluated **once** into a fresh local. `switch (f(x))` must call `f`
  exactly once; a naive chain re-emits the discriminant per test.
- Each non-default clause becomes `disc_local === <literal>` guarding its body, chained as
  nested else-branches, with `default` (if present) as the innermost else.
- The comparison **reuses the existing `===` emit** rather than a hand-rolled one, so string
  discriminants go through `__streq` content equality. R-08's `===` half is FIXED, so switch
  inherits correct strict equality by construction and cannot drift from it later.
- A run of empty clauses lowers as `disc === t1 || disc === t2 || …` guarding one body, so no
  clause body is ever emitted twice.

### 5.4 The `break` frame

Wrap the chain in a wasm `block` and, for the duration of the clause bodies, push
`LoopFrame { break_index: <that block>, continue_index: <inherited from the enclosing loop
frame> }` onto `loop_frames`.

`emit_break_or_continue` (`control_flow.rs:4`) already resolves an unlabeled `break` to
`loop_frames.last().break_index` and an unlabeled `continue` to `.continue_index`, and already
rejects labels. So with the frame above:

- an unlabeled `break` in a clause targets the switch's end block, and
- an unlabeled `continue` in a clause targets the **enclosing loop's** continue target,

not because of a precedence rule a later edit could get wrong, but because the switch frame *is*
the enclosing loop's continue target. `continue_index` becomes an `Option`; `None` — a switch
with no enclosing loop — makes `continue` fail closed through the existing E5506 path.

### 5.5 Arena interaction — verified, not assumed

`emit_break_or_continue`'s comment records that a `break`'s `Br` deliberately emits no inline
arena release, because it lands exactly where `emit_loop` already emits its unconditional
normal-exit release — and that an earlier version which *did* release inline double-released,
splicing an enclosing arena's still-live pages onto the free list.

A switch opens no arena frame. Two properties therefore require explicit verification rather
than assumption: pushing the switch's block must leave `arena_frames` untouched, and the
switch's break target must sit **inside** any enclosing loop's arena scope so a `break` out of a
switch inside a loop still falls through that loop's single release.

### 5.6 Types side

The discriminant's repr is taken from the **existing** repr query, not a newly hand-written one.
`repr_infer.rs:568` already walks switch case bodies for shadow detection. The Spec 2 lesson is
that codegen oracles and `kali_types` predicates are hand-mirrored and a new expression kind
needs arms on both sides or it fails open — so: one query, one caller.

---

## 6. Verification

### 6.1 Gate

`cargo test --workspace --no-fail-fast`, diffed against a **`main` worktree built from the same
commit**, never against a mid-branch baseline. Per stage the bar is **zero newly-red**; a red
that is also red on `main` is baseline, not a regression.

`--no-fail-fast` is required because partial enumeration has produced false drain counts before.
Parallel `cargo test` output interleaving can drop `FAILED` lines — it can under-count drain,
but it cannot fabricate a newly-red.

**No baseline number is quoted in this document.** The workspace gate was not run in this
session, and a number without a named baseline is not a measurement. Establishing the current
newly-red baseline is Stage 1's first task.

### 6.2 Acceptance matrix

Every admitted cell runs under both `kali run` and `node` and must match byte-for-byte. Every
denied cell must exit nonzero with `E5506` and a message naming the actual limit.

Axes: discriminant repr (I64 × String) × clause terminator (return × break × empty-grouping) ×
`default` (absent × last × mid) × **scope (module × in-function)** × nesting (bare × inside a
loop, with `break`, and with `continue`).

Scope is a required axis. Module scope and function scope are different programs in kali, and
§1.3 already measured module scope behaving differently from in-function for this very defect.

### 6.3 Instrument rules

From §4 of the register, plus one specific to this work:

1. One argument per `console.log`, built by literal-rooted concatenation. Multi-argument logging
   was itself a defect (R-04) and multi-arg probes in this repository are unreliable by
   construction.
2. Capture kali's exit status without a pipe, or via `PIPESTATUS` / `set -o pipefail`.
   `cmd | tail` makes `$?` the status of `tail`, erasing the single signal that distinguishes
   "fails closed" from "silently miscompiles".
3. No default parameters anywhere in a fixture (R-01).
4. **No probe may depend on a statement placed after the `switch`** until Stage 1 has landed.
   That is precisely the corrupted position.

### 6.4 Anti-spot-check discipline

For each admitted cell, vary the discriminant and assert the *answer varies with it*. A single
agreeing data point is not evidence: §4 warning 7 records `String(42).length` printing `2` and
matching node for every input, and the R-06 stage lost a round to a `+"3"→3` spot check that
masked a `+"hi"→617` leak.

Concretely, **no clause-selection test may use a discriminant for which the current buggy
lowering happens to be right** — today's `s(10)` is exactly such a value.

### 6.5 Re-masking probe

After Stage 2, deliberately break one clause's comparison and confirm the acceptance matrix goes
red. A suite that stays green when the feature is broken is measuring nothing.

### 6.6 Review protocol

Each stage gets a whole-stage adversarial review, not only per-task reviews. Seven consecutive
stages have had a CRITICAL that only the whole-stage pass could see, and the recurring shape is
a **store site or value sink** the per-task view never enumerated. The Stage 2 analogue is
explicit: enumerate every position a clause body can write to or escape from, not only the
clause-selection logic.

---

## 7. Documentation debt — in the same commits, not as follow-ups

1. **Rewrite §0.3's R-35 bullet.** It is false as written: its recorded boundary is the parser
   leak's shadow, and it omits that clauses beyond the second are dropped entirely (Tier 1), that
   side effects are flipped, and that string discriminants are affected.
2. **Add a new numbered register entry for the parser leak** (defect A) in cluster G1. It is not
   R-35 — different layer, different blast radius, and it outranks R-35 in severity.
3. **Update §0.2's status rows for both, in the same commit as the status change.** §0 is a
   precedence section: a stale §0 row outranks correct per-entry text. That is the exact trap
   PRs #28 and #29 existed to clean up.
4. **File the counted `file:line` inventory** of the parser's remaining blind-`advance()` sites
   (§4.2) as explicit follow-up work.
5. **Record that `E2000`/`E2001` are now emitted.** "The parser has never reported a missing
   required token" is a standing fact about this repository's evidence base, and Stage 1 changes
   it.
6. **Record the deferred `throw`-as-terminator question** (§5.2) as a follow-up.

---

## 8. Success criteria

**Stage 1**
- No statement following a `switch` inside a function body escapes that function; the three
  measured leak shapes are pinned as behavioral tests.
- A malformed switch header or clause separator produces `E2000` rather than being silently
  accepted.
- Workspace gate: zero newly-red against a same-commit `main` worktree.
- The R-35 boundary is re-derived on the fixed parser and recorded.

**Stage 2**
- Every cell of the §6.2 acceptance matrix that the allowlist admits matches node byte-for-byte.
- Every cell it denies exits nonzero with `E5506` naming the real limit. No admitted-but-wrong
  cell, and no silently-accepted denied cell.
- `break` inside a switch inside a loop exits the switch; `continue` inside a switch inside a
  loop continues the loop; `continue` inside a switch with no enclosing loop fails closed.
- The arena properties in §5.5 are verified, not assumed.
- Workspace gate: zero newly-red against a same-commit `main` worktree.
- The register updates in §7 land with the code, not after it.

---

## 9. Assumptions and open questions

- **Assumption**: the workspace gate is currently at a zero-newly-red baseline (the state
  recorded after PR #23). Not re-measured in this session; Stage 1's first task establishes the
  real number and this document's plan does not depend on the assumed one.
- **Provisional**: allowlist rules 4 and 5 (§5.2), pending Stage 1's re-derived boundary.
- **Deferred**: `throw` as an admitted clause terminator (§5.2).
- **Out of scope, inventoried**: the parser-wide blind-`advance()` sweep (§4.2); case-scoped
  `let`/`const`, which is blocked on R-10; true fallthrough, which needs a different lowering.
