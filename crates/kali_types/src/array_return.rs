//! Array-return lane (docs/superpowers/specs/2026-10-02-array-return-design.md
//! §3.1, amendments A2-A4): which functions return a runtime `[len][elem…]`
//! array on every path, which locals are bound to such a call, and which params
//! only ever receive arrays.
//!
//! Pure on purpose. `repr_infer` records the facts during its body walk and
//! applies the [`Solution`] (element-node unions, `ReprTable` writes); this
//! module owns the classification and the fixed point, so both can be tested
//! without the union-find.

use std::collections::{BTreeMap, BTreeSet};

use kali_ast::{ArrayExpression, Expression, ExpressionOrSpread, LiteralValue, Statement};

/// The reserved element-node key for a function's returned array. Not a legal
/// identifier, so it cannot collide with a binding.
pub(crate) const RETURN_ARRAY_KEY: &str = "%return";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ReturnArg {
    /// An array literal of integer-shaped elements (`None`), or a `const`
    /// binding of one (`Some(name)`, amendment A2).
    Literal(Option<String>),
    /// A bare identifier; the fixed point decides whether it is a runtime array.
    Binding(String),
    /// `new Array(n)`, `Array(n)`, `new Array(n).fill(v)`.
    Allocation,
    /// A bare-identifier call.
    Call(String),
    /// Array-shaped but never admitted; carries the refusal reason.
    BadArray(&'static str),
    /// Anything else, including a bare `return;`.
    NonArray,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArgShape {
    Identifier(String),
    Allocation,
    Call(String),
    Other,
}

pub(crate) fn unparen(expr: &Expression) -> &Expression {
    match expr {
        Expression::ParenthesizedExpression(inner) => unparen(&inner.expression),
        // Type-only wrappers are erased at runtime.
        Expression::TypeAssertion(inner) => unparen(&inner.expression),
        Expression::SatisfiesExpression(inner) => unparen(&inner.expression),
        other => other,
    }
}

fn is_allocation(expr: &Expression) -> bool {
    crate::resolve::expression::expression_is_array_allocation(expr)
}

/// True when every element is an expression whose JS value is a number on
/// every evaluation and whose repr the solver can then check for `I64`. A
/// boolean-valued element (`true`, `x > 1`, `!x`, `a && b`) is excluded: kali
/// stores it as `1`/`0`, and a caller printing it would see `1`, not `true`.
pub(crate) fn literal_elements_are_integer_shaped(arr: &ArrayExpression) -> bool {
    arr.elements.iter().all(|element| match element {
        Some(ExpressionOrSpread::Expression(expr)) => element_is_integer_shaped(expr),
        Some(ExpressionOrSpread::Spread(_)) | Some(ExpressionOrSpread::Empty) | None => false,
    })
}

fn element_is_integer_shaped(expr: &Expression) -> bool {
    match unparen(expr) {
        Expression::Literal(LiteralValue::Number(n)) => n.fract() == 0.0,
        Expression::Identifier(_)
        | Expression::CallExpression(_)
        | Expression::MemberExpression(_)
        | Expression::UpdateExpression(_) => true,
        Expression::UnaryExpression(u) => {
            matches!(u.operator.as_str(), "-" | "+" | "~") && element_is_integer_shaped(&u.argument)
        }
        Expression::BinaryExpression(b) => matches!(
            b.operator.as_str(),
            "+" | "-" | "*" | "%" | "|" | "&" | "^" | "<<" | ">>" | ">>>"
        ),
        _ => false,
    }
}

/// Classify one `return` argument. `is_const_literal(name)` is true when `name`
/// is a `const` binding of an array literal in the returning function;
/// `is_let_literal(name)` when it is a `let`/`var` one.
///
/// Exhaustive over `Expression` with no wildcard arm, as `growable.rs`'s
/// scanner is: a new AST variant must be classified here before it compiles.
pub(crate) fn classify_return_arg(
    arg: Option<&Expression>,
    is_const_literal: &dyn Fn(&str) -> bool,
    is_let_literal: &dyn Fn(&str) -> bool,
) -> ReturnArg {
    let Some(arg) = arg else {
        return ReturnArg::NonArray;
    };
    if is_allocation(arg) {
        return ReturnArg::Allocation;
    }
    match unparen(arg) {
        Expression::ArrayExpression(arr) => {
            if literal_elements_are_integer_shaped(arr) {
                ReturnArg::Literal(None)
            } else {
                ReturnArg::BadArray(kali_common::ARRAY_RETURN_ELEMENT)
            }
        }
        Expression::Identifier(name) => {
            if is_const_literal(name) {
                ReturnArg::Literal(Some(name.clone()))
            } else if is_let_literal(name) {
                ReturnArg::BadArray(kali_common::ARRAY_RETURN_LET_LITERAL)
            } else {
                ReturnArg::Binding(name.clone())
            }
        }
        Expression::CallExpression(call) => match unparen(&call.callee) {
            Expression::Identifier(callee) => ReturnArg::Call(callee.clone()),
            _ => ReturnArg::NonArray,
        },
        Expression::Literal(_)
        | Expression::BinaryExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::MemberExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::NewExpression(_)
        | Expression::MetaProperty(_)
        | Expression::TemplateLiteral(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::SequenceExpression(_)
        | Expression::ParenthesizedExpression(_)
        | Expression::YieldExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::OptionalChainExpression(_)
        | Expression::ChainExpression(_)
        | Expression::SpreadElement(_)
        | Expression::RestElement(_)
        | Expression::ImportExpression(_)
        | Expression::DecoratedExpression(_)
        | Expression::JsxElement(_)
        | Expression::JsxFragment(_)
        | Expression::JsxEmptyExpression
        | Expression::TypeAssertion(_)
        | Expression::SatisfiesExpression(_)
        | Expression::ThisExpression
        | Expression::SuperExpression
        | Expression::PrivateIdentifier(_)
        | Expression::BigIntLiteral(_) => ReturnArg::NonArray,
    }
}

/// Argument shape at a call site, for the array-fed param fact (A3).
pub(crate) fn arg_shape(arg: &Expression) -> ArgShape {
    if is_allocation(arg) {
        return ArgShape::Allocation;
    }
    match unparen(arg) {
        Expression::Identifier(name) => ArgShape::Identifier(name.clone()),
        Expression::CallExpression(call) => match unparen(&call.callee) {
            Expression::Identifier(callee) => ArgShape::Call(callee.clone()),
            _ => ArgShape::Other,
        },
        _ => ArgShape::Other,
    }
}

/// A declarator initializer, as the array-return lane records it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InitKind {
    ArrayLiteral,
    Allocation,
    Call(String),
    Other,
}

pub(crate) fn classify_init(init: &Expression) -> InitKind {
    if is_allocation(init) {
        return InitKind::Allocation;
    }
    match unparen(init) {
        Expression::ArrayExpression(_) => InitKind::ArrayLiteral,
        Expression::CallExpression(call) => match unparen(&call.callee) {
            Expression::Identifier(callee) => InitKind::Call(callee.clone()),
            _ => InitKind::Other,
        },
        _ => InitKind::Other,
    }
}

/// The `v` of an allocation spelled `new Array(n).fill(v)` / `Array(n).fill(v)`.
pub(crate) fn fill_value(expr: &Expression) -> Option<&Expression> {
    let call = match unparen(expr) {
        Expression::NewExpression(n) => match unparen(&n.callee) {
            Expression::CallExpression(call) => call,
            _ => return None,
        },
        Expression::CallExpression(call) => call,
        _ => return None,
    };
    match unparen(&call.callee) {
        Expression::MemberExpression(m) if m.dot_name() == Some("fill") => call.args.first(),
        _ => None,
    }
}

/// Conservative: true unless the body provably ends in `return`/`throw` on
/// every path (a trailing `return`/`throw`, or a trailing `if`/`else` whose two
/// arms both provably end so). A false "falls off" taints an otherwise-good
/// function, which refuses; it never admits a bad one.
pub(crate) fn body_falls_off_end(body: &[Statement]) -> bool {
    match body.last() {
        Some(Statement::ReturnStatement(_)) | Some(Statement::ThrowStatement(_)) => false,
        Some(Statement::BlockStatement(block)) => body_falls_off_end(&block.body),
        Some(Statement::IfStatement(stmt)) => match &stmt.alternate {
            Some(alt) => body_falls_off_end(&stmt.consequent.body) || body_falls_off_end(&alt.body),
            None => true,
        },
        _ => true,
    }
}

#[derive(Default, Debug)]
pub(crate) struct ArrayReturnFacts {
    /// Every `function` declaration name, with how many times it is declared.
    pub(crate) declaration_counts: BTreeMap<String, usize>,
    /// Functions of a candidate form: a `function` declaration (any depth, A4).
    pub(crate) candidate_forms: BTreeSet<String>,
    /// Every function's classified returns, candidate form or not.
    pub(crate) returns: BTreeMap<String, Vec<ReturnArg>>,
    pub(crate) falls_off_end: BTreeSet<String>,
    /// `(caller, binding, callee)` for `const`/`let binding = callee()`.
    pub(crate) call_bound: Vec<(String, String, String)>,
    /// Every name that is the callee of a direct in-program call: a call edge
    /// (bare-identifier call), a call-bound declarator, a `return g()`, or a
    /// `g()` argument. A function absent from it and not escaping is never
    /// tainted (ruling R8): a host-exported or tree-shake entry function, or
    /// dead code, hands its result to no kali code, so it keeps its
    /// pre-project lane. Admission is unaffected.
    pub(crate) called: BTreeSet<String>,
}

/// One call-edge argument position, as the array-fed param fact (A3) sees it.
///
/// Contract (built by `repr_infer`'s Phase C0): an edge to a callee with `n`
/// declared params yields exactly one `Feed` per index `0..n`. A position with
/// no argument (a short call, `f(a)` for `function f(x, y)`) and every position
/// at or after the first spread argument (`f(...xs, a)`, whose element count is
/// unknown) carries [`ArgShape::Other`], so such a param is never array-fed. A
/// callee or argument-call name that the caller shadows with a param or local
/// is not a call to the declared function: such an edge is dropped, and such
/// an argument is `Other`.
#[derive(Clone, Debug)]
pub(crate) struct Feed {
    pub(crate) caller: String,
    pub(crate) callee: String,
    pub(crate) index: usize,
    pub(crate) shape: ArgShape,
}

#[derive(Default, Debug)]
pub(crate) struct Solution {
    pub(crate) array_returning: BTreeSet<String>,
    pub(crate) tainted: BTreeMap<String, &'static str>,
    pub(crate) call_bound: BTreeSet<(String, String)>,
    pub(crate) array_fed_params: BTreeSet<(String, String)>,
    /// Functions with returns that are neither directly called
    /// (`ArrayReturnFacts::called`) nor escaping: exempt from every taint,
    /// including `repr_infer`'s emit-time element check (ruling R8).
    pub(crate) taint_exempt: BTreeSet<String>,
}

impl ArrayReturnFacts {
    fn is_candidate(&self, func: &str) -> bool {
        self.candidate_forms.contains(func) && self.declaration_counts.get(func) == Some(&1)
    }
}

/// The optimistic fixed point (spec §3.1, A3). Start with every candidate
/// whose returns are all array-shaped; repeatedly drop any function whose
/// returns are not all admitted under the current sets; recompute the
/// call-bound and array-fed sets from the survivors; stop when nothing
/// changes. Everything only shrinks, so it terminates.
///
/// `is_base_runtime_array(func, name)` answers for the runtime arrays that
/// exist before this lane: an allocation local and an already-array param.
pub(crate) fn solve(
    facts: &ArrayReturnFacts,
    feeds: &[Feed],
    params: &BTreeMap<String, Vec<String>>,
    escaping: &BTreeSet<String>,
    is_base_runtime_array: &dyn Fn(&str, &str) -> bool,
) -> Solution {
    let syntactically_possible = |arg: &ReturnArg| {
        matches!(
            arg,
            ReturnArg::Literal(_)
                | ReturnArg::Allocation
                | ReturnArg::Binding(_)
                | ReturnArg::Call(_)
        )
    };
    let mut returning: BTreeSet<String> = facts
        .returns
        .iter()
        .filter(|(f, args)| {
            facts.is_candidate(f)
                && !facts.falls_off_end.contains(*f)
                && !args.is_empty()
                && args.iter().all(syntactically_possible)
        })
        .map(|(f, _)| f.clone())
        .collect();

    // Params with at least one feed, of a candidate, non-escaping callee.
    let mut feeds_by_param: BTreeMap<(String, String), Vec<&Feed>> = BTreeMap::new();
    for feed in feeds {
        if escaping.contains(&feed.callee) || !facts.is_candidate(&feed.callee) {
            continue;
        }
        if let Some(name) = params.get(&feed.callee).and_then(|ps| ps.get(feed.index)) {
            feeds_by_param
                .entry((feed.callee.clone(), name.clone()))
                .or_default()
                .push(feed);
        }
    }

    loop {
        let call_bound: BTreeSet<(String, String)> = facts
            .call_bound
            .iter()
            .filter(|(_, _, callee)| returning.contains(callee))
            .map(|(caller, binding, _)| (caller.clone(), binding.clone()))
            .collect();

        // Inner optimistic fixed point for array-fed params.
        let mut fed: BTreeSet<(String, String)> = feeds_by_param.keys().cloned().collect();
        loop {
            let is_array = |func: &str, name: &str, fed: &BTreeSet<(String, String)>| {
                is_base_runtime_array(func, name)
                    || call_bound.contains(&(func.to_string(), name.to_string()))
                    || fed.contains(&(func.to_string(), name.to_string()))
            };
            let next: BTreeSet<(String, String)> = fed
                .iter()
                .filter(|key| {
                    feeds_by_param[*key].iter().all(|feed| match &feed.shape {
                        ArgShape::Allocation => true,
                        ArgShape::Call(g) => returning.contains(g),
                        ArgShape::Identifier(n) => is_array(&feed.caller, n, &fed),
                        ArgShape::Other => false,
                    })
                })
                .cloned()
                .collect();
            if next == fed {
                break;
            }
            fed = next;
        }

        let is_array = |func: &str, name: &str| {
            is_base_runtime_array(func, name)
                || call_bound.contains(&(func.to_string(), name.to_string()))
                || fed.contains(&(func.to_string(), name.to_string()))
        };
        let next: BTreeSet<String> = returning
            .iter()
            .filter(|f| {
                facts.returns[*f].iter().all(|arg| match arg {
                    ReturnArg::Literal(_) | ReturnArg::Allocation => true,
                    ReturnArg::Binding(n) => is_array(f, n),
                    ReturnArg::Call(g) => returning.contains(g),
                    ReturnArg::BadArray(_) | ReturnArg::NonArray => false,
                })
            })
            .cloned()
            .collect();
        if next == returning {
            let taint_exempt: BTreeSet<String> = facts
                .returns
                .keys()
                .filter(|f| !facts.called.contains(*f) && !escaping.contains(*f))
                .cloned()
                .collect();
            let mut tainted = BTreeMap::new();
            for (f, args) in &facts.returns {
                // Ruling R8: an uncalled, non-escaping function's result
                // reaches no kali code; it keeps its pre-project lane.
                if taint_exempt.contains(f) {
                    continue;
                }
                if returning.contains(f) {
                    continue;
                }
                // Pre-decided narrowing (plan Task 4 step 10): an arrow or
                // anonymous function expression (`__kali_fn_N`) is never
                // tainted. Callbacks such as `xs.flatMap(x => [x])` return an
                // array the array-method lanes already consume correctly;
                // tainting them refused working programs. Such a function
                // stays on its existing lane (a direct call through its
                // binding already refuses as a first-class function call).
                if !facts.is_candidate(f) && f.starts_with("__kali_fn_") {
                    continue;
                }
                let array_shaped = |arg: &ReturnArg| match arg {
                    ReturnArg::Literal(_) | ReturnArg::Allocation | ReturnArg::BadArray(_) => true,
                    ReturnArg::Binding(n) => is_array(f, n),
                    ReturnArg::Call(g) => returning.contains(g),
                    ReturnArg::NonArray => false,
                };
                if !args.iter().any(array_shaped) {
                    continue;
                }
                let reason = args
                    .iter()
                    .find_map(|arg| match arg {
                        ReturnArg::BadArray(reason) => Some(*reason),
                        _ => None,
                    })
                    .unwrap_or(if facts.is_candidate(f) {
                        kali_common::ARRAY_RETURN_MIXED
                    } else {
                        kali_common::ARRAY_RETURN_FORM
                    });
                tainted.insert(f.clone(), reason);
            }
            return Solution {
                array_returning: returning,
                tainted,
                call_bound,
                array_fed_params: fed,
                taint_exempt,
            };
        }
        returning = next;
    }
}

#[cfg(test)]
#[path = "array_return_tests.rs"]
mod array_return_tests;
