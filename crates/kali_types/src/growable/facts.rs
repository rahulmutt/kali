//! Growable-runtime-arrays spec §3.1: the syntactic facts the growable
//! property is solved over. One exhaustive walk records, per function, every
//! array value flow (edges), every origin (a literal binding, an allocation,
//! a literal or allocation written directly), every length or element
//! mutation (demands), every non-array write, every occurrence of an array
//! value in a position (uses), every `for-of` frame and every stored element
//! value. It decides nothing: `flow::solve` and `positions` do.

use std::collections::BTreeMap;

use kali_ast::{
    ArrayExpression, AssignmentExpression, AssignmentOperator, BlockStatement, CallExpression,
    Expression, ExpressionOrSpread, ForInLefthand, ForInit, ForOfLefthand, ForOfStatement,
    LiteralValue, MemberExpression, OptionalChainInner, Statement, VariableDeclaration,
};

use super::flow::{
    CallFact, ElementValue, GrowFacts, GrowNode, LoopFacts, TempKind, Use, UseKind, TOP_LEVEL,
};

/// What the walk needs from repr inference's Phase A and A2.
pub(crate) struct WalkContext<'a> {
    /// `is_declared(func, name)`: `func` declares `name` itself (a parameter
    /// or a local), the `ReprInfer::is_locally_declared` fact.
    pub(crate) is_declared: &'a dyn Fn(&str, &str) -> bool,
    /// `resolve_callee(site, name)`: the declared function a bare call
    /// `name(…)` made in `site` reaches, if any.
    pub(crate) resolve_callee: &'a dyn Fn(&str, &str) -> Option<String>,
    /// Parameter names of every declared function, by key.
    pub(crate) params: &'a BTreeMap<String, Vec<String>>,
}

pub(crate) fn collect_facts(statements: &[Statement], ctx: &WalkContext<'_>) -> GrowFacts {
    let mut walker = Walker {
        ctx,
        facts: GrowFacts::default(),
        frames: vec![TOP_LEVEL.to_string()],
        loops: Vec::new(),
        anonymous: 0,
    };
    walker.statements(statements);
    walker.facts
}

const CONSOLE_METHODS: &[&str] = &["log", "error", "warn", "info", "debug"];

enum Resolved {
    Local(GrowNode),
    Captured(GrowNode),
    Module(GrowNode),
    Unknown,
}

struct Walker<'w, 'c> {
    ctx: &'w WalkContext<'c>,
    facts: GrowFacts,
    /// Function keys, outermost (`_start`) first; never empty.
    frames: Vec<String>,
    /// Indices into `facts.loops` of the active `for-of` frames of the
    /// CURRENT function (saved and cleared on entering a nested function).
    loops: Vec<usize>,
    anonymous: usize,
}

fn unwrap(expr: &Expression) -> &Expression {
    match expr {
        Expression::ParenthesizedExpression(inner) => unwrap(&inner.expression),
        Expression::TypeAssertion(inner) => unwrap(&inner.expression),
        Expression::SatisfiesExpression(inner) => unwrap(&inner.expression),
        other => other,
    }
}

fn is_allocation(expr: &Expression) -> bool {
    crate::resolve::expression::expression_is_array_allocation(expr)
}

/// Mirrors `repr_infer::is_boolean_valued_init` (captured-bindings A-2.1):
/// kali stores such a value as `1`/`0`, so it cannot be an element.
fn is_boolean_valued(expr: &Expression) -> bool {
    match unwrap(expr) {
        Expression::Literal(LiteralValue::Boolean(_)) => true,
        Expression::BinaryExpression(binary) => matches!(
            binary.operator.as_str(),
            "==" | "===" | "!=" | "!==" | "<" | ">" | "<=" | ">="
        ),
        Expression::UnaryExpression(unary) => unary.operator == "!",
        _ => false,
    }
}

fn element_value(expr: &Expression) -> ElementValue {
    match unwrap(expr) {
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::BigIntLiteral(_)
        | Expression::SpreadElement(_)
        | Expression::Literal(LiteralValue::Null)
        | Expression::Literal(LiteralValue::Regex { .. }) => ElementValue::Unsupported,
        Expression::Identifier(name) if name == "undefined" => ElementValue::Unsupported,
        Expression::Identifier(name) => ElementValue::Identifier(name.clone()),
        other if is_boolean_valued(other) => ElementValue::Unsupported,
        _ => ElementValue::Other,
    }
}

