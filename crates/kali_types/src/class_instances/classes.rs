//! Class plans (spec A-1, A-3, A-8, §3.2, §3.4): which program classes are
//! rewritten to factories, their instance field sets, and the `new` refusals.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{
    AssignmentOperator, ClassBody, ClassDeclaration, ClassExpression, Expression, MethodKind,
    NewExpression, Statement, VariableDeclarator,
};
use kali_common::{
    class_construction_unavailable_message,
    class_field_initializer_this_message, class_field_without_initial_value_message,
    class_generated_name_collision_message, constructor_return_unavailable_message,
    plain_function_construction_unavailable_message, CLASS_REASON_ACCESSOR,
    CLASS_REASON_AMBIGUOUS, CLASS_REASON_COMPUTED, CLASS_REASON_EXPORTED, CLASS_REASON_EXPRESSION,
    CLASS_REASON_EXTENDS, CLASS_REASON_PRIVATE, CLASS_REASON_STATIC,
};

use kali_error::{_error_codes::e5, diagnostic::Diagnostic};

use super::walk::{walk, Cx, Pos, Visitor};
use crate::program_classes::ProgramClasses;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RewrittenClass {
    pub name: String,
    /// The instance field set, in first-binding order (spec §3.2).
    pub fields: Vec<String>,
    /// Instance method names, `constructor` excluded.
    pub methods: BTreeSet<String>,
    /// How many leading constructor statements are `this.f = e` run members (spec §3.2 step 2).
    pub leading_run: usize,
    /// The constructor's parameters (empty without a constructor).
    pub ctor_params: Vec<String>,
}

