//! Linter for Kali source files.

use std::collections::{HashMap, HashSet};

use kali_ast::{BlockStatement, Statement};
use kali_common::FileId;
use kali_error::{_error_codes::w2, Diagnostic};
use kali_fmt::format_source;
use kali_lexer::{Lexer, Token, TokenType};
use kali_parser::Parser;

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
struct FixPlan {
    var_tokens: HashSet<usize>,
    let_to_const_tokens: HashSet<usize>,
    eqeqeq_tokens: HashMap<usize, &'static str>,
    debugger_tokens: HashSet<usize>,
    unused_import_ranges: Vec<(usize, usize)>,
}

struct Analyzer {
    tokens: Vec<Token>,
    statements: Vec<Statement>,
    diagnostics: Vec<Diagnostic>,
    fix_plan: FixPlan,
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

    fn check_no_var_and_prefer_const(&mut self) {
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

    fn check_explicit_any(&mut self) {
        for (idx, window) in self.tokens.windows(2).enumerate() {
            let first = &window[0];
            let second = &window[1];
            if second.kind == TokenType::Identifier
                && second.value == "any"
                && matches!(first.kind, TokenType::Colon | TokenType::As)
            {
                self.diagnostics.push(
                    Diagnostic::warning(w2::EXPLICIT_ANY as u32, "avoid explicit `any`")
                        .with_suggestion("use a more specific type or `unknown`")
                        .with_span(second.span),
                );
                let _ = idx;
            }
        }
    }

    fn check_no_console(&mut self) {
        for window in self.tokens.windows(3) {
            if window[0].kind == TokenType::Identifier
                && window[0].value == "console"
                && window[1].kind == TokenType::Dot
                && window[2].kind == TokenType::Identifier
                && matches!(
                    window[2].value.as_str(),
                    "log" | "warn" | "error" | "info" | "debug"
                )
            {
                self.diagnostics.push(Diagnostic::warning(
                    w2::NO_CONSOLE as u32,
                    format!("avoid console.{} in checked source", window[2].value),
                ));
            }
        }
    }

    fn check_no_empty_and_unreachable(&mut self) {
        for statement in &self.statements {
            check_statement_for_empty_and_unreachable(statement, &mut self.diagnostics);
        }
    }

    fn check_debugger(&mut self) {
        for (index, token) in self.tokens.iter().enumerate() {
            if token.kind == TokenType::Debugger {
                self.diagnostics.push(
                    Diagnostic::error(w2::DEBUGGER as u32, "`debugger` statements are not allowed")
                        .with_suggestion("remove the `debugger` statement"),
                );
                self.fix_plan.debugger_tokens.insert(index);
                if let Some(next) = self.tokens.get(index + 1) {
                    if next.kind == TokenType::Semicolon {
                        self.fix_plan.debugger_tokens.insert(index + 1);
                    }
                }
            }
        }
    }

    fn check_eqeqeq(&mut self) {
        for (index, token) in self.tokens.iter().enumerate() {
            match token.kind {
                TokenType::EqEquals => {
                    self.diagnostics.push(
                        Diagnostic::warning(w2::EQEQEQ as u32, "use `===` instead of `==`")
                            .with_suggestion("replace `==` with `===`"),
                    );
                    self.fix_plan.eqeqeq_tokens.insert(index, "===");
                }
                TokenType::Neq => {
                    self.diagnostics.push(
                        Diagnostic::warning(w2::EQEQEQ as u32, "use `!==` instead of `!=`")
                            .with_suggestion("replace `!=` with `!==`"),
                    );
                    self.fix_plan.eqeqeq_tokens.insert(index, "!==");
                }
                _ => {}
            }
        }
    }

    fn check_no_unused_vars(
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

    fn check_no_unused_imports(&mut self, identifier_counts: &HashMap<String, usize>) {
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

    fn check_no_undef(&mut self, declared: &HashMap<String, usize>) {
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

fn collect_statements_declarations(statements: &[Statement], counts: &mut HashMap<String, usize>) {
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

fn check_statement_for_empty_and_unreachable(
    statement: &Statement,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match statement {
        Statement::BlockStatement(block) => {
            if block.body.is_empty() {
                diagnostics.push(Diagnostic::warning(
                    w2::NO_EMPTY as u32,
                    "empty block statement",
                ));
            }
            check_block_for_unreachable(block, diagnostics);
        }
        Statement::FunctionDeclaration(func) => {
            if func.body.body.is_empty() {
                diagnostics.push(Diagnostic::warning(
                    w2::NO_EMPTY as u32,
                    "empty function body",
                ));
            }
            check_block_for_unreachable(&func.body, diagnostics);
        }
        Statement::ClassDeclaration(class_decl) => {
            for method in &class_decl.body.methods {
                if let Some(body) = &method.body {
                    if body.body.is_empty() {
                        diagnostics.push(Diagnostic::warning(
                            w2::NO_EMPTY as u32,
                            "empty method body",
                        ));
                    }
                    check_block_for_unreachable(body, diagnostics);
                }
            }
        }
        Statement::IfStatement(stmt) => {
            check_statement_for_empty_and_unreachable(
                &Statement::BlockStatement(stmt.consequent.as_ref().clone()),
                diagnostics,
            );
            if let Some(alternate) = &stmt.alternate {
                check_statement_for_empty_and_unreachable(
                    &Statement::BlockStatement(alternate.as_ref().clone()),
                    diagnostics,
                );
            }
        }
        Statement::TryStatement(stmt) => {
            if stmt.block.body.is_empty() {
                diagnostics.push(Diagnostic::warning(w2::NO_EMPTY as u32, "empty try block"));
            }
            check_block_for_unreachable(&stmt.block, diagnostics);
            if let Some(handler) = &stmt.handler {
                if handler.body.body.is_empty() {
                    diagnostics.push(Diagnostic::warning(
                        w2::NO_EMPTY as u32,
                        "empty catch block",
                    ));
                }
                check_block_for_unreachable(&handler.body, diagnostics);
            }
            if let Some(finalizer) = &stmt.finalizer {
                if finalizer.body.is_empty() {
                    diagnostics.push(Diagnostic::warning(
                        w2::NO_EMPTY as u32,
                        "empty finally block",
                    ));
                }
                check_block_for_unreachable(finalizer, diagnostics);
            }
        }
        Statement::SwitchStatement(stmt) => {
            for case in &stmt.cases {
                if case.consequent.is_empty() {
                    diagnostics.push(Diagnostic::warning(
                        w2::NO_EMPTY as u32,
                        "empty switch case",
                    ));
                }
                let mut terminated = false;
                for inner in &case.consequent {
                    if terminated {
                        diagnostics.push(Diagnostic::error(
                            w2::NO_UNREACHABLE as u32,
                            "unreachable statement after a terminating control-flow statement",
                        ));
                        break;
                    }
                    if is_terminating_statement(inner) {
                        terminated = true;
                    }
                    check_statement_for_empty_and_unreachable(inner, diagnostics);
                }
            }
        }
        _ => {}
    }
}

fn check_block_for_unreachable(block: &BlockStatement, diagnostics: &mut Vec<Diagnostic>) {
    let mut terminated = false;
    for statement in &block.body {
        if terminated {
            diagnostics.push(Diagnostic::error(
                w2::NO_UNREACHABLE as u32,
                "unreachable statement after a terminating control-flow statement",
            ));
            break;
        }
        if is_terminating_statement(statement) {
            terminated = true;
        }
        check_statement_for_empty_and_unreachable(statement, diagnostics);
    }
}

fn is_terminating_statement(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::ReturnStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::DebuggerStatement(_)
    )
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

fn apply_fixes(source: &str, plan: &FixPlan) -> String {
    let mut rewritten = source.to_string();

    if !plan.unused_import_ranges.is_empty() {
        rewritten = rewritten
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                !(trimmed.starts_with("import ") || trimmed.starts_with("import{"))
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    if !plan.debugger_tokens.is_empty() {
        rewritten = rewritten.replace("debugger;", "");
        rewritten = rewritten.replace("debugger", "");
    }

    if !plan.var_tokens.is_empty() {
        rewritten = rewritten.replace("var ", "let ");
    }

    if !plan.let_to_const_tokens.is_empty() {
        rewritten = rewritten.replace("let ", "const ");
    }

    for replacement in plan.eqeqeq_tokens.values() {
        match *replacement {
            "===" => {
                rewritten = rewritten.replace("==", "===");
            }
            "!==" => {
                rewritten = rewritten.replace("!=", "!==");
            }
            _ => {}
        }
    }

    format_source(&rewritten)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
