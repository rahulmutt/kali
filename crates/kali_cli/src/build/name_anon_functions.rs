//! Pre-resolver AST pass (throw-fallout Stage 6, Task 2): assign a stable
//! `__kali_fn_{N}` name to every anonymous function-shaped node
//! (`FunctionExpression`/`ArrowFunctionExpression` with `id: None`), in a
//! deterministic pre-order walk.
//!
//! Why this must exist and run BEFORE the resolver: `kali_types`
//! (`binding_repr_function_key`, Task 3) and `kali_hir`
//! (`next_synthetic_function_name`'s fallback, `crates/kali_hir/src/lowering/
//! function.rs`) both need to refer to the SAME anonymous function by the
//! SAME name. Re-deriving one side's counter independently inside the other
//! would be a hand-mirrored oracle — this repo has shipped two of those
//! before, and BOTH failed open (see `MEMORY.md`'s `kali-throw-fallout-stage*`
//! entries). Naming once, in the AST, before either consumer runs, removes
//! the mirror entirely: there is only one counter, one walk, one set of
//! names, and both `kali_types` and `kali_hir` read it (`kali_hir` reads
//! `expr.id` directly; `kali_hir`'s own counter becomes a pure fallback for
//! any node this pass does not reach — Task 4 proves it reaches all of
//! them).
//!
//! ## Exhaustiveness
//!
//! Every match arm in this file is enumerated — no `_ =>` arm anywhere.
//! This mirrors `deny_import_positions_expression`
//! (`crates/kali_cli/src/build/module_link.rs:2601`) and
//! `deny_import_positions_statement` (`module_link.rs:2376`), which already
//! enumerate every `Statement`/`Expression`/JSX variant that exists in
//! `kali_ast` today; those two functions were used as the checklist while
//! writing the walk below. A function-shaped node this walk fails to reach
//! would still lower (via `next_synthetic_function_name`'s fallback), but
//! `kali_types` and `kali_hir` would then key on DIFFERENT names for it —
//! the exact fail-open this pass exists to prevent, so a missed arm here is
//! a silent miscompile, not a compile error, and the walk must be checked
//! by inspection + test, not just by the compiler's exhaustiveness check on
//! the match itself (which only proves every *variant* is handled, not that
//! every *field* of that variant is recursed into).
//!
//! ## Collision guard
//!
//! `__kali_fn_{N}` must never collide with a name the SOURCE program
//! declared itself — a top-level `function __kali_fn_3() {}`, a named
//! function expression `const f = function __kali_fn_3() {}`, a named class
//! `class __kali_fn_3 {}`/`class expression __kali_fn_3 {}`, or a class
//! method literally named that. `collect_taken_names` walks the WHOLE tree
//! first (the identical shape as the assignment walk below, so it has the
//! same exhaustiveness guarantee) and records every such name into a
//! `HashSet`; `fresh_name` then skips any candidate already in that set.
//! Collection always runs to completion BEFORE any name is assigned, so a
//! colliding declaration that appears textually AFTER an anonymous node in
//! the source is still honored — assignment order never depends on textual
//! order, only on the counter, and the counter only ever proposes names not
//! already in the (now-frozen-then-growing) taken set.
//!
//! Idempotent: a node that already has an `id` (a named function expression
//! from source, or a node a previous call to this pass already named) keeps
//! it unchanged — the walk only ever fills in `None`.

use std::collections::HashSet;

use kali_ast::*;

/// Assign `__kali_fn_{N}` to every anonymous function-shaped node, in a
/// deterministic pre-order walk. Idempotent: a node that already has an `id`
/// keeps it.
pub fn name_anonymous_functions(statements: &mut Vec<Statement>) {
    let mut taken = HashSet::new();
    for statement in statements.iter() {
        collect_names_statement(statement, &mut taken);
    }
    let mut counter: usize = 0;
    for statement in statements.iter_mut() {
        assign_names_statement(statement, &mut counter, &mut taken);
    }
}

fn fresh_name(counter: &mut usize, taken: &mut HashSet<String>) -> String {
    loop {
        let candidate = format!("__kali_fn_{}", *counter);
        *counter += 1;
        if taken.insert(candidate.clone()) {
            return candidate;
        }
    }
}

