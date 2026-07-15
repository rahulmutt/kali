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

`visit_expr`'s `_ => self.new_node()` catch-all (`repr_infer.rs`, arm documented
in place) does not recurse into general sub-expressions, so a fn-expr/arrow
nested in an `ArrayExpression`/`ObjectExpression` element, a ternary branch, or an
assignment RHS is registered by the three Phase-A walkers (shared
`descend_stmt_fns`/`descend_expr_fns`) but **not Phase-B-seeded** by walk 4.

**Sound today:** codegen never invokes a callback reached only through those
positions (`obj.f()`, `cbs[0]()`, `let f; f=function…; f()`, ternary-init are
silent no-ops, exit 0), so no reachable body goes unseeded — reviewer verified.

**Becomes an ACTIVE silent miscompile** when Stage C/D (closure capture /
deferred callbacks) make those call shapes invocable: a string-element growable
array in such a body would silently lower to i64. **Stage C/D MUST** route walk
4's fn-expr discovery through the exhaustive `descend_expr_fns` (or fail those
positions closed E5506) before enabling those call shapes. A tripwire comment is
planted at the catch-all arm so this cannot be missed.
