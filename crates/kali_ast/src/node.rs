//! AST node identity and the legacy `NodeKind` tree node.

use kali_common::Span;

#[cfg(test)]
#[path = "node_tests.rs"]
mod node_tests;

/// Node identifier for AST nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

/// A node identifier serializer for JSON output.
impl serde::Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

impl<'de> serde::Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = u32::deserialize(deserializer)?;
        Ok(NodeId(id))
    }
}

impl NodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "n{}", self.0)
    }
}

/// Module items are nodes in a module body.
pub type ModuleItem = Node;

/// The NodeKind enum for the legacy AST system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum NodeKind {
    // Program structure
    Program,
    Module {
        body: Vec<Node>,
        source_type: String,
    },
    Script,
    Block,
    ExpressionStatement,
    EmptyStatement,
    BreakStatement,
    ContinueStatement,
    DebuggerStatement,
    ReturnStatement,
    ThrowStatement,
    LabeledStatement,
    IfStatement,
    SwitchStatement,
    TryStatement,
    WhileStatement,
    DoWhileStatement,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    WithStatement,

    // Declarations
    FunctionDeclaration,
    FunctionExpression,
    ClassDeclaration,
    ClassExpression,
    VariableDeclaration,
    VariableDeclarator,
    ImportDeclaration,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    ImportSpecifier,
    ExportAllDeclaration,
    ExportDefaultDeclaration,
    ExportNamedDeclaration,
    ExportSpecifier,
    InterfaceDeclaration,
    TypeAliasDeclaration,
    EnumDeclaration,
    EnumMember,
    TypeLiteral,
    TsTypeAnnotation,
    TsTypeParameterDeclaration,
    TsTypeParameter,
    TsConstraint,
    TsTypeParameterConstraint,
    TsTypeParameterDefault,
    TsInterfaceBody,
    TsPropertySignature,
    TsMethodSignature,
    TsIndexSignature,
    TsIndexSignatureAnnotation,
    TsCallSignatureDeclaration,
    TsConstructSignatureDeclaration,
    TsPropertyParameter,
}

impl PartialEq for NodeKind {
    fn eq(&self, other: &Self) -> bool {
        use NodeKind::*;
        match (self, other) {
            (Program, Program)
            | (Script, Script)
            | (Block, Block)
            | (ExpressionStatement, ExpressionStatement)
            | (EmptyStatement, EmptyStatement)
            | (BreakStatement, BreakStatement)
            | (ContinueStatement, ContinueStatement)
            | (DebuggerStatement, DebuggerStatement)
            | (ReturnStatement, ReturnStatement)
            | (ThrowStatement, ThrowStatement)
            | (LabeledStatement, LabeledStatement)
            | (IfStatement, IfStatement)
            | (SwitchStatement, SwitchStatement)
            | (TryStatement, TryStatement)
            | (WhileStatement, WhileStatement)
            | (DoWhileStatement, DoWhileStatement)
            | (ForStatement, ForStatement)
            | (ForInStatement, ForInStatement)
            | (ForOfStatement, ForOfStatement)
            | (WithStatement, WithStatement)
            | (FunctionDeclaration, FunctionDeclaration)
            | (FunctionExpression, FunctionExpression)
            | (ClassDeclaration, ClassDeclaration)
            | (ClassExpression, ClassExpression)
            | (VariableDeclaration, VariableDeclaration)
            | (VariableDeclarator, VariableDeclarator)
            | (ImportDeclaration, ImportDeclaration)
            | (ImportDefaultSpecifier, ImportDefaultSpecifier)
            | (ImportNamespaceSpecifier, ImportNamespaceSpecifier)
            | (ImportSpecifier, ImportSpecifier)
            | (ExportAllDeclaration, ExportAllDeclaration)
            | (ExportDefaultDeclaration, ExportDefaultDeclaration)
            | (ExportNamedDeclaration, ExportNamedDeclaration)
            | (ExportSpecifier, ExportSpecifier)
            | (InterfaceDeclaration, InterfaceDeclaration)
            | (TypeAliasDeclaration, TypeAliasDeclaration)
            | (EnumDeclaration, EnumDeclaration)
            | (EnumMember, EnumMember)
            | (TypeLiteral, TypeLiteral)
            | (TsTypeAnnotation, TsTypeAnnotation)
            | (TsTypeParameterDeclaration, TsTypeParameterDeclaration)
            | (TsTypeParameter, TsTypeParameter)
            | (TsConstraint, TsConstraint)
            | (TsTypeParameterConstraint, TsTypeParameterConstraint)
            | (TsTypeParameterDefault, TsTypeParameterDefault)
            | (TsInterfaceBody, TsInterfaceBody)
            | (TsPropertySignature, TsPropertySignature)
            | (TsMethodSignature, TsMethodSignature)
            | (TsIndexSignature, TsIndexSignature)
            | (TsIndexSignatureAnnotation, TsIndexSignatureAnnotation)
            | (TsCallSignatureDeclaration, TsCallSignatureDeclaration)
            | (TsConstructSignatureDeclaration, TsConstructSignatureDeclaration)
            | (TsPropertyParameter, TsPropertyParameter) => true,
            (Module { body: b1, .. }, Module { body: b2, .. }) => b1 == b2,
            _ => false,
        }
    }
}

impl Eq for NodeKind {}

/// An AST node.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// The node's kind/type.
    pub kind: NodeKind,
    /// Source span for this node.
    pub span: Option<Span>,
    /// Child nodes (by NodeId).
    #[serde(skip)]
    pub children: Vec<NodeId>,
}

impl Node {
    /// Create a new AST node.
    pub fn new(id: NodeId, kind: NodeKind, span: Option<Span>) -> Self {
        Self {
            id,
            kind,
            span,
            children: Vec::new(),
        }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: NodeId) {
        self.children.push(child);
    }
}
