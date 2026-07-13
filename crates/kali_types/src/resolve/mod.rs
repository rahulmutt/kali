//! Statement and expression resolution.
use crate::*;

mod call;
mod expression;
mod function;
mod jsx;
mod member;

pub(crate) fn block_contains_yield_delegation(block: &BlockStatement) -> bool {
    block.body.iter().any(statement_contains_yield_delegation)
}

/// Extracts the bound identifier from a `for..in` left-hand side, for the
/// single-declarator `var`/`let`/`const` form (`for (var c in obj)`) AND —
/// Spec 4a Task 5 R1 — the bare-identifier form (`for (c in obj)`, key
/// pre-declared). Returns `None` for destructuring / any non-identifier LHS —
/// fail closed; those binding shapes are not yet reasoned about.
pub(crate) fn for_in_key_binding_name(left: &ForInLefthand) -> Option<String> {
    match left {
        ForInLefthand::VariableDeclaration(decl) if decl.declarations.len() == 1 => {
            Some(decl.declarations[0].id.clone())
        }
        // Spec 4a Task 5 R1: the bare-identifier form `for (c in obj)` (key
        // pre-declared elsewhere), the shape the capstone's `selectRandom`
        // uses. Registers the same key provenance as the declaration form.
        // Destructuring / non-identifier LHS remain fail-closed (`None`).
        ForInLefthand::Expression(kali_ast::Expression::Identifier(name)) => Some(name.clone()),
        _ => None,
    }
}

/// Collect every plain `<ident> = <ident>` assignment pair reachable inside a
/// `for..in` loop body, so the resolver can PRE-REGISTER for-in-key aliases
/// (`last = c`, `y = last`) BEFORE resolving the body. Mirrors codegen's
/// `for_in_key_alias_names` fixpoint pre-pass: a loop can USE an alias
/// (`table[last]`) textually BEFORE the `last = c` that defines it (the value
/// is live from the prior iteration), so a forward, in-order registration
/// alone would miss it and the Task 6 dynamic-key reject would spuriously fire
/// on a valid alias (fail-CLOSED, but an over-reject of makeCumulative).
/// Statement-level assignments cover fasta's `last = c` / `y = last`; an alias
/// buried in an unwalked expression position stays unregistered and merely
/// over-rejects (never miscompiles) — the safe direction.
pub(crate) fn collect_identifier_assignment_pairs(
    stmt: &Statement,
    out: &mut Vec<(String, String)>,
) {
    fn from_expression(expr: &Expression, out: &mut Vec<(String, String)>) {
        if let Expression::AssignmentExpression(assign) = expr {
            if matches!(assign.operator, AssignmentOperator::Assign) {
                if let (Expression::Identifier(lhs), Expression::Identifier(rhs)) =
                    (&assign.left, &assign.right)
                {
                    out.push((lhs.clone(), rhs.clone()));
                }
            }
        }
    }
    match stmt {
        Statement::ExpressionStatement(e) => from_expression(&e.expression, out),
        Statement::BlockStatement(b) => {
            for s in &b.body {
                collect_identifier_assignment_pairs(s, out);
            }
        }
        Statement::IfStatement(i) => {
            for s in &i.consequent.body {
                collect_identifier_assignment_pairs(s, out);
            }
            if let Some(alt) = &i.alternate {
                for s in &alt.body {
                    collect_identifier_assignment_pairs(s, out);
                }
            }
        }
        Statement::ForStatement(f) => {
            for s in &f.body.body {
                collect_identifier_assignment_pairs(s, out);
            }
        }
        Statement::ForInStatement(f) => collect_identifier_assignment_pairs(&f.body, out),
        Statement::ForOfStatement(f) => collect_identifier_assignment_pairs(&f.body, out),
        Statement::WhileStatement(w) => {
            for s in &w.body.body {
                collect_identifier_assignment_pairs(s, out);
            }
        }
        Statement::DoWhileStatement(d) => {
            for s in &d.body.body {
                collect_identifier_assignment_pairs(s, out);
            }
        }
        Statement::LabeledStatement(l) => collect_identifier_assignment_pairs(&l.body, out),
        Statement::WithStatement(w) => collect_identifier_assignment_pairs(&w.body, out),
        _ => {}
    }
}

