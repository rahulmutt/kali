# Defects the class-instances project measured and did NOT fix

**Filed** 2026-10-04 by the **class-instances** project
(`docs/superpowers/specs/2026-10-04-class-instances-design.md`), on the
convention `unresolved-member-call-discovered-defects.md` uses: a project that
measures more than it fixes writes down what it left, so the silence is not
read as absence.

**Oracle:** `node v26.10.0`.
**Measured at:** §4 and §5 at `bb2411ea6` (branch `class-instances`), on
`target/debug/kali` built from that commit (`cargo build -p kali_cli`, the
`dev` profile), with the re-pins at `109dda3ed`. The baseline binary was built
from `7c4daa9f7` in a separate worktree. Probe rows come from
`tools/array-return-probes/probes/cls_*.js` (runner
`tools/array-return-probes/run.sh`); a probe's baseline column is
`tools/array-return-probes/baseline-cls.tsv`, measured at `7c4daa9f7`. The
§6 plain-object-lane items (4 onward) come from controller and reviewer
probes at HEAD during implementation.

**What ships.** The spec's §1 slice was narrowed during implementation by
controller rulings R-13 to R-20, so the list below is the contract, not the
spec's original §1. An in-slice class has no `extends` link to another
program class and no accessor, `static`, `#private` or computed member, and is
a class declaration, not exported (detected only for `export default class` and `export { C }`; `export class C` is not seen, R-11, §6.6), not declared twice, and constructed. For
one, `new C(a)` runs field initializers then the constructor; `this.f` /
`s.f` read and write; `s.m(a)` binds `this`; instances flow through the
parameters and returns of program function declarations and methods (tracked
call sites only); same-named methods in different classes dispatch per class.
Refused with E5506, each because the plain-object lane computes a silent
wrong value in that shape: R-13 (an instance returned from a function whose
callers are not all tracked), R-14 (`arguments` in a function taking an
instance), R-15 (a field access whose receiver is not a variable), R-16 (an
instance, including `this`, inside an arrow or function expression, so the
spec's "`this` in an arrow inside a method" does not work), R-16b (an
instance captured from an enclosing function), R-6 (`recv["m"](…)`), and
`typeof` / `instanceof` / `===` on an instance or the class as a value
(§3.3). R-17: compound assignment and `++` / `--` on an instance field stay
refused by the resolver, so spec §3.5 is not reachable. Stateless out-of-slice
classes keep the baseline lowering, except that `new X().m()` / `new X().f`
of any program class kali does not lower is refused (R-29).

**Final-review narrowing (rulings R-22 to R-30r, 2026-10-04).** The final
whole-branch review found clean-compiling programs whose output differed from
node; each is now refused (or, for R-22, fixed), and the slice is narrower
than the paragraph above on its own suggests. A class kali would lower is
refused at `new` when a field may hold anything but a number or a boolean
(a string, `null`, `undefined`, a BigInt, an array, an object or a function;
R-23, proven by a value-kind fixpoint in the pass and backed by the
re-inferred field reprs); when its body reads a variable of an enclosing
function, so a class declared in a function may read only its own parameters
and locals and program-level names (R-25); when a field initializer names a
parameter or variable of its constructor (R-26); when a field and a method
share a name (R-27); and when a name it generates collides with another
class's generated name or with any name the program spells, function
expression and arrow ids and import locals included (R-30r). An instance
field under `typeof` (R-24) and an instance inside `as` / `<T>` /
`satisfies` (R-28) refuse at the use. R-22 fixed the parser: a field type
closed by `>>`, `>>>`, `>=`, `>>=` or `>>>=` no longer swallows the rest of
the program. Each repro is a case in
`crates/kali_cli/tests/cases/object/class_instances.toml` (`r22_*` to
`r30r_*`), whose rationale gives node's output and kali's at `d75a10075`.

---

## §1. The R-30 boolean field

