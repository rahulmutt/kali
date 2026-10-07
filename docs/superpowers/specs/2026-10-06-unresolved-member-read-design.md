# A member read or store on a value the program built never evaluates silently to `0`

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `28c97e4ed` (`main`, the captured-bindings merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali` (`dev` profile), built at the baseline |
| oracle | `node v26.10.0`, run as `env -u FORCE_COLOR node` |
| measured on | 2026-10-06 |
| item picked | `docs/superpowers/followups/captured-bindings-discovered-defects.md` §5.10 ("Captured objects read through a call, an argument or a computed key are silent `0`") |
| defects this closes | §5.10 (c8, c9, e1, m6, q8: refused); §4's "Objects passed through a `const` copy of a parameter" (refused); the group-A rows of §2.2 below (refused) |

**Scope was chosen by the human partner:**

* **Item:** §5.10, over §5.5 (`Math.PI` / `Math.E`), §5.11 + §5.12 and R14
  (closure F64 reads).
* **Depth:** fail closed only (option 1 of four; the others were "fail closed,
  then real", "… including calls" and "capture-only refusal").
* **Receivers:** program-built receivers refuse; receivers rooted at a builtin
  / host value keep the warn+0 escape hatch (option 1 of three; the others
  were "refuse everything" and "Object-typed receivers only"). The choice was
  made on the measurement in §2.2.
* **`check`:** mirrors the refusal where cheap. Only the absent-field shape is
  cheap (§3.4); the rest is a recorded `check` / `run` gap.
* **Mechanism:** reuse the unresolved-member-call project's provenance gate
  (`docs/superpowers/specs/2026-10-03-unresolved-member-call-design.md` §3.2)
  rather than a builtin-name allowlist (§4).

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** a member read that reaches the generic member-read fallback
(`crates/kali_codegen/src/emit/operators.rs:796-810`: warning `e8::UNIMPLEMENTED`
"unsupported unary operator", then `i64.const 0`) never evaluates silently to
`0` when the receiver chain is rooted at something the program built. A plain
`=` store that reaches the final binary fallback
(`operators.rs:2713-2722`: warning "unsupported binary operator '='", then
`i64.add`) never drops silently either, whether its target is a member or an
identifier (A-2). `kali run` refuses both with `E5506`.

A receiver whose root has host provenance (a free builtin global such as
`Math`, `Number`, `Object`, `globalThis`, or a binding initialized from one)
keeps today's warn+0. That is the boundary the unresolved-member-call project
drew for member calls, and §2.2 measures why it is needed here.

After this project:

1. All five §5.10 programs refuse under `run` (§2.1).
2. The ~30 builtin-alias cases that pass at the baseline still pass (§2.2,
   group B).
3. ~~`kali check` refuses an absent-field read on a `const` object-literal or
   program-class binding (§3.4).~~ `kali check` refuses an absent-field read
   on a `const` object-literal or program-class binding (§3.4) when no more
   specific refusal speaks first. On a program-class binding the refusal is
   held until the class rewrite reports no error. `length` and numeric names
   are left to codegen's own floors (A-6). Every other refused shape exits 0
   under `check`; that gap is pinned by cases and recorded.
4. *(Added 2026-10-07.)* Comma expressions are **not** refused (A-5), so R-27
   stays silent. `typeof` on a program-built member read **is** refused under
   `run` (A-7).

**Not in scope:** making any refused program print node's value; the
`Object.fromEntries` family (R-60), whose receiver is host-rooted and stays
silent (§5); member reads on genuine host objects.

---

## 2. Measurements

### 2.1 The §5.10 programs

Root cause, from the captured-bindings ledger and a code read at the
baseline: the closure loads the object handle correctly in every case. The `0`
is the placeholder at `operators.rs:805`, reached because no `Repr::Object`
shape is known at the read:

| id | program | node | kali | trigger | receiver root |
|---|---|---|---|---|---|
| c8 | `function mk(){ return {a:1}; } function f(){ let o=mk(); const g=()=>o; return g().a; } console.log(f());` | `1` | `0` | member on a call result (no lane exists) | call to `g` (an arrow) |
| c9 | `function mk(){ return {a:1}; } function show(z){ console.log(z.a); } function f(){ let o=mk(); const g=()=>{ show(o); }; g(); } f();` | `1` | `0` | a bare capture passed as an argument makes no shape flow into `z` | parameter `z` |
| e1 | `function show(z){ return z.n * 10; } function outer(p){ const obj = p; function rd(){ return show(obj); } console.log(rd()); } const x={n:4}; outer(x);` | `40` | `0` | as c9 | parameter `z` |
| m6 | `function f(p){ const o=p; const g=()=>o["a"]; return g(); } const x={a:1}; console.log(f(x));` | `1` | `0` | arrows get no capture flow; CB-13's chain-root check misses the bracket spelling | local `o` from a parameter |
| q8 | `function mk(){ return {a:1, s:"xy", arr:[1,2]}; } function f(){ let o=mk(); const g=()=>o["a"]; return g(); } console.log(f());` | `1` | `0` | as m6 | local `o` from `mk()` |

Task 0 rebuilds at the baseline and re-measures every row before anything
lands.

### 2.2 Blast radius of a blanket refusal

Measured on a throwaway branch at the baseline. Both fallbacks were made to
refuse unconditionally (read: any non-unary-operator text reaching
`operators.rs:805`; store: `op == "="` reaching `operators.rs:2713`). The
`cases` target was run with `--test-threads=8`. The baseline is 6552 passing,
measured by the previous session at the same commit. Patched: **6180 passed,
372 failed trials, in 50 cases.**

| group | cases | what they are | classification |
|---|---|---|---|
| **A** | `oracle/tier2` R-21f, R-25l, R-27, R-60a; `oracle/tier3` R-29 (store); `object/property_key_identity` ×6; `object/computed_member_static_name` ×2; `soundness/r06_object_init::returned_object_member_read_no_worse`; `soundness/bitwise_compound` ×2 | rationales already record kali `0` (or a dropped write, or E4201) where node prints a real value | silently wrong today; refusal is the fix |
| **B** | Math aliases (`log2`/`log10`, `atan2`, `hypot`, `imul`/`clz32`, `sqrt`/`cbrt`, `expm1`/`log1p`), Number predicates, `Object.hasOwn` / `.call`, `String.fromCharCode` / `fromCodePoint`, `parseInt` / `parseFloat`, `Array.from`, `Set`, `globalThis.performance` | `const f = Object.freeze(Math.log2)` stores the placeholder `0`, but every call through `f` is resolved statically, so the `0` is never observed | capability loss; every receiver is rooted at a free builtin global |
| **C** | `browser/promise_all_settled_bundle` (`settled[0].status`, `build --bundle` only); `misc/arena_reclamation_runtime_sandboxed` (`taints[0].v - taints[0].v`, two zeros that cancel); `browser/template_literal_dynamic_import_harness` (an empty-named read inside a sequence-wrapped `import()`, not examined) | wrong at runtime but hidden by the test's shape, or not yet understood | handled in §3.5 |

Every group-B receiver root is a free builtin global. No group-A row and no
§5.10 program is rooted at one, with one exception: R-60a and the two
`property_key_identity` "from-entries" rows, whose receiver is initialized
from `Object.fromEntries(...)`. That exception is the §5 residue.

**Resource note.** The previous attempt at this measurement was killed: the
pod hit either its 32Gi memory limit or its 20Gi `/tmp` limit. That run used
a second full `CARGO_TARGET_DIR` under `/tmp`, at the default parallelism.
This run reused `/workspace/target` with `-j 8` and `--test-threads=8` under
a watchdog: peak cgroup memory was 6.5G, `/tmp` stayed at 2M, and `target/`
grew from 4.1G to 5.6G. The plan carries these caps (§6.3).

---

## 3. Design

### 3.1 The gate

A new method in `crates/kali_codegen/src/emit/member_provenance.rs`:

```rust
pub(crate) fn unresolved_member_read_refuses(&self, receiver: LirNodeId) -> bool
```

It reuses `receiver_chain_root` (`member_provenance.rs:27`) and
`root_has_host_provenance` / `init_has_host_provenance`, and it decides on
the root node:

| root | verdict |
|---|---|
| a literal | refuse |
| an object or array literal (a text-less `Value` with two or more children) | refuse |
| a named identifier | refuse unless `root_has_host_provenance` (a free global is host; a parameter is not; a declarator is followed through its initializer, a call to its callee; reassigned non-`const` bindings are not proven; no reachable scope chain means not proven) |
| `this` / `{}` / `[]` (a text-less childless `Value`) | refuse unless emitting a method of a host-derived class |
| **a call** (`g().a`, `mk().a`) | refuse unless `init_has_host_provenance(call)`, i.e. the callee resolves to a host value |
| anything else | keep warn+0 |

The call row is the one difference from `unresolved_member_call_refuses`
(`member_provenance.rs:51`), which keeps warn+0 on a call root. The two gates
share the root classification through one private helper, so they cannot
drift. `unresolved_member_call_refuses` keeps its own call-root verdict and
its `.call` / `.apply` prototype-borrow arm.

Shadowing comes from `is_free_global`: `let Math = {a:1}; Math.b` has a
program-bound root and refuses.

### 3.2 The read site

In `operators.rs`, after the existing `capture_member_fallback_refusal` check
(`:794`) and before the warning (`:796`): when `op` is not unary-operator
text (`is_unary_operator_text`) and `unresolved_member_read_refuses(arg)`
holds, return `self.deny_e5506(function, &unresolved_member_read_unavailable_message(op))`.
A genuine unknown unary operator is unchanged.

### 3.3 The store site

In the `_` arm of `emit_binary`'s operator match (`operators.rs:2713`): when
`op == "="` and the target's chain root refuses by the §3.1 table (a member
target's root is its receiver chain's root; an identifier target is its own
root, A-2), deny with `unresolved_store_unavailable_message(target)`. The site is the
final fallback, not `emit_assignment`'s `return false` (`literal.rs:848`),
because lanes after `emit_assignment` may still lower a store that
`emit_assignment` declines.

### 3.4 The `check` mirror

`kali check` does not run codegen, so in general it cannot know that a read
reaches the fallback. The one cheap shape reuses the unresolved-member-call
project's `known_member_set` (`crates/kali_types/src/resolve/member.rs:515`).
A static member read (dot, or a string-literal bracket) whose object is an
identifier with a known member set refuses with the §3.6 message when the
name is:

* not in the member set;
* not in `kali_common::OBJECT_PROTOTYPE_NAMES`;
* not in the program-wide set of assigned property names.

It skips a member that is the operand of `typeof` (A-3), and a member that
is a call's callee (the call gate owns those; one defect, one diagnostic).

That is exactly the absent-field-read shape (R-21f). It runs from the member
expression resolution site that already hosts `reject_array_mutator_member`,
and it returns early when a diagnostic was already pushed for the same
member. Every name it refuses must also refuse under `run`. Task 0 checks this
with a probe per mirror row under both commands. A row where `run` does not
refuse removes that row from the mirror; it does not widen `run`.

The gap (call roots, parameters, computed keys, locals from user calls) is
pinned by `check` cases that exit 0, so closing it later is a visible diff.

### 3.5 Shared text

`kali_common/src/messages.rs` gains, next to
`unresolved_member_call_unavailable_message`:

* `unresolved_member_read_unavailable_message(name)`: "reading `.{name}` is
  unavailable in the current phase: the receiver is a value this program
  built, and kali has no lowering for that read; node would read a property
  or `undefined`, so kali refuses rather than read 0"
* for op text `""` or `"spread"` (A-1), the same helper returns "this
  expression is unavailable in the current phase: the receiver is a value
  this program built, and kali has no lowering for that read; kali refuses
  rather than evaluate it to 0";
* `unresolved_store_unavailable_message(target)`: "assigning to `{target}` is
  unavailable in the current phase: the receiver is a value this program
  built, and kali has no lowering for that store; node would store the value
  or throw a TypeError, so kali refuses rather than drop the store", where
  `target` is `.name` for a member and `name` for an identifier (A-2).

A string-literal key prints its text. Every text contains "the receiver is a
value this program built", which the call gate's messages also contain, so
existing count-based tests see a duplicate diagnostic if one ever appears.
The distinct substrings are "no lowering for that read" and "no lowering for
that store". Both layers use the one text.

### 3.6 Diagnostics and docs

* `E5506` (`e5::FEATURE_UNAVAILABLE`). No new code.
* `specs/15-errors.md`: one line under E5506 for a member read or store on a
  program-built receiver with no lowering.
* `specs/19-feature-maturity.md`: no row claims more; this only refuses. No
  CLI, flag or schema change, so `specs/12-cli.md`, `specs/18-schemas.md` and
  `README.md` are untouched.

---

### 3.7 Amendments from the plan-writing probes (2026-10-06)

Probed at the baseline, and again under the §2.2 throwaway patch, while the
plan was written:

* **A-1. The read gate is shape-blind.** LIR collapses HIR's `MemberExpr`,
  `Spread` (text `"spread"`) and `SequenceExpr` (text `""`) into one `Value`
  kind (`crates/kali_hir/src/lowering/expression.rs:54,152,192`), so all three
  reach the read fallback the same way. R-25's `console.log([...a])` (op
  `spread`) and R-27's `let a = (1, 2)` (op `""`) are refused by the same gate,
  and both are SILENT today, so the refusal is correct. *(Amended 2026-10-07
  by A-5: the gate no longer refuses op `""`, so R-27 is not refused.)* Their message must
  not claim a property read: for op text `""` or `"spread"`, the read helper
  returns a neutral "this expression …" text (§3.5). A property literally
  named `spread` gets the neutral text as well, which is still a true E5506.
* **A-2. Identifier stores.** Among the probes, the store fallback is reached
  only by an identifier target: R-29's `const x = 1; x = 2`. Member stores are
  refused earlier ("unknown field … on fixed-shape object") or lowered by a
  store lane. The store gate therefore classifies the **target's** chain root,
  and an identifier target is its own root. A `const` declarator is not host,
  so R-29 refuses. A free global (`zz = 3` in sloppy code) is host and keeps
  warn+0.
* **A-3. `typeof` is outside this gate.** `const o={a:1}; typeof o.z` prints
  `0` at the baseline (node `undefined`) through the `typeof` arm's own
  placeholder ("unsupported unary operator 'typeof'"), not through
  `operators.rs:805`. The `check` mirror therefore skips a member that is the
  operand of `typeof`; otherwise `check` would refuse what `run` accepts.
  ~~`typeof` on a program-built value stays silent and is residue (§5).~~
  *(Corrected 2026-10-07 by A-7: under `run` it is refused.)*
* **A-4. A measured capability loss.** `const o={a:1}; console.log(o.z ?? 5)`
  prints node's `5` at the baseline only because kali stores `undefined` and
  `0` alike, so the placeholder reads as nullish. The read reaches the
  fallback, and its root is a `const` object literal, so it refuses after this
  project, under both `run` and `check`. The same shape on a parameter
  already refuses at the baseline ("unknown field 'z' on fixed-shape object").
  This is recorded as a loss in the followups file.

| probe | node | baseline `run` | baseline `check` | reaches |
|---|---|---|---|---|
| `const o={a:1}; console.log(typeof o.z);` | `undefined` | `0` | 0 | the `typeof` placeholder |
| `const o={a:1}; console.log(o.z ?? 5);` | `5` | `5` | 0 | the read fallback |
| `const o={a:1}; console.log(delete o.z);` | `true` | E5506 (delete) | 0 | refused earlier |
| `const o={a:1}; console.log(o.z === undefined);` | `true` | E5506 (`===`) | 0 | refused earlier |
| `class C{ f(){return 1;} } const c=new C(); console.log("v="+c.z);` | `v=undefined` | E5506 (field not declared) | 1 | refused earlier |
| `let Math = {a:1}; console.log(Math.b);` | `undefined` | E5506 (unknown field) | 1 | refused earlier |
| `const x = 1; x = 2; console.log("r=" + x);` | TypeError | `r=1` | 0 | the store fallback |
| `function mk(){ return {a:1}; } const o=mk(); o["a"] = 5; console.log(o.a);` | `5` | `5` | 0 | a store lane |
| `let a = (console.log("x"), 7); console.log("a=" + a);` | `x`, `a=7` | silent | 0 | the read fallback, host root (kept) |
| `globalThis.zz = 3; console.log("ok");` | `ok` | `ok` | 0 | the read fallback, host root (kept) |

### 3.8 Amendments from execution (2026-10-07)

Recorded after Tasks 3–7 ran. Each one corrects or adds to the text above.
The lines above that became false are struck through in place.

* **A-5. Comma expressions are excluded from the read gate (human partner's
  ruling, 2026-10-07).** A-1 refused op `""` along with `spread`. The full
  `cases` run then showed that refusal breaking correct programs that use the
  `(0, x)` indirection idiom, where the sequence's value is stored and the
  placeholder is never observed:
  * `const w=(0,o); Object.hasOwn(w,"a")` prints `true` in node;
  * ``await import((0, `./${name}`))`` in a dynamic-import harness;
  * 92 trials across `object/has_own_js_input`,
    `misc/object_has_own_frozen_js_input`,
    `browser/object_has_own_from_entries` and
    `browser/template_literal_dynamic_import_harness`.

  The read gate now also requires the op text to be non-empty
  (`crates/kali_codegen/src/emit/operators.rs`). Spread still refuses with
  the neutral text. The `""` arm of
  `unresolved_member_read_unavailable_message` is kept so that the function
  stays total.

  Consequence: R-27 stays SILENT and is residue (§5). So is every comma
  expression whose value is read, including
  ``const s=(0, `./${n}`); console.log(s)``, which prints `2` where node
  prints `./x`.
* **A-6. The `check` mirror defers to more specific refusals (human
  partner's second ruling, 2026-10-07).** §3.4 put the mirror first. Under
  `run` too it pre-empted the class rewrite's ``is not declared on class
  `C` `` message and the array-return backstop's `no lane proves this
  receiver is an array`, because `run` runs the resolver first and stops on
  its errors. The mirror now follows three precedence rules:
  * **It runs last.** It runs after the other member checks in
    `resolve_member_expression`, and only if none of them pushed a
    diagnostic for that member.
  * **Class-instance refusals are held.** A refusal on a class-instance
    receiver is held in `ResolutionResult::deferred_read_mirror_diagnostics`.
    The driver (`crates/kali_cli/src/build/compile.rs`) adds the held
    refusals only if `rewrite_class_instances` reports no error. So:
    * an in-slice class gets the class message;
    * an out-of-slice class is refused at `new`;
    * a stateless `extends` chain that is neither rewritten nor refused gets
      the held read refusal.

    **A class-rewrite error drops every held read refusal, including those
    on other receivers.** The program is still refused (fail-closed), but it
    shows fewer diagnostics than it has defects.
  * **It skips names that have their own codegen floor.** These are listed
    by `kali_common::member_read_has_own_refusing_floor`: `length` and
    numeric indexes. The generic read gate never sees those names.
    Consequence: `check` again exits 0 on `o[0]` and `o.length` for a `const`
    object literal. `run` refuses them through the array backstop and the
    `.length` floor. `version` and `pid` have dedicated arms that do **not**
    refuse, so they stay with the mirror.
* **A-7. `typeof` correction (controller ruling).** A-3's premise was wrong.
  Under `run`, `const o={a:1}; typeof o.z` evaluates the operand `o.z`, and
  that read reaches the read gate. The program is refused with E5506, after
  the `typeof` arm's `E8001` warning. `check` still skips the `typeof`
  operand, so this shape is one more `check` / `run` gap. It is not silent.
* **A-8. R-29 measures `both_reject`.** §6.1 predicted `fail_closed`. The
  classifier spells FAIL_CLOSED only when node exits 0, and node throws
  `TypeError: Assignment to constant variable.` Both engines exit 1, so the
  live verdict is BOTH_REJECT. The register retires R-29 at BOTH_REJECT.

---

## 4. Approaches considered

* **A (chosen): the provenance gate.** It reuses a walk that already handles
  shadowing, reassignment, aliases and host-derived classes, so a fix to one
  gate fixes both.
* **B: a builtin-name allowlist** on the receiver root. Rejected: it ignores
  shadowing (`let Math = {…}`) and duplicates the provenance walk.
* **C: refuse only Object/TaggedVal-repr receivers.** Rejected: it depends on
  the shape inference that is exactly what is missing in c9 and e1 (`show`'s
  `z` has no shape), so it would miss the §5.10 targets.

---

## 5. Residue (stays silent, recorded)

* **R-60 and the `Object.fromEntries` family.** `const o =
  Object.fromEntries(...)` follows its initializer to the free global
  `Object`, which is host, so `o.a` keeps warn+0. R-60's register row and
  the `property_key_identity` "from-entries" pins do not move.
* **Reads off genuine host objects** (`globalThis.performance.foo`, a value
  returned by a host call), as for member calls.
* **The `check` / `run` gap** of §3.4.
* ~~**`typeof` on a program-built value** (A-3): `typeof o.z` prints `0`.~~
  Refused under `run` since execution (A-7). `check` still exits 0 on it.
* ~~**A comma expression whose first operand is host-rooted** (A-1): `(console.log("x"), 7)` still evaluates to `0`.~~
  **Every comma expression** (A-5): R-27 and the `(0, x)` family stay
  silent wherever the sequence's value is read.
* **A member read rooted at an index expression** (found in execution):
  `t[0].v`, where `t[0]` holds a program-built object, still reads `0`
  (`misc/arena_reclamation_runtime_sandboxed`).
* **The capability loss of A-4** (`o.z ?? d` on a `const` object literal).

These go into a new
`docs/superpowers/followups/unresolved-member-read-discovered-defects.md`,
together with anything Task 0 or the case runs find.

---

## 6. Verification

### 6.1 Re-pins

Each is re-measured against node in Task 0. The rationale keeps the old value
struck through and states what moved.

| case(s) | baseline | after |
|---|---|---|
| `oracle/tier2` R-21f, R-25l, ~~R-27~~ (both scopes) | `verdict = "silent"` | `fail_closed` (R-27 did not move, A-5) |
| `oracle/tier3` R-29 (both scopes) | `accepts_invalid` | ~~`fail_closed` (A-2)~~ `both_reject` (A-2, A-8) |
| `object/property_key_identity` escaped-quote and member-probe rows (×4, not the from-entries rows) | pins `0` | E5506 |
| `object/computed_member_static_name` ~~×2~~ ×1 (the absent-property row; ~~the argv-index row~~ did not move, because it already refuses through the array-return backstop: `process.argv` is a host root), `soundness/r06_object_init::returned_object_member_read_no_worse`, `soundness/bitwise_compound` ×2 | pin a silent `0`, a dropped write, or E4201 | E5506 |
| `misc/arena_reclamation_runtime_sandboxed::function_scratch_is_reclaimed` | `x.v - x.v` cancels to the right total | the program stops reading the field; its subject is reclamation |
| `browser/promise_all_settled_bundle`, `browser/template_literal_dynamic_import_harness` | succeed | if the gate refuses them, execution stops and asks the human partner before re-pinning |

**Stop rule:** any case that moves and is not in this table stops execution
and goes to the human partner. It is not re-pinned silently.

### 6.2 Register and ranking

The oracle verdict flips trip
`crates/kali_blast_radius/src/oracle_tests.rs:132`
(`every_zero_two_row_is_the_class_set_its_live_cases_assert`). The register's
§0.2 rows for R-21, R-25, ~~R-27~~ and R-29 are re-derived for the lanes that
moved, on the R-12 / R-13 precedent: an entry is retired only when every one
of its lanes moved. If a SILENT lane leaves the filter, `tools/blast-radius/clusters.json`
and `docs/superpowers/followups/blast-radius-ranking.md` get their next §6
amendment. R-60 does not move.

`captured-bindings-discovered-defects.md` §5.10 and the §4 "Objects passed
through a `const` copy of a parameter" item are marked closed (refused), with
this spec cited.

### 6.3 Tests

* **Black-box cases:** a new
  `crates/kali_cli/tests/cases/soundness/unresolved_member_read.toml`. Each row
  is pinned against node, under `run` and under `check`:
  * **Refuse under `run`:** c8, c9, e1, m6 and q8 verbatim; `mk().a`;
    `id(o).a`; an identifier store to a `const` (R-29's shape, A-2); an
    absent field on a `const` object literal (dot and bracket); the same under
    `??` (A-4); a nested `p.a.b.c` rooted at a parameter; R-25's spread ~~and
    R-27's comma~~ (A-1, neutral message; the comma row now pins R-27's
    silent `b=0` under `run`, A-5); a shadowed
    `let Math = {a:1}; console.log(Math.b)` (any E5506; it already refuses
    at the baseline through the fixed-shape gate).
  * **Under `check`:** the absent-field rows (including `??`) refuse; every
    other row exits 0 (the pinned gap).
  * **Keep warn+0 (exemption controls):** a frozen `Math.log2` alias,
    `globalThis.performance`, a `Number["isNaN"]` alias, and an
    `Object.fromEntries` read (R-60, with a rationale stating it is still
    wrong).
  * **No regression:** a present field on a `const` object literal, a
    program-class field read, a bracket store `o["a"] = 5` on a local from
    `mk()`, a free-global store `globalThis.zz = 3`, and the captured-bindings
    d3 / d6 lanes, each printing node's value.
  * **`check` skips `typeof`:** `typeof o.z` on a `const` object literal
    exits 0 under `check` (A-3).
* **Unit tests:** a sibling `crates/kali_codegen/src/emit/member_provenance_tests.rs`
  (no inline `#[cfg(test)]` module), with one test per root kind: a literal,
  a parameter, a call to a user function, a call to a host function, a free
  global, a shadowed global, and `this`.
* **Probes:** the §2.1 programs go into
  `tools/array-return-probes/probes/` with the `umr_` prefix.

**Resource caps for every build and full run:** one shared
`/workspace/target` (no extra `CARGO_TARGET_DIR`, no worktree target dirs
under `/tmp`); `cargo build -j 8`; `cargo test … -- --test-threads=8`; a
watchdog that logs `/sys/fs/cgroup/memory.current` and `du` of `/tmp` and
`target/`, and stops the run past 24G memory or 14G `/tmp`.
`cargo test --workspace` runs once at the end under the same caps.

### 6.4 Order

* **Task 0:** rebuild at the baseline; re-measure §2.1 and every §6.1 row
  against node; run the §3.4 mirror probes under both commands.
* **Phase 1:** messages, the gate and its unit tests, the read site, then the
  store site.
* **Phase 2:** the `check` mirror.
* **Phase 3:** re-pins, register, ranking (if it moves), followups, docs.
* **Final:** a whole-branch review.

### 6.5 Success criteria

* All five §2.1 programs refuse under `run` with the §3.5 read message.
* Every group-B case still passes.
* Every moved case is in §6.1, or was brought to the human partner.
* The `cases` target and `cargo test --workspace` are green.
* `specs/15-errors.md`, the register, the followups and (if moved) the
  ranking match the behaviour.
