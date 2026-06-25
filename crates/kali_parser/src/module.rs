//! Module parsing: import/export declarations and specifiers.

use crate::literal::unquote_string_literal;
use crate::Parser;
use kali_ast::{
    ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration, ExportSpecifier,
    ImportDeclaration, ImportName, ImportNamedSpecifier, ImportSpecifier, Statement,
};
use kali_lexer::TokenType;

impl Parser {
    pub(crate) fn parse_import_declaration(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        if self.stream.current_kind() == Some(&TokenType::StringLiteral) {
            let source = self
                .stream
                .advance()
                .map(|token| unquote_string_literal(&token.value))
                .unwrap_or_default();
            let _ = self.stream.accept(TokenType::Semicolon);
            return Some(Statement::ImportDeclaration(ImportDeclaration {
                specifiers: vec![ImportSpecifier::SideEffect],
                source,
            }));
        }

        let mut specifiers = Vec::new();
        let mut saw_default = false;

        if self.stream.current_kind() == Some(&TokenType::Type) {
            let _ = self.stream.advance();
            let type_specifiers = self.parse_import_named_specifiers();
            specifiers.push(ImportSpecifier::Type(type_specifiers));
        } else if self.stream.current_kind() == Some(&TokenType::Star) {
            if let Some(namespace) = self.parse_import_namespace_specifier() {
                specifiers.push(namespace);
            }
        } else if self.stream.current_kind() == Some(&TokenType::LeftBrace) {
            let named = self.parse_import_named_specifiers();
            specifiers.push(ImportSpecifier::Named(named));
        } else if self.stream.current_kind() == Some(&TokenType::Identifier) {
            let default_local = self
                .stream
                .advance()
                .map(|token| token.value)
                .unwrap_or_default();
            specifiers.push(ImportSpecifier::Default(default_local));
            saw_default = true;
        }

        if saw_default && self.stream.current_kind() == Some(&TokenType::Comma) {
            let _ = self.stream.advance();
            if self.stream.current_kind() == Some(&TokenType::LeftBrace) {
                let named = self.parse_import_named_specifiers();
                specifiers.push(ImportSpecifier::Named(named));
            } else if self.stream.current_kind() == Some(&TokenType::Star) {
                if let Some(namespace) = self.parse_import_namespace_specifier() {
                    specifiers.push(namespace);
                }
            }
        }

        if self.stream.current_kind() == Some(&TokenType::From) {
            let _ = self.stream.advance();
        }

        let source = match self.stream.current_kind() {
            Some(TokenType::StringLiteral) => self
                .stream
                .advance()
                .map(|token| unquote_string_literal(&token.value))
                .unwrap_or_default(),
            _ => "unknown".to_string(),
        };
        let _ = self.stream.accept(TokenType::Semicolon);

        if specifiers.is_empty() {
            specifiers.push(ImportSpecifier::SideEffect);
        }

        Some(Statement::ImportDeclaration(ImportDeclaration {
            specifiers,
            source,
        }))
    }

    pub(crate) fn parse_export_declaration(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        if self.stream.current_kind() == Some(&TokenType::Default) {
            let _ = self.stream.advance();
            let declaration = match self.stream.current_kind() {
                Some(TokenType::Async)
                    if self.stream.peek_next_kind() == Some(&TokenType::Function) =>
                {
                    self.parse_function_declaration_with_async(true, true)
                        .and_then(|statement| match statement {
                            Statement::FunctionDeclaration(function) => {
                                Some(ExportDefaultDeclaration::FunctionDeclaration(function))
                            }
                            _ => None,
                        })
                }
                Some(TokenType::Function) => self
                    .parse_function_declaration_with_async(false, true)
                    .and_then(|statement| match statement {
                        Statement::FunctionDeclaration(function) => {
                            Some(ExportDefaultDeclaration::FunctionDeclaration(function))
                        }
                        _ => None,
                    }),
                Some(TokenType::Class) => {
                    self.parse_class_declaration()
                        .and_then(|statement| match statement {
                            Statement::ClassDeclaration(class_declaration) => Some(
                                ExportDefaultDeclaration::ClassDeclaration(class_declaration),
                            ),
                            _ => None,
                        })
                }
                _ => Some(ExportDefaultDeclaration::Expression(
                    self.parse_expression(),
                )),
            }?;
            let _ = self.stream.accept(TokenType::Semicolon);
            return Some(Statement::ExportDefault(declaration));
        }

        if self.stream.current_kind() == Some(&TokenType::Async)
            && self.stream.peek_next_kind() == Some(&TokenType::Function)
        {
            return self.parse_function_declaration_with_async(true, false);
        }

        if self.stream.current_kind() == Some(&TokenType::Function) {
            return self.parse_function_declaration();
        }

        if self.stream.current_kind() == Some(&TokenType::Class) {
            return self.parse_class_declaration();
        }

        if self.stream.current_kind() == Some(&TokenType::Star) {
            let _ = self.stream.advance();
            if self.stream.current_kind() == Some(&TokenType::From) {
                let _ = self.stream.advance();
            }
            let source = match self.stream.current_kind() {
                Some(TokenType::StringLiteral) => self
                    .stream
                    .advance()
                    .map(|token| unquote_string_literal(&token.value))
                    .unwrap_or_default(),
                _ => "unknown".to_string(),
            };
            let _ = self.stream.accept(TokenType::Semicolon);
            return Some(Statement::ExportAll(ExportAllDeclaration { source }));
        }

        if self.stream.current_kind() == Some(&TokenType::LeftBrace) {
            let specifiers = self.parse_export_named_specifiers();
            let source = if self.stream.current_kind() == Some(&TokenType::From) {
                let _ = self.stream.advance();
                match self.stream.current_kind() {
                    Some(TokenType::StringLiteral) => self
                        .stream
                        .advance()
                        .map(|token| unquote_string_literal(&token.value)),
                    _ => None,
                }
            } else {
                None
            };
            let _ = self.stream.accept(TokenType::Semicolon);
            return Some(Statement::ExportNamed(ExportNamedDeclaration {
                specifiers,
                source,
            }));
        }

        self.parse_expression_statement()
    }

