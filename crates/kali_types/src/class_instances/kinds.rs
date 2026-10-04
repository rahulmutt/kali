//! Field value kinds (ruling R-23): every value a rewritten class's field can
//! hold must be proven a number or a boolean. The object-literal lane stores a
//! field in one 8-byte slot and reads it back as a number, so a string, `null`,
//! `undefined`, a BigInt, an array or an object in a field would print or
//! compare as a silent wrong value.
//!
//! A whole-program fixpoint over the same bindings, parameters and returns as
//! [`Provenance`], plus each field's writes. Anything not proven is `Other`.

use std::collections::BTreeMap;

use kali_ast::{
    AssignmentOperator, ClassDeclaration, Expression, ExpressionOrSpread, FunctionDeclaration,
    LiteralValue, Statement, VariableDeclarator,
};
use kali_common::{class_construction_unavailable_message, CLASS_REASON_FIELD_VALUE};
use kali_error::{_error_codes::e5, diagnostic::Diagnostic};

use super::provenance::{
    abstract_expr, bare_return, falls_through, is_wrapper, strip, Env, Provenance, Val,
};
use super::scopes::{BindingId, Resolved};
use super::walk::{walk, Cx, FnKey, Frame, FrameKind, Pos, Visitor, ANON};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Bottom,
    /// A number or a boolean.
    NumBool,
    Other,
}

impl Kind {
    fn join(self, other: Kind) -> Kind {
        match (self, other) {
            (Kind::Bottom, x) | (x, Kind::Bottom) => x,
            (Kind::NumBool, Kind::NumBool) => Kind::NumBool,
            _ => Kind::Other,
        }
    }
}

#[derive(Clone, Debug)]
enum KSrc {
    NumBool,
    Other,
    Binding(BindingId),
    /// The return value of a frame.
    Call(FnKey),
    Field(String, String),
    Join(Vec<KSrc>),
    /// An arithmetic result: a number when every operand is a number or a
    /// boolean (a BigInt operand gives a BigInt, a string one a string).
    Arith(Vec<KSrc>),
}

const ARITHMETIC: &[&str] = &[
    "+", "-", "*", "/", "%", "**", "&", "|", "^", "<<", ">>", ">>>",
];
const COMPARISON: &[&str] = &[
    "==",
    "!=",
    "===",
    "!==",
    "<",
    ">",
    "<=",
    ">=",
    "instanceof",
    "in",
];
/// Free globals whose call always returns a number or a boolean.
const NUMERIC_GLOBALS: &[&str] = &[
    "Number",
    "Boolean",
    "parseInt",
    "parseFloat",
    "isNaN",
    "isFinite",
];

fn compound_arithmetic(operator: &AssignmentOperator) -> bool {
    !matches!(
        operator,
        AssignmentOperator::Assign
            | AssignmentOperator::NullishAssign
            | AssignmentOperator::AndAssign
            | AssignmentOperator::OrAssign
    )
}

struct Abstract<'e, 'a> {
    env: &'e Env<'a>,
    prov: &'e Provenance,
}

impl Abstract<'_, '_> {
    fn free(&self, name: &str, cx: &Cx) -> bool {
        self.env.scopes.resolve(name, cx) == Resolved::Free
    }

    /// The class of an instance-valued expression.
    fn instance(&self, e: &Expression, cx: &Cx) -> Option<String> {
        match self.prov.eval(&abstract_expr(e, cx, self.env), self.env) {
            Val::Inst(class) => Some(class),
            _ => None,
        }
    }