// ---------------------------------------------------------------------
// Pass 1: collect every name a SOURCE declaration already claims, so the
// counter below never proposes a colliding `__kali_fn_{N}`.
// ---------------------------------------------------------------------

fn collect_names_block(block: &BlockStatement, taken: &mut HashSet<String>) {
    for statement in &block.body {
        collect_names_statement(statement, taken);
    }
}

fn collect_names_var_decl(decl: &VariableDeclaration, taken: &mut HashSet<String>) {
    for declarator in &decl.declarations {
        if let Some(init) = &declarator.init {
            collect_names_expression(init, taken);
        }
    }
}

fn collect_names_class_body(body: &ClassBody, taken: &mut HashSet<String>) {
    for method in &body.methods {
        taken.insert(method.name.clone());
        if let Some(method_body) = &method.body {
            collect_names_block(method_body, taken);
        }
    }
}

fn collect_names_statement(statement: &Statement, taken: &mut HashSet<String>) {
    match statement {
        Statement::ExpressionStatement(stmt) => collect_names_expression(&stmt.expression, taken),
        Statement::BreakStatement(_) => {}
        Statement::ContinueStatement(_) => {}
        Statement::WithStatement(stmt) => {
            collect_names_expression(&stmt.object, taken);
            collect_names_statement(&stmt.body, taken);
        }
        Statement::ReturnStatement(stmt) => {
            if let Some(argument) = &stmt.argument {
                collect_names_expression(argument, taken);
            }
        }
        Statement::LabeledStatement(stmt) => collect_names_statement(&stmt.body, taken),
        Statement::IfStatement(stmt) => {
            collect_names_expression(&stmt.test, taken);
            collect_names_block(&stmt.consequent, taken);
            if let Some(alternate) = &stmt.alternate {
                collect_names_block(alternate, taken);
            }
        }
        Statement::SwitchStatement(stmt) => {
            collect_names_expression(&stmt.discriminant, taken);
            for case in &stmt.cases {
                if let Some(test) = &case.test {
                    collect_names_expression(test, taken);
                }
                for consequent in &case.consequent {
                    collect_names_statement(consequent, taken);
                }
            }
        }
        Statement::ThrowStatement(stmt) => collect_names_expression(&stmt.argument, taken),
        Statement::TryStatement(stmt) => {
            collect_names_block(&stmt.block, taken);
            if let Some(handler) = &stmt.handler {
                collect_names_block(&handler.body, taken);
            }
            if let Some(finalizer) = &stmt.finalizer {
                collect_names_block(finalizer, taken);
            }
        }
        Statement::DebuggerStatement(_) => {}
        Statement::BlockStatement(stmt) => collect_names_block(stmt, taken),
        Statement::ForStatement(stmt) => {
            match &stmt.init {
                Some(ForInit::VariableDeclaration(decl)) => collect_names_var_decl(decl, taken),
                Some(ForInit::Expression(expr)) => collect_names_expression(expr, taken),
                None => {}
            }
            if let Some(test) = &stmt.test {
                collect_names_expression(test, taken);
            }
            if let Some(update) = &stmt.update {
                collect_names_expression(update, taken);
            }
            collect_names_block(&stmt.body, taken);
        }
        Statement::ForInStatement(stmt) => {
            match &stmt.left {
                ForInLefthand::VariableDeclaration(decl) => collect_names_var_decl(decl, taken),
                ForInLefthand::Expression(expr) => collect_names_expression(expr, taken),
            }
            collect_names_expression(&stmt.right, taken);
            collect_names_statement(&stmt.body, taken);
        }
        Statement::ForOfStatement(stmt) => {
            match &stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => collect_names_var_decl(decl, taken),
                ForOfLefthand::Expression(expr) => collect_names_expression(expr, taken),
            }
            collect_names_expression(&stmt.right, taken);
            collect_names_statement(&stmt.body, taken);
        }
        Statement::WhileStatement(stmt) => {
            collect_names_expression(&stmt.test, taken);
            collect_names_block(&stmt.body, taken);
        }
        Statement::DoWhileStatement(stmt) => {
            collect_names_block(&stmt.body, taken);
            collect_names_expression(&stmt.test, taken);
        }
        Statement::FunctionDeclaration(function) => {
            taken.insert(function.name.clone());
            collect_names_block(&function.body, taken);
        }
        Statement::ClassDeclaration(class) => {
            taken.insert(class.name.clone());
            collect_names_class_body(&class.body, taken);
        }
        Statement::VariableDeclaration(decl) => collect_names_var_decl(decl, taken),
        Statement::ImportDeclaration(_) => {}
        Statement::ExportAll(_) => {}
        Statement::ExportNamed(_) => {}
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => collect_names_expression(expr, taken),
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                taken.insert(function.name.clone());
                collect_names_block(&function.body, taken);
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                taken.insert(class.name.clone());
                collect_names_class_body(&class.body, taken);
            }
        },
        Statement::EnumDeclaration(decl) => {
            for member in &decl.members {
                if let Some(value) = &member.value {
                    collect_names_expression(value, taken);
                }
            }
        }
        Statement::TypeAliasDeclaration(_) => {}
        Statement::InterfaceDeclaration(_) => {}
    }
}

