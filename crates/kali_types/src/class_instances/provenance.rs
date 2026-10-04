//! Instance provenance (spec §3.3, A-9): which expressions are instances of
//! which rewritten class, by a whole-program fixpoint over bindings,
//! parameters and returns. Anything not proven is `Unknown`; a wrong
//! `Inst(C)` would be a silent miscompile downstream.
//!
//! `Bottom` can survive the fixpoint for code nothing reaches (e.g. the
//! parameters of a function never called); consumers must treat `Bottom`
//! like `Unknown` when deciding refusals.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{
    AssignmentOperator, ClassBody, ClassDeclaration, ClassExpression, ExportDefaultDeclaration,
    Expression, ExpressionOrSpread, FunctionDeclaration, NewExpression, Statement,
    VariableDeclarator,
};

use super::classes::{new_target, ClassPlans};
use super::scopes::{BindingId, Resolved, Scopes};
use super::walk::{walk, Cx, FnKey, Pos, Visitor, ANON};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Val {
    Bottom,
    Inst(String),
    NotInst,
    Unknown,
}

impl Val {
    pub(crate) fn join(&self, other: &Val) -> Val {
        match (self, other) {
            (Val::Bottom, x) | (x, Val::Bottom) => x.clone(),
            (Val::Inst(a), Val::Inst(b)) if a == b => Val::Inst(a.clone()),
            (Val::NotInst, Val::NotInst) => Val::NotInst,
            _ => Val::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Src {
    Inst(String),
    NotInst,
    Unknown,
    Binding(BindingId),
    Call(FnKey),
    MethodCall(Box<Src>, String),
    Join(Vec<Src>),
}

#[derive(Clone, Debug)]
pub(crate) struct FnInfo {
    pub key: FnKey,
    pub params: Vec<String>,
    pub is_async_or_generator: bool,
}

pub(crate) struct Env<'a> {
    pub scopes: &'a Scopes,
    pub plans: &'a ClassPlans,
    /// Binding of a callable name → the function's frame key, params, and async/generator flag.
    pub functions: BTreeMap<BindingId, FnInfo>,
    /// `(class, method)` → the method's frame key; `None` when declared more than once.
    methods: BTreeMap<(String, String), Option<FnKey>>,
    /// Rewritten class → the binding its declaration introduces; `None` when not unique.
    classes: BTreeMap<String, Option<BindingId>>,
}

impl Env<'_> {
    pub(crate) fn method_key(&self, class: &str, method: &str) -> Option<&FnKey> {
        self.methods
            .get(&(class.to_string(), method.to_string()))
            .and_then(Option::as_ref)
    }

    /// The function binding `name` resolves to at `cx`, if it is one.
    pub(crate) fn function(&self, name: &str, cx: &Cx) -> Option<&FnInfo> {
        match self.scopes.resolve(name, cx) {
            Resolved::Binding(id) => self.functions.get(&id),
            Resolved::Ambiguous | Resolved::Free => None,
        }
    }

    /// Whether `name` at `cx` is the binding a rewritten class's declaration introduces.
    pub(crate) fn is_class_declaration(&self, name: &str, cx: &Cx) -> bool {
        let Some(Some(declared)) = self.classes.get(name) else {
            return false;
        };
        matches!(self.scopes.resolve(name, cx), Resolved::Binding(id) if &id == declared)
    }

    /// Whether `name` at `cx` may be a rewritten class itself: only a binding
    /// proven to be some other declaration is not (R-12 relies on this).
    pub(crate) fn may_be_class(&self, name: &str, cx: &Cx) -> bool {
        match self.classes.get(name) {
            None => false,
            Some(None) => true,
            Some(Some(declared)) => !matches!(
                self.scopes.resolve(name, cx),
                Resolved::Binding(id) if &id != declared
            ),
        }
    }

    /// `Some(C)` for a canonical `new C(..)` whose `C` is the rewritten
    /// class's own declaration at `cx`.
    pub(crate) fn constructed_class<'n>(&self, new: &'n NewExpression, cx: &Cx) -> Option<&'n str> {
        let Expression::Identifier(class) = &new.callee else {
            return None;
        };
        self.is_class_declaration(class, cx)
            .then_some(class.as_str())
    }
}