/// A class may be in `rewritten` while `diagnostics` is non-empty; any diagnostic stops compilation before lowering.
#[derive(Debug, Default)]
pub(crate) struct ClassPlans {
    pub rewritten: BTreeMap<String, RewrittenClass>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ClassPlans {
    /// The first rewritten class (by name) with that instance method.
    pub(crate) fn method_owner(&self, method: &str) -> Option<&str> {
        self.rewritten
            .values()
            .find(|class| class.methods.contains(method))
            .map(|class| class.name.as_str())
    }
}

/// `new X(a)` in either parse shape (A-9): `callee: Identifier(X)`, or
/// `callee` a member/call chain whose deepest node is `Call(Identifier(X), a)`
/// with `args` empty. Returns `X`.
pub(crate) fn new_target(new: &NewExpression) -> Option<&str> {
    if let Expression::Identifier(name) = &new.callee {
        return Some(name);
    }
    if !new.args.is_empty() {
        return None;
    }
    let mut node = &new.callee;
    loop {
        match node {
            Expression::MemberExpression(m) => node = &m.object,
            Expression::CallExpression(c) => match &c.callee {
                Expression::Identifier(name) => return Some(name),
                inner => node = inner,
            },
            _ => return None,
        }
    }
}

/// One declaration (or expression) of a class name.
struct Decl {
    body: ClassBody,
    super_class: Option<String>,
    is_expression: bool,
}

#[derive(Default)]
struct Facts {
    decls: BTreeMap<String, Vec<Decl>>,
    exported: BTreeSet<String>,
    constructed: BTreeSet<String>,
    functions: BTreeSet<String>,
}

impl Facts {
    fn add_class(&mut self, name: &str, super_class: &Option<String>, body: &ClassBody, is_expression: bool) {
        self.decls.entry(name.to_string()).or_default().push(Decl {
            body: body.clone(),
            super_class: super_class.clone(),
            is_expression,
        });
    }
}

impl Visitor for Facts {
    fn expr(&mut self, expr: &mut Expression, _pos: &Pos, _cx: &Cx) {
        if let Expression::NewExpression(new) = expr {
            if let Some(target) = new_target(new) {
                self.constructed.insert(target.to_string());
            }
        }
    }
    fn class_decl(&mut self, class: &ClassDeclaration, exported_default: bool, _cx: &Cx) {
        self.add_class(&class.name, &class.super_class, &class.body, false);
        if exported_default {
            self.exported.insert(class.name.clone());
        }
    }
    fn class_expr(&mut self, class: &ClassExpression, _cx: &Cx) {
        if let Some(id) = &class.id {
            self.add_class(id, &class.super_class, &class.body, true);
        }
    }
    fn var_declarator(&mut self, d: &VariableDeclarator, _kind: &str, _cx: &Cx) {
        if let Some(Expression::ClassExpression(class)) = &d.init {
            self.add_class(&d.id, &class.super_class, &class.body, true);
        }
    }
    fn function_decl(&mut self, f: &kali_ast::FunctionDeclaration, _cx: &Cx) {
        self.functions.insert(f.name.clone());
    }
    fn export_specifier(&mut self, local: &str) {
        self.exported.insert(local.to_string());
    }
}

/// Every `this` in some code, each as `Some(g)` for a plain read `this.g` and
/// `None` for any other use (bare, called, written). Nested arrows and
/// functions are included.
#[derive(Default)]
struct ThisUses(Vec<Option<String>>);

impl Visitor for ThisUses {
    fn expr(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        if matches!(expr, Expression::ThisExpression) {
            // Inside a nested arrow or function (R-9) the read is a snapshot hazard.
            let nested = cx.frames.len() > 1;
            self.0.push(match pos {
                Pos::MemberObject { property: Some(g), call: false, write: false } if !nested => {
                    Some(g.clone())
                }
                _ => None,
            });
        }
    }
}

fn this_uses_in(statements: &[Statement]) -> Vec<Option<String>> {
    let mut uses = ThisUses::default();
    walk(&mut statements.to_vec(), &mut uses);
    uses.0
}

fn this_uses_in_expr(expr: &Expression) -> Vec<Option<String>> {
    let statement = Statement::ExpressionStatement(kali_ast::ExpressionStatement {
        expression: Box::new(expr.clone()),
    });
    this_uses_in(&[statement])
}

/// Only `this.g` reads of fields already in `bound`.
fn only_bound_this_reads(uses: &[Option<String>], bound: &[String]) -> bool {
    uses.iter().all(|u| matches!(u, Some(g) if bound.contains(g)))
}

fn body_has_this(body: &ClassBody) -> bool {
    body.methods.iter().any(|m| {
        m.body
            .as_ref()
            .is_some_and(|b| !this_uses_in(&b.body).is_empty())
    })
}

fn is_stateful(body: &ClassBody) -> bool {
    !body.fields.is_empty()
        || body.has_private_members
        || body
            .methods
            .iter()
            .any(|m| m.name == "constructor" || m.kind != MethodKind::Method)
        || body_has_this(body)
}

/// Whether `statement` has a `return <value>` outside nested functions.
fn returns_value(statement: &Statement) -> bool {
    let any = |list: &[Statement]| list.iter().any(returns_value);
    match statement {
        Statement::ReturnStatement(r) => r.argument.is_some(),
        Statement::BlockStatement(b) => any(&b.body),
        Statement::LabeledStatement(s) => returns_value(&s.body),
        Statement::WithStatement(s) => returns_value(&s.body),
        Statement::IfStatement(s) => {
            any(&s.consequent.body) || s.alternate.as_ref().is_some_and(|a| any(&a.body))
        }
        Statement::SwitchStatement(s) => s.cases.iter().any(|c| any(&c.consequent)),
        Statement::TryStatement(s) => {
            any(&s.block.body)
                || s.handler.as_ref().is_some_and(|h| any(&h.body.body))
                || s.finalizer.as_ref().is_some_and(|f| any(&f.body))
        }
        Statement::ForStatement(s) => any(&s.body.body),
        Statement::ForInStatement(s) => returns_value(&s.body),
        Statement::ForOfStatement(s) => returns_value(&s.body),
        Statement::WhileStatement(s) => any(&s.body.body),
        Statement::DoWhileStatement(s) => any(&s.body.body),
        _ => false,
    }
}

/// `this.f = e` as a statement: `(f, e)`.
fn this_field_assignment(statement: &Statement) -> Option<(&str, &Expression)> {
    let Statement::ExpressionStatement(stmt) = statement else { return None };
    let Expression::AssignmentExpression(assign) = stmt.expression.as_ref() else { return None };
    if assign.operator != AssignmentOperator::Assign {
        return None;
    }
    let Expression::MemberExpression(member) = &assign.left else { return None };
    if !matches!(member.object, Expression::ThisExpression) || member.computed_index.is_some() {
        return None;
    }
    Some((member.property.as_deref()?, &assign.right))
}

/// The chain of `name`: it, every program class it extends, and every one
/// that extends it, transitively.
fn chain_of(name: &str, decls: &BTreeMap<String, Vec<Decl>>) -> BTreeSet<String> {
    let mut chain = BTreeSet::from([name.to_string()]);
    loop {
        let mut grown = false;
        for (class, ds) in decls {
            for d in ds {
                let Some(base) = d.super_class.as_deref().filter(|b| decls.contains_key(*b)) else {
                    continue;
                };
                let linked = chain.contains(class) != chain.contains(base);
                if linked {
                    grown |= chain.insert(class.clone());
                    grown |= chain.insert(base.to_string());
                }
            }
        }
        if !grown {
            return chain;
        }
    }
}

fn refusal(message: String) -> Diagnostic {
    Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message)
}

