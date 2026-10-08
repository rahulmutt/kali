//! Growable-runtime-arrays spec §3.4, A-4 (M2), fail-closed: the syntactic
//! half of the proof that a value stored into a growable array is a number or
//! a string. [`elem_proof`] reduces an expression to the obligations only
//! repr inference can discharge (what a binding, a call or an array holds);
//! `repr_infer::emit_table` discharges them against the solved table. Any
//! shape not listed is [`ElemProof::No`] — over-refusal, never a silent
//! miscompile.

use kali_ast::{AssignmentOperator, Expression, LiteralValue, LogicalOperator};

use super::flow::GrowNode;
use super::unwrap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ElemProof {
    /// Always a number or a string (a literal, arithmetic, a template, …).
    Yes,
    /// Always a string. Task 7 fix round 2: holds only where the growable
    /// array's solved element repr is `String` — a string the element solve
    /// did not see would be stored into a number array and printed as one.
    Str,
    /// May be an object, array, function, boolean, `null` or `undefined`.
    No,
    /// The bare identifier `name`, read in `func`: every value it is ever
    /// given must be proven.
    Binding { func: String, name: String },
    /// A call to the bare-identifier function `callee`, made in `caller`.
    Call { caller: String, callee: String },
    /// One element of `name` (`name[i]`, `name.pop()`), read in `func`. A
    /// string's element is a string.
    Elements { func: String, name: String },
    /// One element of the growable component of the node.
    GrowElements(GrowNode),
    /// `name` is a string array (`words[i]` is a string).
    StringElements { func: String, name: String },
    /// `name` is a string binding (the receiver of `slice`/`at`/`concat`,
    /// which return a string only on a string).
    StringBinding { func: String, name: String },
    /// `name` (in `func`) is a string or an array: its `.length`,
    /// `.join()`, `.indexOf()`, … is then a built-in number or string.
    ArrayOrString { func: String, name: String },
    /// `name` is the unshadowed global (`Math`), read in `func`.
    Global { func: String, name: String },
    /// Every listed obligation.
    All(Vec<ElemProof>),
}

impl ElemProof {
    /// The conjunction of `proofs`, simplified (`Yes` dropped, `No` absorbs).
    pub(crate) fn all(proofs: Vec<ElemProof>) -> ElemProof {
        let mut parts = Vec::new();
        for proof in proofs {
            match proof {
                ElemProof::Yes => {}
                ElemProof::No => return ElemProof::No,
                ElemProof::All(inner) => parts.extend(inner),
                other => parts.push(other),
            }
        }
        match parts.len() {
            0 => ElemProof::Yes,
            1 => parts.pop().expect("one part"),
            _ => ElemProof::All(parts),
        }
    }

    /// The proof with every read of `name` in `func` replaced by `with`
    /// (a `map` callback's parameter, which holds one element). Any other
    /// use of `name` there is `No`.
    pub(crate) fn substitute(&self, func: &str, name: &str, with: &ElemProof) -> ElemProof {
        match self {
            ElemProof::Binding { func: f, name: n } if f == func && n == name => with.clone(),
            ElemProof::All(parts) => ElemProof::all(
                parts
                    .iter()
                    .map(|p| p.substitute(func, name, with))
                    .collect(),
            ),
            ElemProof::Binding { func: f, name: n }
            | ElemProof::Elements { func: f, name: n }
            | ElemProof::StringElements { func: f, name: n }
            | ElemProof::StringBinding { func: f, name: n }
            | ElemProof::ArrayOrString { func: f, name: n }
                if f == func && n == name =>
            {
                ElemProof::No
            }
            other => other.clone(),
        }
    }

    /// True when the proof reads the bare identifier `name` anywhere.
    pub(crate) fn mentions_binding(&self, name: &str) -> bool {
        match self {
            ElemProof::Binding { name: n, .. }
            | ElemProof::Elements { name: n, .. }
            | ElemProof::StringBinding { name: n, .. }
            | ElemProof::StringElements { name: n, .. }
            | ElemProof::ArrayOrString { name: n, .. } => n == name,
            ElemProof::All(parts) => parts.iter().any(|p| p.mentions_binding(name)),
            _ => false,
        }
    }
}

/// Methods whose result is a number whenever the receiver is a string.
const NUMBER_RESULT_METHODS: &[&str] = &["charCodeAt", "codePointAt", "localeCompare", "search"];

