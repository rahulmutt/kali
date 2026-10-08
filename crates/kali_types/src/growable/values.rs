//! Growable-runtime-arrays residual fixes (spec A-39, A-40): two facts about
//! the scalar values growable-array operations consume and produce.
//!
//! * A-39 (residual R2): whether a floating-point index is a whole number.
//!   [`integral_proof`] reduces an index expression to the facts only the
//!   solved table can settle (is a binding, an element or a return f64; is
//!   every write of an f64 binding itself whole). `repr_infer` builds one per
//!   growable index and refuses a float index whose proof fails, so `check`
//!   and `run` agree; codegen truncates an admitted one.
//! * A-40 (residual R1): where a growable `includes` result goes. One walk
//!   ([`collect_value_facts`]) records every occurrence of a value that may be
//!   such a result (the call, a binding read, a declared-function call) with
//!   the position it sits in, every write of every binding and every
//!   `return`. [`solve_search_booleans`] finds the bindings and functions that
//!   only ever hold booleans (codegen gives their reads and calls the boolean
//!   shape) and refuses every occurrence kali cannot keep as a boolean.
//!   The same walk records the binding writes the A-39 proof leans on.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{
    AssignmentOperator, BlockStatement, Expression, ExpressionOrSpread, ForInLefthand, ForInit,
    ForOfLefthand, LiteralValue, OptionalChainInner, Statement, VariableDeclaration,
};

use super::facts::WalkContext;
use super::flow::{GrowNode, GrowSolution, TOP_LEVEL};
use super::unwrap;

// ---- A-39: whole-number proofs ---------------------------------------------

/// The obligation that an expression's value is a whole number (or NaN or
/// an infinity, which codegen's checked truncation traps on).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Integral {
    Yes,
    No,
    /// `(scope, name)`: whole when not an f64, or when every write is.
    Binding(String, String),
    /// One element of the array binding `(scope, name)`: whole when not f64.
    Elements(String, String),
    /// What the declared function returns: whole when not f64.
    Return(String),
    All(Vec<Integral>),
}

/// How a proof names bindings and callees at the site it is built in.
pub(crate) struct Resolver<'a> {
    /// The `(scope, name)` an identifier read denotes, if declared.
    pub(crate) binding: &'a dyn Fn(&str) -> Option<(String, String)>,
    /// The declared function a bare call `name(…)` reaches, if any.
    pub(crate) callee: &'a dyn Fn(&str) -> Option<String>,
}

const WHOLE_MATH: &[&str] = &["floor", "ceil", "trunc", "round", "sign", "imul", "clz32"];
const WHOLE_METHODS: &[&str] = &[
    "charCodeAt",
    "codePointAt",
    "indexOf",
    "lastIndexOf",
    "findIndex",
    "findLastIndex",
    "search",
    "localeCompare",
];

fn all(parts: Vec<Integral>) -> Integral {
    let mut kept = Vec::new();
    for part in parts {
        match part {
            Integral::Yes => {}
            Integral::No => return Integral::No,
            Integral::All(inner) => kept.extend(inner),
            other => kept.push(other),
        }
    }
    match kept.len() {
        0 => Integral::Yes,
        1 => kept.pop().expect("one part"),
        _ => Integral::All(kept),
    }
}

