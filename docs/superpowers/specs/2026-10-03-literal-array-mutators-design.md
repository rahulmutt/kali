# An in-place mutator on a literal array runs as node runs it, or kali refuses

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `9dc751cf8` (`main`, the array-bounds merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali`, built at the baseline (`cargo build -p kali_cli`, `dev` profile) |
| oracle | `node v26.10.0` |
| measured on | 2026-10-03 |
| item picked | `docs/superpowers/followups/array-bounds-discovered-defects.md` §9, "The literal-array lane still silently no-ops mutators" |
| defects this closes | that §9, plus §2 (`reverse` / `sort` / `copyWithin` on a plain runtime array) and §13 (`a.push?.()` on a plain runtime array); every row of §2.1 below |

**Scope was chosen by the human partner:** fail closed (option A of four;
the others were "real semantics" (implement the mutators on the growable lane
and promote top-level literals), "split", and "a different item"). Mechanism:
a receiver-keyed refusal gate that `kali check` mirrors, plus a codegen
backstop keyed on the method name (approach 3 of three; §4 records the other
two). Folding §2 and §13 into the shared mutator list was approved with the
design's first section.

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** an in-place array mutator called on an array literal never
silently does nothing. For an array literal bound by `const`, `let` or `var`,
at top level or in a function, with numeric, string or object elements, and
for a bare array-literal expression, an in-place mutator call or a `.length`
write either produces node's output or is refused. Concretely:

1. A call to `push`, `pop`, `shift`, `unshift`, `splice`, `reverse`, `sort`,
   `fill` or `copyWithin`, or an assignment to `.length`, refuses with `E5506`
   under both `kali check` and `kali run`. The receiver is a literal-array
   binding or an array-literal expression, optionally wrapped in parentheses,
   `as`, `satisfies` or an optional chain. The call may be spelled `a.m()`,
   `a["m"]()` or `a.m?.()`.
2. **The working growable lane is untouched.** An in-function binding that
   `growable_array_candidates` (`kali_types/src/growable.rs:122`) promotes
   keeps compiling: its `push`, `.length`, index read, `for…of` and `.join`
   still print node's output. The scan's existing `E5506` for a `push` mixed
   with another mutator is unchanged.
3. **The `run` backstop.** No mutator call is dropped and replaced by `0` at
   `kali run`, whatever its receiver is: an alias (`const b = a; b.pop()`), a
   parameter, or anything else kali cannot classify. `run` refuses with
   `E5506`: a parameter or unclassified receiver gets the backstop text, and an
   alias that codegen traces to its literal gets the literal text (A-8).
4. **§2 and §13 close with the shared list.** On a plain fixed-length runtime
   array (array-bounds spec §1), `reverse`, `sort` and `copyWithin` now refuse
   under `check` and `run`, as does an optional call `a.push?.(1)`. `fill`
   works on that lane and stays allowed.
5. A program that calls no in-place mutator behaves as it does at the
   baseline.

### 1.1 What this project does NOT claim

* **No mutator semantics.** Nothing that refuses today starts working. The
  real fix is a feature: growable literals beyond `push`, and top-level
  promotion. It is filed (§6), not built.
* **No top-level growable promotion.** `const a = []; a.push(5);` at top level
  refuses. It does not start working.
* **Out of scope:** array-bounds followups §1 (growable out-of-range read), §3
  (anonymous-lane `check` gap), §6 (float index), §10 (write-trap ordering),
  §11 (shadowing) and §12 (growable returns), and R-21's `undefined`.
* **One disclosed `check` / `run` gap.** When the receiver is an alias or a
  parameter of a literal array, `kali check` exits 0 and `kali run` refuses
  (§3.4, amendment A-8: a parameter gets the backstop text, an alias the
  literal text). The type layer cannot tell which calls codegen
  will fail to resolve, so it cannot soundly mirror the backstop. This follows
  the precedent of array-bounds followups §4.

---

## 2. What was measured

### 2.1 The silent surface at the baseline

Every program was run as `node P.js`, `kali check P.js` and `kali run P.js`.
`t_*` programs are top level. `f_*` programs wrap the same body in
`function main(){…} main();`. Every row is wrong at exit 0, and `check` exits 0.

| row | program body | node | kali run |
|---|---|---|---|
| t_push | `const a=[1,2,3]; a.push(4); console.log(a.length); console.log(a[3]);` | `4`, `4` | `3`, `undefined` |
| t_pop | `const a=[1,2,3]; const x=a.pop(); console.log(a.length, x);` | `2 3` | `3 0` |
| t_len | `const a=[1,2,3]; a.length = 1; console.log(a.length);` | `1` | `3` |
| t_shift | `const a=[1,2,3]; a.shift(); console.log(a[0]);` | `2` | `1` |
| t_unshift | `const a=[1,2,3]; a.unshift(0); console.log(a[0]);` | `0` | `1` |
| t_splice | `const a=[1,2,3]; a.splice(0,1); console.log(a[0]);` | `2` | `1` |
| t_rev | `const a=[1,2,3]; a.reverse(); console.log(a[0]);` | `3` | `1` |
| t_sort | `const a=[3,1,2]; a.sort(); console.log(a[0]);` | `1` | `3` |
| t_fill | `const a=[1,2,3]; a.fill(9); console.log(a[0]);` | `9` | `1` |
| t_cw | `const a=[1,2,3]; a.copyWithin(0,2); console.log(a[0]);` | `3` | `1` |
| t_empty_push | `const a=[]; a.push(5); console.log(a.length);` | `1` | `0` |
| f_pop | `const a=[1,2,3]; const x=a.pop(); console.log(a.length); console.log(x);` | `2`, `3` | `3`, `0` |
| f_len | `const a=[1,2,3]; a.length = 1; console.log(a.length);` | `1` | `3` |
| f_shift / f_unshift / f_splice / f_rev / f_sort | as the `t_` rows | as `t_` | as `t_` |
| paren | `const a=[1,2,3]; (a).pop(); console.log(a.length);` | `2` | `3` |
| strkey | `const a=[1,2,3]; a["pop"](); console.log(a.length);` | `2` | `3` |
| opt | `const a=[1,2,3]; a.push?.(4); console.log(a.length);` | `4` | `3` |
| strlit | `function main(){ const a=["x","y"]; a.reverse(); console.log(a[0]); } main();` | `y` | `x` |
| objlit | `function main(){ const a=[{v:1},{v:2}]; a.pop(); console.log(a.length); } main();` | `1` | `2` |
| nameless | `console.log([1,2].push(3));` | `3` | `0` |
| closure | `function main(){ const a=[1,2,3]; const f=()=>a.pop(); f(); console.log(a.length); } main();` | `2` | `3` |
| alias | `function main(){ const a=[1,2,3]; const b=a; b.pop(); console.log(a.length); } main();` | `2` | `3` |

Not silent, but relevant:

| row | program | node | kali |
|---|---|---|---|
| t_let_push | `let a=[1,2,3]; a.push(4); console.log(a.length);` | `4` | `E3100` warning (placeholder `push`), exit 1; `check` exits 0 |
| var_push | `var a=[1,2,3]; a.push(4); …` | `4` | same as t_let_push |
| param | `function g(x){ x.pop(); } function main(){ const a=[1,2,3]; g(a); console.log(a.length); } main();` | `2` | `E3100` warning (placeholder `pop`), exit 1; `check` exits 0 |
| t_idx | `const a=[1,2,3]; a[0]=7; console.log(a[0]);` | `7` | `E5506` "mutating a literal array…" under `check` and `run` |
| gr_push_rev | `function main(){ const a=[1,2]; a.push(3); a.reverse(); … }` | `3` | `E5506` (growable scan) under `check` and `run` |

Rows from array-bounds followups §2 and §13 (`new Array(3).fill(4)` receiver)
are re-measured as probes (§5.2) and are not repeated here.

### 2.2 Controls at the baseline

These are correct today and must stay correct:

| row | program | node and kali |
|---|---|---|
| f_push | `function main(){ const a=[1,2,3]; a.push(4); console.log(a.length); console.log(a[3]); } main();` | `4`, `4` |
| f_empty_push | `function main(){ const a=[]; a.push(5); console.log(a.length); console.log(a[0]); } main();` | `1`, `5` |
| f_loop | `function main(){ const a=[]; for (let i=0;i<3;i++) a.push(i); let s=0; for (const x of a) s+=x; console.log(s); } main();` | `3` |
| plain fill | `function main(){ const a=new Array(3).fill(4); a.fill(5); console.log(a[1]); } main();` | `5` |

### 2.3 Mechanism

* **No gate covers a mutator call on a literal.** Two refusal gates exist.
  The array-bounds gate needs a *plain runtime array* receiver:
  `is_plain_runtime_array_receiver` (`kali_types/src/resolve/member.rs:399`) on
  the type side, and `plain_runtime_array_mutator` / `is_runtime_array_value`
  (`kali_codegen/src/emit/call.rs:42-63`) in codegen. Both are keyed on
  `new Array` / `.fill` / array-return bindings. The literal-store gate
  `reject_literal_array_unfoldable_mutation`
  (`kali_types/src/resolve/expression.rs:1651`) covers only assignments. A
  method call on a literal binding passes both.
* **Unresolved calls drop to the terminal fallback.** It sits at the end of
  `emit_call` (`call.rs:3929-3940`). It never emits the receiver, drops the
  arguments, pushes `i64.const 0`, and records an `E3100` warning through
  `push_placeholder_fallback_diagnostic` (`kali_codegen/src/emitter.rs:1380`).
  Just above it, a deny-set `deny_placeholder_lowering` (`call.rs:4551`)
  already fails some value builtins closed. It is the natural home for the
  §1 item 3 backstop.
* **A second, warning-free drop route.** The top-level `const` rows (t_push,
  t_pop, …) print no `E3100` warning, so they are dropped before they reach
  the terminal fallback. That route is not yet located. Plan Task 1 traces it
  (§5.1).
* **Reads keep folding.** A top-level `const` literal gets
  `ConstPromotion::Fold` (`kali_codegen/src/lower.rs:6050-6130`), so each read
  re-emits the initializer. The mutation scan `collect_mutated_binding_names`
  (`kali_optimize/src/object_fold.rs:519`) sees member stores and `delete`,
  not mutator calls. So after a dropped call, reads still serve the original
  elements.
* **The growable lane only admits `push`.** `growable_array_candidates`
  promotes an in-function binding only when it has a `push` and every other
  use is `.length`, an index read, `for…of` or `.join`. Any other mutator
  makes it unsafe, and with a `push` present that becomes an `E5506`
  (`growable_rejects`, `repr_infer.rs` near 7516). An in-function literal with
  a mutator but *no* `push` is never promoted. It stays on the fold lane and
  is dropped like a top-level one. The top level is never scanned
  (`repr_infer.rs:2831-2850`, `:2935-2945`).

---

## 3. Design

Every call is decided in one order, and the first match wins:

1. The receiver is growable and the call is `push`: lower it as today.
2. The receiver is plain or literal and the method is on that lane's list:
   refuse with that lane's message, under both `check` and `run`.
3. The call reaches a placeholder drop with an in-place mutator name: refuse,
   under `run` only.

A call that is not an in-place mutator never matches a list, and nothing about
it changes.

### 3.1 Shared vocabulary (`kali_common/src/messages.rs`)

* `RUNTIME_ARRAY_MUTATORS` (`:148`) gains `reverse`, `sort` and `copyWithin`.
  `fill` stays out, because the plain lane implements it correctly. The doc
  comment changes from "methods that change a runtime array's length" to
  "methods with no lowering on a plain runtime array".
* New `LITERAL_ARRAY_MUTATORS`: the same list plus `fill`.
* `runtime_array_mutator_unavailable_message` is reworded so it is true of
  `reverse` and `sort`, which do not change length. The new text names the
  missing lowering instead of the fixed length:
  "calling `.{method}()` on a runtime array is unavailable in the current
  phase: kali has no lowering of it on this array, so kali refuses rather than
  silently skip the call". The `.length` write wording is unchanged.
* New `literal_array_mutator_unavailable_message(method)`, roughly:
  "calling `.{method}()` on a literal array is unavailable in the current
  phase: kali folds a literal array to its initial elements, so kali refuses
  rather than silently skip the call".
* New `literal_array_length_write_unavailable_message()` in the same shape.
* The existing literal-store text ("mutating a literal array is unavailable in
  the current direct-runtime path; use new Array(n) for runtime mutation") moves
  into this file unchanged, as a named function. All literal-lane wording then
  comes from one place.
* New `array_mutator_unresolved_receiver_message(method)` for the backstop,
  roughly: "calling `.{method}()` is unavailable in the current phase: kali
  could not prove which array the receiver is, so kali refuses rather than
  silently skip the call".

Exact wording is settled in the plan and pinned by the case files. Changing
`runtime_array_mutator_unavailable_message` re-pins the array-bounds cases
that quote it. That is expected, and the plan lists them.

### 3.2 The `kali_types` gate (`resolve/member.rs`)

* New predicate `is_literal_array_receiver(object)`. It unwraps through
  `expression::unwrap_transparent`. It holds for an `ArrayExpression`, and for
  an `Identifier` where `resolve_array_literal_binding_name` holds and
  `is_growable_array_binding` does not. The binding is resolved through scope,
  so a closure that captures the outer `a` is covered (row closure).
* `reject_runtime_array_mutator_call` (`:428`) becomes the single dispatcher
  for both lanes. It accepts:
  * a static member callee `a.m`
  * a string-literal computed key `a["m"]`
  * an optional call `a.m?.()`

  Then:
  * a plain receiver with a `RUNTIME_ARRAY_MUTATORS` method gives the runtime
    message
  * a literal receiver with a `LITERAL_ARRAY_MUTATORS` method gives the
    literal message
  * no other combination gives a diagnostic

  The plain-lane optional-call coverage is what closes followups §13.
* `reject_runtime_array_length_write` (`:447`) gains a literal arm with the
  literal `.length` message.
* `reject_literal_array_unfoldable_mutation` keeps its logic. It only switches
  to the shared message function.

### 3.3 The codegen gate (`emit/call.rs`, `emit/literal.rs`)

* `plain_runtime_array_mutator` (`call.rs:42`) gets a literal sibling,
  `literal_array_mutator`. It is keyed on codegen's own literal and fold facts
  for the bare receiver name (`bindings` plus `is_array_literal`, as the
  static-index read lane resolves them) or on an array-literal node. The
  receiver is unwrapped through `unwrap_transparent`. It is called beside the
  plain check (`call.rs:1669`), after `growable_push_call_parts`, so a
  growable `push` is still taken first.
* The literal `.length` write refuses in `emit/literal.rs` next to the plain
  `.length` write refusal (`:662-672`).
* This mirrors §3.2 shape for shape, so `check` and `run` agree on every row of
  §2.1 except alias and param.

### 3.4 The `run` backstop

* `deny_placeholder_lowering` (`call.rs:4551`) gains a member-callee arm: a
  `LITERAL_ARRAY_MUTATORS` name with a receiver child is denied, using
  `array_mutator_unresolved_receiver_message`. A user object's own `push` or
  `sort` method resolves before the terminal fallback, so it never reaches
  this arm (pinned by a control, §5.2). A deferred-registration surface has
  none of these names.
* Every other route that drops a mutator call without emitting it is closed
  by the same rule at its own site. That includes the warning-free top-level
  `const` route of §2.3. Plan Task 1 finds these routes, and the probes fail
  if one is missed.
* There is no `check` mirror (§1.1). An alias receiver is usually refused by
  the literal gate in codegen before it reaches this arm (A-8).

### 3.5 Diagnostics

There are no new codes. Every refusal is `E5506`. `specs/15-errors.md` line
223, the array-bounds sentence, is widened to name:

* the added plain-lane methods and the optional-call spelling
* the literal lane
* the backstop and its alias/parameter `check` gap

The CLI surface, schemas and maturity are unchanged, so the AGENTS.md §6 CLI
change packet applies only as far as `15-errors.md`.

---

## 4. Approaches not taken

* **Receiver gate only (approach 1).** `check` and `run` would always agree,
  but alias and parameter receivers would stay silent at exit 0. Array-bounds
  §7 and §13 show that a name-keyed gate alone leaves shapes open.
* **Backstop only (approach 2).** This is sound at `run`, but `kali check`
  would exit 0 on every row, a `check` / `run` gap across the whole surface.
* **Real semantics.** The human partner declined it for this project (§0).

---

## 5. Testing and measurement

### 5.1 Trace first (plan Task 1)

Before writing any gate, find every codegen route a mutator call on a literal
receiver can take and still be dropped. At least the terminal fallback and the
warning-free top-level `const` route exist. Record each route with
`file:line` in the plan. The probes of §5.2 make this checkable: a route that
is missed leaves a row SILENT.

### 5.2 Probes and cases

* **Probes:** new files in `tools/array-return-probes/probes/`.
  * `litmut_*` (about 40): every §2.1 and §2.1-relevant row. Each mutator
    appears on each lane (top-level `const`, `let` and `var`; in-function
    numeric, string and object elements). Also covered: the wrapped spellings
    (`(a)`, `as`, `a["m"]()`, `a.m?.()`), nameless, closure, alias, param, and
    `.length` writes. Array-bounds followups §2's `reverse`, `sort` and
    `copyWithin` rows and §13's `a.push?.(1)` are on a
    `new Array(3).fill(4)` receiver.
  * `litmut_ok_*` (about 12): the §2.2 controls, `slice` / `indexOf` / `join`
    on a literal, a growable `push` loop, `fill` on a plain array, and a user
    object literal with its own `push` and `sort` methods that are called.
* **Runner:** `run.sh` gains a fifth TSV column, the `kali check` exit code.
  `check` / `run` agreement is then measured mechanically. The existing
  `cut -f1,2` diffs are unaffected.
* **Baseline:** `baseline-litmut.tsv` is measured at `9dc751cf8` and committed
  before any gate.
* **Pass condition:**
  * every `litmut_*` row goes from SILENT or OTHER to REFUSES
  * the check column is non-zero on every `litmut_*` row except alias and param
  * no `litmut_ok_*` row moves
  * no row of `baseline.tsv`, `baseline-anon.tsv` or `baseline-bounds.tsv`
    moves, except the §2 and §13 rows, which become REFUSES
* **Case files:** new `.toml` files go in `crates/kali_cli/tests/cases/array/`:
  * one per lane (literal top level, literal in function, plain lane
    additions), pinning the `E5506` text under both `check` and `run`
  * one for the backstop (alias and param: `check` exits 0, and `run` refuses)
  * one for the controls, matching node

  There is no new `tests/*.rs` target.

### 5.3 Unit tests (sibling `*_tests.rs` files)

* `kali_types`:
  * `is_literal_array_receiver` holds on each binding kind and wrapper, and on
    an array-literal expression
  * it does not hold on a growable binding or on a plain runtime array
  * the dispatcher picks the right message on each lane and none on a
    non-mutator
* Codegen:
  * `literal_array_mutator` matches what §3.3 says
  * growable `push` still precedes it
  * the `deny_placeholder_lowering` arm denies a mutator name with a receiver
    and leaves a non-mutator alone
* `kali_common`: `LITERAL_ARRAY_MUTATORS` is `RUNTIME_ARRAY_MUTATORS` plus
  `fill`, exactly.

### 5.4 Capability loss

Some programs ran correctly by luck at the baseline and refuse after this
project. Examples: a `fill` or `sort` on a literal that is never read again,
a mutator in dead code, or a cross-package import call named like a mutator
that previously took the placeholder. These are measured, listed with
node/baseline/HEAD columns in the followups file, and brought to the human
partner. Unless the human partner decides otherwise, array-bounds decision A-5
applies: accept the refusals as fail-closed.

### 5.5 Gates

`cargo test --workspace` and `cargo clippy --workspace` pass. That includes
the `oracle/` cases (R-21's `r21o` stays `silent`) and
`kali_blast_radius::ranking::ranking_tests::spliced_document_matches_the_generator`.

---

## 6. Bookkeeping

* `array-bounds-discovered-defects.md` §2, §9 and §13 are marked FIXED
  (fail-closed), with the closing commit and a pointer here. The note says the
  calls now refuse rather than run.
* A new `docs/superpowers/followups/literal-array-mutators-discovered-defects.md`
  records what was measured and not fixed. At minimum:
  * the alias/param `check` gap
  * the §5.4 capability-loss table
  * the probe diff recomputed at the end of the branch
  * the follow-up feature (real mutators on the growable lane, top-level
    promotion)
* `kali-silent-miscompile-register.md` is amended only if a §0.2 lane moves.
  This is checked by re-running the oracle cases, not assumed. If a lane
  moves, the ranking is regenerated with
  `cargo run -p kali_blast_radius --example rank`.
* `specs/15-errors.md` changes as §3.5 says.
* Two defects found while planning (A-1) are filed in the new followups file:
  warnings are discarded on a successful build, and any unresolved member call
  evaluates to `0`.

---

## 7. Amendments

Added while writing the implementation plan, before any code.

* **A-1. There is one drop route, not two.** §2.3 said the top-level `const`
  rows are dropped by a warning-free route before the terminal fallback. They
  are not. A trace with temporary instrumentation (since reverted) showed that
  every row of §2.1 enters `emit_call` and leaves through the terminal fallback
  (`call.rs:3937`), which does push the `E3100` warning. The warning is then
  discarded: `compile_source_file_uncached`
  (`kali_cli/src/build/compile.rs:431-517`) returns diagnostics only on its
  `Err` path, so a build with no error drops every warning. `var_push` and
  `param` showed the warning only because an unrelated error sent the build down
  that `Err` path. Consequences:
  * §3.4's backstop at the terminal fallback covers every row. §5.1's trace task
    is replaced by a pinned case showing the backstop fires on the top-level
    `const` rows when the type gate is absent (a codegen unit test).
  * Two wider defects are filed (§6), not fixed: warnings are discarded on a
    successful build, and *any* unresolved member call (`o.zork(4)` on any
    receiver) still evaluates to `0` at exit 0.
* **A-2. The backstop is a sibling arm, not a `deny_placeholder_lowering`
  entry.** That function returns `bool`, and its caller pushes one fixed
  "recognized builtin" message. The backstop needs
  `array_mutator_unresolved_receiver_message`. So it is its own check placed
  immediately before the `deny_placeholder_lowering` call, with the same
  drop-arguments-and-push-0 shape.
* **A-3. One diagnostic for `push` mixed with another mutator.** In a function,
  `const a=[1,2]; a.push(3); a.pop();` is a growable reject *and* a literal
  mutator. The resolve pass runs first, and `kali run` / `kali check` return on
  its errors before `repr_infer`'s shape conflicts are reported
  (`compile.rs:768-785`). So the user sees the literal-mutator `E5506` alone,
  where at the baseline they saw the growable-scan `E5506`. Both are refusals.
  The case that pins `gr_push_rev` asserts `E5506` and the `.pop()` mention,
  not the old growable wording.
* **A-4. One case file, not five.** §5.2 named one `.toml` per lane plus one
  for the backstop and one for the controls. Since `[source]` is file-wide
  and the programs are short, the plan puts every case in one file,
  `crates/kali_cli/tests/cases/array/literal_array_mutators.toml`. It is
  sectioned by case name: literal lane, plain-lane additions, backstop and
  controls.
* **A-5. The §5.4 capability loss is accepted.** Added after implementation.
  No human decision was taken, so array-bounds decision A-5 (accept fail-closed)
  stands. The measured losses are in the followups file §5: a `fill` or `sort`
  whose effect is never observed, and a mutator in dead code or an uncalled
  function. Every one is in §5.4's two classes. No loss outside them was found.
* **A-6. Probe reads were rewritten, controls were deleted.** Several plan
  probes logged `a[0]`, which a dropped mutator leaves unchanged in node too, so
  they could not show the mutation. Their trailing reads were rewritten so each
  probe observes its mutation (for example `a.length`, `a.indexOf(4)`, `a[2]`);
  `litmut_push_pop` now logs `a.pop()`. The controls `litmut_ok_slice` and
  `litmut_ok_user_push` were deleted because they were not CORRECT at baseline.
  Four probes refuse at baseline for unrelated pre-existing reasons, so they
  show no SILENT to REFUSES move: `t_var_pop`, `param`, `f_length_compound` and
  `push_pop`.
* **A-7. The optional call `a.push?.()` is gated by the type layer only.** The
  parser drops an optional call's arguments, so `a.push?.(4)` is
  `OptionalChain(a.push)` with no call. The type layer gates it in
  `resolve_optional_chain`, and `kali run` runs the type layer first. Codegen
  cannot tell it from a plain `a.push` read, so codegen has no optional-call
  gate and the backstop cannot see it. A side effect: `a.push?.name` (no call)
  also refuses. §1 item 1 and §3.3 said codegen mirrors the optional spelling;
  it does not. A parenthesized `(a.pop)?.()` is not unwrapped by that gate and
  is silent (followups §8).
* **A-8. The codegen literal gate is checked only at the placeholder
  fallback, and an alias gets the literal text.** §3.3 placed
  `literal_array_mutator` beside the plain check. It is checked only at
  `emit_call`'s terminal placeholder fallback, immediately before the backstop,
  because LIR cannot tell `new C` / `new C()` from `[x]`, and the early
  placement refused user-class methods named like mutators. Consequence for
  §1.1 and §3.4: for `const b = a; b.pop()`, codegen traces `b` back to the
  literal, so `run` refuses with the literal message, not the backstop text.
  `check` still exits 0, so the gap holds. A parameter receiver gets the
  backstop text. "`run` refuses" is the accurate claim for both.
* **A-9. A-3 is wider.** A-3 said `push` mixed with another mutator shows the
  literal-mutator message. The same holds for in-function pushes that
  repr_infer would have rejected on their own: a malformed push
  (`o.push({a:1})`, `o.push(1,2)`) and a mixed-type push (`i64` then string).
  The literal-mutator `E5506` is shown instead of repr_infer's argument-specific
  "has a `.push` call" message, which looks unreachable from the CLI for these
  shapes (followups §10). The plan's in-function unit-test variant `a.push(4)`
  was dropped, because that binding is on the growable lane.