pub(crate) fn statement_contains_yield_delegation(statement: &Statement) -> bool {
    match statement {
        Statement::ExpressionStatement(expression) => {
            expression_contains_yield_delegation(&expression.expression)
        }
        Statement::BreakStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::DebuggerStatement(_)
        | Statement::ImportDeclaration(_)
        | Statement::InterfaceDeclaration(_)
        | Statement::TypeAliasDeclaration(_)
        | Statement::ClassDeclaration(_)
        | Statement::FunctionDeclaration(_) => false,
        Statement::WithStatement(with_stmt) => {
            expression_contains_yield_delegation(&with_stmt.object)
                || statement_contains_yield_delegation(&with_stmt.body)
        }
        Statement::ReturnStatement(return_stmt) => return_stmt
            .argument
            .as_ref()
            .is_some_and(expression_contains_yield_delegation),
        Statement::LabeledStatement(labeled_stmt) => {
            statement_contains_yield_delegation(&labeled_stmt.body)
        }
        Statement::IfStatement(if_stmt) => {
            expression_contains_yield_delegation(&if_stmt.test)
                || block_contains_yield_delegation(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_deref()
                    .is_some_and(block_contains_yield_delegation)
        }
        Statement::SwitchStatement(switch_stmt) => {
            expression_contains_yield_delegation(&switch_stmt.discriminant)
                || switch_stmt.cases.iter().any(|case| {
                    case.test
                        .as_ref()
                        .is_some_and(expression_contains_yield_delegation)
                        || case
                            .consequent
                            .iter()
                            .any(statement_contains_yield_delegation)
                })
        }
        Statement::ThrowStatement(throw_stmt) => {
            expression_contains_yield_delegation(&throw_stmt.argument)
        }
        Statement::TryStatement(try_stmt) => {
            block_contains_yield_delegation(&try_stmt.block)
                || try_stmt
                    .handler
                    .as_ref()
                    .is_some_and(|handler| block_contains_yield_delegation(&handler.body))
                || try_stmt
                    .finalizer
                    .as_ref()
                    .is_some_and(block_contains_yield_delegation)
        }
        Statement::BlockStatement(block) => block_contains_yield_delegation(block),
        Statement::ForStatement(for_stmt) => {
            for_stmt.init.as_ref().is_some_and(|init| match init {
                ForInit::VariableDeclaration(declaration) => {
                    declaration.declarations.iter().any(|declarator| {
                        declarator
                            .init
                            .as_ref()
                            .is_some_and(expression_contains_yield_delegation)
                    })
                }
                ForInit::Expression(expression) => expression_contains_yield_delegation(expression),
            }) || for_stmt
                .test
                .as_ref()
                .is_some_and(expression_contains_yield_delegation)
                || for_stmt
                    .update
                    .as_ref()
                    .is_some_and(expression_contains_yield_delegation)
                || block_contains_yield_delegation(&for_stmt.body)
        }
        Statement::ForInStatement(for_in_stmt) => {
            let left_has_yield_delegation = match &for_in_stmt.left {
                ForInLefthand::VariableDeclaration(declaration) => {
                    declaration.declarations.iter().any(|declarator| {
                        declarator
                            .init
                            .as_ref()
                            .is_some_and(expression_contains_yield_delegation)
                    })
                }
                ForInLefthand::Expression(expression) => {
                    expression_contains_yield_delegation(expression)
                }
            };
            left_has_yield_delegation
                || expression_contains_yield_delegation(&for_in_stmt.right)
                || statement_contains_yield_delegation(&for_in_stmt.body)
        }
        Statement::ForOfStatement(for_of_stmt) => {
            let left_has_yield_delegation = match &for_of_stmt.left {
                ForOfLefthand::VariableDeclaration(declaration) => {
                    declaration.declarations.iter().any(|declarator| {
                        declarator
                            .init
                            .as_ref()
                            .is_some_and(expression_contains_yield_delegation)
                    })
                }
                ForOfLefthand::Expression(expression) => {
                    expression_contains_yield_delegation(expression)
                }
            };
            left_has_yield_delegation
                || expression_contains_yield_delegation(&for_of_stmt.right)
                || statement_contains_yield_delegation(&for_of_stmt.body)
        }
        Statement::WhileStatement(while_stmt) => {
            expression_contains_yield_delegation(&while_stmt.test)
                || block_contains_yield_delegation(&while_stmt.body)
        }
        Statement::DoWhileStatement(do_while_stmt) => {
            expression_contains_yield_delegation(&do_while_stmt.test)
                || block_contains_yield_delegation(&do_while_stmt.body)
        }
        Statement::VariableDeclaration(VariableDeclaration { declarations, .. }) => {
            declarations.iter().any(|declarator| {
                declarator
                    .init
                    .as_ref()
                    .is_some_and(expression_contains_yield_delegation)
            })
        }
        Statement::EnumDeclaration(EnumDeclaration { members, .. }) => {
            members.iter().any(|member| {
                member
                    .value
                    .as_ref()
                    .is_some_and(expression_contains_yield_delegation)
            })
        }
        Statement::ExportAll(_) | Statement::ExportNamed(_) | Statement::ExportDefault(_) => false,
    }
}

pub(crate) fn expression_contains_yield_delegation(expression: &Expression) -> bool {
    match expression {
        Expression::YieldExpression(yield_expr) => yield_expr.delegate,
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_contains_yield_delegation(&parenthesized.expression)
        }
        Expression::ConditionalExpression(conditional) => {
            expression_contains_yield_delegation(&conditional.test)
                || expression_contains_yield_delegation(&conditional.consequent)
                || expression_contains_yield_delegation(&conditional.alternate)
        }
        Expression::LogicalExpression(logical) => {
            expression_contains_yield_delegation(&logical.left)
                || expression_contains_yield_delegation(&logical.right)
        }
        Expression::BinaryExpression(binary) => {
            expression_contains_yield_delegation(&binary.left)
                || expression_contains_yield_delegation(&binary.right)
        }
        Expression::UnaryExpression(unary) => expression_contains_yield_delegation(&unary.argument),
        Expression::AssignmentExpression(assignment) => {
            expression_contains_yield_delegation(&assignment.right)
        }
        Expression::CallExpression(call) => {
            expression_contains_yield_delegation(&call.callee)
                || call.args.iter().any(expression_contains_yield_delegation)
        }
        Expression::MemberExpression(member) => {
            expression_contains_yield_delegation(&member.object)
        }
        Expression::AwaitExpression(await_expr) => {
            expression_contains_yield_delegation(&await_expr.argument)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .iter()
            .any(expression_contains_yield_delegation),
        Expression::OptionalChainExpression(optional) => match optional.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => {
                expression_contains_yield_delegation(object)
            }
        },
        Expression::ChainExpression(chain) => {
            expression_contains_yield_delegation(&chain.expression)
        }
        Expression::SpreadElement(spread) => expression_contains_yield_delegation(&spread.argument),
        Expression::RestElement(rest) => expression_contains_yield_delegation(&rest.argument),
        Expression::ImportExpression(import) => {
            expression_contains_yield_delegation(&import.source)
        }
        Expression::DecoratedExpression(decorated) => {
            expression_contains_yield_delegation(&decorated.expression)
        }
        Expression::JsxElement(element) => element.children.iter().any(|child| match child {
            JsxChild::JsxText(_) => false,
            JsxChild::JsxExpression(container) => container
                .expression
                .as_ref()
                .is_some_and(expression_contains_yield_delegation),
            JsxChild::JsxElement(child) => {
                expression_contains_yield_delegation(&Expression::JsxElement((**child).clone()))
            }
            JsxChild::JsxFragment(fragment) => {
                expression_contains_yield_delegation(&Expression::JsxFragment((**fragment).clone()))
            }
        }),
        Expression::JsxFragment(fragment) => fragment.children.iter().any(|child| match child {
            JsxChild::JsxText(_) => false,
            JsxChild::JsxExpression(container) => container
                .expression
                .as_ref()
                .is_some_and(expression_contains_yield_delegation),
            JsxChild::JsxElement(child) => {
                expression_contains_yield_delegation(&Expression::JsxElement((**child).clone()))
            }
            JsxChild::JsxFragment(child_fragment) => expression_contains_yield_delegation(
                &Expression::JsxFragment((**child_fragment).clone()),
            ),
        }),
        Expression::TypeAssertion(assertion) => {
            expression_contains_yield_delegation(&assertion.expression)
        }
        Expression::SatisfiesExpression(satisfies) => {
            expression_contains_yield_delegation(&satisfies.expression)
        }
        _ => false,
    }
}