/// Strips the wrappers that do not change a value.
pub(crate) fn strip(mut e: &Expression) -> &Expression {
    loop {
        e = match e {
            Expression::ParenthesizedExpression(p) => &p.expression,
            Expression::TypeAssertion(t) => &t.expression,
            Expression::SatisfiesExpression(s) => &s.expression,
            Expression::ChainExpression(c) => &c.expression,
            _ => return e,
        };
    }
}

pub(crate) fn is_wrapper(e: &Expression) -> bool {
    !std::ptr::eq(strip(e), e)
}

/// The binding names an assignment target writes (identifiers, and those
/// inside a destructuring pattern).
fn target_names(e: &Expression, out: &mut Vec<String>) {
    match strip(e) {
        Expression::Identifier(name) => out.push(name.clone()),
        Expression::ArrayExpression(array) => {
            for element in array.elements.iter().flatten() {
                match element {
                    ExpressionOrSpread::Expression(e) => target_names(e, out),
                    ExpressionOrSpread::Spread(spread) => target_names(&spread.argument, out),
                    ExpressionOrSpread::Empty => {}
                }
            }
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                target_names(&property.value, out);
            }
        }
        Expression::AssignmentExpression(default) => target_names(&default.left, out),
        Expression::SpreadElement(spread) => target_names(&spread.argument, out),
        Expression::RestElement(rest) => target_names(&rest.argument, out),
        _ => {}
    }
}

/// Abstracts an expression to a source (spec §3.3 rules 1-5).
pub(crate) fn abstract_expr(e: &Expression, cx: &Cx, env: &Env) -> Src {
    match e {
        Expression::ParenthesizedExpression(p) => abstract_expr(&p.expression, cx, env),
        Expression::TypeAssertion(t) => abstract_expr(&t.expression, cx, env),
        Expression::SatisfiesExpression(s) => abstract_expr(&s.expression, cx, env),
        Expression::ChainExpression(c) => abstract_expr(&c.expression, cx, env),
        Expression::NewExpression(new) => match env.constructed_class(new, cx) {
            Some(class) => Src::Inst(class.to_string()),
            // A rewritten class's name that resolves elsewhere (shadowed): not proven either way.
            None if matches!(&new.callee, Expression::Identifier(c) if env.plans.rewritten.contains_key(c)) => {
                Src::Unknown
            }
            None => Src::NotInst,
        },
        Expression::ThisExpression => match cx.this_class() {
            Some(class) if env.plans.rewritten.contains_key(class) => Src::Inst(class.to_string()),
            _ => Src::NotInst,
        },
        Expression::Identifier(name) => match env.scopes.resolve(name, cx) {
            Resolved::Binding(id) if env.functions.contains_key(&id) => Src::NotInst,
            Resolved::Binding(id) => Src::Binding(id),
            Resolved::Ambiguous => Src::Unknown,
            Resolved::Free => Src::NotInst,
        },
        Expression::CallExpression(call) => match strip(&call.callee) {
            Expression::Identifier(name) => match env.function(name, cx) {
                Some(f) if f.is_async_or_generator => Src::NotInst,
                Some(f) => Src::Call(f.key.clone()),
                None => Src::Unknown,
            },
            Expression::MemberExpression(member) => match &member.property {
                Some(m) => {
                    Src::MethodCall(Box::new(abstract_expr(&member.object, cx, env)), m.clone())
                }
                None => Src::Unknown,
            },
            _ => Src::Unknown,
        },
        Expression::AssignmentExpression(assign) => match assign.operator {
            AssignmentOperator::Assign => abstract_expr(&assign.right, cx, env),
            // `a ||= b` evaluates to `a` or `b`.
            AssignmentOperator::NullishAssign
            | AssignmentOperator::AndAssign
            | AssignmentOperator::OrAssign => Src::Join(vec![
                abstract_expr(&assign.left, cx, env),
                abstract_expr(&assign.right, cx, env),
            ]),
            // Arithmetic and bitwise compound assignments evaluate to a number or string.
            _ => Src::NotInst,
        },
        Expression::SequenceExpression(sequence) => match sequence.expressions.last() {
            Some(last) => abstract_expr(last, cx, env),
            None => Src::Unknown,
        },
        Expression::ConditionalExpression(c) => Src::Join(vec![
            abstract_expr(&c.consequent, cx, env),
            abstract_expr(&c.alternate, cx, env),
        ]),
        Expression::LogicalExpression(l) => Src::Join(vec![
            abstract_expr(&l.left, cx, env),
            abstract_expr(&l.right, cx, env),
        ]),
        // A member can never hold an instance: §3.3's allowlist refuses
        // storing one in a field or an element.
        Expression::Literal(_)
        | Expression::TemplateLiteral(_)
        | Expression::ArrayExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::MemberExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::YieldExpression(_)
        | Expression::BigIntLiteral(_) => Src::NotInst,
        Expression::MetaProperty(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::OptionalChainExpression(_)
        | Expression::SpreadElement(_)
        | Expression::RestElement(_)
        | Expression::ImportExpression(_)
        | Expression::DecoratedExpression(_)
        | Expression::JsxElement(_)
        | Expression::JsxFragment(_)
        | Expression::JsxEmptyExpression
        | Expression::SuperExpression
        | Expression::PrivateIdentifier(_) => Src::Unknown,
    }
}

/// A1-9: rewrites `new (C(a).m())` / `new (C(a).f)` / `new (C(a))` to the
/// canonical `new C(a)` at the chain's root, for rewritten `C` only.
pub(crate) fn reassociate_new(statements: &mut Vec<Statement>, plans: &ClassPlans) {
    walk(statements, &mut Reassociate { plans });
}

struct Reassociate<'a> {
    plans: &'a ClassPlans,
}

