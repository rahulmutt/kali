# Class Instances Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A program-class instance holds its fields. `new C(a)` runs the
field initializers and the constructor. `this.f` and `s.f` read and write the
field, and `s.m()` runs `m` with `this` bound to `s`. Every construction or
use outside the slice refuses with `E5506` under `run` and `check` alike, so
no instance is ever the silent constant `0`.

**Architecture:**
- **A new AST pass, `kali_types::class_instances`.** It runs inside
  `analyze_source_file` after the resolver and before `repr_infer` is
  re-run. It rewrites each in-slice class to a factory `C__new` and one
  function `C__m(__this, …)` per method. An instance becomes an ordinary
  object literal, which the existing object-literal lanes (shapes,
  parameters, returns, monomorphization) already compile correctly.
- **Instance provenance.** A whole-program fixpoint over bindings,
  parameters and returns decides which expressions are instances of which
  class. An allowlist of positions decides where an instance may appear.
  Everything else refuses.
- **Parser and AST.** They gain what the rewrite needs: field initializers,
  accessor/static/`#private` markers, and an `is_arrow` flag on block-bodied
  arrows. Codegen is not changed.

**Tech Stack:** Rust workspace (`kali_ast`, `kali_parser`, `kali_types`,
`kali_common`, `kali_cli`), TOML case files, the bash probe runner, and node
v26.10.0 as the oracle.

**Spec:** `docs/superpowers/specs/2026-10-04-class-instances-design.md`, read
together with its amendments A-1 to A-10 in §6. **The amendments override
§3:**
- A-3: the names are `__this` and `__f_<field>`, not `self`.
- A-9: `new` precedence is re-associated in the rewrite, and the parser's
  `new` is not changed.
- A-10: the backstop is a sweep in the rewrite, not in codegen.

## Global Constraints

- **Diagnostic code.** Every refusal is `E5506` (`e5::FEATURE_UNAVAILABLE`).
  There is no new code, flag or schema, and `specs/12` and `specs/18` stay
  untouched.
- **Message text.** Every refusal string comes from a function in
  `kali_common::messages` (Task 1). The pass never formats its own text.
- **Generated names.** They are `C__new`, `C__<method>`, `__this` and
  `__f_<field>` (A-3). A collision with any identifier the program spells
  refuses (A-3). It is never a silent shadow.
- **No-op guarantee.** A program with no rewritten class and no refusal
  passes through the pass byte-identical (`changed == false`), and
  `infer_reprs` is not re-run (spec §3.1).
- **Untouched classes.** Host-derived classes and stateless out-of-slice
  classes are never touched (A-1). Classes never constructed are never
  touched (spec §3.4).
- **The parser's `new` is not changed (A-9).** The rewrite re-associates
  `new (C(a).m())` only when `C` is a rewritten class.
- **Codegen (`crates/kali_codegen`) is not changed (A-10).**
- **Test layout.** Rust unit tests go in sibling `*_tests.rs` files wired
  with `#[cfg(test)] #[path = "…"] mod …;`, never inline `mod tests {}`
  bodies.
- **CLI tests.** CLI tests are `.toml` cases under
  `crates/kali_cli/tests/cases/`. Don't add a `tests/*.rs` target.
- **Oracle and rationales.** The oracle is `node v26.10.0`. Every case
  rationale quotes node's output and kali's output at the baseline
  `7c4daa9f7`.
- **Stop rule.** If more than about 50 existing case trials move in Task 10,
  stop and report before re-pinning anything.
- **Capability loss.** A loss is acceptable only in spec §5.4 classes 2
  (dead code) and 3 (output never depended on the zero instance). A class-1
  loss (a program node runs correctly, which kali also ran correctly, now
  refuses) goes back to the human partner.
- **Commit messages.** Use `feat(class-instances): …`, `test(…)`, `docs(…)`.

## Review Focus

1. **Two rewritten classes with the same method name.** In
   `class A{ f(){ return 1; } } class B{ f(){ return 2; } }`, each resolved
   receiver must call its own `f` (`A__f` / `B__f`), never the other's. A
   parameter fed both classes must refuse, never pick one. Pinned in Task 6
   (unit) and Task 9 (case).
2. **A method that calls another method or itself through `this`.** In
   `this.b()` and `this.fact(n-1)`, the receiver is `this`, and the return
   provenance of a recursive method starts at `Bottom` and must converge.
   Pinned in Task 5 (unit), Task 6 (unit) and Task 9 (case).
3. **A program that already spells a generated name** (`const __this = 1;`,
   `function C__new(){}`) must refuse with the collision message, never
   shadow it. Pinned in Task 4.
4. **An instance captured by an arrow and used inside it**
   (`const s = new C(); const f = () => { s.add(1); }; f();`). Resolving the
   name from inside the arrow's frame must reach the outer binding. Pinned in
   Task 3 (scope unit) and Task 9 (case).
5. **The same class name declared in two functions**
   (`function a(){ class P{ constructor(){ this.n=1; } } return new P(); }`
   plus another `P` in `b`). `ProgramClasses` marks the name ambiguous, so
   both are out of slice (A-8) and stateful, and each `new P()` refuses.
   Pinned in Task 4.

---

## File Structure

| file | change | responsibility |
|---|---|---|
| `crates/kali_common/src/messages.rs` | modify | the refusal messages (Task 1) |
| `crates/kali_common/src/messages_tests.rs` | modify | message stability tests |
| `crates/kali_ast/src/declaration.rs` | modify | `ClassField`, `MethodKind`, `ClassBody.fields` / `has_private_members` / `has_static_block`, `MethodDefinition.kind` / `is_static`, `Default` derives |
| `crates/kali_ast/src/expression.rs` | modify | `FunctionExpression.is_arrow`, `Default` derive |
| `crates/kali_parser/src/declaration.rs` | modify | the class-member loop keeps fields, initializers and modifiers; block arrows set `is_arrow` |
| `crates/kali_parser/src/declaration_tests/class_method.rs` | modify | parser tests |
| construction sites `cargo build` reports | modify | add the new fields |
| `crates/kali_types/src/program_classes.rs` | modify | `is_ambiguous` accessor |
| `crates/kali_types/src/class_instances/mod.rs` | create | entry `rewrite_class_instances`, phase order, A-10 sweep |
| `crates/kali_types/src/class_instances/walk.rs` | create | exhaustive mutable AST walker with `Pos` and frames |
| `crates/kali_types/src/class_instances/scopes.rs` | create | per-frame declarations and name resolution |
| `crates/kali_types/src/class_instances/classes.rs` | create | class classification (A-1, A-8), field sets, collisions, `new` refusals |
| `crates/kali_types/src/class_instances/provenance.rs` | create | `new` re-association (A-9), abstraction, facts, fixpoint |
| `crates/kali_types/src/class_instances/uses.rs` | create | allowlist refusals, `new` / method-call / compound rewrites |
| `crates/kali_types/src/class_instances/translate.rs` | create | class → `C__new` + `C__m`, `this` → `__this` |
| `crates/kali_types/src/class_instances/*_tests.rs` | create | one sibling test file per module |
| `crates/kali_types/src/lib.rs` | modify | `pub mod class_instances;`, `pub use repr_infer::infer_reprs;` |
| `crates/kali_cli/src/build/compile.rs` | modify | call the pass in `analyze_source_file` |
| `tools/array-return-probes/probes/cls_*.js` | create | probes |
| `tools/array-return-probes/baseline-cls.tsv` | create | baseline at `7c4daa9f7` |
| `crates/kali_cli/tests/cases/object/class_instances.toml` | create | CLI cases |
| `specs/15-errors.md`, `specs/19-feature-maturity.md`, `specs/05-ir.md` | modify | docs (Task 11) |
| `docs/superpowers/followups/kali-silent-miscompile-register.md` | modify | R-36 lane |
| `docs/superpowers/followups/literal-array-mutators-discovered-defects.md` | modify | §14a / §14b fixed |
| `docs/superpowers/followups/class-instances-discovered-defects.md` | create | triage, capability loss, gaps |

**Phase order inside `rewrite_class_instances`.** Every module consumes the
previous one's output:

```
classes::plan_classes            → ClassPlans (rewritten set, field sets, refusals)
provenance::reassociate_new      → canonical `new C(a)` for rewritten classes
scopes::Scopes::build            → declarations per frame
provenance::Provenance::solve    → Val per binding / parameter / return
uses::check_and_rewrite          → refusals, `new` and method-call rewrites
translate::translate_classes     → class declarations become functions
mod.rs sweep (A-10)              → nothing rewritten is left behind
```

---

### Task 0: Probes and the baseline (before any code change)

**Files:**
- Create: `tools/array-return-probes/probes/cls_*.js` (listed below)
- Create: `tools/array-return-probes/baseline-cls.tsv`

**Interfaces:**
- Produces: the probe names Task 9 diffs against.

- [ ] **Step 1: Confirm you are at the baseline.**

Run: `git log --oneline -1 -- crates/ && cargo build -q -p kali_cli && node --version`
Expected: the last commit touching `crates/` is `7c4daa9f7` or earlier, and node prints `v26.10.0`.

- [ ] **Step 2: Write the probe files.** Create each file under `tools/array-return-probes/probes/`, one program per file, with a trailing newline.

| file | content |
|---|---|
| `cls_14b.js` | `class Stack{ constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } function main(){ const s=new Stack(); s.add(3); console.log(s.n); } main();` |
| `cls_top.js` | `class Stack{ constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } const s=new Stack(); s.add(3); console.log(s.n);` |
| `cls_ctor_read.js` | `class C{ constructor(){ this.n=5; } } const s=new C(); console.log(s.n);` |
| `cls_method_read.js` | `class C{ constructor(){ this.n=5; } get(){ return this.n; } } const s=new C(); console.log(s.get());` |
| `cls_method_write.js` | `class C{ constructor(){ this.n=0; } set(v){ this.n=v; } } const s=new C(); s.set(9); console.log(s.n);` |
| `cls_outside_write.js` | `class C{ constructor(){ this.n=0; } } const s=new C(); s.n=4; console.log(s.n);` |
| `cls_field_decl.js` | `class C{ n=1; set(v){ this.n=v; } } const s=new C(); s.set(9); console.log(s.n);` |
| `cls_ctor_chain.js` | `class C{ constructor(){ this.n=0; this.m=this.n+2; } } const s=new C(); console.log(s.m);` |
| `cls_ctor_arg.js` | `class C{ constructor(v){ this.n=v; } get(){ return this.n+1; } } const s=new C(4); console.log(s.get(), s.n);` |
| `cls_update.js` | `class C{ constructor(){ this.n=0; } inc(){ this.n++; } get(){ return this.n; } } const s=new C(); s.inc(); s.inc(); console.log(s.get(), s.n);` |
| `cls_compound.js` | `class Acc{ total=0; add(x){ this.total += x; return this.total; } } const a=new Acc(); a.add(2); console.log(a.add(5));` |
| `cls_14a.js` | `class S { push(v){ return v+1; } } console.log(new S().push(1));` |
| `cls_r36.js` | `class C{ constructor(){ this.v=3; } } console.log(new C().v);` |
| `cls_param.js` | `class P{ constructor(x,y){ this.x=x; this.y=y; } } function len2(p){ return p.x*p.x+p.y*p.y; } console.log(len2(new P(3,4)));` |
| `cls_return.js` | `class C{ constructor(v){ this.v=v; } } function mk(v){ return new C(v); } const c=mk(7); console.log(c.v);` |
| `cls_this_method.js` | `class A{ a(){ return this.b()+1; } b(){ return 2; } } const x=new A(); console.log(x.a());` |
| `cls_recursive.js` | `class M{ constructor(){ this.calls=0; } fact(n){ this.calls=this.calls+1; if (n<=1) { return 1; } return n*this.fact(n-1); } } const m=new M(); console.log(m.fact(5), m.calls);` |
| `cls_this_arrow.js` | `class C{ constructor(){ this.n=1; } bump(){ const f = () => { this.n = this.n + 1; }; f(); f(); return this.n; } } const c=new C(); console.log(c.bump());` |
| `cls_capture.js` | `class C{ constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } const s=new C(); const f = () => { s.add(4); }; f(); console.log(s.n);` |
| `cls_same_method.js` | `class A{ f(){ return 1; } } class B{ f(){ return 2; } } const a=new A(); const b=new B(); console.log(a.f(), b.f());` |
| `cls_in_main.js` | `function main(){ class C{ constructor(v){ this.v=v; } } const c=new C(2); console.log(c.v); } main();` |
| `cls_r_extends_stateful.js` | `class A{ constructor(){ this.n=1; } } class B extends A{ g(){ return this.n; } } const b=new B(); console.log(b.g());` |
| `cls_r_getter.js` | `class A{ get v(){ return 3; } } const a=new A(); console.log(a.v);` |
| `cls_r_plain_fn_new.js` | `function Box(v){ this.v=v; } const b=new Box(9); console.log(b.v);` |
| `cls_r_field_outside.js` | `class C{ constructor(){ this.n=0; } set(){ this.m=1; } } const c=new C(); c.set(); console.log(c.m);` |
| `cls_r_no_init.js` | `class C{ n; } const c=new C(); console.log(c.n);` |
| `cls_r_ctor_return.js` | `class C{ constructor(){ this.n=1; return {n:2}; } } const c=new C(); console.log(c.n);` |
| `cls_r_mixed_param.js` | `class A{ constructor(){ this.n=1; } } class B{ constructor(){ this.n=2; } } function f(x){ return x.n; } console.log(f(new A()), f(new B()));` |
| `cls_r_log_instance.js` | `class C{ constructor(){ this.n=1; } } const c=new C(); console.log(c);` |
| `cls_r_array.js` | `class C{ constructor(){ this.n=1; } } const xs=[new C(), new C()]; console.log(xs[1].n);` |
| `cls_r_unresolved_recv.js` | `class C{ constructor(){ this.n=1; } get(){ return this.n; } } function f(x){ return x.get(); } const g=f; console.log(g(new C()));` |
| `cls_r_method_value.js` | `class C{ constructor(){ this.n=1; } get(){ return this.n; } } const c=new C(); const m=c.get; console.log(typeof m);` |
| `cls_r_undeclared_read.js` | `class C{ constructor(){ this.n=1; } } const c=new C(); console.log(c.zz);` |
| `cls_r_instanceof.js` | `class C{ constructor(){ this.n=1; } } const c=new C(); console.log(c instanceof C);` |
| `cls_r_ambiguous.js` | `function a(){ class P{ constructor(){ this.n=1; } } return new P(); } function b(){ class P{ constructor(){ this.n=2; } } return new P(); } console.log(a().n, b().n);` |
| `cls_ok_host_r8.js` | `class X extends EventTarget { fire(){ this.addEventListener("t", () => {}); return 1; } } const x = new X(); console.log(x.fire());` |
| `cls_ok_static_only.js` | `class U { static twice(x){ return 2*x; } } console.log(U.twice(4));` |
| `cls_ok_extends_stateless.js` | `class A{ f(){return 4;} } class B extends A{ g(){ return 1; } } const b=new B(); console.log(b.f()+b.g());` |
| `cls_ok_user_push.js` | `class Stack{ constructor(){ this.n=0; } push(v){ this.n=this.n+v; return this.n; } } const s=new Stack(); const a=[1]; a.push(2); console.log(a.length, s.push(5));` |
| `cls_ok_objlit.js` | `function mk(){ return {n:0}; } function get(o){ return o.n; } const a=mk(); a.n=2; console.log(get(a));` |
| `cls_known_r30.js` | `class F{ constructor(){ this.ok=false; } set(){ this.ok=true; } } const f=new F(); f.set(); console.log(f.ok);` |

