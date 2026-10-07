//! Declaration node types: functions, classes, variables, types, enums.

use crate::{BlockStatement, Expression};

#[cfg(test)]
#[path = "declaration_tests.rs"]
mod declaration_tests;

/// Function declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    /// Default values, index-aligned with `params` (default-parameters spec
    /// §3.1, A-1). Empty when no parameter has a default; otherwise exactly
    /// `params.len()` long. Only the parser fills it, and `kali_cli`'s
    /// `default_params` pass empties it before any later stage runs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub defaults: Vec<Option<Box<Expression>>>,
    pub body: Box<BlockStatement>,
    pub is_async: bool,
    pub generator: bool,
}

/// Class declaration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassDeclaration {
    pub name: String,
    /// The first token after `extends`, if any (unresolved-member-call spec
    /// A-1). An identifier gives its text; any other token gives `""`.
    #[serde(default)]
    pub super_class: Option<String>,
    pub body: Box<ClassBody>,
}

/// Class body
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassBody {
    pub methods: Vec<MethodDefinition>,
    /// Names of the class-field declarations (`n = 0;`, `label: string;`),
    /// static or not, in source order.
    #[serde(default)]
    pub field_names: Vec<String>,
    /// The body has a member with a computed key (`["foo"](){}`), which the
    /// parser skips, so `methods` and `field_names` do not name every member.
    #[serde(default)]
    pub has_computed_members: bool,
    /// The field declarations with their initializers (class-instances spec A-2).
    #[serde(default)]
    pub fields: Vec<ClassField>,
    /// The body has a `#private` field or method, which the parser skips.
    #[serde(default)]
    pub has_private_members: bool,
    /// The body has a `static { … }` block, which the parser skips.
    #[serde(default)]
    pub has_static_block: bool,
}

/// A class field declaration (`n = 0;`, `static k = 1;`, `label: string;`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassField {
    pub name: String,
    /// The initializer; `None` for a field declared without one.
    pub value: Option<Expression>,
    pub is_static: bool,
}

/// What a class method definition defines.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MethodKind {
    #[default]
    Method,
    Get,
    Set,
}

/// Method definition
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MethodDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Box<BlockStatement>>,
    pub is_async: bool,
    pub generator: bool,
    #[serde(default)]
    pub kind: MethodKind,
    #[serde(default)]
    pub is_static: bool,
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