    pub(crate) fn parse_export_named_specifiers(&mut self) -> Vec<ExportSpecifier> {
        let mut specifiers = Vec::new();
        if self.stream.current_kind() != Some(&TokenType::LeftBrace) {
            return specifiers;
        }

        let _ = self.stream.advance();
        loop {
            match self.stream.current_kind() {
                Some(TokenType::RightBrace) => {
                    let _ = self.stream.advance();
                    break;
                }
                Some(TokenType::Identifier) | Some(TokenType::Default) => {
                    let local = self
                        .stream
                        .advance()
                        .map(|token| token.value)
                        .unwrap_or_default();
                    let mut exported = local.clone();

                    if self.stream.current_kind() == Some(&TokenType::As) {
                        let _ = self.stream.advance();
                        if matches!(
                            self.stream.current_kind(),
                            Some(TokenType::Identifier) | Some(TokenType::Default)
                        ) {
                            exported = self
                                .stream
                                .advance()
                                .map(|token| token.value)
                                .unwrap_or(local.clone());
                        }
                    }

                    specifiers.push(ExportSpecifier { local, exported });
                    let _ = self.stream.accept(TokenType::Comma);
                }
                _ => {
                    let _ = self.stream.advance();
                }
            }
        }

        specifiers
    }

    pub(crate) fn parse_import_named_specifiers(&mut self) -> Vec<ImportNamedSpecifier> {
        let mut specifiers = Vec::new();
        if self.stream.current_kind() != Some(&TokenType::LeftBrace) {
            return specifiers;
        }

        let _ = self.stream.advance();
        loop {
            match self.stream.current_kind() {
                Some(TokenType::RightBrace) => {
                    let _ = self.stream.advance();
                    break;
                }
                Some(TokenType::Identifier) => {
                    let imported = self
                        .stream
                        .advance()
                        .map(|token| token.value)
                        .unwrap_or_default();
                    let mut local = imported.clone();
                    let mut imported_name = None;

                    if self.stream.current_kind() == Some(&TokenType::As) {
                        let _ = self.stream.advance();
                        if self.stream.current_kind() == Some(&TokenType::Identifier) {
                            local = self
                                .stream
                                .advance()
                                .map(|token| token.value)
                                .unwrap_or(imported.clone());
                            imported_name = Some(ImportName::Identifier(imported));
                        }
                    }

                    specifiers.push(ImportNamedSpecifier {
                        local,
                        imported: imported_name,
                    });
                    let _ = self.stream.accept(TokenType::Comma);
                }
                _ => {
                    let _ = self.stream.advance();
                }
            }
        }

        specifiers
    }

    pub(crate) fn parse_import_namespace_specifier(&mut self) -> Option<ImportSpecifier> {
        if self.stream.current_kind() != Some(&TokenType::Star) {
            return None;
        }

        let _ = self.stream.advance();
        if self.stream.current_kind() == Some(&TokenType::As) {
            let _ = self.stream.advance();
            if self.stream.current_kind() == Some(&TokenType::Identifier) {
                let local = self
                    .stream
                    .advance()
                    .map(|token| token.value)
                    .unwrap_or_default();
                return Some(ImportSpecifier::Namespace(local));
            }
        }

        None
    }
}
