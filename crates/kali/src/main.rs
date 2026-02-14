mod cli;

use cli::{Cli, Commands, Parser};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { input } => {
            println!("Running with input: {:?}", input);
        }
    }
}
