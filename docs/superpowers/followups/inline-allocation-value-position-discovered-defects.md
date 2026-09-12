# Defects the inline-allocation-value-position project measured and did NOT fix

**Filed** 2026-09-12 by the **inline-allocation-value-position** project
(`docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md`),
on the convention `length-fails-closed-discovered-defects.md` and its
predecessors use: a project that measures more than it fixes writes down what
it left, so the silence is not read as absence.

**Oracle:** `node v26.8.2`. **Measured at:** each row's own task (Tasks 6-9,
cited per section), and cross-checked against this branch's HEAD, `a09a468516`,
where a row's status says so. None of these rows is proposed as a new register
entry — each task report that measured one explicitly deferred it to this
document rather than to `kali-silent-miscompile-register.md`; §11 is the one
exception, which the register already documents independently.

This project's four register filings — **R-64** (an allocation outside the
materializing lanes evaluates to `0`), **R-65** (a fold-lane array argument
reads as zeros in the callee), **R-66** (a one-element literal of an
allocation IS that allocation), **R-67** (`.fill(v)` re-evaluates `v` per
element) — are all retired (CLOSED, 2026-09-11) and are not repeated here.
What follows is everything *adjacent* to that work that this project found
along the way and chose not to fix.

**Ranked, most consequential first, by "what a reader would work on next and
what it would cost."** §1-§2 are silent wrong values reachable through
ordinary code, with a plausible next step already named. §3-§6 are silent
wrong values in the same shadow-checking machinery this project built, in
decreasing order of how directly this project's own new code caused them.
§7 is a silent wrong value this project deliberately left out of scope on a
correctness trade-off, not an oversight. §8 groups the loud, fail-closed
exceptions, which are already safe and none of them need urgent attention.
§9 is a behaviour this project changed correctly but left unpinned — the
cheapest item in this whole document to close. §10 is maintenance hazards.
§11 is a cross-cutting architectural note for whoever makes cross-module
calls real.

---

## §1. Two call-shaped array producers slip past the widened argument guard

Task 9's guard (the `_refuses` fix behind R-65) is keyed on **text-less,
aggregate** LIR nodes — the shape a fold-lane array literal or a constructed
value lowers to. A call whose *result* is an array, but which is itself an
ordinary `Call` node, is invisible to it:

| program | node | kali |
|---|---|---|
| `function f(x) { return x.length; } console.log(f(globalThis.Array(5)));` | `5` | `0` |
| `function f(x) { return x.length; } const s = "a,b,c"; console.log(f(s.split(",")));` | `3` | `0` |

Both exit 0 with no diagnostic. Both are "adjacent holes" outside R-65's own
nine pinned lanes: call-shaped array producers reaching `emit_value`'s
placeholder in call-argument position — a sibling class to R-65's
fold-lane-literal shape, not covered by Task 9's fix, which is keyed on
array-*literal*/constructed-value LIR nodes, not on arbitrary call results
(`task-9-report.md`, "Filed for Task 10"). Out of scope for Task 9 by the
review's own instruction, and recorded there for this document to fold in.

**What it would cost:** the guard (or a sibling of it) would need to key on
"a call whose *callee* is a known array-returning builtin/method"
(`globalThis.Array`, `.split`, and presumably `.slice`, `.map`, `.filter`,
`.concat`, and any other array-returning method not yet audited), not on LIR
node shape. That is a different, and likely larger, recognizer surface than
the one this project built — sizing it needs its own sweep of array-returning
builtins/methods before scoping a fix.

## §2. An unvalidated allocation size argument is loud for a fraction, silent for a negative integer

Neither `new Array(n)` arm validates its size argument. The two known
programs diverge in observability, and only one of them is safe:

| program | node | kali | class |
|---|---|---|---|
| `function f(x) { return x.length; } console.log(f(new Array(2.5)));` (a fraction) | `RangeError` | `error[E4201]`, invalid wasm module | loud (exit 1), but the wrong shape of error — an invalid module, not a diagnostic |
| `function f(x) { return x.length; } console.log(f(Array(-1)));` (a negative integer) | `RangeError` | prints `-1` at exit 0 | **silent** |

The first is pre-existing on the declarator lane; this project's Task 8
widened its reach to more value positions. The second shares the same root
cause (the size argument is unvalidated) but is silent, not loud — it belongs
in a silent-wrong-value reading despite measurements.md filing it alongside
the loud row it shares a mechanism with.

**What it would cost:** validate the size argument before allocating (reject
a non-integer or negative value with a real diagnostic, matching node's
`RangeError`). This is a size-argument validation gap, not a routing gap —
smaller in scope than §1, but untouched by this project because size
validation was never in its brief.

## §3. The declarator lane and Arm B were never gated against a shadowed constructor name

Task 8 built `allocation_ctor_unshadowed`, a five-namespace shadow guard, and
applied it to exactly the two new arms Task 8 itself added. Two
pre-existing call sites into the same allocation machinery were never gated:

