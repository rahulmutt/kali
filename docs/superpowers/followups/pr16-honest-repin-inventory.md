# PR #16 honest re-pin inventory (canonical) — CORRECTED

This is the single canonical adjudication map for the 694 honest-red workspace tests
carried on `soundness-batch1-pra` (baseline: `pr16-honest-red-baseline.txt`, Task 2).
Each later "wave" task (Task 5 template) is instantiated one-per-row from the tables below.
`stageD-triage.md` §8.6 points here rather than duplicating this content.

Evidence for this revision was gathered on HEAD `139d71189` (tree clean) against
`target/debug/kali` built from that commit, differentially versus Node 26.5.0.

---

## 0. METHODOLOGY CORRECTION — read this before trusting anything in git history

**The previous revision of this file was wrong, and wrong in a way that would have written
falsehoods into `main`.**

### What was wrong

The first triage gathered its evidence with **top-level reproducers** — small snippets typed
at module scope — rather than with each fixture's own source in its own scope. Module scope in
kali is not the same program as function scope. In particular there is a live, silent
module-scope-only defect (`const a=[]; a.push(1)` compiles to a no-op; see B3 below) that
contaminated essentially every probe that used a top-level `push` as its observation sink.

Two consequences, the second far more damaging than the first:

1. **Class calls were wrong.** The old table called 12 of 13 families class B (silent
   miscompile). First-hand in-scope evidence shows **310 of 694 are class A** — kali refuses
   at compile time with an explicit `E5506`, before any value is produced. Four whole families
   flipped B → A (`for-await`, `await-wrapped`, most of `mapset`, half of `string-iter`).

2. **The stated failure reasons were misattributed.** This is the real damage. The old
   reasons named the feature in the test's *name*; the actual limits are usually something
   else entirely. Examples that would have gone into `main` as comments:

   | test family | old reason (wrong) | actual limit (verified) |
   |---|---|---|
   | `for-await` | "async iteration runtime" | growable array escapes via a function argument — `for await` never gets reached; it is a compile error |
   | `mapset` | "Set/Map runtime + iterable protocol" | `try`/`catch`/`finally` is unavailable. `for (const v of new Set([3,4]))` is **node-correct today** |
   | `bool-bundle` | "audit bundle-context boolean/async lowering" | `&&`/`||` do not short-circuit, in the plain direct path — nothing to do with bundles |
   | `microtask` (bundle rows) | "proper microtask queue ordering" | growable array `order` escapes via a function argument |
   | `object-enum` (122 rows) | "materialize enumeration-result arrays" | `Object.keys` works; the growable array collecting the keys cannot be passed to the assert helper |
   | `string-iter` (49 rows) | "dynamic string-char materialization" | chars *do* materialize and `.length` is correct; the characters are stored as raw interned i64 handles that print as numbers |

   Pinning `for_await_*` with "kali has no async iteration runtime" would be a falsehood: kali
   never evaluates the loop, and the construct it actually rejects is unrelated.

### What changed in method

- **Every classification below is derived from running the test in its own scope**, either by
  executing the real `cargo test` and capturing kali's stderr, or by extracting the fixture's
  own function-scoped source and running that. Where a minimization is quoted, the text states
  whether the minimization preserved scope.
- Every behavioral claim is a real transcript, `target/debug/kali` versus `node`, both shown.
- A **complete first-hand terminating-diagnostic census** was taken: one representative test
  actually executed for every (family × defining-file) cell, 58 cells, covering all 694 names.
  Where a cell's first representative produced only a JSON envelope, a non-JSON sibling was
  re-run to surface the diagnostic.

### The standing lesson

> **Verify in the fixture's own scope.** A top-level snippet is a different program from the
> same text inside a function. kali's analyses are keyed on function identity (`_start` versus
> a named function), so scope changes which lanes, gates, and deny-lanes apply. A reproducer
> that does not preserve the fixture's scope is not evidence about the fixture.