`class F{ constructor(){ this.ok=false; } set(){ this.ok=true; } }
const f=new F(); f.set(); console.log(f.ok);` prints `1` (node `true`).
Probe `cls_known_r30`; at `7c4daa9f7` it printed `0` (SILENT), now `1`. A
boolean instance field is a number in the object-literal lane, so it always
renders `1` / `0`, whether or not it was written after construction:
`class C { constructor(){ this.v = true; } } const a = new C(); console.log(a.v);`
with no later write prints `1` too (final-review repro d4; node `true`). The class
rewrite does not change that; R-23 deliberately admits booleans. Pinned as the
known-wrong control `object/class_instances::known_r30_the_bool_field_still_prints_one`,
so a fix shows up as a diff. Not fixed.

## §2. Out-of-slice refusals (future items)

Each is refused with E5506 at `new` when the class keeps state (a stateless
one keeps the baseline lowering). Each is a candidate future slice, not a
defect.

- **Program-class `extends` with state.** The class is in an `extends` chain
  with another program class (`is in an `extends` chain with another program
  class and keeps state`). Needs field layout across the chain and `super`.
- **Accessors.** `get` / `set` members.
- **Statics with state.** `static` fields and methods on a class that keeps state.
- **`#private` members.**
- **Computed members.** `[expr]() {}` and computed field names.
- **Class expressions.** `const C = class { … }`.
- **Exported classes.** A class that is exported and keeps state. Ruling R-11 (§6) means the exported escape cannot even be seen today.
- **Duplicate-named classes.** A class declared more than once.

## §3. Containers, rendering, `instanceof` / `===`, mixed-class parameters, `new` of a plain function

Refused with E5506; future items.

- **Containers of instances.** An instance as an array element (or an object
  value) refuses; an element read would be a zero placeholder.
- **Rendering an instance.** `console.log(c)` and string conversion of an
  instance refuse; the object-literal lane prints no node-shaped output for it.
- **`instanceof` / `===` on instances, `typeof c`, and the class as a value.**
  Refused (spec §3.3). The same refusal pins the five `runtime_smoke`
  fixtures (§4).
- **Mixed-class parameters.** A parameter or binding that may hold instances
  of two classes, or an instance and another value, refuses
  (`may hold an instance of class `C` and other values`).
- **`new` of a plain `function`.** Refused (`constructing an object with the
  plain function `f``), including a function that returns an object (§6.3).

## §4. The triage table

From Task 10. `cargo test --workspace --no-fail-fast` at `bb2411ea6` failed
five cases (`kali_cli --test cases`) and five Rust integration tests
(`kali_cli --test runtime_smoke`); every other target, including the
package-corpus targets, passed. `cargo test -p kali_cli --test cases --
--ignored` showed one ignored case moving from fail to pass. That is 11 moved
trials, under the stop rule's 50. None is capability loss class 1.

