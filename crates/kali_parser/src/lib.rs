#![allow(dead_code)]

mod declaration;
mod expression;
mod literal;
mod module;
mod parser;
mod statement;
mod token_stream;
mod types;

pub use parser::{Parser, ParserOutput};
pub use token_stream::TokenStream;

#[cfg(test)]
mod test_support;
