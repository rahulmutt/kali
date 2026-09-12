# Defects the inline-allocation-value-position project measured and did NOT fix

**Filed** 2026-09-12 by the **inline-allocation-value-position** project
(`docs/superpowers/specs/2026-09-11-inline-allocation-value-position-design.md`),
on the convention `length-fails-closed-discovered-defects.md` and its
predecessors use: a project that measures more than it fixes writes down what
it left, so the silence is not read as absence.

**Oracle:** `node v26.8.2`. **Measured at:** each row's own task (Tasks 6-9,
cited per section) for §3's first two rows, §4, §7-§13 and §15-§16; §1, §2,
§3's third row (`Array.from`) and §5 were re-measured directly at this
branch's HEAD (`a09a468516`), on the existing binary
(`/workspace/.cache/cargo-target/debug/kali`, no rebuild), against
`node v26.8.2`, as part of filing this document — they come from the
project's own Task 10 brief rather than from `task-10-measurements.md`, and
are re-derived here rather than copied, per the brief's own instruction. **§6
was added later, on 2026-09-12**, by the branch's final whole-branch review,
measured at `b070ea5f82` on the same binary against the same oracle; the same
review corrected §12's B1/B2 provenance and widened §5's scope claim (both
noted in place). None
of these rows is proposed as a new register entry: §1 and §2 cross-reference
two *existing* entries (R-14, R-48) rather than filing new ones, and every
other row's own task report explicitly deferred it to this document rather
than to `kali-silent-miscompile-register.md`; §16 is the one exception, which
the register already documents independently.

This project's four register filings — **R-64** (an allocation outside the
materializing lanes evaluates to `0`), **R-65** (a fold-lane array argument
reads as zeros in the callee), **R-66** (a one-element literal of an
allocation IS that allocation), **R-67** (`.fill(v)` re-evaluates `v` per
element) — are all retired (CLOSED, 2026-09-11) and are not repeated here.
What follows is everything *adjacent* to that work that this project found
along the way and chose not to fix.

**Ranked, most consequential first, by "what a reader would work on next and
what it would cost."** §1-§2 are silent wrong values on two of the most
ordinary shapes there are — a returned allocation, an allocation held by an
object field — and are pre-existing, general escape/provenance defects this
project's own work never reaches; a reader chasing either should look at the
named register entry, not at this project's code. §3-§5 are silent-or-loud
wrong values immediately adjacent to the guards this project built, each with
a plausible, scoped next step. §6-§10 are silent wrong values further inside
the recognizer and shadow-checking machinery, in decreasing order of how
directly this project's own new code is implicated — §6 leads that group
because it is R-66's own defect in spellings the recognizer this project
shipped does not cover, and because it carries a coupling warning anyone
working on §8-§10 must read first. §11 is a silent wrong value this
project deliberately left out of scope on a correctness trade-off, not an
oversight. §12 groups the loud, fail-closed exceptions, which are already
safe and need no urgent attention. §13 is a behaviour this project changed
correctly but left unpinned — the cheapest item in this whole document to
close. §14 is an unaudited risk, not a confirmed defect — scoped work with an
unknown outcome. §15 is maintenance hazards and recorded refactor
suggestions. §16 is a cross-cutting architectural note for whoever makes
cross-module calls real.

---

## §1. A returned or passed-through allocation, rebound, reads zeros — cross-reference R-14

| program | node | kali |
|---|---|---|
| `function mk() { const a = new Array(3).fill(4); return a; } const b = mk(); console.log(b[1]);` | `4` | `0` |
| `function f(x) { return x; } const a = f(new Array(3).fill(1)); console.log(a[0]);` | `1` | `0` |

Both silent, exit 0, re-measured directly against this branch's HEAD. Neither
program is inside the shadow-checking or argument-guard machinery this
project built — both allocations reach a callee-argument-shaped lane just
fine on the way in (`mk`'s own body; `f`'s own body), and the value is
correct *inside* the function that first sees it. The defect is one level up:
once the allocation is **returned** (row 1) or **rebound to a brand-new local
after passing through a function** (row 2), the caller's read of it is `0`.