A corollary learned here: **choose the observation sink as carefully as the construct.** Three
of this session's own probes initially failed on `E3200` (`'+' with a string-typed variable
operand`) or `E3100` (`undefined identifier 'await'`) — artifacts of the probe's print
statement and driver, not of the feature under test. A probe that fails for its own reasons
proves nothing about the fixture.

---

## 1. Count provenance — 712 versus 694 (unchanged, still correct)

- **N = 694**, set-identical to the Task-2 canonical baseline (`git show 407c81002` head commit).
- **712 vs 694 — both honest measurements of the *same* red set (fully resolved, cited account):**
  The 712-line artifact lives OUTSIDE the repo (a prior session's scratchpad `p3-baseline-failed.txt`,
  raw `test … FAILED` lines). Verified: 712 raw lines; **18 names appear exactly twice**; deduplicated
  = exactly **694 unique names, set-identical to the frozen Task-2 baseline**. Root cause of the 18
  duplicates (verified against the tree): each is a **root-scope test-fn name defined in TWO different
  test binaries** — a per-*instance* count (712) sees each such name twice, while the enumeration
  recipe's `sort -u` is **name-set based** (694). So **712 = failing test INSTANCES, 694 = unique test
  NAMES**; neither is an undercount, and there was never a real 18-test delta — do not chase a phantom
  regression.
  - **The 18 duplicated names, in 2 binary-pairs:**
    - `{run,test,json_run,json_test}_supports_string_primitive_iteration_when_browser_harness_is_configured_in_{js,jsx,ts,tsx}_input`
      (**16 names**) — defined in both `browser_object_string_enumeration_harness.rs` **and**
      `browser_for_await_object_string_enumeration_harness.rs`.
    - `{run,test}_supports_object_values_spread_iteration_when_browser_harness_is_configured`
      (**2 names**) — defined in both `browser_object_values_harness.rs` **and**
      `browser_object_values_spread_harness.rs`.
  - **Wave-time implication:** a wave touching any of these 18 names must fix the instance in **BOTH**
    binaries — patching one file leaves the same-named test red in the other.

## 2. Classification key

- **A — fail-closed already:** kali fails closed *independent of the fixture's self-check* — a
  compile-time E-code rejection (`E5506`, `E3000`) or a terminating trap on the construct
  itself. The test observes exit≠0 for a reason that is not "the fixture caught a wrong value."
  **Action: pin-reject to the observed terminating diagnostic.** No deny lane needed.
- **B — silent miscompile:** the fixture's in-program `throw` is the *only* thing making the
  test exit≠0. With the self-check removed, kali **exits 0 with a wrong value**. Pinning this
  as "unsupported" would bless a wrong value on main.
  **Action: deny-lane-then-pin.**
- **C — harness artifact:** product behavior is correct; the test's predicate is wrong.
  **No family classified C.**

---

## 3. Adjudication table — regrouped by ROOT CAUSE

The old partition's *boundaries* did not match real root causes: one file could hold both A and
B members, and one root cause spanned several families. The table below regroups by verified
root cause. §6 gives the old→new mapping and re-verifies the arithmetic.

### Class A groups — pin-reject only, no deny lane (310 tests)

| # | root-cause group | count | verified terminating diagnostic | **pin reason string to write into the test comment** |
|---|---|---:|---|---|
| A1 | growable array escapes via a function argument | **195** | `error[E5506]: growable array \`keys\` in \`browserDirectObjectKeysIteration\` uses \`.push\` but also appears in a position the growable-array lane does not support (escaping via \`return\` or an alias, …)` | *"kali's growable-array lane does not support a `.push`-built array escaping into a function call. The fixture collects into an array then passes it to its assert helper; kali rejects at compile time with E5506. The enumeration/iteration feature named in this test is not itself the limit."* |
| A2 | `try`/`catch`/`finally` unavailable | **46** | `error[E5506]: try/catch/finally is unavailable: kali has no exception-handling machinery` | *"kali has no exception-handling machinery; the fixture's `try`/`finally` block is rejected at compile time (E5506). The Map/Set/finalization feature named in this test is not the limit — plain `for (const v of new Set([3,4]))` is node-correct today."* |
| A3 | object enumeration needs a compile-time-known fixed shape | **32** | `error[E5506]: Object enumeration is only supported where the object has a compile-time-known fixed shape` | *"kali enumerates only objects whose shape is known at compile time; the fixture's object comes from `Object.fromEntries(...)` (or an await/sequence wrapper), so kali rejects at compile time with E5506."* |
| A4 | passing an array literal to a function | **22** | `error[E5506]: passing an array literal to function 'assertWrappedObjectEnumeration' is unavailable in the current direct-runtime path (the callee would read zero…)` | *"kali's direct-runtime path cannot pass an array literal to a function; the fixture's assert helper takes one, so kali rejects at compile time with E5506."* |
| A5 | corpus packages hit an unsupported host feature | **15** | `error[E5506]: only \`abort()\` is supported on an AbortController handle; \`addEventListener\`/\`onabort\`/\`reason\`/\`throwIfAborted\` fail closed (Stage P3 scope)`; deno row: `error[E4000]: runtime trap in callback` | *"pin to whatever terminating diagnostic this package actually produces — it is per-package and will change as features land. Verify at pin time; do not pin a fixed string."* |

