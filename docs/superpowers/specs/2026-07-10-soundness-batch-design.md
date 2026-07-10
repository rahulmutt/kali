# Soundness Batch 1: miscompile closures + optional chaining

**Date:** 2026-07-10
**Status:** Approved design, pre-implementation
**Structure:** One design doc, two PRs. PR-A ships nine mechanical/medium
closures; PR-B ships optional chaining alone and branches from merged PR-A.

## 1. Context and goal

The CLBG 6-fixture series completed with fasta N=25M (PR #15, main
`40c7cb71e`). Before adding new surface, this batch pays down the known
wrong-output inventory accumulated across the baseline-green work
(2026-07-03) and the Spec 7 follow-ups.

The fix bar throughout is the standing convention: **reject-don't-miscompile**.
Every item either gets correct JS semantics (verified differentially against
node) or a clean compile-time diagnostic. No silent fall-through survives the
batch.

All ten items were re-verified present on main before this design was written;
the file:line evidence below is from that sweep, not from memory.

## 2. Scope

In scope (10 items):

| # | Item | Class | PR |
|---|------|-------|----|
| 1 | `throw` is a codegen no-op | miscompile | A |
| 2 | Keyword keys in object literals silently dropped | miscompile | A |
| 3 | `obj?.type` compiles as `obj` (silent 0) | miscompile | B |
| 4 | `last = undefined` on a for-in key alias stores 0, not −1 | miscompile | A |
| 5 | Multi-arg `Array(1,2,3)` yields scalar 0 | miscompile | A |
| 6 | Mixed BigInt arithmetic (`3n/2`) floats; node throws TypeError | miscompile | A |
| 7 | `const if = 1` accepted | parser laxity | A |
| 8 | `1n/0n` traps with a generic message; node throws RangeError | diagnostics | A |
| 9 | Block-bodied arrow outside declarator position misparses to E3100 | false-reject UX | A |
| 10 | Task-2 taint misses no-initializer form; two unwritten Spec 7 pins | fail-open + pins | A |

Out of scope: try/catch (stays rejected), full block-bodied-arrow
generalization to all expression positions, `?.` chains / `?.[]` / `?.()`
semantics (rejected cleanly, not implemented), any new nullable value
representation, performance work.

## 3. PR-A: the nine closures

### 3.1 `throw` → print-then-trap (item 1)

Today `ThrowStmt` lowers HIR→MIR `ControlFlow`→LIR `Branch` with `text = None`
(`kali_hir/src/lowering/statement.rs:102`, `kali_mir/src/lower.rs:95`,
`kali_lir/src/lower.rs:64`) and falls into codegen's generic branch arm
(`kali_codegen/src/emit/control_flow.rs:966`), which implements no throw
semantics — execution silently continues.

Fix: carry text `"throw"` from HIR lowering so the node survives to LIR;
codegen's text-keyed `Branch` dispatch (`control_flow.rs:951`) gains a `throw`
arm that lowers to a **print-then-trap helper**:

- If the thrown argument is a string literal or `new Error("<string lit>")`,
  print `Uncaught Error: <lit>` via the existing `console_error` host import
  (`kali_runtime/src/host/imports_default.rs:44` — already imported, so the
  four hand-mirrored JS import lists do not change), then emit
  `Instruction::Unreachable`.
- Any other argument prints a generic `Uncaught exception` and traps.

Observable behavior matches an uncaught JS throw: message on stderr, nonzero
exit. Never-taken throws (fixture self-checks) keep compiling, so the green
suite is not regressed. The helper is shared with item 8.

### 3.2 Keyword object-literal keys (item 2)

`kali_parser/src/expression/object.rs:23-73` matches only
`Identifier`/`StringLiteral`/`NumericLiteral`/`LeftBracket` keys; a reserved
word like `type` lexes as `TokenType::Type` and hits the `_ =>` arm, which
`advance(); continue;` — the whole property is silently discarded.

Fix: reuse `is_property_name_token` (`kali_parser/src/expression/call.rs:139`
— the helper the member-access fix already uses), so `{type: 3}` parses with
the same key set as `obj.type`. The `_ =>` arm becomes a **parse error**: any
property form the parser does not recognize is a hard reject, closing the
entire silent-drop class (not just keywords).

### 3.3 Unified null/undefined recognizer; `= undefined` stores −1 (item 4)

`is_null_or_undefined_literal` (`kali_codegen/src/emit/literal.rs:163`)
requires a `Literal` node, but bare `undefined` parses as an *Identifier*
(`kali_parser/src/expression/primary.rs:57`) and lowers to a `Value` node, so
the −1 sentinel stores at `literal.rs:503/583/598` and
`control_flow.rs:855-864` are skipped and the generic value emit maps
`"undefined"` to `I64Const(0)` (`control_flow.rs:1163`) — wrong truthiness on
the for-in key alias.

