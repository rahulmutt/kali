//! `switch` lowering: admittance plan + emit.
//!
//! The plan is built from POSITIVE evidence only. `SwitchPlan::build` returns
//! `Err(reason)` unless it can prove every part of the switch is in the
//! admitted set, and `emit_switch` denies on `Err`. There is deliberately no
//! denylist of bad shapes anywhere in this file: this repository's most
//! repeated lesson is that a denylist of shapes leaks forever and only an
//! allowlist at the choke point closes a class (Spec 4a needed six rounds
//! before a default-deny at the single read site closed the for-in-key class
//! by construction).
//!
//! Extending the admitted set therefore means adding a proof to `build`, never
//! removing a rejection.

use crate::emit::equality::EqClass;
use crate::*;

/// How a clause body ends. Only terminators in this enum are admitted; a
/// clause that ends any other way is true fallthrough and is denied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClauseTerminator {
    /// The clause's last statement is `return`.
    Return,
    /// The clause's last statement is an unlabeled `break`.
    Break,
    /// The clause has no statements at all and groups onto the next clause.
    EmptyGroup,
}

/// One admitted clause.
pub(crate) struct SwitchClause {
    /// `None` for the `default` clause.
    pub(crate) test: Option<LirNodeId>,
    pub(crate) body: LirNodeId,
    pub(crate) terminator: ClauseTerminator,
}

/// A switch this emitter has proven it can lower correctly.
pub(crate) struct SwitchPlan {
    pub(crate) discriminant: LirNodeId,
    pub(crate) clauses: Vec<SwitchClause>,
}

impl<'a> FunctionEmitter<'a> {
    /// Build a plan, or explain why this switch is not admitted.
    ///
    /// Task 7 admits exactly one shape: a PROVEN i64 discriminant, numeric-
    /// literal (optionally unary `+`/`-`) case tests, at most one `default`,
    /// and every clause ending in `return`. Tasks 8-10 widen this further by
    /// adding MORE proofs here — never by removing a rejection.
    pub(crate) fn switch_plan(&self, node: &LirNode) -> Result<SwitchPlan, String> {
        let mut children = node.children.iter().copied();
        let discriminant = children
            .next()
            .ok_or_else(|| "a switch with no discriminant".to_string())?;

        // Rule 1: the discriminant must be a PROVEN i64 scalar. Anything not
        // proven — float, boolean, object, array, unknown — is denied. Task 8
        // widens this to proven strings.
        if !self.is_provable_i64_scalar(discriminant) {
            return Err("the discriminant is not a proven integer".to_string());
        }

        let mut clauses = Vec::new();
        let mut default_seen = false;
        for case_id in children {
            let case = self.node(case_id);
            let is_default = match case.text.as_deref() {
                Some("case") => false,
                Some("default") => true,
                // Rule 3 of the allowlist is enforced by construction: an
                // untagged clause block cannot be classified, so it is denied.
                _ => return Err("an unclassifiable switch clause".to_string()),
            };
            if is_default {
                if default_seen {
                    return Err("more than one `default` clause".to_string());
                }
                default_seen = true;
            }

            // A "case" block's children are [test, stmts...]; a "default"'s
            // are [stmts...].
            let (test, stmts) = if is_default {
                (None, &case.children[..])
            } else {
                let test = *case
                    .children
                    .first()
                    .ok_or_else(|| "a `case` clause with no test".to_string())?;
                // Rule 2: the test must be a literal in the discriminant's
                // domain, including unary +/- on a numeric literal.
                if !self.is_numeric_literal_case_test(test) {
                    return Err("a `case` test that is not a numeric literal".to_string());
                }
                (Some(test), &case.children[1..])
            };

            // Rule 4: this task admits ONLY `return`-terminated clauses.
            // Empty grouping arrives in Task 10, `break` in Task 9. Anything
            // else is true fallthrough and stays denied.
            let terminator = match stmts.last() {
                Some(last) if self.is_return_statement(*last) => ClauseTerminator::Return,
                _ => {
                    return Err(
                        "a clause that does not end in `return` (true fallthrough is not \
                         in the supported lowering set)"
                            .to_string(),
                    )
                }
            };

            // Rule 5: `let`/`const` in a clause body is denied — block
            // shadowing is unmodeled (register R-10), so a case-scoped binding
            // would build on a known-broken foundation. `var` is
            // function-scoped and admitted.
            if stmts.iter().any(|s| self.declares_block_scoped_binding(*s)) {
                return Err("a `let`/`const` declaration in a clause body".to_string());
            }

            clauses.push(SwitchClause {
                test,
                body: case_id,
                terminator,
            });
        }

        if clauses.is_empty() {
            return Err("a switch with no clauses".to_string());
        }
        Ok(SwitchPlan {
            discriminant,
            clauses,
        })
    }

