//! Structural HIR→MIR lowering.

use kali_hir::{
    FunctionFlavor, HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult,
};

use crate::{MirBuilder, MirNodeId, MirNodeKind, MirProgram, OwnershipAnalyzer};

/// MIR lowering from HIR.
#[derive(Default)]
pub struct MirLowerer;

impl MirLowerer {
    pub fn new() -> Self {
        Self
    }

    /// Preserve the old shape-oriented API.
    pub fn lower_hir(&self, _hir: HirNodeId) -> MirNodeId {
        MirNodeId::new(0)
    }

    pub fn lower_hir_result(&self, hir: &HirLoweringResult) -> MirProgram {
        let mut builder = MirBuilder::new();
        let root = self.lower_hir_node(&mut builder, &hir.nodes, hir.root, hir);
        let functions =
            OwnershipAnalyzer::new(&hir.nodes, &hir.function_flavors).analyze_program(hir.root);
        MirProgram {
            root,
            nodes: builder.nodes,
            functions,
        }
    }

    fn lower_hir_node(
        &self,
        builder: &mut MirBuilder,
        nodes: &[HirNode],
        id: HirNodeId,
        hir: &HirLoweringResult,
    ) -> MirNodeId {
        let node = &nodes[id.0 as usize];
        let kind = map_kind(&node.kind);
        let mir_id = match node.text.as_ref() {
            Some(text) => builder.alloc_text(kind, text.clone()),
            None => builder.alloc(kind),
        };
        if let Some(mir_node) = builder.node_mut(mir_id) {
            mir_node.function_flavor = self.function_flavor(hir, id);
        }
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(self.lower_hir_node(builder, nodes, *child, hir));
        }
        if let Some(mir_node) = builder.node_mut(mir_id) {
            mir_node.children = children;
        }
        mir_id
    }
    fn function_flavor(&self, hir: &HirLoweringResult, id: HirNodeId) -> Option<FunctionFlavor> {
        hir.function_flavors
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, flavor)| *flavor)
    }
}

fn map_kind(kind: &HirNodeKind) -> MirNodeKind {
    match kind {
        HirNodeKind::Program => MirNodeKind::Program,
        HirNodeKind::Block => MirNodeKind::Block,
        HirNodeKind::FunctionDecl
        | HirNodeKind::FunctionExpr
        | HirNodeKind::ClassDecl
        | HirNodeKind::ClassExpr => MirNodeKind::Function,
        HirNodeKind::VarDecl
        | HirNodeKind::VarDeclarator
        | HirNodeKind::ImportDecl
        | HirNodeKind::ExportDecl
        | HirNodeKind::TypeDecl
        | HirNodeKind::InterfaceDecl
        | HirNodeKind::EnumDecl => MirNodeKind::Decl,
        HirNodeKind::IfStmt
        | HirNodeKind::ForStmt
        | HirNodeKind::ForInStmt
        | HirNodeKind::ForOfStmt
        | HirNodeKind::WhileStmt
        | HirNodeKind::DoWhileStmt
        | HirNodeKind::SwitchStmt
        | HirNodeKind::TryStmt
        | HirNodeKind::ReturnStmt
        | HirNodeKind::BreakStmt
        | HirNodeKind::ContinueStmt
        | HirNodeKind::ThrowStmt
        | HirNodeKind::DebuggerStmt
        | HirNodeKind::LabeledStmt
        | HirNodeKind::WithStmt => MirNodeKind::ControlFlow,
        HirNodeKind::Literal => MirNodeKind::Literal,
        HirNodeKind::CallExpr => MirNodeKind::Call,
        HirNodeKind::MemberExpr
        | HirNodeKind::NewExpr
        | HirNodeKind::BinaryExpr
        | HirNodeKind::LogicalExpr
        | HirNodeKind::UnaryExpr
        | HirNodeKind::UpdateExpr
        | HirNodeKind::AssignmentExpr
        | HirNodeKind::ConditionalExpr
        | HirNodeKind::SequenceExpr
        | HirNodeKind::ArrayExpr
        | HirNodeKind::ObjectExpr
        | HirNodeKind::ObjectProperty
        | HirNodeKind::OptionalChain
        | HirNodeKind::ChainExpr
        | HirNodeKind::Spread
        | HirNodeKind::Rest
        | HirNodeKind::ImportExpr
        | HirNodeKind::JsxElement
        | HirNodeKind::JsxFragment
        | HirNodeKind::TypeAssertion
        | HirNodeKind::SatisfiesExpr
        | HirNodeKind::MetaProperty
        | HirNodeKind::YieldExpr
        | HirNodeKind::AwaitExpr
        | HirNodeKind::ThisExpr
        | HirNodeKind::Ident
        | HirNodeKind::ExprStmt
        | HirNodeKind::TemplateLiteral => MirNodeKind::Expr,
        HirNodeKind::Unknown => MirNodeKind::Unknown,
    }
}

#[cfg(test)]
#[path = "lower_tests.rs"]
mod lower_tests;
