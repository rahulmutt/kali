//! Statement lowering: the `lower_statement` dispatcher + block/class/declarator helpers.

use crate::node::{HirNodeId, HirNodeKind};
use crate::result::FunctionFlavor;
use crate::HirLowerer;
use kali_ast::{
    BlockStatement, BreakStatement, CatchClause, ClassBody, ClassDeclaration, ContinueStatement,
    DebuggerStatement, DoWhileStatement, EnumDeclaration, ExportAllDeclaration,
    ExportNamedDeclaration, ExpressionStatement, ForInLefthand, ForInStatement, ForInit,
    ForOfLefthand, ForOfStatement, ForStatement, FunctionDeclaration, IfStatement,
    ImportDeclaration, InterfaceDeclaration, LabeledStatement, MethodDefinition, ReturnStatement,
    Statement, SwitchStatement, ThrowStatement, TryStatement, TypeAliasDeclaration,
    VariableDeclaration, VariableDeclarator, WhileStatement, WithStatement,
};

impl HirLowerer {
    pub(crate) fn lower_statement(&mut self, statement: &Statement) -> HirNodeId {
        match statement {
            Statement::ExpressionStatement(ExpressionStatement { expression }) => {
                let id = self.builder.alloc(HirNodeKind::ExprStmt, None);
                let child = self.lower_expression(expression);
                push_child!(self, id, child);
                id
            }
            Statement::BreakStatement(BreakStatement { label }) => self.builder.alloc_text(
                HirNodeKind::BreakStmt,
                None,
                match label {
                    Some(label) if !label.is_empty() => format!("break:{label}"),
                    _ => "break".to_string(),
                },
            ),
            Statement::ContinueStatement(ContinueStatement { label }) => self.builder.alloc_text(
                HirNodeKind::ContinueStmt,
                None,
                match label {
                    Some(label) if !label.is_empty() => format!("continue:{label}"),
                    _ => "continue".to_string(),
                },
            ),
            Statement::WithStatement(WithStatement { object, body }) => {
                let id = self.builder.alloc(HirNodeKind::WithStmt, None);
                push_child!(self, id, self.lower_expression(object));
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::ReturnStatement(ReturnStatement { argument }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::ReturnStmt, None, "return");
                if let Some(arg) = argument {
                    push_child!(self, id, self.lower_expression(arg));
                }
                id
            }
            Statement::LabeledStatement(LabeledStatement { label, body }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::LabeledStmt, None, label.clone());
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::IfStatement(IfStatement {
                test,
                consequent,
                alternate,
            }) => {
                let id = self.builder.alloc(HirNodeKind::IfStmt, None);
                push_child!(self, id, self.lower_expression(test));
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**consequent).clone()))
                );
                if let Some(alt) = alternate {
                    push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::BlockStatement((**alt).clone()))
                    );
                }
                id
            }
            Statement::SwitchStatement(SwitchStatement {
                discriminant,
                cases,
            }) => {
                let id = self.builder.alloc(HirNodeKind::SwitchStmt, None);
                push_child!(self, id, self.lower_expression(discriminant));
                for case in cases {
                    let case_id = self.builder.alloc(HirNodeKind::Block, None);
                    if let Some(test) = &case.test {
                        push_child!(self, case_id, self.lower_expression(test));
                    }
                    for stmt in &case.consequent {
                        push_child!(self, case_id, self.lower_statement(stmt));
                    }
                    push_child!(self, id, case_id);
                }
                id
            }
            Statement::ThrowStatement(ThrowStatement { argument }) => {
                let id = self.builder.alloc(HirNodeKind::ThrowStmt, None);
                push_child!(self, id, self.lower_expression(argument));
                id
            }
            Statement::TryStatement(TryStatement {
                block,
                handler,
                finalizer,
            }) => {
                if handler.is_none() {
                    if let Some(finalizer) = finalizer {
                        let id = self.builder.alloc(HirNodeKind::Block, None);
                        push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::BlockStatement((**block).clone()))
                        );
                        push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::BlockStatement(finalizer.clone()))
                        );
                        id
                    } else {
                        self.lower_statement(&Statement::BlockStatement((**block).clone()))
                    }
                } else {
                    let id = self.builder.alloc(HirNodeKind::TryStmt, None);
                    push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::BlockStatement((**block).clone()))
                    );
                    if let Some(CatchClause { param, body }) = handler {
                        let catch_id =
                            self.builder
                                .alloc_text(HirNodeKind::Block, None, param.clone());
                        push_child!(
                            self,
                            catch_id,
                            self.lower_statement(&Statement::BlockStatement((**body).clone()))
                        );
                        push_child!(self, id, catch_id);
                    }
                    if let Some(finalizer) = finalizer {
                        push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::BlockStatement(finalizer.clone()))
                        );
                    }
                    id
                }
            }
            Statement::DebuggerStatement(DebuggerStatement {}) => {
                self.builder.alloc(HirNodeKind::DebuggerStmt, None)
            }
            Statement::BlockStatement(block) => self.lower_block(block),
            Statement::ForStatement(ForStatement {
                init,
                test,
                update,
                body,
            }) => {
                let id = self.builder.alloc_text(HirNodeKind::ForStmt, None, "for");
                if let Some(init) = init {
                    match init {
                        ForInit::VariableDeclaration(v) => push_child!(
                            self,
                            id,
                            self.lower_statement(&Statement::VariableDeclaration(v.clone()))
                        ),
                        ForInit::Expression(e) => push_child!(self, id, self.lower_expression(e)),
                    }
                }
                if let Some(test) = test {
                    push_child!(self, id, self.lower_expression(test));
                }
                if let Some(update) = update {
                    push_child!(self, id, self.lower_expression(update));
                }
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                id
            }
            Statement::ForInStatement(ForInStatement { left, right, body }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::ForInStmt, None, "for-in");
                match left {
                    ForInLefthand::VariableDeclaration(v) => push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::VariableDeclaration(v.clone()))
                    ),
                    ForInLefthand::Expression(e) => push_child!(self, id, self.lower_expression(e)),
                }
                push_child!(self, id, self.lower_expression(right));
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::ForOfStatement(ForOfStatement {
                left,
                right,
                body,
                is_await,
                ..
            }) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::ForOfStmt,
                    None,
                    if *is_await { "for-await-of" } else { "for-of" },
                );
                match left {
                    ForOfLefthand::VariableDeclaration(v) => push_child!(
                        self,
                        id,
                        self.lower_statement(&Statement::VariableDeclaration(v.clone()))
                    ),
                    ForOfLefthand::Expression(e) => push_child!(self, id, self.lower_expression(e)),
                }
                push_child!(self, id, self.lower_expression(right));
                push_child!(self, id, self.lower_statement(body));
                id
            }
            Statement::WhileStatement(WhileStatement { test, body }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::WhileStmt, None, "while");
                push_child!(self, id, self.lower_expression(test));
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                id
            }
            Statement::DoWhileStatement(DoWhileStatement { body, test }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::DoWhileStmt, None, "do-while");
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                push_child!(self, id, self.lower_expression(test));
                id
            }
            Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                params,
                body,
                is_async,
                generator,
            }) => {
                let name = if name.is_empty() {
                    self.next_synthetic_function_name()
                } else {
                    name.clone()
                };
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::FunctionDecl, None, name);
                for param in params {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, param.clone())
                    );
                }
                push_child!(
                    self,
                    id,
                    self.lower_statement(&Statement::BlockStatement((**body).clone()))
                );
                self.record_function_flavor(id, FunctionFlavor::from_flags(*is_async, *generator));
                id
            }
            Statement::ClassDeclaration(ClassDeclaration { name, body }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::ClassDecl, None, name.clone());
                push_child!(self, id, self.lower_class_body(body));
                id
            }
            Statement::VariableDeclaration(VariableDeclaration { declarations, kind }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::VarDecl, None, kind.clone());
                for decl in declarations {
                    push_child!(self, id, self.lower_variable_declarator(decl));
                }
                id
            }
            Statement::ImportDeclaration(ImportDeclaration { specifiers, source }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::ImportDecl, None, source.clone());
                for specifier in specifiers {
                    push_child!(self, id, self.lower_import_specifier(specifier));
                }
                id
            }
            Statement::ExportAll(ExportAllDeclaration { source }) => {
                self.builder
                    .alloc_text(HirNodeKind::ExportDecl, None, source.clone())
            }
            Statement::ExportNamed(ExportNamedDeclaration { specifiers, source }) => {
                let id = self.builder.alloc_text(
                    HirNodeKind::ExportDecl,
                    None,
                    source.clone().unwrap_or_default(),
                );
                for specifier in specifiers {
                    push_child!(self, id, self.lower_export_specifier(specifier));
                }
                id
            }
            Statement::ExportDefault(default_decl) => self.lower_export_default(default_decl),
            Statement::EnumDeclaration(EnumDeclaration { name, members }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::EnumDecl, None, name.clone());
                for member in members {
                    let member_id =
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, member.name.clone());
                    if let Some(value) = &member.value {
                        push_child!(self, member_id, self.lower_expression(value));
                    }
                    push_child!(self, id, member_id);
                }
                id
            }
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name,
                type_params,
                type_annotation,
            }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::TypeDecl, None, name.clone());
                for param in type_params {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, param.clone())
                    );
                }
                push_child!(
                    self,
                    id,
                    self.builder
                        .alloc_text(HirNodeKind::Literal, None, type_annotation.clone())
                );
                id
            }
            Statement::InterfaceDeclaration(InterfaceDeclaration { name, properties }) => {
                let id = self
                    .builder
                    .alloc_text(HirNodeKind::InterfaceDecl, None, name.clone());
                for property in properties {
                    push_child!(
                        self,
                        id,
                        self.builder
                            .alloc_text(HirNodeKind::Ident, None, property.name.clone())
                    );
                }
                id
            }
        }
    }

    pub(crate) fn lower_class_body(&mut self, body: &ClassBody) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::Block, None);
        for method in &body.methods {
            push_child!(self, id, self.lower_method_definition(method));
        }
        id
    }

    pub(crate) fn lower_method_definition(&mut self, method: &MethodDefinition) -> HirNodeId {
        let id = self
            .builder
            .alloc_text(HirNodeKind::FunctionDecl, None, method.name.clone());
        for param in &method.params {
            push_child!(
                self,
                id,
                self.builder
                    .alloc_text(HirNodeKind::Ident, None, param.clone())
            );
        }
        if let Some(body) = &method.body {
            push_child!(
                self,
                id,
                self.lower_statement(&Statement::BlockStatement((**body).clone()))
            );
        }
        self.record_function_flavor(
            id,
            FunctionFlavor::from_flags(method.is_async, method.generator),
        );
        id
    }

    pub(crate) fn lower_block(&mut self, block: &BlockStatement) -> HirNodeId {
        let id = self.builder.alloc(HirNodeKind::Block, None);
        for stmt in &block.body {
            push_child!(self, id, self.lower_statement(stmt));
        }
        id
    }

    pub(crate) fn lower_variable_declarator(
        &mut self,
        declarator: &VariableDeclarator,
    ) -> HirNodeId {
        let id = self
            .builder
            .alloc_text(HirNodeKind::VarDeclarator, None, declarator.id.clone());
        push_child!(
            self,
            id,
            self.builder
                .alloc_text(HirNodeKind::Ident, None, declarator.id.clone())
        );
        if let Some(init) = &declarator.init {
            push_child!(self, id, self.lower_expression(init));
        }
        id
    }
}

#[cfg(test)]
#[path = "statement_tests.rs"]
mod statement_tests;