| program | node | kali | which lane |
|---|---|---|---|
| `function Uint8Array(n){return n+1;} const y = Uint8Array(3); console.log(y);` | `4` | `4104` | the declarator lane (`resolve_array_alloc_call`'s pre-existing declarator caller) |
| `function Uint8Array(n){return n+1;} function f(x){return x[0];} console.log(f(new Uint8Array(3).fill(2)));` | throws `.fill is not a function` | `2` | Arm B (`new Uint8Array(n).fill(v)`, the pre-existing fill-on-fresh-allocation lane) |

Both are pre-existing — wrong before this project's own work touched
anything — and both were deliberately left ungated: "Task 8 gated only its
own two new arms" and "Arm B was deliberately left ungated" are the standing
rulings (`task-8-report.md` §14.7, §15.7; `task-9-report.md`'s scope
boundary). Recorded together because the same guard, `allocation_ctor_unshadowed`,
already exists and is already proven correct for the bare-name case — closing
either of these two rows is "call the existing guard from one more call
site," not new design.

**What it would cost:** wire `allocation_ctor_unshadowed`'s bare-name check
into `resolve_array_alloc_call`'s declarator caller and into Arm B. Low
design risk since the guard's bare-identifier logic is already built and
tested (`emit/call_tests/allocation_ctor_shadow.rs`); the work is call-site
wiring plus new pins for both lanes, not a new predicate.

## §4. Two remaining identity gaps in the globalThis-qualified shadow gate

`allocation_ctor_unshadowed`'s qualified-callee arm (round 5,
`4f9298fe37`) accepts exactly one shape — a bare identifier named
`globalThis` — and declines everything else. Two adjacent shapes it declines
by name-matching rather than identity resolution stay silently wrong:

| program | node | kali |
|---|---|---|
| `const globalThis = {Uint8Array:fn}; function f(x){return x;} console.log(f(globalThis.Uint8Array(5)));` (bound through the **declarator lane**, not argument position) | `6` | `4104` |
| `function f(x){return x.length;} console.log(f(globalThis["Uint8Array"](5)));` (bracket-property spelling) | throws `TypeError` | `4104` |

The first: `is_array_like_constructor` matches the qualifying object by
**text only**, with no binding resolution; it is pre-existing and reproduces
identically before Task 8, on the same ungated declarator lane §3 names. The
second: pre-existing, routed identically before Task 8's gate — node throws
because `globalThis["Uint8Array"]` is not a constructor without `new`, and
kali's placeholder still fires.

**What it would cost:** the first needs the same declarator-lane wiring as
§3. The second needs `is_array_like_constructor` (or its caller) to
distinguish a bracket-property access from a dot-property access, which
today it does not; both are narrower instances of the same "matched by text,
not identity" limitation named in `allocation_ctor_unshadowed`'s own rustdoc.

## §5. The shadow gate is blind to imports

| program | node | kali |
|---|---|---|
| `import { Uint8Array } from "./m.mjs"; f(Uint8Array(3))` | `4` | `4104` |

The five-namespace shadow gate (`locals`, `bindings`,
`module_binding_names`, `fn_valued_locals`, `functions`) has no namespace for
an imported binding. This is not unique to this project's guard: codebase-wide,
`url_ctor_unshadowed` (the pre-existing `URL`/`URLSearchParams` shadow gate)
misses imports identically, so this is a shared, pre-existing limitation, not
something this project introduced. It converts no refusal into a silent
value — the un-intercepted path was already a silent placeholder `0` before
this project's work. §11 below is the broader cross-module-call defect this
row is one instance of.

**What it would cost:** the shadow gate needs a name set threaded from the
module linker into codegen — a structural change reaching outside
`kali_codegen`, and shared with `url_ctor_unshadowed`'s identical gap, so a
fix should close both gates' import blindness at once rather than
special-casing the allocation gate alone.

## §6. A nested-`globalThis` casualty this project could not cheaply avoid

| program | node | kali |
|---|---|---|
| `function f(x){return x.length;} console.log(f(new globalThis.globalThis.Uint8Array(5)));` | `5` | `0` |