/// The whole-number obligation of `expr`. Any shape not listed is
/// [`Integral::No`]: a float index of that shape is refused.
pub(crate) fn integral_proof(expr: &Expression, r: &Resolver<'_>) -> Integral {
    let binding = |name: &str| match (r.binding)(name) {
        Some((scope, name)) => Integral::Binding(scope, name),
        None => Integral::No,
    };
    match unwrap(expr) {
        Expression::Literal(LiteralValue::Number(n)) => {
            if n.is_finite() && n.fract() == 0.0 {
                Integral::Yes
            } else {
                Integral::No
            }
        }
        Expression::Identifier(name) => binding(name),
        Expression::UnaryExpression(u) => match u.operator.as_str() {
            "-" | "+" => integral_proof(&u.argument, r),
            "~" => Integral::Yes,
            _ => Integral::No,
        },
        Expression::BinaryExpression(b) => match b.operator.as_str() {
            "+" | "-" | "*" | "%" => all(vec![
                integral_proof(&b.left, r),
                integral_proof(&b.right, r),
            ]),
            "|" | "&" | "^" | "<<" | ">>" | ">>>" => Integral::Yes,
            _ => Integral::No,
        },
        Expression::UpdateExpression(u) => match unwrap(&u.argument) {
            Expression::Identifier(name) => binding(name),
            _ => Integral::No,
        },
        Expression::LogicalExpression(l) => all(vec![
            integral_proof(&l.left, r),
            integral_proof(&l.right, r),
        ]),
        Expression::ConditionalExpression(c) => all(vec![
            integral_proof(&c.consequent, r),
            integral_proof(&c.alternate, r),
        ]),
        Expression::SequenceExpression(s) => s
            .expressions
            .last()
            .map_or(Integral::No, |last| integral_proof(last, r)),
        Expression::AssignmentExpression(a) => match a.operator {
            AssignmentOperator::Assign => integral_proof(&a.right, r),
            AssignmentOperator::AddAssign
            | AssignmentOperator::SubtractAssign
            | AssignmentOperator::MultiplyAssign => all(vec![
                integral_proof(&a.left, r),
                integral_proof(&a.right, r),
            ]),
            _ => Integral::No,
        },
        Expression::MemberExpression(m) => {
            if m.computed_index.is_some() {
                return match unwrap(&m.object) {
                    Expression::Identifier(name) => match (r.binding)(name) {
                        Some((scope, name)) => Integral::Elements(scope, name),
                        None => Integral::No,
                    },
                    _ => Integral::No,
                };
            }
            if m.dot_name() == Some("length") {
                Integral::Yes
            } else {
                Integral::No
            }
        }
        Expression::CallExpression(call) => match unwrap(&call.callee) {
            Expression::Identifier(name) => match (r.callee)(name) {
                Some(key) => Integral::Return(key),
                None if name == "parseInt" => Integral::Yes,
                None => Integral::No,
            },
            Expression::MemberExpression(m) if m.computed_index.is_none() => {
                let Some(method) = m.dot_name() else {
                    return Integral::No;
                };
                if matches!(unwrap(&m.object), Expression::Identifier(o) if o == "Math") {
                    return match method {
                        _ if WHOLE_MATH.contains(&method) => Integral::Yes,
                        "min" | "max" => {
                            all(call.args.iter().map(|a| integral_proof(a, r)).collect())
                        }
                        "abs" => call
                            .args
                            .first()
                            .map_or(Integral::No, |a| integral_proof(a, r)),
                        _ => Integral::No,
                    };
                }
                if WHOLE_METHODS.contains(&method) {
                    Integral::Yes
                } else {
                    Integral::No
                }
            }
            _ => Integral::No,
        },
        _ => Integral::No,
    }
}

/// Every write of every binding, as a whole-number obligation, and the
/// bindings with a write the proof cannot follow.
#[derive(Debug, Default)]
pub(crate) struct IntegralWrites {
    pub(crate) writes: BTreeMap<(String, String), Vec<Integral>>,
    pub(crate) taints: BTreeSet<(String, String)>,
}

/// Whether `proof` holds. `is_float` answers for a `Binding`, `Elements` or
/// `Return` leaf whether the solved table makes it an f64. A binding cycle
/// (`lo = lo + 1`) holds by induction over its other writes.
pub(crate) fn integral_holds(
    proof: &Integral,
    writes: &IntegralWrites,
    is_float: &dyn Fn(&Integral) -> bool,
) -> bool {
    fn go(
        proof: &Integral,
        writes: &IntegralWrites,
        is_float: &dyn Fn(&Integral) -> bool,
        visiting: &mut BTreeSet<(String, String)>,
    ) -> bool {
        match proof {
            Integral::Yes => true,
            Integral::No => false,
            Integral::All(parts) => parts.iter().all(|p| go(p, writes, is_float, visiting)),
            Integral::Elements(..) | Integral::Return(_) => !is_float(proof),
            Integral::Binding(scope, name) => {
                if !is_float(proof) {
                    return true;
                }
                let key = (scope.clone(), name.clone());
                if visiting.contains(&key) {
                    return true;
                }
                if writes.taints.contains(&key) {
                    return false;
                }
                let Some(list) = writes.writes.get(&key) else {
                    return false;
                };
                visiting.insert(key);
                list.iter().all(|w| go(w, writes, is_float, visiting))
            }
        }
    }
    go(proof, writes, is_float, &mut BTreeSet::new())
}

