//! Instance uses (spec §3.3): refuses every instance outside the allowlist and
//! every member call whose receiver kali cannot resolve, then rewrites
//! `new C(a)` to `C__new(a)`, `o.m(a)` to `C__m(o, a)`, and compound or
//! update writes of a field to plain `=` writes. When in doubt, it refuses.

use std::collections::BTreeMap;

use kali_ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, CallExpression, Expression,
    JsxAttributeItem, JsxAttributeValue, JsxChild, JsxElement, JsxName, LiteralValue, Statement,
    UpdateOperator,
};
use kali_common::{
    class_field_outside_set_message, class_field_undeclared_read_message,
    class_instance_mixed_message, class_instance_position_message, class_method_value_message,
    class_receiver_unresolved_message, class_value_message,
    unresolved_member_call_unavailable_message,
};
use kali_error::{_error_codes::e5, diagnostic::Diagnostic};

use super::classes::RewrittenClass;
use super::provenance::{abstract_expr, is_wrapper, strip, Env, Provenance, Val};
use super::scopes::{BindingId, Resolved};
use super::walk::{walk, Cx, Pos, Visitor};

/// Checks every expression against §3.3's allowlist and rewrites the uses it
/// proves. Diagnostics come in walk order.
pub(crate) fn check_and_rewrite(
    statements: &mut Vec<Statement>,
    env: &Env,
    prov: &Provenance,
) -> Vec<Diagnostic> {
    let generated = env
        .plans
        .rewritten
        .values()
        .flat_map(|class| {
            class.methods.iter().map(|m| {
                (
                    format!("{}__{m}", class.name),
                    (class.name.clone(), m.clone()),
                )
            })
        })
        .collect();
    let mut uses = Uses {
        env,
        prov,
        generated,
        diagnostics: Vec::new(),
    };
    walk(statements, &mut uses);
    uses.diagnostics
}

struct Uses<'e, 'a> {
    env: &'e Env<'a>,
    prov: &'e Provenance,
    /// Generated method name `C__m` → `(C, m)`.
    generated: BTreeMap<String, (String, String)>,
    diagnostics: Vec<Diagnostic>,
}

impl Visitor for Uses<'_, '_> {
    fn expr(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        let before = self.diagnostics.len();
        self.class_as_value(expr, pos, cx);
        // A wrapper's inner expression is visited at the same position.
        if !is_wrapper(expr) {
            let v = self.eval(expr, cx);
            if let Val::Inst(class) = &v {
                self.position(class, pos, cx);
            }
            self.receiver(&v, pos);
        }
        if self.diagnostics.len() == before {
            self.rewrite(expr, pos, cx);
        }
    }
}

/// The twelve arithmetic and bitwise compound operators, as binary operators.
fn compound_operator(operator: &AssignmentOperator) -> Option<&'static str> {
    Some(match operator {
        AssignmentOperator::AddAssign => "+",
        AssignmentOperator::SubtractAssign => "-",
        AssignmentOperator::MultiplyAssign => "*",
        AssignmentOperator::DivideAssign => "/",
        AssignmentOperator::ModuloAssign => "%",
        AssignmentOperator::ExponentAssign => "**",
        AssignmentOperator::LeftShiftAssign => "<<",
        AssignmentOperator::RightShiftAssign => ">>",
        AssignmentOperator::UnsignedRightShiftAssign => ">>>",
        AssignmentOperator::BitAndAssign => "&",
        AssignmentOperator::BitOrAssign => "|",
        AssignmentOperator::BitXorAssign => "^",
        AssignmentOperator::Assign
        | AssignmentOperator::NullishAssign
        | AssignmentOperator::AndAssign
        | AssignmentOperator::OrAssign => return None,
    })
}

/// `left = left <op> right`.
fn field_write(left: &Expression, operator: &str, right: Expression) -> Expression {
    Expression::AssignmentExpression(Box::new(AssignmentExpression {
        operator: AssignmentOperator::Assign,
        left: left.clone(),
        right: Expression::BinaryExpression(Box::new(BinaryExpression {
            operator: operator.to_string(),
            left: left.clone(),
            right,
        })),
    }))
}

fn call(callee: String, args: Vec<Expression>) -> Expression {
    Expression::CallExpression(Box::new(CallExpression {
        callee: Expression::Identifier(callee),
        args,
    }))
}

