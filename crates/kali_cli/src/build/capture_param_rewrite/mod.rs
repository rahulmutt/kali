//! The captured-parameter rewrite (captured-bindings spec §3.2, A-1, A-2.2).
//!
//! ```js
//! function f(k) { BODY }                   // before
//! function f(k{p}) { let k = k{p}; BODY }  // after, when a nested function references `k`
//! ```
//!
//! Only the parameter's spelling changes, so `BODY`, call sites and arity are
//! untouched, and the captured `k` takes the local cell path. An arrow with
//! an expression body is first given a block body. Every captured parameter
//! is a simple identifier (A-2.2: the parser refuses the other forms), so
//! every one is rewritten.
//!
//! Two passes, the same shape as the block-scope rename:
//! * **Pass A** runs `capture_refusals`' [`Recorder`] over the rename walk and
//!   collects, per frame (Module / Function scope, numbered in enter order),
//!   the parameters some reference in another frame resolves to.
//! * **Pass B** is the traversal below. It cannot use `walk::walk_program`,
//!   because it replaces whole expressions, so it mirrors `walk.rs` variant
//!   for variant (no `_ =>` arm anywhere, so a new AST variant fails to
//!   compile here) and advances its frame counter at exactly the nodes where
//!   `walk.rs` calls `enter(ScopeKind::Module | ScopeKind::Function, …)`, in
//!   the same order. Every build (release included) asserts both passes
//!   counted the same number of frames, so a desync panics instead of
//!   rewriting the wrong frame's parameters.
//!
//! The pass is idempotent: the body reads `k`, never `k{p}`, so a `{p}`
//! parameter is never captured.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::*;

use super::block_scope_rename::walk::{self, BindKind};
use super::capture_refusals::Recorder;

#[cfg(test)]
#[path = "capture_param_rewrite_tests.rs"]
mod capture_param_rewrite_tests;

/// Rewrite every parameter a nested function captures into a `let`
/// initialized from the renamed parameter. Returns the number of rewritten
/// parameters; a program with none is left untouched.
pub fn rewrite_captured_params(statements: &mut [Statement]) -> usize {
    let mut recorder = Recorder::default();
    walk::walk_program(statements, &mut recorder);
    let captured = captured_params(&recorder);
    if captured.is_empty() {
        return 0;
    }
    let mut rewriter = Rewriter {
        captured,
        next_frame: 0,
        rewritten: 0,
    };
    rewriter.program(statements);
    assert_eq!(
        rewriter.next_frame,
        recorder.frame_count(),
        "capture_param_rewrite: pass B must count the frames pass A counted"
    );
    assert!(
        rewriter.captured.is_empty(),
        "every captured frame is visited"
    );
    rewriter.rewritten
}

/// Pass A: frame -> the names of its parameters referenced from another frame.
fn captured_params(recorder: &Recorder) -> BTreeMap<usize, BTreeSet<String>> {
    let mut captured: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for (scope, name) in recorder.references() {
        let Some((binding_scope, BindKind::Param)) = recorder.resolve(scope, name) else {
            continue;
        };
        let owner = recorder.frame_of(binding_scope);
        if owner != recorder.frame_of(scope) {
            captured.entry(owner).or_default().insert(name.to_string());
        }
    }
    captured
}

struct Rewriter {
    captured: BTreeMap<usize, BTreeSet<String>>,
    next_frame: usize,
    rewritten: usize,
}

impl Rewriter {
    /// Mirrors `walk.rs`'s `enter(Module | Function, …)`: returns the frame's
    /// captured parameters, if any.
    fn enter_frame(&mut self) -> Option<BTreeSet<String>> {
        let frame = self.next_frame;
        self.next_frame += 1;
        self.captured.remove(&frame)
    }

