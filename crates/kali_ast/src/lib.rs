//! AST definitions for TypeScript/JavaScript.
//!
//! This crate defines the Abstract Syntax Tree node types
//! and implements arena-based allocation for efficient AST construction.

use kali_common::Span;

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExpressionStatement {
    pub expression: Box<Expression>,
}

/// Break statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BreakStatement {
    pub label: Option<String>,
}

/// Continue statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ContinueStatement {
    pub label: Option<String>,
}

/// With statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WithStatement {
    pub object: Expression,
    pub body: Box<Statement>,
}

/// Return statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReturnStatement {
    pub argument: Option<Expression>,
}

/// Labeled statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LabeledStatement {
    pub label: String,
    pub body: Box<Statement>,
}

/// If statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IfStatement {
    pub test: Expression,
    pub consequent: Box<BlockStatement>,
    pub alternate: Option<Box<BlockStatement>>,
}

/// Switch statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SwitchStatement {
    pub discriminant: Expression,
    pub cases: Vec<SwitchCase>,
}

/// Switch case
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SwitchCase {
    pub test: Option<Expression>,
    pub consequent: Vec<Statement>,
}

/// Throw statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ThrowStatement {
    pub argument: Expression,
}

/// Try statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TryStatement {
    pub block: Box<BlockStatement>,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStatement>,
}

/// Catch clause
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CatchClause {
    pub param: String,
    pub body: Box<BlockStatement>,
}

/// Debugger statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DebuggerStatement {}

/// Block statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BlockStatement {
    pub body: Vec<Statement>,
}

/// For statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ForStatement {
    pub init: Option<ForInit>,
    pub test: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Box<BlockStatement>,
}

/// For init
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ForInit {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
}

/// For-in statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ForInStatement {
    pub left: ForInLefthand,
    pub right: Expression,
    pub body: Box<Statement>,
}

/// For-in lefthand
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ForInLefthand {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
}

/// For-of statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ForOfStatement {
    pub left: ForOfLefthand,
    pub right: Expression,
    pub body: Box<Statement>,
}

/// For-of lefthand
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ForOfLefthand {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
}

/// While statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WhileStatement {
    pub test: Expression,
    pub body: Box<BlockStatement>,
}

/// Do-while statement
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DoWhileStatement {
    pub body: Box<BlockStatement>,
    pub test: Expression,
}

/// Function declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<BlockStatement>,
}

/// Class declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassDeclaration {
    pub name: String,
    pub body: Box<ClassBody>,
}

/// Class body
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassBody {
    pub methods: Vec<MethodDefinition>,
}

/// Method definition
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MethodDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Box<BlockStatement>>,
}

// Variable declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VariableDeclaration {
    pub declarations: Vec<VariableDeclarator>,
    pub kind: String, // var, let, const
}

// Variable declarator
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VariableDeclarator {
    pub id: String,
    pub init: Option<Expression>,
}

// Type alias declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeAliasDeclaration {
    pub name: String,
    pub type_params: Vec<String>,
    pub type_annotation: String,
}

// Interface declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InterfaceDeclaration {
    pub name: String,
    pub properties: Vec<PropertySignature>,
}

// Property signature
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PropertySignature {
    pub name: String,
    pub type_annotation: String,
}

// Enum declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnumDeclaration {
    pub name: String,
    pub members: Vec<EnumMember>,
}

// Enum member
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<Expression>,
}

/// The unified Statement enum covering all statement types.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Statement {
    ExpressionStatement(ExpressionStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    WithStatement(WithStatement),
    ReturnStatement(ReturnStatement),
    LabeledStatement(LabeledStatement),
    IfStatement(IfStatement),
    SwitchStatement(SwitchStatement),
    ThrowStatement(ThrowStatement),
    TryStatement(TryStatement),
    DebuggerStatement(DebuggerStatement),
    BlockStatement(BlockStatement),
    ForStatement(ForStatement),
    ForInStatement(ForInStatement),
    ForOfStatement(ForOfStatement),
    WhileStatement(WhileStatement),
    DoWhileStatement(DoWhileStatement),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ClassDeclaration),
    VariableDeclaration(VariableDeclaration),
    ImportDeclaration(ImportDeclaration),
    ExportNamed(ExportNamedDeclaration),
    ExportDefault(ExportDefaultDeclaration),
    EnumDeclaration(EnumDeclaration),
    TypeAliasDeclaration(TypeAliasDeclaration),
    InterfaceDeclaration(InterfaceDeclaration),
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