    fn kind(&self, e: &Expression, cx: &Cx) -> KSrc {
        match e {
            Expression::ParenthesizedExpression(p) => self.kind(&p.expression, cx),
            Expression::TypeAssertion(t) => self.kind(&t.expression, cx),
            Expression::SatisfiesExpression(s) => self.kind(&s.expression, cx),
            Expression::ChainExpression(c) => self.kind(&c.expression, cx),
            Expression::Literal(LiteralValue::Number(_) | LiteralValue::Boolean(_)) => {
                KSrc::NumBool
            }
            Expression::Identifier(name) => match self.env.scopes.resolve(name, cx) {
                Resolved::Binding(id) if self.env.functions.contains_key(&id) => KSrc::Other,
                Resolved::Binding(id) => KSrc::Binding(id),
                Resolved::Ambiguous => KSrc::Other,
                Resolved::Free if matches!(name.as_str(), "NaN" | "Infinity") => KSrc::NumBool,
                Resolved::Free => KSrc::Other,
            },
            Expression::UnaryExpression(u) => match u.operator.as_str() {
                "!" | "delete" => KSrc::NumBool,
                "-" | "+" | "~" => KSrc::Arith(vec![self.kind(&u.argument, cx)]),
                _ => KSrc::Other,
            },
            Expression::BinaryExpression(b) if COMPARISON.contains(&b.operator.as_str()) => {
                KSrc::NumBool
            }
            Expression::BinaryExpression(b) if ARITHMETIC.contains(&b.operator.as_str()) => {
                KSrc::Arith(vec![self.kind(&b.left, cx), self.kind(&b.right, cx)])
            }
            Expression::LogicalExpression(l) => {
                KSrc::Join(vec![self.kind(&l.left, cx), self.kind(&l.right, cx)])
            }
            Expression::ConditionalExpression(c) => KSrc::Join(vec![
                self.kind(&c.consequent, cx),
                self.kind(&c.alternate, cx),
            ]),
            Expression::SequenceExpression(s) => match s.expressions.last() {
                Some(last) => self.kind(last, cx),
                None => KSrc::Other,
            },
            Expression::AssignmentExpression(a) => match a.operator {
                AssignmentOperator::Assign => self.kind(&a.right, cx),
                AssignmentOperator::NullishAssign
                | AssignmentOperator::AndAssign
                | AssignmentOperator::OrAssign => {
                    KSrc::Join(vec![self.kind(&a.left, cx), self.kind(&a.right, cx)])
                }
                _ => KSrc::Arith(vec![self.kind(&a.left, cx), self.kind(&a.right, cx)]),
            },
            // `x++` keeps a number a number and a BigInt a BigInt.
            Expression::UpdateExpression(u) => KSrc::Arith(vec![self.kind(&u.argument, cx)]),
            Expression::MemberExpression(m) => match (&m.property, &m.computed_index) {
                (Some(field), None) => match self.instance(&m.object, cx) {
                    Some(class) if self.is_field(&class, field) => {
                        KSrc::Field(class, field.clone())
                    }
                    _ => KSrc::Other,
                },
                _ => KSrc::Other,
            },
            Expression::CallExpression(call) => match strip(&call.callee) {
                Expression::Identifier(name) => match self.env.function(name, cx) {
                    Some(f) if f.is_async_or_generator => KSrc::Other,
                    Some(f) => KSrc::Call(f.key.clone()),
                    None if NUMERIC_GLOBALS.contains(&name.as_str()) && self.free(name, cx) => {
                        KSrc::NumBool
                    }
                    None => KSrc::Other,
                },
                Expression::MemberExpression(member) => {
                    let Some(method) = &member.property else {
                        return KSrc::Other;
                    };
                    if let Expression::Identifier(root) = &member.object {
                        if root == "Math" && self.free(root, cx) {
                            return KSrc::NumBool;
                        }
                    }
                    match self.instance(&member.object, cx) {
                        Some(class) => match self.env.method_key(&class, method) {
                            Some(key) => KSrc::Call(key.clone()),
                            None => KSrc::Other,
                        },
                        None => KSrc::Other,
                    }
                }
                _ => KSrc::Other,
            },
            _ => KSrc::Other,
        }
    }

    fn is_field(&self, class: &str, field: &str) -> bool {
        self.env
            .plans
            .rewritten
            .get(class)
            .is_some_and(|c| c.fields.iter().any(|f| f == field))
    }
}

