#![allow(dead_code)]
use kali_ast::{
    ASTBuilder, ArrayExpression, BinaryExpression, BlockStatement, BreakStatement, CallExpression,
    CatchClause, ClassDeclaration, ContinueStatement, DebuggerStatement, DoWhileStatement,
    Expression, ExpressionOrSpread, ExpressionStatement, ForInit, ForStatement,
    FunctionDeclaration, FunctionExpression, FunctionParam, IfStatement, ImportDeclaration,
    ImportExpression, ImportName, ImportNamedSpecifier, ImportSpecifier, MemberExpression,
    ParenthesizedExpression, ReturnStatement, Statement, SwitchCase, SwitchStatement,
    ThrowStatement, TryStatement, VariableDeclaration, VariableDeclarator, WhileStatement, AST,
};
use kali_common::FileId;
use kali_error::diagnostic::Diagnostic;
use kali_lexer::{Token, TokenType};
use std::boxed::Box;

pub struct TokenStream {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn current_kind(&self) -> Option<&TokenType> {
        self.tokens.get(self.position).map(|t| &t.kind)
    }

    fn peek_next_kind(&self) -> Option<&TokenType> {
        self.tokens.get(self.position + 1).map(|t| &t.kind)
    }

    fn eof(&self) -> bool {
        self.tokens.is_empty() || self.position >= self.tokens.len()
    }

    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let tok = self.tokens[self.position].clone();
            self.position += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn advance_if(&mut self, expected: TokenType) -> bool {
        if self.current_kind() == Some(&expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn accept(&mut self, k: TokenType) -> bool {
        self.advance_if(k)
    }

    fn skip(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
}

pub struct Parser {
    file_id: FileId,
    stream: TokenStream,
    diagnostics: Vec<Diagnostic>,
    jsx_mode: bool,
}

impl Parser {
    pub fn new(file_id: FileId, tokens: Vec<Token>) -> Self {
        Self {
            file_id,
            stream: TokenStream::new(tokens),
            diagnostics: Vec::new(),
            jsx_mode: false,
        }
    }

    fn wrap_statement_as_block(stmt: Statement) -> BlockStatement {
        match stmt {
            Statement::BlockStatement(block) => block,
            other => BlockStatement { body: vec![other] },
        }
    }

    fn parse_parameter_list(&mut self) -> Vec<String> {
        let mut params = Vec::new();
        if self.stream.accept(TokenType::RightParen) {
            return params;
        }

        loop {
            if let Some(token) = self.stream.advance() {
                if token.kind == TokenType::Identifier {
                    params.push(token.value);
                }
            }
            if self.stream.accept(TokenType::RightParen) {
                break;
            }
            if !self.stream.accept(TokenType::Comma) {
                let _ = self.stream.accept(TokenType::RightParen);
                break;
            }
        }

        params
    }

    pub fn parse(&mut self, _path: Option<String>) -> ParserOutput {
        let mut statements = Vec::new();
        while !self.stream.eof() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                // Statement parsing failed, advance to avoid infinite loop
                let _ = self.stream.advance();
            }
        }

        let root = ASTBuilder::new().into_ast();
        ParserOutput {
            root,
            statements,
            diagnostics: self.diagnostics.clone(),
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        let kind = self
            .stream
            .current_kind()
            .copied()
            .unwrap_or(TokenType::Unknown);
        match kind {
            TokenType::Var | TokenType::Let | TokenType::Const => self.parse_variable_declaration(),
            TokenType::LeftBrace => self.parse_block_statement(),
            TokenType::Function => self.parse_function_declaration(),
            TokenType::Class => self.parse_class_declaration(),
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
            TokenType::Identifier
            | TokenType::This
            | TokenType::True
            | TokenType::False
            | TokenType::Null
            | TokenType::Undefined
            | TokenType::NumericLiteral
            | TokenType::StringLiteral
            | TokenType::Template
            | TokenType::Backtick
            | TokenType::LeftParen
            | TokenType::New => self.parse_expression_statement(),
            _ => None,
        }
    }

    fn parse_variable_declaration(&mut self) -> Option<Statement> {
        let kind: String = match self.stream.current_kind() {
            Some(&TokenType::Var) => "var".to_string(),
            Some(&TokenType::Let) => "let".to_string(),
            Some(&TokenType::Const) => "const".to_string(),
            _ => return None,
        };

        // Advance past the keyword
        let _ = self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;

        let init = if self.stream.accept(TokenType::Eq) {
            Some(self.parse_expression())
        } else {
            None
        };

        // Accept optional semicolon
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::VariableDeclaration(VariableDeclaration {
            kind,
            declarations: vec![VariableDeclarator { id: name, init }],
        }))
    }

    fn parse_block_statement(&mut self) -> Option<Statement> {
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

    fn parse_function_declaration(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        let _ = self.stream.advance();

        let mut params = Vec::new();
        if !self.stream.accept(TokenType::RightParen) {
            if let Some(param) = self.stream.advance() {
                params.push(param.value);
            }
            while self.stream.accept(TokenType::Comma) {
                if let Some(param) = self.stream.advance() {
                    params.push(param.value);
                }
            }
        }

        let body_block = match self.parse_block_statement() {
            Some(Statement::BlockStatement(bs)) => bs,
            _ => BlockStatement { body: Vec::new() },
        };

        Some(Statement::FunctionDeclaration(FunctionDeclaration {
            name,
            params,
            body: Box::new(body_block),
        }))
    }

    fn parse_class_declaration(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        let _ = self.stream.accept(TokenType::LeftBrace);

        let mut methods = Vec::new();
        loop {
            if self.stream.eof() || self.stream.current_kind() == Some(&TokenType::RightBrace) {
                let _ = self.stream.accept(TokenType::RightBrace);
                break;
            }

            let is_method = matches!(self.stream.current_kind(), Some(TokenType::Identifier))
                && matches!(self.stream.peek_next_kind(), Some(TokenType::LeftParen));

            if is_method {
                let method_name = self.stream.advance().map(|t| t.value).unwrap_or_default();
                let _ = self.stream.accept(TokenType::LeftParen);
                let params = self.parse_parameter_list();
                let body = match self.parse_block_statement() {
                    Some(Statement::BlockStatement(bs)) => bs,
                    _ => BlockStatement { body: Vec::new() },
                };
                methods.push(kali_ast::MethodDefinition {
                    name: method_name,
                    params,
                    body: Some(Box::new(body)),
                });
            } else {
                let _ = self.stream.advance();
            }
        }

        Some(Statement::ClassDeclaration(ClassDeclaration {
            name,
            body: Box::new(kali_ast::ClassBody { methods }),
        }))
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
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

    fn parse_while_statement(&mut self) -> Option<Statement> {
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

    fn parse_for_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.accept(TokenType::LeftParen);

        let init = if self.stream.current_kind() == Some(&TokenType::Semicolon) {
            let _ = self.stream.advance();
            None
        } else if matches!(
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
            let _ = self.stream.accept(TokenType::Semicolon);
            Some(ForInit::VariableDeclaration(VariableDeclaration {
                kind,
                declarations: vec![VariableDeclarator {
                    id: name,
                    init: init_expr,
                }],
            }))
        } else {
            let expr = self.parse_expression();
            let _ = self.stream.accept(TokenType::Semicolon);
            Some(ForInit::Expression(expr))
        };

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
            init,
            test,
            update,
            body,
        }))
    }

