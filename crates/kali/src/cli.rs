pub use clap::Parser;
use clap::Subcommand;
use clio::Input;

#[derive(Parser)]
#[command(name = "kali")]
#[command(about = "Kali sandboxed, AI-native execution environment")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run command
    Run {
        /// Input file or stdin
        #[arg(default_value = "-")]
        input: Input,
    },
}
