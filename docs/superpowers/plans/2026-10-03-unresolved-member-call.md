# Unresolved Member Call Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A member call that reaches `emit_call`'s terminal warn+0 fallback refuses with `E5506` instead of evaluating to `0` when its receiver is a value the program built. `kali check` mirrors that for object-literal and program-class-instance `const` receivers.

**Architecture:**
- **Codegen.** One gate in `emit_call`, keyed on the receiver chain's root. A root refuses unless it has proven host provenance: a free global, a binding derived from one, or an instance of a class whose `extends` chain leaves the program.
- **Class facts.** The parser now keeps `extends` and field names. `kali_types` turns them into a `host_derived_classes` set on `ReprTable`, which codegen reads.
- **Type layer.** It adds a member-set check for the `check` mirror.

**Tech Stack:** Rust workspace (`kali_parser`, `kali_ast`, `kali_types`, `kali_common`, `kali_codegen`, `kali_cli`), TOML case files, the bash probe runner, and node v26.10.0 as the oracle.

**Spec:** `docs/superpowers/specs/2026-10-03-unresolved-member-call-design.md`, read together with its amendments A-1 to A-3 in §6.

## Global Constraints

- The refusal code is `E5506` (`e5::FEATURE_UNAVAILABLE`). No new diagnostic code, flag or schema. `specs/12`, `18` and `19` stay untouched.
- The refusal text is `kali_common::unresolved_member_call_unavailable_message(name)`, shared by both layers.
- The `run` refusal goes through `deny_e5506` (`crates/kali_codegen/src/intrinsics/host.rs:1666`), which emits `unreachable` and never `i64.const 0`.
- Free-global roots keep warn+0 unchanged (`document.foo()`, `performance.now()`, imports).
- Rust unit tests go in sibling `*_tests.rs` files wired with `#[path]`, never in inline `#[cfg(test)] mod` bodies.
- CLI tests are `.toml` cases under `crates/kali_cli/tests/cases/`. Don't add new `tests/*.rs` targets.
- The oracle is `node v26.10.0`. Every case rationale quotes node's output and the baseline (`6f042548f`) kali output.
- **Stop rule.** If more than about 50 existing tests move, stop and report before re-pinning anything.
- A capability loss is acceptable only if it falls in one of spec §5.4's two classes:
  - an effect never observed
  - dead code

  Anything else goes back to the human partner.
- Commit messages use the `feat(unresolved-member-call): …` / `test(…)` / `docs(…)` style.

## Review Focus

1. **A host object reached through a `let` that is never reassigned** (`let el = document.getElementById("x"); el.foo()`) must keep warn+0, not refuse. This is pinned in Task 4.
2. **A class whose base is not a plain identifier** (`class A extends mixin(B) {}`, `class A extends ns.B {}`) must count as host-derived. That way an inherited host method is never refused. This is pinned in Tasks 2 and 3.
3. **A method that exists only on a parent program class** (`class B extends A{}; new B().f()`) must still print node's output under `run`, and `check` must stay quiet. This is pinned in Tasks 5 and 6.
4. **A method added by assignment** (`o.f = …; o.f()`) or written in a constructor (`this.cb = …`) must not be refused by `check`. This is pinned in Task 5.
5. **A computed or text-less callee over a free-global root** (`globalThis["process"]["kill"](0)`) must keep its current lowering. This is pinned in Task 4.

---

## File Structure

| file | change | responsibility |
|---|---|---|
| `crates/kali_common/src/messages.rs` | modify | `unresolved_member_call_unavailable_message`, `OBJECT_PROTOTYPE_NAMES` |
| `crates/kali_common/src/messages_tests.rs` | modify | message and list tests |
| `crates/kali_common/src/repr.rs` | modify | `host_derived_classes` set and accessors |
| `crates/kali_common/src/repr_tests.rs` | modify | accessor test |
| `crates/kali_ast/src/declaration.rs`, `expression.rs` | modify | `super_class`, `field_names` fields |
| `crates/kali_parser/src/declaration.rs` | modify | parse `extends` and field names |
| `crates/kali_parser/src/declaration_tests/class_method.rs` | modify | parser tests |
| construction sites (Task 2 lists them) | modify | add the new fields |
| `crates/kali_types/src/program_classes.rs` | create | collect program classes; host-derived set; member sets |
| `crates/kali_types/src/program_classes_tests.rs` | create | unit tests |
| `crates/kali_types/src/lib.rs` | modify | module wiring |
| `crates/kali_types/src/repr_infer.rs` | modify | record `host_derived_classes` in `infer_reprs` |
| `crates/kali_types/src/resolve/member.rs` | modify | `reject_unresolved_member_call` (`check` mirror) |
| `crates/kali_types/src/resolve/member_tests.rs` | modify | mirror unit tests |
| `crates/kali_types/src/context.rs` | modify | hold the `ProgramClasses` and assigned-property-name set on the resolver |
| `crates/kali_codegen/src/emit/member_provenance.rs` | create | root walk, host provenance, the gate predicate |
| `crates/kali_codegen/src/emit/member_provenance_tests.rs` | create | unit tests |
| `crates/kali_codegen/src/emit/mod.rs` | modify | `mod member_provenance;` |
| `crates/kali_codegen/src/emit/url.rs` | modify | `receiver_root_is_url_provenance` uses the shared walk |
| `crates/kali_codegen/src/emit/call.rs` | modify | the gate in `emit_call` |
| `crates/kali_codegen/src/emitter.rs` | modify | `program_reassigned_names_cache` field |
| `tools/array-return-probes/probes/unres_*.js`, `probes/unres_lib/m.js` | create | probes |
| `tools/array-return-probes/baseline-unres.tsv` | create | baseline at `6f042548f` |
| `crates/kali_cli/tests/cases/soundness/unresolved_member_call.toml` | create | CLI cases |
| `specs/15-errors.md` | modify | E5506 scope line |
| `docs/superpowers/followups/literal-array-mutators-discovered-defects.md` | modify | §3 and §12 FIXED |
| `docs/superpowers/followups/unresolved-member-call-discovered-defects.md` | create | triage table, capability loss, gaps |

---

### Task 0: Probes and the baseline (before any code change)

**Files:**
- Create: `tools/array-return-probes/probes/unres_*.js` (listed below)
- Create: `tools/array-return-probes/probes/unres_lib/m.js`
- Create: `tools/array-return-probes/baseline-unres.tsv`

**Interfaces:**
- Produces: the probe names Task 6 diffs against.

- [ ] **Step 1: Confirm you are at the baseline.**

Run: `git log --oneline -1 -- crates/ && cargo build -q -p kali_cli && node --version`
Expected: the last commit touching `crates/` is at or before `6f042548f`, and node prints `v26.10.0`.

- [ ] **Step 2: Write the probe files.** Create each file below with exactly the content shown, one program per file, followed by a trailing newline.

| file | content |
|---|---|
| `unres_objlit.js` | `const o={k:1}; console.log(o.zork(4));` |
| `unres_strkey.js` | `const o={k:1}; console.log(o["zork"](4));` |
| `unres_deep.js` | `const o={a:{b:{}}}; console.log(o.a.b.zork());` |
| `unres_inmain.js` | `function main(){ const o={k:1}; console.log(o.zork()); } main();` |
| `unres_inst.js` | `class C{ f(){return 1;} } const c=new C(); console.log(c.g());` |
| `unres_inst_extends.js` | `class A{ f(){return 1;} } class B extends A{} const b=new B(); console.log(b.g());` |
| `unres_str.js` | `const s="abc"; console.log(s.zork());` |
| `unres_num.js` | `const n=5; console.log(n.zork());` |
| `unres_arr.js` | `const a=[1,2]; console.log(a.zork());` |
| `unres_lit_str.js` | `console.log("abc".zork());` |
| `unres_lit_obj.js` | `console.log(({k:1}).zork());` |
| `unres_alias.js` | `const o={k:1}; const p=o; console.log(p.zork());` |
| `unres_param.js` | `function g(x){ return x.zork(); } const o={k:1}; console.log(g(o));` |
| `unres_letre.js` | `let o={k:1}; o={k:2}; console.log(o.zork());` |
| `unres_fnres.js` | `function mk(){ return {k:1}; } const o=mk(); console.log(o.zork());` |
| `unres_hasown.js` | `const o={k:1}; console.log(o.hasOwnProperty("k"));` |
| `unres_call_push.js` | `const a=[1,2,3]; a.push.call(a, 4); console.log(a.length);` |
| `unres_call_pop.js` | `const a=[1,2,3]; a.pop.call(a); console.log(a.length);` |
| `unres_f_call_push.js` | `function main(){ const a=[1,2,3]; a.push.call(a, 4); console.log(a.length); } main();` |
| `unres_proto_push.js` | `const a=[1,2,3]; Array.prototype.push.apply(a, [4]); console.log(a.length);` |
| `unres_proto_pop.js` | `const a=[1,2,3]; Array.prototype.pop.call(a); console.log(a.length);` |
| `unres_ok_extends.js` | `class A{ f(){return 4;} } class B extends A{} const b=new B(); console.log(b.f());` |
| `unres_ok_usp.js` | `const u=new URLSearchParams("a=1"); u.append("b","2"); console.log(u.toString());` |
| `unres_ok_et.js` | `class X extends EventTarget{} const x=new X(); x.addEventListener("t", ()=>{}); console.log("ok");` |
| `unres_ok_freecall.js` | `const r=Math.max(1,2); console.log(r);` |
| `unres_ok_free_unused.js` | `performance.now(); console.log("ok");` |
| `unres_ok_host_let.js` | `let t=globalThis.performance; t.now(); console.log("ok");` |
| `unres_ok_host_alias.js` | `const t=globalThis.performance; console.log(typeof t.now());` |
| `unres_ok_import.js` | `import {m} from "./unres_lib/m.js"; console.log(m.zork());` |
| `unres_ok_inst_method.js` | `class S { push(v){ return v+1; } } const s=new S(); console.log(s.push(1));` |
| `unres_ok_alias_method.js` | `class S { f(){ return 6; } } const s=new S(); const t=s; console.log(t.f());` |
| `unres_ok_proto_toplevel.js` | `class A extends Object{} const a=new A(); console.log("ok");` |

