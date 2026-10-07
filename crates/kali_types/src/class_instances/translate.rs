//! Translates each rewritten class declaration to a factory `C__new` and one
//! `C__m(__this, …)` per instance method (spec §3.5).

use std::collections::BTreeSet;

use kali_ast::{
    AssignmentExpression, AssignmentOperator, BlockStatement, ClassBody, ClassDeclaration,
    Expression, ExpressionStatement, FunctionDeclaration, MethodDefinition, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, PropertyName, ReturnStatement, Statement,
    VariableDeclaration, VariableDeclarator,
};

use super::classes::{ClassPlans, RewrittenClass};
use super::walk::{walk, Cx, FrameKind, Pos, Visitor};

const THIS: &str = "__this";

pub(crate) fn translate_classes(statements: &mut Vec<Statement>, plans: &ClassPlans) {
    walk(statements, &mut Translate { plans });
}

struct Translate<'p> {
    plans: &'p ClassPlans,
}

impl Visitor for Translate<'_> {
    fn stmts(&mut self, list: &mut Vec<Statement>, _cx: &Cx) {
        if !list.iter().any(|s| self.plan_of(s).is_some()) {
            return;
        }
        let old = std::mem::take(list);
        for statement in old {
            match self.plan_of(&statement) {
                Some(plan) => {
                    let Statement::ClassDeclaration(class) = statement else {
                        unreachable!()
                    };
                    list.extend(translate_class(&class, plan));
                }
                None => list.push(statement),
            }
        }
    }
}

impl<'p> Translate<'p> {
    fn plan_of(&self, statement: &Statement) -> Option<&'p RewrittenClass> {
        match statement {
            Statement::ClassDeclaration(class) => self.plans.rewritten.get(&class.name),
            _ => None,
        }
    }
}

fn ident(name: &str) -> Expression {
    Expression::Identifier(name.to_string())
}

fn declare(kind: &str, name: &str, init: Expression) -> Statement {
    Statement::VariableDeclaration(VariableDeclaration {
        declarations: vec![VariableDeclarator {
            id: name.to_string(),
            init: Some(init),
        }],
        kind: kind.to_string(),
    })
}

fn field_var(field: &str) -> String {
    format!("__f_{field}")
}

fn return_this() -> Statement {
    Statement::ReturnStatement(ReturnStatement {
        argument: Some(ident(THIS)),
    })
}

fn function(
    name: String,
    params: Vec<String>,
    body: Vec<Statement>,
    source: Option<&MethodDefinition>,
) -> Statement {
    Statement::FunctionDeclaration(FunctionDeclaration {
        name,
        params,
        defaults: Vec::new(),
        body: Box::new(BlockStatement { body }),
        is_async: source.is_some_and(|m| m.is_async),
        generator: source.is_some_and(|m| m.generator),
    })
}

/// The statements `class` becomes: its factory, then its methods.
fn translate_class(class: &ClassDeclaration, plan: &RewrittenClass) -> Vec<Statement> {
    let mut out = vec![factory(&class.body, plan)];
    for method in class
        .body
        .methods
        .iter()
        .filter(|m| !m.is_static && m.name != "constructor")
    {
        let mut body = method
            .body
            .as_deref()
            .cloned()
            .unwrap_or(BlockStatement { body: vec![] })
            .body;
        replace_this(&mut body, Replace::This);
        let params = std::iter::once(THIS.to_string())
            .chain(method.params.iter().cloned())
            .collect();
        out.push(function(
            format!("{}__{}", plan.name, method.name),
            params,
            body,
            Some(method),
        ));
    }
    out
}