Fix: introduce one shared recognizer `is_null_or_undefined_expr` covering both
the `null` Literal node and the identifier-`undefined` Value node, and use it
at all four store sites. Because the Spec 7 `??= undefined` reject
(`61dfb75c9`) existed *only* because the recognizer twins disagreed on the
identifier form, the unified recognizer also lifts that reject: `??=
undefined` is admitted with sentinel semantics, and its E-code pin flips to a
behavior pin. The types-side twin uses the same shared predicate so the pair
cannot re-diverge.

### 3.4 Multi-arg `Array(1,2,3)` desugars to an array literal (item 5)

Classic twin mismatch: types' `declarator_registers_runtime_array`
(`kali_types/src/resolve/expression.rs:604`) accepts any arity, while
codegen's `resolve_array_alloc_call` (`kali_codegen/src/emit/call.rs:2331`)
bails at >1 argument — the binding registers as an array but the allocation
never lowers; result is scalar 0.

Fix: desugar `Array(e1, …, en)` for n ≥ 2 into an `ArrayExpression` at **HIR
lowering** — exact JS semantics (multi-arg `Array` ≡ array literal), landing
*before* both twins so every existing array-literal gate applies fail-closed
by construction. As defense in depth, the types-side recognizer is narrowed to
n ≤ 1 so the two sides agree on what an `Array(...)` *call* is.

### 3.5 Mixed BigInt arithmetic rejects (item 6)

`kali_codegen/src/emit/operators.rs:1319` floats `/` unless *both* operands
are BigInt-literal-valued, so `3n/2` emits `F64Div`; the same hole exists for
the other arithmetic operators (`3n*2`, `3n+2`). Node throws TypeError.

Fix: types-side reject with a **new E-code** (see §5) for any binary
arithmetic operator with exactly one BigInt-valued operand — the check covers
the full arithmetic operator set, not just `/`. Compile-time reject of a
program node itself refuses at runtime.

### 3.6 Reserved-word binding names reject (item 7)

`parse_variable_declarator` (`kali_parser/src/statement.rs:114`) takes
whatever token follows and uses its `.value` as the binding name. Fix: the
name token must be `TokenType::Identifier` or a member of an explicit
contextual-keyword allowlist (lexer-keywordized names that are legal JS
bindings, e.g. `type`, `of`); true reserved words (`if`, `for`, `const`, …)
are a parse error. `const type = 1` keeps working; `const if = 1` rejects.

### 3.7 `1n/0n` gets a node-shaped message (item 8)

The all-BigInt `/` path emits `I64DivS` with no zero check
(`operators.rs:1384`); division by zero traps as a generic
`UnreachableCodeReached`. Fix: emit an explicit zero test on the divisor that
routes to the §3.1 print-then-trap helper with `RangeError: Division by
zero`. Behavior stays "abort"; the message becomes what node prints.

### 3.8 Block-bodied arrow: targeted parse error (item 9)

Block-bodied arrows are only parsed in declarator-init position
(`kali_parser/src/declaration.rs:298`, invoked from `statement.rs:128`); the
general arrow path silently bails when `{` follows `=>`
(`declaration.rs:272-275`), so the params reparse as a parenthesized
expression against the outer scope and the user sees a baffling E3100
"undefined identifier".