| name | before (`7c4daa9f7`) | after (HEAD) | class |
|---|---|---|---|
| `runtime/inline_allocation_value_position::a_constructed_argument_refuses` (`class C{constructor(){this.v=4}}`, `f(new C())`) | E5506 inline-allocation guard (`the callee would read zero placeholders`), exit 1 | `4`, exit 0 (node: `4`) | **wanted**: a refusal became correct. `C` is in the slice, so `new C()` is a factory call before IR. Re-pinned to `4`. |
| `soundness/block_arrows::a_function_expression_body_supports_compound_and_typeof_and_new` (`function Box(v){this.v=v}`, `new Box(9)` in a function expression) | `4 5` / `boolean`, exit 0 (node: the same) | E5506 `constructing an object with the plain function `Box``, exit 1 | **capability loss, class 3**: `b` is never read; the case pins compound / `typeof` in a function-expression body. Re-pinned to the refusal; R-20 restores that coverage in a new case without `new Box`. |
| `soundness/block_arrows::a_feature_rich_block_arrow_callback_defers_with_correct_ordering` (`class Box{constructor(){this.n=4}}`, `new Box()` inside a `queueMicrotask` arrow) | `MODULE-END-acc` / `0` / `INSIDE-CALLBACK` / `15`, exit 0 (node: the same) | E5506 `using an instance of class `Box` as a value inside an arrow function or function expression` x2, plus the A-10 backstop `constructing class `Box` … reached code generation without being lowered`, exit 1 | **capability loss, class 3**, under **R-16**: `probe = b.n + value` is never observed. The spec predicted this program would run node-correct; R-16 refuses it instead. Re-pinned to the refusal. Ordering stays pinned by `a_queued_microtask_callback_actually_runs_during_the_drain` and the two `deferred_*` cases. |
| `soundness/block_arrows::class_method_bodies_return_their_value` (`ignore = true`; `new C().run()`) | `0`, exit 0 (node: `42`) | `42`, exit 0 | **wanted**: a silent wrong value became correct. It now passes under `-- --ignored`. A dated note was added, and it was un-ignored under ruling R-19; the cases README's ignored-case text now names only `soundness/r06_object_init.toml`. |
| `switch/fail_closed::a_new_invocation_site_of_the_enclosing_function_is_fail_closed` (`new s(true)` on a plain function holding a `switch`) | E5506 switch Rule 1 (`the discriminant is not a proven integer or string`), exit 1 (node: `a=one`) | E5506 `constructing an object with the plain function `s``, exit 1 | **rationale only**: still a refusal; the class-instances pass refuses first, so the switch rule is not reached. Needle and note updated. |
| `switch/fail_closed::a_new_expression_call_site_denies_a_string_parameter_discriminant` (same shape, string axis) | E5506 switch Rule 1, exit 1 (node: `v=3`) | E5506 `constructing an object with the plain function `s``, exit 1 | **rationale only**: same. The `REPR_MIXED_CONFLICT` absence claims still hold. |
| `runtime_smoke::test::json_test_supports_object_type_and_constructor_semantics` (`.ts`; `function Box(){}`, `new Box()` in a `Kali.test` arrow, then `typeof` / `instanceof` checks) | `errors: []`, `failed: 1`, harness stderr `Uncaught Error: expected object from constructor` + E4000 trap in `__kali_callback_43` (node: passes) | top-level `errors[0]` E5506 `constructing an object with the plain function `Box``, `total: 0`, `success: false` | **wanted**: at the baseline `typeof box` of the zero instance was not `'object'`, so the test failed on a wrong value; now it is refused at compile time. **R-18:** the fixture (`runtime_smoke.rs`, `object_type_and_constructor_semantics_source`) now uses `class Box { constructor() {} }`. That version is still refused (R-16 for the instance inside the `Kali.test` arrow, and the §1.1 allowlist for `typeof Box` / `instanceof Box`), so the test was re-pinned to assert only-E5506 `errors[]` with both messages. |
| `runtime_smoke::test::json_test_supports_object_type_and_constructor_semantics_in_js_input` | same as above (`.js`) | same | **wanted**; R-18, re-pinned to the refusal (same assertion). |
| `runtime_smoke::build::build_emits_browser_bundle_object_type_and_constructor_semantics_in_ts_input` (the same checks in an `async function objectTypeSmoke`, `build --bundle --api browser`) | bundle built, exit 0 | E5506 `constructing an object with the plain function `Box``, exit 1 | **wanted**: the baseline bundle built, but it evaluates the same wrong `typeof` of a zero instance and throws. The refusal moved from evaluation to build. **R-18:** the fixture (`browser_bundle_object_type_and_constructor_semantics_source`) now uses `class Box { constructor() {} }`. node prints `object type ok`; kali refuses the build with E5506 (instance as a unary / binary operand, class as a value: the §1.1 allowlist), so the test was re-pinned to the build failure and those messages. |
| `runtime_smoke::build::build_emits_browser_bundle_object_type_and_constructor_semantics_in_js_input` | same (`.js`) | same | **wanted**; R-18, re-pinned to the build failure. |
| `runtime_smoke::build::build_emits_browser_bundle_object_type_and_constructor_semantics_in_json_output` | same, `--output json`, `success: true` | same refusal | **wanted**; R-18, re-pinned to `success: false` with only-E5506 `errors[]`. |
| `soundness/textcodec::inline_decode_does_not_hijack_user_text_decoder` (`function TextDecoder(){ return {decode:…}; }`, `new TextDecoder()`) | E5506 TextEncoder byte-buffer + `calling 'decode'` refusals, exit 1 | E5506 `constructing an object with the plain function `TextDecoder``, exit 1 | **stderr only, no edit**: still asserts E5506 and passes. Not counted as moved. |
| `soundness/textcodec::inline_encode_does_not_hijack_user_text_encoder` | E5506 `calling 'encode'` refusal, exit 1 | E5506 `constructing an object with the plain function `TextEncoder``, exit 1 | **stderr only, no edit**: same. |

