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
