# Stage 6 triage — block-arrow un-flattening (entry measurement)

**Branch:** `soundness-batch1-pra` @ `67f2a92a5`
**Date:** 2026-07-14
**Scope:** Task 1 — measurement + triage only. No product code changed.

---

## 1. Entry denominator: **731**

Two independent full-workspace enumerations on a freshly built binary (`cargo build -p kali_cli`):

```bash
for i in 1 2; do
  cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > "$SCRATCH/stage6-pre-run$i.txt"
done
diff "$SCRATCH/stage6-pre-run1.txt" "$SCRATCH/stage6-pre-run2.txt"   # identical
sort -u "$SCRATCH/stage6-pre-run1.txt" "$SCRATCH/stage6-pre-run2.txt" > "$SCRATCH/stage6-pre.txt"
wc -l "$SCRATCH/stage6-pre.txt"                                      # 731
```

| | run 1 | run 2 |
|---|---|---|
| raw `FAILED` lines | 749 | 749 |
| after `sort -u` | **731** | **731** |
| `diff run1 run2` | *(identical)* | |

The 749 → 731 gap is the known 18 test names that exist in two harness binaries each.
**`sort -u` is mandatory** — plain `sort` fabricates 18 phantom entries.

`$SCRATCH/stage6-pre.txt` (731 lines) is the canonical entry set consumed by every later Stage-6 gate.

---

## 2. The parse defect, pinned

Temporary test in `crates/kali_parser/tests/parser_integration.rs` dumping the AST of
`queueMicrotask(() => { ran = 1; });` — **removed after recording** (`git diff --stat` on that file is empty).

```
STMT COUNT = 2
STMT[0] = ExpressionStatement(
    ExpressionStatement {
        expression: CallExpression(
            CallExpression {
                callee: Identifier("queueMicrotask"),
                args: [
                    ParenthesizedExpression(
                        ParenthesizedExpression {
                            expression: Identifier("unknown"),      // <-- the arrow is GONE
                        },
                    ),
                ],
            },
        ),
    },
)
STMT[1] = BlockStatement(                                            // <-- body hoisted to TOP LEVEL
    BlockStatement {
        body: [
            ExpressionStatement(
                ExpressionStatement {
                    expression: AssignmentExpression(
                        AssignmentExpression {
                            operator: Assign,
                            left: Identifier("ran"),
                            right: Literal(Number(1.0)),
                        },
                    ),
                },
            ),
        ],
    },
)
```

A block-bodied arrow parses into **two** statements: the call keeps a bogus `Identifier("unknown")`
argument, and the arrow's body becomes a **separate top-level `BlockStatement`** that executes
**inline, eagerly, at module scope**.

---

## 3. Baseline miscompiles (observed — node vs kali, both run here)

All kali outputs below are from a freshly built **unpatched** binary at `67f2a92a5`.

### Probe 1 — callback body runs inline

```js
let ran = 0;
queueMicrotask(() => { ran = 1; });
console.log(ran);
```

| node | kali |
|---|---|
| `0` | `1` |

The callback is never deferred; its body already ran. **Silent miscompile.**

### Probe 2 — class-method body returns 0

```js
class C { run(){ return 42; } }
console.log(new C().run());
```

| node | kali |
|---|---|
| `42` | `0` |

**Silent miscompile** (pre-existing; recorded in the Stage-5 inventory).

### Probe 3 — feature-rich callback body: the flatten is *masking* the real gap

```js
class Box { constructor(){ this.n = 4; } }
let acc = 0;
queueMicrotask(() => {        // p3a: block ARROW
  let value = 1;
  value += 2;
  value *= 5;
  let b = new Box();
  acc = value + b.n;
  console.log("INSIDE-CALLBACK");
  console.log(value);
});
console.log("MODULE-END-acc");
console.log(acc);
```

| | node | kali |
|---|---|---|
| **p3a** (block arrow) | `MODULE-END-acc` / `0` / `INSIDE-CALLBACK` / `15` | `INSIDE-CALLBACK` / `15` / `MODULE-END-acc` / `15` |
| **p3b** (identical body, `function () {…}`) | `MODULE-END-acc` / `0` / `INSIDE-CALLBACK` / `15` | `error[E5506]: compound assignment on binding 'value' is unavailable: it is not a provably scalar number or string …` (exit 1) |