/// Every element name of a JSX tree, except inside `{…}` containers (the
/// walker hands those to the visitor as expressions of their own).
fn jsx_names<'j>(element: &'j JsxElement, out: &mut Vec<&'j str>) {
    if let JsxName::Identifier(name) = &element.opening_element.name {
        out.push(name);
    }
    for attribute in &element.opening_element.attributes {
        if let JsxAttributeItem::JsxAttribute(attribute) = attribute {
            if let JsxAttributeValue::JsxElement(element) = &attribute.value {
                jsx_names(element, out);
            }
        }
    }
    jsx_children(&element.children, out);
}

fn jsx_children<'j>(children: &'j [JsxChild], out: &mut Vec<&'j str>) {
    for child in children {
        match child {
            JsxChild::JsxElement(element) => jsx_names(element, out),
            JsxChild::JsxFragment(fragment) => jsx_children(&fragment.children, out),
            JsxChild::JsxText(_) | JsxChild::JsxExpression(_) => {}
        }
    }
}

impl Uses<'_, '_> {
    fn refuse(&mut self, message: String) {
        self.diagnostics
            .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
    }

    fn eval(&self, expr: &Expression, cx: &Cx) -> Val {
        self.prov.eval(&abstract_expr(expr, cx, self.env), self.env)
    }

    fn class(&self, name: &str) -> Option<&RewrittenClass> {
        self.env.plans.rewritten.get(name)
    }

    /// Step 1: a rewritten class's name anywhere but `new C(…)`'s callee (R-12).
    fn class_as_value(&mut self, expr: &Expression, pos: &Pos, cx: &Cx) {
        let mut names: Vec<&str> = Vec::new();
        match expr {
            Expression::Identifier(name) if *pos != Pos::NewCallee => names.push(name),
            Expression::JsxElement(element) => jsx_names(element, &mut names),
            Expression::JsxFragment(fragment) => jsx_children(&fragment.children, &mut names),
            _ => {}
        }
        for name in names {
            if self.env.may_be_class(name, cx) {
                self.refuse(class_value_message(name));
            }
        }
    }

    /// Step 2: an instance of `class` at `pos`.
    fn position(&mut self, class: &str, pos: &Pos, cx: &Cx) {
        let Some(plan) = self.class(class) else {
            return;
        };
        let is_field = |f: &str| plan.fields.iter().any(|g| g == f);
        let is_method = |m: &str| plan.methods.contains(m);
        let message = match pos {
            Pos::BindingInit(name) | Pos::BindingAssign(name) => {
                let holds = match self.env.scopes.resolve(name, cx) {
                    Resolved::Binding(id) => self.prov.binding(&id) == Val::Inst(class.into()),
                    Resolved::Ambiguous | Resolved::Free => false,
                };
                (!holds).then(|| class_instance_mixed_message(class, &format!("binding `{name}`")))
            }
            Pos::CallArg { callee, index } => return self.call_arg(class, callee, *index, cx),
            Pos::Return => (self.prov.returns(cx.key()) != Val::Inst(class.into())).then(|| {
                class_instance_mixed_message(class, &format!("the return value of `{}`", cx.key()))
            }),
            Pos::MemberObject { property: None, .. } => Some(class_instance_position_message(
                class,
                "the object of a computed member access",
            )),
            Pos::MemberObject {
                property: Some(m),
                call: true,
                ..
            } => (!is_method(m) && !is_field(m))
                .then(|| unresolved_member_call_unavailable_message(m)),
            Pos::MemberObject {
                property: Some(f),
                write: true,
                ..
            } => (!is_field(f)).then(|| class_field_outside_set_message(class, f)),
            Pos::MemberObject {
                property: Some(f), ..
            } => {
                if is_field(f) {
                    None
                } else if is_method(f) {
                    Some(class_method_value_message(class, f))
                } else {
                    Some(class_field_undeclared_read_message(class, f))
                }
            }
            Pos::Discarded
            | Pos::Callee
            | Pos::NewCallee
            | Pos::AssignTarget
            | Pos::UpdateTarget => None,
            Pos::Other(position) => Some(class_instance_position_message(class, position)),
        };
        if let Some(message) = message {
            self.refuse(message);
        }
    }

