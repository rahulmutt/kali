//! High-level intermediate representation (HIR) for the Kali compiler.
//!
//! This crate provides a deterministic AST-to-HIR lowering layer used by the
//! later MIR/LIR stages. The implementation is intentionally conservative and
//! source-shaped so the phase-1 pipeline can round-trip representative programs
//! without inventing extra semantics.

#[allow(unused_imports)]
use kali_ast::{
    ArrayExpression, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    AwaitExpression, BinaryExpression, BlockStatement, BreakStatement, CallExpression, CatchClause,
    ChainExpression, ClassBody, ClassDeclaration, ClassDeclaration as AstClassDeclaration,
    ClassExpression, ClassExpression as AstClassExpression, ConditionalExpression,
    ContinueStatement, DebuggerStatement, DecoratedExpression, DoWhileStatement, EnumDeclaration,
    EnumMember, ExportAllDeclaration, ExportDeclaration, ExportDefaultDeclaration,
    ExportNamedDeclaration, ExportSpecifier, ExportTypeDeclaration, Expression, ExpressionOrSpread,
    ExpressionStatement, ForInLefthand, ForInStatement, ForInit, ForOfLefthand, ForOfStatement,
    ForStatement, FunctionDeclaration, FunctionExpression, FunctionParam, IfStatement,
    ImportDeclaration, ImportExpression, ImportSpecifier, InterfaceDeclaration, JsxAttributeItem,
    JsxAttributeValue, JsxChild, JsxClosingElement, JsxElement, JsxExpressionContainer,
    JsxFragment, JsxName, JsxOpeningElement, JsxSelfClosingElement, JsxSpreadAttribute,
    LabeledStatement, LiteralValue, LogicalExpression, MemberExpression, MetaProperty,
    MethodDefinition, NewExpression, NodeId, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    ObjectPropertyKind as AstObjectPropertyKind, OptionalChainExpression, OptionalChainInner,
    ParenthesizedExpression, PropertyName, RestElement, ReturnStatement, SatisfiesExpression,
    SatisfiesExpression as AstSatisfiesExpression, SequenceExpression, SpreadElement, Statement,
    SwitchCase, SwitchStatement, TaggedTemplateExpression, TemplateElement, TemplateLiteral,
    ThrowStatement, TryStatement, TypeAliasDeclaration, TypeAssertion, UnaryExpression,
    UpdateExpression, VariableDeclaration, VariableDeclarator, WhileStatement, WithStatement,
    YieldExpression, AST,
};
use kali_common::Span;
use kali_error::diagnostic::Diagnostic;

/// HIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirNodeKind {
    Program,
    Block,
    FunctionDecl,
    ClassDecl,
    VarDecl,
    VarDeclarator,
    ImportDecl,
    ExportDecl,
    TypeDecl,
    InterfaceDecl,
    EnumDecl,
    ExprStmt,
    IfStmt,
    ForStmt,
    ForInStmt,
    ForOfStmt,
    WhileStmt,
    DoWhileStmt,
    SwitchStmt,
    TryStmt,
    ReturnStmt,
    BreakStmt,
    ContinueStmt,
    ThrowStmt,
    DebuggerStmt,
    LabeledStmt,
    WithStmt,
    Ident,
    Literal,
    BinaryExpr,
    LogicalExpr,
    UnaryExpr,
    UpdateExpr,
    CallExpr,
    MemberExpr,
    NewExpr,
    AssignmentExpr,
    ConditionalExpr,
    SequenceExpr,
    ArrayExpr,
    ObjectExpr,
    ObjectProperty,
    FunctionExpr,
    ClassExpr,
    TemplateLiteral,
    OptionalChain,
    ChainExpr,
    ThisExpr,
    Spread,
    Rest,
    ImportExpr,
    JsxElement,
    JsxFragment,
    TypeAssertion,
    SatisfiesExpr,
    MetaProperty,
    YieldExpr,
    AwaitExpr,
    Unknown,
}

/// An HIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirNode {
    /// Node kind.
    pub kind: HirNodeKind,
    /// Source span.
    pub span: Option<Span>,
    /// Stable text payload used for names and literal values.
    pub text: Option<String>,
    /// Children by index.
    pub children: Vec<HirNodeId>,
}

/// HIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HirNodeId(pub u32);

