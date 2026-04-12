use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::Parser;

fn main() {
    println!("=== Starting parser trace ===");
    println!("Testing: foo();");

    let lexer = Lexer::new(FileId::new(0), "foo();".to_string());
    println!("Lexer created");

    let result = lexer.lex_all();
    println!("Lexer finished. Got {} tokens", result.tokens.len());

    for (i, tok) in result.tokens.iter().enumerate() {
        println!("  Token {}: {:?} = '{}'", i, tok.kind, tok.value);
    }

    let mut parser = Parser::new(FileId::new(0), result.tokens);
    println!("Parser created");

    let result = parser.parse(None);
    println!("Parse complete!");
    println!("Got {} statements", result.statements.len());
    println!("=== Parser trace complete ===");
}