**Cross-reference `kali-silent-miscompile-register.md`'s R-14** ("an array
returned from a function reads back as all zeros," unclustered,
arena/escape-reclamation suspicion, untraced mechanism). R-14's own repro
returns a plain array **literal**; these two rows return/rebind a `new
Array().fill()` **allocation** instead, so this is not a byte-for-byte
duplicate of R-14 — but the shape (a heap-shaped value crossing a
function-return or rebinding boundary) and the symptom (exact `0`) match
R-14's own description closely enough that they are very likely the same
underlying mechanism, triggered by one more kind of value. This project's own
allocation-materializing work (Tasks 6-9) only ever handles a lane that
*directly* consumes an allocation — the declarator initializer, the
assignment right-hand side, the `.fill` receiver, and, as of Tasks 8-9, a
call argument. None of those lanes is "the value that comes back out of a
`return` and gets bound to something new," which is the shape both rows
above exercise.

**What it would cost:** whatever fixes R-14 almost certainly fixes this too —
it is very likely the identical mechanism, not separately scoped work. A
project picking up R-14 should re-run these two rows as discriminating
controls once R-14's mechanism is understood, to confirm the allocation case
moves with the literal case rather than needing its own fix.

## §2. An allocation held by an object property reads zeros — cross-reference R-48

| program | node | kali |
|---|---|---|
| `const o = { arr: new Array(3).fill(6) }; console.log(o.arr[0]);` (module scope) | `6` | `0` |
| `function main() { const o = { arr: new Array(3).fill(6) }; console.log(o.arr[0]); } main();` (in-function) | `6` | `0` |

Both silent, exit 0, both scopes, re-measured directly against this branch's
HEAD. An allocation initialized directly as an object-literal property's
value is read back as `0` through that property, in both scopes identically.

**Cross-reference `kali-silent-miscompile-register.md`'s R-48** ("an array
stored into an object field typed `I64` reads back `0`" — the field's repr
is fixed at `I64` by its own initializer/inference, and a later array value
neither widens the field nor fails closed, so the handle is truncated/lost).
R-48's own repro **stores** an array into a field initialized to a plain
number (`let o={a:6}; o.a=[1,2];`); this row instead initializes the field
**directly** to the allocation, with no separate store — a different code
shape, but R-48's own text already flags it as "≈ R-14 (an array losing
provenance across a boundary)," and this row's symptom (exact `0`, both
scopes, on a value the register already names for a very similar shape)
fits that same family. This project's work never touches object-field
representation at all, so it could not have affected this row either way.

**What it would cost:** whatever fixes R-48 (or the broader provenance
family R-48 and R-14 both belong to) is the relevant work here too — a field
whose declared/inferred repr is scalar needs to widen to hold a handle, or
the store/init needs to fail closed rather than truncate. Not this project's
scope.

## §3. Three call-shaped array producers slip past the widened argument guard

(This section is about what still slips *past* the guard. The guard's own
*capability loss* — which previously-working programs it newly refuses — is a
separate question, recorded in R-65's retirement in
`kali-silent-miscompile-register.md`, which the branch's final whole-branch
review widened from `new C()` / `new AbortController()` to the whole
`new X(…)` family.)

Task 9's guard (the `_refuses` fix behind R-65) is keyed on **text-less,
aggregate** LIR nodes — the shape a fold-lane array literal or a constructed
value lowers to. A call whose *result* is an array, but which is itself an
ordinary `Call` node reached through a different recognizer entirely, is
invisible to it:

| program | node | kali |
|---|---|---|
| `function f(x) { return x.length; } console.log(f(globalThis.Array(5)));` | `5` | `0` |
| `function f(x) { return x.length; } const s = "a,b,c"; console.log(f(s.split(",")));` | `3` | `0` |
| `function f(x) { return x.length; } console.log(f(Array.from({length: 3})));` | `3` | `0` |

All three exit 0 with no diagnostic; the third row re-measured directly
against this branch's HEAD. The first two are "adjacent holes" outside
R-65's own nine pinned lanes: call-shaped array producers reaching
`emit_value`'s placeholder in call-argument position — a sibling class to
R-65's fold-lane-literal shape, not covered by Task 9's fix, which is keyed
on array-*literal*/constructed-value LIR nodes, not on arbitrary call results
(`task-9-report.md`, "Filed for Task 10"); out of scope for Task 9 by the
review's own instruction, and recorded there for this document to fold in.
The third row goes through a **completely different recognizer**,
`is_array_from_call`, which this project's widened guard never touched at
all — a third, structurally separate reason the same symptom (an
array-shaped call result reaching the placeholder) recurs.

