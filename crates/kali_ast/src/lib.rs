//! AST definitions for TypeScript/JavaScript.
//!
//! This crate defines the Abstract Syntax Tree node types.

use kali_common::Span;

/// A node identifier for AST nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(pub u32);

/// A module item represents either a statement or an expression statement in a module.
/// Future implementations will define the full list of statement types.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModuleItem {
    Statement(Statement),
    ExpressionStatement(ExpressionStatement),
}

/// Module-level statements for TypeScript/JavaScript.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Statement {
    BlockStatement(Box<BlockStatement>),
    EmptyStatement,
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    WithStatement(Box<WithStatement>),
    ReturnStatement(ReturnStatement),
    LabeledStatement(Box<LabeledStatement>),
    ExpressionStatement(ExpressionStatement),
    IfStatement(Box<IfStatement>),
    SwitchStatement(Box<SwitchStatement>),
    ThrowStatement(ThrowStatement),
    TryStatement(Box<TryStatement>),
    DebuggerStatement(DebuggerStatement),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ClassDeclaration),
    ImportDeclaration(ImportDeclaration),
    ExportAllDeclaration(ExportAllDeclaration),
    ExportDefaultDeclaration(ExportDefaultDeclaration),
    ExportNamedDeclaration(ExportNamedDeclaration),
    VariableDeclaration(VariableDeclaration),
    ForStatement(Box<ForStatement>),
    ForInStatement(Box<ForInStatement>),
    ForOfStatement(Box<ForOfStatement>),
    WhileStatement(Box<WhileStatement>),
    DoWhileStatement(Box<DoWhileStatement>),
    TypeAliasDeclaration(TypeAliasDeclaration),
    InterfaceDeclaration(InterfaceDeclaration),
    EnumDeclaration(EnumDeclaration),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

/// Break statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BreakStatement {
    pub label: Option<String>,
}

/// Continue statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContinueStatement {
    pub label: Option<String>,
}

/// With statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WithStatement {
    pub object: Expression,
    pub body: Statement,
}

/// Return statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReturnStatement {
    pub argument: Option<Expression>,
}

/// Labeled statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LabeledStatement {
    pub label: String,
    pub body: Statement,
}

/// If statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IfStatement {
    pub test: Expression,
    pub consequent: Statement,
    pub alternate: Option<Statement>,
}

/// Switch statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SwitchStatement {
    pub discriminant: Expression,
    pub cases: Vec<SwitchCase>,
}

/// Switch case
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SwitchCase {
    pub test: Option<Expression>,
    pub consequent: Vec<Statement>,
}

/// Throw statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ThrowStatement {
    pub argument: Expression,
}

/// Try statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TryStatement {
    pub block: Box<BlockStatement>,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStatement>,
}

/// Catch clause
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CatchClause {
    pub param: String,
    pub body: Box<BlockStatement>,
}

/// Debugger statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DebuggerStatement {}

/// Block statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BlockStatement {
    pub body: Vec<Statement>,
}

/// For statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForStatement {
    pub init: Option<ForInit>,
    pub test: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Statement,
}

/// For init
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ForInit {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
}

/// For-in statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForInStatement {
    pub left: ForInLefthand,
    pub right: Expression,
    pub body: Statement,
}

/// For-in lefthand
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ForInLefthand {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
}

/// For-of statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForOfStatement {
    pub left: ForOfLefthand,
    pub right: Expression,
    pub body: Statement,
}

/// For-of lefthand
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ForOfLefthand {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
}

/// While statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WhileStatement {
    pub test: Expression,
    pub body: Statement,
}

/// Do-while statement
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DoWhileStatement {
    pub body: Statement,
    pub test: Expression,
}

/// Function declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<BlockStatement>,
}

/// Class declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClassDeclaration {
    pub name: String,
    pub body: Box<ClassBody>,
}

/// Class body
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClassBody {
    pub methods: Vec<MethodDefinition>,
}

