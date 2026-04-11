//! Mid-level IR (MIR) for the Kali compiler.
//!
//! MIR is a conservative structural lowering of HIR that preserves the source
//! shape while providing a stable bridge for later memory/ownership analysis.

use kali_hir::{HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};

/// MIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirNodeKind {
    Program,
    Block,
    Function,
    Decl,
    Expr,
    Call,
    Literal,
    ControlFlow,
    Unknown,
}

/// MIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MirNodeId(pub u32);

impl MirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// MIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirNode {
    pub kind: MirNodeKind,
    pub text: Option<String>,
    pub children: Vec<MirNodeId>,
}

impl MirNode {
    pub fn new(kind: MirNodeKind) -> Self {
        Self {
            kind,
            text: None,
            children: Vec::new(),
        }
    }

    pub fn with_text(kind: MirNodeKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: Some(text.into()),
            children: Vec::new(),
        }
    }
}

/// MIR builder.
#[derive(Default)]
pub struct MirBuilder {
    nodes: Vec<MirNode>,
}

impl MirBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, kind: MirNodeKind) -> MirNodeId {
        let id = MirNodeId(self.nodes.len() as u32);
        self.nodes.push(MirNode::new(kind));
        id
    }

    pub fn alloc_text(&mut self, kind: MirNodeKind, text: impl Into<String>) -> MirNodeId {
        let id = MirNodeId(self.nodes.len() as u32);
        self.nodes.push(MirNode::with_text(kind, text));
        id
    }

    pub fn node_mut(&mut self, id: MirNodeId) -> Option<&mut MirNode> {
        self.nodes.get_mut(id.0 as usize)
    }
}

/// MIR lowering result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirProgram {
    pub root: MirNodeId,
    pub nodes: Vec<MirNode>,
}

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
        let root = self.lower_hir_node(&mut builder, &hir.nodes, hir.root);
        MirProgram {
            root,
            nodes: builder.nodes,
        }
    }

    fn lower_hir_node(
        &self,
        builder: &mut MirBuilder,
        nodes: &[HirNode],
        id: HirNodeId,
    ) -> MirNodeId {
        let node = &nodes[id.0 as usize];
        let kind = map_kind(&node.kind);
        let mir_id = match node.text.as_ref() {
            Some(text) => builder.alloc_text(kind, text.clone()),
            None => builder.alloc(kind),
        };
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(self.lower_hir_node(builder, nodes, *child));
        }
        if let Some(mir_node) = builder.node_mut(mir_id) {
            mir_node.children = children;
        }
        mir_id
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
        HirNodeKind::CallExpr
        | HirNodeKind::MemberExpr
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
mod tests {
    use super::*;
    use kali_common::FileId;
    use kali_hir::HirLowerer;
    use kali_lexer::Lexer;
    use kali_parser::Parser;

    fn parse_and_lower_hir(source: &str) -> HirLoweringResult {
        let lexer = Lexer::new(FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = Parser::new(FileId::new(0), tokens);
        let statements = parser.parse(None).statements;
        let mut lowerer = HirLowerer::new();
        lowerer.lower_statements(&statements)
    }

    #[test]
    fn test_mir_lowering_preserves_program_shape() {
        let hir = parse_and_lower_hir("const answer = 40 + 2;");
        let mir = MirLowerer::new().lower_hir_result(&hir);

        assert_eq!(mir.nodes[mir.root.0 as usize].kind, MirNodeKind::Program);
        assert_eq!(mir.nodes[mir.root.0 as usize].children.len(), 1);
        assert_eq!(
            mir.nodes[mir.nodes[mir.root.0 as usize].children[0].0 as usize].kind,
            MirNodeKind::Decl
        );
    }
}