impl HirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl HirNode {
    pub fn new(kind: HirNodeKind, span: Option<Span>) -> Self {
        Self {
            kind,
            span,
            text: None,
            children: Vec::new(),
        }
    }

    pub fn with_text(kind: HirNodeKind, span: Option<Span>, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: Some(text.into()),
            children: Vec::new(),
        }
    }
}

/// HIR builder.
pub struct HirBuilder {
    nodes: Vec<HirNode>,
    next_id: HirNodeId,
}

impl HirBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: HirNodeId::new(0),
        }
    }

    pub fn alloc(&mut self, kind: HirNodeKind, span: Option<Span>) -> HirNodeId {
        let id = self.next_id;
        self.next_id.0 += 1;
        self.nodes.push(HirNode::new(kind, span));
        id
    }

    pub fn alloc_text(
        &mut self,
        kind: HirNodeKind,
        span: Option<Span>,
        text: impl Into<String>,
    ) -> HirNodeId {
        let id = self.next_id;
        self.next_id.0 += 1;
        self.nodes.push(HirNode::with_text(kind, span, text));
        id
    }

    pub fn node_mut(&mut self, id: HirNodeId) -> Option<&mut HirNode> {
        self.nodes.get_mut(id.0 as usize)
    }
}

impl Default for HirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Lowering result from AST to HIR.
pub struct LoweringResult {
    /// Root node of the HIR.
    pub root: HirNodeId,
    /// All HIR nodes.
    pub nodes: Vec<HirNode>,
    /// Diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl LoweringResult {
    /// Validate the structural consistency of the lowered HIR tree.
    pub fn validate(&self) -> Result<(), String> {
        validate_tree(
            "HIR",
            self.root,
            &self.nodes,
            |node| &node.children,
            |id| id.0 as usize,
        )
    }
}

/// HIR lowering from AST.
pub struct HirLowerer {
    builder: HirBuilder,
    diagnostics: Vec<Diagnostic>,
    synthetic_function_counter: usize,
}

macro_rules! push_child {
    ($this:expr, $parent:expr, $child:expr) => {{
        let child = $child;
        $this.push_child($parent, child);
    }};
}

