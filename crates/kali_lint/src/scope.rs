//! Scope rules: unused variables, unused imports, undefined references,
//! and the AST declaration-collection used to feed them.

use std::collections::{HashMap, HashSet};

use kali_ast::{BlockStatement, Statement};
use kali_error::{_error_codes::w2, Diagnostic};
use kali_lexer::{Token, TokenType};

use crate::Analyzer;

impl Analyzer {
    pub(crate) fn check_no_unused_vars(
        &mut self,
        declared: &HashMap<String, usize>,
        identifier_counts: &HashMap<String, usize>,
    ) {
        for (name, decl_count) in declared {
            let used_count = identifier_counts.get(name).copied().unwrap_or(0);
            if used_count <= *decl_count {
                self.diagnostics.push(Diagnostic::warning(
                    w2::UNUSED_VARIABLE as u32,
                    format!("`{}` is declared but never used", name),
                ));
            }
        }
    }

    pub(crate) fn check_no_unused_imports(&mut self, identifier_counts: &HashMap<String, usize>) {
        let import_ranges = collect_import_ranges(&self.tokens);
        for import_range in import_ranges {
            let mut names = HashSet::new();
            for token in self.tokens[import_range.0..=import_range.1].iter() {
                if token.kind == TokenType::Identifier {
                    names.insert(token.value.clone());
                }
            }

            for name in names {
                let count = identifier_counts.get(&name).copied().unwrap_or(0);
                if count <= 1 {
                    self.diagnostics.push(Diagnostic::warning(
                        w2::UNUSED_IMPORT as u32,
                        format!("import `{}` is never used", name),
                    ));
                    self.fix_plan.unused_import_ranges.push(import_range);
                }
            }
        }
    }

    pub(crate) fn check_no_undef(&mut self, declared: &HashMap<String, usize>) {
        let builtins = builtin_globals();
        for (index, token) in self.tokens.iter().enumerate() {
            if token.kind != TokenType::Identifier {
                continue;
            }

            if declared.contains_key(&token.value) || builtins.contains(token.value.as_str()) {
                continue;
            }

            if matches!(
                self.tokens.get(index.wrapping_sub(1)).map(|t| t.kind),
                Some(TokenType::Dot)
            ) {
                continue;
            }
            if matches!(
                self.tokens.get(index + 1).map(|t| t.kind),
                Some(TokenType::Colon)
            ) {
                continue;
            }
            if matches!(
                self.tokens.get(index.wrapping_sub(1)).map(|t| t.kind),
                Some(
                    TokenType::Import
                        | TokenType::Export
                        | TokenType::From
                        | TokenType::As
                        | TokenType::Function
                        | TokenType::Class
                        | TokenType::Var
                        | TokenType::Let
                        | TokenType::Const
                        | TokenType::Catch
                        | TokenType::Type
                        | TokenType::Interface
                        | TokenType::Enum
                )
            ) {
                continue;
            }

            self.diagnostics.push(Diagnostic::warning(
                w2::NO_UNDEF as u32,
                format!("`{}` is not defined in this scope", token.value),
            ));
        }
    }
}

pub(crate) fn collect_statements_declarations(
    statements: &[Statement],
    counts: &mut HashMap<String, usize>,
) {
    for statement in statements {
        collect_statement_declarations(statement, counts);
    }
}

fn collect_statement_declarations(statement: &Statement, counts: &mut HashMap<String, usize>) {
    match statement {
        Statement::VariableDeclaration(decl) => {
            for item in &decl.declarations {
                *counts.entry(item.id.clone()).or_insert(0) += 1;
            }
        }
        Statement::FunctionDeclaration(func) => {
            *counts.entry(func.name.clone()).or_insert(0) += 1;
            for param in &func.params {
                *counts.entry(param.clone()).or_insert(0) += 1;
            }
            collect_block_declarations(&func.body, counts);
        }
        Statement::ClassDeclaration(class_decl) => {
            *counts.entry(class_decl.name.clone()).or_insert(0) += 1;
            for method in &class_decl.body.methods {
                for param in &method.params {
                    *counts.entry(param.clone()).or_insert(0) += 1;
                }
                if let Some(body) = &method.body {
                    collect_block_declarations(body, counts);
                }
            }
        }
        Statement::BlockStatement(block) => collect_block_declarations(block, counts),
        Statement::IfStatement(stmt) => {
            collect_statements_declarations(&stmt.consequent.body, counts);
            if let Some(alternate) = &stmt.alternate {
                collect_statements_declarations(&alternate.body, counts);
            }
        }
        Statement::SwitchStatement(stmt) => {
            for case in &stmt.cases {
                collect_statements_declarations(&case.consequent, counts);
            }
        }
        Statement::TryStatement(stmt) => {
            collect_block_declarations(&stmt.block, counts);
            if let Some(handler) = &stmt.handler {
                *counts.entry(handler.param.clone()).or_insert(0) += 1;
                collect_block_declarations(&handler.body, counts);
            }
            if let Some(finalizer) = &stmt.finalizer {
                collect_block_declarations(finalizer, counts);
            }
        }
        Statement::ForStatement(stmt) => {
            if let Some(init) = &stmt.init {
                match init {
                    kali_ast::ForInit::VariableDeclaration(decl) => {
                        for item in &decl.declarations {
                            *counts.entry(item.id.clone()).or_insert(0) += 1;
                        }
                    }
                    kali_ast::ForInit::Expression(_) => {}
                }
            }
            collect_block_declarations(&stmt.body, counts);
        }
        Statement::ForInStatement(stmt) => match &stmt.left {
            kali_ast::ForInLefthand::VariableDeclaration(decl) => {
                for item in &decl.declarations {
                    *counts.entry(item.id.clone()).or_insert(0) += 1;
                }
            }
            kali_ast::ForInLefthand::Expression(_) => {}
        },
        Statement::ForOfStatement(stmt) => match &stmt.left {
            kali_ast::ForOfLefthand::VariableDeclaration(decl) => {
                for item in &decl.declarations {
                    *counts.entry(item.id.clone()).or_insert(0) += 1;
                }
            }
            kali_ast::ForOfLefthand::Expression(_) => {}
        },
        _ => {}
    }
}

fn collect_block_declarations(block: &BlockStatement, counts: &mut HashMap<String, usize>) {
    collect_statements_declarations(&block.body, counts);
}

fn collect_import_ranges(tokens: &[Token]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind == TokenType::Import {
            let start = index;
            let mut end = index;
            while end + 1 < tokens.len() && tokens[end].kind != TokenType::Semicolon {
                end += 1;
            }
            ranges.push((start, end));
            index = end.saturating_add(1);
        } else {
            index += 1;
        }
    }
    ranges
}

fn builtin_globals() -> HashSet<&'static str> {
    [
        "console",
        "Math",
        "Array",
        "String",
        "Number",
        "Boolean",
        "Object",
        "Promise",
        "JSON",
        "Date",
        "ReadableStream",
        "TransformStream",
        "WritableStream",
        "RegExp",
        "Map",
        "Set",
        "WeakMap",
        "WeakSet",
        "Error",
        "Reflect",
        "Symbol",
        "Intl",
        "globalThis",
        "undefined",
        "NaN",
        "Infinity",
    ]
    .into_iter()
    .collect()
}