// Expression types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Expression {
    Identifier(String),
    Literal(LiteralValue),
    BinaryExpression(Box<BinaryExpression>),
    UnaryExpression(Box<UnaryExpression>),
    CallExpression(Box<CallExpression>),
    MemberExpression(Box<MemberExpression>),

    // Array and object literals
    ArrayExpression(ArrayExpression),
    ObjectExpression(ObjectExpression),

    // Function and class expressions
    FunctionExpression(Box<FunctionExpression>),
    ArrowFunctionExpression(Box<ArrowFunctionExpression>),
    ClassExpression(Box<ClassExpression>),

    // Object construction and calling
    NewExpression(Box<NewExpression>),
    MetaProperty(MetaProperty),

    // Template literals
    TemplateLiteral(TemplateLiteral),
    TaggedTemplateExpression(Box<TaggedTemplateExpression>),

    // Update and assignment
    UpdateExpression(Box<UpdateExpression>),
    AssignmentExpression(Box<AssignmentExpression>),

    // Logical and conditional
    LogicalExpression(Box<LogicalExpression>),
    ConditionalExpression(Box<ConditionalExpression>),

    // Sequence and group
    SequenceExpression(Box<SequenceExpression>),
    ParenthesizedExpression(Box<ParenthesizedExpression>),

    // Yield and await
    YieldExpression(Box<YieldExpression>),
    AwaitExpression(Box<AwaitExpression>),

    // Optional chaining
    OptionalChainExpression(Box<OptionalChainExpression>),
    ChainExpression(Box<ChainExpression>),

    // Spread and rest
    SpreadElement(Box<SpreadElement>),
    RestElement(Box<RestElement>),

    // Import
    ImportExpression(Box<ImportExpression>),

    // Decorators and attributes
    DecoratedExpression(DecoratedExpression),

    // JSX expressions
    JsxElement(JsxElement),
    JsxFragment(JsxFragment),
    JsxEmptyExpression,

    // TypeScript type assertions
    TypeAssertion(Box<TypeAssertion>),
    SatisfiesExpression(Box<SatisfiesExpression>),

    // Other expressions
    ThisExpression,
    SuperExpression,
    PrivateIdentifier(String),
    BigIntLiteral(String),
}

impl AsRef<Expression> for Expression {
    fn as_ref(&self) -> &Expression {
        self
    }
}

// Binary expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BinaryExpression {
    pub operator: String,
    pub left: Expression,
    pub right: Expression,
}

// Unary expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UnaryExpression {
    pub operator: String,
    pub argument: Expression,
}

// Call expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CallExpression {
    pub callee: Expression,
    pub args: Vec<Expression>,
}

// Member expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MemberExpression {
    pub object: Expression,
    pub property: String,
}

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

/// An AST builder. After parsing, nodes are allocated in arena-style storage.
pub struct ASTBuilder {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

/// Import declaration with multiple specifier types.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImportDeclaration {
    pub specifiers: Vec<ImportSpecifier>,
    pub source: String,
}

/// Import specifier variants.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ImportSpecifier {
    /// Default import: `import x from "mod"`
    Default(String),
    /// Named imports: `import { x, y } from "mod"`
    Named(Vec<ImportNamedSpecifier>),
    /// Namespace import: `import * as ns from "mod"`
    Namespace(String),
    /// Type-only import: `import type { X } from "mod"`
    Type(Vec<ImportNamedSpecifier>),
    /// Side-effect import: `import "mod"`
    SideEffect,
}

/// Named import specifier with optional alias.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImportNamedSpecifier {
    pub local: String,
    pub imported: Option<ImportName>, // Some if aliased
}

/// The name being imported (local or imported).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ImportName {
    Identifier(String),
    Alias(String),
}