impl HirLowerer {
    pub fn new() -> Self {
        Self {
            builder: HirBuilder::new(),
            diagnostics: Vec::new(),
            synthetic_function_counter: 0,
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    /// Lower a complete program from parsed statements.
    pub fn lower_statements(&mut self, statements: &[Statement]) -> LoweringResult {
        self.clear_diagnostics();
        self.builder = HirBuilder::new();
        self.synthetic_function_counter = 0;

        let root = self.builder.alloc(HirNodeKind::Program, None);
        let mut children = Vec::with_capacity(statements.len());
        for statement in statements {
            children.push(self.lower_statement(statement));
        }
        if let Some(node) = self.builder.node_mut(root) {
            node.children = children;
        }

        LoweringResult {
            root,
            nodes: self.builder.nodes.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    /// Lower an AST root by using its statements as the source of truth.
    ///
    /// The current parser already returns the statement list alongside an empty
    /// AST container, so this helper keeps the lowering API flexible while the
    /// frontend tree ownership model evolves.
    pub fn lower_program_from_ast(
        &mut self,
        ast: &AST,
        statements: &[Statement],
    ) -> LoweringResult {
        let _ = ast;
        self.lower_statements(statements)
    }

    pub fn lower_node(&mut self, node_id: NodeId) -> HirNodeId {
        self.builder.alloc_text(
            HirNodeKind::Unknown,
            None,
            format!("ast:{}", node_id.as_u32()),
        )
    }

    fn lower_statement(&mut self, statement: &Statement) -> HirNodeId {
        match statement {
            Statement::ExpressionStatement(ExpressionStatement { expression }) => {
                let id = self.builder.alloc(HirNodeKind::ExprStmt, None);
                let child = self.lower_expression(expression);
                push_child!(self, id, child);
                id
            }
            Statement::BreakStatement(BreakStatement { label }) => self.builder.alloc_text(
                HirNodeKind::BreakStmt,
                None,
                label.clone().unwrap_or_default(),
            ),
            Statement::ContinueStatement(ContinueStatement { label }) => self.builder.alloc_text(
                HirNodeKind::ContinueStmt,
                None,
                label.clone().unwrap_or_default(),
            ),
            Statement::WithStatement(WithStatement { object, body }) => {
                let id = self.builder.alloc(HirNodeKind::WithStmt, None);
                push_child!(self, id, self.lower_expression(object));
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::ReturnStatement(ReturnStatement { argument }) => {
                let id = self.builder.alloc(HirNodeKind::ReturnStmt, None);
                if let Some(arg) = argument {
                    push_child!(self, id, self.lower_expression(arg));
                }
                id
            }
            Statement::LabeledStatement(LabeledStatement { label, body }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::LabeledStmt, None, label.clone());
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::IfStatement(IfStatement {
                test,
                consequent,
                alternate,
            }) => {
                let id = self.builder.alloc(HirNodeKind::IfStmt, None);
                push_child!(self, id, self.lower_expression(test));
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**consequent).clone()))
                );
                if let Some(alt) = alternate {
                    push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::BlockStatement((**alt).clone()))
                    );
                }
                id
            }
            Statement::SwitchStatement(SwitchStatement {
                discriminant,
                cases,
            }) => {
                let id = self.builder.alloc(HirNodeKind::SwitchStmt, None);
                push_child!(self, id, self.lower_expression(discriminant));
                for case in cases {
                    let case_id = self.builder.alloc(HirNodeKind::Block, None);
                    if let Some(test) = &case.test {
                        push_child!(self, case_id, self.lower_expression(test));
                    }
                    for stmt in &case.consequent {
                        push_child!(self, case_id, self.lower_statement(stmt));
                    }
                    push_child!(self, id, case_id);
                }
                id
            }
            Statement::ThrowStatement(ThrowStatement { argument }) => {
                let id = self.builder.alloc(HirNodeKind::ThrowStmt, None);
                push_child!(self, id, self.lower_expression(argument));
                id
            }
            Statement::TryStatement(TryStatement {
                block,
                handler,
                finalizer,
            }) => {
                if handler.is_none() {
                    if let Some(finalizer) = finalizer {
                        let id = self.builder.alloc(HirNodeKind::Block, None);
                        push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::BlockStatement((**block).clone()))
                        );
                        push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::BlockStatement(finalizer.clone()))
                        );
                        id
                    } else {
                        self.lower_statement(&Statement::BlockStatement((**block).clone()))
                    }
                } else {
                    let id = self.builder.alloc(HirNodeKind::TryStmt, None);
                    push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::BlockStatement((**block).clone()))
                    );
                    if let Some(CatchClause { param, body }) = handler {
                        let catch_id =
                            self.builder
                                .alloc_text(HirNodeKind::Block, None, param.clone());
                        push_child!(
                            self,
                            catch_id,
                            self.lower_statement(&Statement::BlockStatement((**body).clone()))
                        );
                        push_child!(self, id, catch_id);
                    }
                    if let Some(finalizer) = finalizer {
                        push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::BlockStatement(finalizer.clone()))
                        );
                    }
                    id
                }
            }
            Statement::DebuggerStatement(DebuggerStatement {}) => {
                self.builder.alloc(HirNodeKind::DebuggerStmt, None)
            }
            Statement::BlockStatement(block) => self.lower_block(block),
            Statement::ForStatement(ForStatement {
                init,
                test,
                update,
                body,
            }) => {
                let id = self.builder.alloc(HirNodeKind::ForStmt, None);
                if let Some(init) = init {
                    match init {
                        ForInit::VariableDeclaration(v) => push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::VariableDeclaration(v.clone()))
                        ),
                        ForInit::Expression(e) => push_child!(self, id, self.lower_expression(e)),
                    }
                }
                if let Some(test) = test {
                    push_child!(self, id, self.lower_expression(test));
                }
                if let Some(update) = update {
                    push_child!(self, id, self.lower_expression(update));
                }
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                id
            }
            Statement::ForInStatement(ForInStatement { left, right, body }) => {
                let id = self.builder.alloc(HirNodeKind::ForInStmt, None);
                match left {
                    ForInLefthand::VariableDeclaration(v) => push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::VariableDeclaration(v.clone()))
                    ),
                    ForInLefthand::Expression(e) => push_child!(self, id, self.lower_expression(e)),
                }
                push_child!(self, id, self.lower_expression(right));
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::ForOfStatement(ForOfStatement {
                left, right, body, ..
            }) => {
                let id = self.builder.alloc(HirNodeKind::ForOfStmt, None);
                match left {
                    ForOfLefthand::VariableDeclaration(v) => push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::VariableDeclaration(v.clone()))
                    ),
                    ForOfLefthand::Expression(e) => push_child!(self, id, self.lower_expression(e)),
                }
                push_child!(self, id, self.lower_expression(right));
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::WhileStatement(WhileStatement { test, body }) => {
                let id = self.builder.alloc(HirNodeKind::WhileStmt, None);
                push_child!(self, id, self.lower_expression(test));
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                id
            }
            Statement::DoWhileStatement(DoWhileStatement { body, test }) => {
                let id = self.builder.alloc(HirNodeKind::DoWhileStmt, None);
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                push_child!(self, id, self.lower_expression(test));
                id
            }
            Statement::FunctionDeclaration(FunctionDeclaration {
                name, params, body, ..
            }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::FunctionDecl, None, name.clone());
                for param in params {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, param.clone())
                    );
                }
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                id
            }
            Statement::ClassDeclaration(ClassDeclaration { name, body }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::ClassDecl, None, name.clone());
                push_child!(self, id, self.lower_class_body(body));
                id
            }
            Statement::VariableDeclaration(VariableDeclaration { declarations, kind }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::VarDecl, None, kind.clone());
                for decl in declarations {
                    push_child!(self, id, self.lower_variable_declarator(decl));
                }
                id
            }
            Statement::ImportDeclaration(ImportDeclaration { specifiers, source }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::ImportDecl, None, source.clone());
                for specifier in specifiers {
                    push_child!(self, id, self.lower_import_specifier(specifier));
                }
                id
            }
            Statement::ExportNamed(ExportNamedDeclaration { specifiers, source }) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::ExportDecl,
                    None,
                    source.clone().unwrap_or_default(),
                );
                for specifier in specifiers {
                    push_child!(self, id, self.lower_export_specifier(specifier));
                }
                id
            }
            Statement::ExportDefault(default_decl) => self.lower_export_default(default_decl),
            Statement::EnumDeclaration(EnumDeclaration { name, members }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::EnumDecl, None, name.clone());
                for member in members {
                    let member_id =
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, member.name.clone());
                    if let Some(value) = &member.value {
                        push_child!(self, member_id, self.lower_expression(value));
                    }
                    push_child!(self, id, member_id);
                }
                id
            }
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name,
                type_params,
                type_annotation,
            }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::TypeDecl, None, name.clone());
                for param in type_params {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, param.clone())
                    );
                }
                push_child!(
                    self,
                    id,
                    self.builder
                        .alloc_text(HirNodeKind::Literal, None, type_annotation.clone())
                );
                id
            }
            Statement::InterfaceDeclaration(InterfaceDeclaration { name, properties }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::InterfaceDecl, None, name.clone());
                for property in properties {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, property.name.clone())
                    );
                }
                id
            }
        }
    }

    fn lower_class_body(&mut self, body: &ClassBody) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::Block, None);
        for method in &body.methods {
            push_child!(self, id, self.lower_method_definition(method));
        }
        id
    }

    fn lower_method_definition(&mut self, method: &MethodDefinition) -> HirNodeId {
        let id = self
            .builder
            .alloc_text(HirNodeKind::FunctionDecl, None, method.name.clone());
        for param in &method.params {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, param.clone())
            );
        }
        if let Some(body) = &method.body {
            push_child!(
                self,
                id,
                self.lower_statement(&Statement::BlockStatement((**body).clone()))
            );
        }
        id
    }

    fn lower_block(&mut self, block: &BlockStatement) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::Block, None);
        for stmt in &block.body {
            push_child!(self, id, self.lower_statement(stmt));
        }
        id
    }

    fn lower_variable_declarator(&mut self, declarator: &VariableDeclarator) -> HirNodeId {
        let id = self
            .builder
            .alloc_text(HirNodeKind::VarDeclarator, None, declarator.id.clone());
        push_child!(
            self,
            id,
            self.builder
                .alloc_text(HirNodeKind::Ident, None, declarator.id.clone())
        );
        if let Some(init) = &declarator.init {
            push_child!(self, id, self.lower_expression(init));
        }
        id
    }

    fn lower_expression(&mut self, expression: &Expression) -> HirNodeId {
        match expression {
            Expression::Identifier(name) => {
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, name.clone())
            }
            Expression::Literal(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, lower_literal_value(value))
            }
            Expression::BinaryExpression(expr) => {
                let id =
                    self.builder
                        .alloc_text(HirNodeKind::BinaryExpr, None, expr.operator.clone());
                push_child!(self, id, self.lower_expression(&expr.left));
                push_child!(self, id, self.lower_expression(&expr.right));
                id
            }
            Expression::UnaryExpression(expr) => {
                let id =
                    self.builder
                        .alloc_text(HirNodeKind::UnaryExpr, None, expr.operator.clone());
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::CallExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::CallExpr, None);
                push_child!(self, id, self.lower_expression(&expr.callee));
                for arg in &expr.args {
                    push_child!(self, id, self.lower_expression(arg));
                }
                id
            }
            Expression::MemberExpression(expr) => {
                let id =
                    self.builder
                        .alloc_text(HirNodeKind::MemberExpr, None, expr.property.clone());
                push_child!(self, id, self.lower_expression(&expr.object));
                id
            }
            Expression::ArrayExpression(ArrayExpression { elements }) => {
                let id = self.builder.alloc(HirNodeKind::ArrayExpr, None);
                for element in elements {
                    match element {
                        Some(ExpressionOrSpread::Expression(expr)) => {
                            push_child!(self, id, self.lower_expression(expr))
                        }
                        Some(ExpressionOrSpread::Spread(spread)) => push_child!(
                            self,
                            id,
                            self.lower_expression(&Expression::SpreadElement(Box::new(
                                spread.clone()
                            )))
                        ),
                        Some(ExpressionOrSpread::Empty) | None => {}
                    }
                }
                id
            }
            Expression::ObjectExpression(ObjectExpression { properties }) => {
                let id = self.builder.alloc(HirNodeKind::ObjectExpr, None);
                for property in properties {
                    push_child!(self, id, self.lower_object_property(property));
                }
                id
            }
            Expression::FunctionExpression(expr) => self.lower_function_expression(expr),
            Expression::ArrowFunctionExpression(expr) => self.lower_arrow_function_expression(expr),
            Expression::ClassExpression(expr) => self.lower_class_expression(expr),
            Expression::NewExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::NewExpr, None);
                push_child!(self, id, self.lower_expression(&expr.callee));
                for arg in &expr.args {
                    push_child!(self, id, self.lower_expression(arg));
                }
                id
            }
            Expression::TemplateLiteral(TemplateLiteral {
                quasis,
                expressions,
            }) => {
                let id = self.builder.alloc(HirNodeKind::TemplateLiteral, None);
                for quasi in quasis {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Literal, None, quasi.value.clone())
                    );
                }
                for expr in expressions {
                    push_child!(self, id, self.lower_expression(expr));
                }
                id
            }
            Expression::TaggedTemplateExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::TemplateLiteral, None);
                push_child!(self, id, self.lower_expression(&expr.tag));
                push_child!(self, id, self.lower_template_literal(&expr.template));
                id
            }
            Expression::AssignmentExpression(expr) => self.lower_assignment_expression(expr),
            Expression::LogicalExpression(expr) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::LogicalExpr,
                    None,
                    logical_op_text(&expr.operator),
                );
                push_child!(self, id, self.lower_expression(&expr.left));
                push_child!(self, id, self.lower_expression(&expr.right));
                id
            }
            Expression::ConditionalExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::ConditionalExpr, None);
                push_child!(self, id, self.lower_expression(&expr.test));
                push_child!(self, id, self.lower_expression(&expr.consequent));
                push_child!(self, id, self.lower_expression(&expr.alternate));
                id
            }
            Expression::SequenceExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::SequenceExpr, None);
                for subexpr in &expr.expressions {
                    push_child!(self, id, self.lower_expression(subexpr));
                }
                id
            }
            Expression::ParenthesizedExpression(expr) => self.lower_expression(&expr.expression),
            Expression::YieldExpression(expr) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::YieldExpr,
                    None,
                    if expr.delegate { "delegate" } else { "yield" },
                );
                if let Some(argument) = &expr.argument {
                    push_child!(self, id, self.lower_expression(argument));
                }
                id
            }
            Expression::AwaitExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::AwaitExpr, None);
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::OptionalChainExpression(expr) => self.lower_optional_chain(expr),
            Expression::ChainExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::ChainExpr, None);
                push_child!(self, id, self.lower_expression(&expr.expression));
                id
            }
            Expression::SpreadElement(expr) => {
                let id = self.builder.alloc(HirNodeKind::Spread, None);
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::RestElement(expr) => {
                let id = self.builder.alloc(HirNodeKind::Rest, None);
                push_child!(self, id, self.lower_expression(&expr.argument));
                id
            }
            Expression::ImportExpression(expr) => {
                let id = self.builder.alloc(HirNodeKind::ImportExpr, None);
                push_child!(self, id, self.lower_expression(&expr.source));
                id
            }
            Expression::DecoratedExpression(expr) => self.lower_expression(&expr.expression),
            Expression::JsxElement(_expr) => self.builder.alloc(HirNodeKind::JsxElement, None),
            Expression::JsxFragment(_expr) => self.builder.alloc(HirNodeKind::JsxFragment, None),
            Expression::JsxEmptyExpression => self.builder.alloc(HirNodeKind::Unknown, None),
            Expression::TypeAssertion(expr) => self.lower_expression(&expr.expression),
            Expression::SatisfiesExpression(expr) => self.lower_expression(&expr.expression),
            Expression::ThisExpression => self.builder.alloc(HirNodeKind::ThisExpr, None),
            Expression::SuperExpression => {
                self.builder.alloc_text(HirNodeKind::Unknown, None, "super")
            }
            Expression::PrivateIdentifier(name) => {
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, format!("#{}", name))
            }
            Expression::BigIntLiteral(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
            Expression::MetaProperty(MetaProperty { meta, property }) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::MetaProperty,
                    None,
                    format!("{}.{}", meta, property),
                );
                id
            }
            _ => self.builder.alloc(HirNodeKind::Unknown, None),
        }
    }

    fn lower_template_literal(&mut self, template: &TemplateLiteral) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::TemplateLiteral, None);
        for quasi in &template.quasis {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, quasi.value.clone())
            );
        }
        for expr in &template.expressions {
            push_child!(self, id, self.lower_expression(expr));
        }
        id
    }

    fn lower_function_expression(&mut self, expr: &FunctionExpression) -> HirNodeId {
        let name = expr
            .id
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| self.next_synthetic_function_name());
        let id = self
            .builder
            .alloc_text(HirNodeKind::FunctionExpr, None, name);
        for param in &expr.params {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, param.name.clone())
            );
        }
        if let Some(body) = &expr.body {
            push_child!(
                self,
                id,
                self.lower_statement(&Statement::BlockStatement((**body).clone()))
            );
        }
        id
    }

    fn lower_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression) -> HirNodeId {
        let name = self.next_synthetic_function_name();
        let id = self
            .builder
            .alloc_text(HirNodeKind::FunctionExpr, None, name);
        for param in &expr.params {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, param.name.clone())
            );
        }
        push_child!(
            self,
            id,
            self.lower_statement(&Statement::ReturnStatement(ReturnStatement {
                argument: Some(expr.body.clone()),
            }))
        );
        id
    }

    fn next_synthetic_function_name(&mut self) -> String {
        let name = format!("__kali_fn_{}", self.synthetic_function_counter);
        self.synthetic_function_counter += 1;
        name
    }

    fn lower_class_expression(&mut self, expr: &ClassExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::ClassExpr,
            None,
            expr.id.clone().unwrap_or_default(),
        );
        push_child!(self, id, self.lower_class_body(&expr.body));
        id
    }

    fn lower_optional_chain(&mut self, expr: &OptionalChainExpression) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::OptionalChain, None);
        match expr.inner.as_ref() {
            OptionalChainInner::NonNull {
                object,
                optional: _,
            } => {
                push_child!(self, id, self.lower_expression(object));
            }
        }
        id
    }

    fn lower_assignment_expression(&mut self, expr: &AssignmentExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::AssignmentExpr,
            None,
            assignment_op_text(&expr.operator),
        );
        push_child!(self, id, self.lower_expression(&expr.left));
        push_child!(self, id, self.lower_expression(&expr.right));
        id
    }

    fn lower_object_property(&mut self, property: &ObjectProperty) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::ObjectProperty,
            None,
            object_property_kind_text(&property.kind),
        );
        push_child!(self, id, self.lower_property_name(&property.key));
        push_child!(self, id, self.lower_expression(&property.value));
        id
    }

    fn lower_property_name(&mut self, name: &PropertyName) -> HirNodeId {
        match name {
            PropertyName::Identifier(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
            PropertyName::Number(value) => {
                // Preserve numeric property names as strings so object-literal lowering
                // keeps JavaScript's property-key semantics instead of treating them as
                // arithmetic literals during code generation.
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, format!("\"{}\"", value))
            }
            PropertyName::String(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
        }
    }

    fn lower_import_specifier(&mut self, specifier: &ImportSpecifier) -> HirNodeId {
        match specifier {
            ImportSpecifier::Default(name) | ImportSpecifier::Namespace(name) => self
                .builder
                .alloc_text(HirNodeKind::Ident, None, name.clone()),
            ImportSpecifier::Named(specifiers) | ImportSpecifier::Type(specifiers) => {
                let id = self.builder.alloc(HirNodeKind::ImportDecl, None);
                for spec in specifiers {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, spec.local.clone())
                    );
                }
                id
            }
            ImportSpecifier::SideEffect => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, "side-effect")
            }
        }
    }

    fn lower_export_specifier(&mut self, specifier: &ExportSpecifier) -> HirNodeId {
        let id = self
            .builder
            .alloc_text(HirNodeKind::Ident, None, specifier.exported.clone());
        push_child!(
            self,
            id,
            self.builder
                .alloc_text(HirNodeKind::Ident, None, specifier.local.clone())
        );
        id
    }

    fn lower_export_default(&mut self, default_decl: &ExportDefaultDeclaration) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::ExportDecl, None);
        match default_decl {
            ExportDefaultDeclaration::Expression(expr) => {
                push_child!(self, id, self.lower_expression(expr))
            }
            ExportDefaultDeclaration::FunctionDeclaration(func) => push_child!(
                self,
                id,
                self.lower_statement(&Statement::FunctionDeclaration(func.clone()))
            ),
            ExportDefaultDeclaration::ClassDeclaration(class) => push_child!(
                self,
                id,
                self.lower_statement(&Statement::ClassDeclaration(class.clone()))
            ),
        }
        id
    }

    fn push_child(&mut self, parent: HirNodeId, child: HirNodeId) {
        if let Some(node) = self.builder.node_mut(parent) {
            node.children.push(child);
        }
    }
}