Also create `unres_lib/m.js` containing `export const m={k:1};`. It sits in a subdirectory, so `run.sh`'s `probes/*.js` glob doesn't pick it up as a probe.

`unres_ok_host_alias` and `unres_ok_import` are silent at baseline and stay silent (spec §1.1). They are probes that document a disclosed gap; they are not CORRECT controls.

- [ ] **Step 3: Record the baseline.**

Run:
```bash
tools/array-return-probes/run.sh /tmp/claude-probes-all.tsv
grep '^unres_' /tmp/claude-probes-all.tsv > tools/array-return-probes/baseline-unres.tsv
cut -f1,2,5 tools/array-return-probes/baseline-unres.tsv
```
Expected:
- every non-`ok` row is `SILENT` with check `0`
- `unres_ok_*` rows are `CORRECT`, except `unres_ok_host_alias` and `unres_ok_import` (`SILENT`)

If any other row differs, add a line to the Task 8 followups file saying which row and what it showed. Don't change the probe to make it fit.

- [ ] **Step 4: Commit.**

```bash
git add tools/array-return-probes/probes/unres_* tools/array-return-probes/baseline-unres.tsv
git commit -m "test(unresolved-member-call): add unres_* probes and their baseline at 6f042548f"
```

---

### Task 1: Shared message and the `Object.prototype` name list

**Files:**
- Modify: `crates/kali_common/src/messages.rs` (after `array_mutator_unresolved_receiver_message`, about line 211)
- Test: `crates/kali_common/src/messages_tests.rs`

**Interfaces:**
- Produces:
  - `pub fn unresolved_member_call_unavailable_message(method: &str) -> String`
  - `pub const OBJECT_PROTOTYPE_NAMES: &[&str]`

  Both are re-exported through `kali_common::*`.

- [ ] **Step 1: Write the failing test.** Append this to `messages_tests.rs`:

```rust
#[test]
fn unresolved_member_call_message_is_stable() {
    assert_eq!(
        unresolved_member_call_unavailable_message("zork"),
        "calling `.zork()` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for a method of that name on it; node would run a method or throw a TypeError, so kali refuses rather than evaluate the call to 0"
    );
}

#[test]
fn object_prototype_names_are_the_eleven_inherited_methods() {
    assert_eq!(
        OBJECT_PROTOTYPE_NAMES,
        &[
            "constructor",
            "hasOwnProperty",
            "isPrototypeOf",
            "propertyIsEnumerable",
            "toLocaleString",
            "toString",
            "valueOf",
            "__defineGetter__",
            "__defineSetter__",
            "__lookupGetter__",
            "__lookupSetter__",
        ]
    );
}
```

- [ ] **Step 2: Run it and confirm it fails.**

Run: `cargo test -p kali_common unresolved_member_call_message_is_stable object_prototype_names`
Expected: compile error, `cannot find function unresolved_member_call_unavailable_message`.

- [ ] **Step 3: Implement.** Add this to `messages.rs` after `array_mutator_unresolved_receiver_message`:

```rust
/// The methods every object inherits from `Object.prototype`. The `check`
/// mirror of the unresolved-member-call gate never calls one of these
/// missing (unresolved-member-call spec §3.3).
pub const OBJECT_PROTOTYPE_NAMES: &[&str] = &[
    "constructor",
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "toLocaleString",
    "toString",
    "valueOf",
    "__defineGetter__",
    "__defineSetter__",
    "__lookupGetter__",
    "__lookupSetter__",
];

/// Canonical wording for a member call on a program-owned receiver that kali
/// cannot lower (unresolved-member-call spec §3.1). Shared by `check` and `run`.
pub fn unresolved_member_call_unavailable_message(method: &str) -> String {
    format!(
        "calling `.{method}()` is unavailable in the current phase: the receiver is a value this program built, and kali has no lowering for a method of that name on it; node would run a method or throw a TypeError, so kali refuses rather than evaluate the call to 0"
    )
}
```

- [ ] **Step 4: Run it and confirm it passes.**