Reading:

- **p3a**: ordering is fully inverted (body runs *before* module end) and `acc` is mutated before the
  module-end read. The `+=`, `*=`, and `new` all "work" — **because the body is at module scope**,
  which is repr-tracked. *That is the masking.*
- **p3b**: the semantically identical `function(){}` callback — which parses correctly today —
  **rejects E5506**. This is the pre-existing **untracked-function-scope repr gap**, unmasked.

So the flatten is not merely mis-ordering code: it is *hiding* the fact that kali cannot compile a
feature-rich body in any function scope. Un-flattening without first repr-tracking function scopes
converts silent-wrong-answer into loud-reject. **This is why Tasks 2–4 (repr-track scopes) must land
before the parse patch.**

### Probe 4 — `Kali.test` + throwing block arrow

```js
Kali.test('x', () => { throw new Error('boom'); });
```

kali (`test`):
```
error[E4000]: runtime trap (unreachable — allocation failure or an unsupported-path guard): error while executing at wasm backtrace:
    0:  0x42c - <unknown>!<wasm function 22>
Uncaught Error: boom
FAILED 1
```
JSON payload: `{"passed":0,"failed":1,"total":0}`

Control — the same test written `function () { … }`:
```
Uncaught Error: boom
error[E4000]: runtime trap in callback '__kali_callback_33': ...
FAILED 1
```
JSON payload: `{"passed":0,"failed":1,"total":1}`

**Answer to the brief's question: it is BOTH.** The failure surfaces as a bare `_start` trap
(`<wasm function 22>`, *no* `__kali_callback_N` attribution) which the CLI then renders as `FAILED 1`.
The `FAILED 1` is a trap artifact, not a test attribution — **`total: 0` proves the test never
registered.** The `function(){}` control registers properly (`total: 1`, trap attributed to
`__kali_callback_33`).

#### The worse case: block-arrow tests pass **vacuously**

```js
console.log("A-module-start");
Kali.test('vacuous-pass', () => { console.log("B-inside-test-body"); });
console.log("C-module-end");
```
kali:
```
A-module-start
B-inside-test-body
C-module-end
ok 1
```
The body ran **between** module-start and module-end — i.e. inline at module scope, not as the test —
and the registered test is an **empty shell that vacuously reports `ok 1` / `passed: 1`**. Any
assertion inside a block-arrow `Kali.test` body is evaluated at module scope and its result is not
attributed to the test; if it throws, the process traps in `_start` and the test count is `0`.

---

## 4. The predicted regressions — **the prediction is FALSIFIED**

### 4a. Exact names (Task 5's gate greps these)

The deferred doc names `compound|logical|nullish_assignment_wrapped_local_binding` and
`object_type_and_constructor_semantics`. **These are test-BINARY (file) names, not test-fn names** —
the brief's `grep -c "$n" stage6-pre.txt` returns 0 for all four *trivially*, and would return 0 even
if every one of them were red. The gate must use the fn names below.

The 16 predicted (all **GREEN today**: absent from the 731 **and** observed `... ok` in *both* runs):

```
test_supports_wrapped_compound_assignment_in_js_input
test_supports_wrapped_compound_assignment_in_ts_input
json_test_supports_wrapped_compound_assignment_in_browser_harness_js_input
json_test_supports_wrapped_compound_assignment_in_browser_harness_ts_input
test_supports_wrapped_logical_assignment_in_js_input
test_supports_wrapped_logical_assignment_in_ts_input
json_test_supports_wrapped_logical_assignment_in_browser_harness_js_input
json_test_supports_wrapped_logical_assignment_in_browser_harness_ts_input
test_supports_wrapped_nullish_assignment_in_js_input
test_supports_wrapped_nullish_assignment_in_ts_input
json_test_supports_wrapped_nullish_assignment_in_browser_harness_js_input
json_test_supports_wrapped_nullish_assignment_in_browser_harness_ts_input
test::test_supports_object_type_and_constructor_semantics
test::test_supports_object_type_and_constructor_semantics_in_js_input
test::json_test_supports_object_type_and_constructor_semantics
test::json_test_supports_object_type_and_constructor_semantics_in_js_input
```

