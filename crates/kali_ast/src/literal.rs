//! Literal values and array/object literal expressions.

use crate::{Expression, SpreadElement};

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExpressionOrSpread {
    Expression(Expression),
    Spread(SpreadElement),
    Empty,
}
