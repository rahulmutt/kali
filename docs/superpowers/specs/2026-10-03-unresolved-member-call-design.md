# A member call on a value the program built never evaluates silently to `0`

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `6f042548f` (`main`, the literal-array-mutators merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali`, built at the baseline (`cargo build -p kali_cli`, `dev` profile) |
| oracle | `node v26.10.0` |
| measured on | 2026-10-03 |
| item picked | `docs/superpowers/followups/literal-array-mutators-discovered-defects.md` §3, "Any unresolved member call evaluates to `0` at exit 0" |
| defects this closes | that §3, plus §12 of the same file (`.call` / `.apply` spellings of a mutator), which reaches the same fallback (§2.2); every REFUSES row of §2.1 below |

**Scope was chosen by the human partner:**

* **Item:** §3, over §14b (field write in a method), §6 (real growable
  mutators) and §2 (warnings dropped on a successful build).
* **Receivers:** program-owned receivers refuse; host-rooted chains keep the
  warn+0 escape hatch (option 1 of four; the others were "program-owned plus
  §14a", "deny every member call" and "real semantics").
* **`check`:** mirrors the refusal where the type layer knows the receiver's
  member set (option 1 of three; the others were "run-only" and "a
  TS2339-style diagnostic").
* **Mechanism:** approach B of three, default-deny on a program-bound root
  unless host provenance is proven (§4 records A and C).

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** a member call that reaches `emit_call`'s terminal fallback
(`kali_codegen/src/emit/call.rs:3991-4004`, warn + `i64.const 0`) never
evaluates silently to `0` when the receiver chain is rooted at a name the
program binds, unless that binding has proven host provenance (§3.2).
`kali run` refuses it with `E5506` and emits `unreachable`. Concretely:

1. Dot and computed spellings at any depth (`o.zork()`, `o["zork"]()`,
   `o.a.b.zork()`) refuse when the root is an object-literal, array-literal,
   string, number or program-class-instance binding, an alias of one, a
   parameter, a reassigned `let` / `var`, or the bound result of a program
   function.
2. A chain that starts at a literal expression (`({k:1}).zork()`,
   `"abc".zork()`, `[1].zork()`) refuses.
3. `.call` / `.apply` through a program root refuses: `a.push.call(a, 4)`,
   `a.pop.call(a)` (followups §12, rows 1 and 3). The two
   `Array.prototype.m.call(a, …)` rows are rooted at the free global `Array`;
   §3.1 rule 3 covers them (§2.1 `proto_*`).
4. An `Object.prototype` method that kali does not lower and that reaches
   the fallback (`o.hasOwnProperty("k")` printed `0` where node prints
   `true`, §2.1) refuses under `run`.
5. **`kali check` mirrors it** when the receiver is a `const` binding to an
   object literal, or to `new C()` of a program class whose `extends` chain
   stays inside the program, and the method name is missing from the member
   set (§3.3).
6. A program in which no call reaches the fallback with a program-bound,
   non-host root behaves as it does at the baseline.

### 1.1 What this project does NOT claim

* **Free-global roots keep warn+0.** `document.foo()`, `fetch(…)`,
  `globalThis.x.y()` and every unresolved host surface are unchanged. That is
  exactly the population the A2 deny-by-default broke (361 tests, per the
  comment at `call.rs:3964-3971`).
* **Host-provenance roots keep warn+0.** A `const` bound from a free-global
  chain, call or `new` (`const t = globalThis.performance; t.now()` prints
  `0`, node `number` for `typeof`) stays silent. It is filed, not fixed.
* **Imports keep warn+0, including a local program module.**
  `import {m} from "./lib.js"; m.zork()` prints `0` (node throws
  `TypeError`). Telling a program module from an unresolved package is
  linker work. It is disclosed here and filed.
* **§14a is a different route.** `new S().push(1)` never reaches the terminal
  fallback (§2.2), so it is not touched. §14b and §2 are not touched either.
* **No run-time `TypeError`.** Nothing that refuses starts working; this is
  fail-closed only.
* **One disclosed `check` / `run` gap.** `check` exits 0 and `run` refuses for:
  * a string, number or array receiver;
  * an alias, a parameter, a reassigned `let` / `var`, or a program call
    result;
  * an `Object.prototype` name such as `hasOwnProperty`;
  * an object literal with a spread or a computed key.

  The type layer cannot tell which of these codegen lowers, so it cannot
  soundly mirror them. This follows array-bounds followups §4 and
  literal-array-mutators spec §1.1.
* **No new diagnostic code, flag or schema.** The refusal is `E5506`. The CLI
  change packet (`specs/12`, `18`, `19`) is untouched. `specs/15-errors.md`
  widens E5506's scope by one line (§3.5).

---

## 2. What was measured

### 2.1 The silent surface at the baseline

Every program was run as `node P.js`, `kali check P.js` and `kali run P.js`
at `6f042548f`. "throws" means node raises `TypeError: … is not a function`.
Every REFUSES row is wrong at exit 0 with `check` exiting 0, except where
noted.

| row | program | node | kali run | target |
|---|---|---|---|---|
| objlit | `const o={k:1}; console.log(o.zork(4));` | throws | `0` | REFUSES, check=1 |
| strkey | `const o={k:1}; console.log(o["zork"](4));` | throws | `0` | REFUSES, check=1 |
| deep | `const o={a:{b:{}}}; console.log(o.a.b.zork());` | throws | `0` | REFUSES (check gap: the member set is the root's, not `o.a.b`'s) |
| inmain | `function main(){ const o={k:1}; console.log(o.zork()); } main();` | throws | `0` | REFUSES, check=1 |
| inst | `class C{ f(){return 1;} } const c=new C(); console.log(c.g());` | throws | `0` | REFUSES, check=1 |
| str | `const s="abc"; console.log(s.zork());` | throws | `0` | REFUSES (check gap) |
| num | `const n=5; console.log(n.zork());` | throws | `0` | REFUSES (check gap) |
| arr | `const a=[1,2]; console.log(a.zork());` | throws | `0` | REFUSES (check gap) |
| alias | `const o={k:1}; const p=o; console.log(p.zork());` | throws | `0` | REFUSES (check gap) |
| param2 | `function g(x){ return x.zork(); } const o={k:1}; console.log(g(o));` | throws | `0` | REFUSES (check gap) |
| letre | `let o={k:1}; o={k:2}; console.log(o.zork());` | throws | `0` | REFUSES (check gap) |
| fnres | `function mk(){ return {k:1}; } const o=mk(); console.log(o.zork());` | throws | `0` | REFUSES (check gap) |
| hasown | `const o={k:1}; console.log(o.hasOwnProperty("k"));` | `true` | `0` | REFUSES (check gap) |
| call_push | `const a=[1,2,3]; a.push.call(a, 4); console.log(a.length);` | `4` | `3` | REFUSES |
| call_pop | `const a=[1,2,3]; a.pop.call(a); console.log(a.length);` | `2` | `3` | REFUSES |
| proto_push | `const a=[1,2,3]; Array.prototype.push.apply(a, [4]); console.log(a.length);` | `4` | `3` | REFUSES (§3.1 rule 3) |
| proto_pop | `const a=[1,2,3]; Array.prototype.pop.call(a); console.log(a.length);` | `2` | `3` | REFUSES (§3.1 rule 3) |

Rows that stay as they are (controls):

| row | program | node | kali run | target |
|---|---|---|---|---|
| ok_extends | `class A{ f(){return 4;} } class B extends A{} const b=new B(); console.log(b.f());` | `4` | `4` | unchanged |
| ok_usp | `const u=new URLSearchParams("a=1"); u.append("b","2"); console.log(u.toString());` | `a=1&b=2` | same | unchanged (recognizer resolves it first) |
| ok_et | `class X extends EventTarget{} const x=new X(); x.addEventListener("t", ()=>{}); console.log("ok");` | `ok` | `ok` (reaches the fallback with `addEventListener`) | unchanged (§3.2: base leaves the program) |
| ok_freecall | `const r=Math.max(1,2); console.log(r);` | `2` | `2` | unchanged |
| host_alias | `const t=globalThis.performance; console.log(typeof t.now());` | `number` | `0` | unchanged, silent; filed (§1.1) |
| import | `import {m} from "./lib.js"; console.log(m.zork());` (`lib.js`: `export const m={k:1};`) | throws | `0` | unchanged, silent; filed (§1.1) |
| param | `function g(x){ return x.zork(); } console.log(g({k:1}));` | throws | exit 1, pre-existing object-literal-argument E5506 | unchanged |
| ok_added | `const o={k:1}; o.f=function(){return 5;}; console.log(o.f());` | `5` | exit 1, pre-existing "unknown field" E5506 | unchanged |
| ok_method | `const o={f(){return 3;}}; console.log(o.f());` | `3` | exit 1, pre-existing `E3100 undefined identifier 'f'` | unchanged |

### 2.2 Which rows reach the terminal fallback

Each program was run with a trailing `function gg(q){ q.pop(); }`, which
codegen alone refuses, so `kali run` prints every warning. A row reaches the
fallback when it prints `warning[E3100]: undefined call target '<name>'`.

* Reach it: objlit, strkey, str, num, arr, inst, param2, hasown (`'hasOwnProperty'`),
  call_push (`'call'`), ok_et (`'addEventListener'`).
* Do not reach it: `new S().push(1)` (§14a), `n.toFixed(2)`, `u.append(…)`.

So §3 and §12 share one route, and §14a does not.

---

## 3. Design

### 3.1 The `run` gate (codegen)

One new gate in `emit_call`, placed **after** the literal-array mutator
checks (`call.rs:3947-3962`) and **before** `deny_placeholder_lowering`
(`call.rs:3972`). Every recognizer, gate-1 and every existing refusal above
it run first, so they keep their text and their verdicts. The gate fires when
the callee has a receiver child and any of these holds:

1. `receiver_root(receiver)` is `Some(name)` and
   `!root_has_host_provenance(name)`.
2. The chain starts at an object, array, string or number literal.
3. The callee name is `call` or `apply` and the receiver is itself a member
   whose root is the free global `Array`, `Object` or `String` through
   `.prototype` (`Array.prototype.push.call(a, 4)`). These are the
   intrinsic method-borrowing spellings, and they always reach the program
   value named in the arguments.

It refuses through `deny_e5506` with
`kali_common::unresolved_member_call_unavailable_message(name)`:

> calling `.NAME()` is unavailable in the current phase: the receiver is a
> value this program built, and kali has no lowering for a method of that
> name on it; node would run a method or throw a TypeError, and evaluating
> the call to 0 would be silently wrong (fail-closed)

A chain whose root is neither a name nor a literal (a call result, `new X()`)
keeps today's behaviour.

### 3.2 Host provenance

`receiver_root(id)` is the root walk now inside
`receiver_root_is_url_provenance` (`emit/url.rs:417-446`), factored out to
return the root `LirNodeId` (or `None`). `receiver_root_is_url_provenance`
calls it, so there is one walk.

`root_has_host_provenance(name)` is true when any of these holds:

* the name is free: `!name_is_program_bound(name)` (`call.rs`);
* the name is an import binding;
* the name is a `const` binding whose initializer is a call, `new` or member
  chain whose own root has host provenance, recursively, with a visited set;
* the name is a `const` binding to `new C()`, where `C` is a free name or a
  program class whose `extends` chain reaches a class that is not a program
  class (`class X extends EventTarget`).

Everything else answers false: a literal, a program-class instance whose
chain stays in the program, a parameter, a reassigned `let` / `var`, and a
program function's result. Anything the predicate cannot prove is not host.

### 3.3 The `check` mirror (type layer)

One new check in `kali_types/src/resolve/member.rs`, called next to
`reject_array_mutator_member` at both of its call sites (the call callee and
the optional-call member). It does not fire when that function already
pushed a diagnostic for the same member.

`known_member_set(object)` returns `Some(names)` only for:

* a `const` identifier (after `unwrap_transparent`, resolved to its nearest
  binding as `resolve_array_literal_binding_name` does after A-10) bound to
  an object literal with no spread and no computed key: the set is its keys;
* a `const` identifier bound to `new C()`, where `C` is a program class and
  every class on its `extends` chain is a program class: the set is the
  methods, fields and accessors of every class on the chain, plus every
  `this.x =` write in their bodies.

The check refuses with the §3.1 message when the method name is:

* not in the member set;
* not in `kali_common::OBJECT_PROTOTYPE_NAMES` (`constructor`,
  `hasOwnProperty`, `isPrototypeOf`, `propertyIsEnumerable`,
  `toLocaleString`, `toString`, `valueOf`, `__defineGetter__`,
  `__defineSetter__`, `__lookupGetter__`, `__lookupSetter__`);
* not in the program-wide set of assigned property names (`X.name = …` on
  any receiver, any scope).

Every name it refuses also refuses under `run`: the method is not defined, so
no recognizer resolves it, and the receiver is a literal or program-class
binding, which §3.2 does not count as host. Plan Task 1 checks this against
the recognizer list and records any recognizer keyed on the method name alone.

### 3.4 Shared text

`kali_common` gains `unresolved_member_call_unavailable_message(name)` and
`OBJECT_PROTOTYPE_NAMES`, next to `LITERAL_ARRAY_MUTATORS`. Both layers use
the one message, so a case can pin a single substring.

### 3.5 Docs

* `specs/15-errors.md`: one line under E5506 for a member call on a
  program-owned receiver with no lowering.
* `literal-array-mutators-discovered-defects.md`: §3 and §12 marked FIXED,
  with the commit.
* A new `docs/superpowers/followups/unresolved-member-call-discovered-defects.md`
  holding:
  * the measured capability-loss table (§5.4);
  * the triage table of moved tests;
  * the host-alias and local-import gaps (§1.1);
  * anything else the work measures and does not fix.
* `kali-silent-miscompile-register.md` is amended only if a §0.2 lane moves.

---

## 4. Approaches not taken

* **A. Deny known program-owned provenance only.** Refuse only a root
  provably initialized from a literal or `new ProgramClass()`. It has the
  smallest blast radius, but aliases, parameters and call results stay
  silent, and any new spelling defaults to `0`: the denylist failure Fix 5's
  comment (`call.rs:3888-3902`) describes.
* **C. A type-layer table that codegen reads.** `check` and `run` would agree
  row for row, but strings, aliases and parameters would stay silent, which
  falls short of §3's claim.
* **Deny every member call at the terminal.** The A2 attempt. It broke 361
  tests by cutting off free-global host surfaces.
* **Real semantics** (a run-time `TypeError`). This needs a throw path out of
  codegen for an arbitrary call site. The human partner declined it for this
  project.

---

## 5. Testing and measurement

### 5.1 Probes

`tools/array-return-probes/` gains an `unres_*` family. Its baseline,
`baseline-unres.tsv`, is recorded at `6f042548f` before any code changes:

* `unres_*` (about 20): every REFUSES row of §2.1, plus the literal-start
  spellings of §1 item 2 and an in-function variant of each `.call` row.
* `unres_ok_*` (about 12): every control of §2.1, plus:
  * a free-global call (`document.foo()` under the browser surface, or a
    free name kali leaves unresolved);
  * a method that exists on an object literal and on a class, called through
    an alias;
  * a getter-backed member.

Pass conditions:

* every `unres_*` row is REFUSES;
* `check` is non-zero on objlit, strkey, inmain and inst, and on every other
  row inside §3.3's mirror;
* no `unres_ok_*` row moves;
* no row of the older baselines moves between `6f042548f` and HEAD.

### 5.2 Cases

There is one case file:
`crates/kali_cli/tests/cases/soundness/unresolved_member_call.toml`.

* Every REFUSES shape gets a `run` case: `exit = "failure"`,
  `stderr_contains` the §3.1 substring, `stdout = ""`.
* Every shape inside the mirror also gets a `check` case.
* Every control gets a case pinning node's stdout.
* Every rationale quotes node v26.10.0's output.

### 5.3 Unit tests

These go in sibling `*_tests.rs` files, not inline modules:

* `kali_types/src/resolve/member_tests.rs`:
  * member-set construction for an object literal and a class chain;
  * the spread and computed-key opt-outs;
  * the `OBJECT_PROTOTYPE_NAMES` exclusion;
  * the assigned-name exclusion;
  * a non-program base class giving `None`;
  * shadowing, where the nearest binding wins.
* A codegen sibling test file for `receiver_root` and
  `root_has_host_provenance`:
  * free, import, a host-derived `const`, and a recursive alias cycle;
  * a parameter, a reassigned `let`, and an `extends EventTarget` instance.

### 5.4 Blast radius and capability loss

Run `cargo test --workspace` and `cargo test -p kali_cli --test cases` before
any re-pin. Every moved test goes into a triage table: name, before, after,
and class (wanted / capability loss / wrong). A capability loss is accepted
as fail-closed (array-bounds decision A-5) only if it is in one of two
classes:

1. **Effect never observed.** The call's result and effects are never read,
   and node would not have thrown on it.
2. **Dead code.** The call is unreachable at run time, for example inside an
   uncalled function or a false branch.

Any loss outside those classes is brought to the human partner before it is
re-pinned. If the moved-test count approaches A2's order (more than about 50),
work stops and the numbers go to the human partner before anything is
re-pinned.

---

## 6. Amendments

None yet.
