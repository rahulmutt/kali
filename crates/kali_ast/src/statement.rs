//! Statement node types and the unified `Statement` enum.

use crate::{
    ClassDeclaration, EnumDeclaration, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportNamedDeclaration, Expression, FunctionDeclaration, ImportDeclaration,
    InterfaceDeclaration, TypeAliasDeclaration, VariableDeclaration,
};

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
    pub is_await: bool,
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
    ExportAll(ExportAllDeclaration),
    ExportNamed(ExportNamedDeclaration),
    ExportDefault(ExportDefaultDeclaration),
    EnumDeclaration(EnumDeclaration),
    TypeAliasDeclaration(TypeAliasDeclaration),
    InterfaceDeclaration(InterfaceDeclaration),
}
