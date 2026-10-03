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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClassBody {
    pub methods: Vec<MethodDefinition>,
    /// Names of the class-field declarations the parser skipped
    /// (`n = 0;`, `label: string;`). Kept so a member set can include them.
    #[serde(default)]
    pub field_names: Vec<String>,
    /// The body has a member with a computed key (`["foo"](){}`), which the
    /// parser skips, so `methods` and `field_names` do not name every member.
    #[serde(default)]
    pub has_computed_members: bool,
}

/// Method definition
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MethodDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Box<BlockStatement>>,
    pub is_async: bool,
    pub generator: bool,
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