    /// PROOF that `id`'s emitted value is a genuine, non-aggregate i64 scalar
    /// — never a float, a string, a boolean, or an object/array handle (every
    /// one of those is ALSO an i64-shaped wasm value in this compiler's
    /// representation — see `kali_common::Repr` — so the wasm validator
    /// cannot catch a wrongly-admitted one; only this proof stands between a
    /// `switch (obj)` and a silent handle-as-integer miscompile).
    ///
    /// Deliberately composed from the SAME oracles the rest of this emitter
    /// already uses to pick instruction shape, rather than re-deriving repr
    /// inference by hand:
    /// - `is_float_valued` / `is_string_valued` (`emit/operators.rs`) — the
    ///   float and string proofs every `+`/`<`/`===` operand already goes
    ///   through,
    /// - `object_shape_of_node` (`emit/object.rs`) — the fixed-shape object
    ///   proof,
    /// - `static_equality_class` (`emit/equality.rs`) — the ONLY existing
    ///   proof that a value is provably a JS boolean (a plain `Repr::I64`
    ///   scalar carries no boolean axis of its own; see that module's doc
    ///   comment on why `EqClass::Boolean` is the one class that can tell a
    ///   `true`/`false`/comparison/`!`/`delete` result apart from a genuine
    ///   number sharing the same `0`/`1` bit pattern) — NOTE this only proves
    ///   a SYNTACTIC boolean form (a literal, a comparison, `!`, `delete`); a
    ///   bare identifier bound to a boolean elsewhere is NOT caught here, it
    ///   is caught by the identifier arm's POSITIVE-proof requirement below,
    /// - `bitwise_compound_rhs_is_provably_i64` (`emit/operators.rs`) — the
    ///   literal (optionally unary `-`) proof, reused verbatim rather than
    ///   re-derived (One query, one caller),
    /// - the identifier arm mirrors the TWO-source disjunction
    ///   `string_coercion_arg_is_proven_at_depth` / the coercion-sink call
    ///   site (`emit/call.rs:4146`) uses — `name_is_declared_parameter(name)
    ///   || binding_is_proven_numeric(name)` — narrowed from that sink's
    ///   `I64 | F64` union down to `I64` only, since a float discriminant
    ///   must be denied here,
    /// - the call arm mirrors the STRICT sibling arm at `emit/call.rs:4230-4242`
    ///   (`return_is_proven_numeric(name) && return_repr(name) == Repr::I64`),
    ///   not `is_string_valued`'s own lenient call arm — fix round 1: the
    ///   lenient `return_repr(..) == I64` alone is a comparison against
    ///   `Repr`'s `#[default]` (`kali_common::repr.rs:18-21`), which
    ///   `emit/call.rs:4220-4229` and `emit/operators.rs:1522-1545` (the doc
    ///   of `bitwise_compound_rhs_is_provably_i64` itself) both already rule
    ///   out for exactly this reason — measured to admit a boolean-returning
    ///   function's result as a switch discriminant (`g_boolret`) before this
    ///   fix. The evaluate-once test's fixture was rewritten
    ///   (`switch (d(2))`, not `switch (d(x))`) so `d`'s own parameter gets
    ///   literal call-site inflow and this stricter proof actually holds —
    ///   weakening the proof to fit the old fixture would have been the wrong
    ///   direction of fix.
    pub(crate) fn is_provable_i64_scalar(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);

        if self.is_float_valued(id) || self.is_string_valued(id) {
            return false;
        }
        if self.object_shape_of_node(id).is_some() {
            return false;
        }
        if matches!(self.static_equality_class(id), Some(EqClass::Boolean)) {
            return false;
        }