(The `runtime_smoke` ones are module-prefixed `test::`. The 12 `wrapped_*` live in their own binaries
and are unprefixed.)

Note the two groups assert **opposite** things:
- the 12 `wrapped_*` assert **success** (`payload.passed == 1`, `failed == 0`);
- the 4 `object_type_*` are **flipped pins asserting fail-closed** (`!status.success()`, stderr
  contains `E4000`/`E5506`/`RuntimeError: unreachable`).

### 4b. Measured blast radius of the patch: **63 newly-red, not 16**

The 16 exist to be falsified, so they were measured, not assumed. The deferred patch was applied to a
scratch tree, `cargo test --workspace --no-fail-fast` run, then **reverted** (tree verified clean,
HEAD unchanged, patch re-verified to still apply).

| | count |
|---|---|
| baseline failures | **731** |
| failures with patch applied | **794** |
| **newly-red** | **63** |
| **newly-green** | **0** |

The deferred doc's "16" is a **~4× underestimate**, and the patch fixes **zero** currently-red tests
(expected — its value is closing *silent* miscompiles, which by construction are not in the red set).

Of the 16 predicted: **14 regress**, and **2 do not** —
`test::test_supports_object_type_and_constructor_semantics{,_in_js_input}` (the text-mode variants)
**stay green**, because they accept any of `E4000`/`E5506`/`unreachable` anywhere in stderr.

**49 newly-red were not predicted at all**, in families the deferred doc never names:

| family | count |
|---|---|
| `*_reflect_own_keys_*` | 18 |
| `*_wrapped_{compound,logical,nullish}_assignment_*` | 12 (predicted) |
| `*_unary_prefix_semantics_*` (browser harness) | 8 |
| `*_array_callback_identity_slices_in_browser_api_surface_with_harness_*` | 8 |
| `*_browser_bundle_web_baseline_primitives*` | 7 |
| `*_nullish_assignment_in_browser_api_surface_with_harness_*` | 4 |
| `*_wrapped_mutable_{compound_assignment,update}_targets_with_browser_harness_*` | 4 |
| `test::json_test_supports_object_type_and_constructor_semantics*` | 2 (predicted) |
| | **63** |

Full list: `$SCRATCH/stage6-patch-newly-red-63.txt`.

### 4c. Root causes of the 63 — one dominant class

| root cause | count |
|---|---|
| `E5506` compound assignment on a binding in a callback scope | 35 |
| `E5506` update expression (`++`/`--`) on a binding in a callback scope | 10 |
| `E5506` logical/nullish assignment on a binding in a callback scope | 4 |
| callback-scope semantic break — array-callback accumulation (see below) | 8 |
| callback-scope semantic break — `for..in`-key alias + `??=` (see below) | 4 |
| test-side pin on the **flattened diagnostic shape** | 2 |
| | **63** |

**61 of 63 are the single untracked-function-scope repr gap** that Tasks 2–4 exist to close. Verbatim:

```
error[E5506]: compound assignment on binding 'count' is unavailable: it is not a provably scalar
number or string (an array or object value has no compound-assignment lowering)

error[E5506]: update expression on binding 'counter' is unavailable: it is not a provably integer
number (an array, object, float, or string value has no update lowering)
```

This is **strong confirmation of the plan's sequencing** (repr-track scopes first, un-flatten second) —
it just has to survive a 63-test blast radius, not 16.

Two items need separate handling in Task 5:

1. **`test::json_test_supports_object_type_and_constructor_semantics{,_in_js_input}` — re-pin, do not
   "fix".** Under the patch these still **fail closed** (`success: false`, `exitCode: 1`); the trap is
   correctly attributed to `__kali_callback_34` and therefore lands in `stderr` + `payload.failed`
   rather than in `errors[]`. The assertion reads `json["errors"][0]["code"]`
   (`crates/kali_cli/tests/runtime_smoke.rs:3761`), which is now empty. The assertion is pinned to the
   *flattened* error-reporting shape; the patched shape is strictly better (`total: 1`, a real test
   failure). **Task 5 must widen this pin, not chase a product bug.**

