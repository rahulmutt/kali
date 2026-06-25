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
use kali_error::diagnostic::Diagnostic;

use crate::helpers::{
    assignment_op_text, logical_op_text, lower_literal_value, object_property_kind_text,
    update_op_text,
};

mod builder;
mod helpers;
mod lowering;
mod node;
mod result;
pub use builder::HirBuilder;
pub use node::{HirNode, HirNodeId, HirNodeKind};
pub use result::{FunctionFlavor, LoweringResult};

/// HIR lowering from AST.
pub struct HirLowerer {
    pub(crate) builder: HirBuilder,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) function_flavors: Vec<(HirNodeId, FunctionFlavor)>,
    pub(crate) synthetic_function_counter: usize,
}

macro_rules! push_child {
    ($this:expr, $parent:expr, $child:expr) => {{
        let child = $child;
        $this.push_child($parent, child);
    }};
}
pub(crate) use push_child;

impl HirLowerer {
    pub fn new() -> Self {
        Self {
            builder: HirBuilder::new(),
            diagnostics: Vec::new(),
            function_flavors: Vec::new(),
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
        self.function_flavors.clear();
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
            function_flavors: self.function_flavors.clone(),
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

    pub(crate) fn lower_expression(&mut self, expression: &Expression) -> HirNodeId {
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
            Expression::UpdateExpression(expr) => self.lower_update_expression(expr),
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
                let id = self.builder.alloc_text(HirNodeKind::SequenceExpr, None, "");
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
                let id = self.builder.alloc_text(HirNodeKind::Spread, None, "spread");
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
        }
    }

    pub(crate) fn lower_template_literal(&mut self, template: &TemplateLiteral) -> HirNodeId {
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

    pub(crate) fn lower_function_expression(&mut self, expr: &FunctionExpression) -> HirNodeId {
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
        self.record_function_flavor(
            id,
            FunctionFlavor::from_flags(expr.is_async, expr.generator),
        );
        id
    }

    pub(crate) fn lower_arrow_function_expression(&mut self, expr: &ArrowFunctionExpression) -> HirNodeId {
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
        self.record_function_flavor(id, FunctionFlavor::from_flags(expr.is_async, false));
        id
    }

    pub(crate) fn next_synthetic_function_name(&mut self) -> String {
        let name = format!("__kali_fn_{}", self.synthetic_function_counter);
        self.synthetic_function_counter += 1;
        name
    }

    pub(crate) fn lower_class_expression(&mut self, expr: &ClassExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::ClassExpr,
            None,
            expr.id.clone().unwrap_or_default(),
        );
        push_child!(self, id, self.lower_class_body(&expr.body));
        id
    }

    pub(crate) fn lower_optional_chain(&mut self, expr: &OptionalChainExpression) -> HirNodeId {
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

    pub(crate) fn lower_update_expression(&mut self, expr: &UpdateExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::UpdateExpr,
            None,
            update_op_text(&expr.operator, expr.prefix),
        );
        push_child!(self, id, self.lower_expression(&expr.argument));
        id
    }

    pub(crate) fn lower_assignment_expression(&mut self, expr: &AssignmentExpression) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::AssignmentExpr,
            None,
            assignment_op_text(&expr.operator),
        );
        push_child!(self, id, self.lower_expression(&expr.left));
        push_child!(self, id, self.lower_expression(&expr.right));
        id
    }

    pub(crate) fn lower_object_property(&mut self, property: &ObjectProperty) -> HirNodeId {
        let id = self.builder.alloc_text(
            HirNodeKind::ObjectProperty,
            None,
            object_property_kind_text(&property.kind),
        );
        push_child!(self, id, self.lower_property_name(&property.key));
        push_child!(self, id, self.lower_expression(&property.value));
        id
    }

    pub(crate) fn lower_property_name(&mut self, name: &PropertyName) -> HirNodeId {
        match name {
            PropertyName::Identifier(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
            PropertyName::Number(value) => {
                // Preserve numeric property names as strings so object-literal lowering
                // keeps JavaScript's property-key semantics instead of treating them as
                // arithmetic literals during code generation.
                let key = if *value == 0.0 {
                    "0".to_string()
                } else {
                    value.to_string()
                };
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, format!("\"{}\"", key))
            }
            PropertyName::String(value) => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, value.clone())
            }
        }
    }

    pub(crate) fn lower_import_specifier(&mut self, specifier: &ImportSpecifier) -> HirNodeId {
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

    pub(crate) fn lower_export_specifier(&mut self, specifier: &ExportSpecifier) -> HirNodeId {
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

    pub(crate) fn lower_export_default(&mut self, default_decl: &ExportDefaultDeclaration) -> HirNodeId {
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

    pub(crate) fn record_function_flavor(&mut self, node_id: HirNodeId, flavor: FunctionFlavor) {
        self.function_flavors.push((node_id, flavor));
    }

    pub(crate) fn push_child(&mut self, parent: HirNodeId, child: HirNodeId) {
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
