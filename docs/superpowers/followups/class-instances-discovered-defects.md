# Defects the class-instances project measured and did NOT fix

**Filed** 2026-10-04 by the **class-instances** project
(`docs/superpowers/specs/2026-10-04-class-instances-design.md`), on the
convention `unresolved-member-call-discovered-defects.md` uses: a project that
measures more than it fixes writes down what it left, so the silence is not
read as absence.

**Oracle:** `node v26.10.0`.
**Measured at:** `bb2411ea6` (branch `class-instances`), on
`target/debug/kali` built from that commit (`cargo build -p kali_cli`, the
`dev` profile). The baseline binary was built from `7c4daa9f7` in a separate
worktree. Probe rows come from `tools/array-return-probes/probes/cls_*.js`
(runner `tools/array-return-probes/run.sh`); a probe's baseline column is
`tools/array-return-probes/baseline-cls.tsv`, measured at `7c4daa9f7`.

§1 to §3 and the rest of §6 are completed by Task 11.

---

## §1. The R-30 boolean field

*Task 11.*

## §2. Out-of-slice refusals (future items)

*Task 11.*

## §3. Containers, rendering, `instanceof` / `===`, mixed-class parameters, `new` of a plain function

*Task 11.*

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

*Task 11 adds the rest of §6.*