fn collect_names_expression_or_spread(element: &ExpressionOrSpread, taken: &mut HashSet<String>) {
    match element {
        ExpressionOrSpread::Expression(expr) => collect_names_expression(expr, taken),
        ExpressionOrSpread::Spread(spread) => collect_names_expression(&spread.argument, taken),
        ExpressionOrSpread::Empty => {}
    }
}

fn collect_names_expression(expr: &Expression, taken: &mut HashSet<String>) {
    match expr {
        Expression::ParenthesizedExpression(inner) => {
            collect_names_expression(&inner.expression, taken)
        }
        Expression::AwaitExpression(await_expr) => {
            collect_names_expression(&await_expr.argument, taken)
        }
        Expression::ImportExpression(import_expr) => {
            collect_names_expression(&import_expr.source, taken)
        }
        Expression::Identifier(_) => {}
        Expression::Literal(_) => {}
        Expression::BinaryExpression(binary) => {
            collect_names_expression(&binary.left, taken);
            collect_names_expression(&binary.right, taken);
        }
        Expression::UnaryExpression(unary) => collect_names_expression(&unary.argument, taken),
        Expression::CallExpression(call) => {
            collect_names_expression(&call.callee, taken);
            for arg in &call.args {
                collect_names_expression(arg, taken);
            }
        }
        Expression::MemberExpression(member) => {
            collect_names_expression(&member.object, taken);
            if let Some(index) = &member.computed_index {
                collect_names_expression(index, taken);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter().flatten() {
                collect_names_expression_or_spread(element, taken);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                collect_names_expression(&property.value, taken);
            }
        }
        Expression::FunctionExpression(function) => {
            if let Some(id) = &function.id {
                taken.insert(id.clone());
            }
            if let Some(body) = &function.body {
                collect_names_block(body, taken);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if let Some(id) = &arrow.id {
                taken.insert(id.clone());
            }
            collect_names_expression(&arrow.body, taken);
        }
        Expression::ClassExpression(class) => {
            if let Some(id) = &class.id {
                taken.insert(id.clone());
            }
            collect_names_class_body(&class.body, taken);
        }
        Expression::NewExpression(new_expr) => {
            collect_names_expression(&new_expr.callee, taken);
            for arg in &new_expr.args {
                collect_names_expression(arg, taken);
            }
        }
        Expression::MetaProperty(_) => {}
        Expression::TemplateLiteral(template) => {
            for expression in &template.expressions {
                collect_names_expression(expression, taken);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            collect_names_expression(&tagged.tag, taken);
            for expression in &tagged.template.expressions {
                collect_names_expression(expression, taken);
            }
        }
        Expression::UpdateExpression(update) => collect_names_expression(&update.argument, taken),
        Expression::AssignmentExpression(assignment) => {
            collect_names_expression(&assignment.left, taken);
            collect_names_expression(&assignment.right, taken);
        }
        Expression::LogicalExpression(logical) => {
            collect_names_expression(&logical.left, taken);
            collect_names_expression(&logical.right, taken);
        }
        Expression::ConditionalExpression(conditional) => {
            collect_names_expression(&conditional.test, taken);
            collect_names_expression(&conditional.consequent, taken);
            collect_names_expression(&conditional.alternate, taken);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &sequence.expressions {
                collect_names_expression(expression, taken);
            }
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &yield_expr.argument {
                collect_names_expression(argument, taken);
            }
        }
        Expression::OptionalChainExpression(chain) => match chain.inner.as_ref() {
            OptionalChainInner::NonNull { object, .. } => collect_names_expression(object, taken),
        },
        Expression::ChainExpression(chain) => collect_names_expression(&chain.expression, taken),
        Expression::SpreadElement(spread) => collect_names_expression(&spread.argument, taken),
        Expression::RestElement(rest) => collect_names_expression(&rest.argument, taken),
        Expression::DecoratedExpression(decorated) => {
            collect_names_expression(&decorated.expression, taken)
        }
        Expression::JsxElement(element) => collect_names_jsx_element(element, taken),
        Expression::JsxFragment(fragment) => collect_names_jsx_fragment(fragment, taken),
        Expression::JsxEmptyExpression => {}
        Expression::TypeAssertion(assertion) => {
            collect_names_expression(&assertion.expression, taken)
        }
        Expression::SatisfiesExpression(satisfies) => {
            collect_names_expression(&satisfies.expression, taken)
        }
        Expression::ThisExpression => {}
        Expression::SuperExpression => {}
        Expression::PrivateIdentifier(_) => {}
        Expression::BigIntLiteral(_) => {}
    }
}

fn collect_names_jsx_element(element: &JsxElement, taken: &mut HashSet<String>) {
    for attribute in &element.opening_element.attributes {
        collect_names_jsx_attribute_item(attribute, taken);
    }
    for child in &element.children {
        collect_names_jsx_child(child, taken);
    }
}

fn collect_names_jsx_fragment(fragment: &JsxFragment, taken: &mut HashSet<String>) {
    for child in &fragment.children {
        collect_names_jsx_child(child, taken);
    }
}

fn collect_names_jsx_attribute_item(item: &JsxAttributeItem, taken: &mut HashSet<String>) {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            collect_names_jsx_attribute_value(&attribute.value, taken)
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            collect_names_expression(&spread.argument, taken)
        }
    }
}

fn collect_names_jsx_attribute_value(value: &JsxAttributeValue, taken: &mut HashSet<String>) {
    match value {
        JsxAttributeValue::String(_) => {}
        JsxAttributeValue::JsxElement(element) => collect_names_jsx_element(element, taken),
        JsxAttributeValue::JsxExpression(container) => {
            collect_names_jsx_expression_container(container, taken)
        }
    }
}

fn collect_names_jsx_expression_container(
    container: &JsxExpressionContainer,
    taken: &mut HashSet<String>,
) {
    if let Some(expression) = &container.expression {
        collect_names_expression(expression, taken);
    }
}

fn collect_names_jsx_child(child: &JsxChild, taken: &mut HashSet<String>) {
    match child {
        JsxChild::JsxText(_) => {}
        JsxChild::JsxExpression(container) => {
            collect_names_jsx_expression_container(container, taken)
        }
        JsxChild::JsxElement(element) => collect_names_jsx_element(element, taken),
        JsxChild::JsxFragment(fragment) => collect_names_jsx_fragment(fragment, taken),
    }
}

// ---------------------------------------------------------------------
// Pass 2: assign `__kali_fn_{N}` to every anonymous function-shaped node.
// Identical shape to Pass 1, mutable instead of read-only.
// ---------------------------------------------------------------------

fn assign_names_block(
    block: &mut BlockStatement,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    for statement in &mut block.body {
        assign_names_statement(statement, counter, taken);
    }
}

fn assign_names_var_decl(
    decl: &mut VariableDeclaration,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    for declarator in &mut decl.declarations {
        if let Some(init) = &mut declarator.init {
            assign_names_expression(init, counter, taken);
        }
    }
}

fn assign_names_class_body(body: &mut ClassBody, counter: &mut usize, taken: &mut HashSet<String>) {
    for method in &mut body.methods {
        if let Some(method_body) = &mut method.body {
            assign_names_block(method_body, counter, taken);
        }
    }
}

fn assign_names_statement(
    statement: &mut Statement,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    match statement {
        Statement::ExpressionStatement(stmt) => {
            assign_names_expression(&mut stmt.expression, counter, taken)
        }
        Statement::BreakStatement(_) => {}
        Statement::ContinueStatement(_) => {}
        Statement::WithStatement(stmt) => {
            assign_names_expression(&mut stmt.object, counter, taken);
            assign_names_statement(&mut stmt.body, counter, taken);
        }
        Statement::ReturnStatement(stmt) => {
            if let Some(argument) = &mut stmt.argument {
                assign_names_expression(argument, counter, taken);
            }
        }
        Statement::LabeledStatement(stmt) => assign_names_statement(&mut stmt.body, counter, taken),
        Statement::IfStatement(stmt) => {
            assign_names_expression(&mut stmt.test, counter, taken);
            assign_names_block(&mut stmt.consequent, counter, taken);
            if let Some(alternate) = &mut stmt.alternate {
                assign_names_block(alternate, counter, taken);
            }
        }
        Statement::SwitchStatement(stmt) => {
            assign_names_expression(&mut stmt.discriminant, counter, taken);
            for case in &mut stmt.cases {
                if let Some(test) = &mut case.test {
                    assign_names_expression(test, counter, taken);
                }
                for consequent in &mut case.consequent {
                    assign_names_statement(consequent, counter, taken);
                }
            }
        }
        Statement::ThrowStatement(stmt) => {
            assign_names_expression(&mut stmt.argument, counter, taken)
        }
        Statement::TryStatement(stmt) => {
            assign_names_block(&mut stmt.block, counter, taken);
            if let Some(handler) = &mut stmt.handler {
                assign_names_block(&mut handler.body, counter, taken);
            }
            if let Some(finalizer) = &mut stmt.finalizer {
                assign_names_block(finalizer, counter, taken);
            }
        }
        Statement::DebuggerStatement(_) => {}
        Statement::BlockStatement(stmt) => assign_names_block(stmt, counter, taken),
        Statement::ForStatement(stmt) => {
            match &mut stmt.init {
                Some(ForInit::VariableDeclaration(decl)) => {
                    assign_names_var_decl(decl, counter, taken)
                }
                Some(ForInit::Expression(expr)) => assign_names_expression(expr, counter, taken),
                None => {}
            }
            if let Some(test) = &mut stmt.test {
                assign_names_expression(test, counter, taken);
            }
            if let Some(update) = &mut stmt.update {
                assign_names_expression(update, counter, taken);
            }
            assign_names_block(&mut stmt.body, counter, taken);
        }
        Statement::ForInStatement(stmt) => {
            match &mut stmt.left {
                ForInLefthand::VariableDeclaration(decl) => {
                    assign_names_var_decl(decl, counter, taken)
                }
                ForInLefthand::Expression(expr) => assign_names_expression(expr, counter, taken),
            }
            assign_names_expression(&mut stmt.right, counter, taken);
            assign_names_statement(&mut stmt.body, counter, taken);
        }
        Statement::ForOfStatement(stmt) => {
            match &mut stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => {
                    assign_names_var_decl(decl, counter, taken)
                }
                ForOfLefthand::Expression(expr) => assign_names_expression(expr, counter, taken),
            }
            assign_names_expression(&mut stmt.right, counter, taken);
            assign_names_statement(&mut stmt.body, counter, taken);
        }
        Statement::WhileStatement(stmt) => {
            assign_names_expression(&mut stmt.test, counter, taken);
            assign_names_block(&mut stmt.body, counter, taken);
        }
        Statement::DoWhileStatement(stmt) => {
            assign_names_block(&mut stmt.body, counter, taken);
            assign_names_expression(&mut stmt.test, counter, taken);
        }
        Statement::FunctionDeclaration(function) => {
            assign_names_block(&mut function.body, counter, taken)
        }
        Statement::ClassDeclaration(class) => {
            assign_names_class_body(&mut class.body, counter, taken)
        }
        Statement::VariableDeclaration(decl) => assign_names_var_decl(decl, counter, taken),
        Statement::ImportDeclaration(_) => {}
        Statement::ExportAll(_) => {}
        Statement::ExportNamed(_) => {}
        Statement::ExportDefault(export) => match export {
            ExportDefaultDeclaration::Expression(expr) => {
                assign_names_expression(expr, counter, taken)
            }
            ExportDefaultDeclaration::FunctionDeclaration(function) => {
                // Task 3 (B): `export default function () {}` (anonymous)
                // parses with `name: ""` (`kali_ast::FunctionDeclaration.name`
                // is a plain `String`, never `Option`). `kali_hir` already
                // falls back to a synthetic name for this exact empty-name
                // case (`lowering/statement.rs`'s `FunctionDeclaration` arm,
                // reached via `lower_export_default` routing through
                // `Statement::FunctionDeclaration`) — mirror that fallback
                // HERE, in the shared pre-pass, so `kali_types`'s
                // `resolve_export_default` (`resolve/mod.rs`, which binds the
                // scope under the RAW `func.name` with no fallback of its
                // own) binds under the SAME name `kali_hir` mints, instead of
                // the two crates diverging on an empty-string key.
                if function.name.is_empty() {
                    function.name = fresh_name(counter, taken);
                }
                assign_names_block(&mut function.body, counter, taken)
            }
            ExportDefaultDeclaration::ClassDeclaration(class) => {
                assign_names_class_body(&mut class.body, counter, taken)
            }
        },
        Statement::EnumDeclaration(decl) => {
            for member in &mut decl.members {
                if let Some(value) = &mut member.value {
                    assign_names_expression(value, counter, taken);
                }
            }
        }
        Statement::TypeAliasDeclaration(_) => {}
        Statement::InterfaceDeclaration(_) => {}
    }
}

