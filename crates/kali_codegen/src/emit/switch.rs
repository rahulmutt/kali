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
    /// The clause's last statement is an unlabeled `continue`. Admitted for
    /// the same reason `Break` is — it is a control TRANSFER, not fallthrough
    /// — and it needs no separate emission rule: the statement itself lowers
    /// through `emit_break_or_continue`, which resolves it against the switch
    /// frame's INHERITED `continue_index` and fails closed when there is no
    /// enclosing loop to inherit from.
    Continue,
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
    /// Which PROVEN value domain the discriminant is in: `true` = string,
    /// `false` = i64 scalar. There is no third state — `switch_plan` returns
    /// `Err` unless one of the two proofs holds — so the emit cannot be
    /// reached with an unproven domain and cannot "default" to a comparison.
    pub(crate) disc_is_string: bool,
    pub(crate) clauses: Vec<SwitchClause>,
}

impl<'a> FunctionEmitter<'a> {
    /// Build a plan, or explain why this switch is not admitted.
    ///
    /// Task 7 admitted exactly one shape: a PROVEN i64 discriminant, numeric-
    /// literal (optionally unary `+`/`-`) case tests, at most one `default`,
    /// and every clause ending in `return`. Task 8 ADDS a second discriminant
    /// domain — a PROVEN string — and makes rule 2 domain-matched, so a case
    /// test in the other domain is denied rather than silently never matching.
    /// It removes no rejection: every shape Task 7 denied is still denied,
    /// because the widening is a second proof placed beside the first, not a
    /// relaxation of it. Tasks 9-10 widen further the same way.
    pub(crate) fn switch_plan(&self, node: &LirNode) -> Result<SwitchPlan, String> {
        let mut children = node.children.iter().copied();
        let discriminant = children
            .next()
            .ok_or_else(|| "a switch with no discriminant".to_string())?;

        // Rule 1: the discriminant must be PROVEN to be an i64 scalar or a
        // string. Float, boolean, object, array and unknown stay denied — not
        // by being listed, but by failing to construct either proof.
        let disc_is_string = if self.is_provable_i64_scalar(discriminant) {
            false
        } else if self.is_provable_string(discriminant) {
            true
        } else {
            return Err("the discriminant is not a proven integer or string".to_string());
        };

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
                // Rule 2: the test must be a literal in the DISCRIMINANT's
                // domain — a numeric literal (including unary `+`/`-`) for an
                // i64 discriminant, a string literal for a string one. A
                // string case against an integer discriminant, or vice versa,
                // is DENIED rather than silently never matching: node would
                // fall to `default` for it, and while `__streq`'s tag guard
                // happens to produce that same answer, "the two engines agree
                // by accident" is not a lowering proof.
                let test_ok = if disc_is_string {
                    self.is_string_literal_case_test(test)
                } else {
                    self.is_numeric_literal_case_test(test)
                };
                if !test_ok {
                    return Err(
                        "a `case` test that is not a literal in the discriminant's domain"
                            .to_string(),
                    );
                }
                (Some(test), &case.children[1..])
            };

