//! Control-flow rules: empty blocks (no-empty) and unreachable code (no-unreachable).

use kali_ast::{BlockStatement, Statement};
use kali_error::{_error_codes::w2, Diagnostic};

use crate::Analyzer;

impl Analyzer {
    pub(crate) fn check_no_empty_and_unreachable(&mut self) {
        for statement in &self.statements {
            check_statement_for_empty_and_unreachable(statement, &mut self.diagnostics);
        }
    }
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