### Class B groups — deny-lane-then-pin (384 tests)

| # | root-cause group | count | minimized in-scope repro | kali | node | **pin reason string** |
|---|---|---:|---|---|---|---|
| B1 | interned string handles leak into value positions | **115** | `function f(){ const c=[]; for (const x of "ab") c.push(x); console.log("j="+c.join("")); } f();` | `j=-9223354444668731391-9223354440373764095` exit 0 | `j=ab` exit 0 | *"strings produced by kali's runtime (enumeration keys, iterated characters, `Reflect.ownKeys` results) are internal interned i64 handles. Array length and control flow are correct; the values leak as raw integers the moment they reach a value position (`+`, `.join`, an array element read). Silent — exit 0, wrong output."* |
| B2 | spread of an enumeration result | **45** | `function f(){ const a=[...Object.values({x:7,y:8})]; console.log("n="+a.length+" 0="+a[0]); } f();` | `n=1 0=0` exit 0 | `n=2 0=7` exit 0 | *"spreading an `Object.keys/values/entries` result produces a wrong-length array of zeros. Silent — exit 0, wrong output."* |
| B3 | module-scope growable `push` / element store is a no-op | **29** | `const a=[]; a.push(1); console.log("len="+a.length);` (top level — scope is the point) | `len=0` exit 0 | `len=1` exit 0 | *"at module scope kali's growable-array promotion never runs, so `.push` silently lowers to a drop-args no-op and index stores are dropped. The same code inside a function is correct. Silent — exit 0, wrong output."* |
| B4 | Promise combinators return a placeholder | **128** | `async function f(){ const r=await Promise.all([Promise.resolve(1),Promise.resolve(2)]); console.log("n="+r.length+" 0="+r[0]); } f();` | `n=0 0=0` exit 0 | `n=2 0=1` exit 0 | *"`Promise.all`/`race`/`any`/`allSettled` lower to a placeholder `0`; the awaited result has length 0 and every element reads 0. `await Promise.resolve(v)` is an admitted lane; the combinators are the gap. Silent — exit 0, wrong output."* |
| B5 | microtasks drain at program exit, not at the checkpoint | **18** | `async function main(){ let ran=false; queueMicrotask(()=>{ran=true;}); await Promise.resolve(); console.log("after-await ran="+ran); } main();` | `after-await ran=0` exit 0 | `after-await ran=true` exit 0 | *"`queueMicrotask` callbacks are deferred to a drain at program exit rather than to the microtask checkpoint, so a value a microtask writes is not observable after `await Promise.resolve()`. The callback does eventually run — ordering, not scheduling, is wrong. Silent — exit 0, wrong output."* |
| B6 | `&&`/`||` do not short-circuit | **4** | `function f(){ let n=0; const a=true&&(++n,true); const b=false||(++n,true); const c=false&&(++n,true); console.log("n="+n+" a="+a+" b="+b); } f();` | `n=4 a=0 b=0` exit 0 | `n=2 a=true b=true` exit 0 | *"kali evaluates both operands of `&&`/`||`, so side effects in the right operand fire unconditionally, and the operator's value is wrong when an operand is a sequence expression. Reproduces in the plain direct-runtime path — nothing bundle-specific. Silent — exit 0, wrong output."* |
| B7 | TextEncoder / WebCrypto host builtins | **4** | `function f(){ console.log("n="+new TextEncoder().encode("ab").length); } f();` and `console.log("u="+crypto.randomUUID());` | `n=0`; `u=-9223354375949254620` exit 0 | `n=2`; `u=ab4a5132-…` exit 0 | *"`TextEncoder().encode()` returns an empty result and `crypto.randomUUID()` returns a raw internal handle. (`Uint8Array` is separately E3100-undefined.) Silent — exit 0, wrong output."* |
| B8 | Deno host env/cwd builtins return a placeholder | **4** | fixture self-check `Uncaught Error: expected env get`; probe: `typeof Deno.env.get('PATH')` → `0` | returns `0` exit 0 | returns a string | *"`Deno.env.get` returns the placeholder `0` rather than a string, and `Deno.chdir` leaves cwd aliases disagreeing. Silent — exit 0, wrong output."* |
| B9 | `Object.hasOwn` in the browser-harness/bundle context — **MECHANISM NOT ISOLATED** | **28** | see §5 — all 15 disjunct forms and the composed condition are **correct** in the direct path | n/a | n/a | *`unverified` — the wave MUST isolate the failing disjunct in the harness/bundle path before choosing a choke point. Do not pin a mechanism claim.* |
| B10 | Map/Set nullish-variant constructor iteration — **SUB-SHAPE NOT ISOLATED** | **9** | plain `for (const v of new Set([3,4]))` is **node-correct**; fixture self-check is `unexpected nullish Map/Set constructor iteration semantics` | n/a | n/a | *`unverified` — the nullish (`??`/`?.`) sub-shape that fails was not isolated. The wave must isolate it; do NOT pin "Set/Map iteration unsupported", which is false.* |

