use crate::token::{Token, TokenType};
use crate::Lexer;

impl Lexer {
    pub(crate) fn lex_identifier(&mut self) -> Token {
        let _start = self.position;
        while let Some(&c) = self.source.get(self.position) {
            if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                self.position += 1;
            } else {
                break;
            }
        }
        let value: String = self.source[_start..self.position].iter().collect();
        let kind = match value.as_str() {
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "do" => TokenType::Do,
            "switch" => TokenType::Switch,
            "case" => TokenType::Case,
            "default" => TokenType::Default,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "return" => TokenType::Return,
            "throw" => TokenType::Throw,
            "try" => TokenType::Try,
            "catch" => TokenType::Catch,
            "finally" => TokenType::Finally,
            "debugger" => TokenType::Debugger,
            "new" => TokenType::New,
            "function" => TokenType::Function,
            "var" => TokenType::Var,
            "let" => TokenType::Let,
            "const" => TokenType::Const,
            "class" => TokenType::Class,
            "interface" => TokenType::Interface,
            "type" => TokenType::Type,
            "enum" => TokenType::Enum,
            "import" => TokenType::Import,
            "export" => TokenType::Export,
            "from" => TokenType::From,
            "as" => TokenType::As,
            "this" => TokenType::This,
            "super" => TokenType::Super,
            "extends" => TokenType::Extends,
            "implements" => TokenType::Implements,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "yield" => TokenType::Yield,
            "instanceof" => TokenType::InstanceOf,
            "in" => TokenType::In,
            "of" => TokenType::Of,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "undefined" => TokenType::Undefined,
            "void" => TokenType::Void,
            "delete" => TokenType::Delete,
            "typeof" => TokenType::Typeof,
            _ => TokenType::Identifier,
        };
        Token::new(kind, self.slice(_start), self.span())
    }
}