/// Export declaration variants.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExportDeclaration {
    /// Named exports: `export { x, y as z }`
    NamedExport(ExportNamedDeclaration),
    /// All exports re-export: `export * from "mod"`
    ExportAll(ExportAllDeclaration),
    /// Default export: `export default expr` or `export default class C {}`
    Default(ExportDefaultDeclaration),
    /// Type-only exports: `export type { X }`
    TypeExport(ExportTypeDeclaration),
}

/// Named export declaration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExportNamedDeclaration {
    pub specifiers: Vec<ExportSpecifier>,
    pub source: Option<String>,
}

/// Export specifier with local and exported names.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExportSpecifier {
    pub local: String,
    pub exported: String,
}

/// Export all re-export declaration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExportAllDeclaration {
    pub source: String,
}

/// Default export declaration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum ExportDefaultDeclaration {
    Expression(Expression),
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ClassDeclaration),
}

/// Type-only export declaration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExportTypeDeclaration {
    pub specifiers: Vec<ExportSpecifier>,
    pub source: Option<String>,
}

impl ASTBuilder {
    /// Create a new AST builder.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }

    /// Create a new node with an auto-assigned ID.
    pub fn new_node(&mut self, kind: NodeKind, span: Option<Span>) -> NodeId {
        let id = NodeId::new(self.nodes.len() as u32);
        self.nodes.push(Node::new(id, kind, span));
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

    /// Finalize the AST and consume the builder.
    pub fn into_ast(self) -> AST {
        AST {
            nodes: self.nodes,
            root: self.root,
        }
    }
}

impl Default for ASTBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed AST with source-level tree structure.
pub struct AST {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

impl AST {
    /// Create a new AST from nodes and root.
    pub fn new(nodes: Vec<Node>, root: Option<NodeId>) -> Self {
        Self { nodes, root }
    }

    /// Get a reference to a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.as_u32() as usize)
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.as_u32() as usize)
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

    /// Create an empty AST.
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }
}

impl Default for AST {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::convert::From<ASTBuilder> for AST {
    fn from(builder: ASTBuilder) -> Self {
        builder.into_ast()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

// ============== MISSING TYPES TO ADD ==============

/// Literal value types for Literal expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LiteralValue {
    Boolean(bool),
    Number(f64),
    String(String),
    Regex { pattern: String, flags: String },
    Null,
}

/// Array expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArrayExpression {
    pub elements: Vec<Option<ExpressionOrSpread>>,
}

/// Object expression  
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObjectExpression {
    pub properties: Vec<ObjectProperty>,
}

/// Object property
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObjectProperty {
    pub key: PropertyName,
    pub value: Expression,
    pub kind: ObjectPropertyKind,
}

/// Property name
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PropertyName {
    Identifier(String),
    Number(f64),
    String(String),
}

/// Object property kind
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ObjectPropertyKind {
    Init,
    Get,
    Set,
}

/// Function expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionExpression {
    pub id: Option<String>,
    pub params: Vec<FunctionParam>,
    pub body: Option<Box<BlockStatement>>,
    pub is_async: bool,
    pub generator: bool,
}

/// Function param
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionParam {
    pub name: String,
}

/// Arrow function expression
#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArrowFunctionExpression {
    pub params: Vec<FunctionParam>,
    pub body: Expression,
    pub is_async: bool,
    pub returnType: Option<String>,
}

/// Class expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassExpression {
    pub id: Option<String>,
    pub body: Box<ClassBody>,
}

/// New expression  
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NewExpression {
    pub callee: Expression,
    pub args: Vec<Expression>,
}

/// Meta property
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetaProperty {
    pub meta: String,
    pub property: String,
}

/// Template literal
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TemplateLiteral {
    pub quasis: Vec<TemplateElement>,
    pub expressions: Vec<Expression>,
}

/// Template element
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TemplateElement {
    pub value: String,
    pub tail: bool,
}

/// Tagged template expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TaggedTemplateExpression {
    pub tag: Expression,
    pub template: TemplateLiteral,
}

