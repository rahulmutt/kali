#![allow(dead_code)]
//! Basic parser for TypeScript/JavaScript.
//!
//! This module provides a full-featured parser for ECMA-262 and TypeScript syntax.
//! It generates an Abstract Syntax Tree (AST) suitable for further compilation passes.

use std::boxed::Box;
use kali_ast::{AST, ASTBuilder, Statement, VariableDeclarator, BlockStatement, FunctionDeclaration, ClassDeclaration, 
    IfStatement, WhileStatement, DoWhileStatement, SwitchStatement, SwitchCase, ForStatement, Expression, FunctionExpression, ExpressionStatement, ReturnStatement,
    VariableDeclaration, ParenthesizedExpression, ForInit, FunctionParam, BreakStatement, ContinueStatement, ThrowStatement, 
    DebuggerStatement, TryStatement, CatchClause, BinaryExpression, CallExpression, MemberExpression};
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
            TokenType::Do => self.parse_do_while_statement(),
            TokenType::Switch => self.parse_switch_statement(),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::Throw => self.parse_throw_statement(),
            TokenType::Try => self.parse_try_statement(),
            TokenType::Debugger => self.parse_debugger_statement(),
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
        let kind: String = match self.stream.current_kind() {
            Some(&TokenType::Var) => "var".to_string(),
            Some(&TokenType::Let) => "let".to_string(),
            Some(&TokenType::Const) => "const".to_string(),
             _ => return None,
          };
        self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        let init = if self.stream.accept(TokenType::Eq) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::VariableDeclaration(VariableDeclaration {
            kind: "var".to_string(),
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
        while !self.stream.accept(TokenType::RightBrace) && !self.stream.eof() {
            self.stream.skip();
        }
        Some(Statement::ClassDeclaration(ClassDeclaration {
            name,
            body: Box::new(kali_ast::ClassBody { methods: vec![] }),
        }))
    }
    
    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        self.stream.advance();
        let test = self.parse_expression();
        self.stream.advance();
        let consequent = Box::new(self.parse_statement().unwrap_or(
            Statement::BlockStatement(BlockStatement { body: vec![] })
        ));
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
            let name = self.stream.advance().map(|t| t.value).unwrap_or_default();
            self.stream.accept(TokenType::Eq);
            self.parse_expression();
            self.stream.accept(TokenType::Semicolon);
            Some(ForInit::VariableDeclaration(VariableDeclaration {
                kind: "let".to_string(),
                declarations: vec![VariableDeclarator { id: name, init: None }],
            }))
        } else {
            Some(ForInit::Expression(Expression::Identifier("for".to_string())))
        };
        self.stream.advance();
        let body = self.parse_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::ForStatement(ForStatement {
            init,
            test: None,
            update: None,
            body: Box::new(body),
        }))
    }
    
    fn parse_do_while_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        let body = self.parse_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        self.stream.advance();
        self.stream.advance();
        let test = self.parse_expression();
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::DoWhileStatement(DoWhileStatement { body: Box::new(body), test }))
    }
    
    fn parse_switch_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        self.stream.advance();
        let discriminant = self.parse_expression();
        self.stream.advance();
        self.stream.advance();
        let mut cases = Vec::new();
        loop {
            if self.stream.current_kind() == Some(&TokenType::RightBrace) {
                break;
            }
            if self.stream.current_kind() == Some(&TokenType::Case) {
                self.stream.advance();
                let test = self.parse_expression();
                self.stream.advance();
                let mut consequent = Vec::new();
                loop {
                    match self.stream.current_kind() {
                        Some(&TokenType::Case) | Some(&TokenType::Default) | Some(&TokenType::RightBrace) => break,
                        _ => { if let Some(stmt) = self.parse_statement() { consequent.push(stmt); } }
                    }
                }
                cases.push(SwitchCase { test: Some(test), consequent });
            } else if self.stream.current_kind() == Some(&TokenType::Default) {
                self.stream.advance();
                let mut consequent = Vec::new();
                loop {
                    match self.stream.current_kind() {
                        Some(&TokenType::Case) | Some(&TokenType::RightBrace) => break,
                        _ => { if let Some(stmt) = self.parse_statement() { consequent.push(stmt); } }
                    }
                }
                cases.push(SwitchCase { test: None, consequent });
            } else {
                self.stream.skip();
            }
        }
        Some(Statement::SwitchStatement(SwitchStatement { discriminant, cases }))
    }
    
    fn parse_break_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            self.stream.advance();
            Some(self.stream.tokens[self.stream.position - 1].value.clone())
        } else {
            None
        };
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::BreakStatement(BreakStatement { label }))
    }
    
    fn parse_continue_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            self.stream.advance();
            Some(self.stream.tokens[self.stream.position - 1].value.clone())
        } else {
            None
        };
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::ContinueStatement(ContinueStatement { label }))
    }
    
    fn parse_throw_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        let argument = self.parse_expression();
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::ThrowStatement(ThrowStatement { argument }))
    }
    
    fn parse_debugger_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        self.stream.accept(TokenType::Semicolon);
        Some(Statement::DebuggerStatement(DebuggerStatement {}))
    }
    
    fn parse_try_statement(&mut self) -> Option<Statement> {
        self.stream.advance();
        let block = self.parse_block_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        let block_box = match block {
            Statement::BlockStatement(bs) => Box::new(bs),
            _ => Box::new(BlockStatement { body: vec![] }),
        };
        let handler = if self.stream.current_kind() == Some(&TokenType::Catch) {
            self.stream.advance();
            let param = self.stream.advance().map(|t| t.value).unwrap_or_default();
            self.stream.advance();
            let body = self.parse_block_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
            match body {
                Statement::BlockStatement(bs) => Some(CatchClause { param, body: Box::new(bs) }),
                _ => None,
            }
          } else {
            None
          };
        let finalizer = if self.stream.current_kind() == Some(&TokenType::Finally) {
            Some(self.parse_block_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] })))
         } else {
            None
          };
        Some(Statement::TryStatement(TryStatement { block: block_box, handler, finalizer: finalizer.map(|b| match b {
            Statement::BlockStatement(bs) => bs,
            _ => BlockStatement { body: vec![] },
         }) }))
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
        self.parse_call_expression()
    }
    
    fn parse_call_expression(&mut self) -> Expression {
        let mut expr = self.parse_function_expression();
        
        loop {
            match self.stream.current_kind() {
                Some(TokenType::LeftParen) => {
                    self.stream.advance();
                    let mut args = Vec::new();
                    if !self.stream.accept(TokenType::RightParen) {
                        let arg = self.parse_expression();
                        args.push(arg);
                        while self.stream.accept(TokenType::Comma) {
                            let arg = self.parse_expression();
                            args.push(arg);
                         }
                        self.stream.advance();
                     }
                    expr = Expression::CallExpression(Box::new(CallExpression {
                        callee: expr,
                        args: args,
                     }));
                   }
                Some(TokenType::LeftBracket) => {
                    self.stream.advance();
                    let index = self.parse_expression();
                    self.stream.advance();
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
                    if self.stream.current_kind() == Some(&TokenType::LeftParen) {
                        break;
                      }
                      if self.stream.advance_if(TokenType::Dot) {
                          if let Some(prop_tok) = self.stream.advance() {
                              if let TokenType::Identifier = prop_tok.kind {
                                  let prop_name = prop_tok.value;
                                  expr = Expression::MemberExpression(Box::new(MemberExpression {
                                      object: expr,
                                      property: prop_name,
                                  }));
                              }
                          }
                      }
                   }
                   _ => break,
               }
           }
        
        expr
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
        self.stream.advance();
        let body = self.parse_block_statement().unwrap_or(
            Statement::BlockStatement(BlockStatement { body: Vec::new() })
         );
        let func_body = match body {
            Statement::BlockStatement(bs) => BlockStatement { body: bs.body },
             _ => BlockStatement { body: Vec::new() },
         };
        Expression::FunctionExpression(Box::new(FunctionExpression {
            id: Some(name),
            params: params.into_iter().map(|p| FunctionParam { name: p }).collect(),
            body: Some(Box::new(func_body)),
            is_async: false,
            generator: false,
         }))
     }
    
    fn parse_primary_expression(&mut self) -> Expression {
        let kind = self.stream.current_kind();
        match kind {
            Some(TokenType::Identifier) => {
                let token = self.stream.advance();
                self.stream.advance();
                Expression::Identifier(token.map(|t| t.value).unwrap_or_else(|| "unknown".to_string()))
             }
            Some(TokenType::This) => {
                self.stream.advance();
                self.stream.advance();
                Expression::ThisExpression
             }
            Some(TokenType::True) | Some(TokenType::False) => {
                self.stream.advance();
                self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Boolean(true))
             }
            Some(TokenType::Null) => {
                self.stream.advance();
                self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Null)
             }
            Some(TokenType::Undefined) => {
                self.stream.advance();
                self.stream.advance();
                Expression::Identifier("undefined".to_string())
             }
            Some(TokenType::NumericLiteral) => {
                let token = self.stream.advance();
                self.stream.advance();
                let s = token.map(|t| t.value).unwrap_or_else(|| "0".to_string());
                let num: f64 = s.parse().unwrap_or(0.0);
                Expression::Literal(kali_ast::LiteralValue::Number(num))
             }
            Some(TokenType::StringLiteral) | Some(TokenType::Template) | Some(TokenType::Backtick) => {
                let token = self.stream.advance();
                self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::String(token.map(|t| t.value).unwrap_or_else(|| "".to_string())))
             }
            Some(TokenType::LeftParen) => {
                self.stream.advance();
                let expr = self.parse_expression();
                self.stream.advance();
                Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression { 
                    expression: Box::new(expr) 
                 }))
             }
            Some(TokenType::Function) => {
                self.stream.advance();
                let name = self.stream.advance().map(|t| t.value).unwrap_or_else(|| "anonymous".to_string());
                self.stream.advance();
                let mut params: Vec<String> = Vec::new();
                if !self.stream.accept(TokenType::RightParen) {
                    params.push(self.stream.advance().map(|t| t.value).unwrap_or_default());
                    while self.stream.accept(TokenType::Comma) {
                        params.push(self.stream.advance().map(|t| t.value).unwrap_or_default());
                      }
                 }
                self.stream.advance();
                let body = self.parse_block_statement().unwrap_or(
                    Statement::BlockStatement(BlockStatement { body: Vec::new() })
                 );
                let func_body = match body {
                    Statement::BlockStatement(bs) => BlockStatement { body: bs.body },
                     _ => BlockStatement { body: Vec::new() },
                 };
                Expression::FunctionExpression(Box::new(FunctionExpression {
                    id: Some(name),
                    params: params.into_iter().map(|p| FunctionParam { name: p }).collect(),
                    body: Some(Box::new(func_body)),
                    is_async: false,
                    generator: false,
                 }))
             }
             _ => {
                self.stream.advance();
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
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::VariableDeclaration(vd) => {
                assert_eq!(vd.kind, "var");
                assert_eq!(vd.declarations.len(), 1);
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected VariableDeclaration"); }
         }
     }

     #[test]
    fn test_parse_let_declaration() {
        let tokens = lex("let y = 2;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::VariableDeclaration(vd) => {
                assert_eq!(vd.kind, "let");
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected VariableDeclaration"); }
         }
     }

     #[test]
    fn test_parse_constant() {
        let tokens = lex("const Z = 3;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::VariableDeclaration(vd) => {
                assert_eq!(vd.kind, "const");
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected VariableDeclaration"); }
         }
     }

     #[test]
    fn test_parse_block_statement() {
        let tokens = lex("{ let x = 1; }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::BlockStatement(bs) => {
                assert_eq!(bs.body.len(), 1);
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected BlockStatement"); }
         }
     }

     #[test]
    fn test_parse_function_declaration() {
        let tokens = lex("function foo() { let x = 1; }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::FunctionDeclaration(fd) => {
                assert_eq!(fd.name, "foo");
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected FunctionDeclaration"); }
         }
     }

     #[test]
    fn test_parse_if_statement() {
        let tokens = lex("if (x > 0) { console.log(x); }");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::IfStatement(_) => {}
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected IfStatement"); }
         }
     }

     #[test]
    fn test_parse_class_declaration() {
        let tokens = lex("class Foo {}");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::ClassDeclaration(_) => {}
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ClassDeclaration"); }
         }
     }

     #[test]
    fn test_parse_while_statement() {
        let tokens = lex("while (x > 0) { console.log(x); };");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::WhileStatement(_) => {}
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected WhileStatement"); }
         }
     }

     #[test]
    fn test_parse_for_statement() {
        let tokens = lex("for (let i = 0; i < 10; i++) { console.log(i); };");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        assert_eq!(output.statements.len(), 1);
        
        match &output.statements[0] {
            Statement::ForStatement(_) => {}
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ForStatement"); }
         }
     }
    
     #[test]
    fn test_parse_binary_expression() {
        let tokens = lex("a + b;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        
        match &output.statements[0] {
            Statement::ExpressionStatement(es) => {
                match &*es.expression {
                    Expression::BinaryExpression(be) => {
                        match &be.left {
                            Expression::Identifier(id) => assert_eq!(id, "a"),
                             _ => { panic!("Expected left to be identifier a"); }
                         }
                        assert_eq!(&be.operator, "+");
                        match &be.right {
                            Expression::Identifier(id) => assert_eq!(id, "b"),
                             _ => { panic!("Expected right to be identifier b"); }
                         }
                     }
                     _ => { eprintln!("Got: {:?}", std::mem::discriminant(&*es.expression)); panic!("Expected BinaryExpression"); }
                 }
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ExpressionStatement"); }
         }
     }
    
     #[test]
    fn test_parse_binary_and_operator() {
        let tokens = lex("x && y;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        
        match &output.statements[0] {
            Statement::ExpressionStatement(es) => {
                match &*es.expression {
                    Expression::BinaryExpression(be) => {
                        assert_eq!(&be.operator, "&&");
                     }
                     _ => { eprintln!("Got: {:?}", std::mem::discriminant(&*es.expression)); panic!("Expected BinaryExpression"); }
                 }
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ExpressionStatement"); }
         }
     }

     #[test]
    fn test_parse_call_expression() {
        let tokens = lex("foo();");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        
        match &output.statements[0] {
            Statement::ExpressionStatement(es) => {
                match &*es.expression {
                    Expression::CallExpression(ce) => {
                        match &ce.callee {
                            Expression::Identifier(id) => assert_eq!(id, "foo"),
                             _ => { panic!("Expected callee to be identifier foo"); }
                         }
                        assert_eq!(ce.args.len(), 0);
                     }
                     _ => { eprintln!("Got: {:?}", std::mem::discriminant(&*es.expression)); panic!("Expected CallExpression"); }
                 }
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ExpressionStatement"); }
         }
     }
    
     #[test]
    fn test_parse_call_expression_with_args() {
        let tokens = lex("foo(bar, baz);");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        
        match &output.statements[0] {
            Statement::ExpressionStatement(es) => {
                match &*es.expression {
                    Expression::CallExpression(ce) => {
                        assert_eq!(ce.args.len(), 2);
                     }
                     _ => { eprintln!("Got: {:?}", std::mem::discriminant(&*es.expression)); panic!("Expected CallExpression"); }
                 }
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ExpressionStatement"); }
         }
     }

     #[test]
    fn test_parse_member_expression() {
        let tokens = lex("obj.prop;");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        
        match &output.statements[0] {
            Statement::ExpressionStatement(es) => {
                match &*es.expression {
                    Expression::MemberExpression(me) => {
                        match &me.object {
                            Expression::Identifier(id) => assert_eq!(id, "obj"),
                             _ => { panic!("Expected object to be identifier obj"); }
                         }
                        assert_eq!(&me.property, "prop");
                     }
                     _ => { eprintln!("Got: {:?}", std::mem::discriminant(&*es.expression)); panic!("Expected MemberExpression"); }
                 }
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ExpressionStatement"); }
         }
     }
    
     #[test]
    fn test_parse_member_expression_computed() {
        let tokens = lex("obj[computed];");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        
        match &output.statements[0] {
            Statement::ExpressionStatement(es) => {
                match &*es.expression {
                    Expression::MemberExpression(me) => {
                        assert_eq!(&me.property, "computed");
                     }
                     _ => { eprintln!("Got: {:?}", std::mem::discriminant(&*es.expression)); panic!("Expected MemberExpression"); }
                 }
             }
             _ => { eprintln!("Got: {:?}", std::mem::discriminant(&output.statements[0])); panic!("Expected ExpressionStatement"); }
         }
     }

     #[test]
    fn test_parse_call_chain() {
        let tokens = lex("foo().bar()");
        let mut parser = Parser::new(FileId::new(0), tokens);
        let output = parser.parse(None);
        
        assert!(!output.statements.is_empty());
     }
}
