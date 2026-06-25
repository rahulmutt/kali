//! JSX node types.

use crate::Expression;

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxSelfClosingElement {
    pub name: JsxName,
    pub attributes: Vec<JsxAttributeItem>,
}

// Add this type after JsxSelfClosingElement definition
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JsxClosingElement {
    pub name: JsxName,
}
