# Stage AB — follow-ups (from the Task 3 review)

Two Minor findings from the merged-AB repr_infer fix review (fix commit
`a889637d8`; tripwire `102b625d7`). Neither blocks Stage AB — both are recorded
for the final whole-branch review to triage and for Stage C/D to act on.

## F-AB-1 — pre-existing expression-bodied-arrow return-value silent miscompile

**Not introduced by Stage AB** (reviewer built base `ac592a3b4` and confirmed the
wrong output is byte-identical pre-fix). The Task 3 report's §5 claim that
expression-bodied arrows "fail closed (safe), not miscompile" is **false**:

```js
const h = (x) => x + "!";
console.log(h("hi"));   // node: "hi!"  |  kali: -9223354444668731390!  (exit 0)
```

A String handle returned from an expression-bodied arrow is read back as a raw
i64 — a silent wrong answer (reject-don't-miscompile violation). Block-bodied
returns are correctly wired (`return` arm → `return_node_for`); only the
expression-bodied-arrow return path is affected, and it is outside Stage AB's
required (block-bodied) cases. Fix in a future stage: seed/flow the arrow's
return-value repr for expression-bodied arrows, or fail closed E5506.

## F-AB-2 — latent walk-4 vs walks-1–3 lockstep divergence (tripwire planted)

Walk 4 (Phase B `visit_stmt`/`visit_expr`) rides its OWN recursion, not the
shared `descend_expr_fns` of walks 1–3, ending in `_ => self.new_node()`
(`repr_infer.rs`, arm documented in place). It has no `ArrayExpression`/
`ObjectExpression` arm, so a bare/generic array-or-object literal reaching that
`_` arm is not recursed into.

**Already seeded by walk 4 (NOT gaps — verified against the code):** the common
callback positions all reach the fn-expr/arrow arms — **call arguments**
(`arr.map(cb)`, `Kali.test(name, cb)`, `queueMicrotask(cb)`; `visit_call` visits
args via `visit_expr`), **ternary branches** (`ConditionalExpression`),
**assignment RHS** (`visit_assignment`), **declarator-init array elements**
(`note_array_init`), and **object-property values** (`record_object_literal`).

**The genuine unseeded positions (narrow, exotic):** a fn-expr inside an object
literal passed **directly as a call arg** (`foo({f: () => {…}})` — `visit_expr`'s
`_` arm), a **spread arg** (`foo(...[() => {}])`), a **tagged-template / yield /
optional-chain** operand, and a **bare or doubly-nested array literal**. These
are registered by walks 1–3 but not Phase-B-seeded by walk 4.

**Sound today:** codegen never INVOKES a callback reached only through those
exotic positions (silent no-ops, exit 0), so no reachable body goes unseeded —
reviewer-verified.

**Becomes an ACTIVE silent miscompile** when Stage C/D (closure capture /
deferred callbacks) make those shapes invocable: a string-element growable array
in such a body would silently lower to i64. **Stage C/D MUST** seed those
specific positions — via a dedicated Phase-B pass reaching ONLY what `visit_expr`
misses (a blanket re-descent through `descend_expr_fns` would double-visit the
already-seeded positions above and risk a spurious mixed-store E5506 regression)
— or fail them closed E5506, before enabling those shapes. A tripwire comment is
planted at the `_` arm so this cannot be missed. When Stage C/D lands, enforce the
lockstep mechanically: assert the `__kali_fn_N` set discovered by walks 1–3 equals
the set walk 4 seeds.
