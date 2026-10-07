//! Growable runtime arrays: the fact walk (`facts`), the component solve (`flow`)
//! and the position refusals (`positions`), plus the object-field shape helper
//! [`array_literal_of_scalar_seeds`] and the resolver's for-of self-push scan.

use kali_ast::{
    Expression, ExpressionOrSpread, ForInLefthand, ForInit, ForOfLefthand, LiteralValue, Statement,
};

pub(crate) mod elem_proof;
pub(crate) mod facts;
pub(crate) mod flow;
pub(crate) mod positions;

/// True when `stmt` (a `for..of` BODY) contains any `<name>.push(...)` call —
/// the resolve-phase for..of gate's same-binding self-push reject (Stage 4
/// Task 4 review fix): a push into the array being iterated would grow the
/// iteration under node but not under the counted loop's once-snapshotted
/// length — a silent node-divergent miscompile. Purely syntactic and
/// name-based: a shadowing redeclaration of `name` in an inner scope still
/// rejects (conservative, per review guidance), and constructs the walk cannot
/// see through (`with`, class bodies, JSX, import/export/enum) return `true`
/// (reject) rather than risk missing an occurrence — belt-and-braces, since
/// the promotion scanner already poisons any function containing them. Nested
/// function/arrow bodies ARE walked (conservative; a closure capturing a
/// growable never promotes anyway). A push on a DIFFERENT binding never
/// matches — the target fixture's `out.push(v)` inside `for (const v of o)`
/// stays admitted.
pub(crate) fn statement_contains_push_on(stmt: &Statement, name: &str) -> bool {
    push_scan_stmt(stmt, name)
}

fn push_scan_block(block: &kali_ast::BlockStatement, name: &str) -> bool {
    block.body.iter().any(|stmt| push_scan_stmt(stmt, name))
}

fn push_scan_stmt(stmt: &Statement, name: &str) -> bool {
    match stmt {
        Statement::ExpressionStatement(s) => push_scan_expr(&s.expression, name),
        Statement::BreakStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::DebuggerStatement(_)
        | Statement::TypeAliasDeclaration(_)
        | Statement::InterfaceDeclaration(_) => false,
        // Cannot see through these — conservative TRUE (reject).
        Statement::WithStatement(_)
        | Statement::ClassDeclaration(_)
        | Statement::ImportDeclaration(_)
        | Statement::ExportAll(_)
        | Statement::ExportNamed(_)
        | Statement::ExportDefault(_)
        | Statement::EnumDeclaration(_) => true,
        Statement::ReturnStatement(s) => s
            .argument
            .as_ref()
            .is_some_and(|arg| push_scan_expr(arg, name)),
        Statement::LabeledStatement(s) => push_scan_stmt(&s.body, name),
        Statement::IfStatement(s) => {
            push_scan_expr(&s.test, name)
                || push_scan_block(&s.consequent, name)
                || s.alternate
                    .as_ref()
                    .is_some_and(|alt| push_scan_block(alt, name))
        }
        Statement::SwitchStatement(s) => {
            push_scan_expr(&s.discriminant, name)
                || s.cases.iter().any(|case| {
                    case.test
                        .as_ref()
                        .is_some_and(|test| push_scan_expr(test, name))
                        || case
                            .consequent
                            .iter()
                            .any(|stmt| push_scan_stmt(stmt, name))
                })
        }
        Statement::ThrowStatement(s) => push_scan_expr(&s.argument, name),
        Statement::TryStatement(s) => {
            push_scan_block(&s.block, name)
                || s.handler
                    .as_ref()
                    .is_some_and(|handler| push_scan_block(&handler.body, name))
                || s.finalizer
                    .as_ref()
                    .is_some_and(|finalizer| push_scan_block(finalizer, name))
        }
        Statement::BlockStatement(s) => push_scan_block(s, name),
        Statement::ForStatement(s) => {
            (match &s.init {
                Some(ForInit::VariableDeclaration(decl)) => push_scan_var_decl(decl, name),
                Some(ForInit::Expression(expr)) => push_scan_expr(expr, name),
                None => false,
            }) || s
                .test
                .as_ref()
                .is_some_and(|test| push_scan_expr(test, name))
                || s.update
                    .as_ref()
                    .is_some_and(|update| push_scan_expr(update, name))
                || push_scan_block(&s.body, name)
        }
        Statement::ForInStatement(s) => {
            (match &s.left {
                ForInLefthand::VariableDeclaration(decl) => push_scan_var_decl(decl, name),
                ForInLefthand::Expression(expr) => push_scan_expr(expr, name),
            }) || push_scan_expr(&s.right, name)
                || push_scan_stmt(&s.body, name)
        }
        Statement::ForOfStatement(s) => {
            (match &s.left {
                ForOfLefthand::VariableDeclaration(decl) => push_scan_var_decl(decl, name),
                ForOfLefthand::Expression(expr) => push_scan_expr(expr, name),
            }) || push_scan_expr(&s.right, name)
                || push_scan_stmt(&s.body, name)
        }
        Statement::WhileStatement(s) => {
            push_scan_expr(&s.test, name) || push_scan_block(&s.body, name)
        }
        Statement::DoWhileStatement(s) => {
            push_scan_block(&s.body, name) || push_scan_expr(&s.test, name)
        }
        // Nested function bodies ARE walked (conservative).
        Statement::FunctionDeclaration(decl) => push_scan_block(&decl.body, name),
        Statement::VariableDeclaration(decl) => push_scan_var_decl(decl, name),
    }
}

