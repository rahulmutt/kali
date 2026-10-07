# Defects the numeric-literal-grammar project measured and did NOT fix

**Filed** 2026-10-07 by the **numeric-literal-grammar** project (lane A of the
parallel slate agreed on 2026-10-07; it is bounded, so its design is in the
session transcript and the commit messages, with no spec file). It follows
the convention of the other `*-discovered-defects.md` files: a project that
measures more than it fixes writes down what it left.

**Oracle:** `node v26.10.0`. **Baseline:** `76fbdc20e` (`main`), on
`/workspace/target/debug/kali`. **Branch:** `numeric-literal-grammar`, on that
worktree's own `target/debug/kali`. Every row was run by hand with the program
text given in full.

**What the project did.** The lexer reads `0x`/`0b`/`0o` integers, `_`
separators, and legacy-octal and non-octal-decimal integers as one token.
`kali_common::numeric_literal::parse_js_numeric_literal` implements
JavaScript's NumericLiteral grammar and is the only reader of a numeric
token's value: the lexer validates with it (E1100 on a spelling JavaScript
refuses), and the parser's expression and object-key arms, the
`--compat eval` constant folder (`build/eval.rs`) and the effect scanner's
computed keys (`kali_sandbox/src/effects/scan.rs`) read with it. The parser's
`unwrap_or(0.0)` fallback is gone.

**Register:** R-58's SILENT lane moved to FIXED in both scopes (§0.2 row
re-derived, `r58a_*` re-pinned to `fixed`). The entry is not retired, because
its strict-mode face (§1 below) is still open.

---

## §1. R-58's strict-mode face: ACCEPTS_INVALID, still unpinned

| program | node | kali `run` (branch) | kali `run` (`76fbdc20e`) |
|---|---|---|---|
| `"use strict"; const o = {042: 1}; console.log(Object.keys(o)[0]);` | `SyntaxError: Octal literals are not allowed in strict mode.`, exit 1 | `34`, exit 0 | `42`, exit 0 |

It was ACCEPTS_INVALID before and it still is. Only the value it prints
changed. Node refuses `042` and `08` under `"use strict"` and in every ES
module. Kali has no strict-mode notion on the lexer/parser path, so the fix
needs one first.

## §2. A hexadecimal, binary or octal BigInt is refused (capability gap)

`0xffn`, `0b1n` and `0o7n` are valid JavaScript. Kali refuses each with E5506
under `check` and `run`. Kali carries a BigInt literal as decimal text, and
converting a non-decimal spelling to decimal needs arbitrary-precision
arithmetic, which the project did not add.

## §3. Found while measuring; pre-existing and NOT in the register

Both rows are silent wrong output at exit 0 on `main` (`76fbdc20e`) and on
this branch alike. Neither involves a non-decimal literal, so neither was in
scope. They are listed here so they are not lost; neither has a register entry.

| program | node | kali `run` (`main` and branch) |
|---|---|---|
| `const n = 1000n + 1n; console.log(n);` | `1001n` | `1001`, exit 0 |
| `const k = 34; console.log(eval("k + 16"));` under `--compat eval` | `50` | `0`, exit 0 |

- The BigInt row: a BigInt arithmetic result renders without its `n` at the
  direct-log sink. `console.log(1000n)` (a literal, no arithmetic) prints
  `1000n` correctly on both binaries.
- The eval row: `rewrite_static_eval_calls` folds only a constant string
  argument. Here the argument is a source snippet that reads a binding, and
  the result is a silent `0`. `eval("" + k)` (the constant folder's
  supported shape) is correct on the branch.

## §3a. Duplicate numeric keys are newly reachable through hex and legacy-octal spellings

The underlying defect is older: a duplicate property key is not collapsed,
so the object keeps both entries and reads the first. On `main` it already
reproduces with identifiers (`{a: 1, a: 2}`). This project made it reachable
from numeric spellings that now share one canonical key. Measured 2026-10-07,
node v26.10.0 against this branch's `kali run`, all exit 0:

| program | node | kali |
|---|---|---|
| `const o = {16: 1, 0x10: 2}; console.log(Object.keys(o).join(","), o[16]);` | `16 2` | `16,16 1` |
| `const o = {34: 1, 042: 2}; console.log(Object.keys(o).join(","), o[34]);` | `34 2` | `34,34 1` |
| `const o = {a: 1, a: 2}; console.log(Object.keys(o).join(","), o.a);` (control, `main` prints the same) | `a 2` | `a,a 1` |

Not in the register, not fixed here: the fix is duplicate-key collapse in
object-literal lowering, independent of the numeric grammar.

## §3b. `1e5n` is not a lexical error

The lexer ends the token at `1e5` and leaves `n` to the identifier lexer, so
the failure is E3100 `undefined identifier 'n'`, not E1100. Both engines refuse
the program (node: SyntaxError); only the code differs. The stale comment in
`number.rs` that said "the parser rejects it" now says this.

## §4. String-to-number conversion is a different path

`Number("0x10")` is not reachable (E3100 on `Number`, both binaries).
`parseInt("0x10")` already printed node's `16` at the baseline. Neither goes
through the literal grammar, and the project did not touch either.