/// Method definition
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MethodDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Box<BlockStatement>>,
}

// Variable declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VariableDeclaration {
    pub declarations: Vec<VariableDeclarator>,
    pub kind: String, // var, let, const
}

// Variable declarator
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VariableDeclarator {
    pub id: String,
    pub init: Option<Expression>,
}

// Import declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImportDeclaration {
    pub specifiers: Vec<ImportSpecifierType>,
    pub source: String,
}

// Import specifier types
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImportSpecifierType {
    Default(String),
    Named(Vec<String>),
    Namespace(String),
}

// Export all declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExportAllDeclaration {
    pub source: String,
}

// Export default declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExportDefaultDeclaration {
    pub declaration: ExportDefaultDeclarationType,
}

// Export default declaration type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExportDefaultDeclarationType {
    Expression(Expression),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ClassDeclaration),
}

// Export named declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExportNamedDeclaration {
    pub specifiers: Vec<ExportSpecifier>,
    pub source: Option<String>,
}

// Export specifier
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExportSpecifier {
    pub local: String,
    pub exported: String,
}

// Type alias declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TypeAliasDeclaration {
    pub name: String,
    pub type_params: Vec<String>,
    pub type_annotation: String,
}

// Interface declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InterfaceDeclaration {
    pub name: String,
    pub properties: Vec<PropertySignature>,
}

// Property signature
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PropertySignature {
    pub name: String,
    pub type_annotation: String,
}

// Enum declaration
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumDeclaration {
    pub name: String,
    pub members: Vec<EnumMember>,
}

// Enum member
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<Expression>,
}

// Expression types
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Expression {
    Identifier(String),
    Literal(String),
    BinaryExpression(Box<BinaryExpression>),
    UnaryExpression(Box<UnaryExpression>),
    CallExpression(Box<CallExpression>),
    MemberExpression(Box<MemberExpression>),
}

// Binary expression
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BinaryExpression {
    pub operator: String,
    pub left: Expression,
    pub right: Expression,
}

// Unary expression
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnaryExpression {
    pub operator: String,
    pub argument: Expression,
}

// Call expression
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CallExpression {
    pub callee: Expression,
    pub args: Vec<Expression>,
}

// Member expression
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemberExpression {
    pub object: Expression,
    pub property: String,
}

/// Types of AST nodes covering full ECMA-262 + TypeScript.

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