No oracle-tier trial moved: every `oracle/` step passed at HEAD, and none
printed a class-instances refusal in the §5 sweep.

**Rulings R-18 to R-20 (controller, 2026-10-04).** R-18: each of the five
`runtime_smoke` fixtures now constructs an in-slice `class Box` instead of
`new` on a plain function. All five still refuse with the class (the
fixture's point is `typeof` / `instanceof`, which the §1.1 allowlist refuses
on an instance or a class), so all five were re-pinned to the E5506
refusal; none asserts success. The ten other tests that share the two
fixtures (`run`, `test` without JSON, and the browser-requested variants)
already accepted E5506 and pass unchanged. R-19: the case above was
un-ignored. R-20: the new case
`soundness/block_arrows::a_function_expression_body_supports_compound_and_typeof`
carries the compound / `||=` / `typeof` body without `new Box` (node, the
baseline and HEAD all print `4 5` / `boolean`).

After the re-pins: `cargo test -p kali_cli --test cases` (`6302 passed; 0 failed; 1 ignored`) and
`cargo test --workspace` are green (results in the Task 10 report).

## §5. Measured capability loss

Task 10 measured two losses, both in spec §5.4 class 3 (the output never
depended on the zero instance), and no loss of class 1 or class 2:

1. `soundness/block_arrows::a_function_expression_body_supports_compound_and_typeof_and_new`:
   `new` on a plain function (spec §3.4). The constructed `b` is never read.
2. `soundness/block_arrows::a_feature_rich_block_arrow_callback_defers_with_correct_ordering`:
   ruling R-16 (an instance inside an arrow or a function expression). The
   instance's field feeds an unobserved local only. The spec (§5.4) expected
   this program to run node-correct; R-16 refuses it.

What was swept. Task 10 ran every `cli` and `oracle` step of every case file
(6,302 trials, with matrix and constants expanded, ignored cases included)
with HEAD's binary. For each trial whose stderr contained any of
`constructing class`, `constructing an object with the plain function`,
`of class `, `could not determine the class`, `using class ` or
`that kali would generate`, it re-ran the trial with the baseline binary
(`7c4daa9f7`). 38 trials matched:

| group | trials | baseline | HEAD | verdict |
|---|---|---|---|---|
| `object/class_instances::*_is_refused_under_{run,check}` (16 programs) | 32 | `run`: 12 programs exit 0 printing `0` (silent), 4 refused or trapped for another reason (`r_ambiguous` E4201, `r_instanceof` runtime `instanceof` trap, `r_mixed_param` inline-allocation E5506, `r_unresolved_recv` exit 1 with only an E3100 warning); `check`: exit 0 on every one. node prints a non-`0` value for each of the 12 silent programs | E5506 under `run` and `check` | this project's own pins (Task 9 and the R-15/R-16/R-16b follow-ups); not a loss |
| `switch/fail_closed::a_new_*` | 2 | E5506 switch Rule 1 | E5506 plain function | rationale only (§4) |
| `soundness/block_arrows::*` | 2 | exit 0, node-correct | E5506 | class 3 (items 1 and 2 above) |
| `soundness/textcodec::inline_*_does_not_hijack_*` | 2 | E5506 other text | E5506 plain function | stderr only (§4) |

