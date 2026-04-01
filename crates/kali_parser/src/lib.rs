#![allow(dead_code)]
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
    
    pub fn parse(&mut self, _path: Option<String>) -> ParserOutput {
        let mut statements = Vec::new();
        while !self.stream.eof() && self.stream.position < self.stream.tokens.len() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
                self.stream.position += 1;   // Force advancement after successful parse
            } else {
                self.stream.advance();   // Skip token on parse failure
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
        let kind = self.stream.current_kind().copied().unwrap_or(TokenType::Unknown);
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
            TokenType::Identifier | TokenType::This | TokenType::True | TokenType::False |
            TokenType::Null | TokenType::Undefined | TokenType::NumericLiteral |
            TokenType::StringLiteral | TokenType::Template | TokenType::Backtick |
            TokenType::LeftParen => self.parse_expression_statement(),
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
        
         // Ensure we advance at least once
        let _ = self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        
        let init = if self.stream.accept(TokenType::Eq) {
            Some(self.parse_expression())
         } else {
            None
         };
        
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
            if self.stream.eof() { break; }
            if self.stream.accept(TokenType::RightBrace) { break; }
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
             } else {
                self.stream.advance();
             }
         }
        Some(Statement::BlockStatement(BlockStatement { body: statements }))
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
        
        let _ = self.stream.advance();
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
        let _ = self.stream.advance();
        
        loop {
            if self.stream.eof() { break; }
            if self.stream.accept(TokenType::RightBrace) { break; }
            self.stream.advance();
         }
        
         Some(Statement::ClassDeclaration(ClassDeclaration {
            name,
            body: Box::new(kali_ast::ClassBody { methods: vec![] }),
         }))
     }
    
    fn parse_if_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.advance();
        
        let test = self.parse_expression();
        let _ = self.stream.advance();
        
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
        
         let _ = self.stream.accept(TokenType::Semicolon);
        
         Some(Statement::IfStatement(IfStatement {
            test,
            consequent,
            alternate,
         }))
     }
    
    fn parse_while_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.advance();
        
        let test = self.parse_expression();
        let _ = self.stream.advance();
        
        let body = self.parse_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
         let _ = self.stream.accept(TokenType::Semicolon);
        
         Some(Statement::WhileStatement(WhileStatement {
            test,
            body: Box::new(body),
         }))
     }
    
    fn parse_for_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.advance();
        
        let init = if self.stream.current_kind() == Some(&TokenType::Var) ||
                         self.stream.current_kind() == Some(&TokenType::Let) {
            let name = self.stream.advance().map(|t| t.value).unwrap_or_default();
            let init_expr = if self.stream.accept(TokenType::Eq) {
                Some(self.parse_expression())
             } else {
                None
             };
             let _ = self.stream.accept(TokenType::Semicolon);
             Some(ForInit::VariableDeclaration(VariableDeclaration {
                 kind: "var".to_string(),
                 declarations: vec![VariableDeclarator { id: name, init: init_expr }],
             }))
         } else {
            let expr = self.parse_expression();
             let _ = self.stream.accept(TokenType::Semicolon);
             Some(ForInit::Expression(expr))
         };
        
        let test = if !self.stream.eof() &&
                         self.stream.current_kind() != Some(&TokenType::Semicolon) &&
                         self.stream.current_kind() != Some(&TokenType::RightParen) &&
                         self.stream.position < self.stream.tokens.len() {
            Some(self.parse_expression())
         } else {
            None
         };
        
        let update = if !self.stream.eof() &&
                           self.stream.current_kind() != Some(&TokenType::Semicolon) &&
                           self.stream.current_kind() != Some(&TokenType::RightParen) &&
                           self.stream.position < self.stream.tokens.len() {
            Some(self.parse_expression())
         } else {
            None
         };
        
         let _ = self.stream.advance();
         let _ = self.stream.accept(TokenType::RightParen);
        
        let body = self.parse_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
         let _ = self.stream.accept(TokenType::Semicolon);
        
         Some(Statement::ForStatement(ForStatement {
            init,
            test,
            update,
            body: Box::new(body),
         }))
     }
    
    fn parse_do_while_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let body = self.parse_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        
        let _ = self.stream.advance();
        if self.stream.accept(TokenType::While) {
            let _ = self.stream.advance();
         }
        
         let _ = self.stream.advance();
        
         let test = self.parse_expression();
         let _ = self.stream.accept(TokenType::Semicolon);
        
         Some(Statement::DoWhileStatement(DoWhileStatement { body: Box::new(body), test }))
     }
    
    fn parse_switch_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let _ = self.stream.advance();
        
        let discriminant = self.parse_expression();
        let _ = self.stream.advance();
        
        let mut cases = Vec::new();
        loop {
            if self.stream.eof() { break; }
            if self.stream.current_kind() == Some(&TokenType::RightBrace) { break; }
            
            if self.stream.current_kind() == Some(&TokenType::Case) {
                let _ = self.stream.advance();
                let test = self.parse_expression();
                let _ = self.stream.advance();
                
                let mut consequent = Vec::new();
                loop {
                    let stop = self.stream.current_kind().map_or(true, |k| matches!(k, TokenType::Case | TokenType::Default | TokenType::RightBrace));
                    if stop { break; }
                    if let Some(stmt) = self.parse_statement() {
                        consequent.push(stmt);
                     } else {
                        self.stream.advance();
                     }
                 }
                cases.push(SwitchCase { test: Some(test), consequent });
             } else if self.stream.current_kind() == Some(&TokenType::Default) {
                let _ = self.stream.advance();
                let _ = self.stream.advance();
                
                let mut consequent = Vec::new();
                loop {
                    let stop = self.stream.current_kind().map_or(true, |k| matches!(k, TokenType::Case | TokenType::RightBrace));
                    if stop { break; }
                    if let Some(stmt) = self.parse_statement() {
                        consequent.push(stmt);
                     } else {
                        self.stream.advance();
                     }
                 }
                cases.push(SwitchCase { test: None, consequent });
             } else {
                self.stream.advance();
             }
         }
        
         Some(Statement::SwitchStatement(SwitchStatement { discriminant, cases }))
     }
    
    fn parse_break_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        
        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            let _ = self.stream.advance();
            self.stream.tokens.last().map(|t| t.value.clone())
         } else {
            None
         };
        
         let _ = self.stream.accept(TokenType::Semicolon);
        
         Some(Statement::BreakStatement(BreakStatement { label }))
     }
    
    fn parse_continue_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        
        let label = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            let _ = self.stream.advance();
            self.stream.tokens.last().map(|t| t.value.clone())
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
        
        let block = self.parse_block_statement().unwrap_or(Statement::BlockStatement(BlockStatement { body: vec![] }));
        let block_box = match block {
            Statement::BlockStatement(bs) => Box::new(bs),
             _ => Box::new(BlockStatement { body: vec![] }),
         };
        
        let handler = if self.stream.current_kind() == Some(&TokenType::Catch) {
            let _ = self.stream.advance();
            let param = self.stream.advance().map(|t| t.value).unwrap_or("e".to_string());
            
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
        
         Some(Statement::TryStatement(TryStatement { 
            block: block_box, 
            handler, 
            finalizer: finalizer.map(|b| match b {
                Statement::BlockStatement(bs) => bs,
                 _ => BlockStatement { body: vec![] },
             }) 
         }))
     }
    
    fn parse_return_statement(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        
        let argument = if !matches!(self.stream.current_kind(), Some(TokenType::Semicolon) | Some(TokenType::RightBrace) | None) {
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
        
         Some(Statement::ExpressionStatement(ExpressionStatement { expression: Box::new(expr) }))
     }
    
    fn parse_expression(&mut self) -> Expression {
        self.parse_binary_expression(0)
     }
    
    fn parse_binary_expression(&mut self, min_prec: usize) -> Expression {
        let mut left = self.parse_call_expression();
        
         loop {
            let op_kind = self.stream.current_kind().copied().unwrap_or(TokenType::Unknown);
            let prec: Option<usize> = match op_kind {
                TokenType::OrOr if min_prec <= 1 => Some(1),
                TokenType::AndAnd if min_prec <= 2 => Some(2),
                TokenType::Pipe if min_prec <= 3 => Some(3),
                TokenType::Caret if min_prec <= 4 => Some(4),
                TokenType::And if min_prec <= 5 => Some(5),
                TokenType::EqEquals if min_prec <= 6 => Some(6),
                TokenType::Not | TokenType::Lt | TokenType::Gt |
                TokenType::LtEq | TokenType::GtEq if min_prec <= 6 => Some(6),
                TokenType::Plus | TokenType::Minus if min_prec <= 7 => Some(7),
                TokenType::Star | TokenType::Slash | TokenType::Percent if min_prec <= 8 => Some(8),
                 _ => None,
             };
            
             if let Some(op_prec) = prec {
                let _ = self.stream.advance();
                let op_str = match self.stream.current_kind() {
                    Some(&TokenType::Plus) => "+",
                    Some(&TokenType::Minus) => "-",
                    Some(&TokenType::Star) => "*",
                    Some(&TokenType::Slash) => "/",
                    Some(&TokenType::Percent) => "%",
                    Some(&TokenType::AndAnd) => "&&",
                    Some(&TokenType::OrOr) => "||",
                    Some(&TokenType::Pipe) => "|",
                    Some(&TokenType::Caret) => "^",
                    Some(&TokenType::And) => "&",
                    Some(&TokenType::EqEquals) => "==",
                    Some(&TokenType::Not) => "!=",
                    Some(&TokenType::Lt) => "<",
                    Some(&TokenType::Gt) => ">",
                    Some(&TokenType::LtEq) => "<=",
                    Some(&TokenType::GtEq) => ">=",
                     _ => break,
                 };
                
                 left = Expression::BinaryExpression(Box::new(BinaryExpression {
                    left: left,
                    operator: op_str.to_string(),
                    right: self.parse_binary_expression(op_prec + 1),
                 }));
             }
            
             let _ = self.stream.advance();  // Ensure we advance
         }
        
         left
     }
    
    fn parse_call_expression(&mut self) -> Expression {
        let mut expr = self.parse_primary_expression();
        
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
                         let _ = self.stream.advance();
                      }
                    expr = Expression::CallExpression(Box::new(CallExpression {
                        callee: expr,
                        args: args,
                     }));
                  }
                Some(TokenType::LeftBracket) => {
                    let _ = self.stream.advance();
                    let index = self.parse_expression();
                    let _ = self.stream.advance();
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
                     let _ = self.stream.advance();
                     if let Some(TokenType::Identifier) = self.stream.current_kind() {
                        if let Some(token) = self.stream.tokens.get(self.stream.position) {
                            let prop_name = token.value.clone();
                            expr = Expression::MemberExpression(Box::new(MemberExpression {
                                object: expr,
                                property: prop_name,
                             }));
                          }
                      }
                     break;
                   }
                   _ => break,
                }
             }
        
         expr
     }
    
    fn parse_function_expression(&mut self) -> Expression {
        let _ = self.stream.advance();
        let name = self.stream.advance().map(|t| t.value).unwrap_or_else(|| "anonymous".to_string());
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
        
        let _ = self.stream.advance();
        
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
        let kind = self.stream.current_kind().copied().unwrap_or(TokenType::Unknown);
        match kind {
            TokenType::Identifier => {
                let _ = self.stream.advance();
                Expression::Identifier("x".to_string())
             }
            TokenType::This => {
                let _ = self.stream.advance();
                Expression::ThisExpression
             }
            TokenType::True | TokenType::False => {
                let _ = self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Boolean(true))
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
                let _ = self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Number(0.0))
             }
            TokenType::StringLiteral | TokenType::Template | TokenType::Backtick => {
                let _ = self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::String("".to_string()))
             }
            TokenType::LeftParen => {
                let _ = self.stream.advance();
                let expr = self.parse_expression();
                let _ = self.stream.advance();
                Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression { 
                    expression: Box::new(expr) 
                 }))
             }
            TokenType::Function => {
                let _ = self.stream.advance();
                let name = self.stream.advance().map(|t| t.value).unwrap_or_else(|| "anonymous".to_string());
                let _ = self.stream.advance();
                Expression::FunctionExpression(Box::new(FunctionExpression {
                    id: Some(name),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: false,
                    generator: false,
                 }))
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
              _ => panic!("Expected VariableDeclaration"),
         }
     }
}
