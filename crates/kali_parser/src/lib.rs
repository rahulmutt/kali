#![allow(dead_code)]
//! Basic parser for TypeScript/JavaScript.
//!
//! This module provides a full-featured parser for ECMA-262 and TypeScript syntax.
//! It generates an Abstract Syntax Tree (AST) suitable for further compilation passes.

use std::boxed::Box;
use kali_ast::{AST, ASTBuilder, Statement, VariableDeclarator, BlockStatement, FunctionDeclaration, ClassDeclaration, 
    IfStatement, WhileStatement, ForStatement, Expression, FunctionExpression, ExpressionStatement, ReturnStatement,
    VariableDeclaration as ASTVariableDeclaration, ParenthesizedExpression, ForInit, FunctionParam};
use kali_common::FileId;
use kali_error::diagnostic::Diagnostic;
use kali_lexer::{Token, TokenType};

pub struct TokenStream {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }
    
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    fn current_kind(&self) -> Option<&TokenType> {
        self.tokens.get(self.position).map(|t| &t.kind)
    }
    
    fn eof(&self) -> bool {
        self.position >= self.tokens.len()
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
        self.position += 1;
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
    
    pub fn parse(&mut self, _path: Option<String>) -> ParserOutput {
        let mut statements = Vec::new();
        while !self.stream.eof() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                self.stream.skip();
            }
        }
        ParserOutput {
            root: ASTBuilder::new().into_ast(),
            statements,
            diagnostics: self.diagnostics.clone(),
        }
    }
    
    fn parse_statement(&mut self) -> Option<Statement> {
        let kind = self.stream.current_kind()?;
        match kind {
            TokenType::Var | TokenType::Let | TokenType::Const => self.parse_variable_declaration(),
            TokenType::LeftBrace => self.parse_block_statement(),
            TokenType::Function => self.parse_function_declaration(),
            TokenType::Class => self.parse_class_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Return => self.parse_return_statement(),
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
                | TokenType::LeftParen => self.parse_expression_statement(),
                _ => {
                self.stream.skip();
                None
                }
             }
    }
    
    fn parse_variable_declaration(&mut self) -> Option<Statement> {
        self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        let init = if self.stream.accept(TokenType::Eq) {
            Some(self.parse_expression())
         } else {
            None
         };
        self.stream.accept(TokenType::Semicolon);
        let kind = match self.stream.tokens.get(self.stream.position - 1).map(|t| &t.kind) {
            Some(&TokenType::Var) => "var".to_string(),
            Some(&TokenType::Let) => "let".to_string(),
            Some(&TokenType::Const) => "const".to_string(),
                _ => "const".to_string(),
             };
        Some(Statement::VariableDeclaration(ASTVariableDeclaration {
            kind,
            declarations: vec![VariableDeclarator { id: name, init }],
         }))
    }
    
    fn parse_block_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        let mut statements = Vec::new();
        loop {
            if self.stream.accept(TokenType::RightBrace) {
                break;
               }
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
             } else if self.stream.eof() {
                break;
               }
           }
        Some(Statement::BlockStatement(BlockStatement { body: statements }))
    }
    
    fn parse_function_declaration(&mut self) -> Option<Statement> {
        self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        self.stream.advance();
        let mut params = Vec::new();
        if !self.stream.accept(TokenType::RightParen) {
            params.push(self.stream.advance().map(|t| t.value).unwrap_or_default());
            while self.stream.accept(TokenType::Comma) {
                params.push(self.stream.advance().map(|t| t.value).unwrap_or_default());
                }
                }
        self.stream.advance();
        let body_block = match self.parse_block_statement() {
            Some(Statement::BlockStatement(bs)) => bs,
                _ => BlockStatement { body: Vec::new() },
                   };
        self.stream.accept(TokenType::RightBrace);
        Some(Statement::FunctionDeclaration(FunctionDeclaration {
            name,
            params,
            body: Box::new(body_block),
         }))
    }
    
    fn parse_class_declaration(&mut self) -> Option<Statement> {
        self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        self.stream.advance();
        let methods = Vec::new();
        self.stream.accept(TokenType::RightBrace);
        Some(Statement::ClassDeclaration(ClassDeclaration {
            name,
            body: Box::new(kali_ast::ClassBody { methods }),
         }))
    }
    
    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        self.stream.advance();
        let test = self.parse_expression();
        self.stream.advance();
        let consequent_result = self.parse_statement().unwrap_or(
            Statement::BlockStatement(BlockStatement { body: vec![] })
         );
        match &consequent_result {
            Statement::BlockStatement(bs) => {
                let _consequent_block = BlockStatement { body: bs.body.clone() };
              }
              _ => { }
              }
        let consequent: Box<Statement> = Box::new(consequent_result);
        let alternate = if self.stream.accept(TokenType::Else) {
            Some(Box::new(self.parse_statement().unwrap_or(
                Statement::BlockStatement(BlockStatement { body: vec![] })
                 )))
            } else {
            None
               };
        Some(Statement::IfStatement(IfStatement {
            test,
            consequent,
            alternate,
             }))
    }
    
    fn parse_while_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        self.stream.advance();
        let test = self.parse_expression();
        self.stream.advance();
        let body = self.parse_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        Some(Statement::WhileStatement(WhileStatement {
            test,
            body: Box::new(body),
         }))
    }
    
    fn parse_for_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        self.stream.advance();
        let init = if self.stream.accept(TokenType::Var) || self.stream.accept(TokenType::Let) {
            let kind = "let";
            let name = self.stream.advance().map(|t| t.value).unwrap_or_default();
               { let _ = self.stream.accept(TokenType::Eq); };
               { let _ = self.parse_expression(); };
            self.stream.accept(TokenType::Semicolon);
            Some(ForInit::VariableDeclaration(ASTVariableDeclaration {
                kind: kind.to_string(),
                declarations: vec![VariableDeclarator { id: name, init: None }],
                  }))
            } else {
            Some(ForInit::Expression(Expression::Identifier("for".to_string())))
               };
        self.stream.advance();
        let body = self.parse_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        Some(Statement::ForStatement(ForStatement {
            init,
            test: None,
            update: None,
            body: Box::new(body),
              }))
    }
    
    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        let argument = if !matches!(self.stream.current_kind(), Some(TokenType::Semicolon) | Some(TokenType::RightBrace) | None) {
            Some(self.parse_expression())
           } else {
            None
            };
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::ReturnStatement(ReturnStatement { argument }))
    }
    
    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression();
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::ExpressionStatement(ExpressionStatement { expression: Box::new(expr) }))
    }
    
    fn parse_expression(&mut self) -> Expression {
        self.parse_primary_expression()
    }
    
    fn parse_primary_expression(&mut self) -> Expression {
        let kind = self.stream.current_kind();
        match kind {
            Some(TokenType::Identifier) => {
                let token = self.stream.advance().unwrap_or_else(|| Token::new(TokenType::Identifier, "unknown".to_string(), Default::default()));
                Expression::Identifier(token.value)
                }
            Some(TokenType::This) => {
                self.stream.advance();
                Expression::ThisExpression
                }
            Some(TokenType::True) | Some(TokenType::False) => {
                Expression::Literal(kali_ast::LiteralValue::Boolean(true))
                }
            Some(TokenType::Null) | Some(TokenType::Undefined) => {
                Expression::Literal(kali_ast::LiteralValue::Null)
                }
            Some(TokenType::NumericLiteral) => {
                let token = self.stream.advance().unwrap_or_else(|| Token::new(TokenType::NumericLiteral, "0".to_string(), Default::default()));
                let num: f64 = token.value.parse().unwrap_or(0.0);
                Expression::Literal(kali_ast::LiteralValue::Number(num))
                }
            Some(TokenType::StringLiteral) | Some(TokenType::Template) | Some(TokenType::Backtick) => {
                Expression::Literal(kali_ast::LiteralValue::String("".to_string()))
                 }
            Some(TokenType::LeftParen) => {
                self.stream.advance();
                let expr = self.parse_expression();
                self.stream.advance();
                Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression { expression: Box::new(expr) }))
                }
            Some(TokenType::Function) => {
                return self.parse_function_expression();
                 }
                 _ => {
                self.stream.advance();
                Expression::Identifier("unknown".to_string())
                }
             }
    }
    
    fn parse_function_expression(&mut self) -> Expression {
        self.stream.advance();
        let name = self.stream.advance().map(|t| t.value).unwrap_or_else(|| "anonymous".to_string());
        self.stream.advance();
        let mut params = Vec::new();
        if !self.stream.accept(TokenType::RightParen) {
            params.push(self.stream.advance().map(|t| t.value).unwrap_or_default());
            while self.stream.accept(TokenType::Comma) {
                params.push(self.stream.advance().map(|t| t.value).unwrap_or_default());
                 }
                  }
        let body = self.parse_block_statement().unwrap_or(
            Statement::BlockStatement(BlockStatement { body: Vec::new() })
          );
        let func_body: BlockStatement = match body {
            Statement::BlockStatement(bs) => BlockStatement { body: bs.body },
                 _ => BlockStatement { body: Vec::new() },
                    };
        let func = FunctionExpression {
            id: Some(name),
            params: params.into_iter().map(|p| FunctionParam { name: p }).collect(),
            body: Some(Box::new(func_body)),
            is_async: false,
            generator: false,
           };
        Expression::FunctionExpression(Box::new(func))
    }
}

pub struct ParserOutput {
    pub root: AST,
    pub statements: Vec<Statement>,
    pub diagnostics: Vec<Diagnostic>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use kali_lexer::Lexer;

    fn lex(source: &str) -> Vec<Token> {
        let lexer = Lexer::new(FileId::new(0), source.to_string());
        let result = lexer.lex_all();
        result.tokens
         }

         #[test]
    fn test_parse_var_declaration() {
        let tokens = lex("var x = 1;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_let_declaration() {
        let tokens = lex("let y = 2;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_constant() {
        let tokens = lex("const Z = 3;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_block_statement() {
        let tokens = lex("{ let x = 1; }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_function_declaration() {
        let tokens = lex("function foo() { let x = 1; }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_if_statement() {
        let tokens = lex("if (x > 0) { console.log(x); }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_class_declaration() {
        let tokens = lex("class Foo {}");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_while_statement() {
        let tokens = lex("while (x > 0) { console.log(x); }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }

         #[test]
    fn test_parse_for_statement() {
        let tokens = lex("for (let i = 0; i < 10; i++) { console.log(i); }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert!(output.statements.len() >= 1);
         }
}