- [ ] **Step 3: Record the baseline.**

Run:
```bash
tools/array-return-probes/run.sh "$TMPDIR/cls-probes-all.tsv"
grep '^cls_' "$TMPDIR/cls-probes-all.tsv" > tools/array-return-probes/baseline-cls.tsv
cut -f1,2,5 tools/array-return-probes/baseline-cls.tsv
```
(Use the scratchpad directory for `$TMPDIR` if it is unset.)

Expected:
- `cls_ok_*` rows are `CORRECT`.
- `cls_update` is `REFUSES`.
- `cls_ctor_arg` is `OTHER` (E4201).
- `cls_r_plain_fn_new` and `cls_r_log_instance` are `SILENT`.
- Every other row is `SILENT`, except where node itself throws (`cls_r_ctor_return` is `SILENT`, because node prints `2`).

If a row differs, add a line to the Task 11 followups file saying which row and what it showed. Don't change the probe to make it fit.

- [ ] **Step 4: Commit.**

```bash
git add tools/array-return-probes/probes/cls_* tools/array-return-probes/baseline-cls.tsv
git commit -m "test(class-instances): add cls_* probes and their baseline at 7c4daa9f7"
```

---

### Task 1: The refusal messages

**Files:**
- Modify: `crates/kali_common/src/messages.rs` (after `unresolved_member_call_unavailable_message`, about line 236)
- Test: `crates/kali_common/src/messages_tests.rs`

**Interfaces:**
- Produces, all re-exported at the `kali_common` root:
  - `class_construction_unavailable_message(class: &str, reason: &str) -> String`
  - the reason constants `CLASS_REASON_EXTENDS`, `CLASS_REASON_ACCESSOR`, `CLASS_REASON_STATIC`, `CLASS_REASON_PRIVATE`, `CLASS_REASON_COMPUTED`, `CLASS_REASON_EXPRESSION`, `CLASS_REASON_EXPORTED`, `CLASS_REASON_AMBIGUOUS`, `CLASS_REASON_UNLOWERED`, all `&'static str`
  - `plain_function_construction_unavailable_message(name: &str) -> String`
  - `class_field_outside_set_message(class: &str, field: &str) -> String`
  - `class_field_undeclared_read_message(class: &str, field: &str) -> String`
  - `class_field_without_initial_value_message(class: &str, field: &str) -> String`
  - `class_field_initializer_this_message(class: &str, field: &str) -> String`
  - `constructor_return_unavailable_message() -> &'static str`
  - `class_instance_mixed_message(class: &str, place: &str) -> String`
  - `class_instance_position_message(class: &str, position: &str) -> String`
  - `class_receiver_unresolved_message(method: &str, class: &str) -> String`
  - `class_method_value_message(class: &str, method: &str) -> String`
  - `class_value_message(class: &str) -> String`
  - `class_generated_name_collision_message(name: &str, class: &str) -> String`

- [ ] **Step 1: Write the failing tests.** Append to `messages_tests.rs`:

```rust
#[test]
fn class_instance_messages_are_stable() {
    assert_eq!(
        class_construction_unavailable_message("B", CLASS_REASON_EXTENDS),
        "constructing class `B` is unavailable in the current phase: it is in an `extends` chain with another program class and keeps state; kali refuses rather than build an instance whose fields read 0"
    );
    assert_eq!(
        plain_function_construction_unavailable_message("Box"),
        "constructing an object with the plain function `Box` is unavailable in the current phase; use a class"
    );
    assert_eq!(
        class_field_outside_set_message("C", "m"),
        "field `m` of class `C` is assigned outside its declared fields and the constructor's leading `this.m = …` assignments; declare it, or assign it at the start of the constructor"
    );
    assert_eq!(
        class_field_undeclared_read_message("C", "zz"),
        "field `zz` is not declared on class `C`; reading it is unavailable in the current phase"
    );
    assert_eq!(
        class_field_without_initial_value_message("C", "n"),
        "field `n` of class `C` has no initial value; kali cannot hold `undefined` in an instance field"
    );
    assert_eq!(
        class_field_initializer_this_message("C", "n"),
        "the initializer of field `n` of class `C` uses `this` beyond the fields already set; this is unavailable in the current phase"
    );
    assert_eq!(
        constructor_return_unavailable_message(),
        "a constructor that returns a value is unavailable in the current phase"
    );
    assert_eq!(
        class_instance_mixed_message("A", "parameter `x` of `f`"),
        "parameter `x` of `f` may hold an instance of class `A` and other values; this is unavailable in the current phase"
    );
    assert_eq!(
        class_instance_position_message("C", "an argument to a host call"),
        "using an instance of class `C` as an argument to a host call is unavailable in the current phase"
    );
    assert_eq!(
        class_receiver_unresolved_message("get", "C"),
        "could not determine the class of the receiver of `.get()`; method `get` belongs to class `C`, and kali refuses rather than call it without its instance"
    );
    assert_eq!(
        class_method_value_message("C", "get"),
        "taking method `get` of class `C` as a value is unavailable in the current phase"
    );
    assert_eq!(
        class_value_message("C"),
        "using class `C` as a value is unavailable in the current phase; only `new C(…)` is supported"
    );
    assert_eq!(
        class_generated_name_collision_message("__this", "C"),
        "the name `__this` that kali would generate for class `C` is already used by this program; this is unavailable in the current phase"
    );
}
```

- [ ] **Step 2: Run it to verify it fails.**

Run: `cargo test -p kali_common class_instance_messages_are_stable`
Expected: compile error, `cannot find function class_construction_unavailable_message`.

- [ ] **Step 3: Implement.** Add to `messages.rs` after `unresolved_member_call_unavailable_message`:

```rust
/// Why a stateful out-of-slice class refuses at `new` (class-instances spec
/// A-1, A-8, A-10). The text completes "it …".
pub const CLASS_REASON_EXTENDS: &str =
    "is in an `extends` chain with another program class and keeps state";
pub const CLASS_REASON_ACCESSOR: &str = "has a getter or setter";
pub const CLASS_REASON_STATIC: &str = "has a static member and keeps state";
pub const CLASS_REASON_PRIVATE: &str = "has a #private member";
pub const CLASS_REASON_COMPUTED: &str = "has a computed member name and keeps state";
pub const CLASS_REASON_EXPRESSION: &str = "is a class expression and keeps state";
pub const CLASS_REASON_EXPORTED: &str = "is exported and keeps state";
pub const CLASS_REASON_AMBIGUOUS: &str = "is declared more than once and keeps state";
pub const CLASS_REASON_UNLOWERED: &str = "reached code generation without being lowered";

/// `new C(…)` of a program class kali does not lower to an object
/// (class-instances spec §3.4, A-1).
pub fn class_construction_unavailable_message(class: &str, reason: &str) -> String {
    format!(
        "constructing class `{class}` is unavailable in the current phase: it {reason}; kali refuses rather than build an instance whose fields read 0"
    )
}

/// `new f(…)` of a program `function` (class-instances spec §3.4).
pub fn plain_function_construction_unavailable_message(name: &str) -> String {
    format!(
        "constructing an object with the plain function `{name}` is unavailable in the current phase; use a class"
    )
}

pub fn class_field_outside_set_message(class: &str, field: &str) -> String {
    format!(
        "field `{field}` of class `{class}` is assigned outside its declared fields and the constructor's leading `this.{field} = …` assignments; declare it, or assign it at the start of the constructor"
    )
}

pub fn class_field_undeclared_read_message(class: &str, field: &str) -> String {
    format!(
        "field `{field}` is not declared on class `{class}`; reading it is unavailable in the current phase"
    )
}

pub fn class_field_without_initial_value_message(class: &str, field: &str) -> String {
    format!(
        "field `{field}` of class `{class}` has no initial value; kali cannot hold `undefined` in an instance field"
    )
}

pub fn class_field_initializer_this_message(class: &str, field: &str) -> String {
    format!(
        "the initializer of field `{field}` of class `{class}` uses `this` beyond the fields already set; this is unavailable in the current phase"
    )
}

pub const fn constructor_return_unavailable_message() -> &'static str {
    "a constructor that returns a value is unavailable in the current phase"
}

pub fn class_instance_mixed_message(class: &str, place: &str) -> String {
    format!(
        "{place} may hold an instance of class `{class}` and other values; this is unavailable in the current phase"
    )
}

pub fn class_instance_position_message(class: &str, position: &str) -> String {
    format!(
        "using an instance of class `{class}` as {position} is unavailable in the current phase"
    )
}

pub fn class_receiver_unresolved_message(method: &str, class: &str) -> String {
    format!(
        "could not determine the class of the receiver of `.{method}()`; method `{method}` belongs to class `{class}`, and kali refuses rather than call it without its instance"
    )
}

pub fn class_method_value_message(class: &str, method: &str) -> String {
    format!(
        "taking method `{method}` of class `{class}` as a value is unavailable in the current phase"
    )
}

pub fn class_value_message(class: &str) -> String {
    format!(
        "using class `{class}` as a value is unavailable in the current phase; only `new {class}(…)` is supported"
    )
}

pub fn class_generated_name_collision_message(name: &str, class: &str) -> String {
    format!(
        "the name `{name}` that kali would generate for class `{class}` is already used by this program; this is unavailable in the current phase"
    )
}
```

- [ ] **Step 4: Run it to verify it passes.**

Run: `cargo test -p kali_common class_instance_messages_are_stable`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_common/src/messages.rs crates/kali_common/src/messages_tests.rs
git commit -m "feat(class-instances): the refusal messages"
```

---

### Task 2: The AST and the parser keep fields, modifiers and block arrows (A-2, A-7)

**Files:**
- Modify: `crates/kali_ast/src/declaration.rs:20-54`
- Modify: `crates/kali_ast/src/expression.rs:157-166`
- Modify: `crates/kali_parser/src/declaration.rs:341-445` (`parse_class_body`) and `:628-638` (block arrow)
- Modify: every construction site `cargo build --workspace --tests` reports
- Test: `crates/kali_parser/src/declaration_tests/class_method.rs`

**Interfaces:**
- Produces:
  - `kali_ast::ClassField { pub name: String, pub value: Option<Expression>, pub is_static: bool }`
  - `kali_ast::MethodKind { Method, Get, Set }`, with `Default = Method`
  - `ClassBody.fields: Vec<ClassField>`, `ClassBody.has_private_members: bool`, `ClassBody.has_static_block: bool`
  - `MethodDefinition.kind: MethodKind`, `MethodDefinition.is_static: bool`
  - `FunctionExpression.is_arrow: bool`
  - every new field is `#[serde(default)]`, and `ClassBody`, `MethodDefinition` and `FunctionExpression` derive `Default`
- Unchanged: `ClassBody.field_names` still names every parsed field, static or not, so `ProgramClasses::member_names` and the `check` mirror see what they saw before.

- [ ] **Step 1: Write the failing parser tests.** Append to `declaration_tests/class_method.rs`:

```rust
fn parse_class(source: &str) -> kali_ast::ClassBody {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    match &output.statements[0] {
        Statement::ClassDeclaration(class_decl) => (*class_decl.body).clone(),
        other => panic!("expected a class, got {other:?}"),
    }
}

#[test]
fn a_field_keeps_its_initializer() {
    let body = parse_class("class C { n = 1 + 2; label: string = \"x\"; m; }");
    let names: Vec<_> = body.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["n", "label", "m"]);
    assert!(matches!(body.fields[0].value, Some(Expression::BinaryExpression(_))));
    assert!(matches!(body.fields[1].value, Some(Expression::Literal(_))));
    assert_eq!(body.fields[2].value, None);
    assert_eq!(body.field_names, ["n", "label", "m"]);
}

#[test]
fn fields_without_semicolons_end_where_the_expression_ends() {
    let body = parse_class("class C { n = 0\n m = 1\n f(){ return 2; } }");
    let names: Vec<_> = body.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["n", "m"]);
    assert_eq!(body.methods.len(), 1);
    assert_eq!(body.methods[0].name, "f");
}

#[test]
fn an_annotated_field_without_semicolon_stops_before_the_next_member() {
    let body = parse_class("class C { label: string\n m = 1 }");
    assert_eq!(body.fields[0].name, "label");
    assert_eq!(body.fields[0].value, None);
    assert_eq!(body.fields[1].name, "m");
}

#[test]
fn accessors_and_statics_are_marked() {
    let body = parse_class("class C { get v(){ return 1; } set v(x){} static make(){ return 2; } static k = 3; f(){} }");
    let kinds: Vec<_> = body.methods.iter().map(|m| (m.name.as_str(), m.kind.clone(), m.is_static)).collect();
    assert_eq!(kinds, [
        ("v", MethodKind::Get, false),
        ("v", MethodKind::Set, false),
        ("make", MethodKind::Method, true),
        ("f", MethodKind::Method, false),
    ]);
    assert_eq!(body.fields[0].name, "k");
    assert!(body.fields[0].is_static);
}

#[test]
fn a_method_named_like_a_modifier_is_a_plain_method() {
    let body = parse_class("class C { get(){ return 1; } static(){ return 2; } }");
    let names: Vec<_> = body.methods.iter().map(|m| (m.name.as_str(), m.kind.clone(), m.is_static)).collect();
    assert_eq!(names, [("get", MethodKind::Method, false), ("static", MethodKind::Method, false)]);
}

#[test]
fn private_members_and_static_blocks_are_flagged() {
    let body = parse_class("class C { #x = 1; #m(){} static { init(); } f(){} }");
    assert!(body.has_private_members);
    assert!(body.has_static_block);
    assert_eq!(body.methods.len(), 1);
    assert_eq!(body.methods[0].name, "f");
}

#[test]
fn a_block_arrow_is_marked_and_a_function_expression_is_not() {
    let tokens = lex("const f = () => { return 1; }; const g = function(){ return 2; };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    let flags: Vec<bool> = output.statements.iter().map(|s| match s {
        Statement::VariableDeclaration(d) => match &d.declarations[0].init {
            Some(Expression::FunctionExpression(f)) => f.is_arrow,
            other => panic!("expected a function expression, got {other:?}"),
        },
        other => panic!("{other:?}"),
    }).collect();
    assert_eq!(flags, [true, false]);
}
```

