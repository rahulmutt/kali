//! HIR node representation: kinds, nodes, and node identifiers.

use kali_common::Span;

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