fn factory(body: &ClassBody, plan: &RewrittenClass) -> Statement {
    let mut stmts = Vec::new();
    let mut declared = BTreeSet::new();
    for field in body.fields.iter().filter(|f| !f.is_static) {
        if let Some(value) = &field.value {
            let mut value = vec![expression_statement(value.clone())];
            replace_this(&mut value, Replace::FieldReads);
            stmts.push(declare(
                "let",
                &field_var(&field.name),
                take_expression(value),
            ));
            declared.insert(field.name.clone());
        }
    }
    let constructor = body
        .methods
        .iter()
        .find(|m| m.name == "constructor" && !m.is_static);
    let ctor_body: Vec<Statement> = constructor
        .and_then(|m| m.body.as_deref())
        .map(|b| b.body.clone())
        .unwrap_or_default();
    let (run, rest) = ctor_body.split_at(plan.leading_run.min(ctor_body.len()));
    for statement in run {
        let (field, value) = run_member(statement);
        let mut value = vec![expression_statement(value)];
        replace_this(&mut value, Replace::FieldReads);
        let value = take_expression(value);
        if declared.insert(field.clone()) {
            stmts.push(declare("let", &field_var(&field), value));
        } else {
            stmts.push(expression_statement(Expression::AssignmentExpression(
                Box::new(AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: ident(&field_var(&field)),
                    right: value,
                }),
            )));
        }
    }
    let properties = plan
        .fields
        .iter()
        .map(|f| ObjectProperty {
            key: PropertyName::Identifier(f.clone()),
            value: ident(&field_var(f)),
            kind: ObjectPropertyKind::Init,
        })
        .collect();
    stmts.push(declare(
        "const",
        THIS,
        Expression::ObjectExpression(ObjectExpression { properties }),
    ));
    let mut rest = rest.to_vec();
    replace_this(&mut rest, Replace::This);
    replace_bare_returns(&mut rest);
    stmts.extend(rest);
    stmts.push(return_this());
    function(
        format!("{}__new", plan.name),
        plan.ctor_params.clone(),
        stmts,
        None,
    )
}

fn expression_statement(expression: Expression) -> Statement {
    Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(expression),
    })
}

fn take_expression(mut statements: Vec<Statement>) -> Expression {
    match statements.pop() {
        Some(Statement::ExpressionStatement(s)) => *s.expression,
        _ => unreachable!("wrapped by expression_statement"),
    }
}

/// `this.f = e` as `(f, e)`; planning guarantees the leading run has this shape.
fn run_member(statement: &Statement) -> (String, Expression) {
    let Statement::ExpressionStatement(stmt) = statement else {
        unreachable!("leading run")
    };
    let Expression::AssignmentExpression(assign) = stmt.expression.as_ref() else {
        unreachable!("leading run")
    };
    let Expression::MemberExpression(member) = &assign.left else {
        unreachable!("leading run")
    };
    (
        member.property.clone().unwrap_or_default(),
        assign.right.clone(),
    )
}

#[derive(Clone, Copy)]
enum Replace {
    /// `this` becomes `__this`.
    This,
    /// `this.g` becomes `__f_g` (field initializers and the leading run).
    FieldReads,
}

/// Replaces `this` in `statements` per `mode`, where `this` is the one of the
/// code being translated: arrows are descended into; nested functions,
/// function declarations and class bodies are not.
fn replace_this(statements: &mut Vec<Statement>, mode: Replace) {
    walk(statements, &mut ReplaceThis { mode });
}

struct ReplaceThis {
    mode: Replace,
}

impl Visitor for ReplaceThis {
    fn expr(&mut self, expr: &mut Expression, _pos: &Pos, cx: &Cx) {
        if !owns_this(cx) {
            return;
        }
        match (self.mode, &*expr) {
            (Replace::This, Expression::ThisExpression) => *expr = ident(THIS),
            (Replace::FieldReads, Expression::MemberExpression(m))
                if matches!(m.object, Expression::ThisExpression) && m.computed_index.is_none() =>
            {
                if let Some(field) = &m.property {
                    *expr = ident(&field_var(field));
                }
            }
            _ => {}
        }
    }
}

/// Whether the nearest non-arrow frame is the walk's root, the code being translated.
fn owns_this(cx: &Cx) -> bool {
    cx.frames
        .iter()
        .rev()
        .find(|f| f.kind != FrameKind::Arrow)
        .is_some_and(|f| f.kind == FrameKind::Program)
}

/// `return;` → `return __this;`, outside nested functions and arrows.
fn replace_bare_returns(statements: &mut Vec<Statement>) {
    walk(statements, &mut BareReturns);
}

struct BareReturns;

impl Visitor for BareReturns {
    fn stmts(&mut self, list: &mut Vec<Statement>, cx: &Cx) {
        if cx.frames.last().map(|f| &f.kind) != Some(&FrameKind::Program) {
            return;
        }
        for statement in list {
            if let Statement::ReturnStatement(ret) = statement {
                if ret.argument.is_none() {
                    ret.argument = Some(ident(THIS));
                }
            }
        }
    }
}