**Class split (corrected):** **A = 310 tests in 5 root-cause groups · B = 384 tests in 10 root-cause
groups · C = 0.** (Old revision claimed A = 15, B = 679.)

---

## 4. Consolidated class-B register

Every genuine exit-0-wrong silent miscompile confirmed first-hand in this pass. **These, and only
these, need a fail-closed deny lane before their tests can honestly be pinned.** Everything in §3's
class-A table is already fail-closed and is pin-only.

| id | construct | minimized repro (scope as shown) | kali | node | candidate choke point |
|---|---|---|---|---|---|
| **B1** | runtime string handle reaching a value position | `function f(){const c=[];for(const x of "ab")c.push(x);console.log("j="+c.join(""));}f();` | `j=-9223354444668731391-9223354440373764095` (exit 0) | `j=ab` | string-handle read site — the same allowlist-at-the-choke-point pattern used for for-in keys (`resolve_identifier`); sinks are `+`, `.join`, element read |
| **B1a** | enumeration keys | `function f(){for(const e of Object.entries({b:1,a:2}))console.log("k="+e[0]+" v="+e[1]);}f();` | `k=-9223354436078796799 v=1` (exit 0) | `k=b v=1` | same as B1; note **values are correct**, only keys leak |
| **B1b** | `Reflect.ownKeys` | `function f(){const k=Reflect.ownKeys(Object.freeze({b:1,a:2}));console.log("0="+k[0]);}f();` | `0=-9223354444668731391` (exit 0) | `0=b` | same as B1 |
| **B1c** | computed-string iteration | `function f(){const c=[];for(const x of "he"+"llo")c.push(x);console.log(c.join(""));}f();` (via `const prefix/suffix`) | 5 garbage handles, **length correct** (exit 0) | `hello` | same as B1 |
| **B2** | spread of an enumeration result | `function f(){const a=[...Object.values({x:7,y:8})];console.log("n="+a.length+" 0="+a[0]+" 1="+a[1]);}f();` | `n=1 0=0 1=0` (exit 0) | `n=2 0=7 1=8` | spread iterable classification — `kali_types/src/static_analysis/array.rs` |
| **B3** | module-scope growable `push` | `const a=[]; a.push(1); console.log("len="+a.length);` **(module scope is the defect)** | `len=0` (exit 0) | `len=1` | `kali_types/src/repr_infer.rs:473-475` never registers `_start` in `growable_candidates` **or** `growable_rejects`; falls through to the drop-args no-op at `kali_codegen/src/emit/call.rs:1155`. Cheap fix: register `growable_rejects` for `_start` — converts silence into an honest E5506 without enabling promotion |
| **B3a** | module-scope element store | `const a=[0,0]; a[1]=9; console.log("v="+a[1]);` **(module scope)** | `v=0` (exit 0) | `v=9` | same family — the `E5506` literal-mutation gate fires in function scope and in top-level loops but has a hole on top-level straight-line stores |
| **B4** | Promise combinators | `async function f(){const r=await Promise.all([Promise.resolve(1),Promise.resolve(2)]);console.log("n="+r.length+" 0="+r[0]+" 1="+r[1]);}f();` | `n=0 0=0 1=0` (exit 0) | `n=2 0=1 1=2` | `kali_types/src/late_host.rs` / `kali_codegen/src/emit/call.rs` — combinator lowering returns placeholder `0`. `race`→`r=0`, `allSettled`→`n=0 s=0 v=0` verified identically |
| **B5** | `queueMicrotask` checkpoint | `async function main(){let ran=false;queueMicrotask(()=>{ran=true;});await Promise.resolve();console.log("after-await ran="+ran);}main();` | `after-await ran=0` (exit 0) | `after-await ran=true` | event-loop drain point in the codegen runtime. Confirmed the callback *does* run — after end-of-module |
| **B6** | `&&`/`||` short-circuiting | `function f(){let n=0;const a=true&&(++n,true);const b=false||(++n,true);const c=false&&(++n,true);const d=true||(++n,true);console.log("n="+n+" a="+a+" b="+b+" c="+c+" d="+d);}f();` | `n=4 a=0 b=0 c=0 d=1` (exit 0) | `n=2 a=true b=true c=false d=true` | logical-operator lowering. Note the repo's standing finding that the parser has **no LogicalExpression node** — likely the same root |
| **B7** | `TextEncoder`/`crypto.randomUUID` | `function f(){console.log("n="+new TextEncoder().encode("ab").length);}f();` / `console.log("u="+crypto.randomUUID());` | `n=0`; `u=-9223354375949254620` (exit 0) | `n=2`; a UUID string | host-builtin lowering (Stage P5 surface). `new Uint8Array(4)` separately gives `error[E3100]: undefined identifier 'Uint8Array'` (exit 1 — that one is fail-closed) |
| **B8** | `Deno.env.get` / `Deno.chdir` | fixture self-check; probe `typeof Deno.env.get('PATH')` → `0` | placeholder `0` (exit 0) | a string | `late_host.rs` Deno host builtins |