impl Visitor for Reassociate<'_> {
    fn expr(&mut self, expr: &mut Expression, _: &Pos, _: &Cx) {
        let Expression::NewExpression(new) = expr else {
            return;
        };
        if !new.args.is_empty() || matches!(new.callee, Expression::Identifier(_)) {
            return;
        }
        let Some(class) = new_target(new).map(str::to_string) else {
            return;
        };
        if !self.plans.rewritten.contains_key(&class) {
            return;
        }
        let mut chain = std::mem::replace(&mut new.callee, Expression::Identifier(String::new()));
        // Find the deepest Call(Identifier(class), args) and replace it with new class(args).
        fn root(e: &mut Expression) -> &mut Expression {
            let descend = match e {
                Expression::MemberExpression(_) => true,
                Expression::CallExpression(c) => !matches!(c.callee, Expression::Identifier(_)),
                _ => false,
            };
            if !descend {
                return e;
            }
            match e {
                Expression::MemberExpression(m) => root(&mut m.object),
                Expression::CallExpression(c) => root(&mut c.callee),
                _ => unreachable!("only members and calls descend"),
            }
        }
        let slot = root(&mut chain);
        if let Expression::CallExpression(call) = slot {
            let args = std::mem::take(&mut call.args);
            *slot = Expression::NewExpression(Box::new(NewExpression {
                callee: Expression::Identifier(class),
                args,
            }));
        }
        *expr = chain;
    }
}

/// Builds `Env.functions` (function declarations, and `const` declarators whose
/// initializer is a function or arrow expression; methods are keyed `C#m` under
/// the class's frame and recorded in `Env::method_key`).
pub(crate) fn build_env<'a>(
    statements: &mut Vec<Statement>,
    scopes: &'a Scopes,
    plans: &'a ClassPlans,
) -> Env<'a> {
    let mut builder = EnvBuilder {
        scopes,
        plans,
        functions: BTreeMap::new(),
        written: BTreeSet::new(),
        methods: BTreeMap::new(),
        classes: BTreeMap::new(),
    };
    walk(statements, &mut builder);
    let EnvBuilder {
        mut functions,
        written,
        methods,
        classes,
        ..
    } = builder;
    // A function binding that is ever written is not a known function.
    functions.retain(|id, _| !written.contains(id));
    Env {
        scopes,
        plans,
        functions,
        methods,
        classes,
    }
}

struct EnvBuilder<'a> {
    scopes: &'a Scopes,
    plans: &'a ClassPlans,
    functions: BTreeMap<BindingId, FnInfo>,
    written: BTreeSet<BindingId>,
    methods: BTreeMap<(String, String), Option<FnKey>>,
    classes: BTreeMap<String, Option<BindingId>>,
}

/// Inserts `value`, or poisons the entry to `None` on a second insert.
fn insert_unique<K: Ord, V>(map: &mut BTreeMap<K, Option<V>>, key: K, value: V) {
    map.entry(key)
        .and_modify(|v| *v = None)
        .or_insert(Some(value));
}