impl Walker<'_, '_> {
    fn site(&self) -> &str {
        self.frames
            .last()
            .expect("the module frame is never popped")
    }

    fn binding(&self, name: &str) -> GrowNode {
        GrowNode::Binding(self.site().to_string(), name.to_string())
    }

    fn temp(&mut self, kind: TempKind) -> GrowNode {
        let n = self.facts.temps;
        self.facts.temps += 1;
        let site = self.site().to_string();
        self.facts.temp_kinds.insert(n, (kind, site));
        GrowNode::Temp(n)
    }

    fn edge(&mut self, a: GrowNode, b: GrowNode) {
        self.facts.edges.push((a, b));
    }

    fn record(&mut self, node: &GrowNode, kind: UseKind) {
        let site = self.site().to_string();
        self.facts.uses.push(Use {
            node: node.clone(),
            site,
            kind,
        });
    }

    fn anonymous_key(&mut self) -> String {
        self.anonymous += 1;
        format!("<anonymous {}>", self.anonymous)
    }

    fn resolve(&self, name: &str) -> Resolved {
        let site = self.site();
        if (self.ctx.is_declared)(site, name) {
            return Resolved::Local(GrowNode::Binding(site.to_string(), name.to_string()));
        }
        for func in self.frames.iter().rev().skip(1) {
            if (self.ctx.is_declared)(func, name) {
                let node = GrowNode::Binding(func.clone(), name.to_string());
                return if func == TOP_LEVEL {
                    Resolved::Module(node)
                } else {
                    Resolved::Captured(node)
                };
            }
        }
        Resolved::Unknown
    }

    /// The node `name` denotes in this function. A captured or module-scope
    /// name records that fact and yields `None`: its flows are not modelled,
    /// because `positions` refuses it whenever it is growable.
    fn name_node(&mut self, name: &str) -> Option<GrowNode> {
        match self.resolve(name) {
            Resolved::Local(node) => Some(node),
            Resolved::Captured(node) => {
                self.record(&node, UseKind::Captured);
                None
            }
            Resolved::Module(node) => {
                self.record(&node, UseKind::ModuleRead);
                None
            }
            Resolved::Unknown => None,
        }
    }

    fn use_name(&mut self, name: &str, kind: UseKind) -> Option<GrowNode> {
        let node = self.name_node(name)?;
        self.record(&node, kind);
        Some(node)
    }

    fn opaque(&mut self) {
        let stack = self.frames.clone();
        self.facts.opaque_sites.push(stack);
    }

    fn mutation(&mut self, receiver: &GrowNode) {
        self.facts.demands.insert(receiver.clone());
        for &frame in &self.loops {
            self.facts.loops[frame].mutations.push(receiver.clone());
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

    fn statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::ExpressionStatement(s) => self.discarded(&s.expression),
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
            | Statement::EnumDeclaration(_) => self.opaque(),
            Statement::ReturnStatement(s) => self.return_value(s.argument.as_ref()),
            Statement::LabeledStatement(s) => self.statement(&s.body),
            Statement::IfStatement(s) => {
                self.expr(&s.test);
                self.block(&s.consequent);
                if let Some(alternate) = &s.alternate {
                    self.block(alternate);
                }
            }
            Statement::SwitchStatement(s) => {
                self.expr(&s.discriminant);
                for case in &s.cases {
                    if let Some(test) = &case.test {
                        self.expr(test);
                    }
                    self.statements(&case.consequent);
                }
            }
            Statement::ThrowStatement(s) => self.expr(&s.argument),
            Statement::TryStatement(s) => {
                self.block(&s.block);
                if let Some(handler) = &s.handler {
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
                    Some(ForInit::Expression(e)) => self.discarded(e),
                    None => {}
                }
                if let Some(test) = &s.test {
                    self.expr(test);
                }
                if let Some(update) = &s.update {
                    self.discarded(update);
                }
                self.block(&s.body);
            }
            Statement::ForInStatement(s) => {
                match &s.left {
                    ForInLefthand::VariableDeclaration(d) => self.loop_variables(d),
                    ForInLefthand::Expression(e) => self.expr(e),
                }
                // A growable array iterated with `for-in` is a plain use.
                self.expr(&s.right);
                self.statement(&s.body);
            }
            Statement::ForOfStatement(s) => self.for_of(s),
            Statement::WhileStatement(s) => {
                self.expr(&s.test);
                self.block(&s.body);
            }
            Statement::DoWhileStatement(s) => {
                self.block(&s.body);
                self.expr(&s.test);
            }
            Statement::FunctionDeclaration(decl) => {
                self.function(decl.name.clone(), &decl.body.body)
            }
            Statement::VariableDeclaration(d) => self.variable_declaration(d),
        }
    }