        let node = self.node(id);
        // Unary `+`: `bitwise_compound_rhs_is_provably_i64` only recognizes
        // `-` for its own RHS shape, so a leading `+` is peeled here and the
        // operand re-enters this same proof.
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref() == Some("+")
        {
            return self.is_provable_i64_scalar(node.children[0]);
        }
        // The literal (optionally unary `-`) proof: delegate to the EXISTING
        // oracle instead of re-deriving it.
        if self.bitwise_compound_rhs_is_provably_i64(id) {
            return true;
        }

        match node.kind {
            // Bare identifier: proven i64 iff it carries none of the
            // aggregate taints that SHARE the default `Repr::I64` (an array,
            // a growable array, a non-scalar param, an object-initialized
            // binding), is EITHER a declared parameter OR positively proven
            // numeric by `repr_infer` (never trusted on the unrecorded
            // default alone), and its own solved repr IS the plain `I64`
            // default — never `F64`/`String`/`Object`/any other tagged
            // handle variant.
            LirNodeKind::Value if node.children.is_empty() => node
                .text
                .as_deref()
                .is_some_and(|name| self.identifier_is_provable_i64_scalar(name)),
            // Call to a program function whose return is POSITIVELY proven
            // numeric AND whose solved return repr is the plain `I64`
            // default — the exact conjunction `emit/call.rs:4230-4242` uses.
            LirNodeKind::Call => node
                .children
                .first()
                .map(|&callee| self.unwrap_transparent(callee))
                .and_then(|callee| self.node(callee).text.clone())
                .is_some_and(|name| {
                    self.functions.contains_key(&name)
                        && self.repr_table.return_is_proven_numeric(&name)
                        && self.repr_table.return_repr(&name) == kali_common::Repr::I64
                }),
            _ => false,
        }
    }

    /// `name` resolution + aggregate-taint checks mirror
    /// `binding_is_proven_string_coercion_scalar` (`emit/call.rs`), narrowed
    /// to `I64` only (that coercion sink accepts `I64 | F64`; a switch
    /// discriminant must not, since a float must be denied).
    ///
    /// Fix round 1: restores the `name_is_program_bound` gate that
    /// sink's own first line applies (denies `globalThis` / `undefined` /
    /// `NaN` / every other free global — dropping it let `globalThis` reach
    /// the unrecorded-`Repr::I64`-default fallthrough below and be admitted;
    /// measured as the `j_globalthis` differential), and replaces the bare
    /// `scalar(func, name) == Repr::I64` comparison (a comparison against
    /// `Repr`'s `#[default]`, not positive evidence — the exact defect
    /// `emit/call.rs:4220-4229` and `bitwise_compound_rhs_is_provably_i64`'s
    /// own doc both name) with a two-source positive-evidence gate:
    /// `binding_is_proven_numeric(name)` for a non-parameter identifier
    /// (an explicit `repr_infer` numeric-write proof, which a boolean write
    /// like `var d = true;` never earns — closing the `cell16` differential),
    /// OR, for a parameter, `param_has_numeric_literal_inflow(func, name)`.
    ///
    /// Fix round 2 (human ruling): a bare `name_is_declared_parameter(name)`
    /// trust — with NO further evidence — is what the coercion sink's own
    /// call site (`emit/call.rs:4146`) uses, and round 1 mirrored it
    /// verbatim. That trust is NOT narrow enough for a switch discriminant:
    /// `kali_common::Repr` has no `Boolean` variant, so a boolean-valued
    /// parameter (`function s(b) { switch (b) {...} } s(true)`) carries
    /// nothing to distinguish it from a genuinely numeric one, and every
    /// existing per-parameter taint (`is_non_scalar_param`, the
    /// `arg_scalar_syntactic` evidence behind `binding_is_proven_numeric`'s
    /// own writes) treats a boolean/string primitive as "scalar", not
    /// specifically "numeric" — measured to admit `s(true)` and print
    /// `v=one` where node prints `v=other`, both exit 0. Narrowed here to
    /// `param_has_numeric_literal_inflow` (`kali_types::repr_infer`'s
    /// `numeric_literal_inflow_params`): every ENUMERATED call site of the
    /// enclosing function supplies a numeric literal for this parameter, AND
    /// the function itself never escapes as a value (assigned to a
    /// variable, returned, passed as a callback, exported) — an escaping
    /// function could be invoked through a call site this proof cannot
    /// enumerate, which would make "every enumerated site" unsound. A
    /// function with zero enumerated call sites is excluded too (no
    /// evidence is not vacuous truth).
    ///
    /// Fix round 4 closed the two remaining ways a non-numeric value reached
    /// this proof, both measured as silent miscompiles (exit 0, empty stderr,
    /// wrong answer) at HEAD `83a401c311`:
    ///
    /// - a `new s(true)` INVOCATION was invisible to both halves of the
    ///   parameter proof — it builds no `CallEdge`, and `repr_infer`'s
    ///   `visit_expr` never visited a `NewExpression`'s callee at all, so it
    ///   raised no escape mark either. Closed in `kali_types::repr_infer` by
    ///   routing that callee through `visit_expr`, the one walker that
    ///   reaches every identifier read;
    /// - `param_has_numeric_literal_inflow` proved what flowed IN and was
    ///   consumed here as a proof of the discriminant's value AT THE SWITCH,
    ///   so any write to the parameter first (`x = true`, `x = y`, `x = b()`,
    ///   `x += 1`, `x++`, a `for (x of …)` binding, …) laundered through.
    ///   Closed in `kali_types::repr_infer` by conjoining `readonly_params`
    ///   into that proof — a POSITIVE enumeration of the forms in which a
    ///   name is provably only read, where every unenumerated form denies —
    ///   so the inflow proof is now a proof of VALUE. Fixed at the source
    ///   rather than by a second check here so every future consumer of
    ///   `param_has_numeric_literal_inflow` inherits the honest meaning; a
    ///   codegen-side conjunct would have left the misleading name in place
    ///   for the next caller to trip over.
    fn identifier_is_provable_i64_scalar(&self, name: &str) -> bool {
        if !self.name_is_program_bound(name) {
            return false;
        }
        let func: &str = if !self.locals.contains_key(name) && self.function_name != "_start" {
            "_start"
        } else {
            &self.function_name
        };
        if self.repr_table.is_array_binding(func, name)
            || self.repr_table.is_growable_array_binding(func, name)
            || self.repr_table.is_non_scalar_param(func, name)
            || self.repr_table.object_initialized_binding(func, name)
        {
            return false;
        }
        // A parameter is trusted ONLY when narrowed by the numeric-literal
        // proof (never bare `name_is_declared_parameter` alone — fix round
        // 2), and that is now its ONLY admitted proof (fix round 4): the
        // disjunction that also let a parameter qualify through
        // `binding_is_proven_numeric` was itself a leak. That proof accepts a
        // write whose RHS is another PARAMETER, gated only on that
        // parameter's SCALAR inflow (`param_lacks_scalar_inflow`, see
        // `repr_infer`'s `numeric_bindings` finalize step) — and a boolean
        // literal argument is scalar-syntactic, so
        // `function s(x, y) { x = y; switch (x) {…} } s(1, true)` earned the
        // numeric-binding proof for `x` with a boolean in it, measured
        // printing `v=one` against node's `v=other` at exit 0. Making this an
        // if/else rather than an `||` means: for a parameter, the ONLY
        // evidence is "every call site passed a numeric literal AND the body
        // never writes it" (`param_has_numeric_literal_inflow`, which since
        // fix round 4 carries the never-written conjunct — see
        // `kali_types::repr_infer`'s `readonly_params`). A parameter that IS
        // written is denied outright, which is strictly narrower: a written
        // parameter's value at the switch is whatever the writes put there,
        // and no proof in this compiler establishes that it is a NUMBER
        // rather than a same-repr boolean.
        let positively_proven = if self.name_is_declared_parameter(name) {
            self.repr_table.param_has_numeric_literal_inflow(func, name)
        } else {
            self.binding_is_proven_numeric(name)
        };
        if !positively_proven {
            return false;
        }
        self.repr_table.scalar(func, name) == kali_common::Repr::I64
    }

    /// A `case` test is admitted iff it is a numeric literal, optionally
    /// under a unary `+`/`-`. Delegates the literal (optionally unary `-`)
    /// proof entirely to `bitwise_compound_rhs_is_provably_i64`
    /// (`emit/operators.rs`) rather than re-deriving it (fix round 1: this
    /// used to duplicate that function's body inline, one of three
    /// independently-drifting copies) — widened only to also recognize
    /// unary `+`, which that predicate does not need for its own RHS shape
    /// but a `case +1:` test is equally a plain numeric literal.
    fn is_numeric_literal_case_test(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref() == Some("+")
        {
            return self.is_numeric_literal_case_test(node.children[0]);
        }
        self.bitwise_compound_rhs_is_provably_i64(id)
    }

    /// `id` is a `return` statement. The SAME node shape `emit_node`'s own
    /// dispatch (`emit/control_flow.rs`) and `lower.rs`'s
    /// `is_arrow_return_body` check key on: a `Branch` node tagged `"return"`.
    fn is_return_statement(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("return")
    }

    /// True when `id` (or, recursively, any statement nested inside a bare
    /// `{ ... }` block reached through `id`) is a `let`/`const` declarator.
    /// Reuses the SAME `Instruction` + `"let" | "var" | "const"` tagging
    /// convention `collect_function_locals`'s declarator-init walk
    /// (`lower.rs`) and the `for`-loop-left parse (`emit/control_flow.rs`)
    /// both already key on, narrowed to `let`/`const` — `var` is
    /// function-scoped and admitted, matching Rule 5's own text.
    fn declares_block_scoped_binding(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        if node.kind == LirNodeKind::Instruction
            && matches!(node.text.as_deref(), Some("let" | "const"))
        {
            return true;
        }
        if node.kind == LirNodeKind::Block {
            return node
                .children
                .iter()
                .any(|&child| self.declares_block_scoped_binding(child));
        }
        false
    }

    pub(crate) fn emit_switch(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        node: &LirNode,
    ) -> EmittedValue {
        match self.switch_plan(node) {
            Ok(plan) => self.emit_switch_plan(function, id, plan),
            Err(reason) => {
                let message = format!(
                    "this `switch` is not in the supported lowering set ({reason}); \
                     rewrite it as `if`/`else if` or use a supported switch shape \
                     (fail-closed)"
                );
                self.deny_e5506(function, &message)
            }
        }
    }

    /// Emit an admitted plan.
    fn emit_switch_plan(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        plan: SwitchPlan,
    ) -> EmittedValue {
        // Evaluate the discriminant EXACTLY ONCE into this switch's dedicated
        // local. A chain that re-emitted it per clause test would call `f`
        // once per clause in `switch (f(x))`.
        let ordinal = self.switch_ordinals[&id];
        let disc_local = self.locals[&crate::lower::switch_disc_local_name(ordinal)];
        let produced = self.emit_node(function, plan.discriminant, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::LocalSet(disc_local));

        self.emit_clause_chain(function, disc_local, &plan.clauses, 0);

        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// Nested if/else chain over the clauses from `index` onward. The `default`
    /// clause becomes the innermost `else`.
    ///
    /// A duplicate case test needs no rule: an if/else chain is first-match-
    /// wins by construction, which is the correct JS semantics. A `default` in
    /// a non-final position needs no rule either: once true fallthrough is
    /// denied, `default`'s position carries no semantics.
    fn emit_clause_chain(
        &mut self,
        function: &mut Function,
        disc_local: u32,
        clauses: &[SwitchClause],
        index: usize,
    ) {
        let Some(clause) = clauses.get(index) else {
            return;
        };
        let Some(test) = clause.test else {
            // The default clause: run it unconditionally at this depth.
            self.emit_clause_body(function, clause);
            return;
        };

        function.instruction(&Instruction::LocalGet(disc_local));
        let produced = self.emit_node(function, test, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I64Eq);

        self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_clause_body(function, clause);
        function.instruction(&Instruction::Else);
        self.emit_clause_chain(function, disc_local, clauses, index + 1);
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::If);
    }

    /// Emit a clause's statements, skipping a `case` clause's leading test
    /// child (a `default` clause has no test child).
    fn emit_clause_body(&mut self, function: &mut Function, clause: &SwitchClause) {
        let body = self.node(clause.body);
        let skip = usize::from(clause.test.is_some());
        let stmts: Vec<LirNodeId> = body.children.iter().copied().skip(skip).collect();
        for stmt in stmts {
            let produced = self.emit_node(function, stmt, false);
            if produced.produced {
                function.instruction(&Instruction::Drop);
            }
        }
    }
}