Add `use kali_ast::{Expression, MethodKind, Statement};` to that file's imports if they are not already in scope.

- [ ] **Step 2: Run them to verify they fail.**

Run: `cargo test -p kali_parser class_method`
Expected: compile errors, no field `fields`, no type `MethodKind`.

- [ ] **Step 3: Extend the AST.** In `kali_ast/src/declaration.rs`, replace the `ClassBody` and `MethodDefinition` definitions with:

```rust
/// Class body
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassBody {
    pub methods: Vec<MethodDefinition>,
    /// Names of the class-field declarations (`n = 0;`, `label: string;`),
    /// static or not, in source order.
    #[serde(default)]
    pub field_names: Vec<String>,
    /// The body has a member with a computed key (`["foo"](){}`), which the
    /// parser skips, so `methods` and `field_names` do not name every member.
    #[serde(default)]
    pub has_computed_members: bool,
    /// The field declarations with their initializers (class-instances spec A-2).
    #[serde(default)]
    pub fields: Vec<ClassField>,
    /// The body has a `#private` field or method, which the parser skips.
    #[serde(default)]
    pub has_private_members: bool,
    /// The body has a `static { … }` block, which the parser skips.
    #[serde(default)]
    pub has_static_block: bool,
}

/// A class field declaration (`n = 0;`, `static k = 1;`, `label: string;`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassField {
    pub name: String,
    /// The initializer; `None` for a field declared without one.
    pub value: Option<Expression>,
    pub is_static: bool,
}

/// What a class method definition defines.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MethodKind {
    #[default]
    Method,
    Get,
    Set,
}

/// Method definition
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MethodDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Box<BlockStatement>>,
    pub is_async: bool,
    pub generator: bool,
    #[serde(default)]
    pub kind: MethodKind,
    #[serde(default)]
    pub is_static: bool,
}
```

In `kali_ast/src/expression.rs`, make `FunctionExpression` derive `Default` and add the field:

```rust
/// Function expression
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionExpression {
    pub id: Option<String>,
    pub params: Vec<FunctionParam>,
    pub body: Option<Box<BlockStatement>>,
    pub is_async: bool,
    pub generator: bool,
    /// Parsed from block-bodied arrow syntax (`(a) => { … }`), whose `this`
    /// is lexical (class-instances spec A-7). `false` for `function` syntax.
    #[serde(default)]
    pub is_arrow: bool,
}
```

Export the new types from `kali_ast/src/lib.rs` the same way `ClassBody` is exported (follow the existing `pub use declaration::…` line).

- [ ] **Step 4: Fix every construction site.** Run `cargo build --workspace --tests 2>&1 | grep -E "^error|-->"`.
  - **Non-test code:** add the new fields explicitly. The two parser sites are rewritten in Step 5. The block-arrow site gets `is_arrow: true`. Every other `FunctionExpression { … }` gets `is_arrow: false`.
  - **Test code:** add `..Default::default()`.
  
  Repeat until the build is clean.

- [ ] **Step 5: Rewrite the class-member loop.** Replace the body of `parse_class_body` (`kali_parser/src/declaration.rs:341-445`) with a member-at-a-time loop:

```rust
    pub(crate) fn parse_class_body(&mut self) -> ClassBody {
        let _ = self.stream.accept(TokenType::LeftBrace);
        let mut body = ClassBody::default();
        loop {
            match self.stream.current_kind() {
                None | Some(TokenType::Eof) => break,
                Some(TokenType::RightBrace) => {
                    let _ = self.stream.advance();
                    break;
                }
                Some(TokenType::Semicolon) => {
                    let _ = self.stream.advance();
                    continue;
                }
                _ => {}
            }
            self.parse_class_member(&mut body);
        }
        body
    }

    /// Parses one class member into `body`. A member the AST does not model
    /// (`#private`, a computed key, a `static {}` block) is skipped and
    /// flagged.
    fn parse_class_member(&mut self, body: &mut ClassBody) {
        let mut is_static = false;
        let mut kind = MethodKind::Method;
        // A modifier word is a modifier only when another key follows it:
        // `get(){}` is a method named `get`, `get v(){}` is a getter.
        while let Some(token) = self.stream.current() {
            if token.kind != TokenType::Identifier
                || !CLASS_MEMBER_MODIFIERS.contains(&token.value.as_str())
                || matches!(
                    self.stream.peek_next_kind(),
                    Some(
                        TokenType::LeftParen
                            | TokenType::Eq
                            | TokenType::Semicolon
                            | TokenType::Colon
                            | TokenType::Question
                            | TokenType::Not
                            | TokenType::RightBrace
                    )
                )
            {
                break;
            }
            match token.value.as_str() {
                "static" => is_static = true,
                "get" => kind = MethodKind::Get,
                "set" => kind = MethodKind::Set,
                _ => {}
            }
            let _ = self.stream.advance();
        }
        if is_static && self.stream.current_kind() == Some(&TokenType::LeftBrace) {
            body.has_static_block = true;
            self.skip_class_member();
            return;
        }
        let is_async = if self.stream.current_kind() == Some(&TokenType::Async)
            && matches!(
                self.stream.peek_next_kind(),
                Some(TokenType::Star) | Some(TokenType::Identifier)
            ) {
            let _ = self.stream.advance();
            true
        } else {
            false
        };
        let generator = self.stream.accept(TokenType::Star);
        match self.stream.current_kind() {
            Some(TokenType::Hash) => {
                body.has_private_members = true;
                self.skip_class_member();
                return;
            }
            Some(TokenType::LeftBracket) => {
                body.has_computed_members = true;
                self.skip_class_member();
                return;
            }
            Some(TokenType::Identifier) | Some(TokenType::Async) => {}
            _ => {
                // A string or number key: not modelled; skip it.
                self.skip_class_member();
                return;
            }
        }
        let name = self.stream.advance().map(|t| t.value).unwrap_or_default();
        if self.stream.current_kind() == Some(&TokenType::LeftParen) {
            let params = self.parse_parameter_list();
            self.skip_return_type_annotation();
            let previous_async = self.in_async_function;
            let previous_generator = self.in_generator_function;
            self.in_async_function = is_async;
            self.in_generator_function = generator;
            let block = match self.parse_block_statement() {
                Some(Statement::BlockStatement(bs)) => bs,
                _ => BlockStatement { body: Vec::new() },
            };
            self.in_generator_function = previous_generator;
            self.in_async_function = previous_async;
            body.methods.push(MethodDefinition {
                name,
                params,
                body: Some(Box::new(block)),
                is_async,
                generator,
                kind,
                is_static,
            });
            return;
        }
        // A field: `name`, `name?`, `name!`, `name: Type`, each with an
        // optional `= initializer`.
        let _ = self.stream.accept(TokenType::Question) || self.stream.accept(TokenType::Not);
        if self.stream.current_kind() == Some(&TokenType::Colon) {
            self.skip_field_type_annotation();
        }
        let value = if self.stream.accept(TokenType::Eq) {
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let _ = self.stream.accept(TokenType::Semicolon);
        body.field_names.push(name.clone());
        body.fields.push(ClassField { name, value, is_static });
    }

    /// Skips a field's `: Type`, leaving the cursor on `=`, `;`, `}` or the
    /// next member's key. Tokens carry no line breaks, so an annotation
    /// without `;` ends where two words meet that no type spells together
    /// (`string m`).
    fn skip_field_type_annotation(&mut self) {
        const TYPE_WORDS: &[&str] = &[
            "keyof", "typeof", "readonly", "infer", "unique", "asserts", "is", "extends", "new",
        ];
        let _ = self.stream.advance();
        let mut depth = 0usize;
        let mut previous_word: Option<String> = None;
        while let Some(token) = self.stream.current().cloned() {
            match token.kind {
                TokenType::Eq | TokenType::Semicolon | TokenType::RightBrace if depth == 0 => break,
                TokenType::Eof => break,
                TokenType::LeftParen | TokenType::LeftBracket | TokenType::LeftBrace | TokenType::Lt => {
                    depth += 1
                }
                TokenType::RightParen | TokenType::RightBracket | TokenType::RightBrace | TokenType::Gt => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Identifier if depth == 0 => {
                    if let Some(previous) = &previous_word {
                        if !TYPE_WORDS.contains(&previous.as_str()) {
                            break;
                        }
                    }
                }
                _ => {}
            }
            previous_word = (token.kind == TokenType::Identifier).then(|| token.value.clone());
            let _ = self.stream.advance();
        }
    }

    /// Skips one member the AST does not model: through a `;` at depth 0,
    /// through the `}` that closes a body opened at depth 0, or up to the
    /// class's own `}`.
    fn skip_class_member(&mut self) {
        let mut depth = 0usize;
        while let Some(kind) = self.stream.current_kind().copied() {
            match kind {
                TokenType::Eof => return,
                TokenType::Semicolon if depth == 0 => {
                    let _ = self.stream.advance();
                    return;
                }
                TokenType::RightBrace if depth == 0 => return,
                TokenType::LeftParen | TokenType::LeftBracket | TokenType::LeftBrace => depth += 1,
                TokenType::RightParen | TokenType::RightBracket => depth = depth.saturating_sub(1),
                TokenType::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        let _ = self.stream.advance();
                        return;
                    }
                }
                _ => {}
            }
            let _ = self.stream.advance();
        }
    }
```

`TokenType::Hash` exists (`kali_lexer/src/token.rs:54`). If `stream.accept` does not return `bool`, check `token_stream.rs` and adapt the two `accept` uses. Import `ClassField` and `MethodKind` from `kali_ast` at the top of the file. In `try_parse_block_arrow_function_expression` (`:628`), set `is_arrow: true` on the built `FunctionExpression`.

- [ ] **Step 6: Run the parser tests.**

Run: `cargo test -p kali_parser`
Expected: PASS, including every pre-existing test.

- [ ] **Step 7: Check that nothing downstream moved.**

Run: `cargo test -p kali_types && cargo test -p kali_cli --test cases`
Expected: PASS. The parser now keeps field initializers and static members as data, and every consumer ignores the new fields, so no output may change.

If a case moves, the likeliest cause is a member the old loop parsed and the new loop does not:
- a string or number key, now skipped;
- a method whose name follows a modifier, now marked.

Fix the parser so the member parses as it did before. Don't re-pin the case.

- [ ] **Step 8: Commit.**

```bash
git add crates/kali_ast crates/kali_parser crates/kali_types crates/kali_cli
git commit -m "feat(class-instances): the parser keeps class fields, initializers, accessor/static/#private markers and block-arrow identity (A-2, A-7)"
```

---

### Task 3: The walker and the scopes

**Files:**
- Create: `crates/kali_types/src/class_instances/mod.rs` (only the module declarations for now)
- Create: `crates/kali_types/src/class_instances/walk.rs`, `walk_tests.rs`
- Create: `crates/kali_types/src/class_instances/scopes.rs`, `scopes_tests.rs`
- Modify: `crates/kali_types/src/lib.rs` (`pub mod class_instances;`)

**Interfaces:**
- Produces (`walk.rs`):

```rust
pub(crate) type FnKey = String;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FrameKind {
    Program,
    Function { is_async_or_generator: bool },
    /// An expression-bodied or block-bodied arrow (`FunctionExpression { is_arrow: true }`).
    Arrow,
    Method { class: String, method: String, is_static: bool },
    Constructor { class: String },
    /// The class's field initializers (they run with `this` bound to the instance).
    FieldInit { class: String },
}

#[derive(Clone, Debug)]
pub(crate) struct Frame { pub key: FnKey, pub kind: FrameKind }

/// The frames enclosing the node being visited, innermost last.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cx { pub frames: Vec<Frame> }

impl Cx {
    pub(crate) fn key(&self) -> &str;                 // innermost frame's key
    pub(crate) fn child_key(&self, name: &str) -> FnKey; // "<key>/<name>", or "<name>" at the program frame
    pub(crate) fn this_class(&self) -> Option<&str>;  // §3.2: skips Arrow frames; Method{is_static:false}/Constructor/FieldInit give the class
}

/// Where an expression sits in its parent (spec §3.3's allowlist).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Pos {
    BindingInit(String),
    BindingAssign(String),
    CallArg { callee: Expression, index: usize },
    Return,
    MemberObject { property: Option<String>, call: bool, write: bool },
    Discarded,
    Callee,
    NewCallee,
    AssignTarget,
    UpdateTarget,
    Other(&'static str),
}

pub(crate) trait Visitor {
    /// Before the walker descends into `expr`'s children. May replace `*expr`;
    /// the walker then descends into the replacement.
    fn expr(&mut self, _expr: &mut Expression, _pos: &Pos, _cx: &Cx) {}
    /// Before the walker descends into a statement list. May splice it.
    fn stmts(&mut self, _list: &mut Vec<Statement>, _cx: &Cx) {}
    fn enter_frame(&mut self, _cx: &Cx, _params: &[String]) {}
    fn class_decl(&mut self, _class: &ClassDeclaration, _exported_default: bool, _cx: &Cx) {}
    fn class_expr(&mut self, _class: &ClassExpression, _cx: &Cx) {}
    fn var_declarator(&mut self, _d: &VariableDeclarator, _kind: &str, _cx: &Cx) {}
    fn function_decl(&mut self, _f: &FunctionDeclaration, _cx: &Cx) {}
    fn catch_param(&mut self, _name: &str, _cx: &Cx) {}
    fn export_specifier(&mut self, _local: &str) {}
}

