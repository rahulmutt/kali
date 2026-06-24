//! Browser-runtime backend: contract, command resolution, harness generation,
//! checked execution, and summary parsing.
use crate::*;
pub(crate) mod contract;
pub(crate) mod command;
pub(crate) mod summary;
pub(crate) mod harness;
pub(crate) mod execute;
