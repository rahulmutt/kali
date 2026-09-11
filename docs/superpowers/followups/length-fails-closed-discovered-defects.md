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
§6 is cleanup.

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