fn push_scan_var_decl(decl: &kali_ast::VariableDeclaration, name: &str) -> bool {
    decl.declarations.iter().any(|declarator| {
        declarator
            .init
            .as_ref()
            .is_some_and(|init| push_scan_expr(init, name))
    })
}

fn push_scan_expr(expr: &Expression, name: &str) -> bool {
    match expr {
        Expression::Identifier(_)
        | Expression::Literal(_)
        | Expression::BigIntLiteral(_)
        | Expression::MetaProperty(_)
        | Expression::JsxEmptyExpression
        | Expression::ThisExpression
        | Expression::SuperExpression
        | Expression::PrivateIdentifier(_) => false,
        Expression::BinaryExpression(e) => {
            push_scan_expr(&e.left, name) || push_scan_expr(&e.right, name)
        }
        Expression::UnaryExpression(e) => push_scan_expr(&e.argument, name),
        Expression::CallExpression(call) => {
            // THE match: `<name>.push(...)` (paren-stripped receiver).
            if let Expression::MemberExpression(member) = strip_parens(&call.callee) {
                if member.computed_index.is_none()
                    && member.dot_name() == Some("push")
                    && matches!(strip_parens(&member.object),
                        Expression::Identifier(object) if object == name)
                {
                    return true;
                }
            }
            push_scan_expr(&call.callee, name)
                || call.args.iter().any(|arg| push_scan_expr(arg, name))
        }
        Expression::MemberExpression(member) => {
            push_scan_expr(&member.object, name)
                || member
                    .computed_index
                    .as_ref()
                    .is_some_and(|index| push_scan_expr(index, name))
        }
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            Some(ExpressionOrSpread::Expression(expr)) => push_scan_expr(expr, name),
            Some(ExpressionOrSpread::Spread(spread)) => push_scan_expr(&spread.argument, name),
            Some(ExpressionOrSpread::Empty) | None => false,
        }),
        Expression::ObjectExpression(object) => object
            .properties
            .iter()
            .any(|property| push_scan_expr(&property.value, name)),
        // Nested closure bodies ARE walked (conservative).
        Expression::FunctionExpression(func) => func
            .body
            .as_ref()
            .is_some_and(|body| push_scan_block(body, name)),
        Expression::ArrowFunctionExpression(arrow) => push_scan_expr(&arrow.body, name),
        // Cannot see through — conservative TRUE (reject).
        Expression::ClassExpression(_) | Expression::JsxElement(_) | Expression::JsxFragment(_) => {
            true
        }
        Expression::NewExpression(e) => {
            push_scan_expr(&e.callee, name) || e.args.iter().any(|arg| push_scan_expr(arg, name))
        }
        Expression::TemplateLiteral(template) => template
            .expressions
            .iter()
            .any(|expr| push_scan_expr(expr, name)),
        Expression::TaggedTemplateExpression(e) => {
            push_scan_expr(&e.tag, name)
                || e.template
                    .expressions
                    .iter()
                    .any(|expr| push_scan_expr(expr, name))
        }
        Expression::UpdateExpression(e) => push_scan_expr(&e.argument, name),
        Expression::AssignmentExpression(e) => {
            push_scan_expr(&e.left, name) || push_scan_expr(&e.right, name)
        }
        Expression::LogicalExpression(e) => {
            push_scan_expr(&e.left, name) || push_scan_expr(&e.right, name)
        }
        Expression::ConditionalExpression(e) => {
            push_scan_expr(&e.test, name)
                || push_scan_expr(&e.consequent, name)
                || push_scan_expr(&e.alternate, name)
        }
        Expression::SequenceExpression(e) => {
            e.expressions.iter().any(|expr| push_scan_expr(expr, name))
        }
        Expression::ParenthesizedExpression(e) => push_scan_expr(&e.expression, name),
        Expression::YieldExpression(e) => e
            .argument
            .as_ref()
            .is_some_and(|arg| push_scan_expr(arg, name)),
        Expression::AwaitExpression(e) => push_scan_expr(&e.argument, name),
        Expression::OptionalChainExpression(e) => match e.inner.as_ref() {
            kali_ast::OptionalChainInner::NonNull { object, .. } => push_scan_expr(object, name),
        },
        Expression::ChainExpression(e) => push_scan_expr(&e.expression, name),
        Expression::SpreadElement(e) => push_scan_expr(&e.argument, name),
        Expression::RestElement(e) => push_scan_expr(&e.argument, name),
        Expression::ImportExpression(e) => push_scan_expr(&e.source, name),
        Expression::DecoratedExpression(e) => push_scan_expr(&e.expression, name),
        Expression::TypeAssertion(e) => push_scan_expr(&e.expression, name),
        Expression::SatisfiesExpression(e) => push_scan_expr(&e.expression, name),
    }
}