// ---- A-40: growable `includes` results --------------------------------------

/// What a written or returned value is, for the boolean solve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BoolValue {
    /// `r.includes(…)`; the result is a growable search result when `r` is
    /// growable.
    Search(GrowNode),
    /// `true`, `false`, a comparison or `!x`: always a boolean.
    Syntactic,
    /// A read of the binding.
    Binding(GrowNode),
    /// A call of the declared function.
    Call(String),
    Other,
}

/// The position an occurrence sits in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ctx {
    /// A condition, `!`, or a discarded value: only truthiness is read.
    Truthy,
    /// An operand codegen renders or computes from the boolean shape:
    /// a `console` argument, `+`, a template, `typeof`, a comparison, an
    /// arithmetic or bitwise operand.
    Render,
    /// The value written to a binding or returned (`GrowNode::Return`).
    Store(GrowNode),
    /// Anywhere else.
    Refuse,
}

#[derive(Debug, Default)]
pub(crate) struct ValueFacts {
    /// Every write of every binding, and every `return` of every declared
    /// function (`GrowNode::Return`).
    pub(crate) bool_writes: BTreeMap<GrowNode, Vec<BoolValue>>,
    /// Bindings (and returns) with a write or read the solve cannot follow:
    /// a compound assignment, an update, a parameter, a loop or `catch`
    /// variable, a capture, an opaque scope.
    pub(crate) bool_taints: BTreeSet<GrowNode>,
    /// `(value, function whose body holds it, position)`.
    pub(crate) bool_uses: Vec<(BoolValue, String, Ctx)>,
    pub(crate) integral: IntegralWrites,
}

pub(crate) fn collect_value_facts(statements: &[Statement], ctx: &WalkContext<'_>) -> ValueFacts {
    let mut walker = Walker {
        ctx,
        facts: ValueFacts::default(),
        frames: vec![Frame {
            key: TOP_LEVEL.to_string(),
            declared: false,
        }],
        anonymous: 0,
        opaque: Vec::new(),
    };
    walker.statements(statements);
    let opaque = std::mem::take(&mut walker.opaque);
    let mut facts = walker.facts;
    if !opaque.is_empty() {
        let keys: Vec<GrowNode> = facts.bool_writes.keys().cloned().collect();
        for key in keys {
            let scope = match &key {
                GrowNode::Binding(scope, _) | GrowNode::Return(scope) => scope,
                GrowNode::Temp(_) => continue,
            };
            if opaque.iter().any(|stack| stack.contains(scope)) {
                facts.bool_taints.insert(key);
            }
        }
    }
    facts
}

/// The boolean bindings `(scope, name)` and functions, and the functions
/// (by site) holding a refused occurrence.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SearchBooleans {
    pub(crate) bindings: BTreeSet<(String, String)>,
    pub(crate) returns: BTreeSet<String>,
    pub(crate) refused_sites: BTreeSet<String>,
}

pub(crate) fn solve_search_booleans(facts: &ValueFacts, solution: &GrowSolution) -> SearchBooleans {
    let derived = |value: &BoolValue, set: &BTreeSet<GrowNode>| match value {
        BoolValue::Search(receiver) => solution.is_growable(receiver),
        BoolValue::Binding(key) => set.contains(key),
        BoolValue::Call(func) => set.contains(&GrowNode::Return(func.clone())),
        BoolValue::Syntactic | BoolValue::Other => false,
    };
    let mut set: BTreeSet<GrowNode> = BTreeSet::new();
    loop {
        let next: BTreeSet<GrowNode> = facts
            .bool_writes
            .iter()
            .filter(|(key, writes)| {
                !facts.bool_taints.contains(*key)
                    && writes
                        .iter()
                        .all(|w| matches!(w, BoolValue::Syntactic) || derived(w, &set))
                    && writes.iter().any(|w| derived(w, &set))
            })
            .map(|(key, _)| key.clone())
            .collect();
        if next == set {
            break;
        }
        set = next;
    }
    let mut out = SearchBooleans::default();
    for (value, site, ctx) in &facts.bool_uses {
        if !derived(value, &set) {
            continue;
        }
        let kept = match ctx {
            Ctx::Truthy | Ctx::Render => true,
            Ctx::Store(key) => set.contains(key),
            Ctx::Refuse => false,
        };
        if !kept {
            out.refused_sites.insert(site.clone());
        }
    }
    for key in set {
        match key {
            GrowNode::Binding(scope, name) => {
                out.bindings.insert((scope, name));
            }
            GrowNode::Return(func) => {
                out.returns.insert(func);
            }
            GrowNode::Temp(_) => {}
        }
    }
    out
}