No trial printed `that kali would generate`. The Rust integration targets
were measured by `cargo test --workspace --no-fail-fast`, not by the sweep:
only the five `runtime_smoke` tests in §4 moved, all wanted. A capability
loss that no case, fixture or corpus program exercises was not measured.

## §6. Other measured items

1. **Task 0 baseline rows that differed from the plan's expectations**
   (`tools/array-return-probes/baseline-cls.tsv` at `7c4daa9f7`; the probes
   were not changed to fit):
   - `cls_ok_user_push` REFUSES (expected CORRECT), via the literal-array
     `.push()` lane.
   - `cls_compound`, `cls_param` and `cls_r_mixed_param` REFUSE (expected
     SILENT), via other lanes.
   - `cls_r_unresolved_recv` REFUSES with only an E3100 warning; `check`
     exits 0.
   - `cls_in_main`, `cls_r_ambiguous` and `cls_same_method` are OTHER
     (E4201, invalid wasm).
   - `cls_r_instanceof` is OTHER (runtime unsupported `instanceof`).
   - `cls_this_method` is CORRECT at the baseline.
2. **The A-10 backstop repeats a refusal (Task 10).** When a use refusal fires
   (R-15, R-16, array element, binary operand, mixed parameter, unresolved
   receiver), the `new` stays unrewritten, so the sweep also prints
   `constructing class `C` … reached code generation without being lowered`.
   R-16 refusals can also print the same line twice
   (`a_feature_rich_block_arrow_callback_defers_with_correct_ordering`,
   `this_arrow`). The program is refused either way; only the stderr is
   noisy.
3. **`new` of a plain function that returns an object (Task 10).**
   `function TextEncoder(){ return {encode:…}; } new TextEncoder()` is
   refused by the plain-function rule. node uses the returned object. It was
   refused at the baseline too (§4), so this is not a loss.

4. **R-13, R-15, R-16 and R-16b refuse because of plain-object-lane defects.**
   The controller measured these at HEAD with no classes involved. The class
   rewrite refuses rather than reach them; none is fixed.
   - `function mk(){ return {n:3}; } console.log(mk().n);` prints `0` (node `3`).
   - An object captured in an arrow reads `0` and loses its writes.
   - An object captured in a function expression loses its writes.
   - An object passed as a parameter to an arrow and written there loses the write.
   - An object captured by a nested function declaration and passed on to
     another function loses the write.
   Fixing the lane would let the matching refusal be lifted.
5. **R-17: member compound assignment and update are refused.** The resolver
   refuses `o.n += x` and `o.n++` on any object member
   (`resolve/expression.rs` about lines 2104 and 2267), before the rewrite
   runs. Spec §3.5 (compound and update on instance fields) is unreachable,
   and no document may claim it.
6. **R-11: the parser drops `export`.** `kali_parser/src/module.rs` about
   lines 131-142 drops `export` from `export function` and `export class`.
   Spec rule 4's "exported" escape and A-8's "exported class" therefore
   cannot be seen. A library-mode export proven to take an instance could be
   miscompiled if the host calls it.
7. **`eval` compatibility.** With the opt-in `eval` flag,
   `const y = eval('c')` gives an Unknown binding, and field access on it is
   not refused.
8. **TS overload signatures parse as bodiless `MethodDefinition`s.**
   Constructor overloads make the factory take the empty first `constructor`;
   the real body's field writes then refuse as outside the field set, which
   is fail-closed. Method overloads produce duplicate `function C__m`
   declarations.
