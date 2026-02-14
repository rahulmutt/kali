use anyhow::Result;
use syn::{File, parse_file};

/// Parse Rust source code into a syntax tree
pub fn parse_rust_code(input: &str) -> Result<File> {
    Ok(parse_file(input)?)
}

/// Parse and pretty-print the AST structure
pub fn parse_and_debug(input: &str) -> Result<()> {
    let syntax_tree = parse_rust_code(input)?;
    println!("{:#?}", syntax_tree);
    Ok(())
}
