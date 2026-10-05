//! Pass A/B shared AST walk for the block-scope rename (block-scoping spec
//! §3.1–§3.2). The walk reports scope entry/exit, binding sites and
//! identifier references to a [`Hooks`] implementation, in source order, and
//! hands out `&mut String` so the applier can rewrite names in place.
//!
//! ## Exhaustiveness
//!
//! Every match arm in this file is enumerated — no `_ =>` arm anywhere, so a
//! new `Statement`/`Expression`/JSX variant in `kali_ast` fails to compile
//! here. This mirrors `deny_import_positions_expression`
//! (`crates/kali_cli/src/build/module_link.rs`) and
//! `deny_import_positions_statement`, which enumerate every variant that
//! exists in `kali_ast` today; those two functions were the checklist while
//! writing the walk below, as for `name_anon_functions`. The compiler only
//! proves every *variant* is handled, not that every *field* of that variant
//! is recursed into, so the walk is also checked by inspection and test.

use kali_ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScopeKind {
    Module,
    Function,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindKind {
    Var,
    Lexical,
    Param,
    FunctionDecl,
    ClassDecl,
    FunctionExprId,
    ClassExprId,
}

impl BindKind {
    /// Function / class names are keyed program-wide downstream (spec §3.2).
    pub(crate) fn is_program_wide(self) -> bool {
        matches!(
            self,
            Self::FunctionDecl | Self::ClassDecl | Self::FunctionExprId | Self::ClassExprId
        )
    }
}

pub(crate) trait Hooks {
    /// `label` is the function's plan key (captured-bindings A-2.6): the
    /// declaration name, the expression's `id`, `<Class>__<method>`, or
    /// `None` when unknown or when `kind` is not `Function`.
    fn enter(&mut self, kind: ScopeKind, label: Option<&str>);
    fn exit(&mut self);
    fn bind(&mut self, name: &mut String, kind: BindKind);
    fn reference(&mut self, name: &mut String);
    /// A loop statement opens (head, test and body). The rename pass ignores
    /// it; the captured-bindings `check` pass uses it to stay off the
    /// per-iteration record lane (captured-bindings §3.4, A-2.5).
    fn enter_loop(&mut self) {}
    fn exit_loop(&mut self) {}
    /// A call expression, seen before its callee and arguments are walked.
    fn call(&mut self, _callee: &Expression) {}
    /// The initializer of the declarator just reported to `bind`, seen before
    /// it is walked.
    fn initializer(&mut self, _init: &Expression) {}
    /// An expression statement that is an assignment (its value is
    /// discarded), seen before it is walked.
    fn assignment_statement(&mut self, _assign: &kali_ast::AssignmentExpression) {}
}

pub(crate) fn walk_program(statements: &mut [Statement], hooks: &mut impl Hooks) {
    hooks.enter(ScopeKind::Module, None);
    walk_statements(statements, hooks);
    hooks.exit();
}

fn walk_block(block: &mut BlockStatement, hooks: &mut impl Hooks) {
    hooks.enter(ScopeKind::Block, None);
    walk_statements(&mut block.body, hooks);
    hooks.exit();
}

fn walk_statements(statements: &mut [Statement], hooks: &mut impl Hooks) {
    for statement in statements.iter_mut() {
        walk_statement(statement, hooks);
    }
}

fn walk_var_decl(decl: &mut VariableDeclaration, hooks: &mut impl Hooks) {
    let kind = if decl.kind == "var" {
        BindKind::Var
    } else {
        BindKind::Lexical
    };
    for declarator in decl.declarations.iter_mut() {
        hooks.bind(&mut declarator.id, kind);
        if let Some(init) = declarator.init.as_mut() {
            hooks.initializer(init);
            walk_expression(init, hooks);
        }
    }
}

/// Params and body share one Function scope (`function f(c){ const c }` is a
/// redeclaration, not a shadow).
fn walk_function_parts<'p>(
    params: impl Iterator<Item = &'p mut String>,
    body: Option<&mut BlockStatement>,
    own_id: Option<&mut String>,
    label: Option<&str>,
    hooks: &mut impl Hooks,
) {
    hooks.enter(ScopeKind::Function, label);
    if let Some(id) = own_id {
        hooks.bind(id, BindKind::FunctionExprId);
    }
    for param in params {
        hooks.bind(param, BindKind::Param);
    }
    if let Some(body) = body {
        walk_statements(&mut body.body, hooks);
    }
    hooks.exit();
}