struct Facts<'e, 'a> {
    abs: Abstract<'e, 'a>,
    bindings: BTreeMap<BindingId, Vec<KSrc>>,
    returns: BTreeMap<FnKey, Vec<KSrc>>,
    fields: BTreeMap<(String, String), Vec<KSrc>>,
}

impl Facts<'_, '_> {
    fn add_binding(&mut self, name: &str, cx: &Cx, src: KSrc) {
        if let Resolved::Binding(id) = self.abs.env.scopes.resolve(name, cx) {
            self.bindings.entry(id).or_default().push(src);
        }
    }

    fn add_params(&mut self, key: &str, args: &[Expression], cx: &Cx) {
        let spread = args
            .iter()
            .any(|a| matches!(strip(a), Expression::SpreadElement(_)));
        for (index, param) in self.abs.env.scopes.params(key).to_vec().iter().enumerate() {
            let src = match args.get(index) {
                Some(arg) if !spread => self.abs.kind(arg, cx),
                _ => KSrc::Other,
            };
            self.bindings
                .entry(BindingId {
                    frame: key.to_string(),
                    name: param.clone(),
                })
                .or_default()
                .push(src);
        }
    }

    fn add_field(&mut self, class: String, field: &str, src: KSrc) {
        if self.abs.is_field(&class, field) {
            self.fields
                .entry((class, field.to_string()))
                .or_default()
                .push(src);
        }
    }

    /// A write to `target`: an instance field `o.f`, or a binding.
    fn write(&mut self, target: &Expression, src: KSrc, cx: &Cx) {
        match strip(target) {
            Expression::Identifier(name) => self.add_binding(name, cx, src),
            Expression::MemberExpression(m) => {
                if let (Some(field), None) = (&m.property, &m.computed_index) {
                    if let Some(class) = self.abs.instance(&m.object, cx) {
                        self.add_field(class, field, src);
                    }
                }
            }
            // A destructuring pattern: what each target receives is not tracked.
            Expression::ArrayExpression(array) => {
                for element in array.elements.iter().flatten() {
                    match element {
                        ExpressionOrSpread::Expression(e) => self.write(e, KSrc::Other, cx),
                        ExpressionOrSpread::Spread(s) => self.write(&s.argument, KSrc::Other, cx),
                        ExpressionOrSpread::Empty => {}
                    }
                }
            }
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    self.write(&property.value, KSrc::Other, cx);
                }
            }
            Expression::AssignmentExpression(default) => self.write(&default.left, KSrc::Other, cx),
            Expression::SpreadElement(s) => self.write(&s.argument, KSrc::Other, cx),
            Expression::RestElement(r) => self.write(&r.argument, KSrc::Other, cx),
            _ => {}
        }
    }

    fn function(&mut self, key: FnKey, body: Option<&[Statement]>, is_async_or_generator: bool) {
        if is_async_or_generator || body.is_none_or(falls_through) {
            self.returns.entry(key).or_default().push(KSrc::Other);
        }
    }
}