/// An AST node.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Types of AST nodes covering full ECMA-262 + TypeScript.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum NodeKind {
    // Program structure
    Program,
    Module {
        body: Vec<ModuleItem>,
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

impl NodeKind {
    pub fn from_kind(kind: &str) -> Self {
        match kind {
            // Program structure
            "Program" => NodeKind::Program,
            "Module" => NodeKind::Module { body: vec![], source_type: "".to_string() },
            "Script" => NodeKind::Script,
            "Block" => NodeKind::Block,
            "ExpressionStatement" => NodeKind::ExpressionStatement,
            "EmptyStatement" => NodeKind::EmptyStatement,
            "BreakStatement" => NodeKind::BreakStatement,
            "ContinueStatement" => NodeKind::ContinueStatement,
            "DebuggerStatement" => NodeKind::DebuggerStatement,
            "ReturnStatement" => NodeKind::ReturnStatement,
            "ThrowStatement" => NodeKind::ThrowStatement,
            "LabeledStatement" => NodeKind::LabeledStatement,
            "IfStatement" => NodeKind::IfStatement,
            "SwitchStatement" => NodeKind::SwitchStatement,
            "TryStatement" => NodeKind::TryStatement,
            "WhileStatement" => NodeKind::WhileStatement,
            "DoWhileStatement" => NodeKind::DoWhileStatement,
            "ForStatement" => NodeKind::ForStatement,
            "ForInStatement" => NodeKind::ForInStatement,
            "ForOfStatement" => NodeKind::ForOfStatement,
            "WithStatement" => NodeKind::WithStatement,

            // Declarations
            "FunctionDeclaration" => NodeKind::FunctionDeclaration,
            "FunctionExpression" => NodeKind::FunctionExpression,
            "ClassDeclaration" => NodeKind::ClassDeclaration,
            "ClassExpression" => NodeKind::ClassExpression,
            "VariableDeclaration" => NodeKind::VariableDeclaration,
            "VariableDeclarator" => NodeKind::VariableDeclarator,
            "ImportDeclaration" => NodeKind::ImportDeclaration,
            "ImportDefaultSpecifier" => NodeKind::ImportDefaultSpecifier,
            "ImportNamespaceSpecifier" => NodeKind::ImportNamespaceSpecifier,
            "ImportSpecifier" => NodeKind::ImportSpecifier,
            "ExportAllDeclaration" => NodeKind::ExportAllDeclaration,
            "ExportDefaultDeclaration" => NodeKind::ExportDefaultDeclaration,
            "ExportNamedDeclaration" => NodeKind::ExportNamedDeclaration,
            "ExportSpecifier" => NodeKind::ExportSpecifier,
            "InterfaceDeclaration" => NodeKind::InterfaceDeclaration,
            "TypeAliasDeclaration" => NodeKind::TypeAliasDeclaration,
            "EnumDeclaration" => NodeKind::EnumDeclaration,
            "EnumMember" => NodeKind::EnumMember,
            "TypeLiteral" => NodeKind::TypeLiteral,
            "TsTypeAnnotation" => NodeKind::TsTypeAnnotation,
            "TsTypeParameterDeclaration" => NodeKind::TsTypeParameterDeclaration,
            "TsTypeParameter" => NodeKind::TsTypeParameter,
            "TsConstraint" => NodeKind::TsConstraint,
            "TsTypeParameterConstraint" => NodeKind::TsTypeParameterConstraint,
            "TsTypeParameterDefault" => NodeKind::TsTypeParameterDefault,
            "TsInterfaceBody" => NodeKind::TsInterfaceBody,
            "TsPropertySignature" => NodeKind::TsPropertySignature,
            "TsMethodSignature" => NodeKind::TsMethodSignature,
            "TsIndexSignature" => NodeKind::TsIndexSignature,
            "TsIndexSignatureAnnotation" => NodeKind::TsIndexSignatureAnnotation,
            "TsCallSignatureDeclaration" => NodeKind::TsCallSignatureDeclaration,
            "TsConstructSignatureDeclaration" => NodeKind::TsConstructSignatureDeclaration,
            "TsPropertyParameter" => NodeKind::TsPropertyParameter,
            _ => panic!("Unknown NodeKind: {}", kind),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Module { .. } => "Module",
            NodeKind::Script => "Script",
            NodeKind::Block => "Block",
            NodeKind::ExpressionStatement => "ExpressionStatement",
            NodeKind::EmptyStatement => "EmptyStatement",
            NodeKind::BreakStatement => "BreakStatement",
            NodeKind::ContinueStatement => "ContinueStatement",
            NodeKind::DebuggerStatement => "DebuggerStatement",
            NodeKind::ReturnStatement => "ReturnStatement",
            NodeKind::ThrowStatement => "ThrowStatement",
            NodeKind::LabeledStatement => "LabeledStatement",
            NodeKind::IfStatement => "IfStatement",
            NodeKind::SwitchStatement => "SwitchStatement",
            NodeKind::TryStatement => "TryStatement",
            NodeKind::WhileStatement => "WhileStatement",
            NodeKind::DoWhileStatement => "DoWhileStatement",
            NodeKind::ForStatement => "ForStatement",
            NodeKind::ForInStatement => "ForInStatement",
            NodeKind::ForOfStatement => "ForOfStatement",
            NodeKind::WithStatement => "WithStatement",
            NodeKind::FunctionDeclaration => "FunctionDeclaration",
            NodeKind::FunctionExpression => "FunctionExpression",
            NodeKind::ClassDeclaration => "ClassDeclaration",
            NodeKind::ClassExpression => "ClassExpression",
            NodeKind::VariableDeclaration => "VariableDeclaration",
            NodeKind::VariableDeclarator => "VariableDeclarator",
            NodeKind::ImportDeclaration => "ImportDeclaration",
            NodeKind::ImportDefaultSpecifier => "ImportDefaultSpecifier",
            NodeKind::ImportNamespaceSpecifier => "ImportNamespaceSpecifier",
            NodeKind::ImportSpecifier => "ImportSpecifier",
            NodeKind::ExportAllDeclaration => "ExportAllDeclaration",
            NodeKind::ExportDefaultDeclaration => "ExportDefaultDeclaration",
            NodeKind::ExportNamedDeclaration => "ExportNamedDeclaration",
            NodeKind::ExportSpecifier => "ExportSpecifier",
            NodeKind::InterfaceDeclaration => "InterfaceDeclaration",
            NodeKind::TypeAliasDeclaration => "TypeAliasDeclaration",
            NodeKind::EnumDeclaration => "EnumDeclaration",
            NodeKind::EnumMember => "EnumMember",
            NodeKind::TypeLiteral => "TypeLiteral",
            NodeKind::TsTypeAnnotation => "TSTypeAnnotation",
            NodeKind::TsTypeParameterDeclaration => "TSTypeParameterDeclaration",
            NodeKind::TsTypeParameter => "TSTypeParameter",
            NodeKind::TsConstraint => "TsConstraint",
            NodeKind::TsTypeParameterConstraint => "TsTypeParameterConstraint",
            NodeKind::TsTypeParameterDefault => "TsTypeParameterDefault",
            NodeKind::TsInterfaceBody => "TsInterfaceBody",
            NodeKind::TsPropertySignature => "TsPropertySignature",
            NodeKind::TsMethodSignature => "TsMethodSignature",
            NodeKind::TsIndexSignature => "TsIndexSignature",
            NodeKind::TsIndexSignatureAnnotation => "TsIndexSignatureAnnotation",
            NodeKind::TsCallSignatureDeclaration => "TsCallSignatureDeclaration",
            NodeKind::TsConstructSignatureDeclaration => "TsConstructSignatureDeclaration",
            NodeKind::TsPropertyParameter => "TsPropertyParameter",
            NodeKind::Program => "Program",
        }
    }

    pub fn is_declaration(&self) -> bool {
        matches!(self,
            NodeKind::FunctionDeclaration | NodeKind::ClassDeclaration |
            NodeKind::VariableDeclaration | NodeKind::ImportDeclaration |
            NodeKind::ExportNamedDeclaration
        )
    }
}

/// An AST builder.
pub struct AST {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

impl AST {
    /// Create a new AST.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }

    /// Create a new node with an auto-assigned ID.
    pub fn new_node(&mut self, kind: NodeKind, span: Option<Span>) -> NodeId {
        let id = NodeId::new(self.nodes.len() as u32);
        let mut node = Node::new(id, kind, span);
        self.nodes.push(node);
        id
    }

    /// Get a reference to a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.as_u32() as usize)
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.as_u32() as usize)
    }

    /// Set the root of the AST.
    pub fn set_root(&mut self, root: NodeId) {
        self.root = Some(root);
    }

    /// Get the root of the AST.
    pub fn root(&self) -> Option<&NodeId> {
        self.root.as_ref()
    }

    /// Get all nodes in the AST.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Get mutable access to all nodes.
    pub fn nodes_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }
}

impl Default for AST {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.as_u32(), 42);
        assert_eq!(id.to_string(), "n42");
    }

    #[test]
    fn test_ast_creation() {
        let mut ast = AST::new();
        let root_id = ast.new_node(NodeKind::Program, None);
        ast.set_root(root_id);

        let root = ast.get_node(root_id).unwrap();
        assert_eq!(root.kind, NodeKind::Program);

        assert!(ast.root().is_some());
    }
}