impl Default for HirLowerer {
    fn default() -> Self {
        Self::new()
    }
}

fn lower_literal_value(value: &LiteralValue) -> String {
    match value {
        LiteralValue::Boolean(v) => v.to_string(),
        LiteralValue::Number(v) => v.to_string(),
        LiteralValue::String(v) => v.clone(),
        LiteralValue::Regex { pattern, flags } => format!("/{}/{}", pattern, flags),
        LiteralValue::Null => "null".to_string(),
    }
}

fn logical_op_text(op: &kali_ast::LogicalOperator) -> &'static str {
    match op {
        kali_ast::LogicalOperator::And => "&&",
        kali_ast::LogicalOperator::Or => "||",
        kali_ast::LogicalOperator::Coalesce => "??",
    }
}

fn assignment_op_text(op: &AssignmentOperator) -> &'static str {
    match op {
        AssignmentOperator::Assign => "=",
        AssignmentOperator::AddAssign => "+=",
        AssignmentOperator::SubtractAssign => "-=",
        AssignmentOperator::MultiplyAssign => "*=",
        AssignmentOperator::DivideAssign => "/=",
        AssignmentOperator::ModuloAssign => "%=",
        AssignmentOperator::ExponentAssign => "**=",
    }
}

fn object_property_kind_text(kind: &ObjectPropertyKind) -> &'static str {
    match kind {
        ObjectPropertyKind::Init => "init",
        ObjectPropertyKind::Get => "get",
        ObjectPropertyKind::Set => "set",
    }
}

fn validate_tree<Node, Id>(
    label: &str,
    root: Id,
    nodes: &[Node],
    children: impl Fn(&Node) -> &[Id],
    to_index: impl Fn(Id) -> usize,
) -> Result<(), String>
where
    Id: Copy,
{
    if nodes.is_empty() {
        return Err(format!("{label} tree contains no nodes"));
    }

    let root_index = to_index(root);
    if root_index >= nodes.len() {
        return Err(format!(
            "{label} root node id {root_index} is out of bounds for {} nodes",
            nodes.len()
        ));
    }

    for (index, node) in nodes.iter().enumerate() {
        for child in children(node) {
            let child_index = to_index(*child);
            if child_index >= nodes.len() {
                return Err(format!(
                    "{label} node {index} references child node id {child_index} outside the node table of {} nodes",
                    nodes.len()
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