/// Update expression (++, --)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateExpression {
    pub operator: UpdateOperator,
    pub argument: Expression,
    pub prefix: bool,
}

/// Update operator
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum UpdateOperator {
    Increment,
    Decrement,
}

/// Assignment expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssignmentExpression {
    pub operator: AssignmentOperator,
    pub left: Expression,
    pub right: Expression,
}

/// Assignment operator
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    ExponentAssign,
}

/// Logical expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LogicalExpression {
    pub operator: LogicalOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

/// Logical operator
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Coalesce,
}

/// Conditional expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConditionalExpression {
    pub test: Box<Expression>,
    pub consequent: Box<Expression>,
    pub alternate: Box<Expression>,
}

/// Sequence expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SequenceExpression {
    pub expressions: Vec<Expression>,
}

/// Parenthesized expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ParenthesizedExpression {
    pub expression: Box<Expression>,
}

/// Yield expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct YieldExpression {
    pub delegate: bool,
    pub argument: Option<Expression>,
}

/// Await expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AwaitExpression {
    pub argument: Expression,
}

/// Optional chain expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OptionalChainExpression {
    pub inner: Box<OptionalChainInner>,
}

/// Optional chain inner
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OptionalChainInner {
    NonNull {
        object: Box<Expression>,
        optional: bool,
    },
}

/// Chain expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChainExpression {
    pub expression: Box<Expression>,
}

/// Spread element
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpreadElement {
    pub argument: Expression,
}

/// Rest element
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RestElement {
    pub argument: Expression,
}

/// Import expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImportExpression {
    pub source: Expression,
}

/// Decorated expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DecoratedExpression {
    pub expression: Box<Expression>,
}

/// JSX element
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxElement {
    pub opening_element: JsxOpeningElement,
    pub children: Vec<JsxChild>,
    pub closing_element: Option<JsxClosingElement>,
}

/// JSX opening element
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxOpeningElement {
    pub name: JsxName,
    pub attributes: Vec<JsxAttributeItem>,
}

/// JSX child
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JsxChild {
    JsxText(String),
    JsxExpression(JsxExpressionContainer),
    JsxElement(Box<JsxElement>),
    JsxFragment(Box<JsxFragment>),
}

/// JSX expression container
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxExpressionContainer {
    pub expression: Option<Expression>,
}

/// JSX fragment
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxFragment {
    pub children: Vec<JsxChild>,
}

/// JSX name
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JsxName {
    Identifier(String),
    JsxClosedElement(Box<JsxClosingElement>),
}

/// JSX attribute item
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JsxAttributeItem {
    JsxAttribute(JsxAttribute),
    JsxSpreadAttribute(Box<JsxSpreadAttribute>),
}

/// JSX attribute
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxAttribute {
    pub name: JsxName,
    pub value: JsxAttributeValue,
}

/// JSX attribute value
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JsxAttributeValue {
    String(String),
    JsxElement(Box<JsxElement>),
    JsxExpression(JsxExpressionContainer),
}

/// JSX spread attribute
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxSpreadAttribute {
    pub argument: Expression,
}

/// Type assertion
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeAssertion {
    pub type_name: String,
    pub expression: Box<Expression>,
}

/// Satisfies expression
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SatisfiesExpression {
    pub expression: Box<Expression>,
}

// ============== END OF NEW TYPES ==============

// ============== MISSING TYPE: ExpressionOrSpread ==============

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExpressionOrSpread {
    Expression(Expression),
    Spread(SpreadElement),
    Empty,
}

// ============== MISSING TYPE: BinaryOperatorExpression (duplicate that was mentioned) ==============
// Note: This is intentionally removed - use BinaryExpression instead

// ============== MISSING TYPE: UnaryOperatorExpression (duplicate) ==============
// Note: This is intentionally removed - use UnaryExpression instead

// ============== MISSING TYPE ==============

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxSelfClosingElement {
    pub name: JsxName,
    pub attributes: Vec<JsxAttributeItem>,
}

// END OF MISSING TYPES
// Add this type after JsxSelfClosingElement definition
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxClosingElement {
    pub name: JsxName,
}
