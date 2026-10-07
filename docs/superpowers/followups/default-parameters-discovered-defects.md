# Defects the default-parameters project measured and did NOT fix

**Filed** 2026-10-07 by the **default-parameters** project
(`docs/superpowers/specs/2026-10-07-default-parameters-design.md`). **Oracle:**
`node v26.10.0`. **Baseline:** lane A (numeric-literal-grammar, PR #56) is
merged to `main` as `9675521aa`, and this branch is rebased onto that merge.
**Branch:** `default-parameters`.

## §1. Pre-existing, found while planning or executing

Each row re-measured with the branch binary and node on 2026-10-07.

| program | node | kali |
|---|---|---|
| `function f(a, b) { return a + b; } console.log(f(1));` | `NaN` | `check` exit 0; `run` `error[E4201]: failed to load WASM module` (a call omitting an argument of a function with no defaults) |
| `function f(a) { return a; } console.log(f?.(1));` | `1` | `check` exit 0; `run` prints `0`. `f?.(…)` parses as `OptionalChainExpression { object: f }` and the call and its arguments are dropped |
| `console.log(late(1)); function late(a) { return a; }` | `1` | `error[E3100]: undefined identifier 'late'` (a call before the callee's declaration) |
| `function bar(c, p, w) { return Math.round((c / p) * w); } console.log(bar(3, 6, 40));` | `20` | `run`: `error[E4201]: failed to load WASM module`, exit 1 |
| `function tag(o) { return 1; } console.log(tag({}));` | `1` | `error[E5506]: an object literal passed directly as a call argument is unavailable in the current phase; bind it to a const first` (the reason for spec A-7) |
| `function arr(xs) { return xs.length; } console.log(arr([]));` | `0` | now `check` exit 0 and `run` prints `0`, agreeing with node (the planning-time E5506 for an array literal argument no longer reproduces at this baseline); an object literal argument is still refused, so A-7's reasoning for object defaults stands, and an array default is refused by the pass on its own message |
| `function g(a) { return a; } const xs = [2]; console.log(g(...xs));` | `2` | `check` exit 0; `run` prints `0`. The parser drops a spread call silently, with no defaults involved (spec A-9; related to register FL-06). The pass's spread refusal is unreachable from source |
| `function f(a) { return a; } export { f }; console.log(f(1));` | `1` | E5511 "duplicate export name f", check and run alike (spec A-10) |
| `function f(a = /re/) { return 1; } console.log(f());` | `1` | `error[E5506]: this default parameter value is unavailable in the current phase`: a regex default is refused by the parser itself, before the pass |
| `const f = (a, b) => a + b; console.log(f(1));` | `NaN` | `check` exit 0; `run` `error[E4201]: failed to load WASM module`. A sibling arrow called with fewer arguments than parameters breaks check/run parity at the baseline |
| `function f(a = await w()) { return a; }` (TypeScript file) | not valid JS | the parser parses a default's expression with fresh async/generator flags, so `await` and `yield` inside a default are read as plain identifiers; the pass refuses the result as a non-literal default (`this default parameter value is unavailable in the current phase`), so it fails closed |
| `function f(a: Map<string, number> = x) { return 1; }` (TypeScript file) | not valid JS | `error[E5506]: this parameter form is not supported`: the pre-existing parameter splitter breaks on the generic comma; fails closed |
| `const f = (a): number => { return a; }; console.log(f(2));` (TypeScript) | `2` | `error[E3100]: undefined identifier 'a'`, check and run alike, with or without a default (`(a = 1): number => {…}` fails the same way). Pre-existing and loud: a block-bodied annotated arrow loses its parameter |
| `const f = (...r): number => 1; console.log(f());` (TypeScript) | `1` | `error[E3100]: undefined identifier 'r'`, check and run alike. The parser's `Unsupported` arm (rest, destructured or optional parameters) has the same return-type-annotation gap the `Simple` arm had: an annotated arrow with such a parameter is not refused at the parameter list, it fails later and loudly |
| `const h = async (a = 1) => { return a; }; h().then((v) => console.log(v));` | `1` | `error[E3100]: undefined identifier 'async'` then `undefined identifier 'a'` (twice), check and run alike, exit 1. Earlier on this branch (per the final whole-branch review, not re-measured here) it got the default-parameter E5506 "default parameters are only available on function declarations in the current phase". It still fails closed; only the message regressed. The same program without the default (`async (a) => { return a; }`) fails with the same E3100s, so the block-bodied `async` arrow is the pre-existing gap; the expression-bodied `async (a = 1) => a` still gets the E5506 |

## §2. What the 14 extension programs need next

Re-checked with `kali check` after the change. No program reports a
default-parameter error except `task_queue.js`, whose `hooks = {}` object
default is the A-7 refusal, as designed. The rebound-name refusal (A-8) blocks
none of the 14. Remaining `error` lines (backtick contents elided):

**`argv_stats.js`** — 0 default-parameter errors
- `error[E3100]: undefined identifier 'process'`
- `error[E5506]: Array.prototype.join is unavailable unless the receiver is a statically-known array li`
- `error[E5506]: Array.prototype.slice is unavailable unless the receiver is a statically-known array l`
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: Math.max.apply is unavailable in the current phase; use a supported Math builtin or th`
- `error[E5506]: Math.min.apply is unavailable in the current phase; use a supported Math builtin or th`

**`build_report_json.js`** — 0 default-parameter errors
- `error[E5506]: a runtime string value is unavailable as an object-literal property value in the curre`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `

**`heat_diffusion_1d.js`** — 0 default-parameter errors
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: mutating a literal array is unavailable in the current direct-runtime path; use new Ar`

**`hex_dump.js`** — 0 default-parameter errors
- `error[E5506]: Array.prototype.concat is unavailable unless the receiver is a statically-known array `
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: '.length' on a runtime string value is unavailable unless the string is ASCII-provable`
- `error[E5506]: String.fromCharCode is unavailable unless every argument is a statically-known ASCII i`
- `error[E5506]: String.prototype.charCodeAt is unavailable unless the receiver is a statically-known A`
- `error[E5506]: String.prototype.padStart is unavailable unless the receiver is a statically-known ASC`

**`histogram_bars.js`** — 0 default-parameter errors
- `error[E5506]: Array.prototype.slice is unavailable unless the receiver is a statically-known array l`
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: String.prototype.padStart is unavailable unless the receiver is a statically-known ASC`
- `error[E5506]: String.prototype.repeat is unavailable unless the receiver is a statically-known ASCII`

**`matrix_ops.js`** — 0 default-parameter errors
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`
- `error[E5506]: String.prototype.padStart is unavailable unless the receiver is a statically-known ASC`

**`moving_average.js`** — 0 default-parameter errors
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: Math.max.apply is unavailable in the current phase; use a supported Math builtin or th`
- `error[E5506]: Math.min.apply is unavailable in the current phase; use a supported Math builtin or th`
- `error[E5506]: Math.sin is unavailable unless the argument is a statically-known zero numeric literal`
- `error[E5506]: String.prototype.padStart is unavailable unless the receiver is a statically-known ASC`

**`paginate_results.js`** — 0 default-parameter errors
- `error[E5506]: Array.prototype.slice is unavailable unless the receiver is a statically-known array l`
- `error[E5506]: a runtime string value is unavailable as an object-literal property value in the curre`
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`

**`pivot_sales.js`** — 0 default-parameter errors
- `error[E5506]: Array.prototype.concat is unavailable unless the receiver is a statically-known array `
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: String.prototype.concat is unavailable unless the receiver and all operands are static`
- `error[E5506]: String.prototype.padEnd is unavailable unless the receiver is a statically-known ASCII`

**`task_queue.js`** — 1 default-parameter errors
- `error[E5506]: an object or array default parameter is unavailable in the current phase: kali cannot `

**`template_render.js`** — 0 default-parameter errors
- `error[E5506]: Array.prototype.slice is unavailable unless the receiver is a statically-known array l`
- `error[E5506]: array search method 'indexOf' is unavailable unless the receiver, search value, and fr`
- `error[E5506]: computed member access X is unavailable in the current phase unless the index is a l`
- `error[E5506]: reassigning an array binding to a non-array value is unavailable in the current direct`
- `error[E5506]: String.prototype.slice is unavailable on runtime string receivers in the current direc`
- `error[E5506]: String.prototype.split is unavailable unless the receiver is a statically-known ASCII `

**`turnstile_fsm.js`** — 0 default-parameter errors
- `error[E5506]: array search method 'indexOf' is unavailable unless the receiver, search value, and fr`
- `error[E5506]: a runtime string value is unavailable as an object-literal property value in the curre`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `

**`word_frequency.js`** — 0 default-parameter errors
- `error[E5506]: Array.prototype.slice is unavailable unless the receiver is a statically-known array l`
- `error[E5506]: array search method 'indexOf' is unavailable unless the receiver, search value, and fr`
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: compound assignment on binding 'current' is unavailable: it is not a provably scalar n`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: '.length' on a runtime string value is unavailable unless the string is ASCII-provable`
- `error[E5506]: reassigning an array binding to a non-array value is unavailable in the current direct`
- `error[E5506]: String.prototype.toLowerCase is unavailable unless the receiver is a statically-known `

**`wrap_paragraph.js`** — 0 default-parameter errors
- `error[E5506]: calling X on a literal array is unavailable in the current phase: kali folds a liter`
- `error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array `
- `error[E5506]: String.prototype.split is unavailable unless the receiver is a statically-known ASCII `

## §3. Accept set

Unchanged at anchor 125/137, extension 0/40. `accepts.mjs` and `count.mjs` were
re-run against this branch's binary; only `kaliBinary` changed in
`accepts.json`, so `accepts.json` and `counts.json` were reverted. The register
and ranking still move for R-01, which is now two lanes: the declaration lane
`r01` moved FAIL_CLOSED to FIXED, and the function-expression lane `r01b` is
FAIL_CLOSED (ranking §6 amendment THIRTEENTH). No band changes.

## §4. Pins moved by this project

Re-pinned to node's output on 2026-10-07, each with a `RE-PINNED 2026-10-07 by
the default-parameters project` note in its rationale:

- `soundness/param_truncation::default_param_function_declaration_fails_closed`
  (`param_truncation.toml`): was `exit = "failure"` with E5506; now the program
  runs and prints `after`, exit 0, as node does. The sibling function-expression,
  arrow and class-method cases still refuse and are unchanged.
- `oracle/tier1::r01_default_parameter_module_scope` and
  `oracle/tier1::r01_default_parameter_in_function` (`oracle/tier1.toml`): were
  `fail_closed`; now `fixed` (kali prints `A` then `B`, exit 0, as node does).
  Register §0.2's R-01 row is now two lanes: FIXED (declaration, `r01`) and
  FAIL_CLOSED (function expression, `r01b`, two new oracle cases added in the
  fix round); the ranking was re-derived with them (no band moved).
