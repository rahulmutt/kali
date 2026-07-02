# Baseline green: closure-return isolation + 137 sibling fixes (138 failing kali_cli tests)

**Date:** 2026-07-02
**Status:** Approved
**Branch:** `fix/stale-browser-bundle-expectations`
**Baseline:** main @ 81496c912 — `cargo test -p kali_cli --no-fail-fast` has 138 failures.
**Investigation:** `/workspace/.superpowers/sdd/baseline-fix-investigation.md` (verified; mechanism for
classes 2/4/5 refined below).

## Problem

138 kali_cli tests fail, in 11 classes. Three classes are real compiler bugs whose test pins are
already node-correct (do NOT re-pin); the rest are stale test expectations or stale fixtures exposed
by sound new diagnostics.

| # | Class | Tests | Kind | Fix |
|---|-------|------:|------|-----|
| 1 | Math.sqrt(1.6) "rejects" tests — sqrt now supported | 29 | stale rejects-pin | flip to supports, pin `1.2649110640673518` |
| 2 | Math.atan2 trailing-arg evaluation (`const bump = () => {…return 2;}`) | 16 | **real miscompile** | closure-return isolation (Task 1) |
| 3 | compat-eval fixtures trip new E3200 string-`+` guard | 6 | stale fixture | rewrite to literal-rooted concat |
| 4 | Integer-like key ordering enumeration (`const consumeArray = (…) => …`) | 38 | **real miscompile** | closure-return isolation (Task 1) |
| 5 | Plain object/string-primitive/Object.values enumeration | 25 | **real miscompile** | closure-return isolation (Task 1) |
| 6 | Browser bundle `greet(1n,2n)` pinned to stub `0n` | 13 | stale pin | re-pin `1n` (b5c085401 style) |
| 7 | BigInt `/` lowered to `F64Div` (`3n / 2n` prints `1.5`) | 2 | **real miscompile** | `I64DivS` for BigInt-literal operands |
| 8 | `event.type` unparseable inside `${…}` (E2004) | 6 | **real parser bug** | accept keyword tokens as `.` property names |
| 9 | Optimization benchmark suite aborts on E3200 fixture | 1 | stale fixture | rewrite 3 benchmark fixtures + sha256 |
| 10 | `check` fixture-tree walk aborts on same E3200 fixtures | 1 | stale fixture | same rewrites (count stays `Checked 65 file(s)`) |
| 11 | schema_docs slug lists lack fannkuch-redux / spectral-norm | 1 | stale list | add 2 slugs + 2 source files |

Also in scope: the `keys.length !== 2` fixture typo (correct value 4) in 10 class-4 fixtures —
6 inline fixtures + the shared `browser_runtime_object_enumeration_test_source()` helper.

## The closure-return bug — verified mechanism (refines the investigation report)

The report attributed classes 2/4/5 to "closure inlining in kali_codegen". Verified against the
built binary, there are two distinct defects with one shared symptom (a `return` escaping into the
enclosing function):

1. **Expression-bodied arrows** (`const f = (x) => x + 1;`) parse fine, but HIR lowers the arrow to
   a `FunctionExpr` whose body child is a bare synthesized `ReturnStmt`
   (`crates/kali_hir/src/lowering/function.rs:58-64`) — while function declarations/expressions get
   a `Block` body. Codegen's `function_plan` (`crates/kali_codegen/src/lower.rs:729-763`) only
   recognizes `Block`-bodied nodes as functions, so the arrow is never collected as a standalone
   wasm function and never skipped by `is_function_like` — the declaration site emits the arrow's
   children inline, and the synthesized `Instruction::Return` (real since 104ef4de1,
   `crates/kali_codegen/src/emit/control_flow.rs:62-76`) terminates the **enclosing** function.
   Verified: `console.log("A"); const h = (x) => x + 1; console.log(h(41));` prints nothing, exit 0.
2. **Block-bodied arrows** (`const bump = () => { console.log("bump"); return 2; };`) are not
   parseable as arrows at all: both arrow parse paths bail when `{` follows `=>`
   (`crates/kali_parser/src/declaration.rs:254-257`, `crates/kali_parser/src/expression/primary.rs:26-27`).
   The `{…}` ends up as a top-level block statement that executes inline at the declaration point;
   its `return` truncates the enclosing function. Verified: the block's `console.log` fires at
   declaration time, not call time.

