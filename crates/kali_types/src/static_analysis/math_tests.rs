use crate::test_support::*;
use crate::*;
use kali_ast::{
    CallExpression, ConditionalExpression, DecoratedExpression, Expression, ExpressionStatement,
    LiteralValue, MemberExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    ParenthesizedExpression, PropertyName, VariableDeclaration, VariableDeclarator,
};
use kali_common::{
    math_abs_sign_frozen_callable_invocation_source, math_round_frozen_callable_invocation_source,
};
use kali_error::_error_codes::{e3, e5};
use kali_test_support::fixtures;
use std::fs;

#[path = "math_tests/pow.rs"]
mod pow;

#[path = "math_tests/transcendental.rs"]
mod transcendental;

#[path = "math_tests/rounding.rs"]
mod rounding;

#[path = "math_tests/wrappers.rs"]
mod wrappers;
