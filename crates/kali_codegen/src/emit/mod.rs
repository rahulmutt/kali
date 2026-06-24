//! Core emission methods for FunctionEmitter.

mod call;
mod control_flow;
mod literal;
mod operators;

#[cfg(test)]
#[path = "emit_tests.rs"]
mod emit_tests;
