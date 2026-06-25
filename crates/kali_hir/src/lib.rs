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

use crate::helpers::object_property_kind_text;

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