Run: `cargo test -p kali_common`
Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/kali_common/src/messages.rs crates/kali_common/src/messages_tests.rs
git commit -m "feat(unresolved-member-call): shared refusal text and the Object.prototype name list"
```

---

### Task 2: The parser keeps `extends` and field names (A-1)

**Files:**
- Modify: `crates/kali_ast/src/declaration.rs:21-30` (`ClassDeclaration`, `ClassBody`)
- Modify: `crates/kali_ast/src/expression.rs:193-196` (`ClassExpression`)
- Modify: `crates/kali_parser/src/declaration.rs:280-366`
- Modify (add the new fields to every struct literal):
  - `crates/kali_cli/src/build/name_anon_functions_tests.rs:115-117`
  - `crates/kali_ast/src/declaration_tests.rs:21-89`
  - `crates/kali_types/src/resolve/function_tests/class_methods.rs` (every `ClassDeclaration {` / `ClassExpression {` / `ClassBody {`)
- Modify (patterns): `crates/kali_hir/src/lowering/statement.rs:311` and `crates/kali_types/src/resolve/mod.rs:786` both destructure `ClassDeclaration { name, body }`. Change them to `ClassDeclaration { name, body, .. }`.
- Test: `crates/kali_parser/src/declaration_tests/class_method.rs`

**Interfaces:**
- Produces:
  - `ClassDeclaration { name: String, super_class: Option<String>, body: Box<ClassBody> }`
  - `ClassExpression { id: Option<String>, super_class: Option<String>, body: Box<ClassBody> }`
  - `ClassBody { methods: Vec<MethodDefinition>, field_names: Vec<String> }`
- `super_class` is the text of the first token after `extends`:
  - an identifier gives its name (`extends ns.B` gives `"ns"`, `extends mixin(B)` gives `"mixin"`)
  - any other token gives `Some(String::new())`
- Task 3 treats any `super_class` that does not name a program class as leaving the program.

- [ ] **Step 1: Write the failing tests.** Append this to `class_method.rs`:

```rust
fn parse_single_class(source: &str) -> kali_ast::ClassDeclaration {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    match output.statements.into_iter().next() {
        Some(Statement::ClassDeclaration(class_decl)) => class_decl,
        other => panic!("expected a class declaration, got {other:?}"),
    }
}

#[test]
fn a_class_keeps_its_base_name() {
    assert_eq!(parse_single_class("class B extends A { f(){} }").super_class.as_deref(), Some("A"));
    assert_eq!(parse_single_class("class B extends ns.A {}").super_class.as_deref(), Some("ns"));
    assert_eq!(parse_single_class("class B extends mixin(A) {}").super_class.as_deref(), Some("mixin"));
    assert_eq!(parse_single_class("class B<T> extends A<T> {}").super_class.as_deref(), Some("A"));
    assert_eq!(parse_single_class("class B { f(){} }").super_class, None);
    assert_eq!(parse_single_class("class B implements I { f(){} }").super_class, None);
}

#[test]
fn a_class_with_a_base_keeps_its_methods() {
    let class_decl = parse_single_class("class B extends A { f(){ return 1; } g(){} }");
    let names: Vec<_> = class_decl.body.methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["f", "g"]);
}

#[test]
fn a_class_keeps_its_field_names() {
    let class_decl = parse_single_class("class S { n = 0; cb = () => 1; label: string; maybe?: number; done!: boolean; f(){} }");
    assert_eq!(class_decl.body.field_names, ["n", "cb", "label", "maybe", "done"]);
    assert_eq!(class_decl.body.methods.len(), 1);
}

#[test]
fn a_class_expression_keeps_its_base_name() {
    let tokens = lex("const K = class extends EventTarget {};");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    let Some(Statement::VariableDeclaration(decl)) = output.statements.first() else {
        panic!("expected a declaration, got {:?}", output.statements);
    };
    let Some(Expression::ClassExpression(class_expr)) = &decl.declarations[0].init else {
        panic!("expected a class expression");
    };
    assert_eq!(class_expr.super_class.as_deref(), Some("EventTarget"));
    assert_eq!(class_expr.id, None);
}
```

If `Expression` or `kali_ast` are not already imported at the top of `class_method.rs`, add `use kali_ast::Expression;`.

- [ ] **Step 2: Run them and confirm they fail.**

Run: `cargo test -p kali_parser a_class_`
Expected: compile error, `no field super_class`.

- [ ] **Step 3: Add the AST fields.** In `crates/kali_ast/src/declaration.rs`:

```rust
pub struct ClassDeclaration {
    pub name: String,
    /// The first token after `extends`, if any (unresolved-member-call spec
    /// A-1). An identifier gives its text; any other token gives `""`.
    #[serde(default)]
    pub super_class: Option<String>,
    pub body: Box<ClassBody>,
}

pub struct ClassBody {
    pub methods: Vec<MethodDefinition>,
    /// Names of the class-field declarations the parser skipped
    /// (`n = 0;`, `label: string;`). Kept so a member set can include them.
    #[serde(default)]
    pub field_names: Vec<String>,
}
```

In `crates/kali_ast/src/expression.rs`, `ClassExpression` gets the same `#[serde(default)] pub super_class: Option<String>,` between `id` and `body`.

- [ ] **Step 4: Parse them.** In `crates/kali_parser/src/declaration.rs`, add this helper next to `parse_class_body`:

```rust
    /// Consume an optional heritage clause up to (not including) the class
    /// body's `{`, returning the first token after `extends`. Type
    /// parameters and `implements` lists are skipped, as before.
    fn parse_class_heritage(&mut self) -> Option<String> {
        let mut super_class = None;
        while !matches!(self.stream.current_kind(), Some(TokenType::LeftBrace) | None) {
            if self.stream.current_kind() == Some(&TokenType::Extends) {
                let _ = self.stream.advance();
                super_class = Some(match self.stream.current_kind() {
                    Some(TokenType::Identifier) => {
                        self.stream.current().map(|token| token.value.clone()).unwrap_or_default()
                    }
                    _ => String::new(),
                });
                continue;
            }
            let _ = self.stream.advance();
        }
        super_class
    }
```

In `parse_class_declaration`, set `let super_class = self.parse_class_heritage();` before `let body = self.parse_class_body();`. Then build `ClassDeclaration { name, super_class, body: Box::new(body) }`. Do the same in `parse_class_expression`: call the helper after the optional `id`, and add `super_class` to the struct.

In `parse_class_body`, declare `let mut field_names = Vec::new();`. Replace the `else { let _ = self.stream.advance(); }` arm with:

```rust
            } else {
                if self.stream.current_kind() == Some(&TokenType::Identifier)
                    && matches!(
                        self.stream.peek_next_kind(),
                        Some(TokenType::Eq | TokenType::Semicolon | TokenType::Colon | TokenType::Question | TokenType::Bang)
                    )
                {
                    if let Some(token) = self.stream.current() {
                        field_names.push(token.value.clone());
                    }
                }
                let _ = self.stream.advance();
            }
```

End the function with `ClassBody { methods, field_names }`.

Over-collection is acceptable. A `b :` inside a field initializer's ternary adds `b`, which only makes the `check` mirror quieter.

- [ ] **Step 5: Fix the construction and match sites listed under Files.**
  - In every struct literal, add `super_class: None,` to `ClassDeclaration` / `ClassExpression`, and `field_names: Vec::new(),` to `ClassBody`.
  - In the two destructuring patterns, add `..`.

Run: `cargo build --workspace --tests 2>&1 | grep -E "^error" | head`
Expected: no output.

- [ ] **Step 6: Run the tests and confirm they pass.**

Run: `cargo test -p kali_parser && cargo test -p kali_ast && cargo test -p kali_types class_methods`
Expected: PASS.

- [ ] **Step 7: Commit.**

```bash
git add crates/kali_ast crates/kali_parser crates/kali_hir/src/lowering/statement.rs crates/kali_types/src/resolve crates/kali_cli/src/build/name_anon_functions_tests.rs
git commit -m "feat(unresolved-member-call): the parser keeps a class's base name and field names (A-1)"
```

---

### Task 3: Program classes and `ReprTable::host_derived_classes`

**Files:**
- Create: `crates/kali_types/src/program_classes.rs`
- Create: `crates/kali_types/src/program_classes_tests.rs`
- Modify: `crates/kali_types/src/lib.rs`. Add `mod program_classes;` next to `mod growable;`, plus `#[cfg(test)] #[path = "program_classes_tests.rs"] mod program_classes_tests;` next to the other test modules (about line 70).
- Modify: `crates/kali_common/src/repr.rs`. Add the field to `ReprTable` (after `array_returns`, about line 108) and the accessors after `array_return` (about line 629).
- Modify: `crates/kali_common/src/repr_tests.rs`
- Modify: `crates/kali_types/src/repr_infer.rs:1459` (`infer_reprs`)

**Interfaces:**
- Consumes: the AST fields from Task 2.
- Produces:
  - `pub(crate) struct ProgramClasses`, with:
    - `pub(crate) fn collect(statements: &[kali_ast::Statement]) -> ProgramClasses`
    - `pub(crate) fn is_program_class(&self, name: &str) -> bool`
    - `pub(crate) fn host_derived(&self) -> BTreeSet<String>`: every class whose `extends` chain reaches a name that is not a program class (`""` included)
    - `pub(crate) fn member_names(&self, name: &str) -> Option<BTreeSet<String>>`: `None` when the class is host-derived or unknown. Otherwise the method and field names of every class on the chain.
  - `pub(crate) fn assigned_property_names(statements: &[kali_ast::Statement]) -> BTreeSet<String>`: the `property` of every `AssignmentExpression` whose `left` is a `MemberExpression` with `Some(property)`.
  - `ReprTable::set_host_derived_class(&mut self, name: &str)` and `ReprTable::is_host_derived_class(&self, name: &str) -> bool`.

- [ ] **Step 1: Write the failing tests.** Create `program_classes_tests.rs`:

```rust
use crate::program_classes::{assigned_property_names, ProgramClasses};
use crate::test_support::parse_statements;

#[test]
fn a_class_extending_a_program_class_is_not_host_derived() {
    let classes = ProgramClasses::collect(&parse_statements("class A { f(){} } class B extends A { g(){} }"));
    assert!(classes.host_derived().is_empty());
    let names: Vec<_> = classes.member_names("B").unwrap().into_iter().collect();
    assert_eq!(names, ["f", "g"]);
}

#[test]
fn a_class_whose_chain_leaves_the_program_is_host_derived() {
    let classes = ProgramClasses::collect(&parse_statements(
        "class X extends EventTarget {} class Y extends X {} class Z extends mixin(A) {} class W extends ns.B {}",
    ));
    let host: Vec<_> = classes.host_derived().into_iter().collect();
    assert_eq!(host, ["W", "X", "Y", "Z"]);
    assert_eq!(classes.member_names("Y"), None);
}

#[test]
fn nested_and_expression_classes_are_collected() {
    let classes = ProgramClasses::collect(&parse_statements(
        "function f(){ class Inner extends EventTarget {} } const K = class extends HTMLElement {};",
    ));
    let host: Vec<_> = classes.host_derived().into_iter().collect();
    assert_eq!(host, ["Inner", "K"]);
}

#[test]
fn field_names_join_the_member_set() {
    let classes = ProgramClasses::collect(&parse_statements("class S { n = 0; f(){} }"));
    let names: Vec<_> = classes.member_names("S").unwrap().into_iter().collect();
    assert_eq!(names, ["f", "n"]);
}

#[test]
fn a_cyclic_chain_terminates_and_counts_as_host() {
    // Not valid at run time, but the walk must stop.
    let classes = ProgramClasses::collect(&parse_statements("class A extends B {} class B extends A {}"));
    assert!(classes.host_derived().contains("A"));
}

#[test]
fn assigned_property_names_cover_every_receiver_and_this() {
    let names = assigned_property_names(&parse_statements(
        "const o={k:1}; o.f = 1; class S { constructor(){ this.cb = 2; } } function g(x){ x.h += 1; }",
    ));
    let names: Vec<_> = names.into_iter().collect();
    assert_eq!(names, ["cb", "f", "h"]);
}
```

Add this to `crates/kali_common/src/repr_tests.rs`:

```rust
#[test]
fn host_derived_classes_round_trip() {
    let mut table = ReprTable::default();
    assert!(!table.is_host_derived_class("X"));
    table.set_host_derived_class("X");
    assert!(table.is_host_derived_class("X"));
    assert!(!table.is_host_derived_class("Y"));
}
```

Also add one to `crates/kali_types/src/repr_infer_tests.rs`:

```rust
#[test]
fn infer_reprs_records_host_derived_classes() {
    let table = crate::repr_infer::infer_reprs(&crate::test_support::parse_statements(
        "class X extends EventTarget {} class A {} class B extends A {}",
    ));
    assert!(table.is_host_derived_class("X"));
    assert!(!table.is_host_derived_class("A"));
    assert!(!table.is_host_derived_class("B"));
}
```

If `repr_infer_tests.rs` already imports `infer_reprs` and `parse_statements` under shorter paths, use those.

- [ ] **Step 2: Run them and confirm they fail.**

Run: `cargo test -p kali_types program_classes infer_reprs_records_host_derived_classes; cargo test -p kali_common host_derived_classes_round_trip`
Expected: compile errors, unresolved `program_classes` / `set_host_derived_class`.

- [ ] **Step 3: Implement the `ReprTable` accessors.** Add the field to the `ReprTable` struct:

```rust
    /// Program classes whose `extends` chain reaches a name that is not a
    /// program class (unresolved-member-call spec A-1). Codegen counts an
    /// instance of one as host provenance.
    host_derived_classes: HashSet<String>,
```

Add these methods:

```rust
    pub fn set_host_derived_class(&mut self, name: &str) {
        self.host_derived_classes.insert(name.to_string());
    }

    pub fn is_host_derived_class(&self, name: &str) -> bool {
        self.host_derived_classes.contains(name)
    }
```

If `ReprTable` derives `Default` it needs nothing more. If it has a hand-written constructor, add `host_derived_classes: HashSet::new(),` there.

- [ ] **Step 4: Implement `program_classes.rs`.** It walks the serialized AST, the same whole-tree idiom `program_contains_class` uses (`repr_infer.rs:356`). That way nested functions, class bodies and expressions are covered without a hand-written visitor.

```rust
//! Program classes, their bases and member names (unresolved-member-call
//! spec A-1, §3.2, §3.3). Collected over the serialized AST so nested
//! functions, methods and class expressions are all covered.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::Statement;
use serde_json::Value;

struct ClassFacts {
    super_class: Option<String>,
    members: BTreeSet<String>,
}

pub(crate) struct ProgramClasses {
    classes: BTreeMap<String, ClassFacts>,
}

impl ProgramClasses {
    pub(crate) fn collect(statements: &[Statement]) -> ProgramClasses {
        let mut classes = BTreeMap::new();
        if let Ok(tree) = serde_json::to_value(statements) {
            collect_from(&tree, None, &mut classes);
        }
        ProgramClasses { classes }
    }

    pub(crate) fn is_program_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// The chain from `name` upward, or `None` when it leaves the program
    /// (a base that is no program class, or a cycle).
    fn chain(&self, name: &str) -> Option<Vec<&ClassFacts>> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = name;
        loop {
            if !seen.insert(current) {
                return None;
            }
            let facts = self.classes.get(current)?;
            chain.push(facts);
            match facts.super_class.as_deref() {
                None => return Some(chain),
                Some(base) => current = base,
            }
        }
    }

    pub(crate) fn host_derived(&self) -> BTreeSet<String> {
        self.classes
            .keys()
            .filter(|name| self.chain(name).is_none())
            .cloned()
            .collect()
    }

    pub(crate) fn member_names(&self, name: &str) -> Option<BTreeSet<String>> {
        let chain = self.chain(name)?;
        Some(chain.iter().flat_map(|facts| facts.members.iter().cloned()).collect())
    }
}

fn class_facts(class: &Value) -> ClassFacts {
    let super_class = class
        .get("super_class")
        .and_then(Value::as_str)
        .map(str::to_string);
    let mut members = BTreeSet::new();
    if let Some(body) = class.get("body") {
        for method in body.get("methods").and_then(Value::as_array).into_iter().flatten() {
            if let Some(name) = method.get("name").and_then(Value::as_str) {
                members.insert(name.to_string());
            }
        }
        for field in body.get("field_names").and_then(Value::as_array).into_iter().flatten() {
            if let Some(name) = field.as_str() {
                members.insert(name.to_string());
            }
        }
    }
    ClassFacts { super_class, members }
}

/// `binding` is the declarator name when `value` is a declarator's init, so a
/// nameless `const K = class …` is recorded as `K`.
fn collect_from(value: &Value, binding: Option<&str>, out: &mut BTreeMap<String, ClassFacts>) {
    match value {
        Value::Object(map) => {
            if let Some(class) = map.get("ClassDeclaration") {
                if let Some(name) = class.get("name").and_then(Value::as_str) {
                    out.insert(name.to_string(), class_facts(class));
                }
            }
            if let Some(class) = map.get("ClassExpression") {
                let name = class.get("id").and_then(Value::as_str).or(binding);
                if let Some(name) = name {
                    out.insert(name.to_string(), class_facts(class));
                }
            }
            // A declarator: `{ "id": "K", "init": … }`.
            let declarator_name = match (map.get("id"), map.get("init")) {
                (Some(Value::String(id)), Some(_)) => Some(id.as_str()),
                _ => None,
            };
            for (key, child) in map {
                let child_binding = if key == "init" { declarator_name } else { None };
                collect_from(child, child_binding, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_from(item, binding, out);
            }
        }
        _ => {}
    }
}

/// Every property name some assignment writes, on any receiver, in any scope
/// (`o.f = …`, `this.cb = …`, `x.h += 1`).
pub(crate) fn assigned_property_names(statements: &[Statement]) -> BTreeSet<String> {
    fn walk(value: &Value, out: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                if let Some(assign) = map.get("AssignmentExpression") {
                    if let Some(property) = assign
                        .get("left")
                        .and_then(|left| left.get("MemberExpression"))
                        .and_then(|member| member.get("property"))
                        .and_then(Value::as_str)
                    {
                        out.insert(property.to_string());
                    }
                }
                map.values().for_each(|child| walk(child, out));
            }
            Value::Array(items) => items.iter().for_each(|item| walk(item, out)),
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    if let Ok(tree) = serde_json::to_value(statements) {
        walk(&tree, &mut out);
    }
    out
}
```

Before relying on the JSON key names (`ClassDeclaration`, `ClassExpression`, `AssignmentExpression`, `MemberExpression`, `property`, `id`, `init`), confirm them with a one-off test. Print `serde_json::to_string(&parse_statements("const K = class extends E {}; o.f = 1;"))`, check the keys, then delete the debug test. `program_contains_class` already checks both a `ClassDeclaration` key and a `kind` tag, so if the shape is tagged by `kind`, match on that instead.

- [ ] **Step 5: Record the set in `infer_reprs`.** In `crates/kali_types/src/repr_infer.rs`, inside `infer_reprs` after the existing `emit_table` call has produced the table (find the `let mut table` / returned table near the end of the function), add:

```rust
    for name in crate::program_classes::ProgramClasses::collect(statements).host_derived() {
        table.set_host_derived_class(&name);
    }
```

If the function returns `emit_table(...)` directly, bind it first: `let mut table = <that expression>;`, add the loop, then `table`.

- [ ] **Step 6: Run the tests and confirm they pass.**

Run: `cargo test -p kali_types program_classes infer_reprs_records_host_derived_classes && cargo test -p kali_common`
Expected: PASS.

- [ ] **Step 7: Commit.**

```bash
git add crates/kali_types/src/program_classes.rs crates/kali_types/src/program_classes_tests.rs crates/kali_types/src/lib.rs crates/kali_types/src/repr_infer.rs crates/kali_types/src/repr_infer_tests.rs crates/kali_common/src/repr.rs crates/kali_common/src/repr_tests.rs
git commit -m "feat(unresolved-member-call): collect program classes and hand host-derived ones to codegen through ReprTable"
```

---

### Task 4: The codegen gate

**Files:**
- Create: `crates/kali_codegen/src/emit/member_provenance.rs`
- Create: `crates/kali_codegen/src/emit/member_provenance_tests.rs`
- Modify: `crates/kali_codegen/src/emit/mod.rs` (add `mod member_provenance;` after `mod literal;`)
- Modify: `crates/kali_codegen/src/emit/url.rs:417-446`
- Modify: `crates/kali_codegen/src/emit/call.rs`, between the literal-array backstop (ends about line 3962) and `if self.deny_placeholder_lowering(` (about line 3972)
- Modify: `crates/kali_codegen/src/emitter.rs`. Add `pub(crate) program_reassigned_names_cache: std::cell::OnceCell<HashSet<String>>,` next to `program_bound_names_cache` (line 376), and its initializer next to line 734.

**Interfaces:**
- Consumes:
  - `ReprTable::is_host_derived_class` (Task 3)
  - `kali_common::unresolved_member_call_unavailable_message` (Task 1)
  - `self.name_is_program_bound(&str) -> bool` (`call.rs:4826`)
  - `self.name_is_declared_parameter(&str) -> bool` (`call.rs:4427`)
  - `self.functions` (map keyed by name)
  - `self.unwrap_transparent(LirNodeId) -> LirNodeId`
  - `self.node(LirNodeId) -> &LirNode`
  - `crate::lower::is_binary_operator_text(&str)`
  - `crate::lower::is_function_like(&[LirNode], LirNodeId)`
  - `crate::lower::program_reassigned_names(&[LirNode]) -> HashSet<String>`
  - `self.deny_e5506(&mut Function, &str) -> EmittedValue`
- Produces, on `FunctionEmitter`:
  - `pub(crate) fn receiver_chain_root(&self, receiver: LirNodeId) -> LirNodeId`
  - `pub(crate) fn unresolved_member_call_refuses(&self, callee_node: &LirNode, callee_name: &str) -> bool`

LIR facts this task relies on (from `crates/kali_lir`, `kali_hir/src/lowering`):

| construct | LIR shape |
|---|---|
| identifier | childless `Value` with `text = name` |
| dot member | 1-child `Value` with non-empty `text` = property |
| computed member | 2-child `Value` whose text is not a binary operator |
| call | `Call(None, [callee, args…])` |
| `new C(a)` | `Value(None, [Call(None, [Value("C"), a])])`; `unwrap_transparent` passes through the outer `Value` |
| string / number / bool / null / regex literal | `LirNodeKind::Literal` |
| array / object literal | text-less `Value` with 0 or ≥2 children; one element unwraps to the element |
| declarator list | `Instruction("const"\|"let"\|"var", [declarator…])`, with `declarator = Instruction(name, [Value(name), init?])` |

- [ ] **Step 1: Write the failing tests.** Create `member_provenance_tests.rs`:

```rust
use crate::emit::computed_member::computed_member_tests::{assert_e5506, diagnostics_for};

const UNRES: &str = "is unavailable in the current phase: the receiver is a value this program built";

fn assert_not_refused(source: &str) {
    let diagnostics = diagnostics_for(source);
    assert!(
        !diagnostics.iter().any(|d| d.message.contains(UNRES)),
        "{source}: expected no unresolved-member-call refusal, got {diagnostics:?}"
    );
}

#[test]
fn a_program_owned_root_refuses() {
    for source in [
        "const o={k:1}; console.log(o.zork(4));",
        "const o={k:1}; console.log(o[\"zork\"](4));",
        "const o={a:{b:{}}}; console.log(o.a.b.zork());",
        "function main(){ const o={k:1}; console.log(o.zork()); } main();",
        "class C{ f(){return 1;} } const c=new C(); console.log(c.g());",
        "const s=\"abc\"; console.log(s.zork());",
        "const n=5; console.log(n.zork());",
        "const a=[1,2]; console.log(a.zork());",
        "const o={k:1}; const p=o; console.log(p.zork());",
        "function g(x){ return x.zork(); } const o={k:1}; console.log(g(o));",
        "let o={k:1}; o={k:2}; console.log(o.zork());",
        "function mk(){ return {k:1}; } const o=mk(); console.log(o.zork());",
        "const o={k:1}; console.log(o.hasOwnProperty(\"k\"));",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES, source);
    }
}

#[test]
fn a_literal_start_refuses() {
    for source in ["console.log(\"abc\".zork());", "console.log(({k:1}).zork());", "console.log([1,2].zork());"] {
        assert_e5506(&diagnostics_for(source), UNRES, source);
    }
}

#[test]
fn call_and_apply_through_a_program_root_or_an_intrinsic_prototype_refuse() {
    for source in [
        "const a=[1,2,3]; a.push.call(a, 4); console.log(a.length);",
        "const a=[1,2,3]; a.pop.call(a); console.log(a.length);",
        "function main(){ const a=[1,2,3]; a.push.call(a, 4); console.log(a.length); } main();",
        "const a=[1,2,3]; Array.prototype.push.apply(a, [4]); console.log(a.length);",
        "const a=[1,2,3]; Array.prototype.pop.call(a); console.log(a.length);",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES, source);
    }
}

#[test]
fn a_host_root_keeps_its_lowering() {
    for source in [
        "performance.now(); console.log(\"ok\");",
        "const t=globalThis.performance; t.now(); console.log(\"ok\");",
        "let t=globalThis.performance; t.now(); console.log(\"ok\");",
        "function main(){ let el=document.getElementById(\"x\"); el.focus(); } main();",
        "const u=new URLSearchParams(\"a=1\"); u.append(\"b\",\"2\"); console.log(u.toString());",
        "globalThis[\"process\"][\"kill\"](0);",
        "class S { f(){ return 6; } } const s=new S(); console.log(s.f());",
        "class A{ f(){return 4;} } class B extends A{} const b=new B(); console.log(b.f());",
    ] {
        assert_not_refused(source);
    }
}

#[test]
fn a_reassigned_let_bound_from_a_host_is_not_host() {
    let source = "let t=globalThis.performance; t={}; console.log(t.now());";
    assert_e5506(&diagnostics_for(source), UNRES, source);
}

#[test]
fn a_call_result_receiver_keeps_todays_lowering() {
    // §14a's route and `mk().zork()` stop at a call node (spec §3.2).
    assert_not_refused("function mk(){ return {k:1}; } console.log(mk().zork());");
}

#[test]
fn an_alias_cycle_terminates() {
    // Not valid JS at run time (TDZ), but the provenance walk must stop.
    let source = "function main(){ const a=b; const b=a; console.log(a.zork()); } main();";
    assert_e5506(&diagnostics_for(source), UNRES, source);
}
```

Add this to the end of `member_provenance.rs` (created in Step 3):

```rust
#[cfg(test)]
#[path = "member_provenance_tests.rs"]
mod member_provenance_tests;
```

These tests run with an empty `ReprTable`. The `extends EventTarget` row is therefore pinned by the Task 6 CLI case, not here (spec A-3).

- [ ] **Step 2: Run them and confirm they fail.**

Run: `cargo test -p kali_codegen member_provenance`
Expected: compile error, unknown module, until Step 3 creates the file. After that, `a_program_owned_root_refuses` should fail because no refusal exists yet. To see that failure first, create the file with just the test-module wiring, run the tests, then fill it in.

- [ ] **Step 3: Implement `member_provenance.rs`.**

```rust
//! Unresolved-member-call spec §3.1-§3.2: whether a member call that reached
//! `emit_call`'s terminal fallback is on a value this program built (refuse)
//! or on a host value (keep the warn+0 escape hatch).

use std::collections::HashSet;

use crate::*;

impl<'a> FunctionEmitter<'a> {
    /// Walk a member chain (dot and computed, any depth, through transparent
    /// wrappers) down to the node it stops at: a root identifier, a literal,
    /// a call, or any other shape. Shared with the URL/USP root walk.
    pub(crate) fn receiver_chain_root(&self, receiver: LirNodeId) -> LirNodeId {
        let mut current = self.unwrap_transparent(receiver);
        loop {
            let node = self.node(current);
            if node.kind != LirNodeKind::Value {
                return current;
            }
            match node.children.len() {
                1 if node.text.as_deref().is_some_and(|text| !text.is_empty()) => {
                    current = self.unwrap_transparent(node.children[0]);
                }
                2 if !crate::lower::is_binary_operator_text(node.text.as_deref().unwrap_or_default()) => {
                    current = self.unwrap_transparent(node.children[0]);
                }
                _ => return current,
            }
        }
    }

    /// The §3.1 gate. `callee_node` is the call's callee (a member access);
    /// `callee_name` its method name.
    pub(crate) fn unresolved_member_call_refuses(&self, callee_node: &LirNode, callee_name: &str) -> bool {
        let Some(&receiver) = callee_node.children.first() else {
            return false;
        };
        if matches!(callee_name, "call" | "apply") && self.is_intrinsic_prototype_borrow(receiver) {
            return true;
        }
        let root = self.receiver_chain_root(receiver);
        let root_node = self.node(root);
        match root_node.kind {
            LirNodeKind::Literal => true,
            LirNodeKind::Value if root_node.children.is_empty() => match root_node.text.as_deref() {
                Some(name) if !name.is_empty() => {
                    !self.root_has_host_provenance(name, &mut HashSet::new())
                }
                // `{}` / `[]`: a text-less childless Value.
                _ => true,
            },
            // An array or object literal with two or more children.
            LirNodeKind::Value if root_node.text.is_none() && root_node.children.len() >= 2 => true,
            _ => false,
        }
    }

    /// `Array.prototype.m` / `Object.prototype.m` / `String.prototype.m` as
    /// the receiver of `.call` / `.apply`, rooted at the unshadowed global.
    fn is_intrinsic_prototype_borrow(&self, receiver: LirNodeId) -> bool {
        let method = self.node(self.unwrap_transparent(receiver));
        if method.children.len() != 1 {
            return false;
        }
        let prototype = self.node(self.unwrap_transparent(method.children[0]));
        if prototype.text.as_deref() != Some("prototype") || prototype.children.len() != 1 {
            return false;
        }
        let global = self.node(self.unwrap_transparent(prototype.children[0]));
        global.children.is_empty()
            && global
                .text
                .as_deref()
                .is_some_and(|name| matches!(name, "Array" | "Object" | "String") && self.is_free_global(name))
    }

    fn is_free_global(&self, name: &str) -> bool {
        !self.name_is_program_bound(name) && !self.functions.contains_key(name)
    }

    fn program_reassigned_names(&self) -> &HashSet<String> {
        self.program_reassigned_names_cache
            .get_or_init(|| crate::lower::program_reassigned_names(&self.program.nodes))
    }

    /// §3.2. `seen` stops an alias cycle; a name seen twice is not proven.
    fn root_has_host_provenance(&self, name: &str, seen: &mut HashSet<String>) -> bool {
        if !seen.insert(name.to_string()) {
            return false;
        }
        if self.is_free_global(name) {
            return true;
        }
        if self.name_is_declared_parameter(name) {
            return false;
        }
        let Some((kind, Some(init))) = self.declarator_of(name) else {
            return false;
        };
        if kind != "const" && self.program_reassigned_names().contains(name) {
            return false;
        }
        self.init_has_host_provenance(init, seen)
    }

    /// An initializer has host provenance when its member/call/`new` chain
    /// reaches a host root, or calls a host-derived program class.
    fn init_has_host_provenance(&self, init: LirNodeId, seen: &mut HashSet<String>) -> bool {
        let mut current = self.unwrap_transparent(init);
        loop {
            let node = self.node(current);
            match node.kind {
                LirNodeKind::Call => match node.children.first() {
                    Some(&callee) => current = self.unwrap_transparent(callee),
                    None => return false,
                },
                LirNodeKind::Value if node.children.is_empty() => {
                    let Some(name) = node.text.as_deref().filter(|name| !name.is_empty()) else {
                        return false;
                    };
                    if self.repr_table.is_host_derived_class(name) {
                        return true;
                    }
                    return self.root_has_host_provenance(name, seen);
                }
                LirNodeKind::Value
                    if node.children.len() == 1 && node.text.as_deref().is_some_and(|t| !t.is_empty()) =>
                {
                    current = self.unwrap_transparent(node.children[0]);
                }
                LirNodeKind::Value
                    if node.children.len() == 2
                        && !crate::lower::is_binary_operator_text(node.text.as_deref().unwrap_or_default()) =>
                {
                    current = self.unwrap_transparent(node.children[0]);
                }
                _ => return false,
            }
        }
    }

    /// The nearest declarator of `name`: the current function body first,
    /// then the module body. Does not descend into nested function-like
    /// nodes (template: `binding_is_placeholder_construct`,
    /// `intrinsics/host.rs:1821`). Returns `(kind, init)`.
    fn declarator_of(&self, name: &str) -> Option<(&str, Option<LirNodeId>)> {
        self.declarator_in(self.body, name)
            .or_else(|| self.declarator_in(self.program.root, name))
    }

    fn declarator_in(&self, root: LirNodeId, name: &str) -> Option<(&str, Option<LirNodeId>)> {
        let nodes = &self.program.nodes;
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            let node = self.node(id);
            if id != root && crate::lower::is_function_like(nodes, id) {
                continue;
            }
            if node.kind == LirNodeKind::Instruction {
                if let Some(kind @ ("const" | "let" | "var")) = node.text.as_deref() {
                    for &declarator_id in &node.children {
                        let declarator = self.node(declarator_id);
                        if declarator.text.as_deref() == Some(name) {
                            return Some((kind, declarator.children.get(1).copied()));
                        }
                    }
                }
            }
            stack.extend(node.children.iter().copied());
        }
        None
    }
}
```

Compile notes for the implementer:
- `self.functions`, `self.program`, `self.body` and `self.repr_table` are existing `FunctionEmitter` fields.
- If `self.functions` is not keyed by `&str`-compatible `String`, adapt the `contains_key` call to the map's key type (look at its declaration in `emitter.rs`).
- If `is_binary_operator_text` or `program_reassigned_names` is private to `lower.rs`, make it `pub(crate)`.
- The lifetime of the `&str` in `declarator_of`'s return comes from `self.program.nodes`. If the borrow checker objects, return `String` instead.

- [ ] **Step 4: Make the URL walk use the shared root walk.** Replace the body of `receiver_root_is_url_provenance` in `emit/url.rs`, keeping its doc comment and adding one line that names the shared walk:

```rust
    pub(crate) fn receiver_root_is_url_provenance(&self, receiver: LirNodeId) -> bool {
        let root = self.node(self.receiver_chain_root(receiver));
        root.kind == LirNodeKind::Value
            && root.children.is_empty()
            && root.text.as_deref().is_some_and(|name| {
                self.is_url(name)
                    || self.is_url_search_params(name)
                    || self.is_module_scope_url_handle(name)
                    || self.is_captured_url_handle(name)
            })
    }
```

This preserves the old behaviour. The old walk answered false for any non-`Value` node and for every shape that wasn't a member; the new one answers false unless the stop node is a childless `Value` naming a URL handle.

- [ ] **Step 5: Insert the gate in `emit_call`.** In `call.rs`, directly after the literal-array backstop block that ends with `return self.deny_e5506(function, &message);` / `}` (about line 3962), and before the `// Positive DENY-SET` comment:

```rust
        // Unresolved-member-call spec §3.1: a member call on a value this
        // program built reaches here only when nothing lowered it. Evaluating
        // it to 0 is silently wrong, so refuse; a host-rooted chain keeps the
        // warn+0 escape hatch below.
        if self.unresolved_member_call_refuses(&callee_node, callee_name) {
            let method = if callee_name.is_empty() { "[computed]" } else { callee_name };
            let message = kali_common::unresolved_member_call_unavailable_message(method);
            return self.deny_e5506(function, &message);
        }
```

If `callee_node` is an owned `LirNode` at this point, pass `&callee_node`. If it's already a reference, pass `callee_node`. Look at the gate-1 call two blocks above for the form used.

- [ ] **Step 6: Run the tests and confirm they pass.**

Run: `cargo test -p kali_codegen member_provenance && cargo test -p kali_codegen url`
Expected: PASS. If a `a_host_root_keeps_its_lowering` row fails, print that row's diagnostics. Fix the predicate, not the test, unless the row is wrong about node. In that case, record it for Task 8.

- [ ] **Step 7: Commit.**

```bash
git add crates/kali_codegen/src/emit/member_provenance.rs crates/kali_codegen/src/emit/member_provenance_tests.rs crates/kali_codegen/src/emit/mod.rs crates/kali_codegen/src/emit/url.rs crates/kali_codegen/src/emit/call.rs crates/kali_codegen/src/emitter.rs crates/kali_codegen/src/lower.rs
git commit -m "feat(unresolved-member-call): kali run refuses an unresolved member call on a program-owned receiver"
```

---

### Task 5: The `check` mirror

**Files:**
- Modify: `crates/kali_types/src/context.rs`. Add two fields to `TypeContext`:
  - `pub(crate) program_classes: Option<crate::program_classes::ProgramClasses>`
  - `pub(crate) assigned_property_names: std::collections::BTreeSet<String>`
- Modify: `crates/kali_types/src/resolve/mod.rs:350`, next to `self.repr_table = crate::repr_infer::infer_reprs(statements);`
- Modify: `crates/kali_types/src/resolve/member.rs`, after `reject_array_mutator_member` (about line 475)
- Modify: `crates/kali_types/src/resolve/expression.rs:2844` and `crates/kali_types/src/resolve/member.rs:454` (call the new check)
- Test: `crates/kali_types/src/resolve/member_tests.rs`

**Interfaces:**
- Consumes:
  - `ProgramClasses::{collect, is_program_class, member_names}` and `assigned_property_names` (Task 3)
  - `kali_common::{OBJECT_PROTOTYPE_NAMES, unresolved_member_call_unavailable_message}` (Task 1)
  - `super::expression::unwrap_transparent`
  - the scope walk pattern of `resolve_array_literal_binding_name` (`static_analysis/array.rs:433`)
- Produces:
  - `pub(crate) fn reject_unresolved_member_call(&mut self, member: &MemberExpression)`
  - `fn known_member_set(&self, object: &Expression) -> Option<BTreeSet<String>>`
  - per-scope `const_member_receivers: IndexMap<String, MemberReceiver>` on `Scope`, where `enum MemberReceiver { ObjectLiteral(BTreeSet<String>), ClassInstance(String) }`

- [ ] **Step 1: Write the failing tests.** Append this to `member_tests.rs`:

```rust
const UNRES: &str = "the receiver is a value this program built";

fn unres_count(source: &str) -> usize {
    e5506_messages(source).iter().filter(|m| m.contains(UNRES)).count()
}

#[test]
fn check_refuses_a_missing_method_on_a_const_object_literal_or_program_instance() {
    for source in [
        "const o={k:1}; console.log(o.zork(4));",
        "const o={k:1}; console.log(o[\"zork\"](4));",
        "function main(){ const o={k:1}; console.log(o.zork()); } main();",
        "class C{ f(){return 1;} } const c=new C(); console.log(c.g());",
        "class A{ f(){return 1;} } class B extends A{} const b=new B(); console.log(b.g());",
        "const o={k:1}; o.zork?.();",
    ] {
        assert_eq!(unres_count(source), 1, "{source}");
    }
}

#[test]
fn check_stays_quiet_where_it_cannot_know() {
    for source in [
        // members that exist, on the class or its program base
        "class S { push(v){ return v+1; } } const s=new S(); console.log(s.push(1));",
        "class A{ f(){return 4;} } class B extends A{} const b=new B(); console.log(b.f());",
        "class S { n = 0; cb = () => 1; } const s=new S(); console.log(s.cb());",
        // Object.prototype names
        "const o={k:1}; console.log(o.hasOwnProperty(\"k\"));",
        // a name some assignment writes
        "const o={k:1}; o.f = 5; console.log(o.f());",
        "class S { constructor(){ this.cb = 1; } } const s=new S(); console.log(s.cb());",
        // the base leaves the program
        "class X extends EventTarget{} const x=new X(); x.addEventListener(\"t\", ()=>{});",
        // receivers outside the mirror (run-only, spec §1.1)
        "const s=\"abc\"; console.log(s.zork());",
        "const o={k:1}; const p=o; console.log(p.zork());",
        "let o={k:1}; console.log(o.zork());",
        "function g(x){ return x.zork(); }",
    ] {
        assert_eq!(unres_count(source), 0, "{source}");
    }
}

#[test]
fn the_nearest_binding_wins() {
    // Outer: a class instance that has `zork`. Inner: an object literal that does not.
    let inner_lacks = "class S { zork(){ return 1; } } const o=new S(); function main(){ const o={k:1}; console.log(o.zork()); } main();";
    assert_eq!(unres_count(inner_lacks), 1);
    // Outer: an object literal without `zork`. Inner: a class instance that has it.
    let inner_has = "const o={k:1}; class T { zork(){ return 1; } } function main(){ const o=new T(); console.log(o.zork()); } main();";
    assert_eq!(unres_count(inner_has), 0);
}

#[test]
fn a_literal_array_mutator_gets_one_diagnostic_not_two() {
    let messages = e5506_messages("const a=[1,2,3]; a.pop(); console.log(a.length);");
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(!messages[0].contains(UNRES));
}
```

- [ ] **Step 2: Run them and confirm they fail.**

Run: `cargo test -p kali_types check_refuses_a_missing_method check_stays_quiet the_nearest_binding_wins a_literal_array_mutator_gets_one`
Expected: `check_refuses_…` and `the_nearest_binding_wins` fail (count `0`, want `1`). The others pass.

- [ ] **Step 3: Record receivers in scope.** In `crates/kali_types/src/scope.rs`, add this beside `array_literal_bindings`:

```rust
    /// `const` bindings whose member set the `check` mirror of the
    /// unresolved-member-call gate can know (spec §3.3).
    pub(crate) const_member_receivers: IndexMap<String, MemberReceiver>,
```

Add the enum in the same file:

```rust
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MemberReceiver {
    /// An object literal; the set is its keys.
    ObjectLiteral(std::collections::BTreeSet<String>),
    /// `new C(…)`; the class name, resolved through `ProgramClasses`.
    ClassInstance(String),
}
```

Initialize it with `IndexMap::new()` in the scope constructor (next to `array_literal_bindings: IndexMap::new()`, line 111). Also remove it in `invalidate_static_binding` (next to line 142), using `self.const_member_receivers.shift_remove(name);`.

In `resolve_variable_declaration` (`resolve/mod.rs`), put this next to the array-literal recording block at about line 941. Use the same `target_scope` / `global_scope` pattern:

```rust
                if declaration.kind == "const" {
                    let receiver = match super::expression::unwrap_transparent(init) {
                        Expression::ObjectExpression(object) => Some(MemberReceiver::ObjectLiteral(
                            object.properties.iter().filter_map(|p| match &p.key {
                                PropertyName::Identifier(name) | PropertyName::String(name) => Some(name.clone()),
                                PropertyName::Number(n) => Some(kali_common::js_number::format(*n)),
                                PropertyName::BigInt(digits) => Some(digits.clone()),
                            }).collect(),
                        )),
                        Expression::NewExpression(new_expr) => match &new_expr.callee {
                            Expression::Identifier(class_name) => Some(MemberReceiver::ClassInstance(class_name.clone())),
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(receiver) = receiver {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.const_member_receivers.insert(declarator.id.clone(), receiver);
                        } else if self.global_scope.contains(&declarator.id) {
                            self.global_scope.const_member_receivers.insert(declarator.id.clone(), receiver);
                        }
                    }
                }
```

Use whichever number-to-string helper `kali_common::js_number` exports (grep it for `pub fn`). A numeric key never names a method in practice, so if no helper fits, `n.to_string()` is acceptable. If `Expression::NewExpression` is boxed, match `Expression::NewExpression(new_expr)` and deref accordingly. Add `use crate::scope::MemberReceiver;` and `kali_ast::PropertyName` imports as needed.

- [ ] **Step 4: Hold the program-wide facts.** In `resolve/mod.rs` at line 350, after `self.repr_table = …`:

```rust
        self.program_classes = Some(crate::program_classes::ProgramClasses::collect(statements));
        self.assigned_property_names = crate::program_classes::assigned_property_names(statements);
```

Add the two fields to `TypeContext` in `context.rs`, with `None` and `BTreeSet::new()` in its constructor.

- [ ] **Step 5: Implement the check.** In `resolve/member.rs`, after `reject_array_mutator_member`:

```rust
    /// The `check` mirror of the unresolved-member-call gate (spec §3.3):
    /// a method name missing from a `const` object literal's keys or a
    /// program class chain's members. Silent wherever the member set is
    /// unknown; `kali run` refuses those (spec §1.1).
    pub(crate) fn reject_unresolved_member_call(&mut self, member: &MemberExpression) {
        let Some(method) = member.property.as_deref() else {
            return;
        };
        if kali_common::OBJECT_PROTOTYPE_NAMES.contains(&method)
            || self.assigned_property_names.contains(method)
        {
            return;
        }
        let Some(members) = self.known_member_set(&member.object) else {
            return;
        };
        if members.contains(method) {
            return;
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            kali_common::unresolved_member_call_unavailable_message(method),
        ));
    }

    fn known_member_set(&self, object: &Expression) -> Option<std::collections::BTreeSet<String>> {
        let Expression::Identifier(name) = super::expression::unwrap_transparent(object) else {
            return None;
        };
        let receiver = self.nearest_const_member_receiver(name)?;
        match receiver {
            MemberReceiver::ObjectLiteral(keys) => Some(keys.clone()),
            MemberReceiver::ClassInstance(class_name) => {
                let classes = self.program_classes.as_ref()?;
                if !classes.is_program_class(class_name) {
                    return None;
                }
                classes.member_names(class_name)
            }
        }
    }

    /// The nearest binding of `name`, if it is a recorded member receiver.
    /// Stops at the first scope that binds the name (literal-array-mutators
    /// A-10's rule).
    fn nearest_const_member_receiver(&self, name: &str) -> Option<&MemberReceiver> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id).expect("scope exists");
            if let Some(receiver) = scope.const_member_receivers.get(name) {
                return Some(receiver);
            }
            if scope.contains(name) {
                return None;
            }
            current = scope.parent;
        }
        self.global_scope.const_member_receivers.get(name)
    }
```

- [ ] **Step 6: Call it once per member.** In `reject_runtime_array_mutator_call` (`member.rs:450-456`) and at `resolve/expression.rs:2844`, replace `self.reject_array_mutator_member(member);` with:

```rust
            let before = self.diagnostics.len();
            self.reject_array_mutator_member(member);
            if self.diagnostics.len() == before {
                self.reject_unresolved_member_call(member);
            }
```

- [ ] **Step 7: Run the tests and confirm they pass.**

Run: `cargo test -p kali_types`
Expected: PASS, including every existing test.

- [ ] **Step 8: Commit.**

```bash
git add crates/kali_types/src
git commit -m "feat(unresolved-member-call): kali check mirrors the refusal for const object-literal and program-class receivers"
```

---

### Task 6: CLI cases and the probe diff

**Files:**
- Create: `crates/kali_cli/tests/cases/soundness/unresolved_member_call.toml`

**Interfaces:**
- Consumes: the Task 0 probes and the Task 4 and Task 5 behaviour.

- [ ] **Step 1: Write the case file.** Each `[source]` file is one program. Use `[constants] UNRES = "the receiver is a value this program built"`. For every row below, write one `[[case]]` per listed command, with this rationale form:

`"""At \`6f042548f\` kali printed <kali> at exit 0 where node v26.10.0 <node>."""`

Use `exit = "failure"`, `stderr_contains = ["E5506", "${UNRES}"]` and `stdout = ""` for refusals. Use `exit = "success"` and `stdout = "<node output>\n"` for controls.

| source file | program | commands | expect | rationale facts (kali at baseline / node) |
|---|---|---|---|---|
| `objlit.js` | `const o={k:1}; console.log(o.zork(4));` | check, run | refuse | `0` / throws `TypeError: o.zork is not a function` |
| `strkey.js` | `const o={k:1}; console.log(o["zork"](4));` | run | refuse | `0` / throws |
| `inmain.js` | `function main(){ const o={k:1}; console.log(o.zork()); } main();` | check, run | refuse | `0` / throws |
| `inst.js` | `class C{ f(){return 1;} } const c=new C(); console.log(c.g());` | check, run | refuse | `0` / throws |
| `inst_extends.js` | `class A{ f(){return 1;} } class B extends A{} const b=new B(); console.log(b.g());` | check, run | refuse | `0` / throws |
| `deep.js` | `const o={a:{b:{}}}; console.log(o.a.b.zork());` | run | refuse | `0` / throws |
| `str.js` | `const s="abc"; console.log(s.zork());` | run | refuse | `0` / throws |
| `alias.js` | `const o={k:1}; const p=o; console.log(p.zork());` | run | refuse | `0` / throws |
| `param.js` | `function g(x){ return x.zork(); } const o={k:1}; console.log(g(o));` | run | refuse | `0` / throws |
| `fnres.js` | `function mk(){ return {k:1}; } const o=mk(); console.log(o.zork());` | run | refuse | `0` / throws |
| `hasown.js` | `const o={k:1}; console.log(o.hasOwnProperty("k"));` | run | refuse | `0` / `true` |
| `call_push.js` | `const a=[1,2,3]; a.push.call(a, 4); console.log(a.length);` | run | refuse | `3` / `4` |
| `proto_pop.js` | `const a=[1,2,3]; Array.prototype.pop.call(a); console.log(a.length);` | run | refuse | `3` / `2` |
| `ok_extends.js` | `class A{ f(){return 4;} } class B extends A{} const b=new B(); console.log(b.f());` | check, run | `4` | `4` / `4` |
| `ok_et.js` | `class X extends EventTarget{} const x=new X(); x.addEventListener("t", ()=>{}); console.log("ok");` | check, run | `ok` | `ok` / `ok` |
| `ok_usp.js` | `const u=new URLSearchParams("a=1"); u.append("b","2"); console.log(u.toString());` | run | `a=1&b=2` | same / same |
| `ok_host_let.js` | `let t=globalThis.performance; t.now(); console.log("ok");` | run | `ok` | `ok` / `ok` |
| `ok_free_unused.js` | `performance.now(); console.log("ok");` | run | `ok` | `ok` / `ok` |
| `ok_inst_method.js` | `class S { push(v){ return v+1; } } const s=new S(); console.log(s.push(1));` | check, run | `2` | `2` / `2` |

For `check` controls, use `exit = "success"` and no `stdout` assertion. Look at an existing `check` success case in `cases/array/literal_array_mutators.toml` and copy its form. Name cases in the existing style: `a_missing_method_on_a_const_object_literal_refuses_under_run`, `a_method_inherited_from_a_program_base_still_matches_node`, and so on.

Before writing a control's `stdout`, verify the baseline facts against the Task 0 baseline TSV, which has the same programs, and with `node <file>`. Put a row that disagrees with this table in the Task 8 followups file, and quote the measured value.

- [ ] **Step 2: Run the cases.**

Run: `cargo build -q -p kali_cli && cargo test -p kali_cli --test cases -- soundness/unresolved_member_call`
Expected: PASS.

- [ ] **Step 3: Diff the probes.**

Run:
```bash
tools/array-return-probes/run.sh /tmp/claude-probes-head.tsv
grep '^unres_' /tmp/claude-probes-head.tsv > /tmp/claude-unres-head.tsv
diff <(cut -f1,2,5 tools/array-return-probes/baseline-unres.tsv) <(cut -f1,2,5 /tmp/claude-unres-head.tsv)
```
Expected, as spec §5.1's pass conditions:
- every non-`ok` `unres_*` row is `REFUSES`
- check is `1` on `objlit`, `strkey`, `inmain`, `inst` and `inst_extends`
- no `unres_ok_*` row moved

Then compare every older baseline row between the baseline binary and HEAD, the way literal-array-mutators followups §4.b did:
```bash
git worktree add /tmp/claude-base 6f042548f
(cd /tmp/claude-base && cargo build -q -p kali_cli)
KALI=/tmp/claude-base/target/debug/kali tools/array-return-probes/run.sh /tmp/claude-probes-base.tsv
diff <(grep -v '^unres_' /tmp/claude-probes-base.tsv | cut -f1,2,5) <(grep -v '^unres_' /tmp/claude-probes-head.tsv | cut -f1,2,5)
```
Expected: no output. Record every line that does appear for Task 7. Keep the worktree, because Task 7 uses the same base binary; remove it with `git worktree remove /tmp/claude-base` at the end of Task 8.

- [ ] **Step 4: Commit.**

```bash
git add crates/kali_cli/tests/cases/soundness/unresolved_member_call.toml
git commit -m "test(unresolved-member-call): pin refusals, the check mirror and the host controls as cases"
```

---

### Task 7: Blast radius, triage and re-pins

**Files:**
- Modify: whichever existing `.toml` cases and `*_tests.rs` tests move. Classify every one before touching it.

**Interfaces:**
- Produces: the triage table Task 8 files.

- [ ] **Step 1: Run everything.**

Run: `cargo test --workspace 2>&1 | tee /tmp/claude-ws.log | grep -E "^test result|FAILED|failed" | head -40`
Then: `cargo test -p kali_cli --test cases 2>&1 | tee /tmp/claude-cases.log | tail -30`

- [ ] **Step 2: Apply the stop rule.** Count the failing tests. **If more than 50 fail, stop.** Don't re-pin anything. Report the count and the first 20 names, grouped by the callee name in the new E5506 message, to the human partner.

- [ ] **Step 3: Triage each moved test.** Build a table in the form literal-array-mutators followups §4.ta uses:

| name | before | after | class |
|---|---|---|---|

The class is one of these:
- **wanted:** the test pinned a silent `0` or a dropped call.
- **capability loss:** the test passed with node's output before and now refuses. Say which spec §5.4 class it is in: effect never observed, or dead code.
- **wrong:** the gate fired on something it shouldn't have. That's a bug: fix the predicate in Task 4's file, add a unit test for the shape to `member_provenance_tests.rs`, and re-run.

For a capability loss outside both §5.4 classes, stop and bring it to the human partner.

For each test you need the "before" behaviour. Run the single test in the `/tmp/claude-base` worktree from Task 6.

- [ ] **Step 4: Re-pin the wanted moves.** Keep each case's original rationale and add this paragraph:

> Re-pinned 2026-10-03 by the unresolved-member-call project: at `6f042548f` this printed <x> at exit 0; node v26.10.0 prints <y>; kali now refuses with E5506 (spec docs/superpowers/specs/2026-10-03-unresolved-member-call-design.md §3.1).

Set `exit = "failure"` and add `stderr_contains = ["E5506", "the receiver is a value this program built"]`. If a case file is GENERATED (its header names a generator under `tools/migration/`), hand-edit it with a dated `HAND-EDITED` header note, as the `runtime/join.toml` precedent does.

- [ ] **Step 5: Re-run until green.**

Run: `cargo test --workspace && cargo test -p kali_cli --test cases`
Expected: PASS.

- [ ] **Step 6: Commit.**

```bash
git add -A crates
git commit -m "test(unresolved-member-call): re-pin cases that pinned a silent unresolved member call"
```

---

### Task 8: Docs

**Files:**
- Modify: `specs/15-errors.md`, in the E5506 entry. Find it with `grep -n "E5506" specs/15-errors.md | head`.
- Modify: `docs/superpowers/followups/literal-array-mutators-discovered-defects.md` §3 and §12
- Create: `docs/superpowers/followups/unresolved-member-call-discovered-defects.md`
- Modify (only if a §0.2 lane moved): `docs/superpowers/followups/kali-silent-miscompile-register.md`

- [ ] **Step 1: Add the `specs/15` scope line.** Under E5506's list of covered constructs, add this, matching the entry's existing bullet style:

```markdown
- a member call on a receiver the program built (an object, array, string or number value, a program-class instance, or a binding derived from one) when kali has no lowering for that method, including `.call` / `.apply` spellings and `Array.prototype.m.call(…)`; `kali check` reports it for a `const` object literal or program-class instance whose member set lacks the name (unresolved-member-call spec §3)
```

- [ ] **Step 2: Mark the items fixed.** In `literal-array-mutators-discovered-defects.md`, add a line under the §3 heading and under the §12 heading:

```markdown
**Fixed** by the unresolved-member-call project (`docs/superpowers/specs/2026-10-03-unresolved-member-call-design.md`), at `<commit of Task 4>`: `kali run` refuses with E5506, and `kali check` mirrors the refusal for a `const` object-literal or program-class receiver.
```

- [ ] **Step 3: Write the new followups file.** Use the header convention of `literal-array-mutators-discovered-defects.md`: Filed, Oracle, Measured at, Register. Then add these sections:
  1. **The `check` / `run` gap**: the run-only rows of the Task 6 table, with each row's measured check exit.
  2. **Host-provenance silences**: `unres_ok_host_alias` and `unres_ok_import`, with node output, kali output, and the reason (spec §1.1, A-2).
  3. **The probe diff**: the Task 6 Step 3 output, verbatim.
  4. **The triage table**: from Task 7 Step 3.
  5. **Measured capability loss**: each loss with its §5.4 class. If there are none, write "None measured" and say what was swept.
  6. **Anything else** measured and not fixed: every row recorded during Tasks 0-7.

- [ ] **Step 4: Check the register.** Run `cargo test -p kali_cli --test cases -- oracle/` and compare each oracle case's verdict with the register's §0.2 lanes. Edit the register only if a lane moved, and say so in the new followups file's Register line either way.

- [ ] **Step 5: Re-read for consistency.** Read spec §1, §1.1 and §3, the new followups file and the `specs/15` line together. Fix any claim that disagrees with what was measured. If behaviour differs from the spec, add an amendment (A-4 onward) to spec §6; don't quietly edit an earlier section.

- [ ] **Step 6: Final verification.**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo test -p kali_cli --test cases`
Expected: all pass. If clippy flags code this project didn't touch, record it and don't fix it.

- [ ] **Step 7: Commit.**

```bash
git add specs/15-errors.md docs/superpowers
git commit -m "docs(unresolved-member-call): E5506 scope, close followups §3/§12, file what was measured and not fixed"
```
