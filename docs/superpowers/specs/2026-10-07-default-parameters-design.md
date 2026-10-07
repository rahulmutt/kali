# A literal default parameter on a function declaration runs as node runs it, or kali refuses

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `76fbdc20e` (`main`, the unresolved-member-read merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali` (`dev` profile), built at the baseline |
| oracle | `node v26.10.0`, run as `env -u FORCE_COLOR node` |
| measured on | 2026-10-07 |
| item picked | lane B of the parallel slate agreed on 2026-10-07: the largest parse-time gate in the blast-radius extension corpus |
| sibling lanes | A, numeric-literal grammar (branch `numeric-literal-grammar`, ready to merge); C, growable runtime arrays (not started) |

**Scope was chosen by the human partner:**

* **Arguments that might be `undefined`:** pass through unchanged (option 1
  of two; the other was "refuse unless proven"). A default is filled in only
  for an omitted trailing argument or a literal `undefined`.
* **Defaults admitted:** literal defaults on function declarations called
  directly by name (option 1 of three; the others were "literals + earlier
  params" and "also arrows and methods").
* **Mechanism:** an AST rewrite pass in `kali_cli/src/build/` (approach A of
  three; §4 records B and C).

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** a function declaration whose parameters carry literal defaults
compiles, and every direct call of it runs as node runs it. Every default
parameter outside that slice is refused with E5506, under `check` and `run`
alike, with a message naming why.

**Why this item.** `kali check` accepts 0 of the 40 extension programs in
`tools/blast-radius/corpus/extension/`. 14 of them stop at
`error[E5506]: a default parameter is not supported`, raised by the parser
(`crates/kali_parser/src/declaration.rs:98`, `classify_param_segment`). The
parser refuses before the type checker runs, so that one error hides every
other blocker in those 14 programs. Two more stop at "this parameter form is
not supported" (rest parameters, not in scope here).

**What the corpus needs.** The 14 programs declare 18 defaulted functions
with 21 defaults between them, all literals on function declarations: 20
numbers or strings, and one `hooks = {}`. Every defaulted function is called
only directly by name, and no corpus program exports anything. The slice in
§3 covers all 21.

### 1.1 What this project does NOT claim

* **The accept rate.** Each of the 14 programs has other blockers (for-of over
  runtime arrays, computed keys, `.push()` on literal arrays and others). The
  extension accept rate is measured and reported, and is expected to stay
  0/40.
* **`undefined` at runtime.** Kali has no `undefined` distinct from `0`
  (register cluster G4). `f(x)` where `x` holds `undefined` at runtime passes
  `x` through, so `f` sees `0` where node applies the default. That is G4's
  defect, not a new one, and this project does not narrow it.
* **Defaults that read other bindings** (`function f(a, b = a * 2)`), and
  defaults on arrows, methods, function expressions, generators and `async`
  functions. These are refused.
* **The pre-existing missing-argument failure.** On the baseline,
  `function f(a, b) { return a + b; } console.log(f(1));` passes `check` and
  fails `run` with `error[E4201]: failed to load WASM module` (node prints
  `NaN`). This project refuses the defaulted-function form of that call
  (§3.3) and leaves the plain form alone; the followups file records it.

---

## 2. What was measured

| program | node | kali `check` / `run` at the baseline |
|---|---|---|
| `function f(a, b = 2) { return a + b; } console.log(f(1));` | `3` | E5506 "a default parameter is not supported", exit 1, both |
| `function f(a, b) { return a + b; } console.log(f(1));` | `NaN` | `check` exit 0; `run` E4201, exit 1 |

**Mechanism.** The parser classifies each parameter-list segment
(`declaration.rs:83-104`). `ident = expr` returns
`Err("a default parameter")`, with the comment "needs call-site arity
adaptation, which kali's codegen does not have (calls are emitted at exact
arity)". `FunctionParam` (`crates/kali_ast/src/expression.rs:180`) has a
`name` and nothing else, so there is nowhere to keep a default today.

---

## 3. Design

### 3.1 AST and parser

* **AST.** `FunctionParam` gains
  `default: Option<Box<Expression>>`, with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`, so a program
  with no defaults serializes byte-identically. A `FunctionParam::plain(name)`
  constructor keeps the 42 construction sites (31 of them in
  `crates/kali_cli/src/build_tests/collect.rs`) to one-line changes.
* **Parser.** `classify_param_segment` accepts `ident = expr` and
  `ident: Type = expr`, and the default is parsed as an ordinary expression.
  A parameter without a default may follow a defaulted one
  (`function f(a = 1, b)`), as JavaScript allows. Rest, destructured and
  optional (`?`) parameters stay refused, unchanged.
* **The parser does not narrow the default.** It accepts any expression. The
  pass (§3.2) is the single place that decides what is in scope, so every
  refusal comes from one place with one message per reason.
* **Other consumers of the AST.** `kali fmt` and the effect scanner read
  tokens, and the `--compat eval` folder rewrites source text, so none of
  them sees the new field. `kali lint` walks the AST and ignores a field it
  does not read.

### 3.2 The `default_params` pass

A new module, `crates/kali_cli/src/build/default_params/`, called from
`analyze_source_file` (`crates/kali_cli/src/build/compile.rs`) immediately
after `rename_block_scoped_bindings` (`compile.rs:819`) and before
`rewrite_captured_params` (`compile.rs:830`). `check`, `build` and `run` all
go through `analyze_source_file`, so they see the same rewrite and the same
refusals. Block-scope renaming has already given every binding a unique
spelling, so matching a call to its declaration by name is sound.

**Step 1: collect.** Every function declaration with at least one defaulted
parameter enters a table keyed by its name, holding its parameter list and
each default. A declaration is refused when:

* a default is not a **literal**: a number, string, boolean or `null`; unary
  `-` or `+` on a number; or an array or object literal whose elements or
  property values are themselves literals (no spread, no computed keys, no
  methods, no shorthand properties);
* it is a generator or `async`.

**Step 2: check every reference.** Each occurrence of a defaulted function's
name must be the callee of a plain call `f(...)`. Every other occurrence
refuses the function and names the use: passing or storing it as a value,
`f.call` / `f.apply` / `f.bind`, `typeof f`, `new f()`, `f?.()`, and
exporting it (`export function f`, `export { f }`, `export default f`). A
default on an arrow, method or function expression is refused wherever it
appears.

**Step 3: fill.** At each call, for each parameter position `i`:

| argument at `i` | `i` has a default | action |
|---|---|---|
| omitted | yes | append a fresh clone of the default |
| literal `undefined`, or `void` applied to a literal | yes | replace it with a fresh clone of the default |
| anything else | either | pass through unchanged (human partner's ruling) |
| omitted | no | refuse the call (§3.3) |
| beyond the last parameter | — | leave alone |

A clone is made per call site, so `function f(o = {}) { … }` gets a new
object on each call, as node does. Calls inside the function itself
(recursion) are filled the same way.

**Step 4: strip.** The defaults are removed from the declaration. Every later
stage (monomorphize, the resolver and its arity check, repr inference, HIR,
codegen) sees an ordinary fixed-arity function and needs no change.

**Evaluation order.** JavaScript evaluates a default after the explicit
arguments, in the callee. A literal has no side effects and reads no
binding, so evaluating it at the call site instead is not observable.

**`--compat eval`.** A defaulted function in an eval-compat program is
refused, as `compile.rs` already does for a block-scope rename and a
captured-parameter rewrite. `eval` can call a function by a name the pass
never sees.

### 3.3 Diagnostics

All are `E5506` (`FEATURE_UNAVAILABLE`). The texts live in
`crates/kali_common/src/messages.rs` beside the other refusal texts, and each
diagnostic's span is the default, the use, or the call it is about:

| refusal | message |
|---|---|
| non-literal default | `a default parameter value must be a literal (number, string, boolean, null, or an array or object of literals) in the current phase` |
| defaulted function used as a value | `` a function with default parameters can only be called directly by name in the current phase; `f` is used as a value here `` |
| defaulted function exported | `a function with default parameters cannot be exported in the current phase` |
| default outside a function declaration, or on a generator or `async` function | `default parameters are only available on function declarations in the current phase` |
| omitted argument with no default | `` `f(…)` omits an argument for `b`, which has no default `` |
| eval-compat program | `default parameters are unavailable under --compat eval` |

The parser's "a default parameter is not supported" message goes away; the
"optional parameter", "rest parameter" and "destructured parameter" messages
stay.

---

## 4. Approaches not taken

* **B: the same fill-in inside `kali_parser`, after parsing.** No AST field,
  and it stays in one crate. It runs before `module_link`, so a call through
  a linked namespace is not yet visible, and it would be the first
  whole-program rewrite in the parser.
* **C: arity adaptation in codegen's call emission.** No AST rewrite, but the
  resolver's argument-count check (E5105), repr inference and codegen's call
  emission would all have to learn about missing arguments: three layers to
  keep consistent instead of one pass.

---

## 5. Testing and measurement

### 5.1 Unit tests (sibling `*_tests.rs` files)

* **Parser** (`crates/kali_parser/src/declaration_tests/`): `ident = expr` and
  `ident: T = expr` parse with the default kept; a non-defaulted parameter
  after a defaulted one parses; rest, destructured and optional parameters
  stay refused. `unsupported_params.rs` is re-pinned: its default-parameter
  rows move from refused to accepted, and its "the module is not truncated"
  assertion is kept for every row.
* **Pass** (`crates/kali_cli/src/build/default_params/*_tests.rs`): the
  literal classifier, one test per in-scope shape and per out-of-scope shape;
  filling an omitted trailing argument; replacing a literal `undefined`; a
  distinct clone per call site; passthrough of a non-literal argument;
  recursion; each refusal in §3.3.

### 5.2 Black-box cases

A new `crates/kali_cli/tests/cases/soundness/default_parameters.toml`. Every
expected stdout is node v26.10.0's, in module scope and inside
`function main() { … } main();`:

* a number, string, boolean, `null`, `{}` and `[]` default; the `{}` case
  mutates the object in the callee and calls twice, to show each call gets a
  fresh one;
* the second of two defaults omitted, both omitted, and `f(undefined)`;
* the corpus shapes `wrap(text, width = 72, indent = "")` and
  `paginate(items, page = 1, perPage = 20)`;
* one refusal row per §3.3 message, each run under `check` and `run` so the
  two commands are shown to agree. The value-use, export and arrow refusals
  are in separate files where they need a different `[source]`.

### 5.3 Measurement

* The full `cases` target and `cargo test --workspace`, at `-j 6` and
  `--test-threads=6`, under the resource watchdog, launched with `setsid`.
* `kali check` re-run over the 14 extension programs: none may report the
  default-parameter refusal. The errors that were behind it are listed in the
  followups file.
* `node accepts.mjs` and `node count.mjs` re-run against the branch binary.
  If the accept set moves, both JSON files are committed and the ranking is
  re-spliced with a §6 amendment; if a register lane moves, its oracle case
  is re-pinned and its §0.2 row re-derived.

### 5.4 Bookkeeping

* `specs/19-feature-maturity.md`: a new row for default parameters, naming
  the supported slice, every refusal, and what is not claimed (§1.1).
* `specs/15-errors.md`: no new code. Every refusal is E5506.
* `specs/12-cli.md` and `specs/18-schemas.md`: unchanged. No command, flag
  or JSON shape changes.
* `docs/superpowers/followups/default-parameters-discovered-defects.md`: the
  residue, including the baseline's missing-argument E4201 on non-defaulted
  functions.

### 5.5 Branching

Worktree `/workspace/.worktrees/default-parameters`, branch
`default-parameters`, with `CARGO_TARGET_DIR` inside the worktree. The spec is
committed off `main` at `76fbdc20e`. Implementation starts after lane A
(`numeric-literal-grammar`, which also edits `kali_parser`) merges, with this
branch rebased onto that merge.

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

Measured during execution:

* **A-8. Rebound names.** `block_scope_rename` only gives a binding a unique spelling when a rival sits in its own enclosing frame chain, or when both are program-wide declarations. A `const`, `let`, parameter or arrow named `f` in a sibling function keeps the spelling `f`. Matching calls by name would then fill the wrong call: `function h() { function f(a = 41) { return a; } return f(); } function g() { const f = (a) => a; return f(undefined); } console.log(h(), g());` printed `41 41` (node prints `41 undefined`). The pass therefore refuses a defaulted function whose name is bound more than once anywhere in the program, with a tenth message, `default_param_rebound_name_message`: "a function with default parameters must have a name no other binding in the program uses in the current phase: `f` is also bound elsewhere".
* **A-9. Spread calls are unreachable.** The parser silently drops a spread call: `function g(a) { return a; } const xs = [2]; console.log(g(...xs));` gives `check` exit 0, and `run` prints `0` where node prints `2`. This is pre-existing, with no defaults involved, and is related to register FL-06. The pass keeps its spread refusal, unit-tested on a hand-built AST, but no case file can reach it.
* **A-10. Export specifier.** `export { f };` of any function fails first with the pre-existing E5511 "duplicate export name `f`". The case file therefore pins the export-specifier refusal with `export { f as g };`.
