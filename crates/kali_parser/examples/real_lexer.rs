use kali_common::FileId;
use kali_lexer::Lexer;

fn main() {
    println!("Creating lexer...");
    let lexer = Lexer::new(FileId::new(0), "foo();".to_string());
    println!("Lexer struct created");

    println!("Calling lex_all()...");
    let result = lexer.lex_all();
    println!("lex_all() returned!");
    println!("Got {} tokens", result.tokens.len());
}