pub(crate) fn walk(statements: &mut Vec<Statement>, visitor: &mut dyn Visitor);
```

- Produces (`scopes.rs`):

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct BindingId { pub frame: FnKey, pub name: String }

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Resolved { Binding(BindingId), Ambiguous, Free }

pub(crate) struct Scopes { /* per frame: name → declaration count; per frame: params */ }
impl Scopes {
    pub(crate) fn build(statements: &mut Vec<Statement>) -> Scopes;
    pub(crate) fn resolve(&self, name: &str, cx: &Cx) -> Resolved;
    pub(crate) fn params(&self, frame: &str) -> &[String];
    /// Every identifier, declaration, parameter and class name the program spells.
    pub(crate) fn spelled(&self) -> &BTreeSet<String>;
}
```

**Walker rules.** Copy the exhaustive recursion of
`crates/kali_cli/src/build/name_anon_functions.rs:427-830` (the
`assign_names_*` functions). Keep every match arm, with no `_ =>`. Then make
these changes:

1. **Statement lists.** Before descending into any `Vec<Statement>` (a block
   body, a switch case's `consequent`, the program), call `visitor.stmts`,
   then iterate the list as it stands afterwards.
2. **`expr` hook.** For every expression, call `visitor.expr(expr, &pos, &cx)`
   first, then match on `*expr` (it may have been replaced) to descend.
3. **Frames.** Push a frame on the stack around each of the following, call
   `visitor.enter_frame(&cx, &params)` after the push, and pop it after:
   - `FunctionDeclaration`: key `cx.child_key(&f.name)`, kind `Function`.
   - `FunctionExpression`: key `cx.child_key(id or "<anon>")`, kind
     `Arrow` if `is_arrow`, else `Function`.
   - `ArrowFunctionExpression`: key `cx.child_key(id or "<anon>")`, kind
     `Arrow`.
   - each class method: key `cx.child_key(&format!("{class}#{method}"))`,
     kind `Constructor` if the name is `constructor` and it is not static,
     else `Method`.
   - the field initializers of a class, all in one frame: key
     `cx.child_key(&format!("{class}#fields"))`, kind `FieldInit`.

   The class name is `ClassDeclaration.name`, or for a `ClassExpression` its
   `id`, falling back to `"<anon>"`.
4. **Declaration hooks.** Call `visitor.class_decl` for every
   `Statement::ClassDeclaration`, and for `ExportDefault(ClassDeclaration)`
   with `exported_default = true`. Call `visitor.class_expr` for every
   `Expression::ClassExpression`. Call `visitor.var_declarator` for every
   declarator, including `for` / `for-in` / `for-of` heads. Call
   `visitor.function_decl` for every function declaration,
   `visitor.catch_param` for every catch parameter, and
   `visitor.export_specifier` for every `ExportNamed` specifier's `local`.
5. **Positions.** Pass `Pos` per this table. A child not listed gets
   `Other("an operand")`.

| parent → child | `Pos` |
|---|---|
| `ExpressionStatement.expression`, `ForInit::Expression`, `ForStatement.update` | `Discarded` |
| `VariableDeclarator.init` | `BindingInit(id)` |
| `ReturnStatement.argument`, an `ArrowFunctionExpression.body` | `Return` |
| `ThrowStatement.argument` | `Other("a thrown value")` |
| `If` / `While` / `DoWhile` / `For` test | `Other("a condition")` |
| `Switch` discriminant and case tests | `Other("a switch operand")` |
| `ForIn` / `ForOf` right; `ForIn` / `ForOf` left `Expression` | `Other("a loop iterable")`; `Other("a loop target")` |
| `With.object`, enum member value, `ExportDefault(Expression)` | `Other("a with object")`, `Other("an enum value")`, `Other("an exported value")` |
| `AssignmentExpression.left` | `AssignTarget` |
| `AssignmentExpression.right`, `operator == Assign`, `left` an `Identifier(n)` | `BindingAssign(n)` |
| `AssignmentExpression.right`, `operator == Assign`, `left` a member | `Other("the value of a field write")` |
| `AssignmentExpression.right`, any other operator | `Other("an operand")` |
| `UpdateExpression.argument` | `UpdateTarget` |
| `CallExpression.callee` | `Callee` |
| `CallExpression.args[i]` | `CallArg { callee: call.callee.clone(), index: i }` |
| `NewExpression.callee` | `NewCallee` |
| `NewExpression.args[i]` | `Other("an argument to `new`")` |
| `MemberExpression.object`, when the member's own pos is `Callee` | `MemberObject { property: member.property.clone(), call: true, write: false }` |
| `MemberExpression.object`, when the member's own pos is `AssignTarget` or `UpdateTarget` | `MemberObject { property, call: false, write: true }` |
| `MemberExpression.object`, otherwise | `MemberObject { property, call: false, write: false }` |
| `MemberExpression.computed_index` | `Other("a computed key")` |
| `ParenthesizedExpression`, `TypeAssertion`, `SatisfiesExpression`, `ChainExpression` inner | the parent's own pos (transparent) |
| `ArrayExpression` elements | `Other("an array element")` |
| `SpreadElement` / `RestElement` argument | `Other("a spread operand")` |
| `ObjectExpression` property values | `Other("an object-literal value")` |
| `TemplateLiteral` expressions; tagged template tag / expressions | `Other("a template operand")` |
| `BinaryExpression` operands | `Other("an operand of a binary operator")`, including `instanceof`, `===` and `in` |
| `UnaryExpression.argument` | `Other("an operand of a unary operator")`, including `typeof` and `delete` |
| `LogicalExpression` operands, `ConditionalExpression` parts | `Other("a logical or conditional operand")` |
| `SequenceExpression` items | `Other("a sequence operand")` |
| `AwaitExpression` / `YieldExpression` argument | `Other("an awaited or yielded value")` |
| `OptionalChainExpression` object | `Other("an optional-chain operand")` |
| class field initializer values | `Other("a field initializer")` |
| JSX expression containers and attributes | `Other("a JSX operand")` |
| `ImportExpression.source`, `DecoratedExpression` inner | `Other("an operand")` |

- [ ] **Step 1: Write the failing tests.** Create `class_instances/walk_tests.rs`:

```rust
use super::walk::{walk, Cx, FrameKind, Pos, Visitor};
use crate::test_support::parse_statements;
use kali_ast::Expression;

#[derive(Default)]
struct Record(Vec<(String, Pos, String)>);

impl Visitor for Record {
    fn expr(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        let label = match expr {
            Expression::Identifier(n) => n.clone(),
            Expression::ThisExpression => "this".into(),
            _ => return,
        };
        self.0.push((label, pos.clone(), cx.key().to_string()));
    }
}

fn record(src: &str) -> Vec<(String, Pos, String)> {
    let mut stmts = parse_statements(src);
    let mut r = Record::default();
    walk(&mut stmts, &mut r);
    r.0
}

#[test]
fn positions_follow_the_table() {
    let got = record("const s = a; s = b; f(c, d); o.m(e); p.f = q; r.n++; return2(t[u]);");
    let want = [
        ("a", Pos::BindingInit("s".into())),
        ("s", Pos::AssignTarget),
        ("b", Pos::BindingAssign("s".into())),
        ("f", Pos::Callee),
    ];
    for (i, (name, pos)) in want.iter().enumerate() {
        assert_eq!((&got[i].0, &got[i].1), (&name.to_string(), pos), "item {i}: {got:?}");
    }
    assert!(got.iter().any(|(n, p, _)| n == "c" && matches!(p, Pos::CallArg { index: 0, .. })));
    assert!(got.iter().any(|(n, p, _)| n == "o"
        && *p == Pos::MemberObject { property: Some("m".into()), call: true, write: false }));
    assert!(got.iter().any(|(n, p, _)| n == "p"
        && *p == Pos::MemberObject { property: Some("f".into()), call: false, write: true }));
    assert!(got.iter().any(|(n, p, _)| n == "r"
        && *p == Pos::MemberObject { property: Some("n".into()), call: false, write: true }));
    assert!(got.iter().any(|(n, p, _)| n == "u" && *p == Pos::Other("a computed key")));
}

#[test]
fn frames_key_methods_constructors_and_arrows() {
    let got = record(
        "class C { n = this.k; constructor(){ this.k = 1; } m(){ const f = () => { return this; }; return this; } }",
    );
    let keys: Vec<_> = got.iter().filter(|(n, _, _)| n == "this").map(|(_, _, k)| k.clone()).collect();
    assert_eq!(keys, ["C#fields", "C#constructor", "C#m/<anon>", "C#m"]);
}

#[test]
fn this_class_skips_arrows_but_not_functions() {
    struct ThisClass(Vec<Option<String>>);
    impl Visitor for ThisClass {
        fn expr(&mut self, expr: &mut Expression, _: &Pos, cx: &Cx) {
            if matches!(expr, Expression::ThisExpression) {
                self.0.push(cx.this_class().map(str::to_string));
            }
        }
    }
    let mut stmts = parse_statements(
        "class C { m(){ const a = () => { return this; }; const g = function(){ return this; }; return this; } static s(){ return this; } }",
    );
    let mut v = ThisClass(Vec::new());
    walk(&mut stmts, &mut v);
    assert_eq!(v.0, [Some("C".into()), None, Some("C".into()), None]);
}

#[test]
fn a_replacement_is_descended_into() {
    struct Replace(usize);
    impl Visitor for Replace {
        fn expr(&mut self, expr: &mut Expression, _: &Pos, _: &Cx) {
            if let Expression::Identifier(n) = expr {
                if n == "x" {
                    *expr = crate::test_support::parse_statements("[y, y];")
                        .into_iter()
                        .find_map(|s| match s {
                            kali_ast::Statement::ExpressionStatement(e) => Some(*e.expression),
                            _ => None,
                        })
                        .unwrap();
                } else if n == "y" {
                    self.0 += 1;
                }
            }
        }
    }
    let mut stmts = parse_statements("f(x);");
    let mut v = Replace(0);
    walk(&mut stmts, &mut v);
    assert_eq!(v.0, 2);
}

#[test]
fn frame_kinds_are_reported() {
    struct Kinds(Vec<FrameKind>);
    impl Visitor for Kinds {
        fn enter_frame(&mut self, cx: &Cx, _: &[String]) {
            self.0.push(cx.frames.last().unwrap().kind.clone());
        }
    }
    let mut stmts = parse_statements("async function f(){} class C { constructor(){} static s(){} }");
    let mut v = Kinds(Vec::new());
    walk(&mut stmts, &mut v);
    assert_eq!(v.0, [
        FrameKind::Function { is_async_or_generator: true },
        FrameKind::Constructor { class: "C".into() },
        FrameKind::Method { class: "C".into(), method: "s".into(), is_static: true },
    ]);
}
```

Create `class_instances/scopes_tests.rs`:

```rust
use super::scopes::{BindingId, Resolved, Scopes};
use super::walk::{walk, Cx, Pos, Visitor};
use crate::test_support::parse_statements;
use kali_ast::Expression;

fn resolutions(src: &str, name: &str) -> Vec<Resolved> {
    let mut stmts = parse_statements(src);
    let scopes = Scopes::build(&mut stmts);
    struct R<'a> { scopes: &'a Scopes, name: &'a str, out: Vec<Resolved> }
    impl Visitor for R<'_> {
        fn expr(&mut self, expr: &mut Expression, _: &Pos, cx: &Cx) {
            if matches!(expr, Expression::Identifier(n) if n == self.name) {
                self.out.push(self.scopes.resolve(self.name, cx));
            }
        }
    }
    let mut r = R { scopes: &scopes, name, out: Vec::new() };
    walk(&mut stmts, &mut r);
    r.out
}

fn binding(frame: &str, name: &str) -> Resolved {
    Resolved::Binding(BindingId { frame: frame.into(), name: name.into() })
}

#[test]
fn an_arrow_reaches_the_outer_binding() {
    let got = resolutions("const s = 1; const f = () => { s; };", "s");
    assert_eq!(got, [binding("", "s")]);
}

#[test]
fn a_parameter_shadows_the_outer_binding() {
    let got = resolutions("const s = 1; function g(s){ return s; } s;", "s");
    assert_eq!(got, [binding("g", "s"), binding("", "s")]);
}

#[test]
fn a_name_declared_twice_in_one_frame_is_ambiguous() {
    let got = resolutions("function g(){ { const s = 1; s; } { const s = 2; s; } }", "s");
    assert_eq!(got, [Resolved::Ambiguous, Resolved::Ambiguous]);
}

#[test]
fn an_undeclared_name_is_free() {
    assert_eq!(resolutions("console.log(1);", "console"), [Resolved::Free]);
}

#[test]
fn spelled_names_include_params_and_classes() {
    let mut stmts = parse_statements("class C { m(__this){ return q; } }");
    let scopes = Scopes::build(&mut stmts);
    assert!(scopes.spelled().contains("__this"));
    assert!(scopes.spelled().contains("C"));
    assert!(scopes.spelled().contains("q"));
}
```

Create `class_instances/mod.rs`:

```rust
//! Class instances (spec docs/superpowers/specs/2026-10-04-class-instances-design.md):
//! rewrites each in-slice program class to an object-literal factory and
//! `__this`-taking functions, and refuses every instance it cannot prove.

pub(crate) mod scopes;
pub(crate) mod walk;

#[cfg(test)]
#[path = "walk_tests.rs"]
mod walk_tests;
#[cfg(test)]
#[path = "scopes_tests.rs"]
mod scopes_tests;
```

Add `pub mod class_instances;` to `kali_types/src/lib.rs` next to `pub mod monomorphize;`.

- [ ] **Step 2: Run them to verify they fail.**

Run: `cargo test -p kali_types class_instances`
Expected: compile errors (the modules are empty).

- [ ] **Step 3: Implement `walk.rs`** per the walker rules above. The `Cx` methods:

```rust
impl Cx {
    pub(crate) fn key(&self) -> &str {
        self.frames.last().map(|f| f.key.as_str()).unwrap_or("")
    }
    pub(crate) fn child_key(&self, name: &str) -> FnKey {
        match self.key() {
            "" => name.to_string(),
            parent => format!("{parent}/{name}"),
        }
    }
    pub(crate) fn this_class(&self) -> Option<&str> {
        for frame in self.frames.iter().rev() {
            match &frame.kind {
                FrameKind::Arrow => continue,
                FrameKind::Method { class, is_static: false, .. }
                | FrameKind::Constructor { class }
                | FrameKind::FieldInit { class } => return Some(class),
                _ => return None,
            }
        }
        None
    }
}
```

`walk` starts with `Cx { frames: vec![Frame { key: String::new(), kind: FrameKind::Program }] }`.

- [ ] **Step 4: Implement `scopes.rs`.** `Scopes::build` runs `walk` with a visitor that records, for the frame `cx.key()`:
  - `var_declarator`: the declarator id;
  - `function_decl`: the function name;
  - `class_decl`: the class name;
  - `catch_param`: the parameter;
  - `enter_frame`: each parameter (in the new frame);
  - `expr`: every `Identifier` name, into `spelled`.

  Declarations go into a `BTreeMap<FnKey, BTreeMap<String, u32>>` of counts, with parameters stored separately in `BTreeMap<FnKey, Vec<String>>`. Every declared name also goes into `spelled`. `resolve` walks `cx.frames` from innermost to outermost:
  - the first frame whose declarations or parameters contain `name` gives `Binding { frame, name }`, or `Ambiguous` if its count is more than 1;
  - no frame gives `Free`.

  A parameter counts once, and a parameter plus a `var` of the same name counts as two, so it is `Ambiguous`.

- [ ] **Step 5: Run the tests.**

Run: `cargo test -p kali_types class_instances`
Expected: PASS.

- [ ] **Step 6: Commit.**

```bash
git add crates/kali_types/src/class_instances crates/kali_types/src/lib.rs
git commit -m "feat(class-instances): an exhaustive AST walker with positions and frames, and frame scopes"
```

---

### Task 4: Class plans — which classes are rewritten, their field sets, and the `new` refusals

**Files:**
- Create: `crates/kali_types/src/class_instances/classes.rs`, `classes_tests.rs`
- Modify: `crates/kali_types/src/program_classes.rs` (add `is_ambiguous`)
- Modify: `crates/kali_types/src/class_instances/mod.rs` (declare the module and its test file)

**Interfaces:**
- Consumes: `walk`, `Cx`, `Scopes::spelled` (Task 3); the messages (Task 1); `ProgramClasses::collect`, `host_derived`, and the new `is_ambiguous`.
- Produces:

```rust
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RewrittenClass {
    pub name: String,
    /// The instance field set, in first-binding order (spec §3.2).
    pub fields: Vec<String>,
    /// Instance method names, `constructor` excluded.
    pub methods: BTreeSet<String>,
    /// How many leading constructor statements are `this.f = e` run members (spec §3.2 step 2).
    pub leading_run: usize,
    /// The constructor's parameters (empty without a constructor).
    pub ctor_params: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct ClassPlans {
    pub rewritten: BTreeMap<String, RewrittenClass>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ClassPlans {
    pub(crate) fn method_owner(&self, method: &str) -> Option<&str>; // first rewritten class (by name) with that method
}

pub(crate) fn plan_classes(statements: &mut Vec<Statement>, spelled: &BTreeSet<String>) -> ClassPlans;

/// `new X(a)` in either parse shape (A-9): `callee: Identifier(X)`, or
/// `callee` a member/call chain whose deepest node is `Call(Identifier(X), a)`
/// with `args` empty. Returns `X`.
pub(crate) fn new_target(new: &NewExpression) -> Option<&str>;
```

**The rules (spec A-1, A-8, §3.2, §3.4).** Collect, with one `walk`:
- every `ClassDeclaration`, with its body, whether it is `export default`, and the frame key it is declared in;
- every `ClassExpression` name (its `id`, and the declarator id when it is a declarator's initializer);
- every `export { … }` local name;
- every `new` target (`new_target`), as the set `constructed`;
- every `FunctionDeclaration` name.

Then, for each class name `C` declared by a `ClassDeclaration` and not host-derived:
1. **Chain.** `C` plus every program class it extends, or that extends it, transitively, through `super_class` names that are program classes.
2. **Out of slice.** `C` is out of slice if any of these holds. The reason is the first that holds, in this order:
   - its chain has more than one class (`CLASS_REASON_EXTENDS`);
   - it has a `Get` / `Set` method (`ACCESSOR`);
   - it has an `is_static` method or field, or `has_static_block` (`STATIC`);
   - `has_private_members` (`PRIVATE`);
   - `has_computed_members` (`COMPUTED`);
   - it is exported (`EXPORTED`);
   - `ProgramClasses::is_ambiguous(C)` (`AMBIGUOUS`).

   A `ClassExpression` name is always out of slice (`EXPRESSION`).
3. **Stateful.** An out-of-slice class is stateful when any class in its
   chain has:
   - a field (`fields` non-empty, static fields included);
   - a method named `constructor`;
   - a `Get` / `Set` method;
   - `has_private_members`;
   - a `ThisExpression` anywhere in a method body, nested arrows and
     functions included.
   
   A stateful out-of-slice class in `constructed` pushes one diagnostic,
   `class_construction_unavailable_message(C, reason)`. A stateless one, or
   one never constructed, pushes nothing and is not rewritten.
4. **Rewritten.** An in-slice class in `constructed` is rewritten. Compute
   its plan:
   - `fields`. First take the non-static `fields` in order. Then the leading
     run: from the constructor body's first statement, while the statement
     is an `ExpressionStatement` of `AssignmentExpression { operator: Assign,
     left: Member(ThisExpression, static name f), right }`, and `right`
     contains no `ThisExpression` other than `this.g` for a `g` already
     bound, add `f` (once) and count the statement. `leading_run` is the
     count.
   - A declared field with `value: None` that the run does not bind pushes
     `class_field_without_initial_value_message(C, f)`.
   - A field initializer that contains a `ThisExpression` other than `this.g`
     for a `g` bound before it pushes
     `class_field_initializer_this_message(C, f)`.
   - A `ReturnStatement` with an argument anywhere in the constructor body
     (not inside nested functions or arrows) pushes
     `constructor_return_unavailable_message()`.
   - `methods`: the non-static method names other than `constructor`.
   - `ctor_params`: the constructor's params.
5. **Plain functions.** For every `new` target that is a `FunctionDeclaration`
   name and not a class name, push
   `plain_function_construction_unavailable_message(name)`.
6. **Collisions (A-3).** For each rewritten class, each of `C__new`,
   `C__<m>` for each method, `__this`, and `__f_<f>` for each field that is
   in `spelled` pushes
   `class_generated_name_collision_message(name, C)`.

Every diagnostic is `Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message)`.

Add to `program_classes.rs`:

```rust
    pub(crate) fn is_ambiguous(&self, name: &str) -> bool {
        self.ambiguous.contains(name)
    }
```

- [ ] **Step 1: Write the failing tests.** Create `class_instances/classes_tests.rs`:

```rust
use super::classes::{new_target, plan_classes, RewrittenClass};
use super::scopes::Scopes;
use crate::test_support::parse_statements;

fn plan(src: &str) -> super::classes::ClassPlans {
    let mut stmts = parse_statements(src);
    let scopes = Scopes::build(&mut stmts);
    plan_classes(&mut stmts, scopes.spelled())
}

fn messages(src: &str) -> Vec<String> {
    plan(src).diagnostics.into_iter().map(|d| d.message).collect()
}

#[test]
fn a_constructed_plain_class_is_rewritten_with_its_field_set() {
    let p = plan("class C { n = 0; constructor(v){ this.k = v; this.m = this.n + v; log(this); this.n = 1; } add(x){} } new C(1);");
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(p.rewritten["C"], RewrittenClass {
        name: "C".into(),
        fields: vec!["n".into(), "k".into(), "m".into()],
        methods: ["add".to_string()].into_iter().collect(),
        leading_run: 2,
        ctor_params: vec!["v".into()],
    });
}

#[test]
fn a_class_never_constructed_is_not_rewritten() {
    assert!(plan("class C { constructor(){ this.n = 1; } }").rewritten.is_empty());
}

#[test]
fn a_stateful_chain_refuses_and_a_stateless_chain_is_untouched() {
    let stateful = messages("class A{ constructor(){ this.n=1; } } class B extends A{ g(){ return this.n; } } new B();");
    assert_eq!(stateful, [kali_common::class_construction_unavailable_message("B", kali_common::CLASS_REASON_EXTENDS)]);
    let stateless = plan("class A{ f(){return 4;} } class B extends A{} new B();");
    assert!(stateless.diagnostics.is_empty());
    assert!(stateless.rewritten.is_empty(), "a base class is never rewritten (A-1)");
}

#[test]
fn a_getter_refuses_and_a_static_only_class_is_untouched() {
    assert_eq!(messages("class A{ get v(){ return 3; } } new A();"),
        [kali_common::class_construction_unavailable_message("A", kali_common::CLASS_REASON_ACCESSOR)]);
    let statics = plan("class U { static twice(x){ return 2*x; } } U.twice(4);");
    assert!(statics.diagnostics.is_empty() && statics.rewritten.is_empty());
}

#[test]
fn an_ambiguous_class_name_refuses_when_stateful() {
    let got = messages("function a(){ class P{ constructor(){ this.n=1; } } return new P(); } function b(){ class P{ constructor(){ this.n=2; } } return new P(); }");
    assert_eq!(got, [kali_common::class_construction_unavailable_message("P", kali_common::CLASS_REASON_AMBIGUOUS)]);
}

#[test]
fn field_rules_refuse() {
    assert_eq!(messages("class C{ n; } new C();"), [kali_common::class_field_without_initial_value_message("C", "n")]);
    assert_eq!(messages("class C{ constructor(){ this.n=1; return {n:2}; } } new C();"), [kali_common::constructor_return_unavailable_message().to_string()]);
    assert_eq!(messages("class C{ n = this.k; constructor(){ this.k = 1; } } new C();"), [kali_common::class_field_initializer_this_message("C", "n")]);
}

#[test]
fn constructing_a_plain_function_refuses() {
    assert_eq!(messages("function Box(v){ this.v=v; } new Box(9);"), [kali_common::plain_function_construction_unavailable_message("Box")]);
}

#[test]
fn a_spelled_generated_name_refuses() {
    assert_eq!(messages("class C { m(){} } const __this = 1; new C();"), [kali_common::class_generated_name_collision_message("__this", "C")]);
    assert_eq!(messages("class C { m(){} } function C__m(){} new C();"), [kali_common::class_generated_name_collision_message("C__m", "C")]);
}

#[test]
fn new_target_reads_both_parse_shapes() {
    let stmts = parse_statements("new C(1); new C(1).m(2); new C().v;");
    let targets: Vec<_> = stmts.iter().map(|s| match s {
        kali_ast::Statement::ExpressionStatement(e) => match e.expression.as_ref() {
            kali_ast::Expression::NewExpression(n) => new_target(n).map(str::to_string),
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }).collect();
    assert_eq!(targets, [Some("C".into()), Some("C".into()), Some("C".into())]);
}
```

Declare `pub(crate) mod classes;` and its test file in `mod.rs` the same way as Task 3's modules.

- [ ] **Step 2: Run them to verify they fail.**

Run: `cargo test -p kali_types class_instances::classes`
Expected: compile errors.

- [ ] **Step 3: Implement `classes.rs`** per the rules above. Write `new_target` like this:

```rust
pub(crate) fn new_target(new: &NewExpression) -> Option<&str> {
    if let Expression::Identifier(name) = &new.callee {
        return Some(name);
    }
    if !new.args.is_empty() {
        return None;
    }
    let mut node = &new.callee;
    loop {
        match node {
            Expression::MemberExpression(m) => node = &m.object,
            Expression::CallExpression(c) => match &c.callee {
                Expression::Identifier(name) => return Some(name),
                inner => node = inner,
            },
            _ => return None,
        }
    }
}
```

"Contains a `ThisExpression`" uses a small read-only `Visitor` over a cloned
expression or block (`walk` needs `&mut Vec<Statement>`; wrap the expression
in `vec![Statement::ExpressionStatement(…)]`). That visitor also records each
`this.g` member read, so the `this.g`-only test can check those reads against
the fields bound so far.

- [ ] **Step 4: Run the tests.**

Run: `cargo test -p kali_types class_instances::classes && cargo test -p kali_types program_classes`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_types/src/class_instances crates/kali_types/src/program_classes.rs
git commit -m "feat(class-instances): class plans — rewritten classes, field sets, stateful out-of-slice and plain-function refusals (A-1, A-3, A-8)"
```

---

### Task 5: Instance provenance

**Files:**
- Create: `crates/kali_types/src/class_instances/provenance.rs`, `provenance_tests.rs`
- Modify: `crates/kali_types/src/class_instances/mod.rs`

**Interfaces:**
- Consumes: `walk`, `Cx`, `Pos`, `Visitor` (Task 3); `Scopes`, `BindingId`, `Resolved` (Task 3); `ClassPlans`, `new_target` (Task 4).
- Produces:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Val { Bottom, Inst(String), NotInst, Unknown }
impl Val { pub(crate) fn join(&self, other: &Val) -> Val; }

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Src {
    Inst(String),
    NotInst,
    Unknown,
    Binding(BindingId),
    Call(FnKey),
    MethodCall(Box<Src>, String),
    Join(Vec<Src>),
}

/// A1-9: rewrites `new (C(a).m())` / `new (C(a).f)` / `new (C(a))` to the
/// canonical `new C(a)` at the chain's root, for rewritten `C` only.
pub(crate) fn reassociate_new(statements: &mut Vec<Statement>, plans: &ClassPlans);

/// Abstracts an expression to a source (spec §3.3 rules 1-5).
pub(crate) fn abstract_expr(e: &Expression, cx: &Cx, env: &Env) -> Src;

pub(crate) struct Env<'a> {
    pub scopes: &'a Scopes,
    pub plans: &'a ClassPlans,
    /// Binding of a callable name → the function's frame key, params, and async/generator flag.
    pub functions: BTreeMap<BindingId, FnInfo>,
}

#[derive(Clone, Debug)]
pub(crate) struct FnInfo { pub key: FnKey, pub params: Vec<String>, pub is_async_or_generator: bool }

pub(crate) struct Provenance { /* values, returns */ }
impl Provenance {
    pub(crate) fn solve(statements: &mut Vec<Statement>, env: &Env) -> Provenance;
    pub(crate) fn eval(&self, src: &Src, env: &Env) -> Val;
    pub(crate) fn binding(&self, id: &BindingId) -> Val;
    pub(crate) fn returns(&self, key: &str) -> Val;
}

/// Builds `Env.functions` (function declarations, and `const` declarators whose
/// initializer is a function or arrow expression; methods are keyed `C#m` under
/// the class's frame and recorded in `Env::method_key`).
pub(crate) fn build_env<'a>(statements: &mut Vec<Statement>, scopes: &'a Scopes, plans: &'a ClassPlans) -> Env<'a>;
impl Env<'_> {
    pub(crate) fn method_key(&self, class: &str, method: &str) -> Option<&FnKey>;
}
```

**Lattice.** `Bottom ⊔ x = x`; `Inst(C) ⊔ Inst(C) = Inst(C)`; `NotInst ⊔ NotInst = NotInst`; every other pair gives `Unknown`.

**`abstract_expr`:**
- **Transparent wrappers.** `ParenthesizedExpression`, `TypeAssertion`,
  `SatisfiesExpression` and `ChainExpression` abstract to their inner
  expression.
- **`new`.** `NewExpression` in canonical form, with `callee: Identifier(C)`
  and `C` rewritten, gives `Inst(C)`. Any other `new` gives `NotInst`.
- **`this`.** `ThisExpression` gives `Inst(C)` when `cx.this_class() ==
  Some(C)` and `C` is rewritten, and `NotInst` otherwise.
- **`Identifier(n)`**, by `scopes.resolve(n, cx)`:
  - `Binding(id)` with `id` in `env.functions` gives `NotInst` (a function
    value);
  - `Binding(id)` otherwise gives `Src::Binding(id)`;
  - `Ambiguous` gives `Unknown`;
  - `Free` gives `NotInst`.
- **Calls.** `CallExpression` with:
  - an `Identifier` callee resolving to a binding in `env.functions` gives
    `Call(key)`, or `NotInst` when `is_async_or_generator`;
  - a member callee with a static name `m` gives
    `MethodCall(abstract(object), m)`;
  - anything else gives `Unknown`.
- **Value-of-assignment forms.** `AssignmentExpression` gives
  `abstract(right)`. `SequenceExpression` gives `abstract(last)`.
- **Branches.** `ConditionalExpression` gives `Join([cons, alt])`.
  `LogicalExpression` gives `Join([left, right])`.
- **Non-instances.** All of these give `NotInst`:
  - `Literal`, `TemplateLiteral`, `ArrayExpression`, `ObjectExpression`;
  - function, arrow and class expressions;
  - `BinaryExpression`, `UnaryExpression`, `UpdateExpression`;
  - `MemberExpression`: an instance can never be stored in a field or an
    element, because §3.3's allowlist refuses it;
  - `AwaitExpression`, `YieldExpression`, `BigIntLiteral`.
- **Everything else** gives `Unknown`.

**Facts and the fixpoint.** `solve` runs one `walk` collecting:
- **binding sources:**
  - a declarator `id = init` adds `abstract(init)`, or `NotInst` when there
    is no initializer;
  - `BindingAssign(n)` adds `abstract(right)`;
  - a compound assignment or update on an `Identifier` binding adds
    `NotInst`;
  - a `for`-in/of declarator or a catch parameter adds `NotInst`;
- **parameter sources:**
  - a `CallExpression` whose callee resolves to an `env.functions` binding
    adds `abstract(arg_i)` to `BindingId { frame: key, name: params[i] }`
    for each param (a missing argument adds `NotInst`);
  - a canonical `new C(args)` of a rewritten `C` adds the arguments to the
    constructor's params, `BindingId { frame: method_key(C, "constructor"),
    … }`;
- **method call sites:** a `CallExpression` with a static member callee
  `o.m(args)` records `(abstract(o), m, args.map(abstract))`;
- **returns:**
  - `Return` positions add `abstract(argument)` to `cx.key()`'s returns;
  - an argument-less `return` adds `NotInst`;
  - an expression-bodied arrow's body adds to the arrow's returns;
  - a frame whose body's last statement is not a `ReturnStatement` or
    `ThrowStatement` adds `NotInst` (fall-through);
- **escapes:** an `Identifier` resolving to an `env.functions` binding in
  any `Pos` other than `Callee`, plus every `export_specifier` local that
  resolves to one, marks that function escaped.

`solve` then iterates until nothing changes:
- every binding gets the `join` of `eval` over its sources;
- every method-call site whose receiver evaluates to `Inst(C)`, with `m` in
  `C`'s methods, contributes its arguments to `C#m`'s parameters;
- every parameter of an escaped function is `Unknown`;
- every return value is the `join` of `eval` over the frame's return
  sources, and is forced to `Unknown` for async and generator frames.

`eval`:
- `Binding(id)` gives `values[id]`, defaulting to `Bottom`;
- `Call(key)` gives `returns[key]`;
- `MethodCall(recv, m)`:
  - `Inst(C)` with `m` in `C`'s methods gives `returns[method_key(C, m)]`;
  - `Inst(_)` otherwise, and `NotInst`, give `NotInst`;
  - `Bottom` stays `Bottom`, and `Unknown` stays `Unknown`;
- `Join` gives the fold of `join`.

Each iteration recomputes values from scratch with the current values as
input. `eval` is monotone and the lattice has height 3, so the loop
terminates.

- [ ] **Step 1: Write the failing tests.** Create `class_instances/provenance_tests.rs`:

```rust
use super::classes::plan_classes;
use super::provenance::{build_env, reassociate_new, Provenance, Val};
use super::scopes::{BindingId, Scopes};
use crate::test_support::parse_statements;

fn solve(src: &str) -> (Provenance, Vec<kali_ast::Statement>) {
    let mut stmts = parse_statements(src);
    let spelled = Scopes::build(&mut stmts).spelled().clone();
    let plans = plan_classes(&mut stmts, &spelled);
    reassociate_new(&mut stmts, &plans);
    let scopes = Scopes::build(&mut stmts);
    let env = build_env(&mut stmts, &scopes, &plans);
    (Provenance::solve(&mut stmts, &env), stmts)
}

fn b(frame: &str, name: &str) -> BindingId {
    BindingId { frame: frame.into(), name: name.into() }
}

#[test]
fn bindings_parameters_and_returns_resolve() {
    let (p, _) = solve(
        "class C { constructor(v){ this.v = v; } } function mk(v){ return new C(v); } function use(x){ return x.v; } const c = mk(1); use(c);",
    );
    assert_eq!(p.binding(&b("", "c")), Val::Inst("C".into()));
    assert_eq!(p.binding(&b("use", "x")), Val::Inst("C".into()));
    assert_eq!(p.returns("mk"), Val::Inst("C".into()));
}

#[test]
fn two_classes_into_one_parameter_is_unknown() {
    let (p, _) = solve(
        "class A{ constructor(){ this.n=1; } } class B{ constructor(){ this.n=2; } } function f(x){ return x.n; } f(new A()); f(new B());",
    );
    assert_eq!(p.binding(&b("f", "x")), Val::Unknown);
}

#[test]
fn an_escaped_function_has_unknown_parameters() {
    let (p, _) = solve("class C{ constructor(){ this.n=1; } } function f(x){ return x.n; } const g = f; g(new C());");
    assert_eq!(p.binding(&b("f", "x")), Val::Unknown);
}

#[test]
fn a_recursive_method_returning_this_converges() {
    let (p, _) = solve(
        "class C { constructor(){ this.n = 0; } add(x){ this.n = this.n + x; if (x > 0) { return this.add(x - 1); } return this; } } const c = new C(); const d = c.add(3);",
    );
    assert_eq!(p.binding(&b("", "d")), Val::Inst("C".into()));
}

#[test]
fn method_arguments_flow_to_method_parameters() {
    let (p, _) = solve(
        "class P{ constructor(){ this.x=1; } } class C{ constructor(){ this.n=0; } take(p){ return p.x; } } const c = new C(); c.take(new P());",
    );
    assert_eq!(p.binding(&b("C#take", "p")), Val::Inst("P".into()));
}

#[test]
fn reassociation_puts_new_at_the_chain_root() {
    use kali_ast::{Expression, Statement};
    let (_, stmts) = solve("class S { m(v){ return v + 1; } } new S().m(1);");
    let Statement::ExpressionStatement(stmt) = &stmts[1] else { panic!("{:?}", stmts[1]) };
    let Expression::CallExpression(call) = stmt.expression.as_ref() else { panic!("{stmt:?}") };
    assert_eq!(call.args.len(), 1);
    let Expression::MemberExpression(member) = &call.callee else { panic!("{call:?}") };
    assert_eq!(member.property.as_deref(), Some("m"));
    let Expression::NewExpression(new) = &member.object else { panic!("{member:?}") };
    assert_eq!(new.callee, Expression::Identifier("S".into()));
    assert!(new.args.is_empty());
}

#[test]
fn a_captured_instance_resolves_inside_an_arrow() {
    let (p, _) = solve("class C{ constructor(){ this.n=0; } add(x){} } const s = new C(); const f = () => { s.add(4); }; f();");
    assert_eq!(p.binding(&b("", "s")), Val::Inst("C".into()));
}
```

- [ ] **Step 2: Run them to verify they fail.**

Run: `cargo test -p kali_types class_instances::provenance`
Expected: compile errors.

- [ ] **Step 3: Implement `provenance.rs`** per the rules above.
`reassociate_new` is a `Visitor` whose `expr` hook does this:

```rust
fn expr(&mut self, expr: &mut Expression, _: &Pos, _: &Cx) {
    let Expression::NewExpression(new) = expr else { return };
    if !new.args.is_empty() || matches!(new.callee, Expression::Identifier(_)) {
        return;
    }
    let Some(class) = new_target(new).map(str::to_string) else { return };
    if !self.plans.rewritten.contains_key(&class) {
        return;
    }
    let mut chain = std::mem::replace(&mut new.callee, Expression::Identifier(String::new()));
    // Find the deepest Call(Identifier(class), args) and replace it with new class(args).
    fn root(e: &mut Expression) -> &mut Expression {
        match e {
            Expression::MemberExpression(m) => root(&mut m.object),
            Expression::CallExpression(c) if !matches!(c.callee, Expression::Identifier(_)) => root(&mut c.callee),
            _ => e,
        }
    }
    let slot = root(&mut chain);
    if let Expression::CallExpression(call) = slot {
        let args = std::mem::take(&mut call.args);
        *slot = Expression::NewExpression(Box::new(NewExpression {
            callee: Expression::Identifier(class),
            args,
        }));
    }
    *expr = chain;
}
```

- [ ] **Step 4: Run the tests.**

Run: `cargo test -p kali_types class_instances::provenance`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_types/src/class_instances
git commit -m "feat(class-instances): instance provenance — new re-association (A-9), abstraction and the whole-program fixpoint"
```

---

### Task 6: Uses — the allowlist, and the `new` / method-call / compound rewrites

**Files:**
- Create: `crates/kali_types/src/class_instances/uses.rs`, `uses_tests.rs`
- Modify: `crates/kali_types/src/class_instances/mod.rs`

**Interfaces:**
- Consumes: Tasks 3 to 5.
- Produces: `pub(crate) fn check_and_rewrite(statements: &mut Vec<Statement>, env: &Env, prov: &Provenance) -> Vec<Diagnostic>`

**Rules.** A `Visitor` whose `expr` hook does steps 1 to 4 below, in order,
for every expression. Each refusal pushes one
`Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message)`. A refusal never
stops the walk. Nothing is rewritten at a node that refused.

1. **The class as a value.** An `Identifier(C)` that resolves to the binding
   of a rewritten class's declaration, in a `Pos` other than `NewCallee`,
   pushes `class_value_message(C)`.
2. **Instances in position.** Let `v = prov.eval(&abstract_expr(expr, cx,
   env), env)`. If `v == Inst(C)`, check `pos`:
   - **`BindingInit(n)` / `BindingAssign(n)`.** OK if the binding `n`
     resolves to has value `Inst(C)`. Otherwise push
     `class_instance_mixed_message(C, "binding `n`")`.
   - **`CallArg { callee, index }`:**
     - If `callee` is an `Identifier` resolving to an `env.functions`
       binding: when `index < params.len()`, OK if that parameter's binding
       is `Inst(C)`, else the mixed message with "parameter `p` of `f`".
       When `index >= params.len()`, refuse with the position message "an
       extra argument".
     - If `callee` is `Identifier(name)` where `name` is a generated method
       name `D__m` (from the rewrites in step 4): index 0 is OK when
       `D == C`. For index `i ≥ 1`, check `D#m`'s parameter `i-1` in the same
       way.
     - If `callee` is `Identifier("D__new")`: check `D#constructor`'s
       parameter `index`.
     - If `callee` is a member `o.m` whose `o` evaluates to `Inst(D)` with
       `m` in `D`'s methods: check `D#m`'s parameter `index`.
     - Otherwise push the position message with "an argument to a call kali
       cannot resolve to a program function".
   - **`Return`.** OK if `prov.returns(cx.key()) == Inst(C)`. Otherwise
     push the mixed message with "the return value of `cx.key()`".
   - **`MemberObject { property: None, .. }`.** Push the position message
     with "the object of a computed member access".
   - **`MemberObject { property: Some(m), call: true, .. }`.** OK if `m` is
     a method or a field of `C` (A-5). Otherwise push
     `unresolved_member_call_unavailable_message(m)`.
   - **`MemberObject { property: Some(f), write: true, .. }`.** OK if `f` is
     in `C`'s fields. Otherwise push
     `class_field_outside_set_message(C, f)`.
   - **`MemberObject { property: Some(f), call: false, write: false }`.** OK
     if `f` is a field. If `f` is a method, push
     `class_method_value_message(C, f)`. Otherwise push
     `class_field_undeclared_read_message(C, f)`.
   - **`Discarded`.** OK.
   - **`Callee`, `NewCallee`, `AssignTarget`, `UpdateTarget`.** OK. The
     target of `s = …` is the binding, not a use of the value.
   - **`Other(position)`.** Push
     `class_instance_position_message(C, position)`.
