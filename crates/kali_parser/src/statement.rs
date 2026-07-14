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
            | TokenType::New
            // Statement-position `delete <expr>;` was previously absent from
            // this dispatch table entirely: `parse_statement` returned `None`
            // for the `delete` token, so the top-level loop silently
            // discarded it (see `parse` in parser.rs) and re-parsed the
            // remaining tokens as their own statement — `delete r.a;` ran as
            // a bare `r.a;` member read. Route it to the same expression-
            // statement path as every other unary-expression starter so the
            // new `TokenType::Delete` arm in `parse_unary_expression` is
            // actually reachable in statement position (throw-fallout Stage 2).
            | TokenType::Delete
            // Same bug class, found in Stage 5's namespace-typeof-fold work:
            // `typeof <expr>;` in STATEMENT position (not, e.g., after `=` or
            // inside a call argument, both of which already worked — see
            // `parse_unary_expression`'s `TokenType::Typeof` arm) was absent
            // from this dispatch table, so `parse_statement` returned `None`
            // for the leading `typeof` token, the top-level/block loop
            // silently discarded it (see `parse_block_statement` above and
            // `parse` in parser.rs), and re-parsed the remainder as its own
            // statement — `typeof ns.lazyValue;` silently ran as a bare
            // `ns.lazyValue;` member read, dropping the operator entirely
            // (worse than the pre-fix `delete` bug: no placeholder node
            // survives at all). Route it the same way.
            | TokenType::Typeof => self.parse_expression_statement(),
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
        if !self
            .stream
            .current_kind()
            .is_some_and(Self::is_binding_name_token)
        {
            self.push_feature_unavailable("a reserved word cannot be used as a binding name");
            let _ = self.stream.advance();
            return None;
        }
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

        // Expression-form head: a trailing `in` here belongs to `for (c in
        // obj)`, so it must terminate the expression (no_in) instead of being
        // rejected as the unsupported binary `in` operator.
        let previous_no_in = self.no_in;
        self.no_in = true;
        let expr = self.parse_expression();
        self.no_in = previous_no_in;
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

        // kali has no exception-unwinding machinery. try/catch/finally
        // previously lowered to a bogus if-shaped Branch (catch block treated
        // as an `if` `then` arm) — a silent miscompile that only "passed"
        // while `throw` was a no-op. Reject fail-closed rather than pretend to
        // handle exceptions. The tokens are still consumed below so the parse
        // recovers cleanly and no cascade of secondary errors is emitted.
        self.push_feature_unavailable(
            "try/catch/finally is unavailable: kali has no exception-handling machinery",
        );

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
