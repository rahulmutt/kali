//! Import/export specifier and default-export lowering.

use crate::node::{HirNodeId, HirNodeKind};
use crate::HirLowerer;
use kali_ast::{ExportDefaultDeclaration, ExportSpecifier, ImportSpecifier, Statement};

impl HirLowerer {
    pub(crate) fn lower_import_specifier(&mut self, specifier: &ImportSpecifier) -> HirNodeId {
        match specifier {
            ImportSpecifier::Default(name) | ImportSpecifier::Namespace(name) => self
                .builder
                .alloc_text(HirNodeKind::Ident, None, name.clone()),
            ImportSpecifier::Named(specifiers) | ImportSpecifier::Type(specifiers) => {
                let id = self.builder.alloc(HirNodeKind::ImportDecl, None);
                for spec in specifiers {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, spec.local.clone())
                    );
                }
                id
            }
            ImportSpecifier::SideEffect => {
                self.builder
                    .alloc_text(HirNodeKind::Literal, None, "side-effect")
            }
        }
    }

    pub(crate) fn lower_export_specifier(&mut self, specifier: &ExportSpecifier) -> HirNodeId {
        let id = self
            .builder
            .alloc_text(HirNodeKind::Ident, None, specifier.exported.clone());
        push_child!(
            self,
            id,
            self.builder
                .alloc_text(HirNodeKind::Ident, None, specifier.local.clone())
        );
        id
    }

    pub(crate) fn lower_export_default(
        &mut self,
        default_decl: &ExportDefaultDeclaration,
    ) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::ExportDecl, None);
        match default_decl {
            ExportDefaultDeclaration::Expression(expr) => {
                push_child!(self, id, self.lower_expression(expr))
            }
            ExportDefaultDeclaration::FunctionDeclaration(func) => push_child!(
                self,
                id,
                self.lower_statement(&Statement::FunctionDeclaration(func.clone()))
            ),
            ExportDefaultDeclaration::ClassDeclaration(class) => push_child!(
                self,
                id,
                self.lower_statement(&Statement::ClassDeclaration(class.clone()))
            ),
        }
        id
    }
}
