//! Token-level style rules: explicit-any, no-console, debugger, eqeqeq.

use kali_error::{_error_codes::w2, Diagnostic};
use kali_lexer::TokenType;

use crate::Analyzer;

impl Analyzer {
    pub(crate) fn check_explicit_any(&mut self) {
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

    pub(crate) fn check_no_console(&mut self) {
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

    pub(crate) fn check_debugger(&mut self) {
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

    pub(crate) fn check_eqeqeq(&mut self) {
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
}