impl Visitor for Facts<'_, '_> {
    fn expr(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        if matches!(pos, Pos::Return) && !is_wrapper(expr) {
            let src = self.abs.kind(expr, cx);
            self.returns
                .entry(cx.key().to_string())
                .or_default()
                .push(src);
        }
        if matches!(pos, Pos::Other("a loop target")) {
            self.write(expr, KSrc::Other, cx);
        }
        match expr {
            Expression::AssignmentExpression(assign) => {
                let src = match assign.operator {
                    AssignmentOperator::Assign => self.abs.kind(&assign.right, cx),
                    ref op if compound_arithmetic(op) => KSrc::Arith(vec![
                        self.abs.kind(&assign.left, cx),
                        self.abs.kind(&assign.right, cx),
                    ]),
                    _ => KSrc::Join(vec![
                        self.abs.kind(&assign.left, cx),
                        self.abs.kind(&assign.right, cx),
                    ]),
                };
                self.write(&assign.left, src, cx);
            }
            Expression::UpdateExpression(update) => {
                let src = KSrc::Arith(vec![self.abs.kind(&update.argument, cx)]);
                self.write(&update.argument, src, cx);
            }
            Expression::CallExpression(call) => match strip(&call.callee) {
                Expression::Identifier(name) => {
                    if let Some(f) = self.abs.env.function(name, cx) {
                        let key = f.key.clone();
                        self.add_params(&key, &call.args, cx);
                    }
                }
                Expression::MemberExpression(member) => {
                    let Some(method) = &member.property else {
                        return;
                    };
                    match self.abs.prov.eval(
                        &abstract_expr(&member.object, cx, self.abs.env),
                        self.abs.env,
                    ) {
                        Val::Inst(class) => {
                            if let Some(key) = self.abs.env.method_key(&class, method).cloned() {
                                self.add_params(&key, &call.args, cx);
                            }
                        }
                        // An unresolved receiver may be any class with `method`.
                        Val::Unknown | Val::Bottom => {
                            let keys: Vec<FnKey> = self
                                .abs
                                .env
                                .plans
                                .rewritten
                                .values()
                                .filter(|c| c.methods.contains(method))
                                .filter_map(|c| self.abs.env.method_key(&c.name, method).cloned())
                                .collect();
                            for key in keys {
                                self.add_params(&key, &[], cx);
                            }
                        }
                        Val::NotInst => {}
                    }
                }
                _ => {}
            },
            Expression::NewExpression(new) => {
                if let Some(class) = self.abs.env.constructed_class(new, cx) {
                    if let Some(key) = self.abs.env.method_key(class, "constructor").cloned() {
                        self.add_params(&key, &new.args, cx);
                    }
                }
            }
            Expression::FunctionExpression(f) => {
                let key = cx.child_key(f.id.as_deref().unwrap_or(ANON));
                let body = f.body.as_ref().map(|b| b.body.as_slice());
                self.function(key, body, f.is_async || f.generator);
            }
            Expression::ArrowFunctionExpression(a) if a.is_async => {
                let key = cx.child_key(a.id.as_deref().unwrap_or(ANON));
                self.returns.entry(key).or_default().push(KSrc::Other);
            }
            _ => {}
        }
    }
    fn stmts(&mut self, list: &mut Vec<Statement>, cx: &Cx) {
        if list.iter().any(bare_return) {
            self.returns
                .entry(cx.key().to_string())
                .or_default()
                .push(KSrc::Other);
        }
    }
    fn class_decl(&mut self, class: &ClassDeclaration, _exported_default: bool, cx: &Cx) {
        self.add_binding(&class.name, cx, KSrc::Other);
        for method in &class.body.methods {
            let key = cx.child_key(&format!("{}#{}", class.name, method.name));
            let body = method.body.as_ref().map(|b| b.body.as_slice());
            self.function(key, body, method.is_async || method.generator);
        }
        if !self.abs.env.plans.rewritten.contains_key(&class.name) {
            return;
        }
        // Field initializers run with `this` bound to the instance.
        let mut init = cx.clone();
        init.frames.push(Frame {
            key: cx.child_key(&format!("{}#fields", class.name)),
            kind: FrameKind::FieldInit {
                class: class.name.clone(),
            },
        });
        for field in class.body.fields.iter().filter(|f| !f.is_static) {
            if let Some(value) = &field.value {
                let src = self.abs.kind(value, &init);
                self.add_field(class.name.clone(), &field.name, src);
            }
        }
    }
    fn var_declarator(&mut self, d: &VariableDeclarator, _kind: &str, cx: &Cx) {
        let src = match &d.init {
            Some(init) => self.abs.kind(init, cx),
            None => KSrc::Other,
        };
        self.add_binding(&d.id, cx, src);
    }
    fn function_decl(&mut self, f: &FunctionDeclaration, cx: &Cx) {
        let name = if f.name.is_empty() {
            ANON
        } else {
            f.name.as_str()
        };
        if !f.name.is_empty() {
            self.add_binding(name, cx, KSrc::Other);
        }
        self.function(
            cx.child_key(name),
            Some(&f.body.body),
            f.is_async || f.generator,
        );
    }
    fn catch_param(&mut self, name: &str, cx: &Cx) {
        self.add_binding(name, cx, KSrc::Other);
    }
}

