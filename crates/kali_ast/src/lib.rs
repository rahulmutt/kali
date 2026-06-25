//! AST definitions for TypeScript/JavaScript.
//!
//! This crate defines the Abstract Syntax Tree node types
//! and implements arena-based allocation for efficient AST construction.

mod node;
mod builder;
mod statement;
mod declaration;
mod module;

pub use node::*;
pub use builder::*;
pub use statement::*;
pub use declaration::*;
pub use module::*;

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
    NullishAssign,
    AndAssign,
    OrAssign,
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
    pub type_name: String,
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