/// True for the scalar-shaped expressions admitted as push arguments, array
/// seeds, and computed indices in Task 2's i64 lane: numeric literals, bare
/// identifiers (`allow_identifiers` — repr-checked later at `emit_table`),
/// and unary/binary arithmetic over such. Everything else (calls, members,
/// objects, arrays, strings, booleans, templates, …) is out — it could
/// deliver a value this lane would store raw and read back wrong.
fn scalar_value_shape_ok(expr: &Expression, allow_identifiers: bool) -> bool {
    match expr {
        Expression::Literal(LiteralValue::Number(_)) => true,
        Expression::Identifier(_) => allow_identifiers,
        Expression::ParenthesizedExpression(inner) => {
            scalar_value_shape_ok(&inner.expression, allow_identifiers)
        }
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator.as_str(), "-" | "+" | "~")
                && scalar_value_shape_ok(&unary.argument, allow_identifiers)
        }
        Expression::BinaryExpression(binary) => {
            matches!(
                binary.operator.as_str(),
                "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>" | ">>>"
            ) && scalar_value_shape_ok(&binary.left, allow_identifiers)
                && scalar_value_shape_ok(&binary.right, allow_identifiers)
        }
        _ => false,
    }
}

/// The growable-i64 array SHAPE predicate: true when `expr` is an array
/// LITERAL every element of which is a scalar-shaped seed (numeric literal or
/// arithmetic over literals — NO identifiers, whose value the emit-time repr
/// gate cannot see on the element axis, and no strings/objects/arrays). This
/// is the exact declarator-init check `variable_declaration` applies to bare
/// bindings; `kali_types::repr_infer` reuses it to classify object-literal
/// array FIELDS (Stage P2 Lane 1 Task 3) off the SAME provenance — an
/// array-shaped field that fails this predicate fails closed (a shape
/// conflict) rather than silently interning a scalar field repr.
pub(crate) fn array_literal_of_scalar_seeds(expr: &Expression) -> bool {
    matches!(expr, Expression::ArrayExpression(array)
    if array.elements.iter().all(|element| matches!(
        element,
        Some(ExpressionOrSpread::Expression(inner))
            if scalar_value_shape_ok(inner, false)
    )))
}

fn strip_parens(expr: &Expression) -> &Expression {
    let mut current = expr;
    while let Expression::ParenthesizedExpression(inner) = current {
        current = &inner.expression;
    }
    current
}