impl EnvBuilder<'_> {
    fn binding(&self, name: &str, cx: &Cx) -> Option<BindingId> {
        match self.scopes.resolve(name, cx) {
            Resolved::Binding(id) => Some(id),
            Resolved::Ambiguous | Resolved::Free => None,
        }
    }

    fn class(&mut self, name: &str, body: &ClassBody, binding: Option<BindingId>, cx: &Cx) {
        if !self.plans.rewritten.contains_key(name) {
            return;
        }
        match binding {
            Some(id) => insert_unique(&mut self.classes, name.to_string(), id),
            None => {
                self.classes.insert(name.to_string(), None);
            }
        }
        for method in body.methods.iter().filter(|m| !m.is_static) {
            let key = cx.child_key(&format!("{name}#{}", method.name));
            insert_unique(
                &mut self.methods,
                (name.to_string(), method.name.clone()),
                key,
            );
        }
    }
}

impl Visitor for EnvBuilder<'_> {
    fn expr(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        if matches!(
            pos,
            Pos::AssignTarget | Pos::UpdateTarget | Pos::Other("a loop target")
        ) {
            let mut names = Vec::new();
            target_names(expr, &mut names);
            for name in names {
                if let Some(id) = self.binding(&name, cx) {
                    self.written.insert(id);
                }
            }
        }
    }
    fn function_decl(&mut self, f: &FunctionDeclaration, cx: &Cx) {
        if f.name.is_empty() {
            return;
        }
        if let Some(id) = self.binding(&f.name, cx) {
            let info = FnInfo {
                key: cx.child_key(&f.name),
                params: f.params.clone(),
                is_async_or_generator: f.is_async || f.generator,
            };
            self.functions.insert(id, info);
        }
    }
    fn var_declarator(&mut self, d: &VariableDeclarator, kind: &str, cx: &Cx) {
        if kind != "const" {
            return;
        }
        let info = match &d.init {
            Some(Expression::FunctionExpression(f)) => FnInfo {
                key: cx.child_key(f.id.as_deref().unwrap_or(ANON)),
                params: f.params.iter().map(|p| p.name.clone()).collect(),
                is_async_or_generator: f.is_async || f.generator,
            },
            Some(Expression::ArrowFunctionExpression(a)) => FnInfo {
                key: cx.child_key(a.id.as_deref().unwrap_or(ANON)),
                params: a.params.iter().map(|p| p.name.clone()).collect(),
                is_async_or_generator: a.is_async,
            },
            _ => return,
        };
        if let Some(id) = self.binding(&d.id, cx) {
            self.functions.insert(id, info);
        }
    }
    fn class_decl(&mut self, class: &ClassDeclaration, _exported_default: bool, cx: &Cx) {
        let binding = self.binding(&class.name, cx);
        self.class(&class.name, &class.body, binding, cx);
    }
    fn class_expr(&mut self, class: &ClassExpression, cx: &Cx) {
        // A class expression is never rewritten; record it so a same-named
        // rewritten class's lookups are poisoned rather than misattributed.
        let name = class.id.as_deref().unwrap_or(ANON).to_string();
        self.class(&name, &class.body, None, cx);
    }
}

/// Whether control can fall off the end of `body`.
pub(crate) fn falls_through(body: &[Statement]) -> bool {
    !matches!(
        body.last(),
        Some(Statement::ReturnStatement(_) | Statement::ThrowStatement(_))
    )
}

/// An argument-less `return` at `s` or in its non-block statement children
/// (blocks are visited as statement lists of their own).
pub(crate) fn bare_return(s: &Statement) -> bool {
    match s {
        Statement::ReturnStatement(r) => r.argument.is_none(),
        Statement::LabeledStatement(l) => bare_return(&l.body),
        Statement::WithStatement(w) => bare_return(&w.body),
        Statement::ForInStatement(f) => bare_return(&f.body),
        Statement::ForOfStatement(f) => bare_return(&f.body),
        _ => false,
    }
}

/// A call site's arguments as sources; `None` when a spread makes positions unknown.
type Args = Option<Vec<Src>>;

fn args_of(args: &[Expression], cx: &Cx, env: &Env) -> Args {
    if args
        .iter()
        .any(|a| matches!(strip(a), Expression::SpreadElement(_)))
    {
        return None;
    }
    Some(args.iter().map(|a| abstract_expr(a, cx, env)).collect())
}

