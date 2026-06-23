//! kali_types-specific test builders and macros (compiled under cfg(test)).
use kali_ast::{Expression, MemberExpression, OptionalChainExpression, OptionalChainInner};

// --- builder functions (migrated from tests.rs) ---

pub(crate) fn sequence_expression(expressions: Vec<Expression>) -> Expression {
    Expression::SequenceExpression(Box::new(kali_ast::SequenceExpression { expressions }))
}

pub(crate) fn optional_chain_global_this_math() -> Expression {
    Expression::OptionalChainExpression(Box::new(OptionalChainExpression {
        inner: Box::new(OptionalChainInner::NonNull {
            object: Box::new(Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: "Math".to_string(),
            }))),
            optional: true,
        }),
    }))
}

pub(crate) fn optional_chain_global_this_math_pow() -> Expression {
    Expression::OptionalChainExpression(Box::new(OptionalChainExpression {
        inner: Box::new(OptionalChainInner::NonNull {
            object: Box::new(Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Math".to_string(),
                })),
                property: "pow".to_string(),
            }))),
            optional: true,
        }),
    }))
}

// --- AST-builder macros ---

/// Resolve `$stmts` in a fresh `TypeContext` and assert the produced
/// diagnostic count equals `$count`.
macro_rules! assert_resolution {
    ($stmts:expr, diagnostics: $count:expr $(,)?) => {{
        let mut ctx = $crate::TypeContext::new();
        let result = ctx.resolve_statements(&$stmts);
        assert_eq!(
            result.diagnostics.len(),
            $count,
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
        result
    }};
}
pub(crate) use assert_resolution;

/// `ident!("x")` → `Expression::Identifier("x".into())`.
macro_rules! ident {
    ($name:expr) => {
        kali_ast::Expression::Identifier($name.to_string())
    };
}
#[allow(unused_imports)]
pub(crate) use ident;

/// `member!(obj, "prop")` → a `MemberExpression` wrapped in `Expression`.
macro_rules! member {
    ($obj:expr, $prop:expr) => {
        kali_ast::Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: $obj,
            property: $prop.to_string(),
        }))
    };
}
#[allow(unused_imports)]
pub(crate) use member;

/// `call!(callee, [arg, ...])` → a `CallExpression` wrapped in `Expression`.
/// Note: `CallExpression.args` is `Vec<Expression>` (not ExpressionOrSpread).
macro_rules! call {
    ($callee:expr, [ $($arg:expr),* $(,)? ]) => {
        kali_ast::Expression::CallExpression(Box::new(kali_ast::CallExpression {
            callee: $callee,
            args: vec![ $($arg),* ],
        }))
    };
}
#[allow(unused_imports)]
pub(crate) use call;

// Compile guard: invokes each macro once to force type-checking of macro bodies.
// Not a #[test] — must not increment the test count.
#[cfg(test)]
#[allow(dead_code)]
fn _macro_type_check() {
    let _id = ident!("x");
    let _mem = member!(ident!("obj"), "prop");
    let _c = call!(ident!("f"), [ident!("a"), ident!("b")]);
    let stmts: Vec<kali_ast::Statement> = vec![];
    let _res = assert_resolution!(stmts, diagnostics: 0);
}