## Locked design: route const-bound arrows through the existing function-expression lane

**Control (verified on the built binary):** `const f = function () { console.log("X"); return 7; };
console.log(f()); console.log("after");` prints `X`/`7`/`after` — unnamed function expressions
bound to `const` already work end-to-end: `function_plan` collects them under their HIR-synthetic
name (`__kali_fn_N`), the declaration site skips them via `is_function_like`, and call sites resolve
through `resolve_bound_member_callable_node` (`crates/kali_codegen/src/emit/call.rs:2802`), which
follows the const `bindings` map to the function node and dispatches a real wasm `Call`.

Three minimal changes put arrows on that proven lane:

1. **`function_plan` accepts the arrow LIR shape** (`crates/kali_codegen/src/lower.rs`): treat a
   last child that is a `Branch("return")` node (the arrow's synthesized implicit return) as a valid
   function body, alongside `Block`. The closure is then compiled as its own wasm function (where
   its `Instruction::Return` is correct), skipped at the declaration site, and callable through the
   existing binding resolution — no new dispatch machinery.
2. **Parser: block-bodied arrows in variable-declarator init position**
   (`crates/kali_parser/src/statement.rs:93-97` + new fn in `crates/kali_parser/src/declaration.rs`):
   parse `const f = (params) => { statements }` into an unnamed `Expression::FunctionExpression`
   with a real `BlockStatement` body — byte-for-byte the AST shape of the working control above.
   Scoped to declarator init so the (huge, passing) `Kali.test('…', () => {…})` callback lane is
   untouched.
3. **Argument-position safety net** (`crates/kali_codegen/src/emit/call.rs:2187`): the generic call
   arg loop pads a non-produced argument with `I64Const(0)` so a function-valued argument (now
   emitting nothing instead of leaking a `Return`) cannot underflow the wasm stack.

**Why this over alternative (b) (keep inlining, wrap in a wasm block, rewrite returns to `br`):**
(a-as-refined) reuses the exact lane named functions and function expressions already ride, which is
empirically correct today, and needs no new lowering shape. Critically, an earlier draft of (a) that
wrapped the arrow's HIR body in a `Block` was rejected during design verification because the array
identity-callback matchers (`crates/kali_codegen/src/intrinsics/array.rs:153-262` and siblings)
pattern-match the current arrow LIR shape (`children[1]` = `ReturnStmt`) — changing the shape would
silently break the passing `array_callback_*` suites; accepting the shape in `function_plan` instead
leaves every existing consumer untouched. Alternative (b) would leave defect 2 (block arrows aren't
arrows at all) unfixed, and it is exactly the inline path where the known E3100 param-scoping defect
lives. Blast radius check: the only const-bound block-arrow fixtures in the whole test tree are the
class-2 `bump` fixtures (all currently failing), so the parser change has no passing-test exposure.

**Correctness note for classes 4/5:** the pinned stdout of all 63 tests comes from top-level
`Object.keys/entries/values` logs and never depends on `consumeArray`'s return value (its `!== 4n`
self-check throws, and `throw` is a codegen no-op — see follow-ups). The fix bar is: declaring the
arrow must not truncate, and calling it must execute its body at call time (class 2's `bump` must
print at the trailing-argument evaluation point). Both hold on this lane (verified via the
function-expression control, including `Math.atan2(0, 1, bump())` → `bump` then `0`).

## Other compiler fixes

- **BigInt `/` (class 7):** `emit_binary` hardcodes `float_op = true` for `/`
  (`crates/kali_codegen/src/emit/operators.rs:614-615,672-679`), and `is_float_valued` mirrors it
  (`operators.rs:495`). Both sites gain a BigInt exception: when **both** operands are BigInt
  literals (LIR `Literal` with an `n` suffix, resolved through transparent wrappers and const
  bindings), `/` stays on the i64 lane and emits `I64DivS` — JS BigInt division truncates toward
  zero. Scope is literal/const-bound operands only: the repr machinery has no BigInt axis yet, so
  BigInt-typed `let` variables keep today's behavior (follow-up).