9. **Duplicate field initializers.** `n = 0; n = 1;` emits two `let __f_n`.
10. **`C__new` is hoisted.** Where the class is in its temporal dead zone,
    `new C()` before `class C` runs, where node throws.
11. **Scopes are per function frame, not per block.** Two same-named
    functions in one frame share a frame key. Both choices are conservative
    at most sites, but a block-shadowed name can resolve to the wrong binding.
12. **R-6: `recv["m"](…)` on an instance refuses** (computed access) rather
    than dispatching; a literal-key bracket call is not rewritten.
13. **The Task 2 annotation heuristic's limit.** (R-22 fixed a separate bug
    here: a closing `>>`, `>>>`, `>=`, `>>=` or `>>>=` now closes as many
    `<` as it has `>`, and a fused trailing `=` starts the initializer.) Tokens carry no line
    breaks, so `skip_field_type_annotation` (`kali_parser/src/declaration.rs`)
    ends a field's `: Type` where two identifiers meet at depth 0, unless the
    first is one of `TYPE_WORDS` (`keyof`, `typeof`, `readonly`, `infer`,
    `unique`, `asserts`, `is`, `extends`, `new`). A field type that ends in
    one of those words, followed by a newline and the next member's key,
    swallows that key into the type, so the next member is lost. The heuristic
    is not a type parser.
14. **R-31: a second monomorphization pass makes clones share an anonymous
    function name.** The rewrite runs `monomorphize_statements` again after
    `name_anonymous_functions`, so when a method (or any function) is cloned
    for two object-parameter shapes, both clones carry the same
    `__kali_fn_N` for an inner arrow, and the module fails with E4201
    (invalid wasm) under `run` while `check` exits 0. Repros: `class C {
    constructor(){ this.n = 1; } use(o){ const g = (y) => y + 100; return
    g(o.a) + this.n; } }` called with `{ a: 5 }` and `{ b: 7, a: 9 }` (m1;
    node `106` / `110`), and the same with two arrows (m2; node `1011` /
    `1019`). Not silent (E4201), not fixed.
15. **R-31: block scoping without classes.** A block-scoped `const` that
    shadows a parameter replaces it for the rest of the function:
    `function f(c){ { const c = 7; console.log(c); } return c; }
    console.log(f(2));` prints `7` / `7` (r12; node `7` / `2`). The same with
    an instance parameter shadowed by an object literal (r8; node `7` / `2`,
    kali `7` / `7`). Codegen's bindings are flat per function (see item 11 and
    literal-array-mutators followups §14). Silent, pre-existing, not fixed.
16. **R-31: `--compat eval` aliasing.** With the opt-in `eval` flag,
    `const y = eval('c'); y.n = 7; console.log(c.n);` is not refused for an
    instance (ev1; node `7`), nor for a plain object (ev2). Without the flag
    both are refused (`compatibility feature 'eval' … is unavailable`).
    Extends item 7. Not fixed.
17. **R-32: forward references between classes.** A method of one class that
    constructs a class declared after it fails with E3100 `undefined
    identifier` under `run` and `check` (rc2, rc3, rc4: mutually
    constructing classes `A` / `B` and `E` / `O`; node prints `4` / `2` and
    `1` / `0`). Refused, not silent; not fixed.
18. **R-32: an aliased class reaches only code generation.** `const K = C;
    new K()` (o1; node `4`) and `function mk(K){ return new K(); } mk(C)`
    (n3; node `4`) are refused under `run` only at code generation (E3100
    zero-placeholder / E8001), while `check` exits 0: `C` is never the direct
    target of `new`, so it is not rewritten and the class-as-value refusal
    does not apply. Not silent under `run`; `check` misses it. Not fixed.
19. **R-11 addendum: `export class C` in library mode.** Because the parser
    drops `export` from `export class C`, a library-mode build's artifact does
    not export `C` at all; the class disappears from the module's exports
    rather than being refused. Not fixed.

(The A-10 backstop line repeating after a use refusal is item 2.)