    /// The declaration kind is ignored (A-19).
    fn variable_declaration(&mut self, d: &VariableDeclaration) {
        for declarator in &d.declarations {
            self.declarator(&declarator.id, declarator.init.as_ref());
        }
    }

    fn declarator(&mut self, id: &str, init: Option<&Expression>) {
        let node = self.binding(id);
        let Some(init) = init else {
            self.facts.non_array_writes.insert(node);
            return;
        };
        if let Expression::ArrayExpression(array) = unwrap(init) {
            self.facts.literal_origins.insert(node.clone());
            for element in &array.elements {
                match element {
                    Some(ExpressionOrSpread::Expression(e)) => {
                        self.facts
                            .element_values
                            .push((node.clone(), element_value(e)));
                        self.expr(e);
                    }
                    Some(ExpressionOrSpread::Spread(s)) => {
                        self.facts
                            .element_values
                            .push((node.clone(), ElementValue::Unsupported));
                        self.expr(&s.argument);
                    }
                    Some(ExpressionOrSpread::Empty) | None => {
                        self.facts
                            .element_values
                            .push((node.clone(), ElementValue::Unsupported));
                    }
                }
            }
            return;
        }
        if is_allocation(unwrap(init)) {
            self.facts.plain_origins.insert(node);
            self.allocation_operands(init);
            return;
        }
        match self.value(init) {
            Some(n) => self.edge(node, n),
            None => {
                self.facts.non_array_writes.insert(node);
            }
        }
    }

    fn allocation_operands(&mut self, alloc: &Expression) {
        for arg in crate::array_return::allocation_length_args(alloc) {
            self.expr(arg);
        }
        if let Some(v) = crate::array_return::fill_value(alloc) {
            self.expr(v);
        }
    }

    /// A loop variable holds elements or keys, never an array.
    fn loop_variables(&mut self, d: &VariableDeclaration) {
        for declarator in &d.declarations {
            let node = self.binding(&declarator.id);
            self.facts.non_array_writes.insert(node);
        }
    }

    fn for_of(&mut self, s: &ForOfStatement) {
        match &s.left {
            ForOfLefthand::VariableDeclaration(d) => self.loop_variables(d),
            ForOfLefthand::Expression(e) => self.expr(e),
        }
        let mut pushed = false;
        if let Some(node) = self.receiver(&s.right) {
            self.record(&node, UseKind::ForOf);
            let site = self.site().to_string();
            self.facts.loops.push(LoopFacts {
                iterable: node,
                site,
                mutations: Vec::new(),
                calls: Vec::new(),
            });
            self.loops.push(self.facts.loops.len() - 1);
            pushed = true;
        }
        self.statement(&s.body);
        if pushed {
            self.loops.pop();
        }
    }

    fn function(&mut self, key: String, body: &[Statement]) {
        let saved = std::mem::take(&mut self.loops);
        self.frames.push(key.clone());
        self.statements(body);
        if crate::array_return::body_falls_off_end(body) {
            self.facts.non_array_writes.insert(GrowNode::Return(key));
        }
        self.frames.pop();
        self.loops = saved;
    }

    fn arrow(&mut self, key: String, body: &Expression) {
        let saved = std::mem::take(&mut self.loops);
        self.frames.push(key);
        self.return_value(Some(body));
        self.frames.pop();
        self.loops = saved;
    }

    fn return_value(&mut self, arg: Option<&Expression>) {
        let site = GrowNode::Return(self.site().to_string());
        match arg.and_then(|a| self.value(a)) {
            Some(n) => self.edge(site, n),
            None => {
                self.facts.non_array_writes.insert(site);
            }
        }
    }