3. **Unresolved receivers.** If `pos` is `MemberObject { property: Some(m),
   call: true, .. }`, and `v` is neither `Inst(_)` nor `NotInst`, and
   `env.plans.method_owner(m)` is `Some(D)`, push
   `class_receiver_unresolved_message(m, D)`.
4. **Rewrites** (only if steps 1 to 3 pushed nothing at this node):
   - **`new`.** A canonical `NewExpression { callee: Identifier(C), args }`
     of a rewritten `C` becomes `CallExpression { callee:
     Identifier("C__new"), args }`.
   - **Method calls.** A `CallExpression { callee: Member(o, m), args }`
     with a static name, where `o` evaluates to `Inst(C)` and `m` is in
     `C`'s methods, becomes `CallExpression { callee: Identifier("C__m"),
     args: [o, args…] }`.
   - **Compound assignment.** An `AssignmentExpression` whose operator is
     one of the twelve arithmetic and bitwise compound operators, whose
     `left` is `Member(o, f)` with a static name, where `o` is `Identifier`
     or `ThisExpression` evaluating to `Inst(_)`, becomes
     `AssignmentExpression { operator: Assign, left: left.clone(), right:
     Binary { operator: op_str, left: left.clone(), right } }`. The
     operator strings are `+ - * / % ** << >> >>> & | ^`.
   - **Updates.** An `UpdateExpression` in `Pos::Discarded` with the same
     receiver condition becomes `AssignmentExpression { Assign, left: arg,
     right: Binary { "+" or "-", arg, Literal(Number(1.0)) } }`.

Recording the generated names: `check_and_rewrite` keeps a
`BTreeMap<String, (String, String)>` from `C__m` to `(C, m)`, filled before
the walk from `env.plans`, for the `CallArg` rule.

- [ ] **Step 1: Write the failing tests.** Create `class_instances/uses_tests.rs`:

```rust
use super::classes::plan_classes;
use super::provenance::{build_env, reassociate_new, Provenance};
use super::scopes::Scopes;
use super::uses::check_and_rewrite;
use crate::test_support::parse_statements;