/// Methods whose result is a string whenever the receiver is a number or a
/// string.
const SCALAR_RECEIVER_METHODS: &[&str] = &[
    "toUpperCase",
    "toLowerCase",
    "toLocaleUpperCase",
    "toLocaleLowerCase",
    "trim",
    "trimStart",
    "trimEnd",
    "padStart",
    "padEnd",
    "repeat",
    "charAt",
    "substring",
    "substr",
    "replace",
    "replaceAll",
    "normalize",
    "toFixed",
    "toPrecision",
    "toString",
];

/// Methods whose result is a number or a string on any built-in receiver (a
/// string or an array).
const BUILTIN_RECEIVER_METHODS: &[&str] = &[
    "join",
    "indexOf",
    "lastIndexOf",
    "findIndex",
    "findLastIndex",
];

/// Global functions returning a number or a string (when not shadowed:
/// `Call` discharges that).
const SCALAR_GLOBAL_FUNCTIONS: &[&str] = &["String", "Number", "parseInt", "parseFloat"];

/// The root identifier under a chain of method calls (`xs.slice(1).map(f)`
/// → `xs`).
fn chain_root(expr: &Expression) -> Option<&str> {
    match unwrap(expr) {
        Expression::Identifier(name) => Some(name),
        Expression::CallExpression(call) => match unwrap(&call.callee) {
            Expression::MemberExpression(m) if m.computed_index.is_none() => chain_root(&m.object),
            _ => None,
        },
        _ => None,
    }
}

/// The proof obligation that `expr`, evaluated in `func`, is a number or a
/// string.
pub(crate) fn elem_proof(func: &str, expr: &Expression) -> ElemProof {
    let binding = |name: &str| ElemProof::Binding {
        func: func.to_string(),
        name: name.to_string(),
    };
    match unwrap(expr) {
        Expression::Literal(LiteralValue::Number(_)) => ElemProof::Yes,
        Expression::Literal(LiteralValue::String(_)) | Expression::TemplateLiteral(_) => {
            ElemProof::Str
        }
        // `undefined`, `NaN`, … are undeclared: `Binding` refuses them.
        Expression::Identifier(name) => binding(name),
        // `-x`, `+x`, `~x` are numbers and `typeof x` a string, whatever `x`
        // is (a BigInt operand is refused: `-1n` is a BigInt).
        Expression::UnaryExpression(u) => match u.operator.as_str() {
            "-" | "+" | "~" if !matches!(unwrap(&u.argument), Expression::BigIntLiteral(_)) => {
                ElemProof::all(vec![elem_proof(func, &u.argument)])
            }
            "typeof" => ElemProof::Str,
            _ => ElemProof::No,
        },
        // Arithmetic and `+` yield a number or a string (or a BigInt, so the
        // operands are proven too); comparisons, `in` and `instanceof` yield
        // booleans.
        Expression::BinaryExpression(b) => match b.operator.as_str() {
            "+" | "-" | "*" | "/" | "%" | "**" | "|" | "&" | "^" | "<<" | ">>" | ">>>" => {
                ElemProof::all(vec![elem_proof(func, &b.left), elem_proof(func, &b.right)])
            }
            _ => ElemProof::No,
        },
        Expression::UpdateExpression(_) => ElemProof::Yes,
        // `a && b`, `a || b`, `a ?? b` and `c ? a : b` yield one operand.
        Expression::LogicalExpression(l) => match l.operator {
            LogicalOperator::And | LogicalOperator::Or | LogicalOperator::Coalesce => {
                ElemProof::all(vec![elem_proof(func, &l.left), elem_proof(func, &l.right)])
            }
        },
        Expression::ConditionalExpression(c) => ElemProof::all(vec![
            elem_proof(func, &c.consequent),
            elem_proof(func, &c.alternate),
        ]),
        Expression::SequenceExpression(s) => s
            .expressions
            .last()
            .map_or(ElemProof::No, |last| elem_proof(func, last)),
        // `x = v` yields `v`; `x += v` is `x + v` (a string when either is,
        // final review C1); any other compound arithmetic assignment a number
        // (or a BigInt); a logical assignment `x` or `v`.
        Expression::AssignmentExpression(a) => match a.operator {
            AssignmentOperator::Assign => elem_proof(func, &a.right),
            AssignmentOperator::AddAssign
            | AssignmentOperator::NullishAssign
            | AssignmentOperator::AndAssign
            | AssignmentOperator::OrAssign => {
                ElemProof::all(vec![elem_proof(func, &a.left), elem_proof(func, &a.right)])
            }
            _ => ElemProof::Yes,
        },
        Expression::MemberExpression(m) => member_proof(func, m),
        Expression::CallExpression(call) => call_proof(func, call),
        _ => ElemProof::No,
    }
}

