# throw-fallout Stage 6 — repr-tracked function scopes, then un-flatten block arrows

**Program:** throw-fallout (prerequisite for Soundness Batch 1 PR-A / PR #16).
**Branch:** `soundness-batch1-pra`. **Stage base:** `9fdf180a2`.
**Stage-entry denominator:** **731** (measured this session, two enumerations, `sort -u`).
**Drain target: NOT CLAIMED.** See §6 — this stage is soundness + unblocking, not denominator movement.

Supersedes the sequencing (not the content) of
`docs/superpowers/specs/2026-07-14-throw-fallout-stage6a-async-suspension-design.md`, which is
**re-sequenced behind this stage** — the async ordering core is correct but drains ~0 on its own
(§6.1), and its `queue_microtask` target set is blocked on the very bug this stage fixes.

---

## 1. The bug — the parser destroys a block-bodied arrow and executes its body inline

A block-bodied arrow in **call-argument position** does not parse as a function at all. Measured
against the real parser this session (`kali_parser::Parser::parse`, throwaway probe, since removed):

```js
queueMicrotask(() => { ran = 1; });
```

parses to **two** statements:

```
ExpressionStatement( CallExpression {
    callee: Identifier("queueMicrotask"),
    args: [ ParenthesizedExpression( Identifier("unknown") ) ],   // <-- the arrow is GONE
})
BlockStatement { ran = 1; }                                        // <-- its body, spliced inline
```

The arrow is erased, its argument becomes the identifier `unknown`, and its **body is torn off and
spliced into the program as a bare block that executes in sequence**. This is not an optimizer
inlining a callback — it is a parse-level silent miscompile, exit 0, no diagnostic.

**Scope of the defect (measured):**
- Expression-bodied arrows (`() => log(1)`) parse correctly, in every position.
- Block-bodied arrows parse correctly **only** in variable-declarator init position
  (`try_parse_block_arrow_function_expression`, `crates/kali_parser/src/declaration.rs:298`), which
  additionally hardcodes `is_async: false` (`declaration.rs:326`).
- Everywhere else — call arguments above all — the arrow-parse path **bails on `{`**
  (`crates/kali_parser/src/declaration.rs:272-275`) and the body flattens into the enclosing scope.

**Blast radius.** Every callback-taking API: `queueMicrotask` (50 fixture uses), `setTimeout`,
`.map`/`.filter`, and **the entire `Kali.test` lane** — `Kali.test('x', () => { ... })` is the shape of
essentially every test fixture in the repo. Today those test **bodies execute inline during `_start`**
and a failing self-check surfaces as a `_start` trap rather than as an attributed test failure. The
harness has been resting on that accident (`crates/kali_runtime/src/execute.rs:366-373` documents the
"flattened callback lane" in so many words).

---

## 2. Why it is still open — untracked function scopes

**The parse fix is already written and applies cleanly to today's tree**:
`docs/superpowers/followups/task8-block-arrows-deferred.patch` (verified with `git apply --check` this
session). It touches exactly two files:

- `crates/kali_parser/src/expression/primary.rs` — block-bodied arrows parse as an unnamed
  `FunctionExpression` in **all** expression positions (including single-param `x => { … }`).
- `crates/kali_types/src/resolve/call.rs` — `reject_anonymous_function_argument`: an anonymous
  fn/arrow argument is `E5506`-rejected when the callee is a **bound, non-builtin plain identifier**
  (nothing can invoke it; monomorphized dispatch keys on a function *name*). Member-call consumers
  (`Kali.test`, `arr.map`, `p.then`) are exempt structurally; builtin identifier consumers
  (`queueMicrotask` / `setTimeout` / `setInterval`) are exempt by builtin-list.

**It was deferred because landing it alone regresses 16 tests**, and the reason is a real, separate
gap. Un-flattening routes callback **bodies** through **untracked function scopes**:

> `crates/kali_types/src/context.rs:44-54` — arrow functions, function expressions, class methods, and
> `export default function` **never push onto `current_function`**. `binding_repr_function_key`
> (`crates/kali_types/src/resolve/expression.rs:297`) walks out to the nearest `ScopeType::Function`
> scope and returns **`None`** the moment that scope is not the tracked one (`expression.rs:306-308`).

`None` means "no repr proof", and the resolver is (correctly) fail-closed — so inside any such body,
anything requiring a proof rejects `E5506` or traps `E4000`: compound assignment (`+=`), logical and
nullish assignment (`&&=`, `??=`), `new`, `typeof`, `instanceof`. Today the module-scope flatten
**masks** this, because the body isn't in a function scope at all.

**Named fallout if the patch lands alone** (from the deferred doc, to be re-measured, not trusted):
`compound|logical|nullish_assignment_wrapped_local_binding` (test + json_test variants, **12**) and
`object_type_and_constructor_semantics` (test variant, **4**).

**The ordering is therefore forced, and that is the whole design.**

---

## 3. The design

### 3.1 Task order (non-negotiable)

1. **Make function-shaped scopes repr-tracked.** Push arrow / function-expression / class-method /
   `export default function` bodies onto `current_function` **and** `current_function_scopes`, with a
   real `binding_repr_function_key`. Codegen **already** compiles these bodies as synthetic-named
   functions (`__kali_fn_N`, `crates/kali_hir/src/lowering/mod.rs:88`) — it is the `kali_types` half
   that never caught up. The two halves get back in step; the fix is to align them, not to invent a
   new mechanism.
2. **Then** apply the parse patch. With scopes tracked, the 16 regressions never materialize.
3. **Then** gate on the full workspace.

Landing (2) before (1) is the one thing this stage must not do.

### 3.2 The class-method corollary — verify, don't assume

Stage 5 recorded a pre-existing silent miscompile: **`class C { run(){ return 42; } } new C().run()`
→ `0`**. Class-method bodies are named in `context.rs:48-54` as one of the untracked function-shaped
scopes, so (1) is very likely its root cause. **This is a hypothesis, and the stage must test it**
with the Stage-5 reproducer rather than assert it. If it greens, say so with the probe output; if it
does not, record why — the untracked-scope fix is justified on its own terms either way.

### 3.3 The naming key — name it in the AST, before the resolver

Every function-shaped body needs a *stable, unique* `binding_repr_function_key`, and both halves of
the compiler must agree on it. Today they **cannot even in principle**:

- Codegen's synthetic names (`__kali_fn_N`) are minted during **HIR lowering**, which runs *after* the
  resolver — `lower_arrow_function_expression` (`crates/kali_hir/src/lowering/function.rs:47`) calls
  `next_synthetic_function_name()` unconditionally and **ignores any existing id**.
- `FunctionExpression` *has* an `id: Option<String>` (`crates/kali_ast/src/expression.rs:129`), but
  **`ArrowFunctionExpression` has no id field at all** (`expression.rs:145-150`).

Re-deriving HIR's counter inside `kali_types` would be a **hand-mirrored oracle**, and this repo has
paid for that twice (Spec-2: mirrored predicates fail open; Stage-5: the two hand-mirrored specifier
folds agreed on the *same wrong* module, which is why no gate fired). A mirrored-but-divergent key
fails **open** — strictly worse than today's fail-closed `None`.

**So do what Spec 5 already proved out: name it in the AST, before the resolver, and let everything
key on the name.** (Spec-5 monomorphization: AST-level clones named `f${N}` ahead of the resolver;
"zero codegen/repr edits since everything keys on function-name.")

1. Add `id: Option<String>` to `ArrowFunctionExpression`, matching `FunctionExpression`.
2. A small AST pre-pass names every anonymous function-shaped node (`__kali_fn_{N}`) **before** the
   resolver runs — the same slot `module_link` and `monomorphize` occupy.
3. `kali_types` uses that `id` as the `binding_repr_function_key`; HIR lowering **uses the `id` when
   present** instead of minting its own.

One name, written once, read by both halves. The identity is structural rather than mirrored, so it
cannot silently diverge. **A test must still prove it:** a repr fact recorded under the key inside an
arrow body must be the same fact codegen reads back.

---

## 4. Scope boundary

**In:** repr-tracking for arrow / function-expression / class-method / `export default function`
bodies; the parse patch (block arrows in all expression positions); `reject_anonymous_function_argument`;
the `Kali.test` callback lane becoming real (bodies stop executing inline in `_start`).

**Out (unchanged, deferred):** the async ordering core and everything in the Stage-6a spec (now
re-sequenced *after* this stage); the four Promise combinators; `const`-bound-call double evaluation
(measured: `uses + 1` evaluations — a separate live miscompile, and generated/fixture code must keep
using `let`); the parser's `_ => None` silent statement drop (I2, deferred).

**Fail-closed contract.** Anything the un-flattening cannot prove keeps rejecting. The
`reject_anonymous_function_argument` guard is the choke point for "an anonymous function reached a
callee that cannot invoke it". Per the Stage-5 headline lesson, that guard is an **allowlist of
positions** (member-call consumers, builtin identifier consumers) — not a denylist of shapes — and a
census walk over anonymous-function arguments must have **no `_ =>` arm**.

---

## 5. Verification

**Gate discipline** (unchanged, five stages running): two independent
`cargo test --workspace --no-fail-fast` enumerations on a fresh binary, **`sort -u`** (18 test names
exist in two harness binaries each; raw `sort` fabricates newly-red), diffed then unioned; **PRIMARY
GATE = `comm -13 pre post` must be EMPTY**; cross-checked against a **`main` worktree**, never a
mid-branch baseline. Entry = **731**.

**The 16 named regressions are the gate's whole job.** They are the *prediction* this stage's task
order is designed to falsify. If they appear, the repr-tracking is incomplete and the parse patch must
come back out — not be papered over with re-pins.

**Tests must be written from scratch.** The deferred doc claims "`crates/kali_cli/tests/soundness_block_arrows.rs`:
4 tests, all pass" — **that file does not exist**, in the tree or in the patch. Verified this session.
A doc claim that outruns the code is itself a defect (Stage-5 corollary), and this one would have sent
an implementer looking for tests that were never committed.

**Acceptance evidence must be distinguishable and node-compared**, because the legacy fixtures cannot
tell a real callback from a flattened body:
1. `queueMicrotask(() => { ran = 1; })` — the callback must **not** run inline (`ran` still `0` at the
   next statement), byte-compared against node.
2. A block-arrow callback body using `+=`, `&&=`, `??=`, `new`, `typeof` — must compile and run, not
   `E5506` (this is the repr-tracking payoff, and the exact shape of the 16).
3. A `Kali.test` block-arrow callback whose self-check throws — must be attributed as **`FAILED 1`**,
   not surface as a `_start` trap.
4. The Stage-5 class-method reproducer (§3.2).

**Adversarial re-mask probes (mandatory).** Fix reports are not evidence — re-run on a freshly built
binary. Sabotage the scope push so bodies go untracked again → test 2 must go red with `E5506`.
Sabotage the parse patch → test 1 must go red with the body running inline. A suite that stays green
under either sabotage is measuring nothing.

---

## 6. On the drain — deliberately not claimed

**No drain number is asserted for this stage, and none should be inferred.** The value is: closing a
top-tier silent miscompile (a callback's body executed inline, its arrow erased); making the
`Kali.test` lane honest rather than accidentally-correct; and unblocking the 22 `queue_microtask` names
plus the entire async stage. If a drain materializes (plausibly via the class-method corollary), it is
a bonus to be *measured and reported*, not forecast.

### 6.1 Why the async stage was re-sequenced (the correction that produced this spec)

The Stage-6a spec claimed a drain of ~49 (27 `async_await_sequencing` + 22 `queue_microtask`). Both
halves were wrong, and the error was trusting the Stage-0 doc's bucket *prose* instead of the test
names:

- **There is no `async_await_sequencing` test.** `grep -c async_await_sequencing` over the failing set
  → **0**. The 27 `await_*` names are all `for_await_*` / `await_wrapped_*` **enumeration** tests,
  multi-blocked on the #2/#3 enumeration causes rather than on await suspension.
- **The 22 `queue_microtask` names are blocked on the bug in this spec**, not on the scheduler: their
  fixture is `queueMicrotask(() => { microtaskRan = true; })`, a block-bodied arrow the parser
  destroys.

So the async ordering core, though correct and still wanted, drains ~0 on its own. Recorded here
because the same failure mode — a *bucket label* believed over the *test names* — is exactly what the
gate discipline exists to catch, and it nearly shipped a stage aimed at a target set that did not
exist.