/// The sources of one parameter at a call site with `args`.
fn arg_src(args: &Args, index: usize) -> Src {
    match args {
        Some(args) => args.get(index).cloned().unwrap_or(Src::NotInst),
        None => Src::Unknown,
    }
}

struct Facts<'e, 'a> {
    env: &'e Env<'a>,
    bindings: BTreeMap<BindingId, Vec<Src>>,
    returns: BTreeMap<FnKey, Vec<Src>>,
    /// `(receiver, method, args)` of each `o.m(args)`.
    method_sites: Vec<(Src, String, Args)>,
    escaped: BTreeSet<FnKey>,
    /// Every function frame entered: key → (times entered, params).
    frames: BTreeMap<FnKey, (u32, Vec<String>)>,
    async_frames: BTreeSet<FnKey>,
}

impl Facts<'_, '_> {
    fn add_binding(&mut self, name: &str, cx: &Cx, src: Src) {
        if let Resolved::Binding(id) = self.env.scopes.resolve(name, cx) {
            self.bindings.entry(id).or_default().push(src);
        }
    }

    fn add_param(&mut self, frame: &str, name: &str, src: Src) {
        let id = BindingId {
            frame: frame.to_string(),
            name: name.to_string(),
        };
        self.bindings.entry(id).or_default().push(src);
    }

    fn add_return(&mut self, key: FnKey, src: Src) {
        self.returns.entry(key).or_default().push(src);
    }

    /// Fall-through and async/generator facts of a function frame keyed `key`.
    fn function(&mut self, key: FnKey, body: Option<&[Statement]>, is_async_or_generator: bool) {
        if is_async_or_generator {
            self.async_frames.insert(key.clone());
        }
        if body.is_none_or(falls_through) {
            self.add_return(key, Src::NotInst);
        }
    }

    fn class_body(&mut self, name: &str, body: &ClassBody, cx: &Cx) {
        for method in &body.methods {
            let key = cx.child_key(&format!("{name}#{}", method.name));
            let statements = method.body.as_ref().map(|b| b.body.as_slice());
            self.function(key, statements, method.is_async || method.generator);
        }
    }
}