struct Solved {
    bindings: BTreeMap<BindingId, Kind>,
    returns: BTreeMap<FnKey, Kind>,
    fields: BTreeMap<(String, String), Kind>,
}

impl Solved {
    fn eval(&self, src: &KSrc) -> Kind {
        match src {
            KSrc::NumBool => Kind::NumBool,
            KSrc::Other => Kind::Other,
            KSrc::Binding(id) => self.bindings.get(id).copied().unwrap_or(Kind::Bottom),
            KSrc::Call(key) => self.returns.get(key).copied().unwrap_or(Kind::Bottom),
            KSrc::Field(class, field) => self
                .fields
                .get(&(class.clone(), field.clone()))
                .copied()
                .unwrap_or(Kind::Bottom),
            KSrc::Join(sources) => sources
                .iter()
                .fold(Kind::Bottom, |acc, s| acc.join(self.eval(s))),
            KSrc::Arith(operands) => {
                let kinds: Vec<Kind> = operands.iter().map(|s| self.eval(s)).collect();
                if kinds.contains(&Kind::Other) {
                    Kind::Other
                } else if kinds.contains(&Kind::Bottom) {
                    Kind::Bottom
                } else {
                    Kind::NumBool
                }
            }
        }
    }
}

fn fold<K: Ord + Clone>(facts: &BTreeMap<K, Vec<KSrc>>, solved: &Solved) -> BTreeMap<K, Kind> {
    facts
        .iter()
        .map(|(k, sources)| {
            (
                k.clone(),
                sources
                    .iter()
                    .fold(Kind::Bottom, |acc, s| acc.join(solved.eval(s))),
            )
        })
        .collect()
}

/// The kind of every rewritten class's every field.
pub(crate) fn field_kinds(
    statements: &mut Vec<Statement>,
    env: &Env,
    prov: &Provenance,
) -> BTreeMap<(String, String), Kind> {
    let mut facts = Facts {
        abs: Abstract { env, prov },
        bindings: BTreeMap::new(),
        returns: BTreeMap::new(),
        fields: BTreeMap::new(),
    };
    walk(statements, &mut facts);
    // A parameter of a frame with untracked call sites may receive anything.
    for id in &prov.unknown_params {
        facts
            .bindings
            .entry(id.clone())
            .or_default()
            .push(KSrc::Other);
    }
    let mut solved = Solved {
        bindings: BTreeMap::new(),
        returns: BTreeMap::new(),
        fields: BTreeMap::new(),
    };
    loop {
        let next = Solved {
            bindings: fold(&facts.bindings, &solved),
            returns: fold(&facts.returns, &solved),
            fields: fold(&facts.fields, &solved),
        };
        if next.bindings == solved.bindings
            && next.returns == solved.returns
            && next.fields == solved.fields
        {
            break;
        }
        solved = next;
    }
    let mut kinds = BTreeMap::new();
    for class in env.plans.rewritten.values() {
        for field in &class.fields {
            let key = (class.name.clone(), field.clone());
            let kind = solved.fields.get(&key).copied().unwrap_or(Kind::Bottom);
            kinds.insert(key, kind);
        }
    }
    kinds
}

/// R-23: refuses each rewritten class with a field not proven to hold only
/// numbers and booleans (an unproven `Bottom` refuses too).
pub(crate) fn check_field_kinds(
    statements: &mut Vec<Statement>,
    env: &Env,
    prov: &Provenance,
) -> Vec<Diagnostic> {
    let kinds = field_kinds(statements, env, prov);
    let mut refused: Vec<&str> = Vec::new();
    for ((class, _), kind) in &kinds {
        if *kind != Kind::NumBool && !refused.contains(&class.as_str()) {
            refused.push(class);
        }
    }
    refused
        .into_iter()
        .map(|class| {
            Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                class_construction_unavailable_message(class, CLASS_REASON_FIELD_VALUE),
            )
        })
        .collect()
}