fn walk_class_body(body: &mut ClassBody, class: Option<&str>, hooks: &mut impl Hooks) {
    for method in body.methods.iter_mut() {
        let label = class.map(|c| format!("{c}__{}", method.name));
        walk_function_parts(
            method.params.iter_mut(),
            method.body.as_deref_mut(),
            None,
            label.as_deref(),
            hooks,
        );
    }
    for field in body.fields.iter_mut() {
        if let Some(value) = field.value.as_mut() {
            walk_expression(value, hooks);
        }
    }
}

fn walk_super_class(super_class: Option<&mut String>, hooks: &mut impl Hooks) {
    if let Some(super_class) = super_class {
        hooks.reference(super_class);
    }
}

fn walk_statement(statement: &mut Statement, hooks: &mut impl Hooks) {
    match statement {
        Statement::ExpressionStatement(s) => {
            if let Expression::AssignmentExpression(assign) = &*s.expression {
                hooks.assignment_statement(assign);
            }
            walk_expression(&mut s.expression, hooks)
        }
        Statement::BreakStatement(_) => {}
        Statement::ContinueStatement(_) => {}
        Statement::DebuggerStatement(_) => {}
        Statement::WithStatement(s) => {
            walk_expression(&mut s.object, hooks);
            walk_statement(&mut s.body, hooks);
        }
        Statement::ReturnStatement(s) => {
            if let Some(argument) = s.argument.as_mut() {
                walk_expression(argument, hooks);
            }
        }
        Statement::LabeledStatement(s) => walk_statement(&mut s.body, hooks),
        Statement::IfStatement(s) => {
            walk_expression(&mut s.test, hooks);
            walk_block(&mut s.consequent, hooks);
            if let Some(alternate) = s.alternate.as_mut() {
                walk_block(alternate, hooks);
            }
        }
        Statement::SwitchStatement(s) => {
            walk_expression(&mut s.discriminant, hooks);
            hooks.enter(ScopeKind::Block, None);
            for case in s.cases.iter_mut() {
                if let Some(test) = case.test.as_mut() {
                    walk_expression(test, hooks);
                }
                walk_statements(&mut case.consequent, hooks);
            }
            hooks.exit();
        }
        Statement::ThrowStatement(s) => walk_expression(&mut s.argument, hooks),
        Statement::TryStatement(s) => {
            walk_block(&mut s.block, hooks);
            if let Some(handler) = s.handler.as_mut() {
                hooks.enter(ScopeKind::Block, None);
                hooks.bind(&mut handler.param, BindKind::Lexical);
                walk_block(&mut handler.body, hooks);
                hooks.exit();
            }
            if let Some(finalizer) = s.finalizer.as_mut() {
                walk_block(finalizer, hooks);
            }
        }
        Statement::BlockStatement(b) => walk_block(b, hooks),
        Statement::ForStatement(s) => {
            hooks.enter_loop();
            hooks.enter(ScopeKind::Block, None);
            match s.init.as_mut() {
                Some(ForInit::VariableDeclaration(decl)) => walk_var_decl(decl, hooks),
                Some(ForInit::Expression(expr)) => walk_expression(expr, hooks),
                None => {}
            }
            if let Some(test) = s.test.as_mut() {
                walk_expression(test, hooks);
            }
            if let Some(update) = s.update.as_mut() {
                walk_expression(update, hooks);
            }
            walk_block(&mut s.body, hooks);
            hooks.exit();
            hooks.exit_loop();
        }
        Statement::ForInStatement(s) => {
            hooks.enter_loop();
            hooks.enter(ScopeKind::Block, None);
            match &mut s.left {
                ForInLefthand::VariableDeclaration(decl) => walk_var_decl(decl, hooks),
                ForInLefthand::Expression(expr) => walk_expression(expr, hooks),
            }
            walk_expression(&mut s.right, hooks);
            walk_statement(&mut s.body, hooks);
            hooks.exit();
            hooks.exit_loop();
        }
        Statement::ForOfStatement(s) => {
            hooks.enter_loop();
            hooks.enter(ScopeKind::Block, None);
            match &mut s.left {
                ForOfLefthand::VariableDeclaration(decl) => walk_var_decl(decl, hooks),
                ForOfLefthand::Expression(expr) => walk_expression(expr, hooks),
            }
            walk_expression(&mut s.right, hooks);
            walk_statement(&mut s.body, hooks);
            hooks.exit();
            hooks.exit_loop();
        }
        Statement::WhileStatement(s) => {
            hooks.enter_loop();
            walk_expression(&mut s.test, hooks);
            walk_block(&mut s.body, hooks);
            hooks.exit_loop();
        }
        Statement::DoWhileStatement(s) => {
            hooks.enter_loop();
            walk_block(&mut s.body, hooks);
            walk_expression(&mut s.test, hooks);
            hooks.exit_loop();
        }
        Statement::FunctionDeclaration(f) => {
            hooks.bind(&mut f.name, BindKind::FunctionDecl);
            let label = f.name.clone();
            walk_function_parts(
                f.params.iter_mut(),
                Some(&mut f.body),
                None,
                Some(&label),
                hooks,
            );
        }
        Statement::ClassDeclaration(c) => {
            hooks.bind(&mut c.name, BindKind::ClassDecl);
            walk_super_class(c.super_class.as_mut(), hooks);
            let class = c.name.clone();
            walk_class_body(&mut c.body, Some(&class), hooks);
        }
        Statement::VariableDeclaration(d) => walk_var_decl(d, hooks),
        Statement::ImportDeclaration(i) => {
            for specifier in i.specifiers.iter_mut() {
                match specifier {
                    ImportSpecifier::Default(name) | ImportSpecifier::Namespace(name) => {
                        hooks.bind(name, BindKind::Lexical)
                    }
                    ImportSpecifier::Named(entries) | ImportSpecifier::Type(entries) => {
                        for entry in entries.iter_mut() {
                            hooks.bind(&mut entry.local, BindKind::Lexical);
                        }
                    }
                    ImportSpecifier::SideEffect => {}
                }
            }
        }
        Statement::ExportAll(_) => {}
        Statement::ExportNamed(e) => {
            if e.source.is_none() {
                for specifier in e.specifiers.iter_mut() {
                    hooks.reference(&mut specifier.local);
                }
            }
        }
        Statement::ExportDefault(d) => match d {
            ExportDefaultDeclaration::Expression(e) => walk_expression(e, hooks),
            ExportDefaultDeclaration::FunctionDeclaration(f) => {
                if !f.name.is_empty() {
                    hooks.bind(&mut f.name, BindKind::FunctionDecl);
                }
                let label = (!f.name.is_empty()).then(|| f.name.clone());
                walk_function_parts(
                    f.params.iter_mut(),
                    Some(&mut f.body),
                    None,
                    label.as_deref(),
                    hooks,
                );
            }
            ExportDefaultDeclaration::ClassDeclaration(c) => {
                if !c.name.is_empty() {
                    hooks.bind(&mut c.name, BindKind::ClassDecl);
                }
                walk_super_class(c.super_class.as_mut(), hooks);
                let class = (!c.name.is_empty()).then(|| c.name.clone());
                walk_class_body(&mut c.body, class.as_deref(), hooks);
            }
        },
        Statement::EnumDeclaration(e) => {
            hooks.bind(&mut e.name, BindKind::Lexical);
            for member in e.members.iter_mut() {
                if let Some(value) = member.value.as_mut() {
                    walk_expression(value, hooks);
                }
            }
        }
        Statement::TypeAliasDeclaration(_) => {}
        Statement::InterfaceDeclaration(_) => {}
    }
}

