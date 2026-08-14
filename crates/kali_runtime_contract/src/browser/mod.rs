//! Declarative browser-runtime surface: contract, command resolution, and
//! harness script generation. No wasmtime, no reqwest, no sandbox.
pub(crate) mod command;
pub(crate) mod contract;
pub(crate) mod harness;