- **Keyword property names (class 8):** the member parser accepts only
  `Identifier | Delete | From` after `.` (`crates/kali_parser/src/expression/call.rs:44-70`), so
  `event.type` stops after `event` (the lexer reserves `type`,
  `crates/kali_lexer/src/identifier.rs:39`) and the strict `${…}` sub-parser hard-errors E2004.
  Fix: accept every keyword-shaped token the lexer can produce from `lex_identifier` as a property
  name (token `value` already holds the word text) — standard JS/TS behavior.

## Test-side and fixture changes

- **Class 1:** flip 26 runtime_smoke tests + 3 tests in
  `browser_math_unsupported_member_calls_harness_jsx_tsx.rs` from E5506-rejection to success.
  Ground truth (node and the built kali binary agree, bit-for-bit): `Math.sqrt(1.6)` →
  `1.2649110640673518`, across all six access forms in the harness fixture. Per-lane pins:
  run/test assert that stdout value; check/build assert success (build JSON has `stdout: null`).
  The `Math.exp`/`Math.log`/atan2 rejection siblings keep their pins.
- **Class 6:** `assert_browser_bundle_executes` (`crates/kali_cli/tests/runtime_smoke.rs:1638`)
  hardcodes `0n`; its 13 `"greet"` callers ship `function greet(name) { return name; }`, so
  `greet(1n, 2n)` correctly returns `1n`. Add `assert_browser_bundle_executes_with_result(…,
  expected)`; the old name delegates with `"0"` so the other 94 callers (fixtures genuinely
  `return 0n;`) are untouched; the 13 greet callers pass `"1"`.
- **Class 3:** the fixtures' `prefix + suffix` source-string construction is incidental to what
  they test (dynamic eval under `--compat eval`) and now trips the sound E3200 guard
  (string-typed **variable** operand of `+` — probe-verified: even `"x" + prefix` is rejected).
  Rewrite to pure-literal concatenation (`"1" + " + 2"`, `"return " + "1 + 2;"`) — probe-verified
  green under `run/build/check --compat eval`.
- **Classes 9/10:** rewrite the 3 benchmark fixtures to eliminate the string-typed `folded`
  variable (inline it into the return expression, preserving the measured fold shape), update the
  3 metadata `sourceSha256` values (`sha256-<hex of source bytes>`). Rewrites probe-verified green
  under `kali check`; file count unchanged so class 10's `Checked 65 file(s)` pin stands.
- **Class 11:** add `"fannkuch-redux"`/`"spectral-norm"` and their `-benchmark-v1.ts` source files
  to the two hardcoded `BTreeSet` lists (`crates/kali_cli/tests/schema_docs/misc.rs:2073,2140`).
- **Typo:** the 10 class-4 fixtures assert `keys/entries/values.length !== 2` against the 4-key
  object and stop at index 1; align them with the correct 4-key sibling
  (`runtime_smoke.rs:1295`).

## Out of scope — recorded follow-ups (no tasks)

1. **`ThrowStmt` codegen is a no-op** (`HirNodeKind::ThrowStmt` has zero codegen references):
   every `throw`-based fixture self-check in the suite is vacuous. Suite-wide test-integrity gap;
   implementing it will surface latent wrong values (e.g. `consumeArray`'s result).
2. **Block-bodied arrows outside declarator-init position** (call arguments, single-ident
   `x => {…}`, `let`-bound arrows): still the legacy inline-garbage parse, including the spurious
   E3100 param-scoping failure. The `Kali.test` callback lane deliberately stays on it for now.
3. **BigInt `/` for non-literal BigInt-typed operands** (needs a BigInt repr axis).
4. **No re-baselining** of fannkuch-redux / spectral-norm pinned benchmark outputs — a changed
   output there is a regression to investigate.

## Verification

Gate (every task): `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali_cli`.
End state: that command fully green — 138 failures to 0, with the 1696 currently-passing
runtime_smoke tests and all currently-green binaries unchanged.