2. **Two callback-scope breaks beyond the E5506 gates (12 tests) — these do *not* announce themselves
   with a diagnostic; they silently compute the wrong answer inside the callback.**

   **(a) `array_callback_identity_slices` (8).** With the patch, the body's `observed.join(",")`
   self-check throws inside the callback, while the identical code at module scope is correct:
   ```
   node               : some:true  / every:false / joined=1,2,1,2,1,2,1,2,1,2
   kali module scope  : some:1     / every:0     / joined=1,2,1,2,1,2,1,2,1,2   (join CORRECT)
   kali in callback   : some:1     / every:0     / Uncaught Error: unexpected array callback identity semantics
   ```
   Array-callback (`.map`/`.filter`/`.flatMap`) accumulation into a `push`-growable array **breaks when
   the code sits inside a callback scope**.

   **(b) `nullish_assignment_in_browser_api_surface_with_harness` (4).** Body is
   `var table = {a:1,b:2}; var last = null; for (var c in table) { last = c; } last ??= null; …` —
   the surviving `??=` lowering is a **`for..in`-key ALIAS binding** (`-1` null sentinel, fasta Spec 7).
   That alias tracking is module-scope-only, so the lowering does not survive the move into a callback.

   Tasks 2–4 must cover both, or they are a 12-test residual. Note the general scalar `??=` still
   rejects fail-closed (confirmed independently: `let flag = null; … flag ??= 7` →
   `error[E5506]: nullish assignment on binding 'flag' is unavailable: null and 0 are indistinguishable
   for a scalar value; only a for-in-key alias with a null sentinel supports \`??=\``).

---

## 5. The deferred patch

```bash
git apply --check docs/superpowers/followups/task8-block-arrows-deferred.patch && echo APPLIES
# => APPLIES
```

Still applies at `67f2a92a5`. It contains **only two source files**:

```
 crates/kali_parser/src/expression/primary.rs |   62 +++++++++++++++++------
 crates/kali_types/src/resolve/call.rs        |   70 ++++++++++++++++++++++++++
 2 files changed, 117 insertions(+), 15 deletions(-)
```

The deferred doc claims `crates/kali_cli/tests/soundness_block_arrows.rs` exists with "4 tests, all
pass". **It does not exist** — not in the patch, not anywhere in the tree (`find` over the repo returns
nothing). Those tests must be **written fresh in Task 5**; the doc's claim is not evidence.

---

## 6. Pre-existing miscompiles observed in passing (not Stage 6's to fix)

- **Template-literal boolean stringification**: `` `some:${[0,1].some(v=>v)}` `` prints `some:1` in kali
  vs `some:true` in node (and `every:0` vs `every:false`). Present **with and without** the patch, at
  module scope. Not caused by the flatten.
- **Class-method bodies return 0** (Probe 2) — already on the Stage-5 inventory.

---

## 7. Artifacts

| file | meaning |
|---|---|
| `$SCRATCH/stage6-pre.txt` | **canonical 731-entry baseline** (`sort -u`, both runs agree) |
| `$SCRATCH/stage6-pre-run{1,2}.txt` | the two independent enumerations (identical) |
| `$SCRATCH/stage6-patch-newly-red-63.txt` | the 63 tests the deferred patch turns red |
| `$SCRATCH/predicted16.txt` | the 16 predicted names, exact, gate-greppable |

## 8. Gate for later tasks

```bash
# newly-red must be empty against the canonical baseline
cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E '^test .* \.\.\. FAILED' | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
  | sort -u > "$SCRATCH/stage6-post.txt"
comm -13 "$SCRATCH/stage6-pre.txt" "$SCRATCH/stage6-post.txt"   # must be empty
```

Never plain `sort`. Always `--no-fail-fast`. Always re-run on a freshly built binary.

---

## 9. Task 4 — the class-method corollary, decided: hypothesis FALSIFIED