fn run(src: &str) -> (Vec<kali_ast::Statement>, Vec<String>) {
    let mut stmts = parse_statements(src);
    let spelled = Scopes::build(&mut stmts).spelled().clone();
    let plans = plan_classes(&mut stmts, &spelled);
    reassociate_new(&mut stmts, &plans);
    let scopes = Scopes::build(&mut stmts);
    let env = build_env(&mut stmts, &scopes, &plans);
    let prov = Provenance::solve(&mut stmts, &env);
    let diags = check_and_rewrite(&mut stmts, &env, &prov);
    (stmts, diags.into_iter().map(|d| d.message).collect())
}

fn tail(src: &str, n: usize) -> Vec<kali_ast::Statement> {
    let stmts = parse_statements(src);
    stmts[stmts.len() - n..].to_vec()
}

#[test]
fn new_and_method_calls_are_rewritten() {
    let (stmts, diags) = run("class C { constructor(v){ this.n = v; } add(x){ this.n = this.n + x; } } const s = new C(1); s.add(2);");
    assert!(diags.is_empty(), "{diags:?}");
    assert_eq!(stmts[1..].to_vec(), tail("const s = C__new(1); C__add(s, 2);", 2));
}

#[test]
fn this_method_calls_and_compound_assignments_are_rewritten_inside_the_class() {
    let (stmts, diags) = run("class C { constructor(){ this.n = 0; } a(){ this.n += 2; this.n++; return this.b(); } b(){ return 1; } } new C().a();");
    assert!(diags.is_empty(), "{diags:?}");
    let expected = parse_statements("class C { constructor(){ this.n = 0; } a(){ this.n = this.n + 2; this.n = this.n + 1; return C__b(this); } b(){ return 1; } } C__a(C__new());");
    assert_eq!(stmts, expected);
}