    /// Respell each captured parameter and return the `let` prologue, one
    /// declaration per parameter in parameter order.
    fn respell<'p>(
        &mut self,
        params: impl Iterator<Item = &'p mut String>,
        names: &BTreeSet<String>,
    ) -> Vec<Statement> {
        let mut prologue = Vec::new();
        for param in params {
            if !names.contains(param.as_str()) {
                continue;
            }
            let spelled = kali_common::captured_param_spelling(param);
            let original = std::mem::replace(param, spelled.clone());
            prologue.push(Statement::VariableDeclaration(VariableDeclaration {
                kind: "let".into(),
                declarations: vec![VariableDeclarator {
                    id: original,
                    init: Some(Expression::Identifier(spelled)),
                }],
            }));
            self.rewritten += 1;
        }
        prologue
    }

    /// A block-bodied function-like node: walk the body, then rewrite.
    fn function_parts<'p>(
        &mut self,
        params: impl Iterator<Item = &'p mut String>,
        body: Option<&mut BlockStatement>,
    ) {
        let captured = self.enter_frame();
        let Some(body) = body else {
            return;
        };
        self.statements(&mut body.body);
        if let Some(names) = captured {
            let prologue = self.respell(params, &names);
            body.body.splice(0..0, prologue);
        }
    }

    fn program(&mut self, statements: &mut [Statement]) {
        let captured = self.enter_frame();
        debug_assert!(captured.is_none(), "the module root has no parameters");
        self.statements(statements);
    }

    fn statements(&mut self, statements: &mut [Statement]) {
        for statement in statements.iter_mut() {
            self.statement(statement);
        }
    }

    fn block(&mut self, block: &mut BlockStatement) {
        self.statements(&mut block.body);
    }

    fn var_decl(&mut self, decl: &mut VariableDeclaration) {
        for declarator in decl.declarations.iter_mut() {
            if let Some(init) = declarator.init.as_mut() {
                self.expression(init);
            }
        }
    }

    /// `walk_class_body`: methods first, then field initializers.
    fn class_body(&mut self, body: &mut ClassBody) {
        for method in body.methods.iter_mut() {
            self.function_parts(method.params.iter_mut(), method.body.as_deref_mut());
        }
        for field in body.fields.iter_mut() {
            if let Some(value) = field.value.as_mut() {
                self.expression(value);
            }
        }
    }

    fn statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::ExpressionStatement(s) => self.expression(&mut s.expression),
            Statement::BreakStatement(_) => {}
            Statement::ContinueStatement(_) => {}
            Statement::DebuggerStatement(_) => {}
            Statement::WithStatement(s) => {
                self.expression(&mut s.object);
                self.statement(&mut s.body);
            }
            Statement::ReturnStatement(s) => {
                if let Some(argument) = s.argument.as_mut() {
                    self.expression(argument);
                }
            }
            Statement::LabeledStatement(s) => self.statement(&mut s.body),
            Statement::IfStatement(s) => {
                self.expression(&mut s.test);
                self.block(&mut s.consequent);
                if let Some(alternate) = s.alternate.as_mut() {
                    self.block(alternate);
                }
            }
            Statement::SwitchStatement(s) => {
                self.expression(&mut s.discriminant);
                for case in s.cases.iter_mut() {
                    if let Some(test) = case.test.as_mut() {
                        self.expression(test);
                    }
                    self.statements(&mut case.consequent);
                }
            }
            Statement::ThrowStatement(s) => self.expression(&mut s.argument),
            Statement::TryStatement(s) => {
                self.block(&mut s.block);
                if let Some(handler) = s.handler.as_mut() {
                    self.block(&mut handler.body);
                }
                if let Some(finalizer) = s.finalizer.as_mut() {
                    self.block(finalizer);
                }
            }
            Statement::BlockStatement(b) => self.block(b),
            Statement::ForStatement(s) => {
                match s.init.as_mut() {
                    Some(ForInit::VariableDeclaration(decl)) => self.var_decl(decl),
                    Some(ForInit::Expression(expr)) => self.expression(expr),
                    None => {}
                }
                if let Some(test) = s.test.as_mut() {
                    self.expression(test);
                }
                if let Some(update) = s.update.as_mut() {
                    self.expression(update);
                }
                self.block(&mut s.body);
            }
            Statement::ForInStatement(s) => {
                match &mut s.left {
                    ForInLefthand::VariableDeclaration(decl) => self.var_decl(decl),
                    ForInLefthand::Expression(expr) => self.expression(expr),
                }
                self.expression(&mut s.right);
                self.statement(&mut s.body);
            }
            Statement::ForOfStatement(s) => {
                match &mut s.left {
                    ForOfLefthand::VariableDeclaration(decl) => self.var_decl(decl),
                    ForOfLefthand::Expression(expr) => self.expression(expr),
                }
                self.expression(&mut s.right);
                self.statement(&mut s.body);
            }
            Statement::WhileStatement(s) => {
                self.expression(&mut s.test);
                self.block(&mut s.body);
            }
            Statement::DoWhileStatement(s) => {
                self.block(&mut s.body);
                self.expression(&mut s.test);
            }
            Statement::FunctionDeclaration(f) => {
                self.function_parts(f.params.iter_mut(), Some(&mut f.body));
            }
            Statement::ClassDeclaration(c) => self.class_body(&mut c.body),
            Statement::VariableDeclaration(d) => self.var_decl(d),
            Statement::ImportDeclaration(_) => {}
            Statement::ExportAll(_) => {}
            Statement::ExportNamed(_) => {}
            Statement::ExportDefault(d) => match d {
                ExportDefaultDeclaration::Expression(e) => self.expression(e),
                ExportDefaultDeclaration::FunctionDeclaration(f) => {
                    self.function_parts(f.params.iter_mut(), Some(&mut f.body));
                }
                ExportDefaultDeclaration::ClassDeclaration(c) => self.class_body(&mut c.body),
            },
            Statement::EnumDeclaration(e) => {
                for member in e.members.iter_mut() {
                    if let Some(value) = member.value.as_mut() {
                        self.expression(value);
                    }
                }
            }
            Statement::TypeAliasDeclaration(_) => {}
            Statement::InterfaceDeclaration(_) => {}
        }
    }

    fn expression_or_spread(&mut self, element: &mut ExpressionOrSpread) {
        match element {
            ExpressionOrSpread::Expression(expr) => self.expression(expr),
            ExpressionOrSpread::Spread(spread) => self.expression(&mut spread.argument),
            ExpressionOrSpread::Empty => {}
        }
    }

    fn expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Identifier(_) => {}
            Expression::Literal(_) => {}
            Expression::MetaProperty(_) => {}
            Expression::ThisExpression => {}
            Expression::SuperExpression => {}
            Expression::PrivateIdentifier(_) => {}
            Expression::BigIntLiteral(_) => {}
            Expression::JsxEmptyExpression => {}
            Expression::BinaryExpression(e) => {
                self.expression(&mut e.left);
                self.expression(&mut e.right);
            }
            Expression::UnaryExpression(e) => self.expression(&mut e.argument),
            Expression::CallExpression(e) => {
                self.expression(&mut e.callee);
                for arg in e.args.iter_mut() {
                    self.expression(arg);
                }
            }
            Expression::NewExpression(e) => {
                self.expression(&mut e.callee);
                for arg in e.args.iter_mut() {
                    self.expression(arg);
                }
            }
            Expression::MemberExpression(e) => {
                self.expression(&mut e.object);
                if let Some(index) = e.computed_index.as_mut() {
                    self.expression(index);
                }
            }
            Expression::ArrayExpression(e) => {
                for element in e.elements.iter_mut().flatten() {
                    self.expression_or_spread(element);
                }
            }
            Expression::ObjectExpression(e) => {
                for property in e.properties.iter_mut() {
                    self.expression(&mut property.value);
                }
            }
            Expression::FunctionExpression(f) => {
                let f = f.as_mut();
                self.function_parts(
                    f.params.iter_mut().map(|p| &mut p.name),
                    f.body.as_deref_mut(),
                );
            }
            Expression::ArrowFunctionExpression(a) => {
                let captured = self.enter_frame();
                self.expression(&mut a.body);
                if let Some(names) = captured {
                    let prologue = self.respell(a.params.iter_mut().map(|p| &mut p.name), &names);
                    let block = block_arrow(a, prologue);
                    *expr = block;
                }
            }
            Expression::ClassExpression(c) => self.class_body(&mut c.body),
            Expression::TemplateLiteral(t) => {
                for expression in t.expressions.iter_mut() {
                    self.expression(expression);
                }
            }
            Expression::TaggedTemplateExpression(t) => {
                self.expression(&mut t.tag);
                for expression in t.template.expressions.iter_mut() {
                    self.expression(expression);
                }
            }
            Expression::UpdateExpression(e) => self.expression(&mut e.argument),
            Expression::ParenthesizedExpression(e) => self.expression(&mut e.expression),
            Expression::AwaitExpression(e) => self.expression(&mut e.argument),
            Expression::YieldExpression(e) => {
                if let Some(argument) = e.argument.as_mut() {
                    self.expression(argument);
                }
            }
            Expression::ChainExpression(e) => self.expression(&mut e.expression),
            Expression::SpreadElement(e) => self.expression(&mut e.argument),
            Expression::RestElement(e) => self.expression(&mut e.argument),
            Expression::ImportExpression(e) => self.expression(&mut e.source),
            Expression::DecoratedExpression(e) => self.expression(&mut e.expression),
            Expression::TypeAssertion(e) => self.expression(&mut e.expression),
            Expression::SatisfiesExpression(e) => self.expression(&mut e.expression),
            Expression::AssignmentExpression(e) => {
                self.expression(&mut e.left);
                self.expression(&mut e.right);
            }
            Expression::LogicalExpression(e) => {
                self.expression(&mut e.left);
                self.expression(&mut e.right);
            }
            Expression::ConditionalExpression(e) => {
                self.expression(&mut e.test);
                self.expression(&mut e.consequent);
                self.expression(&mut e.alternate);
            }
            Expression::SequenceExpression(e) => {
                for expression in e.expressions.iter_mut() {
                    self.expression(expression);
                }
            }
            Expression::OptionalChainExpression(e) => match e.inner.as_mut() {
                OptionalChainInner::NonNull { object, .. } => self.expression(object),
            },
            Expression::JsxElement(e) => self.jsx_element(e),
            Expression::JsxFragment(f) => self.jsx_fragment(f),
        }
    }

    fn jsx_element(&mut self, element: &mut JsxElement) {
        for attribute in element.opening_element.attributes.iter_mut() {
            match attribute {
                JsxAttributeItem::JsxAttribute(attribute) => {
                    self.jsx_attribute_value(&mut attribute.value)
                }
                JsxAttributeItem::JsxSpreadAttribute(spread) => {
                    self.expression(&mut spread.argument)
                }
            }
        }
        for child in element.children.iter_mut() {
            self.jsx_child(child);
        }
    }

    fn jsx_fragment(&mut self, fragment: &mut JsxFragment) {
        for child in fragment.children.iter_mut() {
            self.jsx_child(child);
        }
    }

    fn jsx_attribute_value(&mut self, value: &mut JsxAttributeValue) {
        match value {
            JsxAttributeValue::String(_) => {}
            JsxAttributeValue::JsxElement(element) => self.jsx_element(element),
            JsxAttributeValue::JsxExpression(container) => self.jsx_container(container),
        }
    }

    fn jsx_container(&mut self, container: &mut JsxExpressionContainer) {
        if let Some(expression) = container.expression.as_mut() {
            self.expression(expression);
        }
    }

    fn jsx_child(&mut self, child: &mut JsxChild) {
        match child {
            JsxChild::JsxText(_) => {}
            JsxChild::JsxExpression(container) => self.jsx_container(container),
            JsxChild::JsxElement(element) => self.jsx_element(element),
            JsxChild::JsxFragment(fragment) => self.jsx_fragment(fragment),
        }
    }
}

/// `(k{p}) => e` becomes `(k{p}) => { let k = k{p}; return e; }`, a
/// block-bodied arrow (`FunctionExpression { is_arrow: true }`, the parser's
/// shape for `(…) => { … }`). The return-type annotation is carried over.
fn block_arrow(arrow: &mut ArrowFunctionExpression, prologue: Vec<Statement>) -> Expression {
    let body = std::mem::replace(&mut arrow.body, Expression::ThisExpression);
    let mut statements = prologue;
    statements.push(Statement::ReturnStatement(ReturnStatement {
        argument: Some(body),
    }));
    Expression::FunctionExpression(Box::new(FunctionExpression {
        returnType: arrow.returnType.take(),
        id: arrow.id.take(),
        params: std::mem::take(&mut arrow.params),
        body: Some(Box::new(BlockStatement { body: statements })),
        is_async: arrow.is_async,
        generator: false,
        is_arrow: true,
    }))
}