**Unverified, in the B register by classification but with no confirmed repro:** **B9**
(`Object.hasOwn`, 28) and **B10** (Map/Set nullish variants, 9). Both are class B by the
self-check rule — the fixture's own `throw` is the only thing making them exit≠0 — but neither
mechanism was isolated. See §5.

---

## 5. Honest gaps — what is NOT verified

These are recorded as gaps rather than guessed at.

### B9 — `Object.hasOwn` harness/bundle failure (28 tests) — `unverified`

The fixture's entire 18-disjunct `if` condition was extracted verbatim and run in its own function
scope. **Every disjunct returns the correct value and the composed condition evaluates correctly:**

```
$ kali run h_bisect.js          $ node …
d01=1 … d18=1   (exit 0)        d01=true … (node itself throws at d11 —
                                 the fixture never reaches it, because `||`
                                 short-circuits on the earlier false operands)
$ kali run h_full.js            $ node …
CONDITION FALSE -> ok           CONDITION FALSE -> ok
```

So in the direct-runtime path kali is correct and the fixture would pass. The failure occurs only
under the browser-harness / bundle surface, and that path was not reproduced standalone here.

**Note a tempting-but-unconfirmed hypothesis:** B6 (`||` does not short-circuit) would cause kali to
evaluate disjuncts node never evaluates. That is suggestive, but it does **not** explain the failure,
because all 18 disjuncts evaluate correctly in kali anyway. Do not adopt it without evidence.