**NET-NEUTRAL against the pre-Task-8 baseline `733cd26125`, where this program
was also `0`.** Task 8 round 4 incidentally made it correct (`5`); round 5's
bare-identifier restriction (§4's "exactly one shape" rule) returned it to
the baseline value. This is not a regression this project introduced and
left — it is a pathological spelling nothing was ever designed to handle,
that briefly worked by accident and now doesn't, at the same value it had
before this project began (`task-8-report.md` §15.4).

**What it would cost:** there is no cheap correct fix. A blanket
"decline ⇒ deny (refuse)" was considered and rejected: it would re-break the
ordinary case where a *declined* bare `Uint8Array(3)` must fall through and
call the user's own function rather than refuse. Closing this needs the same
identity-resolution work as §4 and §5, generalized to an arbitrary property
chain, not a special case for two levels of `globalThis`.

## §7. `.fill` on a bound receiver inside an array literal — deliberately left open

| program | node | kali |
|---|---|---|
| `const ys = new Array(2); const xs = [ys.fill(0)]; console.log(xs.length);` | `1` | `2` |

**Ruled out of scope in Task 6**, on a correctness trade-off, not an
oversight: `kali_types` cannot distinguish an array binding's own `.fill`
call from a user object's unrelated method also named `.fill`, so widening
the allocation-collision guard to catch this shape would trade one measured
miscompile for an unmeasured over-refusal (declining valid programs that call
a real user `.fill`). Task 8's routing made this shape reachable from more
value positions than before (it used to be wrong only as a declarator
initializer; now it is wrong in argument position too), but the declarator
instance itself is byte-for-byte unchanged, and the argument-position
instance was already wrong before Task 8 touched anything — Task 8 only
widened where the *existing* wrong number is reachable from, not the number
itself (`task-8-report.md`, R-64's "Also NOT closed" note).

**What it would cost:** a scope-aware receiver test that can tell an array
binding from a user object at the point `.fill` is called — the same
capability `kali_types` already declined to build for the declarator case.
Not a small addition; it is a new binding-type inference this project
scoped out deliberately.

## §8. Loud, fail-closed exceptions — safe as-is, no urgency

These four all fail closed (exit 1, node-divergent, or an intentional
exception), and none of them is new:

| # | program | node | kali | status |
|---|---|---|---|---|
| B1 | `[new Array(3).fill(0).fill(1)]` | `1` | refuses | Pre-existing since the array-literal-of-an-allocation recognizer's first landing. |
| B2 | `new (await globalThis).Uint8Array(3)` | — | refuses | Codegen reads the object child's text directly and declines; `kali_types` unwraps `await` and refuses independently. Exotic. |
| B5 | `[new new Array(3)]` | throws `TypeError` | `3` | A documented, named exception in `expression_is_array_allocation`. Cannot affect a valid program: a `new` expression's result is never itself constructible, so node throws for *every* value the inner `new` can produce — there is no input on which kali's `3` and node's behaviour could both be observed as a program result. |

**What it would cost:** nothing needs doing here. B1 and B2 are already the
safe direction (a wrong program refuses instead of running with a wrong
value); B5 cannot diverge on any program node itself accepts. Recorded so a
future sweep does not re-discover and re-triage them as if they were new.

## §9. An unpinned behaviour change: `.fill` on a zero-length array now evaluates its argument

| program | before this project | after this project |
|---|---|---|
| `new Array(0).fill(g())` | calls `g` zero times | calls `g` exactly once |

Correct, and node-matching (node also evaluates the argument once regardless
of the receiver's length) — but this is a *second* behaviour change beyond
the three fill-once pins this project's spec named
(`crates/kali_cli/tests/cases/...fill_value_once*`), and no test case covers
the zero-length receiver specifically. It is the cheapest item in this whole
document to close: add one pinned case exercising `new Array(0).fill(g())`
with a call counter, alongside the existing fill-once pins.

## §10. Maintenance hazards

None of these are wrong values — they are traps for the next person to touch
this code:

| # | hazard | what it would cost to close |
|---|---|---|
| D1 | `crates/kali_types/src/resolve/expression.rs` has **two** same-named recognizers: an associated `Self::expression_is_array_allocation` (handles `Array` only — no `Uint8Array`, `await`, `as`, `satisfies`, or `globalThis`) and the free function this project hardened. The gate calls the free one correctly, but neither's doc comment mentions the other exists. | A doc-comment cross-reference on both, naming which one the gate actually calls and why the other is narrower. No behaviour change. |
| D2 | The Task 7 reservation test pins the trailing scratch reservation's SIZE (2→5 slots) but not the INDICES (`+2`/`+3`/`+4`), which is the interface Task 8's two new arms actually consume. A refactor that moves the allocation to, say, `+5` keeps this test green and surfaces only as a downstream breakage in Task 8's code. | Extend the reservation unit test to assert the specific offsets, not just the count. |
| D3 | `tools/blast-radius/counts.json` still describes the pre-Task-9 argument guard: a stale line range and the old all-`Literal` condition. Left deliberately — correcting it would force a `FROZEN_PREDICATES_SHA256` re-freeze for a non-gating note field that no test checks (`task-9-report.md`, "Minor 2"). | Only worth doing at the next re-freeze this file needs for an unrelated reason; not worth a re-freeze on its own. |
| D4 | `crates/kali_codegen/src/emit/control_flow.rs` Arm A's comment describes Task 8's round-4 rule without round-5's bare-identifier restriction, and misnumbers the round it came from. | A comment-only correction; no behaviour change. |

## §11. Cross-cutting note for whoever makes cross-module calls real

kali's whole cross-module call lane is already silently wrong, independent
of this project: `import { K }` of `export const K = 7` yields `0`, and a
renamed imported function call yields `0`. The register documents this
already — it is not a new finding. §5's import-shadow gap (A3) is one
instance of the same underlying blindness, and closing either needs the
shadow-gate namespace extended at the same time as the cross-module call
lane itself, not as two separate pieces of work.