**Test:** `class_method_bodies_return_their_value` in
`crates/kali_cli/tests/soundness_block_arrows.rs` (Stage-5 reproducer, added verbatim per the
task-4 brief).

**Hypothesis under test:** Task 3 repr-tracked class-method bodies (comment at
`kali_types/src/context.rs:48` — "class methods all push here (repr-tracked as of Task 3)"), so
maybe it *also* fixed the Stage-5 Probe-2 silent miscompile (`class C { run(){ return 42; } }
new C().run()` → `0` instead of `42`), which was filed as "pre-existing" before Task 3 landed.

**Method:** built a fresh `kali_cli` binary at this commit (which already includes Task 3), ran
the focused test, then independently ran the raw two-line reproducer through the fresh binary and
compared to `node` as oracle.

**Verdict: FALSIFIED.** The miscompile is still live, unchanged.

Focused-test output (`cargo test -p kali_cli --test soundness_block_arrows -- --test-threads=4
--ignored`):

```
running 1 test
test class_method_bodies_return_their_value ... FAILED

---- class_method_bodies_return_their_value stdout ----
thread 'class_method_bodies_return_their_value' panicked at crates/kali_cli/tests/soundness_block_arrows.rs:189:5:
assertion `left == right` failed
  left: "0\n"
 right: "42\n"

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out
```

Raw reproducer, independently run against the same fresh binary:

```js
class C {
  run() {
    return 42;
  }
}
console.log(new C().run());
```

| | node (oracle) | kali (fresh, post-Task-3) |
|---|---|---|
| stdout | `42` | `0` |
| exit code | 0 | 0 (no diagnostic — still silent) |

Before/after framing requested by the brief:

| | before (Stage 5, pre-Task-3) | after (now, Task 4, post-Task-3) |
|---|---|---|
| kali output | `0` | `0` — **unchanged** |
| node output | `42` | `42` |

**Reading:** Task 3's repr-tracking made class-method-body *bindings* (locals, params, compound
assignment, etc.) provably-scalar inside the method scope — it did not touch `return`-value
propagation out of a class method back to the call site. `current_function_scopes` (context.rs)
now includes the method's scope for repr purposes, but the codegen/HIR path that lowers a class
method's `return <expr>` into the value handed back to `new C().run()` is a **separate,
still-open lowering gap** — grep shows `MethodDefinition` lowering lives in
`crates/kali_hir/src/lowering/statement.rs`, distinct from the repr-tracking context Task 3
changed. This is a genuinely different root cause from the block-arrow-flatten class Stage 6
exists to close, and per the task brief it is **filed, not fixed** — chasing it would widen this
stage.

**Disposition:** test kept in the suite, `#[ignore = "class-method return lowering — separate
root cause, see stage6 triage"]`, assertions intact (`out.status.success()` and
`stdout == "42\n"` both still present and still fire when run with `--ignored`). This keeps the
Stage-5 Probe-2 finding alive as a reproducible, automatically-runnable pin rather than a stale
prose note, without blocking Stage 6's green gate.

---

## §10. Stage 6 outcome — un-flatten DEFERRED; landed portion is Tasks 1–4 (sound)

**Status: the stage's landed deliverable is the repr-tracking foundation (Tasks 1–4). The
un-flatten itself (Task 5) is deferred to a new multi-stage capability project (user decision,
2026-07-15).**

### What landed (committed, reviewed, behavior-neutral — 731 → 731, 0 newly-red, 0 drain)

| Task | Commit | What |
|---|---|---|
| 1 | `5a0cc82a0` | Triage: entry 731 pinned; parse defect pinned; **patch-only blast radius MEASURED = 63 newly-red / 0 newly-green** (plan predicted 16 — a ~4× undercount). |
| 2 | `d294ba8e6` | `name_anonymous_functions` AST pre-pass: `ArrowFunctionExpression.id` + `__kali_fn_{N}` assigned before the resolver so `kali_types`/`kali_hir` share one key. Exhaustive walk, two-pass collision guard, runs after `monomorphize`. |
| 3 | `51de2bb7a` + `52c4bc11e` | Repr-track fn-expr/arrow/class-method bodies (3-site mirror of the `FunctionDeclaration` arm). Fixed `.expect()`→safe fallback; closed the anon-`export default function` naming divergence; corrected stale untracked-scope doc comments. |
| 4 | `3b12dc6cc` | Class-method-return corollary: **hypothesis FALSIFIED** — `new C().run()` still returns 0 (return-value lowering, a separate root cause). Test `#[ignore]`'d, assertions intact. |