**Wave obligation:** isolate the failing disjunct *in the harness/bundle path* before choosing a
choke point. Do not pre-commit to the B1 string-handle choke point.

### B10 — Map/Set nullish constructor iteration (9 tests) — `unverified` sub-shape

Verified node-correct in kali today, in function scope:

```
$ cat q_set.js
function f(){ for (const v of new Set([3,4])) { console.log("v=" + v); } }
f();
$ kali run q_set.js    $ node …
v=3                    v=3
v=4                    v=4
(exit 0)               (exit 0)
```

The failing fixtures self-check with `unexpected nullish Map constructor iteration semantics`. The
specific nullish (`??` / `?.`) sub-shape that diverges was not isolated. Separately confirmed that
`for (const k of m.keys())` is **fail-closed** (`error[E5506]: for-of array iteration lowering is
unavailable unless the iterable is a literal array or supported string iterable …`), so the gap is
narrower than "Map/Set iteration".

**Under no circumstances pin these as "Set/Map iteration is unsupported" — that is demonstrably false.**

### Mixed-class files

Several defining files contain **both** A and B members. `browser_object_keys_harness.rs` is the
clearest case: `run_supports_direct_object_keys_iteration_…` and
`run_supports_const_bound_object_keys_iteration_…` both terminate on the A1 `E5506`, while sibling
tests in the same family reach runtime and self-check. **A wave must pin per test to that test's
observed diagnostic, never per file.**

### Direct-path versus harness-path divergence

For-of over a template literal is **fail-closed** in the direct path —
`error[E5506]: for-of array iteration lowering is unavailable unless the iterable is a literal array
with literal elements` — yet the 24 `browser_template_literal_string_iteration_*` tests reach
runtime and self-check, meaning the harness/bundle path lowers a construct the direct path rejects.
That asymmetry is itself a soundness signal and is currently **unexplained**; the B1 wave should
account for it rather than assume the two paths share a lane.

### Residual method caveat

The `browser_object_keys_iteration/build_json.rs` cell (10 tests) reports through a JSON envelope;
its diagnostic was taken from its non-JSON sibling `build.rs` (same fixture bodies, same feature),
which gives the A1 `E5506`. Treated as A1. Flagged rather than silently assumed.

---

## 6. Coverage ledger — old families → new root-cause groups

The frozen priority-ordered partition (13 families, `scratchpad/families/*.txt`, residue 0) is
**retained as the name-set partition** — it is what waves grep against. The regrouping below is a
*reclassification of those same names*, not a re-partition; no name moves between family files.

### 6a. Old family → new class split