            // Rule 4: a clause must end in a proven control TRANSFER —
            // `return`, an unlabeled `break`, or an unlabeled `continue`. Task
            // 9 widened this from `return`-only by ADDING the two transfer
            // proofs beside it; it removed nothing, so a clause ending in a
            // bare assignment (true fallthrough) is denied exactly as before,
            // and so is a LABELED `break`/`continue` (those bind to an
            // enclosing labeled statement, not to this switch, and
            // `emit_break_or_continue` rejects labels globally anyway).
            // Empty grouping arrives in Task 10.
            let terminator = match stmts.last() {
                Some(last) if self.is_return_statement(*last) => ClauseTerminator::Return,
                Some(last) if self.is_unlabeled_break_statement(*last) => ClauseTerminator::Break,
                Some(last) if self.is_unlabeled_continue_statement(*last) => {
                    ClauseTerminator::Continue
                }
                _ => {
                    return Err(
                        "a clause that does not end in `return`, `break` or `continue` \
                         (true fallthrough is not in the supported lowering set)"
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
            disc_is_string,
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
    /// variable, returned, passed as a callback, `new`-ed — but NOT merely
    /// `export`ed; see `ReprTable::escaping_function_names`) — an escaping
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

    /// PROOF that `id`'s emitted value is a genuine tagged STRING handle.
    ///
    /// The string axis is not symmetric with the i64 axis, and the asymmetry
    /// is what makes this proof short. `Repr::I64` is `Repr`'s `#[default]`
    /// (`kali_common::repr.rs`), so `== Repr::I64` cannot distinguish "proven
    /// a number" from "nothing was ever recorded" — the defect
    /// `emit/call.rs:4220-4229` and `emit/operators.rs`'s
    /// `bitwise_compound_rhs_is_provably_i64` both already rule out, and the
    /// reason `is_provable_i64_scalar` above had to be assembled out of five
    /// separate oracles plus a call-site inflow proof. `Repr::String` is NOT
    /// the default: nothing is ever classified `String` by omission, so
    /// `== Repr::String` IS positive evidence. EVERY arm below rests on that.
    ///
    /// It rests on more than "some string reached this node", too.
    /// `kali_types::repr_infer::emit_table` only writes `Repr::String` for a
    /// scalar when the node is string-reachable AND NOT float-reachable AND
    /// NOT in `plain_write_targets` — the last being a node fed by any
    /// UNBACKED source (a plain integer/boolean, or a downgraded mixed
    /// return). A node failing that third conjunct is an `add_shape_conflict`,
    /// i.e. E5506 for the whole program, not a silent `String`. That is
    /// exactly the "mixed inflow / laundering write" family the i64 axis had
    /// to close by hand, already closed at the source and program-wide:
    /// `s("a"); s(1)`, `x = true`, `x = y`, `x = 1 > 0`, `x = g()` for an
    /// int-returning `g` are all rejected before this proof is ever consulted
    /// (measured: `error[E5506]: param 0 of `s` is used as both a string and a
    /// number`).
    ///
    /// Composition, mirroring `is_provable_i64_scalar`'s "reuse the oracles
    /// the emitter already dispatches with" rule:
    /// - the BARE IDENTIFIER arm is hand-written here rather than delegated,
    ///   because that is the one node shape where the emitter performs no
    ///   coercion of its own — it loads whatever is in the local — so the
    ///   oracle carries the whole burden. See
    ///   `identifier_is_provable_string`.
    /// - EVERY OTHER shape delegates to `is_string_valued`
    ///   (`emit/operators.rs`), the SAME oracle `+`, `===`/`!==`,
    ///   `console.log` and `String()` already dispatch with. Its non-identifier
    ///   arms are documented one by one as keying on "the SAME recognizer the
    ///   emit arm dispatches with, so oracle and emission agree by
    ///   construction" — a string literal, a `+` concat (a `+` with a
    ///   string-valued operand is a string in JS unconditionally, and the
    ///   emitter emits `string_concat` for exactly this predicate), a
    ///   string-armed ternary, `typeof`, `a[i]` on a `Repr::String` element
    ///   axis, `.join`, `.substring`, `process.argv[n]`, a URL component read,
    ///   `TextDecoder().decode`, an admitted `String(..)` coercion, and a call
    ///   whose `return_repr` is `String`. Re-deriving any of those here would
    ///   be the hand-mirrored-oracle drift this repository has paid for
    ///   repeatedly; delegating means `switch` cannot disagree with `===`
    ///   about what a string is.
    ///
    /// NOTE the delegation is deliberately NOT the first thing tried: an
    /// identifier reaching `is_string_valued` would take ITS identifier arm,
    /// which is a bare `scalar(..) == Repr::String` with no
    /// `name_is_program_bound` gate, no aggregate-taint checks and no
    /// parameter narrowing. Matching on the identifier shape FIRST is what
    /// routes it to the hardened arm instead.
    pub(crate) fn is_provable_string(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        match node.kind {
            LirNodeKind::Value if node.children.is_empty() => node
                .text
                .as_deref()
                .is_some_and(|name| self.identifier_is_provable_string(name)),
            _ => self.is_string_valued(id),
        }
    }

    /// The bare-identifier half of `is_provable_string`. Structurally the
    /// twin of `identifier_is_provable_i64_scalar` above — same gate order,
    /// same scope resolution, same if/else (NEVER `||`) split between a
    /// parameter's evidence and a binding's — with the numeric conjuncts
    /// swapped for the string ones:
    ///
    /// 1. `name_is_program_bound(name)` (`emit/call.rs`) FIRST, exactly as the
    ///    i64 twin does: it denies `globalThis`, `undefined`, `NaN` and every
    ///    other free global. Task 7 shipped its identifier arm without this
    ///    and admitted `switch (globalThis)`. (On this axis it is belt to the
    ///    `Repr::String` braces — a free global has no recorded repr, so it is
    ///    `I64` by default, not `String` — but the check is free and the
    ///    ordering lesson is not worth re-learning.)
    /// 2. the SAME four aggregate taints the i64 twin and
    ///    `binding_is_proven_string_coercion_scalar` (`emit/call.rs`) both
    ///    check: array binding, growable array binding, non-scalar param,
    ///    object-initialized binding. An array/object handle is an i64-shaped
    ///    wasm value like every other, so the wasm validator cannot catch one
    ///    reaching `__streq`.
    /// 3. an if/else on `name_is_declared_parameter`, NEVER an `||`. Fix round
    ///    3 of Task 7 established this shape: an `||` lets a parameter
    ///    short-circuit past the proof that was written for parameters, which
    ///    is precisely how `s(1, true)` with `x = y` leaked on the i64 axis.
    ///    - A PARAMETER additionally requires both DOMAIN-INDEPENDENT
    ///      conjuncts of Task 7's parameter proof, reused from the same sets
    ///      rather than re-derived (`kali_types::repr_infer`'s
    ///      `readonly_params` and `escaping_function_names`, surfaced on
    ///      `ReprTable` for this):
    ///      * `param_is_readonly` — the parameter is POSITIVELY proven never
    ///        written anywhere in the body. `plain_write_targets` already
    ///        catches every write whose source is a plain/unbacked value, but
    ///        it can only see writes that build a repr EDGE; `readonly_params`
    ///        is a positive enumeration of the forms in which a name is
    ///        provably only READ, where every unenumerated form denies, so it
    ///        also covers a write form that builds no edge at all.
    ///      * `!function_escapes(func)` — an escaping function (assigned,
    ///        returned, passed as a callback, `new`-ed) can be invoked
    ///        through a site the call-edge builder cannot enumerate, so no
    ///        fact derived from its enumerated call sites is a fact about all
    ///        of its invocations. `new s("x")` builds no call edge at all and
    ///        is visible ONLY through this set (`mark_new_callee_escapes`).
    ///        `export` is NOT an escape trigger — see
    ///        `ReprTable::escaping_function_names`; an exported switch
    ///        function is admitted, and that is safe only for as long as a
    ///        cross-module imported call returns `0` wholesale.
    ///    - A NON-PARAMETER binding uses `Repr::String` alone, which is the
    ///      SAME evidence `is_string_valued`'s identifier arm — and therefore
    ///      every `+`, `===`, `.length` and `console.log` in this compiler —
    ///      already trusts for a bare identifier. A binding that is ALSO
    ///      written with a plain value is an E5506 shape conflict, per this
    ///      function's caller's doc, so `switch` here is not extending trust
    ///      beyond what the rest of the emitter already rests on; it inherits
    ///      exactly that contract instead of inventing a parallel one.
    /// 4. `scalar(func, name) == Repr::String` last: the positive evidence,
    ///    sound as evidence ONLY because `String` is not `Repr`'s default.
    fn identifier_is_provable_string(&self, name: &str) -> bool {
        if !self.name_is_program_bound(name) {
            return false;
        }
        // Mirrors `is_string_valued`'s / the i64 twin's local-vs-module
        // resolution: a name not declared locally in a non-`_start` function
        // reads the module table, so the proof is looked up under the key
        // `repr_infer` recorded it under.
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
        if self.name_is_declared_parameter(name) {
            if !self.repr_table.param_is_readonly(func, name)
                || self.repr_table.function_escapes(func)
            {
                return false;
            }
        }
        self.repr_table.scalar(func, name) == kali_common::Repr::String
    }

    /// A `case` test is admitted iff it is a STRING LITERAL — a node the LIR
    /// still carries as a `Literal`, which `emit_node` materializes as an
    /// INTERNED string handle. The stringness question is delegated to
    /// `is_string_valued` (`emit/operators.rs`), the same oracle
    /// `is_provable_string` delegates to, rather than re-deriving the quoting
    /// rules here; the `Literal` kind check is what keeps it to a LITERAL,
    /// so an interpolated template (lowered to a concat `Value` node) or any
    /// other runtime string expression is not admitted as a case test.
    fn is_string_literal_case_test(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);
        self.node(id).kind == LirNodeKind::Literal && self.is_string_valued(id)
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

    /// `id` is an UNLABELED `break` statement.
    ///
    /// The text encoding is the one `kali_hir`'s statement lowering writes
    /// (`crates/kali_hir/src/lowering/statement.rs:25-32`): `"break"` for the
    /// unlabeled form and `"break:<label>"` for a labeled one — the same
    /// encoding `emit_break_or_continue` decodes
    /// (`emit/control_flow.rs:22-33`). The comparison is therefore EXACT, never
    /// a `starts_with("break")` prefix: `break outer;` inside a switch binds to
    /// the labeled statement, NOT to this switch, so admitting it here would be
    /// planning a clause whose terminator goes somewhere the plan does not
    /// model. Labels are rejected globally by `emit_break_or_continue` too;
    /// this is the allowlist half of the same fact, so the plan never depends
    /// on the emitter's rejection to stay honest.
    fn is_unlabeled_break_statement(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("break")
    }

    /// `id` is an UNLABELED `continue` statement. Exact-match twin of
    /// `is_unlabeled_break_statement`, for the same reasons.
    fn is_unlabeled_continue_statement(&self, id: LirNodeId) -> bool {
        let node = self.node(id);
        node.kind == LirNodeKind::Branch && node.text.as_deref() == Some("continue")
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

        // The switch's own break target. `continue_index` is INHERITED verbatim
        // from the enclosing loop frame, so an unlabeled `break` binds to this
        // switch while an unlabeled `continue` reaches past it to the loop — by
        // construction, not by a precedence rule in `emit_break_or_continue` a
        // later edit could get wrong. `None` (no enclosing loop) makes
        // `continue` fail closed there rather than degrade into `break`.
        //
        // The push is UNCONDITIONAL — deliberately NOT gated on any clause
        // having a `Break` terminator. A `break` nested INSIDE a
        // `return`-terminated clause (`case 1: if (q) { break; } return "a";`)
        // is admitted by Rule 4 and must still bind to the switch. Measured at
        // HEAD a8cc36ea9f, where no switch frame existed at all: that shape in
        // a `for` loop printed `iter=0` / `v=end` against node's `iter=0` /
        // `iter=1` / `v=def0`, exit 0 both sides — a silent miscompile this
        // unconditional push closes. Gating on `terminator` would reopen it,
        // which is why `SwitchClause::terminator` is NOT read here.
        //
        // FIX ROUND 1 — the inheritance is CONDITIONAL on the enclosing loop's
        // `continue` lowering being faithful. Admitting `continue` at all moved
        // two shapes from fail-closed to silently wrong, because a `continue`
        // that binds to a C-style `for` skips that loop's update (register
        // R-09). Measured at `a82d8499be`, exit 0 and no diagnostic on both:
        //
        //   for (var i = 0; i < 6; i = i + 1) { …
        //     switch (i) { case 2: i = i + 1; continue; default: s = s + i; break; } }
        //   node iter=0/1/2/4/5 s=10   kali iter=0/1/2/3/4/5 s=13
        //
        // and the SAME wrongness with the `continue` nested inside a
        // `return`-terminated clause — a shape Rule 4 has admitted since Task 7,
        // so this leak predates the `Continue` terminator and reverting that
        // terminator would not have closed it.
        //
        // Filtering here rather than walking clause bodies for `continue` keeps
        // the whole thing at the one choke point and preserves the
        // by-construction property: `emit_break_or_continue` still has no test
        // for what kind of frame it is looking at, and its existing `None` arm
        // now fails closed for BOTH shapes for free. Faithfulness is a property
        // of the LOOP (see `LoopFrame::continue_is_faithful`), so `switch`
        // states no opinion about loops here — it only declines to widen into
        // one whose `continue` it cannot lower honestly.
        let enclosing = self.loop_frames.last().copied();
        let (inherited_continue, inherited_faithful) = match enclosing {
            // A faithful enclosing frame: inherit its continue target verbatim.
            // (Transitive through nested switches — an inner switch inherits
            // whatever the outer switch inherited.)
            Some(frame) if frame.continue_is_faithful => (frame.continue_index, true),
            // An unfaithful enclosing loop: inherit NOTHING, and record WHY so
            // the diagnostic can say so instead of claiming there is no loop.
            Some(_) => (None, false),
            // No enclosing loop at all.
            None => (None, true),
        };
        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index: inherited_continue,
            continue_is_faithful: inherited_faithful,
        });

        self.emit_clause_chain(function, disc_local, plan.disc_is_string, &plan.clauses, 0);

        // A switch opens NO arena frame, so nothing is pushed to or popped from
        // `arena_frames` here and no `__arena_reset` is emitted. That is not a
        // gap: `emit_break_or_continue`'s comment (`emit/control_flow.rs:57-75`)
        // records that an earlier inline release double-released, splicing an
        // enclosing arena's still-live pages onto the free list. A `break` out
        // of a switch nested in a loop lands after this `End` and then falls
        // through to that loop's single unconditional release, exactly as a
        // normal switch exit does.
        self.loop_frames.pop();
        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

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
        disc_is_string: bool,
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
        if disc_is_string {
            // Reuse the EXISTING runtime string-equality helper `__streq` —
            // the same one `emit_binary`'s both-string `===`/`!==` arm calls
            // (`emit/operators.rs`), reached through the same
            // `streq_fn_index()` accessor. Nothing about the comparison is
            // hand-rolled here, so `switch` inherits R-08's fixed strict-
            // equality semantics by construction and cannot drift from them:
            // CONTENT equality (length + bytes, with handle identity only as
            // an internal fast path), which is why a runtime-built `"a" + "b"`
            // selects `case "ab"` instead of missing every clause.
            //
            // No env-get relocation is needed (unlike the `===` arm's
            // env-vs-env case): the discriminant is already materialized in
            // its own local before any clause test is emitted, and a case test
            // is an interned literal, so the two operands never share the
            // reserved scratch buffer.
            //
            // `__streq` returns an i64 that is exactly 0 or 1, and `If` needs
            // an i32, so the result is compared against 1. Deliberately NOT
            // `i64.eqz`: `__streq` is present in every module and
            // `pipeline_basics::boolean_branches_use_the_layout_fast_path`
            // pins module-wide printed text as containing no `i64.eqz`.
            function.instruction(&Instruction::Call(self.streq_fn_index()));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Eq);
        } else {
            function.instruction(&Instruction::I64Eq);
        }

        self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_clause_body(function, clause);
        function.instruction(&Instruction::Else);
        self.emit_clause_chain(function, disc_local, disc_is_string, clauses, index + 1);
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
