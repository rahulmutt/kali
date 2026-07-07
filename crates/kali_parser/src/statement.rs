//! Statement parsing (`parse_statement` dispatcher + per-keyword parsers).

use crate::Parser;
use kali_ast::{
    BlockStatement, BreakStatement, CatchClause, ContinueStatement, DebuggerStatement,
    DoWhileStatement, ExpressionStatement, ForInLefthand, ForInStatement, ForInit, ForOfLefthand,
    ForOfStatement, ForStatement, IfStatement, ReturnStatement, Statement, SwitchCase,
    SwitchStatement, ThrowStatement, TryStatement, VariableDeclaration, VariableDeclarator,
    WhileStatement,
};
use kali_lexer::TokenType;
use std::boxed::Box;

impl Parser {
    pub(crate) fn parse_statement(&mut self) -> Option<Statement> {
        let kind = self
            .stream
            .current_kind()
            .copied()
            .unwrap_or(TokenType::Unknown);
        match kind {
            TokenType::Var | TokenType::Let | TokenType::Const => self.parse_variable_declaration(),
            TokenType::LeftBrace => self.parse_block_statement(),
            TokenType::Async => {
                if self.stream.peek_next_kind() == Some(&TokenType::Function) {
                    self.parse_function_declaration_with_async(true, false)
                } else {
                    self.parse_expression_statement()
                }
            }
            TokenType::Function => self.parse_function_declaration_with_async(false, false),
            TokenType::Class => self.parse_class_declaration(),
            TokenType::Export => self.parse_export_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Do => self.parse_do_while_statement(),
            TokenType::Switch => self.parse_switch_statement(),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::Throw => self.parse_throw_statement(),
            TokenType::Try => self.parse_try_statement(),
            TokenType::Debugger => self.parse_debugger_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Import => {
                if self.stream.peek_next_kind() == Some(&TokenType::LeftParen) {
                    self.parse_expression_statement()
                } else {
                    self.parse_import_declaration()
                }
            }
            TokenType::Yield => self.parse_expression_statement(),
            TokenType::Plus if self.current_token_value_is("++") => {
                self.parse_expression_statement()
            }
            TokenType::Minus if self.current_token_value_is("--") => {
                self.parse_expression_statement()
            }
            TokenType::Identifier
            | TokenType::Await
            | TokenType::This
            | TokenType::True
            | TokenType::False
            | TokenType::Null
            | TokenType::Undefined
            | TokenType::Void
            | TokenType::Not
            | TokenType::Tilde
            | TokenType::Plus
            | TokenType::Minus
            | TokenType::NumericLiteral
            | TokenType::StringLiteral
            | TokenType::Template
            | TokenType::Backtick
            | TokenType::LeftParen
            | TokenType::New => self.parse_expression_statement(),
            _ => None,
        }
    }

