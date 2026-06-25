//! ES module syntax: import and export declarations.

use crate::{ClassDeclaration, Expression, FunctionDeclaration};

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