impl TypeContext {
    pub fn resolve_statements(&mut self, statements: &[Statement]) -> ResolutionResult {
        self.resolve_statements_at_path(None::<&Path>, statements)
    }

    pub fn resolve_statements_at_path(
        &mut self,
        base_path: Option<impl AsRef<Path>>,
        statements: &[Statement],
    ) -> ResolutionResult {
        self.clear_diagnostics();
        self.scopes.clear();
        self.scope_stack.clear();
        self.type_env.clear();
        self.next_scope_id = 1;
        self.base_path = base_path.map(|path| path.as_ref().to_path_buf());

        self.push_scope(ScopeType::Module);
        self.repr_table = crate::repr_infer::infer_reprs(statements);
        self.resolve_statement_list(statements);
        self.emit_pending_generator_function_lowering_diagnostic();
        self.scope_stack.clear();

        ResolutionResult {
            diagnostics: self.diagnostics.clone(),
            scopes: self.scopes.clone(),
            global_scope: self.global_scope.clone(),
            repr_table: self.repr_table.clone(),
        }
    }

    pub fn resolve_statements_in_file(
        &mut self,
        file_path: impl AsRef<Path>,
        statements: &[Statement],
    ) -> ResolutionResult {
        let file_path = file_path.as_ref();
        self.resolve_statements_at_path(Some(file_path), statements)
    }