**What it would cost:** the first two would need the guard (or a sibling of
it) to key on "a call whose *callee* is a known array-returning
builtin/method" (`globalThis.Array`, `.split`, and presumably `.slice`,
`.map`, `.filter`, `.concat`, and any other array-returning method not yet
audited), not on LIR node shape — a different, and likely larger, recognizer
surface than the one this project built. The third would need
`is_array_from_call`'s own callers routed the same way `resolve_array_alloc_call`
now is. Sizing either needs its own sweep (of array-returning
builtins/methods, or of `is_array_from_call`'s call sites) before scoping a
fix; the two are not the same piece of work just because they share a
symptom.

## §4. An unvalidated allocation size argument is loud for a fraction, silent for a negative integer

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

**The negative row's severity is worse than a wrong printed number
(sharpened 2026-09-12 by the branch's final whole-branch review).** The
mechanism is a memory-safety defect, not a display defect:
`emit_array_allocation_with_len` computes the byte size as `(n + 1) * 8`,
which for `n = -1` is `(-1 + 1) * 8 = 0`; it then calls `__alloc(0)` and
**stores an 8-byte length header at the returned pointer — a write past the
end of a zero-byte allocation**, into whatever the next bump hands out. The
printed `-1` is the visible symptom; corrupting an adjacent allocation is the
actual cost, and it is not observable from the output at all. Task 8 widened
this from the declarator lane to every value position, so the out-of-bounds
write is now reachable from argument, return, property, element and
ternary-arm positions too.

**What it would cost:** validate the size argument before allocating (reject
a non-integer or negative value with a real diagnostic, matching node's
`RangeError`). This is a size-argument validation gap, not a routing gap —
smaller in scope than §3, but untouched by this project because size
validation was never in its brief.

## §5. EVERY float `.fill` produces an invalid module — loud, not silent

| program | node | kali |
|---|---|---|
| `function f(x) { return x[0]; } console.log(f(new Array(3).fill(1.5)));` | `1.5` | `error[E4201]`, failed to load WASM module (exit 1) |
| `function f(x) { return x[0]; } const a = new Array(3).fill(1.5); console.log(f(a));` (bound form) | `1.5` | `error[E4201]`, failed to load WASM module (exit 1) |
| `const b = new Array(3).fill(2.5); console.log("ok");` (**no call at all; the array is never read**) | `ok` | `error[E4201]`, failed to load WASM module (exit 1) |

Re-measured directly against this branch's HEAD; the third row was added
2026-09-12 by the branch's final whole-branch review, which found this
section's original scope claim ("passed to a function") too narrow. **The
failure needs no call, and no read of the array, at all: EVERY float `.fill`
produces an invalid module**, whatever is done with the result. **This one is
loud, not silent — worth keeping visible as a different class of cost to the
next reader than every silent row above it.** The inline, bound and
never-passed spellings fail identically, so this is not specific to the
inline-argument routing this project built — nor to argument position at all;
it is a property of emitting a float `.fill`. The mechanism was not
traced by reading the emitter, but the shape (only the *float-valued*
`.fill` fails; the integer-valued `.fill` cases throughout this document
materialize correctly) suggests a `.fill` value's scratch slot is typed for
the integer case and a float value does not fit it, producing an invalid
compiled module rather than a valid one with a wrong number in it. The
widened scope does not disturb that hypothesis — it is right in kind, and the
no-call row is if anything cleaner evidence for it, since nothing downstream
of the `.fill` is involved at all.

**What it would cost:** this is a compile-failure/repr-mismatch bug, not a
wrong-value or a routing bug — diagnosing why a float `.fill` value produces
an invalid WASM module (likely a scratch-slot type mismatch) is a different,
and probably smaller, task than any guard-widening item above. Because it
already fails closed (loud, exit 1, no silent wrong value shipped), it is
lower urgency than §1-§4 despite being closer to this project's own recent
work on `.fill`.

## §6. The allocation recognizer and codegen are NOT in lockstep for a qualified object that is not bare `globalThis`

| program (with `const a = {globalThis: {Uint8Array: function (n) { return n; }}};` in scope) | node | kali |
|---|---|---|
| `const xs = [a.globalThis.Uint8Array(5)]; console.log(xs.length);` | `1` | `5` |
| `const xs = [new a.globalThis.Uint8Array(5)]; console.log(xs.length);` | `1` | `5` |
| `const xs = [a["globalThis"].Uint8Array(5)]; console.log(xs.length);` | `1` | `5` |
| `const xs = [globalThis.globalThis.Uint8Array(5)]; console.log(xs.length);` | throws | `5` |

All four are silent in kali — exit 0, no diagnostic — measured at this
branch's HEAD (`b070ea5f82`). (Row 4 names the real builtin, so node throws
rather than printing `1`; kali still answers `5` silently, which is the same
divergence class.)
**This is R-66's exact defect — a one-element array literal read as the
allocation itself — in a spelling family R-66's retirement does not cover.**

The mechanism is a false lockstep claim, not a missing guard.
`expression_is_array_allocation`'s rustdoc
(`crates/kali_types/src/resolve/expression.rs`) asserts it stays in lockstep
with `FunctionEmitter::is_array_like_constructor`
(`crates/kali_codegen/src/emit/call.rs`), naming exactly one exception
(`new new Array(3)`). The two disagree on a whole family besides:
`is_array_like_constructor` accepts any callee object node whose **text** is
`"globalThis"`, and a member-expression node carries its *property* name as
its own text — so codegen reads `a.globalThis.Uint8Array` as
`globalThis`-qualified, while `is_global_this_uint8array` requires a bare
`Identifier` and declines. The literal is therefore never refused, and the
declarator lane answers the allocation's length.

**Pre-existing, not a regression.** The declarator lane and
`is_array_like_constructor` are byte-identical to this branch's merge base, so
every row above measures the same before Task 6. R-66's retirement in
`kali-silent-miscompile-register.md` now records the same exclusion, and both
rustdocs now name this family.

**Task 8's two new routing arms are NOT affected, and the reason is a
coupling worth stating out loud.** Arm A (`emit_value`) and `emit_call`'s
bare-call arm stay sound here only because `allocation_ctor_unshadowed`
returns `false` for any qualifying object that is not a bare identifier — it
declines exactly the shapes `expression_is_array_allocation` fails to refuse.
That line is justified in its own rustdoc as *shadow identity*, and §10 below
invites a future project to replace it with real identity resolution.
**Relaxing it without widening the recognizer in the same change reopens R-66
in Arm A.** Both rustdocs, and Arm A's own comment, now carry that warning;
it is repeated here because a reader picking up §8/§9/§10 is the person most
likely to relax it.

**What it would cost:** widening `is_global_this_uint8array` to match
codegen's text rule exactly (accept any object whose text is `"globalThis"`)
would close the collision but would ALSO refuse `a.globalThis.Uint8Array(5)`
in the declarator lane, where the object is a user value and the refusal is
wrong in a different direction — so the honest fix is the same
identity-resolution work §8-§10 name, applied to both sides at once, not a
one-line widening of either. There is no cheap interim fix, and none is
urgent — the shape is exotic — but whoever touches either side must move
both, which is what the coupling note above exists to enforce.

## §7. The declarator lane and Arm B were never gated against a shadowed constructor name

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

## §8. Two remaining identity gaps in the globalThis-qualified shadow gate

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
identically before Task 8, on the same ungated declarator lane §7 names. The
second: pre-existing, routed identically before Task 8's gate — node throws
because `globalThis["Uint8Array"]` is not a constructor without `new`, and
kali's placeholder still fires.

**What it would cost:** the first needs the same declarator-lane wiring as
§7. The second needs `is_array_like_constructor` (or its caller) to
distinguish a bracket-property access from a dot-property access, which
today it does not; both are narrower instances of the same "matched by text,
not identity" limitation named in `allocation_ctor_unshadowed`'s own rustdoc.

## §9. The shadow gate is blind to imports

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
this project's work. §16 below is the broader cross-module-call defect this
row is one instance of.

**What it would cost:** the shadow gate needs a name set threaded from the
module linker into codegen — a structural change reaching outside
`kali_codegen`, and shared with `url_ctor_unshadowed`'s identical gap, so a
fix should close both gates' import blindness at once rather than
special-casing the allocation gate alone.

## §10. A nested-`globalThis` casualty this project could not cheaply avoid

| program | node | kali |
|---|---|---|
| `function f(x){return x.length;} console.log(f(new globalThis.globalThis.Uint8Array(5)));` | `5` | `0` |

**NET-NEUTRAL against the pre-Task-8 baseline `733cd26125`, where this program
was also `0`.** Task 8 round 4 incidentally made it correct (`5`); round 5's
bare-identifier restriction (§8's "exactly one shape" rule) returned it to
the baseline value. This is not a regression this project introduced and
left — it is a pathological spelling nothing was ever designed to handle,
that briefly worked by accident and now doesn't, at the same value it had
before this project began (`task-8-report.md` §15.4).

**What it would cost:** there is no cheap correct fix. A blanket
"decline ⇒ deny (refuse)" was considered and rejected: it would re-break the
ordinary case where a *declined* bare `Uint8Array(3)` must fall through and
call the user's own function rather than refuse. Closing this needs the same
identity-resolution work as §8 and §9, generalized to an arbitrary property
chain, not a special case for two levels of `globalThis`.

## §11. `.fill` on a bound receiver inside an array literal — deliberately left open

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

## §12. Loud, fail-closed exceptions — safe as-is, no urgency

These three all fail closed (exit 1, node-divergent, or an intentional
exception). **Two of the three ARE new — introduced by this branch's Task 6**
(corrected 2026-09-12 by the branch's final whole-branch review, which found
the earlier "none of them is new" framing and B1's "pre-existing" label
wrong). The deferral is unchanged and still right: both are loud refusals,
not wrong values, and both are exotic. Only the provenance was misstated.

| # | program | node | kali | status |
|---|---|---|---|---|
| B1 | `[new Array(3).fill(0).fill(1)]` | `1` | refuses | **Introduced by Task 6** (`81b0c409e8`), which is the array-literal-of-an-allocation recognizer's FIRST landing — so "pre-existing since that landing" was self-contradictory. At the merge base this printed `1`, matching node: `array_fill_call_parts` does not accept a fill-chain receiver, nothing recognized the element as an allocation, and the literal lane answered correctly. Same-lane control at HEAD: `[g()]` → kali `1`, node `1`. At HEAD it refuses. Accepted as the safe direction. |
| B2 | `new (await globalThis).Uint8Array(3)` | — | refuses | **Introduced by Task 6**, same story: codegen reads the `await` node's text as `"await"` and declines, so the baseline answered `1`; the new `kali_types` recognizer unwraps `await` and refuses independently. Exotic; accepted as the safe direction. |
| B5 | `[new new Array(3)]` | throws `TypeError` | `3` | A documented, named exception in `expression_is_array_allocation`. Cannot affect a valid program: a `new` expression's result is never itself constructible, so node throws for *every* value the inner `new` can produce — there is no input on which kali's `3` and node's behaviour could both be observed as a program result. Genuinely not new in the sense that matters: it is an un-refusal, not a new refusal. |

**What it would cost:** nothing needs doing here. B1 and B2 are new refusals
of programs that used to run correctly, but each is loud, exotic, and in the
safe direction (a wrong program refuses instead of running with a wrong
value) — the cost is a narrowed capability, not a wrong answer, and it is
recorded rather than reversed. B5 cannot diverge on any program node itself
accepts. Recorded so a future sweep does not re-discover and re-triage them
as if they were unknown — and, for B1 and B2, so nobody reads "pre-existing"
and looks for the cause outside this branch.

## §13. An unpinned behaviour change: `.fill` on a zero-length array now evaluates its argument

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

## §14. An audit left undone: the 36 scratch-slot holders' mutual safety is unproven

This project (Task 7) took allocation and `.fill` off the list of things
that clobber the function-body trailing scratch reservation, by giving each
its own dedicated slot instead of sharing space with whatever else happened
to be using it. Before that fix, allocation and `.fill` were two of (at
least) 36 total holders of scratch slots across the codebase — a count from
the project's own planning, not independently re-derived here — sharing
space in ways nothing has proven are individually safe against each other.
Removing two known-bad holders is not an audit of the other ~34: nothing in
this project, or (so far as this project's own task reports show) any prior
one, proves that the remaining holders cannot clobber each other's slots the
same way allocation and `.fill` used to.

No specific failing program is known for this row — it is an unaudited
*risk*, not a confirmed defect with a repro, which is why it ranks below the
measured wrong values above despite potentially being a larger source of
future silent miscompiles than any single row in this document.

**What it would cost:** enumerate the remaining scratch-slot holders and
either prove pairwise non-interference or find another instance of the same
"two holders share a slot and clobber each other" bug this project just
fixed for two of them. Scoped work someone could pick up on its own account,
with a genuinely unknown outcome — it might find nothing, or it might find
this project's own bug class recurring elsewhere.

## §15. Maintenance hazards

None of these are wrong values — they are traps for the next person to touch
this code:

| # | hazard | what it would cost to close |
|---|---|---|
| D1 | `crates/kali_types/src/resolve/expression.rs` has **two** same-named recognizers: an associated `Self::expression_is_array_allocation` (handles `Array` only — no `Uint8Array`, `await`, `as`, `satisfies`, or `globalThis`) and the free function this project hardened. The gate calls the free one correctly, but neither's doc comment mentions the other exists. | A doc-comment cross-reference on both, naming which one the gate actually calls and why the other is narrower. No behaviour change. |
| D2 | The Task 7 reservation test pins the trailing scratch reservation's SIZE (2→5 slots) but not the INDICES (`+2`/`+3`/`+4`), which is the interface Task 8's two new arms actually consume. A refactor that moves the allocation to, say, `+5` keeps this test green and surfaces only as a downstream breakage in Task 8's code. | Extend the reservation unit test to assert the specific offsets, not just the count. |
| D3 | `tools/blast-radius/counts.json` still describes the pre-Task-9 argument guard: a stale line range and the old all-`Literal` condition. Left deliberately — correcting it would force a `FROZEN_PREDICATES_SHA256` re-freeze for a non-gating note field that no test checks (`task-9-report.md`, "Minor 2"). | Only worth doing at the next re-freeze this file needs for an unrelated reason; not worth a re-freeze on its own. |
| D4 | ~~`crates/kali_codegen/src/emit/control_flow.rs` Arm A's comment describes Task 8's round-4 rule without round-5's bare-identifier restriction, and misnumbers the round it came from.~~ **CLOSED 2026-09-12** by the branch's final whole-branch review: the comment now states the final narrowing (only a BARE identifier named `globalThis` is admitted; every other qualifying object is declined outright, with no namespace lookup), names it as what makes the arm sound against §6's spelling family, and carries §6's coupling warning. The round numbers were dropped rather than renumbered — `control_flow.rs` and `call.rs` disagreed by one and this review had no independent basis to adjudicate which index was right — and the final narrowing is now cited by commit (`4f9298fe37`) instead. | Done; comment-only, no behaviour change. |
| D5 | **Recorded suggestion, deliberately NOT applied** (reviewer's, from the final whole-branch review): the pairing of `allocation_ctor_unshadowed` (the outer guard) with `resolve_array_alloc_call` (the recognizer) is spelled out by hand at each of the two Task 8 routing sites, so a **third** routing site added later could call `resolve_array_alloc_call` and forget the guard — which would be **unsound**, not merely untidy: it would route a user's own `function Uint8Array(n)` through the allocator (`4104` where node prints `4`), the exact review-found regression the guard exists to close. The reviewer proposed encapsulating the two into one helper so the pairing cannot be split. Not applied because this fix wave was documentation-only and is the last work before hand-off: there is no second wave to catch a refactor error. | Introduce a single helper that performs the guard and the resolve together, and route all call sites through it, so the unsound half-call is not expressible. Small and mechanical, but it touches emitted-code paths, so it needs its own change with the gate and the `allocation_ctor_shadow` pins re-run. |
| D6 | **Recorded suggestion, deliberately NOT applied** (same review): `allocation_ctor_unshadowed`'s NAME misleads. It returns `false` in cases that have nothing to do with shadowing — most importantly for any qualifying object that is not a bare identifier (§6's family, §8's second row), where nothing is shadowed and the guard declines on identity-resolvability grounds instead. A reader who trusts the name will mis-predict the function, and §6's coupling makes that mis-prediction dangerous. Not applied for the same reason as D5: a rename touches every call site, and there is no second fix wave. | Rename to something that names the real postcondition (e.g. "the callee is provably the builtin allocator"), updating both call sites, the `call_tests/allocation_ctor_shadow.rs` references, and the rustdoc cross-references in `expression.rs`, `control_flow.rs` and this document. Mechanical but wide; worth doing at the same time as D5. |

## §16. Cross-cutting note for whoever makes cross-module calls real

kali's whole cross-module call lane is already silently wrong, independent
of this project: `import { K }` of `export const K = 7` yields `0`, and a
renamed imported function call yields `0`. The register documents this
already — it is not a new finding. §9's import-shadow gap is one instance of
the same underlying blindness, and closing either needs the shadow-gate
namespace extended at the same time as the cross-module call lane itself,
not as two separate pieces of work.
