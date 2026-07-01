//! Declaration-keyword rules: no-var and prefer-const.

use kali_ast::{BlockStatement, Statement};
use kali_error::{_error_codes::w2, Diagnostic};
use kali_lexer::{Token, TokenType};

use crate::{Analyzer, FixPlan};

impl Analyzer {
    pub(crate) fn check_no_var_and_prefer_const(&mut self) {
        let mut let_tokens = self
            .tokens
            .iter()
            .enumerate()
            .filter_map(|(index, token)| match token.kind {
                TokenType::Var => Some((index, token.kind)),
                TokenType::Let => Some((index, token.kind)),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut declaration_index = 0usize;
        for statement in &self.statements {
            walk_statement_for_var_rules(
                statement,
                &self.tokens,
                &mut let_tokens,
                &mut declaration_index,
                &mut self.diagnostics,
                &mut self.fix_plan,
            );
        }
    }
}

fn walk_statement_for_var_rules(
    statement: &Statement,
    tokens: &[Token],
    let_tokens: &mut [(usize, TokenType)],
    declaration_index: &mut usize,
    diagnostics: &mut Vec<Diagnostic>,
    fix_plan: &mut FixPlan,
) {
    match statement {
        Statement::VariableDeclaration(decl) => {
            let token_index = let_tokens.get(*declaration_index).map(|(index, _)| *index);
            if let Some(token_index) = token_index {
                match decl.kind.as_str() {
                    "var" => {
                        diagnostics.push(
                            Diagnostic::warning(
                                w2::NO_VAR as u32,
                                "avoid `var`; use `let` or `const`",
                            )
                            .with_suggestion("replace `var` with `let`"),
                        );
                        fix_plan.var_tokens.insert(token_index);
                    }
                    "let" if decl.declarations.iter().any(|item| item.init.is_some()) => {
                        diagnostics.push(
                            Diagnostic::warning(
                                w2::PREFER_CONST as u32,
                                "prefer `const` when a binding is never reassigned",
                            )
                            .with_suggestion("replace `let` with `const`"),
                        );
                        fix_plan.let_to_const_tokens.insert(token_index);
                    }
                    _ => {}
                }
            }
            *declaration_index += 1;
        }
        Statement::BlockStatement(block) => {
            for inner in &block.body {
                walk_statement_for_var_rules(
                    inner,
                    tokens,
                    let_tokens,
                    declaration_index,
                    diagnostics,
                    fix_plan,
                );
            }
        }
        Statement::FunctionDeclaration(func) => {
            for inner in &func.body.body {
                walk_statement_for_var_rules(
                    inner,
                    tokens,
                    let_tokens,
                    declaration_index,
                    diagnostics,
                    fix_plan,
                );
            }
        }
        Statement::ClassDeclaration(class_decl) => {
            for method in &class_decl.body.methods {
                if let Some(body) = &method.body {
                    for inner in &body.body {
                        walk_statement_for_var_rules(
                            inner,
                            tokens,
                            let_tokens,
                            declaration_index,
                            diagnostics,
                            fix_plan,
                        );
                    }
                }
            }
        }
        Statement::IfStatement(stmt) => {
            for inner in &stmt.consequent.body {
                walk_statement_for_var_rules(
                    inner,
                    tokens,
                    let_tokens,
                    declaration_index,
                    diagnostics,
                    fix_plan,
                );
            }
            if let Some(alternate) = &stmt.alternate {
                for inner in &alternate.body {
                    walk_statement_for_var_rules(
                        inner,
                        tokens,
                        let_tokens,
                        declaration_index,
                        diagnostics,
                        fix_plan,
                    );
                }
            }
        }
        Statement::TryStatement(stmt) => {
            for inner in &stmt.block.body {
                walk_statement_for_var_rules(
                    inner,
                    tokens,
                    let_tokens,
                    declaration_index,
                    diagnostics,
                    fix_plan,
                );
            }
            if let Some(handler) = &stmt.handler {
                for inner in &handler.body.body {
                    walk_statement_for_var_rules(
                        inner,
                        tokens,
                        let_tokens,
                        declaration_index,
                        diagnostics,
                        fix_plan,
                    );
                }
            }
            if let Some(finalizer) = &stmt.finalizer {
                for inner in &finalizer.body {
                    walk_statement_for_var_rules(
                        inner,
                        tokens,
                        let_tokens,
                        declaration_index,
                        diagnostics,
                        fix_plan,
                    );
                }
            }
        }
        Statement::SwitchStatement(stmt) => {
            for case in &stmt.cases {
                for inner in &case.consequent {
                    walk_statement_for_var_rules(
                        inner,
                        tokens,
                        let_tokens,
                        declaration_index,
                        diagnostics,
                        fix_plan,
                    );
                }
            }
        }
        Statement::ForStatement(stmt) => {
            if let Some(kali_ast::ForInit::VariableDeclaration(decl)) = &stmt.init {
                check_variable_declaration_kind(
                    &decl.kind,
                    decl.declarations.iter().any(|item| item.init.is_some()),
                    let_tokens,
                    declaration_index,
                    diagnostics,
                    fix_plan,
                );
            }
            walk_statement_for_var_rules(
                &Statement::BlockStatement((*stmt.body).clone()),
                tokens,
                let_tokens,
                declaration_index,
                diagnostics,
                fix_plan,
            );
        }
        Statement::ForInStatement(stmt) => {
            if let kali_ast::ForInLefthand::VariableDeclaration(decl) = &stmt.left {
                check_variable_declaration_kind(
                    &decl.kind,
                    decl.declarations.iter().any(|item| item.init.is_some()),
                    let_tokens,
                    declaration_index,
                    diagnostics,
                    fix_plan,
                );
            }
            walk_statement_for_var_rules(
                &Statement::BlockStatement(BlockStatement { body: Vec::new() }),
                tokens,
                let_tokens,
                declaration_index,
                diagnostics,
                fix_plan,
            );
        }
        Statement::ForOfStatement(stmt) => {
            if let kali_ast::ForOfLefthand::VariableDeclaration(decl) = &stmt.left {
                check_variable_declaration_kind(
                    &decl.kind,
                    decl.declarations.iter().any(|item| item.init.is_some()),
                    let_tokens,
                    declaration_index,
                    diagnostics,
                    fix_plan,
                );
            }
            walk_statement_for_var_rules(
                &Statement::BlockStatement(BlockStatement { body: Vec::new() }),
                tokens,
                let_tokens,
                declaration_index,
                diagnostics,
                fix_plan,
            );
        }
        _ => {
            let _ = tokens;
        }
    }
}

fn check_variable_declaration_kind(
    kind: &str,
    has_initializer: bool,
    let_tokens: &mut [(usize, TokenType)],
    declaration_index: &mut usize,
    diagnostics: &mut Vec<Diagnostic>,
    fix_plan: &mut FixPlan,
) {
    let token_index = let_tokens.get(*declaration_index).map(|(index, _)| *index);
    if let Some(token_index) = token_index {
        match kind {
            "var" => {
                diagnostics.push(
                    Diagnostic::warning(w2::NO_VAR as u32, "avoid `var`; use `let` or `const`")
                        .with_suggestion("replace `var` with `let`"),
                );
                fix_plan.var_tokens.insert(token_index);
            }
            "let" if has_initializer => {
                diagnostics.push(
                    Diagnostic::warning(
                        w2::PREFER_CONST as u32,
                        "prefer `const` when a binding is never reassigned",
                    )
                    .with_suggestion("replace `let` with `const`"),
                );
                fix_plan.let_to_const_tokens.insert(token_index);
            }
            _ => {}
        }
    }
    *declaration_index += 1;
}