fn walk_expression_or_spread(element: &mut ExpressionOrSpread, hooks: &mut impl Hooks) {
    match element {
        ExpressionOrSpread::Expression(expr) => walk_expression(expr, hooks),
        ExpressionOrSpread::Spread(spread) => walk_expression(&mut spread.argument, hooks),
        ExpressionOrSpread::Empty => {}
    }
}

fn walk_expression(expr: &mut Expression, hooks: &mut impl Hooks) {
    match expr {
        Expression::Identifier(name) => hooks.reference(name),
        Expression::Literal(_) => {}
        Expression::MetaProperty(_) => {}
        Expression::ThisExpression => {}
        Expression::SuperExpression => {}
        Expression::PrivateIdentifier(_) => {}
        Expression::BigIntLiteral(_) => {}
        Expression::JsxEmptyExpression => {}
        Expression::BinaryExpression(e) => {
            walk_expression(&mut e.left, hooks);
            walk_expression(&mut e.right, hooks);
        }
        Expression::UnaryExpression(e) => walk_expression(&mut e.argument, hooks),
        Expression::CallExpression(e) => {
            hooks.call(&e.callee);
            walk_expression(&mut e.callee, hooks);
            for arg in e.args.iter_mut() {
                walk_expression(arg, hooks);
            }
        }
        Expression::NewExpression(e) => {
            walk_expression(&mut e.callee, hooks);
            for arg in e.args.iter_mut() {
                walk_expression(arg, hooks);
            }
        }
        // `property` is never touched: it is a member name, not a reference.
        Expression::MemberExpression(e) => {
            walk_expression(&mut e.object, hooks);
            if let Some(index) = e.computed_index.as_mut() {
                walk_expression(index, hooks);
            }
        }
        Expression::ArrayExpression(e) => {
            for element in e.elements.iter_mut().flatten() {
                walk_expression_or_spread(element, hooks);
            }
        }
        // Keys are never touched, so shorthand `{x}` becomes `{x: x{b1}}` (A-1).
        Expression::ObjectExpression(e) => {
            for property in e.properties.iter_mut() {
                walk_expression(&mut property.value, hooks);
            }
        }
        Expression::FunctionExpression(f) => {
            let label = f.id.clone();
            walk_function_parts(
                f.params.iter_mut().map(|p| &mut p.name),
                f.body.as_deref_mut(),
                f.id.as_mut(),
                label.as_deref(),
                hooks,
            )
        }
        Expression::ArrowFunctionExpression(a) => {
            let label = a.id.clone();
            hooks.enter(ScopeKind::Function, label.as_deref());
            for param in a.params.iter_mut() {
                hooks.bind(&mut param.name, BindKind::Param);
            }
            if let Some(id) = a.id.as_mut() {
                hooks.bind(id, BindKind::FunctionExprId);
            }
            walk_expression(&mut a.body, hooks);
            hooks.exit();
        }
        Expression::ClassExpression(c) => {
            hooks.enter(ScopeKind::Block, None);
            if let Some(id) = c.id.as_mut() {
                hooks.bind(id, BindKind::ClassExprId);
            }
            walk_super_class(c.super_class.as_mut(), hooks);
            let class = c.id.clone();
            walk_class_body(&mut c.body, class.as_deref(), hooks);
            hooks.exit();
        }
        Expression::TemplateLiteral(t) => {
            for expression in t.expressions.iter_mut() {
                walk_expression(expression, hooks);
            }
        }
        Expression::TaggedTemplateExpression(t) => {
            walk_expression(&mut t.tag, hooks);
            for expression in t.template.expressions.iter_mut() {
                walk_expression(expression, hooks);
            }
        }
        Expression::UpdateExpression(e) => walk_expression(&mut e.argument, hooks),
        Expression::ParenthesizedExpression(e) => walk_expression(&mut e.expression, hooks),
        Expression::AwaitExpression(e) => walk_expression(&mut e.argument, hooks),
        Expression::YieldExpression(e) => {
            if let Some(argument) = e.argument.as_mut() {
                walk_expression(argument, hooks);
            }
        }
        Expression::ChainExpression(e) => walk_expression(&mut e.expression, hooks),
        Expression::SpreadElement(e) => walk_expression(&mut e.argument, hooks),
        Expression::RestElement(e) => walk_expression(&mut e.argument, hooks),
        Expression::ImportExpression(e) => walk_expression(&mut e.source, hooks),
        Expression::DecoratedExpression(e) => walk_expression(&mut e.expression, hooks),
        Expression::TypeAssertion(e) => walk_expression(&mut e.expression, hooks),
        Expression::SatisfiesExpression(e) => walk_expression(&mut e.expression, hooks),
        Expression::AssignmentExpression(e) => {
            walk_expression(&mut e.left, hooks);
            walk_expression(&mut e.right, hooks);
        }
        Expression::LogicalExpression(e) => {
            walk_expression(&mut e.left, hooks);
            walk_expression(&mut e.right, hooks);
        }
        Expression::ConditionalExpression(e) => {
            walk_expression(&mut e.test, hooks);
            walk_expression(&mut e.consequent, hooks);
            walk_expression(&mut e.alternate, hooks);
        }
        Expression::SequenceExpression(e) => {
            for expression in e.expressions.iter_mut() {
                walk_expression(expression, hooks);
            }
        }
        Expression::OptionalChainExpression(e) => match e.inner.as_mut() {
            OptionalChainInner::NonNull { object, .. } => walk_expression(object, hooks),
        },
        Expression::JsxElement(e) => walk_jsx_element(e, hooks),
        Expression::JsxFragment(f) => walk_jsx_fragment(f, hooks),
    }
}

