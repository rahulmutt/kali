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
    ///   number sharing the same `0`/`1` bit pattern),
    /// - the identifier arm mirrors `binding_is_proven_string_coercion_scalar`
    ///   (`emit/call.rs`) narrowed from its `I64 | F64` union down to `I64`
    ///   only, since a float discriminant must be denied here (unlike that
    ///   coercion sink, which accepts either),
    /// - the call arm mirrors `is_string_valued`'s own call arm
    ///   (`return_repr(callee) == Repr::String`): the callee's SOLVED return
    ///   repr, not the stricter `return_is_proven_numeric` allowlist that
    ///   the bitwise-compound-assign axis requires — that stricter proof
    ///   requires literal-only call-site inflow and would deny
    ///   `switch (d(x))` where `x` is itself a proven-scalar parameter passed
    ///   through unchanged, the exact shape this task's "evaluate once" test
    ///   pins.
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
        match node.kind {
            LirNodeKind::Literal => node
                .text
                .as_deref()
                .is_some_and(|text| !text.ends_with('n') && parse_number_literal(text).is_some()),
            // Unary `-`/`+` over a proven operand stays on the scalar lane.
            LirNodeKind::Value
                if node.children.len() == 1
                    && matches!(node.text.as_deref(), Some("-") | Some("+")) =>
            {
                self.is_provable_i64_scalar(node.children[0])
            }
            // Bare identifier: proven i64 iff it carries none of the
            // aggregate taints that SHARE the default `Repr::I64` (an array,
            // a growable array, a non-scalar param, an object-initialized
            // binding) and its own solved repr IS the plain `I64` default —
            // never `F64`/`String`/`Object`/any other tagged handle variant.
            LirNodeKind::Value if node.children.is_empty() => node
                .text
                .as_deref()
                .is_some_and(|name| self.identifier_is_provable_i64_scalar(name)),
            // Call to a program function whose SOLVED return repr is the
            // plain `I64` default (never float/string/object/etc). See this
            // method's own doc for why the stricter `return_is_proven_numeric`
            // allowlist is not required here.
            LirNodeKind::Call => node
                .children
                .first()
                .map(|&callee| self.unwrap_transparent(callee))
                .and_then(|callee| self.node(callee).text.clone())
                .is_some_and(|name| {
                    self.functions.contains_key(&name)
                        && self.repr_table.return_repr(&name) == kali_common::Repr::I64
                }),
            _ => false,
        }
    }

    /// `name` resolution + aggregate-taint checks mirror
    /// `binding_is_proven_string_coercion_scalar` (`emit/call.rs`) exactly,
    /// narrowed to `I64` only (that coercion sink accepts `I64 | F64`; a
    /// switch discriminant must not, since a float must be denied).
    fn identifier_is_provable_i64_scalar(&self, name: &str) -> bool {
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
        self.repr_table.scalar(func, name) == kali_common::Repr::I64
    }

    /// A `case` test is admitted iff it is a numeric literal, optionally
    /// under a unary `+`/`-`. Mirrors `bitwise_compound_rhs_is_provably_i64`
    /// (`emit/operators.rs`) exactly for the literal proof (including its
    /// explicit BigInt-suffix rejection), widened to also recognize unary
    /// `+` — that predicate only needs `-` for its own RHS shape, but a
    /// `case +1:` test is equally a plain numeric literal.
    fn is_numeric_literal_case_test(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && matches!(node.text.as_deref(), Some("-") | Some("+"))
        {
            return self.is_numeric_literal_case_test(node.children[0]);
        }
        node.kind == LirNodeKind::Literal
            && node
                .text
                .as_deref()
                .is_some_and(|text| !text.ends_with('n') && parse_number_literal(text).is_some())
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

        let frame = self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_clause_body(function, clause);
        function.instruction(&Instruction::Else);
        self.emit_clause_chain(function, disc_local, clauses, index + 1);
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::If);
        let _ = frame;
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
