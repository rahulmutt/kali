//! Linter for Kali source files.

mod control_flow;
mod engine;
mod fixes;
mod scope;
mod style;
mod variables;

pub use engine::*;
pub(crate) use engine::{Analyzer, FixPlan};
