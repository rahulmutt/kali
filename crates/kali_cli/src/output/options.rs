//! CLI output options bag.

use crate::{ColorChoice, OutputFormat};

#[derive(Clone, Debug)]
pub struct CliOutputOptions {
    pub format: OutputFormat,
    pub pretty: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub color: ColorChoice,
}

impl CliOutputOptions {
    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json)
    }
}