impl Visitor for Facts<'_, '_> {
    fn expr(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        let env = self.env;
        if matches!(pos, Pos::Return) && !is_wrapper(expr) {
            self.add_return(cx.key().to_string(), abstract_expr(expr, cx, env));
        }
        if matches!(pos, Pos::UpdateTarget | Pos::Other("a loop target")) {
            let mut names = Vec::new();
            target_names(expr, &mut names);
            for name in names {
                self.add_binding(&name, cx, Src::NotInst);
            }
        }
        match expr {
            Expression::Identifier(name) if !matches!(pos, Pos::Callee) => {
                if let Some(f) = env.function(name, cx) {
                    self.escaped.insert(f.key.clone());
                }
            }
            Expression::AssignmentExpression(assign) => {
                if let Expression::Identifier(name) = strip(&assign.left) {
                    let src = match assign.operator {
                        AssignmentOperator::Assign
                        | AssignmentOperator::NullishAssign
                        | AssignmentOperator::AndAssign
                        | AssignmentOperator::OrAssign => abstract_expr(&assign.right, cx, env),
                        _ => Src::NotInst,
                    };
                    self.add_binding(name, cx, src);
                } else {
                    // A destructuring target: what it binds is not tracked.
                    let mut names = Vec::new();
                    target_names(&assign.left, &mut names);
                    for name in names {
                        self.add_binding(&name, cx, Src::Unknown);
                    }
                }
            }
            Expression::CallExpression(call) => match strip(&call.callee) {
                Expression::Identifier(name) => {
                    if let Some(f) = env.function(name, cx) {
                        let args = args_of(&call.args, cx, env);
                        let (key, params) = (f.key.clone(), f.params.clone());
                        for (index, param) in params.iter().enumerate() {
                            self.add_param(&key, param, arg_src(&args, index));
                        }
                    }
                }
                Expression::MemberExpression(member) => {
                    if let Some(m) = &member.property {
                        let receiver = abstract_expr(&member.object, cx, env);
                        let args = args_of(&call.args, cx, env);
                        self.method_sites.push((receiver, m.clone(), args));
                    }
                }
                _ => {}
            },
            Expression::NewExpression(new) => {
                if let Some(class) = env.constructed_class(new, cx) {
                    if let Some(key) = env.method_key(class, "constructor").cloned() {
                        let args = args_of(&new.args, cx, env);
                        for (index, param) in env.scopes.params(&key).iter().enumerate() {
                            self.add_param(&key, param, arg_src(&args, index));
                        }
                    }
                }
            }
            Expression::FunctionExpression(f) => {
                let key = cx.child_key(f.id.as_deref().unwrap_or(ANON));
                let body = f.body.as_ref().map(|b| b.body.as_slice());
                self.function(key, body, f.is_async || f.generator);
            }
            // The expression body is a `Pos::Return`; it never falls through.
            Expression::ArrowFunctionExpression(a) if a.is_async => {
                self.async_frames
                    .insert(cx.child_key(a.id.as_deref().unwrap_or(ANON)));
            }
            _ => {}
        }
    }
    fn stmts(&mut self, list: &mut Vec<Statement>, cx: &Cx) {
        if list.iter().any(bare_return) {
            self.add_return(cx.key().to_string(), Src::NotInst);
        }
        // `export default function f` uses `f` as a value (rule 4).
        for statement in list.iter() {
            if let Statement::ExportDefault(ExportDefaultDeclaration::FunctionDeclaration(f)) =
                statement
            {
                if let Some(info) = self.env.function(&f.name, cx) {
                    self.escaped.insert(info.key.clone());
                }
            }
        }
    }
    fn enter_frame(&mut self, cx: &Cx, params: &[String]) {
        let entry = self
            .frames
            .entry(cx.key().to_string())
            .or_insert((0, params.to_vec()));
        entry.0 += 1;
    }
    fn class_decl(&mut self, class: &ClassDeclaration, _exported_default: bool, cx: &Cx) {
        self.add_binding(&class.name, cx, Src::NotInst);
        self.class_body(&class.name, &class.body, cx);
    }
    fn class_expr(&mut self, class: &ClassExpression, cx: &Cx) {
        self.class_body(class.id.as_deref().unwrap_or(ANON), &class.body, cx);
    }
    fn var_declarator(&mut self, d: &VariableDeclarator, _kind: &str, cx: &Cx) {
        let src = match &d.init {
            Some(init) => abstract_expr(init, cx, self.env),
            None => Src::NotInst,
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
            self.add_binding(name, cx, Src::NotInst);
        }
        self.function(
            cx.child_key(name),
            Some(&f.body.body),
            f.is_async || f.generator,
        );
    }
    fn catch_param(&mut self, name: &str, cx: &Cx) {
        self.add_binding(name, cx, Src::NotInst);
    }
    fn export_specifier(&mut self, local: &str) {
        let id = BindingId {
            frame: String::new(),
            name: local.to_string(),
        };
        if let Some(f) = self.env.functions.get(&id) {
            self.escaped.insert(f.key.clone());
        }
    }
}

pub(crate) struct Provenance {
    values: BTreeMap<BindingId, Val>,
    returns: BTreeMap<FnKey, Val>,
    /// The parameters of every frame whose call sites are not all known.
    pub unknown_params: Vec<BindingId>,
}