    fn parse_do_while_statement(&mut self) -> Option<Statement> {
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

    fn parse_switch_statement(&mut self) -> Option<Statement> {
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

    fn parse_break_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            self.stream.advance().map(|t| t.value)
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::BreakStatement(BreakStatement { label }))
    }

    fn parse_continue_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            self.stream.advance().map(|t| t.value)
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ContinueStatement(ContinueStatement { label }))
    }

    fn parse_throw_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let argument = self.parse_expression();
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ThrowStatement(ThrowStatement { argument }))
    }

    fn parse_debugger_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::DebuggerStatement(DebuggerStatement {}))
    }

    fn parse_try_statement(&mut self) -> Option<Statement> {
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

    fn parse_return_statement(&mut self) -> Option<Statement> {
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

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression();
        let _ = self.stream.accept(TokenType::Semicolon);

        Some(Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(expr),
        }))
    }

    fn parse_import_declaration(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();

        if self.stream.current_kind() == Some(&TokenType::StringLiteral) {
            let source = self
                .stream
                .advance()
                .map(|token| unquote_string_literal(&token.value))
                .unwrap_or_default();
            let _ = self.stream.accept(TokenType::Semicolon);
            return Some(Statement::ImportDeclaration(ImportDeclaration {
                specifiers: vec![ImportSpecifier::SideEffect],
                source,
            }));
        }

        let mut specifiers = Vec::new();
        let mut saw_default = false;

        if self.stream.current_kind() == Some(&TokenType::Type) {
            let _ = self.stream.advance();
            let type_specifiers = self.parse_import_named_specifiers();
            specifiers.push(ImportSpecifier::Type(type_specifiers));
        } else if self.stream.current_kind() == Some(&TokenType::Star) {
            if let Some(namespace) = self.parse_import_namespace_specifier() {
                specifiers.push(namespace);
            }
        } else if self.stream.current_kind() == Some(&TokenType::LeftBrace) {
            let named = self.parse_import_named_specifiers();
            specifiers.push(ImportSpecifier::Named(named));
        } else if self.stream.current_kind() == Some(&TokenType::Identifier) {
            let default_local = self
                .stream
                .advance()
                .map(|token| token.value)
                .unwrap_or_default();
            specifiers.push(ImportSpecifier::Default(default_local));
            saw_default = true;
        }

        if saw_default && self.stream.current_kind() == Some(&TokenType::Comma) {
            let _ = self.stream.advance();
            if self.stream.current_kind() == Some(&TokenType::LeftBrace) {
                let named = self.parse_import_named_specifiers();
                specifiers.push(ImportSpecifier::Named(named));
            } else if self.stream.current_kind() == Some(&TokenType::Star) {
                if let Some(namespace) = self.parse_import_namespace_specifier() {
                    specifiers.push(namespace);
                }
            }
        }

        if self.stream.current_kind() == Some(&TokenType::From) {
            let _ = self.stream.advance();
        }

        let source = match self.stream.current_kind() {
            Some(TokenType::StringLiteral) => self
                .stream
                .advance()
                .map(|token| unquote_string_literal(&token.value))
                .unwrap_or_default(),
            _ => "unknown".to_string(),
        };
        let _ = self.stream.accept(TokenType::Semicolon);

        if specifiers.is_empty() {
            specifiers.push(ImportSpecifier::SideEffect);
        }

        Some(Statement::ImportDeclaration(ImportDeclaration {
            specifiers,
            source,
        }))
    }

    fn parse_import_named_specifiers(&mut self) -> Vec<ImportNamedSpecifier> {
        let mut specifiers = Vec::new();
        if self.stream.current_kind() != Some(&TokenType::LeftBrace) {
            return specifiers;
        }

        let _ = self.stream.advance();
        loop {
            match self.stream.current_kind() {
                Some(TokenType::RightBrace) => {
                    let _ = self.stream.advance();
                    break;
                }
                Some(TokenType::Identifier) => {
                    let imported = self
                        .stream
                        .advance()
                        .map(|token| token.value)
                        .unwrap_or_default();
                    let mut local = imported.clone();
                    let mut imported_name = None;

                    if self.stream.current_kind() == Some(&TokenType::As) {
                        let _ = self.stream.advance();
                        if self.stream.current_kind() == Some(&TokenType::Identifier) {
                            local = self
                                .stream
                                .advance()
                                .map(|token| token.value)
                                .unwrap_or(imported.clone());
                            imported_name = Some(ImportName::Identifier(imported));
                        }
                    }

                    specifiers.push(ImportNamedSpecifier {
                        local,
                        imported: imported_name,
                    });
                    let _ = self.stream.accept(TokenType::Comma);
                }
                _ => {
                    let _ = self.stream.advance();
                }
            }
        }

        specifiers
    }

    fn parse_import_namespace_specifier(&mut self) -> Option<ImportSpecifier> {
        if self.stream.current_kind() != Some(&TokenType::Star) {
            return None;
        }

        let _ = self.stream.advance();
        if self.stream.current_kind() == Some(&TokenType::As) {
            let _ = self.stream.advance();
            if self.stream.current_kind() == Some(&TokenType::Identifier) {
                let local = self
                    .stream
                    .advance()
                    .map(|token| token.value)
                    .unwrap_or_default();
                return Some(ImportSpecifier::Namespace(local));
            }
        }

        None
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_prec: usize) -> Expression {
        let mut left = self.parse_call_expression();

        let mut iterations = 0;
        loop {
            let op_kind = self
                .stream
                .current_kind()
                .copied()
                .unwrap_or(TokenType::Unknown);

            // Get operator precedence (higher number = tighter binding)
            let op_prec: Option<usize> = match op_kind {
                TokenType::OrOr => Some(1),
                TokenType::AndAnd => Some(2),
                TokenType::Pipe => Some(3),
                TokenType::Caret => Some(4),
                TokenType::And => Some(5),
                TokenType::EqEquals
                | TokenType::Not
                | TokenType::Lt
                | TokenType::Gt
                | TokenType::LtEq
                | TokenType::GtEq => Some(6),
                TokenType::Plus | TokenType::Minus => Some(7),
                TokenType::Star | TokenType::Slash | TokenType::Percent => Some(8),
                _ => None,
            };

            // If operator has lower precedence than min_prec, we're done
            if let Some(prec) = op_prec {
                if prec < min_prec {
                    break;
                }

                let op_str = match op_kind {
                    TokenType::Plus => "+",
                    TokenType::Minus => "-",
                    TokenType::Star => "*",
                    TokenType::Slash => "/",
                    TokenType::Percent => "%",
                    TokenType::AndAnd => "&&",
                    TokenType::OrOr => "||",
                    TokenType::Pipe => "|",
                    TokenType::Caret => "^",
                    TokenType::And => "&",
                    TokenType::EqEquals => "==",
                    TokenType::Not => "!=",
                    TokenType::Lt => "<",
                    TokenType::Gt => ">",
                    TokenType::LtEq => "<=",
                    TokenType::GtEq => ">=",
                    _ => {
                        // Not a binary operator we handle
                        break;
                    }
                };

                let _ = self.stream.advance();
                // Parse right side with higher precedence to get next operand
                // Using prec + 1 ensures left-associativity for same-precedence operators
                left = Expression::BinaryExpression(Box::new(BinaryExpression {
                    left,
                    operator: op_str.to_string(),
                    right: self.parse_binary_expression(prec + 1),
                }));
            } else {
                break;
            }
            iterations += 1;
            if iterations > 100 {
                break;
            }
        }

        left
    }

    fn parse_call_expression(&mut self) -> Expression {
        let mut expr = self.parse_primary_expression();

        let mut iterations = 0;
        loop {
            match self.stream.current_kind() {
                Some(TokenType::LeftParen) => {
                    let _ = self.stream.advance();
                    let mut args = Vec::new();
                    if !self.stream.accept(TokenType::RightParen) {
                        let arg = self.parse_expression();
                        args.push(arg);
                        while self.stream.accept(TokenType::Comma) {
                            let arg = self.parse_expression();
                            args.push(arg);
                        }
                        let _ = self.stream.accept(TokenType::RightParen);
                    }
                    expr =
                        Expression::CallExpression(Box::new(CallExpression { callee: expr, args }));
                }
                Some(TokenType::LeftBracket) => {
                    let _ = self.stream.advance();
                    let index = self.parse_expression();
                    let _ = self.stream.accept(TokenType::RightBracket);
                    let index_str = match &index {
                        Expression::Identifier(s) => s.clone(),
                        _ => "index".to_string(),
                    };
                    expr = Expression::MemberExpression(Box::new(MemberExpression {
                        object: expr,
                        property: index_str,
                    }));
                }
                Some(TokenType::Dot) => {
                    let _ = self.stream.advance();
                    match self.stream.current_kind() {
                        Some(TokenType::Identifier) => {
                            let _ = self.stream.advance();
                            if let Some(token) = self.stream.tokens.get(self.stream.position - 1) {
                                let prop_name = token.value.clone();
                                expr = Expression::MemberExpression(Box::new(MemberExpression {
                                    object: expr,
                                    property: prop_name,
                                }));
                            } else {
                                expr = Expression::MemberExpression(Box::new(MemberExpression {
                                    object: expr,
                                    property: "unknown".to_string(),
                                }));
                            }
                        }
                        _ => {
                            // No identifier after dot, stop the chain
                            break;
                        }
                    }
                }
                Some(TokenType::QuestionDot) => {
                    let _ = self.stream.advance();
                    expr = self.parse_optional_chain_expression(expr);
                }
                _ => break,
            }
            iterations += 1;
            if iterations > 100 {
                break;
            }
        }

        expr
    }

    fn parse_optional_chain_expression(&mut self, object: Expression) -> Expression {
        match self.stream.current_kind() {
            Some(TokenType::Identifier) => {
                let _ = self.stream.advance();
            }
            Some(TokenType::LeftBracket) => {
                let _ = self.stream.advance();
                let _ = self.parse_expression();
                let _ = self.stream.accept(TokenType::RightBracket);
            }
            Some(TokenType::LeftParen) => {
                let _ = self.stream.advance();
                if !self.stream.accept(TokenType::RightParen) {
                    let _ = self.parse_expression();
                    while self.stream.accept(TokenType::Comma) {
                        let _ = self.parse_expression();
                    }
                    let _ = self.stream.accept(TokenType::RightParen);
                }
            }
            Some(TokenType::QuestionDot) => {
                // Support repeated optional chaining segments like `a?.b?.c`.
            }
            _ => {}
        }

        Expression::OptionalChainExpression(Box::new(kali_ast::OptionalChainExpression {
            inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                object: Box::new(object),
                optional: true,
            }),
        }))
    }

    fn parse_function_expression(&mut self) -> Expression {
        let _ = self.stream.advance();

        let id = if self.stream.current_kind() == Some(&TokenType::Identifier)
            && self.stream.peek_next_kind() == Some(&TokenType::LeftParen)
        {
            self.stream.advance().map(|t| t.value)
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::LeftParen);
        let params = self
            .parse_parameter_list()
            .into_iter()
            .map(|p| FunctionParam { name: p })
            .collect();

        let body = self
            .parse_block_statement()
            .unwrap_or(Statement::BlockStatement(BlockStatement {
                body: Vec::new(),
            }));
        let func_body = match body {
            Statement::BlockStatement(bs) => Some(Box::new(bs)),
            _ => Some(Box::new(BlockStatement { body: Vec::new() })),
        };

        Expression::FunctionExpression(Box::new(FunctionExpression {
            id,
            params,
            body: func_body,
            is_async: false,
            generator: false,
        }))
    }

    fn parse_primary_expression(&mut self) -> Expression {
        let kind = self
            .stream
            .current_kind()
            .copied()
            .unwrap_or(TokenType::Unknown);
        match kind {
            TokenType::Identifier => {
                let token = self.stream.advance();
                let name = token
                    .map(|t| t.value)
                    .unwrap_or_else(|| "unknown".to_string());
                Expression::Identifier(name)
            }
            TokenType::This => {
                let _ = self.stream.advance();
                Expression::ThisExpression
            }
            TokenType::True | TokenType::False => {
                let is_true = self.stream.current_kind().copied() == Some(TokenType::True);
                let _ = self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Boolean(is_true))
            }
            TokenType::Null => {
                let _ = self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Null)
            }
            TokenType::Undefined => {
                let _ = self.stream.advance();
                Expression::Identifier("undefined".to_string())
            }
            TokenType::NumericLiteral => {
                let token = self.stream.advance();
                let value = token.map(|t| t.value).unwrap_or_default();
                if value.ends_with('n') {
                    Expression::BigIntLiteral(value)
                } else {
                    let parsed = value.parse::<f64>().unwrap_or(0.0);
                    Expression::Literal(kali_ast::LiteralValue::Number(parsed))
                }
            }
            TokenType::StringLiteral | TokenType::Template | TokenType::Backtick => {
                let token = self.stream.advance();
                let value = token.map(|t| t.value).unwrap_or_default();
                Expression::Literal(kali_ast::LiteralValue::String(value))
            }
            TokenType::LeftParen => {
                let _ = self.stream.advance();
                let expr = self.parse_expression();
                // Expect closing paren
                let _ = self.stream.accept(TokenType::RightParen);
                Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                    expression: Box::new(expr),
                }))
            }
            TokenType::LeftBracket => {
                let _ = self.stream.advance();
                let mut elements = Vec::new();
                if !self.stream.accept(TokenType::RightBracket) {
                    loop {
                        let element = self.parse_expression();
                        elements.push(Some(ExpressionOrSpread::Expression(element)));
                        if self.stream.accept(TokenType::Comma) {
                            if self.stream.current_kind() == Some(&TokenType::RightBracket) {
                                let _ = self.stream.accept(TokenType::RightBracket);
                                break;
                            }
                            continue;
                        }
                        let _ = self.stream.accept(TokenType::RightBracket);
                        break;
                    }
                }
                Expression::ArrayExpression(ArrayExpression { elements })
            }
            TokenType::Function => self.parse_function_expression(),
            TokenType::Import => {
                let _ = self.stream.advance();
                if self.stream.accept(TokenType::LeftParen) {
                    let source = self.parse_expression();
                    let _ = self.stream.accept(TokenType::RightParen);
                    Expression::ImportExpression(Box::new(ImportExpression { source }))
                } else {
                    Expression::Identifier("import".to_string())
                }
            }
            TokenType::New => {
                let _ = self.stream.advance();
                let callee = self.parse_call_expression();
                let mut args = Vec::new();
                if self.stream.accept(TokenType::LeftParen)
                    && !self.stream.accept(TokenType::RightParen)
                {
                    args.push(self.parse_expression());
                    while self.stream.accept(TokenType::Comma) {
                        args.push(self.parse_expression());
                    }
                    let _ = self.stream.accept(TokenType::RightParen);
                }
                Expression::NewExpression(Box::new(kali_ast::NewExpression { callee, args }))
            }
            _ => {
                let _ = self.stream.advance();
                Expression::Identifier("unknown".to_string())
            }
        }
    }
}

pub struct ParserOutput {
    pub root: AST,
    pub statements: Vec<Statement>,
    pub diagnostics: Vec<Diagnostic>,
}

fn unquote_string_literal(value: &str) -> String {
    let trimmed = value.trim();
    let Some(first) = trimmed.chars().next() else {
        return trimmed.to_string();
    };
    let Some(last) = trimmed.chars().last() else {
        return trimmed.to_string();
    };

    if (first == '"' && last == '"')
        || (first == '\'' && last == '\'')
        || (first == '`' && last == '`')
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