    /// Step 2 at `CallArg`: the argument must bind a parameter that holds only `class`.
    fn call_arg(&mut self, class: &str, callee: &Expression, index: usize, cx: &Cx) {
        match strip(callee) {
            Expression::Identifier(name) => {
                if let Some((owner, method)) = self.generated.get(name).cloned() {
                    if index == 0 {
                        if owner != class {
                            let place = format!("the receiver of `{owner}.{method}`");
                            self.refuse(class_instance_mixed_message(class, &place));
                        }
                    } else {
                        self.method_param(class, &owner, &method, index - 1);
                    }
                    return;
                }
                if let Some(owner) = name
                    .strip_suffix("__new")
                    .filter(|c| self.class(c).is_some())
                {
                    return self.method_param(class, owner, "constructor", index);
                }
                if let Some(f) = self.env.function(name, cx) {
                    let (key, params) = (f.key.clone(), f.params.clone());
                    return self.param(class, &key, &params, index, name);
                }
            }
            Expression::MemberExpression(member) => {
                if let Some(method) = &member.property {
                    if let Val::Inst(owner) = self.eval(&member.object, cx) {
                        if self
                            .class(&owner)
                            .is_some_and(|c| c.methods.contains(method))
                        {
                            return self.method_param(class, &owner, method, index);
                        }
                    }
                }
            }
            _ => {}
        }
        self.refuse(class_instance_position_message(
            class,
            "an argument to a call kali cannot resolve to a program function",
        ));
    }

    fn method_param(&mut self, class: &str, owner: &str, method: &str, index: usize) {
        let key = self.env.method_key(owner, method).cloned();
        let params = key
            .as_ref()
            .map(|k| self.env.scopes.params(k).to_vec())
            .unwrap_or_default();
        let key = key.unwrap_or_default();
        self.param(class, &key, &params, index, &format!("{owner}.{method}"));
    }

    fn param(&mut self, class: &str, key: &str, params: &[String], index: usize, owner: &str) {
        let Some(param) = params.get(index) else {
            return self.refuse(class_instance_position_message(class, "an extra argument"));
        };
        let id = BindingId {
            frame: key.to_string(),
            name: param.clone(),
        };
        if self.prov.binding(&id) != Val::Inst(class.into()) {
            let place = format!("parameter `{param}` of `{owner}`");
            self.refuse(class_instance_mixed_message(class, &place));
        }
    }

    /// Step 3: a member call on a receiver that may be an instance of an unknown class.
    fn receiver(&mut self, v: &Val, pos: &Pos) {
        let Pos::MemberObject {
            property: Some(method),
            call: true,
            ..
        } = pos
        else {
            return;
        };
        if matches!(v, Val::Bottom | Val::Unknown) {
            if let Some(owner) = self.env.plans.method_owner(method) {
                let message = class_receiver_unresolved_message(method, owner);
                self.refuse(message);
            }
        }
    }

    /// `o.f` with a static name whose `o` is an identifier or `this` holding an instance.
    fn instance_field(&self, target: &Expression, cx: &Cx) -> bool {
        let Expression::MemberExpression(member) = target else {
            return false;
        };
        member.property.is_some()
            && matches!(
                member.object,
                Expression::Identifier(_) | Expression::ThisExpression
            )
            && matches!(self.eval(&member.object, cx), Val::Inst(_))
    }

    /// Step 4.
    fn rewrite(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        let replacement = match expr {
            Expression::NewExpression(new) => match self.env.constructed_class(new, cx) {
                Some(class) => call(format!("{class}__new"), std::mem::take(&mut new.args)),
                None => return,
            },
            Expression::CallExpression(c) => {
                let Expression::MemberExpression(member) = strip(&c.callee) else {
                    return;
                };
                let Some(method) = &member.property else {
                    return;
                };
                let Val::Inst(class) = self.eval(&member.object, cx) else {
                    return;
                };
                if !self
                    .class(&class)
                    .is_some_and(|plan| plan.methods.contains(method))
                {
                    return;
                }
                let args = std::iter::once(member.object.clone())
                    .chain(std::mem::take(&mut c.args))
                    .collect();
                call(format!("{class}__{method}"), args)
            }
            Expression::AssignmentExpression(assign) => {
                let Some(operator) = compound_operator(&assign.operator) else {
                    return;
                };
                if !self.instance_field(&assign.left, cx) {
                    return;
                }
                let right = std::mem::replace(&mut assign.right, Expression::ThisExpression);
                field_write(&assign.left, operator, right)
            }
            Expression::UpdateExpression(update) => {
                if *pos != Pos::Discarded || !self.instance_field(&update.argument, cx) {
                    return;
                }
                let operator = match update.operator {
                    UpdateOperator::Increment => "+",
                    UpdateOperator::Decrement => "-",
                };
                field_write(
                    &update.argument,
                    operator,
                    Expression::Literal(LiteralValue::Number(1.0)),
                )
            }
            _ => return,
        };
        *expr = replacement;
    }
}