| # | old family | count | class A | class B | which root-cause groups |
|---|---|---:|---:|---:|---|
| 1 | microtask | 22 | 4 | 18 | A1 (4, bundle rows) · B5 (18) |
| 2 | promise | 128 | 0 | 128 | B4 |
| 3 | reflect | 16 | 0 | 16 | B1 |
| 4 | for-await | 24 | **24** | 0 | A1 — **flipped B → A** |
| 5 | corpus | 15 | 15 | 0 | A5 (unchanged) |
| 6 | deno | 4 | 0 | 4 | B8 |
| 7 | mapset | 33 | **24** | 9 | A2 (24) · B10 (9) — **mostly flipped B → A** |
| 8 | await-wrapped | 3 | **3** | 0 | A3 — **flipped B → A** |
| 9 | string-iter | 94 | **45** | 49 | A1 (45) · B1 (49) — **split** |
| 10 | object-hasown | 28 | 0 | 28 | B9 (unverified) |
| 11 | object-enum | 319 | **195** | 124 | A1 (122) · A2 (22) · A3 (29) · A4 (22) · B1 (50) · B2 (45) · B3 (29) — **split** |
| 12 | bool-bundle | 4 | 0 | 4 | B6 |
| 13 | crypto-bundle | 4 | 0 | 4 | B7 |
| | **TOTAL** | **694** | **310** | **384** | |

`22+128+16+24+15+4+33+3+94+28+319+4+4 = 694` ✅ (== baseline N)
`310 + 384 = 694` ✅

### 6b. New root-cause group → contributing old families

| group | count | from |
|---|---:|---|
| A1 growable array escapes via function argument | 195 | object-enum 122, string-iter 45, for-await 24, microtask 4 |
| A2 `try`/`catch`/`finally` unavailable | 46 | mapset 24, object-enum 22 |
| A3 enumeration needs compile-time fixed shape | 32 | object-enum 29, await-wrapped 3 |
| A4 array literal passed to a function | 22 | object-enum 22 |
| A5 corpus per-package terminating diagnostic | 15 | corpus 15 |
| **A subtotal** | **310** | |
| B1 interned string handle leaks to value position | 115 | object-enum 50, string-iter 49, reflect 16 |
| B2 spread of enumeration result | 45 | object-enum 45 |
| B3 module-scope growable push / element store | 29 | object-enum 29 |
| B4 Promise combinators return placeholder | 128 | promise 128 |
| B5 microtask drains at exit not checkpoint | 18 | microtask 18 |
| B6 `&&`/`||` do not short-circuit | 4 | bool-bundle 4 |
| B7 TextEncoder / WebCrypto builtins | 4 | crypto-bundle 4 |
| B8 Deno host env/cwd placeholder | 4 | deno 4 |
| B9 `Object.hasOwn` harness/bundle (`unverified`) | 28 | object-hasown 28 |
| B10 Map/Set nullish sub-shape (`unverified`) | 9 | mapset 9 |
| **B subtotal** | **384** | |
| **TOTAL** | **694** | ✅ |

**Overlap disclosure (unchanged and still safe):** **36 of 694** baseline lines match more than one
family pattern; first-match priority resolves each to exactly one family. **0** overlapping lines
touch the class-A-only `corpus` family. Under the corrected classification the overlaps are
A↔B in some cases (e.g. `for_await_string_primitive_object_enumeration_*` matches for-await=A1,
string-iter=A1/B1, object-enum=mixed), so **priority order can now affect a class call** where the
old revision could claim it never did. This is safe in practice only because the wave action is
per-test pin-to-observed-diagnostic; **a wave must not infer a test's class from its family label.**

---

## 7. Wave plan recommendation

Ordering principle: **deny lanes first, largest-blast-radius first, pin-only waves last**, so that
no pin is written while a silent miscompile in the same construct is still live.

### Phase 1 — deny lanes (class B). 10 waves.