impl Provenance {
    pub(crate) fn solve(statements: &mut Vec<Statement>, env: &Env) -> Provenance {
        let mut facts = Facts {
            env,
            bindings: BTreeMap::new(),
            returns: BTreeMap::new(),
            method_sites: Vec::new(),
            escaped: BTreeSet::new(),
            frames: BTreeMap::new(),
            async_frames: BTreeSet::new(),
        };
        walk(statements, &mut facts);

        // Frames whose call sites are all known: a function bound once and
        // never escaping, or a rewritten class's method or constructor. A
        // frame key entered twice (two same-named functions) is never known.
        let unique = |key: &FnKey| facts.frames.get(key).is_some_and(|(count, _)| *count == 1);
        let mut known: BTreeSet<&FnKey> = env
            .functions
            .values()
            .map(|f| &f.key)
            .filter(|key| !facts.escaped.contains(*key))
            .collect();
        known.extend(env.methods.values().flatten());
        known.retain(|key| unique(key));
        let mut unknown_params: Vec<BindingId> = Vec::new();
        for (key, (_, params)) in &facts.frames {
            if !known.contains(key) {
                for name in params {
                    unknown_params.push(BindingId {
                        frame: key.clone(),
                        name: name.clone(),
                    });
                }
            }
        }

        let mut p = Provenance {
            values: BTreeMap::new(),
            returns: BTreeMap::new(),
            unknown_params: Vec::new(),
        };
        loop {
            let mut values: BTreeMap<BindingId, Val> = BTreeMap::new();
            let join_into = |values: &mut BTreeMap<BindingId, Val>, id: BindingId, v: Val| {
                let slot = values.entry(id).or_insert(Val::Bottom);
                *slot = slot.join(&v);
            };
            for (id, sources) in &facts.bindings {
                for src in sources {
                    join_into(&mut values, id.clone(), p.eval(src, env));
                }
            }
            for (receiver, method, args) in &facts.method_sites {
                // An unresolved receiver may be any class with `method`.
                let (classes, unresolved): (Vec<&str>, bool) = match p.eval(receiver, env) {
                    Val::Inst(class) => match env.plans.rewritten.get(&class) {
                        Some(c) if c.methods.contains(method) => (vec![c.name.as_str()], false),
                        _ => (vec![], false),
                    },
                    Val::Unknown => (
                        env.plans
                            .rewritten
                            .values()
                            .filter(|c| c.methods.contains(method))
                            .map(|c| c.name.as_str())
                            .collect(),
                        true,
                    ),
                    Val::Bottom | Val::NotInst => (vec![], false),
                };
                for class in classes {
                    let Some(key) = env.method_key(class, method) else {
                        continue;
                    };
                    for (index, param) in env.scopes.params(key).iter().enumerate() {
                        let v = if unresolved {
                            Val::Unknown
                        } else {
                            p.eval(&arg_src(args, index), env)
                        };
                        join_into(
                            &mut values,
                            BindingId {
                                frame: key.clone(),
                                name: param.clone(),
                            },
                            v,
                        );
                    }
                }
            }
            for id in &unknown_params {
                join_into(&mut values, id.clone(), Val::Unknown);
            }
            let mut returns: BTreeMap<FnKey, Val> = BTreeMap::new();
            for (key, sources) in &facts.returns {
                // R-13: a frame with untracked call sites (escaped, anonymous
                // callbacks, object-literal functions) hands its return out unseen.
                let v = if facts.async_frames.contains(key) || !known.contains(key) {
                    Val::Unknown
                } else {
                    sources
                        .iter()
                        .fold(Val::Bottom, |acc, src| acc.join(&p.eval(src, env)))
                };
                returns.insert(key.clone(), v);
            }
            let next = Provenance { values, returns, unknown_params: Vec::new() };
            if next.values == p.values && next.returns == p.returns {
                p.unknown_params = unknown_params;
                return p;
            }
            p = next;
        }
    }

    pub(crate) fn eval(&self, src: &Src, env: &Env) -> Val {
        match src {
            Src::Inst(class) => Val::Inst(class.clone()),
            Src::NotInst => Val::NotInst,
            Src::Unknown => Val::Unknown,
            Src::Binding(id) => self.binding(id),
            Src::Call(key) => self.returns(key),
            Src::MethodCall(receiver, method) => match self.eval(receiver, env) {
                Val::Inst(class) => {
                    let is_method = env
                        .plans
                        .rewritten
                        .get(&class)
                        .is_some_and(|c| c.methods.contains(method));
                    if !is_method {
                        return Val::NotInst;
                    }
                    match env.method_key(&class, method) {
                        Some(key) => self.returns(key),
                        // Declared more than once: not proven.
                        None => Val::Unknown,
                    }
                }
                Val::NotInst => Val::NotInst,
                Val::Bottom => Val::Bottom,
                Val::Unknown => Val::Unknown,
            },
            Src::Join(sources) => sources
                .iter()
                .fold(Val::Bottom, |acc, s| acc.join(&self.eval(s, env))),
        }
    }

    pub(crate) fn binding(&self, id: &BindingId) -> Val {
        self.values.get(id).cloned().unwrap_or(Val::Bottom)
    }

    pub(crate) fn returns(&self, key: &str) -> Val {
        self.returns.get(key).cloned().unwrap_or(Val::Bottom)
    }
}
