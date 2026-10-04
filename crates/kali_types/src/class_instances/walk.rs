//! An exhaustive mutable AST walker that reports each expression's position
//! (`Pos`) and the enclosing function frames (`Cx`).
//!
//! The recursion mirrors `kali_cli`'s `name_anon_functions.rs`; every match is
//! exhaustive with no `_ =>` arm, so a new AST variant is a compile error here.

use kali_ast::{
    AssignmentOperator, BlockStatement, ClassBody, ClassDeclaration, ClassExpression, Expression,
    ExpressionOrSpread, ExportDefaultDeclaration, ForInLefthand, ForInit, ForOfLefthand,
    FunctionDeclaration, JsxAttributeItem, JsxAttributeValue, JsxChild, JsxElement,
    JsxExpressionContainer, JsxFragment, MethodDefinition, OptionalChainInner, Statement,
    VariableDeclaration, VariableDeclarator,
};

pub(crate) type FnKey = String;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FrameKind {
    Program,
    Function { is_async_or_generator: bool, is_expression: bool },
    /// An expression-bodied or block-bodied arrow (`FunctionExpression { is_arrow: true }`).
    Arrow,
    Method { class: String, method: String, is_static: bool },
    Constructor { class: String },
    /// The class's field initializers (they run with `this` bound to the instance).
    FieldInit { class: String },
}

#[derive(Clone, Debug)]
pub(crate) struct Frame {
    pub key: FnKey,
    pub kind: FrameKind,
}

/// The frames enclosing the node being visited, innermost last.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cx {
    pub frames: Vec<Frame>,
}

impl Cx {
    /// The innermost frame's key.
    pub(crate) fn key(&self) -> &str {
        self.frames.last().map(|f| f.key.as_str()).unwrap_or("")
    }

    /// `"<key>/<name>"`, or `"<name>"` at the program frame.
    pub(crate) fn child_key(&self, name: &str) -> FnKey {
        match self.key() {
            "" => name.to_string(),
            parent => format!("{parent}/{name}"),
        }
    }

    /// The class `this` refers to here (spec §3.2): arrows are skipped; an
    /// instance method, constructor or field initializer gives its class.
    pub(crate) fn this_class(&self) -> Option<&str> {
        for frame in self.frames.iter().rev() {
            match &frame.kind {
                FrameKind::Arrow => continue,
                FrameKind::Method { class, is_static: false, .. }
                | FrameKind::Constructor { class }
                | FrameKind::FieldInit { class } => return Some(class),
                _ => return None,
            }
        }
        None
    }
}

/// Where an expression sits in its parent (spec §3.3's allowlist).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Pos {
    BindingInit(String),
    BindingAssign(String),
    CallArg { callee: Expression, index: usize },
    Return,
    MemberObject { property: Option<String>, call: bool, write: bool },
    Discarded,
    Callee,
    NewCallee,
    AssignTarget,
    UpdateTarget,
    Other(&'static str),
}

pub(crate) trait Visitor {
    /// Before the walker descends into `expr`'s children. May replace `*expr`;
    /// the walker then descends into the replacement.
    fn expr(&mut self, _expr: &mut Expression, _pos: &Pos, _cx: &Cx) {}
    /// Before the walker descends into a statement list. May splice it.
    fn stmts(&mut self, _list: &mut Vec<Statement>, _cx: &Cx) {}
    fn enter_frame(&mut self, _cx: &Cx, _params: &[String]) {}
    fn class_decl(&mut self, _class: &ClassDeclaration, _exported_default: bool, _cx: &Cx) {}
    fn class_expr(&mut self, _class: &ClassExpression, _cx: &Cx) {}
    fn var_declarator(&mut self, _d: &VariableDeclarator, _kind: &str, _cx: &Cx) {}
    fn function_decl(&mut self, _f: &FunctionDeclaration, _cx: &Cx) {}
    fn catch_param(&mut self, _name: &str, _cx: &Cx) {}
    fn export_specifier(&mut self, _local: &str) {}
}