fn member_proof(func: &str, m: &kali_ast::MemberExpression) -> ElemProof {
    if m.computed_index.is_some() {
        // `name[i]`: one element (or one character of a string).
        return match unwrap(&m.object) {
            Expression::Identifier(name) => ElemProof::Elements {
                func: func.to_string(),
                name: name.clone(),
            },
            _ => ElemProof::No,
        };
    }
    match (m.dot_name(), unwrap(&m.object)) {
        (Some("length"), Expression::Identifier(name)) => ElemProof::ArrayOrString {
            func: func.to_string(),
            name: name.clone(),
        },
        // `words[i].length` on a string array.
        (Some("length"), Expression::MemberExpression(inner)) if inner.computed_index.is_some() => {
            match unwrap(&inner.object) {
                Expression::Identifier(name) => ElemProof::StringElements {
                    func: func.to_string(),
                    name: name.clone(),
                },
                _ => ElemProof::No,
            }
        }
        (
            Some("length"),
            Expression::Literal(LiteralValue::String(_)) | Expression::TemplateLiteral(_),
        ) => ElemProof::Yes,
        // Any other property is an object field: refused.
        _ => ElemProof::No,
    }
}

fn call_proof(func: &str, call: &kali_ast::CallExpression) -> ElemProof {
    match unwrap(&call.callee) {
        Expression::Identifier(callee) => {
            if crate::resolve::expression::expression_is_array_allocation(
                &Expression::CallExpression(Box::new(call.clone())),
            ) {
                return ElemProof::No;
            }
            ElemProof::Call {
                caller: func.to_string(),
                callee: callee.clone(),
            }
        }
        Expression::MemberExpression(m) if m.computed_index.is_none() => {
            let Some(method) = m.dot_name() else {
                return ElemProof::No;
            };
            let receiver = unwrap(&m.object);
            if matches!(receiver, Expression::Identifier(object) if object == "Math") {
                // `Math.<fn>(…)` is a number, unless the program shadows
                // `Math`.
                return ElemProof::Global {
                    func: func.to_string(),
                    name: "Math".to_string(),
                };
            }
            if NUMBER_RESULT_METHODS.contains(&method) {
                return ElemProof::all(vec![elem_proof(func, receiver)]);
            }
            if SCALAR_RECEIVER_METHODS.contains(&method) {
                return ElemProof::all(vec![elem_proof(func, receiver), ElemProof::Str]);
            }
            if BUILTIN_RECEIVER_METHODS.contains(&method) {
                let receiver_ok = match chain_root(receiver) {
                    Some(root) => ElemProof::ArrayOrString {
                        func: func.to_string(),
                        name: root.to_string(),
                    },
                    None => match receiver {
                        Expression::Literal(LiteralValue::String(_))
                        | Expression::TemplateLiteral(_)
                        | Expression::ArrayExpression(_) => ElemProof::Yes,
                        _ => ElemProof::No,
                    },
                };
                // `join` is a string; the searches are numbers.
                return if method == "join" {
                    ElemProof::all(vec![receiver_ok, ElemProof::Str])
                } else {
                    receiver_ok
                };
            }
            match (method, receiver) {
                ("pop" | "shift", Expression::Identifier(name)) => ElemProof::Elements {
                    func: func.to_string(),
                    name: name.clone(),
                },
                ("slice" | "at" | "concat", Expression::Identifier(name)) => {
                    ElemProof::StringBinding {
                        func: func.to_string(),
                        name: name.clone(),
                    }
                }
                ("slice" | "at" | "concat", Expression::Literal(LiteralValue::String(_))) => {
                    ElemProof::Str
                }
                _ => ElemProof::No,
            }
        }
        _ => ElemProof::No,
    }
}

/// The global functions [`ElemProof::Call`] admits when `callee` names no
/// program function or binding.
pub(crate) fn is_scalar_global_function(callee: &str) -> bool {
    SCALAR_GLOBAL_FUNCTIONS.contains(&callee)
}

#[cfg(test)]
#[path = "elem_proof_tests.rs"]
mod elem_proof_tests;
