# Defects the length-fails-closed project measured and did NOT fix

**Filed** 2026-09-11 by the **length-fails-closed** project
(`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`), on the
convention `release-tier-allocation-identity-discovered-defects.md` and its
predecessors use: a project that measures more than it fixes writes down what it
left, so the silence is not read as absence.

**Oracle:** `node v26.8.2`. **Measured at:** `152fdd5364` (the project's
baseline), with `kali run`, unless a row says otherwise. None of these is in the
register; §1 and §2 are silent wrong values and are candidates for it.

**Ranked, most consequential first.** §1 and §2 are silent wrong values on
ordinary code; §3 is silent but narrow; §4 is a cross-reference; §5 is a lead;
§6 is cleanup. §7 and §8 were added by the project's final review, after this
ranking was written; by consequence, §8 (a silent wrong value on ordinary code)
would rank alongside §1-§2, and §7 (already in the register as R-39/R-40) would
rank alongside §4.

> **Notice, 2026-09-12, by the inline-allocation-value-position project**
> (`docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md`,
> Task 10):
>
> - **§1 is closed.** Its two silent rows are retired register entries: **R-64**
>   (an allocation outside the materializing lanes evaluates to `0`, closed
>   FIXED) covers the argument-position rows, and **R-65** (a fold-lane array
>   argument reads as zeros in the callee, closed FAIL_CLOSED) covers the
>   fold-lane-literal row. Both were closed 2026-09-11 by the
>   inline-allocation-value-position project's Tasks 8 and 9. §1's own repro
>   table is left as measured history; do not re-open it as still-open.
> - **§2 gains a measured row.** `console.log((new Array(3).fill(2))[1]);` →
>   kali `undefined`, node `2` — re-verified 2026-09-12 against this branch's
>   HEAD (`a09a468516`) with the existing binary and `node v26.8.2`, no rebuild.
>   The `new` misparse (§2's own subject) still swallows the trailing index
>   access the same way it swallows `.length` and `.message`; the
>   length-fails-closed fix's floor does not reach an index read.
> - **§3's row is unchanged.** `(new Array(3)).length` still prints `1`
>   (node `3`) — re-verified 2026-09-12 against the same HEAD and binary. This
>   is expected, not a regression: the value comes from `render_length`'s
>   array-literal arm (`intrinsics/host.rs:1267-1285`), which accepts a
>   text-less one-`Call`-child node as a one-element array literal and returns
>   its child count *before* `emit_value`'s floor or fallbacks are ever
>   reached — the inline-allocation-value-position project's routing lives in
>   `emit_value`/`emit_call`, downstream of this arm, so it cannot see this
>   value. See this repository's
>   `inline-allocation-value-position-discovered-defects.md` for what that
>   project measured and left open in the lanes it did touch.

---

## §1. An inline allocation passed as an argument reaches the callee as zeros

| program | kali | node |
|---|---|---|
| `function f(x) { return x.length; } console.log(f(new Array(6)));` | `0` | `6` |
| `function f(x) { return x[0]; } console.log(f(new Array(3).fill(2)));` | `0` | `2` |
| `function f(x) { return x.length; } console.log(f(new Array(4).fill(1)));` | `0` | `4` |
| the first row inside `function main() { ... } main();` | `0` | `6` |
| `function f(x) { return x.length; } const a = new Array(6); console.log(f(a));` | `6` | `6` (control: a BOUND allocation is correct) |
| `function f(x) { return x.length; } console.log(f([1, 2, 3]));` | `E5506`, exit 1 | `3` (control: an inline LITERAL is refused) |

All silent rows exit 0 with no diagnostic. **Mechanism, partly traced:**
`crates/kali_codegen/src/emit/call.rs:3591-3614` refuses an inline array
**literal** argument because "the callee would read zero placeholders", and its
comment at `:3579-3582` assumes `new Array(n)` / `.fill()` arguments "live in
locals … and pass a real handle, so they are untouched here". That holds for a
bound allocation and fails for an inline one. It is a guard keyed on one form
with a sibling hole. The length-fails-closed fix does not touch it: inside `f`,
`x` is an array binding and its `.length` reads a header, not a fallback.

## §2. `new` swallows the member chain after its arguments

`new` parses its callee with `parse_call_expression()`
(`crates/kali_parser/src/expression/primary.rs:241`), so the chain after the
arguments becomes part of the callee:

| program | parses as | kali | node |
|---|---|---|---|
| `console.log(new Array(3).length);` | `new (Array(3).length)` | `2` at `152fdd5364`; `E5506` since the length-fails-closed fix | `3` |
| `console.log(new Error("m").message);` | `new (Error("m").message)` | `0` | `m` |
| `const a = new Array(4).fill(7);` | `new (Array(4).fill(7))` | works | — |

The first row's `2` was `render_length`'s tail fallback rendering the misparsed
`Array(3)` call node's child count (callee plus argument). The length-fails-closed
fix declines that fallback, so the read now refuses (pinned in
`crates/kali_cli/tests/cases/runtime/length_fails_closed.toml`, both scopes). That
is fail-closed, not fixed: the misparse is unchanged, and the second row is still
silent. The project's spec (§2.6) had predicted this row would not move.

The third row is why this has hidden: every `new Array(n).fill(v)` in the
benchmark fixtures parses wrongly and still works, because the wrapping `new`
node is transparent to every consumer. Established by a throwaway LIR dump (spec
§2.1). **A fix changes the LIR shape of every `new X(...).m()` in the corpus**,
including every recognizer tuned to the misparsed shape
(`declarator_init_is_array_fill` and its siblings), so size it by the
recognizers that could see the new shape, not by compiler errors
(`release-tier-allocation-identity-discovered-defects.md` §9).

## §3. `(new Array(3)).length` still prints `1`

`console.log((new Array(3)).length);` prints `1` at exit 0; node prints `3`. The
same value before and after the length-fails-closed fix. `new Array(3)` lowers to
a text-less `Value` with one `Call` child, which is also the shape of `[f()]`, and
two `.length` arms accept it as a one-element array literal and return its child
count: `render_length`'s array-literal arm (`intrinsics/host.rs:1267-1285`) and
`emit_unary`'s (`emit/operators.rs:389-413`).

**Now that the floor refuses, narrowing those two arms is fail-closed for
`.length`** (spec §3.5), but it is unmeasured, and it would refuse
`const a = [f()]; a.length`, which prints `1` correctly today. The other ~36
consumers of the same predicate still have silent floors
(`codegen-array-literal-predicate-is-still-negative-space.md`, corrected
2026-09-11), so they must not be narrowed first.

## §4. `render_static_value` renders an aggregate's child count as a value

`crates/kali_codegen/src/intrinsics/host.rs:928-942` renders a text-less node with
two or more children as `children.len()`. It is the value-position sibling of the
`.length` fallbacks the length-fails-closed project closed, and it is the
mechanism of the register's **R-31** ("`console.log` of an array prints its
length"). Not re-filed; recorded here because a reader of R-63's retirement will
look for it.

## §5. Lead: a nested function reading a captured binding gets a warned zero

`function main() { const arr = [1, 2, 3]; function g() { return arr.length; } console.log(g()); } main();`
printed `0` at exit 0 at `152fdd5364` (node `3`). Its stderr carried only
`warning[E3100]: undefined identifier 'arr' reached codegen and was lowered through
a zero placeholder compatibility fallback`. After the fix it refuses, but only
because the read is `.length`. **Other reads of a captured binding through a
nested function were not measured.** Check the register's R-02 and the Stage C
closure notes before filing: this may be a known closure lane rather than a new
entry.

## §6. The per-hazard `.length` bails are now redundant

`render_length` (`intrinsics/host.rs:1195-1250`) and the runtime member lane
(`emit/control_flow.rs:2490` onward) each carry bails for one shape found "the
expensive way": `URLSearchParams`, `TextDecoder.decode`, `String(...)`, inline
`TextEncoder.encode`, `crypto.getRandomValues`. Each exists because that shape fell
through to a fabricated count. With the floor refusing, each bail now declines
onto a refusal the floor would give anyway. Removing them is a behaviour-neutral
cleanup **only if** each bail's own case keeps its current message, because
several of those cases pin a specific needle. The length-fails-closed project left
them alone (spec §3.4).

## §7. Mutated array literals are stale — already the register's R-39/R-40

**Added by the project's final review, 2026-09-11.** The same two array-literal
arms that §3 describes (`render_length`'s `intrinsics/host.rs:1267-1285`,
`emit_unary`'s `emit/operators.rs:389-413`) return the literal's element count
inferred from source, not instrumented, so once the array is mutated the count
is stale rather than fabricated from nothing. Both arms precede every other
`.length` lane, so this is not a gap the length-fails-closed fix touches; the
floor and fallbacks it closed sit downstream of these two arms, not in front of
them.

| program | kali | node |
|---|---|---|
| `const a = []; a.push(1); a.push(2); console.log(a.length);` | `0` | `2` |
| `const a = [7]; a.push(1); console.log(a.length)` | `1` | `2` |
| `const a = [1, 2, 3]; a.pop(); console.log(a.length)` | `3` | `2` |
| `function main() { const a = [1, 2, 3]; a.pop(); console.log(a.length); } main();` | `3` | `2` |
| `const a = [1, 2, 3]; a.length = 1; console.log(a.length);` | `3` | `1` |
| `const a = [1, 2]; a.splice(0, 1); console.log(a.length);` | `2` | `1` |
| `const a = [1, 2]; a.shift(); console.log(a.length);` | `2` | `1` |
| `const a = [1]; a.unshift(0); console.log(a.length);` | `1` | `2` |
| `const a = []; for (let i = 0; i < 3; i++) { a.push(i); } console.log(a.length);` (module scope) | `0` | `3` |
| `const a = []; a.push(1); if (a.length === 0) { console.log("empty"); } else { console.log("nonempty"); }` | `empty` | `nonempty` |

All rows exit 0 with no diagnostic, measured at `d9014ba317` against
`node v26.8.2` with the probe files already in the project's probes directory
(`growable.js`, `g5_nonempty_push.js`, `g6_pop.js`, `g18_pop_fn.js`,
`g8_len_store.js`, `g11_splice.js`, `g12_shift.js`, `g17_unshift.js`,
`g9_push_loop.js`, `g15_push_cmp.js`). Two in-function forms are correct rather
than stale — `function main() { const a = []; a.push(1); a.push(2);
console.log(a.length); } main();` → `2` (node `2`) and the same shape for a
module-scope-declared-in-function for-loop push (`g4_push_fn.js`,
`g19_push_const_fn_empty.js`) — so the discriminator is scope and which mutator
runs, not a blanket in-function fix; `.pop()` in a function still reads `3`
against node's `2` (`g18_pop_fn.js`, row 4 above).

This is not a new class: it is the register's **R-39** (`Array.prototype.pop()`
returns `0` — a different, narrower repro than the row above, but the same
"the receiver's real state was never consulted" defect) and **R-40** (`.push`
on a `const` array literal is silently ignored, both scopes; R-40's own text
already cross-references the module-scope growable-push row above to **§7.9**'s
"Module-scope growable `push` is a silent no-op"
(`P5-R-modulescope-growable-push`) in the same register). Recorded here, with a
current-HEAD measurement, because a reader of R-63's retirement will look for
where these two families live.

## §8. `.length` on a non-array argument reads `0`

**Added by the project's final review, 2026-09-11.** A parameter's `.length` is
not a text-less node, so it does not reach R-63's floor or fallbacks; it is a
different lane.

| program | kali | node |
|---|---|---|
| `function f(x) { return x.length; } console.log(f(42));` | `0` | `undefined` |
| `function f(x) { return x.length; } console.log(f(0));` | `0` | `undefined` |
| `function f(x) { return x.length; } console.log(f(8));` | `0` | `undefined` |
| `function f(x) { return x.length; } console.log(f(1024));` | `0` | `undefined` |
| `function f(x) { return x.length; } console.log(f(true));` | `0` | `undefined` |
| `function main() { function f(x) { return x.length; } console.log(f(42)); } main();` | `0` | `undefined` |
| `function f(x) { return x.length; } const n = 7; console.log(f(n));` | `0` | `undefined` |

All rows exit 0 with no diagnostic, measured at `d9014ba317` against
`node v26.8.2` (`q5_param_num.js`, `q7_param_nums.js`, `q8_param_num_fn.js`,
`q9_param_undef.js` in the project's probes directory). **Mechanism not
traced** — likely the runtime array-header `.length` lane registers any
`.length` receiver as an array binding regardless of the parameter's actual
value, rather than proving it is an array first, but this was not confirmed by
reading the emitter. **A candidate for the register**: no existing entry names
this specific shape (a numeric- or boolean-valued parameter, not an absent
field or an out-of-bounds read), so it is filed here rather than under an
existing ID.

## §9. Notes (cosmetic, no register filing)

**`kali check` disagrees with `kali run` on the R-63 receiver.**
`const o = {a: "xyz"}; console.log(o.a.length);` — `kali run` refuses with the
`.length` floor's `E5506` (exit 1), but `kali check` exits 0 (`Checked 1
file(s)`), measured at `d9014ba317` (`c1_check.js`). This is the same
check/run split the register's **R-12** row already notes for a different
codegen refusal ("`kali check` still exits 0 on the aliased program while
`run` refuses — a spec §8 twin disagreement the oracle harness cannot see,
because it observes `run` only"); not a new mechanism, and not filed
separately.

**`const {length} = o.a` refuses with a misleading message.**
`const o = {a: "xyz"}; const {length} = o.a; console.log(length);` refuses with
`error[E5506]: a reserved word cannot be used as a binding name` (exit 1) in
both scopes, where node prints `3` at exit 0 (`destructure.js`,
`destructure_fn.js`, measured at `d9014ba317`). `length` is not a reserved
word in JavaScript; the message is wrong even though the refusal itself is the
safe fail-closed direction. Cosmetic only — no register filing.