// ---- the walk ----------------------------------------------------------------

struct Frame {
    key: String,
    /// A `function` declaration: its calls resolve to it by name, so its
    /// `return` values are tracked. Function expressions and arrows are not.
    declared: bool,
}

enum Resolved {
    Local(GrowNode),
    Captured(GrowNode),
    Module(GrowNode),
    Unknown,
}

struct Walker<'w, 'c> {
    ctx: &'w WalkContext<'c>,
    facts: ValueFacts,
    frames: Vec<Frame>,
    anonymous: usize,
    /// The function-key stack at every class, `with`, JSX or module-syntax
    /// site the walk does not enter.
    opaque: Vec<Vec<String>>,
}

const CONSOLE_METHODS: &[&str] = &["log", "error", "warn", "info", "debug"];

fn binding_key(node: &GrowNode) -> Option<(String, String)> {
    match node {
        GrowNode::Binding(scope, name) => Some((scope.clone(), name.clone())),
        _ => None,
    }
}

impl Walker<'_, '_> {
    fn site(&self) -> &str {
        &self
            .frames
            .last()
            .expect("the module frame is never popped")
            .key
    }

    fn resolve(&self, name: &str) -> Resolved {
        let site = self.site();
        if (self.ctx.is_declared)(site, name) {
            return Resolved::Local(GrowNode::Binding(site.to_string(), name.to_string()));
        }
        for frame in self.frames.iter().rev().skip(1) {
            if (self.ctx.is_declared)(&frame.key, name) {
                let node = GrowNode::Binding(frame.key.clone(), name.to_string());
                return if frame.key == TOP_LEVEL {
                    Resolved::Module(node)
                } else {
                    Resolved::Captured(node)
                };
            }
        }
        Resolved::Unknown
    }

    /// The binding a name denotes; a captured one is tainted (codegen's
    /// capture lane has no boolean shape).
    fn target(&mut self, name: &str) -> Option<GrowNode> {
        match self.resolve(name) {
            Resolved::Local(node) | Resolved::Module(node) => Some(node),
            Resolved::Captured(node) => {
                self.bool_taint(node.clone());
                Some(node)
            }
            Resolved::Unknown => None,
        }
    }

    fn resolver_binding(&self, name: &str) -> Option<(String, String)> {
        match self.resolve(name) {
            Resolved::Local(n) | Resolved::Captured(n) | Resolved::Module(n) => binding_key(&n),
            Resolved::Unknown => None,
        }
    }

    fn integral(&self, expr: &Expression) -> Integral {
        let site = self.site().to_string();
        let binding = |name: &str| self.resolver_binding(name);
        let callee = |name: &str| (self.ctx.resolve_callee)(&site, name);
        integral_proof(
            expr,
            &Resolver {
                binding: &binding,
                callee: &callee,
            },
        )
    }

    /// A write neither proof can follow.
    fn taint(&mut self, node: GrowNode) {
        if let Some(key) = binding_key(&node) {
            self.facts.integral.taints.insert(key);
        }
        self.bool_taint(node);
    }

    /// A write or read the boolean solve cannot follow.
    fn bool_taint(&mut self, node: GrowNode) {
        self.facts.bool_taints.insert(node);
    }

    fn write(&mut self, node: GrowNode, value: Option<&Expression>) {
        let (bool_value, integral) = match value {
            Some(e) => (self.classify(e), self.integral(e)),
            None => (BoolValue::Other, Integral::No),
        };
        if let Some(key) = binding_key(&node) {
            self.facts
                .integral
                .writes
                .entry(key)
                .or_default()
                .push(integral);
        }
        self.facts
            .bool_writes
            .entry(node)
            .or_default()
            .push(bool_value);
    }

    fn use_value(&mut self, value: BoolValue, ctx: Ctx) {
        if matches!(value, BoolValue::Other | BoolValue::Syntactic) {
            return;
        }
        let site = self.site().to_string();
        self.facts.bool_uses.push((value, site, ctx));
    }