Tasks 1–4 leave the branch exactly as sound as it was: no new silent miscompiles, repr-tracking
now correct for function-shaped scopes. The un-flatten is a prerequisite-heavy feature, not a
diff these tasks could safely carry.

### Why Task 5 (un-flatten) could not land — the plan's premise was falsified

The plan assumed Task 3's repr-tracking would make the 63 predicted regressions not happen, so
landing the parse patch would be clean. **It is not.** Applying the un-flatten + wiring
(all independently verified correct — see the WIP patch) produces **22 newly-red**; after the 2
user-approved re-pins, **20 remain, tracing to THREE structural gaps repr-tracking does not
touch**:

| Gap | Newly-red | Fails how | Fix size |
|---|---|---|---|
| **A. `repr_infer.rs` never walks function bodies** (object-shape + string-seed proofs unavailable for anything declared inside a callback) | 8 | Closed (E5506) | Large: ~800-line exhaustive expr walk × 3 collectors + function-scope threading through a 3487-line whole-program flow-graph pass + BFS re-verification. |
| **B. Array-callback nested-function miscompile** (named `FunctionDeclaration` inside an un-flattened callback + 2+ array-callback for-of loops) | 8 | **SILENT** (empty stdout, exit 0 via `queueMicrotask`; node prints `1,2,3,4`) | Unknown — not yet root-caused to a line; likely a per-function keying collision in codegen loop/arena machinery. |
| **C. No closure-capture mechanism** (`count += 1` where `count` is an *enclosing function's* local, mutated from inside a callback) | 4 | Closed (E5506 at `emit/literal.rs:496`) | Large: kali has NO closure/environment model (only module-scope `module_global_slots`). Needs slot-promotion of captured fn-scope scalars or an environment-pointer model. |

The un-flatten essentially reveals that **kali has no closure model**, and that whole-program
repr inference and parts of codegen were only correct because the flatten kept every callback body
at module scope. Partial-landing is out: it would trade today's silent miscompile (callback bodies
running inline at the wrong time) for a *new* one (gap B).

### The independently-verified Task 5 work (preserved, not committed)

`docs/superpowers/followups/task5-block-arrows-WIP.patch` (605 lines). All sound, no fail-open
found: hand-applied parser un-flatten; **fixed a real patch bug** (`reject_anonymous_function_argument`
detected anonymity via `id.is_none()`, which Task 2's pre-pass had silently defeated — now keys on
the `__kali_fn_` synthetic marker); wired `queueMicrotask` (recognizer + emit + conditional import,
no index shift, callback VERIFIED to run during `drain_event_loop` byte-for-byte vs node);
`setTimeout`/`setInterval` carved out of the builtin exemption → fail closed E5506. This patch is
the seed for the new project's final "land the un-flatten" stage.

### The new project (user decision 2026-07-15): build the capabilities, THEN land un-flatten

- **Stage A** — `repr_infer.rs` walks fn-expr/arrow/class-method bodies (closes gap A; covers BOTH
  the object-shape and string-seed symptoms as one architectural fix).
- **Stage B** — root-cause + fix the array-callback nested-function SILENT miscompile (gap B).
- **Stage C** — closure capture of enclosing function-scope locals (gap C).
- **Stage D** — land the un-flatten + wiring (rebase the WIP patch onto A/B/C) and re-gate.

**Whole-branch review of Tasks 1–4** is deferred to Stage D, where the un-flatten exercises this
foundation for real and everything re-gates together; each of Tasks 1–4 was already independently
task-reviewed with deep verification, and they are behavior-neutral, so a separate review now would
re-verify a foundation nothing yet stresses.
