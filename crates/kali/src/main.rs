mod cli;

use anyhow::Result;
use cli::{Cli, Commands, Parser};
use std::io::Read;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { mut input } => {
            let mut s = String::new();
            input.read_to_string(&mut s)?;
            println!("{s}");
        }
    }

    Ok(())
}