Fix: the bail becomes a **targeted parse error** naming the real limitation
("block-bodied arrow functions are only supported as a declarator
initializer"). Safe because `=>` followed by `{` has no other legal parse.
Full generalization stays deferred.

### 3.9 Taint seed widening + two unwritten pins (item 10)

The Explore sweep falsified the "verified rejecting" note from Spec 7:
`object_initialized_bindings` (`kali_common/src/repr.rs:97`) is seeded only
from declarator-RHS shapes (`kali_types/src/repr_infer.rs:928`), so
`var o; o = {x:1}; o += 1` — the no-initializer form — escapes the taint.

Fix: seed the taint from later assignments with object-literal RHS as well,
so the existing compound/update gate
(`kali_types/src/resolve/expression.rs:2026`) fires for both forms. Pins
added: (a) the later-reassignment compound reject for both the declarator and
no-initializer forms; (b) the `string_arena_loops` poisoning twin pin next to
the existing family in `kali_mir/src/analysis/arena_gate_tests.rs` (the
`poisoned_function_retains_no_arena_string_sites` pattern at line 916).

## 4. PR-B: optional chaining, full short-circuit

### 4.1 Root cause

The property is discarded **at parse time**: `parse_optional_chain_expression`
(`kali_parser/src/expression/call.rs:192`) advances past the property token
without recording it, and `OptionalChainInner::NonNull { object, optional }`
(`kali_ast/src/expression.rs:281`) has no slot for it. HIR, types, and codegen
all see only the base, so `obj?.type` compiles as `obj` — a raw i64 handle or
silent 0.

### 4.2 Admitted form

Single-link `obj?.prop` where `obj` is a binding with one proven fixed shape
(the standard monomorphic lane) that may be nullish — the motivating case is
`var o; if (c) o = {x: 1}; use(o?.x)`. Rejected cleanly: chains (`a?.b?.c`,
`a?.b.c`), computed `?.[e]`, call `?.()`.

### 4.3 Pipeline

Parser records the property name via `is_property_name_token` (same key set
as `.` access and, after PR-A §3.2, object literals); `OptionalChainInner`
gains the property field; HIR lowers to a real optional-member node carrying
the property text; **types and codegen grow recognizer arms for the new node
kind in the same commit** (the Spec 2 hand-mirrored-twins lesson).

### 4.4 Semantics without a new repr axis

Object handles are i64 base addresses; a nullish binding is handle 0
(zero-initialized `var`, null inflow). The possibly-undefined *result* of
`obj?.prop` never materializes as a standalone value. Instead the whole
consumer pattern compiles with the short-circuit baked in:

1. **Guard positions** (`if`/`while`/ternary condition, `!`):
   `base != 0 && truthy(field)`.
2. **Equality.** `obj?.x === E` → `base != 0 && field == E`;
   `obj?.x !== E` → `base == 0 || field != E` (undefined `!==` any
   non-undefined value is *true* — the asymmetry is why this compiles as a
   pattern, not a value). `E` must be a scalar or string with repr proven to
   match the field. `=== undefined` / `!== undefined` compile to a pure
   base-null test (admitted fields cannot hold undefined). `=== null` /
   `!== null` **reject**: under `===`, undefined ≠ null, so the result is
   constant regardless of the base — a constant-folding admit is a footgun
   better refused.
3. **`??`.** `obj?.x ?? d` → `base == 0 ? d : field`, requiring the field's
   repr be a proven non-nullish scalar/string so base-nullness is the only
   nullish source and JS semantics collapse to exactly this select. The
   result is then a plain value — assignable, printable, passable anywhere;
   in particular `v ??= obj?.x ?? d` works because the RHS is a plain value.
   An optional member can only appear on the **right** of `??=`: `obj?.x` is
   never a legal assignment target in JS (`obj?.x = e` and `obj?.x ??= e`
   are SyntaxErrors — the parser rejects them as such), and a bare
   `v ??= obj?.x` RHS rejects with E5506 (a possibly-undefined store would
   need sentinel semantics on `v`; out of scope).

### 4.5 Default-deny

Enforced Spec 4a-style: the consumer-pattern allowlist lives at the **single
resolve site** for the optional-member node, so raw assignment
(`const t = obj?.x`), arithmetic use, argument position, and every other
context rejects by construction — no sink enumeration to keep complete.

PR-B reuses two PR-A pieces (the unified null/undefined recognizer for
`=== undefined`, and the print-then-trap helper is available if needed),
which is why it branches from merged PR-A.

## 5. Error surface

- **Parse errors** (existing parser-error style): reserved-word binding names,
  unrecognized object-literal property forms, block-bodied arrow outside
  declarator position, `?.(` and `?.[` grammar forms.
- **New E-code — mixed BigInt arithmetic: E3202 `MIXED_BIGINT_ARITHMETIC`**
  (next free slot in the E3200-3299 "Type errors (basic)" range of
  `kali_error/src/_error_codes.rs`): a *program* error (node throws
  TypeError), not a kali feature gap, so it lives in the type-error series
  rather than hiding under E5506.
- **E5506 FEATURE_UNAVAILABLE** (existing): legal JS outside the admitted
  surface — `?.` chains, `?.` results in non-admitted consumer positions,
  `=== null` comparisons on optional members.
- **Runtime traps**: node-shaped message via the shared print-then-trap
  helper (`Uncaught Error: …`, `RangeError: Division by zero`), then
  `unreachable`.

## 6. Testing and verification

- **Differential fixtures**: every wrong-output item gets a reproducer whose
  kali output is byte-equal to node. PR-B: each consumer pattern ×
  {base null, base non-null}.
- **Reject pins**: every reject gets an E-code pin in the established
  `crates/kali_cli/tests/*_reject.rs` pattern; envelope-level assertions in
  `runtime_smoke/{check,build,run}.rs` where appropriate.
- **Regression guards**: the six CLBG goldens stay byte-for-byte; the 5-crate
  gate (`kali_lexer`, `kali_common`, `kali_types`, `kali_codegen`,
  `kali_cli`) stays green; fmt and clippy clean.
- **PR-B specific pin**: bare `obj?.x` can never again silently compile as
  `obj`.
- **Process guards** (carried over from the fasta series): clear
  `crates/kali_cli/tests/fixtures/.kali-cache` before fixture verification;
  re-run every reproducer on a freshly built binary before claiming a fix
  (Spec 5 lesson); whole-branch adversarial review with live reproducers
  before each merge, re-reviewing after every fix wave (Spec 7 lesson).

## 7. Sequencing and integration

1. PR-A on a branch from main: items ordered mechanical-first (§3.2, §3.3,
   §3.4, §3.6, §3.9 pins), then the medium items (§3.1 helper + throw, §3.7,
   §3.5, §3.8).
2. PR-B branches from merged PR-A.
3. Each PR: whole-branch adversarial review, fix waves re-reviewed, then push
   and self-merge per the standing integration convention.