pub(crate) fn plan_classes(statements: &mut Vec<Statement>, spelled: &BTreeSet<String>) -> ClassPlans {
    let program = ProgramClasses::collect(statements);
    let host = program.host_derived();
    let mut facts = Facts::default();
    walk(statements, &mut facts);
    let mut plans = ClassPlans::default();

    for (name, decls) in &facts.decls {
        let chain = chain_of(name, &facts.decls);
        if chain.iter().any(|c| host.contains(c)) {
            continue;
        }
        let any = |f: &dyn Fn(&ClassBody) -> bool| decls.iter().any(|d| f(&d.body));
        let reason = if decls.iter().any(|d| d.is_expression) {
            Some(CLASS_REASON_EXPRESSION)
        } else if chain.len() > 1 {
            Some(CLASS_REASON_EXTENDS)
        } else if any(&|b| b.methods.iter().any(|m| m.kind != MethodKind::Method)) {
            Some(CLASS_REASON_ACCESSOR)
        } else if any(&|b| {
            b.methods.iter().any(|m| m.is_static)
                || b.fields.iter().any(|f| f.is_static)
                || b.has_static_block
        }) {
            Some(CLASS_REASON_STATIC)
        } else if any(&|b| b.has_private_members) {
            Some(CLASS_REASON_PRIVATE)
        } else if any(&|b| b.has_computed_members) {
            Some(CLASS_REASON_COMPUTED)
        } else if facts.exported.contains(name) {
            Some(CLASS_REASON_EXPORTED)
        } else if program.is_ambiguous(name) {
            Some(CLASS_REASON_AMBIGUOUS)
        } else {
            None
        };
        if !facts.constructed.contains(name) {
            continue;
        }
        match reason {
            Some(reason) => {
                let stateful = chain
                    .iter()
                    .filter_map(|c| facts.decls.get(c))
                    .flatten()
                    .any(|d| is_stateful(&d.body));
                if stateful {
                    plans
                        .diagnostics
                        .push(refusal(class_construction_unavailable_message(name, reason)));
                }
            }
            None => {
                let plan = plan_one(name, &decls[0].body, &mut plans.diagnostics);
                plans.rewritten.insert(name.clone(), plan);
            }
        }
    }

    for target in &facts.constructed {
        if facts.functions.contains(target) && !facts.decls.contains_key(target) {
            plans
                .diagnostics
                .push(refusal(plain_function_construction_unavailable_message(target)));
        }
    }

    for class in plans.rewritten.values() {
        let generated = [format!("{}__new", class.name), "__this".to_string()]
            .into_iter()
            .chain(class.methods.iter().map(|m| format!("{}__{m}", class.name)))
            .chain(class.fields.iter().map(|f| format!("__f_{f}")));
        for generated in generated.filter(|g| spelled.contains(g)) {
            plans
                .diagnostics
                .push(refusal(class_generated_name_collision_message(&generated, &class.name)));
        }
    }
    plans
}

/// The plan of one in-slice, constructed class; refusals go to `diagnostics`.
fn plan_one(name: &str, body: &ClassBody, diagnostics: &mut Vec<Diagnostic>) -> RewrittenClass {
    let constructor = body
        .methods
        .iter()
        .find(|m| m.name == "constructor" && !m.is_static);
    let ctor_statements: &[Statement] = constructor
        .and_then(|m| m.body.as_ref())
        .map(|b| b.body.as_slice())
        .unwrap_or(&[]);

    let mut fields: Vec<String> = Vec::new();
    // Fields initialized so far (an initializer, or the run); `fields` keeps first-binding order.
    let mut bound: Vec<String> = Vec::new();
    for field in body.fields.iter().filter(|f| !f.is_static) {
        if let Some(value) = &field.value {
            if !only_bound_this_reads(&this_uses_in_expr(value), &bound) {
                diagnostics.push(refusal(class_field_initializer_this_message(name, &field.name)));
            }
        }
        if !fields.contains(&field.name) {
            fields.push(field.name.clone());
        }
        if field.value.is_some() && !bound.contains(&field.name) {
            bound.push(field.name.clone());
        }
    }

    let mut run_bound: Vec<String> = Vec::new();
    let mut leading_run = 0;
    for statement in ctor_statements {
        let Some((field, right)) = this_field_assignment(statement) else { break };
        if !only_bound_this_reads(&this_uses_in_expr(right), &bound) {
            break;
        }
        if !fields.iter().any(|f| f == field) {
            fields.push(field.to_string());
        }
        if !bound.iter().any(|f| f == field) {
            bound.push(field.to_string());
        }
        run_bound.push(field.to_string());
        leading_run += 1;
    }

    for field in body.fields.iter().filter(|f| !f.is_static && f.value.is_none()) {
        if !run_bound.contains(&field.name) {
            diagnostics.push(refusal(class_field_without_initial_value_message(name, &field.name)));
        }
    }

    if ctor_statements.iter().any(returns_value) {
        diagnostics.push(refusal(constructor_return_unavailable_message().to_string()));
    }

    RewrittenClass {
        name: name.to_string(),
        fields,
        methods: body
            .methods
            .iter()
            .filter(|m| !m.is_static && m.name != "constructor")
            .map(|m| m.name.clone())
            .collect(),
        leading_run,
        ctor_params: constructor.map(|m| m.params.clone()).unwrap_or_default(),
    }
}