#[test]
fn same_named_methods_dispatch_per_class() {
    let (stmts, diags) = run("class A{ f(){ return 1; } } class B{ f(){ return 2; } } const a=new A(); const b=new B(); a.f(); b.f();");
    assert!(diags.is_empty(), "{diags:?}");
    assert_eq!(stmts[2..].to_vec(), tail("const a=A__new(); const b=B__new(); A__f(a); B__f(b);", 4));
}

#[test]
fn positions_outside_the_allowlist_refuse() {
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); console.log(c);");
    assert_eq!(d, [kali_common::class_instance_position_message("C", "an argument to a call kali cannot resolve to a program function")]);
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const xs=[new C()];");
    assert_eq!(d, [kali_common::class_instance_position_message("C", "an array element")]);
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); c instanceof C;");
    assert!(d.contains(&kali_common::class_instance_position_message("C", "an operand of a binary operator")), "{d:?}");
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
}

#[test]
fn field_rules_refuse_at_the_use() {
    let (_, d) = run("class C{ constructor(){ this.n=0; } set(){ this.m=1; } } new C().set();");
    assert_eq!(d, [kali_common::class_field_outside_set_message("C", "m")]);
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); c.zz;");
    assert_eq!(d, [kali_common::class_field_undeclared_read_message("C", "zz")]);
    let (_, d) = run("class C{ constructor(){ this.n=1; } get(){ return 1; } } const c=new C(); const m=c.get;");
    assert!(d.contains(&kali_common::class_method_value_message("C", "get")), "{d:?}");
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); c.zork();");
    assert_eq!(d, [kali_common::unresolved_member_call_unavailable_message("zork")]);
}

#[test]
fn mixed_and_unresolved_receivers_refuse() {
    let (_, d) = run("class A{ constructor(){ this.n=1; } } class B{ constructor(){ this.n=2; } } function f(x){ return x.n; } f(new A()); f(new B());");
    assert!(d.contains(&kali_common::class_instance_mixed_message("A", "parameter `x` of `f`")), "{d:?}");
    let (_, d) = run("class C{ constructor(){ this.n=1; } get(){ return this.n; } } function f(x){ return x.get(); } const g=f; g(new C());");
    assert!(d.contains(&kali_common::class_receiver_unresolved_message("get", "C")), "{d:?}");
}

#[test]
fn an_array_push_next_to_a_user_push_is_left_alone() {
    let (stmts, d) = run("class Stack{ constructor(){ this.n=0; } push(v){ this.n=this.n+v; } } const s=new Stack(); const a=[1]; a.push(2); s.push(5);");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(stmts[1..].to_vec(), tail("const s=Stack__new(); const a=[1]; a.push(2); Stack__push(s, 5);", 4));
}

#[test]
fn an_update_in_value_position_is_left_for_the_lane_to_refuse() {
    let (stmts, d) = run("class C{ constructor(){ this.n=0; } inc(){ return this.n++; } } new C().inc();");
    assert!(d.is_empty(), "{d:?}");
    let expected = parse_statements("class C{ constructor(){ this.n=0; } inc(){ return this.n++; } } C__inc(C__new());");
    assert_eq!(stmts, expected);
}
```

- [ ] **Step 2: Run them to verify they fail.**

Run: `cargo test -p kali_types class_instances::uses`
Expected: compile errors.

- [ ] **Step 3: Implement `uses.rs`** per the rules.

- [ ] **Step 4: Run the tests.**

Run: `cargo test -p kali_types class_instances`
Expected: PASS.

If an `assert_eq!` on statements fails only because the parser builds a
different shape for the expected source (for example a
`ParenthesizedExpression`), fix the expected source, not the rewrite. Any
other difference is a bug in the rewrite.

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_types/src/class_instances
git commit -m "feat(class-instances): the use allowlist and the new / method-call / compound rewrites"
```

---

### Task 7: Translate the classes, sweep, and the entry point

**Files:**
- Create: `crates/kali_types/src/class_instances/translate.rs`, `translate_tests.rs`
- Modify: `crates/kali_types/src/class_instances/mod.rs` (entry and sweep), add `mod_tests.rs`
- Modify: `crates/kali_types/src/lib.rs` (`pub use repr_infer::infer_reprs;`)

**Interfaces:**
- Consumes: Tasks 3 to 6.
- Produces:

```rust
// translate.rs
pub(crate) fn translate_classes(statements: &mut Vec<Statement>, plans: &ClassPlans);

// mod.rs
pub struct ClassRewrite { pub changed: bool, pub diagnostics: Vec<Diagnostic> }
pub fn rewrite_class_instances(statements: &mut Vec<Statement>) -> ClassRewrite;
```

**`translate_classes`.** This is a `Visitor` whose `stmts` hook replaces each
`Statement::ClassDeclaration` of a rewritten class with, in order:

1. **The factory.** `FunctionDeclaration { name: "C__new", params:
   plan.ctor_params, body, is_async: false, generator: false }`. Its body is:
   - for each non-static field `f` with `Some(value)`, in order:
     `let __f_f = value'`, where `value'` replaces `this.g` with `__f_g`;
   - for each of the first `plan.leading_run` constructor statements
     `this.f = e`: `let __f_f = e'`, or `__f_f = e'` when `__f_f` was
     already declared;
   - `const __this = { f1: __f_f1, …, fk: __f_fk };`, in `plan.fields`
     order;
   - the remaining constructor statements, with `this` replaced by
     `__this`, and every argument-less `return;` (outside nested functions
     and arrows) replaced by `return __this;`;
   - `return __this;`.
2. **The methods.** For each non-static method `m` other than `constructor`:
   `FunctionDeclaration { name: "C__m", params: ["__this", …m.params], body:
   m.body with this → __this, is_async: m.is_async, generator: m.generator
   }`.

**`this` → `__this`.** Replace every `ThisExpression`, descending into
`ArrowFunctionExpression` bodies and `FunctionExpression { is_arrow: true }`
bodies. Do **not** descend into `FunctionExpression { is_arrow: false }`,
`FunctionDeclaration` bodies, or class bodies. Implement it as its own small
recursive function, `fn replace_this(expr: &mut Expression)` and
`fn replace_this_in_block(block: &mut BlockStatement)`, using `walk` with a
visitor that tracks depth: the walker's frames tell the visitor which kind of
frame it is in.

**`rewrite_class_instances`:**

```rust
pub fn rewrite_class_instances(statements: &mut Vec<Statement>) -> ClassRewrite {
    let spelled = scopes::Scopes::build(statements).spelled().clone();
    let mut plans = classes::plan_classes(statements, &spelled);
    let mut diagnostics = std::mem::take(&mut plans.diagnostics);
    if plans.rewritten.is_empty() {
        return ClassRewrite { changed: false, diagnostics };
    }
    provenance::reassociate_new(statements, &plans);
    let scopes = scopes::Scopes::build(statements);
    let env = provenance::build_env(statements, &scopes, &plans);
    let prov = provenance::Provenance::solve(statements, &env);
    diagnostics.extend(uses::check_and_rewrite(statements, &env, &prov));
    translate::translate_classes(statements, &plans);
    diagnostics.extend(sweep(statements, &plans));
    ClassRewrite { changed: true, diagnostics }
}
```

`sweep` (A-10) walks the result. Each of these pushes
`class_construction_unavailable_message(C, CLASS_REASON_UNLOWERED)`, once per
class:
- a `NewExpression` whose `new_target` is a rewritten class;
- a `ThisExpression` whose nearest non-arrow frame key ends in `C__new` or
  `C__m` for a rewritten `C`.

The program frame `""` is excluded.

- [ ] **Step 1: Write the failing tests.** Create `class_instances/translate_tests.rs`:

```rust
use super::rewrite_class_instances;
use crate::test_support::parse_statements;

fn rewrite(src: &str) -> (Vec<kali_ast::Statement>, Vec<String>, bool) {
    let mut stmts = parse_statements(src);
    let r = rewrite_class_instances(&mut stmts);
    (stmts, r.diagnostics.into_iter().map(|d| d.message).collect(), r.changed)
}

#[test]
fn the_spec_example_translates() {
    let (got, d, changed) = rewrite(
        "class C { n = 0; constructor(v){ this.k = v; this.m = this.n + v; log(this); this.n = 1; } } function log(x){ return x.n; } const c = new C(2);",
    );
    assert!(d.is_empty(), "{d:?}");
    assert!(changed);
    let want = parse_statements(
        "function C__new(v){ let __f_n = 0; let __f_k = v; let __f_m = __f_n + v; const __this = { n: __f_n, k: __f_k, m: __f_m }; log(__this); __this.n = 1; return __this; } function log(x){ return x.n; } const c = C__new(2);",
    );
    assert_eq!(got, want);
}

#[test]
fn methods_take_this_and_arrows_keep_it_but_functions_do_not() {
    let (got, d, _) = rewrite(
        "class C { constructor(){ this.n = 1; } bump(){ const f = () => { this.n = this.n + 1; }; const g = function(){ return this; }; f(); return this.n; } } const c = new C(); c.bump();",
    );
    assert!(d.is_empty(), "{d:?}");
    let want = parse_statements(
        "function C__new(){ let __f_n = 1; const __this = { n: __f_n }; return __this; } function C__bump(__this){ const f = () => { __this.n = __this.n + 1; }; const g = function(){ return this; }; f(); return __this.n; } const c = C__new(); C__bump(c);",
    );
    assert_eq!(got, want);
}

#[test]
fn a_bare_return_in_the_constructor_returns_the_instance() {
    let (got, d, _) = rewrite("class C { constructor(v){ this.n = v; if (v > 1) { return; } this.n = 0; } } new C(2);");
    assert!(d.is_empty(), "{d:?}");
    let want = parse_statements("function C__new(v){ let __f_n = v; const __this = { n: __f_n }; if (v > 1) { return __this; } __this.n = 0; return __this; } C__new(2);");
    assert_eq!(got, want);
}

#[test]
fn a_program_without_rewritable_classes_is_byte_identical() {
    for src in [
        "const o = { n: 1 }; o.n = 2; console.log(o.n);",
        "class X extends EventTarget { fire(){ this.addEventListener('t', () => {}); return 1; } } const x = new X(); x.fire();",
        "class U { static twice(x){ return 2*x; } } U.twice(4);",
        "class A{ f(){return 4;} } class B extends A{} new B().f();",
    ] {
        let before = parse_statements(src);
        let (after, d, changed) = rewrite(src);
        assert!(!changed && d.is_empty(), "{src}: {d:?}");
        assert_eq!(after, before, "{src}");
    }
}

#[test]
fn nested_class_translates_in_place() {
    let (got, d, _) = rewrite("function main(){ class C{ constructor(v){ this.v=v; } } const c=new C(2); return c.v; }");
    assert!(d.is_empty(), "{d:?}");
    let want = parse_statements("function main(){ function C__new(v){ let __f_v = v; const __this = { v: __f_v }; return __this; } const c=C__new(2); return c.v; }");
    assert_eq!(got, want);
}
```

