# Defects the growable-runtime-arrays project measured and did NOT fix

**Filed** 2026-10-08 by the **growable-runtime-arrays** project
(`docs/superpowers/specs/2026-10-07-growable-runtime-arrays-design.md`). **Oracle:**
`node v26.10.0`, run as `env -u FORCE_COLOR node`. **Baseline:** `edb3a77df` on `main`.
**Branch:** `growable-runtime-arrays`, measured at `417b58fb8` (the last
implementation commit; Task 13 changed no compiler code).

Every row below was re-measured on 2026-10-08 with the branch binary unless it says
otherwise. "check" is `kali check`, "run" is `kali run`.

## §1. Pre-existing, found while planning

Re-measured with the branch binary. Only the module-read and capture rows moved, and
only for growable arrays (sub-rows); the plain-array rows stay as measured at the
baseline.

| program | node | kali at the baseline | kali on the branch |
|---|---|---|---|
| `function sum(a) {…} const a = [1, 2]; console.log(sum(a));` | `3` | `run`: E5506 "passing an array literal to function 'sum'…"; `check` exits 0 (refused in codegen only) | unchanged |
| `const a = new Array(3); a[0] = 7; console.log(a);` | `[ 7, <2 empty items> ]` | `run`: E5506 "printing a whole runtime array…"; `check` exits 0 | unchanged |
| `const out = [1, 2]; function size() { return out.length; } console.log(size());` | `2` | `run`: E5506 "reading module binding 'out' from a function…"; `check` exits 0 | unchanged (the literal is not growable) |
| … the same with `out.push(3)` at module scope | `3` | `run`: E5506 at the `push` | `check` and `run`: E5506 "function `size` uses the module-level growable array `out`…" (A-18) |
| `function main() { const o = new Array(2); const f = () => o.length; console.log(f()); } main();` | `2` | `run`: E5506 "a closure `__kali_fn_0` that captures `o`…"; `check` exits 0 | unchanged |
| … the same with `const o = []; o.push(1);` | `1` | `run`: E5506 at the `push` | `check` and `run`: E5506 "the growable array `o` in `main` is captured by a closure…" |
| `let x = 0.5; console.log("c" + x * 1e21);` (any integer-valued float literal ≥ 2^63 in a runtime expression) | `c500000000000000000000` | `run`: `error[E4201]: failed to load WASM module` | unchanged (the plan's table quoted node as `c5e+20`; node prints the 21-digit form) |
| `let y = 1e21; let x = 0.5; console.log("c" + y * x);` | `c500000000000000000000` | prints `c-4611677222334365700`, exit 0 (silent) | unchanged |
| `let z = Math.pow(10, 21); console.log("c" + z);` (also `10 ** 21`) | `c1e+21` | prints `c3875820019684212736`, exit 0 | unchanged |
| `function main() { let line = "> "; for (let i = 0; i < 3; i++) { if (line.length > 3) { line = "> "; } else { line = line + "ab"; } } console.log(line); } main();` | `> ab` | E5506 "reassigning an array binding to a non-array value…" (`.length` on a string registers it as an array; 7 corpus lines) | unchanged (still 7 corpus lines) |
| `function j(a) { return a.join("-"); } const a = [4, 5]; console.log(j(a));` | `4-5` | E5506 "elements of `a` … are used as both strings and numbers" | unchanged |
| `function main() { const a = []; a.push(1); let i = 1.5; console.log(a[i]); } main();` (float index) | `undefined` | codegen-only E5506 (Stage 4): `check` exits 0 | unchanged: `run` E5506 "indexing a growable array with a floating-point value…", `check` exits 0. The same holds for a float index on a write (A-24) |

## §2. Corpus

`kali check` over the 40 extension programs in `tools/blast-radius/corpus/extension/`,
with the command in the plan's Task 13 Step 3. Baseline binary `/workspace/target/debug/kali`
(`edb3a77df`), branch binary at `417b58fb8`. The baseline reproduces the spec's §2.2
exactly.

| message family | baseline | branch |
|---|---|---|
| all `error` lines | 386 | 302 |
| `computed member access X is unavailable … unless the index is a l…` | 86 | 68 |
| `for-of array iteration lowering is unavailable unless the iterable is a literal array …` | 65 | 50 |
| `calling X on a literal array is unavailable in the current phase: kali folds …` | 52 | 0 |
| `Array.prototype.slice is unavailable unless the receiver is a statically-known …` | 22 | 18 |
| `array search method 'indexOf' is unavailable unless …` | 10 | 10 |
| `Array.prototype.join is unavailable unless …` | 10 | 7 |
| `reassigning an array binding to a non-array value is unavailable …` | 7 | 7 |
| `Array.prototype.concat is unavailable …` | 3 | 3 |
| `mutating a literal array is unavailable in the current direct-runtime path …` | 1 | 0 |
| any `growable array` message (new) | 0 | 1 (an object element) |
| **programs with at least one array-family refusal** | **36** | **34** |

**Non-array families that changed** (from the family diff; every other family is
unchanged):

| family | baseline | branch |
|---|---|---|
| `String.prototype.slice is unavailable on runtime string receivers …` | 6 | 7 |
| `computed key access X where the key is not a for..in key …` | 3 | 6 |
| `compound assignment lowering is unavailable unless the target is a mutable local binding …` | 3 | 5 |
| `String.prototype.repeat is unavailable unless …` | 2 | 3 |
| `String.fromCharCode is unavailable unless …` | 2 | 3 |

Investigated, per the plan: all nine new lines are **pre-existing refusals now reached,
not new refusals**. Each sits in the body of a `for-of` whose iterable (or a `push`
in it) was refused at the baseline, which stopped resolution of that body; now the
loop is admitted and the body's own unsupported construct is reported. Examples:
`counts[word] += 1;` inside `for (const word of words)` (`word_frequency.js:42`, the
compound-assignment and computed-key lines), `counts[index] += 1;` inside
`for (const value of values)` (`logistic_map.js:28`), `String.fromCharCode(byte)`
inside `for (const byte of bytes)` (`base64_encode.js:56`), `"  ".repeat(…)` inside
`for (const heading of toc)` (`markdown_toc.js:66`), `sample.slice(0, 20)` inside
`for (const sample of SAMPLES)` (`crc32_checksum.js:75`). No program gained a refusal
for a construct it did not already contain.

## §3. Accept set

Unchanged: `accepts.mjs` against the branch binary reads anchor **125/137** and
extension **0/40**, as the spec predicted. `accepts.json` and `counts.json` changed
only in the binary path and were not committed. The blast-radius ranking was
re-spliced anyway, because a register lane moved (§4, R-12); only R-12's §3 verdict
cell changed and no band moved (ranking §6 amendment of 2026-10-08).

## §4. Pins moved by this project

**Unit tests (Tasks 6-11).**

- `kali_types/src/resolve/member_tests.rs`: 8 assertions that pinned the literal-array
  mutator refusal on `push`, `pop` or an index store now pin the growable admission,
  or moved the refusal property onto `reverse`, which is no growable demand (Task 6,
  Task 8). Each carries a `Growable-runtime-arrays moved pin` comment.
- `kali_types/src/growable_tests.rs` (the Stage 4 allowlist tests) was deleted with
  the allowlist; `repr_infer_tests.rs`'s `growable_promotion_*` and
  `array_return_growable_const_literal_taints_growable` tests were replaced by the
  component-solve tests (Task 6).
- `kali_codegen/src/intrinsics/string_tests/lookup.rs`: two `i64.const 1` counts grew
  by 5 for the new method synthetics present in every module (Task 10).
- `kali_cli/tests/runtime_smoke.rs`: the fixed-slot synthetic list gained the six
  growable helpers (Tasks 9-10).

**Case files (Task 13, 270 trials, all measured at `417b58fb8` against node v26.10.0,
each with a dated `RE-PINNED 2026-10-08` note in its rationale).**

| family | trials | now | evidence |
|---|---|---|---|
| `array/literal_array_mutators` (push/pop/alias/parameter under `run`) | 5 | runs | stdout equals node's (`4`; `2 3`; `1`; `1`; `1`) |
| `array/literal_array_mutators` (`check` twins) | 2 | `check` accepts | the `run` twins print node's output |
| `misc/growable_array_fail_closed[js,ts]` (escaping return, pop, alias, wrong arity, float push, two multi-arg console rows) | 14 | runs | stdout equals node's (`3`, `1`, `3`, `2`, `2`, `2 1`, `len 1`) |
| `misc/growable_array_fail_closed_push_diagnostics::…wrong_arity` | 1 | runs | node `2` |
| `object/class_instances::ok_user_push_*` | 2 | runs / `check` accepts | node `2 5` |
| `object/computed_member_static_name::*_array_literal_element_store` | 2 | runs / `check` accepts | node `9` |
| `runtime/join` literal-array index mutations | 3 | runs | node `42` |
| `soundness/structured_clone::named_growable_alias_is_broken_tripwire` | 1 | runs | node `1,2,3,4` (the tripwire's known-wrong `1,2,3` is gone) |
| `misc/for_of_object_keys_iteration` (`run_supports_*`) | 34 | runs, exit 0, empty stdout | node exits 0 with empty stdout; the fixtures throw on any wrong result |
| `browser/object_keys_harness` | 64 | runs under the harness | node prints `browser object keys iteration ok`; the `Kali.test` bodies pass under a one-line shim |
| `browser/object_keys_break_continue_harness` | 16 | runs | node prints `browser object keys break/continue iteration ok` |
| `browser/object_keys_iteration` (direct, global, const-bound builds) | 24 | `build --bundle` succeeds | node runs each fixture's function without throwing; so does kali via `run --api browser` with a trailing call |
| `browser/string_concatenation_{harness,bundle}` | 16 + 8 | runs | node prints `browser string concatenation ok` |
| `browser/template_literal_string_iteration_{harness,bundle}` | 16 + 8 | runs | node prints `browser template literal iteration ok` |
| `misc/growable_array_core` (push into the iterated array; mixed i64/string) | 4 | still refused, message moved | node `5` / `2`; the snapshot refusal (A-8) and the mixed-elements refusal (A-4) |
| `misc/growable_array_fail_closed_push_diagnostics::…object_literal_arg` | 1 | still refused, message moved | node `1`; the element refusal |
| `runtime/string_value_flow::mixed_literal_int_and_string_element_store_is_rejected` | 1 | still refused, message moved | node `1!`; the mixed-elements refusal |
| `soundness/bitwise_compound::bitwise_compound_growable_rhs_push_now_refuses_at_the_push` | 1 | still refused, message moved | node `1`; the plain-use refusal (A-3) on the `&=` operand |
| `misc/arena_reclamation_runtime::module_global_store_fails_closed` | 1 | **refused (capability loss)** | node `201`, which kali printed at the baseline (§5) |
| `browser/for_await_object_string_enumeration_browser_smoke` (`app` check/build) | 16 | **refused (capability loss)** | `[key, value]` arrays pushed as elements (§5) |
| `array/callback_identity_browser_harness` | 16 | **refused (over-refusal)** | node prints `some:true`, `every:false`, `1 2` ×5; passed at the baseline (§5) |
| `misc/for_of_object_keys_iteration` (`test_supports_object_values_*`) | 8 | **refused (over-refusal)** | node passes the bodies; passed at the baseline (§5) |
| `misc/set_iteration_runtime::run_supports_set_constructor_iteration_*` | 4 | **runs a program node rejects** (known-wrong tripwire) | node: `SyntaxError` (§6) |
| `oracle/tier2::r12_alias_defeats_array_store_guard_*` | 2 | `fail_closed` → `fixed` | kali and node both print `b0=7`; register §0.2 R-12 row re-derived FIXED |

The six generated case files touched (`misc/{growable_array_core,set_iteration_runtime,
arena_reclamation_runtime,for_of_object_keys_iteration}.toml`,
`runtime/{join,string_value_flow}.toml`) carry a `HAND-EDITED 2026-10-08` header note:
their generators have no re-pin channel.

## §5. Residue inside the lane

**Capability losses and over-refusals (all fail closed; none is silent).**

- **Objects stored by index into an array literal** — `const arr = [mk(0)]; arr[0] = mk(v);`
  ran at the baseline (node `201`). The index write makes the literal growable (A-2)
  and objects are refused as elements (§3.4). Accepted by the controller (Task 6
  concern 2). A fix would exclude object-element components from growability.
- **Arrays pushed as elements** — `entries.push(entry)` over `Object.entries('ab')`
  (16 `for_await_object_string_enumeration_browser_smoke` trials); a reduced program
  reading `entries.length` printed node's `2` at the baseline.
- **Element-proof over-refusals (A-21).** Measured on the branch:
  - a callback inside a spread operand (`for (const x of [...xs.filter((v) => v > 1)]) out.push(x)`)
    refuses every binding element in the program: the program-wide `unwalked` flag in
    repr inference (`repr_infer.rs`) disables binding-element admission when any
    nested function is unseeded, which a callback inside a spread is after the CLI
    names anonymous functions. Fix: seed nested functions inside spread operands in
    the lockstep Phase-B pass. Cause of the 16 `callback_identity_browser_harness`
    refusals. node `2,3`.
  - `for (const v of Object.values(p)) out.push(v)` where `p` aliases an object
    binding: refused (node `1,2`); 8 `for_of_object_keys_iteration` trials.
  - `out.push("b".toUpperCase())`, and any string-method result, refused with the
    misleading pair "used as both strings and numbers" plus "an element that is an
    object…" (node `a,B`).
  - `a.push(NaN)` refused as an unsupported element (node runs).
  - `new Set(a)` for a growable `a` refused with the plain-use message (node `3`).
- **A mixed number/string array gets two messages**, the mixed-elements refusal and
  a second element refusal that wrongly names objects/booleans (`o.push(1); o.push("a")`;
  also `s.indexOf(1)` on a string array).
- **The snapshot rule (A-8) treats two arrays as one when they share a component**
  because both reach the same parameter: `fill(a); fill(b); for (const x of a) b.push(x);`
  is refused, though node would agree with the snapshot (node `2`).
- **Every growable array that leaves its function is allocated globally and never
  reclaimed**, even when the callee does not keep it. Strings pushed into an escaping
  array survive the creating call (`cases/array/growable_layout.toml` `two_builds`).
- **A floating-point index is refused only by `run`** (reads and writes; A-24):
  `check` accepts. A check/run disagreement against spec §3.6.
- **A growable array returned by a module-level arrow and used from another function
  is refused** with legacy messages: `const mk = () => { const o = []; o.push(1); o.push(2); return o; }; function main() { for (const x of mk()) console.log(x); } main();`
  — node `1 2`; `check` and `run` refuse with "for-of array iteration lowering is
  unavailable…" (inference resolves the arrow alias only in the declaring scope). The
  same arrow used at module scope runs (A-22). Unpinned (Task 12 C1).
- **The module-read message names a synthetic function**: `const out = []; out.push(1); const f = () => out.length;`
  refuses with "function `__kali_fn_0` uses the module-level growable array `out`"
  (Task 12 C2; the refusal case's needle omits the function name).
- **`a["1"] = 6` on a growable array** is refused by `run` only, with the misleading
  "rendering a String() result bound to a variable…" message (`check` exits 0; node `6`).
- **`s[0] = s[1] + "d"` on a growable string array** is refused ("storing a runtime
  string value into this element … unless the target is an array whose elements are
  all proven strings"); node `bd`.
- **`-z` with an i64 runtime zero pushed into an f64 array stores `+0`**:
  `let z = 0; a.push(0.5); a.push(-z); 1 / a[1]` prints `Infinity` (node `-Infinity`).
  A-28 fixed the literal and float-zero cases only. Silent, edge-case.
- **`console.log` of an f64 `-0` prints `0`** (node `-0`), for a growable element and
  for any f64 value (`let b = 0.5 * -0; console.log(b)`). Pre-existing console
  rendering; `join` printing `0` is correct (A-12). Silent.
- **A `NaN` or `Infinity` identifier in a runtime f64 expression fails to load**
  (`error[E4201]: failed to load WASM module`), pre-existing at the baseline
  (`let x = 1.5; console.log(x === NaN)`); this lane now exposes it through
  `a.indexOf(NaN)`, `a.includes(NaN)` and `a.indexOf(Infinity)` on f64 arrays
  (`check` exits 0). A computed NaN (`0 / 0`) works. Loud, not silent.
- **Closures over loop bindings outside the growable lane.**
  - Module scope, any non-growable loop: `for (let i = 0; i < 3; i++) { const g = () => i; console.log(g()); }`
    prints `0 0 0` (node `0 1 2`), and the same with `for (const x of [5, 6])`
    prints `0 0` (node `10 12`). Pre-existing at the baseline, silent. A-27 refuses
    only the growable form.
  - Function scope: a closure capturing a `for-of` loop variable passes `check` and
    is refused by `run` (`calling 'f' is unavailable … first-class function value`;
    node `2`).
  - Module scope, `let x; for (x of a) { f = () => x; }` over a growable `a`: `check`
    exits 0, `run` refuses ("growable for-of loop variable `x` has no local slot").
- **Task 11b's capture refusal matches names per function, not per lexical scope**
  (`positions.rs`), so it can over-refuse a module-scope closure that captures a
  different binding with the same name as a growable loop variable. Recorded by the
  Task 11b review; a block-scoped probe (`{ const x = 3; const g = () => x + 1; … }`
  after a growable `for (const x of a)`) was admitted, so the over-refusal needs a
  shape this measurement did not find.

**Pre-existing defects outside the lane, surfaced by it.**

- **E3200 `var + var + "lit"` gate.** `line = line + x + " "` (the left operand a
  `var + var` `+`) is refused on `main` too (`reject_unsupported_string_variable_addition`,
  `resolve/expression.rs`): `function main() { let line = ""; let x = "a"; line = line + x + " "; console.log(line + "|"); } main();`
  → E3200, node `a |`. The p1 cases were spelled around it (A-31). Fixing it needs
  the resolver and the codegen `+` arm in lockstep.
- **The parser accepts a malformed string literal.** `console.log('a['Set'] b');`
  prints `a[` at exit 0 at the baseline and on the branch; node:
  `SyntaxError: missing ) after argument list`. ACCEPTS_INVALID. Four
  `misc/set_iteration_runtime` fixtures contain this shape; they are pinned as a
  known-wrong tripwire (A-34) so a parser fix turns them red.
- **`for await` is accepted inside a non-async function.**
  `function f() { for await (const x of "ab") { console.log(x); } } f();` prints `a`,
  `b` at exit 0; node: `SyntaxError: Unexpected reserved word`. The
  `string_concatenation` and `template_literal_string_iteration` harness smoke tests
  rely on it (their `Kali.test` callbacks are non-async arrows).
- **1e21 arithmetic** (§1): E4201 for a large float literal in a runtime expression,
  garbage for `1e21 * x` through a binding, and `Math.pow(10, 21)`.

**Code hygiene left by the task reviews (minor, deferred; true at `417b58fb8`).**

- `ReprTable`'s growable accessors (`set_growable_return`, `growable_return`,
  `mark_growable_local_only`, `is_growable_local_only`; `kali_common/src/repr.rs`)
  have no doc comments.
- `growable_returns` / `array_returns` disjointness is stated, not enforced.
- `facts.rs` records `UseKind::LengthRead` for an update expression on `a.length`
  without a comment saying it is safe only while the resolver refuses member updates;
  the `module_loops` pop-by-last lockstep invariant is uncommented.
- Untested fact-walk paths (merge temporaries, spread parameters, a closure in a
  `for-of`, optional-chain methods, `a[k]()`, compound-assign plain uses); no direct
  refusal-dedup test; no `Temp(slice)` / merge subject-text tests; the `[…]` / `?:`
  subject text is hard-coded in `positions.rs` rather than a `kali_common` helper.
- No negative test that a non-growable string-array return is still refused by I2;
  the W2 key test lacks a negative (module alias called in `main`) and a chain
  (`const h = mk`) case; the `for-of` unsupported-target message is unpinned.
- `emit/growable.rs`'s `for-of` length load inlines the header mask/length sequence
  that `emit_growable_length` already emits; `node_is_growable_value`'s `slice` check
  does not exclude computed members (harmless over-reservation).
- `cases/array/growable_runtime_arrays_refused.toml`: the object and boolean element
  rows share one needle; some `check_*` rows lack a rationale.