| order | wave | tests | effort | rationale |
|---:|---|---:|---|---|
| 1 | **B3** module-scope growable push + element store | 29 | **S** | Smallest and cheapest: registering `_start` in `growable_rejects` (without enabling promotion) converts silence into an honest `E5506`. Closes a silent class affecting arbitrary user code, not just tests. Do this first — it also removes the contamination that broke the original triage. |
| 2 | **B1** interned string-handle leak | 115 | **L** | Largest B group and the most dangerous (internal representation reaching stdout). Use an **allowlist at the single read site**, not a denylist of sinks — this repo has learned that lesson at least three times (for-in keys, `_start` abort handles, stage-D C-1). |
| 3 | **B4** Promise combinators | 128 | **M** | Largest by count but structurally simple: one placeholder-returning lowering to fail closed. |
| 4 | **B2** enumeration spread | 45 | **M** | Shares the `static_analysis/array.rs` iterable classifier; sequence after B1 so the string-handle decision is already made. |
| 5 | **B5** microtask checkpoint | 18 | **M** | Self-contained in the event-loop drain. |
| 6 | **B6** `&&`/`||` short-circuit | 4 | **M–L** | Only 4 tests, but the *worst* defect found: it silently changes control flow in arbitrary user code. Likely needs a parser-level `LogicalExpression` node. **Recommend promoting this above its test count** — consider a real fix rather than a deny lane. |
| 7 | **B7** TextEncoder / WebCrypto | 4 | **S** | Narrow host-builtin surface; overlaps planned Stage P5. |
| 8 | **B8** Deno host env/cwd | 4 | **S** | Narrow host-builtin surface. |
| 9 | **B9** `Object.hasOwn` | 28 | **?** | **Blocked on isolation.** Budget an investigation spike before any lane work; do not schedule as a normal wave. |
| 10 | **B10** Map/Set nullish sub-shape | 9 | **?** | **Blocked on isolation.** Same. |

### Phase 2 — pin-only (class A). 5 waves, no product change.

| order | wave | tests | effort |
|---:|---|---:|---|
| 11 | **A1** growable escape via function argument | 195 | **M** — large but mechanical; one reason string, applied per test |
| 12 | **A2** `try`/`catch`/`finally` | 46 | **S** |
| 13 | **A3** fixed-shape enumeration | 32 | **S** |
| 14 | **A4** array literal passed to a function | 22 | **S** |
| 15 | **A5** corpus | 15 | **S** — but **re-verify each row's terminating diagnostic at pin time**; it is per-package and moves as features land |

### Standing wave obligations

- **Pin per test to that test's own observed diagnostic**, never per file and never per family label
  (§5, mixed-class files).
- **Re-run the reproducer on a freshly built binary** before believing any fix report — this repo has
  been burned by fix reports that were false.
- The **18 duplicated names** (§1) must be patched in **both** defining binaries.
- **A corpus row must be pinned to a terminating error or trap.** The corpus build also emits `E3100`
  *zero-placeholder warnings* (`describe`, `CustomEvent`, `dispatchEvent`, `Event`, `URLSearchParams`,
  `String`) before the terminating `E5506`. `E3100` is a latent silent-miscompile vector: if any
  corpus package ever reaches exit 0 on `E3100`-only warnings, that package is class B and must be
  split out with its own deny lane.

### Scope-exception candidates (maintainer decides — recommending, not deciding)

- **B6 (`&&`/`||`)**: 4 tests, but a control-flow-changing silent miscompile in the core language. A
  real fix is very likely cheaper *in risk terms* than a deny lane, and a deny lane on `&&`/`||`
  would reject an enormous amount of ordinary code.
- **B3 (module-scope growable)**: a full fix (enable `_start` promotion) is estimated contained — the
  runtime machinery already works at module scope through an object field (`const o={v:[]};
  o.v.push(3)` is node-correct). It would green ~29 tests instead of pinning them. The fail-closed
  variant is still the right *first* move.
- **B1 (string handles)**: `.length` and control flow are already correct; only the value read leaks.
  A narrow real fix at the read site may be comparable in cost to a sound deny lane.
