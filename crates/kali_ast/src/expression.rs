//! Expression node types and the unified `Expression` enum.

use crate::{
    ArrayExpression, BlockStatement, ClassBody, JsxElement, JsxFragment, LiteralValue,
    ObjectExpression,
};

#[cfg(test)]
#[path = "expression_tests.rs"]
mod expression_tests;

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
    /// `Some(name)`: the property name JavaScript will read is statically
    /// known — always for dot access, and for a computed access only when the
    /// parser could read the index (a string or number literal, or a
    /// parenthesized, sequence-last or `+`/`-` unary form of one). `None`: a
    /// computed access whose index must be evaluated; `computed_index` is
    /// `Some`. There is no sentinel value — the parser used to fabricate one
    /// (register entry R-59), and this field is an `Option` so it cannot.
    #[serde(default)]
    pub property: Option<String>,
    /// Structured index expression for computed access `a[<expr>]`.
    ///
    /// `None` for dot access (`a.b`). `Some(expr)` for bracket access, so
    /// codegen can evaluate arithmetic/dynamic indices (e.g. `a[i + 1]`) at
    /// runtime rather than relying on `property`.
    #[serde(default)]
    pub computed_index: Option<Box<Expression>>,
}

impl MemberExpression {
    /// The property name of a DOT access, `None` for any computed access. For
    /// sites that are only correct on the dot form.
    pub fn dot_name(&self) -> Option<&str> {
        if self.computed_index.is_none() {
            self.property.as_deref()
        } else {
            None
        }
    }

    /// The statically known property name: dot access, or a computed access
    /// whose index the parser could read. `None` means the index must be
    /// evaluated, and a consumer that formats or compares the name must
    /// decline rather than substitute anything.
    pub fn static_name(&self) -> Option<&str> {
        self.property.as_deref()
    }
}

// ============== MISSING TYPES TO ADD ==============

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
    /// Synthetic `__kali_fn_{N}` identity assigned by the pre-resolver
    /// `name_anonymous_functions` AST pass (mirrors `FunctionExpression.id`,
    /// but arrows have no source-level named-function-expression syntax, so
    /// the parser always produces `None` here). `kali_types` and `kali_hir`
    /// both key on this name once it is assigned — see
    /// `crates/kali_cli/src/build/name_anon_functions.rs`.
    #[serde(default)]
    pub id: Option<String>,
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
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
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

// ============== MISSING TYPE: BinaryOperatorExpression (duplicate that was mentioned) ==============
// Note: This is intentionally removed - use BinaryExpression instead

// ============== MISSING TYPE: UnaryOperatorExpression (duplicate) ==============
// Note: This is intentionally removed - use UnaryExpression instead