fn assign_names_expression_or_spread(
    element: &mut ExpressionOrSpread,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    match element {
        ExpressionOrSpread::Expression(expr) => assign_names_expression(expr, counter, taken),
        ExpressionOrSpread::Spread(spread) => {
            assign_names_expression(&mut spread.argument, counter, taken)
        }
        ExpressionOrSpread::Empty => {}
    }
}

fn assign_names_expression(
    expr: &mut Expression,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    match expr {
        Expression::ParenthesizedExpression(inner) => {
            assign_names_expression(&mut inner.expression, counter, taken)
        }
        Expression::AwaitExpression(await_expr) => {
            assign_names_expression(&mut await_expr.argument, counter, taken)
        }
        Expression::ImportExpression(import_expr) => {
            assign_names_expression(&mut import_expr.source, counter, taken)
        }
        Expression::Identifier(_) => {}
        Expression::Literal(_) => {}
        Expression::BinaryExpression(binary) => {
            assign_names_expression(&mut binary.left, counter, taken);
            assign_names_expression(&mut binary.right, counter, taken);
        }
        Expression::UnaryExpression(unary) => {
            assign_names_expression(&mut unary.argument, counter, taken)
        }
        Expression::CallExpression(call) => {
            assign_names_expression(&mut call.callee, counter, taken);
            for arg in &mut call.args {
                assign_names_expression(arg, counter, taken);
            }
        }
        Expression::MemberExpression(member) => {
            assign_names_expression(&mut member.object, counter, taken);
            if let Some(index) = &mut member.computed_index {
                assign_names_expression(index, counter, taken);
            }
        }
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter_mut().flatten() {
                assign_names_expression_or_spread(element, counter, taken);
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &mut object.properties {
                assign_names_expression(&mut property.value, counter, taken);
            }
        }
        Expression::FunctionExpression(function) => {
            if function.id.is_none() {
                function.id = Some(fresh_name(counter, taken));
            }
            if let Some(body) = &mut function.body {
                assign_names_block(body, counter, taken);
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.id.is_none() {
                arrow.id = Some(fresh_name(counter, taken));
            }
            assign_names_expression(&mut arrow.body, counter, taken);
        }
        Expression::ClassExpression(class) => {
            assign_names_class_body(&mut class.body, counter, taken)
        }
        Expression::NewExpression(new_expr) => {
            assign_names_expression(&mut new_expr.callee, counter, taken);
            for arg in &mut new_expr.args {
                assign_names_expression(arg, counter, taken);
            }
        }
        Expression::MetaProperty(_) => {}
        Expression::TemplateLiteral(template) => {
            for expression in &mut template.expressions {
                assign_names_expression(expression, counter, taken);
            }
        }
        Expression::TaggedTemplateExpression(tagged) => {
            assign_names_expression(&mut tagged.tag, counter, taken);
            for expression in &mut tagged.template.expressions {
                assign_names_expression(expression, counter, taken);
            }
        }
        Expression::UpdateExpression(update) => {
            assign_names_expression(&mut update.argument, counter, taken)
        }
        Expression::AssignmentExpression(assignment) => {
            assign_names_expression(&mut assignment.left, counter, taken);
            assign_names_expression(&mut assignment.right, counter, taken);
        }
        Expression::LogicalExpression(logical) => {
            assign_names_expression(&mut logical.left, counter, taken);
            assign_names_expression(&mut logical.right, counter, taken);
        }
        Expression::ConditionalExpression(conditional) => {
            assign_names_expression(&mut conditional.test, counter, taken);
            assign_names_expression(&mut conditional.consequent, counter, taken);
            assign_names_expression(&mut conditional.alternate, counter, taken);
        }
        Expression::SequenceExpression(sequence) => {
            for expression in &mut sequence.expressions {
                assign_names_expression(expression, counter, taken);
            }
        }
        Expression::YieldExpression(yield_expr) => {
            if let Some(argument) = &mut yield_expr.argument {
                assign_names_expression(argument, counter, taken);
            }
        }
        Expression::OptionalChainExpression(chain) => match chain.inner.as_mut() {
            OptionalChainInner::NonNull { object, .. } => {
                assign_names_expression(object, counter, taken)
            }
        },
        Expression::ChainExpression(chain) => {
            assign_names_expression(&mut chain.expression, counter, taken)
        }
        Expression::SpreadElement(spread) => {
            assign_names_expression(&mut spread.argument, counter, taken)
        }
        Expression::RestElement(rest) => {
            assign_names_expression(&mut rest.argument, counter, taken)
        }
        Expression::DecoratedExpression(decorated) => {
            assign_names_expression(&mut decorated.expression, counter, taken)
        }
        Expression::JsxElement(element) => assign_names_jsx_element(element, counter, taken),
        Expression::JsxFragment(fragment) => assign_names_jsx_fragment(fragment, counter, taken),
        Expression::JsxEmptyExpression => {}
        Expression::TypeAssertion(assertion) => {
            assign_names_expression(&mut assertion.expression, counter, taken)
        }
        Expression::SatisfiesExpression(satisfies) => {
            assign_names_expression(&mut satisfies.expression, counter, taken)
        }
        Expression::ThisExpression => {}
        Expression::SuperExpression => {}
        Expression::PrivateIdentifier(_) => {}
        Expression::BigIntLiteral(_) => {}
    }
}

fn assign_names_jsx_element(
    element: &mut JsxElement,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    for attribute in &mut element.opening_element.attributes {
        assign_names_jsx_attribute_item(attribute, counter, taken);
    }
    for child in &mut element.children {
        assign_names_jsx_child(child, counter, taken);
    }
}

fn assign_names_jsx_fragment(
    fragment: &mut JsxFragment,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    for child in &mut fragment.children {
        assign_names_jsx_child(child, counter, taken);
    }
}

fn assign_names_jsx_attribute_item(
    item: &mut JsxAttributeItem,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            assign_names_jsx_attribute_value(&mut attribute.value, counter, taken)
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            assign_names_expression(&mut spread.argument, counter, taken)
        }
    }
}

fn assign_names_jsx_attribute_value(
    value: &mut JsxAttributeValue,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    match value {
        JsxAttributeValue::String(_) => {}
        JsxAttributeValue::JsxElement(element) => assign_names_jsx_element(element, counter, taken),
        JsxAttributeValue::JsxExpression(container) => {
            assign_names_jsx_expression_container(container, counter, taken)
        }
    }
}

fn assign_names_jsx_expression_container(
    container: &mut JsxExpressionContainer,
    counter: &mut usize,
    taken: &mut HashSet<String>,
) {
    if let Some(expression) = &mut container.expression {
        assign_names_expression(expression, counter, taken);
    }
}

fn assign_names_jsx_child(child: &mut JsxChild, counter: &mut usize, taken: &mut HashSet<String>) {
    match child {
        JsxChild::JsxText(_) => {}
        JsxChild::JsxExpression(container) => {
            assign_names_jsx_expression_container(container, counter, taken)
        }
        JsxChild::JsxElement(element) => assign_names_jsx_element(element, counter, taken),
        JsxChild::JsxFragment(fragment) => assign_names_jsx_fragment(fragment, counter, taken),
    }
}

#[cfg(test)]
#[path = "name_anon_functions_tests.rs"]
mod name_anon_functions_tests;