    pub(crate) fn parse_variable_declaration(&mut self) -> Option<Statement> {
        let kind: String = match self.stream.current_kind() {
            Some(&TokenType::Var) => "var".to_string(),
            Some(&TokenType::Let) => "let".to_string(),
            Some(&TokenType::Const) => "const".to_string(),
            _ => return None,
        };

        // Advance past the keyword
        let _ = self.stream.advance();

        // Parse the first declarator, then loop over any comma-separated
        // additional declarators (`var a = 1, b = 2, c;`). Each declarator is a
        // name plus an optional `= init`; the single trailing semicolon is
        // consumed once, after the last declarator.
        let mut declarations = Vec::new();
        declarations.push(self.parse_variable_declarator()?);
        while self.stream.accept(TokenType::Comma) {
            declarations.push(self.parse_variable_declarator()?);
        }

        // Accept optional semicolon
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::VariableDeclaration(VariableDeclaration {
            kind,
            declarations,
        }))
    }

    /// Parse a single `name` or `name = init` declarator. Shared by every
    /// comma-separated declarator in a `var`/`let`/`const` statement so the
    /// block-arrow init special-case applies uniformly to each.
    fn parse_variable_declarator(&mut self) -> Option<VariableDeclarator> {
        let name_token = self.stream.advance()?;
        let name = name_token.value;

        let init = if self.stream.accept(TokenType::Eq) {
            // Statement-bodied arrows (`(a, b) => { ... }`) are not representable
            // in the expression grammar (`ArrowFunctionExpression.body` is an
            // `Expression`, and `return` inside `{}` is a statement), so the
            // general arrow parser bails on `{` bodies. In declarator-init
            // position parse them as an unnamed `FunctionExpression` — the exact
            // AST shape `const f = function () { ... }` produces, which the whole
            // pipeline (resolver scoping, HIR synthetic naming, codegen
            // standalone-function collection, const-binding call dispatch)
            // already compiles correctly.
            if let Some(arrow) = self.try_parse_block_arrow_function_expression() {
                Some(arrow)
            } else {
                Some(self.parse_expression())
            }
        } else {
            None
        };

        Some(VariableDeclarator { id: name, init })
    }

    pub(crate) fn parse_block_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let mut statements = Vec::new();
        loop {
            if self.stream.eof() {
                break;
            }
            if self.stream.accept(TokenType::RightBrace) {
                break;
            }
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                let _ = self.stream.advance();
            }
        }
        Some(Statement::BlockStatement(BlockStatement {
            body: statements,
        }))
    }

    pub(crate) fn parse_if_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.accept(TokenType::LeftParen);

        let test = self.parse_expression();
        let _ = self.stream.accept(TokenType::RightParen);

        let consequent_stmt = self
            .parse_statement()
            .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        let consequent = Box::new(Self::wrap_statement_as_block(consequent_stmt));

        let alternate = if self.stream.accept(TokenType::Else) {
            let alternate_stmt = self
                .parse_statement()
                .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
            Some(Box::new(Self::wrap_statement_as_block(alternate_stmt)))
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::IfStatement(IfStatement {
            test,
            consequent,
            alternate,
        }))
    }

    pub(crate) fn parse_while_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.accept(TokenType::LeftParen);

        let test = self.parse_expression();
        let _ = self.stream.accept(TokenType::RightParen);

        let body_stmt = self
            .parse_statement()
            .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        let body = Box::new(Self::wrap_statement_as_block(body_stmt));
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::WhileStatement(WhileStatement { test, body }))
    }

    pub(crate) fn parse_for_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let is_await = self.stream.accept(TokenType::Await);
        let _ = self.stream.accept(TokenType::LeftParen);

        if self.stream.current_kind() == Some(&TokenType::Semicolon) {
            let _ = self.stream.advance();
            let test = if self.stream.current_kind() != Some(&TokenType::Semicolon)
                && self.stream.current_kind() != Some(&TokenType::RightParen)
                && !self.stream.eof()
            {
                Some(self.parse_expression())
            } else {
                None
            };

            let _ = self.stream.accept(TokenType::Semicolon);

            let update = if self.stream.current_kind() != Some(&TokenType::RightParen)
                && !self.stream.eof()
            {
                Some(self.parse_expression())
            } else {
                None
            };

            let _ = self.stream.accept(TokenType::RightParen);

            let body_stmt = self
                .parse_statement()
                .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
            let body = Box::new(Self::wrap_statement_as_block(body_stmt));
            let _ = self.stream.accept(TokenType::Semicolon);

            return Some(Statement::ForStatement(ForStatement {
                init: None,
                test,
                update,
                body,
            }));
        }

        if matches!(
            self.stream.current_kind(),
            Some(TokenType::Var | TokenType::Let | TokenType::Const)
        ) {
            let kind = match self.stream.advance().map(|t| t.kind) {
                Some(TokenType::Let) => "let".to_string(),
                Some(TokenType::Const) => "const".to_string(),
                _ => "var".to_string(),
            };
            let name = self.stream.advance().map(|t| t.value).unwrap_or_default();
            let init_expr = if self.stream.accept(TokenType::Eq) {
                Some(self.parse_expression())
            } else {
                None
            };

            if init_expr.is_none() && self.stream.current_kind() == Some(&TokenType::Of) {
                let _ = self.stream.advance();
                let previous_async = self.in_async_function;
                self.in_async_function = true;
                let right = self.parse_expression();
                self.in_async_function = previous_async;
                let right = self
                    .unwrap_await_literal_array_expression(right.clone())
                    .unwrap_or(right);
                let _ = self.stream.accept(TokenType::RightParen);

                let body_stmt = self
                    .parse_statement()
                    .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
                let body = Box::new(Statement::BlockStatement(Self::wrap_statement_as_block(
                    body_stmt,
                )));
                let _ = self.stream.accept(TokenType::Semicolon);

                return Some(Statement::ForOfStatement(ForOfStatement {
                    left: ForOfLefthand::VariableDeclaration(VariableDeclaration {
                        kind,
                        declarations: vec![VariableDeclarator {
                            id: name,
                            init: None,
                        }],
                    }),
                    right,
                    body,
                    is_await,
                }));
            }

            // `for (var c in obj) { ... }` — mirrors the `of` arm above, but
            // produces a `ForInStatement` (enumerate the object's own keys).
            if init_expr.is_none() && self.stream.current_kind() == Some(&TokenType::In) {
                let _ = self.stream.advance();
                let right = self.parse_expression();
                let _ = self.stream.accept(TokenType::RightParen);

                let body_stmt = self
                    .parse_statement()
                    .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
                let body = Box::new(Statement::BlockStatement(Self::wrap_statement_as_block(
                    body_stmt,
                )));
                let _ = self.stream.accept(TokenType::Semicolon);

                return Some(Statement::ForInStatement(ForInStatement {
                    left: ForInLefthand::VariableDeclaration(VariableDeclaration {
                        kind,
                        declarations: vec![VariableDeclarator {
                            id: name,
                            init: None,
                        }],
                    }),
                    right,
                    body,
                }));
            }

            let _ = self.stream.accept(TokenType::Semicolon);
            let init = Some(ForInit::VariableDeclaration(VariableDeclaration {
                kind,
                declarations: vec![VariableDeclarator {
                    id: name,
                    init: init_expr,
                }],
            }));

            let test = if self.stream.current_kind() != Some(&TokenType::Semicolon)
                && self.stream.current_kind() != Some(&TokenType::RightParen)
                && !self.stream.eof()
            {
                Some(self.parse_expression())
            } else {
                None
            };

            let _ = self.stream.accept(TokenType::Semicolon);

            let update = if self.stream.current_kind() != Some(&TokenType::RightParen)
                && !self.stream.eof()
            {
                Some(self.parse_expression())
            } else {
                None
            };

            let _ = self.stream.accept(TokenType::RightParen);

            let body_stmt = self
                .parse_statement()
                .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
            let body = Box::new(Self::wrap_statement_as_block(body_stmt));
            let _ = self.stream.accept(TokenType::Semicolon);

            return Some(Statement::ForStatement(ForStatement {
                init,
                test,
                update,
                body,
            }));
        }

        let expr = self.parse_expression();
        if self.stream.current_kind() == Some(&TokenType::Of) {
            let _ = self.stream.advance();
            let right = self.parse_expression();
            let _ = self.stream.accept(TokenType::RightParen);

            let body_stmt = self
                .parse_statement()
                .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
            let body = Box::new(Statement::BlockStatement(Self::wrap_statement_as_block(
                body_stmt,
            )));
            let _ = self.stream.accept(TokenType::Semicolon);

            return Some(Statement::ForOfStatement(ForOfStatement {
                left: ForOfLefthand::Expression(expr),
                right,
                body,
                is_await,
            }));
        }

        // `for (c in obj) { ... }` with a pre-declared key binding — mirrors
        // the expression-form `of` arm above, producing a `ForInStatement`.
        if self.stream.current_kind() == Some(&TokenType::In) {
            let _ = self.stream.advance();
            let right = self.parse_expression();
            let _ = self.stream.accept(TokenType::RightParen);

            let body_stmt = self
                .parse_statement()
                .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
            let body = Box::new(Statement::BlockStatement(Self::wrap_statement_as_block(
                body_stmt,
            )));
            let _ = self.stream.accept(TokenType::Semicolon);

            return Some(Statement::ForInStatement(ForInStatement {
                left: ForInLefthand::Expression(expr),
                right,
                body,
            }));
        }

        let _ = self.stream.accept(TokenType::Semicolon);
        let test = if self.stream.current_kind() != Some(&TokenType::Semicolon)
            && self.stream.current_kind() != Some(&TokenType::RightParen)
            && !self.stream.eof()
        {
            Some(self.parse_expression())
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::Semicolon);

        let update =
            if self.stream.current_kind() != Some(&TokenType::RightParen) && !self.stream.eof() {
                Some(self.parse_expression())
            } else {
                None
            };

        let _ = self.stream.accept(TokenType::RightParen);

        let body_stmt = self
            .parse_statement()
            .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        let body = Box::new(Self::wrap_statement_as_block(body_stmt));
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ForStatement(ForStatement {
            init: Some(ForInit::Expression(expr)),
            test,
            update,
            body,
        }))
    }

    pub(crate) fn parse_do_while_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let body_stmt = self
            .parse_statement()
            .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        let body = Box::new(Self::wrap_statement_as_block(body_stmt));

        let _ = self.stream.accept(TokenType::While);
        let _ = self.stream.accept(TokenType::LeftParen);
        let test = self.parse_expression();
        let _ = self.stream.accept(TokenType::RightParen);
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::DoWhileStatement(DoWhileStatement { body, test }))
    }

    pub(crate) fn parse_switch_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.advance();

        let discriminant = self.parse_expression();
        let _ = self.stream.advance();

        let mut cases = Vec::new();
        loop {
            if self.stream.eof() {
                break;
            }
            if self.stream.current_kind() == Some(&TokenType::RightBrace) {
                break;
            }

            if self.stream.current_kind() == Some(&TokenType::Case) {
                let _ = self.stream.advance();
                let test = self.parse_expression();
                let _ = self.stream.advance();

                let mut consequent = Vec::new();
                loop {
                    let stop = self.stream.current_kind().is_none_or(|k| {
                        matches!(
                            k,
                            TokenType::Case | TokenType::Default | TokenType::RightBrace
                        )
                    });
                    if stop {
                        break;
                    }
                    if let Some(stmt) = self.parse_statement() {
                        consequent.push(stmt);
                    } else {
                        self.stream.advance();
                    }
                }
                cases.push(SwitchCase {
                    test: Some(test),
                    consequent,
                });
            } else if self.stream.current_kind() == Some(&TokenType::Default) {
                let _ = self.stream.advance();
                let _ = self.stream.advance();

                let mut consequent = Vec::new();
                loop {
                    let stop = self
                        .stream
                        .current_kind()
                        .is_none_or(|k| matches!(k, TokenType::Case | TokenType::RightBrace));
                    if stop {
                        break;
                    }
                    if let Some(stmt) = self.parse_statement() {
                        consequent.push(stmt);
                    } else {
                        self.stream.advance();
                    }
                }
                cases.push(SwitchCase {
                    test: None,
                    consequent,
                });
            } else {
                self.stream.advance();
            }
        }

        Some(Statement::SwitchStatement(SwitchStatement {
            discriminant,
            cases,
        }))
    }

    pub(crate) fn parse_break_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            self.stream.advance().map(|t| t.value)
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::BreakStatement(BreakStatement { label }))
    }

    pub(crate) fn parse_continue_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            self.stream.advance().map(|t| t.value)
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ContinueStatement(ContinueStatement { label }))
    }

    pub(crate) fn parse_throw_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let argument = self.parse_expression();
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ThrowStatement(ThrowStatement { argument }))
    }

    pub(crate) fn parse_debugger_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::DebuggerStatement(DebuggerStatement {}))
    }

    pub(crate) fn parse_try_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        let block = self
            .parse_block_statement()
            .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        let block_box = match block {
            Statement::BlockStatement(bs) => Box::new(bs),
            _ => Box::new(BlockStatement { body: vec![] }),
        };

        let handler = if self.stream.current_kind() == Some(&TokenType::Catch) {
            let _ = self.stream.advance();
            let param = self
                .stream
                .advance()
                .map(|t| t.value)
                .unwrap_or("e".to_string());

            let body = self
                .parse_block_statement()
                .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
            match body {
                Statement::BlockStatement(bs) => Some(CatchClause {
                    param,
                    body: Box::new(bs),
                }),
                _ => None,
            }
        } else {
            None
        };

        let finalizer = if self.stream.current_kind() == Some(&TokenType::Finally) {
            let _ = self.stream.advance();
            Some(
                self.parse_block_statement()
                    .unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] })),
            )
        } else {
            None
        };

        Some(Statement::TryStatement(TryStatement {
            block: block_box,
            handler,
            finalizer: finalizer.map(|b| match b {
                Statement::BlockStatement(bs) => bs,
                _ => BlockStatement { body: vec![] },
            }),
        }))
    }

    pub(crate) fn parse_return_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        let argument = if !matches!(
            self.stream.current_kind(),
            Some(TokenType::Semicolon) | Some(TokenType::RightBrace) | None
        ) {
            Some(self.parse_expression())
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ReturnStatement(ReturnStatement { argument }))
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression();
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(expr),
        }))
    }
}

#[cfg(test)]
#[path = "statement_tests.rs"]
mod statement_tests;