// JSX tag names are never touched; only attribute values, spread arguments,
// container expressions and children are walked.

fn walk_jsx_element(element: &mut JsxElement, hooks: &mut impl Hooks) {
    for attribute in element.opening_element.attributes.iter_mut() {
        walk_jsx_attribute_item(attribute, hooks);
    }
    for child in element.children.iter_mut() {
        walk_jsx_child(child, hooks);
    }
}

fn walk_jsx_fragment(fragment: &mut JsxFragment, hooks: &mut impl Hooks) {
    for child in fragment.children.iter_mut() {
        walk_jsx_child(child, hooks);
    }
}

fn walk_jsx_attribute_item(item: &mut JsxAttributeItem, hooks: &mut impl Hooks) {
    match item {
        JsxAttributeItem::JsxAttribute(attribute) => {
            walk_jsx_attribute_value(&mut attribute.value, hooks)
        }
        JsxAttributeItem::JsxSpreadAttribute(spread) => {
            walk_expression(&mut spread.argument, hooks)
        }
    }
}

fn walk_jsx_attribute_value(value: &mut JsxAttributeValue, hooks: &mut impl Hooks) {
    match value {
        JsxAttributeValue::String(_) => {}
        JsxAttributeValue::JsxElement(element) => walk_jsx_element(element, hooks),
        JsxAttributeValue::JsxExpression(container) => {
            walk_jsx_expression_container(container, hooks)
        }
    }
}

fn walk_jsx_expression_container(container: &mut JsxExpressionContainer, hooks: &mut impl Hooks) {
    if let Some(expression) = container.expression.as_mut() {
        walk_expression(expression, hooks);
    }
}

fn walk_jsx_child(child: &mut JsxChild, hooks: &mut impl Hooks) {
    match child {
        JsxChild::JsxText(_) => {}
        JsxChild::JsxExpression(container) => walk_jsx_expression_container(container, hooks),
        JsxChild::JsxElement(element) => walk_jsx_element(element, hooks),
        JsxChild::JsxFragment(fragment) => walk_jsx_fragment(fragment, hooks),
    }
}