    /// The array value `r` of `r.includes(…)`, as the growable solve names it.
    fn receiver(&self, expr: &Expression) -> Option<GrowNode> {
        match unwrap(expr) {
            Expression::Identifier(name) => match self.resolve(name) {
                Resolved::Local(n) | Resolved::Module(n) => Some(n),
                _ => None,
            },
            Expression::CallExpression(call) => match unwrap(&call.callee) {
                Expression::Identifier(name) => {
                    (self.ctx.resolve_callee)(self.site(), name).map(GrowNode::Return)
                }
                Expression::MemberExpression(m)
                    if m.computed_index.is_none() && m.dot_name() == Some("slice") =>
                {
                    self.receiver(&m.object)
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn search_receiver(&self, expr: &Expression) -> Option<GrowNode> {
        let Expression::CallExpression(call) = unwrap(expr) else {
            return None;
        };
        match unwrap(&call.callee) {
            Expression::MemberExpression(m)
                if m.computed_index.is_none() && m.dot_name() == Some("includes") =>
            {
                self.receiver(&m.object)
            }
            _ => None,
        }
    }

    fn classify(&self, expr: &Expression) -> BoolValue {
        let expr = unwrap(expr);
        if let Some(receiver) = self.search_receiver(expr) {
            return BoolValue::Search(receiver);
        }
        match expr {
            Expression::Literal(LiteralValue::Boolean(_)) => BoolValue::Syntactic,
            Expression::BinaryExpression(b)
                if matches!(
                    b.operator.as_str(),
                    "==" | "===" | "!=" | "!==" | "<" | ">" | "<=" | ">="
                ) =>
            {
                BoolValue::Syntactic
            }
            Expression::UnaryExpression(u) if u.operator == "!" => BoolValue::Syntactic,
            Expression::Identifier(name) => match self.resolve(name) {
                Resolved::Local(n) | Resolved::Module(n) | Resolved::Captured(n) => {
                    BoolValue::Binding(n)
                }
                Resolved::Unknown => BoolValue::Other,
            },
            Expression::CallExpression(call) => match unwrap(&call.callee) {
                Expression::Identifier(name) => (self.ctx.resolve_callee)(self.site(), name)
                    .map_or(BoolValue::Other, BoolValue::Call),
                _ => BoolValue::Other,
            },
            _ => BoolValue::Other,
        }
    }

    fn statements(&mut self, statements: &[Statement]) {
        for stmt in statements {
            self.statement(stmt);
        }
    }

    fn block(&mut self, block: &BlockStatement) {
        self.statements(&block.body);
    }

    fn opaque_site(&mut self) {
        let stack = self.frames.iter().map(|f| f.key.clone()).collect();
        self.opaque.push(stack);
    }

    fn statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::ExpressionStatement(s) => self.expr(&s.expression, Ctx::Truthy),
            Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::TypeAliasDeclaration(_)
            | Statement::InterfaceDeclaration(_) => {}
            Statement::WithStatement(_)
            | Statement::ClassDeclaration(_)
            | Statement::ImportDeclaration(_)
            | Statement::ExportAll(_)
            | Statement::ExportNamed(_)
            | Statement::ExportDefault(_)
            | Statement::EnumDeclaration(_) => self.opaque_site(),
            Statement::ReturnStatement(s) => self.return_value(s.argument.as_ref()),
            Statement::LabeledStatement(s) => self.statement(&s.body),
            Statement::IfStatement(s) => {
                self.expr(&s.test, Ctx::Truthy);
                self.block(&s.consequent);
                if let Some(alternate) = &s.alternate {
                    self.block(alternate);
                }
            }
            Statement::SwitchStatement(s) => {
                self.expr(&s.discriminant, Ctx::Refuse);
                for case in &s.cases {
                    if let Some(test) = &case.test {
                        self.expr(test, Ctx::Refuse);
                    }
                    self.statements(&case.consequent);
                }
            }
            Statement::ThrowStatement(s) => self.expr(&s.argument, Ctx::Refuse),
            Statement::TryStatement(s) => {
                self.block(&s.block);
                if let Some(handler) = &s.handler {
                    let node = GrowNode::Binding(self.site().to_string(), handler.param.clone());
                    self.taint(node);
                    self.block(&handler.body);
                }
                if let Some(finalizer) = &s.finalizer {
                    self.block(finalizer);
                }
            }
            Statement::BlockStatement(s) => self.block(s),
            Statement::ForStatement(s) => {
                match &s.init {
                    Some(ForInit::VariableDeclaration(d)) => self.variable_declaration(d),
                    Some(ForInit::Expression(e)) => self.expr(e, Ctx::Truthy),
                    None => {}
                }
                if let Some(test) = &s.test {
                    self.expr(test, Ctx::Truthy);
                }
                if let Some(update) = &s.update {
                    self.expr(update, Ctx::Truthy);
                }
                self.block(&s.body);
            }
            Statement::ForInStatement(s) => {
                match &s.left {
                    ForInLefthand::VariableDeclaration(d) => self.loop_variables(d),
                    ForInLefthand::Expression(e) => self.loop_target(e),
                }
                self.expr(&s.right, Ctx::Refuse);
                self.statement(&s.body);
            }
            Statement::ForOfStatement(s) => {
                match &s.left {
                    ForOfLefthand::VariableDeclaration(d) => self.loop_variables(d),
                    ForOfLefthand::Expression(e) => self.loop_target(e),
                }
                self.expr(&s.right, Ctx::Refuse);
                self.statement(&s.body);
            }
            Statement::WhileStatement(s) => {
                self.expr(&s.test, Ctx::Truthy);
                self.block(&s.body);
            }
            Statement::DoWhileStatement(s) => {
                self.block(&s.body);
                self.expr(&s.test, Ctx::Truthy);
            }
            Statement::FunctionDeclaration(decl) => {
                self.function(decl.name.clone(), true, &decl.params, |w| {
                    w.statements(&decl.body.body);
                    if crate::array_return::body_falls_off_end(&decl.body.body) {
                        let node = GrowNode::Return(w.site().to_string());
                        w.write(node, None);
                    }
                })
            }
            Statement::VariableDeclaration(d) => self.variable_declaration(d),
        }
    }

    fn variable_declaration(&mut self, d: &VariableDeclaration) {
        for declarator in &d.declarations {
            let node = GrowNode::Binding(self.site().to_string(), declarator.id.clone());
            self.write(node.clone(), declarator.init.as_ref());
            if let Some(init) = &declarator.init {
                self.expr(init, Ctx::Store(node));
            }
        }
    }

    fn loop_variables(&mut self, d: &VariableDeclaration) {
        for declarator in &d.declarations {
            let node = GrowNode::Binding(self.site().to_string(), declarator.id.clone());
            self.taint(node);
        }
    }

    fn loop_target(&mut self, e: &Expression) {
        match unwrap(e) {
            Expression::Identifier(name) => {
                if let Some(node) = self.target(name) {
                    self.taint(node);
                }
            }
            other => self.expr(other, Ctx::Refuse),
        }
    }

    fn function(
        &mut self,
        key: String,
        declared: bool,
        params: &[String],
        body: impl FnOnce(&mut Self),
    ) {
        self.frames.push(Frame {
            key: key.clone(),
            declared,
        });
        for param in params {
            self.taint(GrowNode::Binding(key.clone(), param.clone()));
        }
        body(self);
        self.frames.pop();
    }

    fn return_value(&mut self, arg: Option<&Expression>) {
        let frame = self
            .frames
            .last()
            .expect("the module frame is never popped");
        if !frame.declared {
            if let Some(arg) = arg {
                self.expr(arg, Ctx::Refuse);
            }
            return;
        }
        let node = GrowNode::Return(frame.key.clone());
        self.write(node.clone(), arg);
        if let Some(arg) = arg {
            self.expr(arg, Ctx::Store(node));
        }
    }

    fn anonymous_key(&mut self) -> String {
        self.anonymous += 1;
        format!("<anonymous {}>", self.anonymous)
    }

    /// Records `expr` (when it may be a search result) in `ctx`, then walks
    /// its operands in the positions they sit in.
    fn expr(&mut self, expr: &Expression, ctx: Ctx) {
        let value = self.classify(expr);
        if matches!(
            value,
            BoolValue::Search(_) | BoolValue::Binding(_) | BoolValue::Call(_)
        ) {
            if let Expression::Identifier(name) = unwrap(expr) {
                // A captured read: codegen's capture lane has no boolean shape.
                if let Resolved::Captured(node) = self.resolve(name) {
                    self.bool_taint(node);
                    self.use_value(value, Ctx::Refuse);
                    return;
                }
            }
            self.use_value(value, ctx.clone());
        }
        self.operands(unwrap(expr), ctx);
    }

    /// Walks the operands of `expr` (already unwrapped and recorded) in the
    /// positions they sit in.
    fn operands(&mut self, expr: &Expression, ctx: Ctx) {
        let truthy = matches!(ctx, Ctx::Truthy);
        match expr {
            Expression::ParenthesizedExpression(_)
            | Expression::TypeAssertion(_)
            | Expression::SatisfiesExpression(_) => unreachable!("unwrapped by the caller"),
            Expression::Identifier(_)
            | Expression::Literal(_)
            | Expression::BigIntLiteral(_)
            | Expression::MetaProperty(_)
            | Expression::JsxEmptyExpression
            | Expression::ThisExpression
            | Expression::SuperExpression
            | Expression::PrivateIdentifier(_) => {}
            Expression::BinaryExpression(b) => {
                let operand = if matches!(b.operator.as_str(), "in" | "instanceof") {
                    Ctx::Refuse
                } else if matches!(b.operator.as_str(), "&&" | "||" | "??") {
                    // A logical operator in binary spelling: as the
                    // `LogicalExpression` arm.
                    if truthy {
                        Ctx::Truthy
                    } else {
                        Ctx::Refuse
                    }
                } else {
                    Ctx::Render
                };
                self.expr(&b.left, operand.clone());
                self.expr(&b.right, operand);
            }
            Expression::UnaryExpression(u) => {
                let operand = match u.operator.as_str() {
                    "!" => Ctx::Truthy,
                    "-" | "+" | "~" | "typeof" | "void" => Ctx::Render,
                    _ => Ctx::Refuse,
                };
                self.expr(&u.argument, operand);
            }
            Expression::LogicalExpression(l) => {
                let operand = if truthy { Ctx::Truthy } else { Ctx::Refuse };
                self.expr(&l.left, operand.clone());
                self.expr(&l.right, operand);
            }
            Expression::ConditionalExpression(c) => {
                self.expr(&c.test, Ctx::Truthy);
                let arm = if truthy { Ctx::Truthy } else { Ctx::Refuse };
                self.expr(&c.consequent, arm.clone());
                self.expr(&c.alternate, arm);
            }
            Expression::SequenceExpression(s) => {
                if let Some((last, init)) = s.expressions.split_last() {
                    for e in init {
                        self.expr(e, Ctx::Truthy);
                    }
                    self.expr(last, ctx);
                }
            }
            Expression::UpdateExpression(u) => match unwrap(&u.argument) {
                Expression::Identifier(name) => {
                    if let Some(node) = self.target(name) {
                        // `x++` keeps a whole number whole (A-39).
                        self.bool_taint(node.clone());
                        if let Some(key) = binding_key(&node) {
                            self.facts
                                .integral
                                .writes
                                .entry(key)
                                .or_default()
                                .push(Integral::Yes);
                        }
                    }
                }
                other => self.expr(other, Ctx::Refuse),
            },
            Expression::CallExpression(call) => self.call(call),
            Expression::MemberExpression(m) => {
                self.expr(&m.object, Ctx::Refuse);
                if let Some(index) = &m.computed_index {
                    self.expr(index, Ctx::Refuse);
                }
            }
            Expression::ArrayExpression(a) => {
                for element in &a.elements {
                    match element {
                        Some(ExpressionOrSpread::Expression(e)) => self.expr(e, Ctx::Refuse),
                        Some(ExpressionOrSpread::Spread(s)) => self.expr(&s.argument, Ctx::Refuse),
                        Some(ExpressionOrSpread::Empty) | None => {}
                    }
                }
            }
            Expression::ObjectExpression(o) => {
                for property in &o.properties {
                    self.expr(&property.value, Ctx::Refuse);
                }
            }
            Expression::FunctionExpression(f) => {
                let key = match &f.id {
                    Some(id) => id.clone(),
                    None => self.anonymous_key(),
                };
                let params: Vec<String> = f.params.iter().map(|p| p.name.clone()).collect();
                if let Some(body) = &f.body {
                    self.function(key, false, &params, |w| w.statements(&body.body));
                }
            }
            Expression::ArrowFunctionExpression(a) => {
                let key = match &a.id {
                    Some(id) => id.clone(),
                    None => self.anonymous_key(),
                };
                let params: Vec<String> = a.params.iter().map(|p| p.name.clone()).collect();
                self.function(key, false, &params, |w| w.return_value(Some(&a.body)));
            }
            Expression::ClassExpression(_)
            | Expression::JsxElement(_)
            | Expression::JsxFragment(_) => self.opaque_site(),
            Expression::NewExpression(e) => {
                self.expr(&e.callee, Ctx::Refuse);
                for arg in &e.args {
                    self.expr(arg, Ctx::Refuse);
                }
            }
            Expression::TemplateLiteral(t) => {
                for e in &t.expressions {
                    self.expr(e, Ctx::Render);
                }
            }
            Expression::TaggedTemplateExpression(t) => {
                self.expr(&t.tag, Ctx::Refuse);
                for e in &t.template.expressions {
                    self.expr(e, Ctx::Refuse);
                }
            }
            Expression::AssignmentExpression(a) => self.assignment(a, truthy),
            Expression::ChainExpression(c) => self.expr(&c.expression, Ctx::Refuse),
            Expression::DecoratedExpression(d) => self.expr(&d.expression, Ctx::Refuse),
            Expression::AwaitExpression(a) => self.expr(&a.argument, Ctx::Refuse),
            Expression::SpreadElement(s) => self.expr(&s.argument, Ctx::Refuse),
            Expression::RestElement(r) => self.expr(&r.argument, Ctx::Refuse),
            Expression::YieldExpression(y) => {
                if let Some(argument) = &y.argument {
                    self.expr(argument, Ctx::Refuse);
                }
            }
            Expression::ImportExpression(i) => self.expr(&i.source, Ctx::Refuse),
            Expression::OptionalChainExpression(e) => match &*e.inner {
                OptionalChainInner::NonNull { object, .. } => self.expr(object, Ctx::Refuse),
            },
        }
    }

    fn call(&mut self, call: &kali_ast::CallExpression) {
        match unwrap(&call.callee) {
            Expression::MemberExpression(m) if m.computed_index.is_none() => {
                let console = matches!(unwrap(&m.object), Expression::Identifier(n) if n == "console")
                    && m.dot_name()
                        .is_some_and(|name| CONSOLE_METHODS.contains(&name));
                if console {
                    for arg in &call.args {
                        self.expr(arg, Ctx::Render);
                    }
                    return;
                }
                self.expr(&m.object, Ctx::Refuse);
                for arg in &call.args {
                    self.expr(arg, Ctx::Refuse);
                }
            }
            Expression::Identifier(_) => {
                for arg in &call.args {
                    self.expr(arg, Ctx::Refuse);
                }
            }
            callee => {
                self.expr(callee, Ctx::Refuse);
                for arg in &call.args {
                    self.expr(arg, Ctx::Refuse);
                }
            }
        }
    }

    fn assignment(&mut self, a: &kali_ast::AssignmentExpression, truthy: bool) {
        match unwrap(&a.left) {
            Expression::Identifier(name) => {
                let Some(node) = self.target(name) else {
                    self.expr(&a.right, Ctx::Refuse);
                    return;
                };
                if matches!(a.operator, AssignmentOperator::Assign) {
                    self.write(node.clone(), Some(&a.right));
                    self.expr(&a.right, Ctx::Store(node));
                    // The assignment's own value is the right-hand side.
                    if !truthy {
                        let value = self.classify(&a.right);
                        self.use_value(value, Ctx::Refuse);
                    }
                } else {
                    // `x += v` is whole when `x` and `v` are (A-39); the
                    // boolean solve cannot follow it.
                    self.bool_taint(node.clone());
                    let proof =
                        self.integral(&Expression::AssignmentExpression(Box::new(a.clone())));
                    if let Some(key) = binding_key(&node) {
                        self.facts
                            .integral
                            .writes
                            .entry(key)
                            .or_default()
                            .push(proof);
                    }
                    self.expr(&a.right, Ctx::Refuse);
                }
            }
            Expression::MemberExpression(m) => {
                self.expr(&m.object, Ctx::Refuse);
                if let Some(index) = &m.computed_index {
                    self.expr(index, Ctx::Refuse);
                }
                self.expr(&a.right, Ctx::Refuse);
            }
            left => {
                self.expr(left, Ctx::Refuse);
                self.expr(&a.right, Ctx::Refuse);
            }
        }
    }
}

#[cfg(test)]
#[path = "values_tests.rs"]
mod values_tests;
