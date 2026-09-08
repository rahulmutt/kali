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
    /// A BigInt literal key, as its decimal digits without the `n` suffix.
    ///
    /// Text, not a parsed value: `{123456789012345678901234567890n: 1}` has no
    /// exact `f64`, and the property name JavaScript computes for it is the
    /// exact digits (`String(123456789012345678901234567890n)`).
    BigInt(String),
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