pub(crate) fn walk(statements: &mut Vec<Statement>, visitor: &mut dyn Visitor) {
    let mut walker = Walker {
        visitor,
        cx: Cx { frames: vec![Frame { key: String::new(), kind: FrameKind::Program }] },
    };
    walker.list(statements);
}

pub(crate) const ANON: &str = "<anon>";

struct Walker<'v> {
    visitor: &'v mut dyn Visitor,
    cx: Cx,
}

impl Walker<'_> {
    /// Push a frame, announce it, run `body`, pop it.
    fn in_frame(
        &mut self,
        name: &str,
        kind: FrameKind,
        params: &[String],
        body: impl FnOnce(&mut Self),
    ) {
        let key = self.cx.child_key(name);
        self.cx.frames.push(Frame { key, kind });
        self.visitor.enter_frame(&self.cx, params);
        body(self);
        self.cx.frames.pop();
    }

    fn list(&mut self, list: &mut Vec<Statement>) {
        self.visitor.stmts(list, &self.cx);
        for statement in list.iter_mut() {
            self.statement(statement);
        }
    }

    fn block(&mut self, block: &mut BlockStatement) {
        self.list(&mut block.body);
    }

    fn var_decl(&mut self, decl: &mut VariableDeclaration) {
        for declarator in &mut decl.declarations {
            self.visitor.var_declarator(declarator, &decl.kind, &self.cx);
            if let Some(init) = &mut declarator.init {
                self.expr(init, Pos::BindingInit(declarator.id.clone()));
            }
        }
    }

    fn function_decl(&mut self, function: &mut FunctionDeclaration) {
        self.visitor.function_decl(function, &self.cx);
        let name = if function.name.is_empty() { ANON } else { function.name.as_str() };
        let kind = FrameKind::Function {
            is_async_or_generator: function.is_async || function.generator,
            is_expression: false,
        };
        let params = function.params.clone();
        let body = &mut function.body;
        self.in_frame(name, kind, &params, |w| w.block(body));
    }

    fn class_body(&mut self, class: &str, body: &mut ClassBody) {
        if body.fields.iter().any(|f| f.value.is_some()) {
            let kind = FrameKind::FieldInit { class: class.to_string() };
            let fields = &mut body.fields;
            self.in_frame(&format!("{class}#fields"), kind, &[], |w| {
                for field in fields.iter_mut() {
                    if let Some(value) = &mut field.value {
                        w.expr(value, Pos::Other("a field initializer"));
                    }
                }
            });
        }
        for method in &mut body.methods {
            self.method(class, method);
        }
    }

    fn method(&mut self, class: &str, method: &mut MethodDefinition) {
        let kind = if method.name == "constructor" && !method.is_static {
            FrameKind::Constructor { class: class.to_string() }
        } else {
            FrameKind::Method {
                class: class.to_string(),
                method: method.name.clone(),
                is_static: method.is_static,
            }
        };
        let key = format!("{class}#{}", method.name);
        let params = method.params.clone();
        let body = &mut method.body;
        self.in_frame(&key, kind, &params, |w| {
            if let Some(body) = body {
                w.block(body);
            }
        });
    }

    fn statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::ExpressionStatement(stmt) => self.expr(&mut stmt.expression, Pos::Discarded),
            Statement::BreakStatement(_) => {}
            Statement::ContinueStatement(_) => {}
            Statement::WithStatement(stmt) => {
                self.expr(&mut stmt.object, Pos::Other("a with object"));
                self.statement(&mut stmt.body);
            }
            Statement::ReturnStatement(stmt) => {
                if let Some(argument) = &mut stmt.argument {
                    self.expr(argument, Pos::Return);
                }
            }
            Statement::LabeledStatement(stmt) => self.statement(&mut stmt.body),
            Statement::IfStatement(stmt) => {
                self.expr(&mut stmt.test, Pos::Other("a condition"));
                self.block(&mut stmt.consequent);
                if let Some(alternate) = &mut stmt.alternate {
                    self.block(alternate);
                }
            }
            Statement::SwitchStatement(stmt) => {
                self.expr(&mut stmt.discriminant, Pos::Other("a switch operand"));
                for case in &mut stmt.cases {
                    if let Some(test) = &mut case.test {
                        self.expr(test, Pos::Other("a switch operand"));
                    }
                    self.list(&mut case.consequent);
                }
            }
            Statement::ThrowStatement(stmt) => {
                self.expr(&mut stmt.argument, Pos::Other("a thrown value"))
            }
            Statement::TryStatement(stmt) => {
                self.block(&mut stmt.block);
                if let Some(handler) = &mut stmt.handler {
                    self.visitor.catch_param(&handler.param, &self.cx);
                    self.block(&mut handler.body);
                }
                if let Some(finalizer) = &mut stmt.finalizer {
                    self.block(finalizer);
                }
            }
            Statement::DebuggerStatement(_) => {}
            Statement::BlockStatement(stmt) => self.block(stmt),
            Statement::ForStatement(stmt) => {
                match &mut stmt.init {
                    Some(ForInit::VariableDeclaration(decl)) => self.var_decl(decl),
                    Some(ForInit::Expression(expr)) => self.expr(expr, Pos::Discarded),
                    None => {}
                }
                if let Some(test) = &mut stmt.test {
                    self.expr(test, Pos::Other("a condition"));
                }
                if let Some(update) = &mut stmt.update {
                    self.expr(update, Pos::Discarded);
                }
                self.block(&mut stmt.body);
            }
            Statement::ForInStatement(stmt) => {
                match &mut stmt.left {
                    ForInLefthand::VariableDeclaration(decl) => self.var_decl(decl),
                    ForInLefthand::Expression(expr) => {
                        self.expr(expr, Pos::Other("a loop target"))
                    }
                }
                self.expr(&mut stmt.right, Pos::Other("a loop iterable"));
                self.statement(&mut stmt.body);
            }
            Statement::ForOfStatement(stmt) => {
                match &mut stmt.left {
                    ForOfLefthand::VariableDeclaration(decl) => self.var_decl(decl),
                    ForOfLefthand::Expression(expr) => {
                        self.expr(expr, Pos::Other("a loop target"))
                    }
                }
                self.expr(&mut stmt.right, Pos::Other("a loop iterable"));
                self.statement(&mut stmt.body);
            }
            Statement::WhileStatement(stmt) => {
                self.expr(&mut stmt.test, Pos::Other("a condition"));
                self.block(&mut stmt.body);
            }
            Statement::DoWhileStatement(stmt) => {
                self.block(&mut stmt.body);
                self.expr(&mut stmt.test, Pos::Other("a condition"));
            }
            Statement::FunctionDeclaration(function) => self.function_decl(function),
            Statement::ClassDeclaration(class) => {
                self.visitor.class_decl(class, false, &self.cx);
                self.class_body(&class.name, &mut class.body);
            }
            Statement::VariableDeclaration(decl) => self.var_decl(decl),
            Statement::ImportDeclaration(_) => {}
            Statement::ExportAll(_) => {}
            Statement::ExportNamed(export) => {
                for specifier in &export.specifiers {
                    self.visitor.export_specifier(&specifier.local);
                }
            }
            Statement::ExportDefault(export) => match export {
                ExportDefaultDeclaration::Expression(expr) => {
                    self.expr(expr, Pos::Other("an exported value"))
                }
                ExportDefaultDeclaration::FunctionDeclaration(function) => {
                    self.function_decl(function)
                }
                ExportDefaultDeclaration::ClassDeclaration(class) => {
                    self.visitor.class_decl(class, true, &self.cx);
                    self.class_body(&class.name, &mut class.body);
                }
            },
            Statement::EnumDeclaration(decl) => {
                for member in &mut decl.members {
                    if let Some(value) = &mut member.value {
                        self.expr(value, Pos::Other("an enum value"));
                    }
                }
            }
            Statement::TypeAliasDeclaration(_) => {}
            Statement::InterfaceDeclaration(_) => {}
        }
    }

    fn expression_or_spread(&mut self, element: &mut ExpressionOrSpread) {
        match element {
            ExpressionOrSpread::Expression(expr) => {
                self.expr(expr, Pos::Other("an array element"))
            }
            ExpressionOrSpread::Spread(spread) => {
                self.expr(&mut spread.argument, Pos::Other("a spread operand"))
            }
            ExpressionOrSpread::Empty => {}
        }
    }

    fn expr(&mut self, expr: &mut Expression, pos: Pos) {
        self.visitor.expr(expr, &pos, &self.cx);
        const OPERAND: Pos = Pos::Other("an operand");
        match expr {
            Expression::ParenthesizedExpression(inner) => self.expr(&mut inner.expression, pos),
            Expression::AwaitExpression(e) => {
                self.expr(&mut e.argument, Pos::Other("an awaited or yielded value"))
            }
            Expression::ImportExpression(e) => self.expr(&mut e.source, OPERAND),
            Expression::Identifier(_) => {}
            Expression::Literal(_) => {}
            Expression::BinaryExpression(binary) => {
                self.expr(&mut binary.left, Pos::Other("an operand of a binary operator"));
                self.expr(&mut binary.right, Pos::Other("an operand of a binary operator"));
            }
            Expression::UnaryExpression(unary) => {
                self.expr(&mut unary.argument, Pos::Other("an operand of a unary operator"))
            }
            Expression::CallExpression(call) => {
                self.expr(&mut call.callee, Pos::Callee);
                let callee = call.callee.clone();
                for (index, arg) in call.args.iter_mut().enumerate() {
                    self.expr(arg, Pos::CallArg { callee: callee.clone(), index });
                }
            }
            Expression::MemberExpression(member) => {
                let (call, write) = match pos {
                    Pos::Callee => (true, false),
                    Pos::AssignTarget | Pos::UpdateTarget => (false, true),
                    _ => (false, false),
                };
                let property = member.property.clone();
                self.expr(&mut member.object, Pos::MemberObject { property, call, write });
                if let Some(index) = &mut member.computed_index {
                    self.expr(index, Pos::Other("a computed key"));
                }
            }
            Expression::ArrayExpression(array) => {
                for element in array.elements.iter_mut().flatten() {
                    self.expression_or_spread(element);
                }
            }
            Expression::ObjectExpression(object) => {
                for property in &mut object.properties {
                    self.expr(&mut property.value, Pos::Other("an object-literal value"));
                }
            }
            Expression::FunctionExpression(function) => {
                let name = function.id.clone().unwrap_or_else(|| ANON.to_string());
                let kind = if function.is_arrow {
                    FrameKind::Arrow
                } else {
                    FrameKind::Function {
                        is_async_or_generator: function.is_async || function.generator,
                        is_expression: true,
                    }
                };
                let params: Vec<String> = function.params.iter().map(|p| p.name.clone()).collect();
                let body = &mut function.body;
                self.in_frame(&name, kind, &params, |w| {
                    if let Some(body) = body {
                        w.block(body);
                    }
                });
            }
            Expression::ArrowFunctionExpression(arrow) => {
                let name = arrow.id.clone().unwrap_or_else(|| ANON.to_string());
                let params: Vec<String> = arrow.params.iter().map(|p| p.name.clone()).collect();
                let body = &mut arrow.body;
                self.in_frame(&name, FrameKind::Arrow, &params, |w| w.expr(body, Pos::Return));
            }
            Expression::ClassExpression(class) => {
                self.visitor.class_expr(class, &self.cx);
                let name = class.id.clone().unwrap_or_else(|| ANON.to_string());
                self.class_body(&name, &mut class.body);
            }
            Expression::NewExpression(new_expr) => {
                self.expr(&mut new_expr.callee, Pos::NewCallee);
                for arg in &mut new_expr.args {
                    self.expr(arg, Pos::Other("an argument to `new`"));
                }
            }
            Expression::MetaProperty(_) => {}
            Expression::TemplateLiteral(template) => {
                for expression in &mut template.expressions {
                    self.expr(expression, Pos::Other("a template operand"));
                }
            }
            Expression::TaggedTemplateExpression(tagged) => {
                self.expr(&mut tagged.tag, Pos::Other("a template operand"));
                for expression in &mut tagged.template.expressions {
                    self.expr(expression, Pos::Other("a template operand"));
                }
            }
            Expression::UpdateExpression(update) => {
                self.expr(&mut update.argument, Pos::UpdateTarget)
            }
            Expression::AssignmentExpression(assignment) => {
                let right_pos = match (&assignment.operator, &assignment.left) {
                    (AssignmentOperator::Assign, Expression::Identifier(name)) => {
                        Pos::BindingAssign(name.clone())
                    }
                    (AssignmentOperator::Assign, Expression::MemberExpression(_)) => {
                        Pos::Other("the value of a field write")
                    }
                    _ => Pos::Other("an operand"),
                };
                self.expr(&mut assignment.left, Pos::AssignTarget);
                self.expr(&mut assignment.right, right_pos);
            }
            Expression::LogicalExpression(logical) => {
                self.expr(&mut logical.left, Pos::Other("a logical or conditional operand"));
                self.expr(&mut logical.right, Pos::Other("a logical or conditional operand"));
            }
            Expression::ConditionalExpression(conditional) => {
                self.expr(&mut conditional.test, Pos::Other("a logical or conditional operand"));
                self.expr(
                    &mut conditional.consequent,
                    Pos::Other("a logical or conditional operand"),
                );
                self.expr(
                    &mut conditional.alternate,
                    Pos::Other("a logical or conditional operand"),
                );
            }
            Expression::SequenceExpression(sequence) => {
                for expression in &mut sequence.expressions {
                    self.expr(expression, Pos::Other("a sequence operand"));
                }
            }
            Expression::YieldExpression(yield_expr) => {
                if let Some(argument) = &mut yield_expr.argument {
                    self.expr(argument, Pos::Other("an awaited or yielded value"));
                }
            }
            Expression::OptionalChainExpression(chain) => match chain.inner.as_mut() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.expr(object, Pos::Other("an optional-chain operand"))
                }
            },
            Expression::ChainExpression(chain) => self.expr(&mut chain.expression, pos),
            Expression::SpreadElement(spread) => {
                self.expr(&mut spread.argument, Pos::Other("a spread operand"))
            }
            Expression::RestElement(rest) => {
                self.expr(&mut rest.argument, Pos::Other("a spread operand"))
            }
            Expression::DecoratedExpression(decorated) => {
                self.expr(&mut decorated.expression, OPERAND)
            }
            Expression::JsxElement(element) => self.jsx_element(element),
            Expression::JsxFragment(fragment) => self.jsx_fragment(fragment),
            Expression::JsxEmptyExpression => {}
            Expression::TypeAssertion(assertion) => self.expr(&mut assertion.expression, pos),
            Expression::SatisfiesExpression(satisfies) => {
                self.expr(&mut satisfies.expression, pos)
            }
            Expression::ThisExpression => {}
            Expression::SuperExpression => {}
            Expression::PrivateIdentifier(_) => {}
            Expression::BigIntLiteral(_) => {}
        }
    }

    fn jsx_element(&mut self, element: &mut JsxElement) {
        for attribute in &mut element.opening_element.attributes {
            match attribute {
                JsxAttributeItem::JsxAttribute(attribute) => match &mut attribute.value {
                    JsxAttributeValue::String(_) => {}
                    JsxAttributeValue::JsxElement(element) => self.jsx_element(element),
                    JsxAttributeValue::JsxExpression(container) => self.jsx_container(container),
                },
                JsxAttributeItem::JsxSpreadAttribute(spread) => {
                    self.expr(&mut spread.argument, Pos::Other("a JSX operand"))
                }
            }
        }
        for child in &mut element.children {
            self.jsx_child(child);
        }
    }

    fn jsx_fragment(&mut self, fragment: &mut JsxFragment) {
        for child in &mut fragment.children {
            self.jsx_child(child);
        }
    }

    fn jsx_container(&mut self, container: &mut JsxExpressionContainer) {
        if let Some(expression) = &mut container.expression {
            self.expr(expression, Pos::Other("a JSX operand"));
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
