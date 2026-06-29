use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[path = "math_tests/pow.rs"]
mod pow;

#[path = "math_tests/rounding.rs"]
mod rounding;

#[path = "math_tests/integer_ops.rs"]
mod integer_ops;

#[path = "math_tests/transcendental.rs"]
mod transcendental;