Wire it in `mod.rs` as `#[cfg(test)] #[path = "translate_tests.rs"] mod translate_tests;`.

- [ ] **Step 2: Run them to verify they fail.**

Run: `cargo test -p kali_types class_instances::translate`
Expected: compile errors.

- [ ] **Step 3: Implement `translate.rs`, the entry and `sweep`** per the rules above. Add `pub use repr_infer::infer_reprs;` to `lib.rs`.

- [ ] **Step 4: Run all the module tests.**

Run: `cargo test -p kali_types class_instances`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_types
git commit -m "feat(class-instances): translate classes to C__new / C__m, the A-10 sweep, and rewrite_class_instances"
```

---

### Task 8: Wire the pass into `analyze_source_file`

**Files:**
- Modify: `crates/kali_cli/src/build/compile.rs:770-776`
- Create: `crates/kali_cli/tests/cases/object/class_instances.toml` (first cases)

**Interfaces:**
- Consumes: `kali_types::class_instances::rewrite_class_instances`, `kali_types::infer_reprs`, `kali_types::monomorphize::monomorphize_statements`.

- [ ] **Step 1: Write the failing cases.** Create `crates/kali_cli/tests/cases/object/class_instances.toml`:

```toml
# Cases for the class-instances project (spec
# docs/superpowers/specs/2026-10-04-class-instances-design.md and its §6
# amendments). A program-class instance holds its fields, or kali refuses with
# E5506 under `run` and `check` alike. Each rationale gives kali's output at
# the baseline `7c4daa9f7` (tools/array-return-probes/baseline-cls.tsv) and
# node v26.10.0's.
#
# `[source]` keys are one file per program, because `[source]` is file-wide.

[source]
"field_write_in_method.js" = '''
class Stack{ constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } function main(){ const s=new Stack(); s.add(3); console.log(s.n); } main();
'''
"ctor_arg.js" = '''
class C{ constructor(v){ this.n=v; } get(){ return this.n+1; } } const s=new C(4); console.log(s.get(), s.n);
'''

[[case]]
name = "a_field_write_in_a_method_is_kept"
rationale = """literal-array-mutators followups §14b. node v26.10.0 prints `3`. At `7c4daa9f7` kali printed `0` at exit 0: the constructor never ran and the store in `add` was dropped."""
args = ["run", "field_write_in_method.js"]
exit = "success"
stdout = "3\n"

[[case]]
name = "a_constructor_argument_reaches_the_field"
rationale = """node v26.10.0 prints `5 4`. At `7c4daa9f7` kali failed with E4201 (invalid wasm): the class body was a zero-parameter function called with one argument."""
args = ["run", "ctor_arg.js"]
exit = "success"
stdout = "5 4\n"
```

- [ ] **Step 2: Run them to verify they fail.**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- object/class_instances`
Expected: 2 FAILED (stdout `0`, and E4201).

- [ ] **Step 3: Call the pass.** In `analyze_source_file`, immediately after `repr_table = resolved.repr_table;` and before the `shape_conflicts` loop:

```rust
        // Class instances (class-instances spec §3.1): rewrite in-slice
        // classes to object-literal factories AFTER the resolver has checked
        // the program as written, then re-infer reprs over the rewritten
        // program. A program with nothing to rewrite is untouched.
        let rewrite =
            kali_types::class_instances::rewrite_class_instances(&mut parsed.statements);
        diagnostics.extend(rewrite.diagnostics);
        if has_errors(&diagnostics) {
            return Err(diagnostics);
        }
        if rewrite.changed {
            kali_types::monomorphize::monomorphize_statements(&mut parsed.statements);
            repr_table = kali_types::infer_reprs(&parsed.statements);
        }
```

- [ ] **Step 4: Run the cases.**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- object/class_instances`
Expected: PASS.

If a case fails at codegen or with an unexpected E5506, print the rewritten
program to find which object-literal lane refuses it. Add a temporary
`eprintln!("{:#?}", parsed.statements)` after the rewrite, and remove it
before committing. Fix the rewrite's output, not codegen (Global
Constraints).

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_cli/src/build/compile.rs crates/kali_cli/tests/cases/object/class_instances.toml
git commit -m "feat(class-instances): run the class-instance rewrite in analyze_source_file"
```

---

### Task 9: The full case file and the probe diff

**Files:**
- Modify: `crates/kali_cli/tests/cases/object/class_instances.toml`

**Interfaces:**
- Consumes: Task 0's probes and baseline.

- [ ] **Step 1: Run the probes and diff.**

Run:
```bash
cargo build -q -p kali_cli
tools/array-return-probes/run.sh "$TMPDIR/cls-after.tsv"
grep '^cls_' "$TMPDIR/cls-after.tsv" | cut -f1,2,5 > "$TMPDIR/cls-after.cut"
cut -f1,2,5 tools/array-return-probes/baseline-cls.tsv | diff - "$TMPDIR/cls-after.cut"
```
Expected after:
- every row whose name has no `_r_`, `_ok_` or `_known_` infix is `CORRECT` with check `0`;
- every `cls_r_*` row is `REFUSES` with check `1`;
- every `cls_ok_*` row is unchanged (`CORRECT`);
- `cls_known_r30` is `SILENT` (kali `1`, node `true`, spec §1.1).

`cls_update`'s value-position update (`this.n++` as a statement) becomes
`CORRECT` through §3.5. Any other row that differs is a bug: fix the code,
not the probe.

- [ ] **Step 2: Add one case per probe row.** For each `cls_*` probe, add a
  `[source]` entry with the probe's program and a `[[case]]`. Take the probe
  name, drop the `cls_` prefix, and use the remainder plus `.js` as the file
  name:
  - a `CORRECT` row gets a `run` case with `exit = "success"` and node's
    stdout;
  - a `REFUSES` row gets a `run` case **and** a `check` case, each with
    `exit = "failure"`, `stderr_contains = ["E5506", "<the message
    fragment>"]` and `stdout = ""`. The fragment is a distinctive literal
    substring of the Task 1 message, for example `"constructing class `B`"`
    or `"an array element"`;
  - `cls_known_r30` gets a `run` case pinning `stdout = "1\n"`, with a
    rationale that says it is a known-wrong control for R-30 (spec §1.1) and
    that node prints `true`.

Add a ts twin of six core programs (`14b`, `ctor_arg`, `param`, `return`,
`this_arrow`, `compound`) under `[source]` keys ending `.ts`. Give each class
field a type annotation (`n: number = 0;`) and each parameter one too.

Every rationale quotes node v26.10.0's output and the baseline row from
`baseline-cls.tsv`.

- [ ] **Step 3: Run the case file.**

Run: `cargo test -p kali_cli --test cases -- object/class_instances`
Expected: PASS.

- [ ] **Step 4: Commit.**

```bash
git add crates/kali_cli/tests/cases/object/class_instances.toml
git commit -m "test(class-instances): pin every probe row, refusal and control as cases"
```

---

### Task 10: Blast radius, triage and re-pins

**Files:**
- Modify: whichever case files moved (re-pins only, each with a dated rationale note)
- Create: `docs/superpowers/followups/class-instances-discovered-defects.md` (§4 triage and §5 capability loss sections; Task 11 completes it)

- [ ] **Step 1: Run everything.**

Run: `cargo test --workspace 2>&1 | grep -E "^test result|FAILED|panicked" | sort | uniq -c`
and `cargo test -p kali_cli --test cases 2>&1 | tail -30`
Record every failing trial id.

**Stop rule:** more than about 50 moved trials means stop and report to the human partner.

- [ ] **Step 2: Classify each moved trial.** For each one, run the trial's
  program with the baseline binary (`git stash; git checkout 7c4daa9f7 &&
  cargo build -q -p kali_cli && cp target/debug/kali "$TMPDIR/kali-base";
  git checkout -; git stash pop`) and with HEAD. Put a row in the
  followups file's triage table: `| name | before (7c4daa9f7) | after (HEAD)
  | class |`, where class is one of:
  - **wanted:** a silent wrong value became correct, or became a refusal.
  - **capability loss, class 2 (dead code) or class 3 (output never
    depended on the zero instance):** acceptable; re-pin it.
  - **capability loss, class 1:** don't re-pin it. Stop and bring it to
    the human partner.
  - **rationale only:** the verdict is unchanged and only the text moved.

  Expected movers (spec §5.4):
  - `soundness/block_arrows.toml`'s `function Box` / `new Box(9)` programs
    now refuse with the plain-function message (class 3: the case pins
    callback ordering, not `b.v`);
  - its `class Box{constructor(){this.n=4}}` program now computes `b.n`
    correctly, so check whether its stdout changed;
  - `runtime/inline_allocation_value_position.toml`'s
    `a_constructed_argument_refuses` (`f(new C())`): if `C` is now in
    slice, the program runs, so classify it and pin node's output;
  - the oracle tier cases that construct a class.

- [ ] **Step 3: The refusal sweep.** For every `cli` and `oracle` step of
  every case file, run HEAD's binary. For each trial whose stderr contains
  any of these fragments, re-run it with the baseline binary and record the
  pair in the followups file's §5:
  - `constructing class`
  - `constructing an object with the plain function`
  - `of class `
  - `could not determine the class`
  - `using class `
  - `that kali would generate`

  This is the same sweep the unresolved-member-call project's §5 ran.

- [ ] **Step 4: Re-pin.** Re-pin each wanted, class-2 and class-3 trial.
  Append a dated note to its rationale: `**2026-10-04 (class-instances):**
  <what moved and why>`. Don't edit any assertion of a class-1 trial.

- [ ] **Step 5: Run everything again.**

Run: `cargo test --workspace` and `cargo test -p kali_cli --test cases`
Expected: PASS.

- [ ] **Step 6: Commit.**

```bash
git add crates/kali_cli/tests/cases docs/superpowers/followups/class-instances-discovered-defects.md
git commit -m "test(class-instances): triage and re-pin the trials the rewrite moved"
```

---

### Task 11: Docs

**Files:**
- Modify: `specs/15-errors.md` (the `E5506` scope list, near line 204)
- Modify: `specs/19-feature-maturity.md` (a new row next to the class-method rows, near line 583)
- Modify: `specs/05-ir.md:97`
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (R-36, about line 629)
- Modify: `docs/superpowers/followups/literal-array-mutators-discovered-defects.md` (§14)
- Modify: `docs/superpowers/followups/class-instances-discovered-defects.md` (complete it)

- [ ] **Step 1: `specs/15-errors.md`.** Under the `E5506` "Use `E5506` for
  cases such as:" list, add one bullet: "a program class kali cannot lower to
  an object, or an instance of one used where kali cannot prove its class
  (class-instances spec §3.4)". Then quote each Task 1 message's fixed
  prefix.

- [ ] **Step 2: `specs/19-feature-maturity.md`.** Add a row titled "Program-class instances". Its text must state exactly:
  - the slice: no `extends` link to another program class, and no accessor, `static`, `#private` or computed member;
  - what works: `new` runs field initializers then the constructor, `this.f` / `s.f` reads and writes, method calls with `this`, instances through parameters and returns, `this` in arrows, and compound assignment and statement-position `++` / `--` on fields;
  - the refusals of spec §1.1 and A-1;
  - that stateless out-of-slice classes keep the baseline lowering;
  - that boolean instance fields still render `1` / `0` after a write (R-30).

  Don't claim general class support.

- [ ] **Step 3: `specs/05-ir.md:97`.** After "Class instances where no
  dynamic property access occurs", add: "(implemented for the
  class-instances slice by lowering each class to an object-literal factory
  before IR; see `specs/19-feature-maturity.md`)".

- [ ] **Step 4: The register.** In R-36's bullet, append:
  `**Moved 2026-10-04 (class-instances, <commit of Task 8>):** SILENT →
  FIXED for an in-slice class (`object/class_instances::a_field_write_in_a_method_is_kept`
  and the `r36` case); a stateful out-of-slice class is FAIL_CLOSED (E5506).`
  If the R-36 entry is in the ranked §2 list, regenerate the blast-radius
  ranking as its document's procedure says:
  `cargo run -p kali_blast_radius --example rank`, then the
  `spliced_document_matches_the_generator` test. If R-36 is not in §2, say
  so in the followups file and don't regenerate.

- [ ] **Step 5: `literal-array-mutators-discovered-defects.md` §14.** Mark
  the first bullet (§14a) and the second bullet (§14b) **Fixed** by the
  class-instances project, with the commit and the case names, using the
  format §3 of that file uses.

- [ ] **Step 6: Complete the followups file.** Use the structure of
  `unresolved-member-call-discovered-defects.md`. The header gives the
  oracle, the commit it was measured at, and the probe source. Then:
  - **§1** The R-30 boolean field (`cls_known_r30`).
  - **§2** The out-of-slice refusals, as future items: program-class
    `extends` with state, accessors, statics with state, `#private`,
    computed members, class expressions, exported classes, and
    duplicate-named classes.
  - **§3** Containers of instances, rendering an instance, `instanceof` /
    `===` on instances, mixed-class parameters, and `new` of a plain
    function.
  - **§4** Task 10's triage table.
  - **§5** Task 10's capability-loss sweep.
  - **§6** Anything else measured and not fixed, including every Task 0
    baseline row that differed from its expectation, and the Task 2
    annotation heuristic's limit (a field type ending in one of
    `TYPE_WORDS` followed by a newline and a member key).

- [ ] **Step 7: Re-read for consistency.** Check that `specs/19`'s row,
  `specs/15`'s bullet and the followups file all describe the same slice as
  the spec with its amendments. Run `cargo test --workspace` one last time.

- [ ] **Step 8: Commit.**

```bash
git add specs docs/superpowers/followups
git commit -m "docs(class-instances): errors, maturity row, IR note, register R-36, §14a/§14b fixed, followups"
```