    pub(crate) fn resolve_statement_list(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    pub(crate) fn record_generator_function_lowering(&mut self, is_async: bool) {
        self.has_generator_function = true;
        if is_async {
            self.has_async_generator_function = true;
        }
    }

    pub(crate) fn emit_pending_generator_function_lowering_diagnostic(&mut self) {
        if !self.has_generator_function && !self.has_async_generator_function {
            return;
        }

        let message = if self.has_generator_yield_delegation
            && (self.has_generator_function ^ self.has_async_generator_function)
        {
            generator_function_yield_lowering_unavailable_message(
                self.has_async_generator_function,
                true,
            )
        } else {
            generator_function_lowering_unavailable_message_for_flavors(
                self.has_generator_function,
                self.has_async_generator_function,
            )
        };

        self.diagnostics
            .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
        self.has_generator_function = false;
        self.has_async_generator_function = false;
    }

    pub(crate) fn resolve_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ExpressionStatement(ExpressionStatement { expression }) => {
                // Statement position: the value is discarded, so an alias-copy
                // `last = c` is a safe ordinal-domain propagation (suppressed).
                self.resolve_statement_position_expression(expression)
            }
            Statement::BreakStatement(BreakStatement { .. }) => {}
            Statement::ContinueStatement(ContinueStatement { .. }) => {}
            Statement::WithStatement(WithStatement { object, body }) => {
                self.resolve_expression(object);
                self.resolve_statement(body);
            }
            Statement::ReturnStatement(ReturnStatement { argument }) => {
                if let Some(argument) = argument {
                    // Spec 4a Task 5: `return d` for a non-materialized for-in-key
                    // value is rejected structurally by the default-deny in
                    // `resolve_identifier` (the argument is resolved as a value);
                    // a materialized direct key (`return c`, repr `String`) is not.
                    self.resolve_expression(argument);
                }
            }
            Statement::LabeledStatement(LabeledStatement { body, .. }) => {
                self.resolve_statement(body)
            }
            Statement::IfStatement(IfStatement {
                test,
                consequent,
                alternate,
            }) => {
                // Spec 4a Task 5 allowlist: an `if` condition is a PROVEN-SAFE
                // truthiness position for a bare for-in-key value (`if (last)`,
                // Task 4's `>= 0` lowering) — resolve via the safe-position path
                // so the default-deny value-escape reject is suppressed for a
                // bare key. A complex test (`if (r < table[c])`, `if (id(c))`)
                // resolves normally: the index stays accepted, a nested escape
                // still rejects.
                self.resolve_forin_key_safe_position(test);
                self.resolve_block_statement(consequent);
                if let Some(alternate) = alternate {
                    self.resolve_block_statement(alternate);
                }
            }
            Statement::SwitchStatement(SwitchStatement {
                discriminant,
                cases,
            }) => {
                self.resolve_expression(discriminant);
                self.resolve_switch_cases(cases);
            }
            Statement::ThrowStatement(ThrowStatement { argument }) => {
                self.resolve_expression(argument)
            }
            Statement::TryStatement(TryStatement {
                block,
                handler,
                finalizer,
            }) => {
                self.resolve_block_statement(block);
                if let Some(CatchClause { param, body }) = handler {
                    self.push_scope(ScopeType::Catch);
                    self.bind_current_scope(param.clone());
                    self.resolve_block_body(body);
                    self.pop_scope();
                }
                if let Some(finalizer) = finalizer {
                    self.resolve_block_statement(finalizer);
                }
            }
            Statement::DebuggerStatement(_) => {}
            Statement::BlockStatement(block) => self.resolve_block_statement(block),
            Statement::ForStatement(ForStatement {
                init,
                test,
                update,
                body,
            }) => {
                self.push_scope(ScopeType::Block);
                if let Some(init) = init {
                    match init {
                        ForInit::VariableDeclaration(decl) => {
                            self.resolve_variable_declaration(decl)
                        }
                        ForInit::Expression(expr) => self.resolve_expression(expr),
                    }
                }
                if let Some(test) = test {
                    self.resolve_expression(test);
                    // H2: a for-in-key/alias `for (; last;)` condition lowers via
                    // default `!= 0` truthiness (the `>= 0` sentinel path is
                    // `if`-only) — reject fail-closed.
                    self.reject_forin_key_test_operand(test, "for-loop condition");
                }
                if let Some(update) = update {
                    self.resolve_expression(update);
                }
                self.resolve_block_body(body);
                self.pop_scope();
            }
            Statement::ForInStatement(ForInStatement { left, right, body }) => {
                self.push_scope(ScopeType::Block);
                match left {
                    ForInLefthand::VariableDeclaration(decl) => {
                        self.resolve_variable_declaration(decl)
                    }
                    // The bare-form key `for (c in obj)` LHS is a binding target,
                    // not a value read (and is resolved before the key is
                    // registered) — resolve via the safe-position path so a
                    // same-named already-registered key is never mis-rejected.
                    ForInLefthand::Expression(expr) => self.resolve_forin_key_safe_position(expr),
                }
                self.resolve_expression(right);
                // Spec 4a Task 2: tag the key binding with the enumerated
                // object's shape when known (dormant provenance registry —
                // unconsumed until Task 3+). Fail-closed: only a `var`/`let`/
                // `const` single-declarator LHS and a bare-identifier RHS
                // proven `Repr::Object(shape)` register anything.
                // Spec 4a Task 6 fail-closed gate: `for..in` only lowers over an
                // object with a compile-time-known fixed shape. A `None` shape
                // (an array, a non-object, or an unproven/polymorphic base) has
                // no field count to count against and no ordinal-to-slot map —
                // reject rather than miscompile. This is the types-side twin of
                // codegen's `emit_for_in` shape guard; both fail closed on the
                // same set. The accept lanes all enumerate a bare-identifier RHS
                // proven `Repr::Object(shape)` (register below), so this never
                // over-rejects them.
                match self.object_shape_of_expression(right) {
                    Some(shape) => {
                        if let Some(key_name) = for_in_key_binding_name(left) {
                            self.register_for_in_key(&key_name, shape);
                            // Pre-register transitive for-in-key aliases
                            // (`last = c`, `y = last`) so a computed access
                            // `table[last]` used BEFORE its defining assignment
                            // (value live from the prior iteration) still
                            // resolves as a key rather than tripping the Task 6
                            // dynamic-key reject. Fixpoint over the body's
                            // identifier-assignment pairs — the types-side twin
                            // of codegen's `for_in_key_alias_names`.
                            let mut pairs = Vec::new();
                            collect_identifier_assignment_pairs(body, &mut pairs);
                            let mut registered = std::collections::HashSet::new();
                            registered.insert(key_name);
                            loop {
                                let mut changed = false;
                                for (lhs, rhs) in &pairs {
                                    if registered.contains(rhs) && !registered.contains(lhs) {
                                        self.register_for_in_key(lhs, shape);
                                        registered.insert(lhs.clone());
                                        changed = true;
                                    }
                                }
                                if !changed {
                                    break;
                                }
                            }
                        }
                    }
                    None => {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            "for..in is only supported over an object with a compile-time-known fixed shape in the current direct-runtime path; iterating an array or a value of unknown shape is unavailable".to_string(),
                        ));
                    }
                }
                self.resolve_loop_body(body);
                self.pop_scope();
            }
            Statement::ForOfStatement(ForOfStatement {
                left,
                right,
                body,
                is_await,
            }) => {
                // Growable-array receiver (throw-fallout Stage 4 Task 4): a
                // promoted push-accumulated array iterates a REAL runtime
                // counted loop in codegen (`i=0; n=len; loop { v=data[i]; body;
                // i++ }`), NOT the static-unroll lane below —
                // `is_static_array_iteration_target` still sees the stale
                // declarator literal and would silently unroll ZERO iterations
                // over an array that now really accumulates. Admit it here (the
                // codegen runtime branch keys on the SAME predicate — bare
                // identifier + growable binding) instead of taking the static
                // path. Only the I64 element repr is admitted; a String-element
                // growable still fails closed (the loop-var string repr is not
                // wired this stage — Task brief authorizes fail-closed), as does
                // `for-await-of` and an unsupported loop target.
                {
                    let mut rhs = right;
                    while let Expression::ParenthesizedExpression(inner) = rhs {
                        rhs = &inner.expression;
                    }
                    if let Expression::Identifier(name) = rhs {
                        if self.is_growable_array_binding(name) {
                            let left_is_supported = match left {
                                ForOfLefthand::VariableDeclaration(_) => true,
                                ForOfLefthand::Expression(expression) => {
                                    self.is_simple_for_of_binding_expression(expression)
                                }
                            };
                            let elem_is_i64 = self
                                .binding_repr_function_key(name)
                                .map(|func| {
                                    self.repr_table.array_element(&func, name)
                                        == kali_common::Repr::I64
                                })
                                .unwrap_or(false);
                            // Self-push reject (review fix): a `<name>.push(...)`
                            // anywhere in the body pushes into the array being
                            // iterated — node GROWS the iteration, the counted
                            // loop's once-snapshotted length does not: a silent
                            // node-divergent miscompile if admitted. Fail closed
                            // (E5506). Name-based and conservative (a shadowing
                            // redeclaration still rejects); a push on a DIFFERENT
                            // binding (`out.push(v)`) stays admitted. Codegen
                            // carries a defensive by-construction mirror in
                            // `emit_growable_push_call`.
                            if crate::growable::statement_contains_push_on(body, name) {
                                let loop_kind = if *is_await { "for-await-of" } else { "for-of" };
                                self.diagnostics.push(Diagnostic::error(
                                    e5::FEATURE_UNAVAILABLE as u32,
                                    format!(
                                        "pushing to a growable array inside a {} loop iterating that same array is unavailable in the current phase (the iteration count is fixed at loop entry, diverging from JS growth semantics); use an index loop over `.length` or the later compatibility path",
                                        loop_kind
                                    ),
                                ));
                                return;
                            }
                            if left_is_supported && elem_is_i64 && !*is_await {
                                // ADMIT: resolve the loop var + RHS + body
                                // exactly like the static lane's admit path.
                                // The codegen runtime branch emits the counted
                                // loop.
                                self.push_scope(ScopeType::Block);
                                if let ForOfLefthand::VariableDeclaration(decl) = left {
                                    self.resolve_variable_declaration(decl)
                                }
                                self.resolve_static_string_fold_position(right);
                                self.resolve_loop_body(body);
                                self.pop_scope();
                                return;
                            }
                            // Fail closed: String element repr, unsupported loop
                            // target, or `for-await-of` — never a silent wrong
                            // answer (and never the stale-literal static unroll).
                            let loop_kind = if *is_await { "for-await-of" } else { "for-of" };
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                format!(
                                    "{} iteration over a growable (push-accumulated) array is unavailable in the current phase unless the loop target is a variable/identifier binding and elements are integers; use an index loop over `.length` or the later compatibility path",
                                    loop_kind
                                ),
                            ));
                            return;
                        }
                    }
                }
                let left_is_supported = match left {
                    ForOfLefthand::VariableDeclaration(_) => true,
                    ForOfLefthand::Expression(expression) => {
                        self.is_simple_for_of_binding_expression(expression)
                    }
                };
                if !left_is_supported || !self.is_static_array_iteration_target(right) {
                    let loop_kind = if *is_await { "for-await-of" } else { "for-of" };
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "{} array iteration lowering is unavailable unless the iterable is a literal array or supported string iterable with literal elements and the loop target is a variable declaration or simple identifier binding; use a supported loop form or the later compatibility path",
                            loop_kind
                        ),
                    ));
                    return;
                }

                self.push_scope(ScopeType::Block);
                if let ForOfLefthand::VariableDeclaration(decl) = left {
                    self.resolve_variable_declaration(decl)
                }
                self.resolve_static_string_fold_position(right);
                self.resolve_loop_body(body);
                self.pop_scope();
            }
            Statement::WhileStatement(WhileStatement { test, body }) => {
                self.resolve_expression(test);
                // H2: a for-in-key/alias `while (last)` condition lowers via
                // default `!= 0` truthiness (the `>= 0` sentinel path is
                // `if`-only) — reject fail-closed.
                self.reject_forin_key_test_operand(test, "while-loop condition");
                self.resolve_block_statement(body);
            }
            Statement::DoWhileStatement(DoWhileStatement { body, test }) => {
                self.resolve_block_statement(body);
                self.resolve_expression(test);
                // H2: a for-in-key/alias `do..while (last)` condition lowers via
                // default `!= 0` truthiness (the `>= 0` sentinel path is
                // `if`-only) — reject fail-closed.
                self.reject_forin_key_test_operand(test, "do-while-loop condition");
            }
            Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                params,
                body,
                generator,
                is_async,
                ..
            }) => {
                self.bind_current_scope(name.clone());
                let function_scope_id = self.push_scope(ScopeType::Function);
                self.current_function.push(name.clone());
                self.current_function_scopes.push(function_scope_id);
                let previous_generator = self.in_generator_function;
                self.in_generator_function = *generator;
                if *generator {
                    self.record_generator_function_lowering(*is_async);
                }
                self.bind_name_list(params);
                // Function-declaration parameters are mutable JS bindings (see
                // context::bind_function_params) — mark them so a compound/
                // update assignment on a parameter (`n -= lenOut`) is admitted.
                for param in params {
                    self.mark_binding_mutable(function_scope_id, param);
                }
                // Structural runtime-array registry (C1): an array-typed
                // PARAMETER is registered by codegen's emitter (emitter.rs:
                // `repr_table.is_array_binding(function_name, name)`). Mirror
                // that here so a `join`/store/`.length` on an array param stays
                // in lockstep with what codegen lowers.
                for param in params {
                    if self.repr_table.is_array_binding(name, param) {
                        if let Some(scope) = self.scopes.get_mut(&function_scope_id) {
                            scope.runtime_array_bindings.insert(param.clone(), true);
                        }
                    }
                }
                self.resolve_block_body(body);
                self.in_generator_function = previous_generator;
                self.current_function_scopes.pop();
                self.current_function.pop();
                self.pop_scope();
            }
            Statement::ClassDeclaration(ClassDeclaration { name, body }) => {
                self.bind_current_scope(name.clone());
                self.resolve_class_body(body);
            }
            Statement::VariableDeclaration(declaration) => {
                self.resolve_variable_declaration(declaration)
            }
            Statement::ImportDeclaration(declaration) => {
                self.resolve_import_declaration(declaration)
            }
            Statement::ExportAll(declaration) => self.resolve_export_all(declaration),
            Statement::ExportNamed(declaration) => self.resolve_export_named(declaration),
            Statement::ExportDefault(declaration) => self.resolve_export_default(declaration),
            Statement::EnumDeclaration(EnumDeclaration { name, members }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::Class);
                for EnumMember { name, value } in members {
                    self.bind_current_scope(name.clone());
                    if let Some(value) = value {
                        self.resolve_expression(value);
                    }
                }
                self.pop_scope();
            }
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name,
                type_params,
                type_annotation,
            }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::TypeAlias);
                self.bind_type_params(type_params);
                self.resolve_type_annotation_text(type_annotation);
                self.pop_scope();
            }
            Statement::InterfaceDeclaration(InterfaceDeclaration { name, properties }) => {
                self.bind_current_scope(name.clone());
                self.push_scope(ScopeType::Interface);
                for property in properties {
                    self.bind_current_scope(property.name.clone());
                    self.resolve_type_annotation_text(&property.type_annotation);
                }
                self.pop_scope();
            }
        }
    }

    pub(crate) fn resolve_block_statement(&mut self, block: &BlockStatement) {
        self.push_scope(ScopeType::Block);
        self.resolve_block_body(block);
        self.pop_scope();
    }

    pub(crate) fn resolve_block_body(&mut self, block: &BlockStatement) {
        for statement in &block.body {
            self.resolve_statement(statement);
        }
    }

    pub(crate) fn resolve_loop_body(&mut self, body: &Statement) {
        match body {
            Statement::BlockStatement(block) => self.resolve_block_body(block),
            other => self.resolve_statement(other),
        }
    }

    pub(crate) fn resolve_switch_cases(&mut self, cases: &[SwitchCase]) {
        self.push_scope(ScopeType::Block);
        for case in cases {
            if let Some(test) = &case.test {
                self.resolve_expression(test);
            }
            for statement in &case.consequent {
                self.resolve_statement(statement);
            }
        }
        self.pop_scope();
    }

    pub(crate) fn resolve_variable_declaration(&mut self, declaration: &VariableDeclaration) {
        let target_scope = self.variable_binding_scope(&declaration.kind);
        for declarator in &declaration.declarations {
            self.bind_in_scope(target_scope, declarator.id.clone());
        }
        for declarator in &declaration.declarations {
            if let Some(init) = &declarator.init {
                // Spec 4a Task 5 allowlist: a bare-key declarator init
                // (`let d = c`) is an ALIAS-COPY (ordinal domain) — resolve via
                // the safe-position path (suppresses the default-deny reject for
                // a bare key; `d` is tainted below). A non-bare init
                // (`let d = id(c)`) resolves normally → the nested escape rejects.
                self.resolve_forin_key_safe_position(init);
                if let Some(scope) = self.scopes.get_mut(&target_scope) {
                    scope
                        .mutable_bindings
                        .insert(declarator.id.clone(), declaration.kind != "const");
                } else if self.global_scope.contains(&declarator.id) {
                    self.global_scope
                        .mutable_bindings
                        .insert(declarator.id.clone(), declaration.kind != "const");
                }
                // String-typedness is tracked for every binding kind, including
                // hoisted `var`, so a `+` on a `var` string is rejected too. The
                // target scope for `var` is the function/global scope, so the flag
                // is visible wherever the binding is.
                if self.expression_is_string_typed(init) {
                    if let Some(scope) = self.scopes.get_mut(&target_scope) {
                        scope
                            .static_string_typed
                            .insert(declarator.id.clone(), true);
                    } else if self.global_scope.contains(&declarator.id) {
                        self.global_scope
                            .static_string_typed
                            .insert(declarator.id.clone(), true);
                    }
                }
                // Spec 4a Task 5 fail-closed: a declarator-init alias
                // `let d = c` (RHS carries for-in-key value provenance) marks
                // `d` as a value copy for the value-escape reject gate ONLY
                // (never the index/truthiness/materialization lanes). Chains
                // (`let e = d`) propagate because `is_for_in_key_value` is
                // transitive. Assignment aliases (`d = c`) register in
                // `for_in_key_bindings` separately.
                if let Expression::Identifier(rhs_name) = init {
                    if self.is_for_in_key_value(rhs_name) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope
                                .for_in_key_value_bindings
                                .insert(declarator.id.clone(), true);
                        } else if self.global_scope.contains(&declarator.id) {
                            self.global_scope
                                .for_in_key_value_bindings
                                .insert(declarator.id.clone(), true);
                        }
                    }
                }
                // Track array-literal bindings for EVERY kind (incl. `var`):
                // the runtime `join` lane rejects them (codegen never linearizes
                // a literal array into a runtime binding — it would emit `0`).
                if matches!(init, Expression::ArrayExpression(_)) {
                    if let Some(scope) = self.scopes.get_mut(&target_scope) {
                        scope
                            .array_literal_bindings
                            .insert(declarator.id.clone(), true);
                    } else if self.global_scope.contains(&declarator.id) {
                        self.global_scope
                            .array_literal_bindings
                            .insert(declarator.id.clone(), true);
                    }
                    // Growable-array promotion (throw-fallout Stage 4): the
                    // repr inference already adjudicated the safe-position
                    // allowlist + i64 element proof (see `crate::growable` +
                    // `repr_infer`'s emit gate); mirror the promotion into
                    // the scope registry so the resolve-phase gates
                    // (`for..of`, `.join`) can key on it.
                    if self
                        .binding_repr_function_key(&declarator.id)
                        .is_some_and(|func| {
                            self.repr_table
                                .is_growable_array_binding(&func, &declarator.id)
                        })
                    {
                        self.register_growable_array_binding(&declarator.id);
                    }
                }
                // Structural runtime-array registry (C1): register a declarator
                // whose init codegen backs with a linear-memory allocation
                // (`new Array(n)` / `Array(n)` / `.fill(...)`), mirroring
                // codegen's `array_bindings` declarator registration. The
                // string store / `.length` / `join` lanes require this
                // structural proof, not the repr-table proof alone.
                if self.declarator_registers_runtime_array(init) {
                    if let Some(scope) = self.scopes.get_mut(&target_scope) {
                        scope
                            .runtime_array_bindings
                            .insert(declarator.id.clone(), true);
                    } else if self.global_scope.contains(&declarator.id) {
                        self.global_scope
                            .runtime_array_bindings
                            .insert(declarator.id.clone(), true);
                    }
                }
                if declaration.kind != "var" {
                    if let Some(value) = self.resolve_static_string_expression(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_values.insert(declarator.id.clone(), value);
                        }
                    }
                    if let Some(value) = self.resolve_static_numeric_literal_value(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope
                                .static_numeric_values
                                .insert(declarator.id.clone(), value.to_string());
                        }
                    }
                    if let Some(value) = self.resolve_static_object_identity_literal_value(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope
                                .static_identity_values
                                .insert(declarator.id.clone(), value);
                        }
                    }
                    if self.is_static_array_iteration_target(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_arrays.insert(declarator.id.clone(), true);
                        }
                    }
                    if self.resolve_static_object_model_target(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_objects.insert(declarator.id.clone(), true);
                        }
                    }
                    if self.resolve_static_object_model_target(init)
                        || self.is_static_array_iteration_target(init)
                    {
                        let root = self
                            .resolve_static_reference_root(init)
                            .unwrap_or_else(|| declarator.id.clone());
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope
                                .static_reference_values
                                .insert(declarator.id.clone(), root);
                        }
                    }
                    if let Some(root) = self.resolve_static_reference_root(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope
                                .static_reference_values
                                .insert(declarator.id.clone(), root);
                        }
                    }
                    if self.resolve_static_object_keys_target(init) {
                        if let Some(scope) = self.scopes.get_mut(&target_scope) {
                            scope.static_object_keys.insert(declarator.id.clone(), true);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn resolve_import_declaration(&mut self, declaration: &ImportDeclaration) {
        match self.resolve_import_source(&declaration.source) {
            Ok(true) => {}
            Ok(false) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        e3::IMPORT_NOT_FOUND as u32,
                        format!(
                            "import source '{}' could not be resolved",
                            declaration.source
                        ),
                    )
                    .with_suggestion("check the relative path or package specifier"),
                );
                return;
            }
            Err(diagnostic) => {
                self.diagnostics.push(diagnostic);
                return;
            }
        }

        for specifier in &declaration.specifiers {
            match specifier {
                ImportSpecifier::Default(local) => {
                    self.bind_current_scope(local.clone());
                }
                ImportSpecifier::Named(specifiers) | ImportSpecifier::Type(specifiers) => {
                    for specifier in specifiers {
                        self.bind_current_scope(specifier.local.clone());
                    }
                }
                ImportSpecifier::Namespace(local) => {
                    self.bind_current_scope(local.clone());
                }
                ImportSpecifier::SideEffect => {}
            }
        }
    }

    pub(crate) fn resolve_export_all(&mut self, declaration: &ExportAllDeclaration) {
        match self.resolve_import_source(&declaration.source) {
            Ok(true) => {}
            Ok(false) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        e3::IMPORT_NOT_FOUND as u32,
                        format!(
                            "re-export source '{}' could not be resolved",
                            declaration.source
                        ),
                    )
                    .with_suggestion("check the relative path or package specifier"),
                );
            }
            Err(diagnostic) => {
                self.diagnostics.push(diagnostic);
            }
        }
    }

    pub(crate) fn resolve_export_named(&mut self, declaration: &kali_ast::ExportNamedDeclaration) {
        if let Some(source) = &declaration.source {
            match self.resolve_import_source(source) {
                Ok(true) => {}
                Ok(false) => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            e3::IMPORT_NOT_FOUND as u32,
                            format!("re-export source '{}' could not be resolved", source),
                        )
                        .with_suggestion("check the relative path or package specifier"),
                    );
                }
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                }
            }
            return;
        }

        for specifier in &declaration.specifiers {
            self.resolve_identifier(&specifier.local);
        }
    }

    pub(crate) fn resolve_export_default(
        &mut self,
        declaration: &kali_ast::ExportDefaultDeclaration,
    ) {
        match declaration {
            kali_ast::ExportDefaultDeclaration::Expression(expr) => self.resolve_expression(expr),
            kali_ast::ExportDefaultDeclaration::FunctionDeclaration(func) => {
                self.push_scope(ScopeType::Function);
                self.bind_current_scope(func.name.clone());
                self.bind_function_params(
                    &func
                        .params
                        .iter()
                        .cloned()
                        .map(|name| FunctionParam { name })
                        .collect::<Vec<_>>(),
                );
                self.resolve_block_body(&func.body);
                self.pop_scope();
            }
            kali_ast::ExportDefaultDeclaration::ClassDeclaration(class) => {
                self.push_scope(ScopeType::Class);
                self.bind_current_scope(class.name.clone());
                self.resolve_class_body(&class.body);
                self.pop_scope();
            }
        }
    }
}
