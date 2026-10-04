# A program-class instance holds its fields, or kali refuses

## 0. Provenance

| what | value |
|---|---|
| baseline commit | `7c4daa9f7` (`main`, the unresolved-member-call merge) |
| kali binary | `kali 0.1.0`, `target/debug/kali`, built at the baseline (`cargo build -p kali_cli`, `dev` profile) |
| oracle | `node v26.10.0` |
| measured on | 2026-10-04 |
| item picked | `docs/superpowers/followups/literal-array-mutators-discovered-defects.md` §14, second bullet ("A field write in a method is lost", called §14b below) |
| defects this closes | §14b; §14 first bullet ("A method call on a nameless constructed value evaluates to `0`", §14a); register R-36 ("class instance fields round-trip to `0`"); the `new C(arg)` invalid-wasm row of §2.1 |

**Scope was chosen by the human partner:**

* **Item:** §14b, over §6 (real growable mutators), the
  unresolved-member-call §1 `check` / `run` gap, and §14a / §14c / §2.
* **Depth:** real instances (option 2 of three; the others were "fail
  closed now" and "fail closed, then real as a second project").
* **Slice:** the core (§1 items 1 to 6) plus instances crossing calls and
  nameless instance calls. Program-class `extends` and instances in
  containers fail closed (the human partner took the recommended set).
* **`check`:** mirrors the refusals where cheap (option 1 of two; the other
  was "run-only"). The placement in §3.1 makes the mirror exact for every
  refusal the rewrite raises.
* **Mechanism:** approach A of three, an AST rewrite of classes to object
  literals and functions before `repr_infer` (§4 records B and C).

**Citation convention.** Every line reference is as of the baseline commit.

---

## 1. What this project is

**The claim:** a program-class instance is an object that holds its fields.
At the baseline it is the constant `0`: the constructor never runs, a method
gets no `this`, a field store is dropped and a field read yields `0`, all at
exit 0 (§2.2). After this project, for a class in the slice (§3.2), `kali
run` gives node's output; for every other construction or use the rewrite
cannot prove, `kali run` and `kali check` refuse with `E5506`. Concretely:

1. `new C(args)` runs `C`'s field initializers and then its constructor body,
   in JS order, and yields an object.
2. `this.f = v` and `this.f` in the constructor, in methods, and in arrow
   functions nested in either, store and read the instance's field.
3. `s.f` and `s.f = v` outside the class read and write the field.
4. `s.m(args)` runs `C`'s method `m` with `this` bound to `s`.
5. An instance passes to a program function's parameter and returns from a
   program function, and its fields and methods work there.
6. `new C(args).m()` and `new C(args).f` (the nameless forms, §14a) work.
7. `this.f op= e`, `this.f++` and `this.f--` work in statement position for a
   receiver that is `this` or a plain identifier holding an instance (§3.5).
8. Every construction or use outside the slice refuses with `E5506`, under
   `run` and `check` alike (§3.4).
9. A program that constructs no program class behaves as it does at the
   baseline, byte for byte through `repr_infer` (§3.1).

### 1.1 What this project does NOT claim

* **No inheritance between program classes.** `class B extends A` where `A`
  is a program class refuses at `new B(...)`. Host-derived classes
  (`extends EventTarget`) are not touched at all and keep the baseline's
  behaviour, including ruling R8 and the warn+0 escape hatch.
* **No accessors, statics, `#private` members or computed member names.** A
  class using any of them refuses at `new`. A class never constructed is not
  touched, so a static-only utility class keeps the baseline's bare-name
  dispatch.
* **No instance in a container or a host call.** An instance as an array
  element, an object-literal value, a field's right-hand side, an argument to
  `console.log`, `JSON.stringify` or any host function, a concat or template
  operand, or the operand of `instanceof`, `typeof`, `===`, spread or
  destructuring refuses (§3.3's allowlist). node renders an instance as
  `C { n: 1 }`, and an object literal renders as `{ n: 1 }`, so rendering is
  refused rather than wrong.
* **No per-class specialization of user functions.** A parameter fed
  instances of two classes refuses.
* **Booleans in fields stay wrong (R-30, inherited).** An instance field is
  an object-literal field, and `p.ok = true; console.log(o.ok)` prints `1`
  for an object literal at the baseline (§2.3). The same holds for instances.
  Filed, pinned as a known-wrong control, not fixed.
* **No widening of the generic member fallbacks.** The member-store drop
  (`kali_codegen/src/emit/literal.rs:828-831`) and the member-read fallback
  (`emit/operators.rs:767-797`, the R-60 site) are not changed. After the
  rewrite no instance reaches them; widening them is R-60's project.
* **`new` of a plain `function` is refused, not supported.**

---

## 2. What was measured

### 2.1 The silent surface at the baseline

Programs and outputs at `7c4daa9f7`. `check` is `kali check`'s exit code.

| probe | program | node | kali `run` | exit | `check` |
|---|---|---|---|---|---|
| §14b | `class Stack{ constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } function main(){ const s=new Stack(); s.add(3); console.log(s.n); } main();` | `3` | `0` | 0 | 0 |
| top-level | same, at top level | `3` | `0` | 0 | 0 |
| ctor read | `class C{ constructor(){ this.n=5; } } const s=new C(); console.log(s.n);` | `5` | `0` | 0 | 0 |
| method read | `class C{ constructor(){ this.n=5; } get(){ return this.n; } } const s=new C(); console.log(s.get());` | `5` | `0` | 0 | 0 |
| method write | `class C{ constructor(){ this.n=0; } set(v){ this.n=v; } } const s=new C(); s.set(9); console.log(s.n);` | `9` | `0` | 0 | 0 |
| outside write | `class C{ constructor(){ this.n=0; } } const s=new C(); s.n=4; console.log(s.n);` | `4` | `0` | 0 | 0 |
| field decl | `class C{ n=1; set(v){ this.n=v; } } const s=new C(); s.set(9); console.log(s.n);` | `9` | `0` | 0 | 0 |
| ctor chain | `class C{ constructor(){ this.n=0; this.m=this.n+2; } } const s=new C(); console.log(s.m);` | `2` | `0` | 0 | 0 |
| ctor arg | `class C{ constructor(v){ this.n=v; } get(){ return this.n+1; } } const s=new C(4); console.log(s.get(), s.n);` | `5 4` | `E4201` (invalid wasm) | 1 | 0 |
| update | `class C{ constructor(){ this.n=0; } inc(){ this.n++; } get(){ return this.n; } } const s=new C(); s.inc(); s.inc(); console.log(s.get(), s.n);` | `2 2` | `E5506` (update expression) | 1 | 1 |
| §14a | `class S { push(v){ return v+1; } } console.log(new S().push(1));` | `2` | `0` | 0 | 0 |
| R-36 | `class C{ constructor(){ this.v=3; } } console.log(new C().v);` | `3` | `0` | 0 | 0 |

### 2.2 Where the value is lost

* **Parser.** `new` takes the whole following call chain as its callee
  (`kali_parser/src/expression/primary.rs:241-262`), so `new S().push(1)` is
  `new (S().push(1))`. The class body keeps methods only; a field's name goes
  to `field_names` and its initializer tokens are skipped
  (`kali_parser/src/declaration.rs:341-445`).
* **HIR → MIR → LIR.** A class is `ClassDecl("C")` over a block of its
  methods (`kali_hir/src/lowering/statement.rs:408-438`). MIR maps it to
  `Function`, and `NewExpr` and `ThisExpr` both to `Expr`
  (`kali_mir/src/lower.rs:81-131`). LIR spells `new C(4)` as
  `Value(None, [Call(None, [Value("C"), 4])])` and `this` as `Value(None, [])`,
  the same node as `{}` and `[]` (`kali_lir/src/lower.rs:56-67`).
* **Codegen.** The class node is a zero-parameter function `C` whose body is
  the class body (`kali_codegen/src/lower.rs:2840-2925`). Methods are
  top-level functions keyed by bare name, last-wins (`lower.rs:917`), with no
  `this` parameter. Nothing calls `constructor`. `const s = new C()` emits
  the inner call, drops its result and pushes `i64.const 0`
  (`emit/literal.rs:13-45`); `new C(4)` passes one argument to the
  zero-parameter function, which is invalid wasm (`emit/call.rs:3713-3781`).
  A member call dispatches on the bare method name and never passes the
  receiver (`call.rs:978-979`). `this.n = v` and `s.n = 4` have no object
  shape, so the store is dropped (`literal.rs:828-831`, then the `_` arm at
  `operators.rs:2701-2710`), and `s.n` reads `0` at the R-60 site
  (`operators.rs:767-797`).

### 2.3 The object-literal lane the rewrite rides

Hand-lowered forms of §3.2's output, at the baseline:

| program | node | kali |
|---|---|---|
| `function C__new(){ return {n:0}; } function C__get(self){ return self.n; } function main(){ const a=C__new(); const b=C__new(); a.n=2; console.log(C__get(a), C__get(b)); } main();` | `2 0` | `2 0` |
| `function C__new(){ return {n:0, f:1.5}; } function C__add(self,x){ self.n=self.n+x; self.f=self.f*2; return self.n; } const s=C__new(); console.log(C__add(s,2), s.f);` | `2 3` | `2 3` |
| `function P__new(x,y){ return {x:x, y:y}; } function P__len2(self){ return self.x*self.x+self.y*self.y; } function sum(a,b){ return P__len2(a)+P__len2(b); } console.log(sum(P__new(1,2), P__new(3,4)));` | `30` | `30` |
| `function C__new(){ const self={n:0}; for (let i=0;i<3;i++){ self.n=self.n+i; } return self; } const s=C__new(); console.log(s.n);` | `3` | `3` |
| `function C__new(){ return {xs:[1,2]}; } const s=C__new(); s.xs.push(3); console.log(s.xs.length);` | `3` | `3` |
| `function C__new(v){ const self={n:0}; self.n=v; return self; } function C__add(self,x){ self.n+=x; } …` | `7` | `E5506` (compound assignment), exit 1 |
| `… function C__inc(self){ self.n++; } …` | `5` | `E5506` (update expression), exit 1 |
| `function C__new(nm){ const self={name:nm, k:1}; return self; } …` | `hi bob` | `E5506` (runtime string as a property value), exit 1 |
| `function f(p){ p.ok=true; } const o={ok:false}; f(o); console.log(o.ok);` | `true` | `1`, exit 0 (R-30) |
| `const o={p:{n:1}}; o.p.n=4; console.log(o.p.n);` | `4` | `E5506` (no object shape), exit 1 |

The lane matches node through bindings, parameters, returns and loops, and
fails closed on compound assignment, runtime strings and nested objects. The
one silent row is R-30, pre-existing (§1.1). Compound assignment on an
instance is made to work by §3.5; on an object literal it stays refused.

---

## 3. Design

### 3.1 Placement

`kali check` and `kali run` both go through `analyze_source_file`
(`kali_cli/src/build/compile.rs:46-60`, `:668`). The resolver type-checks and
runs `repr_infer` in one call (`kali_types/src/resolve/mod.rs:351-352`). The
rewrite goes after the resolver:

```
parse → link → monomorphize → name_anon → resolver        (unchanged: type check, diagnostics, member-set mirror)
      → class_instances::rewrite(&mut statements)          (NEW: refusals become E5506 diagnostics)
      → if anything was rewritten:
            monomorphize again
            repr_table = infer_reprs(&statements)
            shape_conflicts check
      → HIR → MIR → LIR → codegen                          (unchanged but for §3.6's backstop)
```

* The type checker sees the program as written, so `s: C` annotations, TS
  diagnostics and the unresolved-member-call `check` mirror (that spec §3.3)
  behave as at the baseline.
* The rewrite's refusals are diagnostics of `analyze_source_file`, so `check`
  and `run` raise the same ones.
* A program with nothing to rewrite passes through byte-identical, and
  `infer_reprs` is not re-run. This is the no-op discipline of the
  monomorphize pass (`compile.rs:740-746`).
* Monomorphize runs again because the rewrite creates new functions (each
  `C__m`) whose parameters may now be reached by object literals of several
  shapes, for example a method called with `{a:1}` at one site and `{b:2}` at
  another. That is the pass's ordinary job. An instance parameter never needs
  it: §3.3 resolves a parameter to exactly one class or refuses.
* The module is `kali_types/src/class_instances.rs`, with
  `class_instances_tests.rs` beside it. It reuses
  `kali_types/src/program_classes.rs` for class facts and host derivation.

### 3.2 The translation

**Rewritten classes.** A class declaration, at top level or in a function
body, that is not host-derived and has no `extends`, no getter or setter, no
`static` member, no `#private` member and no computed member name, and that
is constructed somewhere. Each such class `C` is replaced by a factory
`C__new` and one function `C__m` per method.

**Constructor → factory.** JS runs field initializers, then the constructor
body. The factory does the same:

1. Each declared field initializer `f = e`, in order, becomes `let __f_f = e`.
2. The constructor's **leading run** follows: the maximal prefix of
   statements of the form `this.f = e` where `e` uses `this` only as
   `this.g` for a field `g` already initialized by step 1 or earlier in the
   run. Each becomes `let __f_f = e'` (or `__f_f = e'` when `f` was already
   bound), where `e'` replaces each `this.g` with `__f_g`.
3. `const self = { f1: __f_f1, …, fk: __f_fk }`, with fields in first-binding
   order.
4. The rest of the constructor body, with `this` replaced by `self`.
5. `return self;`

```js
class C { n = 0; constructor(v){ this.k = v; this.m = this.n + v; log(this); this.n = 1; } }
// becomes
function C__new(v){ let __f_n = 0; let __f_k = v; let __f_m = __f_n + v;
                    const self = { n: __f_n, k: __f_k, m: __f_m };
                    log(self); self.n = 1; return self; }
```

(`log(self)` here passes an instance to a call; it is legal only when `log`
is a program function, by §3.3's allowlist.)

**The field set** is the fields bound by steps 1 and 2. A class without a
constructor gets steps 1, 3 and 5. A class without fields gets `{}`.

**Methods.** `m(a){ body }` becomes `function C__m(self, a){ body' }`, where
`body'` replaces `this` with `self`, **including inside arrow functions**
(lexical `this`) and **excluding nested `function` expressions and
declarations** (their own `this`). Async and generator methods become async
and generator functions.

**Uses.**

* `new C(args)` becomes `C__new(args)`.
* `recv.m(args)` and `recv["m"](args)` become `C__m(recv, args)` when §3.3
  resolves `recv` to `C`.
* Field reads and writes are not rewritten. `s.n`, `s.n = 4` and
  `self.n = v` are ordinary object-literal accesses, which `repr_infer`
  shapes through bindings, parameters and returns (§2.3).

**Names.** `C__new`, `C__m` and `__f_f` are checked against every program
name. A collision is an internal error, the discipline of `__link` names. A
class declared in a function body gets its functions at the same scope.

### 3.3 Instance provenance

The rewrite computes, as a fixpoint over the whole program's AST, which
expressions are **instances of `C`**:

1. `new C(...)`, including in callee position (`new C().m()`).
2. `this` in `C`'s constructor or methods, or in an arrow function nested in
   them.
3. A `const`, `let` or `var` binding whose initializer and every assignment
   are instances of `C`.
4. A parameter of a program function where every call site passes an
   instance of `C` at that position, and the function is never used as a
   value (passed, stored, assigned, exported). A TS parameter annotated
   `: C` must agree with the call sites; disagreement is a refusal.
5. A call to a program function, or to a method of a rewritten class, whose
   every `return` expression is an instance of `C`.

A binding, parameter or return reached by instances of two classes, or by an
instance and a non-instance, is not resolved, and any instance flowing into it
is refused.

**Where an instance may appear (allowlist).** An expression that is an
instance of `C` may appear only as:

* a binding's initializer, or the right-hand side of an assignment to a
  binding;
* an argument to a program function;
* a `return` expression;
* the receiver of a field read or field write;
* the receiver of a method call of `C`.

Any other position refuses: an array element, an object-literal value, the
right-hand side of a field write (`o.p = s`), an argument to a host function,
a concat or template operand, a conditional or logical operand, and the
operand of `instanceof`, `typeof`, `===`, `!==`, `==`, `!=`, spread,
destructuring, `delete` and `in`.

**Method calls on an unresolved receiver.** If `recv.m(...)` names a method
of some rewritten class and `recv` does not resolve, the call refuses, unless
`recv` is provably not an instance. "Provably not" means a binding whose
initializer and every assignment are array, object or string literals, a
literal expression, a host value under the unresolved-member-call spec's
§3.2, or a free global. So `arr.push(x)` keeps working next to
`class Stack { push(){…} }`, where the baseline's bare-name dispatch would
have called the user's `push`.

**Method values.** `s.m` not in callee position, `C.prototype.m`, and a
method call with `.call` / `.apply` / `.bind` refuse.

### 3.4 Refusals

All are `E5506` (`FEATURE_UNAVAILABLE`), raised by the rewrite through
`analyze_source_file`, and so identical under `run`, `build` and `check`.

| refused | message |
|---|---|
| `new C(...)`, `C` a program class not rewritten | constructing class `C` is unavailable in the current phase: it uses `<extends a program class / a getter or setter / a static member / a #private member / a computed member name>` |
| `new f(...)`, `f` a program `function` | constructing an object with the plain function `f` is unavailable in the current phase; use a class |
| a field write to `f` not in `C`'s field set | field `f` of class `C` is assigned outside its declared fields and the constructor's leading `this.f = …` assignments; declare it, or assign it at the start of the constructor |
| a field read `r.f` on an instance of `C`, where `f` is neither in `C`'s field set nor a method of `C` | field `f` is not declared on class `C`; reading it is unavailable in the current phase |
| a declared field with no initializer, not assigned by the leading run | field `f` of class `C` has no initial value |
| a `return` with a value in a constructor | a constructor that returns a value is unavailable in the current phase |
| an instance in a binding, parameter or return reached by another class or a non-instance | `<binding / parameter p of f / the return of f>` holds instances of more than one class; this is unavailable in the current phase |
| an instance in a position outside the allowlist | using an instance of class `C` as `<position>` is unavailable in the current phase |
| a method call on an unresolved receiver | could not determine the class of the receiver of `.m()`; method `m` belongs to class `C` |
| a method value | taking method `m` of class `C` as a value is unavailable in the current phase |

A class never constructed is not rewritten and raises nothing.

### 3.5 Compound assignment and update on an instance

For a receiver `r` that is `this` (before replacement) or a plain identifier
holding an instance:

* `r.f op= e` becomes `r.f = r.f op e`, for `op` in `+ - * / % ** << >> >>> & | ^`.
* `r.f++`, `++r.f`, `r.f--` and `--r.f` become `r.f = r.f + 1` (or `- 1`) **in
  statement position only**: an expression statement or a `for` update
  clause. There the value is unused, so prefix and postfix agree.

Evaluating a plain receiver twice is unobservable, so the rewrite is exact.
An update in value position, a logical assignment (`&&=`, `||=`, `??=`) and
any other receiver are left as written, and the object-field lane refuses
them as at the baseline (§2.3). Object-literal receivers are not rewritten.

### 3.6 The codegen backstop

A call whose callee resolves to a class-body function (`function_shape`'s
class node, `kali_codegen/src/lower.rs:2840-2925`) of a class that is not
host-derived refuses with `E5506` ("constructing class `C` is unavailable in
the current phase"), in place of pushing `i64.const 0`. After the rewrite this
should be unreachable; it exists so that a `new` form the rewrite misses
(`new (C)()`, for one) fails closed rather than producing a zero instance.
Host-derived classes keep the baseline path.

### 3.7 Docs

* `specs/15-errors.md`: the §3.4 messages under `E5506`.
* `specs/19-feature-maturity.md`: a row for program-class instances that
  states the §1 slice and the §1.1 exclusions exactly. No "supports classes"
  claim.
* `specs/05-ir.md:97`: a note that class instances get static layout by
  lowering to object literals before IR, for the §3.2 slice.
* `kali-silent-miscompile-register.md`: R-36's lane moves, with the commit
  and the case that pins it. The blast-radius ranking is regenerated as its
  procedure requires.
* `literal-array-mutators-discovered-defects.md`: §14a and §14b marked fixed,
  with the commit.
* A new `docs/superpowers/followups/class-instances-discovered-defects.md`
  (§5.4).
* `README.md` is unchanged: no CLI usage moves.

---

## 4. Approaches not taken

* **B. Carry `new`, `this` and `class` natively through the IR.** Distinct
  LIR markers, class shapes in `repr_infer`, allocation on `new`, an implicit
  `this` parameter and per-class mangling in codegen. It is the faithful
  reading of `specs/05-ir.md:97` and the base inheritance and dynamic dispatch
  would want, but it touches the parser, HIR, MIR, LIR, `kali_types` and
  codegen, and rebuilds for a second kind of object the parameter, return and
  monomorphization lanes the object-literal projects already hardened. A's
  rewrite can be replaced by B later; the cases pin behaviour, not the
  mechanism.
* **C. Rewrite on the HIR.** `repr_infer` runs on the AST, so a HIR rewrite
  would never be shaped.

---

## 5. Testing and measurement

### 5.1 Probes

A `cls_*` family in `tools/array-return-probes/probes/`, measured with
`run.sh` into `baseline-cls.tsv` at `7c4daa9f7` before any code change. It
holds:

* every §2.1 row, and one row per §1 item, which must move SILENT (or
  `E4201` / `E5506`) → MATCHES;
* one row per §3.4 refusal, which must move to REFUSES with `check` exit 1;
* controls that must not move: a host-derived class (ruling R8's
  `this.addEventListener`), a static-only class, `arr.push` next to a user
  `push`, and the §2.3 object-literal rows;
* the R-30 boolean field on an instance, which stays SILENT (§1.1).

### 5.2 Cases

A new `crates/kali_cli/tests/cases/object/class_instances.toml`, js and ts,
`run` and `check` steps. It covers the §2.1 programs, §1 items 1 to 7 (this
in an arrow inside a method, instances through parameters and returns,
compound assignment), every §3.4 refusal with its message, the controls of
§5.1 and the R-30 row as a known-wrong control with its rationale.

### 5.3 Unit tests

`kali_types/src/class_instances_tests.rs`: the provenance fixpoint (each of
§3.3's five rules, the mixed-class and unresolved cases, the
provably-not-an-instance exception), the translation (the printed AST for
§3.2's steps 1 to 5, arrow versus `function` replacement of `this`, §3.5's
forms, name collision), and a test that a program without a rewritable class
is returned byte-identical.

The codegen backstop (§3.6) gets a test in the sibling test file of the module
it lands in.

### 5.4 Blast radius and capability loss

After the change, `cargo test --workspace` and
`cargo test -p kali_cli --test cases`. Every trial whose outcome moved goes in
a triage table in the followups file, classed as one of:

* **wanted**: a silent wrong value became correct or a refusal;
* **capability loss**, in one of three classes: (1) a program node runs
  correctly that kali ran correctly and now refuses; (2) the same, where the
  refused code is dead; (3) a program whose output never depended on the
  zero instance;
* **rationale only**: the verdict is unchanged and only the text moved.

Known movers: `soundness/block_arrows.toml`'s `function Box(v){this.v=v}` /
`new Box(9)` (now refused: a plain-function constructor, class 3) and its
`class Box{constructor(){this.n=4}}` (now node-correct);
`runtime/inline_allocation_value_position.toml`'s `a_constructed_argument_refuses`
(`f(new C())`: correct if `C` is in the slice, otherwise the refusal text
moves); the oracle tier cases that construct a class.

The sweep re-runs every `cli` and `oracle` step whose stderr carries one of
§3.4's messages against the baseline binary, as the unresolved-member-call
project's §5 did, so each new refusal of a previously exit-0 program is
classified.

The followups file also records, as future items: `extends` between program
classes, accessors, statics, `#private`, containers of instances, mixed-class
parameters, rendering an instance, `new` of a plain function, and R-30 on
instance fields.

---

## 6. Amendments

Found while planning, at the baseline `7c4daa9f7`.

* **A-1. Out-of-slice classes refuse only when their chain is stateful (the
  human partner's choice, option 1 of two).** `class A{ f(){return 4;} }
  class B extends A{}` then `new B().f()` prints `4` at the baseline, as node
  does. The unresolved-member-call project pins it as
  `a_method_inherited_from_a_program_base_still_matches_node_under_run`, and
  §3.4's first row would have refused it. A stateful chain is silently wrong
  at the baseline: `class A{ constructor(){ this.n=1; } } class B extends A{
  g(){ return this.n; } }` then `new B().g()` prints `0` where node prints `1`.
  The other option was to refuse every chain. §3.2 and §3.4 are amended:
  * A class's **chain** is the class plus every program class it extends or
    that extends it, transitively. A host-derived chain is never touched.
  * A class is **rewritten** when its chain is the class alone and it has no
    getter, setter, `static` member, `#private` member or computed member
    name.
  * Every other program class is **out of slice**. Its chain is
    **stateful** when any class in it has a declared field, a `constructor`,
    a getter or setter, a `#private` member, or a `this` anywhere in a method
    body, nested arrows included. A stateful out-of-slice class refuses at
    `new` (§3.4's first row, with the reason). A stateless out-of-slice
    class is not touched, and keeps the baseline's bare-name dispatch. So
    `class U { static twice(x){ return 2*x; } }` (`U.twice(4)` prints `8`)
    and the control above keep working, and
    `class A{ get v(){ return 3; } }` (`new A().v` prints `0` at the
    baseline; node `3`) refuses.
  * A base class of any program class is never rewritten, so an inherited
    method keeps its bare name.
* **A-2. The parser keeps what the rewrite needs.** At the baseline
  `parse_class_body` (`kali_parser/src/declaration.rs:341-445`) parses
  `get v(){}`, `set v(x){}` and `static f(){}` as plain methods `v`, `v` and
  `f` (the modifier token is skipped), skips every field initializer, and
  records nothing for `#x`. The AST gains `ClassBody.fields`
  (`ClassField { name, value: Option<Expression>, is_static }`),
  `ClassBody.has_private_members`, `MethodDefinition.kind` (`Method`, `Get`,
  `Set`) and `MethodDefinition.is_static`, all `#[serde(default)]`. And `new`
  takes a member expression and its own arguments, then the call/member chain
  continues on the `NewExpression`: `new C(a).m()` parses as
  `(new C(a)).m()`, where it parsed as `new (C(a).m())`
  (`kali_parser/src/expression/primary.rs:228-263`). That makes §1 item 6
  reachable.
* **A-3. The receiver is `__this`, not `self`.** `self` is a common user name
  and a browser global. The method parameter and the factory's object are
  named `__this`, and the factory's field locals `__f_<field>`. §3.2's
  examples read `self` for `__this`. Every mangled name (`C__new`, `C__m`,
  `__this`, `__f_<field>`) is checked against every identifier the program
  spells. A collision refuses with `E5506` ("the name `X` that kali would
  generate for class `C` is already used by this program"), rather than being
  an internal error.
* **A-4. A parameter carries no type annotation in the AST.**
  `FunctionDeclaration.params` and `MethodDefinition.params` are
  `Vec<String>`, so §3.3 rule 4's annotation cross-check has nothing to read
  and is dropped. A parameter resolves from its call sites alone.
* **A-5. A call naming neither a method nor a field of `C`** on an instance of
  `C` refuses with `kali_common::unresolved_member_call_unavailable_message`,
  the shared text of the unresolved-member-call project. For a `const`
  receiver the resolver's mirror fires first, and `analyze_source_file`
  returns before the rewrite runs (`compile.rs:770-772`), so the two never
  both report. A call to a *field* (`c.cb()` where `cb = () => 5`) is left as
  written; the object-literal lane decides it.
* **A-6. Only `repr_infer` re-runs on the rewritten program.** The resolver's
  other repr-driven checks (`resolve/mod.rs:615`, `:777`) ran on the program
  as written, before the rewrite. A method body moves into `C__m` unchanged
  but for `this`, so those checks have already seen its code.
* **A-7. A block-bodied arrow is marked.** At the baseline
  `try_parse_block_arrow_function_expression`
  (`kali_parser/src/declaration.rs:602-640`) returns a plain
  `FunctionExpression`, so the AST cannot tell `() => { this.n = 1; }` from
  `function(){ this.n = 1; }`. §3.2's "into arrow functions, not into nested
  `function` expressions" needs the difference. `FunctionExpression` gains
  `#[serde(default)] pub is_arrow: bool`, set by that parser path only.
* **A-8. Class expressions and exported classes are out of slice.**
  §3.2 rewrites a class *declaration*. A `ClassExpression`
  (`const K = class {…}`), an `export default class`, a class named in an
  `export { … }` specifier, and a class whose name `ProgramClasses` marks
  ambiguous (declared twice) are out of slice under A-1. Each escapes the
  rewrite's whole-program view, the way a function used as a value escapes
  §3.3 rule 4. So each refuses at `new` when stateful, and is untouched when
  stateless.
