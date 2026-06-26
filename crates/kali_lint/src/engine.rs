//! Lint driver: public entry points, the shared `Analyzer`/`FixPlan` state,
//! and the `run()` orchestration that sequences the rule checks.

use std::collections::{HashMap, HashSet};

use kali_ast::Statement;
use kali_common::FileId;
use kali_error::Diagnostic;
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
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod engine_tests;