    /// A value nobody reads.
    fn discarded(&mut self, expr: &Expression) {
        match unwrap(expr) {
            Expression::Identifier(_) => {}
            Expression::CallExpression(c) => {
                let _ = self.call(c);
            }
            Expression::AssignmentExpression(a) => {
                let _ = self.assignment(a);
            }
            _ => self.expr(expr),
        }
    }

    /// The plain-context walk.
    fn expr(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                self.use_name(name, UseKind::Plain);
            }
            Expression::Literal(_)
            | Expression::BigIntLiteral(_)
            | Expression::MetaProperty(_)
            | Expression::JsxEmptyExpression
            | Expression::ThisExpression
            | Expression::SuperExpression
            | Expression::PrivateIdentifier(_) => {}
            Expression::BinaryExpression(b) => {
                self.expr(&b.left);
                self.expr(&b.right);
            }
            Expression::UnaryExpression(u) => self.expr(&u.argument),
            Expression::LogicalExpression(l) => {
                self.expr(&l.left);
                self.expr(&l.right);
            }
            Expression::ConditionalExpression(c) => {
                self.expr(&c.test);
                self.expr(&c.consequent);
                self.expr(&c.alternate);
            }
            Expression::SequenceExpression(s) => {
                for e in &s.expressions {
                    self.expr(e);
                }
            }
            Expression::ParenthesizedExpression(p) => self.expr(&p.expression),
            Expression::TypeAssertion(t) => self.expr(&t.expression),
            Expression::SatisfiesExpression(t) => self.expr(&t.expression),
            Expression::ChainExpression(c) => self.expr(&c.expression),
            Expression::DecoratedExpression(d) => self.expr(&d.expression),
            Expression::AwaitExpression(a) => self.expr(&a.argument),
            Expression::SpreadElement(s) => self.expr(&s.argument),
            Expression::RestElement(r) => self.expr(&r.argument),
            Expression::YieldExpression(y) => {
                if let Some(argument) = &y.argument {
                    self.expr(argument);
                }
            }
            Expression::ImportExpression(i) => self.expr(&i.source),
            Expression::UpdateExpression(u) => self.expr(&u.argument),
            Expression::CallExpression(call) => {
                if let Some(node) = self.call(call) {
                    self.record(&node, UseKind::Plain);
                }
            }
            Expression::MemberExpression(m) => self.member_read(m),
            Expression::ArrayExpression(a) => self.elements(a),
            Expression::ObjectExpression(o) => {
                for property in &o.properties {
                    self.expr(&property.value);
                }
            }
            Expression::FunctionExpression(f) => {
                let key = match &f.id {
                    Some(id) => id.clone(),
                    None => self.anonymous_key(),
                };
                if let Some(body) = &f.body {
                    self.function(key, &body.body);
                }
            }
            Expression::ArrowFunctionExpression(a) => {
                let key = match &a.id {
                    Some(id) => id.clone(),
                    None => self.anonymous_key(),
                };
                self.arrow(key, &a.body);
            }
            Expression::ClassExpression(_)
            | Expression::JsxElement(_)
            | Expression::JsxFragment(_) => self.opaque(),
            Expression::NewExpression(e) => {
                self.expr(&e.callee);
                for arg in &e.args {
                    self.expr(arg);
                }
            }
            Expression::TemplateLiteral(t) => {
                for e in &t.expressions {
                    self.expr(e);
                }
            }
            Expression::TaggedTemplateExpression(t) => {
                self.expr(&t.tag);
                for e in &t.template.expressions {
                    self.expr(e);
                }
            }
            Expression::AssignmentExpression(a) => {
                if let Some(node) = self.assignment(a) {
                    self.record(&node, UseKind::Plain);
                }
            }
            Expression::OptionalChainExpression(e) => match &*e.inner {
                OptionalChainInner::NonNull { object, .. } => {
                    if let Some(node) = self.receiver(object) {
                        self.record(&node, UseKind::Method("an optional chain `?.`".to_string()));
                    }
                }
            },
        }
    }

    fn elements(&mut self, array: &ArrayExpression) {
        for element in &array.elements {
            match element {
                Some(ExpressionOrSpread::Expression(e)) => self.expr(e),
                Some(ExpressionOrSpread::Spread(s)) => self.expr(&s.argument),
                Some(ExpressionOrSpread::Empty) | None => {}
            }
        }
    }

    fn member_read(&mut self, member: &MemberExpression) {
        if let Some(index) = &member.computed_index {
            // A string key such as `a["length"]` is a plain use.
            let kind = match member.property.as_deref() {
                Some(text) if text.parse::<i64>().is_err() => UseKind::Plain,
                _ => UseKind::IndexRead,
            };
            if let Some(node) = self.receiver(&member.object) {
                self.record(&node, kind);
            }
            self.expr(index);
        } else {
            let kind = if member.property.as_deref() == Some("length") {
                UseKind::LengthRead
            } else {
                UseKind::Plain
            };
            if let Some(node) = self.receiver(&member.object) {
                self.record(&node, kind);
            }
        }
    }

    /// A method/member base: the caller records the use.
    fn receiver(&mut self, expr: &Expression) -> Option<GrowNode> {
        match unwrap(expr) {
            Expression::Identifier(name) => self.name_node(name),
            Expression::CallExpression(call) => self.call(call),
            other => {
                self.expr(other);
                None
            }
        }
    }

    /// The flow-context walk: the node an array value flows from, if any.
    fn value(&mut self, expr: &Expression) -> Option<GrowNode> {
        let expr = unwrap(expr);
        if is_allocation(expr) {
            self.allocation_operands(expr);
            let t = self.temp(TempKind::Allocation);
            self.facts.plain_origins.insert(t.clone());
            return Some(t);
        }
        match expr {
            Expression::Identifier(name) => self.use_name(name, UseKind::Flow),
            Expression::ArrayExpression(a) => {
                self.elements(a);
                Some(self.temp(TempKind::LiteralExpression))
            }
            Expression::CallExpression(c) => self.call(c),
            Expression::ConditionalExpression(c) => {
                self.expr(&c.test);
                Some(self.merge(&[&c.consequent, &c.alternate]))
            }
            Expression::LogicalExpression(l) => Some(self.merge(&[&l.left, &l.right])),
            Expression::SequenceExpression(s) => {
                let (last, init) = s.expressions.split_last()?;
                for e in init {
                    self.discarded(e);
                }
                self.value(last)
            }
            Expression::AssignmentExpression(a) => self.assignment(a),
            other => {
                self.expr(other);
                None
            }
        }
    }

    fn merge(&mut self, arms: &[&Expression]) -> GrowNode {
        let m = self.temp(TempKind::Merge);
        for arm in arms {
            match self.value(arm) {
                Some(n) => self.edge(m.clone(), n),
                None => {
                    self.facts.non_array_writes.insert(m.clone());
                }
            }
        }
        m
    }

    fn call(&mut self, call: &CallExpression) -> Option<GrowNode> {
        match unwrap(&call.callee) {
            Expression::MemberExpression(member) => self.method_call(member, &call.args),
            Expression::Identifier(name) => {
                let site = self.site().to_string();
                if let Some(key) = (self.ctx.resolve_callee)(&site, name) {
                    self.declared_call(&key, &call.args);
                    return Some(GrowNode::Return(key));
                }
                self.use_name(name, UseKind::Plain);
                self.plain_args(&call.args);
                None
            }
            callee => {
                self.expr(callee);
                self.plain_args(&call.args);
                None
            }
        }
    }

    fn plain_args(&mut self, args: &[Expression]) {
        for arg in args {
            self.expr(arg);
        }
    }

    fn declared_call(&mut self, key: &str, args: &[Expression]) {
        let params = self.ctx.params.get(key).cloned().unwrap_or_default();
        let spread = args
            .iter()
            .position(|a| matches!(a, Expression::SpreadElement(_)))
            .unwrap_or(args.len());
        for (index, arg) in args.iter().enumerate() {
            if index >= spread {
                self.expr(arg);
                continue;
            }
            let value = self.value(arg);
            let Some(param) = params.get(index) else {
                continue;
            };
            let target = GrowNode::Binding(key.to_string(), param.clone());
            match value {
                Some(node) => {
                    self.edge(target, node.clone());
                    let fact = CallFact {
                        site: self.site().to_string(),
                        callee: key.to_string(),
                        index,
                        node,
                    };
                    for &frame in &self.loops {
                        self.facts.loops[frame].calls.push(fact.clone());
                    }
                    self.facts.calls.push(fact);
                }
                None => {
                    self.facts.non_array_writes.insert(target);
                }
            }
        }
        // A missing argument, or one a spread hides.
        for param in params.iter().skip(spread) {
            self.facts
                .non_array_writes
                .insert(GrowNode::Binding(key.to_string(), param.clone()));
        }
    }

    fn method_call(&mut self, member: &MemberExpression, args: &[Expression]) -> Option<GrowNode> {
        if member.computed_index.is_none()
            && matches!(unwrap(&member.object), Expression::Identifier(n) if n == "console")
            && member
                .property
                .as_deref()
                .is_some_and(|m| CONSOLE_METHODS.contains(&m))
        {
            for arg in args {
                if let Some(node) = self.receiver(arg) {
                    self.record(&node, UseKind::Console);
                }
            }
            return None;
        }
        let Some(receiver) = self.receiver(&member.object) else {
            if let Some(index) = &member.computed_index {
                self.expr(index);
            }
            self.plain_args(args);
            return None;
        };
        if let Some(index) = &member.computed_index {
            let kind = match member.property.as_deref() {
                Some(m) => UseKind::Method(format!("`[\"{m}\"]()`")),
                None => UseKind::Method("a computed method call".to_string()),
            };
            self.record(&receiver, kind);
            self.expr(index);
            self.plain_args(args);
            return None;
        }
        match member.property.as_deref().unwrap_or_default() {
            "push" => {
                self.record(&receiver, UseKind::Push);
                self.mutation(&receiver);
                for arg in args {
                    self.facts
                        .element_values
                        .push((receiver.clone(), element_value(arg)));
                    self.expr(arg);
                }
                None
            }
            "pop" => {
                self.record(&receiver, UseKind::Pop);
                self.mutation(&receiver);
                self.plain_args(args);
                None
            }
            "join" => {
                self.record(&receiver, UseKind::Join);
                self.plain_args(args);
                None
            }
            "slice" => {
                self.record(&receiver, UseKind::Slice);
                self.plain_args(args);
                let t = self.temp(TempKind::Slice);
                self.edge(t.clone(), receiver);
                Some(t)
            }
            method @ ("indexOf" | "includes") => {
                self.record(
                    &receiver,
                    UseKind::Search {
                        method: method.to_string(),
                        from_index: args.len() > 1,
                    },
                );
                self.plain_args(args);
                None
            }
            other => {
                self.record(&receiver, UseKind::Method(format!("`.{other}()`")));
                self.plain_args(args);
                None
            }
        }
    }

    fn assignment(&mut self, a: &AssignmentExpression) -> Option<GrowNode> {
        let plain = matches!(a.operator, AssignmentOperator::Assign);
        match unwrap(&a.left) {
            Expression::Identifier(name) => {
                if !plain {
                    self.use_name(name, UseKind::Plain);
                    self.expr(&a.right);
                    return None;
                }
                let value = self.value(&a.right);
                let target = self.name_node(name)?;
                match value {
                    Some(n) => self.edge(target.clone(), n),
                    None => {
                        self.facts.non_array_writes.insert(target.clone());
                    }
                }
                Some(target)
            }
            Expression::MemberExpression(m) => {
                if let Some(index) = &m.computed_index {
                    let string_key = m
                        .property
                        .as_deref()
                        .is_some_and(|t| t.parse::<i64>().is_err());
                    if let Some(receiver) = self.receiver(&m.object) {
                        if plain && !string_key {
                            self.record(&receiver, UseKind::IndexWrite);
                            // An index write keeps the length: not a loop mutation.
                            self.facts.demands.insert(receiver.clone());
                            self.facts
                                .element_values
                                .push((receiver, element_value(&a.right)));
                        } else {
                            self.record(&receiver, UseKind::Plain);
                        }
                    }
                    self.expr(index);
                } else {
                    let kind = if m.property.as_deref() == Some("length") {
                        UseKind::LengthWrite
                    } else {
                        UseKind::Plain
                    };
                    if let Some(receiver) = self.receiver(&m.object) {
                        self.record(&receiver, kind);
                    }
                }
                self.expr(&a.right);
                None
            }
            left => {
                self.expr(left);
                self.expr(&a.right);
                None
            }
        }
    }
}

#[cfg(test)]
#[path = "facts_tests.rs"]
mod facts_tests;
