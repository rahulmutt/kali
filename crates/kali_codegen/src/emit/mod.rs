//! Core emission methods for FunctionEmitter.

mod control_flow;
mod literal;
mod operators;
mod call;

#[cfg(test)]
#[path = "emit_tests.rs"]
mod emit_tests;
