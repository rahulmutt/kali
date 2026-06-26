//! Linter for Kali source files.

mod control_flow;
mod fixes;
mod scope;
mod style;

use std::collections::{HashMap, HashSet};

use kali_ast::{BlockStatement, Statement};
use kali_common::FileId;
use kali_error::{_error_codes::w2, Diagnostic};
use kali_fmt::format_source;
use kali_lexer::{Lexer, Token, TokenType};
use kali_parser::Parser;

use crate::fixes::apply_fixes;
use crate::scope::collect_statements_declarations;

/// Lint the given source text.
pub fn lint(source: &str) -> Vec<Diagnostic> {
    lint_with_options(source, false).diagnostics
}

/// Lint the given source text and optionally apply safe fixes.
pub fn lint_with_options(source: &str, fix: bool) -> LintResult {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let lexer_result = lexer.lex_all();
    let mut diagnostics = lexer_result.diagnostics;
    let tokens = lexer_result.tokens;

    let mut parser = Parser::new(FileId::new(0), tokens.clone());
    let parsed = parser.parse(None);
    diagnostics.extend(parsed.diagnostics.clone());

    if diagnostics.iter().any(|diag| diag.is_error()) {
        return LintResult {
            diagnostics,
            fixed_source: None,
        };
    }

    let mut analyzer = Analyzer::new(source, tokens, parsed.statements);
    analyzer.run();
    diagnostics.extend(analyzer.diagnostics);

    let fixed_source = if fix {
        Some(apply_fixes(source, &analyzer.fix_plan))
    } else {
        None
    };

    LintResult {
        diagnostics,
        fixed_source,
    }
}

/// Lint result with optional fixed source.
#[derive(Debug, Clone)]
pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
    pub fixed_source: Option<String>,
}

#[derive(Default)]
pub(crate) struct FixPlan {
    pub(crate) var_tokens: HashSet<usize>,
    pub(crate) let_to_const_tokens: HashSet<usize>,
    pub(crate) eqeqeq_tokens: HashMap<usize, &'static str>,
    pub(crate) debugger_tokens: HashSet<usize>,
    pub(crate) unused_import_ranges: Vec<(usize, usize)>,
}

pub(crate) struct Analyzer {
    pub(crate) tokens: Vec<Token>,
    pub(crate) statements: Vec<Statement>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) fix_plan: FixPlan,
}

impl Analyzer {
    fn new(_source: &str, tokens: Vec<Token>, statements: Vec<Statement>) -> Self {
        Self {
            tokens,
            statements,
            diagnostics: Vec::new(),
            fix_plan: FixPlan::default(),
        }
    }

    fn run(&mut self) {
        let declared = self.collect_declared_names();
        let identifier_counts = self.count_identifier_tokens();
        self.check_no_var_and_prefer_const();
        self.check_explicit_any();
        self.check_no_console();
        self.check_no_empty_and_unreachable();
        self.check_debugger();
        self.check_eqeqeq();
        self.check_no_unused_vars(&declared, &identifier_counts);
        self.check_no_unused_imports(&identifier_counts);
        self.check_no_undef(&declared);
    }

    fn collect_declared_names(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        collect_statements_declarations(&self.statements, &mut counts);
        counts
    }

    fn count_identifier_tokens(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for token in &self.tokens {
            if token.kind == TokenType::Identifier {
                *counts.entry(token.value.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

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


#[cfg(test)]
#[path = "tests.rs"]
mod tests;
